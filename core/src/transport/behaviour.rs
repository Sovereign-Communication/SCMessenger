// Combined NetworkBehaviour for Iron Core — Aggressive Discovery Mode
//
// Philosophy: "A node is a node." All nodes are mandatory relays.
//
// This combines all the libp2p protocols we need:
// - request_response: direct peer-to-peer message delivery
// - address_reflection: sovereign mesh address discovery (replaces STUN)
// - gossipsub: pub/sub — PERMISSIVE mode for dynamic topic negotiation
// - kademlia: DHT for peer discovery on WAN
// - mdns: peer discovery on LAN (disabled on Android - uses NsdManager instead)
// - identify: exchange peer metadata (advertises relay capability)
// - relay: NAT traversal — all nodes are mandatory relays
// - ledger_exchange: automatic peer list sharing for aggressive discovery

use super::discovery::DiscoveryConfig;
use super::reflection::{AddressReflectionRequest, AddressReflectionResponse};
use crate::identity::IdentityKeys;
use crate::store::ledger_entry::{LedgerExchangeRequest, LedgerExchangeResponse};
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
use libp2p::mdns;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::{
    autonat, connection_limits, dcutr, gossipsub, identify, kad, ping, relay,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use uuid::Uuid;
use web_time::Duration;

/// The Iron Core network behaviour combining all protocols.
#[derive(NetworkBehaviour)]
pub struct IronCoreBehaviour {
    /// Connection admission must run before stateful child behaviours. The
    /// derive macro calls `handle_established_*_connection` in field order;
    /// placing the limit guard first prevents request-response from recording
    /// a connection that this behaviour later rejects.
    pub connection_limits: connection_limits::Behaviour,
    /// Circuit Relay v2 client for relay reservations and relayed dials.
    pub relay_client: relay::client::Behaviour,
    /// Circuit Relay v2 server - all nodes act as relays for NAT traversal.
    pub relay_server: relay::Behaviour,
    /// Direct connection upgrade through relay (hole punching).
    pub dcutr: dcutr::Behaviour,
    /// NAT status probing via observed reachability.
    pub autonat: autonat::Behaviour,
    /// Keepalive and round-trip telemetry.
    pub ping: ping::Behaviour,
    /// Direct message delivery (request-response pattern)
    pub messaging: request_response::cbor::Behaviour<Libp2pMessageRequest, Libp2pMessageResponse>,
    /// Address reflection for sovereign NAT discovery (replaces external STUN)
    pub address_reflection:
        request_response::cbor::Behaviour<AddressReflectionRequest, AddressReflectionResponse>,
    /// Relay protocol for mesh routing (Phase 3)
    pub relay: request_response::cbor::Behaviour<RelayRequest, RelayResponse>,
    /// Signed registration/deregistration protocol for WS13.3.
    pub registration: request_response::cbor::Behaviour<RegistrationMessage, RegistrationResponse>,
    /// Ledger exchange — peers share their known peer lists on connect
    pub ledger_exchange:
        request_response::cbor::Behaviour<LedgerExchangeRequest, LedgerExchangeResponse>,
    /// Pub/sub for group messaging — PERMISSIVE mode for topic auto-negotiation
    pub gossipsub: gossipsub::Behaviour,
    /// DHT for WAN peer discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// LAN peer discovery — wrapped in Toggle so it can be disabled in
    /// environments without multicast support (containers, CI, cloud VMs).
    /// On Android, NsdManager is used instead (see MdnsServiceDiscovery.kt).
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    pub mdns: Toggle<mdns::tokio::Behaviour>,
    /// Peer identification — advertises relay capability
    pub identify: identify::Behaviour,
}

/// A libp2p request_response message request sent to a peer
/// Renamed from MessageRequest to avoid collision with UniFFI api.udl MessageRequest
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Libp2pMessageRequest {
    /// Serialized Envelope bytes
    pub envelope_data: Vec<u8>,
}

/// A libp2p request_response message response
/// Renamed from MessageResponse to avoid collision with UniFFI api.udl types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Libp2pMessageResponse {
    /// Whether the message was accepted
    pub accepted: bool,
    /// Optional error message
    pub error: Option<String>,
}

/// A relay request (asking a peer to forward a message to another peer)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayRequest {
    /// The final destination peer ID (serialized)
    pub destination_peer: Vec<u8>,
    /// Serialized Envelope bytes to relay
    pub envelope_data: Vec<u8>,
    /// Unique message ID for tracking
    pub message_id: String,
    /// SCMessenger identity ID of the intended recipient (WS13 tight-pair metadata).
    /// None for legacy senders that have not yet upgraded.
    #[serde(default)]
    pub recipient_identity_id: Option<String>,
    /// Device UUID (UUIDv4) of the specific device the sender is targeting (WS13).
    /// None for legacy senders or when the sender has no device record for the recipient.
    #[serde(default)]
    pub intended_device_id: Option<String>,
}

/// Response to a relay request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayResponse {
    /// Whether the relay was accepted
    pub accepted: bool,
    /// Optional error message
    pub error: Option<String>,
    /// Message ID being acknowledged
    pub message_id: String,
}

/// Canonically signed registration payload for WS13.3.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistrationPayload {
    pub identity_id: String,
    pub device_id: String,
    pub seniority_ts: u64,
}

impl RegistrationPayload {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, &'static str> {
        bincode::serialize(self).map_err(|_| "registration_payload_serialize_failed")
    }

    fn validate_fields(&self) -> Result<(), &'static str> {
        validate_identity_id(&self.identity_id, "registration_identity_id_invalid")?;
        validate_uuid_v4(&self.device_id, "registration_device_id_invalid")?;
        if self.seniority_ts == 0 {
            return Err("registration_seniority_invalid");
        }
        Ok(())
    }
}

/// Signed registration request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistrationRequest {
    pub payload: RegistrationPayload,
    pub signature: Vec<u8>,
}

impl RegistrationRequest {
    pub fn new_signed(
        keys: &IdentityKeys,
        device_id: String,
        seniority_ts: u64,
    ) -> Result<Self, &'static str> {
        let payload = RegistrationPayload {
            identity_id: keys.identity_id(),
            device_id,
            seniority_ts,
        };
        payload.validate_fields()?;
        let signature = keys
            .sign(&payload.canonical_bytes()?)
            .map_err(|_| "registration_signature_generation_failed")?;
        Ok(Self { payload, signature })
    }

    pub fn verify_for_public_key(&self, public_key: &[u8]) -> Result<(), &'static str> {
        self.payload.validate_fields()?;
        validate_signature_bytes(&self.signature, "registration_signature_invalid")?;
        validate_identity_owner(
            public_key,
            &self.payload.identity_id,
            "registration_identity_mismatch",
        )?;
        let valid = IdentityKeys::verify(
            &self.payload.canonical_bytes()?,
            &self.signature,
            public_key,
        )
        .map_err(|_| "registration_signature_invalid")?;
        if !valid {
            return Err("registration_signature_invalid");
        }
        Ok(())
    }
}

/// Canonically signed deregistration payload for WS13.3.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeregistrationPayload {
    pub identity_id: String,
    pub from_device_id: String,
    #[serde(default)]
    pub target_device_id: Option<String>,
}

impl DeregistrationPayload {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, &'static str> {
        bincode::serialize(self).map_err(|_| "deregistration_payload_serialize_failed")
    }

    fn validate_fields(&self) -> Result<(), &'static str> {
        validate_identity_id(&self.identity_id, "deregistration_identity_id_invalid")?;
        validate_uuid_v4(
            &self.from_device_id,
            "deregistration_from_device_id_invalid",
        )?;
        if let Some(target_device_id) = self.target_device_id.as_deref() {
            validate_uuid_v4(target_device_id, "deregistration_target_device_id_invalid")?;
            let from_uuid = Uuid::parse_str(&self.from_device_id)
                .map_err(|_| "deregistration_from_device_id_invalid")?;
            let target_uuid = Uuid::parse_str(target_device_id)
                .map_err(|_| "deregistration_target_device_id_invalid")?;
            if target_uuid == from_uuid {
                return Err("deregistration_target_matches_source");
            }
        }
        Ok(())
    }
}

/// Signed deregistration request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeregistrationRequest {
    pub payload: DeregistrationPayload,
    pub signature: Vec<u8>,
}

impl DeregistrationRequest {
    pub fn new_signed(
        keys: &IdentityKeys,
        from_device_id: String,
        target_device_id: Option<String>,
    ) -> Result<Self, &'static str> {
        let payload = DeregistrationPayload {
            identity_id: keys.identity_id(),
            from_device_id,
            target_device_id,
        };
        payload.validate_fields()?;
        let signature = keys
            .sign(&payload.canonical_bytes()?)
            .map_err(|_| "deregistration_signature_generation_failed")?;
        Ok(Self { payload, signature })
    }

    pub fn verify_for_public_key(&self, public_key: &[u8]) -> Result<(), &'static str> {
        self.payload.validate_fields()?;
        validate_signature_bytes(&self.signature, "deregistration_signature_invalid")?;
        validate_identity_owner(
            public_key,
            &self.payload.identity_id,
            "deregistration_identity_mismatch",
        )?;
        let valid = IdentityKeys::verify(
            &self.payload.canonical_bytes()?,
            &self.signature,
            public_key,
        )
        .map_err(|_| "deregistration_signature_invalid")?;
        if !valid {
            return Err("deregistration_signature_invalid");
        }
        Ok(())
    }
}

/// Protocol wrapper for `/sc/registration/1.0.0`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegistrationMessage {
    Register(RegistrationRequest),
    Deregister(DeregistrationRequest),
}

/// Response to a registration/deregistration request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistrationResponse {
    pub accepted: bool,
    pub error: Option<String>,
}

fn validate_identity_id(identity_id: &str, error: &'static str) -> Result<(), &'static str> {
    if identity_id.len() != 64
        || !identity_id
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(error);
    }
    Ok(())
}

fn validate_uuid_v4(value: &str, error: &'static str) -> Result<(), &'static str> {
    let uuid = Uuid::parse_str(value).map_err(|_| error)?;
    if uuid.get_version_num() != 4 {
        return Err(error);
    }
    Ok(())
}

fn validate_signature_bytes(signature: &[u8], error: &'static str) -> Result<(), &'static str> {
    if signature.len() != 64 {
        return Err(error);
    }
    Ok(())
}

fn validate_identity_owner(
    public_key: &[u8],
    claimed_identity_id: &str,
    mismatch_error: &'static str,
) -> Result<(), &'static str> {
    let derived_identity_id = hex::encode(blake3::hash(public_key).as_bytes());
    if !derived_identity_id.eq_ignore_ascii_case(claimed_identity_id) {
        return Err(mismatch_error);
    }
    Ok(())
}

// SharedPeerEntry, LedgerExchangeRequest, and LedgerExchangeResponse
// have been moved to crate::store::ledger_entry

impl IronCoreBehaviour {
    /// Create a new behaviour with the given keypair.
    ///
    /// Key design decisions for Aggressive Discovery:
    /// - Gossipsub uses PERMISSIVE validation (accept messages from any topic)
    /// - Identify advertises this node as a relay
    /// - Kademlia set to Server mode by default
    /// - Ledger exchange for automatic peer list sharing
    /// - All timeouts are generous to survive flaky networks
    #[allow(unused_variables)]
    pub fn new(
        keypair: &libp2p::identity::Keypair,
        relay_client: relay::client::Behaviour,
        headless: bool,
        discovery_config: Option<DiscoveryConfig>,
    ) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();
        let dcutr = dcutr::Behaviour::new(peer_id);
        let autonat = autonat::Behaviour::new(
            peer_id,
            autonat::Config {
                // Use a longer retry_interval (300s) instead of the default 90s.
                // This reduces spurious "NoServer" probe errors when isolated
                // (no connected peers to use as probe servers). The probe will
                // still fire, but less aggressively — and the swarm event loop
                // gates the log on connected_peers being non-empty.
                retry_interval: Duration::from_secs(300),
                ..autonat::Config::default()
            },
        );
        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(15))
                .with_timeout(Duration::from_secs(20)),
        );

        // In libp2p 0.56 the request/response size ceiling lives on the cbor
        // Codec (default 1 MiB request / 10 MiB response), not on Config.
        // DriftFrame-wrapped envelopes plus custody batches can exceed the
        // request default, which surfaces as truncated frames and
        // "Failed to decode envelope: unexpected end of file" on the receiver
        // (farm-sim root cause family, HERMES_FARM_AUDIT 2026-07-16).
        const MAX_REQUEST_SIZE: u64 = 4 * 1024 * 1024; // 4 MiB
        const MAX_RESPONSE_SIZE: u64 = 16 * 1024 * 1024; // 16 MiB
        fn sized_codec<Req, Resp>() -> request_response::cbor::codec::Codec<Req, Resp> {
            request_response::cbor::codec::Codec::default()
                .set_request_size_maximum(MAX_REQUEST_SIZE)
                .set_response_size_maximum(MAX_RESPONSE_SIZE)
        }

        // Request-response for direct messaging
        let messaging = request_response::Behaviour::with_codec(
            sized_codec(),
            [(
                StreamProtocol::new("/sc/message/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Request-response for address reflection (sovereign NAT discovery).
        // Payloads are tiny fixed structs; keep the codec at library defaults
        // (1 MiB/10 MiB) rather than raising attack surface (adversarial
        // review 2026-07-17 F5).
        let address_reflection = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/address-reflection/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(10)),
        );

        // Request-response for relay (mesh routing - Phase 3)
        // All nodes are mandatory relays - always Full support
        let relay = request_response::Behaviour::with_codec(
            sized_codec(),
            [(
                StreamProtocol::new("/sc/relay/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(60)), // Generous timeout for relay
        );

        // Registration payloads are small fixed structs; keep library defaults.
        let registration = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/registration/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Ledger exchange — automatic peer list sharing
        // When two peers connect, they exchange their known peer lists.
        // This dramatically accelerates mesh discovery.
        let ledger_exchange = request_response::Behaviour::with_codec(
            sized_codec(),
            [(
                StreamProtocol::new("/sc/ledger-exchange/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Gossipsub for pub/sub — PERMISSIVE mode for aggressive discovery
        //
        // ValidationMode::Permissive means:
        //   - Accept messages even if we don't have the signing key
        //   - Accept messages from topics we're not explicitly subscribed to
        //   - Log anomalies but don't drop connections
        //
        // This enables dynamic topic negotiation: when a peer advertises
        // a different topic, we can see it and auto-subscribe.
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(5)) // Faster heartbeat for quicker discovery
            .validation_mode(gossipsub::ValidationMode::Permissive) // PERMISSIVE: accept everything
            .mesh_outbound_min(1) // Must be <= mesh_n_low
            .mesh_n_low(1) // Accept mesh with just 1 peer
            .mesh_n(3) // Target 3 peers in mesh
            .mesh_n_high(12) // Allow up to 12
            .gossip_lazy(3) // Gossip to at least 3 non-mesh peers
            .history_length(5) // Keep 5 heartbeat windows of message history
            .history_gossip(3) // Gossip about last 3 windows
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub error: {}", e))?;

        // Kademlia DHT for peer discovery
        // Apply DHT Hyper-Optimization (Alpha 8, Replication 5)
        let mut kad_config = kad::Config::default();
        kad_config
            .set_parallelism(std::num::NonZeroUsize::new(8).expect("parallelism must be non-zero"));
        kad_config.set_replication_factor(
            std::num::NonZeroUsize::new(5).expect("replication factor must be non-zero"),
        );
        let mut kademlia =
            kad::Behaviour::with_config(peer_id, kad::store::MemoryStore::new(peer_id), kad_config);
        // Set server mode immediately — we want to be discoverable
        kademlia.set_mode(Some(kad::Mode::Server));

        // mDNS for LAN discovery — gracefully disabled in environments without
        // multicast support (Docker containers, cloud VMs, CI runners).
        // On Android, NsdManager is used instead (see MdnsServiceDiscovery.kt).
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
        let mdns = if discovery_config
            .as_ref()
            .map(|c| c.mode.allows_mdns())
            .unwrap_or(true)
        {
            match mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id) {
                Ok(m) => {
                    tracing::info!("mDNS LAN discovery: enabled (libp2p-mdns)");
                    Toggle::from(Some(m))
                }
                Err(e) => {
                    tracing::warn!(
                        "mDNS LAN discovery disabled ({}): container/VM without multicast support",
                        e
                    );
                    Toggle::from(None)
                }
            }
        } else {
            tracing::info!("mDNS LAN discovery: disabled by config");
            Toggle::from(None)
        };

        // Identify protocol — advertise this node as a relay
        //
        // agent_version includes "relay" to signal we're a mandatory relay.
        // We also distinguish "headless" (infrastructure) vs "full" (human) nodes.
        // push_listen_addr_updates ensures peers learn our addresses quickly.
        let type_str = if headless { "headless" } else { "full" };
        let identify = identify::Behaviour::new(
            identify::Config::new("/sc/id/1.0.0".to_string(), keypair.public())
                .with_push_listen_addr_updates(true)
                .with_interval(Duration::from_secs(60)) // Reduced frequency to prevent identify storms
                .with_agent_version(format!(
                    "scmessenger/{}/{}/relay/{}",
                    env!("CARGO_PKG_VERSION"),
                    type_str,
                    peer_id
                )),
        );
        // Relay server - all nodes act as relays for NAT traversal
        let relay_server = relay::Behaviour::new(peer_id, relay::Config::default());

        // Connection limits to prevent resource exhaustion
        let connection_limits = connection_limits::Behaviour::new(
            connection_limits::ConnectionLimits::default()
                .with_max_pending_outgoing(Some(32))
                .with_max_established_outgoing(Some(128))
                .with_max_established_incoming(Some(64))
                // Keep one relay path and one direct/hole-punch path available,
                // but prevent address churn from opening a third connection to
                // the same peer. request-response 0.29 assumes a bounded
                // per-peer connection set and has panicked during six-path
                // convergence in the Windows soak.
                .with_max_established_per_peer(Some(2)),
        );

        Ok(Self {
            relay_client,
            relay_server,
            dcutr,
            autonat,
            ping,
            messaging,
            address_reflection,
            relay,
            registration,
            ledger_exchange,
            gossipsub,
            kademlia,
            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
            mdns,
            identify,
            connection_limits,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::IdentityKeys;

    #[test]
    fn relay_request_carries_ws13_metadata_when_set() {
        let req = RelayRequest {
            destination_peer: vec![0xAB],
            envelope_data: vec![0xCD],
            message_id: "msg-2".to_string(),
            recipient_identity_id: Some("identity-abc".to_string()),
            intended_device_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        };
        assert_eq!(req.recipient_identity_id.as_deref(), Some("identity-abc"));
        assert_eq!(
            req.intended_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn relay_request_missing_ws13_fields_deserialize_with_defaults() {
        // Verify generic serde defaulting when optional WS13 fields are absent.
        let json =
            r#"{"destination_peer":[1,2,3],"envelope_data":[4,5,6],"message_id":"msg-legacy"}"#;
        let req: RelayRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message_id, "msg-legacy");
        assert!(
            req.recipient_identity_id.is_none(),
            "missing recipient_identity_id must default to None"
        );
        assert!(
            req.intended_device_id.is_none(),
            "missing intended_device_id must default to None"
        );
    }

    #[test]
    fn registration_payload_canonical_bytes_are_stable() {
        let payload = RegistrationPayload {
            identity_id: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            seniority_ts: 1_731_000_000,
        };

        let first = payload.canonical_bytes().unwrap();
        let second = payload.canonical_bytes().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn signed_registration_request_verifies_against_matching_public_key() {
        let keys = IdentityKeys::generate();
        let request = RegistrationRequest::new_signed(
            &keys,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            1_731_000_000,
        )
        .unwrap();

        let public_key = keys.signing_key.verifying_key().to_bytes();
        assert!(request.verify_for_public_key(&public_key).is_ok());
    }

    #[test]
    fn signed_registration_request_rejects_tampered_payload() {
        let keys = IdentityKeys::generate();
        let mut request = RegistrationRequest::new_signed(
            &keys,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            1_731_000_000,
        )
        .unwrap();
        request.payload.device_id = "550e8400-e29b-41d4-a716-446655440001".to_string();

        let public_key = keys.signing_key.verifying_key().to_bytes();
        assert_eq!(
            request.verify_for_public_key(&public_key),
            Err("registration_signature_invalid")
        );
    }

    #[test]
    fn signed_registration_request_rejects_malformed_identity_id() {
        let keys = IdentityKeys::generate();
        let mut request = RegistrationRequest::new_signed(
            &keys,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            1_731_000_000,
        )
        .unwrap();
        request.payload.identity_id = "not-hex".to_string();

        let public_key = keys.signing_key.verifying_key().to_bytes();
        assert_eq!(
            request.verify_for_public_key(&public_key),
            Err("registration_identity_id_invalid")
        );
    }

    #[test]
    fn signed_deregistration_request_verifies_against_matching_public_key() {
        let keys = IdentityKeys::generate();
        let request = DeregistrationRequest::new_signed(
            &keys,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            Some("550e8400-e29b-41d4-a716-446655440001".to_string()),
        )
        .unwrap();

        let public_key = keys.signing_key.verifying_key().to_bytes();
        assert!(request.verify_for_public_key(&public_key).is_ok());
    }

    #[test]
    fn signed_deregistration_request_rejects_same_source_and_target_device() {
        let keys = IdentityKeys::generate();
        let result = DeregistrationRequest::new_signed(
            &keys,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        );

        assert_eq!(result, Err("deregistration_target_matches_source"));
    }
}
