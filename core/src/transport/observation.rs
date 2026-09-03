// Address Observation and Consensus
//
// Tracks observations from multiple peers to determine our actual external addresses.
// Implements consensus-based address discovery without relying on external STUN servers.

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use std::collections::HashMap;
use std::net::SocketAddr;
use web_time::{SystemTime, UNIX_EPOCH};

/// Observation of our address from a peer
#[derive(Debug, Clone)]
pub struct AddressObservation {
    /// The peer that observed this address
    pub observer: PeerId,
    /// The observed address
    pub address: SocketAddr,
    /// When this observation was made (unix timestamp)
    pub timestamp: u64,
    /// How many times this peer has confirmed this address
    pub confirmation_count: u32,
}

/// Tracks and aggregates address observations from multiple peers
#[derive(Debug, Clone)]
pub struct AddressObserver {
    /// Observations indexed by observer peer ID
    observations: HashMap<PeerId, AddressObservation>,
    /// Cached consensus result (recalculated when observations change)
    cached_external_addresses: Vec<SocketAddr>,
    /// Ports this node actually listens on. Observations whose port is not in
    /// this set are the NAT-mapped *source* ports of outbound flows --
    /// ephemeral and never dialable -- and are dropped at the source
    /// (V040-T14 P0). Empty set = accept all (browser/wasm transport has no
    /// listeners).
    listen_ports: Vec<u16>,
}

impl Default for AddressObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl AddressObserver {
    /// Create a new address observer
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            cached_external_addresses: Vec::new(),
            listen_ports: Vec::new(),
        }
    }

    /// Restrict accepted observations to addresses whose port is in `ports`
    /// (our own listen ports). Call whenever the listener set changes.
    /// Already-stored observations are re-filtered immediately.
    pub fn set_listen_ports(&mut self, ports: Vec<u16>) {
        self.listen_ports = ports;
        self.recalculate_consensus();
    }

    /// Record an observation from a peer
    pub fn record_observation(&mut self, observer: PeerId, address: SocketAddr) {
        // V040-T14 P0: an observed address whose port is not one we listen on
        // is the NAT-mapped source port of an outbound flow -- ephemeral and
        // never dialable. It must never enter the consensus: once advertised,
        // every peer that learns it wastes dial budget on it and inbound
        // reachability for that address is impossible. Empty listen_ports =
        // accept all (browser/wasm transport has no listeners).
        if !self.listen_ports.is_empty() && !self.listen_ports.contains(&address.port()) {
            tracing::debug!(
                "Dropping address observation {} from {}: port {} is not a listen port",
                address,
                observer,
                address.port()
            );
            return;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();

        self.observations
            .entry(observer)
            .and_modify(|obs| {
                if obs.address == address {
                    // Same address confirmed
                    obs.confirmation_count += 1;
                    obs.timestamp = now;
                } else {
                    // Address changed
                    obs.address = address;
                    obs.confirmation_count = 1;
                    obs.timestamp = now;
                }
            })
            .or_insert(AddressObservation {
                observer,
                address,
                timestamp: now,
                confirmation_count: 1,
            });

        // Recalculate consensus
        self.recalculate_consensus();
    }

    /// Get the most likely external addresses based on consensus
    pub fn external_addresses(&self) -> &[SocketAddr] {
        &self.cached_external_addresses
    }

    /// Get the primary external address (most commonly observed)
    pub fn primary_external_address(&self) -> Option<SocketAddr> {
        self.cached_external_addresses.first().copied()
    }

    /// Get all observations for debugging
    pub fn all_observations(&self) -> Vec<AddressObservation> {
        self.observations.values().cloned().collect()
    }

    /// Remove observations older than max_age_secs
    pub fn expire_old_observations(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();

        self.observations
            .retain(|_, obs| now - obs.timestamp < max_age_secs);

        self.recalculate_consensus();
    }

    /// Recalculate consensus addresses from observations
    fn recalculate_consensus(&mut self) {
        // Count observations per address
        let mut address_counts: HashMap<SocketAddr, u32> = HashMap::new();

        for obs in self.observations.values() {
            // Belt-and-suspenders with the record-time gate: any stored
            // observation that predates a set_listen_ports call is excluded
            // from the consensus the same way.
            if !self.listen_ports.is_empty() && !self.listen_ports.contains(&obs.address.port()) {
                continue;
            }
            *address_counts.entry(obs.address).or_insert(0) += obs.confirmation_count;
        }

        // Sort by count (most observed first). Equal counts resolve by address
        // so the consensus is deterministic -- the previous sort left ties to
        // HashMap iteration order, letting an attacker's permitted-port
        // observation win promotion over an equally-voted legitimate one
        // depending on the run (V040-T14 P0 audit).
        let mut addresses: Vec<(SocketAddr, u32)> = address_counts.into_iter().collect();
        addresses.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Cache the sorted addresses
        self.cached_external_addresses = addresses.into_iter().map(|(addr, _)| addr).collect();
    }
}

/// Extract the set of TCP/UDP ports from a list of multiaddrs (our listeners).
/// Used to distinguish legitimate NAT-reflected observations (port == a port
/// we listen on) from ephemeral NAT source ports of outbound flows.
pub fn listen_ports_from_multiaddrs(addrs: &[Multiaddr]) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();
    for addr in addrs {
        for proto in addr.iter() {
            let port = match proto {
                Protocol::Tcp(p) | Protocol::Udp(p) => p,
                _ => continue,
            };
            if !ports.contains(&port) {
                ports.push(port);
            }
        }
    }
    ports
}

/// Connection endpoint information
#[derive(Debug, Clone)]
pub struct ConnectionEndpoint {
    /// Remote peer ID
    pub peer_id: PeerId,
    /// Remote address (what we see for them)
    pub remote_addr: Multiaddr,
    /// Local address (what we're using to connect)
    pub local_addr: Multiaddr,
    /// Connection ID
    pub connection_id: String,
    /// Timestamp when connection was established
    pub established_at: u64,
}

/// Tracks active connections and their endpoints
#[derive(Debug, Clone)]
pub struct ConnectionTracker {
    /// Active connections indexed by peer ID and libp2p connection ID.
    /// A peer can legitimately have simultaneous direct and relayed paths.
    connections: HashMap<PeerId, HashMap<String, ConnectionEndpoint>>,
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionTracker {
    /// Create a new connection tracker
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Record a new connection
    pub fn add_connection(
        &mut self,
        peer_id: PeerId,
        remote_addr: Multiaddr,
        local_addr: Multiaddr,
        connection_id: String,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();

        self.connections.entry(peer_id).or_default().insert(
            connection_id.clone(),
            ConnectionEndpoint {
                peer_id,
                remote_addr,
                local_addr,
                connection_id,
                established_at: now,
            },
        );
    }

    /// Remove every active connection for a peer.
    pub fn remove_connection(&mut self, peer_id: &PeerId) {
        self.connections.remove(peer_id);
    }

    /// Remove one connection while preserving any other active path to the
    /// same peer.
    pub fn remove_connection_by_id(&mut self, peer_id: &PeerId, connection_id: &str) {
        let Some(connections) = self.connections.get_mut(peer_id) else {
            return;
        };
        connections.remove(connection_id);
        if connections.is_empty() {
            self.connections.remove(peer_id);
        }
    }

    /// Get connection info for a peer
    pub fn get_connection(&self, peer_id: &PeerId) -> Option<&ConnectionEndpoint> {
        self.connections
            .get(peer_id)
            .and_then(|connections| connections.values().max_by_key(|conn| conn.established_at))
    }

    /// Get the endpoint for the exact connection that delivered a
    /// request-response message. This prevents a stale direct-LAN path from
    /// authorizing disclosure on a different public or relayed connection.
    pub fn get_connection_by_id(
        &self,
        peer_id: &PeerId,
        connection_id: &str,
    ) -> Option<&ConnectionEndpoint> {
        self.connections
            .get(peer_id)
            .and_then(|connections| connections.get(connection_id))
    }

    /// Get all active connections
    pub fn all_connections(&self) -> Vec<ConnectionEndpoint> {
        self.connections
            .values()
            .flat_map(|connections| connections.values().cloned())
            .collect()
    }

    /// Extract SocketAddr from a Multiaddr (best effort)
    pub fn extract_socket_addr(addr: &Multiaddr) -> Option<SocketAddr> {
        use libp2p::multiaddr::Protocol;

        let mut ip = None;
        let mut port = None;

        for protocol in addr.iter() {
            match protocol {
                Protocol::Ip4(addr) => ip = Some(std::net::IpAddr::V4(addr)),
                Protocol::Ip6(addr) => ip = Some(std::net::IpAddr::V6(addr)),
                Protocol::Tcp(p) => port = Some(p),
                Protocol::Udp(p) => port = Some(p),
                _ => {}
            }
        }

        match (ip, port) {
            (Some(ip), Some(port)) => Some(SocketAddr::new(ip, port)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_observer_consensus() {
        let mut observer = AddressObserver::new();

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        let addr1: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let addr2: SocketAddr = "5.6.7.8:5678".parse().unwrap();

        // Three peers observe addr1
        observer.record_observation(peer1, addr1);
        observer.record_observation(peer2, addr1);
        observer.record_observation(peer3, addr1);

        // One peer observes addr2
        observer.record_observation(PeerId::random(), addr2);

        // Consensus should be addr1 (3 votes vs 1)
        assert_eq!(observer.primary_external_address(), Some(addr1));
        assert_eq!(observer.external_addresses().len(), 2);
        assert_eq!(observer.external_addresses()[0], addr1);
    }

    #[test]
    fn test_ephemeral_source_port_observation_is_dropped() {
        let mut observer = AddressObserver::new();
        observer.set_listen_ports(vec![9001]);

        let peer = PeerId::random();
        // 7196 is an ephemeral NAT source port, not a listen port -- must never
        // enter the consensus.
        observer.record_observation(peer, "147.81.41.188:7196".parse().unwrap());
        assert!(observer.external_addresses().is_empty());

        // The same public IP on a real listen port IS accepted.
        observer.record_observation(peer, "147.81.41.188:9001".parse().unwrap());
        assert_eq!(
            observer.primary_external_address(),
            Some("147.81.41.188:9001".parse().unwrap())
        );
    }

    #[test]
    fn test_consensus_excludes_ephemeral_port_even_when_more_common() {
        let mut observer = AddressObserver::new();
        observer.set_listen_ports(vec![9001]);

        // Five peers observe the ephemeral source port; one observes the real
        // listen port. Without the filter the ephemeral port wins the vote and
        // becomes the advertised external address.
        for _ in 0..5 {
            observer.record_observation(PeerId::random(), "147.81.41.188:7196".parse().unwrap());
        }
        observer.record_observation(PeerId::random(), "147.81.41.188:9001".parse().unwrap());

        assert_eq!(
            observer.primary_external_address(),
            Some("147.81.41.188:9001".parse().unwrap())
        );
    }

    #[test]
    fn test_consensus_tie_breaks_deterministically_by_address() {
        let mut observer = AddressObserver::new();
        // Two permitted-port addresses with EQUAL vote counts. The winner was
        // previously HashMap-iteration order (nondeterministic); it must now be
        // the deterministically-lower address regardless of insertion order.
        let lower: SocketAddr = "203.0.113.5:9001".parse().unwrap();
        let higher: SocketAddr = "203.0.113.9:9001".parse().unwrap();

        // Insert higher first, lower second -- equal totals either way.
        observer.record_observation(PeerId::random(), higher);
        observer.record_observation(PeerId::random(), lower);

        assert_eq!(observer.external_addresses().len(), 2);
        assert_eq!(observer.primary_external_address(), Some(lower));

        // And with insertion reversed, the same address still wins.
        let mut observer2 = AddressObserver::new();
        observer2.record_observation(PeerId::random(), lower);
        observer2.record_observation(PeerId::random(), higher);
        assert_eq!(observer2.primary_external_address(), Some(lower));
    }

    #[test]
    fn test_set_listen_ports_re_filters_stored_observations() {
        let mut observer = AddressObserver::new();
        // Accept-all observer (wasm transport): the ephemeral observation is
        // stored.
        observer.record_observation(PeerId::random(), "147.81.41.188:7196".parse().unwrap());
        assert_eq!(observer.external_addresses().len(), 1);

        // Once the listener set is known, the stored observation is excluded
        // from the consensus immediately.
        observer.set_listen_ports(vec![9001]);
        assert!(observer.external_addresses().is_empty());
    }

    #[test]
    fn test_listen_ports_from_multiaddrs() {
        let addrs: Vec<Multiaddr> = vec![
            "/ip4/0.0.0.0/tcp/9001".parse().unwrap(),
            "/ip6/::/tcp/9001".parse().unwrap(),
            "/ip4/0.0.0.0/udp/9002".parse().unwrap(),
        ];
        let ports = listen_ports_from_multiaddrs(&addrs);
        assert_eq!(ports, vec![9001, 9002]);
    }

    #[test]
    fn test_address_confirmation_count() {
        let mut observer = AddressObserver::new();
        let peer = PeerId::random();
        let addr: SocketAddr = "1.2.3.4:1234".parse().unwrap();

        // Record same observation multiple times
        observer.record_observation(peer, addr);
        observer.record_observation(peer, addr);
        observer.record_observation(peer, addr);

        let obs = observer.all_observations();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].confirmation_count, 3);
    }

    #[test]
    fn test_extract_socket_addr() {
        let addr: Multiaddr = "/ip4/1.2.3.4/tcp/1234".parse().unwrap();
        let socket_addr = ConnectionTracker::extract_socket_addr(&addr);
        assert_eq!(socket_addr, Some("1.2.3.4:1234".parse().unwrap()));
    }

    #[test]
    fn tracker_keeps_direct_and_relayed_connections_separate() {
        let mut tracker = ConnectionTracker::new();
        let peer = PeerId::random();
        let direct: Multiaddr = "/ip4/192.168.1.20/tcp/9001".parse().unwrap();
        let relayed: Multiaddr = "/ip4/203.0.113.20/tcp/9001/p2p-circuit".parse().unwrap();
        let local: Multiaddr = "/ip4/192.168.1.5/tcp/9001".parse().unwrap();

        tracker.add_connection(peer, direct.clone(), local.clone(), "direct".to_string());
        tracker.add_connection(peer, relayed.clone(), local, "relay".to_string());

        assert_eq!(tracker.all_connections().len(), 2);
        assert_eq!(
            tracker
                .get_connection_by_id(&peer, "direct")
                .map(|connection| &connection.remote_addr),
            Some(&direct)
        );
        assert_eq!(
            tracker
                .get_connection_by_id(&peer, "relay")
                .map(|connection| &connection.remote_addr),
            Some(&relayed)
        );

        tracker.remove_connection_by_id(&peer, "direct");
        assert!(tracker.get_connection_by_id(&peer, "direct").is_none());
        assert!(tracker.get_connection_by_id(&peer, "relay").is_some());
        assert_eq!(tracker.all_connections().len(), 1);

        tracker.remove_connection(&peer);
        assert!(tracker.all_connections().is_empty());
    }
}
