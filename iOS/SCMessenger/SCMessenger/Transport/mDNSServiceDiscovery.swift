//
//  mDNSServiceDiscovery.swift
//  SCMessenger
//
//  mDNS/DNS-SD service discovery for cross-platform LAN discovery
//  Mirrors: android/.../transport/WifiDirectTransport.kt DNS-SD implementation
//  Service types: _p2p._udp (libp2p/Android) and _scmessenger._tcp (legacy iOS)
//

import Foundation
import Network
import os
import Combine

/// mDNS/DNS-SD service discovery for cross-platform LAN discovery
///
/// Browse both the libp2p service used by Android and the legacy iOS service.
/// Keeping both avoids regressing iOS-to-iOS discovery while adding Android parity.
@MainActor
final class mDNSServiceDiscovery: NSObject {
    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "mDNS")
    private weak var meshRepository: MeshRepository?

    // Service discovery
    private var netServiceBrowsers: [NetServiceBrowser] = []
    private var discoveredServices: [String: NetService] = [:]
    private var isBrowsing: Bool = false

    // Service advertisement
    private var localServices: [NetService] = []
    private var isAdvertising: Bool = false
    private var advertisingGeneration: UInt64 = 0

    private let serviceTypes: [String] = ["_p2p._udp", "_scmessenger._tcp"]
    private let serviceName: String = "SCMessenger"

    /// Callback when a LAN peer is resolved (`peerId`, `host`, `port`).
    /// The caller can construct a peer-specific multiaddr and dial via SwarmBridge.
    var onLanPeerResolved: ((String, String, Int32) -> Void)?

    init(meshRepository: MeshRepository?) {
        self.meshRepository = meshRepository
        super.init()
    }

    // MARK: - Public API

    func startBrowsing() {
        guard !isBrowsing else {
            logger.debug("Already browsing for mDNS services")
            return
        }

        logger.info("Starting mDNS browsing for \(self.serviceTypes.joined(separator: ", "))")
        netServiceBrowsers = serviceTypes.map { serviceType in
            let browser = NetServiceBrowser()
            browser.delegate = self
            browser.searchForServices(ofType: serviceType, inDomain: "local.")
            return browser
        }
        isBrowsing = true
    }

    func stopBrowsing() {
        guard isBrowsing else { return }
        logger.info("Stopping mDNS browsing")
        netServiceBrowsers.forEach { $0.stop() }
        netServiceBrowsers.removeAll()
        discoveredServices.removeAll()
        isBrowsing = false
    }

    func startAdvertising(port: Int32) {
        guard localServices.isEmpty else {
            logger.debug("Already advertising mDNS service")
            return
        }

        advertisingGeneration &+= 1
        let generation = advertisingGeneration
        let identity = meshRepository?.getFullIdentityInfo()
        let advertisedPeerId = identity?.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines)
        let advertisedServiceName = advertisedPeerId.flatMap { peerId in
            peerId.isEmpty ? nil : peerId
        } ?? serviceName
        logger.info("Starting mDNS advertising for \(advertisedServiceName) on port \(port)")
        let services = serviceTypes.map { serviceType in
            let service = NetService(
                domain: "local.",
                type: serviceType,
                name: advertisedServiceName,
                port: port
            )
            service.delegate = self
            return service
        }
        localServices = services
        isAdvertising = true

        // Set TXT records for cross-platform compatibility (match Android format)
        Task { @MainActor [weak self] in
            guard let self,
                  self.advertisingGeneration == generation,
                  self.localServices.count == services.count else { return }

            if let identity = self.meshRepository?.getFullIdentityInfo(),
               let peerId = identity.libp2pPeerId?.trimmingCharacters(in: .whitespacesAndNewlines),
               !peerId.isEmpty {
                let advertisedHost = self.meshRepository?.getLocalIpAddress() ?? "0.0.0.0"
                let dnsaddr = "/ip4/\(advertisedHost)/tcp/\(port)/p2p/\(peerId)"
                var txtRecord: [String: Data] = [
                    // libp2p's canonical mDNS TXT contract.
                    "dnsaddr": Data(dnsaddr.utf8),
                    // Compatibility aliases used by the mobile DNS-SD peers.
                    "peer-id": Data(peerId.utf8),
                    "p2p": Data(peerId.utf8),
                    "peer_id": Data(peerId.utf8),
                    "device_id": Data((identity.deviceId ?? "").utf8),
                    "version": Data("1.0".utf8),
                    "transport": Data("tcp".utf8)
                ]
                if let publicKey = identity.publicKeyHex {
                    txtRecord["pubkey"] = Data((String(publicKey.prefix(16)) + "...").utf8)
                }
                services.forEach { $0.setTXTRecord(NetService.data(fromTXTRecord: txtRecord)) }
                self.logger.debug("mDNS TXT record set: \(txtRecord.keys.sorted()) dnsaddr=\(dnsaddr)")
            }
            services.forEach { $0.publish() }
        }
    }

    func stopAdvertising() {
        advertisingGeneration &+= 1
        guard !localServices.isEmpty || isAdvertising else { return }
        logger.info("Stopping mDNS advertising")
        localServices.forEach {
            $0.stop()
            $0.delegate = nil
        }
        localServices.removeAll()
        isAdvertising = false
    }

    func cleanup() {
        stopBrowsing()
        stopAdvertising()
    }
}

// MARK: - NetServiceBrowserDelegate

extension mDNSServiceDiscovery: NetServiceBrowserDelegate {
    func netServiceBrowser(_ browser: NetServiceBrowser, didFind service: NetService, moreComing: Bool) {
        let serviceKey: String = "\(service.name):\(service.type)"
        logger.info("mDNS service found: \(service.name) type: \(service.type)")

        // Resolve the service to get the address
        service.delegate = self
        service.resolve(withTimeout: 5.0)
        discoveredServices[serviceKey] = service
    }

    func netServiceBrowser(_ browser: NetServiceBrowser, didRemove service: NetService, moreComing: Bool) {
        let serviceKey: String = "\(service.name):\(service.type)"
        logger.info("mDNS service removed: \(service.name)")
        discoveredServices.removeValue(forKey: serviceKey)
    }

    func netServiceBrowserDidStopSearch(_ browser: NetServiceBrowser) {
        logger.info("mDNS browser stopped")
        isBrowsing = false
    }

    func netServiceBrowser(_ browser: NetServiceBrowser, didNotSearch errorDict: [String: NSNumber]) {
        logger.error("mDNS browser failed: \(errorDict)")
        isBrowsing = false
    }
}

// MARK: - NetServiceDelegate

extension mDNSServiceDiscovery: NetServiceDelegate {
    private func resolvedTXTValues(for service: NetService) -> [String: String] {
        guard let data = service.txtRecordData() else {
            return [:]
        }
        let records = NetService.dictionary(fromTXTRecord: data)
        return records.reduce(into: [String: String]()) { result, entry in
            guard let value = String(data: entry.value, encoding: .utf8) else { return }
            result[entry.key.lowercased()] = value
        }
    }

    private func peerId(from txtValues: [String: String]) -> String? {
        let direct = ["peer-id", "p2p", "peer_id"]
            .compactMap { txtValues[$0]?.trimmingCharacters(in: .whitespacesAndNewlines) }
            .first { PeerIdValidator.isLibp2pPeerId($0) }
        if let direct { return direct }

        guard let dnsaddr = txtValues["dnsaddr"] else { return nil }
        let components = dnsaddr.split(separator: "/", omittingEmptySubsequences: true)
        guard let p2pIndex = components.firstIndex(of: "p2p"),
              components.index(after: p2pIndex) < components.endIndex else { return nil }
        let candidate = String(components[components.index(after: p2pIndex)])
        return PeerIdValidator.isLibp2pPeerId(candidate) ? candidate : nil
    }

    func netServiceDidResolveAddress(_ sender: NetService) {
        guard let addresses = sender.addresses, !addresses.isEmpty else {
            logger.warning("mDNS service resolved but no addresses: \(sender.name)")
            return
        }

        // Use the first address and convert to string
        let address: Data = addresses[0]
        var host: String = "unknown"
        var port: Int32 = Int32(0)
        
        // Convert sockaddr to string representation
        address.withUnsafeBytes { ptr in
            let sockaddrPtr = ptr.bindMemory(to: sockaddr.self)
            guard let firstSockaddr = sockaddrPtr.first else { return }
            var buffer: [CChar] = [CChar](repeating: 0, count: Int(INET6_ADDRSTRLEN))
            if firstSockaddr.sa_family == sa_family_t(AF_INET) {
                var sin: sockaddr_in = address.withUnsafeBytes { $0.load(as: sockaddr_in.self) }
                inet_ntop(AF_INET, &sin.sin_addr, &buffer, socklen_t(INET_ADDRSTRLEN))
                host = String(cString: buffer)
                port = Int32(UInt16(bigEndian: sin.sin_port))
            } else if firstSockaddr.sa_family == sa_family_t(AF_INET6) {
                var sin6: sockaddr_in6 = address.withUnsafeBytes { $0.load(as: sockaddr_in6.self) }
                inet_ntop(AF_INET6, &sin6.sin6_addr, &buffer, socklen_t(INET6_ADDRSTRLEN))
                host = String(cString: buffer)
                port = Int32(UInt16(bigEndian: sin6.sin6_port))
            }
        }

        let txtValues = resolvedTXTValues(for: sender)
        let resolvedPeerId = peerId(from: txtValues)
        logger.info("mDNS service resolved: \(sender.name) at \(host):\(port) peer=\(resolvedPeerId ?? "unknown")")

        guard host != "unknown", port > 0 else { return }
        guard host != "127.0.0.1", host != "::1" else {
            logger.debug("Ignoring loopback mDNS result for \(sender.name)")
            return
        }

        guard !host.hasPrefix("169.254."), !host.lowercased().hasPrefix("fe80:") else {
            logger.debug("Ignoring link-local mDNS result for \(sender.name) at \(host)")
            return
        }

        guard let peerId = resolvedPeerId else {
            logger.warning("Ignoring mDNS result without a valid libp2p peer ID: \(sender.name)")
            return
        }

        if let localPeerId = meshRepository?.getFullIdentityInfo()?.libp2pPeerId,
           PeerIdValidator.isSame(localPeerId, peerId) {
            logger.debug("Ignoring self mDNS result for \(peerId)")
            return
        }

        // Notify discovery
        let repo: MeshRepository? = meshRepository
        DispatchQueue.main.async {
            repo?.handleTransportPeerDiscovered(peerId: peerId)
            // Also send to event bus for UI
            MeshEventBus.shared.peerEvents.send(.discovered(peerId: peerId))
        }

        // TCP/mDNS parity: Notify the resolved LAN address so the caller
        // can generate a libp2p multiaddr and dial via SwarmBridge.
        logger.info("mDNS: LAN peer resolved \(peerId) at \(host):\(port) — notifying for pinned SwarmBridge dial")
        onLanPeerResolved?(peerId, host, port)
    }

    func netService(_ sender: NetService, didNotResolve errorDict: [String: NSNumber]) {
        logger.error("mDNS service failed to resolve: \(sender.name)")
    }
}
