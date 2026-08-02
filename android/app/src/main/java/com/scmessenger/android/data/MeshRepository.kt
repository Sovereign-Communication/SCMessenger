package com.scmessenger.android.data

import android.content.Context
import android.content.pm.PackageManager
import android.content.SharedPreferences
import androidx.core.content.ContextCompat
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.utils.Permissions
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import com.scmessenger.android.utils.PeerIdValidator
import com.scmessenger.android.utils.PeerKeyUtils
import com.scmessenger.android.transport.TransportManager
import com.scmessenger.android.transport.SmartTransportRouter
import com.scmessenger.android.service.TransportType
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean
import timber.log.Timber
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import java.io.File

/**
 * Repository abstracting access to the Rust core via UniFFI bindings.
 *
 * This is the single source of truth for:
 * - Mesh service lifecycle
 * - Contacts management
 * - Message history
 * - Connection ledger
 * - Network settings
 *
 * All UniFFI objects are initialized lazily and managed here to ensure
 * proper lifecycle and resource cleanup.
 */
open class MeshRepository(
    private val context: Context,
    private val preferencesRepository: PreferencesRepository? = null
) {
    private val storagePath = context.filesDir.absolutePath
    private val networkFailureMetrics = NetworkFailureMetrics()
    private val transportHealthMonitor = com.scmessenger.android.transport.TransportHealthMonitor()
    private val retryBackoff = com.scmessenger.android.utils.BackoffStrategy()


    companion object {
        private const val IDENTITY_BACKUP_PREFS = "identity_backup_prefs"
        private const val IDENTITY_BACKUP_KEY = "identity_backup_v1"

        // P0: Cache key identity fields in SharedPreferences for instant UI load.
        // Eliminates 30-60s "Unavailable" gap while service starts up.
        private const val IDENTITY_CACHE_PREFS = "identity_cache_prefs"
        private const val IDENTITY_CACHE_ID = "identity_id"
        private const val IDENTITY_CACHE_PUBLIC_KEY = "public_key_hex"
        private const val IDENTITY_CACHE_DEVICE_ID = "device_id"
        private const val IDENTITY_CACHE_SENIORITY = "seniority_timestamp"
        private const val IDENTITY_CACHE_NICKNAME = "nickname"
        private const val IDENTITY_CACHE_PEER_ID = "libp2p_peer_id"
        private const val IDENTITY_CACHE_INITIALIZED = "initialized"
        // v0.4.0: there are no dedicated relays and no hardcoded node addresses --
        // every node is a full relay, and discovery is ledger sharing (invite/QR
        // seeds a new node's ledger; mDNS/BLE learns LAN peers). The static
        // "/ip4/100.56.248.69/tcp/9001" fallback and the dead BootstrapSource /
        // EnvironmentBootstrapSource plumbing that used to live here are removed;
        // see getBootstrapNodesForSettings(), ensureBootstrapRelayConnected(),
        // and racingBootstrapWithFallback() below, which now source addresses
        // from ledgerManager?.getPreferredRelays()/dialableAddresses().

        /** Cap on how many ledger-sourced relays are surfaced in the Settings UI. */
        private const val MAX_SETTINGS_RELAYS = 10u

        /** Cap on how many ledger addresses the bounded startup dial sweep attempts. */
        private const val STARTUP_DIAL_SWEEP_MAX_ADDRESSES = 10

        internal fun isMeshParticipationEnabled(settings: uniffi.api.MeshSettings?): Boolean {
            // Default to ENABLED when settings unavailable (matches Rust default: relay_enabled=true)
            return settings?.relayEnabled ?: true
        }

        /**
         * A message that has been repeatedly transport-acked but never received a
         * receipt is NOT a delivery failure - the send genuinely succeeded every
         * time. It should stop retrying eventually (battery/network), but only
         * on a patient, age-based ceiling, and never via the same
         * markMessageCorrupted() path used for genuine transport failures.
         */
        internal fun shouldStopAckedWithoutReceiptRetries(
            ackedWithoutReceiptCount: Int,
            createdAtEpochSec: Long,
            nowEpochSec: Long,
            maxAgeSeconds: Long
        ): Boolean {
            return ackedWithoutReceiptCount > 0 && (nowEpochSec - createdAtEpochSec) >= maxAgeSeconds
        }

        /**
         * Map service TransportType to SmartTransportRouter TransportType for message deduplication.
         */
        internal fun mapToSmartTransportType(transport: TransportType): SmartTransportRouter.TransportType {
            return when (transport) {
                TransportType.BLE -> SmartTransportRouter.TransportType.BLE
                TransportType.WIFI_AWARE -> SmartTransportRouter.TransportType.WIFI_DIRECT
                TransportType.WIFI_DIRECT -> SmartTransportRouter.TransportType.WIFI_DIRECT
                TransportType.INTERNET -> SmartTransportRouter.TransportType.CORE
                TransportType.TCP_MDNS -> SmartTransportRouter.TransportType.TCP_MDNS
            }
        }

        /**
         * Map SmartTransportRouter TransportType back to service TransportType.
         */
        internal fun mapFromSmartTransportType(transport: SmartTransportRouter.TransportType): TransportType {
            return when (transport) {
                SmartTransportRouter.TransportType.BLE -> TransportType.BLE
                SmartTransportRouter.TransportType.WIFI_DIRECT -> TransportType.WIFI_DIRECT
                SmartTransportRouter.TransportType.CORE -> TransportType.INTERNET
                SmartTransportRouter.TransportType.TCP_MDNS -> TransportType.TCP_MDNS
            }
        }

        internal fun checkMeshParticipationEnabled(settings: uniffi.api.MeshSettings?): Boolean {
            if (!isMeshParticipationEnabled(settings)) {
                Timber.w("Cannot send messages: mesh participation is disabled. Enable mesh participation in settings to send and receive messages.")
                return false
            }
            return true
        }

        internal fun isEnabledFlag(raw: String?): Boolean {
            return when (raw?.trim()?.lowercase()) {
                "1", "true", "yes", "on" -> true
                else -> false
            }
        }

        internal data class LocalTransportFallbackResult(
            val wifiAttempted: Boolean,
            val wifiAcked: Boolean,
            val bleAttempted: Boolean,
            val bleAcked: Boolean
        ) {
            val acked: Boolean
                get() = wifiAcked || bleAcked
        }

        internal suspend fun attemptWifiThenBleFallback(
            wifiPeerId: String?,
            blePeerId: String?,
            tryWifi: suspend (String) -> Boolean,
            tryBle: suspend (String) -> Boolean
        ): LocalTransportFallbackResult {
            val normalizedWifi = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (normalizedWifi != null) {
                if (tryWifi(normalizedWifi)) {
                    return LocalTransportFallbackResult(
                        wifiAttempted = true,
                        wifiAcked = true,
                        bleAttempted = false,
                        bleAcked = false
                    )
                }
            }

            val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (normalizedBle != null) {
                val bleAcked = tryBle(normalizedBle)
                return LocalTransportFallbackResult(
                    wifiAttempted = normalizedWifi != null,
                    wifiAcked = false,
                    bleAttempted = true,
                    bleAcked = bleAcked
                )
            }

            return LocalTransportFallbackResult(
                wifiAttempted = normalizedWifi != null,
                wifiAcked = false,
                bleAttempted = false,
                bleAcked = false
            )
        }
    }

    /** Available internal storage in MB. used for low-storage warnings. */
    fun getAvailableStorageMB(): Long {
        return com.scmessenger.android.utils.StorageManager.getAvailableStorageMB(context)
    }

    /**
     * Check if a message is a duplicate and record it for cross-transport deduplication.
     * Returns Triple(isDuplicate, timeVarianceMs, firstTransport).
     */
    private suspend fun checkAndRecordMessage(messageId: String, transport: TransportType): Triple<Boolean, Long?, TransportType?> {
        val smartTransportType = mapToSmartTransportType(transport)
        return smartTransportRouter?.checkAndRecordMessage(messageId, smartTransportType)?.let { (isDuplicate, timeVariance, firstSmartTransport) ->
            Triple(isDuplicate, timeVariance, mapFromSmartTransportType(firstSmartTransport ?: SmartTransportRouter.TransportType.CORE))
        } ?: Triple(false, null, null)
    }

    private suspend fun enhanceNetworkErrorLogging(exception: Exception, node: String) {
        val errorDetails = classifyBootstrapError(exception, node)
        Timber.w("Bootstrap failed for $node - $errorDetails")
        trackNetworkFailure(node, errorDetails, exception)
    }

    private suspend fun trackNetworkFailure(nodeId: String, reason: String, exception: Exception) {
        networkFailureMetrics.recordFailure(nodeId, reason, exception)

        // P0_ANDROID_010: Prevent recursive fallback triggering (StackOverflowError)
        // Atomic check-and-set ensures only one concurrent fallback attempt
        if (inFallbackProtocol.get()) {
            Timber.w("Skipping recursive fallback for $nodeId - already in fallback protocol")
            return
        }

        // Trigger fallback if node is marked unreachable
        if (networkFailureMetrics.isNodeUnreachable(nodeId)) {
            triggerFallbackProtocol(nodeId)
        }
    }

    /**
     * P0_ANDROID_007: When a bootstrap node becomes unreachable, attempt alternative sources.
     * Falls back through: environment override → remote config → static fallback → WebSocket on standard ports.
     */
    private suspend fun triggerFallbackProtocol(failedNodeId: String) {
        // P0_ANDROID_010: Atomic compareAnd-set ensures race-free recursion guard
        if (!inFallbackProtocol.compareAndSet(false, true)) {
            Timber.w("Skipping re-entrant fallback for $failedNodeId - already in progress")
            return
        }
        try {
            Timber.w("Node $failedNodeId marked unreachable, triggering fallback protocol")
            val fallbackNodes = emptyList<String>()
            if (fallbackNodes.isNotEmpty()) {
                Timber.i("Fallback protocol: ${fallbackNodes.size} alternative bootstrap nodes available")
                val bridge = swarmBridge ?: return
                for (addr in fallbackNodes) {
                    if (addr != failedNodeId && relayCircuitBreaker.allowRequest(addr)) {
                        try {
                            bridge.dial(addr)
                            Timber.i("Fallback bootstrap dial: %s", addr)
                            return
                        } catch (e: Exception) {
                            // Log fallback failure directly — do NOT call enhanceNetworkErrorLogging
                            // to prevent infinite recursion (trackNetworkFailure → triggerFallbackProtocol loop)
                            Timber.w(e, "Fallback bootstrap dial failed for $addr")
                        }
                    }
                }
            }
        } finally {
            inFallbackProtocol.set(false)
        }
    }

    private val identityBackupPrefs: SharedPreferences by lazy {
        context.getSharedPreferences(IDENTITY_BACKUP_PREFS, Context.MODE_PRIVATE)
    }

    // P0: Fast-access cache for identity fields. Read synchronously on UI thread
    // without needing Rust core, eliminating the 30-60s "Unavailable" gap.
    private val identityCachePrefs: SharedPreferences by lazy {
        context.getSharedPreferences(IDENTITY_CACHE_PREFS, Context.MODE_PRIVATE)
    }

    // Mesh service instance (lazy init)
    @Volatile private var meshService: uniffi.api.MeshService? = null

    // Managers (lazy init)
    @Volatile private var contactManager: uniffi.api.ContactManager? = null
    @Volatile private var historyManager: uniffi.api.HistoryManager? = null
    @Volatile private var ledgerManager: uniffi.api.LedgerManager? = null
    @Volatile private var settingsManager: uniffi.api.MeshSettingsManager? = null
    @Volatile private var autoAdjustEngine: uniffi.api.AutoAdjustEngine? = null

    // P0_NETWORK_001: Circuit breaker for relay failure tracking
    private val relayCircuitBreaker = CircuitBreaker()
    // P0_NETWORK_001: Network detector for cellular-aware transport selection
    private val networkDetector = NetworkDetector(context)
    // P0_ANDROID_007: Diagnostics reporter for connectivity analysis
    private val diagnosticsReporter = com.scmessenger.android.network.DiagnosticsReporter(
        context,
        com.scmessenger.android.network.NetworkDiagnostics(context),
        com.scmessenger.android.network.NetworkTypeDetector(context),
        networkFailureMetrics,
        networkDetector,
        relayCircuitBreaker
    )

    // Core & Network (lazy init)
    @Volatile private var ironCore: uniffi.api.IronCore? = null
    // Swarm Bridge (Internet/Libp2p)
    @Volatile private var swarmBridge: uniffi.api.SwarmBridge? = null

    // P0_ANDROID_024: Serializes identity creation to prevent the concurrent
    // initializeIdentity() race observed in production logs (4 threads in 12ms).
    // Pairs with the _isCreating gate in IdentityViewModel.
    private val identityCreationMutex = kotlinx.coroutines.sync.Mutex()

    // P0: Latch ensureLocalIdentityFederation's persistIdentityBackup to once per process
    // lifecycle. The lazy path was triggering 10+ redundant backup writes in 30s.
    private val identityBackupLatch = java.util.concurrent.atomic.AtomicBoolean(false)

    // Wifi Transport
    @Volatile private var wifiTransportManager: com.scmessenger.android.transport.WifiTransportManager? = null

    // Smart Transport Router (500ms timeout fallback + health tracking)
    @Volatile private var smartTransportRouter: com.scmessenger.android.transport.SmartTransportRouter? = null

    // Transport Manager for BLE/WiFi transport control
    @Volatile private var transportManager: TransportManager? = null

    // Topic Manager for gossipsub topic subscription and filtering
    @Volatile private var topicManager: TopicManager? = null


    // Service state
    private val _serviceState = MutableStateFlow(uniffi.api.ServiceState.STOPPED)
    open val serviceState: StateFlow<uniffi.api.ServiceState> = _serviceState.asStateFlow()

    // P0_SHARED_IDENTITY: Centralized identity StateFlow. All ViewModels (Main,
    // Identity, Settings, MeshService, Dashboard) should observe this single
    // source of truth instead of maintaining independent _identityInfo flows.
    // Publishing happens at every read/mutation point via publishIdentityInfo().
    // Replay = 1 so a late subscriber immediately sees the current value (this
    // eliminates the "first load returns null then re-fires" race in any VM
    // constructed after the identity was already hydrated).
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(readCachedIdentityFields())
    open val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()

    // P0_SHARED_IDENTITY: Publish identity changes to all subscribers. Called
    // from every code path that reads or mutates identity (getIdentityInfo,
    // getIdentityInfoNonBlocking, setNickname, createIdentity,
    // restoreIdentityFromBackup). Equality-checked so we don't churn downstream
    // collectors on structurally-equal updates (e.g., the Rust core re-emitting
    // the same IdentityInfo after a nickname set).
    private fun publishIdentityInfo(info: uniffi.api.IdentityInfo?) {
        if (_identityInfo.value != info) {
            if (info != null && info.initialized) {
                cacheIdentityFields(info)
            }
            _identityInfo.value = info
        }
    }

    // Service stats
    private val _serviceStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    open val serviceStats: StateFlow<uniffi.api.ServiceStats?> = _serviceStats.asStateFlow()

    // Message updates flow (both sent and received) used for UI updates
    private val _messageUpdates = kotlinx.coroutines.flow.MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0, extraBufferCapacity = 1, onBufferOverflow = kotlinx.coroutines.channels.BufferOverflow.DROP_OLDEST)
    open val messageUpdates = _messageUpdates.asSharedFlow()

    // Compatibility for notifications (incoming only)
    open val incomingMessages = messageUpdates.filter { it.direction == uniffi.api.MessageDirection.RECEIVED }

    private val repoScope = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO + kotlinx.coroutines.SupervisorJob())
    private var pendingOutboxRetryJob: kotlinx.coroutines.Job? = null
    private var coverTrafficJob: kotlinx.coroutines.Job? = null
    private var maintenanceJob: kotlinx.coroutines.Job? = null
    private val pendingOutboxFile = File(storagePath, "pending_outbox.json")
    private val pendingOutboxFlushMutex = kotlinx.coroutines.sync.Mutex()
    // P1: Mutex to synchronize contact upsert operations and prevent duplicate contact creation
    // during concurrent peer identification callbacks (AND-CONTACT-DUP-001)
    private val contactUpsertMutex = kotlinx.coroutines.sync.Mutex()
    // P3_ANDROID_RETRY_SUPPRESSION: Receipt ACK window expanded to 60 seconds
    // to allow for relay custody delay + network latency. Prevents premature
    // timeout-induced downgrade of successfully-sent messages.
    private val RECEIPT_ACK_TIMEOUT_MS: Long = 60_000L
    private val receiptAwaitSeconds: Long = RECEIPT_ACK_TIMEOUT_MS / 1000L  // 60 seconds
    private val pendingOutboxMaxAttempts: Int = 12
    private val pendingOutboxMaxAgeSeconds: Long = 7L * 24L * 60L * 60L
    private val historySyncSentPeers = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val HISTORY_SYNC_COOLDOWN_MS = 60_000L
    private val identitySyncSentPeers = java.util.Collections.synchronizedSet(mutableSetOf<String>())
    private val deliveredReceiptCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val deliveredReceiptCacheTtlMs = 2L * 60L * 60L * 1000L
    private val pendingReceiptSendJobs = java.util.concurrent.ConcurrentHashMap<String, kotlinx.coroutines.Job>()
    private val receiptSendMaxAttempts = 6
    @Volatile
    private var serviceStartedAtEpochSec: Long = 0L
    @Volatile
    private var lastRelayBootstrapDialMs: Long = 0L
    // P1_ANDROID_013: Track consecutive full-bootstrap failures for exponential backoff
    @Volatile
    private var consecutiveBootstrapFailures: Int = 0
    @Volatile
    private var nextBootstrapAttemptMs: Long = 0L
    private val dialThrottleState = java.util.concurrent.ConcurrentHashMap<String, Pair<Int, Long>>()

    // Reinstall detection: true when SharedPreferences has identity backup but contacts.db
    // is missing from filesDir. Triggers post-start aggressive identity beacon.
    @Volatile private var isReinstallWithMissingData = false

    // P0: Peer tracking to prevent ghost connections (700+ peers bug)
    private val transportToCanonicalMap = ConcurrentHashMap<String, String>() // libp2pPeerId -> canonicalPeerId
    private val activeSessions = ConcurrentHashMap<String, Long>() // libp2pPeerId -> lastActive
    // P0: Dedup cache — suppress redundant peer-identified callbacks for the same peer
    // within a 30-second window. The Rust core fires identify per-substream.
    private val peerIdentifiedDedupCache =
        java.util.concurrent.ConcurrentHashMap<String, Pair<String, Long>>()
    private val peerIdentifiedDedupIntervalMs = 30_000L

    // P1: Dedup cache — suppress duplicate disconnect callbacks for the same peer
    // within a 1-second window. The Rust core fires one disconnect per-substream.
    private val peerDisconnectDedupCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val peerDisconnectDedupIntervalMs = 1_000L

    // P0_ANDROID_005: Message ID tracking cache with corruption detection and recovery
    // Thread-safe ConcurrentHashMap for storing message delivery tracking information
    private val messageTrackingCache = java.util.concurrent.ConcurrentHashMap<String, MessageTracking>()

    // P0_ANDROID_005: Retry lock for thread-safe attempt count updates
    private val retryLock = kotlinx.coroutines.sync.Mutex()

    // P0_ANDROID_010: Atomic recursion guard for fallback protocol
    // Prevents enhanceNetworkErrorLogging → trackNetworkFailure → triggerFallbackProtocol
    // from re-entering concurrently. AtomicBoolean ensures check-and-set is race-free.
    private val inFallbackProtocol = AtomicBoolean(false)

    // P4: Dedup cache — suppress redundant dial-throttle log lines
    // for the same address within a 5-minute window.
    private val dialThrottleLogCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val dialThrottleLogIntervalMs = 300_000L

    // AND-SEND-BTN-001: In-memory cache for identity ID resolution.
    // canonicalContactId() calls ironCore?.resolveIdentity() via synchronous FFI on every
    // recomposition when invoked from Composable code. This cache eliminates repeated FFI calls
    // for the same peer ID, preventing UI thread freezes.
    private val identityIdCache = java.util.concurrent.ConcurrentHashMap<String, String>()

    // TCP/mDNS transport parity: Track peers discovered on LAN via libp2p mDNS.
    // Key = libp2p PeerId, Value = set of LAN multiaddresses for direct TCP delivery.
    private val mdnsLanPeers = ConcurrentHashMap<String, List<String>>()

    // Core Delegate reference to prevent GC
    @Volatile private var coreDelegate: uniffi.api.CoreDelegate? = null

    // BLE Components
    @Volatile private var bleScanner: com.scmessenger.android.transport.ble.BleScanner? = null
    @Volatile private var bleAdvertiser: com.scmessenger.android.transport.ble.BleAdvertiser? = null
    @Volatile private var bleGattServer: com.scmessenger.android.transport.ble.BleGattServer? = null
    @Volatile private var bleGattClient: com.scmessenger.android.transport.ble.BleGattClient? = null
    @Volatile private var lastBleBeaconUpdateMillis: Long = 0
    private var lastBleBeaconPayload: ByteArray = byteArrayOf()
    @Volatile private var lastBleBeaconPayloadPublishedAtMillis: Long = 0

    private data class RoutingHints(
        val wifiPeerId: String?,
        val blePeerId: String?,
        val libp2pPeerId: String?,
        val listeners: List<String>
    )

    private data class TransportIdentityResolution(
        val canonicalPeerId: String,
        val publicKey: String,
        val nickname: String?,
        val localNickname: String? = null
    )

    internal data class PendingOutboundEnvelope(
        val queueId: String,
        val historyRecordId: String,
        val peerId: String,
        val routePeerId: String?,
        val listeners: List<String>,
        val envelopeBase64: String,
        val createdAtEpochSec: Long,
        val attemptCount: Int,
        val nextAttemptAtEpochSec: Long,
        val strictBleOnlyMode: Boolean? = null,
        val recipientIdentityId: String? = null,
        val intendedDeviceId: String? = null,
        val terminalFailureCode: String? = null,
        val ackedWithoutReceiptCount: Int = 0
    )

    private data class MessageIdentityHints(
        val identityId: String?,
        val publicKey: String?,
        val deviceId: String?,
        val nickname: String?,
        val libp2pPeerId: String?,
        val listeners: List<String>,
        val externalAddresses: List<String>,
        val connectionHints: List<String>
    )

    private data class DecodedMessagePayload(
        val kind: String,
        val text: String,
        val hints: MessageIdentityHints?
    )

    // Track discovered peers (both headless and full)
    private val _discoveredPeers = MutableStateFlow<Map<String, PeerDiscoveryInfo>>(emptyMap())
    open val discoveredPeers: StateFlow<Map<String, PeerDiscoveryInfo>> = _discoveredPeers.asStateFlow()

    data class PeerDiscoveryInfo(
        val peerId: String,          // Key (libp2p or canonical)
        val publicKey: String?,      // Extracted from PeerId or from identity beacon
        val nickname: String?,       // From Identity Beacon or Contact DB
        val localNickname: String? = null,
        val libp2pPeerId: String? = null, // Libp2p peer ID for routing
        val transport: com.scmessenger.android.service.TransportType,
        val isFull: Boolean,         // True if peer identity is authenticated (non-relay)
        val isRelay: Boolean = false,
        val lastSeen: ULong = System.currentTimeMillis().toULong() / 1000u
    )

    private data class DeliveryAttemptResult(
        val acked: Boolean,
        val routePeerId: String?,
        val terminalFailureCode: String? = null
    )

    private data class ReplayDiscoveredIdentity(
        var canonicalPeerId: String,
        var publicKey: String?,
        var nickname: String?,
        var localNickname: String?,
        var routePeerId: String?,
        var transport: com.scmessenger.android.service.TransportType
    )

    private data class IdentityEmissionSignature(
        val canonicalPeerId: String,
        val publicKey: String,
        val nickname: String?,
        val libp2pPeerId: String?,
        val blePeerId: String?
    )

    private data class BleRouteObservation(
        val address: String,
        val lastSeenMs: Long,
        val source: String
    )

    /**
     * P0_ANDROID_005: Message tracking data structure.
     *
     * Tracks message delivery status, attempt counts, and corruption indicators
     * for robust message retry handling.
     */
    private data class MessageTracking(
        val messageId: String,
        var attemptCount: Int = 0,
        var lastAttemptAt: Long = 0L,
        var lastDeliveryAttemptAt: Long = 0L,
        var lastDeliveryOutcome: String? = null,
        var deliveryStatus: DeliveryStatus = DeliveryStatus.PENDING,
        var lastErrorCode: String? = null,
        var corruptionDetected: Boolean = false,
        var recoveredAt: Long? = null,
        // P3_ANDROID_RETRY_SUPPRESSION: number of transport-level acks observed.
        // Non-zero blocks the corruption downgrade in markMessageCorrupted().
        var ackedCount: Int = 0
    ) {
        /**
         * Check if this tracking entry is corrupted and needs recovery.
         * Criteria: (1) explicitly flagged as corrupted, or (2) retry overflow (> MAX_RETRY_ATTEMPTS).
         * The attemptCount check is a safety net since recordFailure() also sets corruptionDetected.
         */
        fun isCorrupted(): Boolean {
            return corruptionDetected || attemptCount > MAX_RETRY_ATTEMPTS
        }

        /**
         * Mark this tracking as corrupted.
         */
        fun markCorrupted() {
            this.corruptionDetected = true
        }

        /**
         * Record a successful delivery attempt.
         */
        fun recordSuccess() {
            this.attemptCount = 0
            this.deliveryStatus = DeliveryStatus.DELIVERED
            this.lastDeliveryOutcome = "success"
            this.lastDeliveryAttemptAt = System.currentTimeMillis()
            this.corruptionDetected = false
            this.recoveredAt = null
            this.ackedCount++
        }

        /**
         * Record a failed delivery attempt.
         */
        fun recordFailure(errorCode: String?) {
            this.attemptCount++
            this.lastAttemptAt = System.currentTimeMillis()
            this.lastErrorCode = errorCode
            this.deliveryStatus = DeliveryStatus.PENDING
            this.corruptionDetected = attemptCount > MAX_RETRY_ATTEMPTS
        }

        companion object {
            const val MAX_RETRY_ATTEMPTS = 12

            /**
             * Create a new tracking entry for a message.
             */
            fun forMessage(messageId: String): MessageTracking {
                return MessageTracking(messageId = messageId)
            }

            /**
             * Recover from corruption by resetting tracking state.
             */
            fun recoverFromCorruption(tracking: MessageTracking): MessageTracking {
                return MessageTracking(
                    messageId = tracking.messageId,
                    attemptCount = 0,
                    corruptionDetected = false,
                    recoveredAt = System.currentTimeMillis(),
                    // P3: transport-ack history survives recovery so the
                    // no-downgrade rule keeps holding for this message.
                    ackedCount = tracking.ackedCount
                )
            }
        }
    }

    /**
     * P0_ANDROID_005: Message delivery status enum.
     */
    private enum class DeliveryStatus {
        PENDING,
        DELIVERED,
        FAILED,
        EXPIRED
    }

    // Maximum retry attempts before marking message as failed
    // Defined here for external references; also in MessageTracking.Companion for inner class access
    private val MAX_RETRY_ATTEMPTS: Int = 12

    private val strictBleOnlyValidation = isEnabledFlag(System.getenv("SC_BLE_ONLY_VALIDATION"))
    private val bleRouteObservations = ConcurrentHashMap<String, BleRouteObservation>()
    // Extended from 2 minutes to 5 minutes to fix TRANSPORT-001: BLE hint staleness
    // When iOS app crashes, BLE hints become stale. Extended TTL makes fallback more resilient.
    private val bleRouteFreshnessTtlMs = 300_000L
    // Stale hint grace period: allow slightly stale hints to be used for fallback
    private val bleRouteStaleGraceMs = 600_000L  // 10 minutes

    private fun isTerminalIdentityFailure(errorCode: String?): Boolean {
        return when (errorCode?.trim()) {
            "identity_device_mismatch",
            "identity_abandoned" -> true
            else -> false
        }
    }

    private fun terminalIdentityFailureMessage(errorCode: String?): String {
        return when (errorCode?.trim()) {
            "identity_device_mismatch" ->
                "This contact's identity has been recycled onto another device. Refresh their contact details before retrying."
            "identity_abandoned" ->
                "This contact abandoned the identity you tried to reach. Re-verify the contact before sending again."
            else -> "This message was rejected because the recipient identity is no longer valid."
        }
    }

    // ========================================
    // P0_ANDROID_005: Message ID Tracking Functions
    // ========================================

    /**
     * Mark a message as corrupted in the tracking cache.
     * Called when message processing encounters unrecoverable errors.
     * P3_ANDROID_RETRY_SUPPRESSION: Guards against downgrading transport-acked messages.
     */
    fun markMessageCorrupted(messageId: String) {
        val tracking = messageTrackingCache[messageId] ?: return
        // P3: No-downgrade rule - log if attempting to corrupt an acked or delivered message
        if (tracking.ackedCount > 0 || tracking.deliveryStatus == DeliveryStatus.DELIVERED || isMessageDeliveredLocally(messageId)) {
            Timber.e("NO-DOWNGRADE VIOLATION: Attempted to mark transport-acked or delivered message $messageId as corrupted (ackedCount=${tracking.ackedCount}, status=${tracking.deliveryStatus}). Skipping corruption flag.")
            Timber.d("Message $messageId state: acked=${tracking.ackedCount}, attempts=${tracking.attemptCount}")
            return  // Prevent downgrade
        }
        tracking.markCorrupted()
        Timber.w("Message $messageId marked as corrupted (acked=false, attempts=${tracking.attemptCount})")
    }

    /**
     * Get or create message tracking for a message ID.
     * This function gracefully handles missing tracking entries by creating new ones.
     */
    private fun getMessageIdTracking(messageId: String): MessageTracking {
        return messageTrackingCache[messageId] ?: run {
            // If tracking is missing, create a fresh entry
            val tracking = MessageTracking.forMessage(messageId)
            messageTrackingCache[messageId] = tracking
            Timber.w("Message ID tracking recreated for $messageId (was missing)")
            tracking
        }
    }

    /**
     * Detect and recover from corrupted message tracking entries.
     * This prevents tracking corruption from causing retry storms.
     */
    private fun detectAndRecoverMessageTracking() {
        val corruptedIds = mutableListOf<String>()
        messageTrackingCache.forEach { (messageId, tracking) ->
            if (tracking.isCorrupted()) {
                Timber.w("Corrupted message tracking detected for $messageId, recovering")
                corruptedIds.add(messageId)
            }
        }

        // Recover corrupted entries
        for (messageId in corruptedIds) {
            val tracking = messageTrackingCache[messageId]
            if (tracking != null) {
                val recovered = MessageTracking.recoverFromCorruption(tracking)
                messageTrackingCache[messageId] = recovered
                Timber.i("Message tracking recovered for $messageId (was corrupted)")
            }
        }

        // Log if we found corrupted entries
        if (corruptedIds.isNotEmpty()) {
            Timber.w("Message tracking corruption recovery: ${corruptedIds.size} entries recovered")
        }
    }

    /**
     * Safely increment the attempt count for a message with proper synchronization.
     */
    open suspend fun incrementAttemptCount(messageId: String) {
        retryLock.withLock {
            val tracking = getMessageIdTracking(messageId)
            tracking.recordFailure(null)
        }
    }

    /**
     * P3_ANDROID_RETRY_SUPPRESSION: Record a transport-level ack for a message.
     * Feeds the no-downgrade rule in markMessageCorrupted().
     */
    private fun recordMessageTransportAck(messageId: String) {
        getMessageIdTracking(messageId).ackedCount++
    }

    /**
     * Get the next retry delay for a message based on exponential backoff.
     */
    fun getRetryDelay(attemptCount: Int): Long {
        return when (attemptCount) {
            0 -> 1000L
            1 -> 2000L
            2 -> 4000L
            3 -> 8000L
            4 -> 16000L
            else -> 30000L // Cap at 30 seconds
        }
    }

    /**
     * Check if a message should be retried based on attempt count.
     */
    fun shouldRetryMessage(messageId: String): Boolean {
        val tracking = getMessageIdTracking(messageId)
        return tracking.attemptCount < MAX_RETRY_ATTEMPTS
    }

    /**
     * Log message delivery attempt for monitoring and debugging.
     */
    fun logMessageDeliveryAttempt(messageId: String, attempt: Int, outcome: String) {
        Timber.d("Message delivery: id=$messageId, attempt=$attempt, outcome=$outcome")
    }

    /**
     * Detect and log retry storms (messages with >5 attempts).
     */
    private fun logRetryStormDetection() {
        val highRetryMessages = messageTrackingCache.values
            .filter { it.attemptCount > 5 }
            .count()

        if (highRetryMessages > 10) {
            Timber.w("Retry storm detected: $highRetryMessages messages with >5 attempts")
        }
    }

    /**
     * In-memory snapshot of the DataStore-backed nickname fallback, refreshed
     * by the collector started in init. May briefly be null on cold start
     * before the first DataStore emission; callers treat null as "no fallback".
     */
    @Volatile
    private var dataStoreNicknameSnapshot: String? = null

    init {
        Timber.d("MeshRepository initialized with storage: $storagePath")
        if (strictBleOnlyValidation) {
            Timber.w("Strict BLE-only validation mode is enabled (SC_BLE_ONLY_VALIDATION)")
        }
        checkReinstallState()
        initializeManagers()
        // Issue 6 (blocking I/O): keep an in-memory snapshot of the DataStore
        // nickname fallback fresh from a background collector. The identity
        // getters used to runBlocking { ... firstOrNull() } on every call —
        // DataStore disk I/O on whatever thread called them, including main
        // during UI composition.
        repoScope.launch {
            try {
                preferencesRepository?.identityNickname?.collect { value ->
                    dataStoreNicknameSnapshot = value
                }
            } catch (e: Exception) {
                Timber.w(e, "DataStore nickname collector failed; fallback snapshot frozen")
            }
        }
    }

    /**
     * Deferred repository initialization for heavy operations.
     * Call from a background thread (e.g., lifecycleScope on Dispatchers.IO)
     * to avoid blocking the main thread during Activity.onCreate().
     */
    fun initializeRepository() {
        try {
            startStorageMaintenance()
            Timber.i("Repository background initialization completed")
        } catch (e: Exception) {
            Timber.w(e, "Repository background initialization failed")
        }
    }

    private fun checkReinstallState() {
        // Detect reinstall: SharedPreferences identity backup exists but contacts.db is gone.
        // Triggers post-start aggressive identity beacon to recover contact info from peers.
        val hasIdentityBackup = identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)
        val contactsOnDisk = java.io.File(storagePath, "contacts.db").exists()
        val historyOnDisk = java.io.File(storagePath, "history.db").exists()

        Timber.d("AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=$hasIdentityBackup, contactsOnDisk=$contactsOnDisk, historyOnDisk=$historyOnDisk")

        if (hasIdentityBackup && (!contactsOnDisk || !historyOnDisk)) {
            isReinstallWithMissingData = true
            Timber.i("Reinstall detected: identity backup present but contacts=$contactsOnDisk history=$historyOnDisk")
            // AND-CONTACTS-WIPE-001: Log additional details for data recovery
            if (!contactsOnDisk) {
                Timber.w("AND-CONTACTS-WIPE-001: CONTACT DATA MISSING - Will attempt peer recovery")
            }
            if (!historyOnDisk) {
                Timber.w("AND-CONTACTS-WIPE-001: HISTORY DATA MISSING")
            }
        } else if (hasIdentityBackup && contactsOnDisk && historyOnDisk) {
            Timber.d("AND-CONTACTS-WIPE-001: Normal startup - all data present")
        } else if (!hasIdentityBackup) {
            Timber.d("AND-CONTACTS-WIPE-001: Fresh install - no identity backup found")
        }
    }

    private fun initializeManagers() {
        try {
            // REGRESSION FIX (AND-CONTACTS-WIPE-001): Migrate contacts BEFORE ContactManager
            // opens the new database. Previously the migration ran after construction, which
            // meant sled had the file locked and the copied data was invisible to the open handle.
            migrateContactsFromOldLocation()

            // Initialize Data Managers
            settingsManager = uniffi.api.MeshSettingsManager(storagePath)
            historyManager = uniffi.api.HistoryManager(storagePath)
            // P0_SECURITY_001: Bounded retention enforcement at startup.
            // Prune messages older than 90 days and enforce a 50k message cap.
            // The periodic maintenance loop (startStorageMaintenance) also enforces this
            // every 15 minutes, but we run it at init for immediate cleanup.
            try {
                val prunedByAge = historyManager?.pruneBefore(
                    (System.currentTimeMillis() / 1000 - 90L * 24 * 3600).toULong()
                )
                if (prunedByAge != null && prunedByAge > 0u) {
                    Timber.i("P0_SECURITY_001: Pruned $prunedByAge messages older than 90 days at startup")
                }
                val count = historyManager?.count() ?: 0u
                if (count > 50_000u) {
                    val prunedByCap = historyManager?.enforceRetention(50_000u)
                    if (prunedByCap != null && prunedByCap > 0u) {
                        Timber.i("P0_SECURITY_001: Pruned $prunedByCap messages exceeding 50k cap at startup")
                    }
                }
            } catch (e: Exception) {
                Timber.w(e, "P0_SECURITY_001: Retention enforcement at startup failed")
            }
            contactManager = uniffi.api.ContactManager(storagePath)
            ledgerManager = uniffi.api.LedgerManager(storagePath)
            autoAdjustEngine = meshService?.getAutoAdjustEngine() ?: uniffi.api.AutoAdjustEngine()

            // Initialize TransportManager for BLE/WiFi transport control
            // This will be used to enable/disable transports when settings change
            ensureTransportManager()

            // Pre-load data where applicable
            ledgerManager?.load()

            Timber.i("all_managers_init_success")
            Timber.i("All managers initialized successfully")

            // One-time migration: clear stale routing hints inherited from
            // pre-fix builds.  The old appendRoutingHint accumulated duplicate
            // BLE MACs and stale libp2p_peer_id entries that now cause endless
            // retry loops to unreachable peers.
            migrateStaleRoutingHints()

            // One-time migration: repair contacts with truncated/invalid public keys
            // that were stored before validation was added (March 2026).
            migrateTruncatedPublicKeys()

            // AND-CONTACTS-WIPE-001: Verify contact data integrity after initialization
            verifyContactDataIntegrity()

            // P0_ANDROID_001: Emergency contact corruption detection and recovery
            // Run corruption detection after managers are initialized
            repoScope.launch {
                try {
                    val corruptionDetected = detectAndRepairCorruption()
                    if (corruptionDetected) {
                        Timber.i("P0_ANDROID_001: Database corruption detected and repaired at startup")
                    }
                } catch (e: Exception) {
                    Timber.w(e, "P0_ANDROID_001: Corruption detection failed at startup")
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize managers")
        }
    }

    /**
     * Construct TransportManager if it does not already exist.
     *
     * TransportManager is nulled out by stopMeshService() (so WiFi Aware/BLE
     * sessions are released cleanly), but unlike bleScanner/bleAdvertiser/
     * bleGattServer/bleGattClient — which initializeAndStartBle() lazily
     * re-creates on every start — nothing previously re-created
     * TransportManager. Any stop (explicit, or the internal failure-recovery
     * stopMeshService() call inside startMeshService()'s own catch block)
     * therefore left it permanently null for the rest of the process
     * lifetime: every later transportManager?.startAll(...) silently no-op'd
     * with no exception and no log line, permanently disabling mDNS + the
     * SubnetProbe LAN fallback. Call this instead of constructing
     * TransportManager directly so restart-after-stop self-heals the same
     * way the BLE components already do.
     */
    private fun ensureTransportManager() {
        if (transportManager != null) return
        transportManager = TransportManager(
            context = context,
            onPeerDiscovered = { peerId, _ ->
                // TransportManager peers are handled by MeshService via onPeerDiscovered
                meshService?.onPeerDiscovered(peerId)
            },
            onDataReceived = { peerId, data, _ ->
                // TransportManager data is handled by MeshService via onDataReceived
                meshService?.onDataReceived(peerId, data)
            },
            // P1 (Bug 5): forward mDNS peer-loss to MeshService so the dedup/
            // prune/emit pipeline in onPeerDisconnected runs for mDNS-only
            // peers. Without this, mDNS-discovered peers stayed marked as
            // "connected" forever in the UI/connection state.
            onPeerDisconnected = { peerId, _ ->
                meshService?.onPeerDisconnected(peerId)
            },
            onLanAddressResolved = { multiaddr ->
                repoScope.launch {
                    try {
                        // Route through the suspend dial() wrapper (Dispatchers.IO)
                        // rather than hitting the bridge FFI directly — the FFI
                        // dial blocks its calling thread until the swarm replies.
                        dial(multiaddr)
                        Timber.i("Successfully dialed discovered LAN peer $multiaddr via SwarmBridge")
                    } catch (e: Exception) {
                        Timber.w("Failed to dial discovered LAN peer $multiaddr: ${e.message}")
                    }
                }
            },
            getLocalPeerId = {
                ironCore?.getIdentityInfo()?.libp2pPeerId
            },
            onWifiAwarePeerDiscovered = { peerId, serviceInfo, rssi ->
                meshService?.onWifiAwarePeerDiscovered(peerId, serviceInfo, rssi)
            },
            onWifiAwareDataPathConfirmed = { peerId, ipAddress, port ->
                meshService?.onWifiAwareDataPathConfirmed(peerId, ipAddress, port.toUShort())
            },
            onWifiDirectPeerDiscovered = { peerId, deviceName, deviceAddress ->
                meshService?.onWifiDirectPeerDiscovered(peerId, deviceName, deviceAddress, 0)
            },
            onWifiDirectConnectionInfo = { peerId, groupOwnerIp, isGroupOwner ->
                meshService?.onWifiDirectConnectionInfo(peerId, groupOwnerIp, isGroupOwner)
            }
        )
    }

    /**
     * AND-CONTACTS-WIPE-001: Verify contact data integrity after initialization.
     * Helps detect and log any potential data loss issues for diagnostics.
     */
    private fun verifyContactDataIntegrity() {
        try {
            val contacts = contactManager?.list().orEmpty()
            Timber.d("AND-CONTACTS-WIPE-001: Contact data verification - Found ${contacts.size} contacts")

            if (contacts.isEmpty()) {
                Timber.w("AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.")

                // Check if there are any messages - if yes, but no contacts, that's a red flag
                val messageCount = try {
                    getMessageCount()
                } catch (e: Exception) {
                    0u
                }

                if (messageCount > 0u) {
                    Timber.e("AND-CONTACTS-WIPE-001: CRITICAL - Messages exist (${messageCount} total) but no contacts found. Possible data loss scenario.")
                }
            } else {
                Timber.i("AND-CONTACTS-WIPE-001: Contact data verification successful - ${contacts.size} contacts available")

                // Log sample contact info for diagnostics (without exposing sensitive data)
                val sampleSize = minOf(5, contacts.size)
                Timber.d("AND-CONTACTS-WIPE-001: Sample contacts (${sampleSize} of ${contacts.size}):")
                contacts.take(sampleSize).forEach { contact ->
                    val peerIdPreview = contact.peerId.take(12) + if (contact.peerId.length > 12) "..." else ""
                    val hasValidKey = !contact.publicKey.isNullOrEmpty() && contact.publicKey.length >= 64
                    Timber.d("  - $peerIdPreview (added: ${contact.addedAt}, hasKey: $hasValidKey)")
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "AND-CONTACTS-WIPE-001: Failed to verify contact data integrity")
        }
    }

    /**
     * REGRESSION FIX (AND-CONTACTS-WIPE-001): Migrate contacts from old storage location.
     *
     * Issue: UniFFI contract update changed ContactManager to use "contacts.db/"
     * instead of "contacts/", causing contacts to disappear after app update.
     *
     * Solution: Copy sled database files from old to new location at file system level.
     * Must run BEFORE ContactManager construction so the new DB isn't locked.
     */
    private fun migrateContactsFromOldLocation() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_contacts_db_migration", false)) {
                Timber.d("Contacts migration already completed, skipping")
                return
            }

            Timber.i("AND-CONTACTS-WIPE-001: Starting contacts migration process")

            val oldDir = java.io.File(storagePath, "contacts")
            val newDir = java.io.File(storagePath, "contacts.db")

            Timber.d("AND-CONTACTS-WIPE-001: Checking old directory existence: ${oldDir.exists()}, isDirectory: ${oldDir.isDirectory}")

            if (!oldDir.exists() || !oldDir.isDirectory) {
                Timber.d("AND-CONTACTS-WIPE-001: Old contacts directory not found, skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                return
            }

            val oldDb = java.io.File(oldDir, "db")
            Timber.d("AND-CONTACTS-WIPE-001: Old DB exists: ${oldDb.exists()}, size: ${oldDb.length()} bytes")

            if (!oldDb.exists() || oldDb.length() == 0L) {
                Timber.d("AND-CONTACTS-WIPE-001: Old contacts database empty or missing, skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                return
            }

            // Create new directory if it doesn't exist
            if (!newDir.exists()) {
                Timber.d("AND-CONTACTS-WIPE-001: Creating new contacts directory")
                newDir.mkdirs()
            }

            // Check if new database has meaningful data (not just sled's initial empty tree)
            val newDb = java.io.File(newDir, "db")
            val newDbSize = if (newDb.exists()) newDb.length() else 0L
            Timber.d("AND-CONTACTS-WIPE-001: New DB size: $newDbSize bytes")

            // Migrate if old DB has more data than new DB, or new DB is absent/tiny.
            // Sled creates a ~4KB empty tree on open, so we use a low threshold.
            if (oldDb.length() > newDbSize || newDbSize < 4096) {
                Timber.i("AND-CONTACTS-WIPE-001: MIGRATION NEEDED - Copying contacts from old location (${oldDb.length()} bytes -> new ${newDbSize} bytes)")

                var copySuccess = true
                var copiedFiles = 0
                var totalBytes = 0L

                // Copy all sled files from old to new directory
                oldDir.listFiles()?.forEach { file ->
                    if (file.isFile) {
                        val dest = java.io.File(newDir, file.name)
                        try {
                            file.copyTo(dest, overwrite = true)
                            copiedFiles++
                            totalBytes += file.length()
                            Timber.d("AND-CONTACTS-WIPE-001: Copied ${file.name} (${file.length()} bytes)")
                        } catch (e: Exception) {
                            Timber.e(e, "AND-CONTACTS-WIPE-001: Failed to copy ${file.name}")
                            copySuccess = false
                        }
                    }
                }

                Timber.i("AND-CONTACTS-WIPE-001: Copy operation completed. Files: $copiedFiles, Total bytes: $totalBytes, Success: $copySuccess")

                if (copySuccess) {
                    Timber.i("AND-CONTACTS-WIPE-001: MIGRATION COMPLETED SUCCESSFULLY")
                    prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
                } else {
                    // Don't mark migration as complete — allow retry on next app start
                    Timber.w("AND-CONTACTS-WIPE-001: Some files failed to copy, will retry on next launch")
                }
            } else {
                Timber.d("AND-CONTACTS-WIPE-001: New database already has sufficient data ($newDbSize bytes), skipping migration")
                prefs.edit().putBoolean("v2_contacts_db_migration", true).apply()
            }

        } catch (e: Exception) {
            Timber.e(e, "AND-CONTACTS-WIPE-001: Contacts migration failed — NOT marking complete to allow retry")
            // AND-CONTACTS-WIPE-001: Do NOT mark migration as complete on failure.
            // Previously this set the flag to true on failure, preventing any retry.
        }
    }

    private fun migrateStaleRoutingHints() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v1_routing_hint_cleanup", false)) return

            val contacts = contactManager?.list().orEmpty()
            var cleaned = 0
            for (contact in contacts) {
                val notes = contact.notes ?: continue
                if (!notes.contains("libp2p_peer_id:") && !notes.contains("ble_peer_id:")) continue
                // Strip stale routing entries — fresh discovery will repopulate them.
                val stripped = notes.split(';', '\n')
                    .map { it.trim() }
                    .filter { segment ->
                        !segment.startsWith("libp2p_peer_id:") &&
                            !segment.startsWith("ble_peer_id:")
                    }
                    .joinToString(";")
                val updatedNotes = stripped.ifEmpty { null }
                if (updatedNotes != notes) {
                    contactManager?.add(uniffi.api.Contact(
                        peerId = contact.peerId,
                        nickname = contact.nickname,
                        localNickname = contact.localNickname,
                        publicKey = contact.publicKey,
                        addedAt = contact.addedAt,
                        lastSeen = contact.lastSeen,
                        notes = updatedNotes,
                        lastKnownDeviceId = null,
                        verifiedAt = null,
                        isTombstone = false
                    ))
                    cleaned++
                }
            }
            prefs.edit().putBoolean("v1_routing_hint_cleanup", true).apply()
            if (cleaned > 0) {
                Timber.i("Routing hint migration: cleaned $cleaned contact(s) with stale routing entries")
            }
        } catch (e: Exception) {
            Timber.w(e, "Routing hint migration failed (non-fatal)")
        }
    }

    /**
     * Migration: Repair contacts with truncated/invalid public keys.
     *
     * Before March 2026, addContact() had no validation, so contacts could be stored
     * with truncated keys (e.g., 8 chars like "f669fb0f" instead of 64-char hex).
     * This caused BLE decryption failures when receiving messages from those peers.
     *
     * This migration:
     * 1. Finds contacts with invalid public keys
     * 2. Attempts to repair them from discovered peers or BLE identity beacons
     * 3. If repair fails, keeps the contact but logs a warning (user can re-pair)
     */
    private fun migrateTruncatedPublicKeys() {
        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_truncated_key_migration", false)) return

            val contacts = contactManager?.list().orEmpty()
            var repaired = 0
            var unrepaired = 0

            for (contact in contacts) {
                val rawKey = contact.publicKey ?: continue
                val trimmedKey = rawKey.trim()

                // Skip contacts that already have valid keys
                if (trimmedKey.length == 64 && trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    continue
                }

                Timber.w("Migration: Found contact with invalid key (${trimmedKey.length} chars): ${contact.peerId.take(8)}...")

                // Try to find a matching discovered peer with a valid key
                val discoveredPeer = _discoveredPeers.value.entries.firstOrNull { (key, info) ->
                    // Match by peerId or by partial key match
                    info.peerId == contact.peerId ||
                    key.startsWith(trimmedKey) ||
                    trimmedKey.startsWith(key.take(8))
                }

                val discoveredKey = discoveredPeer?.value?.publicKey?.trim()
                if (discoveredKey != null && discoveredKey.length == 64 && discoveredKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    // Repair the contact with the valid key
                    Timber.i("Migration: Repairing contact ${contact.peerId.take(8)} with full key from discovered peer")
                    contactManager?.add(uniffi.api.Contact(
                        peerId = contact.peerId,
                        nickname = contact.nickname,
                        localNickname = contact.localNickname,
                        publicKey = discoveredKey,  // Use the repaired key
                        addedAt = contact.addedAt,
                        lastSeen = contact.lastSeen,
                        notes = contact.notes,
                        lastKnownDeviceId = contact.lastKnownDeviceId,
                        verifiedAt = null,
                        isTombstone = false
                    ))
                    repaired++
                } else {
                    // Can't repair - keep contact but log warning
                    // User will need to re-pair with this peer to get the full key
                    Timber.w("Migration: Cannot repair contact ${contact.peerId.take(8)} - no discovered peer with valid key. User must re-pair.")
                    unrepaired++
                }
            }

            prefs.edit().putBoolean("v2_truncated_key_migration", true).apply()
            if (repaired > 0 || unrepaired > 0) {
                Timber.i("Truncated key migration: repaired=$repaired unrepaired=$unrepaired")
            }
        } catch (e: Exception) {
            Timber.w(e, "Truncated key migration failed (non-fatal)")
        }
    }

    // ========================================================================
    // MESH SERVICE LIFECYCLE
    // ========================================================================

    // ANR FIX: Add network connectivity test to diagnose ledger relay failures
    fun testLedgerRelayConnectivity(): Boolean {
        return try {
            // Test connectivity to addresses from ledger instead of static bootstrap
            val ledgerAddresses = ledgerManager?.getPreferredRelays(3u) ?: emptyList()
            if (ledgerAddresses.isEmpty()) {
                Timber.w("Network connectivity test: No preferred relays in ledger")
                return false
            }
            
            ledgerAddresses.any { relay ->
                try {
                    // Extract IP and port from multiaddr
                    val multiaddr = relay.multiaddr ?: return@any false
                    val parts = multiaddr.split("/")
                    val ipIndex = parts.indexOf("ip4")
                    val tcpIndex = parts.indexOf("tcp")
                    if (ipIndex < 0 || tcpIndex < 0 || ipIndex + 1 >= parts.size || tcpIndex + 1 >= parts.size) {
                        return@any false
                    }
                    
                    val ip = parts[ipIndex + 1]
                    val port = parts[tcpIndex + 1].toIntOrNull() ?: return@any false
                    
                    val socket = java.net.Socket()
                    socket.connect(java.net.InetSocketAddress(ip, port), 3000)
                    socket.close()
                    Timber.d("Network connectivity test: $ip:$port reachable (ledger relay)")
                    true
                } catch (e: Exception) {
                    Timber.w("Network connectivity test: ${relay.multiaddr} unreachable - ${e.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Timber.w("Network connectivity test failed: ${e.message}")
            false
        }
    }

    /**
     * Start the mesh service with the given configuration.
     * This initializes the Rust core, starts BLE transport, and wires up events.
     */
    @Synchronized
    fun startMeshService(config: uniffi.api.MeshServiceConfig) {
        Timber.i("service_start_requested")
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING) {
            _serviceState.value = uniffi.api.ServiceState.RUNNING
            if (serviceStartedAtEpochSec == 0L) {
                serviceStartedAtEpochSec = System.currentTimeMillis() / 1000
            }
            Timber.d("MeshService is already running")
            return
        }

        try {
            Timber.d("Starting MeshService...")
            if (meshService == null) {
                // Recreate service instance after stop/failure so start is always clean.
                val logsDir = context.filesDir.absolutePath + "/logs"
                meshService = uniffi.api.MeshService.withStorageAndLogs(config, storagePath, logsDir)
            }

            // 1. Start the Rust Core service
            meshService?.start()

            // 2. Obtain shared IronCore instance
            ironCore = meshService?.getCore()
            if (ironCore == null) {
                Timber.e("Failed to obtain IronCore from MeshService")
                _serviceState.value = uniffi.api.ServiceState.STOPPED
                return
            }

            // Initialize SmartTransportRouter for intelligent transport selection
            smartTransportRouter = com.scmessenger.android.transport.SmartTransportRouter()
            Timber.i("SmartTransportRouter initialized for intelligent transport selection")

            // Initialize TopicManager for gossipsub topic subscription
            topicManager = TopicManager(this)
            topicManager?.initialize()
            Timber.i("TopicManager initialized for gossipsub topic management")

            // P0_NETWORK_001: Start network detection for cellular-aware fallback
            networkDetector.startMonitoring()
            Timber.i("NetworkDetector started — cellular-aware transport fallback active")

            // P0_NETWORK_001: Watch for network type changes and re-bootstrap
            startNetworkChangeWatch()

            // P0: One-time migration to unify legacy IDs (libp2p, etc) into canonical Identity IDs (hash)
            migrateToCanonicalIds()

            // WS12.41: Inject IronCore into FileLoggingTree for summarized logging
            timber.log.Timber.forest().forEach { tree ->
                if (tree is com.scmessenger.android.utils.FileLoggingTree) {
                    tree.setIronCore(ironCore)
                }
            }

            // WS12.41: Start storage maintenance loop
            startStorageMaintenance()

            ensureLocalIdentityFederation()

            // 3. Wire up CoreDelegate (Rust -> Android Events)
            coreDelegate = object : uniffi.api.CoreDelegate {
                override fun onPeerDiscovered(peerId: String) {
                    Timber.d("Core notified discovery: $peerId")
                    refreshAddressesSnapshots()
                    repoScope.launch {
                        val selfLibp2pPeerId = ironCore?.getIdentityInfo()?.libp2pPeerId
                        if (!selfLibp2pPeerId.isNullOrBlank() && peerId == selfLibp2pPeerId) {
                            Timber.d("Ignoring self transport discovery: $peerId")
                            return@launch
                        }

                        val isRelay = isBootstrapRelayPeer(peerId)

                        val transportIdentity = resolveTransportIdentity(peerId)
                        val extractedKey = try { ironCore?.extractPublicKeyFromPeerId(peerId) } catch (_: Exception) { null }

                        val discoveredNickname = prepopulateDiscoveryNickname(
                            nickname = transportIdentity?.nickname,
                            peerId = transportIdentity?.canonicalPeerId ?: peerId,
                            publicKey = transportIdentity?.publicKey ?: extractedKey
                        )

                        // Update discovery map
                        val discoveryInfo = PeerDiscoveryInfo(
                            peerId = transportIdentity?.canonicalPeerId ?: peerId,
                            publicKey = transportIdentity?.publicKey ?: extractedKey,
                            nickname = discoveredNickname,
                            localNickname = transportIdentity?.localNickname,
                            libp2pPeerId = peerId,
                            transport = com.scmessenger.android.service.TransportType.INTERNET, // Default for swarm
                            isFull = !isRelay && (
                                transportIdentity != null ||
                                    !extractedKey.isNullOrBlank()
                                ),
                            isRelay = isRelay,
                            lastSeen = System.currentTimeMillis().toULong() / 1000u
                        )
                        updateDiscoveredPeer(peerId, discoveryInfo)
                        if (discoveryInfo.peerId != peerId) {
                            updateDiscoveredPeer(discoveryInfo.peerId, discoveryInfo)
                        }

                        if (transportIdentity != null) {
                            val relayHints = buildDialCandidatesForPeer(
                                routePeerId = peerId,
                                rawAddresses = emptyList(),
                                includeRelayCircuits = true
                            )
                            emitIdentityDiscoveredIfChanged(
                                peerId = transportIdentity.canonicalPeerId,
                                publicKey = transportIdentity.publicKey,
                                nickname = discoveredNickname,
                                libp2pPeerId = peerId,
                                listeners = relayHints
                            )
                            annotateIdentityInLedger(
                                routePeerId = peerId,
                                listeners = relayHints,
                                publicKey = transportIdentity.publicKey,
                                nickname = discoveredNickname
                            )
                            persistRouteHintsForTransportPeer(
                                libp2pPeerId = peerId,
                                listeners = relayHints,
                                knownPublicKey = transportIdentity.publicKey
                            )
                            // Don't auto-create contacts for relay peers - they are infrastructure, not user contacts
                            if (!isRelay) {
                                upsertFederatedContact(
                                    canonicalPeerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = transportIdentity.nickname,
                                    libp2pPeerId = peerId,
                                    listeners = relayHints,
                                    createIfMissing = false
                                )
                                Timber.d("Auto-created/updated contact for discovered peer: ${transportIdentity.canonicalPeerId}")

                                // Auto-subscribe to topics for the newly discovered peer
                                topicManager?.autoSubscribeToPeerTopics(transportIdentity.canonicalPeerId)
                            } else {
                                Timber.d("Skipping contact creation for relay peer in onPeerDiscovered: $peerId")
                            }
                            try { contactManager?.updateLastSeen(transportIdentity.canonicalPeerId) } catch (_: Exception) { }
                            try { contactManager?.updateLastSeen(peerId) } catch (_: Exception) { }
                            if (!isRelay && relayHints.isNotEmpty()) {
                                connectToPeer(peerId, relayHints)
                            }
                        }
                    }
                }

	                override fun onPeerIdentified(peerId: String, agentVersion: String, listenAddrs: List<String>) {
                    // P0: Deduplicate peer-identified events — Rust core fires one per substream
                    val trimmedPeerId = peerId.trim()
                    val identifySignature = (
                        listOf(agentVersion.trim()) +
                            listenAddrs.map { it.trim() }.filter { it.isNotEmpty() }.sorted()
                        ).joinToString("|")
                    val now = System.currentTimeMillis()
                    val lastIdentified = peerIdentifiedDedupCache[trimmedPeerId]
                    if (
                        lastIdentified != null &&
                        lastIdentified.first == identifySignature &&
                        (now - lastIdentified.second) < peerIdentifiedDedupIntervalMs
                    ) {
                        return
                    }
                    peerIdentifiedDedupCache[trimmedPeerId] = identifySignature to now

                    Timber.d("Core notified identified: $peerId (agent: $agentVersion) with ${listenAddrs.size} addresses")
                    refreshAddressesSnapshots()
                    // Record successful transport event for health monitoring
                    val transport = when {
                        listenAddrs.any { it.contains("/ble/") } -> "ble"
                        listenAddrs.any { it.contains("/ws/") } -> "websocket"
                        listenAddrs.any { it.contains("/quic") } -> "quic"
                        listenAddrs.any { it.contains("/tcp/") } -> "tcp"
                        else -> "unknown"
                    }
                    recordTransportEvent(transport, true)
                    repoScope.launch {
                        dialThrottleState.keys
                            .filter { it.endsWith("/p2p/$peerId") || it == peerId }
                            .forEach { dialThrottleState.remove(it) }
                        val selfLibp2pPeerId = ironCore?.getIdentityInfo()?.libp2pPeerId
                        if (!selfLibp2pPeerId.isNullOrBlank() && peerId == selfLibp2pPeerId) {
                            Timber.d("Ignoring self transport identity: $peerId")
                            return@launch
                        }

                        // TCP/mDNS parity: Detect LAN addresses (RFC1918) from listen_addrs.
                        // If any private-network TCP/QUIC address is present, this peer
                        // was discovered on the local network (typically via libp2p mDNS).
                        val lanAddrs = listenAddrs.filter { addr ->
                            val a = addr.trim()
                            val isPrivateIp = a.startsWith("/ip4/192.168.") ||
                             a.startsWith("/ip4/10.") ||
                             (a.startsWith("/ip4/172.") && run {
                                 val parts = a.removePrefix("/ip4/").split(".")
                                 val secondOctet = parts.getOrNull(1)?.toIntOrNull() ?: 0
                                 secondOctet in 16..31
                             })
                            isPrivateIp && (a.contains("/tcp/") || a.contains("/udp/"))
                        }
                        if (lanAddrs.isNotEmpty()) {
                            mdnsLanPeers[trimmedPeerId] = lanAddrs
                            Timber.i("TCP/mDNS: LAN peer detected $trimmedPeerId with ${lanAddrs.size} local addresses")
                        } else {
                            mdnsLanPeers.remove(trimmedPeerId)
                        }

	                        val dialCandidates = buildDialCandidatesForPeer(
	                            routePeerId = peerId,
	                            rawAddresses = listenAddrs,
	                            includeRelayCircuits = true
	                        )

	                        val syncPeerIds = linkedSetOf(peerId.trim())
	                        val isHeadless = agentVersion.contains("/headless/")
	                        val transportIdentity = resolveTransportIdentity(peerId)
	                        val shouldTreatAsHeadless = isBootstrapRelayPeer(peerId) || (isHeadless && transportIdentity == null)
	                        if (shouldTreatAsHeadless) {
	                            Timber.i("Headless/Relay transport node identified: $peerId (agent: $agentVersion)")
	                            emitConnectedIfChanged(
	                                peerId = peerId,
	                                transport = com.scmessenger.android.service.TransportType.INTERNET
	                            )
	                            transportToCanonicalMap[peerId] = peerId // Relay counts as its own canonical
	                            activeSessions[peerId] = System.currentTimeMillis()
	                        } else {
	                            if (isHeadless && transportIdentity != null) {
	                                Timber.i("Promoting peer $peerId to full node: identity resolved despite headless agent $agentVersion")
	                            }
	                            if (transportIdentity != null) {
	                                syncPeerIds.add(transportIdentity.canonicalPeerId)
	                        }
                            val canonicalId = transportIdentity?.canonicalPeerId ?: peerId
                            transportToCanonicalMap[peerId] = canonicalId
                            activeSessions[peerId] = System.currentTimeMillis()

                            val discoveredNickname = prepopulateDiscoveryNickname(
                                nickname = transportIdentity?.nickname,
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey
                            )

                            // Update discovery map
                            val peerTransportType = if (mdnsLanPeers.containsKey(trimmedPeerId))
                                com.scmessenger.android.service.TransportType.TCP_MDNS
                            else
                                com.scmessenger.android.service.TransportType.INTERNET
                            val discoveryInfo = PeerDiscoveryInfo(
                                peerId = transportIdentity?.canonicalPeerId ?: peerId,
                                publicKey = transportIdentity?.publicKey,
                                nickname = discoveredNickname,
                                localNickname = transportIdentity?.localNickname,
                                libp2pPeerId = peerId,
                                transport = peerTransportType,
                                isFull = transportIdentity != null,
                                isRelay = isBootstrapRelayPeer(peerId),
                                lastSeen = System.currentTimeMillis().toULong() / 1000u
                            )
                            updateDiscoveredPeer(peerId, discoveryInfo)
                            if (discoveryInfo.peerId != peerId) {
                                updateDiscoveredPeer(discoveryInfo.peerId, discoveryInfo)
                            }

                            if (transportIdentity != null) {
                                emitIdentityDiscoveredIfChanged(
                                    peerId = transportIdentity.canonicalPeerId,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = discoveredNickname,
                                    libp2pPeerId = peerId,
                                    listeners = dialCandidates
                                )
                                annotateIdentityInLedger(
                                    routePeerId = peerId,
                                    listeners = dialCandidates,
                                    publicKey = transportIdentity.publicKey,
                                    nickname = discoveredNickname
                                )
                                try { contactManager?.updateLastSeen(transportIdentity.canonicalPeerId) } catch (_: Exception) { }
                                try { contactManager?.updateLastSeen(peerId) } catch (_: Exception) { }
                            } else {
                                Timber.d("Transport identity unavailable for $peerId")
                                // FIX: Auto-create contact even when transportIdentity is null
                                // This happens on fresh install when no contact exists yet
                                if (!isBootstrapRelayPeer(peerId) && !isHeadless) {
                                    val extractedKey = try {
                                        ironCore?.extractPublicKeyFromPeerId(peerId)
                                    } catch (_: Exception) { null }

                                    if (extractedKey != null) {
                                        val normalizedKey = normalizePublicKey(extractedKey)
                                        if (normalizedKey != null) {
                                            // ID-STANDARDIZATION-002: Check for existing contact before auto-creating
                                            val existingContact = try {
                                                contactManager?.list()?.firstOrNull {
                                                    normalizePublicKey(it.publicKey) == normalizedKey
                                                }
                                            } catch (e: Exception) {
                                                null
                                            }
                                            
                                            if (existingContact != null) {
                                                Timber.i("Contact already exists for public key ${normalizedKey.take(8)}... (peerId=${existingContact.peerId}), skipping auto-creation")
                                                // Update last seen for existing contact
                                                try { contactManager?.updateLastSeen(existingContact.peerId) } catch (_: Exception) {}
                                            } else {
                                                // ID-STANDARDIZATION-003: Use proper canonical ID resolution
                                                val canonicalId = try {
                                                    validateAndStandardizeId(peerId, "auto-contact-creation")
                                                } catch (e: Exception) {
                                                    Timber.w("Using fallback peerId for contact creation: ${e.message}")
                                                    PeerIdValidator.normalize(peerId)
                                                }
                                                
                                                Timber.i("Auto-creating contact for newly discovered peer: $peerId -> $canonicalId (extracted key: ${normalizedKey.take(8)}...)")
                                                repoScope.launch {
                                                    upsertFederatedContact(
                                                        canonicalPeerId = canonicalId,
                                                        publicKey = normalizedKey,
                                                        nickname = null,
                                                        libp2pPeerId = peerId,
                                                        listeners = dialCandidates,
                                                        createIfMissing = true
                                                    )
                                                }
                                            }
                                        }
                                    } else {
                                        Timber.w("Could not extract public key from peer $peerId for auto-contact creation")
                                    }
                                }
                            }
                            emitConnectedIfChanged(
                                peerId = peerId,
                                transport = com.scmessenger.android.service.TransportType.INTERNET
                            )
                            persistRouteHintsForTransportPeer(
                                libp2pPeerId = peerId,
                                listeners = dialCandidates,
                                knownPublicKey = transportIdentity?.publicKey
                            )
                            if (transportIdentity != null) {
                                // Don't auto-create contacts for relay peers - they are infrastructure, not user contacts
                                if (!isBootstrapRelayPeer(peerId)) {
                                    upsertFederatedContact(
                                        canonicalPeerId = transportIdentity.canonicalPeerId,
                                        publicKey = transportIdentity.publicKey,
                                        nickname = transportIdentity.nickname,
                                        libp2pPeerId = peerId,
                                        listeners = dialCandidates,
                                        createIfMissing = true  // AUTO-CREATE contacts for all discovered peers
                                    )
                                    Timber.i("Auto-created/updated contact for peer: ${transportIdentity.canonicalPeerId} (nickname: ${transportIdentity.nickname})")
                                } else {
                                    Timber.d("Skipping contact creation for relay peer: $peerId")
                                }
                            }
                            sendIdentitySyncIfNeeded(
                                routePeerId = peerId,
                                knownPublicKey = transportIdentity?.publicKey
                            )
                            sendHistorySyncIfNeeded(
                                routePeerId = peerId,
                                knownPublicKey = transportIdentity?.publicKey
                            )
	                    }

                        // Identified implies an active session exists; avoid immediate re-dial loops.
                        syncPeerIds
                            .map { it.trim() }
                            .filter { it.isNotEmpty() }
                            .forEach { promotePendingOutboundForPeer(peerId = it) }
                        flushPendingOutbox("peer_identified:$peerId")
                        updateBleIdentityBeacon()
                    }
                }

                override fun onPeerDisconnected(peerId: String) {
                    // P0: Deduplicate disconnect events — libp2p may fire multiple per session
                    val trimmedPeerId = peerId.trim()
                    val now = System.currentTimeMillis()
                    val lastDisconnected = peerDisconnectDedupCache[trimmedPeerId]
                    if (lastDisconnected != null && (now - lastDisconnected) < peerDisconnectDedupIntervalMs) {
                        return
                    }
                    peerDisconnectDedupCache[trimmedPeerId] = now

                    Timber.d("Core notified disconnected: $peerId")
                    refreshAddressesSnapshots()
                    // Record transport failure event for health monitoring
                    recordTransportEvent("mesh", false)
                    repoScope.launch {
                        val canonicalId = transportToCanonicalMap.remove(peerId) ?: peerId
                        activeSessions.remove(peerId)

                        // Remove disconnected aliases (peerId + canonical + same-key aliases).
                        pruneDisconnectedPeer(peerId)
                        if (canonicalId != peerId) {
                            pruneDisconnectedPeer(canonicalId)
                        }
                        mdnsLanPeers.remove(trimmedPeerId)

                        emitDisconnectedIfChanged(
                            peerId = peerId
                        )
                        if (canonicalId != peerId) {
                            emitDisconnectedIfChanged(
                                peerId = canonicalId
                            )
                        }
                    }
                }

                override fun onMessageReceived(
                    senderId: String,
                    senderPublicKeyHex: String,
                    messageId: String,
                    senderTimestamp: ULong,
                    data: ByteArray
                ) {
                    Timber.i("Message from $senderId: $messageId")
                    repoScope.launch {
                        // Check for duplicate messages across transports
                        val (isDuplicate, timeVarianceMs, firstTransport) = checkAndRecordMessage(messageId, TransportType.INTERNET)
                        if (isDuplicate) {
                            Timber.i("Message duplicate detected (core transport): $messageId time_variance=${timeVarianceMs}ms first_transport=$firstTransport")
                            return@launch
                        }
                    }

                    logDeliveryAttempt(
                        messageId = messageId,
                        medium = "core",
                        phase = "rx",
                        outcome = "received",
                        detail = "sender=$senderId",
                        callerContext = "onMessageReceived"
                    )
                    try {
                        // Check if relay/messaging is enabled (bidirectional control)
                        // Treat null/missing settings as disabled (fail-safe)
                        // Cache settings value to avoid race condition during check
                        val currentSettings = settingsManager?.load()
                        if (!Companion.isMeshParticipationEnabled(currentSettings)) {
                            Timber.w("Dropping received message - mesh participation is disabled or settings unavailable")
                            return
                        }

                        val normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
                        val rawContent = data.toString(Charsets.UTF_8)
                        val decodedPayload = decodeMessageWithIdentityHints(rawContent)
                        val hintedIdentity = decodedPayload.hints
                        val hintedKey = normalizePublicKey(hintedIdentity?.publicKey)
                        val verifiedHints = if (
                            hintedKey != null &&
                                normalizedSenderKey != null &&
                                hintedKey != normalizedSenderKey
                        ) {
                            Timber.w(
                                "Ignoring forged message identity hint for $senderId: key mismatch " +
                                    "(hint=${hintedKey.take(8)}..., envelope=${normalizedSenderKey.take(8)}...)"
                            )
                            null
                        } else {
                            hintedIdentity
                        }

                        var canonicalPeerId = resolveCanonicalPeerId(senderId, senderPublicKeyHex)
                        canonicalPeerId = resolveCanonicalPeerIdFromMessageHints(
                            resolvedCanonicalPeerId = canonicalPeerId,
                            senderId = senderId,
                            senderPublicKeyHex = senderPublicKeyHex,
                            hintedIdentityId = verifiedHints?.identityId
                        )

                        val hintedRoutePeerId = verifiedHints?.libp2pPeerId
                            ?.trim()
                            ?.takeIf { PeerIdValidator.isLibp2pPeerId(it) }
                        val routeWifiPeerId = senderId.takeIf { isWifiPeerId(it) }
                        val routePeerId = senderId.takeIf { PeerIdValidator.isLibp2pPeerId(it) } ?: hintedRoutePeerId
                        val hintedAddresses = (
                            verifiedHints?.listeners.orEmpty() +
                                verifiedHints?.externalAddresses.orEmpty() +
                                verifiedHints?.connectionHints.orEmpty()
                            )
                        val hintedDialCandidates = buildDialCandidatesForPeer(
                            routePeerId = routePeerId,
                            rawAddresses = hintedAddresses,
                            includeRelayCircuits = true
                        )
                        val knownNickname = selectAuthoritativeNickname(
                            verifiedHints?.nickname,
                            resolveKnownPeerNickname(
                                canonicalPeerId = canonicalPeerId,
                                routePeerId = routePeerId,
                                publicKey = normalizedSenderKey
                            )
                        )
                        if (canonicalPeerId != senderId) {
                            Timber.i("Canonicalized sender $senderId -> $canonicalPeerId using public key match")
                        }

                        if (isBootstrapRelayPeer(canonicalPeerId)) {
                            Timber.i("Ignoring payload attributed to bootstrap relay peer $canonicalPeerId")
                            return
                        }

                        // Auto-upsert contact: senderPublicKeyHex is guaranteed valid Ed25519 key
                        // (Rust only fires this callback after successful decryption)
                        val messageKind = decodedPayload.kind.trim().lowercase()
                        val isChatEvent = messageKind == "text" || messageKind.isEmpty()

                        val existingContact = try { contactManager?.get(canonicalPeerId) } catch (e: Exception) { null }
                        if (existingContact == null && normalizedSenderKey != null && isChatEvent) {
                            var routeNotes = if (!routePeerId.isNullOrBlank()) {
                                appendRoutingHint(notes = null, key = "libp2p_peer_id", value = routePeerId)
                            } else {
                                null
                            }
                            routeNotes = appendRoutingHint(
                                notes = routeNotes,
                                key = "wifi_peer_id",
                                value = routeWifiPeerId
                            )
                            routeNotes = upsertRoutingListeners(
                                routeNotes,
                                normalizeOutboundListenerHints(hintedDialCandidates)
                            )
                             val autoContact = uniffi.api.Contact(
                                peerId = canonicalPeerId,
                                nickname = knownNickname,
                                localNickname = null,
                                publicKey = normalizedSenderKey,
                                addedAt = (System.currentTimeMillis() / 1000).toULong(),
                                lastSeen = (System.currentTimeMillis() / 1000).toULong(),
                                notes = routeNotes,
                                lastKnownDeviceId = verifiedHints?.deviceId,
                                verifiedAt = null,
                                isTombstone = false
                            )
                            try {
                                contactManager?.add(autoContact)
                                Timber.i("Auto-created contact from received message: ${canonicalPeerId.take(8)} key: ${senderPublicKeyHex.take(8)}...")
                            } catch (e: Exception) {
                                Timber.w("Auto-create contact failed for ${canonicalPeerId.take(8)}: ${e.message}")
                            }
                        } else if (existingContact != null) {
                            try { contactManager?.updateLastSeen(canonicalPeerId) } catch (e: Exception) {
                                Timber.d("updateLastSeen failed: ${e.message}")
                            }

                            // Update device ID when discovered from message hints
                            val discoveredDeviceId = verifiedHints?.deviceId
                            if (discoveredDeviceId != null && discoveredDeviceId != existingContact.lastKnownDeviceId) {
                                updateContactDeviceId(canonicalPeerId, discoveredDeviceId)
                            }

                            if (existingContact.nickname.isNullOrBlank() && !knownNickname.isNullOrBlank()) {
                                val updatedContact = uniffi.api.Contact(
                                    peerId = existingContact.peerId,
                                    nickname = knownNickname,
                                    localNickname = existingContact.localNickname,
                                    publicKey = existingContact.publicKey,
                                    addedAt = existingContact.addedAt,
                                    lastSeen = existingContact.lastSeen,
                                    notes = existingContact.notes,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId,
                                    verifiedAt = existingContact.verifiedAt,
                                    isTombstone = false
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist nickname hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }

                            val currentRouting = parseRoutingHints(existingContact.notes)
                            val normalizedRoutePeerId = routePeerId?.trim()?.takeIf { it.isNotEmpty() }
                            val normalizedRouteWifiPeerId = routeWifiPeerId?.trim()?.takeIf { it.isNotEmpty() }

                            // Persist updated libp2p alias mapping when known so identity/libp2p IDs
                            // stay canonicalized to one conversation thread across peer-id rotations.
                            if (!normalizedRoutePeerId.isNullOrBlank() &&
                                normalizedSenderKey != null &&
                                normalizePublicKey(existingContact.publicKey) == normalizedSenderKey &&
                                currentRouting.libp2pPeerId?.trim() != normalizedRoutePeerId
                            ) {
                                val updatedNotes = appendRoutingHint(existingContact.notes, "libp2p_peer_id", routePeerId)
                                val updatedNotesWithListeners = upsertRoutingListeners(
                                    updatedNotes,
                                    normalizeOutboundListenerHints(hintedDialCandidates)
                                )
                                val updatedContact = uniffi.api.Contact(
                                    peerId = existingContact.peerId,
                                    nickname = existingContact.nickname,
                                    localNickname = existingContact.localNickname,
                                    publicKey = existingContact.publicKey,
                                    addedAt = existingContact.addedAt,
                                    lastSeen = existingContact.lastSeen,
                                    notes = updatedNotesWithListeners,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId,
                                    verifiedAt = existingContact.verifiedAt,
                                    isTombstone = false
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist libp2p alias hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }

                            if (!normalizedRouteWifiPeerId.isNullOrBlank() &&
                                normalizedSenderKey != null &&
                                normalizePublicKey(existingContact.publicKey) == normalizedSenderKey &&
                                currentRouting.wifiPeerId?.trim() != normalizedRouteWifiPeerId
                            ) {
                                val updatedNotes = appendRoutingHint(existingContact.notes, "wifi_peer_id", routeWifiPeerId)
                                val updatedNotesWithListeners = upsertRoutingListeners(
                                    updatedNotes,
                                    normalizeOutboundListenerHints(hintedDialCandidates)
                                )
                                val updatedContact = uniffi.api.Contact(
                                    peerId = existingContact.peerId,
                                    nickname = existingContact.nickname,
                                    localNickname = existingContact.localNickname,
                                    publicKey = existingContact.publicKey,
                                    addedAt = existingContact.addedAt,
                                    lastSeen = existingContact.lastSeen,
                                    notes = updatedNotesWithListeners,
                                    lastKnownDeviceId = verifiedHints?.deviceId ?: existingContact.lastKnownDeviceId,
                                    verifiedAt = existingContact.verifiedAt,
                                    isTombstone = false
                                )
                                try {
                                    contactManager?.add(updatedContact)
                                } catch (e: Exception) {
                                    Timber.d("Failed to persist WiFi alias hint for ${existingContact.peerId}: ${e.message}")
                                }
                            }
                        }

                        if (normalizedSenderKey != null) {
                            repoScope.launch {
                                upsertFederatedContact(
                                    canonicalPeerId = canonicalPeerId,
                                    publicKey = normalizedSenderKey,
                                    nickname = knownNickname,
                                    libp2pPeerId = routePeerId,
                                    wifiPeerId = routeWifiPeerId,
                                    listeners = hintedDialCandidates,
                                    deviceId = verifiedHints?.deviceId,
                                    createIfMissing = false
                                )
                            }
                            val discoveredNickname = prepopulateDiscoveryNickname(
                                nickname = knownNickname,
                                peerId = canonicalPeerId,
                                publicKey = normalizedSenderKey
                            )
                            val discoveryInfo = PeerDiscoveryInfo(
                                peerId = canonicalPeerId,
                                publicKey = normalizedSenderKey,
                                nickname = discoveredNickname,
                                localNickname = existingContact?.localNickname,
                                libp2pPeerId = routePeerId,
                                transport = com.scmessenger.android.service.TransportType.INTERNET,
                                isFull = true,
                                lastSeen = (System.currentTimeMillis() / 1000).toULong()
                            )
                            updateDiscoveredPeer(canonicalPeerId, discoveryInfo)
                            if (!routePeerId.isNullOrBlank() && routePeerId != canonicalPeerId) {
                                updateDiscoveredPeer(routePeerId, discoveryInfo)
                            }
                            val listeners = (
                                routePeerId?.let(::getDialHintsForRoutePeer).orEmpty() +
                                    hintedDialCandidates
                                ).distinct()
                            repoScope.launch {
                                emitIdentityDiscoveredIfChanged(
                                    peerId = canonicalPeerId,
                                    publicKey = normalizedSenderKey,
                                    nickname = discoveredNickname,
                                    libp2pPeerId = routePeerId,
                                    listeners = listeners
                                )
                            }
                            annotateIdentityInLedger(
                                routePeerId = routePeerId,
                                listeners = listeners,
                                publicKey = normalizedSenderKey,
                                nickname = discoveredNickname
                            )
                        }

                        if (messageKind == "identity_sync") {
                            Timber.d("Processed identity sync from $canonicalPeerId (route=$routePeerId)")
                            sendDeliveryReceiptAsync(
                                senderPublicKeyHex = senderPublicKeyHex,
                                messageId = messageId,
                                senderId = canonicalPeerId,
                                preferredRoutePeerId = routePeerId,
                                preferredWifiPeerId = routeWifiPeerId,
                                preferredListenerHints = hintedDialCandidates
                            )
                            return
                        }

                        if (messageKind == "history_sync") {
                            Timber.d("Processed history sync request from $canonicalPeerId")
                            sendHistorySyncDataIfNeeded(canonicalPeerId, routePeerId, senderPublicKeyHex, hintedDialCandidates, routeWifiPeerId)
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, canonicalPeerId, routePeerId, routeWifiPeerId, preferredListenerHints = hintedDialCandidates)
                            return
                        }
                        if (messageKind == "history_sync_data") {
                            Timber.d("Processed history sync data from $canonicalPeerId")
                            try {
                                val arr = org.json.JSONArray(decodedPayload.text)
                                for (i in 0 until arr.length()) {
                                    val obj = arr.getJSONObject(i)
                                    val msgId = obj.getString("id")
                                    val existing = historyManager?.get(msgId)
                                    if (existing == null) {
                                        val record = uniffi.api.MessageRecord(
                                            id = msgId,
                                            direction = if (obj.getString("dir") == "sent") uniffi.api.MessageDirection.RECEIVED else uniffi.api.MessageDirection.SENT,
                                            peerId = canonicalPeerId,
                                            content = obj.getString("txt"),
                                            timestamp = obj.getLong("ts").toULong(),
                                            senderTimestamp = obj.getLong("sts").toULong(),
                                            delivered = obj.getBoolean("del"),
                                            status = if (obj.getBoolean("del")) uniffi.api.MessageStatus.DELIVERED else uniffi.api.MessageStatus.SENT,
                                            hidden = false
                                        )
                                        historyManager?.add(record)
                                        repoScope.launch { _messageUpdates.emit(record) }
                                    } else {
                                        val dirStr = obj.optString("dir")
                                        val isPeerReflectingRecv = dirStr == "recv"
                                        if (existing.direction == uniffi.api.MessageDirection.SENT && !existing.delivered && isPeerReflectingRecv) {
                                            historyManager?.markDelivered(msgId)
                                            val updated = historyManager?.get(msgId)
                                            if (updated != null) {
                                                repoScope.launch { _messageUpdates.emit(updated) }
                                                repoScope.launch {
                                                    com.scmessenger.android.service.MeshEventBus.emitMessageEvent(
                                                        com.scmessenger.android.service.MessageEvent.Delivered(msgId)
                                                    )
                                                }
                                            }
                                            removePendingOutbound(msgId)
                                        }
                                    }
                                }
                                historyManager?.flush()
                            } catch (e: Exception) { Timber.e(e, "Failed to parse history_sync_data") }
                            sendDeliveryReceiptAsync(senderPublicKeyHex, messageId, canonicalPeerId, routePeerId, routeWifiPeerId, preferredListenerHints = hintedDialCandidates)
                            return
                        }

                        val existingRecord = try {
                            historyManager?.get(messageId)
                        } catch (_: Exception) {
                            null
                        }
                        if (existingRecord?.direction == uniffi.api.MessageDirection.RECEIVED) {
                            Timber.d("Duplicate inbound message $messageId from $senderId; acknowledging without re-emitting UI")
                            sendDeliveryReceiptAsync(
                                senderPublicKeyHex = senderPublicKeyHex,
                                messageId = messageId,
                                senderId = canonicalPeerId,
                                preferredRoutePeerId = routePeerId,
                                preferredWifiPeerId = routeWifiPeerId,
                                preferredListenerHints = hintedDialCandidates
                            )
                            return
                        }

                        val content = decodedPayload.text
                        val fallbackNow = (System.currentTimeMillis() / 1000).toULong()
                        val canonicalTimestamp = if (senderTimestamp > 0uL) senderTimestamp else fallbackNow
                        val record = uniffi.api.MessageRecord(
                            id = messageId,
                            direction = uniffi.api.MessageDirection.RECEIVED,
                            peerId = canonicalPeerId,
                            content = content,
                            timestamp = canonicalTimestamp,
                            senderTimestamp = senderTimestamp,
                            delivered = true,
                            status = uniffi.api.MessageStatus.DELIVERED,
                            hidden = false
                        )
                        historyManager?.add(record)
                        logDeliveryAttempt(
                            messageId = messageId,
                            medium = "core",
                            phase = "rx",
                            outcome = "processed",
                            detail = "stored_in_history sender=$senderId",
                            callerContext = "onMessageReceived_processing"
                        )

                        // Emit for notifications and UI updates
                        repoScope.launch {
                            _messageUpdates.emit(record)
                        }

                        // Send delivery receipt ACK back to sender.
                        sendDeliveryReceiptAsync(
                            senderPublicKeyHex = senderPublicKeyHex,
                            messageId = messageId,
                            senderId = canonicalPeerId,
                            preferredRoutePeerId = routePeerId,
                            preferredWifiPeerId = routeWifiPeerId,
                            preferredListenerHints = hintedDialCandidates
                        )
                    } catch (e: Exception) {
                        Timber.e(e, "Failed to process received message")
                    }
                }

                override fun onReceiptReceived(messageId: String, status: String) {
                    // [VERBOSE] Log: Receipt arrived from core
                    Timber.i(
                        "[RECEIPT-RX] Received from core: msg=$messageId status=$status"
                    )

                    val normalized = status.trim().lowercase()
                    Timber.d("[RECEIPT-RX] Normalized status: $normalized")

                    if (normalized != "delivered" && normalized != "read") {
                        Timber.w(
                            "[RECEIPT-RX] IGNORING: Unknown delivery status: " +
                            "msg=$messageId status=$status normalized=$normalized"
                        )
                        logDeliveryAttempt(
                            messageId = messageId,
                            medium = "receipt",
                            phase = "rx",
                            outcome = "ignored_unknown_status",
                            detail = "status=$status normalized=$normalized"
                        )
                        return
                    }

                    // Receipt arrival is authoritative over tracking state / retry counter
                    getMessageIdTracking(messageId).recordSuccess()
                    // [VERBOSE] Step 1: Look up message in history
                    val existingRecord = try {
                        val rec = historyManager?.get(messageId)
                        Timber.d(
                            "[RECEIPT-RX] History lookup: msg=$messageId " +
                            "found=${rec != null} direction=${rec?.direction}"
                        )
                        rec
                    } catch (e: Exception) {
                        Timber.e(
                            e,
                            "[ERROR] History lookup FAILED: msg=$messageId error=${e.message}"
                        )
                        null
                    }

                    // [VERBOSE] Step 2: Check pending outbox
                    val hasPendingForReceipt = loadPendingOutbox().any { it.historyRecordId == messageId }
                    Timber.d(
                        "[RECEIPT-RX] Pending outbox check: msg=$messageId " +
                        "has_pending=$hasPendingForReceipt existing=${existingRecord != null}"
                    )

                    if (existingRecord == null || existingRecord.direction != uniffi.api.MessageDirection.SENT) {
                        Timber.w(
                            "[RECEIPT-RX] INVALID: Receipt for non-sent message: " +
                            "msg=$messageId direction=${existingRecord?.direction ?: "null"}"
                        )
                        if (hasPendingForReceipt) {
                            Timber.i(
                                "[RECEIPT-RX] RECOVERY: Found in pending outbox, marking delivered: msg=$messageId"
                            )
                            removePendingOutbound(messageId)
                            logDeliveryState(
                                messageId = messageId,
                                state = "delivered",
                                detail = "delivery_receipt_recovered_without_history status=$normalized direction=${existingRecord?.direction ?: "missing"}"
                            )
                            return
                        }
                        Timber.d(
                            "[RECEIPT-RX] IGNORING: No pending outbox entry: " +
                            "msg=$messageId status=$normalized direction=${existingRecord?.direction ?: "missing"}"
                        )
                        logDeliveryState(
                            messageId = messageId,
                            state = "pending",
                            detail = "delivery_receipt_ignored_non_outbound status=$normalized direction=${existingRecord?.direction ?: "missing"}"
                        )
                        return
                    }
                    // [VERBOSE] Step 3: Check if already delivered
                    val wasAlreadyDelivered = existingRecord.delivered
                    Timber.d(
                        "[RECEIPT-RX] Delivery state check: msg=$messageId " +
                        "was_delivered=$wasAlreadyDelivered"
                    )

                    // [VERBOSE] Step 4: Deduplication - mark receipt seen
                    val firstReceiptSeen = markDeliveredReceiptSeen(messageId)
                    Timber.d(
                        "[RECEIPT-RX] Dedup check: msg=$messageId " +
                        "first_receipt=$firstReceiptSeen was_already_delivered=$wasAlreadyDelivered"
                    )

                    if (!firstReceiptSeen && wasAlreadyDelivered) {
                        Timber.i(
                            "[RECEIPT-RX] DUPLICATE: Already delivered and already processed receipt: msg=$messageId"
                        )
                        removePendingOutbound(messageId)
                        logDeliveryState(
                            messageId = messageId,
                            state = "delivered",
                            detail = "delivery_receipt_duplicate_status=$normalized"
                        )
                        return
                    }

                    // [VERBOSE] Step 5: Update history if first delivery
                    if (!wasAlreadyDelivered) {
                        Timber.i(
                            "[RECEIPT-RX] Marking as delivered: msg=$messageId status=$normalized"
                        )
                        try {
                            historyManager?.markDelivered(messageId)
                            historyManager?.flush()
                            Timber.d("[RECEIPT-RX] History updated: msg=$messageId")
                        } catch (e: Exception) {
                            Timber.e(
                                e,
                                "[ERROR] History update FAILED: msg=$messageId error=${e.message}"
                            )
                        }
                    } else {
                        Timber.d(
                            "[RECEIPT-RX] Already delivered in history, skipping update: msg=$messageId"
                        )
                    }

                    // [VERBOSE] Step 6: Clean up pending outbox
                    removePendingOutbound(messageId)
                    Timber.d("[RECEIPT-RX] Removed from pending outbox: msg=$messageId")

                    // [VERBOSE] Step 7: Refresh and emit UI updates
                    val refreshedRecord = try {
                        val rec = historyManager?.get(messageId)
                        Timber.d(
                            "[RECEIPT-RX] Refreshed record: msg=$messageId " +
                            "delivered=${rec?.delivered} status=${rec?.status}"
                        )
                        rec
                    } catch (e: Exception) {
                        Timber.e(
                            e,
                            "[ERROR] Refresh FAILED: msg=$messageId error=${e.message}"
                        )
                        null
                    }

                    if (refreshedRecord != null) {
                        repoScope.launch {
                            _messageUpdates.emit(refreshedRecord)
                            Timber.d("[RECEIPT-RX] Emitted UI update: msg=$messageId")
                        }
                    }

                    // [VERBOSE] Step 8: Call core API if first delivery
                    if (wasAlreadyDelivered) {
                        Timber.d(
                            "[RECEIPT-RX] Skipping core.markMessageSent (already delivered): msg=$messageId"
                        )
                        return
                    }

                    try {
                        ironCore?.markMessageSent(messageId)
                        Timber.i("[RECEIPT-RX] Called core.markMessageSent: msg=$messageId")
                    } catch (e: Exception) {
                        Timber.e(
                            e,
                            "[ERROR] core.markMessageSent FAILED: msg=$messageId error=${e.message}"
                        )
                    }

                    // [VERBOSE] Step 9: Bridge to ChatViewModel
                    repoScope.launch {
                        try {
                            com.scmessenger.android.service.MeshEventBus.emitMessageEvent(
                                com.scmessenger.android.service.MessageEvent.Delivered(messageId)
                            )
                            Timber.d("[RECEIPT-RX] Emitted MessageEvent.Delivered: msg=$messageId")
                        } catch (e: Exception) {
                            Timber.e(
                                e,
                                "[ERROR] MeshEventBus.emit FAILED: msg=$messageId error=${e.message}"
                            )
                        }
                    }

                    // [VERBOSE] Step 10: Final logging
                    logDeliveryState(
                        messageId = messageId,
                        state = "delivered",
                        detail = "delivery_receipt_status=$normalized first_receipt=$firstReceiptSeen"
                    )
                    Timber.i(
                        "[OK] Receipt processing complete: msg=$messageId status=$normalized"
                    )
                }
            }
            ironCore?.setDelegate(coreDelegate)

            // 4. Start Android transports. Individual transport failures should
            // not abort the entire mesh core lifecycle.
            repoScope.launch {
                try {
                    initializeAndStartBle()
                } catch (e: Exception) {
                    Timber.w(e, "BLE transport failed to initialize; continuing with remaining transports")
                }
                try {
                    initializeAndStartWifi()
                } catch (e: Exception) {
                    Timber.w(e, "WiFi transport failed to initialize; continuing with remaining transports")
                }
                try {
                    initializeAndStartSwarm()
                } catch (e: Exception) {
                    Timber.w(e, "Swarm transport failed to initialize; core service remains active")
                }

                // Start all transports via TransportManager, gating mDNS by internetEnabled setting
                try {
                    ensureTransportManager()
                    val settings = loadSettings()
                    transportManager?.initialize(
                        bleEnabled = settings.bleEnabled,
                        wifiAwareEnabled = settings.wifiAwareEnabled,
                        wifiDirectEnabled = settings.wifiDirectEnabled
                    )
                    transportManager?.startAll(enableMdns = settings.internetEnabled)
                } catch (e: Exception) {
                    Timber.w(e, "TransportManager startAll failed; continuing with individual transports")
                }

                ensurePendingOutboxRetryLoop()
                ensureCoverTrafficLoop()
                flushPendingOutbox("service_started")
            }

            // 5. Update State
            _serviceState.value = meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
            if (_serviceState.value != uniffi.api.ServiceState.RUNNING) {
                Timber.e("MeshService did not reach RUNNING state")
                return
            }
            serviceStartedAtEpochSec = System.currentTimeMillis() / 1000
            updateStats()
            startPeriodicStatsUpdate()

            // On reinstall with missing data: broadcast identity beacon after swarm connects
            // so nearby and relay-connected peers can re-send identity info.
            if (isReinstallWithMissingData) {
                isReinstallWithMissingData = false
                Timber.i("Reinstall recovery: scheduling post-start identity beacon")
                repoScope.launch {
                    kotlinx.coroutines.delay(4_000L)
                    updateBleIdentityBeacon()
                    Timber.i("Reinstall recovery beacon sent")
                }
            }

            val info = ironCore?.getIdentityInfo()
            Timber.i("SC_IDENTITY_OWN p2p_id=${info?.libp2pPeerId ?: "unknown"} pk=${info?.publicKeyHex ?: "unknown"}")
            Timber.i("Mesh service started successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to start mesh service")
            stopMeshService()
            return
        }
    }

    private fun sendDeliveryReceiptAsync(
        senderPublicKeyHex: String,
        messageId: String,
        senderId: String,
        preferredRoutePeerId: String? = null,
        preferredWifiPeerId: String? = null,
        preferredBlePeerId: String? = null,
        preferredListenerHints: List<String> = emptyList()
    ) {
        // [BLOCKING]: Skip receipt if sender is blocked (relay unaffected)
        if (isBlocked(senderId)) {
            Timber.i("[BLOCKING] Skipping receipt for blocked peer $senderId (relay unaffected)")
            return
        }

        val normalizedMessageId = messageId.trim()
        if (normalizedMessageId.isEmpty()) return

        val candidateJob = repoScope.launch(
            kotlinx.coroutines.Dispatchers.IO,
            start = kotlinx.coroutines.CoroutineStart.LAZY
        ) {
            try {
                for (attempt in 1..receiptSendMaxAttempts) {
                    // [VERBOSE] Log: Starting receipt encode attempt
                    Timber.i(
                        "[RECEIPT-ENCODE] Attempt $attempt/$receiptSendMaxAttempts: " +
                        "msg=$normalizedMessageId sender=$senderId"
                    )

                    // [CRITICAL] Step 1: Create Receipt struct with core types
                    val receipt = uniffi.api.Receipt(
                        messageId = normalizedMessageId,
                        status = uniffi.api.DeliveryStatus.DELIVERED,
                        timestamp = (System.currentTimeMillis() / 1000).toULong()
                    )

                    // [CRITICAL] Step 2: Encode Receipt to JSON bytes using core bindings
                    val receiptBytes = try {
                        Timber.d(
                            "[RECEIPT-ENCODE] Encoding Receipt struct: " +
                            "msg=$normalizedMessageId status=${receipt.status} ts=${receipt.timestamp}"
                        )
                        uniffi.api.encodeReceipt(receipt)
                    } catch (e: Exception) {
                        Timber.e(
                            e,
                            "[ERROR] Receipt encode FAILED: msg=$normalizedMessageId " +
                            "error=${e.message} class=${e.javaClass.simpleName} attempt=$attempt"
                        )
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "encode",
                            outcome = "error",
                            detail = "encode_error msg=$normalizedMessageId error=${e.message} attempt=$attempt"
                        )
                        if (attempt < receiptSendMaxAttempts) {
                            kotlinx.coroutines.delay(getRetryDelay(attempt - 1))
                            continue
                        } else {
                            Timber.e(
                                "[ERROR] Receipt encode exhausted after $receiptSendMaxAttempts attempts: " +
                                "msg=$normalizedMessageId sender=$senderId"
                            )
                            logDeliveryAttempt(
                                messageId = normalizedMessageId,
                                medium = "receipt",
                                phase = "encode",
                                outcome = "exhausted",
                                detail = "encode_error_final msg=$normalizedMessageId attempts=$receiptSendMaxAttempts error=${e.message}"
                            )
                            return@launch
                        }
                    }

                    Timber.i(
                        "[RECEIPT-ENCODE] SUCCESS: msg=$normalizedMessageId " +
                        "bytes=${receiptBytes.size} status=${receipt.status}"
                    )
                    logDeliveryAttempt(
                        messageId = normalizedMessageId,
                        medium = "receipt",
                        phase = "encode",
                        outcome = "success",
                        detail = "encoded_bytes=${receiptBytes.size} attempt=$attempt"
                    )
                    val contact = try { contactManager?.get(senderId) } catch (_: Exception) { null }
                    val hints = parseRoutingHints(contact?.notes)
                    val routeCandidates = buildRoutePeerCandidates(
                        peerId = senderId,
                        cachedRoutePeerId = preferredRoutePeerId ?: hints.libp2pPeerId,
                        notes = contact?.notes,
                        recipientPublicKey = senderPublicKeyHex
                    )
                    val delivery = attemptDirectSwarmDelivery(
                        routePeerCandidates = routeCandidates,
                        listeners = (preferredListenerHints + hints.listeners).distinct(),
                        encryptedData = receiptBytes,
                        wifiPeerId = preferredWifiPeerId ?: hints.wifiPeerId,
                        blePeerId = preferredBlePeerId ?: hints.blePeerId,
                        traceMessageId = normalizedMessageId,
                        attemptContext = "receipt_send",
                    )
                    if (delivery.acked) {
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "aggregate",
                            outcome = "acked",
                            detail = "ctx=receipt_send sender=$senderId attempt=$attempt"
                        )
                        Timber.d("Targeted delivery receipt sent for $normalizedMessageId to $senderId")
                        return@launch
                    }

                    if (attempt < receiptSendMaxAttempts) {
                        val delayMs = getRetryDelay(attempt - 1)
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "retry",
                            outcome = "scheduled",
                            detail = "ctx=receipt_send sender=$senderId attempt=$attempt delay_sec=${delayMs / 1000}"
                        )
                        kotlinx.coroutines.delay(delayMs)
                    } else {
                        logDeliveryAttempt(
                            messageId = normalizedMessageId,
                            medium = "receipt",
                            phase = "aggregate",
                            outcome = "exhausted",
                            detail = "ctx=receipt_send sender=$senderId attempts=$receiptSendMaxAttempts"
                        )
                    }
                }
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                Timber.d("Failed to send delivery receipt for $normalizedMessageId: ${e.message}")
            } finally {
                pendingReceiptSendJobs.remove(normalizedMessageId)
            }
        }

        val existingJob = pendingReceiptSendJobs.putIfAbsent(normalizedMessageId, candidateJob)
        if (existingJob != null && existingJob.isActive) {
            candidateJob.cancel()
            logDeliveryAttempt(
                messageId = normalizedMessageId,
                medium = "receipt",
                phase = "dedupe",
                outcome = "skipped_active",
                detail = "ctx=receipt_send sender=$senderId"
            )
            return
        }
        if (existingJob != null) {
            pendingReceiptSendJobs[normalizedMessageId] = candidateJob
        }
        candidateJob.invokeOnCompletion {
            pendingReceiptSendJobs.remove(normalizedMessageId, candidateJob)
        }
        candidateJob.start()
    }

    private fun sendIdentitySyncIfNeeded(routePeerId: String, knownPublicKey: String? = null) {
        val normalizedRoute = routePeerId.trim()
        if (normalizedRoute.isEmpty() || isBootstrapRelayPeer(normalizedRoute)) return

        // Check if core is initialized before attempting identity sync
        if (ironCore == null) {
            Timber.d("sendIdentitySyncIfNeeded: IronCore not initialized, skipping for $normalizedRoute")
            return
        }

        val shouldSend = identitySyncSentPeers.add(normalizedRoute)
        if (!shouldSend) return

        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val recipientPublicKey = normalizePublicKey(knownPublicKey)
                    ?: normalizePublicKey(
                        try {
                            ironCore?.extractPublicKeyFromPeerId(normalizedRoute)
                        } catch (_: Exception) {
                            null
                        }
                    )
                if (recipientPublicKey == null) {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val payload = encodeIdentitySyncPayload()
                val prepared = ironCore?.prepareMessageWithId(recipientPublicKey, payload, uniffi.api.MessageType.TEXT, null)
                if (prepared == null) {
                    identitySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                // Use targeted delivery (swarm + BLE fallback) for identity sync
                val contact = contactManager?.list()?.firstOrNull {
                    it.peerId == normalizedRoute || parseRoutingHints(it.notes).libp2pPeerId == normalizedRoute
                }
                val hints = parseRoutingHints(contact?.notes)
                val routeCandidates = buildRoutePeerCandidates(
                    peerId = contact?.peerId ?: normalizedRoute,
                    cachedRoutePeerId = normalizedRoute,
                    notes = contact?.notes,
                    recipientPublicKey = recipientPublicKey
                )

                attemptDirectSwarmDelivery(
                    routePeerCandidates = routeCandidates,
                    listeners = hints.listeners,
                    encryptedData = prepared.envelopeData,
                    blePeerId = hints.blePeerId
                )
                Timber.d("Identity sync sent to $normalizedRoute")
            } catch (e: Exception) {
                identitySyncSentPeers.remove(normalizedRoute)
                Timber.d("Failed to send identity sync to $normalizedRoute: ${e.message}")
            }
        }
    }


    private fun sendHistorySyncIfNeeded(routePeerId: String, knownPublicKey: String? = null) {
        val normalizedRoute = routePeerId.trim()
        Timber.w("sendHistorySyncIfNeeded called for $normalizedRoute")
        if (normalizedRoute.isEmpty() || isBootstrapRelayPeer(normalizedRoute)) return

        // Gate on both IronCore instance availability AND identity readiness.
        // A non-null IronCore with an un-initialized identity will throw
        // IronCoreException.NotInitialized from prepareMessageWithId, producing
        // noisy false-positive error logs on fresh-install startup.
        val core = ironCore ?: run {
            Timber.w("sendHistorySyncIfNeeded: IronCore not initialized, skipping for $normalizedRoute")
            return
        }
        val identityReady = try {
            core.getIdentityInfo().identityId != null
        } catch (_: uniffi.api.IronCoreException.NotInitialized) {
            false
        } catch (e: Exception) {
            Timber.w(e, "sendHistorySyncIfNeeded: unexpected error checking identity readiness for $normalizedRoute")
            false
        }
        if (!identityReady) {
            Timber.w("sendHistorySyncIfNeeded: identity not ready yet, skipping for $normalizedRoute")
            return
        }

        val now = System.currentTimeMillis()
        val lastSent = historySyncSentPeers[normalizedRoute] ?: 0L
        val shouldSend = (now - lastSent) > HISTORY_SYNC_COOLDOWN_MS
        Timber.w("sendHistorySyncIfNeeded shouldSend=$shouldSend for $normalizedRoute (age=${now - lastSent}ms)")
        if (!shouldSend) return
        historySyncSentPeers[normalizedRoute] = now

        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val extractedPublicKey = try { ironCore?.extractPublicKeyFromPeerId(normalizedRoute) } catch (_: Exception) { null }
                val recipientPublicKey = normalizePublicKey(knownPublicKey) ?: normalizePublicKey(extractedPublicKey)
                if (recipientPublicKey == null) {
                    Timber.e("sendHistorySyncIfNeeded: missing recipientPublicKey for $normalizedRoute (known=$knownPublicKey, extracted=$extractedPublicKey)")
                    historySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val payload = encodeMeshMessagePayload(content = "", kind = "history_sync")
                val prepared = try { ironCore?.prepareMessageWithId(recipientPublicKey, payload, uniffi.api.MessageType.TEXT, null) } catch (e: Exception) { Timber.e(e, "prepareMessageWithId failed in history_sync"); null }
                if (prepared == null) {
                    Timber.e("sendHistorySyncIfNeeded: prepared is null for $normalizedRoute")
                    historySyncSentPeers.remove(normalizedRoute)
                    return@launch
                }

                val contact = contactManager?.list()?.firstOrNull { it.peerId == normalizedRoute || parseRoutingHints(it.notes).libp2pPeerId == normalizedRoute }
                val hints = parseRoutingHints(contact?.notes)
                val routeCandidates = buildRoutePeerCandidates(contact?.peerId ?: normalizedRoute, normalizedRoute, contact?.notes, recipientPublicKey)

                attemptDirectSwarmDelivery(routeCandidates, hints.listeners, prepared.envelopeData, hints.wifiPeerId, hints.blePeerId)
                Timber.w("History sync request sent to $normalizedRoute")
            } catch (e: Exception) {
                Timber.e(e, "History sync error for $normalizedRoute")
                historySyncSentPeers.remove(normalizedRoute)
            }
        }
    }

    private val historySyncDataInProgress = java.util.concurrent.ConcurrentHashMap<String, Boolean>()

    private fun sendHistorySyncDataIfNeeded(canonicalPeerId: String, routePeerId: String?, recipientPublicKey: String, listeners: List<String>, wifiPeerId: String?) {
        if (historySyncDataInProgress.putIfAbsent(canonicalPeerId, true) != null) {
            Timber.d("sendHistorySyncDataIfNeeded: already in progress for $canonicalPeerId")
            return
        }
        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val recentMsgs = historyManager?.conversation(canonicalPeerId, 400u)?.sortedBy { it.timestamp } ?: emptyList()
                if (recentMsgs.isEmpty()) {
                    Timber.w("sendHistorySyncDataIfNeeded: no recent msgs for $canonicalPeerId")
                    return@launch
                }
                Timber.w("sendHistorySyncDataIfNeeded: compiling ${recentMsgs.size} msgs for $canonicalPeerId")

                val hints = parseRoutingHints(contactManager?.get(canonicalPeerId)?.notes)
                val routeCandidates = buildRoutePeerCandidates(canonicalPeerId, routePeerId ?: hints.libp2pPeerId, contactManager?.get(canonicalPeerId)?.notes, recipientPublicKey)
                val allListeners = (listeners + hints.listeners).distinct()

                // Chunk into batches of 20 to stay within encryption payload limits
                val batchSize = 20
                val batches = recentMsgs.chunked(batchSize)
                var sentBatches = 0

                for ((batchIndex, batch) in batches.withIndex()) {
                    val arr = org.json.JSONArray()
                    batch.forEach { msg ->
                        val obj = org.json.JSONObject()
                        obj.put("id", msg.id)
                        obj.put("dir", if (msg.direction == uniffi.api.MessageDirection.SENT) "sent" else "recv")
                        obj.put("pid", msg.peerId)
                        obj.put("txt", msg.content)
                        obj.put("ts", msg.timestamp.toLong())
                        obj.put("sts", msg.senderTimestamp.toLong())
                        obj.put("del", msg.delivered)
                        arr.put(obj)
                    }
                    val payload = encodeMeshMessagePayload(content = arr.toString(), kind = "history_sync_data")
                    val prepared = try { ironCore?.prepareMessageWithId(recipientPublicKey, payload, uniffi.api.MessageType.TEXT, null) } catch (e: Exception) {
                        Timber.e(e, "prepareMessage failed in sync_data batch $batchIndex (${batch.size} msgs)")
                        null
                    }
                    if (prepared == null) {
                        Timber.e("sendHistorySyncDataIfNeeded: prepared is null for batch $batchIndex of $canonicalPeerId")
                        continue
                    }

                    attemptDirectSwarmDelivery(routeCandidates, allListeners, prepared.envelopeData, wifiPeerId ?: hints.wifiPeerId, hints.blePeerId)
                    sentBatches++
                    // Small delay between batches to avoid overwhelming BLE
                    if (batchIndex < batches.size - 1) {
                        kotlinx.coroutines.delay(200)
                    }
                }
                Timber.w("History sync data sent to $canonicalPeerId ($sentBatches/${batches.size} batches, ${recentMsgs.size} items total)")
            } catch (e: Exception) {
                Timber.e(e, "sendHistorySyncDataIfNeeded error for $canonicalPeerId")
            } finally {
                historySyncDataInProgress.remove(canonicalPeerId)
            }
        }
    }
    private suspend fun initializeAndStartBle() {
        val settings = loadSettings()
        if (!settings.bleEnabled) {
            Timber.d("BLE disabled in settings")
            return
        }
        if (!hasAllPermissions(Permissions.bluetooth)) {
            Timber.w("Skipping BLE transport start: missing Bluetooth permissions")
            return
        }

        // BLE GATT Client: must exist before scanner callbacks to avoid missing first identity reads.
        if (bleGattClient == null) {
            bleGattClient = com.scmessenger.android.transport.ble.BleGattClient(
                context,
                onIdentityReceived = { peerId, data ->
                    onPeerIdentityRead(peerId, data)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }

        // BLE Scanner: Feeds discovered peers to MeshService and handles GATT connections
        if (bleScanner == null) {
            bleScanner = com.scmessenger.android.transport.ble.BleScanner(
                context,
                onPeerDiscovered = { peerId ->
                    noteBleRouteObservation(peerId = peerId, bleAddress = peerId, source = "scan")
                    meshService?.onPeerDiscovered(peerId)
                    // Connect via GATT client to read identity if needed.
                    kotlin.runCatching { bleGattClient?.connect(peerId) }
                        .onFailure { Timber.w(it, "BLE GATT connect failed for $peerId") }
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                },
                onScanFailure = {
                    // Wire BLE failure to TransportManager for graceful degradation
                    transportManager?.handleBleFailure()
                }
            )
        }
        repoScope.launch {
            bleScanner?.startScanning()
        }

        // BLE Advertiser: Broadcasts our presence
        if (bleAdvertiser == null) {
            bleAdvertiser = com.scmessenger.android.transport.ble.BleAdvertiser(context)
        }
        bleAdvertiser?.setRotationInterval(900_000L) // 15-minute default identity beacon rotation
        bleAdvertiser?.startAdvertising()

        // BLE GATT Server: Serves our identity and accepts writes from nearby peers
        if (bleGattServer == null) {
            bleGattServer = com.scmessenger.android.transport.ble.BleGattServer(
                context,
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        bleGattServer?.start()

        // Identity beacon data served to BLE scanners via IDENTITY_CHAR_UUID reads.
        // Initialize with a valid empty JSON to avoid platform parsing errors during startup.
        bleGattServer?.setIdentityData(identityData)

        // Set identity beacon on BLE GATT server so nearby peers can read our Ed25519 public key
        updateBleIdentityBeacon()

        // Wire BLE components into TransportManager for recovery coordination
        transportManager?.setBleComponents(bleScanner, bleAdvertiser, bleGattClient, bleGattServer)
    }

    // Identity beacon data served to BLE scanners via IDENTITY_CHAR_UUID reads.
    // Initialize with a valid empty JSON to avoid platform parsing errors during startup.
    private var identityData: ByteArray = "{}".toByteArray()

    private fun updateBleIdentityBeacon() {
        val identity = ironCore?.getIdentityInfo()
        val publicKeyHex = identity?.publicKeyHex
        if (!publicKeyHex.isNullOrEmpty()) {
            val now = System.currentTimeMillis()
            if (now - lastBleBeaconUpdateMillis < 5000) {
                // Throttled: wait for the next call or periodic update
                return
            }
            lastBleBeaconUpdateMillis = now

            // 1. Immediate update with just identity (no listeners yet)
            // This allows peers to "see" us in the UI immediately while we wait for swarm listeners to bind.
            setIdentityBeaconInternal(identity, emptyList())

            // 2. Delayed update with full connection hints
            repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
                var listeners = getListeningAddresses()
                var attempts = 0
                while (listeners.isEmpty() && attempts < 10 && isActive) {
                    kotlinx.coroutines.delay(500)
                    listeners = getListeningAddresses()
                    attempts++
                }
                if (isActive) {
                    setIdentityBeaconInternal(identity, listeners)
                }
            }
        }
    }

    private fun setIdentityBeaconInternal(identity: uniffi.api.IdentityInfo, listeners: List<String>) {
        val publicKeyHex = identity.publicKeyHex ?: return
        try {
            // Keep BLE identity beacons compact
            var resolvedListeners = normalizeOutboundListenerHints(listeners).take(2)
            var resolvedExternal = normalizeExternalAddressHints(getExternalAddresses()).take(2)
            val nickname = (identity.nickname ?: "").take(32)

            fun buildBeacon(): org.json.JSONObject {
                val connectionHints = (resolvedListeners + resolvedExternal).distinct()
                // UNIFIED ID FIX: BLE beacon primary ID is libp2p_peer_id (network routable)
                return org.json.JSONObject()
                    .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY: libp2p Peer ID
                    .put("public_key", publicKeyHex)                         // Canonical identity key
                    .put("device_id", identity.deviceId ?: "")             // Multi-device routing
                    .put("identity_id", identity.identityId ?: "")         // SECONDARY: Blake3 hash
                    .put("nickname", nickname)
                    .put("libp2p_peer_id", identity.libp2pPeerId ?: "")    // Backward compatibility
                    .put("listeners", org.json.JSONArray(resolvedListeners))
                    .put("external_addresses", org.json.JSONArray(resolvedExternal))
                    .put("connection_hints", org.json.JSONArray(connectionHints))
            }

            var beaconJsonObject = buildBeacon()
            var beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)

            if (beaconJson.size > 480) {
                resolvedListeners = resolvedListeners.take(1)
                resolvedExternal = resolvedExternal.take(1)
                beaconJsonObject = buildBeacon()
                beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
            }
            if (beaconJson.size > 480) {
                resolvedListeners = emptyList()
                resolvedExternal = emptyList()
                beaconJsonObject = buildBeacon()
                beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
            }
            if (beaconJson.size > 480) {
                beaconJsonObject = org.json.JSONObject()
                    .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY
                    .put("public_key", publicKeyHex)
                    .put("device_id", identity.deviceId ?: "")
                    .put("identity_id", identity.identityId ?: "")
                    .put("nickname", nickname)
                    .put("libp2p_peer_id", identity.libp2pPeerId ?: "")
                    .put("listeners", org.json.JSONArray())
                    .put("external_addresses", org.json.JSONArray())
                    .put("connection_hints", org.json.JSONArray())
                beaconJson = beaconJsonObject.toString().toByteArray(Charsets.UTF_8)
            }
            val publishedAt = System.currentTimeMillis()
            if (
                beaconJson.contentEquals(lastBleBeaconPayload) &&
                (publishedAt - lastBleBeaconPayloadPublishedAtMillis) < 5000
            ) {
                return
            }
            lastBleBeaconPayload = beaconJson.copyOf()
            lastBleBeaconPayloadPublishedAtMillis = publishedAt

            // Create compact BLE beacon (≤24 bytes) for advertising
            // Full identity is served via GATT for detailed discovery
            val compactBeacon = createCompactBleBeacon(identity, publicKeyHex)
            bleAdvertiser?.updateIdentityBeacon(compactBeacon)
            bleGattServer?.setIdentityData(beaconJson)
            identityData = beaconJson // Store for immediate use by GATT server
            Timber.i(
                "BLE GATT identity beacon updated: ${publicKeyHex.take(8)}... " +
                    "(${beaconJson.size} bytes, listeners=${resolvedListeners.size}, external=${resolvedExternal.size}) " +
                    "p2p_id=${identity.libp2pPeerId ?: "unknown"}"
            )
        } catch (e: Exception) {
            Timber.w("Failed to set BLE GATT identity beacon: ${e.message}")
        }
    }

    private fun createCompactBleBeacon(identity: uniffi.api.IdentityInfo, publicKeyHex: String): ByteArray {
        try {
            val beacon = org.json.JSONObject()
            // Compact beacon: only essential discovery data, <=24 bytes total
            // Full identity is served via GATT for devices that connect
            beacon.put("p", identity.libp2pPeerId?.take(8) ?: "")
            beacon.put("k", publicKeyHex.take(16))
            beacon.put("n", (identity.nickname ?: "").take(12))
            val beaconBytes = beacon.toString().toByteArray()

            return if (beaconBytes.size <= 24) {
                beaconBytes
            } else {
                // If still too large, use ultra-compact format: peer_id hash only
                val hash = publicKeyHex.take(8).toByteArray()
                hash.take(16).toByteArray().plus(byteArrayOf(0, 0, 0, 0, 0, 0, 0, 0))
            }
        } catch (e: Exception) {
            Timber.w("Failed to create compact BLE beacon: ${e.message}")
            // Fallback: just the peer ID
            return (identity.libp2pPeerId ?: "").take(16).toByteArray()
        }
    }

    /**
     * Called when BLE identity beacon is read from a peer.
     * Extracts identity info and attempts to dial the peer via libp2p if possible.
     */
    private fun onPeerIdentityRead(blePeerId: String, data: ByteArray) {
        try {
            val json = org.json.JSONObject(data.toString(Charsets.UTF_8))
            val publicKeyHexRaw = json.getString("public_key")
            val publicKeyHex = normalizePublicKey(publicKeyHexRaw)
                ?: run {
                    Timber.w("Ignoring BLE identity from $blePeerId: invalid public key")
                    return
                }
            // UNIFIED ID FIX: BLE beacon primary ID is "peer_id" (libp2p Peer ID)
            val identityId = json.optString("identity_id", "")
                .trim()
                .takeIf { it.isNotBlank() }
                ?: blePeerId
            val rawNickname = (
                json.optString("nickname", "")
                    .ifBlank { json.optString("name", "") }
                ).trim()
            val libp2pPeerId = json.optString("peer_id", "")
                .trim()
                .takeIf { it.isNotBlank() }
                ?: json.optString("libp2p_peer_id", "")
                    .trim()
                    .takeIf { it.isNotBlank() }
                    ?: ""
            val listeners = json.optJSONArray("listeners")
            val externalAddresses = json.optJSONArray("external_addresses")
            val connectionHints = json.optJSONArray("connection_hints")
            val normalizedLibp2p = libp2pPeerId.takeIf { !it.isNullOrBlank() }?.trim()
            noteBleRouteObservation(peerId = blePeerId, bleAddress = blePeerId, source = "identity_self")
            noteBleRouteObservation(peerId = identityId, bleAddress = blePeerId, source = "identity_canonical")
            noteBleRouteObservation(peerId = normalizedLibp2p, bleAddress = blePeerId, source = "identity_route")

            val discoveredNickname = prepopulateDiscoveryNickname(
                nickname = rawNickname,
                peerId = identityId,
                publicKey = publicKeyHex
            )

            Timber.i(
                "Peer identity read from $blePeerId: ${publicKeyHex.take(8)}... " +
                    "identity=$identityId nickname='${discoveredNickname?.take(24) ?: ""}'"
            )

            // Persist BLE -> Identity mapping in contact notes so it survives restarts
            // and helps routing even when BLE is the only transport initially.
            repoScope.launch {
                try {
                    val contact = contactManager?.get(identityId)
                    if (contact != null) {
                        val updatedNotes = appendRoutingHint(contact.notes, "ble_peer_id", blePeerId)
                        if (updatedNotes != contact.notes) {
                            contactManager?.add(contact.copy(notes = updatedNotes))
                            Timber.d("Updated persistent BLE routing for $identityId: $blePeerId")
                        }
                    }
                } catch (e: Exception) {
                    Timber.w(e, "Failed to persist BLE routing for $identityId")
                }
            }

            val selfIdentity = ironCore?.getIdentityInfo()
            val selfKey = normalizePublicKey(selfIdentity?.publicKeyHex)
            val selfIdentityId = selfIdentity?.identityId?.trim().orEmpty()
            val selfLibp2pPeerId = selfIdentity?.libp2pPeerId?.trim().orEmpty()
            if ((selfKey != null && selfKey == publicKeyHex) ||
                (selfIdentityId.isNotEmpty() && selfIdentityId == identityId) ||
                (selfLibp2pPeerId.isNotEmpty() && selfLibp2pPeerId == normalizedLibp2p)
            ) {
                Timber.d("Ignoring self BLE identity beacon from $blePeerId")
                return
            }

            repoScope.launch {
                listOfNotNull(identityId, blePeerId, normalizedLibp2p)
                    .map { it.trim() }
                    .filter { it.isNotEmpty() }
                    .forEach { promotePendingOutboundForPeer(peerId = it) }
                flushPendingOutbox("peer_discovered")
            }

            // Update discovery map
            val discoveryInfo = PeerDiscoveryInfo(
                peerId = identityId,
                publicKey = publicKeyHex,
                nickname = discoveredNickname,
                localNickname = try { contactManager?.get(identityId)?.localNickname } catch (_: Exception) { null },
                libp2pPeerId = normalizedLibp2p,
                transport = com.scmessenger.android.service.TransportType.BLE,
                isFull = true,
                lastSeen = System.currentTimeMillis().toULong() / 1000u
            )
            updateDiscoveredPeer(identityId, discoveryInfo)
            if (!normalizedLibp2p.isNullOrBlank()) {
                updateDiscoveredPeer(normalizedLibp2p, discoveryInfo)
            }
            // Remove the preliminary BLE-UUID entry (isFull=false) that was created when
            // the connection was first established before identity was read. Now that we
            // have the real identityId, the BLE UUID key is an duplicate we no longer need.
            if (blePeerId != identityId && blePeerId != normalizedLibp2p) {
                _discoveredPeers.update { current ->
                    current.filterKeys { key ->
                        key != blePeerId &&
                            // Also remove any entry whose peerId field matches the BLE UUID
                            current[key]?.peerId != blePeerId
                    }
                }
                Timber.d("Removed preliminary BLE entry $blePeerId → promoted to identity $identityId")
            }

            // Emit identity to nearby peers bus — UI will show peer in Nearby section for user to add
            val rawHints = mutableListOf<String>()
            listeners?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
            }
            externalAddresses?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
            }
            connectionHints?.let { arr ->
                for (i in 0 until arr.length()) {
                    rawHints.add(arr.getString(i))
                }
            }

            val routePeerId = normalizedLibp2p
            val listenersStrings = buildDialCandidatesForPeer(
                routePeerId = routePeerId,
                rawAddresses = rawHints,
                includeRelayCircuits = true
            )
            repoScope.launch {
                emitIdentityDiscoveredIfChanged(
                    peerId = identityId,
                    publicKey = publicKeyHex,
                    nickname = discoveredNickname,
                    libp2pPeerId = routePeerId,
                    listeners = listenersStrings,
                    blePeerId = blePeerId
                )
            }
            annotateIdentityInLedger(
                routePeerId = routePeerId,
                listeners = listenersStrings,
                publicKey = publicKeyHex,
                nickname = discoveredNickname
            )
            Timber.i("Emitted IdentityDiscovered for $blePeerId: ${publicKeyHex.take(8)}...")
            // Trigger history sync over BLE when we discover a peer's identity
            if (!routePeerId.isNullOrEmpty()) {
                sendHistorySyncIfNeeded(routePeerId, publicKeyHex)
            } else {
                sendHistorySyncIfNeeded(identityId, publicKeyHex)
            }
            // Update lastSeen if already a saved contact
            try { contactManager?.updateLastSeen(blePeerId) } catch (_: Exception) { }
            try { contactManager?.updateLastSeen(identityId) } catch (_: Exception) { }
            routePeerId?.let {
                try { contactManager?.updateLastSeen(it) } catch (_: Exception) { }
            }
            repoScope.launch {
                upsertFederatedContact(
                    canonicalPeerId = publicKeyHex,       // UNIFIED ID FIX: canonical = public_key_hex
                    publicKey = publicKeyHex,
                    nickname = rawNickname.takeIf { it.isNotBlank() },
                    libp2pPeerId = routePeerId,
                    listeners = listenersStrings,
                    blePeerId = blePeerId,
                    createIfMissing = false
                )
            }

            if (!routePeerId.isNullOrEmpty() && listenersStrings.isNotEmpty()) {
                connectToPeer(routePeerId, listenersStrings)
            }
        } catch (e: Exception) {
            Timber.w("Failed to parse peer identity read: ${e.message}")
        }
    }

    private fun updateDiscoveredPeer(key: String, info: PeerDiscoveryInfo) {
        val normalizedKey = PeerIdValidator.normalize(key)
        _discoveredPeers.update { current ->
            val existing = current[normalizedKey]
            if (existing != null && existing.isFull && !info.isFull && (info.lastSeen - existing.lastSeen < 300u)) {
                // Don't downgrade a full identity to headless if we've seen it recently.
                current
            } else {
                val merged = if (existing == null) {
                    info
                } else {
                    info.copy(
                        peerId = selectCanonicalPeerId(info.peerId, existing.peerId),
                        publicKey = info.publicKey ?: existing.publicKey,
                        nickname = selectAuthoritativeNickname(info.nickname, existing.nickname),
                        localNickname = normalizeNickname(info.localNickname) ?: normalizeNickname(existing.localNickname),
                        libp2pPeerId = info.libp2pPeerId ?: existing.libp2pPeerId,
                        transport = if (
                            info.transport == com.scmessenger.android.service.TransportType.INTERNET ||
                                existing.transport == com.scmessenger.android.service.TransportType.INTERNET
                        ) {
                            com.scmessenger.android.service.TransportType.INTERNET
                        } else {
                            info.transport
                        },
                        isFull = info.isFull || existing.isFull,
                        lastSeen = maxOf(info.lastSeen, existing.lastSeen)
                    )
                }
                val canonicalPeerId = PeerIdValidator.normalize(merged.peerId.ifEmpty { normalizedKey })

                val withCanonical = current + (canonicalPeerId to merged)
                withCanonical.filterNot { (mapKey, _) ->
                    mapKey != canonicalPeerId && PeerIdValidator.isSame(mapKey, canonicalPeerId)
                }
            }
        }
    }

    private fun noteBleRouteObservation(peerId: String?, bleAddress: String?, source: String) {
        val normalizedPeerId = peerId?.trim().orEmpty()
        val normalizedBleAddress = bleAddress?.trim().orEmpty()
        if (normalizedPeerId.isEmpty() || normalizedBleAddress.isEmpty()) return

        val observation = BleRouteObservation(
            address = normalizedBleAddress,
            lastSeenMs = System.currentTimeMillis(),
            source = source
        )
        bleRouteObservations[normalizedPeerId] = observation
        bleRouteObservations[normalizedBleAddress] = observation
    }

    private fun resolveFreshBlePeerId(candidates: List<String>): BleRouteObservation? {
        if (candidates.isEmpty()) return null

        val now = System.currentTimeMillis()
        return candidates
            .asSequence()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .mapNotNull { candidate ->
                val observation = bleRouteObservations[candidate] ?: return@mapNotNull null
                val ageMs = now - observation.lastSeenMs
                
                // Extended freshness check for TRANSPORT-001: BLE hint staleness fix
                // 1. Within fresh TTL: use normally
                // 2. Within stale grace period: use for fallback (slightly stale but better than nothing)
                // 3. Beyond grace period: remove and skip
                when {
                    ageMs <= bleRouteFreshnessTtlMs -> observation  // Fresh
                    ageMs <= bleRouteStaleGraceMs -> {
                        // Stale but within grace period - use for fallback
                        Timber.d("Using stale BLE hint for $candidate (age=${ageMs}ms, grace=${bleRouteStaleGraceMs}ms)")
                        observation
                    }
                    else -> {
                        // Too stale, remove
                        bleRouteObservations.remove(candidate, observation)
                        null
                    }
                }
            }
            .maxByOrNull { it.lastSeenMs }
    }

    private fun pruneDisconnectedPeer(peerId: String) {
        val normalizedPeerId = peerId.trim()
        if (normalizedPeerId.isEmpty()) return

        _discoveredPeers.update { current ->
            if (current.isEmpty()) return@update current

            val disconnectedPublicKey = normalizePublicKey(current[normalizedPeerId]?.publicKey)
            val keysToRemove = current
                .filter { (key, info) ->
                    key == normalizedPeerId ||
                        info.peerId == normalizedPeerId ||
                        (
                            disconnectedPublicKey != null &&
                                normalizePublicKey(info.publicKey) == disconnectedPublicKey
                            )
                }
                .keys

            if (keysToRemove.isEmpty()) current else current - keysToRemove
        }
    }

    private fun initializeAndStartWifi() {
        val settings = loadSettings()
        if (!settings.wifiAwareEnabled && !settings.wifiDirectEnabled) {
            Timber.d("WiFi Transports disabled in settings")
            // Note: WifiTransportManager manages both. Need granular control?
            // For now, if either is enabled, we start it, let it handle internals?
            // But WifiTransportManager likely starts both.
            // Assuming strict check:
             return
        }
        val wifiPermissions = Permissions.location + Permissions.nearbyWifi
        if (!hasAllPermissions(wifiPermissions)) {
            Timber.w("Skipping WiFi transport start: missing Location/Nearby WiFi permissions")
            return
        }

        if (wifiTransportManager == null) {
            wifiTransportManager = com.scmessenger.android.transport.WifiTransportManager(
                context,
                onPeerDiscovered = { peerId ->
                    meshService?.onPeerDiscovered(peerId)
                },
                onDataReceived = { peerId, data ->
                    meshService?.onDataReceived(peerId, data)
                }
            )
        }
        wifiTransportManager?.initialize()
        wifiTransportManager?.startDiscovery()
    }

    private suspend fun initializeAndStartSwarm() = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
        val settings = loadSettings()
        if (!settings.internetEnabled) {
            Timber.d("Swarm/Internet disabled in settings")
            return@withContext
        }

        try {
            ensureLocalIdentityFederation()
            // Initiate swarm in Rust core.
            // Core auto-selects headless mode when identity is absent and upgrades when identity appears.
            // P0_TRANSPORT_001: Prefer port 9001 for LAN connectivity with the CLI daemon.
            // The Rust multi-port listener may fall back when that port is unavailable;
            // all exported identity hints must therefore come from the live listener list.
            // Bootstrap addrs: empty for now — mobile will discover LAN peers via mDNS
            // and relay peers via the ledger exchange protocol. Can be populated from
            // config or QR-scanned contact addrs in future.
            //
            // TCP-listener-zombie fix: startSwarm now blocks (up to 15s) until the
            // first listener is actually bound, and throws NetworkException if the
            // bind fails or times out. That is why this function is a suspend fun
            // pinned to Dispatchers.IO — it must never run on the main thread.
            meshService?.startSwarm("/ip4/0.0.0.0/tcp/9001", listOf())

            // Obtain the SwarmBridge managed by Rust MeshService — only after the
            // listener is confirmed bound, so a wired bridge implies real
            // inbound connectivity rather than a zombie listener.
            swarmBridge = meshService?.getSwarmBridge()
            refreshAddressesSnapshots()
            updateBleIdentityBeacon()
            ensureBootstrapRelayConnected()

            // v0.4.0 item 4: bounded startup dial sweep over the rest of the
            // ledger's proven-good addresses, fire-and-forget so it never
            // blocks swarm startup (mirrors the CLI's tokio::spawn sweep).
            repoScope.launch {
                performStartupDialSweep()
            }

            Timber.i("[OK] Internet transport (Swarm) started and bridge wired; listeners=${getListeningAddresses()}")
        } catch (e: Exception) {
            swarmBridge = null
            Timber.e(e, "Swarm failed to start listening — inbound internet/LAN transport unavailable")
        }
    }

    private fun ensureLocalIdentityFederation() {
        val core = ironCore ?: return
        try {
            var info = core.getIdentityInfo()
            if (!info.initialized) {
                val restored = restoreIdentityFromBackup(core)
                if (restored) {
                    // P0_ANDROID_010: Grant consent after successful backup restore.
                    // The Rust core starts with consent_granted=false, but if we have a
                    // persisted identity backup, the user already consented in a prior session.
                    core.grantConsent()
                    info = core.getIdentityInfo()
                }
            }
            if (!info.initialized) {
                Timber.i("Identity not initialized; onboarding required")
                return
            }

            // P0_ANDROID_010: Ensure consent is granted whenever identity is initialized.
            // This handles the case where identity was loaded from sled but consent wasn't set
            // (e.g., after a process restart where consent_granted resets to false).
            // grantConsent is idempotent — calling it when already granted is a no-op.
            core.grantConsent()

            val nickname = info.nickname?.trim().orEmpty()
            // P0: Cache identity fields for instant UI load on next startup
            cacheIdentityFields(info)
            
            if (nickname.isNotEmpty()) {
                // P2 (Bug 4): Latch the lazy-path persistIdentityBackup so a flurry of
                // post-startup ensureLocalIdentityFederation() calls (each touching
                // SharedPreferences and writing the sentinel file) collapses to one.
                // The explicit createIdentity() path is unchanged and still writes
                // immediately. compareAndSet returns true exactly once per process.
                if (identityBackupLatch.compareAndSet(false, true)) {
                    persistIdentityBackup(core)
                } else {
                    Timber.d("ensureLocalIdentityFederation: backup already persisted this process; skipping")
                }
            }
        } catch (e: Exception) {
            Timber.w("Failed to ensure local identity federation: ${e.message}")
        }
    }

    private fun getPlatformSecuredPassphrase(): String {
        val prefs = context.getSharedPreferences("platform_secure_keys", Context.MODE_PRIVATE)
        var key = prefs.getString("backup_passphrase_v1", null)
        if (key == null) {
            val bytes = ByteArray(32)
            java.security.SecureRandom().nextBytes(bytes)
            key = android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
            prefs.edit().putString("backup_passphrase_v1", key).commit()
        }
        return key ?: ""
    }

    private fun restoreIdentityFromBackup(core: uniffi.api.IronCore): Boolean {
        val backup = identityBackupPrefs.getString(IDENTITY_BACKUP_KEY, null)
        if (backup.isNullOrBlank()) {
            return false
        }
        return try {
            core.importIdentityBackup(backup, getPlatformSecuredPassphrase())
            Timber.i("Restored identity from Android backup payload")
            // P0_SHARED_IDENTITY: publish restored identity to all subscribers
            val restored = core.getIdentityInfo()
            if (restored != null && restored.initialized) {
                cacheIdentityFields(restored)
            }
            publishIdentityInfo(restored)
            true
        } catch (e: Exception) {
            Timber.w("Identity backup restore failed; fallback to new identity: ${e.message}")
            // Fallback to empty string for compatibility with old backups if present
            try {
                core.importIdentityBackup(backup, "")
                Timber.i("Restored identity using legacy empty passphrase fallback")
                val restored = core.getIdentityInfo()
                if (restored != null && restored.initialized) {
                    cacheIdentityFields(restored)
                }
                publishIdentityInfo(restored)
                true
            } catch (fallbackEx: Exception) {
                Timber.w("Legacy backup restore fallback also failed: ${fallbackEx.message}")
                false
            }
        }
    }

    /**
     * P1_ANDROID_003: Public entry-point for manual identity import from backup string.
     *
     * If [passphrase] is supplied (the user typed the passphrase they used when
     * exporting via [exportIdentityBackup]), it is used directly with no fallback —
     * a wrong user-supplied passphrase should surface as an error, not silently
     * retry with the device-bound key. If null, falls back to the historical
     * device-bound-passphrase behavior (for the app's own auto-backup restore flow).
     */
    fun restoreIdentityFromBackup(backup: String, passphrase: String? = null) {
        val core = ironCore ?: run {
            Timber.w("restoreIdentityFromBackup: Core not initialized, skipping")
            return
        }
        if (passphrase != null) {
            core.importIdentityBackup(backup, passphrase)
            Timber.i("Restored identity from manually pasted backup payload using supplied passphrase")
        } else {
            try {
                core.importIdentityBackup(backup, getPlatformSecuredPassphrase())
                Timber.i("Restored identity from manually pasted backup payload")
            } catch (e: Exception) {
                // Fallback for empty passphrase for legacy manually exported backups
                core.importIdentityBackup(backup, "")
                Timber.i("Restored identity from manually pasted backup payload using legacy fallback")
            }
        }
        // P0_SHARED_IDENTITY: publish the restored identity to all subscribers
        val restored = core.getIdentityInfo()
        if (restored != null && restored.initialized) {
            cacheIdentityFields(restored)
        }
        publishIdentityInfo(restored)
    }

    /**
     * Export a passphrase-encrypted identity backup: the identity signing key,
     * active ratchet sessions, and contacts, encrypted with an Argon2id-derived
     * key. Distinct from [getIdentityExportString], which exports the public
     * identity card (peer ID / public key / listeners) with no encryption.
     */
    fun exportIdentityBackup(passphrase: String): String {
        val core = ironCore ?: throw IllegalStateException("Core not initialized")
        return core.exportIdentityBackup(passphrase)
    }

    private fun persistIdentityBackup(
        core: uniffi.api.IronCore?,
        customSalt: ByteArray? = null,
        // P0_ANDROID_PROGRESS_CALLBACK: drive the UI through the 3 sub-points
        // of the persist flow. The SharedPreferences commit() and the sentinel
        // createNewFile() are both blocking syscalls; the XChaCha20-Poly1305
        // export is a Rust FFI call that takes a few hundred ms on Pixel 6a.
        // Firing a callback before each one gives the user motion during the
        // otherwise-silent "Saving to encrypted storage" stage.
        progress: IdentityProgressCallback = { _, _ -> },
    ) {
        val activeCore = core ?: return
        try {
            progress(IdentityCreationEvent.PersistingToStorage, "Encrypting with XChaCha20-Poly1305…")
            val passphrase = getPlatformSecuredPassphrase()
            val backup = if (customSalt != null) {
                activeCore.exportIdentityBackupFastWithSalt(passphrase, customSalt)
            } else {
                activeCore.exportIdentityBackupFast(passphrase)
            }
            // P0_ANDROID_010: Use commit() for synchronous write.
            // apply() is async and can lose the backup if the process is killed
            // before the disk write completes (e.g., during an ANR crash).
            progress(IdentityCreationEvent.PersistingToStorage, "Committing to encrypted storage…")
            val committed = identityBackupPrefs.edit().putString(IDENTITY_BACKUP_KEY, backup).commit()
            if (!committed) {
                Timber.e("Failed to commit identity backup to SharedPreferences")
            }

            // P0_ANDROID_IDENTITY_003: Create disk-based sentinel file as backup redundancy.
            // If SharedPreferences backup is lost (app data clear, migration, corruption),
            // the sentinel file serves as a secondary indicator of prior identity creation.
            progress(IdentityCreationEvent.PersistingToStorage, "Writing sentinel file…")
            val sentinelFile = File(storagePath, IDENTITY_BACKUP_PREFS + ".sentinel")
            try {
                sentinelFile.createNewFile()
                Timber.d("Created identity backup sentinel file")
            } catch (e: Exception) {
                Timber.w(e, "Failed to create identity backup sentinel file")
            }
        } catch (e: Exception) {
            Timber.w("Failed to persist identity backup payload: ${e.message}")
        }
    }

    // P0: Cache key identity fields to SharedPreferences for instant UI load.
    // Called whenever identity is successfully obtained from Rust core.
    private fun cacheIdentityFields(info: uniffi.api.IdentityInfo) {
        identityCachePrefs.edit().apply {
            putString(IDENTITY_CACHE_ID, info.identityId)
            putString(IDENTITY_CACHE_PUBLIC_KEY, info.publicKeyHex)
            putString(IDENTITY_CACHE_DEVICE_ID, info.deviceId)
            val seniority = info.seniorityTimestamp
            if (seniority != null) {
                putLong(IDENTITY_CACHE_SENIORITY, seniority.toLong())
            } else {
                remove(IDENTITY_CACHE_SENIORITY)
            }
            putString(IDENTITY_CACHE_NICKNAME, info.nickname)
            putString(IDENTITY_CACHE_PEER_ID, info.libp2pPeerId)
            putBoolean(IDENTITY_CACHE_INITIALIZED, info.initialized)
        }.apply()
        Timber.d("Cached identity fields: peerId=%s, id=%s",
            info.libp2pPeerId?.take(8), info.identityId?.take(8))
    }

    // P0: Read cached identity fields for instant UI display without needing Rust core.
    // Returns null if no cache exists (fresh install or cleared data).
    fun readCachedIdentityFields(): uniffi.api.IdentityInfo? {
        if (!identityCachePrefs.contains(IDENTITY_CACHE_INITIALIZED)) {
            return null
        }
        val initialized = identityCachePrefs.getBoolean(IDENTITY_CACHE_INITIALIZED, false)
        if (!initialized) {
            return null
        }
        return uniffi.api.IdentityInfo(
            identityId = identityCachePrefs.getString(IDENTITY_CACHE_ID, null),
            publicKeyHex = identityCachePrefs.getString(IDENTITY_CACHE_PUBLIC_KEY, null),
            deviceId = identityCachePrefs.getString(IDENTITY_CACHE_DEVICE_ID, null),
            seniorityTimestamp = identityCachePrefs.getLong(IDENTITY_CACHE_SENIORITY, 0L)
                .toULong().let { if (it == 0uL) null else it },
            initialized = initialized,
            nickname = identityCachePrefs.getString(IDENTITY_CACHE_NICKNAME, null),
            libp2pPeerId = identityCachePrefs.getString(IDENTITY_CACHE_PEER_ID, null)
        )
    }

    fun getMeshService(): uniffi.api.MeshService? = meshService

    fun setPlatformBridge(bridge: uniffi.api.PlatformBridge) {
        meshService?.setPlatformBridge(bridge)
        // Wire TransportManager to AndroidPlatformBridge for BLE adjustment application
        if (bridge is com.scmessenger.android.service.AndroidPlatformBridge) {
            transportManager?.let { tm ->
                bridge.setTransportManager(tm)
                // Wire WiFi Aware transport for PlatformBridge FFI delegation
                tm.getWifiAwareTransport()?.let { aware ->
                    bridge.setWifiAwareTransport(aware)
                }
                // Wire WiFi Direct transport for PlatformBridge FFI delegation
                tm.getWifiDirectTransport()?.let { direct ->
                    bridge.setWifiDirectTransport(direct)
                }
            }
            // Wire BLE components into platform bridge for data forwarding
            bridge.setBleComponents(bleAdvertiser, bleScanner, bleGattClient, bleGattServer)
        }
    }

    /**
     * Stop the mesh service and all transports.
     */
    @Synchronized
    fun stopMeshService() {
        stopNetworkChangeWatch()
        networkDetector.stopMonitoring()
        pendingOutboxRetryJob?.cancel()
        pendingOutboxRetryJob = null
        coverTrafficJob?.cancel()
        coverTrafficJob = null
        pendingReceiptSendJobs.values.forEach { it.cancel() }
        pendingReceiptSendJobs.clear()

        try {
            repoScope.launch { bleScanner?.stopScanning() }
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE scanner")
        }
        try {
            bleAdvertiser?.stopAdvertising()
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE advertiser")
        }
        try {
            bleGattServer?.stop()
        } catch (e: Exception) {
            Timber.w(e, "Failed to stop BLE GATT server")
        }
        try {
            bleGattClient?.cleanup()
        } catch (e: Exception) {
            Timber.w(e, "Failed to cleanup BLE GATT client")
        }

        kotlin.runCatching { wifiTransportManager?.stopDiscovery() }
            .onFailure { Timber.w(it, "Failed to stop WiFi transport") }

        // TransportManager.startAll() is called when the service starts (see
        // above); stopAll() must be called here too so WiFi Aware detaches
        // (WifiAwareSession.close()) instead of leaking an attach session
        // for the lifetime of the process after the mesh service stops.
        kotlin.runCatching { transportManager?.stopAll() }
            .onFailure { Timber.w(it, "Failed to stop TransportManager (BLE/WiFi Aware/WiFi Direct/mDNS)") }

        // shutdown() is now a suspend FFI call; teardown must complete before
        // meshService.stop() below, so block here (bounded: it only awaits the
        // swarm command-channel acknowledgement).
        kotlin.runCatching { kotlinx.coroutines.runBlocking { swarmBridge?.shutdown() } }
            .onFailure { Timber.w(it, "Failed to shutdown swarm bridge") }

        kotlin.runCatching { meshService?.stop() }
            .onFailure { Timber.w(it, "Failed to stop Rust mesh service") }
        
        identitySyncSentPeers.clear()
        historySyncSentPeers.clear()
        identityEmissionCache.clear()
        connectedEmissionCache.clear()
        mdnsLanPeers.clear()
        
        // Clear discovered peers from UI on service stop
        _discoveredPeers.value = emptyMap()
        Timber.i("Cleared all discovered peers on mesh service stop")

        // Clear references to avoid stale lifecycle state on next start.
        coreDelegate = null
        swarmBridge = null
        ironCore = null
        meshService = null
        bleScanner = null
        bleAdvertiser = null
        bleGattServer = null
        bleGattClient = null
        wifiTransportManager = null
        transportManager = null

        _serviceState.value = uniffi.api.ServiceState.STOPPED
        _serviceStats.value = null
        serviceStartedAtEpochSec = 0L

        Timber.i("Mesh service stopped")
    }

    /**
     * Pause the mesh service (reduced activity).
     */
    fun pauseMeshService() {
        meshService?.pause()
        Timber.d("Mesh service paused")
    }

    /**
     * Resume the mesh service (full activity).
     */
    fun resumeMeshService() {
        meshService?.resume()
        Timber.d("Mesh service resumed")
    }

    /**
     * Reset service runtime counters for a fresh diagnostics window.
     */
    fun resetServiceStats() {
        meshService?.resetStats()
        _serviceStats.value = meshService?.getStats()
        Timber.d("Mesh service stats reset")
    }

    /**
     * Called when WiFi connectivity is recovered. Immediately flushes
     * the pending outbox and re-primes relay connections to maximize
     * delivery opportunity.
     */
    fun notifyNetworkRecovered() {
        Timber.i("WiFi recovered — flushing pending outbox immediately")
        Timber.i("network_recovery wifi=true flush_triggered=true")
        repoScope.launch {
            primeRelayBootstrapConnections()
            flushPendingOutbox("wifi_recovered")
        }
    }

    /**
     * Get current service state.
     */
    fun getServiceStateSync(): uniffi.api.ServiceState {
        return meshService?.getState() ?: uniffi.api.ServiceState.STOPPED
    }

    /**
     * Update and emit service stats.
     */
    private fun updateStats() {
        refreshAddressesSnapshots()
        try {
            val stats = meshService?.getStats()
            if (stats != null) {
                // Combine swarm stats with our own discovery map
                val discovered = _discoveredPeers.value
                val fullCount = discovered.values.count { it.isFull }
                val headlessCount = discovered.values.count { !it.isFull }
                val fallbackUptimeSecs = if (serviceStartedAtEpochSec > 0L) {
                    ((System.currentTimeMillis() / 1000) - serviceStartedAtEpochSec).coerceAtLeast(0L).toULong()
                } else {
                    0uL
                }
                val normalizedStats = if (stats.uptimeSecs == 0uL && fallbackUptimeSecs > 0uL) {
                    uniffi.api.ServiceStats(
                        peersDiscovered = stats.peersDiscovered,
                        messagesRelayed = stats.messagesRelayed,
                        bytesTransferred = stats.bytesTransferred,
                        uptimeSecs = fallbackUptimeSecs
                    )
                } else {
                    stats
                }

                // We use a custom stats object or just update the one from core
                // For now, let's keep the core one but maybe log the detailed count
                Timber.d("Mesh Stats: ${normalizedStats.peersDiscovered} peers (Core), $fullCount full, $headlessCount headless (Repo)")

                _serviceStats.value = normalizedStats

                // Emit event for UI
                repoScope.launch {
                    com.scmessenger.android.service.MeshEventBus.emitStatusEvent(
                        com.scmessenger.android.service.StatusEvent.StatsUpdated(normalizedStats)
                    )
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "Failed to get service stats")
        }
    }

    private fun startPeriodicStatsUpdate() {
        repoScope.launch {
            while (isActive) {
                kotlinx.coroutines.delay(5000) // Update every 5 seconds
                updateStats()
            }
        }
    }

    // ========================================================================
    // CONTACTS
    // ========================================================================

    /**
     * Canonical contact ID normalization function.
     * Maps all ID variants (SHA-256(publicKey), Hash(pubkey+identity), LibP2P Peer ID)
     * to a single canonical form using public key as the primary key.
     *
     * This ensures consistent contact identification across all ID schemes:
     * - Identity/Discovery: SHA-256(publicKey) → canonical
     * - Storage/Database: Hash(pubkey+identity) → canonical
     * - Transport: LibP2P Peer ID → canonical
     *
     * @param id Any ID format (public_key_hex, identity_id, or libp2p_peer_id)
     * @return Canonical identity_id (Blake3 hash of public key)
     */
    /**
     * ID-STANDARDIZATION-001: Comprehensive ID sanity check
     * Validates that an ID is consistent and can be properly resolved
     */
    private fun validateAndStandardizeId(id: String, operation: String): String {
        if (id.isBlank()) {
            throw IllegalArgumentException("ID cannot be blank for operation: $operation")
        }
        
        val trimmed = id.trim()
        val resolvedCanonicalId = canonicalContactId(trimmed)
        
        // Additional validation: Check if this ID maps to multiple contacts (ambiguity check)
        try {
            val contacts = contactManager?.list().orEmpty()
            val matchingContacts = contacts.filter {
                val contactId = canonicalContactId(it.peerId)
                PeerIdValidator.isSame(contactId, resolvedCanonicalId)
            }
            
            if (matchingContacts.size > 1) {
                Timber.w("ID_AMBIGUITY: Operation '$operation' on ID '$trimmed' matches ${matchingContacts.size} contacts:")
                matchingContacts.forEach { contact ->
                    Timber.w("  - Contact: ${contact.peerId} (key=${contact.publicKey?.take(8)})")
                }
                // In case of ambiguity, prefer the first one but log the issue
            }
        } catch (e: Exception) {
            Timber.w("ID_VALIDATION: Could not check contact ambiguity for '$trimmed': ${e.message}")
        }
        
        return resolvedCanonicalId
    }

    private fun canonicalContactId(id: String): String {
        val trimmed = id.trim()
        if (trimmed.isEmpty()) return trimmed

        // AND-SEND-BTN-001: Check in-memory cache first to avoid synchronous FFI on UI thread.
        // Identity resolution is deterministic — once resolved, the mapping never changes.
        identityIdCache[trimmed]?.let { return it }

        // ID-STANDARDIZATION-001: Comprehensive ID resolution with sanity checks
        Timber.d("ID_RESOLUTION: Input ID '$trimmed' (length=${trimmed.length})")

        // UNIFIED ID FIX: Never short-circuit on identity_id shape.
        // resolveIdentity() handles all formats (public_key_hex, identity_id, libp2p_peer_id)
        // and always returns public_key_hex (canonical). The 64-hex short-circuit was
        // allowing identity_id to be stored as Contact.peerId, causing duplicate contacts.

        // Try to resolve to canonical public_key_hex via IronCore
        try {
            val resolvedPk = ironCore?.resolveIdentity(trimmed)
            if (resolvedPk != null) {
                val normalized = PeerIdValidator.normalize(resolvedPk)
                identityIdCache[trimmed] = normalized
                Timber.d("ID_RESOLUTION: Resolved '$trimmed' -> public_key_hex: ${normalized.take(8)}...")
                return normalized
            } else {
                Timber.w("ID_RESOLUTION: IronCore returned null for '$trimmed'")
            }
        } catch (e: Exception) {
            Timber.w("ID_RESOLUTION: Failed to resolve ID '$trimmed' to public_key_hex: ${e.message}")
        }

        // Fallback: Normalize the input ID (libp2p peer ID or other format)
        val normalizedFallback = PeerIdValidator.normalize(trimmed)
        identityIdCache[trimmed] = normalizedFallback
        Timber.d("ID_RESOLUTION: Fallback to normalized input: ${normalizedFallback.take(16)}...")
        return normalizedFallback
    }
    
    /**
     * Legacy canonicalId function - kept for backward compatibility.
     * @deprecated Use canonicalContactId() for new code.
     */
    private fun canonicalId(id: String): String {
        return canonicalContactId(id)
    }

    fun addContact(contact: uniffi.api.Contact) {
        // CRITICAL: Validate public key before storing
        val trimmedKey = contact.publicKey?.trim()
        if (trimmedKey.isNullOrEmpty()) {
            Timber.e("addContact rejected: public key is empty for peer ${contact.peerId}")
            return
        }
        if (trimmedKey.length != 64) {
            Timber.e("addContact rejected: public key has invalid length ${trimmedKey.length} (expected 64) for peer ${contact.peerId}")
            return
        }
        if (!trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
            Timber.e("addContact rejected: public key contains invalid characters for peer ${contact.peerId}")
            return
        }

        // P0_ANDROID_022: Reject relay peers — they are infrastructure, not user
        // contacts. Mirrors the check inside `upsertFederatedContact` and
        // `validatePeerBeforeContactCreation` so the user-facing public path
        // is also guarded. We check both the canonical peerId and the
        // derived peerId from the public key (relay peerIds can be encoded
        // in either form).
        if (isBootstrapRelayPeer(contact.peerId) ||
            isBootstrapRelayPeerFromKey(trimmedKey)) {
            Timber.i("addContact rejected: peer ${contact.peerId} is a bootstrap relay (infrastructure, not a user contact)")
            return
        }

        // Idempotent upsert unique constraint check: match existing contact by public key
        val existingWithKey = try {
            contactManager?.list()?.firstOrNull {
                it.publicKey?.trim()?.lowercase() == trimmedKey.lowercase()
            }
        } catch (_: Exception) { null }

        // Public-key hex is the sole persisted contact identity. A libp2p
        // peer ID is transport routing metadata and may change across device
        // reinstalls; keeping it as Contact.peerId creates duplicate contacts
        // and makes message lookup depend on the import surface used.
        val canonicalContactId = trimmedKey.lowercase()
        val finalContact = if (existingWithKey != null) {
            Timber.i("Idempotent contact upsert: peer ${contact.peerId} matches existing contact ${existingWithKey.peerId} by public key")
            uniffi.api.Contact(
                peerId = canonicalContactId,
                nickname = contact.nickname ?: existingWithKey.nickname,
                localNickname = contact.localNickname ?: existingWithKey.localNickname,
                publicKey = trimmedKey,
                addedAt = existingWithKey.addedAt,
                lastSeen = if (contact.lastSeen != null) {
                    val currentLastSeen = existingWithKey.lastSeen ?: 0u
                    if (contact.lastSeen!! > currentLastSeen) contact.lastSeen else currentLastSeen
                } else existingWithKey.lastSeen,
                notes = contact.notes ?: existingWithKey.notes,
                lastKnownDeviceId = contact.lastKnownDeviceId ?: existingWithKey.lastKnownDeviceId,
                verifiedAt = contact.verifiedAt ?: existingWithKey.verifiedAt,
                isTombstone = false
            )
        } else {
            uniffi.api.Contact(
                peerId = canonicalContactId,
                nickname = contact.nickname,
                localNickname = contact.localNickname,
                publicKey = trimmedKey,
                addedAt = contact.addedAt,
                lastSeen = contact.lastSeen,
                notes = contact.notes,
                lastKnownDeviceId = contact.lastKnownDeviceId,
                verifiedAt = contact.verifiedAt,
                isTombstone = contact.isTombstone
            )
        }

        contactManager?.add(finalContact)
        val routing = parseRoutingHints(finalContact.notes)
        annotateIdentityInLedger(
            routePeerId = routing.libp2pPeerId,
            listeners = routing.listeners,
            publicKey = finalContact.publicKey,
            nickname = finalContact.nickname
        )
        Timber.d("Contact added/updated: ${finalContact.peerId}")
    }

    /**
     * Add a contact by peer ID, resolving the public key from discovery data.
     * Returns true if the contact was added, false if the public key could not be resolved.
     */
    fun addContactByPeerId(peerId: String, nickname: String? = null): Boolean {
        ensureServiceInitializedFireAndForget()
        try {
            val canonical = canonicalId(peerId)

            // Already a contact? Just update nickname if provided.
            val existing = contactManager?.get(canonical)
            if (existing != null) {
                if (nickname != null) {
                    addContact(uniffi.api.Contact(
                        peerId = existing.peerId,
                        nickname = nickname,
                        localNickname = existing.localNickname,
                        publicKey = existing.publicKey,
                        addedAt = existing.addedAt,
                        lastSeen = existing.lastSeen,
                        notes = existing.notes,
                        lastKnownDeviceId = existing.lastKnownDeviceId,
                        verifiedAt = existing.verifiedAt,
                        isTombstone = false
                    ))
                }
                return true
            }

            // Resolve public key from discovery data
            val discoveryInfo = _discoveredPeers.value[canonical]
                ?: _discoveredPeers.value.entries.firstOrNull { it.value.peerId == canonical }?.value
            val publicKey = discoveryInfo?.publicKey

            if (publicKey.isNullOrBlank()) {
                Timber.e("Cannot add contact for $canonical: no public key available from discovery data")
                return false
            }

            val contact = uniffi.api.Contact(
                peerId = canonical,
                nickname = nickname ?: discoveryInfo.nickname,
                localNickname = null,
                publicKey = publicKey,
                addedAt = (System.currentTimeMillis() / 1000).toULong(),
                lastSeen = null,
                notes = null,
                lastKnownDeviceId = null,
                verifiedAt = null,
                isTombstone = false
            )
            addContact(contact)
            return true
        } catch (e: Exception) {
            Timber.e(e, "Failed to add contact by peer ID: $peerId")
            return false
        }
    }

    fun getContact(peerId: String): uniffi.api.Contact? {
        return contactManager?.get(canonicalId(peerId))
    }

    /**
     * WS14: Check if a conversation exists with the given peer.
     * Used for notification classification (DM vs DM Request).
     */
    fun hasConversationWith(peerId: String): Boolean {
        val canonical = canonicalId(peerId)
        return try {
            // Check if we have any message history with this peer
            historyManager?.conversation(canonical, 1u)?.isNotEmpty() == true
        } catch (e: Exception) {
            false
        }
    }

    fun removeContact(peerId: String) {
        val canonical = canonicalId(peerId)
        contactManager?.remove(canonical)
        try {
            historyManager?.removeConversation(canonical)
        } catch (e: Exception) {
            Timber.w("Failed to remove conversation history for $canonical: ${e.message}")
        }
        
        // Clear in-memory caches to prevent stale contact from showing (CONTACT-STALE-001)
        // 1. Remove from discovered peers cache
        _discoveredPeers.update { current ->
            val keysToRemove = current.filter { (key, info) ->
                key == canonical ||
                    info.peerId == canonical ||
                    PeerIdValidator.isSame(key, canonical) ||
                    PeerIdValidator.isSame(info.peerId, canonical)
            }.keys
            
            if (keysToRemove.isEmpty()) current else current - keysToRemove
        }
        
        // 2. Remove from BLE route observations cache
        val keysToRemove = bleRouteObservations.keys.filter { key ->
            key == canonical || PeerIdValidator.isSame(key, canonical)
        }
        keysToRemove.forEach { bleRouteObservations.remove(it) }
        
        Timber.d("Contact removed: $canonical and their message history, caches cleared")
    }

    fun listContacts(): List<uniffi.api.Contact> {
        return contactManager?.list() ?: emptyList()
    }

    fun searchContacts(query: String): List<uniffi.api.Contact> {
        return contactManager?.search(query) ?: emptyList()
    }

    fun setContactNickname(peerId: String, nickname: String?) {
        contactManager?.setNickname(peerId, nickname)
        Timber.d("Contact nickname updated: $peerId -> $nickname")
    }

    /** Mark a contact as verified after an out-of-band safety-number comparison. */
    fun markContactVerified(peerId: String) {
        contactManager?.markVerified(peerId)
        Timber.d("Contact marked verified: $peerId")
    }

    /** Clear a contact's verification status (e.g. after a key change). */
    fun unverifyContact(peerId: String) {
        contactManager?.unverify(peerId)
        Timber.d("Contact verification cleared: $peerId")
    }

    /**
     * Compute the Signal-style safety number for comparing identities with [theirPublicKeyHex]
     * out-of-band. Returns null if our own identity isn't initialized yet, or an empty string
     * if the underlying keys are malformed (S5) - callers must treat both as error states, never
     * render an empty string as if it were a real safety number.
     */
    fun computeSafetyNumber(theirPublicKeyHex: String): String? {
        val ourKey = identityInfo.value?.publicKeyHex ?: return null
        return uniffi.api.safetyNumber(ourKey, theirPublicKeyHex)
    }

    fun getContactCount(): UInt {
        return contactManager?.count() ?: 0u
    }

    // ========================================================================
    // BLOCKING
    // ========================================================================

    fun blockPeer(peerId: String, deviceId: String? = null, reason: String? = null) {
        ensureServiceInitializedFireAndForget()
        try {
            ironCore?.blockPeer(peerId, deviceId, reason)
            Timber.i("Blocked peer: $peerId (device: $deviceId, reason: $reason)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to block peer: $peerId")
        }
    }

    fun unblockPeer(peerId: String, deviceId: String? = null) {
        ensureServiceInitializedFireAndForget()
        try {
            ironCore?.unblockPeer(peerId, deviceId)
            Timber.i("Unblocked peer: $peerId (device: $deviceId)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to unblock peer: $peerId")
        }
    }

    /**
     * WS14: Mute notifications for a peer by blocking them.
     * This is the Android implementation of "mute" - blocking a peer
     * prevents them from sending notifications (per NotificationHelper logic).
     */
    fun mutePeer(peerId: String, deviceId: String? = null, reason: String? = null) {
        ensureServiceInitializedFireAndForget()
        try {
            ironCore?.blockPeer(peerId, deviceId, reason)
            Timber.i("Muted peer: $peerId (device: $deviceId, reason: $reason)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to mute peer: $peerId")
        }
    }

    /**
     * Block a peer AND delete all their stored messages (cascade purge).
     * Future payloads from this peer are dropped at the ingress layer.
     */
    fun blockAndDeletePeer(peerId: String, deviceId: String? = null, reason: String? = null) {
        ensureServiceInitializedFireAndForget()
        try {
            ironCore?.blockAndDeletePeer(peerId, deviceId, reason)
            Timber.i("Blocked and deleted peer: $peerId (device: $deviceId, reason: $reason)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to block and delete peer: $peerId")
        }
    }

    fun isBlocked(peerId: String, deviceId: String? = null): Boolean {
        ensureServiceInitializedFireAndForget()
        return try {
            ironCore?.isPeerBlocked(peerId, deviceId) ?: false
        } catch (e: Exception) {
            Timber.w(e, "Failed to check if peer blocked: $peerId")
            false
        }
    }

    fun listBlockedPeers(): List<uniffi.api.BlockedIdentity> {
        ensureServiceInitializedFireAndForget()
        return try {
            ironCore?.listBlockedPeers() ?: emptyList()
        } catch (e: Exception) {
            Timber.w(e, "Failed to list blocked peers")
            emptyList()
        }
    }

    fun getBlockedCount(): UInt {
        ensureServiceInitializedFireAndForget()
        return try {
            ironCore?.blockedCount() ?: 0u
        } catch (e: Exception) {
            Timber.w(e, "Failed to get blocked count")
            0u
        }
    }

    /**
     * WS14: Get pending message requests.
     * A message request is from a peer who has sent a message but is not yet a contact.
     * Returns a list of MessageRequest objects with sender info and preview.
     */
    fun getPendingMessageRequests(): List<uniffi.api.MessageRequest> {
        ensureServiceInitializedFireAndForget()
        return try {
            val contacts = contactManager?.list() ?: emptyList()
            val contactPeerIds = contacts.map { it.peerId }.toSet()

            // Get all messages from inbox to find senders without conversations
            val inboxMessages = ironCore?.drainReceivedMessages() ?: emptyList()

            // Get unique peerIds from messages that aren't contacts
            val requestPeerIds = inboxMessages
                .map { it.senderId }
                .filter { it !in contactPeerIds }
                .distinct()

            // Build MessageRequest objects for each peer
            requestPeerIds.map { peerId ->
                // Get the most recent message from this peer for preview
                val recentMessage = inboxMessages
                    .filter { it.senderId == peerId }
                    .maxByOrNull { it.receivedAt }

                // Get contact info if available (for nickname)
                val contact = contactManager?.get(canonicalId(peerId))

                uniffi.api.MessageRequest(
                    peerId = peerId,
                    nickname = contact?.nickname ?: contact?.localNickname,
                    messagePreview = recentMessage?.payload?.let { bytes ->
                        try { bytes.decodeToString() } catch (_: Exception) { "[binary]" }
                    } ?: "",
                    messageTimestamp = recentMessage?.receivedAt ?: 0u,
                    messageCount = inboxMessages.count { it.senderId == peerId }.toUInt()
                )
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to get pending message requests")
            emptyList()
        }
    }

    // ========================================================================
    // CRYPTO UTILITIES
    // ========================================================================

    fun signData(data: ByteArray): uniffi.api.SignatureResult? {
        ensureServiceInitializedFireAndForget()
        return try {
            ironCore?.signData(data)
        } catch (e: Exception) {
            Timber.e(e, "Failed to sign data")
            null
        }
    }

    fun verifySignature(data: ByteArray, signature: ByteArray, publicKeyHex: String): Boolean {
        ensureServiceInitializedFireAndForget()
        return try {
            ironCore?.verifySignature(data, signature, publicKeyHex) ?: false
        } catch (e: Exception) {
            Timber.e(e, "Failed to verify signature")
            false
        }
    }

    // ========================================================================
    // WS13 DEVICE MANAGEMENT
    // ========================================================================

    fun getDeviceId(): String? {
        return ironCore?.getDeviceId()
    }

    fun getSeniorityTimestamp(): ULong? {
        return ironCore?.getSeniorityTimestamp()
    }

    fun getRegistrationState(identityId: String): uniffi.api.RegistrationStateInfo? {
        return ironCore?.getRegistrationState(identityId)
    }

    // ========================================================================
    // LOGGING
    // ========================================================================

    fun exportLogs(): String? {
        return try {
            ironCore?.exportLogs()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export logs")
            null
        }
    }

    // ========================================================================
    // QUEUE COUNTS
    // ========================================================================

    fun getInboxCount(): UInt {
        return ironCore?.inboxCount() ?: 0u
    }

    // ========================================================================
    // CONTACT DEVICE ID (WS13)
    // ========================================================================

    fun updateContactDeviceId(peerId: String, deviceId: String?) {
        try {
            contactManager?.updateDeviceId(peerId, deviceId)
            Timber.i("Updated device ID for $peerId: $deviceId")
        } catch (e: Exception) {
            Timber.w(e, "Failed to update device ID for $peerId")
        }
    }

    // ========================================================================
    // MESSAGE HISTORY
    // ========================================================================

    /**
     * Get identity info without blocking.
     * Returns null if the service is not yet initialized; call again after a delay if needed.
     * This is non-blocking and safe to call from the main thread during UI composition.
     */
    fun getIdentityInfoNonBlocking(): uniffi.api.IdentityInfo? {
        // P0_SHARED_IDENTITY: Check the published StateFlow first — this is the
        // fastest path and catches identity that was just created in another
        // ViewModel (e.g., onboarding) before the core/cache paths run.
        val published = _identityInfo.value
        if (published != null && published.initialized) {
            return published
        }

        // Try to get identity info even if the service isn't fully RUNNING yet.
        // ironCore may already have identity loaded from sled before the swarm
        // finishes binding. This eliminates the 30-60s polling gap in Settings UI.
        val core = ironCore
        if (core != null) {
            val info = core.getIdentityInfo()
            if (info != null && info.initialized) {
                val resolvedInfo = if (info.nickname.isNullOrBlank()) {
                    val cachedNickname = dataStoreNicknameSnapshot
                    if (!cachedNickname.isNullOrBlank()) {
                        Timber.w("getIdentityInfoNonBlocking: Rust core nickname blank; using DataStore fallback: $cachedNickname")
                        setNickname(cachedNickname)
                        info.copy(nickname = cachedNickname)
                    } else {
                        info
                    }
                } else {
                    info
                }
                cacheIdentityFields(resolvedInfo)
                publishIdentityInfo(resolvedInfo)
                return resolvedInfo
            }
            // Core exists but reports uninitialized (sled lost/corrupt).
            // Fall back to SharedPreferences cache — the identity was created
            // in a previous session and the backup is still valid.
            val cached = readCachedIdentityFields()
            if (cached != null && cached.initialized) {
                val resolvedCached = if (cached.nickname.isNullOrBlank()) {
                    val cachedNickname = dataStoreNicknameSnapshot
                    if (!cachedNickname.isNullOrBlank()) {
                        cached.copy(nickname = cachedNickname)
                    } else {
                        cached
                    }
                } else {
                    cached
                }
                Timber.w("getIdentityInfoNonBlocking: core uninitialized but cache has identity; using cached")
                publishIdentityInfo(resolvedCached)
                return resolvedCached
            }
            // Neither core nor cache has identity
            publishIdentityInfo(info)
            return info
        }
        // P0: Fall back to cached identity fields when Rust core isn't ready yet.
        // This eliminates the 30-60s "Unavailable" gap during service startup.
        val cached = readCachedIdentityFields()
        if (cached != null) {
            val resolvedCached = if (cached.nickname.isNullOrBlank()) {
                val cachedNickname = dataStoreNicknameSnapshot
                if (!cachedNickname.isNullOrBlank()) {
                    cached.copy(nickname = cachedNickname)
                } else {
                    cached
                }
            } else {
                cached
            }
            Timber.d("getIdentityInfoNonBlocking: using cached identity (core not ready)")
            // P0_SHARED_IDENTITY: also publish cached identity so subscribers see it
            publishIdentityInfo(resolvedCached)
            return resolvedCached
        }
        // Last resort: only return null if there's truly no core available and no cache.
        val state = meshService?.getState()
        if (state != uniffi.api.ServiceState.RUNNING) {
            return null
        }
        return getIdentityInfoSync()
    }

    fun getIdentityInfoSync(): uniffi.api.IdentityInfo? {
        ensureServiceInitializedFireAndForget()
        kotlin.runCatching { ensureLocalIdentityFederation() }
            .onFailure { Timber.w(it, "Failed to hydrate identity before getIdentityInfo") }
        val result = ironCore?.getIdentityInfo()
        Timber.d(
            "getIdentityInfo: result=%s, initialized=%s, nickname=%s, libp2pPeerId=%s",
            result?.identityId,
            result?.initialized,
            result?.nickname,
            result?.libp2pPeerId
        )
        // P0: Cache identity fields for instant UI load on next startup
        if (result != null && result.initialized) {
            val resolvedResult = if (result.nickname.isNullOrBlank()) {
                val cachedNickname = dataStoreNicknameSnapshot
                if (!cachedNickname.isNullOrBlank()) {
                    Timber.w("getIdentityInfo: Rust core nickname blank; using DataStore fallback: $cachedNickname")
                    setNickname(cachedNickname)
                    result.copy(nickname = cachedNickname)
                } else {
                    result
                }
            } else {
                result
            }
            cacheIdentityFields(resolvedResult)
            publishIdentityInfo(resolvedResult)
            return resolvedResult
        }
        // Core returned uninitialized — fall back to SharedPreferences cache
        val cached = readCachedIdentityFields()
        if (cached != null && cached.initialized) {
            val resolvedCached = if (cached.nickname.isNullOrBlank()) {
                val cachedNickname = dataStoreNicknameSnapshot
                if (!cachedNickname.isNullOrBlank()) {
                    cached.copy(nickname = cachedNickname)
                } else {
                    cached
                }
            } else {
                cached
            }
            Timber.w("getIdentityInfo: core uninitialized but cache has identity; using cached")
            publishIdentityInfo(resolvedCached)
            return resolvedCached
        }
        // P0_SHARED_IDENTITY: publish to all subscribers
        publishIdentityInfo(result)
        return result
    }

    fun setNickname(nickname: String) {
        val trimmed = nickname.trim()
        if (trimmed.isEmpty()) {
            Timber.w("Refusing to set blank nickname")
            return
        }
        Timber.d("setNickname: Requested nickname='%s', trimmed='%s'", nickname, trimmed)
        val core = ironCore ?: run {
            Timber.w("setNickname: Cannot set nickname: IronCore is not initialized")
            return
        }
        try {
            Timber.d("setNickname: Calling core.setNickname()")
            core.setNickname(trimmed)
            Timber.i("Nickname set to: $trimmed")
            // Verify the nickname was persisted
            val info = ironCore?.getIdentityInfo()
            Timber.d(
                "setNickname: Verification - nickname=%s, initialized=%s",
                info?.nickname,
                info?.initialized
            )
            // P0: Update cache after nickname change
            if (info != null && info.initialized) {
                cacheIdentityFields(info)
            }
            // P0_SHARED_IDENTITY: publish to all subscribers (Main/Identity/Settings VMs)
            publishIdentityInfo(info)
        } catch (e: Exception) {
            Timber.e(e, "Rust core failed to set nickname")
            return
        }
        persistIdentityBackup(core)
        // If swarm start was postponed before identity/nickname was ready, resume now.
        // Launched on repoScope: initializeAndStartSwarm is a suspend fun that can
        // block up to 15s waiting for the listener bind, and setNickname may be
        // invoked from the main thread.
        updateBleIdentityBeacon()
        identitySyncSentPeers.clear()
        historySyncSentPeers.clear()
        repoScope.launch {
            initializeAndStartSwarm()
            // getPeers() is a suspend FFI call now; enumerate after the swarm
            // (re)start so the nickname sync reaches currently-connected peers.
            val connectedPeers = try {
                swarmBridge?.getPeers().orEmpty()
            } catch (e: Exception) {
                Timber.d("Unable to enumerate peers for nickname sync: ${e.message}")
                emptyList()
            }
            connectedPeers.forEach { routePeerId ->
                sendIdentitySyncIfNeeded(routePeerId = routePeerId)
            }
        }
    }

    /**
     * Sync nickname from DataStore fallback to Rust Core.
     *
     * Called when Rust Core becomes available and we need to push any locally-cached
     * nickname that was stored before the Rust Core was initialized.
     *
     * This is the P1_ANDROID_020 fix: ensures federated nickname in the mesh
     * doesn't remain stale when DataStore has a cached value.
     *
     * The nickname is stored in the DataStore file "app_preferences" under the key "identity_nickname".
     * We read this directly from the SharedPreferences backup file that DataStore creates.
     */
    fun syncNicknameFromDatastore() {
        // Issue 6: read DataStore on repoScope instead of runBlocking on the
        // caller's thread. The sync is inherently asynchronous — callers
        // (SettingsViewModel) only trigger it, they don't consume a result.
        repoScope.launch { syncNicknameFromDatastoreInternal() }
    }

    private suspend fun syncNicknameFromDatastoreInternal() {
        // Read the nickname from the PreferencesRepository (DataStore fallback)
        val cachedNickname = try {
            preferencesRepository?.identityNickname?.firstOrNull()
        } catch (e: Exception) {
            Timber.e(e, "syncNicknameFromDatastore: Failed to read from preferencesRepository")
            null
        }

        if (cachedNickname.isNullOrBlank()) {
            Timber.d("syncNicknameFromDatastore: No nickname in DataStore")
            return
        }

        // Check if IronCore is available
        val core = ironCore ?: run {
            Timber.d("syncNicknameFromDatastore: IronCore not yet initialized, skipping")
            return
        }

        // Get current identity info to check if we already have this nickname
        val currentInfo = core.getIdentityInfo()
        if (currentInfo?.nickname == cachedNickname) {
            Timber.d("syncNicknameFromDatastore: Nickname already synced: %s", cachedNickname)
            return
        }

        // Push the cached nickname to Rust Core
        Timber.i("syncNicknameFromDatastore: Pushing DataStore nickname to IronCore: %s", cachedNickname)
        try {
            core.setNickname(cachedNickname.trim())
            Timber.i("syncNicknameFromDatastore: Successfully pushed nickname to IronCore")
            // Update the cache as well
            val info = core.getIdentityInfo()
            if (info != null && info.initialized) {
                cacheIdentityFields(info)
            }
        } catch (e: Exception) {
            Timber.e(e, "syncNicknameFromDatastore: Failed to push nickname to IronCore")
        }
    }

    fun setLocalNickname(peerId: String, nickname: String?) {
        try {
            contactManager?.setLocalNickname(peerId, nickname)
            Timber.i("Local nickname for $peerId set to: $nickname")
            // Refresh discovery map if this peer is in it
                _discoveredPeers.update { current ->
                    val existing = current[peerId]
                    if (existing != null) {
                        current + (peerId to existing.copy(localNickname = nickname))
                    } else {
                        current
                    }
                }
        } catch (e: Exception) {
            Timber.e(e, "Failed to set local nickname for $peerId")
        }
    }

    open suspend fun sendMessage(peerId: String, content: String) {
        kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            val routingPeerId = PeerIdValidator.normalize(peerId)
            val initialMessageId = java.util.UUID.randomUUID().toString()
            val now = (System.currentTimeMillis() / 1000).toULong()

            // 1. Resolve public key FIRST to determine canonical ID
            var publicKey: String? = try {
                ironCore?.resolveIdentity(routingPeerId)
            } catch (e: Exception) {
                null
            }

            // Fallback: Try contact manager
            val contact = contactManager?.get(routingPeerId)
            if (publicKey == null && contact != null && !contact.publicKey.isNullOrEmpty()) {
                publicKey = contact.publicKey.trim()
                
                // CRITICAL: Validate public key length
                if (publicKey.length != 64) {
                    Timber.e("SEND_MSG_BUG: Contact has invalid public key length: ${publicKey.length} (expected 64)")
                    Timber.e("SEND_MSG_BUG: Contact peer_id: ${contact.peerId}")
                    Timber.e("SEND_MSG_BUG: Routing peer_id: $routingPeerId")
                    Timber.e("SEND_MSG_BUG: Public key value: '$publicKey'")
                    
                    // Try to recover from discovered peers
                    val discoveredPeer = _discoveredPeers.value.entries.find {
                        PeerIdValidator.isSame(it.key, routingPeerId) ||
                        PeerIdValidator.isSame(it.key, contact.peerId) ||
                        (it.value.publicKey == contact.publicKey)
                    }?.value
                    
                    if (discoveredPeer != null && !discoveredPeer.publicKey.isNullOrEmpty() && 
                        discoveredPeer.publicKey.trim().length == 64) {
                        publicKey = discoveredPeer.publicKey.trim()
                        Timber.w("SEND_MSG_RECOVER: Using public key from discovered peers: ${publicKey?.take(8)}")
                    } else {
                        Timber.e("Contact public key is invalid (${publicKey.length} chars) and no discovered peer available for recovery")
                        return@withContext
                    }
                } else {
                    Timber.d("SEND_MSG: Resolved from contact: key=${publicKey?.take(8)}")
                }
            }

            // Fallback: Try discovered peers
            if (publicKey == null) {
                val discoveredPeer = _discoveredPeers.value.entries.find {
                    PeerIdValidator.isSame(it.key, routingPeerId)
                }?.value
                if (discoveredPeer != null && !discoveredPeer.publicKey.isNullOrEmpty()) {
                    publicKey = discoveredPeer.publicKey.trim()
                    Timber.d("SEND_MSG: Resolved from discovered peers: key=${publicKey?.take(8)}")
                }
            }

            // Fallback: Try extracting public key directly from libp2p peer ID
            if (publicKey == null && PeerIdValidator.isLibp2pPeerId(routingPeerId)) {
                val transportIdentity = resolveTransportIdentity(routingPeerId)
                if (transportIdentity != null) {
                    publicKey = transportIdentity.publicKey
                    Timber.d("SEND_MSG: Resolved via transport identity: key=${publicKey?.take(8)}, canonical=${transportIdentity.canonicalPeerId}")
                }

                if (publicKey == null) {
                    try {
                        val extractedKey = ironCore?.extractPublicKeyFromPeerId(routingPeerId)
                        if (!extractedKey.isNullOrEmpty()) {
                            publicKey = normalizePublicKey(extractedKey)
                            Timber.d("SEND_MSG: Extracted public key from peer ID: ${publicKey?.take(8)}")
                        }
                    } catch (e: Exception) {
                        Timber.d("SEND_MSG: Failed to extract public key from peer ID: ${e.message}")
                    }
                }
            }

            // Use public key as canonical peer ID for history storage
            val normalizedPeerId = if (publicKey != null) {
                normalizePublicKey(publicKey) ?: routingPeerId
            } else {
                routingPeerId
            }

            // 1. Save to history IMMEDIATELY.
            val initialRecord = uniffi.api.MessageRecord(
                id = initialMessageId,
                peerId = normalizedPeerId,
                direction = uniffi.api.MessageDirection.SENT,
                content = content,
                timestamp = now,
                senderTimestamp = now,
                delivered = false,
                status = uniffi.api.MessageStatus.QUEUED,
                hidden = false
            )

            var coreEnqueued = false
            try {
                historyManager?.add(initialRecord)
                historyManager?.flush()

                // Emit for UI update
                repoScope.launch {
                    _messageUpdates.emit(initialRecord)
                }

                Timber.d("SEND_MSG_START: original='$peerId', normalized='$normalizedPeerId', routing='$routingPeerId', publicKey='${publicKey?.take(8)}', initialId=$initialMessageId")

                // Check if relay/messaging is enabled (bidirectional control)
                val currentSettings = settingsManager?.load()
                if (!Companion.checkMeshParticipationEnabled(currentSettings)) {
                    return@withContext
                }

            if (publicKey == null) {
                    // 3. Queue with placeholder encrypted data (will re-encrypt when peer discovered)
                    Timber.w("SEND_MSG_QUEUE: Peer not found - will retry when discovered: $normalizedPeerId")

                    logDeliveryState(
                        messageId = initialMessageId,
                        state = "queued",
                        detail = "peer_not_discovered_yet awaiting_public_key"
                    )

                    enqueuePendingOutbound(
                        historyRecordId = initialMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = null,
                        listeners = emptyList(),
                        encryptedData = content.toByteArray(), // Plaintext for now, encrypt on flush
                        initialAttemptCount = 0,
                        initialDelaySec = 5, // Retry in 5 seconds
                        strictBleOnlyMode = false
                    )

                    Timber.i("Message queued for $normalizedPeerId - will send when peer discovered")
                    return@withContext
                }

                val finalPublicKey = publicKey!!
                // Pre-validate public key
                if (finalPublicKey.length != 64 || !finalPublicKey.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }) {
                    Timber.e("Invalid public key format for $normalizedPeerId: $finalPublicKey")
                    return@withContext
                }

                Timber.d("Preparing message for $normalizedPeerId with key: ${finalPublicKey.take(8)}...")

                val routePeerCandidates = buildRoutePeerCandidates(
                    peerId = routingPeerId,
                    cachedRoutePeerId = null,
                    notes = contact?.notes ?: "",
                    recipientPublicKey = finalPublicKey
                )

                // AND-NO-ROUTE-001: Emit "connecting" status when no route candidates yet
                if (routePeerCandidates.isEmpty()) {
                    Timber.i("No route candidates for $normalizedPeerId yet - message will be queued for route discovery")
                }

                if (isKnownRelay(normalizedPeerId) || isBootstrapRelayPeer(normalizedPeerId)) {
                    Timber.w("Refusing to use headless relay identity as a chat recipient: $normalizedPeerId")
                    return@withContext
                }

                val preferredRoutePeerId = routePeerCandidates.firstOrNull()

                // 4. Encrypt/Prepare message
                val outboundContent = encodeMessageWithIdentityHints(content)
                val prepared = ironCore?.prepareMessageWithId(finalPublicKey, outboundContent, uniffi.api.MessageType.TEXT, null)
                    ?: run {
                        Timber.e("Failed to prepare message: IronCore not initialized")
                        return@withContext
                    }
                
                coreEnqueued = true

                val realMessageId = prepared.messageId.trim()
                if (realMessageId.isBlank()) {
                    Timber.e("Failed to prepare message: core returned empty message ID")
                    return@withContext
                }
                val encryptedData = prepared.envelopeData

                // 5. Reconcile message IDs: update the initial history record to use
                // Core's wire message ID. Delivery receipts and markMessageSent are
                // keyed by realMessageId, so our local record must match.
                if (realMessageId != initialMessageId) {
                    try {
                        historyManager?.delete(initialMessageId)
                        val reconciledRecord = uniffi.api.MessageRecord(
                            id = realMessageId,
                            peerId = normalizedPeerId,
                            direction = uniffi.api.MessageDirection.SENT,
                            content = content,
                            timestamp = now,
                            senderTimestamp = now,
                            delivered = false,
                            status = uniffi.api.MessageStatus.QUEUED,
                            hidden = false
                        )
                        historyManager?.add(reconciledRecord)
                        historyManager?.flush()
                        repoScope.launch { _messageUpdates.emit(reconciledRecord) }
                        Timber.d("SEND_MSG: Reconciled message ID: $initialMessageId -> $realMessageId")
                        
                        // AND-MSG-DISAPPEAR-001: Verify message was actually stored
                        val storedMessage = try {
                            historyManager?.get(realMessageId)
                        } catch (e: Exception) {
                            null
                        }
                        if (storedMessage == null) {
                            Timber.e("CRITICAL: Message $realMessageId was not found in history after storage!")
                        } else {
                            Timber.d("SEND_MSG: Verified message stored in history: ${realMessageId.take(8)}...")
                        }
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to reconcile message ID; delivery tracking may be unreliable")
                    }
                }

                logDeliveryState(
                    messageId = realMessageId,
                    state = "pending",
                    detail = "message_prepared_local_history_written"
                )

                // 6. Send over core-selected swarm route only.
                val delivery = attemptDirectSwarmDelivery(
                    routePeerCandidates = routePeerCandidates,
                    listeners = emptyList(),
                    encryptedData = encryptedData,
                    wifiPeerId = preferredRoutePeerId,
                    blePeerId = null,
                    traceMessageId = realMessageId,
                    attemptContext = "initial_send",
                    recipientIdentityId = finalPublicKey,
                    intendedDeviceId = contact?.lastKnownDeviceId
                )

                val selectedRoutePeerId = delivery.routePeerId ?: preferredRoutePeerId

                if (delivery.acked) {
                    // P3: feed the no-downgrade rule in markMessageCorrupted()
                    recordMessageTransportAck(realMessageId)
                    promotePendingOutboundForPeer(peerId = normalizedPeerId, excludingMessageId = realMessageId)
                    // AND-NO-ROUTE-001: Persist last-known-good route in contact notes for future fallback
                    storeLastKnownRoutePeerId(normalizedPeerId, selectedRoutePeerId)
                }

                val receiptAwaitSeconds = RECEIPT_ACK_TIMEOUT_MS / 1000L
                val strictBleOnlyValidation = false

                if (isMessageDeliveredLocally(realMessageId)) {
                    removePendingOutbound(realMessageId)
                    logDeliveryState(
                        messageId = realMessageId,
                        state = "delivered",
                        detail = "delivery_receipt_arrived_before_enqueue"
                    )
                } else if (delivery.terminalFailureCode != null) {
                    enqueuePendingOutbound(
                        historyRecordId = realMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = emptyList(),
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = 0,
                        strictBleOnlyMode = strictBleOnlyValidation,
                        recipientIdentityId = finalPublicKey,
                        intendedDeviceId = contact?.lastKnownDeviceId,
                        terminalFailureCode = delivery.terminalFailureCode
                    )
                    logDeliveryState(
                        messageId = realMessageId,
                        state = "rejected",
                        detail = "terminal_failure_code=${delivery.terminalFailureCode}"
                    )
                    Timber.e(terminalIdentityFailureMessage(delivery.terminalFailureCode))
                    return@withContext
                } else {
                    enqueuePendingOutbound(
                        historyRecordId = realMessageId,
                        peerId = normalizedPeerId,
                        routePeerId = selectedRoutePeerId,
                        listeners = emptyList(),
                        encryptedData = encryptedData,
                        initialAttemptCount = 1,
                        initialDelaySec = if (delivery.acked) receiptAwaitSeconds else 0,
                        strictBleOnlyMode = strictBleOnlyValidation,
                        recipientIdentityId = finalPublicKey,
                        intendedDeviceId = contact?.lastKnownDeviceId
                    )
                }

                Timber.i("Message sent (encrypted) to $normalizedPeerId (id=$realMessageId)")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send message to $normalizedPeerId")
                
                if (coreEnqueued) {
                    Timber.i("Suppressing exception because message was successfully enqueued in core outbox: ${e.message}")
                    return@withContext
                }

                // Even on error, the message is in history (initialRecord),
                // but we should probably ensure it's also in the pending outbox for retry if it's a transient error.
                // However, most exceptions here (invalid key, etc.) are non-transient.
                // Transient transport errors are handled within attemptDirectSwarmDelivery.
                throw e
            }
        }
    }

    open suspend fun dial(multiaddr: String) {
        // No withContext(Dispatchers.IO) needed: SwarmBridge.dial is a suspend
        // FFI call (Issue 5) that awaits the swarm reply without blocking.
        try {
            swarmBridge?.dial(multiaddr)
            Timber.i("Dialed $multiaddr via SwarmBridge")
        } catch (e: Exception) {
            Timber.e(e, "Failed to dial $multiaddr")
            throw e
        }
    }

    open suspend fun dialPeer(multiaddr: String) = dial(multiaddr)

    /**
     * NAT hole-punch Priority 1 (Android): proactively dial the best known seed
     * relay right after the swarm/bridge is wired up, so an outbound NAT mapping
     * exists before any inbound circuit-relay traffic is expected. Non-fatal on
     * failure -- mesh startup must not be blocked by an unreachable relay.
     *
     * v0.4.0: there are no dedicated relays -- every node is a full relay -- and
     * no hardcoded node address. The candidate comes from the ledger, matching
     * the pattern in getPreferredRelay()/testLedgerRelayConnectivity(). A fresh
     * install has an empty ledger and legitimately has no candidate here until
     * it learns a peer via invite/QR or LAN discovery (mDNS/BLE); that is the
     * intended cold-start behavior, not a bug, so we just log and return.
     */
    private suspend fun ensureBootstrapRelayConnected() {
        if (swarmBridge == null) {
            Timber.w("Skipping bootstrap relay connect: SwarmBridge not wired yet")
            return
        }
        val relay = ledgerManager?.getPreferredRelays(1u)?.firstOrNull()
        if (relay == null) {
            Timber.i("No known relay in ledger yet -- skipping proactive NAT dial (fresh install; ledger fills in via invite/QR or LAN discovery)")
            return
        }
        try {
            dial(relay.multiaddr)
            Timber.i("Dialed seed relay from ledger: ${relay.multiaddr}")
        } catch (e: Exception) {
            Timber.w(e, "Seed relay unavailable at startup (non-fatal): ${relay.multiaddr}")
        }
    }

    // Identity Management
    /**
     * P0_ANDROID_IDENTITY_003: Hardened identity detection with redundant backup mechanisms.
     * Uses a multi-tier approach:
     * 1. SharedPreferences backup (fast path)
     * 2. Disk-based sentinel file (fallback for SharedPreferences loss)
     * 3. Rust core identity database check (authoritative)
     */
    fun isIdentityInitialized(): Boolean {
        // P0_SHARED_IDENTITY: Check the published StateFlow first — this is the
        // fastest path and catches identity that was just created in another
        // ViewModel (e.g., onboarding) before cache/backup checks run.
        val published = _identityInfo.value
        if (published != null && published.initialized) {
            return true
        }

        // P0: Check the identity cache (SharedPreferences) — this is populated
        // by cacheIdentityFields() immediately after initializeIdentity() succeeds,
        // before persistIdentityBackup() runs. This eliminates the race where
        // isIdentityInitialized() returns false because the backup hasn't been
        // written yet.
        val cached = readCachedIdentityFields()
        if (cached != null && cached.initialized) {
            return true
        }

        // Check if we have an identity backup first (fast path)
        if (identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)) {
            // P0_ANDROID_010: Verify Rust core also has the identity.
            // The backup may exist in SharedPreferences but the Rust core may have
            // lost its sled database (e.g., after an unclean shutdown). If the service
            // is running and the core reports uninitialized, trigger a restore.
            if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
                val coreInitialized = ironCore?.getIdentityInfo()?.initialized == true
                if (!coreInitialized) {
                    Timber.w("Backup exists but Rust core identity is lost — attempting restore")
                    try {
                        val restored = restoreIdentityFromBackup(ironCore!!)
                        if (restored) {
                            ironCore?.grantConsent()
                            Timber.i("Identity restored from backup during isIdentityInitialized check")
                        }
                    } catch (e: Exception) {
                        Timber.w(e, "Failed to restore identity during isIdentityInitialized check")
                    }
                }
            }
            return true
        }

        // P0_ANDROID_IDENTITY_003: Fallback - check for disk-based sentinel file
        // If SharedPreferences backup is lost (app data clear, migration, corruption),
        // check for the sentinel file as a secondary indicator of prior identity creation.
        val sentinelFile = File(storagePath, IDENTITY_BACKUP_PREFS + ".sentinel")
        if (sentinelFile.exists()) {
            Timber.i("Identity backup sentinel file found, identity exists")
            return true
        }

        // Only start service if we already have it running or explicitly need it
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
            return ironCore?.getIdentityInfo()?.initialized == true
        }

        // If service is not running, check if identity files exist on disk
        val identityFile = File(storagePath, "identity.db")
        return identityFile.exists()
    }

    /**
     * Grant explicit user consent for cryptographic identity generation.
     * Must be called BEFORE initializeIdentity() or createIdentity().
     * Persists consent in the Rust core (sled-backed).
     */
    fun grantConsent() {
        try {
            ensureServiceInitializedFireAndForget()
            ironCore?.grantConsent()
            Timber.i("Consent granted in Rust core")
        } catch (e: Exception) {
            Timber.e(e, "Failed to grant consent")
        }
    }

    fun hasRequiredRuntimePermissions(): Boolean = hasAllPermissions(Permissions.required)

    @Synchronized
    fun onRuntimePermissionsGranted() {
        if (meshService?.getState() != uniffi.api.ServiceState.RUNNING) {
            Timber.d("Permission refresh skipped: mesh service is not running")
            return
        }
        repoScope.launch {
            try {
                initializeAndStartBle()
            } catch (e: Exception) {
                Timber.w(e, "BLE transport failed to start after permission grant")
            }
        }
        repoScope.launch {
            try {
                initializeAndStartWifi()
            } catch (e: Exception) {
                Timber.w(e, "WiFi transport failed to start after permission grant")
            }
        }
        repoScope.launch {
            try {
                initializeAndStartSwarm()
            } catch (e: Exception) {
                Timber.w(e, "Swarm transport failed to refresh after permission grant")
            }
        }
    }

    open suspend fun createIdentity(
        customSalt: ByteArray? = null,
        // P0_ANDROID_PROGRESS_CALLBACK: surface sub-stage progress to the caller
        // so the UI's 6-stage IdentityProgressDisplay actually advances during
        // the ~10 s createIdentity flow instead of sitting on stage 3 for the
        // entire wall (Ed25519 keygen + persistIdentityBackup +
        // initializeAndStartSwarm + updateBleIdentityBeacon all happen inside
        // the single FFI call from the caller's perspective). Defaulted to a
        // no-op so existing call sites (and unit tests that mock the
        // single-arg signature) continue to compile. Second slot is a
        // transient sub-detail string the UI can render under the active
        // stage's detail line; null when not applicable.
        progress: IdentityProgressCallback = { _, _ -> },
    ) {
        // P0_ANDROID_024: Serialize concurrent createIdentity() calls. Log evidence
        // showed 4 threads invoking initializeIdentity() within 12ms; the second
        // through Nth callers would race on sled writes and could produce a half-
        // initialized identity. The mutex also makes the early-return pre-check
        // safe against TOCTOU between "check initialized" and "call initialize".
        identityCreationMutex.withLock {
            kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                try {
                    // P0_ANDROID_PROGRESS_CALLBACK: stage 1 — visible during the
                    // ensureServiceInitialized wait, which is the longest blocking
                    // piece on a cold start and the wall the user perceives as a hang.
                    // Fire it before the wait so the UI recomposes immediately.
                    // The progress callback is now passed through so ensureServiceInitialized
                    // can fire periodic elapsed-time updates during the wait.
                    progress(IdentityCreationEvent.PreparingStorage, "Waking the encrypted key vault…")
                    if (!ensureServiceInitialized(progress) || ironCore == null) {
                        // Throw instead of silent return — the ViewModel catch block
                        // will set _identityError and reset _isCreating, giving the
                        // user visible feedback instead of a frozen progress display.
                        throw IllegalStateException(
                            "Service initialization timed out — identity cannot be created. " +
                            "The encrypted storage is taking longer than expected to start."
                        )
                    }
                    // P0_ANDROID_024 (Bug 3): Skip if identity is already initialized.
                    // Multiple UI surfaces (Onboarding, Settings) and onRuntimePermissionsGranted
                    // can each trigger createIdentity(); without this guard we'd clobber the
                    // existing identity with a fresh one on every entry point.
                    val current = ironCore?.getIdentityInfo()
                    if (current?.initialized == true) {
                        Timber.d("createIdentity: identity already initialized (id=${current.identityId?.take(8)}); skipping")
                        // P0_SHARED_IDENTITY: publish existing identity so the UI
                        // state flips to initialized even on redundant create calls.
                        // Without this, the StateFlow stays null and IdentityScreen
                        // shows the creation form again.
                        cacheIdentityFields(current)
                        publishIdentityInfo(current)
                        progress(IdentityCreationEvent.VerifyingIdentity, "Identity already on disk; verifying…")
                        return@withContext
                    }
                    // P0_ANDROID_010: Grant consent before identity initialization.
                    // The Rust core requires consent_granted=true for initialize_identity().
                    // OnboardingScreen's consent checkbox gates the UI; this propagates consent to the core.
                    progress(IdentityCreationEvent.GeneratingSalt, "Recording your consent in the vault…")
                    ironCore?.grantConsent()
                    // P0_ANDROID_PROGRESS_CALLBACK: stage 3 — Ed25519 keygen is
                    // the longest single step (~3s on Pixel 6a) and the spot
                    // where users most need to see "yes, work is happening."
                    progress(IdentityCreationEvent.GeneratingKeypair, null)
                    Timber.d("Calling ironCore.initializeIdentity()...")
                    ironCore?.initializeIdentity()
                    // P0_SHARED_IDENTITY: publish identity IMMEDIATELY after keygen
                    // succeeds, before backup/swarm steps. If persistIdentityBackup or
                    // initializeAndStartSwarm fails, the UI still knows identity exists
                    // and won't show the redundant creation form.
                    val created = ironCore?.getIdentityInfo()
                    if (created != null && created.initialized) {
                        cacheIdentityFields(created)
                        publishIdentityInfo(created)
                    }
                    // P0_IDENTITY_FAST_BACKUP: Set the latch BEFORE calling
                    // ensureLocalIdentityFederation so it SKIPS the redundant
                    // persistIdentityBackup call. The explicit persist call at
                    // L4818 (with custom salt + progress) is the only one we need.
                    // This eliminates a wasted 30-90s backup encryption that was
                    // immediately overwritten.
                    identityBackupLatch.set(true)
                    ensureLocalIdentityFederation()
                    // P0_ANDROID_PROGRESS_CALLBACK: stage 4 — fingerprint is
                    // computed by the Rust core during initializeIdentity; we
                    // emit ComputingFingerprint here so the user sees the
                    // transition from "Creating your identity key" to
                    // "Deriving your identity fingerprint" between the two
                    // FFI calls. The actual hash work is sub-millisecond.
                    progress(IdentityCreationEvent.ComputingFingerprint, "Hashing your key…")
                    persistIdentityBackup(ironCore, customSalt, progress)
                    Timber.i("Identity created successfully")
                    // Publish again after backup to ensure latest state (nickname, etc.)
                    val finalInfo = ironCore?.getIdentityInfo()
                    if (finalInfo != null && finalInfo.initialized) {
                        cacheIdentityFields(finalInfo)
                    }
                    publishIdentityInfo(finalInfo)
                    // P0_ANDROID_PROGRESS_CALLBACK: stage 6 — final verification
                    // step that starts the libp2p swarm listener and updates
                    // the BLE beacon. The libp2p TCP bind on port 9001 can
                    // take a few hundred ms; showing "Starting swarm
                    // listener…" gives the user motion during that wait.
                    progress(IdentityCreationEvent.VerifyingIdentity, "Starting swarm listener…")
                    // Identity is now available; bring up internet transport if it was deferred.
                    // Fire-and-forget: the swarm startup is slow (~5-10s for TCP bind + mDNS)
                    // but identity is already created and published. Don't block the UI.
                    repoScope.launch {
                        try {
                            initializeAndStartSwarm()
                        } catch (e: Exception) {
                            Timber.w(e, "Swarm startup failed after identity creation (non-blocking)")
                        }
                    }
                    updateBleIdentityBeacon()
                } catch (e: Exception) {
                    Timber.e(e, "Failed to create identity")
                    throw e
                }
            }
        }
    }

    // Helper to ensure service is initialized lazily
    @Volatile
    private var isServiceStarting = false

    /**
     * ANR FIX (P0_ANDROID_017): Defer service initialization to background thread without blocking.
     * The original implementation called loadSettings() synchronously inside the coroutine,
     * which blocked because settingsManager?.load() waits for the Rust core to initialize.
     * This caused a deadlock: UI thread wanted identity info → called ensureServiceInitializedDeferred()
     * → startMeshService() → loadSettings() → Rust core not ready → block.
     *
     * Solution: Use default settings (already cached) for initial config, then reload settings
     * asynchronously after service starts. This allows Settings screen to load immediately.
     */
    private fun ensureServiceInitializedDeferred() {
        repoScope.launch {
            val state = meshService?.getState()
            if (state == uniffi.api.ServiceState.RUNNING) {
                return@launch
            }

            // Skip if already starting (prevents pile-up on @Synchronized startMeshService)
            if (state == uniffi.api.ServiceState.STARTING || isServiceStarting) {
                Timber.d("MeshService is already starting, skipping redundant init")
                return@launch
            }

            Timber.d("Lazy starting MeshService (async settings reload)...")
            isServiceStarting = true
            try {
                // Use default settings for initial config - no blocking I/O
                val defaultSettings = uniffi.api.MeshSettings(
                    relayEnabled = true,
                    maxRelayBudget = 200u,
                    batteryFloor = 20u,
                    bleEnabled = true,
                    wifiAwareEnabled = true,
                    wifiDirectEnabled = true,
                    internetEnabled = true,
                    discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
                    onionRouting = false,
                    coverTrafficEnabled = false,
                    messagePaddingEnabled = false,
                    timingObfuscationEnabled = false,
                    notificationsEnabled = true,
                    notifyDmEnabled = true,
                    notifyDmRequestEnabled = true,
                    notifyDmInForeground = false,
                    notifyDmRequestInForeground = true,
                    soundEnabled = true,
                    badgeEnabled = true,
                    requirePq = false,
                )
                val config = uniffi.api.MeshServiceConfig(
                    discoveryIntervalMs = 30000u,
                    batteryFloorPct = defaultSettings.batteryFloor
                )
                startMeshService(config)
                Timber.d("MeshService started lazily (async settings reload)")

                // Async reload of settings after service started - doesn't block
                repoScope.launch {
                    try {
                        loadSettings()
                        Timber.d("Settings reloaded asynchronously after service startup")
                    } catch (e: Exception) {
                        Timber.w(e, "Async settings reload failed")
                    }
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to start MeshService lazily")
            } finally {
                isServiceStarting = false
            }

            // Refresh ironCore reference just in case
            if (ironCore == null) {
                ironCore = meshService?.getCore()
                Timber.d("IronCore reference refreshed: ${ironCore != null}")
            }
            ensureLocalIdentityFederation()
        }
    }

    private fun ensureServiceInitializedFireAndForget() {
        // Fire-and-forget for non-critical paths (prevents UI thread blocking)
        ensureServiceInitializedDeferred()
    }

    /**
     * Non-blocking variant for suspend functions that require the service to be running.
     * Waits up to 60 seconds for MeshService to reach RUNNING state using delay().
     *
     * Accepts an optional progress callback so the UI can show elapsed time during
     * the potentially long cold-start wait (sled DB init, manager setup, data migration).
     * Fires PreparingStorage periodically with elapsed-seconds info so the user sees
     * motion instead of a frozen "4 seconds remaining" display.
     */
    private suspend fun ensureServiceInitialized(
        progress: IdentityProgressCallback = { _, _ -> }
    ): Boolean {
        // Fast path: already running
        if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
            return true
        }

        // Start initialization if needed
        ensureServiceInitializedDeferred()

        // Wait for service to actually start (with generous timeout for cold starts).
        // Cold-start on low-end devices can take 30-60s due to sled DB init, manager
        // construction, and data migrations. The old 10s timeout caused silent failures
        // where createIdentity() returned without creating identity or notifying the UI.
        val startTime = System.currentTimeMillis()
        val timeoutMs = 60_000L
        var lastProgressMs = 0L
        while (System.currentTimeMillis() - startTime < timeoutMs) {
            if (meshService?.getState() == uniffi.api.ServiceState.RUNNING && ironCore != null) {
                return true
            }
            val elapsed = System.currentTimeMillis() - startTime
            if (elapsed - lastProgressMs >= 2_000L) {
                lastProgressMs = elapsed
                progress(IdentityCreationEvent.PreparingStorage,
                    "Starting secure service… (${elapsed / 1000}s)")
            }
            kotlinx.coroutines.delay(200)
        }
        Timber.e("ensureServiceInitialized timed out after ${timeoutMs}ms")
        return false
    }

    private fun hasAllPermissions(permissions: List<String>): Boolean =
        permissions.all { permission ->
            ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
        }

    // Keep legacy addMessage for receiving side or manual adds
    fun addMessage(record: uniffi.api.MessageRecord) {
        historyManager?.add(record)
        historyManager?.flush()
    }

    fun getMessage(id: String): uniffi.api.MessageRecord? {
        return historyManager?.get(id)
    }

    fun getRecentMessages(peerFilter: String? = null, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        val filter = peerFilter?.let { canonicalId(it) }
        return historyManager?.recent(filter, limit) ?: emptyList()
    }

    fun getConversation(peerId: String, limit: UInt = 100u): List<uniffi.api.MessageRecord> {
        return historyManager?.conversation(canonicalId(peerId), limit) ?: emptyList()
    }

    fun searchMessages(query: String, limit: UInt = 50u): List<uniffi.api.MessageRecord> {
        return historyManager?.search(query, limit) ?: emptyList()
    }

    fun markMessageDelivered(id: String) {
        historyManager?.markDelivered(id)
        removePendingOutbound(id)
    }

    fun clearHistory() {
        historyManager?.clear()
        Timber.i("Message history cleared")
    }

    fun clearConversation(peerId: String) {
        // ID-STANDARDIZATION-001: Use comprehensive ID validation for deletion operations
        try {
            val validatedId = validateAndStandardizeId(peerId, "clearConversation")
            historyManager?.clearConversation(validatedId)
            Timber.i("Conversation cleared: $validatedId")
        } catch (e: Exception) {
            Timber.e(e, "Failed to clear conversation due to ID validation error")
            throw e
        }
    }

    fun getHistoryStats(): uniffi.api.HistoryStats? {
        return historyManager?.stats()
    }

    fun getMessageCount(): UInt {
        return historyManager?.count() ?: 0u
    }

    // ── History Retention ────────────────────────────────────────────────

    /**
     * Enforce message retention by keeping only the newest [maxMessages] messages.
     * Returns the number of messages pruned.
     */
    fun enforceRetention(maxMessages: UInt): UInt {
        return try {
            historyManager?.enforceRetention(maxMessages = maxMessages) ?: 0u
        } catch (e: Exception) {
            Timber.e(e, "Failed to enforce retention")
            0u
        }
    }

    /**
     * Prune messages older than the given Unix timestamp (seconds).
     * Returns the number of messages pruned.
     */
    fun pruneBefore(beforeTimestamp: ULong): UInt {
        return try {
            historyManager?.pruneBefore(beforeTimestamp = beforeTimestamp) ?: 0u
        } catch (e: Exception) {
            Timber.e(e, "Failed to prune history")
            0u
        }
    }

    /**
     * Resets all application data, including identity, contacts, history, and preferences.
     * WARNING: This is destructive and permanent.
     */
    fun resetAllData() {
        Timber.w("RESETTING ALL APPLICATION DATA")

        pendingReceiptSendJobs.values.forEach { it.cancel() }
        pendingReceiptSendJobs.clear()

        // 1. Stop all active services and release UniFFI objects.
        // shutdown() is a suspend FFI call; the reset flow must not proceed to
        // data wiping until the swarm has stopped, so block (bounded await).
        kotlin.runCatching { kotlinx.coroutines.runBlocking { swarmBridge?.shutdown() } }
            .onFailure { Timber.w(it, "Swarm shutdown failed during reset") }
        swarmBridge = null

        meshService?.stop()
        meshService = null

        ironCore?.stop()
        try {
            contactManager?.flush()
            historyManager?.flush()
        } catch (e: Exception) {
            Timber.w("Failed to flush managers during shutdown: ${e.message}")
        }
        ironCore = null

        contactManager = null
        historyManager = null
        ledgerManager = null
        settingsManager = null
        autoAdjustEngine = null

        // 2. Clear identity backup SharedPreferences
        identityBackupPrefs.edit().clear().apply()

        // P0: Clear cached identity fields so reset is reflected on next launch
        identityCachePrefs.edit().clear().apply()

        // 3. Delete all files in storage path (Rust DBs, keys, etc)
        val files = context.filesDir.listFiles()
        files?.forEach { file ->
            if (file.isDirectory) {
                file.deleteRecursively()
            } else {
                file.delete()
            }
        }

        publishIdentityInfo(null)
        Timber.i("All application data reset successfully")
    }

    // ========================================================================
    // LEDGER
    // ========================================================================

    fun recordConnection(multiaddr: String, peerId: String) {
        ledgerManager?.recordConnection(multiaddr, peerId)
    }

    fun recordConnectionFailure(multiaddr: String) {
        ledgerManager?.recordFailure(multiaddr)
    }

    fun getDialableAddresses(): List<uniffi.api.LedgerEntry> {
        return ledgerManager?.dialableAddresses() ?: emptyList()
    }

    /**
     * Bootstrap nodes for the Settings screen, sourced from the local ledger.
     *
     * v0.4.0: there are no dedicated relays and no hardcoded node addresses --
     * discovery is ledger sharing. A freshly installed node has an empty
     * ledger and legitimately returns emptyList() here until it learns a peer
     * via invite/QR or LAN discovery (mDNS/BLE); that is the intended
     * cold-start architecture, not a bug.
     */
    fun getBootstrapNodesForSettings(): List<String> {
        val relays = ledgerManager?.getPreferredRelays(MAX_SETTINGS_RELAYS) ?: emptyList()
        if (relays.isEmpty()) {
            Timber.i("getBootstrapNodesForSettings: ledger has no known relays yet (fresh install or no successful connections)")
        }
        return relays.map { it.multiaddr }
    }

    /**
     * v0.4.0 item 4: bounded startup dial sweep, Android parity with the CLI's
     * startup DialScheduler sweep over dialable_addresses() (cli/src/main.rs).
     * Fires dial() for each proven-good ledger address (capped) right after the
     * swarm starts listening.
     *
     * Concurrency limiting and per-peer backoff are already enforced by the
     * Rust core's DialPolicyManager on every SwarmCommand::Dial (max 3
     * concurrent outbound dials, 3-strike backoff -- core/src/transport/
     * dial_policy.rs), the same policy the CLI's sweep relies on. This loop
     * therefore only needs to bound HOW MANY addresses it offers, not
     * reimplement concurrency/backoff on the Kotlin side.
     *
     * A fresh install has an empty ledger, so this legitimately does nothing
     * until a peer is learned via invite/QR or LAN discovery (mDNS/BLE) --
     * intended cold-start behavior, not a bug.
     */
    private suspend fun performStartupDialSweep() {
        val addresses = (ledgerManager?.dialableAddresses() ?: emptyList())
            .map { it.multiaddr }
            .take(STARTUP_DIAL_SWEEP_MAX_ADDRESSES)

        if (addresses.isEmpty()) {
            Timber.i("Startup dial sweep: no dialable addresses in ledger yet (fresh install or no successful connections)")
            return
        }

        Timber.i("Startup dial sweep: attempting ${addresses.size} known peer(s) from ledger")
        for (addr in addresses) {
            try {
                dial(addr)
            } catch (e: Exception) {
                // Non-fatal: may be a genuine failure, or the core's own
                // per-peer backoff / concurrent-dial-limit rejection.
                Timber.d(e, "Startup dial sweep: $addr not reachable (non-fatal)")
            }
            // Brief pause between dials to avoid overwhelming the core's dial
            // policy queue, matching the CLI sweep's stagger.
            kotlinx.coroutines.delay(200L)
        }
    }

    /**
     * Trigger an actual BLE + WiFi rescan for nearby peer discovery.
     * Restarts BLE scanning and WiFi P2P discovery to find new peers.
     */
    fun triggerTransportRescan() {
        repoScope.launch {
            try {
                // Restart BLE scanning
                transportManager?.let { tm ->
                    tm.stopAll()
                    delay(200)
                    tm.startAll()
                }
                Timber.d("triggerTransportRescan: BLE + WiFi restarted")
            } catch (e: Exception) {
                Timber.w(e, "triggerTransportRescan failed")
            }
        }
    }

    fun replayDiscoveredPeerEvents() {
        repoScope.launch {
            val snapshot = _discoveredPeers.value
            if (snapshot.isEmpty()) return@launch

            val aggregates = linkedMapOf<String, ReplayDiscoveredIdentity>()

            snapshot.forEach { (mapKey, info) ->
                val canonicalPeerId = info.peerId.trim().ifEmpty { mapKey.trim() }
                if (canonicalPeerId.isEmpty()) return@forEach

                val normalizedKey = normalizePublicKey(info.publicKey)
                val aggregateKey = normalizedKey ?: canonicalPeerId
                val routeCandidate = when {
                    PeerIdValidator.isLibp2pPeerId(mapKey) -> mapKey
                    PeerIdValidator.isLibp2pPeerId(canonicalPeerId) -> canonicalPeerId
                    else -> null
                }
                val discoveredNickname = prepopulateDiscoveryNickname(
                    nickname = info.nickname,
                    peerId = canonicalPeerId,
                    publicKey = normalizedKey
                )

                val existing = aggregates[aggregateKey]
                if (existing == null) {
                    aggregates[aggregateKey] = ReplayDiscoveredIdentity(
                        canonicalPeerId = canonicalPeerId,
                        publicKey = normalizedKey,
                        nickname = discoveredNickname,
                        localNickname = info.localNickname,
                        routePeerId = routeCandidate,
                        transport = com.scmessenger.android.service.TransportType.INTERNET
                    )
                } else {
                    if (existing.publicKey.isNullOrBlank() && !normalizedKey.isNullOrBlank()) {
                        existing.publicKey = normalizedKey
                    }
                    existing.canonicalPeerId = selectCanonicalPeerId(canonicalPeerId, existing.canonicalPeerId)
                    existing.nickname = selectAuthoritativeNickname(discoveredNickname, existing.nickname)
                    if (existing.localNickname.isNullOrBlank() && !info.localNickname.isNullOrBlank()) {
                        existing.localNickname = info.localNickname
                    }
                    if (existing.routePeerId.isNullOrBlank() && !routeCandidate.isNullOrBlank()) {
                        existing.routePeerId = routeCandidate
                    }
                    if (existing.transport != com.scmessenger.android.service.TransportType.INTERNET &&
                        info.transport == com.scmessenger.android.service.TransportType.INTERNET
                    ) {
                        existing.transport = info.transport
                    }
                }
            }

            aggregates.values.forEach { peer ->
                val listeners = peer.routePeerId?.let(::getDialHintsForRoutePeer).orEmpty()
                val publicKey = peer.publicKey
                if (!publicKey.isNullOrBlank()) {
                    emitIdentityDiscoveredIfChanged(
                        peerId = peer.canonicalPeerId,
                        publicKey = publicKey,
                        nickname = peer.nickname,
                        libp2pPeerId = peer.routePeerId,
                        listeners = listeners
                    )
                } else {
                    com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
                        com.scmessenger.android.service.PeerEvent.Discovered(
                            peerId = peer.canonicalPeerId,
                            transport = peer.transport
                        )
                    )
                }
            }
        }
    }

    fun getAllKnownTopics(): List<String> {
        return ledgerManager?.allKnownTopics() ?: emptyList()
    }

    fun getLedgerSummary(): String {
        return ledgerManager?.summary() ?: "Ledger not available"
    }

    fun getConnectionPathState(): uniffi.api.ConnectionPathState {
        return try {
            meshService?.getConnectionPathState() ?: uniffi.api.ConnectionPathState.DISCONNECTED
        } catch (e: Exception) {
            Timber.w(e, "Failed to read connection path state")
            uniffi.api.ConnectionPathState.DISCONNECTED
        }
    }

    fun getNatStatus(): String {
        return try {
            meshService?.getNatStatus() ?: "unknown"
        } catch (e: Exception) {
            Timber.w(e, "Failed to read NAT status")
            "unknown"
        }
    }

    fun getServiceStateName(): String {
        return meshService?.getState()?.name ?: "STOPPED"
    }

    fun getDiscoveredPeerCount(): Int {
        return _discoveredPeers.value.size
    }

    fun getPendingOutboxCount(): Int {
        return loadPendingOutbox().size
    }

    fun getPendingDeliverySnapshot(messageId: String): Pair<Int, Long>? {
        if (messageId.isBlank()) return null
        val pending = loadPendingOutbox().firstOrNull { it.historyRecordId == messageId } ?: return null
        return pending.attemptCount to pending.nextAttemptAtEpochSec
    }

    fun getPendingTerminalFailureCode(messageId: String): String? {
        if (messageId.isBlank()) return null
        return loadPendingOutbox()
            .firstOrNull { it.historyRecordId == messageId }
            ?.terminalFailureCode
    }

    fun getMissingRuntimePermissions(): List<String> {
        return Permissions.required.filter { permission ->
            ContextCompat.checkSelfPermission(context, permission) != PackageManager.PERMISSION_GRANTED
        }
    }

    /**
     * ANR FIX (P0_ANDROID_017): Export diagnostics asynchronously to avoid Main thread I/O.
     * The original implementation blocked on:
     * 1. meshService?.exportDiagnostics() - Rust FFI call that can hang
     * 2. loadPendingOutbox() - File I/O operation
     *
     * Solution: Run diagnostics export on IO dispatcher and cache results.
     */
    suspend fun exportDiagnosticsAsync(): String = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
        try {
            exportDiagnosticsInternal()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export diagnostics")
            // Return minimal fallback JSON to avoid crash
            org.json.JSONObject()
                .put("error", "Diagnostics export failed: ${e.message}")
                .put("service_state", meshService?.getState()?.name ?: "UNKNOWN")
                .put("generated_at_ms", System.currentTimeMillis())
                .toString()
        }
    }

    /**
     * Original diagnostics export - now only called from IO dispatcher.
     */
    fun exportDiagnostics(): String = exportDiagnosticsInternal()

    private fun exportDiagnosticsInternal(): String {
        val coreDiagnostics = try {
            meshService?.exportDiagnostics()
        } catch (e: Exception) {
            Timber.w(e, "Failed to export core diagnostics")
            null
        }

        val bleDiscovery = bleScanner?.getDiscoveryStats()
        val bleClient = bleGattClient?.getClientStats()

        if (!coreDiagnostics.isNullOrBlank()) {
            try {
                val obj = org.json.JSONObject(coreDiagnostics)
                bleDiscovery?.let { stats ->
                    obj.put("ble_advertisements_seen", stats.advertisementsSeen)
                    obj.put("ble_peers_discovered", stats.peersDiscovered)
                    obj.put("ble_scan_failures", stats.scanFailures)
                    obj.put("ble_peer_cache_size", stats.peerCacheSize)
                }
                bleClient?.let { stats ->
                    obj.put("ble_gatt_connect_attempts", stats.connectAttempts)
                    obj.put("ble_gatt_connect_initiated", stats.connectInitiated)
                    obj.put("ble_gatt_connect_failures", stats.connectFailures)
                    obj.put("ble_gatt_connect_state_successes", stats.connectStateSuccesses)
                    obj.put("ble_gatt_disconnects", stats.disconnects)
                    obj.put("ble_duplicate_permit_releases_ignored", stats.duplicatePermitReleasesIgnored)
                    obj.put("ble_semaphore_release_overflows", stats.semaphoreReleaseOverflows)
                    obj.put("ble_no_response_callbacks_ignored", stats.noResponseCallbacksIgnored)
                    obj.put("ble_address_type_mismatch_signals", stats.addressTypeMismatchSignals)
                    obj.put("ble_active_connections", stats.activeConnections)
                }
                obj.put("strict_ble_only_validation", strictBleOnlyValidation)
                return obj.toString()
            } catch (_: Exception) {
                return coreDiagnostics
            }
        }

        // ANR FIX: Use async pending outbox loader
        val pendingOutboxSize = try {
            loadPendingOutbox().size
        } catch (e: Exception) {
            Timber.w(e, "Failed to load pending outbox for diagnostics")
            0
        }

        val fallback = org.json.JSONObject()
            .put("service_state", meshService?.getState()?.name ?: "STOPPED")
            .put("connection_path_state", getConnectionPathState().name)
            .put("nat_status", getNatStatus())
            .put("discovered_peers", _discoveredPeers.value.size)
            .put("pending_outbox", pendingOutboxSize)
            .put("strict_ble_only_validation", strictBleOnlyValidation)
            .put("generated_at_ms", System.currentTimeMillis())
        bleDiscovery?.let { stats ->
            fallback.put("ble_advertisements_seen", stats.advertisementsSeen)
            fallback.put("ble_peers_discovered", stats.peersDiscovered)
            fallback.put("ble_scan_failures", stats.scanFailures)
            fallback.put("ble_peer_cache_size", stats.peerCacheSize)
        }
        bleClient?.let { stats ->
            fallback.put("ble_gatt_connect_attempts", stats.connectAttempts)
            fallback.put("ble_gatt_connect_initiated", stats.connectInitiated)
            fallback.put("ble_gatt_connect_failures", stats.connectFailures)
            fallback.put("ble_gatt_connect_state_successes", stats.connectStateSuccesses)
            fallback.put("ble_gatt_disconnects", stats.disconnects)
            fallback.put("ble_duplicate_permit_releases_ignored", stats.duplicatePermitReleasesIgnored)
            fallback.put("ble_semaphore_release_overflows", stats.semaphoreReleaseOverflows)
            fallback.put("ble_no_response_callbacks_ignored", stats.noResponseCallbacksIgnored)
            fallback.put("ble_address_type_mismatch_signals", stats.addressTypeMismatchSignals)
            fallback.put("ble_active_connections", stats.activeConnections)
        }
        return fallback.toString()
    }

    fun saveLedger() {
        ledgerManager?.save()
    }

    private val identityEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Pair<IdentityEmissionSignature, Long>>()
    private val identityReemitIntervalMs = 15_000L
    private val connectedEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Long>()
    private val connectedReemitIntervalMs = 15_000L
    private val disconnectEmissionCache = java.util.concurrent.ConcurrentHashMap<String, Long>()

    private suspend fun emitIdentityDiscoveredIfChanged(
        peerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        listeners: List<String>,
        blePeerId: String? = null
    ) {
        val canonicalPeerId = PeerIdValidator.normalize(peerId)
        val normalizedKey = normalizePublicKey(publicKey)
        if (canonicalPeerId.isEmpty() || normalizedKey.isNullOrBlank()) {
            return
        }

        val normalizedRoute = libp2pPeerId?.let { PeerIdValidator.normalize(it) }?.takeIf { it.isNotEmpty() }
        val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
        val normalizedNickname = normalizeNickname(nickname)
        val normalizedListeners = listeners
            .asSequence()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .distinct()
            .sorted()
            .toList()
        val signature = IdentityEmissionSignature(
            canonicalPeerId = canonicalPeerId,
            publicKey = normalizedKey,
            nickname = normalizedNickname,
            libp2pPeerId = normalizedRoute,
            blePeerId = normalizedBle
        )
        val cacheKey = "$canonicalPeerId|$normalizedKey"
        val now = System.currentTimeMillis()
        val previous = identityEmissionCache[cacheKey]
        if (previous != null && previous.first == signature && (now - previous.second) < identityReemitIntervalMs) {
            return
        }
        identityEmissionCache[cacheKey] = signature to now

        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.IdentityDiscovered(
                peerId = canonicalPeerId,
                publicKey = normalizedKey,
                nickname = normalizedNickname,
                libp2pPeerId = normalizedRoute,
                listeners = normalizedListeners,
                blePeerId = normalizedBle
            )
        )
    }

    private suspend fun emitConnectedIfChanged(
        peerId: String,
        transport: com.scmessenger.android.service.TransportType
    ) {
        val normalizedPeerId = PeerIdValidator.normalize(peerId)
        if (normalizedPeerId.isEmpty()) return

        val now = System.currentTimeMillis()
        val lastEmitted = connectedEmissionCache[normalizedPeerId]
        if (lastEmitted != null && (now - lastEmitted) < connectedReemitIntervalMs) {
            return
        }
        connectedEmissionCache[normalizedPeerId] = now
        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.Connected(
                normalizedPeerId,
                transport
            )
        )
    }
    private suspend fun emitDisconnectedIfChanged(
        peerId: String
    ) {
        val normalizedPeerId = PeerIdValidator.normalize(peerId)
        if (normalizedPeerId.isEmpty()) return

        val now = System.currentTimeMillis()
        val lastEmitted = disconnectEmissionCache[normalizedPeerId]
        if (lastEmitted != null && (now - lastEmitted) < peerDisconnectDedupIntervalMs) {
            return
        }
        disconnectEmissionCache[normalizedPeerId] = now
        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            com.scmessenger.android.service.PeerEvent.Disconnected(
                normalizedPeerId
            )
        )
    }

    // ========================================================================
    // SETTINGS
    // ========================================================================

    fun loadSettings(): uniffi.api.MeshSettings {
        val loaded = try {
            settingsManager?.load()
        } catch (e: Exception) {
            Timber.w("Settings load failed (likely first run), using defaults: ${e.message}")
            null
        }

        val settings = loaded ?: getDefaultSettings()

        // Auto-upgrade WiFi settings for existing identities
        // (P0_TRANSPORT_002: Enable WiFi Aware/Direct discovery on all installations)
        if (!settings.wifiAwareEnabled || !settings.wifiDirectEnabled) {
            val upgraded = settings.copy(
                wifiAwareEnabled = true,
                wifiDirectEnabled = true
            )
            try {
                settingsManager?.save(upgraded)
                Timber.i("Auto-upgraded settings: WiFi Aware/Direct now enabled")
            } catch (e: Exception) {
                Timber.w("Failed to save upgraded settings: ${e.message}")
            }
            return upgraded
        }

        return settings
    }

    /**
     * Get the default mesh settings from the Rust core.
     */
    fun getDefaultSettings(): uniffi.api.MeshSettings {
        return settingsManager?.defaultSettings()
            ?: uniffi.api.MeshSettings(
                relayEnabled = true,
                maxRelayBudget = 200u,
                batteryFloor = 20u,
                bleEnabled = true,
                wifiAwareEnabled = true,
                wifiDirectEnabled = true,
                internetEnabled = true,
                discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
                onionRouting = false,
                coverTrafficEnabled = false,
                messagePaddingEnabled = false,
                timingObfuscationEnabled = false,
                notificationsEnabled = true,
                notifyDmEnabled = true,
                notifyDmRequestEnabled = true,
                notifyDmInForeground = false,
                notifyDmRequestInForeground = true,
                soundEnabled = true,
                badgeEnabled = true,
                requirePq = false,
            )
    }

    fun saveSettings(settings: uniffi.api.MeshSettings) {
        settingsManager?.save(settings)
        Timber.i("Settings saved")
    }

    /**
     * Apply transport settings by enabling/disabling transports based on settings.
     * This is called when settings change to dynamically update transport state.
     */
    fun applyTransportSettings(settings: uniffi.api.MeshSettings) {
        val currentSettings = loadSettings()

        // Apply BLE settings
        if (settings.bleEnabled != currentSettings.bleEnabled) {
            if (settings.bleEnabled) {
                transportManager?.enableTransport(TransportType.BLE)
                Timber.d("BLE transport enabled via settings")
            } else {
                transportManager?.disableTransport(TransportType.BLE)
                Timber.d("BLE transport disabled via settings")
            }
        }

        // Apply WiFi Aware settings
        if (settings.wifiAwareEnabled != currentSettings.wifiAwareEnabled) {
            if (settings.wifiAwareEnabled) {
                transportManager?.enableTransport(TransportType.WIFI_AWARE)
                Timber.d("WiFi Aware transport enabled via settings")
            } else {
                transportManager?.disableTransport(TransportType.WIFI_AWARE)
                Timber.d("WiFi Aware transport disabled via settings")
            }
        }

        // Apply WiFi Direct settings
        if (settings.wifiDirectEnabled != currentSettings.wifiDirectEnabled) {
            if (settings.wifiDirectEnabled) {
                transportManager?.enableTransport(TransportType.WIFI_DIRECT)
                Timber.d("WiFi Direct transport enabled via settings")
            } else {
                transportManager?.disableTransport(TransportType.WIFI_DIRECT)
                Timber.d("WiFi Direct transport disabled via settings")
            }
        }

        // Apply Internet settings (Swarm + TCP/mDNS)
        if (settings.internetEnabled != currentSettings.internetEnabled) {
            if (settings.internetEnabled) {
                // Internet transport is handled by SwarmBridge, started via initializeAndStartSwarm
                transportManager?.enableTransport(TransportType.TCP_MDNS)
                Timber.d("Internet transport (and mDNS LAN) enabled via settings")
            } else {
                // Internet transport is handled by SwarmBridge, shutdown via stopMeshService
                transportManager?.disableTransport(TransportType.TCP_MDNS)
                Timber.d("Internet transport (and mDNS LAN) disabled via settings")
            }
        }
    }

    fun validateSettings(settings: uniffi.api.MeshSettings): Boolean {
        return try {
            settingsManager?.validate(settings)
            true
        } catch (e: Exception) {
            Timber.w(e, "Settings validation failed")
            false
        }
    }

    // ========================================================================
    // AUTO-ADJUST ENGINE
    // ========================================================================

    fun computeAdjustmentProfile(deviceProfile: uniffi.api.DeviceProfile): uniffi.api.AdjustmentProfile {
        return autoAdjustEngine?.computeProfile(deviceProfile)
            ?: uniffi.api.AdjustmentProfile.STANDARD
    }

    fun computeBleAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.BleAdjustment {
        return autoAdjustEngine?.computeBleAdjustment(profile)
            ?: uniffi.api.BleAdjustment(
                scanIntervalMs = 30000u,
                advertiseIntervalMs = 30000u,
                txPowerDbm = -4
            )
    }

    fun computeRelayAdjustment(profile: uniffi.api.AdjustmentProfile): uniffi.api.RelayAdjustment {
        return autoAdjustEngine?.computeRelayAdjustment(profile)
            ?: uniffi.api.RelayAdjustment(
                maxPerHour = 200u,
                priorityThreshold = 100u,
                maxPayloadBytes = 16384u
            )
    }

    fun overrideBleInterval(intervalMs: UInt) {
        autoAdjustEngine?.overrideBleScanInterval(intervalMs)
    }

    fun setRelayBudget(messagesPerHour: UInt) {
        // Suspend FFI call (Issue 5); fire-and-forget so sync callers (e.g. the
        // battery-adjustment path in AndroidPlatformBridge) never block on it.
        repoScope.launch {
            try {
                meshService?.setRelayBudget(messagesPerHour)
            } catch (e: Exception) {
                Timber.w("setRelayBudget failed: ${e.message}")
            }
        }
    }

    fun updateDeviceState(profile: uniffi.api.DeviceProfile) {
        meshService?.updateDeviceState(profile)
    }

    fun overrideRelayMax(max: UInt) {
        autoAdjustEngine?.overrideRelayMaxPerHour(max)
    }

    fun clearAdjustmentOverrides() {
        autoAdjustEngine?.clearOverrides()
    }

    /**
     * Connect to a peer using provided addresses.
     */
    // ========================================================================
    // SWARM BRIDGE DELEGATIONS
    // ========================================================================

    /**
     * Get list of currently subscribed gossipsub topics from SwarmBridge.
     *
     * Bounded blocking bridge query: the FFI is a suspend fun (Issue 5), but
     * legacy sync callers (TopicManager.refreshKnownTopics) need a value in
     * place. The await is a fast in-process command-channel roundtrip.
     */
    fun getTopics(): List<String> = try {
        kotlinx.coroutines.runBlocking { swarmBridge?.getTopics() ?: emptyList() }
    } catch (e: Exception) {
        Timber.w("getTopics failed: ${e.message}")
        emptyList()
    }

    /**
     * Subscribe to a gossipsub topic via SwarmBridge (fire-and-forget).
     */
    fun subscribeTopic(topic: String) {
        repoScope.launch {
            try {
                swarmBridge?.subscribeTopic(topic)
            } catch (e: Exception) {
                Timber.w("subscribeTopic failed for $topic: ${e.message}")
            }
        }
    }

    fun unsubscribeTopic(topic: String) {
        repoScope.launch {
            try {
                swarmBridge?.unsubscribeTopic(topic)
            } catch (e: Exception) {
                Timber.w("unsubscribeTopic failed for $topic: ${e.message}")
            }
        }
    }

    fun publishTopic(topic: String, data: ByteArray) {
        repoScope.launch {
            try {
                swarmBridge?.publishTopic(topic, data)
            } catch (e: Exception) {
                Timber.w("publishTopic failed for $topic: ${e.message}")
            }
        }
    }

    /**
     * Broadcast data to all connected peers via SwarmBridge (fire-and-forget).
     */
    fun sendToAllPeers(data: ByteArray) {
        repoScope.launch {
            try {
                swarmBridge?.sendToAllPeers(data)
            } catch (e: Exception) {
                Timber.w("sendToAllPeers failed: ${e.message}")
            }
        }
    }

    fun connectToPeer(peerId: String, addresses: List<String>) {
        val dialCandidates = buildDialCandidatesForPeer(
            routePeerId = peerId,
            rawAddresses = addresses,
            includeRelayCircuits = false
        )
        // dial() is a suspend FFI call (Issue 5); issue the dials on repoScope
        // so sync callers (ViewModels, delivery paths) never block. Callers
        // that need the connection use awaitPeerConnection(), which polls and
        // tolerates in-flight dials.
        repoScope.launch {
            dialCandidates.forEach { addr ->
                try {
                    // Only append /p2p/ component if peerId is a valid libp2p PeerId format
                    // (base58btc multihash, starts with "12D3Koo" or "Qm").
                    // Blake3 hex identity_ids (64 hex chars) are NOT valid libp2p PeerIds.
                    val finalAddr = if (PeerIdValidator.isLibp2pPeerId(peerId) && !addr.contains("/p2p/")) {
                        "$addr/p2p/$peerId"
                    } else {
                        addr
                    }
                    if (shouldAttemptDial(finalAddr)) {
                        swarmBridge?.dial(finalAddr)
                        Timber.d("Dialing $finalAddr")
                    }
                } catch (e: Exception) {
                    Timber.e(e, "Failed to dial $addr")
                }
            }
        }
    }

    private fun ensurePendingOutboxRetryLoop() {
        if (pendingOutboxRetryJob?.isActive == true) return

        pendingOutboxRetryJob = repoScope.launch {
            while (true) {
                try {
                    // Proactively ensure we stay connected to relays
                    primeRelayBootstrapConnections()

                    flushPendingOutbox("periodic")
                    kotlinx.coroutines.delay(8000)
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (e: Exception) {
                    Timber.w(e, "Pending outbox retry loop error")
                    kotlinx.coroutines.delay(5000)
                }
            }
        }
    }

    /** Periodically broadcasts cover traffic when `coverTrafficEnabled` is true in settings. */
    private fun ensureCoverTrafficLoop() {
        if (coverTrafficJob?.isActive == true) return

        coverTrafficJob = repoScope.launch {
            while (true) {
                try {
                    kotlinx.coroutines.delay(30_000)
                    val enabled = try { settingsManager?.load()?.coverTrafficEnabled == true } catch (_: Exception) { false }
                    if (enabled) {
                        val core = ironCore
                        val bridge = swarmBridge
                        if (core != null && bridge != null) {
                            try {
                                val payload = core.prepareCoverTraffic(256u)
                                bridge.sendToAllPeers(payload)
                            } catch (e: Exception) {
                                Timber.d("Cover traffic send skipped: ${e.message}")
                            }
                        }
                    }
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (e: Exception) {
                    Timber.w(e, "Cover traffic loop error")
                    kotlinx.coroutines.delay(30_000)
                }
            }
        }
    }

    private suspend fun attemptDirectSwarmDelivery(
        routePeerCandidates: List<String>,
        listeners: List<String>,
        encryptedData: ByteArray,
        wifiPeerId: String? = null,
        blePeerId: String? = null,
        traceMessageId: String? = null,
        attemptContext: String = "send",
        strictBleOnlyOverride: Boolean? = null,
        recipientIdentityId: String? = null,
        intendedDeviceId: String? = null
    ): DeliveryAttemptResult {
        // AND-DELIVERY-001: Resolve nullable traceMessageId to a non-null value early,
        // so all internal logDeliveryAttempt calls use a real message ID and never emit msg=unknown.
        val traceMessageId = traceMessageId?.takeIf { it.isNotBlank() }
            ?: "unassigned_${System.currentTimeMillis()}_${attemptContext}"
        val strictBleOnly = strictBleOnlyOverride ?: strictBleOnlyValidation
        val routePeerFallback = routePeerCandidates.firstOrNull() ?: "unknown_route_${System.currentTimeMillis()}"
        if (strictBleOnly) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble-only",
                phase = "mode",
                outcome = "enabled",
                detail = "ctx=$attemptContext route_candidates=${routePeerCandidates.size}"
            )
            if (!wifiPeerId.isNullOrBlank()) {
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "wifi-direct",
                    phase = "ble_only",
                    outcome = "blocked",
                    detail = "ctx=$attemptContext reason=strict_ble_only_mode"
                )
            }
            if (routePeerCandidates.isNotEmpty() || listeners.isNotEmpty()) {
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "ble_only",
                    outcome = "blocked",
                    detail = "ctx=$attemptContext reason=strict_ble_only_mode"
                )
            }
        }

        val connectedBleDevices = kotlin.runCatching { bleGattServer?.getConnectedDeviceAddresses().orEmpty() }
            .getOrDefault(emptyList())
            .mapNotNull { it.trim().takeIf { value -> value.isNotEmpty() } }
            .distinct()
        val requestedBlePeerId = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
        val freshBleObservation = resolveFreshBlePeerId(
            buildList {
                addAll(routePeerCandidates)
                requestedBlePeerId?.let(::add)
                recipientIdentityId?.trim()?.takeIf { it.isNotEmpty() }?.let(::add)
                intendedDeviceId?.trim()?.takeIf { it.isNotEmpty() }?.let(::add)
            }
        )
        val effectiveBlePeerId = connectedBleDevices.firstOrNull()
            ?: freshBleObservation?.address
            ?: requestedBlePeerId?.takeIf {
                resolveFreshBlePeerId(listOf(it))?.address == it
            }

        if (requestedBlePeerId == null && effectiveBlePeerId != null) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId reason=ble_peer_missing_connected_device_available"
            )
        } else if (
            freshBleObservation != null &&
            effectiveBlePeerId != null &&
            requestedBlePeerId != null &&
            effectiveBlePeerId != requestedBlePeerId
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId requested_target=$requestedBlePeerId reason=fresh_ble_observation source=${freshBleObservation.source}"
            )
        } else if (
            requestedBlePeerId != null &&
            effectiveBlePeerId != null &&
            requestedBlePeerId != effectiveBlePeerId
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "target_fallback",
                detail = "ctx=$attemptContext target=$effectiveBlePeerId requested_target=$requestedBlePeerId reason=prefer_connected_device"
            )
        } else if (
            requestedBlePeerId != null &&
            effectiveBlePeerId == null &&
            connectedBleDevices.isEmpty()
        ) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble",
                phase = "local_fallback",
                outcome = "skipped",
                detail = "ctx=$attemptContext requested_target=$requestedBlePeerId reason=stale_ble_hint_no_fresh_observation"
            )
        }

        // Use SmartTransportRouter for intelligent transport selection with 500ms timeout fallback
        val smartResult = if (smartTransportRouter != null) {
            smartTransportRouter!!.attemptDelivery(
                peerId = routePeerFallback,
                envelopeData = encryptedData,
                wifiPeerId = if (strictBleOnly) null else wifiPeerId,
                blePeerId = effectiveBlePeerId,
                tcpMdnsPeerId = routePeerCandidates
                    .firstOrNull { candidate ->
                        val trimmed = candidate.trim()
                        mdnsLanPeers[trimmed]?.isNotEmpty() == true
                    }
                    ?.trim(),
                routePeerCandidates = routePeerCandidates,
                listeners = listeners,
                traceMessageId = traceMessageId,
                attemptContext = attemptContext,
                tryWifi = { wifiId ->
                    val wifi = wifiTransportManager ?: run {
                        Timber.d("tryWifiDelivery: wifiTransportManager is null, returning false")
                        return@attemptDelivery false
                    }
                    try {
                        if (wifi.sendData(wifiId, encryptedData)) {
                            Timber.i("[OK] Delivery via WiFi Direct (target=$wifiId)")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "smart_router",
                                outcome = "success",
                                detail = "ctx=$attemptContext target=$wifiId"
                            )
                            true
                        } else {
                            Timber.d("tryWifiDelivery: wifiTransportManager.sendData returned false for $wifiId")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "smart_router",
                                outcome = "failed",
                                detail = "ctx=$attemptContext target=$wifiId reason=sendData_false"
                            )
                            false
                        }
                    } catch (wifiEx: Exception) {
                        Timber.w(wifiEx, "WiFi send failed for $wifiId")
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "wifi-direct",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext target=$wifiId reason=${wifiEx.message ?: "exception"}"
                        )
                        false
                    }
                },
                tryBle = { bleAddr ->
                    val bleClient = bleGattClient
                    val bleServer = bleGattServer
                    if (bleClient == null && bleServer == null) {
                        Timber.d("tryBleDelivery: both bleGattClient and bleGattServer are null, returning false")
                        return@attemptDelivery false
                    }

                    // Fast-skip: if no BLE devices are connected and the requested
                    // address is a cached hint (not actively connected), skip entirely
                    // to avoid wasting time on stale MACs.
                    if (bleAddr.isNullOrBlank() && connectedBleDevices.isEmpty()) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "ble",
                            phase = "smart_router",
                            outcome = "skipped",
                            detail = "ctx=$attemptContext no_hint_and_no_connected_devices"
                        )
                        return@attemptDelivery false
                    }

                    val sendTargets = linkedSetOf(bleAddr).apply { addAll(connectedBleDevices) }

                    for (target in sendTargets) {
                        if (bleClient != null) {
                            try {
                                if (bleClient.sendData(target, encryptedData)) {
                                    Timber.i("[OK] Delivery via BLE client (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "smart_router",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=central requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptDelivery true
                                }
                            } catch (bleClientEx: Exception) {
                                Timber.w(bleClientEx, "BLE client send failed for $target")
                            }
                        }

                        if (bleServer != null) {
                            try {
                                if (bleServer.sendData(target, encryptedData)) {
                                    Timber.i("[OK] Delivery via BLE server notify (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "smart_router",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=peripheral requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptDelivery true
                                }
                            } catch (bleServerEx: Exception) {
                                Timber.w(bleServerEx, "BLE server send failed for $target")
                            }
                        }
                    }
                    false
                },
                tryTcpMdns = { lanPeerId ->
                    // TCP/mDNS transport: Direct LAN delivery via libp2p TCP.
                    // This peer was discovered via mDNS and has LAN addresses — skip relay.
                    val bridge = swarmBridge ?: run {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext reason=swarm_bridge_unavailable"
                        )
                        return@attemptDelivery false
                    }

                    // Dial LAN addresses directly (no relay circuits)
                    val lanAddrs = mdnsLanPeers[lanPeerId] ?: emptyList()
                    if (lanAddrs.isNotEmpty()) {
                        val dialCandidates = buildDialCandidatesForPeer(
                            routePeerId = lanPeerId,
                            rawAddresses = lanAddrs,
                            includeRelayCircuits = false
                        )
                        if (dialCandidates.isNotEmpty()) {
                            connectToPeer(lanPeerId, dialCandidates)
                            awaitPeerConnection(lanPeerId, timeoutMs = 1000L)
                        }
                    }

                    val directError = bridge.sendMessageStatus(
                        lanPeerId,
                        encryptedData,
                        recipientIdentityId,
                        intendedDeviceId
                    )

                    if (directError == null) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "success",
                            detail = "ctx=$attemptContext route=$lanPeerId lan_addrs=${lanAddrs.size}"
                        )
                        true
                    } else {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "tcp_mdns",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext route=$lanPeerId reason=$directError"
                        )
                        false
                    }
                },
                tryCore = { corePeerId ->
                    // Core transport attempt (libp2p/internet relay)
                    val bridge = swarmBridge ?: run {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext reason=swarm_bridge_unavailable"
                        )
                        return@attemptDelivery false
                    }
                    
                    val liveRouteHints = getDialHintsForRoutePeer(corePeerId)
                    val dialCandidates = buildDialCandidatesForPeer(
                        routePeerId = corePeerId,
                        rawAddresses = listeners + liveRouteHints,
                        includeRelayCircuits = true
                    )
                    if (dialCandidates.isNotEmpty()) {
                        connectToPeer(corePeerId, dialCandidates)
                        val connected = awaitPeerConnection(corePeerId, timeoutMs = 2000L)
                        Timber.d("[ROUTE] Transport: route=$corePeerId connected=$connected timeout=2000ms")
                    }
                    
                    val directError = bridge.sendMessageStatus(
                        corePeerId,
                        encryptedData,
                        recipientIdentityId,
                        intendedDeviceId
                    )
                    
                    if (directError == null) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "success",
                            detail = "ctx=$attemptContext route=$corePeerId"
                        )
                        true
                    } else {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "core",
                            phase = "smart_router",
                            outcome = "failed",
                            detail = "ctx=$attemptContext route=$corePeerId reason=$directError"
                        )
                        false
                    }
                }
            )
        } else {
            // Fallback to legacy LocalTransportFallback if router not available
            val localFallback = Companion.attemptWifiThenBleFallback(
                wifiPeerId = if (strictBleOnly) null else wifiPeerId,
                blePeerId = effectiveBlePeerId,
                tryWifi = { wifiId ->
                    val wifi = wifiTransportManager ?: run {
                        Timber.d("tryWifiDelivery: wifiTransportManager is null, returning false")
                        return@attemptWifiThenBleFallback false
                    }
                    try {
                        if (wifi.sendData(wifiId, encryptedData)) {
                            Timber.i("[OK] Delivery via WiFi Direct (target=$wifiId)")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "local_fallback",
                                outcome = "success",
                                detail = "ctx=$attemptContext target=$wifiId"
                            )
                            true
                        } else {
                            Timber.d("tryWifiDelivery: wifiTransportManager.sendData returned false for $wifiId")
                            logDeliveryAttempt(
                                messageId = traceMessageId,
                                medium = "wifi-direct",
                                phase = "local_fallback",
                                outcome = "failed",
                                detail = "ctx=$attemptContext target=$wifiId reason=sendData_false"
                            )
                            false
                        }
                    } catch (wifiEx: Exception) {
                        Timber.w(wifiEx, "WiFi send failed for $wifiId")
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "wifi-direct",
                            phase = "local_fallback",
                            outcome = "failed",
                            detail = "ctx=$attemptContext target=$wifiId reason=${wifiEx.message ?: "exception"}"
                        )
                        false
                    }
                },
                tryBle = { bleAddr ->
                    val bleClient = bleGattClient
                    val bleServer = bleGattServer
                    if (bleClient == null && bleServer == null) {
                        Timber.d("tryBleDelivery: both bleGattClient and bleGattServer are null, returning false")
                        return@attemptWifiThenBleFallback false
                    }

                    // Fast-skip: if no BLE devices are connected and the requested
                    // address is a cached hint (not actively connected), skip entirely
                    // to avoid wasting time on stale MACs.
                    if (bleAddr.isNullOrBlank() && connectedBleDevices.isEmpty()) {
                        logDeliveryAttempt(
                            messageId = traceMessageId,
                            medium = "ble",
                            phase = "local_fallback",
                            outcome = "skipped",
                            detail = "ctx=$attemptContext no_hint_and_no_connected_devices"
                        )
                        return@attemptWifiThenBleFallback false
                    }

                    val sendTargets = linkedSetOf(bleAddr).apply { addAll(connectedBleDevices) }

                    for (target in sendTargets) {
                        if (bleClient != null) {
                            try {
                                if (bleClient.sendData(target, encryptedData)) {
                                    Timber.i("[OK] Delivery via BLE client (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "local_fallback",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=central requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptWifiThenBleFallback true
                                }
                            } catch (bleClientEx: Exception) {
                                Timber.w(bleClientEx, "BLE client send failed for $target")
                            }
                        }

                        if (bleServer != null) {
                            try {
                                if (bleServer.sendData(target, encryptedData)) {
                                    Timber.i("[OK] Delivery via BLE server notify (target=$target)")
                                    logDeliveryAttempt(
                                        messageId = traceMessageId,
                                        medium = "ble",
                                        phase = "local_fallback",
                                        outcome = "accepted",
                                        detail = "ctx=$attemptContext role=peripheral requested_target=$bleAddr target=$target"
                                    )
                                    return@attemptWifiThenBleFallback true
                                }
                            } catch (bleServerEx: Exception) {
                                Timber.w(bleServerEx, "BLE server send failed for $target")
                            }
                        }
                    }
                    false
                }
            )
            com.scmessenger.android.transport.SmartTransportRouter.TransportDeliveryResult(
                transport = if (localFallback.acked) {
                    if (localFallback.wifiAcked) com.scmessenger.android.transport.SmartTransportRouter.TransportType.WIFI_DIRECT
                    else com.scmessenger.android.transport.SmartTransportRouter.TransportType.BLE
                } else com.scmessenger.android.transport.SmartTransportRouter.TransportType.CORE,
                success = localFallback.acked,
                latencyMs = 0,
                error = if (localFallback.acked) null else "legacy_fallback_failed"
            )
        }
        
        val localAcked = smartResult.success

        if (strictBleOnly) {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "ble-only",
                phase = "aggregate",
                outcome = if (localAcked) "accepted" else "failed",
                detail = "ctx=$attemptContext route_fallback=$blePeerId"
            )
            // BLE-only mode: accept local ACK as delivery
            return DeliveryAttemptResult(acked = localAcked, routePeerId = blePeerId)
        }

        val bridge = swarmBridge ?: run {
            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "failed",
                detail = "ctx=$attemptContext reason=swarm_bridge_unavailable route_fallback=$wifiPeerId ble_only=${localAcked}"
            )
            // Core unavailable - don't mark as delivered even if BLE succeeded
            return DeliveryAttemptResult(acked = false, routePeerId = wifiPeerId)
        }
        val sanitizedCandidates = routePeerCandidates
            .map { it.trim() }
            .filter { it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it) }
            .distinct()

        if (sanitizedCandidates.isEmpty()) {
            // AND-NO-ROUTE-001: Add diagnostic context for empty route candidates
            val diagnosticDetails = buildString {
                append("ctx=$attemptContext ")
                append("route_fallback=$wifiPeerId ")
                append("ble_only=${localAcked} ")

                // Analyze why candidates might be empty (using available parameters)
                val discoveryCount = discoverRoutePeersForPublicKey(recipientIdentityId).size
                val candidatesCount = routePeerCandidates.size
                val listenersCount = listeners.size

                append("discovery=$discoveryCount ")
                append("input_candidates=$candidatesCount ")
                append("listeners=$listenersCount ")
                append("recipient_id=${recipientIdentityId?.take(8) ?: "null"}")
                append("intended_device=${intendedDeviceId?.take(8) ?: "null"}")
            }

            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "failed",
                detail = "reason=no_route_candidates $diagnosticDetails",
                callerContext = "attemptDirectSwarmDelivery"
            )
            // AND-NO-ROUTE-001: Emit user-visible "connecting" status during route discovery
            if (!traceMessageId.isNullOrBlank()) {
                logDeliveryState(
                    messageId = traceMessageId,
                    state = "connecting",
                    detail = "route_discovery_in_progress ctx=$attemptContext"
                )
            }
            // No core routes - don't mark as delivered even if BLE succeeded
            return DeliveryAttemptResult(acked = false, routePeerId = wifiPeerId)
        }

        primeRelayBootstrapConnections()

        for (routePeerId in sanitizedCandidates) {
            val liveRouteHints = getDialHintsForRoutePeer(routePeerId)
            val dialCandidates = buildDialCandidatesForPeer(
                routePeerId = routePeerId,
                rawAddresses = listeners + liveRouteHints,
                includeRelayCircuits = true
            )
            Timber.d("[ROUTE] Transport: route=$routePeerId dialCandidates=${dialCandidates.size} (${dialCandidates.joinToString { it.substringBefore("/") }})")
            if (dialCandidates.isNotEmpty()) {
                connectToPeer(routePeerId, dialCandidates)
                val connected = awaitPeerConnection(routePeerId, timeoutMs = 2000L)
                Timber.d("[ROUTE] Transport: route=$routePeerId connected=$connected timeout=2000ms")
            }

            logDeliveryAttempt(
                messageId = traceMessageId,
                medium = "core",
                phase = "direct",
                outcome = "attempt",
                detail = "ctx=$attemptContext route=$routePeerId transports=${dialCandidates.size}"
            )
            val attemptStart = System.currentTimeMillis()
            val directError = bridge.sendMessageStatus(
                routePeerId,
                encryptedData,
                recipientIdentityId,
                intendedDeviceId
            )
            if (directError == null) {
                val latencyMs = System.currentTimeMillis() - attemptStart
                Timber.i("[OK] Direct delivery ACK from $routePeerId (${latencyMs}ms)")
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "direct",
                    outcome = "success",
                    detail = "ctx=$attemptContext route=$routePeerId latency=${latencyMs}ms"
                )
                return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
            } else {
                Timber.w("Core-routed delivery failed for $routePeerId: $directError; trying alternative transports")
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "core",
                    phase = "direct",
                    outcome = "failed",
                    detail = "ctx=$attemptContext route=$routePeerId reason=$directError"
                )
                if (isTerminalIdentityFailure(directError)) {
                    return DeliveryAttemptResult(
                        acked = false,
                        routePeerId = routePeerId,
                        terminalFailureCode = directError
                    )
                }
            }

            val relayOnlyCandidates = relayCircuitAddressesForPeer(routePeerId)
            if (relayOnlyCandidates.isNotEmpty()) {
                Timber.d("[ROUTE] Transport: Attempting relay-circuit for $routePeerId (${relayOnlyCandidates.size} candidates)")
                connectToPeer(routePeerId, relayOnlyCandidates)
                val connected = awaitPeerConnection(routePeerId, timeoutMs = 1500L)
                Timber.d("[ROUTE] Transport: relay-circuit route=$routePeerId connected=$connected timeout=1500ms")
                kotlinx.coroutines.delay(500)
                logDeliveryAttempt(
                    messageId = traceMessageId,
                    medium = "relay-circuit",
                    phase = "retry",
                    outcome = "attempt",
                    detail = "ctx=$attemptContext route=$routePeerId relays=${relayOnlyCandidates.size}"
                )
                val relayStart = System.currentTimeMillis()
                val relayError = bridge.sendMessageStatus(
                    routePeerId,
                    encryptedData,
                    recipientIdentityId,
                    intendedDeviceId
                )
                if (relayError == null) {
                    val latencyMs = System.currentTimeMillis() - relayStart
                    Timber.i("[OK] Delivery ACK from $routePeerId after relay-circuit retry (${latencyMs}ms)")
                    logDeliveryAttempt(
                        messageId = traceMessageId,
                        medium = "relay-circuit",
                        phase = "retry",
                        outcome = "success",
                        detail = "ctx=$attemptContext route=$routePeerId latency=${latencyMs}ms"
                    )
                    return DeliveryAttemptResult(acked = true, routePeerId = routePeerId)
                } else {
                    Timber.w("Relay-circuit retry failed for $routePeerId: $relayError")
                    logDeliveryAttempt(
                        messageId = traceMessageId,
                        medium = "relay-circuit",
                        phase = "retry",
                        outcome = "failed",
                        detail = "ctx=$attemptContext route=$routePeerId reason=$relayError"
                    )
                    if (isTerminalIdentityFailure(relayError)) {
                        return DeliveryAttemptResult(
                            acked = false,
                            routePeerId = routePeerId,
                            terminalFailureCode = relayError
                        )
                    }
                }
            }
        }
        logDeliveryAttempt(
            messageId = traceMessageId,
            medium = "final",
            phase = "aggregate",
            outcome = "failed",
            detail = "ctx=$attemptContext reason=all_transports_failed ble_only=${localAcked}"
        )
        return DeliveryAttemptResult(acked = false, routePeerId = null)
    }

    private suspend fun awaitPeerConnection(peerId: String, timeoutMs: Long = 1200L): Boolean {
        val bridge = swarmBridge ?: return false
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val connected = try {
                bridge.getPeers().any { it == peerId }
            } catch (_: Exception) {
                false
            }
            if (connected) return true
            kotlinx.coroutines.delay(100)
        }
        return false
    }

    private suspend fun flushPendingOutbox(reason: String) {
        pendingOutboxFlushMutex.lock()
        try {
            // Ensure relay backbone is reachable whenever we check outbox
            primeRelayBootstrapConnections()

            val now = System.currentTimeMillis() / 1000
            val queue = loadPendingOutbox().toMutableList()
            if (queue.isEmpty()) return
            Timber.d("Flushing pending outbox (${queue.size} item(s)); reason=$reason")

            var updated = false
            val iterator = queue.listIterator()
            while (iterator.hasNext()) {
                // Yield between items to prevent CPU starvation under retry load.
                kotlinx.coroutines.yield()
                val item = iterator.next()
                val expiryReason = pendingOutboxExpiryReason(item, now)
                if (expiryReason != null) {
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "failed",
                        detail = "dropped_pending_outbox reason=$expiryReason attempt=${item.attemptCount}"
                    )
                    iterator.remove()
                    updated = true
                    continue
                }
                if (item.terminalFailureCode != null) {
                    continue
                }
                // A message that keeps getting acked by transport but never sees a
                // receipt is not a failure - stop retrying on a patient age-based
                // ceiling instead of the attempt-count ceiling below, and never
                // flag it as corrupted (transport-confirmed-success must never
                // look like corruption to the user).
                if (Companion.shouldStopAckedWithoutReceiptRetries(
                        ackedWithoutReceiptCount = item.ackedWithoutReceiptCount,
                        createdAtEpochSec = item.createdAtEpochSec,
                        nowEpochSec = now,
                        maxAgeSeconds = pendingOutboxMaxAgeSeconds
                    )
                ) {
                    Timber.w("Stopping retries for message ${item.historyRecordId} after ${item.ackedWithoutReceiptCount} acked-without-receipt attempts (max age reached)")
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "delivered_unconfirmed",
                        detail = "stopped_pending_outbox reason=max_age_exceeded_acked_without_receipt acked_without_receipt=${item.ackedWithoutReceiptCount}"
                    )
                    iterator.remove()
                    updated = true
                    continue
                }
                // P3_ANDROID_RETRY_SUPPRESSION: No-downgrade rule enforcement
                // Once message reaches Sent state (confirmed by transport), it may NOT move to Failed or Corrupted.
                // Messages that have been acked by transport are tracked separately (ackedWithoutReceiptCount)
                // and should only be stopped via age-based ceiling (above), never via attempt-count ceiling.
                if (item.ackedWithoutReceiptCount > 0) {
                    // Transport-confirmed success: keep message in Sent state indefinitely (until age-based stop)
                    Timber.d("Skipping retry for ${item.historyRecordId}: transport-acked message cannot be downgraded acked_count=${item.ackedWithoutReceiptCount}")
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "held",
                        detail = "acked_without_receipt_protection acked_count=${item.ackedWithoutReceiptCount} attempt=${item.attemptCount}"
                    )
                    iterator.set(
                        item.copy(
                            nextAttemptAtEpochSec = now + 120L  // Schedule next check in 2 minutes
                        )
                    )
                    updated = true
                    continue
                }
                // AND-DELIVERY-001: Enforce maximum retry limit to prevent infinite retries
                // Only applies to messages that have NOT been transport-acked
                if (item.attemptCount >= pendingOutboxMaxAttempts) {
                    Timber.w("Dropping message ${item.historyRecordId} after ${item.attemptCount} attempts (max=$pendingOutboxMaxAttempts) - NOT transport-acked")
                    markMessageCorrupted(item.historyRecordId)
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "failed",
                        detail = "dropped_pending_outbox reason=max_attempts_exceeded attempt=${item.attemptCount}"
                    )
                    iterator.remove()
                    updated = true
                    continue
                }
                if (item.nextAttemptAtEpochSec > now) continue
                if (!shouldRetryMessage(item.historyRecordId)) {
                    Timber.w("Skipping retry for ${item.historyRecordId}: shouldRetryMessage=false")
                    continue
                }
                if (isMessageDeliveredLocally(item.historyRecordId)) {
                    iterator.remove()
                    updated = true
                    continue
                }
                logDeliveryState(
                    messageId = item.historyRecordId,
                    state = "forwarding",
                    detail = "retry_attempt=${item.attemptCount + 1}"
                )
                logMessageDeliveryAttempt(item.historyRecordId, item.attemptCount + 1, "forwarding")

                val envelope = try {
                    android.util.Base64.decode(item.envelopeBase64, android.util.Base64.NO_WRAP)
                } catch (_: Exception) {
                    Timber.w("Dropping corrupt pending envelope ${item.queueId}")
                    iterator.remove()
                    updated = true
                    continue
                }

                val contact = contactManager?.get(item.peerId)
                val latestRouting = parseRoutingHints(contact?.notes)
                val routePeerCandidates = buildRoutePeerCandidates(
                    peerId = item.peerId,
                    cachedRoutePeerId = item.routePeerId,
                    notes = contact?.notes,
                    recipientPublicKey = contact?.publicKey
                )
                val resolvedRoutePeerId = routePeerCandidates.firstOrNull()
                val resolvedListeners = buildDialCandidatesForPeer(
                    routePeerId = resolvedRoutePeerId,
                    rawAddresses = item.listeners + latestRouting.listeners,
                    includeRelayCircuits = true
                )

                val delivery = attemptDirectSwarmDelivery(
                    routePeerCandidates = routePeerCandidates,
                    listeners = resolvedListeners,
                    encryptedData = envelope,
                    wifiPeerId = latestRouting.wifiPeerId,
                    blePeerId = latestRouting.blePeerId,
                    traceMessageId = item.historyRecordId,
                    attemptContext = "outbox_retry",
                    strictBleOnlyOverride = item.strictBleOnlyMode,
                    recipientIdentityId = item.recipientIdentityId ?: contact?.publicKey,
                    intendedDeviceId = item.intendedDeviceId ?: contact?.lastKnownDeviceId
                )
                val selectedRoutePeerId = if (delivery.acked) {
                    delivery.routePeerId ?: resolvedRoutePeerId
                } else {
                    delivery.routePeerId
                }
                if (isMessageDeliveredLocally(item.historyRecordId)) {
                    iterator.remove()
                    updated = true
                    continue
                }

                if (delivery.terminalFailureCode != null) {
                    iterator.set(
                        item.copy(
                            routePeerId = selectedRoutePeerId,
                            listeners = resolvedListeners,
                            terminalFailureCode = delivery.terminalFailureCode
                        )
                    )
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "rejected",
                        detail = "terminal_failure_code=${delivery.terminalFailureCode}"
                    )
                    updated = true
                    continue
                }

                if (delivery.acked) {
                    // P3: feed the no-downgrade rule in markMessageCorrupted()
                    recordMessageTransportAck(item.historyRecordId)
                    // AND-NO-ROUTE-001: Persist last-known-good route in contact notes for future fallback
                    storeLastKnownRoutePeerId(item.peerId, selectedRoutePeerId)
                    // AND-DELIVERY-001 / FARM WS-A3: transport genuinely succeeded, so
                    // track this on the separate ackedWithoutReceiptCount counter, NOT
                    // attemptCount - a confirmed send must never count toward the
                    // genuine-failure ceiling below (pendingOutboxMaxAttempts).
                    // P3_ANDROID_RETRY_SUPPRESSION: First receipt window is now 60s (RECEIPT_ACK_TIMEOUT_MS)
                    // to allow relay custody delay + network latency. No-downgrade rule prevents
                    // transport-acked messages from being marked Failed/Corrupted.
                    val nextAckedWithoutReceiptCount = item.ackedWithoutReceiptCount + 1
                    val adaptiveReceiptWait = when {
                        nextAckedWithoutReceiptCount <= 3 -> receiptAwaitSeconds  // 60s for first few attempts (P3 hardened)
                        nextAckedWithoutReceiptCount <= 8 -> 30L                 // 30s for moderate retries
                        else -> 120L                                             // 2 min for later ones
                    }
                    iterator.set(
                        item.copy(
                            routePeerId = selectedRoutePeerId,
                            listeners = resolvedListeners,
                            ackedWithoutReceiptCount = nextAckedWithoutReceiptCount,
                            nextAttemptAtEpochSec = now + adaptiveReceiptWait,
                            strictBleOnlyMode = item.strictBleOnlyMode
                        )
                    )
                    logDeliveryState(
                        messageId = item.historyRecordId,
                        state = "stored",
                        detail = "awaiting_receipt_delay_sec=$adaptiveReceiptWait acked_without_receipt=$nextAckedWithoutReceiptCount"
                    )
                    updated = true
                    continue
                }

                val nextAttemptCount = item.attemptCount + 1
                incrementAttemptCount(item.historyRecordId)
                // Aggressive retry for transport transitions: fast initial attempts
                // Attempt 1: 0.5s, 2: 1s, 3: 2s, 4: 4s, 5: 8s, 6: 16s
                // Attempts 7-20: 60 seconds (steady retry)
                // Attempts 21+: 300 seconds (patient long-term)
                val backoffSecs = when (nextAttemptCount) {
                    1 -> 0L  // Immediate first retry
                    2 -> 1L
                    3 -> 2L
                    4 -> 4L
                    5 -> 8L
                    6 -> 16L
                    in 7..20 -> 60L
                    else -> 300L
                }
                iterator.set(
                    item.copy(
                        routePeerId = selectedRoutePeerId,
                        listeners = resolvedListeners,
                        attemptCount = nextAttemptCount,
                        nextAttemptAtEpochSec = now + backoffSecs,
                        strictBleOnlyMode = item.strictBleOnlyMode
                    )
                )
                logDeliveryState(
                    messageId = item.historyRecordId,
                    state = "stored",
                    detail = "retry_backoff_sec=$backoffSecs attempt=$nextAttemptCount"
                )
                updated = true
            }

            if (updated) {
                savePendingOutbox(queue)
                // P0_ANDROID_005: Check for retry storms after outbox state changes
                logRetryStormDetection()
            }
        } finally {
            pendingOutboxFlushMutex.unlock()
        }
    }

    private fun enqueuePendingOutbound(
        historyRecordId: String,
        peerId: String,
        routePeerId: String?,
        listeners: List<String>,
        encryptedData: ByteArray,
        initialAttemptCount: Int = 0,
        initialDelaySec: Long = 0,
        strictBleOnlyMode: Boolean = false,
        recipientIdentityId: String? = null,
        intendedDeviceId: String? = null,
        terminalFailureCode: String? = null
    ) {
        if (isMessageDeliveredLocally(historyRecordId)) {
            logDeliveryState(
                messageId = historyRecordId,
                state = "delivered",
                detail = "skip_enqueue_already_delivered"
            )
            return
        }
        val now = System.currentTimeMillis() / 1000
        val queue = loadPendingOutbox().toMutableList()
        queue.removeAll { it.historyRecordId == historyRecordId }
        queue.add(
            PendingOutboundEnvelope(
                queueId = java.util.UUID.randomUUID().toString(),
                historyRecordId = historyRecordId,
                peerId = peerId,
                routePeerId = routePeerId,
                listeners = listeners,
                envelopeBase64 = android.util.Base64.encodeToString(
                    encryptedData,
                    android.util.Base64.NO_WRAP
                ),
                createdAtEpochSec = now,
                attemptCount = initialAttemptCount,
                nextAttemptAtEpochSec = now + initialDelaySec,
                strictBleOnlyMode = strictBleOnlyMode,
                recipientIdentityId = recipientIdentityId,
                intendedDeviceId = intendedDeviceId,
                terminalFailureCode = terminalFailureCode
            )
        )
        savePendingOutbox(queue)
        val initialState = if (initialDelaySec > 0) "stored" else "forwarding"
        logDeliveryState(
            messageId = historyRecordId,
            state = initialState,
            detail = "enqueued attempt=$initialAttemptCount next_attempt_delay_sec=$initialDelaySec"
        )
        repoScope.launch { flushPendingOutbox("enqueue") }
    }

    // ANR FIX: Cache for pending outbox to avoid repeated I/O
    @Volatile private var cachedPendingOutbox: List<PendingOutboundEnvelope> = emptyList()
    @Volatile private var pendingOutboxCacheTimeMs: Long = 0L
    private val pendingOutboxCacheTtlMs = 1000L  // 1 second TTL

    /**
     * ANR FIX (P0_ANDROID_017): Load pending outbox asynchronously to avoid Main thread I/O.
     * Uses cached results when available to prevent repeated file reads during diagnostics export.
     */
    internal suspend fun loadPendingOutboxAsync(): List<PendingOutboundEnvelope> = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
        // Return cached result if fresh enough
        if (cachedPendingOutbox.isNotEmpty() && System.currentTimeMillis() - pendingOutboxCacheTimeMs < pendingOutboxCacheTtlMs) {
            return@withContext cachedPendingOutbox
        }

        val result = if (!pendingOutboxFile.exists()) {
            emptyList()
        } else {
            try {
                val raw = pendingOutboxFile.readText()
                if (raw.isBlank()) return@withContext emptyList()
                val arr = org.json.JSONArray(raw)
                buildList {
                    for (i in 0 until arr.length()) {
                        val obj = arr.optJSONObject(i) ?: continue
                        val routePeerId = obj.optString("route_peer_id").ifBlank { null }
                        val listenersJson = obj.optJSONArray("listeners")
                        val listeners = buildList {
                            for (idx in 0 until (listenersJson?.length() ?: 0)) {
                                val value = listenersJson?.optString(idx).orEmpty().trim()
                                if (value.isNotEmpty()) add(value)
                            }
                        }
                        add(
                            PendingOutboundEnvelope(
                                queueId = obj.optString("queue_id").ifBlank { java.util.UUID.randomUUID().toString() },
                                historyRecordId = obj.optString("history_record_id"),
                                peerId = obj.optString("peer_id"),
                                routePeerId = routePeerId,
                                listeners = listeners,
                                envelopeBase64 = obj.optString("envelope_b64"),
                                createdAtEpochSec = obj.optLong("created_at", System.currentTimeMillis() / 1000),
                                attemptCount = obj.optInt("attempt_count", 0),
                                nextAttemptAtEpochSec = obj.optLong("next_attempt_at", 0),
                                strictBleOnlyMode = if (obj.has("strict_ble_only_mode")) obj.optBoolean("strict_ble_only_mode") else null,
                                recipientIdentityId = obj.optString("recipient_identity_id").ifBlank { null },
                                intendedDeviceId = obj.optString("intended_device_id").ifBlank { null },
                                terminalFailureCode = obj.optString("terminal_failure_code").ifBlank { null }
                            )
                        )
                    }
                }
            } catch (e: Exception) {
                Timber.w(e, "Failed to parse pending outbox")
                emptyList()
            }
        }

        // Update cache
        cachedPendingOutbox = result
        pendingOutboxCacheTimeMs = System.currentTimeMillis()
        result
    }

    /**
     * Synchronous load for internal use (must only be called from IO dispatcher).
     */
    @Synchronized
    private fun loadPendingOutboxSync(): List<PendingOutboundEnvelope> {
        if (!pendingOutboxFile.exists()) return emptyList()
        return try {
            val raw = pendingOutboxFile.readText()
            if (raw.isBlank()) return emptyList()
            val arr = org.json.JSONArray(raw)
            buildList {
                for (i in 0 until arr.length()) {
                    val obj = arr.optJSONObject(i) ?: continue
                    val routePeerId = obj.optString("route_peer_id").ifBlank { null }
                    val listenersJson = obj.optJSONArray("listeners")
                    val listeners = buildList {
                        for (idx in 0 until (listenersJson?.length() ?: 0)) {
                            val value = listenersJson?.optString(idx).orEmpty().trim()
                            if (value.isNotEmpty()) add(value)
                        }
                    }
                    add(
                        PendingOutboundEnvelope(
                            queueId = obj.optString("queue_id").ifBlank { java.util.UUID.randomUUID().toString() },
                            historyRecordId = obj.optString("history_record_id"),
                            peerId = obj.optString("peer_id"),
                            routePeerId = routePeerId,
                            listeners = listeners,
                            envelopeBase64 = obj.optString("envelope_b64"),
                            createdAtEpochSec = obj.optLong("created_at", System.currentTimeMillis() / 1000),
                            attemptCount = obj.optInt("attempt_count", 0),
                            nextAttemptAtEpochSec = obj.optLong("next_attempt_at", 0),
                            strictBleOnlyMode = if (obj.has("strict_ble_only_mode")) obj.optBoolean("strict_ble_only_mode") else null,
                            recipientIdentityId = obj.optString("recipient_identity_id").ifBlank { null },
                            intendedDeviceId = obj.optString("intended_device_id").ifBlank { null },
                            terminalFailureCode = obj.optString("terminal_failure_code").ifBlank { null },
                            // P3: ack history must survive restart or the
                            // no-downgrade/held protections reset to zero.
                            ackedWithoutReceiptCount = obj.optInt("acked_without_receipt_count", 0)
                        )
                    )
                }
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to parse pending outbox")
            emptyList()
        }
    }

    /**
     * Public sync version - must only be called from IO dispatcher.
     */
    internal fun loadPendingOutbox(): List<PendingOutboundEnvelope> = loadPendingOutboxSync()

    @Synchronized
    private fun savePendingOutbox(queue: List<PendingOutboundEnvelope>) {
        try {
            val arr = org.json.JSONArray()
            queue.forEach { item ->
                val listeners = org.json.JSONArray()
                item.listeners.forEach { listeners.put(it) }
                arr.put(
                    org.json.JSONObject()
                        .put("queue_id", item.queueId)
                        .put("history_record_id", item.historyRecordId)
                        .put("peer_id", item.peerId)
                        .put("route_peer_id", item.routePeerId ?: "")
                        .put("listeners", listeners)
                        .put("envelope_b64", item.envelopeBase64)
                        .put("created_at", item.createdAtEpochSec)
                        .put("attempt_count", item.attemptCount)
                        .put("next_attempt_at", item.nextAttemptAtEpochSec)
                        .put("strict_ble_only_mode", item.strictBleOnlyMode ?: false)
                        .put("recipient_identity_id", item.recipientIdentityId ?: "")
                        .put("intended_device_id", item.intendedDeviceId ?: "")
                        .put("terminal_failure_code", item.terminalFailureCode ?: "")
                        .put("acked_without_receipt_count", item.ackedWithoutReceiptCount)
                )
            }
            pendingOutboxFile.writeText(arr.toString())
        } catch (e: Exception) {
            Timber.w(e, "Failed to persist pending outbox")
        }
    }

    @Suppress("UNUSED_PARAMETER")
    private fun pendingOutboxExpiryReason(item: PendingOutboundEnvelope, nowEpochSec: Long): String? {
        // PHILOSOPHY: Messages NEVER expire. Every message retries
        // until successfully delivered. No attempt limit, no age limit.
        return null
    }

    // ========================================================================
    // ROUTING HELPERS
    // ========================================================================

    private fun resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String): String {
        val normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex) ?: return senderId

        // Priority 1: Use direct hash resolution from core
        val identityId = try { ironCore?.resolveIdentity(senderPublicKeyHex) } catch (e: Exception) { null }
        if (identityId != null) {
            // Check if we already have a contact pointing to ANOTHER peerId for this key
            // (e.g. legacy libp2p ID contact). If so, we merge by returning the existing ID.
            val contacts = try { contactManager?.list().orEmpty() } catch (e: Exception) { emptyList() }
            val existingContact = contacts.firstOrNull { normalizePublicKey(it.publicKey) == normalizedIncomingKey }
            return existingContact?.peerId ?: identityId
        }

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to resolve canonical sender ID: ${e.message}")
            return senderId
        }

        val exactMatch = contacts.firstOrNull {
            PeerIdValidator.isSame(it.peerId, senderId) && normalizePublicKey(it.publicKey) == normalizedIncomingKey
        }
        if (exactMatch != null) return exactMatch.peerId

        // Prefer one stable canonical identity per public key whenever unique.
        val keyedContacts = contacts.filter {
            normalizePublicKey(it.publicKey) == normalizedIncomingKey
        }
        if (keyedContacts.size == 1) {
            return keyedContacts.first().peerId
        }
        if (keyedContacts.size > 1) {
            Timber.w(
                "Ambiguous canonical sender mapping for key ${normalizedIncomingKey.take(8)}... (matched ${keyedContacts.size} contacts); trying route-hint fallback"
            )
        }

        // Alias libp2p sender IDs only when there is explicit linkage via notes:
        // "libp2p_peer_id:<senderId>" and the key matches.
        if (PeerIdValidator.isLibp2pPeerId(senderId)) {
            val linkedIdentityContacts = contacts.filter {
                normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                    PeerIdValidator.isSame(parseRoutingHints(it.notes).libp2pPeerId.orEmpty(), senderId) &&
                    !PeerIdValidator.isSame(it.peerId, senderId)
            }

            return when (linkedIdentityContacts.size) {
                1 -> linkedIdentityContacts.first().peerId
                else -> {
                    if (linkedIdentityContacts.size > 1) {
                        Timber.w(
                            "Ambiguous canonical sender mapping for $senderId (matched ${linkedIdentityContacts.size} contacts); keeping raw sender ID"
                        )
                    }
                    PeerIdValidator.normalize(senderId)
                }
            }
        }

        // Identity IDs (Blake3 hex) can represent the same peer that was
        // previously saved under a libp2p contact ID; map only when unique.
        if (!PeerIdValidator.isIdentityId(senderId)) return senderId
        val keyedRoutedContacts = contacts.filter {
            normalizePublicKey(it.publicKey) == normalizedIncomingKey &&
                it.peerId != senderId &&
                (
                    !parseRoutingHints(it.notes).libp2pPeerId.isNullOrBlank() ||
                    PeerIdValidator.isLibp2pPeerId(it.peerId)
                )
        }
        return when (keyedRoutedContacts.size) {
            1 -> keyedRoutedContacts.first().peerId
            else -> {
                if (keyedRoutedContacts.size > 1) {
                    Timber.w(
                        "Ambiguous identity sender mapping for $senderId (matched ${keyedRoutedContacts.size} contacts); keeping raw sender ID"
                    )
                }
                PeerIdValidator.normalize(senderId)
            }
        }
    }

    private fun resolveCanonicalPeerIdFromMessageHints(
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ): String {
        val hint = hintedIdentityId?.trim() ?: return resolvedCanonicalPeerId
        val normalizedHint = PeerIdValidator.normalize(hint)
        if (!PeerIdValidator.isIdentityId(normalizedHint)) return resolvedCanonicalPeerId
        if (PeerIdValidator.isSame(normalizedHint, resolvedCanonicalPeerId)) return resolvedCanonicalPeerId
        if (isBootstrapRelayPeer(normalizedHint)) return resolvedCanonicalPeerId

        val normalizedSenderKey = normalizePublicKey(senderPublicKeyHex)
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (_: Exception) {
            emptyList()
        }

        if (normalizedSenderKey != null) {
            val hintedContact = contacts.firstOrNull { PeerIdValidator.isSame(it.peerId, normalizedHint) }
            if (hintedContact != null && normalizePublicKey(hintedContact.publicKey) == normalizedSenderKey) {
                return hintedContact.peerId
            }

            val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedSenderKey }
            if (keyMatches.size == 1) return keyMatches.first().peerId
            if (keyMatches.isNotEmpty()) return resolvedCanonicalPeerId
        }

        return if (PeerIdValidator.isSame(resolvedCanonicalPeerId, senderId) || PeerIdValidator.isLibp2pPeerId(resolvedCanonicalPeerId)) {
            normalizedHint
        } else {
            resolvedCanonicalPeerId
        }
    }

    private fun encodeMessageWithIdentityHints(content: String): String {
        return encodeMeshMessagePayload(content = content, kind = "text")
    }

    private fun encodeIdentitySyncPayload(): String {
        return encodeMeshMessagePayload(content = "", kind = "identity_sync")
    }

    private fun encodeMeshMessagePayload(content: String, kind: String): String {
        val identity = ironCore?.getIdentityInfo() ?: return content
        val publicKeyHex = normalizePublicKey(identity.publicKeyHex) ?: return content

        val identityId = identity.identityId?.trim().orEmpty()
        val deviceId = identity.deviceId?.trim().orEmpty()
        val nickname = normalizeNickname(identity.nickname)?.take(64).orEmpty()
        val libp2pPeerId = identity.libp2pPeerId?.trim().orEmpty()
        val listeners = normalizeOutboundListenerHints(getListeningAddresses()).take(3)
        val externalAddresses = normalizeExternalAddressHints(getExternalAddresses()).take(3)
        val connectionHints = (listeners + externalAddresses).distinct().take(6)

        return try {
            org.json.JSONObject()
                .put("schema", "scm.message.identity.v1")
                .put("kind", kind)
                .put("text", content)
                .put(
                    "sender",
                    org.json.JSONObject()
                        .put("identity_id", identityId)
                        .put("public_key", publicKeyHex)
                        .put("device_id", deviceId)
                        .put("nickname", nickname)
                        .put("libp2p_peer_id", libp2pPeerId)
                        .put("listeners", org.json.JSONArray(listeners))
                        .put("external_addresses", org.json.JSONArray(externalAddresses))
                        .put("connection_hints", org.json.JSONArray(connectionHints))
                )
                .toString()
        } catch (_: Exception) {
            content
        }
    }

    private fun decodeMessageWithIdentityHints(raw: String): DecodedMessagePayload {
        val trimmed = raw.trim()
        if (!trimmed.startsWith("{")) {
            return DecodedMessagePayload(kind = "text", text = raw, hints = null)
        }

        return try {
            val json = org.json.JSONObject(trimmed)
            if (json.optString("schema") != "scm.message.identity.v1") {
                return DecodedMessagePayload(kind = "text", text = raw, hints = null)
            }

            val sender = json.optJSONObject("sender")
            val hints = sender?.let {
                MessageIdentityHints(
                    identityId = it.optString("identity_id", "").trim().takeIf { value -> value.isNotBlank() },
                    publicKey = normalizePublicKey(it.optString("public_key", "")),
                    deviceId = it.optString("device_id", "").trim().takeIf { value -> value.isNotBlank() },
                    nickname = normalizeNickname(it.optString("nickname", "")),
                    libp2pPeerId = it.optString("libp2p_peer_id", "").trim().takeIf { value ->
                        value.isNotBlank() && PeerIdValidator.isLibp2pPeerId(value)
                    },
                    listeners = jsonArrayToStringList(it.optJSONArray("listeners")),
                    externalAddresses = jsonArrayToStringList(it.optJSONArray("external_addresses")),
                    connectionHints = jsonArrayToStringList(it.optJSONArray("connection_hints"))
                )
            }

            val text = json.optString("text", raw)
            val kind = json.optString("kind", "text").trim().ifEmpty { "text" }
            DecodedMessagePayload(
                kind = kind,
                text = text,
                hints = hints
            )
        } catch (_: Exception) {
            DecodedMessagePayload(kind = "text", text = raw, hints = null)
        }
    }

    private fun jsonArrayToStringList(array: org.json.JSONArray?): List<String> {
        if (array == null) return emptyList()
        return buildList {
            for (i in 0 until array.length()) {
                val value = array.optString(i).trim()
                if (value.isNotEmpty()) add(value)
            }
        }.distinct()
    }

    private fun normalizePublicKey(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
        return trimmed.lowercase()
    }

    private fun normalizeNickname(value: String?): String? {
        return value?.trim()?.takeIf { it.isNotEmpty() }
    }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val normalized = normalizeNickname(value)?.lowercase() ?: return false
        // "peer-xxxxxx" is our receiver-side placeholder. It should never
        // overwrite a sender-provided nickname.
        return normalized.startsWith("peer-")
    }

    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)

        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            else -> incomingNormalized
        }
    }

    private fun isBlePeerId(value: String): Boolean {
        return kotlin.runCatching { java.util.UUID.fromString(value.trim()) }.isSuccess
    }

    private fun isWifiPeerId(value: String): Boolean {
        val normalized = value.trim()
        if (normalized.isEmpty()) return false
        val macRegex = Regex("(?i)^([0-9a-f]{2}:){5}[0-9a-f]{2}$")
        val ipv4Regex = Regex("^\\d{1,3}(\\.\\d{1,3}){3}$")
        return macRegex.matches(normalized) || ipv4Regex.matches(normalized)
    }

    private fun selectCanonicalPeerId(incomingPeerId: String, existingPeerId: String): String {
        val incoming = incomingPeerId.trim()
        val existing = existingPeerId.trim()
        if (incoming.isEmpty()) return existing
        if (existing.isEmpty() || existing == incoming) return incoming

        val incomingIsLibp2p = PeerIdValidator.isLibp2pPeerId(incoming)
        val existingIsLibp2p = PeerIdValidator.isLibp2pPeerId(existing)
        val incomingIsIdentity = PeerIdValidator.isIdentityId(incoming)
        val existingIsIdentity = PeerIdValidator.isIdentityId(existing)
        val incomingIsBle = isBlePeerId(incoming)
        val existingIsBle = isBlePeerId(existing)

        return when {
            existingIsIdentity && incomingIsLibp2p -> existing
            incomingIsIdentity && existingIsLibp2p -> incoming
            existingIsBle && !incomingIsBle -> incoming
            !existingIsBle && incomingIsBle -> existing
            else -> incoming
        }
    }

    private fun prepopulateDiscoveryNickname(
        nickname: String?,
        peerId: String,
        publicKey: String?
    ): String {
        val incomingNickname = normalizeNickname(nickname)

        val normalizedKey = normalizePublicKey(publicKey)
        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (_: Exception) {
            emptyList()
        }

        val fromContact = contacts.firstOrNull {
            it.peerId == peerId ||
                (
                    normalizedKey != null &&
                        normalizePublicKey(it.publicKey) == normalizedKey
                    )
        }?.nickname

        val resolved = selectAuthoritativeNickname(incomingNickname, fromContact)
        if (!resolved.isNullOrBlank()) {
            return resolved
        }

        // Auto-generate nickname if none found
        val shortId = if (peerId.startsWith("12D3KooW")) {
            peerId.takeLast(8)
        } else {
            peerId.take(8)
        }
        val generated = "peer-$shortId"
        Timber.d("Generated default nickname '$generated' for peer $peerId")
        return generated
    }

    private fun resolveKnownPeerNickname(
        canonicalPeerId: String,
        routePeerId: String?,
        publicKey: String?
    ): String? {
        val normalizedKey = normalizePublicKey(publicKey)
        val routeCandidate = routePeerId?.trim()?.takeIf { it.isNotBlank() }

        val fromDiscovery = _discoveredPeers.value.values
            .asSequence()
            .filter { info ->
                info.peerId == canonicalPeerId ||
                    (!routeCandidate.isNullOrBlank() && info.peerId == routeCandidate) ||
                    (
                        normalizedKey != null &&
                            normalizePublicKey(info.publicKey) == normalizedKey
                        )
            }
            .mapNotNull { normalizeNickname(it.nickname) }
            .firstOrNull()

        val fromLedger = ledgerManager?.dialableAddresses()
            ?.asSequence()
            ?.filter { entry ->
                entry.peerId == canonicalPeerId ||
                    (!routeCandidate.isNullOrBlank() && entry.peerId == routeCandidate) ||
                    (
                        normalizedKey != null &&
                            normalizePublicKey(entry.publicKey) == normalizedKey
                        )
            }
            ?.mapNotNull { normalizeNickname(it.nickname) }
            ?.firstOrNull()

        val fromContact = try {
            contactManager?.list()
                ?.asSequence()
                ?.filter { contact ->
                    contact.peerId == canonicalPeerId ||
                        (!routeCandidate.isNullOrBlank() && contact.peerId == routeCandidate) ||
                        (
                            normalizedKey != null &&
                                normalizePublicKey(contact.publicKey) == normalizedKey
                            )
                }
                ?.mapNotNull { normalizeNickname(it.nickname) }
                ?.firstOrNull()
        } catch (_: Exception) {
            null
        }

        val discoveryOrLedger = selectAuthoritativeNickname(fromDiscovery, fromLedger)
        return selectAuthoritativeNickname(discoveryOrLedger, fromContact)
    }

    private fun annotateIdentityInLedger(
        routePeerId: String?,
        listeners: List<String>,
        publicKey: String?,
        nickname: String?
    ) {
        val normalizedRoute = routePeerId?.trim().orEmpty()
        if (normalizedRoute.isEmpty() || !PeerIdValidator.isLibp2pPeerId(normalizedRoute)) return

        val dialHints = buildDialCandidatesForPeer(
            routePeerId = normalizedRoute,
            rawAddresses = listeners,
            includeRelayCircuits = true
        )
        if (dialHints.isEmpty()) return

        val normalizedKey = normalizePublicKey(publicKey)
        val normalizedNickname = nickname?.trim()?.takeIf { it.isNotEmpty() }
        dialHints.forEach { multiaddr ->
            kotlin.runCatching {
                ledgerManager?.annotateIdentity(
                    multiaddr,
                    normalizedRoute,
                    normalizedKey,
                    normalizedNickname
                )
            }.onFailure {
                Timber.d("Failed to annotate identity for ledger entry $multiaddr: ${it.message}")
            }
        }
    }

    private fun appendRoutingHint(notes: String?, key: String, value: String?): String? {
        val normalizedValue = value?.trim().orEmpty()
        if (normalizedValue.isEmpty()) return notes

        val existing = notes?.trim().orEmpty()
        val components = if (existing.isEmpty()) {
            mutableListOf<String>()
        } else {
            existing.split(';', '\n').map { it.trim() }.toMutableList()
        }

        // Replace existing entry for this key instead of appending a duplicate.
        // This prevents stale BLE MACs / route peer IDs from accumulating.
        val existingIndex = components.indexOfFirst { it.startsWith("$key:") }
        val newEntry = "$key:$normalizedValue"
        if (existingIndex >= 0) {
            if (components[existingIndex] == newEntry) return notes // unchanged
            Timber.d("Routing hint update: replacing old ${components[existingIndex]} with $newEntry")
            components[existingIndex] = newEntry
        } else {
            components.add(newEntry)
        }

        return components.filter { it.isNotEmpty() }.joinToString(";")
    }

    /**
     * AND-NO-ROUTE-001: Persist last-known-good routePeerId in contact notes so that
     * future deliveries can fall back to it when fresh discovery returns empty.
     */
    private fun storeLastKnownRoutePeerId(peerId: String, routePeerId: String?) {
        if (routePeerId.isNullOrBlank()) return
        if (!PeerIdValidator.isLibp2pPeerId(routePeerId)) return
        try {
            val contact = contactManager?.get(peerId) ?: return
            val updatedNotes = appendRoutingHint(contact.notes, "last_known_route", routePeerId)
            if (updatedNotes != contact.notes) {
                contactManager?.add(contact.copy(notes = updatedNotes))
                Timber.d("Stored last-known route for $peerId: $routePeerId")
            }
        } catch (e: Exception) {
            Timber.w(e, "Failed to store last-known route for $peerId")
        }
    }

    /**
     * Merge routing notes from two contacts, preserving all unique hints.
     * Used during ID coalescence to avoid losing routing information.
     */
    private fun mergeNotes(existing: String?, incoming: String?): String? {
        if (existing.isNullOrBlank()) return incoming
        if (incoming.isNullOrBlank()) return existing
        
        val existingComponents = existing.split(';', '\n').map { it.trim() }.filter { it.isNotEmpty() }
        val incomingComponents = incoming.split(';', '\n').map { it.trim() }.filter { it.isNotEmpty() }
        
        // Build a map of key:value pairs, preferring existing values
        val merged = mutableMapOf<String, String>()
        for (component in existingComponents) {
            val colonIndex = component.indexOf(':')
            if (colonIndex > 0) {
                merged[component.substring(0, colonIndex)] = component
            } else {
                merged[component] = component
            }
        }
        // Add incoming components that don't conflict
        for (component in incomingComponents) {
            val colonIndex = component.indexOf(':')
            val key = if (colonIndex > 0) component.substring(0, colonIndex) else component
            if (key !in merged) {
                merged[key] = component
            }
        }
        
        return merged.values.filter { it.isNotEmpty() }.joinToString(";").ifEmpty { null }
    }

    private fun resolveTransportIdentity(libp2pPeerId: String): TransportIdentityResolution? {
        if (!PeerIdValidator.isLibp2pPeerId(libp2pPeerId)) {
            Timber.d("Invalid libp2p peer ID format: $libp2pPeerId")
            return null
        }
        
        Timber.d("resolveTransportIdentity called for: $libp2pPeerId")
        
        // Relay peers should not have user-visible transport identities
        val isRelay = isBootstrapRelayPeer(libp2pPeerId)
        Timber.d("  isBootstrapRelayPeer check: $isRelay")
        
        if (isRelay) {
            Timber.d("  → Filtering relay peer from transport identity resolution")
            return null
        }

        val extractedKey = try {
            ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
        } catch (e: Exception) {
            Timber.d("  Failed to extract public key from peer $libp2pPeerId: ${e.message}")
            null
        }
        val normalizedKey = normalizePublicKey(extractedKey) ?: return null

        val selfKey = normalizePublicKey(ironCore?.getIdentityInfo()?.publicKeyHex)
        if (selfKey != null && selfKey == normalizedKey) return null

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to resolve transport identity for $libp2pPeerId: ${e.message}")
            emptyList()
        }

        val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedKey }
        if (keyMatches.size > 1) {
            Timber.w(
                "Multiple contacts share transport key ${normalizedKey.take(8)}...; using explicit route match where possible"
            )
        }

        val routeLinked = keyMatches.firstOrNull { contact ->
            contact.peerId == libp2pPeerId || parseRoutingHints(contact.notes).libp2pPeerId == libp2pPeerId
        }
        val canonicalContact = routeLinked ?: keyMatches.firstOrNull()
        
        if (canonicalContact == null) {
            if (isBootstrapRelayPeerFromKey(normalizedKey)) {
                Timber.d("No existing contact for transport key ${normalizedKey.take(8)}..., treating as transient relay")
                return null
            }

            val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(normalizedKey)
            if (!validatePeerBeforeContactCreation(peerId, normalizedKey)) {
                Timber.w("Peer validation failed for ${normalizedKey.take(8)}..., skipping emergency contact creation")
                return TransportIdentityResolution(
                    canonicalPeerId = normalizedKey,
                    publicKey = normalizedKey,
                    nickname = null,
                    localNickname = null
                )
            }

            Timber.w("Creating emergency contact for unknown peer: ${normalizedKey.take(8)}...")
            val emergencyContact = createEmergencyContact(normalizedKey)
            logIdentityResolutionDetails(normalizedKey, emergencyContact, createdNew = true)

            return TransportIdentityResolution(
                canonicalPeerId = normalizedKey,
                publicKey = normalizedKey,
                nickname = emergencyContact.nickname?.takeIf { it.isNotBlank() },
                localNickname = emergencyContact.localNickname?.takeIf { it.isNotBlank() }
            )
        }

        logIdentityResolutionDetails(normalizedKey, canonicalContact, createdNew = false)

        // UNIFIED ID FIX: canonicalPeerId is ALWAYS public_key_hex for contact storage.
        // libp2p_peer_id is only used for network transport, never as the contact key.
        return TransportIdentityResolution(
            canonicalPeerId = normalizedKey,
            publicKey = normalizedKey,
            nickname = canonicalContact.nickname?.takeIf { it.isNotBlank() },
            localNickname = canonicalContact.localNickname?.takeIf { it.isNotBlank() }
        )
    }

    private fun persistRouteHintsForTransportPeer(
        libp2pPeerId: String,
        listeners: List<String>,
        knownPublicKey: String? = null
    ) {
        val normalizedListeners = normalizeOutboundListenerHints(listeners)
        if (libp2pPeerId.isBlank()) return

        val normalizedTransportKey = knownPublicKey
            ?: normalizePublicKey(
                try {
                    ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
                } catch (e: Exception) {
                    Timber.d("Unable to derive public key for transport peer $libp2pPeerId: ${e.message}")
                    null
                }
            )

        val contacts = try {
            contactManager?.list().orEmpty()
        } catch (e: Exception) {
            Timber.d("Unable to update route hints for $libp2pPeerId: ${e.message}")
            return
        }

        contacts.forEach { contact ->
            val routing = parseRoutingHints(contact.notes)
            val isMatch = contact.peerId == libp2pPeerId ||
                routing.libp2pPeerId == libp2pPeerId ||
                (
                    normalizedTransportKey != null &&
                        normalizePublicKey(contact.publicKey) == normalizedTransportKey
                    )
            if (!isMatch) return@forEach

            val withPeerId = appendRoutingHint(contact.notes, "libp2p_peer_id", libp2pPeerId)
            val withListeners = upsertRoutingListeners(withPeerId, normalizedListeners)
            if (withListeners == contact.notes) return@forEach

            val updated = uniffi.api.Contact(
                peerId = contact.peerId,
                nickname = contact.nickname,
                localNickname = contact.localNickname,
                publicKey = contact.publicKey,
                addedAt = contact.addedAt,
                lastSeen = contact.lastSeen,
                notes = withListeners,
                lastKnownDeviceId = contact.lastKnownDeviceId,
                verifiedAt = contact.verifiedAt,
                isTombstone = false
            )
            try {
                contactManager?.add(updated)
            } catch (e: Exception) {
                Timber.d("Failed to persist route hints for ${contact.peerId}: ${e.message}")
            }
        }
    }

    /**
     * Upserts a federated contact with synchronization to prevent duplicate creation.
     *
     * This function is synchronized using contactUpsertMutex to prevent race conditions
     * where concurrent peer identification callbacks could create duplicate contacts
     * for the same peer (AND-CONTACT-DUP-001).
     *
     * The function:
     * 1. Acquires the mutex lock to ensure atomic contact lookup and creation
     * 2. Checks for existing contacts by both public key and peer ID
     * 3. Merges duplicates if found by both key and ID but with different peer IDs
     * 4. Creates or updates the contact with resolved identity information
     */
    private suspend fun upsertFederatedContact(
        canonicalPeerId: String,
        publicKey: String,
        nickname: String?,
        libp2pPeerId: String?,
        wifiPeerId: String? = null,
        listeners: List<String>,
        blePeerId: String? = null,
        deviceId: String? = null,
        createIfMissing: Boolean = true
    ) {
        contactUpsertMutex.withLock {
            val normalizedPeerId = canonicalPeerId.trim()
            val normalizedKey = normalizePublicKey(publicKey) ?: return@withLock
            if (normalizedPeerId.isEmpty()) return@withLock

            val routePeer = libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!routePeer.isNullOrBlank() && isBootstrapRelayPeer(routePeer)) return@withLock

            val contacts = try {
                contactManager?.list().orEmpty()
            } catch (e: Exception) {
                Timber.d("Failed to list contacts for federation upsert: ${e.message}")
                return@withLock
            }

            val existingByKey = contacts.firstOrNull { normalizePublicKey(it.publicKey) == normalizedKey }
            val existingById = contacts.firstOrNull { it.peerId == normalizedPeerId }

            // Auth guard: for an existing canonical peerId, only accept federated updates
            // when the source public key is consistent with the stored contact key.
            if (existingById != null && normalizePublicKey(existingById.publicKey) != normalizedKey) {
                Timber.w(
                    "Rejected federated nickname update for $normalizedPeerId: key mismatch " +
                        "(stored=${existingById.publicKey.take(8)}..., incoming=${normalizedKey.take(8)}...)"
                )
                return@withLock
            }

            // If we found them by key but they have a different PeerID now,
            // we should merge them to prevent duplicates.
            if (existingByKey != null && existingById != null && existingByKey.peerId != existingById.peerId) {
                Timber.i("Merging duplicate identities for key ${normalizedKey.take(8)}...: ${existingById.peerId} -> ${existingByKey.peerId}")
                try { contactManager?.remove(existingById.peerId) } catch (_: Exception) {}
            }

            val existing = existingByKey ?: existingById
            if (existing == null && !createIfMissing) {
                return@withLock
            }

            var notes = existing?.notes
            if (!routePeer.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "libp2p_peer_id", value = routePeer)
            }
            val normalizedBle = blePeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!normalizedBle.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "ble_peer_id", value = normalizedBle)
            }
            val normalizedWifi = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
            if (!normalizedWifi.isNullOrBlank()) {
                notes = appendRoutingHint(notes = notes, key = "wifi_peer_id", value = normalizedWifi)
            }
            notes = upsertRoutingListeners(notes, normalizeOutboundListenerHints(listeners))

            val now = (System.currentTimeMillis() / 1000).toULong()
            val resolvedPeerId = existing?.peerId ?: normalizedPeerId
            val resolvedPublicKey = existingByKey?.publicKey ?: normalizedKey
            val incomingNickname = normalizeNickname(nickname)
            val resolvedNickname = selectAuthoritativeNickname(incomingNickname, existing?.nickname)
            val resolvedLocalNickname = normalizeNickname(existing?.localNickname)

            val updated = uniffi.api.Contact(
                peerId = resolvedPeerId,
                nickname = resolvedNickname,
                localNickname = resolvedLocalNickname,
                publicKey = resolvedPublicKey,
                addedAt = existing?.addedAt ?: now,
                lastSeen = now,
                notes = notes,
                lastKnownDeviceId = deviceId?.trim()?.takeIf { it.isNotEmpty() } ?: existing?.lastKnownDeviceId,
                verifiedAt = existing?.verifiedAt,
                isTombstone = false
            )

            try {
                contactManager?.add(updated)
            } catch (e: Exception) {
                Timber.d("Failed to upsert federated contact for $resolvedPeerId: ${e.message}")
            }

            annotateIdentityInLedger(
                routePeerId = routePeer,
                listeners = listeners,
                publicKey = resolvedPublicKey,
                nickname = resolvedNickname
            )
        }
    }

    private fun upsertRoutingListeners(notes: String?, listeners: List<String>): String? {
        if (listeners.isEmpty()) return notes
        val base = notes?.trim().orEmpty()
        val filtered = base
            .split(';', '\n')
            .map { it.trim() }
            .filter { it.isNotEmpty() && !it.startsWith("listeners:") }
        return (filtered + "listeners:${listeners.joinToString(",")}").joinToString(";")
    }

    @Synchronized
    private fun removePendingOutbound(historyRecordId: String) {
        if (historyRecordId.isBlank()) return
        val queue = loadPendingOutbox().toMutableList()
        val removed = queue.removeAll { it.historyRecordId == historyRecordId }
        if (removed) savePendingOutbox(queue)
    }

    @Synchronized
    private fun promotePendingOutboundForPeer(peerId: String, excludingMessageId: String? = null) {
        val trimmedPeerId = peerId.trim()
        if (trimmedPeerId.isEmpty()) return
        val now = System.currentTimeMillis() / 1000
        val queue = loadPendingOutbox().toMutableList()
        var changed = false
        for (idx in queue.indices) {
            val item = queue[idx]
            val routePeerId = item.routePeerId?.trim()
            if (item.peerId != trimmedPeerId && routePeerId != trimmedPeerId) continue
            if (!excludingMessageId.isNullOrBlank() && item.historyRecordId == excludingMessageId) continue
            if (item.terminalFailureCode != null) continue
            if (item.nextAttemptAtEpochSec <= now) continue
            queue[idx] = item.copy(nextAttemptAtEpochSec = now)
            changed = true
        }
        if (!changed) return
        savePendingOutbox(queue)
        logDeliveryState(
            messageId = excludingMessageId ?: "unknown",
            state = "forwarding",
            detail = "peer_queue_promoted peer=$trimmedPeerId"
        )
    }

    private fun isMessageDeliveredLocally(messageId: String): Boolean {
        pruneDeliveredReceiptCache()
        if (deliveredReceiptCache.containsKey(messageId)) {
            return true
        }
        return try {
            historyManager?.get(messageId)?.delivered == true
        } catch (_: Exception) {
            false
        }
    }

    @Synchronized
    private fun markDeliveredReceiptSeen(messageId: String): Boolean {
        pruneDeliveredReceiptCache()
        val now = System.currentTimeMillis()
        return deliveredReceiptCache.putIfAbsent(messageId, now) == null
    }

    private fun pruneDeliveredReceiptCache(nowMs: Long = System.currentTimeMillis()) {
        deliveredReceiptCache.forEach { (messageId, seenAtMs) ->
            if (nowMs - seenAtMs > deliveredReceiptCacheTtlMs) {
                deliveredReceiptCache.remove(messageId, seenAtMs)
            }
        }
        if (deliveredReceiptCache.size <= 2048) {
            return
        }
        val keep = deliveredReceiptCache.entries
            .sortedByDescending { it.value }
            .take(1024)
            .associate { it.key to it.value }
        deliveredReceiptCache.clear()
        deliveredReceiptCache.putAll(keep)
    }

    private fun parseRoutingHints(notes: String?): RoutingHints {
        if (notes.isNullOrEmpty()) {
            return RoutingHints(
                wifiPeerId = null,
                blePeerId = null,
                libp2pPeerId = null,
                listeners = emptyList()
            )
        }

        var wifiPeerId: String? = null
        var blePeerId: String? = null
        var peerId: String? = null
        var listeners: List<String> = emptyList()

        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (kv.startsWith("wifi_peer_id:")) {
                val value = kv.removePrefix("wifi_peer_id:").trim()
                wifiPeerId = value.ifEmpty { null }
            } else if (kv.startsWith("ble_peer_id:")) {
                val value = kv.removePrefix("ble_peer_id:").trim()
                blePeerId = value.ifEmpty { null }
            } else if (kv.startsWith("libp2p_peer_id:")) {
                val value = kv.removePrefix("libp2p_peer_id:").trim()
                peerId = value.ifEmpty { null }
            } else if (kv.startsWith("listeners:")) {
                val value = kv.removePrefix("listeners:").trim()
                if (value.isNotEmpty()) {
                    listeners = value
                        .split(",")
                        .map { it.trim() }
                        .filter { it.isNotEmpty() }
                }
            }
        }

        return RoutingHints(
            wifiPeerId = wifiPeerId,
            blePeerId = blePeerId,
            libp2pPeerId = peerId,
            listeners = listeners
        )
    }

    private fun parseAllRoutingPeerIds(notes: String?): List<String> {
        if (notes.isNullOrBlank()) return emptyList()
        val out = mutableListOf<String>()
        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (!kv.startsWith("libp2p_peer_id:")) continue
            val value = kv.removePrefix("libp2p_peer_id:").trim()
            if (value.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(value)) {
                out.add(value)
            }
        }
        return out.distinct()
    }

    /**
     * AND-NO-ROUTE-001: Extract the last-known-good route peer ID stored in contact notes.
     * This is used as a fallback when fresh discovery returns no candidates.
     */
    private fun parseLastKnownRoute(notes: String?): String? {
        if (notes.isNullOrBlank()) return null
        for (component in notes.split(';', '\n')) {
            val kv = component.trim()
            if (!kv.startsWith("last_known_route:")) continue
            val value = kv.removePrefix("last_known_route:").trim()
            if (value.isNotEmpty()) return value
        }
        return null
    }

    private fun buildRoutePeerCandidates(
        peerId: String,
        cachedRoutePeerId: String?,
        notes: String?,
        recipientPublicKey: String? = null
    ): List<String> {
        val candidates = mutableListOf<String>()

        // Diagnostic tracking: why each source contributed or failed
        val diagSources = mutableListOf<String>()

        val discovered = discoverRoutePeersForPublicKey(recipientPublicKey)
        if (discovered.isEmpty()) {
            diagSources.add("discovery=empty")
        } else {
            diagSources.add("discovery=${discovered.size}")
        }
        candidates.addAll(discovered)

        val notedPeerIds = parseAllRoutingPeerIds(notes)
        if (notedPeerIds.isEmpty()) {
            diagSources.add("notes_routing_hints=empty")
        } else {
            diagSources.add("notes_routing_hints=${notedPeerIds.size}")
        }
        val newestHint = notedPeerIds.lastOrNull()
        if (!newestHint.isNullOrBlank()) candidates.add(newestHint)
        for (hint in notedPeerIds.asReversed()) candidates.add(hint)

        val trimmedCached = cachedRoutePeerId?.trim()?.takeIf { it.isNotEmpty() }
        if (trimmedCached == null) {
            diagSources.add("cached_route_peer_id=null")
        } else if (!PeerIdValidator.isLibp2pPeerId(trimmedCached)) {
            diagSources.add("cached_route_peer_id=invalid_format")
        } else {
            diagSources.add("cached_route_peer_id=valid")
        }
        trimmedCached?.let { candidates.add(it) }

        if (PeerIdValidator.isLibp2pPeerId(peerId)) {
            diagSources.add("peer_id=valid")
            candidates.add(peerId)
        } else {
            diagSources.add("peer_id=invalid_format")
        }

        val filtered = candidates
            .map { it.trim() }
            .filter { candidate ->
                candidate.isNotEmpty() &&
                    PeerIdValidator.isLibp2pPeerId(candidate) &&
                    routeCandidateMatchesRecipient(candidate, recipientPublicKey)
            }
            .distinct()

        if (filtered.isEmpty()) {
            // AND-NO-ROUTE-001: Fallback to last-known-good route stored in contact notes
            val lastKnownRoute = parseLastKnownRoute(notes)
            if (lastKnownRoute != null && PeerIdValidator.isLibp2pPeerId(lastKnownRoute)) {
                Timber.w("buildRoutePeerCandidates: falling back to last_known_route=$lastKnownRoute for peerId=${peerId.take(12)}")
                return listOf(lastKnownRoute)
            }
            Timber.w("buildRoutePeerCandidates: no valid candidates after filtering [${diagSources.joinToString(", ")}] peerId=${peerId.take(12)} recipientKey=${recipientPublicKey?.take(8) ?: "null"} notesLen=${notes?.length ?: 0}")
        } else {
            Timber.d("buildRoutePeerCandidates: ${filtered.size} candidates [${diagSources.joinToString(", ")}]")
        }

        return filtered
    }

    private fun discoverRoutePeersForPublicKey(recipientPublicKey: String?): List<String> {
        val normalizedRecipientKey = normalizePublicKey(recipientPublicKey) ?: return emptyList()
        val fromDiscovery = _discoveredPeers.value.values
            .asSequence()
            .filter { info ->
                normalizePublicKey(info.publicKey) == normalizedRecipientKey
            }
            .mapNotNull { info ->
                // Prefer libp2pPeerId for routing if available
                val routeId = info.libp2pPeerId?.trim()?.takeIf { 
                    it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it)
                } ?: info.peerId.trim().takeIf { 
                    it.isNotEmpty() && PeerIdValidator.isLibp2pPeerId(it) 
                }
                routeId
            }
            .toList()

        val fromLedger = (ledgerManager?.dialableAddresses() ?: emptyList())
            .asSequence()
            .mapNotNull { entry ->
                val candidate = entry.peerId?.trim().orEmpty()
                if (candidate.isEmpty() || !PeerIdValidator.isLibp2pPeerId(candidate)) {
                    return@mapNotNull null
                }
                val candidateKey = normalizePublicKey(entry.publicKey) ?: return@mapNotNull null
                if (candidateKey != normalizedRecipientKey) return@mapNotNull null
                candidate
            }
            .toList()

        return (fromDiscovery + fromLedger).distinct()
    }

    private fun routeCandidateMatchesRecipient(
        routePeerId: String,
        recipientPublicKey: String?
    ): Boolean {
        val normalizedRoute = routePeerId.trim()
        if (normalizedRoute.isEmpty() || !PeerIdValidator.isLibp2pPeerId(normalizedRoute)) return false
        if (isKnownRelay(normalizedRoute)) return false

        val normalizedRecipientKey = normalizePublicKey(recipientPublicKey) ?: return true
        val extractedKey = try {
            ironCore?.extractPublicKeyFromPeerId(normalizedRoute)
        } catch (_: Exception) {
            null
        }
        val normalizedExtractedKey = normalizePublicKey(extractedKey)
        if (normalizedExtractedKey != null) {
            return normalizedExtractedKey == normalizedRecipientKey
        }

        val discoveryMatch = _discoveredPeers.value.entries.any { (key, info) ->
            (key == normalizedRoute || info.peerId == normalizedRoute) &&
                normalizePublicKey(info.publicKey) == normalizedRecipientKey
        }
        if (discoveryMatch) return true

        val ledgerMatch = (ledgerManager?.dialableAddresses() ?: emptyList()).any { entry ->
            entry.peerId?.trim() == normalizedRoute &&
                normalizePublicKey(entry.publicKey) == normalizedRecipientKey
        }
        return ledgerMatch
    }

    // Identity validation logic centralized in PeerIdValidator

    private fun buildDialCandidatesForPeer(
        routePeerId: String?,
        rawAddresses: List<String>,
        includeRelayCircuits: Boolean
    ): List<String> {
        val normalized = rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
        val prioritized = prioritizeAddressesForCurrentNetwork(normalized)
        val relayCircuits = if (includeRelayCircuits && !routePeerId.isNullOrBlank() && PeerIdValidator.isLibp2pPeerId(routePeerId)) {
            relayCircuitAddressesForPeer(routePeerId)
        } else {
            emptyList()
        }
        // Cap at 6 candidates to avoid excessive dialing.
        // Priority: LAN addresses first (from prioritized), then relay circuits,
        // then remaining public addresses.
        return (prioritized + relayCircuits).distinct().take(6)
    }

    fun getDialHintsForRoutePeer(routePeerId: String): List<String> {
        if (!PeerIdValidator.isLibp2pPeerId(routePeerId)) return emptyList()
        val fromLedger = (ledgerManager?.dialableAddresses() ?: emptyList())
            .filter { it.peerId == routePeerId }
            .map { it.multiaddr }
        return buildDialCandidatesForPeer(
            routePeerId = routePeerId,
            rawAddresses = fromLedger,
            includeRelayCircuits = false // Avoid infinite recursion
        )
    }

    private fun normalizeOutboundListenerHints(rawAddresses: List<String>): List<String> {
        return rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
    }

    private fun normalizeExternalAddressHints(rawAddresses: List<String>): List<String> {
        return rawAddresses
            .mapNotNull { normalizeAddressHint(it) }
            .distinct()
    }

    private fun normalizeAddressHint(raw: String): String? {
        val trimmed = raw.trim()
        if (trimmed.isEmpty()) return null

        val normalizedZeroAddr = if (trimmed.contains("/ip4/0.0.0.0/")) {
            val localIp = getLocalIpAddress() ?: return null
            trimmed.replace("/ip4/0.0.0.0/", "/ip4/$localIp/")
        } else {
            trimmed
        }

        val asMultiaddr = if (normalizedZeroAddr.startsWith("/")) {
            normalizedZeroAddr
        } else {
            toMultiaddrFromSocketAddress(normalizedZeroAddr) ?: return null
        }

        if (!isDialableAddress(asMultiaddr)) return null
        return asMultiaddr
    }

    private fun toMultiaddrFromSocketAddress(value: String): String? {
        val trimmed = value.trim()
        if (trimmed.isEmpty()) return null
        if (trimmed.startsWith("/")) return trimmed

        val separatorIdx = trimmed.lastIndexOf(':')
        if (separatorIdx <= 0 || separatorIdx >= trimmed.lastIndex) return null

        val host = trimmed.substring(0, separatorIdx).trim().removePrefix("[").removeSuffix("]")
        val port = trimmed.substring(separatorIdx + 1).toIntOrNull() ?: return null
        if (port !in 1..65535 || host.isEmpty()) return null

        val ipv4Regex = Regex("^\\d{1,3}(\\.\\d{1,3}){3}$")
        return when {
            host.contains(":") -> "/ip6/$host/tcp/$port"
            ipv4Regex.matches(host) -> "/ip4/$host/tcp/$port"
            else -> "/dns4/$host/tcp/$port"
        }
    }

    /**
     * SECURITY: default-deny address admission.
     *
     * This gate gained a remotely-reachable caller when contact-import and
     * deep-link payloads began carrying a `listeners` array, so an untrusted
     * QR code or URL can now propose dial targets. It previously returned
     * `true` for anything that was not literal `/ip4/`, which admitted every
     * `/ip6/`, `/dns4/`, `/dns6/`, `/dnsaddr/` and `/dns/` form -- reaching
     * loopback, link-local, ULA and NAT64-mapped metadata addresses, and
     * allowing DNS rebinding. Unknown forms are now rejected rather than
     * allowed.
     */
    private fun isDialableAddress(multiaddr: String): Boolean {
        // DNS forms are never accepted from import-sourced hints: the name can
        // be re-pointed after validation (rebinding). The Rust seed path
        // enforces the same rule via DnsPolicy::Reject.
        if (multiaddr.contains("/dns")) return false

        val ipv4 = extractIpv4FromMultiaddr(multiaddr)
        if (ipv4 != null) {
            if (isSpecialUseIpv4(ipv4)) return false
            val ipv4Ok = if (isPrivateIpv4(ipv4)) isSameLanAddress(multiaddr) else true
            if (!ipv4Ok) return false
            // A circuit address carries a real socket to the relay hop, which
            // is the leading IP just validated above.
            return true
        }

        val ipv6 = extractIpv6FromMultiaddr(multiaddr)
        if (ipv6 != null) {
            return !isRestrictedIpv6(ipv6)
        }

        // Circuit address with no resolvable leading IP, or an unrecognised
        // protocol stack. Default deny.
        return false
    }

    /** Extract the address literal from a `/ip6/<addr>/...` multiaddr. */
    private fun extractIpv6FromMultiaddr(multiaddr: String): String? {
        val marker = "/ip6/"
        val start = multiaddr.indexOf(marker)
        if (start == -1) return null
        val rest = multiaddr.substring(start + marker.length)
        val end = rest.indexOf('/')
        val literal = if (end == -1) rest else rest.substring(0, end)
        return literal.takeIf { it.isNotEmpty() }
    }

    /**
     * IPv6 equivalents of [isSpecialUseIpv4]. Unique-local (fc00::/7) is
     * rejected outright here rather than LAN-gated: there is no reliable
     * same-subnet test for ULA on Android without the interface prefix, and
     * admitting it would reopen the internal-probe vector.
     */
    private fun isRestrictedIpv6(ip: String): Boolean {
        val v = ip.lowercase().trim()
        if (v.isEmpty()) return true
        if (v == "::" || v == "::1") return true // unspecified, loopback
        // Link-local fe80::/10 spans fe80..febf.
        if (v.startsWith("fe8") || v.startsWith("fe9") ||
            v.startsWith("fea") || v.startsWith("feb")
        ) {
            return true
        }
        if (v.startsWith("ff")) return true // multicast ff00::/8
        if (v.startsWith("fc") || v.startsWith("fd")) return true // ULA fc00::/7
        // IPv4-mapped/compatible and NAT64 (64:ff9b::/96) can smuggle a
        // restricted IPv4 target through the IPv6 path.
        if (v.startsWith("::ffff:") || v.startsWith("64:ff9b:")) return true
        if (v.startsWith("2002:")) return true // 6to4, embeds an IPv4 target
        return false
    }

    private fun parseIpv4Octets(ip: String): List<Int>? {
        val octets = ip.split('.').mapNotNull { it.toIntOrNull() }
        if (octets.size != 4) return null
        if (octets.any { it !in 0..255 }) return null
        return octets
    }

    private fun isPrivateIpv4(ip: String): Boolean {
        val octets = parseIpv4Octets(ip) ?: return false
        return (octets[0] == 10) ||
            (octets[0] == 172 && octets[1] in 16..31) ||
            (octets[0] == 192 && octets[1] == 168)
    }

    private fun isSpecialUseIpv4(ip: String): Boolean {
        val octets = parseIpv4Octets(ip) ?: return true
        val o0 = octets[0]
        val o1 = octets[1]
        val o2 = octets[2]

        if (o0 == 0 || o0 == 127) return true
        if (o0 == 169 && o1 == 254) return true
        if (o0 == 100 && o1 in 64..127) return true // RFC6598 CGNAT
        if (o0 == 192 && o1 == 0 && (o2 == 0 || o2 == 2)) return true
        if (o0 == 198 && ((o1 == 18) || (o1 == 19))) return true // Benchmark network
        if (o0 == 198 && o1 == 51 && o2 == 100) return true
        if (o0 == 203 && o1 == 0 && o2 == 113) return true
        if (o0 >= 224) return true // multicast/reserved/broadcast
        return false
    }

    fun isKnownRelay(peerId: String): Boolean {
        val normalized = peerId.trim()
        if (isBootstrapRelayPeer(normalized)) return true
        val info = _discoveredPeers.value.entries.firstOrNull {
            it.key.equals(normalized, ignoreCase = true)
        }?.value ?: return false
        return info.isRelay && !info.isFull
    }

    private fun relayCircuitAddressesForPeer(targetPeerId: String): List<String> {
        if (!PeerIdValidator.isLibp2pPeerId(targetPeerId)) return emptyList()
        val circuits = mutableListOf<String>()

        // Use getHealthyRelays to pre-filter relays with closed (healthy) circuits
        val healthyRelayAddrs = relayCircuitBreaker.getHealthyRelays().toSet()

        // 1. Static Bootstrap Relays (prioritized by network type)
        val prioritizedNodes = emptyList<String>()

        prioritizedNodes.forEach { bootstrap ->
            val relayInfo = parseBootstrapRelay(bootstrap)
            if (relayInfo != null) {
                val (relayTransportAddr, relayPeerId) = relayInfo
                // Skip circuit addresses for relays with open circuit breakers
                if (relayCircuitBreaker.isCircuitOpen(bootstrap)) return@forEach
                // Prioritize relays confirmed healthy by circuit breaker
                if (bootstrap !in healthyRelayAddrs && relayCircuitBreaker.getFailureCount(bootstrap) > 0) {
                    Timber.d("Skipping unhealthy relay: $bootstrap")
                    return@forEach
                }
                circuits.add("$relayTransportAddr/p2p/$relayPeerId/p2p-circuit/p2p/$targetPeerId")
            }
        }

        // 2. Dynamic Discovered Relays
        _discoveredPeers.value.entries.filter {
            it.value.isRelay && !it.value.isFull && it.key != targetPeerId
        }.forEach { entry ->
            val relayPeerId = entry.key
            if (PeerIdValidator.isLibp2pPeerId(relayPeerId)) {
                val directAddrs = getDialHintsForRoutePeer(relayPeerId)
                directAddrs.forEach { addr ->
                    val circuit = "$addr/p2p/$relayPeerId/p2p-circuit/p2p/$targetPeerId"
                    if (!circuits.contains(circuit)) circuits.add(circuit)
                }
            }
        }

        return circuits
    }

    private fun parseBootstrapRelay(multiaddr: String): Pair<String, String>? {
        val marker = "/p2p/"
        val idx = multiaddr.lastIndexOf(marker)
        if (idx <= 0) return null
        val transportAddr = multiaddr.substring(0, idx).trimEnd('/')
        val relayPeerId = multiaddr.substring(idx + marker.length).trim()
        if (transportAddr.isEmpty() || relayPeerId.isEmpty()) return null
        return transportAddr to relayPeerId
    }

    /**
     * Lazily-built set of known relay peer IDs extracted from circuit breaker
     * addresses and discovered peer entries.  Rebuilt on each call to pick up
     * newly-discovered relays without requiring a manual refresh.
     */
    private fun collectKnownRelayPeerIds(): Set<String> {
        val ids = mutableSetOf<String>()
        // 1. Relay peer IDs seen by the circuit breaker (multiaddr strings like
        //    "/ip4/.../tcp/.../p2p/12D3KooW…").  Covers both healthy (closed)
        //    and failed (open) relays so we don't miss relays merely because
        //    they just went down.
        val relayAddrs = relayCircuitBreaker.getHealthyRelays() +
            relayCircuitBreaker.getOpenCircuits()
        relayAddrs.forEach { addr ->
            parseBootstrapRelay(addr)?.second?.let { ids.add(it) }
        }
        // 2. Peers the discovery layer has already identified as relays.
        _discoveredPeers.value.values
            .filter { it.isRelay }
            .forEach { info ->
                ids.add(info.peerId)
                info.libp2pPeerId?.let { ids.add(it) }
            }
        return ids
    }

    fun isBootstrapRelayPeer(peerId: String): Boolean {
        if (peerId.isBlank()) return false
        val knownRelays = collectKnownRelayPeerIds()
        if (peerId in knownRelays) return true
        // Also check with /p2p/ prefix in case callers include it
        val stripped = peerId.substringAfterLast("/p2p/")
        if (stripped != peerId && stripped in knownRelays) return true
        return false
    }

    /**
     * Check if a public key corresponds to a bootstrap relay peer.
     * This is used when resolving transport identities to determine if
     * we should create an emergency contact or treat as a transient relay.
     */
    fun isBootstrapRelayPeerFromKey(normalizedKey: String): Boolean {
        if (normalizedKey.isBlank()) return false
        // Convert public key → peer ID using the same routine the rest of
        // the codebase relies on, then delegate to the main check.
        val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(normalizedKey)
        return isBootstrapRelayPeer(peerId)
    }

    /**
     * P0_ANDROID_004: Create an emergency contact for a newly discovered peer.
     *
     * This function is called when a peer is identified but no existing contact
     * is found in the database. It creates a minimal contact entry that can be
     * later enriched through federated updates.
     *
     * @param publicKey The normalized public key of the peer
     * @return The newly created Contact
     */
    private fun createEmergencyContact(publicKey: String): uniffi.api.Contact {
        // Extract peer ID from public key, or generate a fallback
        val peerId = PeerKeyUtils.extractPeerIdFromPublicKey(publicKey)
        val normalizedPeerId = PeerIdValidator.normalize(peerId)

        // Create minimal contact with discovered state
        val contact = uniffi.api.Contact(
            peerId = normalizedPeerId,
            nickname = "Unknown Peer",
            localNickname = null,
            publicKey = publicKey,
            addedAt = (System.currentTimeMillis() / 1000).toULong(),
            lastSeen = (System.currentTimeMillis() / 1000).toULong(),
            notes = "Emergency contact created during peer discovery",
            lastKnownDeviceId = null,
            verifiedAt = null,
            isTombstone = false
        )

        // Save to database
        try {
            contactManager?.add(contact)
            Timber.i("Emergency contact created for peer: ${normalizedPeerId.take(8)}... (key: ${publicKey.take(8)}...)")
        } catch (e: Exception) {
            Timber.e("Failed to create emergency contact: ${e.message}")
            // Continue anyway - contact may already exist
        }

        return contact
    }

    /**
     * P0_ANDROID_004: Validate peer before contact creation.
     *
     * Comprehensive peer validation to ensure we only create contacts for
     * valid, non-relay peers.
     *
     * @param peerId The peer ID to validate
     * @param publicKey The public key to validate
     * @return true if the peer is valid for contact creation
     */
    private fun validatePeerBeforeContactCreation(peerId: String, publicKey: String): Boolean {
        return when {
            peerId.isBlank() -> {
                Timber.w("Peer validation failed: blank peer ID")
                false
            }
            publicKey.isBlank() -> {
                Timber.w("Peer validation failed: blank public key")
                false
            }
            !PeerKeyUtils.isValidPeerId(peerId) -> {
                Timber.w("Peer validation failed: invalid peer ID format: $peerId")
                false
            }
            !PeerKeyUtils.isValidPublicKey(publicKey) -> {
                Timber.w("Peer validation failed: invalid public key format: ${publicKey.take(8)}...")
                false
            }
            isBootstrapRelayPeer(peerId) -> {
                Timber.d("Peer validation: skipping relay peer")
                false
            }
            else -> true
        }
    }

    /**
     * P0_ANDROID_004: Enhanced logging for identity resolution.
     *
     * Logs detailed information about identity resolution for debugging
     * and monitoring purposes.
     */
    private fun logIdentityResolutionDetails(
        normalizedKey: String,
        canonicalContact: uniffi.api.Contact?,
        createdNew: Boolean
    ) {
        Timber.d(
            "Identity resolution: key=${normalizedKey.take(8)}..., " +
                "contact=${canonicalContact?.peerId?.take(8)}..., " +
                "createdNew=$createdNew"
        )
    }

    /**
     * P0_NETWORK_001: Bootstrap relay connections with circuit breaker and
     * WebSocket fallback for cellular networks.
     *
     * Priority order respects network type:
     * - WiFi/Ethernet: QUIC → TCP → WebSocket (full connectivity)
     * - Cellular: WebSocket(443) → TCP(443) → QUIC → TCP → WebSocket(80)
     */
    private suspend fun primeRelayBootstrapConnections() {
        val bridge = swarmBridge ?: return
        val nowMs = System.currentTimeMillis()

        // P1_ANDROID_013: Respect exponential backoff from consecutive failures
        if (nowMs < nextBootstrapAttemptMs) {
            return
        }
        if (nowMs - lastRelayBootstrapDialMs < 10_000L) return
        lastRelayBootstrapDialMs = nowMs

        // Determine transport priority based on current network
        val transportPriority = networkDetector.getTransportPriority()
        val isCellular = networkDetector.isCellularNetwork
        Timber.i("Bootstrap: network=%s, cellular=%b, priority=%s",
            networkDetector.networkType.value, isCellular, transportPriority)

        // Build address list: primary nodes + WebSocket fallback if cellular
        val addresses = emptyList<String>()

        var anySuccess = false
        for (addr in addresses) {
            try {
                // Check circuit breaker before attempting
                if (!relayCircuitBreaker.allowRequest(addr)) {
                    Timber.d("Circuit breaker blocked %s, skipping", addr)
                    continue
                }
                if (!shouldAttemptDial(addr)) continue
                bridge.dial(addr)
                Timber.d("Bootstrap dial initiated: %s", addr)
                anySuccess = true
            } catch (e: Exception) {
                // P1_ANDROID_013: Record failure metrics directly without triggering
                // enhanceNetworkErrorLogging for each failure. The fallback protocol
                // is now handled by the racing bootstrap, not per-dial error logging.
                val errorDetail = classifyBootstrapError(e, addr)
                Timber.w("Bootstrap dial failed for $addr - $errorDetail")
                relayCircuitBreaker.recordFailure(addr, errorDetail)
                networkFailureMetrics.recordFailure(addr, errorDetail, e)
                recordConnectionFailure(addr)
            }
        }

        // P1_ANDROID_013: Update consecutive failure tracking and backoff
        if (!anySuccess) {
            consecutiveBootstrapFailures++
            val backoffMs = when {
                consecutiveBootstrapFailures <= 1 -> 10_000L
                consecutiveBootstrapFailures <= 3 -> 30_000L
                else -> 60_000L.coerceAtMost(1000L * (1L shl consecutiveBootstrapFailures.coerceAtMost(6)))
            }
            nextBootstrapAttemptMs = nowMs + backoffMs
            Timber.w("Bootstrap all-failed (consecutive=%d), next attempt in %dms",
                consecutiveBootstrapFailures, backoffMs)
        } else {
            consecutiveBootstrapFailures = 0
            nextBootstrapAttemptMs = 0L
        }
    }

    /** @deprecated Use the suspend version with circuit breaker support */
    fun primeRelayBootstrapConnectionsLegacy() {
        val bridge = swarmBridge ?: return
        val nowMs = System.currentTimeMillis()
        if (nowMs - lastRelayBootstrapDialMs < 10_000L) return
        lastRelayBootstrapDialMs = nowMs

        emptyList<String>().forEach { addr ->
            if (!shouldAttemptDial(addr)) return@forEach
            repoScope.launch {
                try {
                    bridge.dial(addr)
                } catch (e: Exception) {
                    Timber.d("Relay bootstrap dial skipped for $addr: ${e.message}")
                }
            }
        }
    }

    private fun prioritizeAddressesForCurrentNetwork(addresses: List<String>): List<String> {
        if (addresses.size <= 1) return addresses
        val lan = addresses.filter { isSameLanAddress(it) }
        if (lan.isEmpty()) return addresses
        return (lan + addresses.filterNot { it in lan }).distinct()
    }

    private fun isSameLanAddress(multiaddr: String): Boolean {
        val targetIp = extractIpv4FromMultiaddr(multiaddr) ?: return false
        val localIp = getLocalIpAddress() ?: return false
        return sameSubnet24(localIp, targetIp)
    }

    private fun extractIpv4FromMultiaddr(multiaddr: String): String? {
        val marker = "/ip4/"
        val start = multiaddr.indexOf(marker)
        if (start < 0) return null
        val rest = multiaddr.substring(start + marker.length)
        val end = rest.indexOf('/')
        return if (end >= 0) rest.substring(0, end) else rest
    }

    private fun sameSubnet24(ipA: String, ipB: String): Boolean {
        val a = ipA.split(".")
        val b = ipB.split(".")
        if (a.size != 4 || b.size != 4) return false
        return a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
    }

    /**
     * P0_NETWORK_001: Racing bootstrap with fallback strategy.
     *
     * Races all transport multiaddrs for each relay in parallel within a
     * 500ms connectivity window. On cellular networks, WebSocket on standard
     * ports (80/443) is prioritized since carriers commonly block non-standard
     * ports like 9001/9010. Circuit breakers gate each address before dialing.
     * If all relay bootstrap fails, falls back to mDNS local discovery.
     */
    internal suspend fun racingBootstrapWithFallback(): BootstrapResult {
        val transportPriority = networkDetector.getTransportPriority()
        val diagnostics = networkDetector.getNetworkDiagnostics()
        Timber.i("Racing bootstrap: network=%s, transports=%s", diagnostics.networkType,
            transportPriority.joinToString("→") { it.scheme })

        // Reset circuit breakers on network change — but only when transitioning
        // TO a healthy network (WiFi) FROM a restrictive one (cellular).
        // Resetting on every WIFI↔CELLULAR flap destroys circuit state and
        // causes aggressive re-bootstrapping + mDNS spam.
        val isRecoveryTransition = diagnostics.networkType == com.scmessenger.android.transport.NetworkType.WIFI ||
            diagnostics.networkType == com.scmessenger.android.transport.NetworkType.ETHERNET
        if (isRecoveryTransition && relayCircuitBreaker.getOpenCircuits().isNotEmpty()) {
            Timber.i("Network recovered to %s — resetting %d open circuits",
                diagnostics.networkType, relayCircuitBreaker.getOpenCircuits().size)
            relayCircuitBreaker.resetAll()
        }

        // Build prioritized address list from the ledger (v0.4.0: no dedicated
        // relays, no hardcoded node addresses -- discovery is ledger sharing).
        // A fresh install has an empty ledger and legitimately has no
        // candidates here; that falls straight through to the mDNS fallback
        // below, which is the intended cold-start path, not a bug.
        val prioritizedAddresses = (ledgerManager?.getPreferredRelays(5u) ?: emptyList())
            .map { it.multiaddr }
        if (prioritizedAddresses.isEmpty()) {
            Timber.i("Racing bootstrap: no known relays in ledger yet, going straight to mDNS fallback")
        }

        // Proactively probe known relay ports to deprioritize blocked addresses
        val probeTargets = emptyList<Pair<String, Int>>()
        val portProbeResults = networkDetector.probePorts(probeTargets)

        // Filter out circuit-breaker-blocked and throttle-blocked addresses,
        // and deprioritize addresses whose host:port is confirmed blocked
        val candidateAddresses = prioritizedAddresses.filter { addr ->
            relayCircuitBreaker.allowRequest(addr) && shouldAttemptDial(addr)
        }.sortedByDescending { addr ->
            // Boost priority for addresses whose ports are confirmed reachable
            // Deprioritize addresses with ports likely blocked by current network (isPortLikelyBlocked)
            val port = extractPortFromMultiaddr(addr)
            val host = Regex("""/ip4/([^/]+)|/dns4/([^/]+)""").find(addr)?.groupValues?.let {
                if (it[1].isNotEmpty()) it[1] else (it[2] ?: "")
            } ?: ""
            val key = "$host:$port"
            val portReachable = portProbeResults[key] != false
            val portNotBlocked = port == null || !networkDetector.isPortLikelyBlocked(port)
            portReachable && portNotBlocked
        }

        if (candidateAddresses.isEmpty()) {
            Timber.w("No candidate addresses available (all circuit-breaker-blocked or throttled)")
            return attemptMdnsFallback()
        }

        // Race all candidate addresses in parallel with 3s timeout
        // (Individual dials may take longer; first success wins, others are cancelled)
        val result = kotlinx.coroutines.withTimeoutOrNull(3_000L) {
            kotlinx.coroutines.coroutineScope {
                val deferreds = candidateAddresses.map { addr ->
                    async(Dispatchers.IO) {
                        try {
                            val bridge = swarmBridge ?: return@async BootstrapAttempt.Failure(addr, "no bridge")
                            bridge.dial(addr)
                            relayCircuitBreaker.recordSuccess(addr)
                            Timber.i("Bootstrap connected: %s", addr)
                            BootstrapAttempt.Success(addr)
                        } catch (e: Exception) {
                            // P1_ANDROID_013: Record failure metrics directly without triggering
                            // enhanceNetworkErrorLogging, which would cascade into fallback protocol
                            val detail = classifyBootstrapError(e, addr)
                            relayCircuitBreaker.recordFailure(addr, detail)
                            networkFailureMetrics.recordFailure(addr, detail, e)
                            recordConnectionFailure(addr)
                            Timber.d("Bootstrap race attempt failed for $addr: $detail")
                            BootstrapAttempt.Failure(addr, e.message ?: "unknown")
                        }
                    }
                }

                // Take first success, or all failures
                for (deferred in deferreds) {
                    val attempt = deferred.await()
                    if (attempt is BootstrapAttempt.Success) {
                        // Cancel remaining coroutines
                        deferreds.forEach { if (!it.isCompleted) it.cancel() }
                        return@coroutineScope attempt
                    }
                }
                BootstrapAttempt.Failure("all", "all bootstrap attempts failed")
            }
        }

        return when (result) {
            is BootstrapAttempt.Success -> BootstrapResult.Connected(result.addr, transportPriority.firstOrNull())
            else -> {
                Timber.w("All bootstrap addresses failed or timed out, falling back to mDNS")
                attemptMdnsFallback()
            }
        }
    }

    /**
     * P0_NETWORK_001: mDNS fallback when all relay bootstrap fails.
     */
    private suspend fun attemptMdnsFallback(): BootstrapResult {
        Timber.i("All relay bootstrap failed, attempting mDNS local discovery")
        // mDNS discovery is handled by MdnsServiceDiscovery in the transport layer.
        // The service should already be running; wait up to 5s for a peer to resolve.
        val mdnsResult = kotlinx.coroutines.withTimeoutOrNull(5_000L) {
            // Wait for the mDNS service to discover a LAN peer
            // The onLanPeerResolved callback will trigger a SwarmBridge dial
            // via the TransportManager, so we just need to confirm connectivity
            var checks = 0
            while (checks < 10) {
                val peerCount = meshService?.getStats()?.peersDiscovered?.toInt() ?: 0
                if (peerCount > 0) {
                    Timber.i("mDNS fallback: connected to %d LAN peer(s)", peerCount)
                    return@withTimeoutOrNull BootstrapResult.MdnsFallback("lan-peer")
                }
                kotlinx.coroutines.delay(500)
                checks++
            }
            null
        }

        return if (mdnsResult != null) {
            mdnsResult
        } else {
            // Fall back to legacy bootstrap as last resort before declaring complete failure
            primeRelayBootstrapConnectionsLegacy()
            Timber.e("mDNS fallback: no LAN peers discovered within timeout, legacy bootstrap attempted")
            BootstrapResult.AllRelaysFailed
        }
    }

    /** Racing bootstrap attempt result */
    private sealed class BootstrapAttempt {
        data class Success(val addr: String) : BootstrapAttempt()
        data class Failure(val addr: String, val reason: String) : BootstrapAttempt()
    }

    /** Public bootstrap result for callers */
    sealed class BootstrapResult {
        data class Connected(val multiaddr: String, val transport: com.scmessenger.android.transport.FallbackTransport?) : BootstrapResult()
        data class MdnsFallback(val peerId: String) : BootstrapResult()
        data object AllRelaysFailed : BootstrapResult()
    }

    /**
     * Legacy sequential bootstrap — kept as fallback if racing is unavailable.
     * Delegates to racingBootstrapWithFallback() which subsumes this logic.
     */
    internal suspend fun bootstrapWithFallbackStrategy() {
        when (val result = racingBootstrapWithFallback()) {
            is BootstrapResult.Connected -> {
                Timber.i("Bootstrap succeeded: %s (transport: %s)", result.multiaddr, result.transport)
            }
            is BootstrapResult.MdnsFallback -> {
                Timber.i("Bootstrap fell back to mDNS: %s", result.peerId)
            }
            is BootstrapResult.AllRelaysFailed -> {
                Timber.e("All bootstrap methods failed (relay + mDNS)")
            }
        }
    }

    // P0_NETWORK_001: Watch for network type changes to trigger re-bootstrap
    private var networkWatchJob: kotlinx.coroutines.Job? = null
    // Exponential backoff cooldown: each consecutive flap doubles the cooldown
    // up to a 10-minute cap. Resets after a stable period without changes.
    private var lastNetworkChangeMs: Long = 0L
    private var networkFlapCount: Int = 0
    private val networkChangeCooldownBaseMs: Long = 30_000L
    private val networkChangeCooldownMaxMs: Long = 600_000L
    private val networkChangeStableResetMs: Long = 120_000L

    private fun startNetworkChangeWatch() {
        networkWatchJob?.cancel()
        networkWatchJob = repoScope.launch {
            var previousType = networkDetector.networkType.value
            networkDetector.networkType.collect { newType ->
                if (newType != previousType && newType != com.scmessenger.android.transport.NetworkType.UNKNOWN) {
                    val now = System.currentTimeMillis()
                    val elapsedSinceStable = now - lastNetworkChangeMs

                    // If the last change was long ago (>2min), reset the flap counter.
                    if (elapsedSinceStable >= networkChangeStableResetMs) {
                        networkFlapCount = 0
                    }

                    val cooldownMs = minOf(
                        networkChangeCooldownBaseMs * (1L shl networkFlapCount.coerceAtMost(5)),
                        networkChangeCooldownMaxMs
                    )

                    if (elapsedSinceStable < cooldownMs && networkFlapCount > 0) {
                        Timber.w("Network change throttled: %s → %s (cooldown=%dms, flapping=%d)",
                            previousType, newType, cooldownMs, networkFlapCount)
                        previousType = newType
                        return@collect
                    }

                    networkFlapCount++
                    lastNetworkChangeMs = now
                    Timber.i("Network type changed: %s → %s (flap=%d, cooldown=%dms)",
                        previousType, newType, networkFlapCount, cooldownMs)

                    // Only reset circuits when transitioning TO a healthy network.
                    val isRecovery = newType == com.scmessenger.android.transport.NetworkType.WIFI ||
                        newType == com.scmessenger.android.transport.NetworkType.ETHERNET
                    if (isRecovery) {
                        relayCircuitBreaker.resetAll()
                        if (swarmBridge == null) {
                            Timber.i("Network recovered to WiFi/Ethernet and swarm is inactive — attempting recovery restart")
                            repoScope.launch {
                                try {
                                    initializeAndStartSwarm()
                                } catch (e: Exception) {
                                    Timber.w(e, "Swarm startup failed during network recovery self-healing")
                                }
                            }
                        }
                    }
                    previousType = newType
                    racingBootstrapWithFallback()
                }
            }
        }
    }

    private fun stopNetworkChangeWatch() {
        networkWatchJob?.cancel()
        networkWatchJob = null
    }

    /**
     * P0_NETWORK_001 / P0_ANDROID_007: Classify bootstrap connection errors for diagnostics.
     * Records failure metrics for adaptive routing and user-facing diagnostics.
     */
    private fun classifyBootstrapError(exception: Exception, nodeAddr: String? = null): String {
        val port = nodeAddr?.let { extractPortFromMultiaddr(it) }

        // P1_ANDROID_013: Distinguish "device offline" from "server unreachable" from "timeout"
        val isDeviceOffline = networkDetector.networkType.value == com.scmessenger.android.transport.NetworkType.UNKNOWN
        val detail = when {
            isDeviceOffline -> "Device offline — no active network"
            exception is java.net.UnknownHostException ->
                "DNS resolution failed for ${exception.message ?: "unknown host"}"
            exception is java.net.ConnectException -> {
                if (networkDetector.isCellularNetwork && port != null && port !in setOf(80, 443)) {
                    "Port $port blocked on cellular — carrier filtering non-standard ports"
                } else {
                    "Connection refused — port blocked or service down"
                }
            }
            exception is java.net.SocketTimeoutException -> "Connection timeout after dial period"
            exception is javax.net.ssl.SSLException -> "TLS handshake failed: ${exception.message}"
            exception is java.net.PortUnreachableException -> {
                if (networkDetector.isCellularNetwork) {
                    "UDP port unreachable on cellular — carrier blocking QUIC/UDP, try WebSocket"
                } else {
                    "Port unreachable (likely firewall-blocked)"
                }
            }
            exception is java.net.NoRouteToHostException -> "No route to host (network unreachable)"
            exception is java.net.SocketException -> "Socket error: ${exception.message}"
            exception is java.io.IOException -> "Network I/O error: ${exception.message}"
            else -> "Unknown network error: ${exception.javaClass.simpleName}: ${exception.message}"
        }
        // Record failure metrics for diagnostics reporting
        if (nodeAddr != null) {
            networkFailureMetrics.recordFailure(nodeAddr, detail, exception)
        }
        return detail
    }

    /** Extract port number from a libp2p multiaddr string. */
    private fun extractPortFromMultiaddr(multiaddr: String): Int? {
        // e.g. "/ip4/1.2.3.4/tcp/9001/p2p/..." → 9001
        // e.g. "/dns4/bootstrap.example.com/tcp/443/ws/p2p/..." → 443
        val portRegex = Regex("""/tcp/(\d+)""").find(multiaddr)
            ?: Regex("""/udp/(\d+)""").find(multiaddr)
        return portRegex?.groupValues?.get(1)?.toIntOrNull()
    }

    private fun shouldAttemptDial(multiaddr: String): Boolean {
        val key = multiaddr.trim()
        if (key.isEmpty()) return false

        val now = System.currentTimeMillis()
        val (attempts, nextAllowedMs) = dialThrottleState[key] ?: (0 to 0L)
        if (now < nextAllowedMs) {
            // P4: Only log dial throttle once per address per 5-minute window
            val lastLogged = dialThrottleLogCache[key]
            if (lastLogged == null || (now - lastLogged) >= dialThrottleLogIntervalMs) {
                dialThrottleLogCache[key] = now
            }
            return false
        }

        val nextAttempts = (attempts + 1).coerceAtMost(8)
        val backoffMs = when (nextAttempts) {
            1 -> 500L
            2 -> 1_500L
            3 -> 3_000L
            4 -> 6_000L
            5 -> 10_000L
            else -> 15_000L
        }
        dialThrottleState[key] = nextAttempts to (now + backoffMs)
        return true
    }

    // ========================================================================
    // IDENTITY EXPORT HELPERS
    // ========================================================================    // MARK: - Identity Helpers

    fun getPreferredRelay(): String? {
        val relays = ledgerManager?.getPreferredRelays(1u)
        return relays?.firstOrNull()?.peerId
    }

    /** P0_ANDROID_007: Get network failure metrics summary for diagnostics UI */
    fun getNetworkFailureSummary(): com.scmessenger.android.utils.NetworkFailureMetrics.Summary {
        return networkFailureMetrics.getSummary()
    }

    /** P0_ANDROID_007: Get network detector diagnostics */
    fun getNetworkDiagnosticsSnapshot(): com.scmessenger.android.transport.NetworkDiagnostics {
        return networkDetector.getNetworkDiagnostics()
    }

    private fun startStorageMaintenance() {
        if (maintenanceJob?.isActive == true) return

        maintenanceJob = repoScope.launch {
            while (isActive) {
                try {
                    val stat = android.os.StatFs(context.filesDir.path)
                    val total = stat.blockCountLong * stat.blockSizeLong
                    val free = stat.availableBlocksLong * stat.blockSizeLong

                    ironCore?.updateDiskStats(total.toULong(), free.toULong())
                    ironCore?.performMaintenance()

                    // P0_ANDROID_005: Periodic message tracking health check
                    detectAndRecoverMessageTracking()
                    logRetryStormDetection()

                    // P0_ANDROID_003: Periodic BLE transport health check
                    handleBleTransportDegradation()

                    Timber.d("Storage maintenance check: free=${free / 1024 / 1024}MB / total=${total / 1024 / 1024}MB")
                } catch (e: Exception) {
                    Timber.w("Storage maintenance loop error: ${e.message}")
                }
                kotlinx.coroutines.delay(15 * 60 * 1000) // Every 15 minutes
            }
        }
    }

    @Volatile private var externalAddressesSnapshot: List<String> = emptyList()
    @Volatile private var listeningAddressesSnapshot: List<String> = emptyList()

    private fun refreshAddressesSnapshots() {
        repoScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val swarm = swarmBridge
                if (swarm != null) {
                    val external = swarm.getExternalAddresses()
                    val listeners = swarm.getListeners()
                    externalAddressesSnapshot = external
                    listeningAddressesSnapshot = listeners
                    Timber.d("Refreshed address snapshots: listeners=$listeners, external=$external")
                }
            } catch (e: Exception) {
                Timber.w(e, "Failed to refresh address snapshots: ${e.message}")
            }
        }
    }

    /**
     * Get build provenance information from the core (git hash, branch, timestamp).
     */
    fun getBuildProvenance(): String {
        return uniffi.api.getBuildProvenance()
    }

    /**
     * Returns external NAT-mapped addresses observed by peer nodes on the mesh.
     * Uses relay/peer-confirmed observations (identify + reflection consensus).
     */
    fun getExternalAddresses(): List<String> {
        return externalAddressesSnapshot
    }

    /**
     * Returns local listener addresses (bound TCP ports on LAN interfaces).
     * External NAT-mapped addresses are intentionally excluded — they are observed
     * outbound ports, not stable inbound addresses, and including them causes remote
     * peers to attempt unreachable dials.
     */
    fun getListeningAddresses(): List<String> {
        return listeningAddressesSnapshot
    }

    fun getLocalIpAddress(): String? {
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces() ?: return null
            var bestIp: String? = null
            var bestScore = Int.MIN_VALUE

            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (!networkInterface.isUp || networkInterface.isLoopback || networkInterface.isVirtual) {
                    continue
                }

                val ifaceName = networkInterface.name?.lowercase().orEmpty()
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (address !is java.net.Inet4Address || address.isLoopbackAddress || address.isLinkLocalAddress) {
                        continue
                    }

                    val ip = address.hostAddress?.trim().orEmpty()
                    if (ip.isEmpty() || isSpecialUseIpv4(ip)) continue

                    val isPrivate = isPrivateIpv4(ip)
                    val ifaceScore = when {
                        ifaceName.startsWith("wlan") || ifaceName.startsWith("wifi") || ifaceName == "en0" -> 3
                        ifaceName.startsWith("eth") || ifaceName.startsWith("en") -> 2
                        else -> 1
                    }
                    val score = (if (isPrivate) 100 else 10) + ifaceScore
                    if (score > bestScore) {
                        bestScore = score
                        bestIp = ip
                    }
                }
            }
            return bestIp
        } catch (e: Exception) {
            timber.log.Timber.e(e, "Failed to get local IP")
        }
        return null
    }

    suspend fun getIdentityExportString(
        minimalForQr: Boolean = false,
        identityInfo: uniffi.api.IdentityInfo? = null
    ): String {
        val identity = identityInfo ?: getIdentityInfoNonBlocking() ?: return "{}"
        // Read the authoritative listener list at export time.  The cached list is
        // retained as a fallback for a just-started swarm, but a guessed port must
        // never be emitted as routing truth.
        val liveListeners = try {
            swarmBridge?.getListeners().orEmpty()
        } catch (e: Exception) {
            Timber.w(e, "Failed to read live listeners while exporting identity")
            emptyList()
        }
        if (liveListeners.isNotEmpty()) {
            listeningAddressesSnapshot = liveListeners
        }
        val listenerAddresses = liveListeners.ifEmpty { listeningAddressesSnapshot }
        val localIp = getLocalIpAddress()
        val relay = if (minimalForQr) null else getPreferredRelay()

        // UNIFIED ID FIX: QR code primary ID is libp2p_peer_id (network routable)
        // identity_id is secondary (human fingerprint for backup/recovery)
        val payload = org.json.JSONObject()
            .put("version", "1.0")
            .put("peer_id", identity.libp2pPeerId ?: "")           // PRIMARY: libp2p Peer ID for contact add
            .put("public_key", identity.publicKeyHex ?: "")         // Canonical identity key
            .put("device_id", identity.deviceId ?: "")              // Multi-device routing
            .put("identity_id", identity.identityId ?: "")           // SECONDARY: Blake3 hash
            .put("nickname", identity.nickname ?: "")
            .put("libp2p_peer_id", identity.libp2pPeerId ?: "")      // Backward compatibility
            .put("relay", relay ?: "None")

        if (minimalForQr) {
            // P0_ANDROID_QR_FIX: Keep payload small to avoid QR code capacity errors.
            // Do not synthesize /tcp/9001: the core may have selected another
            // port after a bind conflict.  An empty hint list is safer than a
            // false address; mDNS/ledger discovery remains available.
            val minimalHints = normalizeOutboundListenerHints(listenerAddresses)
                .map { addr ->
                    if (localIp != null && addr.contains("0.0.0.0")) {
                        addr.replace("0.0.0.0", localIp)
                    } else {
                        addr
                    }
                }
                .distinct()
                .take(3)
            payload.put("connection_hints", org.json.JSONArray(minimalHints))
        } else {
            var listeners = normalizeOutboundListenerHints(listenerAddresses).toMutableList()
            val externalAddresses = normalizeExternalAddressHints(getExternalAddresses())
            
            if (localIp != null) {
                listeners = listeners.map { addr ->
                    if (addr.contains("0.0.0.0")) addr.replace("0.0.0.0", localIp) else addr
                }.toMutableList()
            }
            
            payload.put("listeners", org.json.JSONArray(listeners))
            payload.put("external_addresses", org.json.JSONArray(externalAddresses))
            payload.put("connection_hints", org.json.JSONArray((listeners + externalAddresses).distinct()))
        }

        return payload.toString()
    }

    // ========================================================================
    // OBSERVABLES FOR UI (NEW)
    // ========================================================================

    /**
     * Observe incoming messages from MeshEventBus.
     */
    fun observeIncomingMessages(): kotlinx.coroutines.flow.Flow<com.scmessenger.android.service.MessageEvent> {
        return com.scmessenger.android.service.MeshEventBus.messageEvents
    }

    /**
     * Observe peer events from MeshEventBus.
     */
    fun observePeers(): kotlinx.coroutines.flow.Flow<List<String>> {
        return kotlinx.coroutines.flow.flow {
            com.scmessenger.android.service.MeshEventBus.peerEvents.collect { _ ->
                // Convert peer events to peer list
                val peers = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    ledgerManager?.dialableAddresses()?.mapNotNull { it.peerId }?.distinct() ?: emptyList()
                }
                emit(peers)
            }
        }
    }

    /**
     * Observe network stats with periodic refresh.
     */
    fun observeNetworkStats(): kotlinx.coroutines.flow.Flow<uniffi.api.ServiceStats> {
        return kotlinx.coroutines.flow.flow {
            while (true) {
                val stats = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                    meshService?.getStats()
                }
                if (stats != null) {
                    emit(stats)
                }
                kotlinx.coroutines.delay(2000) // Refresh every 2 seconds
            }
        }
    }

    // ========================================================================
    // CLEANUP
    // ========================================================================

    fun cleanup() {
        try {
            stopMeshService()
            saveLedger()
            pendingOutboxRetryJob?.cancel()
            pendingOutboxRetryJob = null
            coverTrafficJob?.cancel()
            coverTrafficJob = null
            pendingReceiptSendJobs.values.forEach { it.cancel() }
            pendingReceiptSendJobs.clear()

            // Clear references
            meshService = null
            contactManager = null
            historyManager = null
            ledgerManager = null
            settingsManager = null
            autoAdjustEngine = null

            Timber.i("MeshRepository cleaned up")
        } catch (e: Exception) {
            Timber.e(e, "Error during cleanup")
        }
    }

    // ========================================================================
    // DIAGNOSTICS
    // ========================================================================

    /**
     * Get path to the diagnostics log file.
     */
    fun getDiagnosticsLogPath(): String {
        return File(context.filesDir, "mesh_diagnostics.log").absolutePath
    }

    /**
     * Get recent diagnostics logs.
     */
    fun getDiagnosticsLogs(limit: Int = 500): String {
        return try {
            val logFile = File(getDiagnosticsLogPath())
            if (!logFile.exists()) return "No diagnostics recorded yet."

            val lines = logFile.readLines()
            if (lines.isEmpty()) return "No diagnostics recorded yet."

            lines.takeLast(limit).joinToString("\n")
        } catch (e: Exception) {
            "Error reading diagnostics: ${e.message}"
        }
    }

    /**
     * Clear all diagnostics logs.
     */
    fun clearDiagnosticsLogs() {
        try {
            val logFile = File(getDiagnosticsLogPath())
            if (logFile.exists()) {
                logFile.writeText("")
                Timber.i("Diagnostics logs cleared")
            }
        } catch (e: Exception) {
            Timber.e(e, "Error clearing diagnostics")
        }
    }

    private fun logDeliveryState(messageId: String, state: String, detail: String) {
        if (messageId.isBlank()) return
        Timber.i("delivery_state msg=$messageId state=$state detail=$detail")
    }

    private fun logDeliveryAttempt(
        messageId: String?,
        medium: String,
        phase: String,
        outcome: String,
        detail: String,
        callerContext: String? = null
    ) {
        val msg = messageId?.takeIf { it.isNotBlank() }
        if (msg == null) {
            // AND-DELIVERY-001: Guard against null/blank messageId — log diagnostic warning only,
            // do not emit the main delivery_attempt line with msg=unknown as it pollutes log analysis.
            val contextInfo = callerContext?.takeIf { it.isNotBlank() } ?: "unknown_caller"
            Timber.w("delivery_attempt msg=unknown (messageId was null or blank) medium=%s phase=%s outcome=%s detail=%s caller=%s",
                medium, phase, outcome, detail, contextInfo)
            return
        }
        Timber.i(
            "delivery_attempt msg=%s medium=%s phase=%s outcome=%s detail=%s",
            msg,
            medium,
            phase,
            outcome,
            detail
        )
    }
    private fun migrateToCanonicalIds() {
        val iron = ironCore ?: return
        val history = historyManager ?: return
        val contacts = contactManager ?: return

        try {
            val prefs = context.getSharedPreferences("mesh_migrations", android.content.Context.MODE_PRIVATE)
            if (prefs.getBoolean("v2_id_coalescence", false)) return

            Timber.i("Starting ID Coalescence Migration...")

            // 1. Migrate Contacts
            val contactList = contacts.list()
            val idMap = mutableMapOf<String, String>() // old -> new

            for (contact in contactList) {
                val identityId = try { iron.resolveIdentity(contact.publicKey) } catch (e: Exception) { null }
                if (identityId != null && identityId != contact.peerId) {
                    Timber.i("Coalescing ID for ${contact.nickname ?: contact.peerId}: ${contact.peerId} -> $identityId")
                    idMap[contact.peerId] = identityId

                    val existingCanonical = try { contacts.get(identityId) } catch (e: Exception) { null }
                    if (existingCanonical == null) {
                        // No existing canonical contact - create one with the identity ID
                        try {
                            contacts.add(contact.copy(peerId = identityId))
                        } catch (e: Exception) {
                            Timber.w("Failed to create canonical contact $identityId")
                        }
                    } else {
                        // Canonical contact already exists - merge data from old contact
                        val merged = existingCanonical.copy(
                            nickname = existingCanonical.nickname ?: contact.nickname,
                            localNickname = existingCanonical.localNickname ?: contact.localNickname,
                            lastSeen = maxOf(existingCanonical.lastSeen ?: 0u, contact.lastSeen ?: 0u),
                            notes = mergeNotes(existingCanonical.notes, contact.notes),
                            lastKnownDeviceId = existingCanonical.lastKnownDeviceId ?: contact.lastKnownDeviceId
                        )
                        if (merged != existingCanonical) {
                            try {
                                contacts.add(merged)
                                Timber.i("Merged contact data from ${contact.peerId} into $identityId")
                            } catch (e: Exception) {
                                Timber.w("Failed to merge contact data: ${e.message}")
                            }
                        }
                    }
                    // Only remove old contact after successful merge/creation
                    try { contacts.remove(contact.peerId) } catch (e: Exception) { }
                }
            }

            // 2. Migrate History
            val stats = history.stats()
            if (stats.totalMessages > 0u) {
                val allMessages = history.recent(null, 100000u)
                var updatedMessages = 0
                for (msg in allMessages) {
                    val canonical = idMap[msg.peerId] ?: try { iron.resolveIdentity(msg.peerId) } catch (e: Exception) { null }
                    if (canonical != null && canonical != msg.peerId) {
                        try {
                            history.add(msg.copy(peerId = canonical))
                            updatedMessages++
                        } catch (e: Exception) {
                            Timber.w("Failed to update message peer_id for ${msg.id}")
                        }
                    }
                }
                if (updatedMessages > 0) {
                    Timber.i("Migrated $updatedMessages messages to canonical peer IDs")
                }
            }

            history.flush()
            contacts.flush()
            prefs.edit().putBoolean("v2_id_coalescence", true).apply()
            Timber.i("ID Coalescence Migration completed successfully")
        } catch (e: Exception) {
            Timber.e(e, "ID Coalescence Migration failed")
        }
    }

    // ============================================================================
    // Emergency Contact Recovery (P0_ANDROID_001)
    // ============================================================================

    /**
     * Emergency contact reconstruction from message history.
     * Reconstructs contacts when the contacts database appears corrupted (0 contacts but messages exist).
     */
    private suspend fun emergencyContactRecovery(): Int {
        val history = historyManager ?: run {
            Timber.e("Emergency recovery: HistoryManager not initialized")
            return 0
        }
        val contacts = contactManager ?: run {
            Timber.e("Emergency recovery: ContactManager not initialized")
            return 0
        }

        return try {
            // Get all messages to identify peer IDs
            val messages = history.recent(null, 100000u)
            val peerIds = messages.map { it.peerId }.distinct()

            var recoveredCount = 0
            for (peerId in peerIds) {
                // Check if contact exists
                val existingContact = try { contacts.get(peerId) } catch (e: Exception) { null }
                if (existingContact == null) {
                    // Create a basic contact from the peer ID
                    // Extract actual public key from libp2p PeerId for proper cryptographic operations
                    try {
                        val extractedKey = try {
                            ironCore?.extractPublicKeyFromPeerId(peerId)
                        } catch (e: Exception) {
                            Timber.w("Emergency recovery: Failed to extract public key from $peerId: ${e.message}")
                            null
                        }

                        val publicKey = if (!extractedKey.isNullOrEmpty()) {
                            extractedKey
                        } else {
                            // Fallback: use peer ID as placeholder (will be updated later via federation)
                            peerId
                        }

                        val contact = uniffi.api.Contact(
                            peerId = peerId,
                            nickname = null,
                            localNickname = null,
                            publicKey = publicKey,
                            addedAt = System.currentTimeMillis().toULong(),
                            lastSeen = null,
                            notes = "Emergency contact recovered from message history",
                            lastKnownDeviceId = null,
                            verifiedAt = null,
                            isTombstone = false
                        )
                        contacts.add(contact)
                        recoveredCount++
                        Timber.d("Emergency recovery: Created contact for ${peerId.take(8)}... with key: ${publicKey.take(8)}...")
                    } catch (e: Exception) {
                        Timber.w("Emergency recovery: Failed to create contact for $peerId: ${e.message}")
                    }
                }
            }
            contacts.flush()
            Timber.i("Emergency contact recovery completed: $recoveredCount contacts recovered")
            recoveredCount
        } catch (e: Exception) {
            Timber.e(e, "Emergency contact recovery failed")
            0
        }
    }

    /**
     * Detect and repair data corruption in the contacts database.
     * Checks if contact count is 0 while message count is > 0, indicating corruption.
     */
    private suspend fun detectAndRepairCorruption(): Boolean {
        val history = historyManager ?: run {
            Timber.e("Corruption detection: HistoryManager not initialized")
            return false
        }
        val contacts = contactManager ?: run {
            Timber.e("Corruption detection: ContactManager not initialized")
            return false
        }

        return try {
            val contactCount = contacts.list().size
            val messageCount = history.stats().totalMessages.toInt()

            Timber.d("Corruption check: contacts=$contactCount, messages=$messageCount")

            if (contactCount == 0 && messageCount > 0) {
                Timber.e("CRITICAL: Data corruption detected - $messageCount messages but 0 contacts")

                // Backup corrupted database
                backupCorruptedDatabase()

                // Attempt emergency recovery
                val recovered = emergencyContactRecovery()

                if (recovered > 0) {
                    Timber.i("Corruption repair successful: $recovered contacts recovered")
                    true
                } else {
                    Timber.e("Corruption repair failed: No contacts could be recovered")
                    false
                }
            } else {
                Timber.d("Database integrity check passed")
                false
            }
        } catch (e: Exception) {
            Timber.e(e, "Corruption detection failed")
            false
        }
    }

    /**
     * Backup corrupted database files before repair.
     * Creates timestamped backups in the app's backup directory.
     */
    private fun backupCorruptedDatabase() {
        val backupDir = File(storagePath, "backup_corrupted_${System.currentTimeMillis()}")

        try {
            backupDir.mkdirs()
            Timber.i("Created backup directory: ${backupDir.absolutePath}")

            // Backup contacts.db if it exists
            val contactsDb = File(storagePath, "contacts.db")
            if (contactsDb.exists()) {
                val backupFile = File(backupDir, "contacts.db")
                contactsDb.copyTo(backupFile, overwrite = true)
                Timber.i("Backed up contacts.db to ${backupFile.absolutePath}")
            }

            // Backup history files if they exist
            val historyDir = File(storagePath, "history")
            if (historyDir.exists()) {
                val backupHistoryDir = File(backupDir, "history")
                backupHistoryDir.mkdirs()
                historyDir.listFiles()?.forEach { file ->
                    if (file.isFile) {
                        file.copyTo(File(backupHistoryDir, file.name), overwrite = true)
                    }
                }
                Timber.i("Backed up history directory to ${backupHistoryDir.absolutePath}")
            }

            Timber.i("Corrupted database backup completed")
        } catch (e: Exception) {
            Timber.e(e, "Failed to backup corrupted database")
        }
    }

    // ========================================
    // P0_ANDROID_003: BLE Transport Stabilization & Graceful Degradation
    // ========================================

    /**
     * Check if BLE transport is degraded and initiate fallback.
     * Called from periodic health checks when BLE failures are detected.
     */
    fun handleBleTransportDegradation() {
        if (transportHealthMonitor.isDegraded("ble")) {
            Timber.w("P0_ANDROID_003: BLE transport degraded, initiating graceful fallback")
            // Reduce BLE scan frequency by switching to background mode
            bleScanner?.setBackgroundMode(true)
            // Clear peer cache to allow re-discovery after recovery
            bleScanner?.clearPeerCache()
            // Log current transport health for diagnostics
            val bleHealth = transportHealthMonitor.getHealth("ble")
            Timber.w("P0_ANDROID_003: BLE health — successes: ${bleHealth.successCount}, failures: ${bleHealth.failureCount}, consecutive failures: ${bleHealth.consecutiveFailures}")
        }
    }

    /**
     * Record a transport event for health monitoring.
     */
    fun recordTransportEvent(transport: String, success: Boolean, latencyMs: Long? = null) {
        if (success) {
            transportHealthMonitor.recordSuccess(transport, latencyMs)
            // If BLE was degraded and is now succeeding, attempt recovery
            if (transport == "ble" && !transportHealthMonitor.isDegraded("ble")) {
                transportManager?.attemptBleRecovery()
            }
        } else {
            transportHealthMonitor.recordFailure(transport)
            // Check if degraded after recording failure
            if (transportHealthMonitor.isDegraded(transport)) {
                handleBleTransportDegradation()
            }
        }
    }

    fun getTransportHealthSummary(): Map<String, com.scmessenger.android.transport.TransportHealthMonitor.TransportHealth> {
        return transportHealthMonitor.getSummary()
    }

    /**
     * Get list of currently active transports for status display.
     */
    fun getActiveTransports(): List<com.scmessenger.android.service.TransportType> {
        return transportManager?.getActiveTransports() ?: emptyList()
    }

    /**
     * Handle BLE transport failure with graceful degradation.
     * Reduces BLE usage and prioritizes other transports when BLE fails.
     * Wired from TransportManager.handleBleFailure.
     */
    fun handleBleFailure() {
        transportManager?.handleBleFailure()
        Timber.d("BLE transport failure handled, initiating graceful degradation")
    }

    /**
     * Attempt BLE recovery after degradation.
     * Should be called after a cooldown period to retry BLE operations.
     * Wired from TransportManager.attemptBleRecovery.
     */
    fun attemptBleRecovery() {
        transportManager?.attemptBleRecovery()
        Timber.d("Attempting BLE recovery after degradation cooldown")
    }

    /**
     * Force restart BLE scanning with proper backoff after a failure.
     * Wired from BleScanner.forceRestartScanning.
     */
    fun forceRestartScanning() {
        bleScanner?.forceRestartScanning()
        Timber.d("BLE scanning force restarted with backoff")
    }

    /**
     * Clear BLE peer cache to allow re-discovery.
     * Wired from BleScanner.clearPeerCache.
     */
    fun clearPeerCache() {
        bleScanner?.clearPeerCache()
        Timber.d("BLE peer cache cleared")
    }

    // --- Wired functions from dead-end resolution tasks ---

    /**
     * Create a SmartTransportRouter.TransportType from a string value.
     * Wired from SmartTransportRouter.TransportType.fromValue.
     * Used by transport selection logic that receives transport type as string.
     */
    fun transportTypeFromValue(value: String): SmartTransportRouter.TransportType? {
        return smartTransportRouter?.let { SmartTransportRouter.TransportType.fromValue(value) }
    }

    /**
     * Get the list of available transport types for a peer from SmartTransportRouter.
     * Wired from SmartTransportRouter.getAvailableTransports.
     */
    fun getSmartAvailableTransports(peerId: String): List<SmartTransportRouter.TransportType> {
        return smartTransportRouter?.getAvailableTransports(peerId) ?: emptyList()
    }

    /**
     * Get available transports sorted by priority/score for a peer.
     * Wired from SmartTransportRouter.getAvailableTransportsSorted.
     */
    fun getAvailableTransportsSorted(peerId: String): List<SmartTransportRouter.TransportType> {
        return smartTransportRouter?.getAvailableTransportsSorted(peerId) ?: emptyList()
    }

    /**
     * Get deduplication statistics for a message.
     * Wired from SmartTransportRouter.getDedupStats.
     */
    fun getDedupStats(messageId: String): SmartTransportRouter.MessageDedupEntry? {
        return smartTransportRouter?.getDedupStats(messageId)
    }

    /**
     * Get the list of subscribed topics from TopicManager.
     * Wired from TopicManager.getSubscribedTopicsList.
     */
    fun getSubscribedTopicsList(): List<String> {
        return topicManager?.getSubscribedTopicsList() ?: emptyList()
    }

    /**
     * Get the list of known topics from TopicManager.
     * Wired from TopicManager.getKnownTopicsList.
     */
    fun getKnownTopicsList(): List<String> {
        return topicManager?.getKnownTopicsList() ?: emptyList()
    }

    /**
     * Filter messages by topic from TopicManager.
     * Wired from TopicManager.filterMessagesByTopic.
     */
    fun filterMessagesByTopic(messages: List<uniffi.api.MessageRecord>, topic: String): List<uniffi.api.MessageRecord> {
        return topicManager?.filterMessagesByTopic(messages, topic) ?: messages
    }

    /**
     * Enable a specific transport type at runtime.
     * Wired from TransportManager.enableTransport.
     */
    fun enableTransport(transport: com.scmessenger.android.service.TransportType) {
        transportManager?.enableTransport(transport)
    }

    /**
     * Start all enabled transports.
     * Wired from TransportManager.startAll.
     */
    fun startAllTransports() {
        ensureTransportManager()
        val settings = loadSettings()
        transportManager?.startAll(enableMdns = settings.internetEnabled)
    }

    /**
     * Check if a transport should be used based on health metrics.
     * Wired from TransportHealthMonitor.shouldUseTransport.
     */
    fun shouldUseTransport(transport: String): Boolean {
        return transportHealthMonitor.shouldUseTransport(transport)
    }

    /**
     * Get the current BLE quota count from BleQuotaManager.
     * Wired from BleQuotaManager.currentCount.
     */
    fun getBleQuotaCount(): Int {
        return transportManager?.getBleQuotaCount() ?: 0
    }

    /**
     * Check if a port is likely blocked on the current network.
     * Wired from NetworkDetector.isPortLikelyBlocked.
     */
    fun isPortLikelyBlocked(port: Int): Boolean {
        return networkDetector.isPortLikelyBlocked(port)
    }

    /**
     * Convert current network state to a log string.
     * Wired from NetworkDiagnostics.toLogString.
     */
    fun getNetworkStateLogString(): String {
        return networkDetector.getNetworkDiagnostics().toLogString()
    }

    /**
     * Get list of healthy relay endpoints from CircuitBreaker.
     * Wired from CircuitBreaker.getHealthyRelays.
     */
    fun getHealthyRelays(): List<String> {
        return relayCircuitBreaker.getHealthyRelays()
    }

    /**
     * Get the last failure entry for a relay from NetworkFailureMetrics.
     * Wired from NetworkFailureMetrics.getLastFailure.
     */
    fun getLastFailure(nodeId: String): NetworkFailureMetrics.FailureRecord? {
        return networkFailureMetrics.getLastFailure(nodeId)
    }

    /**
     * Get the last failure reason string for a relay from CircuitBreaker.
     * Wired from CircuitBreaker.getLastFailureReason.
     */
    fun getLastFailureReason(relayAddress: String): String? {
        return relayCircuitBreaker.getLastFailureReason(relayAddress)
    }

    /**
     * Get the count of open circuits from CircuitBreaker.
     * Wired from CircuitBreaker.getOpenCircuits.
     */
    fun getOpenCircuitCount(): Int {
        return relayCircuitBreaker.getOpenCircuits().size
    }

    /**
     * Format a diagnostics report for user display.
     * Wired from DiagnosticsReporter.formatReportForUser.
     */
    suspend fun formatDiagnosticsReportForUser(): String {
        return try {
            val report = diagnosticsReporter.generateReport()
            diagnosticsReporter.formatReportForUser(report)
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate diagnostics report for user")
            "Error generating diagnostics report"
        }
    }

    /**
     * Check for DNS failures from NetworkFailureMetrics.
     * Wired from NetworkFailureMetrics.hasDnsFailures.
     */
    fun hasDnsFailures(): Boolean {
        return networkFailureMetrics.hasDnsFailures()
    }

    /**
     * Check for port blocking from NetworkFailureMetrics.
     * Wired from NetworkFailureMetrics.hasPortBlocking.
     */
    fun hasPortBlocking(): Boolean {
        return networkFailureMetrics.hasPortBlocking()
    }

}
