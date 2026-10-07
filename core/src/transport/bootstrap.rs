// Bootstrap Recovery & Fallback System
//
// Resilient connectivity to ledger-derived candidates (#469): there are no
// static, environment-supplied, or compiled-in seed addresses. Candidates are
// added at runtime from the peer ledger (invite-seeded) via
// `add_bootstrap_node`.
//
// - Exponential backoff on connection failures
// - Local network peer discovery as fallback
// - Dynamic relay discovery from connected peers
// - WebSocket fallback for cellular networks (P0_NETWORK_001)
//
// P0_NETWORK_002: Enhanced error diagnostics for relay connectivity failures

use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::transport::internet::{InternetRelay, InternetTransportError};
use crate::transport::relay_health::{RelayDiscovery, RelayFallback, RelayMetrics};
use crate::transport::swarm::SwarmHandle;
use libp2p::{Multiaddr, PeerId};
use std::collections::VecDeque;
use tracing::{debug, info, warn};
use web_time::{Duration, SystemTime};

/// Configuration for bootstrap recovery
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Maximum concurrent bootstrap attempts
    pub max_concurrent_attempts: usize,
    /// Initial backoff delay before first retry
    pub initial_backoff: Duration,
    /// Maximum backoff delay between retries
    pub max_backoff: Duration,
    /// Backoff multiplier (exponential)
    pub backoff_multiplier: f64,
    /// Maximum retries per bootstrap node before giving up
    pub max_retries_per_node: u32,
    /// Timeout for individual connection attempts
    pub connect_timeout: Duration,
    /// Whether to enable local network discovery fallback
    pub enable_local_discovery: bool,
    /// Circuit breaker configuration for relay failures
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            max_concurrent_attempts: 3,
            initial_backoff: Duration::from_secs(2),
            max_backoff: Duration::from_secs(300),
            backoff_multiplier: 1.5,
            max_retries_per_node: 5,
            connect_timeout: Duration::from_secs(10),
            enable_local_discovery: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }
}

/// State of bootstrap connection progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapState {
    /// No bootstrap attempted yet
    Idle,
    /// Attempting to connect to bootstrap nodes
    Connecting,
    /// Successfully connected to at least one bootstrap node
    Connected,
    /// All bootstrap nodes failed, trying fallback discovery
    FallbackDiscovery,
    /// Connected via fallback (local network or discovered peer)
    FallbackConnected,
    /// All connection attempts exhausted
    Failed,
}

/// Represents a bootstrap node
#[derive(Debug, Clone)]
struct BootstrapNode {
    addr: Multiaddr,
    peer_id: Option<PeerId>,
    attempts: u32,
    last_attempt: Option<SystemTime>,
    last_failure: Option<String>,
    connected: bool,
}

/// Bootstrap recovery manager — coordinates resilient bootstrap connectivity
pub struct BootstrapManager {
    config: BootstrapConfig,
    state: BootstrapState,
    nodes: VecDeque<BootstrapNode>,
    relay_discovery: RelayDiscovery,
    relay_fallback: RelayFallback,
    /// Circuit breaker for tracking relay failures (P0_NETWORK_001)
    circuit_breaker: CircuitBreakerManager,
    connected_count: usize,
}

impl BootstrapManager {
    /// Create a new bootstrap manager with the given config
    pub fn new(config: BootstrapConfig) -> Self {
        // No static seeds: the candidate set starts empty and is filled from
        // the ledger through `add_bootstrap_node`.
        let relay_discovery = RelayDiscovery::new(Vec::new());
        let relay_fallback = RelayFallback::new(config.max_retries_per_node);
        let circuit_breaker = CircuitBreakerManager::new(config.circuit_breaker_config.clone());
        let nodes: VecDeque<BootstrapNode> = VecDeque::new();

        Self {
            config,
            state: BootstrapState::Idle,
            nodes,
            relay_discovery,
            relay_fallback,
            circuit_breaker,
            connected_count: 0,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BootstrapConfig::default())
    }

    /// Get current bootstrap state
    pub fn state(&self) -> &BootstrapState {
        &self.state
    }

    /// Get number of connected bootstrap nodes
    pub fn connected_count(&self) -> usize {
        self.connected_count
    }

    /// Get total number of configured bootstrap nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get the relay discovery system
    pub fn relay_discovery(&self) -> &RelayDiscovery {
        &self.relay_discovery
    }

    /// Get the relay discovery system (mutable)
    pub fn relay_discovery_mut(&mut self) -> &mut RelayDiscovery {
        &mut self.relay_discovery
    }

    /// Add a bootstrap node address
    pub fn add_bootstrap_node(&mut self, addr: Multiaddr) {
        let exists = self.nodes.iter().any(|n| n.addr == addr);
        if !exists {
            self.relay_discovery.add_fallback_relay(addr.clone());
            info!("Added bootstrap node: {}", addr);
            self.nodes.push_back(BootstrapNode {
                addr,
                peer_id: None,
                attempts: 0,
                last_attempt: None,
                last_failure: None,
                connected: false,
            });
        }
    }

    /// Attempt bootstrap connection via the internet relay and swarm
    ///
    /// Tries each configured node with exponential backoff, respecting
    /// circuit breaker state to avoid hammering failed relays. Implements
    /// multi-transport fallback order: UDP → TCP → WebSocket.
    /// Falls back to WebSocket on standard ports when cellular networks block
    /// non-standard ports (P0_NETWORK_001, P0_NETWORK_002).
    pub async fn bootstrap(
        &mut self,
        relay: &InternetRelay,
        swarm: &SwarmHandle,
    ) -> Result<PeerId, InternetTransportError> {
        self.state = BootstrapState::Connecting;

        loop {
            let candidate = self.next_connectable_node();
            match candidate {
                Some(node) => {
                    let addr = node.addr.clone();
                    let addr_str = addr.to_string();

                    // P0_NETWORK_001: Check circuit breaker before attempting
                    if !self.circuit_breaker.allow_request(&addr_str) {
                        debug!("Circuit breaker blocked attempt to {}", addr);
                        self.record_failure(&addr, "circuit breaker open");
                        continue;
                    }

                    let delay = self.backoff_for_node(node);

                    if delay > Duration::ZERO {
                        debug!(
                            "Bootstrap backoff: waiting {:?} before retrying {}",
                            delay, addr
                        );
                        tokio::time::sleep(delay).await;
                    }

                    let peer_id = PeerId::random(); // Promiscuous: accept whatever peer presents
                    self.record_attempt(&addr);

                    // P0_NETWORK_002: Try multi-transport fallback order
                    // First try direct connection via swarm
                    match relay
                        .connect_to_relay_via_swarm(peer_id, addr.clone(), swarm)
                        .await
                    {
                        Ok(()) => {
                            info!(
                                "Bootstrap connected to {} via direct swarm connection",
                                addr
                            );
                            self.record_success(&addr, peer_id);
                            self.circuit_breaker.record_success(&addr_str);
                            self.relay_discovery.record_success(&peer_id, 0);
                            self.connected_count += 1;
                            self.state = BootstrapState::Connected;
                            return Ok(peer_id);
                        }
                        Err(e) => {
                            let err_str = e.to_string();
                            warn!("Direct swarm connection failed for {}: {}", addr, err_str);

                            // P0_NETWORK_002: Try WebSocket fallback if this is a WebSocket-capable address
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.is_websocket_address(&addr) {
                                debug!("Attempting WebSocket fallback for {}", addr);
                                match self.try_websocket_connection(&addr).await {
                                    Ok(()) => {
                                        info!(
                                            "Bootstrap connected to {} via WebSocket fallback",
                                            addr
                                        );
                                        self.record_success(&addr, peer_id);
                                        self.circuit_breaker.record_success(&addr_str);
                                        self.relay_discovery.record_success(&peer_id, 0);
                                        self.connected_count += 1;
                                        self.state = BootstrapState::Connected;
                                        return Ok(peer_id);
                                    }
                                    Err(ws_err) => {
                                        let ws_err_str = ws_err.to_string();
                                        warn!(
                                            "WebSocket fallback failed for {}: {}",
                                            addr, ws_err_str
                                        );
                                        self.record_failure(
                                            &addr,
                                            &format!(
                                                "Direct: {}, WebSocket: {}",
                                                err_str, ws_err_str
                                            ),
                                        );
                                        self.circuit_breaker.record_failure(&addr_str, &ws_err_str);
                                        self.relay_discovery.record_failure(&peer_id, &ws_err_str);
                                    }
                                }
                            } else {
                                // Not a WebSocket address, record the original failure
                                self.record_failure(&addr, &err_str);
                                self.circuit_breaker.record_failure(&addr_str, &err_str);
                                self.relay_discovery.record_failure(&peer_id, &err_str);
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                // On WASM, WebSocket fallback is not available
                                self.record_failure(&addr, &err_str);
                                self.circuit_breaker.record_failure(&addr_str, &err_str);
                                self.relay_discovery.record_failure(&peer_id, &err_str);
                            }
                        }
                    }
                }
                None => {
                    // All primary nodes exhausted — try fallback discovery
                    if self.state != BootstrapState::FallbackDiscovery {
                        info!(
                            "All primary bootstrap nodes exhausted, attempting fallback discovery"
                        );
                        self.state = BootstrapState::FallbackDiscovery;
                        let discovered = self.discover_fallback_nodes();
                        for addr in discovered {
                            self.add_bootstrap_node(addr);
                        }
                        if self.nodes.iter().any(|n| n.attempts == 0 && !n.connected) {
                            continue; // Try newly discovered nodes
                        }
                    }

                    self.state = BootstrapState::Failed;
                    return Err(InternetTransportError::ConnectionFailed(
                        "All bootstrap nodes failed after retries".to_string(),
                    ));
                }
            }
        }
    }

    /// Record a successful connection to a bootstrap node
    pub fn record_success(&mut self, addr: &Multiaddr, peer_id: PeerId) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.connected = true;
            node.peer_id = Some(peer_id);
        }
        self.relay_fallback.reset_attempts();
    }

    /// Record a failed connection attempt
    pub fn record_failure(&mut self, addr: &Multiaddr, reason: &str) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.last_failure = Some(reason.to_string());
        }
    }

    /// Record a connection attempt
    pub fn record_attempt(&mut self, addr: &Multiaddr) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.attempts += 1;
            node.last_attempt = Some(SystemTime::now());
        }
        self.relay_fallback.record_attempt(addr);
    }

    /// Calculate exponential backoff delay for a node
    fn backoff_for_node(&self, node: &BootstrapNode) -> Duration {
        if node.attempts == 0 {
            return Duration::ZERO;
        }
        let delay_secs = self.config.initial_backoff.as_secs_f64()
            * self
                .config
                .backoff_multiplier
                .powi(node.attempts as i32 - 1);
        let capped = delay_secs.min(self.config.max_backoff.as_secs_f64());
        Duration::from_secs_f64(capped)
    }

    /// Find the next node eligible for a connection attempt
    fn next_connectable_node(&self) -> Option<&BootstrapNode> {
        self.nodes
            .iter()
            .find(|n| !n.connected && self.relay_fallback.should_retry(&n.addr))
    }

    /// Discover fallback candidates from the local network.
    ///
    /// Static DNS / WebSocket / hardcoded sources were removed (#469); the only
    /// remaining source is local discovery, which the swarm's mDNS layer drives.
    fn discover_fallback_nodes(&self) -> Vec<Multiaddr> {
        let mut discovered = Vec::new();

        // Local network mDNS discovery
        if self.config.enable_local_discovery {
            if let Ok(addrs) = discover_local_peers() {
                info!("Local discovery found {} fallback nodes", addrs.len());
                discovered.extend(addrs);
            }
        }

        discovered
    }

    /// Check if address is WebSocket-capable
    #[cfg(not(target_arch = "wasm32"))]
    fn is_websocket_address(&self, addr: &Multiaddr) -> bool {
        addr.iter().any(|proto| {
            matches!(
                proto,
                libp2p::multiaddr::Protocol::Ws(_) | libp2p::multiaddr::Protocol::Wss(_)
            )
        })
    }

    /// Try WebSocket connection as fallback
    #[cfg(not(target_arch = "wasm32"))]
    async fn try_websocket_connection(
        &self,
        addr: &Multiaddr,
    ) -> Result<(), InternetTransportError> {
        // Import WebSocket transport
        use crate::transport::websocket::{diagnose_websocket_error, WebSocketTransport};

        info!("Attempting WebSocket connection to {}", addr);

        // Create WebSocket transport from Multiaddr
        let mut ws_transport = WebSocketTransport::from_multiaddr(addr)
            .map_err(|e| diagnose_websocket_error(e, addr))?;

        // Attempt connection
        ws_transport
            .connect()
            .await
            .map_err(|e| diagnose_websocket_error(e, addr))?;

        // Connection successful (we don't actually use it here, just testing connectivity)
        info!("WebSocket connection successful to {}", addr);

        Ok(())
    }

    /// Get a reference to the circuit breaker manager
    pub fn circuit_breaker(&self) -> &CircuitBreakerManager {
        &self.circuit_breaker
    }

    /// Get healthy relays from the circuit breaker.
    /// Returns addresses of relays whose circuit breaker state is Closed,
    /// meaning they are currently healthy and available for connections.
    pub fn get_healthy_relays(&self) -> Vec<String> {
        self.circuit_breaker.get_healthy_relays()
    }

    /// Get all relay statistics from the relay discovery system.
    /// Returns statistics for all known relays, including health metrics.
    pub fn get_all_relay_stats(&self) -> Vec<(PeerId, RelayMetrics)> {
        self.relay_discovery
            .get_all_metrics()
            .into_iter()
            .map(|(peer_id, metrics)| (*peer_id, metrics.clone()))
            .collect()
    }

    /// Get fallback relay addresses for connectivity when primary relays fail.
    /// These are pre-configured addresses that serve as last-resort bootstrap targets.
    pub fn get_fallback_relay_addresses(&self) -> Vec<Multiaddr> {
        self.relay_discovery.get_fallback_relays().to_vec()
    }

    /// Reset all circuit breakers (e.g., on network type change)
    pub fn reset_circuit_breakers(&self) {
        self.circuit_breaker.reset_all();
    }
}

/// Local network peer discovery
/// Uses mDNS to find SCMessenger peers on the local network
fn discover_local_peers() -> Result<Vec<Multiaddr>, String> {
    // mDNS discovery is handled by libp2p's built-in mDNS behaviour
    // which fires PeerDiscovered events for local peers.
    // This function provides addresses that the swarm's mDNS layer
    // would discover — actual discovery happens at the swarm level.
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_config_defaults() {
        let config = BootstrapConfig::default();
        assert_eq!(config.max_retries_per_node, 5);
        assert_eq!(config.initial_backoff, Duration::from_secs(2));
        assert_eq!(config.max_backoff, Duration::from_secs(300));
        assert!(config.enable_local_discovery);
    }

    #[test]
    fn test_manager_ignores_env_seed() {
        // SC_BOOTSTRAP_NODES is no longer read; a fresh manager is always empty.
        assert_eq!(BootstrapManager::with_defaults().total_nodes(), 0);
    }

    #[test]
    fn test_bootstrap_manager_creation() {
        // No static seeds: a fresh manager is empty until a node is added.
        assert_eq!(BootstrapManager::with_defaults().total_nodes(), 0);
        let mut mgr = BootstrapManager::with_defaults();
        let addr: Multiaddr = "/ip4/10.0.0.1/tcp/9001".parse().unwrap();
        mgr.add_bootstrap_node(addr);
        assert_eq!(*mgr.state(), BootstrapState::Idle);
        assert!(mgr.total_nodes() > 0);
        assert_eq!(mgr.connected_count(), 0);
    }

    #[test]
    fn test_bootstrap_manager_add_node() {
        let mut mgr = BootstrapManager::with_defaults();
        let initial = mgr.total_nodes();
        let addr: Multiaddr = "/ip4/10.0.0.1/tcp/9001".parse().unwrap();
        mgr.add_bootstrap_node(addr);
        assert_eq!(mgr.total_nodes(), initial + 1);
    }

    #[test]
    fn test_bootstrap_manager_no_duplicate() {
        let mut mgr = BootstrapManager::with_defaults();
        let initial = mgr.total_nodes();
        let addr: Multiaddr = "/ip4/10.0.0.1/tcp/9001".parse().unwrap();
        mgr.add_bootstrap_node(addr.clone());
        mgr.add_bootstrap_node(addr);
        assert_eq!(mgr.total_nodes(), initial + 1);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = BootstrapConfig::default();
        let mgr = BootstrapManager::new(config);

        let node = BootstrapNode {
            addr: "/ip4/1.2.3.4/tcp/9001".parse().unwrap(),
            peer_id: None,
            attempts: 3,
            last_attempt: Some(SystemTime::now()),
            last_failure: None,
            connected: false,
        };

        let delay = mgr.backoff_for_node(&node);
        assert!(delay > Duration::ZERO);
        assert!(delay <= Duration::from_secs(300));
    }

    #[test]
    fn test_local_discovery() {
        let addrs = discover_local_peers().unwrap();
        // Local discovery relies on libp2p mDNS, returns empty here
        assert!(addrs.is_empty());
    }
}
