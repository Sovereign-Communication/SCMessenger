//
//  MultipeerTransport.swift
//  SCMessenger
//
//  WiFi Direct equivalent using Apple's Multipeer Connectivity
//  Transport priority: Multipeer > BLE > Internet
//

import Combine
import MultipeerConnectivity
import os

/// WiFi Direct equivalent using Apple's Multipeer Connectivity
///
/// Multipeer Connectivity provides peer-to-peer WiFi and Bluetooth connectivity
/// without requiring internet infrastructure. Preferred over BLE for speed.
final class MultipeerTransport: NSObject {
    struct DiagnosticsSnapshot {
        let connectedPeers: Int
        let connectingPeers: Int
        let inviteInFlight: Int
        let inviteTimeoutCount: Int
        let inviteDeclineCount: Int
        let effectiveMediumEstimate: String
    }

    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "Multipeer")
    private weak var meshRepository: MeshRepository?

    // Multipeer components
    private var peerID: MCPeerID!
    private var session: MCSession!
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?

    // Service type (must be ≤15 chars, lowercase, no special chars)
    private let serviceType: String = "scmesh"

    // Connection state
    private var connectedPeers: Set<MCPeerID> = []
    private var peerIdsByDisplayName: [String: String] = [:]
    private var connectingPeerNames: Set<String> = []
    private var isAdvertising: Bool = false
    private var isBrowsing: Bool = false

    // Reconnection state: maps peer display name → retry count
    private var reconnectAttempts: [String: Int] = [:]
    // Maximum reconnect attempts before giving up
    private let maxReconnectAttempts: Int = 5
    // Base delay in seconds for exponential backoff
    private let reconnectBaseDelay: TimeInterval = 2.0
    // Invite storm guardrails for reconnect/discovery churn
    private var inviteInFlight: Set<String> = []
    private var lastInviteAttemptAt: [String: Date] = [:]
    private var inviteTimeoutWorkItems: [String: DispatchWorkItem] = [:]
    private var inviteTimeoutCount: Int = 0
    private var inviteDeclineCount: Int = 0
    private let inviteDebounceSeconds: TimeInterval = 4.0
    private let inviteTimeoutSeconds: TimeInterval = 12.0
    private let maxConcurrentInvites: Int = 4

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        setupPeerID()
        setupSession()
    }

    private func identitySnippetForDisplayName() -> String {
        if Thread.isMainThread {
            return MainActor.assumeIsolated { meshRepository?.getIdentitySnippet() ?? "SCMesh" }
        }

        // Use async to avoid deadlock - return default name for background thread calls
        // The peer ID will be updated on next identity change via broadcastIdentityBeacon
        logger.warning("identitySnippetForDisplayName called from background thread - using default name")
        return "SCMesh"
    }

    private func appendRepositoryDiagnostic(_ message: String) {
        Task { @MainActor [weak meshRepository] in
            meshRepository?.appendDiagnostic(message)
        }
    }

    // MARK: - Setup

    private func setupPeerID() {
        // Use identity snippet as display name
        let displayName: String = identitySnippetForDisplayName()
        peerID = MCPeerID(displayName: displayName)
        logger.info("Multipeer peer ID: \(displayName)")
    }

    private func discoveryInfo() -> [String: String]? {
        guard Thread.isMainThread else { return nil }
        return MainActor.assumeIsolated {
            guard let identity = meshRepository?.getFullIdentityInfo(),
                  let peerId = identity.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
                  !peerId.isEmpty else {
                return nil
            }

            var info: [String: String] = ["peer_id": peerId, "version": "1.0"]
            if let publicKey = identity.publicKeyHex?.trimmingCharacters(in: .whitespacesAndNewlines),
               !publicKey.isEmpty {
                info["public_key"] = publicKey
            }
            return info
        }
    }

    private func correlatedRoutePeerId(for peer: MCPeerID) -> String? {
        guard let peerId = peerIdsByDisplayName[peer.displayName] else { return nil }
        let normalizedPeerId = PeerIdValidator.normalize(peerId)
        guard PeerIdValidator.isLibp2pPeerId(normalizedPeerId) else { return nil }
        return normalizedPeerId
    }

    private func setupSession() {
        // .required encryption enforces TLS-like security for all Multipeer frames.
        // This is non-negotiable for a sovereign messenger.
        session = MCSession(
            peer: peerID,
            securityIdentity: nil,
            encryptionPreference: .required
        )
        session.delegate = self
    }

    // MARK: - Reconnection

    /// Schedule a reconnect attempt for a peer that dropped off.
    ///
    /// Uses exponential backoff: delay = base * 2^attempt, capped at 60 s.
    /// Gives up after `maxReconnectAttempts` tries and removes the peer
    /// from the retry table so it can be re-discovered organically.
    private func scheduleReconnect(for peer: MCPeerID) {
        let name: String = peer.displayName
        let attempt: Int = reconnectAttempts[name, default: 0]

        guard attempt < maxReconnectAttempts else {
            logger.warning("Reconnect: giving up on \(name) after \(attempt) attempts")
            reconnectAttempts.removeValue(forKey: name)
            return
        }

        let delay: Double = reconnectBaseDelay * pow(2.0, Double(attempt))
        let cappedDelay: Double = min(delay, 60.0)
        reconnectAttempts[name] = attempt + 1

        logger.info("Reconnect: scheduling attempt \(attempt + 1)/\(self.maxReconnectAttempts) for \(name) in \(Int(cappedDelay))s")

        DispatchQueue.main.asyncAfter(deadline: .now() + cappedDelay) { [weak self] in
            guard let self else { return }
            // Only invite if the peer is not already connected and we are still browsing
            guard !self.connectedPeers.contains(peer), self.browser != nil else {
                self.logger.debug("Reconnect: skipping \(name) — already connected or not browsing")
                return
            }
            self.logger.info("Reconnect: re-inviting \(name)")
            self.invitePeerIfAllowed(peer, source: "reconnect")
        }
    }

    private func clearInviteTracking(for peerName: String) {
        inviteInFlight.remove(peerName)
        inviteTimeoutWorkItems.removeValue(forKey: peerName)?.cancel()
    }

    private func invitePeerIfAllowed(_ peer: MCPeerID, source: String) {
        let name: String = peer.displayName
        let now: Date = Date()

        guard browser != nil else { return }
        guard !connectedPeers.contains(peer) else {
            appendRepositoryDiagnostic(
                "multipeer_invite_skipped id=\(name) source=\(source) reason=already_connected"
            )
            return
        }
        guard !connectingPeerNames.contains(name) else {
            appendRepositoryDiagnostic(
                "multipeer_invite_skipped id=\(name) source=\(source) reason=already_connecting"
            )
            return
        }
        guard !inviteInFlight.contains(name) else {
            appendRepositoryDiagnostic(
                "multipeer_invite_skipped id=\(name) source=\(source) reason=in_flight"
            )
            return
        }
        guard inviteInFlight.count < maxConcurrentInvites else {
            appendRepositoryDiagnostic(
                "multipeer_invite_skipped id=\(name) source=\(source) " +
                "reason=concurrency_limit in_flight=\(inviteInFlight.count)"
            )
            return
        }
        if let last = lastInviteAttemptAt[name], now.timeIntervalSince(last) < inviteDebounceSeconds {
            appendRepositoryDiagnostic(
                "multipeer_invite_debounced id=\(name) source=\(source)"
            )
            return
        }

        lastInviteAttemptAt[name] = now
        inviteInFlight.insert(name)
        browser?.invitePeer(peer, to: session, withContext: nil, timeout: 10)
        appendRepositoryDiagnostic(
            "multipeer_invite_sent id=\(name) source=\(source) in_flight=\(inviteInFlight.count)"
        )

        inviteTimeoutWorkItems[name]?.cancel()
        let timeoutWork: DispatchWorkItem = DispatchWorkItem { [weak self] in
            guard let self else { return }
            guard self.inviteInFlight.contains(name) else { return }
            self.inviteTimeoutCount += 1
            self.clearInviteTracking(for: name)
            self.appendRepositoryDiagnostic(
                "multipeer_invite_timeout id=\(name) source=\(source) timeouts=\(self.inviteTimeoutCount)"
            )
        }
        inviteTimeoutWorkItems[name] = timeoutWork
        DispatchQueue.main.asyncAfter(deadline: .now() + inviteTimeoutSeconds, execute: timeoutWork)
    }

    // MARK: - Public API

    func startAdvertising() {
        guard !isAdvertising else {
            logger.debug("Already advertising")
            return
        }

        advertiser = MCNearbyServiceAdvertiser(
            peer: peerID,
            discoveryInfo: discoveryInfo(),
            serviceType: serviceType
        )
        advertiser?.delegate = self
        advertiser?.startAdvertisingPeer()
        isAdvertising = true
        logger.info("Started Multipeer advertising")
    }

    func stopAdvertising() {
        advertiser?.stopAdvertisingPeer()
        advertiser = nil
        isAdvertising = false
        logger.info("Stopped Multipeer advertising")
    }

    func startBrowsing() {
        guard !isBrowsing else {
            logger.debug("Already browsing")
            return
        }

        browser = MCNearbyServiceBrowser(peer: peerID, serviceType: serviceType)
        browser?.delegate = self
        browser?.startBrowsingForPeers()
        isBrowsing = true
        logger.info("Started Multipeer browsing")
    }

    func stopBrowsing() {
        browser?.stopBrowsingForPeers()
        browser = nil
        isBrowsing = false
        logger.info("Stopped Multipeer browsing")
    }

    func sendData(to peer: MCPeerID, data: Data) throws {
        guard connectedPeers.contains(peer) else {
            logger.error("Peer \(peer.displayName) not connected")
            throw MultipeerError.notConnected
        }

        try session.send(data, toPeers: [peer], with: .reliable)
        logger.debug("Sent \(data.count) bytes to \(peer.displayName)")
    }

    func sendData(toPeerId peerId: String, data: Data) throws {
        guard let connectedPeer = connectedPeers.first(where: {
            $0.displayName == peerId || peerIdsByDisplayName[$0.displayName] == peerId
        }) else {
            logger.debug("Peer \(peerId) is not connected on Multipeer")
            throw MultipeerError.notConnected
        }
        try sendData(to: connectedPeer, data: data)
    }

    func sendDataToAll(_ data: Data) {
        let peers: [MCPeerID] = Array(connectedPeers)
        guard !peers.isEmpty else {
            logger.warning("No connected peers")
            return
        }

        do {
            try session.send(data, toPeers: peers, with: .reliable)
            logger.debug("Sent \(data.count) bytes to \(peers.count) peers")
        } catch {
            logger.error("Failed to send data: \(error.localizedDescription)")
        }
    }

    func disconnect() {
        // Clear reconnect table first so scheduled retries become no-ops
        reconnectAttempts.removeAll()
        connectingPeerNames.removeAll()
        inviteInFlight.removeAll()
        lastInviteAttemptAt.removeAll()
        inviteTimeoutWorkItems.values.forEach { $0.cancel() }
        inviteTimeoutWorkItems.removeAll()
        session.disconnect()
        stopAdvertising()
        stopBrowsing()
        connectedPeers.removeAll()
        logger.info("Disconnected from Multipeer session")
    }

    func hasConnection(_ peerId: String) -> Bool {
        connectedPeers.contains(where: {
            $0.displayName == peerId || peerIdsByDisplayName[$0.displayName] == peerId
        })
    }

    func diagnosticsSnapshot() -> DiagnosticsSnapshot {
        let mediumEstimate: String = connectedPeers.isEmpty ? "none" : "unknown"
        return DiagnosticsSnapshot(
            connectedPeers: connectedPeers.count,
            connectingPeers: connectingPeerNames.count,
            inviteInFlight: inviteInFlight.count,
            inviteTimeoutCount: inviteTimeoutCount,
            inviteDeclineCount: inviteDeclineCount,
            effectiveMediumEstimate: mediumEstimate
        )
    }
}

// MARK: - MCSessionDelegate

extension MultipeerTransport: MCSessionDelegate {
    func session(
        _ session: MCSession,
        didReceive stream: InputStream,
        withName streamName: String,
        fromPeer peerID: MCPeerID
    ) {
        logger.debug("Received stream from \(peerID.displayName)")
        // Handle stream if needed
    }

    func session(
        _ session: MCSession,
        didStartReceivingResourceWithName resourceName: String,
        fromPeer peerID: MCPeerID,
        with progress: Progress
    ) {
        logger.debug("Receiving resource from \(peerID.displayName)")
    }

    func session(
        _ session: MCSession,
        didFinishReceivingResourceWithName resourceName: String,
        fromPeer peerID: MCPeerID,
        at localURL: URL?,
        withError error: Error?
    ) {
        if let error = error {
            logger.error("Resource receive error: \(error.localizedDescription)")
        }
    }

    private func handleSessionState(peerID: MCPeerID, state: MCSessionState) {
        logger.info("Peer \(peerID.displayName) state changed: \(state.rawValue)")
        let peerName: String = peerID.displayName

        switch state {
        case .connected:
            connectedPeers.insert(peerID)
            connectingPeerNames.remove(peerName)
            clearInviteTracking(for: peerName)
            appendRepositoryDiagnostic("multipeer_connected id=\(peerID.displayName)")
            // Clear any pending reconnect counter — peer is healthy again
            reconnectAttempts.removeValue(forKey: peerID.displayName)
            if let routePeerId = correlatedRoutePeerId(for: peerID) {
                let repo = meshRepository
                Task { @MainActor in
                    repo?.handleTransportPeerDiscovered(peerId: routePeerId)
                }
                MeshEventBus.shared.peerEvents.send(.connected(peerId: routePeerId))
            } else {
                logger.warning("Multipeer peer \(peerName) connected without a valid libp2p peer_id correlation")
            }

        case .connecting:
            connectingPeerNames.insert(peerName)
            logger.debug("Connecting to \(peerID.displayName)")
            appendRepositoryDiagnostic("multipeer_connecting id=\(peerID.displayName)")

        case .notConnected:
            connectedPeers.remove(peerID)
            connectingPeerNames.remove(peerName)
            if inviteInFlight.contains(peerName) {
                inviteDeclineCount += 1
                appendRepositoryDiagnostic(
                    "multipeer_invite_not_connected id=\(peerName) declines=\(inviteDeclineCount)"
                )
            }
            clearInviteTracking(for: peerName)
            appendRepositoryDiagnostic("multipeer_disconnected id=\(peerID.displayName)")
            if let routePeerId = correlatedRoutePeerId(for: peerID) {
                MeshEventBus.shared.peerEvents.send(.disconnected(peerId: routePeerId))
            } else {
                logger.debug("Multipeer peer \(peerName) disconnected without a valid libp2p peer_id correlation")
            }
            // Attempt to re-establish the connection with exponential backoff
            scheduleReconnect(for: peerID)

        @unknown default:
            logger.warning("Unknown session state")
        }
    }

    func session(_ session: MCSession, peer peerID: MCPeerID, didChange state: MCSessionState) {
        guard !Thread.isMainThread else {
            handleSessionState(peerID: peerID, state: state)
            return
        }
        DispatchQueue.main.async { [weak self] in
            self?.handleSessionState(peerID: peerID, state: state)
        }
    }

    func session(_ session: MCSession, didReceive data: Data, fromPeer peerID: MCPeerID) {
        logger.debug("Received \(data.count) bytes from \(peerID.displayName)")
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            guard let routePeerId = self.correlatedRoutePeerId(for: peerID) else {
                self.logger.warning("Dropping Multipeer data from \(peerID.displayName) without a valid libp2p peer_id correlation")
                return
            }
            Task { @MainActor [weak self] in
                self?.meshRepository?.onMultipeerDataReceived(peerId: routePeerId, data: data)
            }
        }
    }
}

// MARK: - MCNearbyServiceAdvertiserDelegate

extension MultipeerTransport: MCNearbyServiceAdvertiserDelegate {
    func advertiser(
        _ advertiser: MCNearbyServiceAdvertiser,
        didReceiveInvitationFromPeer peerID: MCPeerID,
        withContext context: Data?,
        invitationHandler: @escaping (Bool, MCSession?) -> Void
    ) {
        logger.info("Received invitation from \(peerID.displayName)")

        // Auto-accept invitations (mesh network)
        invitationHandler(true, session)
    }

    func advertiser(
        _ advertiser: MCNearbyServiceAdvertiser,
        didNotStartAdvertisingPeer error: Error
    ) {
        logger.error("Failed to start advertising: \(error.localizedDescription)")
        isAdvertising = false
    }
}

// MARK: - MCNearbyServiceBrowserDelegate

extension MultipeerTransport: MCNearbyServiceBrowserDelegate {
    private func handleFoundPeer(
        foundPeer peerID: MCPeerID,
        withDiscoveryInfo info: [String: String]?
    ) {
        logger.info("Found peer: \(peerID.displayName)")

        if let advertisedPeerId = info?["peer_id"] {
            let normalizedPeerId = PeerIdValidator.normalize(advertisedPeerId)
            if PeerIdValidator.isLibp2pPeerId(normalizedPeerId) {
                peerIdsByDisplayName[peerID.displayName] = normalizedPeerId
                let repo = meshRepository
                Task { @MainActor in
                    repo?.handleTransportPeerDiscovered(peerId: normalizedPeerId)
                }
                MeshEventBus.shared.peerEvents.send(.discovered(peerId: normalizedPeerId))
            } else {
                peerIdsByDisplayName.removeValue(forKey: peerID.displayName)
                logger.warning("Multipeer peer \(peerID.displayName) did not advertise a valid libp2p peer_id")
            }
        } else {
            peerIdsByDisplayName.removeValue(forKey: peerID.displayName)
            logger.warning("Multipeer peer \(peerID.displayName) did not advertise a valid libp2p peer_id")
        }

        // Auto-invite with debounce/in-flight guardrails to prevent invitation storms.
        invitePeerIfAllowed(peerID, source: "discovery")
        appendRepositoryDiagnostic("multipeer_discovered id=\(peerID.displayName)")
    }

    func browser(
        _ browser: MCNearbyServiceBrowser,
        foundPeer peerID: MCPeerID,
        withDiscoveryInfo info: [String: String]?
    ) {
        guard !Thread.isMainThread else {
            handleFoundPeer(foundPeer: peerID, withDiscoveryInfo: info)
            return
        }
        DispatchQueue.main.async { [weak self] in
            self?.handleFoundPeer(foundPeer: peerID, withDiscoveryInfo: info)
        }
    }

    func browser(_ browser: MCNearbyServiceBrowser, lostPeer peerID: MCPeerID) {
        logger.info("Lost peer: \(peerID.displayName)")
    }

    func browser(
        _ browser: MCNearbyServiceBrowser,
        didNotStartBrowsingForPeers error: Error
    ) {
        logger.error("Failed to start browsing: \(error.localizedDescription)")
        isBrowsing = false
    }
}

// MARK: - Error Types

enum MultipeerError: LocalizedError {
    case notConnected
    case sendFailed

    var errorDescription: String? {
        switch self {
        case .notConnected:
            return "Peer not connected"
        case .sendFailed:
            return "Failed to send data"
        }
    }
}
