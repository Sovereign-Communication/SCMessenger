//! Self-Relay Network Protocol — Messages and serialization

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Relay capability flags indicating node capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayCapability {
    /// Node can relay messages to other peers
    pub can_relay: bool,
    /// Node can store messages for offline peers (store-and-forward)
    pub can_store: bool,
    /// Node has internet connectivity
    pub has_internet: bool,
    /// Node is a full node (not mobile-constrained)
    pub full_node: bool,
}

impl RelayCapability {
    /// Create capabilities for a fully-capable relay
    pub fn full_relay() -> Self {
        Self {
            can_relay: true,
            can_store: true,
            has_internet: true,
            full_node: true,
        }
    }

    /// Create capabilities for a mobile node — UNIFICATION_V2: all nodes are relays.
    /// Formerly mobile had can_relay=false; now every node is a full relay per "a node is a node" philosophy.
    pub fn mobile() -> Self {
        Self {
            can_relay: true,
            can_store: true,
            has_internet: true,
            full_node: false,
        }
    }

    /// Check if this node can act as a relay
    pub fn is_relay(&self) -> bool {
        self.can_relay && self.has_internet
    }

    /// Check if this node can accept store-and-forward
    pub fn is_store(&self) -> bool {
        self.can_store && self.has_internet
    }
}

impl Default for RelayCapability {
    fn default() -> Self {
        Self::full_relay()
    }
}

/// A relay message sent between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    /// Handshake: initial connection negotiation
    Handshake {
        /// Protocol version
        version: u32,
        /// Sender's peer ID (Blake3 hash of public key)
        peer_id: String,
        /// Sender's capabilities
        capabilities: RelayCapability,
    },
    /// Handshake acknowledgment
    HandshakeAck {
        /// Protocol version
        version: u32,
        /// Responder's peer ID
        peer_id: String,
        /// Responder's capabilities
        capabilities: RelayCapability,
    },
    /// Store request: sender wants us to store envelopes for offline peer
    StoreRequest {
        /// Envelopes to store (raw serialized message envelopes)
        envelopes: Vec<Vec<u8>>,
    },
    /// Store acknowledgment: confirms how many accepted/rejected
    StoreAck {
        /// Number of envelopes accepted for storage
        accepted: u32,
        /// Number of envelopes rejected
        rejected: u32,
    },
    /// Pull request: retrieve stored envelopes
    PullRequest {
        /// Get envelopes stored since this timestamp (seconds)
        since_timestamp: u64,
        /// Optional hints for filtering (e.g., sender ID hashes)
        hints: Vec<[u8; 4]>,
    },
    /// Pull response: envelopes for the requesting peer
    PullResponse {
        /// Retrieved envelopes
        envelopes: Vec<Vec<u8>>,
    },
    /// Peer exchange: share known relay peers
    PeerExchange {
        /// List of known relay peers
        known_relays: Vec<RelayPeerInfoMessage>,
    },
    /// Peer announcement: relay broadcasts when new peer joins
    PeerJoined {
        /// Peer that just joined
        peer_info: RelayPeerInfoMessage,
    },
    /// Peer departure: relay broadcasts when peer leaves
    PeerLeft {
        /// Peer ID that left
        peer_id: String,
    },
    /// Peer list request: client requests full list of connected peers
    PeerListRequest,
    /// Peer list response: relay sends list of all connected peers
    PeerListResponse {
        /// All currently connected peers
        peers: Vec<RelayPeerInfoMessage>,
    },
    /// Keep-alive ping
    Ping,
    /// Ping response
    Pong,
    /// Graceful disconnect
    Disconnect {
        /// Reason for disconnect
        reason: String,
    },
}

/// Serialized peer info for inclusion in PeerExchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayPeerInfoMessage {
    /// Peer's ID (Blake3 hash of public key)
    pub peer_id: String,
    /// Known addresses (TCP, etc.)
    pub addresses: Vec<String>,
    /// Last time we saw this peer online (Unix timestamp)
    pub last_seen: u64,
    /// Reliability score (0.0-1.0)
    pub reliability_score: f32,
    /// Peer's capabilities
    pub capabilities: RelayCapability,
}

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Relay message serialization errors
#[derive(Debug, Error)]
pub enum RelayProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid message format")]
    InvalidFormat,
}

impl RelayMessage {
    /// Maximum allowed payload size for a relay message (64 KiB)
    pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;

    /// Serialize a relay message to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, RelayProtocolError> {
        bincode::serialize(self).map_err(|e| RelayProtocolError::SerializationError(e.to_string()))
    }

    /// Deserialize a relay message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RelayProtocolError> {
        if bytes.len() > Self::MAX_MESSAGE_BYTES {
            return Err(RelayProtocolError::DeserializationError(format!(
                "Message size {} exceeds maximum allowed {}",
                bytes.len(),
                Self::MAX_MESSAGE_BYTES
            )));
        }
        bincode::deserialize(bytes)
            .map_err(|e| RelayProtocolError::DeserializationError(e.to_string()))
    }

    /// Get a human-readable description of the message type
    pub fn message_type(&self) -> &'static str {
        match self {
            RelayMessage::Handshake { .. } => "Handshake",
            RelayMessage::HandshakeAck { .. } => "HandshakeAck",
            RelayMessage::StoreRequest { .. } => "StoreRequest",
            RelayMessage::StoreAck { .. } => "StoreAck",
            RelayMessage::PullRequest { .. } => "PullRequest",
            RelayMessage::PullResponse { .. } => "PullResponse",
            RelayMessage::PeerExchange { .. } => "PeerExchange",
            RelayMessage::PeerJoined { .. } => "PeerJoined",
            RelayMessage::PeerLeft { .. } => "PeerLeft",
            RelayMessage::PeerListRequest => "PeerListRequest",
            RelayMessage::PeerListResponse { .. } => "PeerListResponse",
            RelayMessage::Ping => "Ping",
            RelayMessage::Pong => "Pong",
            RelayMessage::Disconnect { .. } => "Disconnect",
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_full_relay() {
        let cap = RelayCapability::full_relay();
        assert!(cap.can_relay);
        assert!(cap.can_store);
        assert!(cap.has_internet);
        assert!(cap.full_node);
        assert!(cap.is_relay());
        assert!(cap.is_store());
    }

    #[test]
    fn test_oversized_message_rejected() {
        let oversized = vec![0u8; RelayMessage::MAX_MESSAGE_BYTES + 1];
        let result = RelayMessage::from_bytes(&oversized);
        assert!(result.is_err());
    }

    #[test]
    fn test_capability_mobile() {
        let cap = RelayCapability::mobile();
        // UNIFICATION_V2: all nodes are relays — mobile now also relays
        assert!(cap.can_relay);
        assert!(cap.can_store);
        assert!(cap.has_internet);
        assert!(!cap.full_node);
        assert!(cap.is_relay());
        assert!(cap.is_store());
    }

    #[test]
    fn test_relay_message_handshake_serialization() {
        let msg = RelayMessage::Handshake {
            version: PROTOCOL_VERSION,
            peer_id: "abc123".to_string(),
            capabilities: RelayCapability::full_relay(),
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(msg.message_type(), restored.message_type());
    }

    #[test]
    fn test_relay_message_store_request_serialization() {
        let msg = RelayMessage::StoreRequest {
            envelopes: vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]],
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::StoreRequest { envelopes } => {
                assert_eq!(envelopes.len(), 2);
                assert_eq!(envelopes[0], vec![1, 2, 3, 4]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_relay_message_pull_request_serialization() {
        let msg = RelayMessage::PullRequest {
            since_timestamp: 1000000,
            hints: vec![[1, 2, 3, 4]],
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::PullRequest {
                since_timestamp,
                hints,
            } => {
                assert_eq!(since_timestamp, 1000000);
                assert_eq!(hints.len(), 1);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_relay_message_peer_exchange_serialization() {
        let msg = RelayMessage::PeerExchange {
            known_relays: vec![RelayPeerInfoMessage {
                peer_id: "peer1".to_string(),
                addresses: vec!["127.0.0.1:8080".to_string()],
                last_seen: 1000000,
                reliability_score: 0.95,
                capabilities: RelayCapability::full_relay(),
            }],
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::PeerExchange { known_relays } => {
                assert_eq!(known_relays.len(), 1);
                assert_eq!(known_relays[0].peer_id, "peer1");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_relay_message_disconnect_serialization() {
        let msg = RelayMessage::Disconnect {
            reason: "Node shutting down".to_string(),
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::Disconnect { reason } => {
                assert_eq!(reason, "Node shutting down");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_relay_message_ping_pong() {
        let ping = RelayMessage::Ping;
        let bytes = ping.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(restored.message_type(), "Ping");

        let pong = RelayMessage::Pong;
        let bytes = pong.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(restored.message_type(), "Pong");
    }

    #[test]
    fn test_relay_message_store_ack() {
        let msg = RelayMessage::StoreAck {
            accepted: 10,
            rejected: 2,
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::StoreAck { accepted, rejected } => {
                assert_eq!(accepted, 10);
                assert_eq!(rejected, 2);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_relay_message_pull_response() {
        let msg = RelayMessage::PullResponse {
            envelopes: vec![vec![100, 101, 102], vec![103, 104, 105]],
        };

        let bytes = msg.to_bytes().expect("Failed to serialize");
        let restored = RelayMessage::from_bytes(&bytes).expect("Failed to deserialize");

        match restored {
            RelayMessage::PullResponse { envelopes } => {
                assert_eq!(envelopes.len(), 2);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        let invalid_bytes = vec![255, 254, 253];
        let result = RelayMessage::from_bytes(&invalid_bytes);
        assert!(result.is_err());
    }
}
