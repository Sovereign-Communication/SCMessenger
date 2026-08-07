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
    /// emitted or displayed.
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
    /// Sender's identity ID (Blake3 hash of Ed25519 public key)
    pub sender_id: String,
    /// Recipient's identity ID
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

impl Envelope {
    /// Get canonical bytes for signing
    pub fn to_bytes_for_signing(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sender_public_key);
        bytes.extend_from_slice(&self.ephemeral_public_key);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.ciphertext);
        if let Some(ref dh) = self.ratchet_dh_public {
            bytes.extend_from_slice(dh);
        }
        if let Some(num) = self.ratchet_message_number {
            bytes.extend_from_slice(&num.to_le_bytes());
        }
        Ok(bytes)
    }
}

/// A signed envelope — adds Ed25519 signature for relay verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    /// The encrypted envelope
    pub envelope: Envelope,
    /// Ed25519 signature over the canonical serialization of the envelope
    /// (64 bytes)
    pub signature: Vec<u8>,
}

/// TTL configuration for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlConfig {
    /// Time-to-live in seconds
    pub expires_in_seconds: u64,
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
}

/// Encode a receipt to canonical wire format (JSON bytes)
pub fn encode_receipt(receipt: &Receipt) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = serde_json::to_vec(receipt)?;
    Ok(json)
}

/// Decode a receipt from canonical wire format (JSON bytes)
pub fn decode_receipt(data: &[u8]) -> Result<Receipt, Box<dyn std::error::Error>> {
    let receipt: Receipt = serde_json::from_slice(data)?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_encode_decode() {
        let receipt = Receipt {
            message_id: "test-123".to_string(),
            status: DeliveryStatus::Delivered,
            timestamp: 1234567890,
        };
        let encoded = encode_receipt(&receipt).unwrap();
        let decoded = decode_receipt(&encoded).unwrap();
        assert_eq!(receipt.message_id, decoded.message_id);
        assert_eq!(receipt.status, decoded.status);
        assert_eq!(receipt.timestamp, decoded.timestamp);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::text("sender".to_string(), "recipient".to_string(), "Hello");
        assert_eq!(msg.message_type, MessageType::Text);
        assert_eq!(msg.text_content(), Some("Hello".to_string()));
    }
}