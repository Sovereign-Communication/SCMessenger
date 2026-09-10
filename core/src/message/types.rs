// Message types — the literal point of this app

use serde::{Deserialize, Serialize};

/// What kind of message this is
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Plain text message
    Text,
    /// Delivery/read receipt
    Receipt,
    /// Onion relay packet (internal use for forwarding)
    OnionRelay,
}

/// Delivery status of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Message sent (left this device)
    Sent,
    /// Message delivered to recipient's device
    Delivered,
    /// Deprecated: retained for backward-compatible deserialization of receipts
    /// from older peers. Treated as no-op (mapped to `Delivered` in processing).
    /// Do not emit — Zero-Status Architecture: read receipts are no longer
    /// emitted or displayed. (Not `#[deprecated]` because the UniFFI scaffolding
    /// must construct it, and generated code cannot carry allow attributes.)
    Read,
    /// Delivery failed
    Failed,
}

/// A plaintext message before encryption.
///
/// This is what the application layer creates. It gets encrypted into
/// an `Envelope` before hitting the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (UUID v4)
    pub id: String,
    // UNIFICATION_V2: canonical identity is public_key_hex; identity_id/libp2p_peer_id are derived metadata only.
    /// Sender's canonical public key hex (64-char lowercase hex of Ed25519 VerifyingKey)
    pub sender_id: String,
    /// Recipient's canonical public key hex (64-char lowercase hex)
    pub recipient_id: String,
    /// Message type
    pub message_type: MessageType,
    /// Payload bytes (UTF-8 text for Text messages, serialized Receipt for receipts)
    pub payload: Vec<u8>,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
}

/// A delivery receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// ID of the message this receipt is for
    pub message_id: String,
    /// New delivery status
    pub status: DeliveryStatus,
    /// Unix timestamp of the status change
    pub timestamp: u64,
}

/// An encrypted message envelope — what actually goes on the wire.
///
/// Contains everything a recipient needs to decrypt the message,
/// assuming they have their own private key.
///
/// When `ratchet_dh_public` is present, the envelope was encrypted using
/// the Double Ratchet protocol (forward secrecy). The `ephemeral_public_key`
/// field carries the ratchet DH public key instead of a per-message ECDH key,
/// and `ratchet_message_number` identifies the chain position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Sender's Ed25519 public key (32 bytes) — so recipient knows who sent it
    pub sender_public_key: Vec<u8>,
    /// Ephemeral X25519 public key (32 bytes) — for ECDH key agreement.
    /// In ratcheted mode, this carries the DH ratchet public key.
    pub ephemeral_public_key: Vec<u8>,
    /// XChaCha20-Poly1305 nonce (24 bytes)
    pub nonce: Vec<u8>,
    /// Encrypted + authenticated ciphertext
    pub ciphertext: Vec<u8>,
    /// Double Ratchet: sender's current DH ratchet public key (32 bytes).
    /// `None` for legacy per-message ECDH envelopes (backward compatible).
    #[serde(default)]
    pub ratchet_dh_public: Option<Vec<u8>>,
    /// Double Ratchet: message number in the current sending chain.
    /// `None` for legacy per-message ECDH envelopes.
    #[serde(default)]
    pub ratchet_message_number: Option<u32>,
}

/// A signed envelope — adds Ed25519 signature for relay verification.
///
/// This allows relays to verify that the envelope was genuinely created by
/// the sender without decrypting the ciphertext. Relays can reject forged
/// or tampered envelopes before forwarding them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    /// The encrypted envelope
    pub envelope: Envelope,
    /// Ed25519 signature over the canonical serialization of the envelope
    /// (64 bytes)
    pub signature: Vec<u8>,
}

pub const WIRE_TAG_V2: u8 = 0x02;

/// Next-generation encrypted envelope supporting hybrid post-quantum cryptography.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvelopeV2 {
    /// Suite identifier (from PQC-00 suite registry)
    pub suite: u8,
    /// Sender's Ed25519 public key (32 bytes)
    pub sender_public_key: Vec<u8>,
    /// Ephemeral X25519 public key (32 bytes)
    pub ephemeral_public_key: Vec<u8>,
    /// XChaCha20-Poly1305 nonce (24 bytes)
    pub nonce: Vec<u8>,
    /// Encrypted + authenticated ciphertext
    pub ciphertext: Vec<u8>,
    /// Double Ratchet: sender's current DH ratchet public key (32 bytes)
    pub ratchet_dh_public: Option<Vec<u8>>,
    /// Double Ratchet: message number in the current sending chain
    pub ratchet_message_number: Option<u32>,
    /// ML-KEM-768 ciphertext (1088 bytes)
    pub pq_kem_ciphertext: Option<Vec<u8>>,
    /// Sender's next ML-KEM encapsulation key (1184 bytes)
    pub pq_encaps_key: Option<Vec<u8>>,
    /// Suite negotiation binding hash (32 bytes)
    pub transcript_hash: Option<Vec<u8>>,
}

/// A signed v2 envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedEnvelopeV2 {
    /// The encrypted v2 envelope
    pub envelope: EnvelopeV2,
    /// Ed25519 signature over the wire format of the envelope
    pub signature: Vec<u8>,
}

/// Polymorphic envelope wrapper for transport.
#[derive(Debug, Clone)]
pub enum WireEnvelope {
    V1(Envelope),
    V2(EnvelopeV2),
}

/// Polymorphic signed envelope wrapper for transport.
#[derive(Debug, Clone)]
pub enum WireSignedEnvelope {
    V1(SignedEnvelope),
    V2(SignedEnvelopeV2),
}

impl Message {
    /// Create a new text message
    pub fn text(sender_id: String, recipient_id: String, text: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            message_type: MessageType::Text,
            payload: text.as_bytes().to_vec(),
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a receipt message
    pub fn receipt(
        sender_id: String,
        recipient_id: String,
        receipt: &Receipt,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let payload = encode_receipt(receipt)?;
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            message_type: MessageType::Receipt,
            payload,
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Get text content (only valid for Text messages)
    pub fn text_content(&self) -> Option<String> {
        if self.message_type == MessageType::Text {
            String::from_utf8(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Check if message is recent (within threshold_secs)
    pub fn is_recent(&self, threshold_secs: u64) -> bool {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.timestamp) < threshold_secs
    }
}

impl Receipt {
    /// Create a delivery receipt
    pub fn delivered(message_id: String) -> Self {
        Self {
            message_id,
            status: DeliveryStatus::Delivered,
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Serialize a Receipt to JSON bytes (the canonical wire format).
///
/// This is the ONLY way receipts should be serialized anywhere in the codebase.
/// All platforms (CLI, Android, iOS, WASM) use this function.
pub fn encode_receipt(receipt: &Receipt) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(serde_json::to_vec(receipt)?)
}

/// Deserialize a Receipt from JSON bytes (the canonical wire format).
///
/// This is the ONLY way receipts should be deserialized anywhere in the codebase.
/// All platforms (CLI, Android, iOS, WASM) use this function.
pub fn decode_receipt(buf: &[u8]) -> Result<Receipt, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_message() {
        let msg = Message::text(
            "sender123".to_string(),
            "recipient456".to_string(),
            "hello world",
        );

        assert_eq!(msg.message_type, MessageType::Text);
        assert_eq!(msg.text_content().unwrap(), "hello world");
        assert_eq!(msg.sender_id, "sender123");
        assert_eq!(msg.recipient_id, "recipient456");
        assert!(!msg.id.is_empty());
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_create_receipt() {
        let receipt = Receipt::delivered("msg-id-123".to_string());
        assert_eq!(receipt.message_id, "msg-id-123");
        assert!(matches!(receipt.status, DeliveryStatus::Delivered));
    }

    #[test]
    fn test_receipt_message() {
        let receipt = Receipt::delivered("msg-123".to_string());
        let msg =
            Message::receipt("sender".to_string(), "recipient".to_string(), &receipt).unwrap();

        assert_eq!(msg.message_type, MessageType::Receipt);
        assert!(msg.text_content().is_none());
    }

    #[test]
    fn test_message_recency() {
        let msg = Message::text("a".into(), "b".into(), "test");
        assert!(msg.is_recent(60)); // Should be recent within 60 seconds

        let mut old_msg = Message::text("a".into(), "b".into(), "test");
        old_msg.timestamp = 0; // epoch
        assert!(!old_msg.is_recent(60));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::text("sender".into(), "recipient".into(), "hello");
        let bytes = bincode::serialize(&msg).unwrap();
        let restored: Message = bincode::deserialize(&bytes).unwrap();

        assert_eq!(msg.id, restored.id);
        assert_eq!(msg.text_content(), restored.text_content());
    }
}
