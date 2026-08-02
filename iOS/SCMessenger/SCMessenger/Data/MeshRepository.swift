//
//  MeshRepository.swift
//  SCMessenger
//
//  Repository abstracting access to the Rust core via UniFFI bindings.
//  Single source of truth for mesh service lifecycle, contacts, messages, and settings.
//
//  Mirrors: android/.../data/MeshRepository.kt
//

import Foundation
import Combine
import os
import Security

/// Default settings for mesh service configuration
private enum DefaultSettings {
    static let maxRelayBudget: UInt32 = 1000  // Messages per hour
    static let maxRelayBudgetLimit: UInt32 = 10000  // Maximum allowed
    static let batteryFloor: UInt8 = 20       // Minimum 20% battery
}

/// Repository abstracting access to the Rust core via UniFFI bindings.
///
/// This is the single source of truth for:
/// - Mesh service lifecycle
/// - Contacts management
/// - Message history
/// - Connection ledger
/// - Network settings
/// - Relay enforcement (relay = messaging bidirectional control)
///
/// All UniFFI objects are initialized lazily and managed here to ensure
/// proper lifecycle and resource cleanup.
@MainActor
@Observable
final class MeshRepository {
    enum LiveTransportAction: Equatable {
        case startBle
        case stopBle
        case startInternet
        case stopInternet
    }

    static func liveTransportActions(
        serviceRunning: Bool,
        previousBleEnabled: Bool,
        nextBleEnabled: Bool,
        previousInternetEnabled: Bool,
        nextInternetEnabled: Bool
    ) -> [LiveTransportAction] {
        guard serviceRunning else { return [] }

        var actions: [LiveTransportAction] = []
        if previousBleEnabled != nextBleEnabled {
            actions.append(nextBleEnabled ? .startBle : .stopBle)
        }
        if previousInternetEnabled != nextInternetEnabled {
            actions.append(nextInternetEnabled ? .startInternet : .stopInternet)
        }
        return actions
    }

    enum IdentityHydrationState: Equatable {
        /// A public snapshot may be available, but it has not yet been checked
        /// against the live Rust core and encrypted Keychain backup.
        case pending
        case hydrating
        case ready
        case absent
    }

    private enum IdentityBackupStore {
        static let service = "com.scmessenger.identity"
        static let account = "identity_backup_v1"
    }
    /// A small public-identity snapshot lets the UI render immediately while the
    /// Rust core is restoring its encrypted identity store.  The private key is
    /// never placed in UserDefaults; recovery still requires the Keychain backup.
    private enum IdentityCacheStore {
        static let key = "com.scmessenger.identity.public_snapshot_v1"
    }

    private struct CachedIdentityInfo: Codable {
        let identityId: String?
        let publicKeyHex: String?
        let deviceId: String?
        let seniorityTimestamp: UInt64?
        let initialized: Bool
        let nickname: String?
        let libp2pPeerId: String?

        init(_ identity: IdentityInfo) {
            identityId = identity.identityId
            publicKeyHex = identity.publicKeyHex
            deviceId = identity.deviceId
            seniorityTimestamp = identity.seniorityTimestamp
            initialized = identity.initialized
            nickname = identity.nickname
            libp2pPeerId = identity.libp2pPeerId
        }

        var identityInfo: IdentityInfo {
            IdentityInfo(
                identityId: identityId,
                publicKeyHex: publicKeyHex,
                deviceId: deviceId,
                seniorityTimestamp: seniorityTimestamp,
                initialized: initialized,
                nickname: nickname,
                libp2pPeerId: libp2pPeerId
            )
        }
    }

    private enum InstallMarker {
        static let key = "mesh_install_marker_v1"
    }
    private let logger = Logger(subsystem: "com.scmessenger", category: "Repository")

    /// Conditional logging - only log verbose messages in DEBUG builds
    private func logVerbose(_ message: @autoclosure @escaping () -> String) {
        #if DEBUG
        logger.debug("\(message())")
        #endif
    }

    private func logDiagnostic(_ message: @autoclosure @escaping () -> String) {
        #if DEBUG
        appendDiagnostic(message())
        #endif
    }
    private let storagePath: String
    private let diagnosticsLogFileName = "mesh_diagnostics.log"
    private let diagnosticsIOQueue = DispatchQueue(label: "com.scmessenger.diagnostics.io", qos: .utility)
    private var diagnosticsBuffer: [String] = []
    private let diagnosticsMaxLines = 1000
    private var cachedDiagnostics: String = ""
    private var diagnosticsCacheTime: Date = .distantPast
    private let diagnosticsCacheTTL: TimeInterval = 1.0  // 1 second
    private var heartbeatTimer: Timer?

    // MARK: - Bootstrap Nodes for NAT Traversal

    private static func isEnabledFlag(_ raw: String?) -> Bool {
        guard let value = raw?.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() else {
            return false
        }
        return value == "1" || value == "true" || value == "yes" || value == "on"
    }

    // MARK: - UniFFI Components (lazy initialization)

    private(set) var ironCore: IronCore?
    private(set) var meshService: MeshService?
    private(set) var contactManager: ContactManager?
    private(set) var historyManager: HistoryManager?
    private(set) var ledgerManager: LedgerManager?
    private(set) var settingsManager: MeshSettingsManager?
    private(set) var autoAdjustEngine: AutoAdjustEngine?
    private(set) var swarmBridge: SwarmBridge?

    // Transport Managers
    private var bleCentralManager: BLECentralManager?
    private var blePeripheralManager: BLEPeripheralManager?
    private var multipeerTransport: MultipeerTransport?
    private var mdnsDiscovery: mDNSServiceDiscovery?

    // Smart Transport Router (500ms timeout fallback + health tracking)
    private var smartTransportRouter: SmartTransportRouter?

    // Platform bridge
    private var platformBridge: IosPlatformBridge?

    // Rust → Swift callback delegate (strong reference required; Rust holds weak)
    private var coreDelegateImpl: CoreDelegateImpl?
    private var pendingOutboxRetryTask: Task<Void, Never>?
    private var coverTrafficTask: Task<Void, Never>?
    private var storageMaintenanceTask: Task<Void, Never>?
    /// Coalesces high-frequency contact metadata updates (for example, last-seen
    /// changes) into one durable Keychain recovery checkpoint.
    private var pendingContactBackupTask: Task<Void, Never>?
    private let contactBackupDebounceNanoseconds: UInt64 = 500_000_000
    private var pendingBleBeaconRefreshTask: Task<Void, Never>?
    private var pendingBleBeaconListenerRefreshTask: Task<Void, Never>?
    private var lastRelayBootstrapDialAt: Date = .distantPast
    private var relayBootstrapDialInProgress = false
    private var dialThrottleState: [String: (attempts: Int, nextAllowedAt: Date)] = [:]
    private var relayDialDebounceState: [String: Date] = [:]

    // MARK: - Retry Backoff & Circuit Breaker (LOG-AUDIT-001 fix)
    /// Tracks consecutive failures per peer for exponential backoff
    private var consecutiveDeliveryFailures: [String: Int] = [:]
    /// Tracks last failure time per peer for circuit breaker
    private var lastFailureTime: [String: Date] = [:]
    /// Circuit breaker threshold - pause retries after this many consecutive failures
    private let circuitBreakerThreshold = 10
    /// Circuit breaker duration - pause retries for this long after threshold reached
    private let circuitBreakerDuration: TimeInterval = 300 // 5 minutes
    private let relayDialDebounceInterval: TimeInterval = 10
    static let initialReceiptAwaitSeconds: UInt64 = 60
    private let receiptAwaitSeconds: UInt64 = MeshRepository.initialReceiptAwaitSeconds
    private let pendingOutboxMaxAttempts: UInt32 = 12
    private let pendingOutboxMaxAgeSeconds: UInt64 = 7 * 24 * 60 * 60
    private var historySyncSentPeers: [String: Date] = [:]
    private let historySyncCooldown: TimeInterval = 60
    private var identitySyncSentPeers: Set<String> = []
    private var deliveredReceiptCache: [String: Date] = [:]
    private let deliveredReceiptCacheTtl: TimeInterval = 2 * 60 * 60
    private var pendingReceiptSendTasks: [String: Task<Void, Never>] = [:]
    private let receiptSendMaxAttempts = 6
    private var notificationAppInForeground = true
    private var notificationActiveConversationId: String?

    static func shouldStopAckedWithoutReceiptRetries(
        ackedWithoutReceiptCount: UInt32,
        createdAtEpochSec: UInt64,
        nowEpochSec: UInt64,
        maxAgeSeconds: UInt64
    ) -> Bool {
        ackedWithoutReceiptCount > 0 &&
            nowEpochSec >= createdAtEpochSec &&
            nowEpochSec - createdAtEpochSec >= maxAgeSeconds
    }

    static func receiptRetryDelaySeconds(ackedWithoutReceiptCount: UInt32) -> UInt64 {
        if ackedWithoutReceiptCount <= 3 {
            return initialReceiptAwaitSeconds
        }
        if ackedWithoutReceiptCount <= 8 {
            return 30
        }
        return 120
    }

    // TCP/mDNS transport parity: Track peers discovered on LAN via libp2p mDNS.
    // Key = libp2p PeerId, Value = array of LAN multiaddresses for direct TCP delivery.
    private var mdnsLanPeers: [String: [String]] = [:]

    private enum RelayAvailabilityState: String {
        case stable
        case flapping
        case backoff
        case recovering
    }
    private var relayAvailabilityState: RelayAvailabilityState = .stable
    private var relayAvailabilityUpdatedAt: Date = .distantPast
    private var relayRecentEventTimes: [Date] = []
    private var relayLastDisconnectAt: Date?
    private var relayBackoffUntil: Date = .distantPast
    private let strictBleOnlyValidation = MeshRepository.isEnabledFlag(ProcessInfo.processInfo.environment["SC_BLE_ONLY_VALIDATION"])

    private var pendingOutboxURL: URL {
        URL(fileURLWithPath: storagePath).appendingPathComponent("pending_outbox.json")
    }

    private var diagnosticsLogURL: URL {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        return docs.appendingPathComponent(diagnosticsLogFileName)
    }

    private struct RoutingHints {
        let libp2pPeerId: String?
        let listeners: [String]
        let multipeerPeerId: String?
        let blePeerId: String?
    }

    private struct TransportIdentityResolution {
        let canonicalPeerId: String
        let publicKey: String
        let nickname: String?
    }

    private struct PendingOutboundEnvelope: Codable {
        let queueId: String
        let historyRecordId: String
        let peerId: String
        let routePeerId: String?
        let addresses: [String]
        let envelopeBase64: String
        let createdAtEpochSec: UInt64
        let attemptCount: UInt32
        let nextAttemptAtEpochSec: UInt64
        let strictBleOnlyMode: Bool?
        let recipientIdentityId: String?
        let intendedDeviceId: String?
        let terminalFailureCode: String?
        var ackedWithoutReceiptCount: UInt32? = nil
    }

    struct DeliveryStatePresentation {
        let label: String
        let detail: String
    }

    private struct MessageIdentityHints {
        let identityId: String?
        let publicKey: String?
        let deviceId: String?
        let nickname: String?
        let libp2pPeerId: String?
        let listeners: [String]
        let externalAddresses: [String]
        let connectionHints: [String]
    }

    private struct DecodedMessagePayload {
        let kind: String
        let text: String
        let hints: MessageIdentityHints?
    }

    private struct DeliveryAttemptResult {
        let acked: Bool
        let routePeerId: String?
        let terminalFailureCode: String?
    }

    private enum NotificationNoteKey {
        static let requestPending = "notification_request_pending"
    }

    private struct PeerDiscoveryInfo {
        let canonicalPeerId: String
        let publicKey: String?
        let nickname: String?
        let transport: MeshEventBus.TransportType
        let isFull: Bool
        let isRelay: Bool
        let lastSeen: UInt64
    }

    private struct ReplayDiscoveredIdentity {
        var canonicalPeerId: String
        var publicKey: String?
        var nickname: String?
        var routePeerId: String?
        var transport: MeshEventBus.TransportType
        var isRelay: Bool
    }

    private struct IdentityEmissionSignature: Equatable {
        let canonicalPeerId: String
        let publicKey: String
        let nickname: String?
        let libp2pPeerId: String?
        let blePeerId: String?
    }

    // Device state for auto-adjustment
    private var currentBatteryPct: UInt8 = 100
    private var currentIsCharging: Bool = true
    private var currentMotionState: MotionState = .unknown
    private var lastAppliedPowerSnapshot: String?
    private var identityEmissionCache: [String: (signature: IdentityEmissionSignature, emittedAt: Date)] = [:]
    private let identityReemitInterval: TimeInterval = 15
    private var connectedEmissionCache: [String: Date] = [:]
    private let connectedReemitInterval: TimeInterval = 15

    private func isTerminalIdentityFailure(_ errorCode: String?) -> Bool {
        switch errorCode?.trimmingCharacters(in: .whitespacesAndNewlines) {
        case "identity_device_mismatch", "identity_abandoned":
            return true
        default:
            return false
        }
    }

    private func terminalIdentityFailureMessage(_ errorCode: String?) -> String {
        switch errorCode?.trimmingCharacters(in: .whitespacesAndNewlines) {
        case "identity_device_mismatch":
            return "This contact's identity has been recycled onto another device. Refresh their contact details before retrying."
        case "identity_abandoned":
            return "This contact abandoned the identity you tried to reach. Re-verify the contact before sending again."
        default:
            return "This message was rejected because the recipient identity is no longer valid."
        }
    }

    // Reinstall detection: set true when Keychain has identity but disk has no contacts/history.
    // Triggers aggressive post-start identity beacon to recover contacts from mesh peers.
    private var isReinstallWithMissingData = false

    // P0: Dedup cache — suppress redundant peer-identified callbacks for the same peer
    // within a 30-second window. The Rust core fires identify per-substream, producing
    // 34K+ duplicate events in a typical session.
    private var peerIdentifiedDedupCache: [String: (signature: String, observedAt: Date)] = [:]
    private let peerIdentifiedDedupInterval: TimeInterval = 30

    // Transmission throttle to prevent rapid-click duplicates
    private var transmissionThrottleCache: [String: Date] = [:]
    private let transmissionThrottleInterval: TimeInterval = 1.0

    // P0: Throttle BLE identity beacon updates to 5s to reduce radio churn and bridge flood.
    private var lastBleBeaconUpdate: Date?
    private let bleBeaconUpdateInterval: TimeInterval = 5.0
    private var lastBleBeaconPayload: Data?
    private var lastBleBeaconPayloadPublishedAt: Date?

    // P1: Dedup cache — suppress duplicate disconnect callbacks for the same peer
    // within a 1-second window. The Rust core fires one disconnect per-substream,
    // producing 254+ events in under 1 second.
    private var peerDisconnectDedupCache: [String: Date] = [:]
    private let peerDisconnectDedupInterval: TimeInterval = 1.0

    // P4: Dedup cache — suppress redundant dial_throttled diagnostic log lines
    // for the same address within a 5-minute window (9.6K+ events in a session).
    private var dialThrottleLogCache: [String: Date] = [:]
    private let dialThrottleLogInterval: TimeInterval = 300

    // MARK: - Published State

    var serviceState: ServiceState = .stopped
    var serviceStats: ServiceStats?
    var networkStatus = NetworkStatus()
    /// Shared identity state for all UI surfaces.  Android exposes the same
    /// state through a StateFlow so onboarding, Settings, and the tab gate
    /// cannot disagree while the core is warming up.
    private(set) var identityInfo: IdentityInfo?
    private(set) var identityHydrationState: IdentityHydrationState = .pending
    private(set) var isCreatingIdentity: Bool = false
    private var discoveredPeerMap: [String: PeerDiscoveryInfo] = [:]

    /// True only after the running core (or a successful Keychain restore) has
    /// confirmed the identity.  UI gates must use this rather than a cached
    /// public snapshot, which can survive independently from secure key data.
    var hasVerifiedIdentity: Bool {
        identityHydrationState == .ready && identityInfo?.initialized == true
    }

    struct NetworkStatus {
        var wifi: Bool = false
        var cellular: Bool = false
        var available: Bool { wifi || cellular }
    }

    // BLE Privacy Settings
    var blePrivacyEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "ble_rotation_enabled") as? Bool ?? true }
        set {
            UserDefaults.standard.set(newValue, forKey: "ble_rotation_enabled")
            blePeripheralManager?.setRotationEnabled(newValue)
        }
    }

    var blePrivacyInterval: TimeInterval {
        get { UserDefaults.standard.object(forKey: "ble_rotation_interval") as? Double ?? 900 }
        set {
            UserDefaults.standard.set(newValue, forKey: "ble_rotation_interval")
            blePeripheralManager?.setRotationInterval(newValue)
        }
    }

    var isAutoAdjustEnabled: Bool {
        UserDefaults.standard.object(forKey: "auto_adjust_enabled") as? Bool ?? true
    }

    // MARK: - Event Streams

    /// Stream of ALL message updates (sent and received)
    let messageUpdates = PassthroughSubject<MessageRecord, Never>()

    /// Legacy alias for backward compatibility (filtered to received only if needed, but for now direct alias)
    var incomingMessages: PassthroughSubject<MessageRecord, Never> { messageUpdates }

    let peerEvents = PassthroughSubject<PeerEvent, Never>()
    let statusEvents = PassthroughSubject<StatusEvent, Never>()

    enum PeerEvent {
        case discovered(peerId: String)
        case connected(peerId: String)
        case disconnected(peerId: String)
    }

    enum StatusEvent {
        case serviceStateChanged(ServiceState)
        case statsUpdated(ServiceStats)
    }

    // MARK: - Initialization

    init() {
        // Use Application Support for internal app state (not user-facing docs).
        let appSupportPath = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let meshPath = appSupportPath.appendingPathComponent("mesh", isDirectory: true)
        self.storagePath = meshPath.path
        self.identityInfo = Self.readCachedIdentityInfo()

        // Android previously shipped the bridge contacts store under
        // `contacts/`; the current cross-platform contract is `contacts.db/`.
        // Promote that legacy store before any ContactManager opens it so a
        // normal app update cannot make an existing address book appear empty.
        migrateLegacyContactStoreIfNeeded()

        // Load existing logs into memory buffer if they exist
        if let existingLogs = try? String(contentsOf: diagnosticsLogURL, encoding: .utf8) {
            let lines = existingLogs.components(separatedBy: .newlines).filter { !$0.isEmpty }
            self.diagnosticsBuffer = Array(lines.suffix(diagnosticsMaxLines))
        }

        logVerbose("MeshRepository initialized with storage: \(self.storagePath)")
        if strictBleOnlyValidation {
            logger.warning("Strict BLE-only validation mode is enabled (SC_BLE_ONLY_VALIDATION)")
        }

        // Do NOT exclude the entire mesh directory from backup. In-place app
        // updates retain this container. If a container is ever lost, the
        // current encrypted Keychain recovery snapshot restores contacts.
        // The raw identity key directory (identity/) is excluded individually
        // after start() because private keys are already Keychain-protected.

        reconcileInstallScopedIdentityState()

        if identityInfo?.initialized == true {
            logVerbose("Loaded cached public identity snapshot before core hydration")
        }

        logDiagnostic("repo_init storage=\(self.storagePath)")
        startHeartbeat()
    }

    /// Migrate the only known legacy UniFFI contact-store layout:
    /// `<mesh>/contacts/` -> `<mesh>/contacts.db/`.
    ///
    /// The migration is deliberately filesystem-first and runs before any sled
    /// handle is opened. It never replaces a current populated store, keeps the
    /// legacy directory intact as rollback data, and promotes only a staged copy
    /// that can be opened and read as contacts.
    private func migrateLegacyContactStoreIfNeeded() {
        let fileManager = FileManager.default
        let storageURL = URL(fileURLWithPath: storagePath, isDirectory: true)
        let legacyStoreURL = storageURL.appendingPathComponent("contacts", isDirectory: true)
        let currentStoreURL = storageURL.appendingPathComponent("contacts.db", isDirectory: true)

        guard fileManager.fileExists(atPath: legacyStoreURL.path) else { return }

        if fileManager.fileExists(atPath: currentStoreURL.path),
           let currentCount = contactCount(inStorageDirectory: storageURL),
           currentCount > 0 {
            logger.info("Contact-store migration skipped: current contacts.db already has \(currentCount) contact(s)")
            return
        }

        let stagingRoot = storageURL.appendingPathComponent(
            ".contacts-migration-\(UUID().uuidString)",
            isDirectory: true
        )
        let stagedStoreURL = stagingRoot.appendingPathComponent("contacts.db", isDirectory: true)

        do {
            try fileManager.createDirectory(at: stagingRoot, withIntermediateDirectories: true)
            try fileManager.copyItem(at: legacyStoreURL, to: stagedStoreURL)

            guard let stagedCount = contactCount(inStorageDirectory: stagingRoot), stagedCount > 0 else {
                logger.warning("Contact-store migration skipped: legacy contacts store could not be validated")
                try? fileManager.removeItem(at: stagingRoot)
                return
            }

            var rollbackStoreURL: URL?
            if fileManager.fileExists(atPath: currentStoreURL.path) {
                let rollback = storageURL.appendingPathComponent(
                    ".contacts.db-pre-migration-\(UUID().uuidString)",
                    isDirectory: true
                )
                try fileManager.moveItem(at: currentStoreURL, to: rollback)
                rollbackStoreURL = rollback
            }

            do {
                // Both paths are siblings in the app container, so this is an
                // atomic rename on iOS's local filesystem.
                try fileManager.moveItem(at: stagedStoreURL, to: currentStoreURL)
                try? fileManager.removeItem(at: stagingRoot)
                logger.info("Migrated \(stagedCount) contact(s) from legacy contacts store")
            } catch {
                // Restore the untouched prior current store if promotion failed.
                if let rollbackStoreURL,
                   !fileManager.fileExists(atPath: currentStoreURL.path) {
                    try? fileManager.moveItem(at: rollbackStoreURL, to: currentStoreURL)
                }
                throw error
            }
        } catch {
            try? fileManager.removeItem(at: stagingRoot)
            logger.warning("Contact-store migration failed safely: \(error.localizedDescription, privacy: .public)")
        }
    }

    /// Open a known `contacts.db` directory just long enough to verify that it
    /// is readable and contains contacts. Returning nil means the store is not
    /// safe to promote over another store.
    private func contactCount(inStorageDirectory storageDirectory: URL) -> Int? {
        let storeURL = storageDirectory.appendingPathComponent("contacts.db", isDirectory: true)
        guard FileManager.default.fileExists(atPath: storeURL.path) else { return nil }
        do {
            let manager = try ContactManager(storagePath: storageDirectory.path)
            return try manager.list().count
        } catch {
            logger.warning("Could not validate contact store at \(storeURL.path, privacy: .private): \(error.localizedDescription, privacy: .public)")
            return nil
        }
    }

    private func startHeartbeat() {
        heartbeatTimer?.invalidate()
        heartbeatTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            Task { @MainActor [weak self] in
                self?.logDiagnostic("pulse uptime=\(Int(Date().timeIntervalSinceReferenceDate) % 100000)")
            }
        }
    }

    /// Initialize all managers
    @MainActor
    func initialize() throws {
        logVerbose("Initializing managers")
        logDiagnostic("repo_managers_init_start")

        do {
            // Initialize data managers
            settingsManager = MeshSettingsManager(storagePath: storagePath)

            // Ensure settings exist (load or create defaults)
            if (try? settingsManager?.load()) == nil {
                logger.info("No settings found, applying defaults")
                if let defaults = settingsManager?.defaultSettings() {
                    try? settingsManager?.save(settings: defaults)
                }
            }
            historyManager = try HistoryManager(storagePath: storagePath)
            // WS12.41: Removed fixed 10k limit. Retention is now disk-percent aware.
            // _ = try? historyManager?.enforceRetention(maxMessages: 10000)
            contactManager = try ContactManager(storagePath: storagePath)
            ledgerManager = LedgerManager(storagePath: storagePath)
            autoAdjustEngine = AutoAdjustEngine()

            // Transport managers are created only when their persisted setting
            // is enabled. This prevents CoreBluetooth restoration callbacks
            // from resurrecting a transport that the user turned off.
            smartTransportRouter = SmartTransportRouter()

            // Pre-load data where applicable
            try? ledgerManager?.load()

            logDiagnostic("repo_managers_init_success")
            logVerbose("[OK] All managers initialized successfully")

            // One-time migration: clear stale routing hints inherited from
            // pre-fix builds that accumulated duplicate BLE MACs and stale
            // libp2p_peer_id entries causing endless retry loops.
            migrateStaleRoutingHints()
        } catch {
            logger.error("Failed to initialize managers: \(error.localizedDescription)")
            throw error
        }
    }

    private func migrateStaleRoutingHints() {
        let key = "v1_routing_hint_cleanup"
        guard !UserDefaults.standard.bool(forKey: key) else { return }

        do {
            let contacts = try contactManager?.list() ?? []
            var cleaned = 0
            for contact in contacts {
                guard let notes = contact.notes, !notes.isEmpty else { continue }
                guard notes.contains("libp2p_peer_id:") || notes.contains("ble_peer_id:") else { continue }
                // Strip stale routing entries — fresh discovery will repopulate them.
                let stripped = notes
                    .split(whereSeparator: { $0 == ";" || $0 == "\n" })
                    .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
                    .filter { segment in
                        !segment.hasPrefix("libp2p_peer_id:") &&
                            !segment.hasPrefix("ble_peer_id:")
                    }
                    .joined(separator: ";")
                let updatedNotes: String? = stripped.isEmpty ? nil : stripped
                if updatedNotes != notes {
                    var updated = contact
                    updated.notes = updatedNotes
                    try? contactManager?.add(contact: updated)
                    cleaned += 1
                }
            }
            UserDefaults.standard.set(true, forKey: key)
            if cleaned > 0 {
                checkpointContactPersistence(reason: "routing_hint_migration")
                logVerbose("Routing hint migration: cleaned \(cleaned) contact(s) with stale routing entries")
            }
        } catch {
            logger.warning("Routing hint migration failed (non-fatal): \(error.localizedDescription)")
        }
    }

    /// Public start method called from App entry point
    func start() {
        logDiagnostic("repo_start_requested")
        logVerbose("Application requested repository start")
        identityHydrationState = .hydrating
        do {
            try ensureServiceInitialized()
            try ensureLocalIdentityFederation()
            validatePublishedIdentityAfterHydration()

        } catch {
            // A startup error is not proof that the snapshot is valid.  Keep the
            // state pending so UI surfaces remain in the identity-creation path
            // until a later successful service transition can verify it.
            identityHydrationState = .pending
            logger.error("Failed to start repository: \(error.localizedDescription)")
        }
    }

    /// Revalidate identity whenever a UI surface appears or the service returns
    /// to running.  This mirrors Android's Settings/MainViewModel force-refresh
    /// and prevents a stale public snapshot from suppressing identity creation.
    func refreshIdentityHydration() {
        identityHydrationState = .hydrating
        do {
            try ensureServiceInitialized()
            try ensureLocalIdentityFederation()
            validatePublishedIdentityAfterHydration()
        } catch {
            identityHydrationState = .pending
            logger.error("Failed to hydrate identity state: \(error.localizedDescription)")
        }
    }

    /// Ensure service is initialized (lazy start if needed)
    /// This enables identity operations before full mesh service is running
    private func ensureServiceInitialized() throws {
        // Initialize when service is not running (mirrors Android: state != RUNNING)
        // This properly handles all non-running states including .stopped, .starting, .stopping, .paused
        if meshService == nil || serviceState != .running {
            logDiagnostic("lazy_service_start_init")
            logVerbose("Lazy starting MeshService for Identity access")

            // Clean up existing service if stopped but not nil
            if meshService != nil {
                meshService?.stop()
                meshService = nil
                ironCore = nil
            }

            // Initialize managers if not already done
            if settingsManager == nil {
                try initialize()
            }

            // Create minimal config for lazy start
            // Use saved settings or defaults from settings manager
            guard let settingsManager = settingsManager else {
                throw MeshError.notInitialized("SettingsManager not initialized for lazy start")
            }

            // Startup must honor persisted transport choices. Defaults are
            // appropriate only when no readable settings file exists yet.
            let settings = (try? settingsManager.load()) ?? settingsManager.defaultSettings()

            let config = MeshServiceConfig(
                discoveryIntervalMs: 30000,
                batteryFloorPct: settings.batteryFloor
            )

            try startMeshService(config: config)
            logVerbose("[OK] MeshService started lazily")
        }

        // Verify ironCore is available after initialization
        if ironCore == nil {
            logger.error("[WARNING] IronCore is nil despite service running - attempting refresh")
            ironCore = meshService?.getCore()
            if ironCore == nil {
                throw MeshError.notInitialized("Failed to obtain IronCore from running service")
            }
        }
    }

    // MARK: - Service Lifecycle

    /// Start the mesh service with configuration
    func startMeshService(config: MeshServiceConfig) throws {
        logVerbose("Starting mesh service")
        logDiagnostic("service_start requested")

        guard serviceState == .stopped else {
            logger.warning("Service already started or starting")
            return
        }

        serviceState = .starting
        statusEvents.send(.serviceStateChanged(.starting))

        do {
            // Create mesh service with persistent storage and structured tracing
            let logsDir = storagePath + "/logs"
            meshService = MeshService.withStorageAndLogs(config: config, storagePath: storagePath, logDirectory: logsDir)

            // Start service first — IronCore is created during start()
            try meshService?.start()

            // Configure platform bridge (Swift -> Rust callbacks)
            platformBridge = IosPlatformBridge()
            platformBridge?.configure(repository: self)
            meshService?.setPlatformBridge(bridge: platformBridge)

            // Now obtain IronCore (only available after start())
            ironCore = meshService?.getCore()
            if ironCore == nil {
                throw MeshError.notInitialized("Failed to obtain IronCore from MeshService")
            }
            try ensureLocalIdentityFederation()

            // Wire CoreDelegate: Rust → Swift callbacks
            let coreDelegate = CoreDelegateImpl(meshRepository: self)
            self.coreDelegateImpl = coreDelegate  // store strong reference
            ironCore?.setDelegate(delegate: coreDelegate)
            logger.info("CoreDelegate registered for Rust->Swift callbacks")

            // Initialize internet transport if enabled.
            // Core auto-selects headless mode when identity is absent and upgrades when identity appears.
            guard let settingsManager else {
                throw MeshError.notInitialized("SettingsManager unavailable during service start")
            }
            let runtimeSettings = (try? settingsManager.load()) ?? settingsManager.defaultSettings()
            var swarmTransportStarted = false
            if runtimeSettings.internetEnabled {
                do {
                    try startInternetTransport(reason: "startup")
                    swarmTransportStarted = true
                } catch {
                    // Identity/Core startup remains valid without internet transport.
                    // Keep it alive for local data, BLE, Multipeer, and a later
                    // Swarm retry rather than routing the UI back to onboarding.
                    swarmBridge = nil
                    appendDiagnostic("swarm_start degraded error=\(error.localizedDescription)")
                    logger.error("Swarm transport unavailable; continuing with identity services: \(error.localizedDescription)")
                }
            }

            serviceState = .running
            statusEvents.send(.serviceStateChanged(.running))

            if swarmTransportStarted {
                Task { [weak self] in
                    await self?.ensureBootstrapRelayConnected()
                }
            }

            // Protect raw identity sled store from backup — keys are already in Keychain.
            // In-place updates retain contacts.db; the current encrypted Keychain
            // snapshot is the deterministic fallback if an app container is lost.
            excludeIdentitySubdirFromBackup()

            // Apple's Multipeer transport remains service-bound and automatic.
            multipeerTransport?.disconnect()
            multipeerTransport = MultipeerTransport(meshRepository: self)
            multipeerTransport?.startAdvertising()
            multipeerTransport?.startBrowsing()
            if runtimeSettings.bleEnabled {
                startBleTransport()
            } else {
                stopBleTransport()
            }
            Task { @MainActor [weak self] in
                await self?.applyPowerAdjustments(reason: "service_started")
            }
            startPendingOutboxRetryLoop()
            startCoverTrafficLoopIfEnabled()
            Task { await flushPendingOutbox(reason: "service_started") }

            // On reinstall with existing Keychain identity: broadcast aggressively so
            // known peers can re-send their identity info and we can rebuild contacts.
            if isReinstallWithMissingData {
                isReinstallWithMissingData = false
                appendDiagnostic("reinstall_recovery_beacon_scheduled")
                Task {
                    try? await Task.sleep(nanoseconds: 4_000_000_000) // wait for swarm connect
                    broadcastIdentityBeacon()
                    appendDiagnostic("reinstall_recovery_beacon_sent")
                }
            }

            // WS12.41: Start storage maintenance loop
            startStorageMaintenance()

            let info = getIdentityInfo()
            logger.info("SC_IDENTITY_OWN p2p_id=\(info?.libp2pPeerId ?? "unknown") pk=\(info?.publicKeyHex ?? "unknown")")
            if !runtimeSettings.internetEnabled || swarmTransportStarted {
                logVerbose("[OK] Mesh service started successfully")
                logDiagnostic("service_start success")
            } else {
                logVerbose("Mesh service started with Swarm transport unavailable")
                logDiagnostic("service_start degraded swarm_unavailable")
            }
        } catch {
            // MeshService itself is already running by the time Swarm startup
            // is attempted. Tear it down so a later retry creates a fresh
            // service instead of retaining a bridge with no SwarmHandle. Keep
            // the verified public identity published first: a transport error
            // must never present as identity loss in the UI.
            if let liveIdentity = ironCore?.getIdentityInfo(), liveIdentity.initialized {
                publishIdentityInfo(liveIdentity)
                persistIdentityBackupToKeychain(ironCore: ironCore)
            }
            meshService?.stop()
            meshService = nil
            swarmBridge = nil
            ironCore = nil
            serviceState = .stopped
            statusEvents.send(.serviceStateChanged(.stopped))
            logger.error("Failed to start mesh service: \(error.localizedDescription)")
            appendDiagnostic("service_start failure error=\(error.localizedDescription)")
            throw error
        }
    }

    private func startBleTransport() {
        if bleCentralManager == nil {
            bleCentralManager = BLECentralManager(meshRepository: self)
        }
        if blePeripheralManager == nil {
            blePeripheralManager = BLEPeripheralManager(meshRepository: self)
        }

        blePeripheralManager?.setRotationEnabled(blePrivacyEnabled)
        blePeripheralManager?.setRotationInterval(blePrivacyInterval)
        broadcastIdentityBeacon()
        blePeripheralManager?.startAdvertising()
        bleCentralManager?.startScanning()
        appendDiagnostic("transport_ble enabled")
    }

    private func stopBleTransport() {
        bleCentralManager?.stopScanning()
        blePeripheralManager?.stopAdvertising()
        bleCentralManager = nil
        blePeripheralManager = nil
        appendDiagnostic("transport_ble disabled")
    }

    private func startMdnsDiscovery() {
        guard mdnsDiscovery == nil else { return }

        let discovery = mDNSServiceDiscovery(meshRepository: self)
        discovery.onLanPeerResolved = { [weak self] peerId, host, port in
            Task { @MainActor [weak self] in
                guard let self, let bridge = self.swarmBridge else { return }
                let ipProto = host.contains(":") ? "ip6" : "ip4"
                let multiaddr = "/\(ipProto)/\(host)/tcp/\(port)"
                self.logger.info("mDNS: Dialing resolved LAN peer \(peerId) at \(multiaddr)")
                do {
                    try await bridge.dial(multiaddr: multiaddr)
                } catch {
                    self.logger.error("mDNS: Failed to dial \(multiaddr): \(error.localizedDescription)")
                }
            }
        }
        discovery.startBrowsing()
        discovery.startAdvertising(port: 9001)
        mdnsDiscovery = discovery
    }

    private func stopMdnsDiscovery() {
        mdnsDiscovery?.cleanup()
        mdnsDiscovery = nil
        mdnsLanPeers.removeAll()
    }

    private func startInternetTransport(reason: String) throws {
        if swarmBridge != nil {
            startMdnsDiscovery()
            return
        }
        guard let service = meshService else {
            throw MeshError.notInitialized("MeshService is unavailable for Swarm startup")
        }

        let bootstrapAddrs = ledgerBootstrapAddresses()
        appendDiagnostic("swarm_start_\(reason) bootstrap_count=\(bootstrapAddrs.count)")
        do {
            try service.startSwarm(
                listenAddr: "/ip4/0.0.0.0/tcp/9001",
                bootstrapAddrs: bootstrapAddrs
            )
            swarmBridge = service.getSwarmBridge()
            startMdnsDiscovery()
            broadcastIdentityBeacon()
            logger.info("[OK] Internet transport (Swarm) started and bridge wired")
        } catch {
            stopMdnsDiscovery()
            swarmBridge = nil
            throw error
        }
    }

    private func stopInternetTransport() throws {
        // SwarmBridge.shutdown() stops the live handle but the current
        // MeshService still retains that handle. A later startSwarm() would
        // therefore treat the dead handle as already running. Recreate the
        // service so Rust owns a fresh bridge while the newly persisted
        // internet=false setting keeps Swarm and mDNS disabled.
        if serviceState == .running {
            stopMeshService()
            try ensureServiceInitialized()
        } else {
            stopMdnsDiscovery()
            swarmBridge = nil
        }
        appendDiagnostic("transport_internet disabled")
        logger.info("[OK] Internet transport (Swarm) stopped with fresh service state")
    }

    /// Proactively dial the best ledger-sourced relay after mesh startup so an
    /// outbound NAT mapping exists before inbound circuit-relay traffic is expected.
    /// A fresh install has no candidate until invite/QR or LAN discovery seeds
    /// the ledger. Failure is non-fatal.
    private func ensureBootstrapRelayConnected() async {
        guard let swarmBridge else {
            logger.warning("Skipping bootstrap relay connect: SwarmBridge not wired yet")
            return
        }
        guard let bootstrapRelay = ledgerBootstrapAddresses(maxCount: 1).first else {
            logger.info("No known relay in ledger; skipping proactive bootstrap dial")
            return
        }
        do {
            try await swarmBridge.dial(multiaddr: bootstrapRelay)
            logger.info("Connected to ledger bootstrap relay")
        } catch {
            logger.warning("Bootstrap relay unavailable at startup (non-fatal): \(error.localizedDescription)")
        }
    }

    /// Stop the mesh service
    func stopMeshService() {
        logVerbose("Stopping mesh service")
        logDiagnostic("service_stop requested")

        guard serviceState == .running else {
            logger.warning("Service not running")
            return
        }

        checkpointPersistentState(reason: "service_stop")

        serviceState = .stopping
        statusEvents.send(.serviceStateChanged(.stopping))

        meshService?.stop()
        swarmBridge = nil
        pendingOutboxRetryTask?.cancel()
        pendingOutboxRetryTask = nil
        coverTrafficTask?.cancel()
        coverTrafficTask = nil
        pendingBleBeaconRefreshTask?.cancel()
        pendingBleBeaconRefreshTask = nil
        pendingBleBeaconListenerRefreshTask?.cancel()
        pendingBleBeaconListenerRefreshTask = nil
        pendingReceiptSendTasks.values.forEach { $0.cancel() }
        pendingReceiptSendTasks.removeAll()
        identitySyncSentPeers.removeAll()
        historySyncSentPeers.removeAll()
        identityEmissionCache.removeAll()
        connectedEmissionCache.removeAll()
        mdnsLanPeers.removeAll()

        serviceState = .stopped
        statusEvents.send(.serviceStateChanged(.stopped))

        stopMdnsDiscovery()
        stopBleTransport()
        multipeerTransport?.disconnect()
        multipeerTransport = nil

        logVerbose("[OK] Mesh service stopped")
        logDiagnostic("service_stop success")
    }

    /// Pause the mesh service (background mode)
    func pauseMeshService() {
        logVerbose("Pausing mesh service")
        guard serviceState == .running else {
            logger.warning("Service not running (current state: \(self.serviceState))")
            return
        }
        meshService?.pause()
        // Note: pause() is an internal operation that reduces activity
        // The external serviceState remains .running (no .paused state exists)
        logVerbose("[OK] Mesh service paused")
    }

    /// Resume the mesh service (foreground mode)
    func resumeMeshService() {
        logVerbose("Resuming mesh service")
        guard serviceState == .running else {
            logger.warning("Cannot resume - service not in running state (current: \(self.serviceState))")
            return
        }
        meshService?.resume()
        logVerbose("[OK] Mesh service resumed")
    }

    /// Get current service state
    func getServiceState() -> ServiceState {
        return serviceState
    }

    /// Initialize internet transport (Swarm) if enabled.
    func initializeAndStartSwarm() {
        try? ensureLocalIdentityFederation()

        let settings = try? settingsManager?.load()
        if settings?.internetEnabled == true {
            do {
                try startInternetTransport(reason: "manual")
            } catch {
                logger.error("Failed to start swarm: \(error.localizedDescription)")
            }
        }
    }

    // MARK: - Identity Management

    /// Get identity information
    func getIdentityInfo() -> IdentityInfo? {
        if let liveIdentity = ironCore?.getIdentityInfo(), liveIdentity.initialized {
            return liveIdentity
        }
        return identityInfo
    }

    /// Check whether a verified identity is available.  A Keychain backup and
    /// public snapshot are recovery inputs, not proof of a working identity:
    /// only the live core or a completed restore can unlock identity-only UI.
    /// This remains lightweight and never starts the service recursively.
    func isIdentityInitialized() -> Bool {
        if hasVerifiedIdentity {
            return true
        }
        if identityHydrationState == .absent {
            return false
        }
        if ironCore?.getIdentityInfo().initialized == true {
            return true
        }
        // A backup is only a recovery candidate.  It must be imported and
        // verified before it is allowed to unlock the identity-only UI.
        return false
    }

    /// Records the onboarding consent in the core.  This is intentionally
    /// idempotent so it can be called from both onboarding and recovery flows.
    func grantIdentityConsent() {
        do {
            try ensureServiceInitialized()
            ironCore?.grantConsent()
        } catch {
            logger.error("Failed to record identity consent: \(error.localizedDescription)")
        }
    }

    /// Create a new identity (first-time setup).  This is deliberately safe to
    /// call from more than one surface: an existing identity is published rather
    /// than replaced, matching Android's serialized creation coordinator.
    @discardableResult
    func createIdentity() throws -> IdentityInfo {
        logVerbose("Creating identity")

        do {
            try ensureServiceInitialized()

            guard let ironCore = ironCore else {
                logger.error("IronCore is nil after ensureServiceInitialized! Cannot create identity.")
                throw MeshError.notInitialized("Mesh service initialization failed")
            }

            let existing = ironCore.getIdentityInfo()
            if existing.initialized {
                publishIdentityInfo(existing)
                return existing
            }

            guard !isCreatingIdentity else {
                throw MeshError.invalidInput("Identity creation is already in progress")
            }
            isCreatingIdentity = true
            defer { isCreatingIdentity = false }

            // P0: Grant consent before identity initialization.
            // The Rust core requires consent_granted=true for initialize_identity().
            // Android already does this; adding for iOS parity.
            ironCore.grantConsent()
            logVerbose("Calling ironCore.initializeIdentity()...")
            try ironCore.initializeIdentity()

            // Publish and persist immediately after key generation.  The UI must
            // never wait for nickname assignment, BLE advertising, or swarm startup
            // to learn that the identity exists.
            let created = ironCore.getIdentityInfo()
            guard created.initialized else {
                throw MeshError.notInitialized("Identity generation completed without an initialized identity")
            }
            publishIdentityInfo(created)
            persistIdentityBackupToKeychain(ironCore: ironCore)

            logVerbose("[OK] Identity created successfully")
            initializeAndStartSwarm()
            broadcastIdentityBeacon()
            return created
        } catch {
            logger.error("Failed to create identity: \(error.localizedDescription)")
            throw error
        }
    }

    /// Complete the identity setup shared by onboarding and Settings.  Keeping
    /// creation, nickname persistence, verification, and onboarding completion
    /// together prevents the split-state race the Android coordinator avoids.
    @discardableResult
    func createIdentity(nickname: String) async throws -> IdentityInfo {
        let trimmedNickname = nickname.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedNickname.isEmpty else {
            throw MeshError.invalidInput("Nickname is required")
        }

        try ensureServiceInitialized()
        guard let ironCore else {
            throw MeshError.notInitialized("Mesh service initialization failed")
        }

        guard !isCreatingIdentity else {
            throw MeshError.invalidInput("Identity creation is already in progress")
        }
        isCreatingIdentity = true
        defer { isCreatingIdentity = false }

        let current = ironCore.getIdentityInfo()
        if current.initialized {
            publishIdentityInfo(current)
        } else {
            // The generated UniFFI surface is isolated to this target's main
            // actor. Yield once so SwiftUI can render the in-progress state,
            // then perform the synchronous FFI transaction on its required actor.
            await Task.yield()
            ironCore.grantConsent()
            try ironCore.initializeIdentity()
            let created = ironCore.getIdentityInfo()

            guard created.initialized else {
                throw MeshError.notInitialized("Identity generation completed without an initialized identity")
            }
            publishIdentityInfo(created)
            await persistIdentityBackupToKeychainAsync(ironCore: ironCore)
        }

        try await setNickname(trimmedNickname)

        guard let verified = getIdentityInfo(), verified.initialized else {
            throw MeshError.notInitialized("Identity was created but verification failed")
        }

        publishIdentityInfo(verified)
        UserDefaults.standard.set(true, forKey: "hasCompletedInstallModeChoice")
        UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
        return verified
    }

    private func ensureLocalIdentityFederation() throws {
        guard let ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }

        var info = ironCore.getIdentityInfo()
        if !info.initialized {
            let restored = restoreIdentityFromKeychain(ironCore: ironCore)
            if restored {
                // P0: Grant consent after successful Keychain restore.
                // The Rust core starts with consent_granted=false, but if we have a
                // persisted identity backup, the user already consented in a prior session.
                // Matches Android's pattern in MeshRepository.kt:3033.
                ironCore.grantConsent()
                info = ironCore.getIdentityInfo()
            }
        }
        if !info.initialized {
            logVerbose("Identity not initialized; onboarding required")
            markIdentityAbsent()
            return
        }

        // P0: Ensure consent is granted whenever identity is initialized.
        // This handles the case where identity was loaded from sled but consent wasn't set
        // (e.g., after a process restart where consent_granted resets to false).
        // grantConsent is idempotent — calling it when already granted is a no-op.
        ironCore.grantConsent()
        publishIdentityInfo(info)
        // Persist even before a nickname is selected.  Android creates its backup
        // immediately after key generation; doing the same keeps a skipped or
        // interrupted setup recoverable on the next launch.
        persistIdentityBackupToKeychain(ironCore: ironCore)
    }

    /// Refresh the shared identity state after a service transition.  This is
    /// intentionally a lifecycle operation rather than a getter side effect, so
    /// SwiftUI never mutates observable state while rendering a view.
    private func refreshPublishedIdentityFromCore() {
        guard let ironCore else { return }
        let liveIdentity = ironCore.getIdentityInfo()
        guard liveIdentity.initialized else {
            markIdentityAbsent()
            return
        }
        publishIdentityInfo(liveIdentity)
    }

    private func validatePublishedIdentityAfterHydration() {
        guard let ironCore else {
            identityHydrationState = .pending
            return
        }
        let liveIdentity = ironCore.getIdentityInfo()
        guard liveIdentity.initialized else {
            markIdentityAbsent()
            return
        }
        publishIdentityInfo(liveIdentity)
    }

    private func publishIdentityInfo(_ info: IdentityInfo) {
        guard info.initialized else { return }
        if identityInfo != info {
            identityInfo = info
        }
        identityHydrationState = .ready
        Self.writeCachedIdentityInfo(info)
    }

    private func clearPublishedIdentity() {
        identityInfo = nil
        identityHydrationState = .absent
        Self.clearCachedIdentityInfo()
    }

    private func markIdentityAbsent() {
        clearPublishedIdentity()
    }

    private static func readCachedIdentityInfo() -> IdentityInfo? {
        guard let data = UserDefaults.standard.data(forKey: IdentityCacheStore.key),
              let cached = try? JSONDecoder().decode(CachedIdentityInfo.self, from: data),
              cached.initialized else {
            return nil
        }
        return cached.identityInfo
    }

    private static func writeCachedIdentityInfo(_ info: IdentityInfo) {
        guard info.initialized,
              let data = try? JSONEncoder().encode(CachedIdentityInfo(info)) else {
            return
        }
        UserDefaults.standard.set(data, forKey: IdentityCacheStore.key)
    }

    private static func clearCachedIdentityInfo() {
        UserDefaults.standard.removeObject(forKey: IdentityCacheStore.key)
    }

    private func getPlatformSecuredPassphrase() -> String {
        let service = "com.scmessenger.identity.passphrase"
        let account = "passphrase_v1"
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        if status == errSecSuccess, let data = result as? Data, let passphrase = String(data: data, encoding: .utf8) {
            return passphrase
        }
        let newPassphrase = UUID().uuidString + UUID().uuidString
        if let data = newPassphrase.data(using: .utf8) {
            let addQuery: [String: Any] = [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrService as String: service,
                kSecAttrAccount as String: account,
                kSecValueData as String: data,
                kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            ]
            _ = SecItemAdd(addQuery as CFDictionary, nil)
        }
        return newPassphrase
    }

    @discardableResult
    private func restoreIdentityFromKeychain(ironCore: IronCore) -> Bool {
        guard let backupPayload = readIdentityBackupFromKeychain() else {
            return false
        }
        let passphrase = getPlatformSecuredPassphrase()
        do {
            try ironCore.importIdentityBackup(backup: backupPayload, passphrase: passphrase)
            reloadContactManagerAfterIdentityImport()
            refreshPublishedIdentityFromCore()
            logVerbose("Restored identity and \(self.contactManager?.count() ?? 0) contact(s) from iOS Keychain backup payload")
            return true
        } catch {
            // Legacy fallback if the backup was encrypted with the old placeholder
            do {
                try ironCore.importIdentityBackup(backup: backupPayload, passphrase: "SCMessenger-Backup-Placeholder")
                reloadContactManagerAfterIdentityImport()
                refreshPublishedIdentityFromCore()
                logVerbose("Restored identity and \(self.contactManager?.count() ?? 0) contact(s) using legacy iOS Keychain backup placeholder passphrase")
                return true
            } catch {
                logger.warning("Identity Keychain restore failed: \(error.localizedDescription, privacy: .public)")
                return false
            }
        }
    }

    /// Recovery writes are committed before this method is called. Rebind the
    /// UI-facing manager to that durable store after any identity import. The
    /// current core shares contact handles for the same store, and reopening is
    /// an extra lifecycle safeguard for recovery paths that began with an
    /// already-open address book.
    private func reloadContactManagerAfterIdentityImport() {
        contactManager = nil
        do {
            contactManager = try ContactManager(storagePath: storagePath)
            contactManager?.flush()
        } catch {
            logger.error("Failed to reopen contacts after identity recovery: \(error.localizedDescription, privacy: .public)")
        }
    }

    private func persistIdentityBackupToKeychain(ironCore: IronCore?) {
        guard let ironCore else { return }
        do {
            // A bridge ContactManager buffers sled writes. Flush before taking
            // the recovery snapshot so the exported payload always reflects
            // every contact mutation that completed on this device.
            contactManager?.flush()
            let passphrase = getPlatformSecuredPassphrase()
            // This is a device-bound auto-backup: its passphrase is a 256-bit
            // random Keychain value, so the fast Blake3 KDF is appropriate and
            // lets us checkpoint contacts after every semantic change. The
            // user-facing export flow intentionally continues using Argon2id.
            let backup = try ironCore.exportIdentityBackupFast(passphrase: passphrase)
            writeIdentityBackupToKeychain(backup)
        } catch {
            logger.warning("Failed to persist identity backup payload: \(error.localizedDescription, privacy: .public)")
        }
    }

    private func persistIdentityBackupToKeychainAsync(ironCore: IronCore?) async {
        guard let ironCore else { return }
        // See createIdentity(nickname:): yield for responsive state publication,
        // then remain on the generated binding's required actor.
        await Task.yield()
        persistIdentityBackupToKeychain(ironCore: ironCore)
    }

    /// Make the address book durable both in its normal sled store and in the
    /// encrypted device recovery snapshot. This is intentionally synchronous:
    /// the fast backup KDF is intended for high-entropy device keys, and a
    /// contact saved immediately before an app update must not be lost to a
    /// delayed background task.
    private func checkpointContactPersistence(reason: String) {
        contactManager?.flush()
        guard let ironCore, ironCore.getIdentityInfo().initialized else { return }
        persistIdentityBackupToKeychain(ironCore: ironCore)
        logVerbose("Contact recovery checkpoint completed: \(reason)")
    }

    /// Coalesce non-user-facing, high-frequency metadata changes. Explicit
    /// address-book edits use `checkpointContactPersistence(reason:)` directly.
    private func scheduleContactPersistenceCheckpoint(reason: String) {
        guard contactManager != nil,
              ironCore?.getIdentityInfo().initialized == true else {
            return
        }

        pendingContactBackupTask?.cancel()
        let delay = contactBackupDebounceNanoseconds
        pendingContactBackupTask = Task { @MainActor [weak self] in
            try? await Task.sleep(nanoseconds: delay)
            guard !Task.isCancelled, let self else { return }
            self.pendingContactBackupTask = nil
            self.checkpointContactPersistence(reason: reason)
        }
    }

    /// Called by lifecycle owners before the app is backgrounded or the mesh
    /// service is stopped. It cancels a pending coalesced checkpoint and writes
    /// the current contact snapshot immediately.
    func checkpointPersistentState(reason: String) {
        pendingContactBackupTask?.cancel()
        pendingContactBackupTask = nil
        historyManager?.flush()
        checkpointContactPersistence(reason: reason)
    }

    private func readIdentityBackupFromKeychain() -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: IdentityBackupStore.service,
            kSecAttrAccount as String: IdentityBackupStore.account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else {
            return nil
        }
        return String(data: data, encoding: .utf8)
    }

    private func writeIdentityBackupToKeychain(_ payload: String) {
        guard let data = payload.data(using: .utf8) else { return }
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: IdentityBackupStore.service,
            kSecAttrAccount as String: IdentityBackupStore.account
        ]
        let attributes: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        ]
        let updateStatus = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        if updateStatus == errSecSuccess {
            return
        }
        if updateStatus == errSecItemNotFound {
            var addQuery = query
            addQuery.merge(attributes) { _, new in new }
            _ = SecItemAdd(addQuery as CFDictionary, nil)
        }
    }

    private func reconcileInstallScopedIdentityState() {
        let defaults = UserDefaults.standard
        if defaults.bool(forKey: InstallMarker.key) {
            return
        }

        defaults.set(true, forKey: InstallMarker.key)

        // Detect a new app container with a surviving device Keychain recovery
        // snapshot. Contacts are restored from that snapshot during hydration;
        // the post-start beacon remains a secondary recovery path for historic
        // snapshots that predate contact backup support.
        let keychainHasIdentity = readIdentityBackupFromKeychain() != nil
        let contactsOnDisk = FileManager.default.fileExists(
            atPath: URL(fileURLWithPath: storagePath).appendingPathComponent("contacts.db").path)
        let historyOnDisk = FileManager.default.fileExists(
            atPath: URL(fileURLWithPath: storagePath).appendingPathComponent("history.db").path)

        if keychainHasIdentity && (!contactsOnDisk || !historyOnDisk) {
            isReinstallWithMissingData = true
            appendDiagnostic("install_type=reinstall_identity_found contacts=\(contactsOnDisk) history=\(historyOnDisk)")
        } else {
            appendDiagnostic("install_type=first_install")
        }
    }

    /// Exclude raw identity storage from iCloud/local backup after the identity
    /// subdirectory has been created by ensure_storage_layout (called during start()).
    /// Contacts remain in normal backup scope for ordinary device restoration;
    /// immediate reinstall recovery is provided by the encrypted Keychain snapshot.
    private func excludeIdentitySubdirFromBackup() {
        let base = URL(fileURLWithPath: storagePath)
        // Exclude the Rust identity sled directory (keys already live in Keychain).
        for subpath in ["identity", "inbox", "outbox"] {
            var url = base.appendingPathComponent(subpath)
            guard FileManager.default.fileExists(atPath: url.path) else { continue }
            var v = URLResourceValues()
            v.isExcludedFromBackup = true
            try? url.setResourceValues(v)
        }
        // Exclude volatile transport state files.
        for filename in ["pending_outbox.json", "diagnostics.log"] {
            var url = base.appendingPathComponent(filename)
            guard FileManager.default.fileExists(atPath: url.path) else { continue }
            var v = URLResourceValues()
            v.isExcludedFromBackup = true
            try? url.setResourceValues(v)
        }
    }

    // MARK: - Messaging (with Relay Enforcement)

    /// Send a message to a peer
    /// CRITICAL: Enforces relay = messaging coupling
    func sendMessage(peerId: String, content: String) async throws {
        logVerbose("Send message to \(peerId)")

        // RELAY ENFORCEMENT
        // Check if relay/messaging is enabled (bidirectional control)
        // Default to ENABLED when settings unavailable (matches Rust default: relay_enabled=true)
        let currentSettings = try? settingsManager?.load()
        let isRelayEnabled = currentSettings?.relayEnabled ?? true

        if !isRelayEnabled {
            let errorMsg = "Cannot send message: Relay is disabled. Enable relay in Settings to send and receive messages."
            logger.error("\(errorMsg)")
            throw MeshError.relayDisabled(errorMsg)
        }

        // Throttling: prevent sending identical content to same peer within 1s
        let throttleKey = "\(peerId):\(content.hashValue)"
        if let lastSend = transmissionThrottleCache[throttleKey], Date().timeIntervalSince(lastSend) < transmissionThrottleInterval {
            logVerbose("Throttling duplicate send for \(peerId)")
            return
        }
        transmissionThrottleCache[throttleKey] = Date()

        // Proceed with sending message
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }

        // Get recipient's public key
        let contact = try? contactManager?.get(peerId: peerId)
        guard let recipientPublicKey = contact?.publicKey else {
            throw MeshError.contactNotFound("Contact \(peerId) not found or has no public key")
        }

        // Pre-validate public key format to provide descriptive errors
        let trimmedKey = recipientPublicKey.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmedKey.isEmpty {
            logger.error("[ERROR] Contact \(peerId) has an empty public key")
            throw MeshError.contactNotFound("Contact \(peerId) has no public key. Please re-add this contact with a valid public key.")
        }
        if trimmedKey.count != 64 {
            logger.error("[ERROR] Contact \(peerId) has invalid public key length: \(trimmedKey.count) chars (expected 64)")
            throw MeshError.contactNotFound("Contact \(peerId) has an invalid public key (wrong length: \(trimmedKey.count), expected 64 hex characters). Please re-add this contact.")
        }
        let hexChars = CharacterSet(charactersIn: "0123456789abcdefABCDEF")
        if !trimmedKey.unicodeScalars.allSatisfy({ hexChars.contains($0) }) {
            logger.error("[ERROR] Contact \(peerId) has non-hex characters in public key")
            throw MeshError.contactNotFound("Contact \(peerId) has an invalid public key (non-hex characters found). Please re-add this contact.")
        }

        logVerbose("Preparing message for \(peerId) with key: \(trimmedKey.prefix(8))...")
        let routing = parseRoutingHintsFromNotes(contact?.notes)
        let multipeerPeerId = routing.multipeerPeerId ?? defaultMultipeerPeerId(fromPublicKey: trimmedKey)
        let routePeerCandidates = buildRoutePeerCandidates(
            peerId: peerId,
            cachedRoutePeerId: routing.libp2pPeerId,
            notes: contact?.notes,
            recipientPublicKey: trimmedKey
        )
        if isKnownRelay(peerId) || isBootstrapRelayPeer(peerId) {
            throw MeshError.contactNotFound("Refusing to use headless relay identity as a chat recipient: \(peerId)")
        }
        let preferredRoutePeerId = routePeerCandidates.first

        // Prepare and send message (use trimmed key to handle any stored whitespace)
        let outboundContent = await encodeMessageWithIdentityHints(content)
        let prepared = try ironCore.prepareMessageWithId(
            recipientPublicKeyHex: trimmedKey,
            text: outboundContent,
            msgType: .text,
            ttl: nil
        )
        let messageId = prepared.messageId.trimmingCharacters(in: .whitespacesAndNewlines)
        if messageId.isEmpty {
            throw MeshError.notInitialized("Core returned empty message ID")
        }
        let envelopeData = Data(prepared.envelopeData)

        // Record in history FIRST so it's persisted even if bridge fails
        let messageRecord = MessageRecord(
            id: messageId,
            direction: .sent,
            peerId: peerId,
            content: content,
            timestamp: UInt64(Date().timeIntervalSince1970),
            senderTimestamp: UInt64(Date().timeIntervalSince1970),
            delivered: false,
            status: .queued,
            hidden: false
        )
        try? historyManager?.add(record: messageRecord)
        historyManager?.flush()

        // Notify UI (Unified flow for sent messages)
        messageUpdates.send(messageRecord)
        logDiagnostic("delivery_state msg=\(messageId) state=pending detail=message_prepared_local_history_written")
        let delivery = await attemptDirectSwarmDelivery(
            routePeerCandidates: routePeerCandidates,
            addresses: routing.listeners,
            envelopeData: envelopeData,
            multipeerPeerId: multipeerPeerId,
            blePeerId: routing.blePeerId,
            traceMessageId: messageId,
            attemptContext: "initial_send",
            strictBleOnlyOverride: strictBleOnlyValidation,
            recipientIdentityId: trimmedKey,
            intendedDeviceId: contact?.lastKnownDeviceId
        )
        let selectedRoutePeerId = delivery.routePeerId ?? preferredRoutePeerId

        if isMessageDeliveredLocally(messageId) {
            removePendingOutbound(historyRecordId: messageId)
            appendDiagnostic("delivery_state msg=\(messageId) state=delivered detail=delivery_receipt_arrived_before_enqueue")
        } else if let terminalFailureCode = delivery.terminalFailureCode {
            enqueuePendingOutbound(
                historyRecordId: messageId,
                peerId: peerId,
                routePeerId: selectedRoutePeerId,
                addresses: routing.listeners,
                envelopeData: envelopeData,
                initialAttemptCount: 1,
                initialDelaySec: 0,
                strictBleOnlyMode: strictBleOnlyValidation,
                recipientIdentityId: trimmedKey,
                intendedDeviceId: contact?.lastKnownDeviceId,
                terminalFailureCode: terminalFailureCode
            )
            appendDiagnostic("delivery_state msg=\(messageId) state=rejected detail=terminal_failure_code=\(terminalFailureCode)")
            throw MeshError.invalidInput(terminalIdentityFailureMessage(terminalFailureCode))
        } else {
            enqueuePendingOutbound(
                historyRecordId: messageId,
                peerId: peerId,
                routePeerId: selectedRoutePeerId,
                addresses: routing.listeners,
                envelopeData: envelopeData,
                initialAttemptCount: 1,
                initialDelaySec: delivery.acked ? receiptAwaitSeconds : 0,
                strictBleOnlyMode: strictBleOnlyValidation,
                recipientIdentityId: trimmedKey,
                intendedDeviceId: contact?.lastKnownDeviceId
            )
        }

        clearNotificationRequestPending(peerId: peerId)
        NotificationManager.shared.markConversationRead(conversationId: peerId)
    }

    /// Handle incoming message (from CoreDelegate callback)
    func onMessageReceived(
        senderId: String,
        senderPublicKeyHex: String,
        messageId: String,
        senderTimestamp: UInt64,
        data: Data
    ) {
        logVerbose("Message from \(senderId): \(messageId)")
        logDiagnostic("msg_rx sender=\(senderId) msg=\(messageId)")

        // RELAY ENFORCEMENT
        // Check if relay/messaging is enabled (bidirectional control)
        // Default to ENABLED when settings unavailable (matches Rust default: relay_enabled=true)
        let currentSettings = try? settingsManager?.load()
        let isRelayEnabled = currentSettings?.relayEnabled ?? true

        if !isRelayEnabled {
            logger.warning("Dropped message from \(senderId): relay disabled")
            logDiagnostic("msg_rx_dropped sender=\(senderId) msg=\(messageId) reason=relay_disabled")
            return
        }

        let normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        let rawContent = String(data: data, encoding: .utf8) ?? "[binary]"
        let decodedPayload = decodeMessageWithIdentityHints(rawContent)
        let hintedIdentity = decodedPayload.hints
        let hintedKey = normalizePublicKey(hintedIdentity?.publicKey)
        let verifiedHints: MessageIdentityHints? = {
            if let hintedKey, let normalizedSenderKey, hintedKey != normalizedSenderKey {
                logger.warning(
                    "Ignoring forged message identity hint for \(senderId): key mismatch (hint=\(hintedKey.prefix(8))..., envelope=\(normalizedSenderKey.prefix(8))...)"
                )
                return nil
            }
            return hintedIdentity
        }()

        var canonicalPeerId = resolveCanonicalPeerId(senderId: senderId, senderPublicKeyHex: senderPublicKeyHex)
        canonicalPeerId = resolveCanonicalPeerIdFromMessageHints(
            resolvedCanonicalPeerId: canonicalPeerId,
            senderId: senderId,
            senderPublicKeyHex: senderPublicKeyHex,
            hintedIdentityId: verifiedHints?.identityId
        )

        let hintedRoutePeerId = verifiedHints?.libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        let routePeerId: String? = isLibp2pPeerId(senderId) ? senderId : hintedRoutePeerId
        let hintedAddresses = (verifiedHints?.listeners ?? []) +
            (verifiedHints?.externalAddresses ?? []) +
            (verifiedHints?.connectionHints ?? [])
        let hintedDialCandidates = buildDialCandidatesForPeer(
            routePeerId: routePeerId,
            rawAddresses: hintedAddresses,
            includeRelayCircuits: true
        )
        let knownNickname = selectAuthoritativeNickname(
            incoming: verifiedHints?.nickname,
            existing: resolveKnownPeerNickname(
                canonicalPeerId: canonicalPeerId,
                routePeerId: routePeerId,
                publicKey: normalizedSenderKey
            )
        )
        if canonicalPeerId != senderId {
            logVerbose("Canonicalized sender \(senderId) -> \(canonicalPeerId) using public key match")
        }
        if isBootstrapRelayPeer(canonicalPeerId) {
            logVerbose("Ignoring payload attributed to bootstrap relay peer \(canonicalPeerId)")
            return
        }

        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid (Rust verified it during decrypt)
        let messageKind = decodedPayload.kind.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        let isChatEvent = messageKind == "text" || messageKind.isEmpty

        let existingContact = try? contactManager?.get(peerId: canonicalPeerId)
        let requestPending = isNotificationRequestPending(notes: existingContact?.notes)
        let hasExistingConversation = ((try? historyManager?.conversation(peerId: canonicalPeerId, limit: 1)) ?? []).isEmpty == false
        let notificationSettings = currentNotificationSettings()
        let notificationDecision = ironCore?.classifyNotification(
            message: NotificationMessageContext(
                conversationId: canonicalPeerId,
                senderPeerId: canonicalPeerId,
                messageId: messageId,
                explicitDmRequest: nil,
                senderIsKnownContact: existingContact != nil && !requestPending,
                hasExistingConversation: hasExistingConversation,
                isSelfOriginated: false,
                isDuplicate: false,
                alreadySeen: false,
                isBlocked: false
            ),
            uiState: NotificationUiState(
                appInForeground: notificationAppInForeground,
                activeConversationId: notificationActiveConversationId
            ),
            settings: notificationSettings
        )
        if existingContact == nil && isChatEvent {
            if let normalizedSenderKey {
                var routeNotes: String?
                if let routePeerId, !routePeerId.isEmpty {
                    routeNotes = appendRoutingHint(notes: nil, key: "libp2p_peer_id", value: routePeerId)
                }
                routeNotes = upsertRoutingListeners(
                    notes: routeNotes,
                    listeners: normalizeOutboundListenerHints(hintedDialCandidates)
                )
                if notificationDecision?.kind == .directMessageRequest {
                    routeNotes = appendRoutingHint(
                        notes: routeNotes,
                        key: NotificationNoteKey.requestPending,
                        value: "true"
                    )
                }
                let autoContact = Contact(
                    peerId: canonicalPeerId,
                    nickname: knownNickname,
                    localNickname: nil,
                    publicKey: normalizedSenderKey,
                    addedAt: UInt64(Date().timeIntervalSince1970),
                    lastSeen: UInt64(Date().timeIntervalSince1970),
                    notes: routeNotes,
                    lastKnownDeviceId: verifiedHints?.deviceId,
                    verifiedAt: nil,
                    isTombstone: false
                )
                do {
                    try contactManager?.add(contact: autoContact)
                    contactManager?.flush()
                    checkpointContactPersistence(reason: "incoming_contact_created")
                    logVerbose("Auto-created contact from received message: \(canonicalPeerId.prefix(8)) key: \(normalizedSenderKey.prefix(8))...")
                } catch {
                    logger.warning("Auto-create contact failed for \(canonicalPeerId.prefix(8)): \(error.localizedDescription)")
                }
            }
        } else if let existingContact {
            try? contactManager?.updateLastSeen(peerId: canonicalPeerId)
            scheduleContactPersistenceCheckpoint(reason: "incoming_contact_last_seen")

            var updatedNotes = existingContact.notes
            var updatedNickname = existingContact.nickname
            let currentRouting = parseRoutingHintsFromNotes(existingContact.notes)
            let normalizedRoutePeerId = routePeerId?
                .trimmingCharacters(in: .whitespacesAndNewlines)
                .nilIfEmpty
            var shouldPersistContact = false

            if existingContact.nickname?.isEmpty ?? true, let knownNickname, !knownNickname.isEmpty {
                updatedNickname = knownNickname
                shouldPersistContact = true
            }

            if let normalizedRoutePeerId,
               let normalizedSenderKey,
               normalizePublicKey(existingContact.publicKey) == normalizedSenderKey,
               currentRouting.libp2pPeerId != normalizedRoutePeerId {
                updatedNotes = appendRoutingHint(notes: updatedNotes, key: "libp2p_peer_id", value: routePeerId)
                updatedNotes = upsertRoutingListeners(
                    notes: updatedNotes,
                    listeners: normalizeOutboundListenerHints(hintedDialCandidates)
                )
                shouldPersistContact = true
            }

            if notificationDecision?.kind == .directMessageRequest && !requestPending {
                updatedNotes = appendRoutingHint(
                    notes: updatedNotes,
                    key: NotificationNoteKey.requestPending,
                    value: "true"
                )
                shouldPersistContact = true
            }

            if shouldPersistContact {
                let updatedContact = Contact(
                    peerId: existingContact.peerId,
                    nickname: updatedNickname,
                    localNickname: existingContact.localNickname,
                    publicKey: existingContact.publicKey,
                    addedAt: existingContact.addedAt,
                    lastSeen: existingContact.lastSeen,
                    notes: updatedNotes,
                    lastKnownDeviceId: verifiedHints?.deviceId ?? existingContact.lastKnownDeviceId,
                    verifiedAt: existingContact.verifiedAt,
                    isTombstone: existingContact.isTombstone
                )
                try? contactManager?.add(contact: updatedContact)
                contactManager?.flush()
                checkpointContactPersistence(reason: "incoming_contact_updated")
            }
        }

        if let normalizedSenderKey {
            upsertFederatedContact(
                canonicalPeerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: knownNickname,
                libp2pPeerId: routePeerId,
                listeners: hintedDialCandidates,
                deviceId: verifiedHints?.deviceId,
                createIfMissing: false
            )
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: knownNickname,
                peerId: canonicalPeerId,
                publicKey: normalizedSenderKey
            )
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname,
                transport: .internet,
                isFull: true,
                isRelay: false,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(canonicalPeerId, info: discoveryInfo)
            if let routePeerId, routePeerId != canonicalPeerId {
                updateDiscoveredPeer(routePeerId, info: discoveryInfo)
            }
            let listeners = ((routePeerId.map(getDialHintsForRoutePeer(_:)) ?? []) + hintedDialCandidates)
                .reduce(into: [String]()) { acc, addr in
                    if !acc.contains(addr) { acc.append(addr) }
                }
            emitIdentityDiscoveredIfChanged(
                peerId: canonicalPeerId,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname,
                libp2pPeerId: routePeerId,
                listeners: listeners
            )
            annotateIdentityInLedger(
                routePeerId: routePeerId,
                listeners: listeners,
                publicKey: normalizedSenderKey,
                nickname: discoveredNickname
            )
        }

        if messageKind == "identity_sync" {
            logVerbose("Processed identity sync from \(canonicalPeerId) (route=\(routePeerId ?? "none"))")
            logDiagnostic("msg_identity_sync peer=\(canonicalPeerId) route=\(routePeerId ?? "none")")
            sendDeliveryReceiptAsync(
                senderPublicKeyHex: senderPublicKeyHex,
                messageId: messageId,
                senderId: canonicalPeerId,
                preferredRoutePeerId: routePeerId,
                preferredListenerHints: hintedDialCandidates
            )
            return
        }

        if messageKind == "history_sync" {
            logVerbose("Processed history sync request from \(canonicalPeerId)")
            logDiagnostic("processed_history_sync_request_from peer=\(canonicalPeerId)")
            sendHistorySyncDataIfNeeded(canonicalPeerId: canonicalPeerId, routePeerId: routePeerId, recipientPublicKey: senderPublicKeyHex, listeners: hintedDialCandidates, wifiPeerId: nil)
            sendDeliveryReceiptAsync(senderPublicKeyHex: senderPublicKeyHex, messageId: messageId, senderId: canonicalPeerId, preferredRoutePeerId: routePeerId, preferredListenerHints: hintedDialCandidates)
            return
        }
        if messageKind == "history_sync_data" {
            logVerbose("Processed history sync data from \(canonicalPeerId)")
            logDiagnostic("processed_history_sync_data_from peer=\(canonicalPeerId)")
            if let data = decodedPayload.text.data(using: .utf8),
               let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
                for obj in arr {
                    guard let msgId = obj["id"] as? String else { continue }
                    if let existing = try? historyManager?.get(id: msgId) {
                        let dirStr = obj["dir"] as? String
                        let isPeerReflectingRecv = dirStr == "recv"

                        // If peer has the message we sent, it's delivered.
                        if existing.direction == .sent && isPeerReflectingRecv {
                            if !existing.delivered {
                                try? historyManager?.markDelivered(id: msgId)
                                if let updated = try? historyManager?.get(id: msgId) {
                                    messageUpdates.send(updated)
                                    MeshEventBus.shared.messageEvents.send(.delivered(messageId: msgId))
                                }
                            }
                            // Always clear from outbox if peer has it
                            removePendingOutbound(historyRecordId: msgId)
                        }
                        continue
                    }

                    let dirStr = obj["dir"] as? String
                    let record = MessageRecord(
                        id: msgId,
                        direction: dirStr == "sent" ? .received : .sent,
                        peerId: canonicalPeerId,
                        content: obj["txt"] as? String ?? "",
                        timestamp: UInt64(obj["ts"] as? Int64 ?? 0),
                        senderTimestamp: UInt64(obj["sts"] as? Int64 ?? 0),
                        delivered: obj["del"] as? Bool ?? false,
                        status: obj["del"] as? Bool ?? false ? .delivered : .queued,
                        hidden: false
                    )
                    try? historyManager?.add(record: record)
                    messageUpdates.send(record)
                }
                historyManager?.flush()
            }
            sendDeliveryReceiptAsync(senderPublicKeyHex: senderPublicKeyHex, messageId: messageId, senderId: canonicalPeerId, preferredRoutePeerId: routePeerId, preferredListenerHints: hintedDialCandidates)
            return
        }

        // Process message
        let content = decodedPayload.text

        // Smart deduplication: check if this is a duplicate and track time variance
        let dedupResult = smartTransportRouter?.checkAndRecordMessage(
            messageId: messageId,
            transport: .internet // Default transport for incoming messages
        )

        if let existing = try? historyManager?.get(id: messageId),
           existing.direction == .received {
            logVerbose("Duplicate inbound message \(messageId); acknowledging without UI emit")

            // Log duplicate with time variance for mesh enhancement
            if let dedup = dedupResult, dedup.isDuplicate, let timeVariance = dedup.timeVarianceMs {
                let dupCount = self.smartTransportRouter?.getDedupStats(messageId: messageId)?.duplicateCount ?? 0
                logDiagnostic("msg_duplicate msg=\(messageId) time_variance_ms=\(timeVariance) first_transport=\(dedup.firstTransport?.rawValue ?? "unknown") duplicate_count=\(dupCount)")
            }

            sendDeliveryReceiptAsync(
                senderPublicKeyHex: senderPublicKeyHex,
                messageId: messageId,
                senderId: canonicalPeerId,
                preferredRoutePeerId: routePeerId,
                preferredListenerHints: hintedDialCandidates
            )
            return
        }

        // First receipt of this message - log for mesh enhancement tracking
        if let dedup = dedupResult, !dedup.isDuplicate {
            logDiagnostic("msg_first_receipt msg=\(messageId) transport=\(dedup.firstTransport?.rawValue ?? "unknown")")
        }

        let fallbackNow = UInt64(Date().timeIntervalSince1970)
        let canonicalTimestamp = senderTimestamp > 0 ? senderTimestamp : fallbackNow
        let messageRecord = MessageRecord(
            id: messageId,
            direction: .received,
            peerId: canonicalPeerId,
            content: content,
            timestamp: canonicalTimestamp,
            senderTimestamp: senderTimestamp,
            delivered: true,
            status: .delivered,
            hidden: false
        )

        try? historyManager?.add(record: messageRecord)
        historyManager?.flush()

        // Notify UI
        messageUpdates.send(messageRecord)
        logVerbose("Message received and processed from \(canonicalPeerId)")
        logDiagnostic("msg_rx_processed peer=\(canonicalPeerId) msg=\(messageId)")

        if isChatEvent && notificationSettings.notificationsEnabled,
           let notificationDecision,
           notificationDecision.kind != .none {
            NotificationManager.shared.sendNotification(
                decision: notificationDecision,
                senderDisplayName: displayNameForPeer(peerId: canonicalPeerId),
                content: content,
                soundEnabled: notificationSettings.soundEnabled,
                badgeEnabled: notificationSettings.badgeEnabled,
                routesToRequestsInbox: notificationDecision.kind == .directMessageRequest
            )
        }

        // Send delivery receipt ACK back to sender
        sendDeliveryReceiptAsync(
            senderPublicKeyHex: senderPublicKeyHex,
            messageId: messageId,
            senderId: canonicalPeerId,
            preferredRoutePeerId: routePeerId,
            preferredListenerHints: hintedDialCandidates
        )
    }

    /// Handle delivery receipt callbacks from CoreDelegate.
    /// Marks local history and removes pending retry entries when IDs match.
    func onDeliveryReceipt(messageId: String, status: String) {
        let normalized = status.lowercased()
        let normalizedMessageId = messageId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard normalized == "delivered" || normalized == "read" else { return }
        let hasPendingForReceipt = loadPendingOutbox().contains { $0.historyRecordId == normalizedMessageId }
        let existingRecord = try? historyManager?.get(id: normalizedMessageId)
        guard let existingRecord else {
            if hasPendingForReceipt {
                removePendingOutbound(historyRecordId: normalizedMessageId)
                logDiagnostic("delivery_state msg=\(normalizedMessageId) state=delivered detail=delivery_receipt_recovered_without_history status=\(normalized) direction=missing")
            } else {
                logDiagnostic("delivery_state msg=\(normalizedMessageId) state=pending detail=delivery_receipt_ignored_non_outbound status=\(normalized) direction=missing")
            }
            return
        }
        guard existingRecord.direction == .sent else {
            if hasPendingForReceipt {
                removePendingOutbound(historyRecordId: normalizedMessageId)
                logDiagnostic("delivery_state msg=\(normalizedMessageId) state=delivered detail=delivery_receipt_recovered_without_history status=\(normalized) direction=\(existingRecord.direction)")
            } else {
                logDiagnostic("delivery_state msg=\(normalizedMessageId) state=pending detail=delivery_receipt_ignored_non_outbound status=\(normalized) direction=\(existingRecord.direction)")
            }
            return
        }
        let wasAlreadyDelivered = existingRecord.delivered
        let firstReceiptSeen = markDeliveredReceiptSeen(normalizedMessageId)
        if !firstReceiptSeen && wasAlreadyDelivered {
            removePendingOutbound(historyRecordId: normalizedMessageId)
            logDiagnostic("delivery_state msg=\(normalizedMessageId) state=delivered detail=delivery_receipt_duplicate_status=\(normalized)")
            return
        }
        logDiagnostic("receipt_rx msg=\(normalizedMessageId) status=\(normalized)")
        if !wasAlreadyDelivered {
            try? historyManager?.markDelivered(id: normalizedMessageId)
            historyManager?.flush()
        }
        removePendingOutbound(historyRecordId: normalizedMessageId)
        if let updated = try? historyManager?.get(id: normalizedMessageId) {
            // Keep chat and conversation views aligned after receipt-driven status changes.
            messageUpdates.send(updated)
        }
        if wasAlreadyDelivered { return }
        _ = ironCore?.markMessageSent(messageId: normalizedMessageId)
        logDiagnostic("delivery_state msg=\(normalizedMessageId) state=delivered detail=delivery_receipt_status=\(normalized)")
        // CoreDelegateImpl also sends this to the event bus; avoiding double-emission here.
        // MeshEventBus.shared.messageEvents.send(.delivered(messageId: messageId))
    }

    private func sendDeliveryReceiptAsync(
        senderPublicKeyHex: String,
        messageId: String,
        senderId: String,
        preferredRoutePeerId: String? = nil,
        preferredListenerHints: [String] = [],
        preferredMultipeerPeerId: String? = nil,
        preferredBlePeerId: String? = nil
    ) {
        let normalizedMessageId = messageId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedMessageId.isEmpty else { return }

        if let activeTask = pendingReceiptSendTasks[normalizedMessageId], !activeTask.isCancelled {
            logDiagnostic("receipt_send msg=\(normalizedMessageId) state=deduped sender=\(senderId)")
            return
        }

        let task = Task { [weak self] in
            guard let self else { return }
            defer {
                Task { @MainActor [weak self] in
                    self?.pendingReceiptSendTasks.removeValue(forKey: normalizedMessageId)
                }
            }

            for attempt in 1...self.receiptSendMaxAttempts {
                if Task.isCancelled { return }
                do {
                    guard let receiptBytes = try self.ironCore?.prepareReceipt(
                        recipientPublicKeyHex: senderPublicKeyHex,
                        messageId: normalizedMessageId
                    ) else {
                        self.logVerbose("Skipping delivery receipt for \(normalizedMessageId): prepareReceipt returned nil")
                        return
                    }

                    let contact = try? self.contactManager?.get(peerId: senderId)
                    let hints = self.parseRoutingHintsFromNotes(contact?.notes)
                    let routeCandidates = self.buildRoutePeerCandidates(
                        peerId: senderId,
                        cachedRoutePeerId: preferredRoutePeerId ?? hints.libp2pPeerId,
                        notes: contact?.notes,
                        recipientPublicKey: senderPublicKeyHex
                    )

                    let delivery = await self.attemptDirectSwarmDelivery(
                        routePeerCandidates: routeCandidates,
                        addresses: (preferredListenerHints + hints.listeners).reduce(into: [String]()) { acc, addr in
                            if !acc.contains(addr) { acc.append(addr) }
                        },
                        envelopeData: receiptBytes,
                        multipeerPeerId: preferredMultipeerPeerId ?? hints.multipeerPeerId,
                        blePeerId: preferredBlePeerId ?? hints.blePeerId,
                        traceMessageId: normalizedMessageId,
                        attemptContext: "receipt_send",
                        recipientIdentityId: senderPublicKeyHex,
                        intendedDeviceId: contact?.lastKnownDeviceId
                    )
                    if delivery.acked {
                        self.logDiagnostic("receipt_send msg=\(normalizedMessageId) state=acked sender=\(senderId) attempt=\(attempt)")
                        self.logVerbose("Targeted delivery receipt sent for \(normalizedMessageId) to \(senderId)")
                        return
                    }

                    if attempt < self.receiptSendMaxAttempts {
                        let delaySec = self.receiptRetryDelaySeconds(forAttempt: attempt)
                        self.logDiagnostic(
                            "receipt_send msg=\(normalizedMessageId) state=retry_scheduled sender=\(senderId) attempt=\(attempt) delay_sec=\(delaySec)"
                        )
                        try? await Task.sleep(nanoseconds: delaySec * 1_000_000_000)
                    } else {
                        self.logDiagnostic(
                            "receipt_send msg=\(normalizedMessageId) state=exhausted sender=\(senderId) attempts=\(self.receiptSendMaxAttempts)"
                        )
                    }
                } catch is CancellationError {
                    return
                } catch {
                    self.logVerbose("Failed to send delivery receipt for \(normalizedMessageId): \(error)")
                }
            }
        }
        pendingReceiptSendTasks[normalizedMessageId] = task
    }

    private func receiptRetryDelaySeconds(forAttempt attempt: Int) -> UInt64 {
        let exponent = min(max(attempt - 1, 0), 3)
        return UInt64(1 << exponent)
    }

    private func sendIdentitySyncIfNeeded(routePeerId: String, knownPublicKey: String? = nil) {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedRoute.isEmpty,
              !isBootstrapRelayPeer(normalizedRoute) else {
            return
        }
        guard identitySyncSentPeers.insert(normalizedRoute).inserted else {
            return
        }

        Task {
            let extractedPublicKey: String? = {
                guard let ironCore else { return nil }
                return try? ironCore.extractPublicKeyFromPeerId(peerId: normalizedRoute)
            }()
            let recipientPublicKey = normalizePublicKey(knownPublicKey) ??
                normalizePublicKey(extractedPublicKey)
            guard let recipientPublicKey else {
                identitySyncSentPeers.remove(normalizedRoute)
                return
            }

            do {
                let payload = await encodeIdentitySyncPayload()
                let prepared = try ironCore?.prepareMessageWithId(
                    recipientPublicKeyHex: recipientPublicKey,
                    text: payload,
                    msgType: .text,
                    ttl: nil
                )
                guard let prepared = prepared else {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return
                }

                let contact = (try? contactManager?.list())?.first(where: {
                    $0.peerId == normalizedRoute || parseRoutingHintsFromNotes($0.notes).libp2pPeerId == normalizedRoute
                })
                let hints = parseRoutingHintsFromNotes(contact?.notes)
                let routeCandidates = buildRoutePeerCandidates(
                    peerId: contact?.peerId ?? normalizedRoute,
                    cachedRoutePeerId: normalizedRoute,
                    notes: contact?.notes,
                    recipientPublicKey: recipientPublicKey
                )

                _ = await self.attemptDirectSwarmDelivery(
                    routePeerCandidates: routeCandidates,
                    addresses: hints.listeners,
                    envelopeData: Data(prepared.envelopeData),
                    blePeerId: hints.blePeerId,
                    recipientIdentityId: recipientPublicKey
                )
                self.logVerbose("Identity sync sent to \(normalizedRoute)")
            } catch {
                self.identitySyncSentPeers.remove(normalizedRoute)
                self.logVerbose("Failed to send identity sync to \(normalizedRoute): \(error.localizedDescription)")
            }
        }
    }

    /// Resolve incoming sender IDs to a canonical contact ID.
    ///
    /// Canonicalization prefers one stable contact per public key.
    /// Exact sender ID matches still win, then a unique public-key match wins.
    /// Routing hints are used only as fallback when key-based matching is ambiguous.

    private func sendHistorySyncIfNeeded(routePeerId: String, knownPublicKey: String? = nil) {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        logDiagnostic("sending_history_sync_request_consider route=\(normalizedRoute)")
        logVerbose("sendHistorySyncIfNeeded called for \(normalizedRoute)")
        guard !normalizedRoute.isEmpty, !isBootstrapRelayPeer(normalizedRoute) else { return }
        let now = Date()
        let lastSent = historySyncSentPeers[normalizedRoute]
        let shouldSend = lastSent == nil || now.timeIntervalSince(lastSent ?? now) > historySyncCooldown
        logVerbose("sendHistorySyncIfNeeded shouldSend=\(shouldSend) for \(normalizedRoute) (age=\(lastSent.map { now.timeIntervalSince($0) } ?? 999)s)")
        guard shouldSend else { return }
        historySyncSentPeers[normalizedRoute] = now
        logVerbose("sendHistorySyncIfNeeded inserting for \(normalizedRoute)")

        Task {
            let extractedPublicKey = try? ironCore?.extractPublicKeyFromPeerId(peerId: normalizedRoute)
            guard let recipientPublicKey = normalizePublicKey(knownPublicKey) ?? normalizePublicKey(extractedPublicKey) else {
                historySyncSentPeers.removeValue(forKey: normalizedRoute)
                logDiagnostic("history_sync_request_failed_no_pubkey route=\(normalizedRoute)")
                logger.error("historySync failed: missing recipientPublicKey for \(normalizedRoute)")
                return
            }

            do {
                let payload = await encodeMeshMessagePayload(content: "", kind: "history_sync")
                guard let prepared = try? ironCore?.prepareMessageWithId(
                    recipientPublicKeyHex: recipientPublicKey,
                    text: payload,
                    msgType: .text,
                    ttl: nil
                ) else {
                    historySyncSentPeers.removeValue(forKey: normalizedRoute)
                    logDiagnostic("history_sync_request_failed_prepare route=\(normalizedRoute)")
                    logger.error("historySync request failed to prepare message")
                    return
                }

                let contact = (try? contactManager?.list())?.first(where: {
                    $0.peerId == normalizedRoute || parseRoutingHintsFromNotes($0.notes).libp2pPeerId == normalizedRoute
                })
                let hints = parseRoutingHintsFromNotes(contact?.notes)
                let routeCandidates = buildRoutePeerCandidates(
                    peerId: contact?.peerId ?? normalizedRoute,
                    cachedRoutePeerId: normalizedRoute,
                    notes: contact?.notes,
                    recipientPublicKey: recipientPublicKey
                )

                _ = await self.attemptDirectSwarmDelivery(
                    routePeerCandidates: routeCandidates,
                    addresses: hints.listeners,
                    envelopeData: Data(prepared.envelopeData),
                    blePeerId: hints.blePeerId,
                    recipientIdentityId: recipientPublicKey
                )
                self.logVerbose("History sync request sent to \(normalizedRoute)")
            }
        }
    }

    private var historySyncDataInProgress: Set<String> = []

    private func sendHistorySyncDataIfNeeded(canonicalPeerId: String, routePeerId: String?, recipientPublicKey: String, listeners: [String], wifiPeerId: String?) {
        guard !historySyncDataInProgress.contains(canonicalPeerId) else {
            logVerbose("sendHistorySyncDataIfNeeded: already in progress for \(canonicalPeerId)")
            return
        }
        historySyncDataInProgress.insert(canonicalPeerId)

        Task {
            defer { DispatchQueue.main.async { self.historySyncDataInProgress.remove(canonicalPeerId) } }
            do {
                self.logVerbose("sendHistorySyncDataIfNeeded started for \(canonicalPeerId)")
                guard let manager = historyManager,
                      let fetchedMsgs = try? manager.conversation(peerId: canonicalPeerId, limit: 400),
                      !fetchedMsgs.isEmpty else {
                    self.logVerbose("sendHistorySyncDataIfNeeded: no recent msgs for \(canonicalPeerId)")
                    return
                }
                self.logVerbose("sendHistorySyncDataIfNeeded: compiling \(fetchedMsgs.count) msgs for \(canonicalPeerId)")
                let recentMsgs = fetchedMsgs.sorted { $0.timestamp < $1.timestamp }

                let hints = parseRoutingHintsFromNotes((try? contactManager?.get(peerId: canonicalPeerId))?.notes)
                let routeCandidates = buildRoutePeerCandidates(peerId: canonicalPeerId, cachedRoutePeerId: routePeerId ?? hints.libp2pPeerId, notes: nil, recipientPublicKey: recipientPublicKey)
                let allListeners = Array(Set(listeners + hints.listeners))

                // Chunk into batches of 20 to stay within encryption payload limits
                let batchSize = 20
                let batches = stride(from: 0, to: recentMsgs.count, by: batchSize).map {
                    Array(recentMsgs[$0..<min($0 + batchSize, recentMsgs.count)])
                }
                var sentBatches = 0

                for (batchIndex, batch) in batches.enumerated() {
                    var arr: [[String: Any]] = []
                    for msg in batch {
                        arr.append([
                            "id": msg.id,
                            "dir": msg.direction == .sent ? "sent" : "recv",
                            "pid": msg.peerId,
                            "txt": msg.content,
                            "ts": Int64(msg.timestamp),
                            "sts": Int64(msg.senderTimestamp),
                            "del": msg.delivered
                        ])
                    }
                    guard let data = try? JSONSerialization.data(withJSONObject: arr), let jsonStr = String(data: data, encoding: .utf8) else {
                        self.logger.error("sendHistorySyncData json serialization failed for batch \(batchIndex)")
                        continue
                    }

                    let payload = await encodeMeshMessagePayload(content: jsonStr, kind: "history_sync_data")
                    guard let prepared = try? ironCore?.prepareMessageWithId(
                        recipientPublicKeyHex: recipientPublicKey,
                        text: payload,
                        msgType: .text,
                        ttl: nil
                    ) else {
                        self.logger.error("sendHistorySyncData prepareMessageWithId failed for batch \(batchIndex) (\(batch.count) msgs)")
                        continue
                    }

                    _ = await self.attemptDirectSwarmDelivery(
                        routePeerCandidates: routeCandidates,
                        addresses: allListeners,
                        envelopeData: Data(prepared.envelopeData),
                        multipeerPeerId: hints.multipeerPeerId,
                        blePeerId: hints.blePeerId,
                        recipientIdentityId: recipientPublicKey
                    )
                    sentBatches += 1
                    // Small delay between batches to avoid overwhelming BLE
                    if batchIndex < batches.count - 1 {
                        try? await Task.sleep(nanoseconds: 200_000_000)
                    }
                }
                self.logVerbose("History sync data sent to \(canonicalPeerId) (\(sentBatches)/\(batches.count) batches, \(recentMsgs.count) items total)")
            }
        }
    }
    private func resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String) -> String {
        guard let normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex),
              let contacts = try? contactManager?.list() else {
            return senderId
        }

        let exactMatch = contacts.first {
            isSamePeerId($0.peerId, id2: senderId) && normalizePublicKey($0.publicKey) == normalizedIncomingKey
        }
        if let match = exactMatch { return match.peerId }

        let keyedMatches = contacts.filter {
            normalizePublicKey($0.publicKey) == normalizedIncomingKey
        }
        if keyedMatches.count == 1 {
            return keyedMatches[0].peerId
        }
        if keyedMatches.count > 1 {
            logger.warning("Ambiguous canonical sender mapping for key \(normalizedIncomingKey.prefix(8))...; trying route-hint fallback")
        }

        if isLibp2pPeerId(senderId) {
            let linkedIdentityMatches = contacts.filter {
                guard normalizePublicKey($0.publicKey) == normalizedIncomingKey else { return false }
                guard !isSamePeerId($0.peerId, id2: senderId) else { return false }
                guard let notes = $0.notes,
                      let routing = parseRoutingInfo(notes: notes) else { return false }

                if !routing.libp2pPeerId.isEmpty {
                    return isSamePeerId(routing.libp2pPeerId, id2: senderId)
                }
                return false
            }

            if linkedIdentityMatches.count == 1 {
                return linkedIdentityMatches[0].peerId
            }
            if linkedIdentityMatches.count > 1 {
                logger.warning("Ambiguous canonical sender mapping for \(senderId); keeping raw sender ID")
            }
            return normalizePeerId(senderId)
        }

        guard isIdentityId(senderId) else { return normalizePeerId(senderId) }
        let keyedRoutedMatches = contacts.filter {
            guard normalizePublicKey($0.publicKey) == normalizedIncomingKey else { return false }
            guard $0.peerId != senderId else { return false }
            return parseRoutingHintsFromNotes($0.notes).libp2pPeerId != nil || isLibp2pPeerId($0.peerId)
        }
        if keyedRoutedMatches.count == 1 {
            return keyedRoutedMatches[0].peerId
        }
        if keyedRoutedMatches.count > 1 {
            logger.warning("Ambiguous identity sender mapping for \(senderId); keeping raw sender ID")
        }

        return normalizePeerId(senderId)
    }

    private func resolveCanonicalPeerIdFromMessageHints(
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ) -> String {
        guard let hint = hintedIdentityId,
              !hint.isEmpty else {
            return resolvedCanonicalPeerId
        }
        let normalizedHint = normalizePeerId(hint)
        guard isIdentityId(normalizedHint) else {
            return resolvedCanonicalPeerId
        }
        if isSamePeerId(normalizedHint, id2: resolvedCanonicalPeerId) { return resolvedCanonicalPeerId }
        if isBootstrapRelayPeer(normalizedHint) { return resolvedCanonicalPeerId }

        let normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        let contacts = (try? contactManager?.list()) ?? []

        if let normalizedSenderKey {
            if let hintedContact = contacts.first(where: { isSamePeerId($0.peerId, id2: normalizedHint) }),
               normalizePublicKey(hintedContact.publicKey) == normalizedSenderKey {
                return hintedContact.peerId
            }

            let keyMatches = contacts.filter { normalizePublicKey($0.publicKey) == normalizedSenderKey }
            if keyMatches.count == 1 {
                return keyMatches[0].peerId
            }
            if !keyMatches.isEmpty {
                return resolvedCanonicalPeerId
            }
        }

        if isSamePeerId(resolvedCanonicalPeerId, id2: senderId) || isLibp2pPeerId(resolvedCanonicalPeerId) {
            return normalizedHint
        }
        return resolvedCanonicalPeerId
    }

    private func encodeMessageWithIdentityHints(_ content: String) async -> String {
        return await encodeMeshMessagePayload(content: content, kind: "text")
    }

    private func encodeIdentitySyncPayload() async -> String {
        return await encodeMeshMessagePayload(content: "", kind: "identity_sync")
    }

    private func encodeMeshMessagePayload(content: String, kind: String) async -> String {
        guard let info = ironCore?.getIdentityInfo(),
              let publicKeyHex = normalizePublicKey(info.publicKeyHex) else {
            return kind == "identity_sync" ? "" : content
        }

        let listeners = Array(normalizeOutboundListenerHints(await getListeningAddresses()).prefix(3))
        let externalAddresses = Array(normalizeExternalAddressHints(await getExternalAddresses()).prefix(3))
        let connectionHints = (listeners + externalAddresses).reduce(into: [String]()) { acc, addr in
            if !acc.contains(addr) { acc.append(addr) }
        }

        let sender: [String: Any] = [
            "identity_id": info.identityId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "",
            "public_key": publicKeyHex,
            "device_id": info.deviceId ?? "",
            "nickname": String((normalizeNickname(info.nickname) ?? "").prefix(64)),
            "libp2p_peer_id": info.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "",
            "listeners": listeners,
            "external_addresses": externalAddresses,
            "connection_hints": connectionHints
        ]
        let payload: [String: Any] = [
            "schema": "scm.message.identity.v1",
            "kind": kind,
            "text": content,
            "sender": sender
        ]
        guard JSONSerialization.isValidJSONObject(payload),
              let data = try? JSONSerialization.data(withJSONObject: payload),
              let encoded = String(data: data, encoding: .utf8) else {
            return kind == "identity_sync" ? "" : content
        }
        return encoded
    }

    private func decodeMessageWithIdentityHints(_ raw: String) -> DecodedMessagePayload {
        guard let data = raw.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              (json["schema"] as? String) == "scm.message.identity.v1" else {
            return DecodedMessagePayload(kind: "text", text: raw, hints: nil)
        }

        let kind = ((json["kind"] as? String) ?? "text")
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty ?? "text"
        let text = (json["text"] as? String) ?? raw
        let sender = json["sender"] as? [String: Any]
        let hintedLibp2p = (sender?["libp2p_peer_id"] as? String)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        let validatedLibp2p = hintedLibp2p.flatMap { candidate in
            isLibp2pPeerId(candidate) ? candidate : nil
        }
        let hints = MessageIdentityHints(
            identityId: (sender?["identity_id"] as? String)?
                .trimmingCharacters(in: .whitespacesAndNewlines)
                .nilIfEmpty,
            publicKey: normalizePublicKey(sender?["public_key"] as? String),
            deviceId: (sender?["device_id"] as? String)?
                .trimmingCharacters(in: .whitespacesAndNewlines)
                .nilIfEmpty,
            nickname: normalizeNickname(sender?["nickname"] as? String),
            libp2pPeerId: validatedLibp2p,
            listeners: parseStringArray(sender?["listeners"]),
            externalAddresses: parseStringArray(sender?["external_addresses"]),
            connectionHints: parseStringArray(sender?["connection_hints"])
        )
        return DecodedMessagePayload(kind: kind, text: text, hints: hints)
    }

    private func parseStringArray(_ value: Any?) -> [String] {
        guard let array = value as? [Any] else { return [] }
        return array
            .compactMap { item in
                if let str = item as? String {
                    let trimmed = str.trimmingCharacters(in: .whitespacesAndNewlines)
                    return trimmed.isEmpty ? nil : trimmed
                }
                return nil
            }
            .reduce(into: [String]()) { acc, item in
                if !acc.contains(item) { acc.append(item) }
            }
    }

    private func normalizePublicKey(_ key: String?) -> String? {
        guard let value = key?.trimmingCharacters(in: .whitespacesAndNewlines),
              value.count == 64 else {
            return nil
        }
        let validHex = value.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
        guard validHex else { return nil }
        return value.lowercased()
    }

    private func normalizeNickname(_ nickname: String?) -> String? {
        let normalized = nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return normalized.isEmpty ? nil : normalized
    }

    private func isSyntheticFallbackNickname(_ nickname: String?) -> Bool {
        guard let normalized = normalizeNickname(nickname)?.lowercased() else { return false }
        // "peer-xxxxxx" is a receiver-side placeholder and should never overwrite
        // a sender-provided nickname.
        return normalized.hasPrefix("peer-")
    }

    private func selectAuthoritativeNickname(incoming: String?, existing: String?) -> String? {
        let incomingNormalized = normalizeNickname(incoming)
        let existingNormalized = normalizeNickname(existing)

        let incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        let existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        if incomingNormalized == nil && existingSynthetic { return nil }
        if incomingNormalized == nil { return existingNormalized }
        if incomingSynthetic && existingNormalized == nil { return nil }
        if incomingSynthetic && existingSynthetic { return nil }
        if incomingSynthetic { return existingNormalized }
        if existingSynthetic { return incomingNormalized }
        return incomingNormalized
    }

    private func isBlePeerId(_ value: String) -> Bool {
        UUID(uuidString: value.trimmingCharacters(in: .whitespacesAndNewlines)) != nil
    }

    private func selectCanonicalPeerId(incoming: String, existing: String) -> String {
        let incomingId = incoming.trimmingCharacters(in: .whitespacesAndNewlines)
        let existingId = existing.trimmingCharacters(in: .whitespacesAndNewlines)
        if incomingId.isEmpty { return existingId }
        if existingId.isEmpty || existingId == incomingId { return incomingId }

        let incomingIsLibp2p = isLibp2pPeerId(incomingId)
        let existingIsLibp2p = isLibp2pPeerId(existingId)
        let incomingIsIdentity = isIdentityId(incomingId)
        let existingIsIdentity = isIdentityId(existingId)
        let incomingIsBle = isBlePeerId(incomingId)
        let existingIsBle = isBlePeerId(existingId)

        if existingIsIdentity && incomingIsLibp2p { return existingId }
        if incomingIsIdentity && existingIsLibp2p { return incomingId }
        if existingIsBle && !incomingIsBle { return incomingId }
        if !existingIsBle && incomingIsBle { return existingId }
        return incomingId
    }

    private func prepopulateDiscoveryNickname(
        nickname: String?,
        peerId: String,
        publicKey: String?
    ) -> String? {
        let incomingNickname = normalizeNickname(nickname)

        let normalizedKey = normalizePublicKey(publicKey)
        let contacts = (try? contactManager?.list()) ?? []
        let fromContact = contacts.first(where: {
            $0.peerId == peerId ||
            (
                normalizedKey != nil &&
                normalizePublicKey($0.publicKey) == normalizedKey
            )
        })?.nickname
        return selectAuthoritativeNickname(incoming: incomingNickname, existing: fromContact)
    }

    private func resolveKnownPeerNickname(
        canonicalPeerId: String,
        routePeerId: String?,
        publicKey: String?
    ) -> String? {
        let normalizedKey = normalizePublicKey(publicKey)
        let routeCandidate = routePeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty

        let fromDiscovery = discoveredPeerMap.values
            .first(where: { info in
                info.canonicalPeerId == canonicalPeerId ||
                (routeCandidate != nil && info.canonicalPeerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(info.publicKey) == normalizedKey
                )
            })?
            .nickname

        let fromLedger = (ledgerManager?.dialableAddresses() ?? [])
            .first(where: { entry in
                entry.peerId == canonicalPeerId ||
                (routeCandidate != nil && entry.peerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(entry.publicKey) == normalizedKey
                )
            })?
            .nickname

        let fromContact = ((try? contactManager?.list()) ?? [])
            .first(where: { contact in
                contact.peerId == canonicalPeerId ||
                (routeCandidate != nil && contact.peerId == routeCandidate) ||
                (
                    normalizedKey != nil &&
                    normalizePublicKey(contact.publicKey) == normalizedKey
                )
            })?
            .nickname

        let discoveryOrLedger = selectAuthoritativeNickname(incoming: fromDiscovery, existing: fromLedger)
        return selectAuthoritativeNickname(incoming: discoveryOrLedger, existing: fromContact)
    }

    private func updateDiscoveredPeer(_ key: String, info: PeerDiscoveryInfo) {
        let normalizedKey = PeerIdValidator.normalize(key)
        let existing = discoveredPeerMap[normalizedKey]
        if let existing,
           existing.publicKey != nil,
           info.publicKey == nil,
           info.lastSeen >= existing.lastSeen,
           info.lastSeen - existing.lastSeen < 300 {
            return
        }
        let merged: PeerDiscoveryInfo
        if let existing {
            merged = PeerDiscoveryInfo(
                canonicalPeerId: selectCanonicalPeerId(
                    incoming: info.canonicalPeerId,
                    existing: existing.canonicalPeerId
                ),
                publicKey: info.publicKey ?? existing.publicKey,
                nickname: selectAuthoritativeNickname(incoming: info.nickname, existing: existing.nickname),
                transport: (info.transport == .internet || existing.transport == .internet) ? .internet : info.transport,
                isFull: info.isFull || existing.isFull,
                isRelay: info.isRelay || existing.isRelay,
                lastSeen: max(info.lastSeen, existing.lastSeen)
            )
        } else {
            merged = info
        }

        let canonicalPeerId = PeerIdValidator.normalize(
            merged.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                ? key
                : merged.canonicalPeerId
        )
        let canonicalPublicKey = normalizePublicKey(merged.publicKey)

        discoveredPeerMap[canonicalPeerId] = merged
        discoveredPeerMap = discoveredPeerMap.filter { mapKey, candidate in
            if mapKey == canonicalPeerId { return true }
            let sameCanonicalPeerId = candidate.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines) == canonicalPeerId
            let samePublicKey = canonicalPublicKey != nil &&
                normalizePublicKey(candidate.publicKey) == canonicalPublicKey
            return !(sameCanonicalPeerId || samePublicKey)
        }
    }

    private func pruneDisconnectedPeer(_ peerId: String) {
        let normalizedPeerId = PeerIdValidator.normalize(peerId)
        guard !normalizedPeerId.isEmpty else { return }

        let disconnectedPublicKey = normalizePublicKey(discoveredPeerMap[normalizedPeerId]?.publicKey)
        discoveredPeerMap = discoveredPeerMap.filter { key, info in
            if key == normalizedPeerId { return false }
            if info.canonicalPeerId == normalizedPeerId { return false }
            if let disconnectedPublicKey,
               normalizePublicKey(info.publicKey) == disconnectedPublicKey {
                return false
            }
            return true
        }
    }

    private func annotateIdentityInLedger(
        routePeerId: String?,
        listeners: [String],
        publicKey: String?,
        nickname: String?
    ) {
        guard let routePeerId = routePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
              !routePeerId.isEmpty,
              isLibp2pPeerId(routePeerId) else {
            return
        }

        let dialHints = buildDialCandidatesForPeer(
            routePeerId: routePeerId,
            rawAddresses: listeners,
            includeRelayCircuits: true
        )
        guard !dialHints.isEmpty else { return }

        let normalizedKey = normalizePublicKey(publicKey)
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty

        for multiaddr in dialHints {
            ledgerManager?.annotateIdentity(
                multiaddr: multiaddr,
                peerId: routePeerId,
                publicKey: normalizedKey,
                nickname: normalizedNickname
            )
        }
    }

    func getDialHintsForRoutePeer(_ routePeerId: String) -> [String] {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard isLibp2pPeerId(normalizedRoute) else { return [] }

        let fromLedger = (ledgerManager?.dialableAddresses() ?? [])
            .filter { $0.peerId == normalizedRoute }
            .map { $0.multiaddr }
        return buildDialCandidatesForPeer(
            routePeerId: normalizedRoute,
            rawAddresses: fromLedger,
            includeRelayCircuits: true
        )
    }

    func replayDiscoveredPeerEvents() {
        guard !discoveredPeerMap.isEmpty else { return }

        var aggregates: [String: ReplayDiscoveredIdentity] = [:]

        for (mapKey, info) in discoveredPeerMap {
            let canonicalPeerId = info.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !canonicalPeerId.isEmpty else { continue }

            let normalizedKey = normalizePublicKey(info.publicKey)
            let aggregateKey = normalizedKey ?? canonicalPeerId
            let routeCandidate: String? = {
                if isLibp2pPeerId(mapKey) { return mapKey }
                if isLibp2pPeerId(canonicalPeerId) { return canonicalPeerId }
                return nil
            }()
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: info.nickname,
                peerId: canonicalPeerId,
                publicKey: normalizedKey
            )

            if var existing = aggregates[aggregateKey] {
                if existing.publicKey == nil, let normalizedKey { existing.publicKey = normalizedKey }
                existing.canonicalPeerId = selectCanonicalPeerId(
                    incoming: canonicalPeerId,
                    existing: existing.canonicalPeerId
                )
                existing.nickname = selectAuthoritativeNickname(
                    incoming: discoveredNickname,
                    existing: existing.nickname
                )
                if existing.routePeerId?.isEmpty ?? true, let routeCandidate { existing.routePeerId = routeCandidate }
                if existing.transport != .internet, info.transport == .internet {
                    existing.transport = info.transport
                }
                if info.isRelay { existing.isRelay = true }
                aggregates[aggregateKey] = existing
            } else {
                aggregates[aggregateKey] = ReplayDiscoveredIdentity(
                    canonicalPeerId: canonicalPeerId,
                    publicKey: normalizedKey,
                    nickname: discoveredNickname,
                    routePeerId: routeCandidate,
                    transport: info.transport,
                    isRelay: info.isRelay
                )
            }
        }

        for peer in aggregates.values {
            let listeners = peer.routePeerId.map(getDialHintsForRoutePeer(_:)) ?? []
            if let publicKey = peer.publicKey, !publicKey.isEmpty {
                emitIdentityDiscoveredIfChanged(
                    peerId: peer.canonicalPeerId,
                    publicKey: publicKey,
                    nickname: peer.nickname,
                    libp2pPeerId: peer.routePeerId,
                    listeners: listeners
                )
            } else {
                MeshEventBus.shared.peerEvents.send(.discovered(peerId: peer.canonicalPeerId))
            }
        }
    }

    private func appendRoutingHint(notes: String?, key: String, value: String?) -> String? {
        guard let value = value?.trimmingCharacters(in: .whitespacesAndNewlines), !value.isEmpty else {
            return notes
        }

        let existing = notes?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        var segments = existing.split(whereSeparator: { $0 == ";" || $0 == "\n" }).map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
        let newEntry = "\(key):\(value)"

        // Replace existing entry for this key instead of appending a duplicate.
        // This prevents stale BLE MACs / route peer IDs from accumulating.
        if let existingIndex = segments.firstIndex(where: { $0.hasPrefix("\(key):") }) {
            if segments[existingIndex] == newEntry { return notes } // unchanged
            logger.info("Routing hint update: replacing old \(segments[existingIndex]) with \(newEntry)")
            segments[existingIndex] = newEntry
        } else {
            segments.append(newEntry)
        }

        let merged = segments.filter { !$0.isEmpty }.joined(separator: ";")
        return merged.isEmpty ? nil : merged
    }

    private func removeRoutingHint(notes: String?, key: String) -> String? {
        guard let notes else { return nil }
        let filtered = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty && !$0.hasPrefix("\(key):") }
        let merged = filtered.joined(separator: ";")
        return merged.isEmpty ? nil : merged
    }

    private func currentNotificationSettings() -> MeshSettings {
        if let loaded = try? settingsManager?.load() {
            return loaded
        }
        if let defaults = settingsManager?.defaultSettings() {
            return defaults
        }
        return MeshSettings(
            relayEnabled: true,
            maxRelayBudget: DefaultSettings.maxRelayBudget,
            batteryFloor: DefaultSettings.batteryFloor,
            bleEnabled: true,
            wifiAwareEnabled: false,
            wifiDirectEnabled: false,
            internetEnabled: true,
            discoveryMode: .normal,
            onionRouting: false,
            coverTrafficEnabled: false,
            messagePaddingEnabled: false,
            timingObfuscationEnabled: false,
            notificationsEnabled: true,
            notifyDmEnabled: true,
            notifyDmRequestEnabled: true,
            notifyDmInForeground: false,
            notifyDmRequestInForeground: true,
            soundEnabled: true,
            badgeEnabled: true,
            requirePq: true
        )
    }

    private func isNotificationRequestPending(notes: String?) -> Bool {
        guard let notes else { return false }
        return notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .contains { $0 == "\(NotificationNoteKey.requestPending):true" }
    }

    private func clearNotificationRequestPending(peerId: String) {
        guard let existingContact = try? contactManager?.get(peerId: peerId),
              isNotificationRequestPending(notes: existingContact.notes) else {
            return
        }

        let updated = Contact(
            peerId: existingContact.peerId,
            nickname: existingContact.nickname,
            localNickname: existingContact.localNickname,
            publicKey: existingContact.publicKey,
            addedAt: existingContact.addedAt,
            lastSeen: existingContact.lastSeen,
            notes: removeRoutingHint(notes: existingContact.notes, key: NotificationNoteKey.requestPending),
            lastKnownDeviceId: existingContact.lastKnownDeviceId,
            verifiedAt: existingContact.verifiedAt,
            isTombstone: existingContact.isTombstone
        )
        try? contactManager?.add(contact: updated)
        contactManager?.flush()
        checkpointContactPersistence(reason: "notification_contact_updated")
    }

    private func displayNameForContact(_ contact: Contact?, fallbackPeerId: String) -> String {
        let localNickname = contact?.localNickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        if let localNickname {
            return localNickname
        }

        let nickname = contact?.nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        if let nickname {
            return nickname
        }

        let normalizedPeerId = fallbackPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        if normalizedPeerId.count > 8 {
            return String(normalizedPeerId.prefix(8)) + "..."
        }
        return normalizedPeerId
    }

    private func effectiveTimestamp(for message: MessageRecord) -> UInt64 {
        message.senderTimestamp > 0 ? message.senderTimestamp : message.timestamp
    }

    private func compareMessageRecency(_ lhs: MessageRecord, _ rhs: MessageRecord) -> Bool {
        let lhsTimestamp = effectiveTimestamp(for: lhs)
        let rhsTimestamp = effectiveTimestamp(for: rhs)
        if lhsTimestamp == rhsTimestamp {
            return lhs.timestamp < rhs.timestamp
        }
        return lhsTimestamp < rhsTimestamp
    }

    private func resolveTransportIdentity(libp2pPeerId: String) -> TransportIdentityResolution? {
        guard isLibp2pPeerId(libp2pPeerId) else { return nil }
        let extractedKey: String? = {
            guard let core = ironCore else { return nil }
            return try? core.extractPublicKeyFromPeerId(peerId: libp2pPeerId)
        }()
        guard let extractedKey,
              let normalizedKey = normalizePublicKey(extractedKey) else {
            return nil
        }

        let selfKey = normalizePublicKey(ironCore?.getIdentityInfo().publicKeyHex)
        if selfKey == normalizedKey { return nil }

        // UNIFIED ID FIX: canonicalPeerId is ALWAYS public_key_hex for contact storage.
        guard let contacts = try? contactManager?.list() else {
            return TransportIdentityResolution(
                canonicalPeerId: normalizedKey,
                publicKey: normalizedKey,
                nickname: nil
            )
        }

        let keyMatches = contacts.filter { normalizePublicKey($0.publicKey) == normalizedKey }
        if keyMatches.count > 1 {
            logger.warning("Multiple contacts share transport key \(normalizedKey.prefix(8))...; using explicit route match where possible")
        }

        let routeLinked = keyMatches.first {
            $0.peerId == libp2pPeerId || parseRoutingHintsFromNotes($0.notes).libp2pPeerId == libp2pPeerId
        }
        let canonicalContact = routeLinked ?? keyMatches.first

        return TransportIdentityResolution(
            canonicalPeerId: canonicalContact?.peerId ?? normalizedKey,
            publicKey: normalizedKey,
            nickname: canonicalContact?.nickname
        )
    }

    // MARK: - Settings Management

    func loadSettings() throws -> MeshSettings {
        guard let settingsManager = settingsManager else {
            throw MeshError.notInitialized("SettingsManager not initialized")
        }
        return try settingsManager.load()
    }

    func saveSettings(_ settings: MeshSettings) throws {
        guard let settingsManager = settingsManager else {
            throw MeshError.notInitialized("SettingsManager not initialized")
        }
        try settingsManager.save(settings: settings)
        if !settings.notificationsEnabled {
            NotificationManager.shared.clearBadge()
        }
        logger.info("[OK] Settings saved (relay: \(settings.relayEnabled))")
    }

    func saveAndApplyTransportSettings(_ settings: MeshSettings) async throws {
        guard let settingsManager else {
            throw MeshError.notInitialized("SettingsManager not initialized")
        }

        let previous = (try? settingsManager.load()) ?? settingsManager.defaultSettings()
        try saveSettings(settings)

        let actions = Self.liveTransportActions(
            serviceRunning: serviceState == .running,
            previousBleEnabled: previous.bleEnabled,
            nextBleEnabled: settings.bleEnabled,
            previousInternetEnabled: previous.internetEnabled,
            nextInternetEnabled: settings.internetEnabled
        )

        for action in actions {
            switch action {
            case .startBle:
                startBleTransport()
            case .stopBle:
                stopBleTransport()
            case .startInternet:
                try startInternetTransport(reason: "settings")
            case .stopInternet:
                try stopInternetTransport()
            }
        }
    }

    func validateSettings(_ settings: MeshSettings) -> Bool {
        // Delegate to Rust-side validation via UniFFI for consistency with Android
        guard let settingsManager = settingsManager else {
            logger.error("SettingsManager not initialized; cannot validate settings")
            return false
        }

        do {
            try settingsManager.validate(settings: settings)
            return true
        } catch {
            logger.warning("Settings validation failed: \(String(describing: error))")
            return false
        }
    }

    // MARK: - Ledger Management

    func recordConnection(multiaddr: String, peerId: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        ledgerManager.recordConnection(multiaddr: multiaddr, peerId: peerId)
    }

    func recordConnectionFailure(multiaddr: String) throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        ledgerManager.recordFailure(multiaddr: multiaddr)
    }

    func getDialableAddresses() throws -> [LedgerEntry] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.dialableAddresses()
    }

    /// Returns bounded, deduplicated bootstrap candidates from the ledger.
    /// Preferred relays lead the list, followed by any other proven dialable
    /// addresses. No platform-owned fallback address is injected.
    private func ledgerBootstrapEntries(maxCount: UInt32 = 10) -> [LedgerEntry] {
        guard let ledgerManager else { return [] }

        let preferred = ledgerManager.getPreferredRelays(limit: maxCount)
        let dialable = ledgerManager.dialableAddresses()
        var seen = Set<String>()
        var result: [LedgerEntry] = []

        for entry in preferred + dialable {
            let address = entry.multiaddr.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !address.isEmpty, seen.insert(address).inserted else { continue }
            result.append(entry)
            if result.count >= Int(maxCount) { break }
        }
        return result
    }

    private func ledgerBootstrapAddresses(maxCount: UInt32 = 10) -> [String] {
        ledgerBootstrapEntries(maxCount: maxCount).map(\.multiaddr)
    }

    func getAllKnownTopics() throws -> [String] {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.allKnownTopics()
    }

    func getLedgerSummary() throws -> String {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        return ledgerManager.summary()
    }

    func getConnectionPathState() -> ConnectionPathState {
        return meshService?.getConnectionPathState() ?? .disconnected
    }

    func getNatStatus() -> String {
        return meshService?.getNatStatus() ?? "unknown"
    }

    func deliveryStatePresentation(for message: MessageRecord, nowEpochSec: UInt64 = UInt64(Date().timeIntervalSince1970)) -> DeliveryStatePresentation {
        if let pending = loadPendingOutbox().first(where: { $0.historyRecordId == message.id }) {
            if let terminalFailureCode = pending.terminalFailureCode {
                let detail: String
                switch terminalFailureCode {
                case "identity_device_mismatch":
                    detail = "Rejected because this identity moved to another device. Refresh the contact before retrying."
                case "identity_abandoned":
                    detail = "Rejected because the contact abandoned this identity. Re-verify the contact before sending again."
                default:
                    detail = "Rejected because the recipient identity is no longer valid."
                }
                return DeliveryStatePresentation(
                    label: "rejected",
                    detail: detail
                )
            }
            if pending.nextAttemptAtEpochSec <= nowEpochSec {
                return DeliveryStatePresentation(
                    label: "forwarding",
                    detail: "Actively retrying through direct or relay paths."
                )
            }
            return DeliveryStatePresentation(
                label: "stored",
                detail: "Stored for retry while the recipient is offline or unreachable."
            )
        }

        if message.delivered {
            return DeliveryStatePresentation(
                label: "delivered",
                detail: "Delivery receipt confirmed by the recipient node."
            )
        }

        return DeliveryStatePresentation(
            label: "pending",
            detail: "Queued locally. First route attempt is still in progress."
        )
    }

    func exportDiagnostics() -> String {
        // Return cached if fresh
        if Date().timeIntervalSince(diagnosticsCacheTime) < diagnosticsCacheTTL,
           !cachedDiagnostics.isEmpty {
            return cachedDiagnostics
        }
        // For main thread calls, return stale cache and trigger async refresh
        Task { await exportDiagnosticsAsync() }
        return cachedDiagnostics
    }

    func exportDiagnosticsAsync() async -> String {
        return await Task.detached(priority: .utility) { [weak self] in
            await self?.exportDiagnosticsInternal() ?? ""
        }.value
    }

    private func exportDiagnosticsInternal() -> String {
        let multipeerStats = multipeerTransport?.diagnosticsSnapshot()
        if let diagnostics = meshService?.exportDiagnostics(),
           !diagnostics.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let data = diagnostics.data(using: .utf8),
               var object = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] {
                object["relay_availability_state"] = relayAvailabilityState.rawValue
                object["relay_availability_updated_at_ms"] = Int(relayAvailabilityUpdatedAt.timeIntervalSince1970 * 1000)
                object["relay_recent_events_60s"] = relayRecentEventTimes.count
                if let relayLastDisconnectAt {
                    object["relay_last_disconnect_at_ms"] = Int(relayLastDisconnectAt.timeIntervalSince1970 * 1000)
                }
                object["relay_backoff_until_ms"] = Int(relayBackoffUntil.timeIntervalSince1970 * 1000)
                object["strict_ble_only_validation"] = strictBleOnlyValidation
                if let multipeerStats {
                    object["multipeer_connected_peers"] = multipeerStats.connectedPeers
                    object["multipeer_connecting_peers"] = multipeerStats.connectingPeers
                    object["multipeer_invites_in_flight"] = multipeerStats.inviteInFlight
                    object["multipeer_invite_timeouts"] = multipeerStats.inviteTimeoutCount
                    object["multipeer_invite_declines"] = multipeerStats.inviteDeclineCount
                    object["multipeer_effective_medium_estimate"] = multipeerStats.effectiveMediumEstimate
                }
                if let mergedData = try? JSONSerialization.data(withJSONObject: object),
                   let merged = String(data: mergedData, encoding: .utf8) {
                    // Update cache
                    self.cachedDiagnostics = merged
                    self.diagnosticsCacheTime = Date()
                    return merged
                }
            }
            // Update cache
            self.cachedDiagnostics = diagnostics
            self.diagnosticsCacheTime = Date()
            return diagnostics
        }

        var fallback: [String: Any] = [
            "service_state": String(describing: serviceState),
            "connection_path_state": String(describing: getConnectionPathState()),
            "nat_status": getNatStatus(),
            "discovered_peers": discoveredPeerMap.count,
            "pending_outbox": loadPendingOutbox().count,
            "relay_availability_state": relayAvailabilityState.rawValue,
            "relay_availability_updated_at_ms": Int(relayAvailabilityUpdatedAt.timeIntervalSince1970 * 1000),
            "relay_recent_events_60s": relayRecentEventTimes.count,
            "relay_backoff_until_ms": Int(relayBackoffUntil.timeIntervalSince1970 * 1000),
            "strict_ble_only_validation": strictBleOnlyValidation,
            "generated_at_ms": Int(Date().timeIntervalSince1970 * 1000)
        ]
        if let multipeerStats {
            fallback["multipeer_connected_peers"] = multipeerStats.connectedPeers
            fallback["multipeer_connecting_peers"] = multipeerStats.connectingPeers
            fallback["multipeer_invites_in_flight"] = multipeerStats.inviteInFlight
            fallback["multipeer_invite_timeouts"] = multipeerStats.inviteTimeoutCount
            fallback["multipeer_invite_declines"] = multipeerStats.inviteDeclineCount
            fallback["multipeer_effective_medium_estimate"] = multipeerStats.effectiveMediumEstimate
        }
        guard let data = try? JSONSerialization.data(withJSONObject: fallback),
              let json = String(data: data, encoding: .utf8) else {
            return "{}"
        }
        // Update cache
        self.cachedDiagnostics = json
        self.diagnosticsCacheTime = Date()
        return json
    }

    func saveLedger() throws {
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.save()
    }

    // MARK: - Contacts Management

    func getContacts() throws -> [Contact] {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.list()
    }

    func getContact(peerId: String) throws -> Contact? {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.get(peerId: peerId)
    }

    func displayNameForPeer(peerId: String) -> String {
        let contact = try? contactManager?.get(peerId: peerId)
        return displayNameForContact(contact, fallbackPeerId: peerId)
    }

    func addContact(_ contact: Contact) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }

        // UNIFIED ID FIX: Canonicalize peerId to public_key_hex before storage.
        let canonicalPeerId: String
        do {
            if let resolved = try ironCore?.resolveIdentity(anyId: contact.peerId) {
                canonicalPeerId = resolved
            } else {
                canonicalPeerId = contact.peerId
            }
        } catch {
            canonicalPeerId = contact.peerId
        }

        let finalContact: Contact
        if canonicalPeerId != contact.peerId {
            finalContact = Contact(
                peerId: canonicalPeerId,
                nickname: contact.nickname,
                localNickname: contact.localNickname,
                publicKey: contact.publicKey,
                addedAt: contact.addedAt,
                lastSeen: contact.lastSeen,
                notes: contact.notes,
                lastKnownDeviceId: contact.lastKnownDeviceId,
                verifiedAt: contact.verifiedAt,
                isTombstone: contact.isTombstone
            )
        } else {
            finalContact = contact
        }

        try contactManager.add(contact: finalContact)
        let routing = parseRoutingHintsFromNotes(finalContact.notes)
        annotateIdentityInLedger(
            routePeerId: routing.libp2pPeerId,
            listeners: routing.listeners,
            publicKey: finalContact.publicKey,
            nickname: finalContact.nickname
        )
        checkpointContactPersistence(reason: "contact_added")
        logger.info("[OK] Contact added: \(finalContact.peerId)")
    }

    func removeContact(peerId: String) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.remove(peerId: peerId)
        try? historyManager?.removeConversation(peerId: peerId)
        checkpointContactPersistence(reason: "contact_removed")
        logger.info("[OK] Contact removed: \(peerId) and their message history")
    }

    func searchContacts(query: String) throws -> [Contact] {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return try contactManager.search(query: query)
    }

    func setContactNickname(peerId: String, nickname: String?) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        try contactManager.setNickname(peerId: peerId, nickname: normalizedNickname)
        checkpointContactPersistence(reason: "contact_nickname_updated")
        logger.info("[OK] Contact nickname updated: \(peerId)")
    }

    func setLocalNickname(peerId: String, nickname: String?) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        let normalizedNickname = nickname?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        try contactManager.setLocalNickname(peerId: peerId, nickname: normalizedNickname)
        checkpointContactPersistence(reason: "contact_local_nickname_updated")
        logger.info("[OK] Local nickname updated: \(peerId)")
    }

    func getContactCount() throws -> UInt32 {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        return contactManager.count()
    }

    /// Mark a contact as verified after an out-of-band safety-number comparison.
    func markContactVerified(peerId: String) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.markVerified(peerId: peerId)
        checkpointContactPersistence(reason: "contact_verified")
        logger.info("[OK] Contact marked verified: \(peerId)")
    }

    /// Clear a contact's verification status (e.g. after a key change).
    func unverifyContact(peerId: String) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.unverify(peerId: peerId)
        checkpointContactPersistence(reason: "contact_verification_cleared")
        logger.info("[OK] Contact verification cleared: \(peerId)")
    }

    /// Compute the Signal-style safety number for comparing identities with
    /// `theirPublicKeyHex` out-of-band. Returns nil if our own identity isn't
    /// initialized yet, or an empty string if the underlying keys are
    /// malformed (S5) - callers must treat both as error states, never
    /// render an empty string as if it were a real safety number.
    func computeSafetyNumber(theirPublicKeyHex: String) -> String? {
        guard let ourKey = getIdentityInfo()?.publicKeyHex else { return nil }
        return safetyNumber(ourPubkeyHex: ourKey, theirPubkeyHex: theirPublicKeyHex)
    }

    // MARK: - Message History

    func getConversation(peerId: String, limit: UInt32 = 100) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.conversation(peerId: peerId, limit: limit)
    }

    func getMessageRequests(limit: UInt32 = 100) -> [MessageRequestThread] {
        let contacts = (try? contactManager?.list()) ?? []
        let pendingContacts = contacts.filter { isNotificationRequestPending(notes: $0.notes) }

        return pendingContacts.compactMap { contact in
            let messages = (try? historyManager?.conversation(peerId: contact.peerId, limit: limit)) ?? []
            let lastMessage = messages.max(by: compareMessageRecency(_:_:))
            return MessageRequestThread(
                peerId: contact.peerId,
                displayName: displayNameForContact(contact, fallbackPeerId: contact.peerId),
                previewText: lastMessage?.content,
                lastMessageTime: lastMessage.map { Date(timeIntervalSince1970: Double(effectiveTimestamp(for: $0))) },
                unreadCount: Int(messages.filter { $0.direction == .received }.count)
            )
        }
        .sorted { ($0.lastMessageTime ?? .distantPast) > ($1.lastMessageTime ?? .distantPast) }
    }

    func getRecentMessages(peerIdFilter: String? = nil, limit: UInt32 = 50) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.recent(peerFilter: peerIdFilter, limit: limit)
    }

    func getMessage(id: String) throws -> MessageRecord? {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.get(id: id)
    }

    func addMessage(record: MessageRecord) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.add(record: record)
    }

    func searchMessages(query: String, limit: UInt32 = 50) throws -> [MessageRecord] {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.search(query: query, limit: limit)
    }

    func markMessageDelivered(id: String) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.markDelivered(id: id)
        removePendingOutbound(historyRecordId: id)
    }

    func clearHistory() throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.clear()
        logger.info("[OK] Message history cleared")
    }

    func clearConversation(peerId: String) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.clearConversation(peerId: peerId)
        logger.info("[OK] Conversation cleared for peer: \(peerId)")
    }

    func acceptMessageRequest(peerId: String) throws {
        clearNotificationRequestPending(peerId: peerId)
        NotificationManager.shared.markConversationRead(conversationId: peerId)
    }

    func notificationSettingsEnabled() -> Bool {
        currentNotificationSettings().notificationsEnabled
    }

    func setNotificationAppInForeground(_ inForeground: Bool) {
        notificationAppInForeground = inForeground
    }

    func setNotificationActiveConversation(peerId: String?) {
        notificationActiveConversationId = peerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
        if let activeConversationId = notificationActiveConversationId {
            NotificationManager.shared.markConversationRead(conversationId: activeConversationId)
        }
    }

    func getHistoryStats() throws -> HistoryStats? {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return try historyManager.stats()
    }

    func getMessageCount() throws -> UInt32 {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        return historyManager.count()
    }

    // MARK: - Blocking

    /// Block a peer by ID with an optional reason.
    func blockPeer(peerId: String, reason: String? = nil) throws {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        try ironCore.blockPeer(peerId: peerId, deviceId: nil, reason: reason)
        logger.info("[OK] Blocked peer: \(peerId)")
    }

    /// Unblock a previously blocked peer.
    func unblockPeer(peerId: String) throws {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        try ironCore.unblockPeer(peerId: peerId, deviceId: nil)
        logger.info("[OK] Unblocked peer: \(peerId)")
    }

    /// Block a peer AND delete all their stored messages (cascade purge).
    /// Future payloads from this peer are dropped at the ingress layer.
    func blockAndDeletePeer(peerId: String, reason: String? = nil) throws {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        try ironCore.blockAndDeletePeer(peerId: peerId, deviceId: nil, reason: reason)
        logger.info("[OK] Blocked and deleted peer: \(peerId)")
    }

    /// Check whether a peer is currently blocked.
    func isPeerBlocked(peerId: String) throws -> Bool {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.isPeerBlocked(peerId: peerId, deviceId: nil)
    }

    /// List all blocked peers.
    func listBlockedPeers() throws -> [BlockedIdentity] {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.listBlockedPeers()
    }

    /// Get the count of blocked peers.
    func blockedCount() throws -> UInt32 {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.blockedCount()
    }

    // MARK: - Crypto Utilities

    /// Sign arbitrary data with the local identity key.
    func signData(data: Data) throws -> SignatureResult {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.signData(data: data)
    }

    /// Verify a signature against data and a public key.
    func verifySignature(data: Data, signature: Data, publicKeyHex: String) throws -> Bool {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.verifySignature(data: data, signature: signature, publicKeyHex: publicKeyHex)
    }

    // MARK: - WS13 Device Management

    /// Get the local device ID (WS13).
    func getDeviceId() -> String? {
        return ironCore?.getDeviceId()
    }

    /// Get the seniority timestamp for this installation (WS13).
    func getSeniorityTimestamp() -> UInt64? {
        return ironCore?.getSeniorityTimestamp()
    }

    /// Get the registration state for a given identity (WS13).
    func getRegistrationState(identityId: String) -> RegistrationStateInfo? {
        return ironCore?.getRegistrationState(identityId: identityId)
    }

    // MARK: - Logging

    /// Export all recorded log entries as a single string.
    func exportLogs() throws -> String {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.exportLogs()
    }

    // MARK: - Queue Counts

    /// Get the number of messages in the outbox queue.
    func outboxCount() -> UInt32 {
        return ironCore?.outboxCount() ?? 0
    }

    /// Get the number of messages in the inbox queue.
    func inboxCount() -> UInt32 {
        return ironCore?.inboxCount() ?? 0
    }

    // MARK: - Contact Device ID (WS13)

    /// Update the last known device ID for a contact (WS13.2).
    func updateContactDeviceId(peerId: String, deviceId: String?) throws {
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        try contactManager.updateDeviceId(peerId: peerId, deviceId: deviceId)
        checkpointContactPersistence(reason: "contact_device_id_updated")
    }

    // MARK: - History Retention

    /// Delete a single message by ID.
    func deleteMessage(id: String) throws {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        try historyManager.delete(id: id)
        logger.info("[OK] Deleted message: \(id)")
    }

    /// Enforce message retention by keeping only the newest N messages.
    /// Returns the number of messages pruned.
    func enforceRetention(maxMessages: UInt32) throws -> UInt32 {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        let pruned = try historyManager.enforceRetention(maxMessages: maxMessages)
        logger.info("[OK] Retention enforced: kept \(maxMessages), pruned \(pruned)")
        return pruned
    }

    /// Prune messages older than the given Unix timestamp.
    /// Returns the number of messages pruned.
    func pruneBefore(timestamp: UInt64) throws -> UInt32 {
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }
        let pruned = try historyManager.pruneBefore(beforeTimestamp: timestamp)
        logger.info("[OK] Pruned \(pruned) messages before timestamp \(timestamp)")
        return pruned
    }

    /// Resolve any identifier format to the canonical public key hex.
    func resolveIdentity(anyId: String) throws -> String {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.resolveIdentity(anyId: anyId)
    }

    // MARK: - Platform Reporting

    func reportBattery(pct: UInt8, charging: Bool) {
        logger.debug("Battery: \(pct)% charging=\(charging)")
        currentBatteryPct = pct
        currentIsCharging = charging

        // 1. Report to Rust MeshService
        let profile = DeviceProfile(
            peerId: nil,
            deviceId: nil,
            batteryPct: pct,
            isCharging: charging,
            hasWifi: networkStatus.wifi,
            motionState: currentMotionState
        )
        meshService?.updateDeviceState(profile: profile)
        Task { @MainActor [weak self] in
            await self?.applyPowerAdjustments(reason: "battery_changed")
        }
    }

    func reportNetwork(wifi: Bool, cellular: Bool) {
        let previousWifi = networkStatus.wifi
        logger.debug("Network: wifi=\(wifi) cellular=\(cellular)")
        networkStatus.wifi = wifi
        networkStatus.cellular = cellular

        // Report to Rust
        let profile = DeviceProfile(
            peerId: nil,
            deviceId: nil,
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: wifi,
            motionState: currentMotionState
        )
        meshService?.updateDeviceState(profile: profile)
        Task { @MainActor [weak self] in
            await self?.applyPowerAdjustments(reason: "network_changed")
        }
        broadcastIdentityBeacon()

        // When WiFi comes back, immediately try to deliver pending messages
        if wifi && !previousWifi {
            logger.info("WiFi recovered — flushing pending outbox")
            appendDiagnostic("network_recovery wifi=true flush_triggered=true")
            Task { @MainActor [weak self] in
                await self?.primeRelayBootstrapConnections()
            }
            dispatchFlushPendingOutbox(reason: "wifi_recovered")
        }
    }

    func reportMotion(state: MotionState) {
        logger.debug("Motion: \(state)")
        currentMotionState = state

        // Report to Rust
        let profile = DeviceProfile(
            peerId: nil,
            deviceId: nil,
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: networkStatus.wifi,
            motionState: state
        )
        meshService?.updateDeviceState(profile: profile)
        Task { @MainActor [weak self] in
            await self?.applyPowerAdjustments(reason: "motion_changed")
        }
    }

    func setAutoAdjustEnabled(_ enabled: Bool) {
        UserDefaults.standard.set(enabled, forKey: "auto_adjust_enabled")
        logger.info("AutoAdjust toggled: \(enabled)")
        Task { @MainActor [weak self] in
            await self?.applyPowerAdjustments(reason: "settings_toggled")
        }
    }

    // MARK: - Background Operations

    func onEnteringBackground() {
        logger.info("Repository: entering background")
        pauseMeshService()
        do {
            try saveLedger()
        } catch {
            logger.warning("Failed to save ledger on background transition: \(error.localizedDescription)")
        }
    }

    func onEnteringForeground() {
        logger.info("Repository: entering foreground")
        resumeMeshService()
        updateStats()
    }

    func pauseService() {
        pauseMeshService()
    }

    func syncPendingMessages() async throws {
        logger.info("Syncing pending messages")
        // Pending outbox data is managed by Rust core. The best available
        // sync trigger here is to refresh transport connectivity so queued
        // messages can be retried by the core.
        if serviceState != .running {
            try ensureServiceInitialized()
        }

        let contacts = (try? contactManager?.list()) ?? []
        for contact in contacts {
            guard let notes = contact.notes,
                  let routing = parseRoutingInfo(notes: notes),
                  !routing.addresses.isEmpty else {
                continue
            }
            await connectToPeer(routing.libp2pPeerId, addresses: routing.addresses)
        }

        await flushPendingOutbox(reason: "sync_pending")

        updateStats()
    }

    func updateStats() {
        logger.info("Updating stats")
        if let service = meshService {
            serviceStats = service.getStats()
            if let stats = serviceStats {
                statusEvents.send(.statsUpdated(stats))
            }
        }
    }

    func resetServiceStats() {
        logger.info("Resetting mesh service stats")
        meshService?.resetStats()
        updateStats()
    }

    func quickPeerDiscovery() async throws {
        logger.info("Quick peer discovery")
        if serviceState != .running {
            try ensureServiceInitialized()
        }

        if (try? loadSettings().bleEnabled) != false {
            startBleTransport()
        } else {
            stopBleTransport()
        }
        multipeerTransport?.startAdvertising()
        multipeerTransport?.startBrowsing()
        updateStats()
    }

    func performBulkSync() async throws {
        logger.info("Performing bulk sync")
        try await syncPendingMessages()
        try await quickPeerDiscovery()
        try await updatePeerLedger()
    }

    func cleanupOldMessages() async throws {
        logger.info("Cleaning up old messages")
        guard let contactManager = contactManager else {
            throw MeshError.notInitialized("ContactManager not initialized")
        }
        guard let historyManager = historyManager else {
            throw MeshError.notInitialized("HistoryManager not initialized")
        }

        // Retention policy: clear conversations for peers not seen in 180 days.
        let staleThresholdSeconds: UInt64 = 180 * 24 * 60 * 60
        let now = UInt64(Date().timeIntervalSince1970)

        for contact in try contactManager.list() {
            guard let lastSeen = contact.lastSeen else { continue }
            if now > lastSeen && (now - lastSeen) > staleThresholdSeconds {
                try? historyManager.removeConversation(peerId: contact.peerId)
            }
        }
    }

    func updatePeerLedger() async throws {
        logger.info("Updating peer ledger")
        guard let ledgerManager = ledgerManager else {
            throw MeshError.notInitialized("LedgerManager not initialized")
        }
        try ledgerManager.save()
    }

    /// Handle libp2p transport identity updates from the Rust core.
    /// Transport peer IDs are route hints, not user/contact identities.
    func handleTransportPeerDiscovered(peerId: String) {
        _ = PeerIdValidator.normalize(peerId)
        let selfLibp2p = ironCore?.getIdentityInfo().libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if let selfLibp2p, !selfLibp2p.isEmpty, selfLibp2p == peerId {
            logger.debug("Ignoring self transport discovery: \(peerId)")
            return
        }

        let isRelay = isBootstrapRelayPeer(peerId)

        if let transportIdentity = resolveTransportIdentity(libp2pPeerId: peerId) {
            let discoveredNickname = prepopulateDiscoveryNickname(
                nickname: transportIdentity.nickname,
                peerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey
            )
            let relayHints = buildDialCandidatesForPeer(
                routePeerId: peerId,
                rawAddresses: [],
                includeRelayCircuits: true
            )
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname,
                transport: .internet,
                isFull: true,
                isRelay: isRelay,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(peerId, info: discoveryInfo)
            if transportIdentity.canonicalPeerId != peerId {
                updateDiscoveredPeer(transportIdentity.canonicalPeerId, info: discoveryInfo)
            }
            emitIdentityDiscoveredIfChanged(
                peerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname,
                libp2pPeerId: peerId,
                listeners: relayHints
            )
            annotateIdentityInLedger(
                routePeerId: peerId,
                listeners: relayHints,
                publicKey: transportIdentity.publicKey,
                nickname: discoveredNickname
            )
            persistRouteHintsForTransportPeer(
                libp2pPeerId: peerId,
                listeners: relayHints,
                knownPublicKey: transportIdentity.publicKey
            )
            upsertFederatedContact(
                canonicalPeerId: transportIdentity.canonicalPeerId,
                publicKey: transportIdentity.publicKey,
                nickname: transportIdentity.nickname,
                libp2pPeerId: peerId,
                listeners: relayHints,
                createIfMissing: false
            )
            try? contactManager?.updateLastSeen(peerId: transportIdentity.canonicalPeerId)
            try? contactManager?.updateLastSeen(peerId: peerId)
            scheduleContactPersistenceCheckpoint(reason: "transport_contact_last_seen")
            if !relayHints.isEmpty {
                Task { @MainActor [weak self] in
                    await self?.connectToPeer(peerId, addresses: relayHints)
                }
            }
        } else {
            let discoveryInfo = PeerDiscoveryInfo(
                canonicalPeerId: peerId,
                publicKey: nil,
                nickname: nil,
                transport: .internet,
                isFull: false,
                isRelay: isRelay,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(peerId, info: discoveryInfo)
        }

        // Ensure UI (and dashboard) sees the discovery immediately
        MeshEventBus.shared.peerEvents.send(.discovered(peerId: peerId))
    }

    func handleTransportPeerDisconnected(peerId: String) {
        // P1: Deduplicate disconnect events — Rust core fires one per substream
        let trimmedId = PeerIdValidator.normalize(peerId)
        let now = Date()
        if let lastDisconnect = peerDisconnectDedupCache[trimmedId],
           now.timeIntervalSince(lastDisconnect) < peerDisconnectDedupInterval {
            return // Already processed this disconnect within the window
        }
        peerDisconnectDedupCache[trimmedId] = now

        if isKnownRelay(trimmedId) || isBootstrapRelayPeer(trimmedId) {
            updateRelayAvailability(peerId: trimmedId, event: "disconnected")
        }
        connectedEmissionCache.removeValue(forKey: trimmedId)
        mdnsLanPeers.removeValue(forKey: trimmedId)
        pruneDisconnectedPeer(peerId)
    }

    func handleTransportPeerIdentified(peerId: String, agentVersion: String, listenAddrs: [String]) {
        // P0: Deduplicate peer-identified events — Rust core fires one per substream,
        // producing 34K+ duplicate events. Skip if same peer identified within 30s.
        let trimmedPeerId = PeerIdValidator.normalize(peerId)
        let identifySignature = ([agentVersion.trimmingCharacters(in: .whitespacesAndNewlines)] +
            listenAddrs
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
            .sorted()).joined(separator: "|")
        let now = Date()
        if let lastIdentified = peerIdentifiedDedupCache[trimmedPeerId],
           lastIdentified.signature == identifySignature,
           now.timeIntervalSince(lastIdentified.observedAt) < peerIdentifiedDedupInterval {
            return
        }
        peerIdentifiedDedupCache[trimmedPeerId] = (identifySignature, now)

        appendDiagnostic("peer_identified transport=\(peerId) agent=\(agentVersion) addrs=\(listenAddrs.count)")
        let resetSuffix = "/p2p/\(peerId)"
        for key in dialThrottleState.keys where key.hasSuffix(resetSuffix) || key == peerId {
            dialThrottleState.removeValue(forKey: key)
        }
        let selfLibp2p = ironCore?.getIdentityInfo().libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        if let selfLibp2p, !selfLibp2p.isEmpty, selfLibp2p == peerId {
            logger.debug("Ignoring self transport identity: \(peerId)")
            return
        }

        // TCP/mDNS parity: Detect LAN addresses (RFC1918) from listen_addrs.
        // If any private-network TCP/QUIC address is present, this peer
        // was discovered on the local network (typically via libp2p mDNS).
        let lanAddrs = listenAddrs.filter { addr in
            let a = addr.trimmingCharacters(in: .whitespacesAndNewlines)
            let isPrivateIp: Bool = {
                if a.hasPrefix("/ip4/192.168.") || a.hasPrefix("/ip4/10.") {
                    return true
                }
                if a.hasPrefix("/ip4/172.") {
                    let parts = a.dropFirst("/ip4/".count).split(separator: ".")
                    if parts.count >= 2, let octet = Int(parts[1]) {
                        return (16...31).contains(octet)
                    }
                }
                return false
            }()
            return isPrivateIp && (a.contains("/tcp/") || a.contains("/udp/"))
        }
        if !lanAddrs.isEmpty {
            mdnsLanPeers[trimmedPeerId] = lanAddrs
            logger.info("TCP/mDNS: LAN peer detected \(trimmedPeerId) with \(lanAddrs.count) local addresses")
        } else {
            mdnsLanPeers.removeValue(forKey: trimmedPeerId)
        }

        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: peerId,
            rawAddresses: listenAddrs,
            includeRelayCircuits: true
        )

        var syncPeerIds: [String] = [peerId]
        let isHeadless = agentVersion.contains("/headless/")
        let transportIdentity = resolveTransportIdentity(libp2pPeerId: peerId)
        let shouldTreatAsHeadless = isBootstrapRelayPeer(peerId) || (isHeadless && transportIdentity == nil)
        if shouldTreatAsHeadless {
            logger.info("Headless/Relay transport node identified: \(peerId) (agent: \(agentVersion))")
            updateRelayAvailability(peerId: trimmedPeerId, event: "identified")
            let relayDiscovery = PeerDiscoveryInfo(
                canonicalPeerId: peerId,
                publicKey: nil,
                nickname: nil,
                transport: .internet,
                isFull: false,
                isRelay: true,
                lastSeen: UInt64(Date().timeIntervalSince1970)
            )
            updateDiscoveredPeer(peerId, info: relayDiscovery)
            emitConnectedIfChanged(peerId: peerId)
        } else {
            if isHeadless, transportIdentity != nil {
                logger.info("Promoting peer \(peerId) to full node: identity resolved despite headless agent \(agentVersion)")
            }
            if let transportIdentity {
                syncPeerIds.append(transportIdentity.canonicalPeerId)
                let discoveredNickname = prepopulateDiscoveryNickname(
                    nickname: transportIdentity.nickname,
                    peerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey
                )
                let discoveryInfo = PeerDiscoveryInfo(
                    canonicalPeerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname,
                    transport: mdnsLanPeers[trimmedPeerId] != nil ? .tcpMdns : .internet,
                    isFull: true,
                    isRelay: isBootstrapRelayPeer(peerId),
                    lastSeen: UInt64(Date().timeIntervalSince1970)
                )
                updateDiscoveredPeer(peerId, info: discoveryInfo)
                if transportIdentity.canonicalPeerId != peerId {
                    updateDiscoveredPeer(transportIdentity.canonicalPeerId, info: discoveryInfo)
                }
                emitIdentityDiscoveredIfChanged(
                    peerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname,
                    libp2pPeerId: peerId,
                    listeners: dialCandidates
                )
                annotateIdentityInLedger(
                    routePeerId: peerId,
                    listeners: dialCandidates,
                    publicKey: transportIdentity.publicKey,
                    nickname: discoveredNickname
                )
                try? contactManager?.updateLastSeen(peerId: transportIdentity.canonicalPeerId)
                try? contactManager?.updateLastSeen(peerId: peerId)
                scheduleContactPersistenceCheckpoint(reason: "transport_identity_last_seen")
            } else {
                let discoveryInfo = PeerDiscoveryInfo(
                    canonicalPeerId: peerId,
                    publicKey: nil,
                    nickname: nil,
                    transport: .internet,
                    isFull: false,
                    isRelay: isBootstrapRelayPeer(peerId),
                    lastSeen: UInt64(Date().timeIntervalSince1970)
                )
                updateDiscoveredPeer(peerId, info: discoveryInfo)
                logger.debug("Transport identity unavailable for \(peerId)")
            }
            emitConnectedIfChanged(peerId: peerId)
            persistRouteHintsForTransportPeer(
                libp2pPeerId: peerId,
                listeners: dialCandidates,
                knownPublicKey: transportIdentity?.publicKey
            )
            if let transportIdentity {
                upsertFederatedContact(
                    canonicalPeerId: transportIdentity.canonicalPeerId,
                    publicKey: transportIdentity.publicKey,
                    nickname: transportIdentity.nickname,
                    libp2pPeerId: peerId,
                    listeners: dialCandidates,
                    createIfMissing: false
                )
            }
            sendIdentitySyncIfNeeded(routePeerId: peerId, knownPublicKey: transportIdentity?.publicKey)
            sendHistorySyncIfNeeded(routePeerId: peerId, knownPublicKey: transportIdentity?.publicKey)
        }

        // Identified implies an active session already exists; avoid re-dial loops here.
        triggerPendingSyncForPeerIds(syncPeerIds, reason: "peer_identified:\(peerId)")
        scheduleIdentityBeaconRefresh(reason: "peer_identified")
    }

    // MARK: - BLE Transport Integration

    func onMultipeerDataReceived(peerId: String, data: Data) {
        logger.debug("Multipeer data from \(peerId): \(data.count) bytes")
        // Forward to MeshService using the same ingress path as BLE.
        meshService?.onDataReceived(peerId: peerId, data: data)
    }

    func onBleDataReceived(peerId: String, data: Data) {
        logger.debug("BLE data from \(peerId): \(data.count) bytes")
        // Forward to MeshService
        meshService?.onDataReceived(peerId: peerId, data: data)
    }

    func sendBlePacket(peerId: String, data: Data) {
        logger.debug("Send BLE packet to \(peerId): \(data.count) bytes")

        // Direct packet to appropriate manager based on UUID match
        // Note: peerId here is likely the UUID from the transport layer if Rust is treating it as a handle
        let sendTargets = ([peerId] + (bleCentralManager?.connectedPeripheralIds() ?? []) + (blePeripheralManager?.subscribedCentralIds() ?? []))
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
            .reduce(into: [String]()) { deduped, next in
                if !deduped.contains(next) {
                    deduped.append(next)
                }
            }
        if sendTargets.isEmpty {
            logger.warning("sendBlePacket: no BLE targets available for \(peerId)")
            return
        }

        var accepted = false
        for target in sendTargets {
            if let blePeripheralManager = blePeripheralManager, !blePeripheralManager.subscribedCentralIds().isEmpty {
                if blePeripheralManager.sendDataToConnectedCentral(peerId: target, data: data) == true {
                    accepted = true
                    break
                }
            } else {
                appendDiagnostic("ble_peripheral_send_skip_no_subscribers")
            }

            if let uuid = UUID(uuidString: target), bleCentralManager?.sendData(to: uuid, data: data) == true {
                accepted = true
                break
            }
        }
        if !accepted {
            logger.warning("sendBlePacket: no BLE transport accepted payload for requested \(peerId)")
        }
    }

    /// Called when BLE central reads the identity GATT characteristic from a peer.
    /// Automatically creates a contact if none exists for this peer.
    func onPeerIdentityRead(blePeerId: String, info: [String: Any]) {
        guard let publicKeyHex = info["public_key"] as? String else { return }
        let rawNickname = ((info["nickname"] as? String) ?? (info["name"] as? String) ?? "")
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let libp2pPeerId = info["libp2p_peer_id"] as? String
        let listeners = info["listeners"] as? [String]
        let externalAddresses = info["external_addresses"] as? [String]
        let connectionHints = info["connection_hints"] as? [String]

        let idFromInfo = (info["identity_id"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
        let identityId = (idFromInfo?.isEmpty == false) ? (idFromInfo ?? blePeerId) : blePeerId

        let discoveredNickname = prepopulateDiscoveryNickname(
            nickname: rawNickname,
            peerId: identityId,
            publicKey: publicKeyHex
        )
        logger.info(
            "Peer BLE identity read: \(blePeerId.prefix(8)) key: \(publicKeyHex.prefix(8))... identity=\(identityId) nickname='\((discoveredNickname ?? "").prefix(24))'"
        )
        guard let normalizedKey = normalizePublicKey(publicKeyHex) else {
            let trimmed = publicKeyHex.trimmingCharacters(in: .whitespacesAndNewlines)
            logger.warning("Ignoring BLE identity from \(blePeerId.prefix(8)): invalid key (\(trimmed.count) chars)")
            return
        }
        let selfIdentity = ironCore?.getIdentityInfo()
        let selfKey = normalizePublicKey(selfIdentity?.publicKeyHex)
        let selfIdentityId = selfIdentity?.identityId?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let selfLibp2pPeerId = selfIdentity?.libp2pPeerId?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let normalizedLibp2p: String? = {
            guard let raw = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                  !raw.isEmpty else { return nil }
            return raw
        }()
        if (selfKey != nil && selfKey == normalizedKey) ||
            (!selfIdentityId.isEmpty && selfIdentityId == identityId) ||
            (!selfLibp2pPeerId.isEmpty && selfLibp2pPeerId == normalizedLibp2p) {
            logger.debug("Ignoring self BLE identity beacon from \(blePeerId.prefix(8))")
            return
        }

        // Persist BLE -> Identity mapping in contact notes
        if let contact = try? contactManager?.get(peerId: identityId) {
            let updatedNotes = appendRoutingHint(notes: contact.notes, key: "ble_peer_id", value: blePeerId)
            if updatedNotes != contact.notes {
                let updatedContact = Contact(
                    peerId: contact.peerId,
                    nickname: contact.nickname,
                    localNickname: contact.localNickname,
                    publicKey: contact.publicKey,
                    addedAt: contact.addedAt,
                    lastSeen: UInt64(Date().timeIntervalSince1970),
                    notes: updatedNotes,
                    lastKnownDeviceId: contact.lastKnownDeviceId,
                    verifiedAt: contact.verifiedAt,
                    isTombstone: contact.isTombstone
                )
                try? contactManager?.add(contact: updatedContact)
                contactManager?.flush()
                checkpointContactPersistence(reason: "ble_routing_updated")
                logger.debug("Updated persistent BLE routing for \(identityId.prefix(8)): \(blePeerId.prefix(8))")
            }
        }

        // Emit to nearby peers bus — UI will show peer in Nearby section for user to manually add
        let nonEmptyNickname = rawNickname.isEmpty ? nil : rawNickname
        let nonEmptyLibp2p = normalizedLibp2p
        triggerPendingSyncForPeerIds(
            [identityId, blePeerId, nonEmptyLibp2p].compactMap { $0 },
            reason: "peer_discovered"
        )
        let mergedHints = (listeners ?? []) + (externalAddresses ?? []) + (connectionHints ?? [])
        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: nonEmptyLibp2p,
            rawAddresses: mergedHints,
            includeRelayCircuits: true
        )
        let discoveryInfo = PeerDiscoveryInfo(
            canonicalPeerId: identityId,
            publicKey: normalizedKey,
            nickname: discoveredNickname,
            transport: .ble,
            isFull: true,
            isRelay: false,
            lastSeen: UInt64(Date().timeIntervalSince1970)
        )
        updateDiscoveredPeer(identityId, info: discoveryInfo)
        if let nonEmptyLibp2p, nonEmptyLibp2p != identityId {
            updateDiscoveredPeer(nonEmptyLibp2p, info: discoveryInfo)
        }
        // Remove the preliminary BLE-UUID entry (isFull=false) created at connection time.
        // Now that identity is confirmed, the blePeerId key is a stale duplicate.
        if blePeerId != identityId && blePeerId != normalizedLibp2p {
            discoveredPeerMap = discoveredPeerMap.filter { key, value in
                key != blePeerId && value.canonicalPeerId != blePeerId
            }
            logger.debug("Removed preliminary BLE entry \(blePeerId.prefix(8)) → promoted to \(identityId)")
        }
        emitIdentityDiscoveredIfChanged(
            peerId: identityId,
            publicKey: normalizedKey,
            nickname: discoveredNickname,
            libp2pPeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            blePeerId: blePeerId
        )
        annotateIdentityInLedger(
            routePeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            publicKey: normalizedKey,
            nickname: discoveredNickname
        )
        logger.info("Emitted identityDiscovered for \(blePeerId.prefix(8)) key: \(normalizedKey.prefix(8))...")
        // Trigger history sync over BLE when we discover a peer's identity
        if let nonEmptyLibp2p {
            sendHistorySyncIfNeeded(routePeerId: nonEmptyLibp2p, knownPublicKey: normalizedKey)
        } else {
            sendHistorySyncIfNeeded(routePeerId: identityId, knownPublicKey: normalizedKey)
        }
        // Update lastSeen if already a saved contact
        try? contactManager?.updateLastSeen(peerId: blePeerId)
        try? contactManager?.updateLastSeen(peerId: identityId)
        if let libp2pPeerId, !libp2pPeerId.isEmpty {
            try? contactManager?.updateLastSeen(peerId: libp2pPeerId)
        }
        scheduleContactPersistenceCheckpoint(reason: "ble_contact_last_seen")
        upsertFederatedContact(
            canonicalPeerId: normalizedKey,       // UNIFIED ID FIX: canonical = public_key_hex
            publicKey: normalizedKey,
            nickname: nonEmptyNickname,
            libp2pPeerId: nonEmptyLibp2p,
            listeners: dialCandidates,
            blePeerId: blePeerId,
            createIfMissing: false
        )

        // Auto-dial discovered peer via Swarm if we have libp2p info
        if let peerId = nonEmptyLibp2p, !dialCandidates.isEmpty {
            logger.info("Auto-dialing discovered peer over Swarm: \(peerId)")
            Task { @MainActor [weak self] in
                await self?.connectToPeer(peerId, addresses: dialCandidates)
            }
            triggerPendingSyncForPeerIds(
                [identityId, blePeerId, peerId],
                reason: "peer_identity_read"
            )
        }
    }

    private func parseRoutingInfo(notes: String) -> (libp2pPeerId: String, addresses: [String])? {
        let segments = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0) }
        var libp2pPeerId: String?
        var addresses: [String] = []

        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.hasPrefix("libp2p_peer_id:") {
                let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty { libp2pPeerId = value }
            } else if trimmed.hasPrefix("listeners:") {
                let value = trimmed.replacingOccurrences(of: "listeners:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty {
                    addresses = value
                        .split(separator: ",")
                        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                        .filter { !$0.isEmpty }
                }
            }
        }

        guard let peerId = libp2pPeerId else { return nil }
        return (peerId, addresses)
    }

    private func emitIdentityDiscoveredIfChanged(
        peerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: [String],
        blePeerId: String? = nil
    ) {
        let canonicalPeerId = normalizePeerId(peerId)
        guard !canonicalPeerId.isEmpty,
              let normalizedKey = normalizePublicKey(publicKey) else {
            return
        }

        let normalizedNickname = normalizeNickname(nickname)
        let normalizedRoute: String? = {
            guard let routeId = libp2pPeerId else { return nil }
            let normalized = normalizePeerId(routeId)
            return normalized.isEmpty ? nil : normalized
        }()
        let normalizedBle = {
            let trimmed = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            return trimmed.isEmpty ? nil : trimmed
        }()
        let normalizedListeners = Array(
            Set(
                listeners
                    .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                    .filter { !$0.isEmpty }
            )
        ).sorted()

        let signature = IdentityEmissionSignature(
            canonicalPeerId: canonicalPeerId,
            publicKey: normalizedKey,
            nickname: normalizedNickname,
            libp2pPeerId: normalizedRoute,
            blePeerId: normalizedBle
        )
        let cacheKey = "\(canonicalPeerId)|\(normalizedKey)"
        let now = Date()
        if let previous = identityEmissionCache[cacheKey],
           previous.signature == signature,
           now.timeIntervalSince(previous.emittedAt) < identityReemitInterval {
            return
        }
        identityEmissionCache[cacheKey] = (signature, now)

        MeshEventBus.shared.peerEvents.send(.identityDiscovered(
            peerId: canonicalPeerId,
            publicKey: normalizedKey,
            nickname: normalizedNickname,
            libp2pPeerId: normalizedRoute,
            listeners: normalizedListeners,
            blePeerId: normalizedBle
        ))
    }

    private func emitConnectedIfChanged(peerId: String) {
        let normalizedPeerId = PeerIdValidator.normalize(peerId)
        guard !normalizedPeerId.isEmpty else { return }

        let now = Date()
        if let previous = connectedEmissionCache[normalizedPeerId],
           now.timeIntervalSince(previous) < connectedReemitInterval {
            return
        }
        connectedEmissionCache[normalizedPeerId] = now
        MeshEventBus.shared.peerEvents.send(.connected(peerId: normalizedPeerId))
        if !isBootstrapRelayPeer(normalizedPeerId) {
            triggerPendingSyncForPeerIds([normalizedPeerId], reason: "peer_connected:\(normalizedPeerId)")
        }
    }

    private func parseRoutingHintsFromNotes(_ notes: String?) -> RoutingHints {
        guard let notes else { return RoutingHints(libp2pPeerId: nil, listeners: [], multipeerPeerId: nil, blePeerId: nil) }
        let segments = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0) }
        var libp2pPeerId: String?
        var listeners: [String] = []
        var multipeerPeerId: String?
        var blePeerId: String?

        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.hasPrefix("libp2p_peer_id:") {
                let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty { libp2pPeerId = value }
            } else if trimmed.hasPrefix("multipeer_peer_id:") {
                let value = trimmed.replacingOccurrences(of: "multipeer_peer_id:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty { multipeerPeerId = value }
            } else if trimmed.hasPrefix("ble_peer_id:") {
                let value = trimmed.replacingOccurrences(of: "ble_peer_id:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty { blePeerId = value }
            } else if trimmed.hasPrefix("listeners:") {
                let value = trimmed.replacingOccurrences(of: "listeners:", with: "")
                    .trimmingCharacters(in: .whitespacesAndNewlines)
                if !value.isEmpty {
                    listeners = value
                        .split(separator: ",")
                        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                        .filter { !$0.isEmpty }
                }
            }
        }
        return RoutingHints(
            libp2pPeerId: libp2pPeerId,
            listeners: listeners,
            multipeerPeerId: multipeerPeerId,
            blePeerId: blePeerId
        )
    }

    private func defaultMultipeerPeerId(fromPublicKey publicKey: String?) -> String? {
        guard let normalizedKey = normalizePublicKey(publicKey), normalizedKey.count >= 8 else {
            return nil
        }
        return String(normalizedKey.prefix(8))
    }

    private func parseAllRoutingPeerIds(from notes: String?) -> [String] {
        guard let notes, !notes.isEmpty else { return [] }
        var out: [String] = []
        let segments = notes
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0) }

        for segment in segments {
            let trimmed = segment.trimmingCharacters(in: .whitespacesAndNewlines)
            guard trimmed.hasPrefix("libp2p_peer_id:") else { continue }
            let value = trimmed.replacingOccurrences(of: "libp2p_peer_id:", with: "")
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty, isLibp2pPeerId(value), !out.contains(value) {
                out.append(value)
            }
        }
        return out
    }

    private func buildRoutePeerCandidates(
        peerId: String,
        cachedRoutePeerId: String?,
        notes: String?,
        recipientPublicKey: String? = nil
    ) -> [String] {
        var candidates: [String] = []
        for discovered in discoverRoutePeersForPublicKey(recipientPublicKey) where !candidates.contains(discovered) {
            candidates.append(discovered)
        }
        let notedPeerIds = parseAllRoutingPeerIds(from: notes)
        if let newest = notedPeerIds.last, !newest.isEmpty {
            candidates.append(newest)
        }
        for hint in notedPeerIds.reversed() where !candidates.contains(hint) {
            candidates.append(hint)
        }
        if let cached = cachedRoutePeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
           !cached.isEmpty,
           !candidates.contains(cached) {
            candidates.append(cached)
        }
        if isLibp2pPeerId(peerId), !candidates.contains(peerId) {
            candidates.append(peerId)
        }
        return candidates.filter {
            isLibp2pPeerId($0) && routeCandidateMatchesRecipient($0, recipientPublicKey: recipientPublicKey)
        }
    }

    private func discoverRoutePeersForPublicKey(_ recipientPublicKey: String?) -> [String] {
        guard let normalizedRecipientKey = normalizePublicKey(recipientPublicKey) else { return [] }

        let fromDiscovery = discoveredPeerMap.values
            .compactMap { info -> String? in
                guard normalizePublicKey(info.publicKey) == normalizedRecipientKey else { return nil }
                let candidate = info.canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !candidate.isEmpty, isLibp2pPeerId(candidate) else { return nil }
                return candidate
            }

        let fromLedger = (ledgerManager?.dialableAddresses() ?? [])
            .compactMap { entry -> String? in
                guard let candidate = entry.peerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                      !candidate.isEmpty,
                      isLibp2pPeerId(candidate),
                      normalizePublicKey(entry.publicKey) == normalizedRecipientKey else {
                    return nil
                }
                return candidate
            }

        return (fromDiscovery + fromLedger).reduce(into: [String]()) { acc, next in
            if !acc.contains(next) { acc.append(next) }
        }
    }

    private func routeCandidateMatchesRecipient(_ routePeerId: String, recipientPublicKey: String?) -> Bool {
        let normalizedRoute = routePeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedRoute.isEmpty, isLibp2pPeerId(normalizedRoute) else { return false }
        guard !isKnownRelay(normalizedRoute) else { return false }

        guard let normalizedRecipientKey = normalizePublicKey(recipientPublicKey) else { return true }
        let extractedKey = try? ironCore?.extractPublicKeyFromPeerId(peerId: normalizedRoute)
        if let normalizedExtracted = normalizePublicKey(extractedKey) {
            return normalizedExtracted == normalizedRecipientKey
        }

        let discoveryMatch = discoveredPeerMap.contains { key, info in
            let keyMatches = key == normalizedRoute || info.canonicalPeerId == normalizedRoute
            return keyMatches && normalizePublicKey(info.publicKey) == normalizedRecipientKey
        }
        if discoveryMatch { return true }

        let ledgerMatch = (ledgerManager?.dialableAddresses() ?? []).contains { entry in
            entry.peerId?.trimmingCharacters(in: .whitespacesAndNewlines) == normalizedRoute &&
                normalizePublicKey(entry.publicKey) == normalizedRecipientKey
        }
        return ledgerMatch
    }

    private func isLibp2pPeerId(_ value: String) -> Bool {
        return PeerIdValidator.isLibp2pPeerId(value)
    }

    private func normalizePeerId(_ id: String) -> String {
        return PeerIdValidator.normalize(id)
    }

    private func isSamePeerId(_ id1: String, id2: String) -> Bool {
        return PeerIdValidator.isSame(id1, id2)
    }

    private func isIdentityId(_ value: String) -> Bool {
        return PeerIdValidator.isIdentityId(value)
    }

    private func startPendingOutboxRetryLoop() {
        guard pendingOutboxRetryTask == nil else { return }
        pendingOutboxRetryTask = Task { [weak self] in
            while !Task.isCancelled {
                // Proactively ensure we stay connected to relays
                await self?.primeRelayBootstrapConnections()

                await self?.flushPendingOutbox(reason: "periodic")
                try? await Task.sleep(nanoseconds: 8_000_000_000) // 8s loop
            }
        }
    }

    /// Starts a background loop that periodically broadcasts cover traffic when
    /// `coverTrafficEnabled` is true in settings. Broadcasts every 30 seconds.
    private func startCoverTrafficLoopIfEnabled() {
        guard coverTrafficTask == nil else { return }
        coverTrafficTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 30_000_000_000)
                guard let self, !Task.isCancelled else { break }
                let enabled = (try? self.settingsManager?.load())?.coverTrafficEnabled == true
                guard enabled else { continue }
                guard let core = self.ironCore, let bridge = self.swarmBridge else { continue }
                do {
                    let payload = try core.prepareCoverTraffic(sizeBytes: 256)
                    try await bridge.sendToAllPeers(data: payload)
                } catch {
                    self.logger.debug("Cover traffic send skipped: \(error.localizedDescription)")
                }
            }
        }
    }

    private func attemptDirectSwarmDelivery(
        routePeerCandidates: [String],
        addresses: [String],
        envelopeData: Data,
        multipeerPeerId: String? = nil,
        blePeerId: String? = nil,
        traceMessageId: String? = nil,
        attemptContext: String? = nil,
        strictBleOnlyOverride: Bool? = nil,
        recipientIdentityId: String? = nil,
        intendedDeviceId: String? = nil
    ) async -> DeliveryAttemptResult {
        let strictBleOnly = strictBleOnlyOverride ?? strictBleOnlyValidation
        let routePeerFallback = routePeerCandidates.first ?? "unknown_route_\(Date().timeIntervalSince1970)"
        if strictBleOnly {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "ble-only",
                phase: "mode",
                outcome: "enabled",
                detail: "ctx=" + (attemptContext ?? "") + ", route_candidates=" + String(routePeerCandidates.count)
            )
            if !(multipeerPeerId?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ?? true) {
                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "multipeer",
                    phase: "ble_only",
                    outcome: "blocked",
                    detail: "ctx=" + (attemptContext ?? "") + ", reason=strict_ble_only_mode"
                )
            }
            if !routePeerCandidates.isEmpty || !addresses.isEmpty {
                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "core",
                    phase: "ble_only",
                    outcome: "blocked",
                    detail: "ctx=" + (attemptContext ?? "") + ", reason=strict_ble_only_mode"
                )
            }
        }

        let connectedBlePeerIds = (
            (bleCentralManager?.connectedPeripheralIds() ?? []) +
            (blePeripheralManager?.subscribedCentralIds() ?? [])
        )
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
            .reduce(into: [String]()) { deduped, next in
                if !deduped.contains(next) {
                    deduped.append(next)
                }
            }
        let requestedBlePeerId = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines).nilIfEmpty
        let effectiveBlePeerId = connectedBlePeerIds.first ?? requestedBlePeerId
        if requestedBlePeerId == nil, let fallbackPeer = effectiveBlePeerId {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "ble",
                phase: "local_fallback",
                outcome: "target_fallback",
                detail: "ctx=\(attemptContext) target=\(fallbackPeer) reason=ble_peer_missing_connected_device_available"
            )
        } else if let requestedBlePeerId, let effectiveBlePeerId, requestedBlePeerId != effectiveBlePeerId {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "ble",
                phase: "local_fallback",
                outcome: "target_fallback",
                detail: "ctx=\(attemptContext) target=\(effectiveBlePeerId) requested_target=\(requestedBlePeerId) reason=prefer_connected_device"
            )
        }

        // Use SmartTransportRouter for intelligent transport selection with 500ms timeout fallback
        let smartResult: TransportDeliveryResult
        if let router = smartTransportRouter {
            smartResult = await router.attemptDelivery(
                peerId: routePeerFallback,
                envelopeData: envelopeData,
                multipeerPeerId: strictBleOnly ? nil : multipeerPeerId,
                blePeerId: effectiveBlePeerId,
                tcpMdnsPeerId: routePeerCandidates.first(where: { candidate in
                    let trimmed = candidate.trimmingCharacters(in: .whitespacesAndNewlines)
                    return !trimmed.isEmpty && (mdnsLanPeers[trimmed]?.isEmpty == false)
                }).map { $0.trimmingCharacters(in: .whitespacesAndNewlines) },
                routePeerCandidates: routePeerCandidates,
                addresses: addresses,
                traceMessageId: traceMessageId,
                attemptContext: attemptContext,
                tryMultipeer: { [self] multipeerAddr in
                    guard let transport = multipeerTransport else {
                        self.logger.debug("tryMultipeerDelivery: multipeerTransport is null")
                        return false
                    }
                    do {
                        try transport.sendData(toPeerId: multipeerAddr, data: envelopeData)
                        logger.info("[OK] Delivery via Multipeer (target=\(multipeerAddr))")
                        logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "multipeer",
                            phase: "smart_router",
                            outcome: "success",
                            detail: "ctx=\(attemptContext) target=\(multipeerAddr)"
                        )
                        return true
                    } catch {
                        logger.debug("tryMultipeerDelivery failed for \(multipeerAddr): \(error.localizedDescription)")
                        logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "multipeer",
                            phase: "smart_router",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) target=\(multipeerAddr) reason=\(String(describing: error.localizedDescription))"
                        )
                        return false
                    }
                },
                tryBle: { [self] bleAddr in
                    self.logger.debug("tryBleDelivery: given blePeerId=\(bleAddr)")
                    let sendTargets = ([bleAddr] + connectedBlePeerIds)
                        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                        .filter { !$0.isEmpty }
                        .reduce(into: [String]()) { deduped, next in
                            if !deduped.contains(next) {
                                deduped.append(next)
                            }
                        }
                    if sendTargets.isEmpty {
                        logger.debug("tryBleDelivery: no send targets available")
                        return false
                    }

                    var lastFailureReason = "no_target_attempted"
                    for target in sendTargets {
                        // Prefer Peripheral path first (notifications to subscribed Android central)
                        // This is the ONLY reliable iOS→Android path when WiFi is off
                        if let peripheral = blePeripheralManager {
                            if peripheral.sendDataToConnectedCentral(peerId: target, data: envelopeData) {
                                logDeliveryAttempt(
                                    messageId: traceMessageId,
                                    medium: "ble",
                                    phase: "smart_router",
                                    outcome: "accepted",
                                    detail: "ctx=\(attemptContext) role=peripheral requested_target=\(bleAddr) target=\(target)"
                                )
                                return true
                            }
                            lastFailureReason = "peripheral_send_false:\(target)"
                        }

                        // Fallback: Central path (write to Android's GATT server)
                        if let central = bleCentralManager {
                            if let uuid = UUID(uuidString: target) {
                                if central.sendData(to: uuid, data: envelopeData) {
                                    logger.info("[OK] Delivery via BLE Central (target=\(target))")
                                    logDeliveryAttempt(
                                        messageId: traceMessageId,
                                        medium: "ble",
                                        phase: "smart_router",
                                        outcome: "accepted",
                                        detail: "ctx=\(attemptContext) role=central requested_target=\(String(describing: bleAddr)) target=\(String(describing: target))"
                                    )
                                    return true
                                }
                                lastFailureReason = "central_send_false:\(target)"
                            } else {
                                lastFailureReason = "central_invalid_uuid:\(target)"
                            }
                        }
                    }

                    logDeliveryAttempt(
                        messageId: traceMessageId,
                        medium: "ble",
                        phase: "smart_router",
                        outcome: "failed",
                        detail: "ctx=\(attemptContext) requested_target=\(bleAddr) reason=\(lastFailureReason) connected=\(connectedBlePeerIds.count)"
                    )
                    return false
                },
                tryTcpMdns: { [self] lanPeerId in
                    // TCP/mDNS transport: Direct LAN delivery via libp2p TCP.
                    // This peer was discovered via mDNS and has LAN addresses — skip relay.
                    guard let swarmBridge = self.swarmBridge else {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "tcp_mdns",
                            phase: "smart_router",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) reason=swarm_bridge_unavailable"
                        )
                        return false
                    }

                    // Dial LAN addresses directly (no relay circuits)
                    let lanAddrs = self.mdnsLanPeers[lanPeerId] ?? []
                    if !lanAddrs.isEmpty {
                        let dialCandidates = self.buildDialCandidatesForPeer(
                            routePeerId: lanPeerId,
                            rawAddresses: lanAddrs,
                            includeRelayCircuits: false
                        )
                        if !dialCandidates.isEmpty {
                            await self.connectToPeer(lanPeerId, addresses: dialCandidates)
                            _ = await self.awaitPeerConnection(peerId: lanPeerId)
                        }
                    }

                    let sendError = await swarmBridge.sendMessageStatus(
                        peerId: lanPeerId,
                        data: envelopeData,
                        recipientIdentityId: recipientIdentityId,
                        intendedDeviceId: intendedDeviceId
                    )

                    if sendError == nil {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "tcp_mdns",
                            phase: "smart_router",
                            outcome: "success",
                            detail: "ctx=\(attemptContext) route=\(lanPeerId) lan_addrs=\(lanAddrs.count)"
                        )
                        return true
                    } else {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "tcp_mdns",
                            phase: "smart_router",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) route=\(lanPeerId) reason=\(sendError ?? "unknown")"
                        )
                        return false
                    }
                },
                tryCore: { [self] corePeerId in
                    // Core transport attempt (libp2p/internet relay)
                    guard let swarmBridge = self.swarmBridge else {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "core",
                            phase: "smart_router",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) reason=swarm_bridge_unavailable"
                        )
                        return false
                    }

                    let dialCandidates = self.buildDialCandidatesForPeer(
                        routePeerId: corePeerId,
                        rawAddresses: addresses,
                        includeRelayCircuits: true
                    )
                    if !dialCandidates.isEmpty {
                        await self.connectToPeer(corePeerId, addresses: dialCandidates)
                        _ = await self.awaitPeerConnection(peerId: corePeerId)
                    }

                    let sendError = await swarmBridge.sendMessageStatus(
                        peerId: corePeerId,
                        data: envelopeData,
                        recipientIdentityId: recipientIdentityId,
                        intendedDeviceId: intendedDeviceId
                    )

                    if sendError == nil {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "core",
                            phase: "smart_router",
                            outcome: "success",
                            detail: "ctx=\(attemptContext) route=\(corePeerId)"
                        )
                        return true
                    } else {
                        self.logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "core",
                            phase: "smart_router",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) route=\(String(describing: corePeerId)) reason=\(String(describing: sendError ?? "unknown"))"
                        )
                        return false
                    }
                }
            )
        } else {
            // Fallback to legacy LocalTransportFallback if router not available
            let localFallback = LocalTransportFallback.attemptMultipeerThenBle(
                multipeerPeerId: strictBleOnly ? nil : multipeerPeerId,
                blePeerId: effectiveBlePeerId,
                tryMultipeer: { multipeerAddr in
                    guard let transport = multipeerTransport else {
                        logger.debug("tryMultipeerDelivery: multipeerTransport is null")
                        return false
                    }
                    do {
                        try transport.sendData(toPeerId: multipeerAddr, data: envelopeData)
                        logger.info("[OK] Delivery via Multipeer (target=\(multipeerAddr))")
                        logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "multipeer",
                            phase: "local_fallback",
                            outcome: "success",
                            detail: "ctx=\(attemptContext) target=\(multipeerAddr)"
                        )
                        return true
                    } catch {
                        logger.debug("tryMultipeerDelivery failed for \(multipeerAddr): \(error.localizedDescription)")
                        logDeliveryAttempt(
                            messageId: traceMessageId,
                            medium: "multipeer",
                            phase: "local_fallback",
                            outcome: "failed",
                            detail: "ctx=\(attemptContext) target=\(multipeerAddr) reason=\(String(describing: error.localizedDescription))"
                        )
                        return false
                    }
                },
                tryBle: { bleAddr in
                    logger.debug("tryBleDelivery: given blePeerId=\(bleAddr)")
                    let sendTargets = ([bleAddr] + connectedBlePeerIds)
                        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                        .filter { !$0.isEmpty }
                        .reduce(into: [String]()) { deduped, next in
                            if !deduped.contains(next) {
                                deduped.append(next)
                            }
                        }
                    if sendTargets.isEmpty {
                        logger.debug("tryBleDelivery: no send targets available")
                        return false
                    }

                    var lastFailureReason = "no_target_attempted"
                    for target in sendTargets {
                        // Prefer Peripheral path first (notifications to subscribed Android central)
                        // This is the ONLY reliable iOS→Android path when WiFi is off
                        if let peripheral = blePeripheralManager {
                            if peripheral.sendDataToConnectedCentral(peerId: target, data: envelopeData) {
                                logDeliveryAttempt(
                                    messageId: traceMessageId,
                                    medium: "ble",
                                    phase: "local_fallback",
                                    outcome: "accepted",
                                    detail: "ctx=\(attemptContext) role=peripheral requested_target=\(bleAddr) target=\(target)"
                                )
                                return true
                            }
                            lastFailureReason = "peripheral_send_false:\(target)"
                        }

                        // Fallback: Central path (write to Android's GATT server)
                        if let central = bleCentralManager {
                            if let uuid = UUID(uuidString: target) {
                                if central.sendData(to: uuid, data: envelopeData) {
                                    logger.info("[OK] Delivery via BLE Central (target=\(target))")
                                    logDeliveryAttempt(
                                        messageId: traceMessageId,
                                        medium: "ble",
                                        phase: "local_fallback",
                                        outcome: "accepted",
                                        detail: "ctx=\(attemptContext) role=central requested_target=\(String(describing: bleAddr)) target=\(String(describing: target))"
                                    )
                                    return true
                                }
                                lastFailureReason = "central_send_false:\(target)"
                            } else {
                                lastFailureReason = "central_invalid_uuid:\(target)"
                            }
                        }
                    }

                    logDeliveryAttempt(
                        messageId: traceMessageId,
                        medium: "ble",
                        phase: "local_fallback",
                        outcome: "failed",
                        detail: "ctx=\(attemptContext) requested_target=\(bleAddr) reason=\(lastFailureReason) connected=\(connectedBlePeerIds.count)"
                    )
                    return false
                }
            )
            smartResult = TransportDeliveryResult(
                transport: localFallback.acked ? (localFallback.multipeerAcked ? .multipeer : .ble) : .internet,
                success: localFallback.acked,
                latencyMs: 0,
                error: localFallback.acked ? nil : "legacy_fallback_failed",
                timestamp: Date()
            )
        }

        let localAcked = smartResult.success
        if strictBleOnly {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "ble-only",
                phase: "aggregate",
                outcome: localAcked ? "accepted" : "failed",
                detail: "ctx=\(attemptContext) route_fallback=\(routePeerFallback)"
            )
            return DeliveryAttemptResult(
                acked: localAcked,
                routePeerId: routePeerFallback,
                terminalFailureCode: nil
            )
        }

        guard let swarmBridge else {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "core",
                phase: "direct",
                outcome: localAcked ? "skipped_local_accepted" : "failed",
                detail: "ctx=\(attemptContext) reason=swarm_bridge_unavailable route_fallback=\(routePeerFallback)"
            )
            return DeliveryAttemptResult(
                acked: localAcked,
                routePeerId: routePeerFallback,
                terminalFailureCode: nil
            )
        }

        let sanitizedCandidates = routePeerCandidates
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty && isLibp2pPeerId($0) }
            .reduce(into: [String]()) { acc, peer in
                if !acc.contains(peer) { acc.append(peer) }
            }

        guard !sanitizedCandidates.isEmpty else {
            logDeliveryAttempt(
                messageId: traceMessageId,
                medium: "core",
                phase: "direct",
                outcome: localAcked ? "skipped_local_accepted" : "failed",
                detail: "ctx=\(attemptContext) reason=no_route_candidates route_fallback=\(routePeerFallback)"
            )
            return DeliveryAttemptResult(
                acked: localAcked,
                routePeerId: routePeerFallback,
                terminalFailureCode: nil
            )
        }

        await primeRelayBootstrapConnections()

        for routePeerId in sanitizedCandidates {
            let dialCandidates = buildDialCandidatesForPeer(
                routePeerId: routePeerId,
                rawAddresses: addresses,
                includeRelayCircuits: true
            )
            if !dialCandidates.isEmpty {
                await connectToPeer(routePeerId, addresses: dialCandidates)
                _ = await awaitPeerConnection(peerId: routePeerId)
            }

            // Deduplication: if message was already acked via Multipeer or BLE,
            // skip the duplicate swarm send to avoid double notifications on recipient side.
            if !localAcked {
                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "core",
                    phase: "direct",
                    outcome: "attempt",
                    detail: "ctx=\(attemptContext) route=\(routePeerId)"
                )
                let sendError = await swarmBridge.sendMessageStatus(
                    peerId: routePeerId,
                    data: envelopeData,
                    recipientIdentityId: recipientIdentityId,
                    intendedDeviceId: intendedDeviceId
                )
                guard sendError == nil else {
                    logger.warning("Core-routed delivery failed for \(String(describing: routePeerId)): \(String(describing: sendError ?? "unknown")); trying alternative transports")
                    logDeliveryAttempt(
                        messageId: traceMessageId,
                        medium: "core",
                        phase: "direct",
                        outcome: "failed",
                        detail: "ctx=\(attemptContext) route=\(String(describing: routePeerId)) reason=\(String(describing: sendError ?? "unknown"))"
                    )
                    if isTerminalIdentityFailure(sendError) {
                        return DeliveryAttemptResult(
                            acked: false,
                            routePeerId: routePeerId,
                            terminalFailureCode: sendError
                        )
                    }
                    continue
                }
                logger.info("[OK] Direct delivery ACK from \(routePeerId)")
                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "core",
                    phase: "direct",
                    outcome: "success",
                    detail: "ctx=\(attemptContext) route=\(routePeerId)"
                )

                // Reset failure count on success (LOG-AUDIT-001 fix)
                consecutiveDeliveryFailures[routePeerId] = 0
                lastFailureTime.removeValue(forKey: routePeerId)

                return DeliveryAttemptResult(
                    acked: true,
                    routePeerId: routePeerId,
                    terminalFailureCode: nil
                )
            } else {
                logger.debug("Skipping redundant core send for \(traceMessageId ?? "unknown"): already delivered via local transport")
            }

            // Check circuit breaker before attempting relay-circuit (LOG-AUDIT-001 fix)
            let peerKey = routePeerId
            let failureCount = consecutiveDeliveryFailures[peerKey] ?? 0
            let lastFailure = lastFailureTime[peerKey]

            // Circuit breaker: if too many consecutive failures, pause retries
            if failureCount >= circuitBreakerThreshold {
                if let lastFailure = lastFailure,
                   Date().timeIntervalSince(lastFailure) < circuitBreakerDuration {
                    let remaining = circuitBreakerDuration - Date().timeIntervalSince(lastFailure)
                    logger.warning("Circuit breaker active for \(peerKey): \(failureCount) failures, retry in \(Int(remaining))s")
                    appendDiagnostic("delivery_circuit_breaker peer=\(peerKey) failures=\(failureCount) remaining_sec=\(Int(remaining))")
                    continue
                }
                // Reset after circuit breaker duration
                consecutiveDeliveryFailures[peerKey] = 0
            }

            let relayOnly = relayCircuitAddresses(for: routePeerId)
            if !relayOnly.isEmpty {
                await connectToPeer(routePeerId, addresses: relayOnly)
                _ = await awaitPeerConnection(peerId: routePeerId, timeoutMs: 3000)

                // Exponential backoff before relay attempt (1s → 2s → 4s → 8s → 16s → 32s max)
                let backoffExponent = min(failureCount, 5)
                let backoffSeconds = TimeInterval(1 << backoffExponent)
                logger.info("Relay-circuit backoff: \(backoffSeconds)s (failure count: \(failureCount))")
                try? await Task.sleep(nanoseconds: UInt64(backoffSeconds * 1_000_000_000))

                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "relay-circuit",
                    phase: "retry",
                    outcome: "attempt",
                    detail: "ctx=\(attemptContext) route=\(routePeerId)"
                )
                let sendError = await swarmBridge.sendMessageStatus(
                    peerId: routePeerId,
                    data: envelopeData,
                    recipientIdentityId: recipientIdentityId,
                    intendedDeviceId: intendedDeviceId
                )
                guard sendError == nil else {
                    logger.warning("Relay-circuit retry failed for \(routePeerId): \(sendError ?? "unknown")")
                    logDeliveryAttempt(
                        messageId: traceMessageId,
                        medium: "relay-circuit",
                        phase: "retry",
                        outcome: "failed",
                        detail: "ctx=\(attemptContext) route=\(routePeerId) reason=\(sendError ?? "unknown")"
                    )

                    // Track failure for circuit breaker (LOG-AUDIT-001 fix)
                    self.consecutiveDeliveryFailures[peerKey] = (self.consecutiveDeliveryFailures[peerKey] ?? 0) + 1
                    self.lastFailureTime[peerKey] = Date()
                    logger.info("Delivery failure tracked: peer=\(String(describing: peerKey)) consecutive=\(self.consecutiveDeliveryFailures[peerKey] ?? 0)")

                    if isTerminalIdentityFailure(sendError) {
                        return DeliveryAttemptResult(
                            acked: false,
                            routePeerId: routePeerId,
                            terminalFailureCode: sendError
                        )
                    }
                    continue
                }

                // Reset failure count on success (LOG-AUDIT-001 fix)
                consecutiveDeliveryFailures[peerKey] = 0
                lastFailureTime.removeValue(forKey: peerKey)
                logger.info("[OK] Delivery ACK from \(routePeerId) after relay-circuit retry")
                logDeliveryAttempt(
                    messageId: traceMessageId,
                    medium: "relay-circuit",
                    phase: "retry",
                    outcome: "success",
                    detail: "ctx=\(attemptContext) route=\(routePeerId)"
                )
                return DeliveryAttemptResult(
                    acked: true,
                    routePeerId: routePeerId,
                    terminalFailureCode: nil
                )
            }
        }

        logDeliveryAttempt(
            messageId: traceMessageId,
            medium: "final",
            phase: "aggregate",
            outcome: localAcked ? "local_accepted_no_core_ack" : "failed",
            detail: "ctx=\(attemptContext) route_fallback=\(sanitizedCandidates.first ?? routePeerFallback)"
        )
        return DeliveryAttemptResult(
            acked: localAcked,
            routePeerId: nil,
            terminalFailureCode: nil
        )
    }

    private func awaitPeerConnection(peerId: String, timeoutMs: UInt64 = 5000) async -> Bool {
        guard let swarmBridge else { return false }
        let deadline = Date().addingTimeInterval(Double(timeoutMs) / 1000.0)
        while Date() < deadline {
            if await swarmBridge.getPeers().contains(peerId) {
                return true
            }
            try? await Task.sleep(nanoseconds: 100_000_000)
        }
        return false
    }

    private func enqueuePendingOutbound(
        historyRecordId: String,
        peerId: String,
        routePeerId: String?,
        addresses: [String],
        envelopeData: Data,
        initialAttemptCount: UInt32 = 0,
        initialDelaySec: UInt64 = 0,
        strictBleOnlyMode: Bool = false,
        recipientIdentityId: String? = nil,
        intendedDeviceId: String? = nil,
        terminalFailureCode: String? = nil
    ) {
        if isMessageDeliveredLocally(historyRecordId) {
            appendDiagnostic("delivery_state msg=\(historyRecordId) state=delivered detail=skip_enqueue_already_delivered")
            return
        }
        let now = UInt64(Date().timeIntervalSince1970)
        var queue = loadPendingOutbox().filter { $0.historyRecordId != historyRecordId }
        queue.append(
            PendingOutboundEnvelope(
                queueId: UUID().uuidString,
                historyRecordId: historyRecordId,
                peerId: peerId,
                routePeerId: routePeerId,
                addresses: addresses,
                envelopeBase64: envelopeData.base64EncodedString(),
                createdAtEpochSec: now,
                attemptCount: initialAttemptCount,
                nextAttemptAtEpochSec: now + initialDelaySec,
                strictBleOnlyMode: strictBleOnlyMode,
                recipientIdentityId: recipientIdentityId,
                intendedDeviceId: intendedDeviceId,
                terminalFailureCode: terminalFailureCode
            )
        )
        savePendingOutbox(queue)
        let initialState = initialDelaySec > 0 ? "stored" : "forwarding"
        appendDiagnostic("delivery_state msg=\(historyRecordId) state=\(initialState) detail=enqueued attempt=\(initialAttemptCount) next_attempt_delay_sec=\(initialDelaySec)")
        dispatchFlushPendingOutbox(reason: "enqueue")
    }

    private var outboxFlushInFlight = false
    private let retryThrottleMs: Int = 2000

    private func dispatchFlushPendingOutbox(reason: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(retryThrottleMs)) { [weak self] in
            Task { [weak self] in
                await self?.flushPendingOutbox(reason: reason)
            }
        }
    }

    private func flushPendingOutbox(reason: String) async {
        guard !outboxFlushInFlight else { return }
        outboxFlushInFlight = true
        defer { outboxFlushInFlight = false }

        // Ensure relay backbone is reachable whenever we check outbox
        await primeRelayBootstrapConnections()

        let now = UInt64(Date().timeIntervalSince1970)
        let queue = loadPendingOutbox()
        if queue.isEmpty { return }

        var nextQueue: [PendingOutboundEnvelope] = []
        nextQueue.reserveCapacity(queue.count)

        for item in queue {
            // Yield between items to prevent CPU starvation (IOS-PERF-001).
            // Without this, a large outbox under retry load can spike CPU to
            // 99% and trigger the iOS watchdog kill.
            await Task.yield()

            let ackedWithoutReceiptCount = item.ackedWithoutReceiptCount ?? 0
            if Self.shouldStopAckedWithoutReceiptRetries(
                ackedWithoutReceiptCount: ackedWithoutReceiptCount,
                createdAtEpochSec: item.createdAtEpochSec,
                nowEpochSec: now,
                maxAgeSeconds: pendingOutboxMaxAgeSeconds
            ) {
                appendDiagnostic(
                    "delivery_state msg=\(item.historyRecordId) state=delivered_unconfirmed detail=stopped_pending_outbox reason=max_age_exceeded_acked_without_receipt acked_without_receipt=\(ackedWithoutReceiptCount)"
                )
                continue
            }
            if let expiryReason = pendingOutboxExpiryReason(for: item, nowEpochSec: now) {
                appendDiagnostic(
                    "delivery_state msg=\(item.historyRecordId) state=failed detail=dropped_pending_outbox reason=\(expiryReason) attempt=\(item.attemptCount)"
                )
                continue
            }
            if item.terminalFailureCode != nil {
                nextQueue.append(item)
                continue
            }
            if item.nextAttemptAtEpochSec > now {
                nextQueue.append(item)
                continue
            }

            if isMessageDeliveredLocally(item.historyRecordId) {
                continue
            }
            appendDiagnostic("delivery_state msg=\(item.historyRecordId) state=forwarding detail=retry_attempt=\(item.attemptCount + 1)")

            guard let envelopeData = Data(base64Encoded: item.envelopeBase64) else {
                logger.warning("Dropping corrupt pending envelope \(item.queueId)")
                continue
            }

            let contact = try? contactManager?.get(peerId: item.peerId)
            let latestRouting = parseRoutingHintsFromNotes(contact?.notes)
            let fallbackMultipeerPeerId = defaultMultipeerPeerId(fromPublicKey: contact?.publicKey)
            let routePeerCandidates = buildRoutePeerCandidates(
                peerId: item.peerId,
                cachedRoutePeerId: item.routePeerId,
                notes: contact?.notes,
                recipientPublicKey: contact?.publicKey
            )
            let resolvedRoutePeerId = routePeerCandidates.first
            let resolvedAddresses = buildDialCandidatesForPeer(
                routePeerId: resolvedRoutePeerId,
                rawAddresses: item.addresses + latestRouting.listeners,
                includeRelayCircuits: true
            )

            let delivery = await attemptDirectSwarmDelivery(
                routePeerCandidates: routePeerCandidates,
                addresses: resolvedAddresses,
                envelopeData: envelopeData,
                multipeerPeerId: latestRouting.multipeerPeerId ?? fallbackMultipeerPeerId,
                blePeerId: latestRouting.blePeerId,
                traceMessageId: item.historyRecordId,
                attemptContext: "outbox_retry",
                strictBleOnlyOverride: item.strictBleOnlyMode,
                recipientIdentityId: item.recipientIdentityId ?? contact?.publicKey,
                intendedDeviceId: item.intendedDeviceId ?? contact?.lastKnownDeviceId
            )
            let selectedRoutePeerId = delivery.acked
                ? (delivery.routePeerId ?? resolvedRoutePeerId)
                : delivery.routePeerId
            if isMessageDeliveredLocally(item.historyRecordId) {
                continue
            }

            if let terminalFailureCode = delivery.terminalFailureCode {
                nextQueue.append(
                    PendingOutboundEnvelope(
                        queueId: item.queueId,
                        historyRecordId: item.historyRecordId,
                        peerId: item.peerId,
                        routePeerId: selectedRoutePeerId,
                        addresses: resolvedAddresses,
                        envelopeBase64: item.envelopeBase64,
                        createdAtEpochSec: item.createdAtEpochSec,
                        attemptCount: item.attemptCount,
                        nextAttemptAtEpochSec: item.nextAttemptAtEpochSec,
                        strictBleOnlyMode: item.strictBleOnlyMode,
                        recipientIdentityId: item.recipientIdentityId,
                        intendedDeviceId: item.intendedDeviceId,
                        terminalFailureCode: terminalFailureCode,
                        ackedWithoutReceiptCount: item.ackedWithoutReceiptCount
                    )
                )
                appendDiagnostic("delivery_state msg=\(item.historyRecordId) state=rejected detail=terminal_failure_code=\(terminalFailureCode)")
                continue
            }

            if delivery.acked {
                // A transport ACK is not a genuine delivery failure. Track it
                // separately so it never consumes the failure-attempt ceiling.
                let nextAckedWithoutReceiptCount = ackedWithoutReceiptCount + 1
                let adaptiveReceiptWait = Self.receiptRetryDelaySeconds(
                    ackedWithoutReceiptCount: nextAckedWithoutReceiptCount
                )
                nextQueue.append(
                    PendingOutboundEnvelope(
                        queueId: item.queueId,
                        historyRecordId: item.historyRecordId,
                        peerId: item.peerId,
                        routePeerId: selectedRoutePeerId,
                        addresses: resolvedAddresses,
                        envelopeBase64: item.envelopeBase64,
                        createdAtEpochSec: item.createdAtEpochSec,
                        attemptCount: item.attemptCount,
                        nextAttemptAtEpochSec: now + adaptiveReceiptWait,
                        strictBleOnlyMode: item.strictBleOnlyMode,
                        recipientIdentityId: item.recipientIdentityId,
                        intendedDeviceId: item.intendedDeviceId,
                        terminalFailureCode: item.terminalFailureCode,
                        ackedWithoutReceiptCount: nextAckedWithoutReceiptCount
                    )
                )
                appendDiagnostic("delivery_state msg=\(item.historyRecordId) state=stored detail=awaiting_receipt_delay_sec=\(adaptiveReceiptWait) acked_without_receipt=\(nextAckedWithoutReceiptCount)")
                continue
            }

            let nextAttemptCount = item.attemptCount + 1
            // Progressive backoff: fast retries first, then slow down
            // Attempts 1-6: 2^n seconds (2, 4, 8, 16, 32, 64)
            // Attempts 7-20: 60 seconds
            // Attempts 21+: 300 seconds (5 min) — patient but persistent
            let backoff: UInt64
            if nextAttemptCount <= 6 {
                backoff = UInt64(min(64, 1 << Int(nextAttemptCount)))
            } else if nextAttemptCount <= 20 {
                backoff = 60
            } else {
                backoff = 300
            }
            nextQueue.append(
                    PendingOutboundEnvelope(
                        queueId: item.queueId,
                        historyRecordId: item.historyRecordId,
                        peerId: item.peerId,
                        routePeerId: selectedRoutePeerId,
                        addresses: resolvedAddresses,
                        envelopeBase64: item.envelopeBase64,
                        createdAtEpochSec: item.createdAtEpochSec,
                        attemptCount: nextAttemptCount,
                        nextAttemptAtEpochSec: now + backoff,
                        strictBleOnlyMode: item.strictBleOnlyMode,
                        recipientIdentityId: item.recipientIdentityId,
                        intendedDeviceId: item.intendedDeviceId,
                        terminalFailureCode: item.terminalFailureCode,
                        ackedWithoutReceiptCount: item.ackedWithoutReceiptCount
                    )
            )
            appendDiagnostic("delivery_state msg=\(item.historyRecordId) state=stored detail=retry_backoff_sec=\(backoff) attempt=\(nextAttemptCount)")
        }

        savePendingOutbox(nextQueue)
    }

    private func loadPendingOutbox() -> [PendingOutboundEnvelope] {
        guard let data = try? Data(contentsOf: pendingOutboxURL), !data.isEmpty else {
            return []
        }
        guard let decoded = try? JSONDecoder().decode([PendingOutboundEnvelope].self, from: data) else {
            logger.warning("Failed to parse pending outbox")
            return []
        }
        return decoded
    }

    private func savePendingOutbox(_ queue: [PendingOutboundEnvelope]) {
        do {
            let data = try JSONEncoder().encode(queue)
            try data.write(to: pendingOutboxURL, options: .atomic)
        } catch {
            logger.warning("Failed to persist pending outbox: \(error.localizedDescription)")
        }
    }

    private func pendingOutboxExpiryReason(
        for item: PendingOutboundEnvelope,
        nowEpochSec: UInt64
    ) -> String? {
        if (item.ackedWithoutReceiptCount ?? 0) == 0,
           item.attemptCount >= pendingOutboxMaxAttempts {
            return "max_attempts_exceeded"
        }
        return nil
    }

    private func removePendingOutbound(historyRecordId: String) {
        guard !historyRecordId.isEmpty else { return }
        let queue = loadPendingOutbox()
        let filtered = queue.filter { $0.historyRecordId != historyRecordId }
        guard filtered.count != queue.count else { return }
        savePendingOutbox(filtered)
    }

    private func triggerPendingSyncForPeerIds(_ peerIds: [String], reason: String) {
        let normalizedIds = peerIds
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
            .reduce(into: [String]()) { acc, next in
                if !acc.contains(next) {
                    acc.append(next)
                }
            }
        if normalizedIds.isEmpty {
            dispatchFlushPendingOutbox(reason: reason)
            return
        }
        normalizedIds.forEach { promotePendingOutboundForPeer(peerId: $0) }
        dispatchFlushPendingOutbox(reason: reason)
    }

    private func promotePendingOutboundForPeer(peerId: String, excludingMessageId: String? = nil) {
        let trimmedPeerId = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedPeerId.isEmpty else { return }
        let now = UInt64(Date().timeIntervalSince1970)
        let queue = loadPendingOutbox()
        var changed = false
        let promoted = queue.map { item -> PendingOutboundEnvelope in
            let routePeerId = item.routePeerId?.trimmingCharacters(in: .whitespacesAndNewlines)
            guard item.peerId == trimmedPeerId || routePeerId == trimmedPeerId else { return item }
            if let excludingMessageId, item.historyRecordId == excludingMessageId {
                return item
            }
            if item.terminalFailureCode != nil {
                return item
            }
            if item.nextAttemptAtEpochSec <= now {
                return item
            }
            changed = true
            return PendingOutboundEnvelope(
                queueId: item.queueId,
                historyRecordId: item.historyRecordId,
                peerId: item.peerId,
                routePeerId: item.routePeerId,
                addresses: item.addresses,
                envelopeBase64: item.envelopeBase64,
                createdAtEpochSec: item.createdAtEpochSec,
                attemptCount: item.attemptCount,
                nextAttemptAtEpochSec: now,
                strictBleOnlyMode: item.strictBleOnlyMode,
                recipientIdentityId: item.recipientIdentityId,
                intendedDeviceId: item.intendedDeviceId,
                terminalFailureCode: item.terminalFailureCode,
                ackedWithoutReceiptCount: item.ackedWithoutReceiptCount
            )
        }
        guard changed else { return }
        savePendingOutbox(promoted)
        appendDiagnostic("delivery_state msg=\(excludingMessageId ?? "unknown") state=forwarding detail=peer_queue_promoted peer=\(trimmedPeerId)")
    }

    private func isMessageDeliveredLocally(_ messageId: String) -> Bool {
        pruneDeliveredReceiptCache()
        if deliveredReceiptCache[messageId] != nil {
            return true
        }
        return ((try? historyManager?.get(id: messageId))?.delivered == true)
    }

    @discardableResult
    private func markDeliveredReceiptSeen(_ messageId: String) -> Bool {
        pruneDeliveredReceiptCache()
        if deliveredReceiptCache[messageId] != nil {
            return false
        }
        deliveredReceiptCache[messageId] = Date()
        return true
    }

    private func pruneDeliveredReceiptCache(now: Date = Date()) {
        deliveredReceiptCache = deliveredReceiptCache.filter { now.timeIntervalSince($0.value) <= deliveredReceiptCacheTtl }
        if deliveredReceiptCache.count <= 2048 {
            return
        }
        let trimmed = deliveredReceiptCache
            .sorted { $0.value > $1.value }
            .prefix(1024)
            .map { ($0.key, $0.value) }
        deliveredReceiptCache = Dictionary(uniqueKeysWithValues: trimmed)
    }

    private func prioritizeAddressesForCurrentNetwork(_ addresses: [String]) -> [String] {
        guard addresses.count > 1 else { return addresses }
        let lan = addresses.filter { isSameLanAddress($0) }
        guard !lan.isEmpty else { return addresses }
        return (lan + addresses.filter { !lan.contains($0) })
    }

    private func isSameLanAddress(_ multiaddr: String) -> Bool {
        guard let targetIp = extractIpv4FromMultiaddr(multiaddr),
              let localIp = getLocalIpAddress() else {
            return false
        }
        return sameSubnet24(localIp, targetIp)
    }

    private func extractIpv4FromMultiaddr(_ multiaddr: String) -> String? {
        let marker = "/ip4/"
        guard let markerRange = multiaddr.range(of: marker) else { return nil }
        let tail = multiaddr[markerRange.upperBound...]
        let components = tail.split(separator: "/")
        guard let first = components.first else { return nil }
        return String(first)
    }

    private func sameSubnet24(_ a: String, _ b: String) -> Bool {
        let lhs = a.split(separator: ".")
        let rhs = b.split(separator: ".")
        guard lhs.count == 4, rhs.count == 4 else { return false }
        return lhs[0] == rhs[0] && lhs[1] == rhs[1] && lhs[2] == rhs[2]
    }

    private static func parseBootstrapRelay(from multiaddr: String) -> (transportAddr: String, relayPeerId: String)? {
        guard let range = multiaddr.range(of: "/p2p/", options: .backwards) else {
            return nil
        }
        let transportAddr = String(multiaddr[..<range.lowerBound]).trimmingCharacters(in: .whitespacesAndNewlines)
        let relayPeerId = String(multiaddr[range.upperBound...]).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !transportAddr.isEmpty, !relayPeerId.isEmpty else { return nil }
        return (transportAddr, relayPeerId)
    }

    func isBootstrapRelayPeer(_ peerId: String) -> Bool {
        return ledgerBootstrapEntries().contains { $0.peerId == peerId }
    }

    /// Check if a peer is a known relay (either bootstrap or dynamically discovered headless)
    func isKnownRelay(_ peerId: String) -> Bool {
        if isBootstrapRelayPeer(peerId) { return true }
        guard let info = discoveredPeerMap[peerId] else { return false }
        return info.isRelay && !info.isFull
    }

    private func buildDialCandidatesForPeer(
        routePeerId: String?,
        rawAddresses: [String],
        includeRelayCircuits: Bool
    ) -> [String] {
        var deduped: [String] = []
        for addr in rawAddresses.compactMap({ normalizeAddressHint($0) }) where !deduped.contains(addr) {
            deduped.append(addr)
        }
        let prioritized = prioritizeAddressesForCurrentNetwork(deduped)
        let relayCircuits: [String]
        if includeRelayCircuits,
           let routePeerId,
           !routePeerId.isEmpty {
            relayCircuits = relayCircuitAddresses(for: routePeerId)
        } else {
            relayCircuits = []
        }
        var merged: [String] = []
        for addr in prioritized + relayCircuits where !merged.contains(addr) {
            merged.append(addr)
        }
        // Cap at 6 candidates to avoid excessive dialing.
        // Priority: LAN addresses first, then relay circuits, then public
        return Array(merged.prefix(6))
    }

    private func normalizeOutboundListenerHints(_ raw: [String]) -> [String] {
        var out: [String] = []
        for addr in raw.compactMap({ normalizeAddressHint($0) }) where !out.contains(addr) {
            out.append(addr)
        }
        return out
    }

    private func normalizeExternalAddressHints(_ raw: [String]) -> [String] {
        var out: [String] = []
        for addr in raw.compactMap({ normalizeAddressHint($0) }) where !out.contains(addr) {
            out.append(addr)
        }
        return out
    }

    private func normalizeAddressHint(_ raw: String) -> String? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }

        let replacedWildcard: String
        if trimmed.contains("/ip4/0.0.0.0/"), let localIp = getLocalIpAddress() {
            replacedWildcard = trimmed.replacingOccurrences(of: "/ip4/0.0.0.0/", with: "/ip4/\(localIp)/")
        } else {
            replacedWildcard = trimmed
        }

        let candidate = replacedWildcard.hasPrefix("/")
            ? replacedWildcard
            : (toMultiaddrFromSocketAddress(replacedWildcard) ?? "")
        guard !candidate.isEmpty else { return nil }
        guard isDialableAddress(candidate) else { return nil }
        return candidate
    }

    private func toMultiaddrFromSocketAddress(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        if trimmed.hasPrefix("/") { return trimmed }

        guard let separatorIdx = trimmed.lastIndex(of: ":") else { return nil }
        let host = String(trimmed[..<separatorIdx]).trimmingCharacters(in: CharacterSet(charactersIn: "[]"))
        let portStr = String(trimmed[trimmed.index(after: separatorIdx)...])
        guard let port = Int(portStr), (1...65535).contains(port), !host.isEmpty else { return nil }

        let ipv4Regex = try? NSRegularExpression(pattern: #"^\d{1,3}(\.\d{1,3}){3}$"#)
        let hostRange = NSRange(host.startIndex..<host.endIndex, in: host)
        if host.contains(":") {
            return "/ip6/\(host)/tcp/\(port)"
        }
        if let ipv4Regex, ipv4Regex.firstMatch(in: host, options: [], range: hostRange) != nil {
            return "/ip4/\(host)/tcp/\(port)"
        }
        return "/dns4/\(host)/tcp/\(port)"
    }

    private func isDialableAddress(_ multiaddr: String) -> Bool {
        if multiaddr.contains("/p2p-circuit") { return true }
        guard let ip = extractIpv4FromMultiaddr(multiaddr) else { return true }
        if isSpecialUseIPv4(ip) { return false }
        if isPrivateIPv4(ip) {
            return isSameLanAddress(multiaddr)
        }
        return true
    }

    private func parseIPv4Octets(_ ip: String) -> [Int]? {
        let octets = ip.split(separator: ".").compactMap { Int($0) }
        guard octets.count == 4 else { return nil }
        guard octets.allSatisfy({ (0...255).contains($0) }) else { return nil }
        return octets
    }

    private func isPrivateIPv4(_ ip: String) -> Bool {
        guard let octets = parseIPv4Octets(ip) else { return false }
        return octets[0] == 10
            || (octets[0] == 172 && (16...31).contains(octets[1]))
            || (octets[0] == 192 && octets[1] == 168)
    }

    private func isSpecialUseIPv4(_ ip: String) -> Bool {
        guard let octets = parseIPv4Octets(ip) else { return true }
        let o0 = octets[0]
        let o1 = octets[1]
        let o2 = octets[2]

        if o0 == 0 || o0 == 127 { return true }
        if o0 == 169 && o1 == 254 { return true }
        if o0 == 100 && (64...127).contains(o1) { return true } // RFC6598 CGNAT
        if o0 == 192 && o1 == 0 && (o2 == 0 || o2 == 2) { return true }
        if o0 == 198 && (o1 == 18 || o1 == 19) { return true } // Benchmark network
        if o0 == 198 && o1 == 51 && o2 == 100 { return true }
        if o0 == 203 && o1 == 0 && o2 == 113 { return true }
        if o0 >= 224 { return true } // multicast/reserved/broadcast
        return false
    }

    private func relayCircuitAddresses(for targetPeerId: String) -> [String] {
        guard isLibp2pPeerId(targetPeerId) else { return [] }

        // 1. Ledger-sourced bootstrap nodes
        var relays: [String] = ledgerBootstrapAddresses().compactMap { bootstrap in
            guard let relay = Self.parseBootstrapRelay(from: bootstrap) else { return nil }
            return "\(relay.transportAddr)/p2p/\(relay.relayPeerId)/p2p-circuit/p2p/\(targetPeerId)"
        }

        // 2. Dynamically discovered headless/relay nodes
        let dynamicRelays = discoveredPeerMap.filter { $0.value.isRelay && !$0.value.isFull && $0.key != targetPeerId }
        for (relayPeerId, _) in dynamicRelays where isLibp2pPeerId(relayPeerId) {
            // If we have direct addresses for this relay, try using it
            let directAddrs = getDialHintsForRoutePeer(relayPeerId)
            for addr in directAddrs {
                let circuit = "\(addr)/p2p/\(relayPeerId)/p2p-circuit/p2p/\(targetPeerId)"
                if !relays.contains(circuit) { relays.append(circuit) }
            }
        }

        return relays
    }

    private func extractRelayPeerId(from multiaddr: String) -> String? {
        let components = multiaddr
            .split(separator: "/")
            .map(String.init)
            .filter { !$0.isEmpty }
        guard !components.isEmpty else { return nil }

        if let circuitIndex = components.firstIndex(of: "p2p-circuit"),
           circuitIndex >= 2,
           components[circuitIndex - 2] == "p2p" {
            let relayPeerId = components[circuitIndex - 1]
            return isLibp2pPeerId(relayPeerId) ? relayPeerId : nil
        }

        if components.count >= 2, components[components.count - 2] == "p2p" {
            let trailingPeer = components[components.count - 1]
            return isLibp2pPeerId(trailingPeer) ? trailingPeer : nil
        }

        return nil
    }

    private func shouldAttemptRelayDial(peerId: String, source: String) -> Bool {
        let trimmed = peerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return false }
        let now = Date()
        if let last = relayDialDebounceState[trimmed],
           now.timeIntervalSince(last) < relayDialDebounceInterval {
            appendDiagnostic("relay_dial_debounced peer=\(trimmed) source=\(source)")
            updateRelayAvailability(peerId: trimmed, event: "dial_debounced")
            return false
        }
        relayDialDebounceState[trimmed] = now
        updateRelayAvailability(peerId: trimmed, event: "dial_allowed")
        return true
    }
    private func updateRelayAvailability(peerId: String, event: String) {
        let now = Date()
        // Don't count debounced or proactive maintenance events towards flapping.
        // Proactive dial attempts (allowed, attempt, started) are part of normal
        // maintenance and don't indicate connection instability.
        let isProactiveIntent = event == "dial_allowed" || event == "dial_attempt" || event == "dial_started" || event == "dial_debounced"
        if !isProactiveIntent {
            relayRecentEventTimes.append(now)
        }
        relayRecentEventTimes = relayRecentEventTimes.filter { now.timeIntervalSince($0) <= 60 }

        if event == "disconnected" {
            relayLastDisconnectAt = now
        } else if event == "identified" {
            if let lastDisconnect = relayLastDisconnectAt, now.timeIntervalSince(lastDisconnect) <= 20 {
                relayAvailabilityState = .recovering
            }
        }

        // Raised from 6→30: each relay dial generates 3+ events (allowed+attempt+started),
        // and with 2 relay peers one normal round is ~6 events. The old threshold caused
        // immediate self-reinforcing flapping that prevented relay reservations.
        if relayRecentEventTimes.count >= 30 {
            relayAvailabilityState = .flapping
            relayBackoffUntil = now.addingTimeInterval(30)
        } else if now < relayBackoffUntil {
            relayAvailabilityState = .backoff
        } else if relayAvailabilityState != .recovering {
            relayAvailabilityState = .stable
        }
        relayAvailabilityUpdatedAt = now
        // Throttle relay diagnostics: log every 10th event when flapping
        // to reduce disk/CPU pressure (was generating 70+ log lines/minute)
        if relayAvailabilityState != .flapping || relayRecentEventTimes.count % 10 == 0 {
            appendDiagnostic("relay_state peer=\(peerId) event=\(event) state=\(relayAvailabilityState.rawValue) events_60s=\(relayRecentEventTimes.count)")
        }
    }

    private func primeRelayBootstrapConnections() async {
        guard let swarmBridge else { return }
        guard !relayBootstrapDialInProgress else {
            appendDiagnostic("relay_prime_skipped reason=in_progress")
            return
        }
        let now = Date()
        guard now.timeIntervalSince(lastRelayBootstrapDialAt) >= 10 else { return }
        relayBootstrapDialInProgress = true
        defer { relayBootstrapDialInProgress = false }
        lastRelayBootstrapDialAt = now

        for addr in ledgerBootstrapAddresses() {
            let relayPeerId = Self.parseBootstrapRelay(from: addr)?.relayPeerId
            do {
                if let relayPeerId,
                   !shouldAttemptRelayDial(peerId: relayPeerId, source: "prime_bootstrap") {
                    continue
                }
                if !shouldAttemptDial(addr) { continue }
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_attempt")
                }
                try await swarmBridge.dial(multiaddr: addr)
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_started")
                }
            } catch {
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_failed")
                }
                logger.debug("Relay bootstrap dial skipped for \(addr): \(error.localizedDescription)")
            }
        }
    }

    private func persistRouteHintsForTransportPeer(
        libp2pPeerId: String,
        listeners: [String],
        knownPublicKey: String? = nil
    ) {
        guard !libp2pPeerId.isEmpty else { return }
        guard let contacts = try? contactManager?.list() else { return }
        let normalizedListeners = normalizeOutboundListenerHints(listeners)
        let extractedTransportKey: String? = {
            guard let core = ironCore else { return nil }
            return try? core.extractPublicKeyFromPeerId(peerId: libp2pPeerId)
        }()
        let normalizedTransportKey = knownPublicKey
            ?? normalizePublicKey(extractedTransportKey)
        var didUpdateContact = false

        for contact in contacts {
            let routing = parseRoutingHintsFromNotes(contact.notes)
            let match = contact.peerId == libp2pPeerId
                || routing.libp2pPeerId == libp2pPeerId
                || (
                    normalizedTransportKey != nil &&
                    normalizePublicKey(contact.publicKey) == normalizedTransportKey
                )
            if !match { continue }

            let withPeerId = appendRoutingHint(notes: contact.notes, key: "libp2p_peer_id", value: libp2pPeerId)
            let withListeners = upsertRoutingListeners(notes: withPeerId, listeners: normalizedListeners)
            if withListeners == contact.notes { continue }

            let updated = Contact(
                peerId: contact.peerId,
                nickname: contact.nickname,
                localNickname: contact.localNickname,
                publicKey: contact.publicKey,
                addedAt: contact.addedAt,
                lastSeen: contact.lastSeen,
                notes: withListeners,
                lastKnownDeviceId: contact.lastKnownDeviceId,
                verifiedAt: contact.verifiedAt,
                isTombstone: contact.isTombstone
            )
            try? contactManager?.add(contact: updated)
            contactManager?.flush()
            didUpdateContact = true
        }

        if didUpdateContact {
            scheduleContactPersistenceCheckpoint(reason: "transport_route_hints_updated")
        }
    }

    private func upsertFederatedContact(
        canonicalPeerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: [String],
        blePeerId: String? = nil,
        deviceId: String? = nil,
        createIfMissing: Bool = true
    ) {
        let normalizedPeerId = canonicalPeerId.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedPeerId.isEmpty else { return }
        guard let normalizedKey = normalizePublicKey(publicKey) else { return }

        let normalizedLibp2p = libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines)
        if let normalizedLibp2p, !normalizedLibp2p.isEmpty, isBootstrapRelayPeer(normalizedLibp2p) {
            return
        }

        let contacts = (try? contactManager?.list()) ?? []
        let existingByKey = contacts.first { normalizePublicKey($0.publicKey) == normalizedKey }
        let existingById = contacts.first { $0.peerId == normalizedPeerId }

        // Auth guard: do not accept federated nickname updates for an existing peerId
        // unless the source key matches the stored key for that identity.
        if let existingById,
           normalizePublicKey(existingById.publicKey) != normalizedKey {
            logger.warning(
                "Rejected federated nickname update for \(normalizedPeerId, privacy: .public): key mismatch"
            )
            return
        }
        let existing = existingByKey ?? existingById
        if existing == nil && !createIfMissing {
            return
        }

        var notes = existing?.notes
        if let normalizedLibp2p, !normalizedLibp2p.isEmpty {
            notes = appendRoutingHint(notes: notes, key: "libp2p_peer_id", value: normalizedLibp2p)
        }
        if let normalizedBle = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines), !normalizedBle.isEmpty {
            notes = appendRoutingHint(notes: notes, key: "ble_peer_id", value: normalizedBle)
        }
        notes = upsertRoutingListeners(notes: notes, listeners: normalizeOutboundListenerHints(listeners))

        let now = UInt64(Date().timeIntervalSince1970)
        let resolvedPeerId = existing?.peerId ?? normalizedPeerId
        let resolvedPublicKey = existingByKey?.publicKey ?? normalizedKey
        let incomingNickname = normalizeNickname(nickname)
        let resolvedNickname = selectAuthoritativeNickname(
            incoming: incomingNickname,
            existing: existing?.nickname
        )

        let updated = Contact(
            peerId: resolvedPeerId,
            nickname: resolvedNickname,
            localNickname: existing?.localNickname,
            publicKey: resolvedPublicKey,
            addedAt: existing?.addedAt ?? now,
            lastSeen: now,
            notes: notes,
            lastKnownDeviceId: deviceId ?? existing?.lastKnownDeviceId,
            verifiedAt: existing?.verifiedAt,
            isTombstone: existing?.isTombstone ?? false
        )
        try? contactManager?.add(contact: updated)
        contactManager?.flush()
        scheduleContactPersistenceCheckpoint(reason: "federated_contact_upserted")
        annotateIdentityInLedger(
            routePeerId: normalizedLibp2p,
            listeners: listeners,
            publicKey: resolvedPublicKey,
            nickname: resolvedNickname
        )
    }

    private func upsertRoutingListeners(notes: String?, listeners: [String]) -> String? {
        guard !listeners.isEmpty else { return notes }
        let base = notes?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let filtered = base
            .split(whereSeparator: { $0 == ";" || $0 == "\n" })
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty && !$0.hasPrefix("listeners:") }
        return (filtered + ["listeners:\(listeners.joined(separator: ","))"]).joined(separator: ";")
    }

    private func broadcastIdentityBeacon() {
        Task { @MainActor [weak self] in
            guard let self else { return }
            await self.broadcastIdentityBeaconInternal()
        }
    }

    private func broadcastIdentityBeaconInternal() async {
        let now = Date()
        if let last = lastBleBeaconUpdate, now.timeIntervalSince(last) < bleBeaconUpdateInterval {
            // Already sent a beacon recently; skip to avoid radio churn
            return
        }
        lastBleBeaconUpdate = now

        guard let info = ironCore?.getIdentityInfo(),
              info.publicKeyHex != nil else { return }

        // 1. Immediate update with just identity (no listeners yet)
        // This allows peers to "see" us in the UI immediately while we wait for swarm listeners to bind.
        await setIdentityBeaconInternal(info: info, listeners: [])

        // 2. Delayed update with full connection hints
        pendingBleBeaconListenerRefreshTask?.cancel()
        pendingBleBeaconListenerRefreshTask = Task { @MainActor [weak self] in
            guard let self else { return }
            defer { self.pendingBleBeaconListenerRefreshTask = nil }
            var listeners = await getListeningAddresses()
            var attempts = 0
            while listeners.isEmpty && attempts < 10 {
                try? await Task.sleep(nanoseconds: 500_000_000) // 500ms
                guard !Task.isCancelled else { return }
                listeners = await getListeningAddresses()
                attempts += 1
            }
            guard !Task.isCancelled else { return }
            await setIdentityBeaconInternal(info: info, listeners: listeners)
        }
    }

    private func scheduleIdentityBeaconRefresh(reason: String, delayNanoseconds: UInt64 = 1_000_000_000) {
        guard pendingBleBeaconRefreshTask == nil else {
            logger.debug("BLE identity beacon refresh already pending (\(reason))")
            return
        }
        pendingBleBeaconRefreshTask = Task { [weak self] in
            guard let self else { return }
            defer { self.pendingBleBeaconRefreshTask = nil }
            try? await Task.sleep(nanoseconds: delayNanoseconds)
            guard !Task.isCancelled else { return }
            logger.debug("Refreshing BLE identity beacon (\(reason))")
            broadcastIdentityBeacon()
        }
    }

    private func setIdentityBeaconInternal(info: IdentityInfo, listeners rawListeners: [String]) async {
        guard let publicKeyHex = info.publicKeyHex else { return }

        // Keep BLE identity beacons compact to avoid platform read failures.
        // Android/iOS both have observed issues when payload exceeds ~512 bytes.
        var listeners = Array(normalizeOutboundListenerHints(rawListeners).prefix(2))
        var externalAddresses = Array(normalizeExternalAddressHints(await getExternalAddresses()).prefix(2))
        let nickname = String((info.nickname ?? "").prefix(32))

        func buildBeacon() -> [String: Any] {
            let connectionHints = Array(Set(listeners + externalAddresses)).sorted()
            return [
                "identity_id": info.identityId ?? "",
                "public_key": publicKeyHex,
                "nickname": nickname,
                "peer_id": info.libp2pPeerId ?? "",
                "libp2p_peer_id": info.libp2pPeerId ?? "",
                "listeners": listeners,
                "external_addresses": externalAddresses,
                "connection_hints": connectionHints
            ]
        }

        var beacon: [String: Any] = buildBeacon()
        var data = try? JSONSerialization.data(withJSONObject: beacon)
        if let encoded = data, encoded.count > 480 {
            listeners = Array(listeners.prefix(1))
            externalAddresses = Array(externalAddresses.prefix(1))
            beacon = buildBeacon()
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        if let encoded = data, encoded.count > 480 {
            listeners = []
            externalAddresses = []
            beacon = buildBeacon()
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        if let encoded = data, encoded.count > 480 {
            beacon = [
                "identity_id": info.identityId ?? "",
                "public_key": publicKeyHex,
                "nickname": nickname,
                "peer_id": info.libp2pPeerId ?? "",
                "libp2p_peer_id": info.libp2pPeerId ?? "",
                "listeners": [],
                "external_addresses": [],
                "connection_hints": []
            ]
            data = try? JSONSerialization.data(withJSONObject: beacon)
        }
        guard let data else {
            logger.error("Failed to serialize identity beacon")
            return
        }
        let now = Date()
        if let lastPayload = lastBleBeaconPayload,
           lastPayload == data,
           let lastPublishedAt = lastBleBeaconPayloadPublishedAt,
           now.timeIntervalSince(lastPublishedAt) < bleBeaconUpdateInterval {
            return
        }
        lastBleBeaconPayload = data
        lastBleBeaconPayloadPublishedAt = now
        blePeripheralManager?.setIdentityData(data)
        logger.info("BLE identity beacon set: \(publicKeyHex.prefix(8))... (\(data.count) bytes, listeners=\(listeners.count)) p2p_id=\(info.libp2pPeerId ?? "unknown")")
    }

    // MARK: - Auto-Adjustment Engine

    func computeAdjustmentProfile(deviceProfile: DeviceProfile) throws -> AdjustmentProfile {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeProfile(device: deviceProfile)
    }

    func computeBleAdjustment(profile: AdjustmentProfile) throws -> BleAdjustment {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeBleAdjustment(profile: profile)
    }

    func computeRelayAdjustment(profile: AdjustmentProfile) throws -> RelayAdjustment {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        return autoAdjustEngine.computeRelayAdjustment(profile: profile)
    }

    func overrideBleInterval(scanMs: UInt32, advertiseMs: UInt32) throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        // Note: Only scan interval override is supported in new API
        autoAdjustEngine.overrideBleScanInterval(intervalMs: scanMs)
        logger.info("[OK] BLE interval overridden: scan=\(scanMs)ms advertise=\(advertiseMs)ms")
    }

    func overrideRelayMax(maxRelayPerHour: UInt32) throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.overrideRelayMaxPerHour(max: maxRelayPerHour)
        logger.info("[OK] Relay max overridden: \(maxRelayPerHour)/hour")
    }

    func clearAdjustmentOverrides() throws {
        guard let autoAdjustEngine = autoAdjustEngine else {
            throw MeshError.notInitialized("AutoAdjustEngine not initialized")
        }
        autoAdjustEngine.clearOverrides()
        logger.info("[OK] Adjustment overrides cleared")
        Task { @MainActor [weak self] in
            await self?.applyPowerAdjustments(reason: "overrides_cleared")
        }
    }

    private func applyPowerAdjustments(reason: String) async {
        guard let meshService = meshService else { return }
        guard let engine = autoAdjustEngine else {
            logger.debug("Power profile skipped (\(reason)): AutoAdjustEngine unavailable")
            return
        }

        guard isAutoAdjustEnabled else {
            if lastAppliedPowerSnapshot != "disabled" {
                logger.info("Power profile skipped (\(reason)): AutoAdjust disabled")
                lastAppliedPowerSnapshot = "disabled"
            }
            return
        }

        let profile = DeviceProfile(
            peerId: nil,
            deviceId: nil,
            batteryPct: currentBatteryPct,
            isCharging: currentIsCharging,
            hasWifi: networkStatus.wifi,
            motionState: currentMotionState
        )

        let adjustmentProfile = engine.computeProfile(device: profile)
        let relayAdjustment = engine.computeRelayAdjustment(profile: adjustmentProfile)
        let bleAdjustment = engine.computeBleAdjustment(profile: adjustmentProfile)

        await meshService.setRelayBudget(messagesPerHour: relayAdjustment.maxPerHour)
        bleCentralManager?.applyScanSettings(intervalMs: bleAdjustment.scanIntervalMs)

        let snapshot = [
            "p:\(adjustmentProfile)",
            "relay:\(relayAdjustment.maxPerHour)",
            "scan:\(bleAdjustment.scanIntervalMs)",
            "adv:\(bleAdjustment.advertiseIntervalMs)",
            "tx:\(bleAdjustment.txPowerDbm)",
            "bat:\(profile.batteryPct)",
            "chg:\(profile.isCharging)",
            "wifi:\(profile.hasWifi)",
            "motion:\(profile.motionState)"
        ].joined(separator: "|")

        if lastAppliedPowerSnapshot != snapshot {
            let message = "Power profile: \(adjustmentProfile) (relay:\(relayAdjustment.maxPerHour)/h, ble:\(bleAdjustment.scanIntervalMs)ms) [bat:\(profile.batteryPct)%]"
            logger.info("\(message)")
            appendDiagnostic("power_profile: \(message) reason=\(reason)")
            lastAppliedPowerSnapshot = snapshot
        }
    }

    // MARK: - Identity Export Helpers

    func getPreferredRelay() -> String? {
        return ledgerManager?.getPreferredRelays(limit: 1).first?.peerId
    }

    func connectToPeer(_ peerId: String, addresses: [String]) async {
        appendDiagnostic("connect_to_peer peer=\(peerId) addr_count=\(addresses.count)")
        let dialCandidates = buildDialCandidatesForPeer(
            routePeerId: peerId,
            rawAddresses: addresses,
            includeRelayCircuits: false
        )

        var consecutiveDebounces = 0
        for addr in dialCandidates {
            // Only append /p2p/ component if the peerId is a valid libp2p PeerId format
            // (base58btc multihash, starts with "12D3Koo" or "Qm").
            // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
            var finalAddr = addr
            if isLibp2pPeerId(peerId) && !addr.contains("/p2p/") {
                finalAddr = "\(addr)/p2p/\(peerId)"
            }
            let relayPeerId = extractRelayPeerId(from: finalAddr)
            do {
                if let relayPeerId,
                   !shouldAttemptRelayDial(peerId: relayPeerId, source: "connect_to_peer") {
                    consecutiveDebounces += 1
                    // P0: Stop trying after 2 consecutive debounces — all remaining
                    // relay addresses for the same peer will also be debounced, and
                    // each attempt generates log I/O that starves the main thread.
                    if consecutiveDebounces >= 2 { break }
                    continue
                }
                consecutiveDebounces = 0
                if !shouldAttemptDial(finalAddr) { continue }
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_attempt")
                }
                try await swarmBridge?.dial(multiaddr: finalAddr)
                logger.info("Dialing \(finalAddr)")
                appendDiagnostic("dial_attempt addr=\(finalAddr)")
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_started")
                }
            } catch {
                logger.error("Failed to dial \(finalAddr): \(error.localizedDescription)")
                appendDiagnostic("dial_failure addr=\(finalAddr) error=\(error.localizedDescription)")
                if let relayPeerId {
                    updateRelayAvailability(peerId: relayPeerId, event: "dial_failed")
                }
            }
        }
    }

    private func shouldAttemptDial(_ multiaddr: String) -> Bool {
        let key = multiaddr.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !key.isEmpty else { return false }

        let now = Date()
        if let state = dialThrottleState[key], now < state.nextAllowedAt {
            // P4: Only log dial_throttled once per address per 5-minute window
            if dialThrottleLogCache[key] == nil || now.timeIntervalSince(dialThrottleLogCache[key] ?? .distantPast) >= dialThrottleLogInterval {
                appendDiagnostic("dial_throttled addr=\(key)")
                dialThrottleLogCache[key] = now
            }
            return false
        }

        let attempts = min((dialThrottleState[key]?.attempts ?? 0) + 1, 8)
        let backoffSeconds: TimeInterval
        switch attempts {
        case 1: backoffSeconds = 0.5
        case 2: backoffSeconds = 1.5
        case 3: backoffSeconds = 3
        case 4: backoffSeconds = 6
        case 5: backoffSeconds = 10
        default: backoffSeconds = 15
        }
        dialThrottleState[key] = (attempts: attempts, nextAllowedAt: now.addingTimeInterval(backoffSeconds))
        return true
    }

    func diagnosticsLogPath() -> String {
        diagnosticsLogURL.path
    }

    private func logDeliveryAttempt(
        messageId: String?,
        medium: String,
        phase: String,
        outcome: String,
        detail: String
    ) {
        let trimmedId = messageId?.trimmingCharacters(in: .whitespacesAndNewlines)
        let msg = (trimmedId?.isEmpty == false) ? (trimmedId ?? "unknown") : "unknown"
        appendDiagnostic("delivery_attempt msg=\(msg) medium=\(medium) phase=\(phase) outcome=\(outcome) detail=\(detail)")
    }

    func diagnosticsSnapshot(limit: Int = 120) -> String {
        diagnosticsBuffer.suffix(max(1, limit)).joined(separator: "\n")
    }

    @MainActor
    private static let diagnosticDateFormatter: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()
    // Throttle os_log output: only log every Nth diagnostic to avoid flooding
    // the system console (each os_log call has significant overhead).
    private var diagnosticLogCounter: Int = 0
    private let diagnosticLogThrottle: Int = 10 // log 1 in every 10

    func appendDiagnostic(_ message: String) {
        let ts = Self.diagnosticDateFormatter.string(from: Date())
        let line = "\(ts) \(message)"

        // 1. Update memory buffer
        diagnosticsBuffer.append(line)
        if diagnosticsBuffer.count > diagnosticsMaxLines {
            diagnosticsBuffer.removeFirst(diagnosticsBuffer.count - diagnosticsMaxLines)
        }

        // 2. Write to System Console — throttled to avoid flooding
        diagnosticLogCounter += 1
        if diagnosticLogCounter % diagnosticLogThrottle == 0 {
            logger.info("DIAG: \(message)")
        }

        // WS12.41: Send to IronCore for summarized storage
        ironCore?.recordLog(line: line)

        // 3. Persist to Disk (Append only) - keep as fallback with smaller limit
        persistDiagnosticLine(line)
    }

    private func persistDiagnosticLine(_ line: String) {
        let url = diagnosticsLogURL
        let data = (line + "\n").data(using: .utf8) ?? Data()

        // Keep diagnostic file I/O off the main actor to avoid startup/runtime warnings.
        diagnosticsIOQueue.async {
            let parent = url.deletingLastPathComponent()
            try? FileManager.default.createDirectory(at: parent, withIntermediateDirectories: true)

            // Limit file size to ~100KB (much smaller now that we have summarizer)
            if let attrs = try? FileManager.default.attributesOfItem(atPath: url.path),
               let size = attrs[.size] as? UInt64,
               size > 100 * 1024 {
                self.rotateDiagnosticFiles()
            }

            if !FileManager.default.fileExists(atPath: url.path) {
                try? data.write(to: url)
                return
            }

            do {
                let fileHandle = try FileHandle(forWritingTo: url)
                defer { try? fileHandle.close() }
                try fileHandle.seekToEnd()
                try fileHandle.write(contentsOf: data)
            } catch {
                try? data.write(to: url, options: .atomic)
            }
        }
    }

    private func rotateDiagnosticFiles() {
        let url = diagnosticsLogURL
        let parent = url.deletingLastPathComponent()
        let name = url.lastPathComponent

        // Consolidate logs: .4 -> .5, .3 -> .4, etc.
        for i in (1...4).reversed() {
            let current = parent.appendingPathComponent("\(name).\(i)")
            let next = parent.appendingPathComponent("\(name).\(i + 1)")
            if FileManager.default.fileExists(atPath: current.path) {
                try? FileManager.default.removeItem(atPath: next.path)
                try? FileManager.default.moveItem(at: current, to: next)
            }
        }
        // Move current to .1
        let firstHistory = parent.appendingPathComponent("\(name).1")
        try? FileManager.default.removeItem(atPath: firstHistory.path)
        try? FileManager.default.moveItem(at: url, to: firstHistory)
    }

    func clearDiagnostics() {
        diagnosticsBuffer = []
        let url = diagnosticsLogURL
        let parent = url.deletingLastPathComponent()
        let name = url.lastPathComponent

        try? FileManager.default.removeItem(at: url)
        for i in 1...5 {
            let history = parent.appendingPathComponent("\(name).\(i)")
            try? FileManager.default.removeItem(at: history)
        }
    }

    private func startStorageMaintenance() {
        storageMaintenanceTask?.cancel()
        storageMaintenanceTask = Task { @MainActor in
            while !Task.isCancelled {
                do {
                    let path = NSHomeDirectory()
                    let attributes = try FileManager.default.attributesOfFileSystem(forPath: path)
                    let total = attributes[.systemSize] as? UInt64 ?? 0
                    let free = attributes[.systemFreeSize] as? UInt64 ?? 0

                    ironCore?.updateDiskStats(totalBytes: total, freeBytes: free)
                    try ironCore?.performMaintenance()

                    appendDiagnostic("storage_pulse free=\(free / 1024 / 1024)MB total=\(total / 1024 / 1024)MB")
                } catch {
                    logger.warning("Storage maintenance loop error: \(error.localizedDescription)")
                }

                try? await Task.sleep(nanoseconds: 15 * 60 * 1_000_000_000) // 15 minutes
            }
        }
    }

    // MARK: - Factory Reset

    /// Delete all app data and reset to factory defaults.
    /// Mirrors: android/.../data/MeshRepository.kt resetAllData()
    @MainActor
    func resetAllData() async {
        logger.warning("RESETTING ALL APPLICATION DATA")
        appendDiagnostic("factory_reset requested")

        // A factory reset is intentionally destructive. Prevent a delayed
        // contact checkpoint from recreating the Keychain recovery snapshot
        // after this method has removed it.
        pendingContactBackupTask?.cancel()
        pendingContactBackupTask = nil

        // 1. Stop all active services
        if serviceState == .running {
            stopMeshService()
        }
        pendingOutboxRetryTask?.cancel()
        pendingOutboxRetryTask = nil
        coverTrafficTask?.cancel()
        coverTrafficTask = nil
        pendingReceiptSendTasks.values.forEach { $0.cancel() }
        pendingReceiptSendTasks.removeAll()
        bleCentralManager?.stopScanning()
        blePeripheralManager?.stopAdvertising()

        // 2. Release UniFFI objects
        await swarmBridge?.shutdown()
        swarmBridge = nil
        meshService?.stop()
        meshService = nil
        ironCore?.stop()
        contactManager?.flush()
        historyManager?.flush()
        ironCore = nil
        contactManager = nil
        historyManager = nil
        ledgerManager = nil
        settingsManager = nil

        // 3. Clear in-memory state
        diagnosticsBuffer = []
        identityEmissionCache.removeAll()
        connectedEmissionCache.removeAll()
        identitySyncSentPeers.removeAll()
        historySyncSentPeers.removeAll()
        discoveredPeerMap.removeAll()
        isCreatingIdentity = false
        clearPublishedIdentity()

        // 4. Delete Keychain backup (persisted identity)
        let keychainQuery: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "com.scmessenger.identity"
        ]
        SecItemDelete(keychainQuery as CFDictionary)
        let passphraseQuery: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "com.scmessenger.identity.passphrase"
        ]
        SecItemDelete(passphraseQuery as CFDictionary)

        // 5. Clear UserDefaults app state
        let keysToRemove = [
            "hasCompletedOnboarding",
            "hasCompletedInstallModeChoice",
            "identity_backup",
            "consent_accepted"
        ]
        keysToRemove.forEach { UserDefaults.standard.removeObject(forKey: $0) }

        // 6. Delete all files in the mesh storage directory
        let meshDir = URL(fileURLWithPath: storagePath)
        if let contents = try? FileManager.default.contentsOfDirectory(
            at: meshDir, includingPropertiesForKeys: nil
        ) {
            for item in contents {
                try? FileManager.default.removeItem(at: item)
            }
        }

        // 7. Delete diagnostics log
        try? FileManager.default.removeItem(at: diagnosticsLogURL)

        logger.info("[OK] All application data reset")
    }

    func getListeningAddresses() async -> [String] {
        return await swarmBridge?.getListeners() ?? []
    }

    func getExternalAddresses() async -> [String] {
        return await swarmBridge?.getExternalAddresses() ?? []
    }

    func getTopics() async -> [String] {
        return await swarmBridge?.getTopics() ?? []
    }

    func subscribeTopic(_ topic: String) async throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try await swarmBridge.subscribeTopic(topic: topic)
    }

    func unsubscribeTopic(_ topic: String) async throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try await swarmBridge.unsubscribeTopic(topic: topic)
    }

    func publishTopic(_ topic: String, data: Data) async throws {
        guard let swarmBridge = swarmBridge else {
            throw MeshError.notInitialized("SwarmBridge not initialized")
        }
        try await swarmBridge.publishTopic(topic: topic, data: data)
    }

    func getLocalIpAddress() -> String? {
        var bestAddress: String?
        var bestScore = Int.min
        var ifaddr: UnsafeMutablePointer<ifaddrs>?
        if getifaddrs(&ifaddr) == 0 {
            var ptr = ifaddr
            while ptr != nil {
                defer { ptr = ptr?.pointee.ifa_next }

                let interface = ptr?.pointee
                let addrFamily = interface?.ifa_addr.pointee.sa_family

                if addrFamily == UInt8(AF_INET) { // IPv4 only for now
                    let flags = Int32(interface?.ifa_flags ?? 0)
                    if (flags & IFF_UP) != 0 && (flags & IFF_LOOPBACK) == 0 {
                        var hostname = [CChar](repeating: 0, count: Int(NI_MAXHOST))
                        getnameinfo(interface?.ifa_addr, socklen_t(interface?.ifa_addr.pointee.sa_len ?? 0),
                                   &hostname, socklen_t(hostname.count),
                                   nil, socklen_t(0), NI_NUMERICHOST)
                        let ip = String(cString: hostname)
                        if ip.isEmpty || isSpecialUseIPv4(ip) { continue }

                        let ifaceName: String
                        if let namePtr = interface?.ifa_name, let name = String(validatingUTF8: namePtr) {
                            ifaceName = name
                        } else {
                            ifaceName = ""
                        }
                        let isPrivate = isPrivateIPv4(ip)
                        let ifaceScore: Int
                        if ifaceName == "en0" || ifaceName.hasPrefix("en") {
                            ifaceScore = 3
                        } else if ifaceName.hasPrefix("pdp_ip") {
                            ifaceScore = 2
                        } else {
                            ifaceScore = 1
                        }
                        let score = (isPrivate ? 100 : 10) + ifaceScore
                        if score > bestScore {
                            bestScore = score
                            bestAddress = ip
                        }
                    }
                }
            }
            freeifaddrs(ifaddr)
        }
        return bestAddress
    }

    // MARK: - Identity Helpers

    func getFullIdentityInfo() -> IdentityInfo? {
        return getIdentityInfo()
    }

    /// Export a passphrase-encrypted identity backup: the identity signing
    /// key, active ratchet sessions, and contacts, encrypted with an
    /// Argon2id-derived key. Distinct from `getIdentityExportString()`,
    /// which exports the public identity card with no encryption.
    func exportIdentityBackup(passphrase: String) throws -> String {
        guard let ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        return try ironCore.exportIdentityBackup(passphrase: passphrase)
    }

    /// Import an identity backup using a passphrase the user supplied
    /// directly (as opposed to the device-bound Keychain auto-backup
    /// passphrase used internally by `restoreIdentityFromKeychain`). A
    /// wrong passphrase surfaces as an error with no fallback.
    func importIdentityBackup(backup: String, passphrase: String) throws {
        guard let ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        try ironCore.importIdentityBackup(backup: backup, passphrase: passphrase)
        finalizeImportedIdentity()
    }

    /// Complete identity activation after an import performed by a background
    /// FFI task.  Keeping this separate preserves a responsive import UI while
    /// still publishing the result on the repository's main-actor state.
    func finalizeImportedIdentity() {
        guard let ironCore else { return }
        let imported = ironCore.getIdentityInfo()
        guard imported.initialized else { return }
        reloadContactManagerAfterIdentityImport()
        ironCore.grantConsent()
        publishIdentityInfo(imported)
        persistIdentityBackupToKeychain(ironCore: ironCore)
        initializeAndStartSwarm()
        broadcastIdentityBeacon()
        UserDefaults.standard.set(true, forKey: "hasCompletedInstallModeChoice")
        UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
    }

    func getIdentityExportString() async -> String {
        guard let identity = getFullIdentityInfo() else { return "{}" }
        var listeners = normalizeOutboundListenerHints(await getListeningAddresses())
        let externalAddresses = normalizeExternalAddressHints(await getExternalAddresses())
        let relay = getPreferredRelay() ?? "None"
        let localIp = getLocalIpAddress()

        // Replace 0.0.0.0 with actual LAN IP
        if let localIp = localIp {
            var updatedListeners = [String]()
            for addr in listeners {
                if addr.contains("0.0.0.0") {
                    updatedListeners.append(addr.replacingOccurrences(of: "0.0.0.0", with: localIp))
                } else {
                    updatedListeners.append(addr)
                }
            }

            listeners = updatedListeners
        }

        let payload: [String: Any] = [
            // PRIMARY: libp2p Peer ID for cross-OS QR scanning (matches Android's "peer_id")
            "peer_id": identity.libp2pPeerId ?? "",
            // CANONICAL: Hex-encoded Ed25519 public key
            "public_key": identity.publicKeyHex ?? "",
            // MULTI-DEVICE: Routing identifier for multi-device setups
            "device_id": identity.deviceId ?? "",
            // LEGACY: Blake3 hash of public key (backward compatibility)
            "identity_id": identity.identityId ?? "",
            "nickname": identity.nickname ?? "",
            "libp2p_peer_id": identity.libp2pPeerId ?? "",  // Backward compatibility
            "listeners": listeners,
            "external_addresses": externalAddresses,
            "connection_hints": Array(Set(listeners + externalAddresses)),
            "relay": relay
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: payload),
              let json = String(data: data, encoding: .utf8) else {
            return "{}"
        }
        return json
    }

    func getIdentityQrPayload() -> String {
        guard let identity = getFullIdentityInfo(),
              let publicKey = identity.publicKeyHex,
              !publicKey.isEmpty else {
            return ""
        }
        // QR identity cards must use the shared JSON contract. Android's
        // importer intentionally rejects the old colon-delimited form, while
        // JSON preserves routing and multi-device fields for a routable contact.
        let payload: [String: Any] = [
            "version": "1.0",
            "peer_id": identity.libp2pPeerId ?? "",
            "public_key": publicKey,
            "device_id": identity.deviceId ?? "",
            "identity_id": identity.identityId ?? "",
            "nickname": identity.nickname ?? "",
            "libp2p_peer_id": identity.libp2pPeerId ?? ""
        ]
        guard JSONSerialization.isValidJSONObject(payload),
              let data = try? JSONSerialization.data(withJSONObject: payload),
              let json = String(data: data, encoding: .utf8) else {
            return ""
        }
        return json
    }

    func getIdentitySnippet() -> String {
        guard let identity = getFullIdentityInfo(),
              let publicKey = identity.publicKeyHex else {
            return "????????"
        }
        return String(publicKey.prefix(8))
    }

    func getIdentityDisplay() -> String {
        if let nick = getFullIdentityInfo()?.nickname {
            return nick
        }
        return getIdentitySnippet()
    }

    func getNickname() -> String? {
        return getFullIdentityInfo()?.nickname
    }

    func setNickname(_ nickname: String) async throws {
        guard let ironCore = ironCore else {
            throw MeshError.notInitialized("IronCore not initialized")
        }
        let trimmedNickname = nickname.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedNickname.isEmpty else {
            throw MeshError.invalidInput("Nickname cannot be empty")
        }
        try ironCore.setNickname(nickname: trimmedNickname)
        let updated = ironCore.getIdentityInfo()
        if updated.initialized {
            publishIdentityInfo(updated)
        }
        await persistIdentityBackupToKeychainAsync(ironCore: ironCore)
        logger.info("[OK] Nickname set to: \(trimmedNickname)")
        // If swarm start was postponed before identity/nickname was ready, resume now.
        initializeAndStartSwarm()
        broadcastIdentityBeacon()
        identitySyncSentPeers.removeAll()
        historySyncSentPeers.removeAll()
        let connectedPeers = await swarmBridge?.getPeers() ?? []
        for routePeerId in connectedPeers {
            sendIdentitySyncIfNeeded(routePeerId: routePeerId)
            sendHistorySyncIfNeeded(routePeerId: routePeerId)
        }
    }

    // MARK: - ID Coalescence Migration

    private func migrateToCanonicalIds() {
        guard let ironCore = ironCore,
              let historyManager = historyManager,
              let contactManager = contactManager else { return }

        // Use a flag in UserDefaults to run this only once
        if UserDefaults.standard.bool(forKey: "v2_id_coalescence") { return }

        logger.info("Starting ID Coalescence Migration...")

        do {
            let contacts = try contactManager.list()
            var idMap: [String: String] = [:]
            var migratedContactCount = 0

            for contact in contacts {
                if let identityId = try? ironCore.resolveIdentity(anyId: contact.publicKey),
                   identityId != contact.peerId {
                    logger.info("Migrating contact \(contact.peerId) -> \(identityId)")
                    idMap[contact.peerId] = identityId

                    if (try? contactManager.get(peerId: identityId)) == nil {
                         let newContact = Contact(
                            peerId: identityId,
                            nickname: contact.nickname,
                            localNickname: contact.localNickname,
                            publicKey: contact.publicKey,
                            addedAt: contact.addedAt,
                            lastSeen: contact.lastSeen,
                            notes: contact.notes,
                            lastKnownDeviceId: contact.lastKnownDeviceId,
                            verifiedAt: contact.verifiedAt,
                            isTombstone: contact.isTombstone
                         )
                         try contactManager.add(contact: newContact)
                    }
                    try contactManager.remove(peerId: contact.peerId)
                    migratedContactCount += 1
                }
            }

            // Migrate history
            let allMessages = try historyManager.recent(peerFilter: Optional<String>.none, limit: 100000)
            var updatedCount = 0
            for msg in allMessages {
                let canonical = idMap[msg.peerId] ?? (try? ironCore.resolveIdentity(anyId: msg.peerId))
                if let canonical = canonical, canonical != msg.peerId {
                    let updatedMsg = MessageRecord(
                        id: msg.id,
                        direction: msg.direction,
                        peerId: canonical,
                        content: msg.content,
                        timestamp: msg.timestamp,
                        senderTimestamp: msg.senderTimestamp,
                        delivered: msg.delivered,
                        status: msg.status,
                        hidden: msg.hidden
                    )
                    try historyManager.add(record: updatedMsg)
                    updatedCount += 1
                }
            }

            if updatedCount > 0 {
                logger.info("Migrated \(updatedCount) messages to canonical peer IDs")
                historyManager.flush()
            }
            if migratedContactCount > 0 {
                contactManager.flush()
                checkpointContactPersistence(reason: "canonical_contact_id_migration")
            }

            UserDefaults.standard.set(true, forKey: "v2_id_coalescence")
            logger.info("ID Coalescence Migration completed")
        } catch {
            logger.error("ID Coalescence Migration failed: \(error)")
        }
    }
}

// MARK: - Error Types

enum MeshError: LocalizedError {
    case notInitialized(String)
    case relayDisabled(String)
    case contactNotFound(String)
    case invalidInput(String)
    case alreadyRunning

    var errorDescription: String? {
        switch self {
        case .notInitialized(let msg): return msg
        case .relayDisabled(let msg): return msg
        case .contactNotFound(let msg): return msg
        case .invalidInput(let msg): return msg
        case .alreadyRunning: return "Service is already running"
        }
    }
}

private extension String {
    var nilIfEmpty: String? {
        isEmpty ? nil : self
    }
}

// MARK: - Peer ID Validation & Normalization

struct PeerIdValidator {
    /// Normalizes a Peer ID based on its type.
    /// - Identity IDs (64-char hex) are converted to lowercase.
    /// - libp2p Peer IDs (Base58) are preserved as-is (they are case-sensitive).
    static func normalize(_ id: String) -> String {
        let trimmed = id.trimmingCharacters(in: .whitespacesAndNewlines)
        if isIdentityId(trimmed) {
            return trimmed.lowercased()
        }
        return trimmed
    }

    /// Checks if the given string is a 64-character hex identity ID.
    static func isIdentityId(_ id: String) -> Bool {
        guard id.count == 64 else { return false }
        return id.unicodeScalars.allSatisfy { scalar in
            CharacterSet(charactersIn: "0123456789abcdefABCDEF").contains(scalar)
        }
    }

    /// Checks if the given string starts with common libp2p Peer ID prefixes.
    static func isLibp2pPeerId(_ id: String) -> Bool {
        return id.hasPrefix("12D3Koo") || id.hasPrefix("Qm")
    }

    /// Performs a case-sensitive comparison after normalization.
    static func isSame(_ id1: String, _ id2: String) -> Bool {
        return normalize(id1) == normalize(id2)
    }
}
