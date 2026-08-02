// Phase 4D: NAT observation and relay-circuit bookkeeping.
//
// Direct hole punching is provided by libp2p DCUtR in transport/swarm.rs.
// This module must not report simulated direct-connect success.
//
// This module provides:
// - NAT type detection (Open, Restricted, Symmetric, Unknown)
// - Relay circuit bookkeeping and fallback
// - STUN server support for external address discovery
// - Configurable timeouts and retry logic

use super::swarm::SwarmHandle;
use libp2p::PeerId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};
use web_time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone, Error)]
pub enum NatTraversalError {
    #[error("NAT probe failed: {0}")]
    ProbesFailed(String),
    #[error("No external address detected")]
    NoExternalAddress,
    #[error("Relay circuit failed: {0}")]
    RelayCircuitFailed(String),
    #[error("Timeout waiting for peer response")]
    Timeout,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Peer connection failed: {0}")]
    PeerConnectionFailed(String),
    #[error("STUN server error: {0}")]
    StunError(String),
}

// ============================================================================
// NAT TYPE DETECTION
// ============================================================================

/// Result of NAT type probing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT, directly reachable from internet
    Open,
    /// Full cone NAT (port predictable)
    FullCone,
    /// Address-restricted cone NAT (port predictable)
    AddressRestrictedCone,
    /// Port-restricted cone NAT (port unpredictable)
    PortRestrictedCone,
    /// Symmetric NAT (both address and port unpredictable)
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

/// Peer-assisted address discovery
///
/// Uses other mesh nodes to discover external address without external dependencies.
/// Protocol:
/// 1. Send "what's my address?" request to mesh peers
/// 2. Peers respond with observed source IP:port
/// 3. Aggregate responses to determine external address
///
/// Not yet wired into the active NAT-traversal path outside this module.
#[allow(dead_code)]
pub struct PeerAddressDiscovery {
    /// Known mesh peers who can report our address
    peer_reflectors: Vec<String>,
    /// Timeout for address discovery
    timeout_secs: u64,
    /// Minimum peer responses needed for consensus
    min_responses: u32,
}

impl PeerAddressDiscovery {
    /// Create with specific peer reflectors
    pub fn with_peers(peer_reflectors: Vec<String>, timeout_secs: u64) -> Self {
        Self {
            peer_reflectors,
            timeout_secs,
            min_responses: 2,
        }
    }

    /// Detect NAT type by asking multiple mesh peers for observed address
    pub async fn detect_nat_type(
        &self,
        swarm_handle: &SwarmHandle,
    ) -> Result<NatType, NatTraversalError> {
        if self.peer_reflectors.is_empty() {
            return Err(NatTraversalError::ProbesFailed(
                "No peer reflectors configured".to_string(),
            ));
        }

        let mut detected_addresses = Vec::new();
        let mut detected_ports = Vec::new();

        // Query multiple mesh peers using libp2p request-response protocol
        for (i, peer_id_str) in self
            .peer_reflectors
            .iter()
            .enumerate()
            .take(self.min_responses as usize + 1)
        {
            debug!("Querying peer reflector {} ({})", i + 1, peer_id_str);

            // Parse peer ID
            let peer_id = match peer_id_str.parse::<PeerId>() {
                Ok(id) => id,
                Err(e) => {
                    debug!("Failed to parse peer ID {}: {}", peer_id_str, e);
                    continue;
                }
            };

            // Make actual libp2p request-response call
            match swarm_handle.request_address_reflection(peer_id).await {
                Ok(observed_addr_str) => {
                    // Parse the observed address
                    if let Ok(socket_addr) = observed_addr_str.parse::<SocketAddr>() {
                        detected_addresses.push(socket_addr.ip());
                        detected_ports.push(socket_addr.port());
                        info!("Peer {} observed us at {}", peer_id_str, socket_addr);
                    } else {
                        debug!("Failed to parse observed address: {}", observed_addr_str);
                    }
                }
                Err(e) => {
                    debug!(
                        "Address reflection request to {} failed: {}",
                        peer_id_str, e
                    );
                    // Continue with other peers
                }
            }
        }

        if detected_addresses.is_empty() {
            return Err(NatTraversalError::NoExternalAddress);
        }

        // Determine NAT type based on address/port consistency
        let nat_type = if detected_addresses.len() == 1 && detected_ports.len() == 1 {
            NatType::Open
        } else if detected_addresses
            .iter()
            .all(|a| a == &detected_addresses[0])
        {
            // All addresses same, check ports
            if detected_ports.iter().all(|p| p == &detected_ports[0]) {
                NatType::FullCone
            } else {
                // Ports differ
                NatType::PortRestrictedCone
            }
        } else {
            // Addresses differ, must be symmetric
            NatType::Symmetric
        };

        info!("Detected NAT type: {:?}", nat_type);
        Ok(nat_type)
    }

    /// Get external address from mesh peers (peer-assisted discovery)
    pub async fn get_external_address(
        &self,
        swarm_handle: &SwarmHandle,
    ) -> Result<SocketAddr, NatTraversalError> {
        if self.peer_reflectors.is_empty() {
            return Err(NatTraversalError::StunError(
                "No peer reflectors configured".to_string(),
            ));
        }

        // Query mesh peer using libp2p request-response protocol
        let peer_id_str = &self.peer_reflectors[0];

        debug!(
            "Querying peer reflector {} for external address",
            peer_id_str
        );

        // Parse peer ID
        let peer_id = peer_id_str
            .parse::<PeerId>()
            .map_err(|e| NatTraversalError::StunError(format!("Invalid peer ID: {}", e)))?;

        // Make actual libp2p request-response call
        let observed_addr_str = swarm_handle
            .request_address_reflection(peer_id)
            .await
            .map_err(|e| {
                NatTraversalError::StunError(format!("Address reflection failed: {}", e))
            })?;

        // Parse the observed address
        let addr: SocketAddr =
            observed_addr_str
                .parse()
                .map_err(|e: std::net::AddrParseError| {
                    NatTraversalError::StunError(format!("Failed to parse address: {}", e))
                })?;

        info!(
            "Received address reflection from peer {}: {}",
            peer_id_str, addr
        );
        debug!("External address from peer: {}", addr);
        Ok(addr)
    }
}

// ============================================================================
// RELAY CIRCUIT
// ============================================================================

/// Relay circuit for when a direct libp2p DCUtR connection fails
#[derive(Debug, Clone)]
pub struct RelayCircuit {
    /// Source peer ID
    pub source_peer_id: PeerId,
    /// Destination peer ID
    pub dest_peer_id: PeerId,
    /// Relay peer ID (the relaying node)
    pub relay_peer_id: PeerId,
    /// Circuit creation timestamp (unix seconds)
    pub created_at: u64,
    /// Total bytes relayed
    pub bytes_relayed: u64,
    /// Is this circuit active?
    pub active: bool,
}

// ============================================================================
// NAT CONFIGURATION
// ============================================================================

/// NAT traversal configuration
#[derive(Debug, Clone)]
pub struct NatConfig {
    /// Peer reflectors for address discovery (mesh peers, not external servers)
    /// These are libp2p peer IDs of mesh nodes that provide address reflection
    pub peer_reflectors: Vec<String>,
    /// Timeout for relay circuit establishment (seconds)
    pub relay_timeout: u64,
    /// Timeout per attempt (seconds)
    pub attempt_timeout: u64,
    /// Enable relay fallback
    pub enable_relay_fallback: bool,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            // Peer reflectors populated dynamically from connected mesh peers
            // Bootstrap nodes and web deploys are prime candidates
            peer_reflectors: vec![],
            relay_timeout: 30,
            attempt_timeout: 10,
            enable_relay_fallback: true,
        }
    }
}

// ============================================================================
// MAIN NAT TRAVERSAL STRUCT
// ============================================================================

/// NAT traversal coordinator
pub struct NatTraversal {
    config: NatConfig,
    nat_type: Arc<RwLock<NatType>>,
    relay_circuits: Arc<RwLock<HashMap<String, RelayCircuit>>>,
    external_address: Arc<RwLock<Option<SocketAddr>>>,
}

impl NatTraversal {
    /// Create a new NAT traversal instance
    pub fn new(config: NatConfig) -> Result<Self, NatTraversalError> {
        Ok(Self {
            config,
            nat_type: Arc::new(RwLock::new(NatType::Unknown)),
            relay_circuits: Arc::new(RwLock::new(HashMap::new())),
            external_address: Arc::new(RwLock::new(None)),
        })
    }

    /// Detect NAT type and external address using peer-assisted discovery
    pub async fn probe_nat(
        &self,
        swarm_handle: &SwarmHandle,
    ) -> Result<NatType, NatTraversalError> {
        let discovery = PeerAddressDiscovery::with_peers(
            self.config.peer_reflectors.clone(),
            self.config.attempt_timeout,
        );

        let nat_type = discovery.detect_nat_type(swarm_handle).await?;
        *self.nat_type.write() = nat_type;

        let external_addr = discovery.get_external_address(swarm_handle).await?;
        *self.external_address.write() = Some(external_addr);

        info!(
            "Peer-assisted NAT discovery complete: {:?} at {}",
            nat_type, external_addr
        );
        Ok(nat_type)
    }

    /// Get current NAT type
    pub fn get_nat_type(&self) -> NatType {
        *self.nat_type.read()
    }

    /// Get external address
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        *self.external_address.read()
    }

    /// Establish relay circuit (fallback when direct DCUtR fails)
    pub async fn establish_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Result<(), NatTraversalError> {
        if !self.config.enable_relay_fallback {
            return Err(NatTraversalError::RelayCircuitFailed(
                "Relay fallback disabled".to_string(),
            ));
        }

        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);

        let circuit = RelayCircuit {
            source_peer_id: local_peer_id,
            dest_peer_id: remote_peer_id,
            relay_peer_id,
            created_at: current_unix_timestamp(),
            bytes_relayed: 0,
            active: true,
        };

        self.relay_circuits
            .write()
            .insert(circuit_key.clone(), circuit);

        info!(
            "Established relay circuit: {} -> {} via {}",
            local_peer_id, remote_peer_id, relay_peer_id
        );

        Ok(())
    }

    /// Close relay circuit
    pub async fn close_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Result<(), NatTraversalError> {
        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);
        self.relay_circuits.write().remove(&circuit_key);
        debug!(
            "Closed relay circuit: {} -> {} via {}",
            local_peer_id, remote_peer_id, relay_peer_id
        );
        Ok(())
    }

    /// Get all active relay circuits
    pub fn get_active_circuits(&self) -> Vec<RelayCircuit> {
        self.relay_circuits
            .read()
            .values()
            .filter(|c| c.active)
            .cloned()
            .collect()
    }

    /// Get relay circuit
    pub fn get_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Option<RelayCircuit> {
        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);
        self.relay_circuits.read().get(&circuit_key).cloned()
    }

    /// Shutdown NAT traversal
    pub async fn shutdown(&self) -> Result<(), NatTraversalError> {
        self.relay_circuits.write().clear();
        info!("NAT traversal shutdown complete");
        Ok(())
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get current unix timestamp in seconds
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery_creation() {
        let peers = vec!["peer1".to_string(), "peer2".to_string()];
        let discovery = PeerAddressDiscovery::with_peers(peers.clone(), 10);
        assert_eq!(discovery.peer_reflectors.len(), 2);
    }

    // NOTE: These tests now require a real SwarmHandle with live libp2p connections
    // They are moved to integration tests in tests/integration_nat.rs
    // Unit tests cannot create SwarmHandles without spinning up actual network infrastructure

    #[tokio::test]
    #[ignore = "Requires SwarmHandle integration test"]
    async fn test_peer_discovery_no_peers() {
        // This test now requires SwarmHandle parameter
        // See tests/integration_nat.rs for full integration test
    }

    #[tokio::test]
    #[ignore = "Requires SwarmHandle integration test"]
    async fn test_detect_nat_type_with_peers() {
        // This test now requires SwarmHandle parameter
        // See tests/integration_nat.rs for full integration test
    }

    #[tokio::test]
    #[ignore = "Requires SwarmHandle integration test"]
    async fn test_get_external_address_from_peer() {
        // This test now requires SwarmHandle parameter
        // See tests/integration_nat.rs for full integration test
    }

    #[test]
    fn test_nat_traversal_creation() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");
        assert_eq!(traversal.get_nat_type(), NatType::Unknown);
    }

    #[tokio::test]
    #[ignore = "Requires SwarmHandle integration test"]
    async fn test_probe_nat() {
        // This test now requires SwarmHandle parameter
        // See tests/integration_nat.rs for full integration test
    }

    #[tokio::test]
    async fn test_establish_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        assert!(traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .is_ok());

        let circuits = traversal.get_active_circuits();
        assert_eq!(circuits.len(), 1);
    }

    #[tokio::test]
    async fn test_relay_fallback_disabled() {
        let config = NatConfig {
            enable_relay_fallback: false,
            ..Default::default()
        };

        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        assert!(traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_close_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        assert!(traversal
            .close_relay_circuit(local, remote, relay)
            .await
            .is_ok());

        let circuits = traversal.get_active_circuits();
        assert!(circuits.is_empty());
    }

    #[tokio::test]
    async fn test_get_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        let circuit = traversal.get_relay_circuit(local, remote, relay);
        assert!(circuit.is_some());
        let c = circuit.unwrap();
        assert_eq!(c.source_peer_id, local);
        assert_eq!(c.dest_peer_id, remote);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        assert_eq!(traversal.get_active_circuits().len(), 1);

        traversal.shutdown().await.unwrap();
        assert_eq!(traversal.get_active_circuits().len(), 0);
    }

    #[test]
    fn test_nat_type_equality() {
        assert_eq!(NatType::Open, NatType::Open);
        assert_ne!(NatType::Open, NatType::Symmetric);
    }

    #[test]
    fn test_nat_config_defaults() {
        let config = NatConfig::default();
        // Default config has empty peer_reflectors (populated dynamically from mesh)
        assert!(config.peer_reflectors.is_empty());
        assert!(config.enable_relay_fallback);
    }
}
