// Address Observation and Consensus
//
// Tracks observations from multiple peers to determine our actual external addresses.
// Implements consensus-based address discovery without relying on external STUN servers.

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use web_time::{SystemTime, UNIX_EPOCH};

/// Current Unix time in seconds, or None if the clock reads before the epoch.
/// Never panics; logs once at debug level on failure.
fn unix_now_secs() -> Option<u64> {
    static WARNED: AtomicBool = AtomicBool::new(false);
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => Some(d.as_secs()),
        Err(_) => {
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::debug!("system clock before UNIX_EPOCH; skipping time-based bookkeeping");
            }
            None
        }
    }
}

/// Upper bound on components of an identify-observed address we are willing
/// to inspect (ip, transport, quic/ws suffix, p2p id fit well inside this).
const MAX_OBSERVED_ADDR_COMPONENTS: usize = 6;

/// Whether a peer-reported socket can name our external endpoint at all.
/// Rejects the placeholder and non-unicast classes: unspecified ip, port 0,
/// multicast, and IPv4 broadcast. Loopback and private ranges are accepted:
/// same-host and LAN peers legitimately observe us there.
pub fn is_plausible_external_socket(socket: &SocketAddr) -> bool {
    let ip = socket.ip();
    if ip.is_unspecified() || ip.is_multicast() || socket.port() == 0 {
        return false;
    }
    !matches!(ip, std::net::IpAddr::V4(v4) if v4.is_broadcast())
}

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
    /// Operator-configured external address (T14). When set, it takes primacy
    /// over every peer observation: it is always the consensus primary and the
    /// first address reported, so an ephemeral or NAT-mangled observed port can
    /// never outrank the configured external endpoint.
    configured_external: Option<SocketAddr>,
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
            configured_external: None,
            cached_external_addresses: Vec::new(),
            listen_ports: Vec::new(),
        }
    }

    /// Replace the local listener port set and remove observations that are no
    /// longer eligible for advertisement.
    pub fn set_listen_ports(&mut self, ports: impl IntoIterator<Item = u16>) {
        self.listen_ports = ports.into_iter().collect();
        if !self.listen_ports.is_empty() {
            self.observations
                .retain(|_, observation| self.listen_ports.contains(&observation.address.port()));
        }
        self.recalculate_consensus();
    }

    /// Set or clear the operator-configured external address (T14). The
    /// configured address wins over every peer observation: it is pinned to the
    /// front of the consensus list so `primary_external_address()` — the value
    /// every advertisement and diagnostics path reads — is always the
    /// configured endpoint while one is set.
    pub fn set_configured_external(&mut self, addr: Option<SocketAddr>) {
        self.configured_external = addr;
        self.recalculate_consensus();
    }

    /// Record an observation from a peer.
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

        let now = unix_now_secs().unwrap_or(0);

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
        self.expire_at(unix_now_secs(), max_age_secs);
    }

    /// Expiry against an explicit clock reading; `None` (pre-epoch or failed
    /// clock) skips expiry for this tick.
    fn expire_at(&mut self, now: Option<u64>, max_age_secs: u64) {
        let Some(now) = now else {
            // Pre-epoch or failed clock: skip expiry for this tick.
            return;
        };

        self.observations.retain(|_, obs| {
            // Backward clock step: clamp future-stamped entries to now so
            // they count as fresh and still expire later.
            obs.timestamp = obs.timestamp.min(now);
            now.saturating_sub(obs.timestamp) < max_age_secs
        });

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
        addresses.sort_by_key(|(address, count)| (Reverse(*count), *address));

        // Cache the sorted addresses
        let mut cached: Vec<SocketAddr> = addresses.into_iter().map(|(addr, _)| addr).collect();

        // T14: the configured external address has primacy over every
        // observation. Deduplicate it out of the consensus list and pin it at
        // the front so the primary entry is always the configured endpoint.
        if let Some(configured) = self.configured_external {
            cached.retain(|addr| *addr != configured);
            cached.insert(0, configured);
        }

        self.cached_external_addresses = cached;
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
        let now = unix_now_secs().unwrap_or(0);

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

    /// Extract the socket of an identify-OBSERVED address, refusing relayed
    /// forms. When a peer sees us through a relay circuit, the observed
    /// address is `/ip4/<RELAY>/tcp/<port>/p2p/<relay>/p2p-circuit`: its
    /// ip/port name the RELAY's socket, not ours. `extract_socket_addr`
    /// ignores the circuit marker and would hand the relay's endpoint to the
    /// AddressObserver as our own external address (3-node run 2026-10-08:
    /// AWS 18.234.62.247:9001 became "our" external addr, so every dial to
    /// AWS was skipped as a self-dial).
    pub fn extract_direct_observed_socket_addr(addr: &Multiaddr) -> Option<SocketAddr> {
        use libp2p::multiaddr::Protocol;

        // Peer-supplied input: bound the component count, accept exactly one
        // ip and one transport port (the lenient extractor keeps the LAST of
        // each, so a hostile `/ip4/a/tcp/1/ip4/b/tcp/2` would pick the
        // attacker's tail), and refuse relayed forms.
        let mut ip = None;
        let mut port = None;
        for (index, protocol) in addr.iter().enumerate() {
            if index >= MAX_OBSERVED_ADDR_COMPONENTS {
                return None;
            }
            match protocol {
                Protocol::P2pCircuit => return None,
                Protocol::Ip4(a) => {
                    if ip.replace(std::net::IpAddr::V4(a)).is_some() {
                        return None;
                    }
                }
                Protocol::Ip6(a) => {
                    if ip.replace(std::net::IpAddr::V6(a)).is_some() {
                        return None;
                    }
                }
                Protocol::Tcp(p) | Protocol::Udp(p) => {
                    if port.replace(p).is_some() {
                        return None;
                    }
                }
                _ => {}
            }
        }
        let socket = SocketAddr::new(ip?, port?);
        is_plausible_external_socket(&socket).then_some(socket)
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
    fn expire_clamps_future_stamps_after_backward_clock_step() {
        let mut observer = AddressObserver::new();
        observer.record_observation(PeerId::random(), "203.0.113.5:9001".parse().unwrap());
        for obs in observer.observations.values_mut() {
            obs.timestamp = 5_000;
        }
        // Clock stepped back to 1_000: the 5_000 stamp is in the future.
        observer.expire_at(Some(1_000), 300);
        let remaining = observer.all_observations();
        assert_eq!(remaining.len(), 1, "future stamp is treated as fresh");
        assert_eq!(remaining[0].timestamp, 1_000, "timestamp clamped to now");
        // The clamped entry still expires once time moves on.
        observer.expire_at(Some(1_300), 300);
        assert!(observer.all_observations().is_empty());
    }

    #[test]
    fn pre_epoch_clock_skips_expiry_without_panic() {
        let mut observer = AddressObserver::new();
        observer.record_observation(PeerId::random(), "203.0.113.5:9001".parse().unwrap());
        for obs in observer.observations.values_mut() {
            obs.timestamp = 0;
        }
        observer.expire_at(None, 1);
        assert_eq!(observer.all_observations().len(), 1, "expiry skipped");
        assert!(unix_now_secs().is_some());
    }

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
        observer.set_listen_ports([9001]);

        let observer_peer = PeerId::random();
        observer.record_observation(observer_peer, "203.0.113.5:7196".parse().unwrap());
        assert!(observer.external_addresses().is_empty());

        observer.record_observation(observer_peer, "203.0.113.5:9001".parse().unwrap());
        assert_eq!(
            observer.primary_external_address(),
            Some("203.0.113.5:9001".parse().unwrap())
        );
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
    fn configured_external_wins_over_consensus() {
        let mut observer = AddressObserver::new();
        observer.set_listen_ports([9001]);
        let configured: SocketAddr = "147.81.41.188:9001".parse().unwrap();
        observer.set_configured_external(Some(configured));

        // Peers consistently observe a different (wrong) address; consensus
        // alone would rank it first, but the configured endpoint must win.
        let wrong: SocketAddr = "203.0.113.5:9001".parse().unwrap();
        for _ in 0..5 {
            observer.record_observation(PeerId::random(), wrong);
        }

        assert_eq!(observer.primary_external_address(), Some(configured));
        assert_eq!(observer.external_addresses().first(), Some(&configured));
        // The observed address is still reported, but only after the configured one.
        assert!(observer.external_addresses().contains(&wrong));

        // Clearing the configured address restores pure observation consensus.
        observer.set_configured_external(None);
        assert_eq!(observer.primary_external_address(), Some(wrong));
    }

    #[test]
    fn empty_listen_port_set_accepts_all_until_ports_known() {
        // Unified 2026-09-10: empty listen set = accept all (browser/wasm has no
        // listeners). Once set_listen_ports is called, non-listen ports are dropped.
        let mut observer = AddressObserver::new();
        observer.record_observation(PeerId::random(), "203.0.113.5:9001".parse().unwrap());
        assert_eq!(
            observer.primary_external_address(),
            Some("203.0.113.5:9001".parse().unwrap())
        );

        observer.set_listen_ports([9001]);
        assert_eq!(
            observer.primary_external_address(),
            Some("203.0.113.5:9001".parse().unwrap())
        );
    }

    #[test]
    fn hostile_observed_addresses_are_rejected() {
        let rejected = [
            "/ip4/0.0.0.0/tcp/9001",
            "/ip6/::/tcp/9001",
            "/ip4/203.0.113.7/tcp/0",
            "/ip4/224.0.0.1/tcp/9001",
            "/ip6/ff02::1/tcp/9001",
            "/ip4/255.255.255.255/tcp/9001",
            "/ip4/203.0.113.7/ip4/198.51.100.9/tcp/9001",
            "/ip4/203.0.113.7/tcp/1/tcp/2",
            "/ip4/203.0.113.7/udp/9001/udp/9002",
            "/dns4/example.com/tcp/9001",
            "/ip4/203.0.113.7",
            "/tcp/9001",
            "/ip4/203.0.113.7/tcp/9001/p2p-circuit",
            "/ip4/1.1.1.1/tcp/1/ws/ws/ws/ws/ws",
        ];
        for text in rejected {
            let addr: Multiaddr = text.parse().unwrap();
            assert!(
                ConnectionTracker::extract_direct_observed_socket_addr(&addr).is_none(),
                "{text}"
            );
        }
        for text in [
            "/ip4/203.0.113.7/tcp/9001",
            "/ip4/192.168.0.5/tcp/9001",
            "/ip4/127.0.0.1/tcp/9001",
            "/ip4/203.0.113.7/udp/9001/quic-v1",
            "/ip6/2001:db8::1/tcp/9001",
        ] {
            let addr: Multiaddr = text.parse().unwrap();
            assert!(
                ConnectionTracker::extract_direct_observed_socket_addr(&addr).is_some(),
                "{text}"
            );
        }
    }

    #[test]
    fn expiry_survives_future_timestamps() {
        // A clock step backwards leaves a timestamp in the future; expiry
        // must not underflow (debug panic / release wraparound).
        let mut observer = AddressObserver::new();
        observer.record_observation(PeerId::random(), "203.0.113.7:9001".parse().unwrap());
        for obs in observer.observations.values_mut() {
            obs.timestamp = u64::MAX;
        }
        observer.expire_old_observations(60);
        assert_eq!(observer.all_observations().len(), 1);
        observer.expire_old_observations(0);
        assert!(observer.all_observations().is_empty());
    }

    #[test]
    fn observation_map_is_bounded_by_expiry() {
        let mut observer = AddressObserver::new();
        for _ in 0..500 {
            observer.record_observation(PeerId::random(), "203.0.113.7:9001".parse().unwrap());
        }
        assert_eq!(observer.all_observations().len(), 500);
        observer.expire_old_observations(0);
        assert!(observer.all_observations().is_empty());
        assert!(observer.external_addresses().is_empty());
    }

    #[test]
    fn observed_circuit_address_is_not_a_direct_socket() {
        let relayed: Multiaddr =
            "/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31/p2p-circuit"
                .parse()
                .unwrap();
        assert!(ConnectionTracker::extract_direct_observed_socket_addr(&relayed).is_none());
        let direct: Multiaddr = "/ip4/147.81.41.188/tcp/9001".parse().unwrap();
        assert_eq!(
            ConnectionTracker::extract_direct_observed_socket_addr(&direct),
            Some("147.81.41.188:9001".parse().unwrap())
        );
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
