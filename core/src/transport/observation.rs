// Address Observation and Consensus
//
// Tracks observations from multiple peers to determine our actual external addresses.
// Implements consensus-based address discovery without relying on external STUN servers.

use libp2p::{Multiaddr, PeerId};
use std::cmp::Reverse;
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
    /// Ports currently bound by this node. This is the single source of truth
    /// for whether an observed address can be advertised.
    listen_ports: Vec<u16>,
    /// Cached consensus result (recalculated when observations change)
    cached_external_addresses: Vec<SocketAddr>,
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
            listen_ports: Vec::new(),
            cached_external_addresses: Vec::new(),
        }
    }

    /// Replace the local listener port set and remove observations that are no
    /// longer eligible for advertisement.
    pub fn set_listen_ports(&mut self, ports: impl IntoIterator<Item = u16>) {
        self.listen_ports = ports.into_iter().collect();
        self.observations
            .retain(|_, observation| self.listen_ports.contains(&observation.address.port()));
        self.recalculate_consensus();
    }

    /// Record an observation from a peer. Observations are accepted only for
    /// ports this node currently listens on; an empty allowlist fails closed.
    pub fn record_observation(&mut self, observer: PeerId, address: SocketAddr) {
        if !self.listen_ports.contains(&address.port()) {
            // A newer observation from this peer that uses a non-listening port
            // invalidates its previous address. Retaining that old value would
            // keep a stale endpoint eligible for advertisement.
            if self.observations.remove(&observer).is_some() {
                self.recalculate_consensus();
            }
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
            *address_counts.entry(obs.address).or_insert(0) += obs.confirmation_count;
        }

        // Sort by count (most observed first)
        let mut addresses: Vec<(SocketAddr, u32)> = address_counts.into_iter().collect();
        addresses.sort_by_key(|(address, count)| (Reverse(*count), *address));

        // Cache the sorted addresses
        self.cached_external_addresses = addresses.into_iter().map(|(addr, _)| addr).collect();
    }
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
    fn address_observer_contract() {
        let mut observer = AddressObserver::new();
        observer.set_listen_ports([1234, 5678]);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let addr1: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let addr2: SocketAddr = "5.6.7.8:5678".parse().unwrap();

        for peer in [peer1, peer2] {
            observer.record_observation(peer, addr1);
        }
        observer.record_observation(PeerId::random(), addr2);

        assert_eq!(observer.primary_external_address(), Some(addr1));
        assert_eq!(observer.external_addresses(), &[addr1, addr2]);
    }

    #[test]
    fn non_listen_port_observations_are_rejected() {
        let mut observer = AddressObserver::new();
        let observer_peer = PeerId::random();
        observer.set_listen_ports([9001]);

        observer.record_observation(observer_peer, "203.0.113.5:9001".parse().unwrap());
        observer.record_observation(observer_peer, "203.0.113.5:7196".parse().unwrap());
        assert!(observer.external_addresses().is_empty());

        observer.record_observation(observer_peer, "203.0.113.5:9001".parse().unwrap());
        assert_eq!(
            observer.primary_external_address(),
            Some("203.0.113.5:9001".parse().unwrap())
        );
    }

    #[test]
    fn removing_a_listen_port_removes_its_observations() {
        let mut observer = AddressObserver::new();
        let peer = PeerId::random();
        observer.set_listen_ports([9001, 9002]);
        observer.record_observation(peer, "203.0.113.5:9001".parse().unwrap());
        assert!(observer.primary_external_address().is_some());

        observer.set_listen_ports([9002]);
        assert!(observer.external_addresses().is_empty());

        observer.set_listen_ports([]);
        assert!(observer.external_addresses().is_empty());
    }

    #[test]
    fn empty_listen_port_set_fails_closed() {
        let mut observer = AddressObserver::new();
        observer.record_observation(PeerId::random(), "203.0.113.5:9001".parse().unwrap());
        assert!(observer.external_addresses().is_empty());
    }

    #[test]
    fn test_extract_socket_addr() {
        let cases = [
            ("/ip4/1.2.3.4/tcp/1234", Some("1.2.3.4:1234")),
            ("/ip6/2001:db8::1/udp/5678", Some("[2001:db8::1]:5678")),
            ("/dns4/example.com/tcp/1234", None),
        ];
        for (multiaddr, expected) in cases {
            let addr: Multiaddr = multiaddr.parse().unwrap();
            let actual =
                ConnectionTracker::extract_socket_addr(&addr).map(|socket| socket.to_string());
            assert_eq!(actual, expected.map(str::to_string));
        }
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
