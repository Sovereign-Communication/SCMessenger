/// Drift Envelope — compact binary format for mesh relay
///
/// Fixed overhead: 187 bytes (18 + 14 + 152 + 2 + 1)
/// Format: Header(18) + Routing(14) + Crypto(152) + Payload(2+N) + RatchetFlag(1)
///
/// Layout (little-endian, no padding):
/// [1]  version
/// [1]  envelope_type
/// [16] message_id
/// [4]  recipient_hint
/// [4]  created_at (LE u32)
/// [4]  ttl_expiry (LE u32)
/// [1]  hop_count
/// [1]  priority
/// [32] sender_public_key
/// [32] ephemeral_public_key
/// [24] nonce
/// [64] signature
/// [2]  ciphertext_len (LE u16)
/// [N]  ciphertext
use super::{DriftError, DRIFT_VERSION};
use crate::message::Envelope;
use ed25519_dalek::Signer;
use uuid::Uuid;
use web_time::{SystemTime, UNIX_EPOCH};

/// Compression threshold: payloads larger than this are LZ4-compressed.
/// 256 bytes is the crossover point where LZ4 compression typically saves
/// more bandwidth than the overhead of the size-prepended format.
pub const COMPRESSION_THRESHOLD: usize = 256;

/// Bit mask for the compression flag in the envelope_type byte.
/// When set, the ciphertext field contains LZ4-compressed data.
pub const COMPRESSION_FLAG: u8 = 0x80;

/// Default envelope lifetime in seconds (7 days). The single source for the
/// creation TTL and for the freshness window the transport trust gate applies.
pub const DEFAULT_TTL_SECS: u32 = 604_800;

/// Drift Protocol Envelope — compact binary format for mesh relay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftEnvelope {
    // Header (18 bytes)
    /// Protocol version (0x01)
    pub version: u8,
    /// Envelope type (encrypted message, receipt, sync, etc.)
    pub envelope_type: EnvelopeType,
    /// Whether the ciphertext is LZ4-compressed
    pub compressed: bool,
    /// Message ID as raw bytes (16 bytes, UUID)
    pub message_id: [u8; 16],

    // Routing header (14 bytes)
    /// First 4 bytes of blake3(recipient_pk) for efficient routing
    pub recipient_hint: [u8; 4],
    /// Unix timestamp (seconds) when envelope was created
    pub created_at: u32,
    /// TTL expiry (0 = never expires), Unix timestamp
    pub ttl_expiry: u32,
    /// Hop count for relay tracking
    pub hop_count: u8,
    /// Message priority (0-255, higher = more important)
    pub priority: u8,

    // Crypto header (152 bytes)
    /// Sender's Ed25519 public key (32 bytes)
    pub sender_public_key: [u8; 32],
    /// Ephemeral X25519 public key for ECDH (32 bytes)
    pub ephemeral_public_key: [u8; 32],
    /// XChaCha20-Poly1305 nonce (24 bytes)
    pub nonce: [u8; 24],
    /// Ed25519 signature over everything except signature itself (64 bytes)
    pub signature: [u8; 64],

    // Payload (variable length)
    /// Encrypted + authenticated ciphertext
    pub ciphertext: Vec<u8>,

    // Ratchet metadata (optional, for Double Ratchet forward secrecy)
    /// Sender's DH ratchet public key (32 bytes) — None for legacy ECDH
    pub ratchet_dh_public: Option<[u8; 32]>,
    /// Message number in the current sending chain — None for legacy ECDH
    pub ratchet_message_number: Option<u32>,

    // Post-quantum extension fields
    /// Suite identifier for post-quantum cryptography
    pub suite: Option<u8>,
    /// Post-quantum KEM ciphertext
    pub pq_kem_ciphertext: Option<Vec<u8>>,
    /// Post-quantum encapsulated key
    pub pq_encaps_key: Option<Vec<u8>>,
    /// Transcript hash for post-quantum verification
    pub transcript_hash: Option<Vec<u8>>,
}

/// Message type enumeration for Drift Envelopes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnvelopeType {
    /// Encrypted message (0x01)
    EncryptedMessage = 0x01,
    /// Delivery receipt (0x02)
    DeliveryReceipt = 0x02,
    /// Sync request (0x03)
    SyncRequest = 0x03,
    /// Sync response (0x04)
    SyncResponse = 0x04,
    /// Peer announcement (0x05)
    PeerAnnouncement = 0x05,
    /// Route advertisement (0x06)
    RouteAdvertisement = 0x06,
    /// Cover traffic — indistinguishable from real messages (0x07)
    /// Relays forward these identically to EncryptedMessage.
    /// Only the final recipient can determine it's cover traffic
    /// (by failing to decrypt to a valid message).
    CoverTraffic = 0x07,
    /// Onion-routed message — multi-hop relay (0x08)
    /// Contains an onion envelope that each relay peels one layer from.
    OnionMessage = 0x08,
}

impl EnvelopeType {
    /// Convert from u8 to EnvelopeType
    pub fn from_u8(value: u8) -> Result<Self, DriftError> {
        match value {
            0x01 => Ok(EnvelopeType::EncryptedMessage),
            0x02 => Ok(EnvelopeType::DeliveryReceipt),
            0x03 => Ok(EnvelopeType::SyncRequest),
            0x04 => Ok(EnvelopeType::SyncResponse),
            0x05 => Ok(EnvelopeType::PeerAnnouncement),
            0x06 => Ok(EnvelopeType::RouteAdvertisement),
            0x07 => Ok(EnvelopeType::CoverTraffic),
            0x08 => Ok(EnvelopeType::OnionMessage),
            other => Err(DriftError::InvalidEnvelopeType(other)),
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl DriftEnvelope {
    /// Fixed overhead size: 18 + 14 + 152 + 2 + 1 = 187 bytes (includes ratchet flag)
    pub const FIXED_OVERHEAD: usize = 187;

    /// Maximum ciphertext size (2^16 - 1 bytes due to u16 length field)
    pub const MAX_CIPHERTEXT: usize = 65535;

    /// Serialize envelope to bytes (little-endian).
    ///
    /// If `compressed` is true, the ciphertext is LZ4-compressed before serialization.
    /// Returns `Err(CiphertextTooLarge)` if ciphertext exceeds MAX_CIPHERTEXT.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        // Compress if flagged
        let ciphertext = if self.compressed {
            super::compress::compress(&self.ciphertext)
        } else {
            self.ciphertext.clone()
        };

        if ciphertext.len() > Self::MAX_CIPHERTEXT {
            return Err(DriftError::CiphertextTooLarge(ciphertext.len()));
        }

        let mut buf = Vec::with_capacity(Self::FIXED_OVERHEAD + ciphertext.len());

        // Header (18 bytes)
        buf.push(self.version);
        // Encode envelope_type with compression flag
        let type_byte =
            self.envelope_type.as_u8() | if self.compressed { COMPRESSION_FLAG } else { 0 };
        buf.push(type_byte);
        buf.extend_from_slice(&self.message_id);

        // Routing header (14 bytes)
        buf.extend_from_slice(&self.recipient_hint);
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&self.ttl_expiry.to_le_bytes());
        buf.push(self.hop_count);
        buf.push(self.priority);

        // Crypto header (152 bytes)
        buf.extend_from_slice(&self.sender_public_key);
        buf.extend_from_slice(&self.ephemeral_public_key);
        buf.extend_from_slice(&self.nonce);
        buf.extend_from_slice(&self.signature);

        // Payload (2 + N bytes)
        buf.extend_from_slice(&(ciphertext.len() as u16).to_le_bytes());
        buf.extend_from_slice(&ciphertext);

        // Ratchet extension (optional, backward compatible)
        // Flag byte: 0x00 = no ratchet, 0x01 = ratchet present
        if let Some(ref dh_public) = self.ratchet_dh_public {
            buf.push(0x01); // ratchet flag
            buf.extend_from_slice(dh_public);
            let msg_num = self.ratchet_message_number.unwrap_or(0);
            buf.extend_from_slice(&msg_num.to_le_bytes());
        } else {
            buf.push(0x00); // no ratchet
        }

        // PQ extension (optional, backward compatible)
        // Flag byte: 0x00 = no PQ extension, 0x01 = PQ extension present
        if self.suite.is_some()
            || self.pq_kem_ciphertext.is_some()
            || self.pq_encaps_key.is_some()
            || self.transcript_hash.is_some()
        {
            buf.push(0x01); // pq_flag = present
            buf.push(self.suite.unwrap_or(0x02));
            // append each blob as u32 LE len + bytes (empty if None)
            let append_blob = |buf: &mut Vec<u8>, v: &Option<Vec<u8>>| {
                let d = v.as_deref().unwrap_or(&[]);
                buf.extend_from_slice(&(d.len() as u32).to_le_bytes());
                buf.extend_from_slice(d);
            };
            append_blob(&mut buf, &self.pq_kem_ciphertext);
            append_blob(&mut buf, &self.pq_encaps_key);
            append_blob(&mut buf, &self.transcript_hash);
        }

        Ok(buf)
    }

    /// Deserialize envelope from bytes
    ///
    /// Returns error if:
    /// - Buffer too short
    /// - Invalid version
    /// - Invalid envelope type
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        if data.len() < Self::FIXED_OVERHEAD {
            return Err(DriftError::BufferTooShort {
                need: Self::FIXED_OVERHEAD,
                got: data.len(),
            });
        }

        let mut offset = 0;

        // Header (18 bytes)
        let version = data[offset];
        offset += 1;

        if version != DRIFT_VERSION {
            return Err(DriftError::InvalidVersion(version));
        }

        // Decode envelope type with optional compression flag
        let type_byte = data[offset];
        let compressed = (type_byte & COMPRESSION_FLAG) != 0;
        let envelope_type = EnvelopeType::from_u8(type_byte & !COMPRESSION_FLAG)?;
        offset += 1;

        let mut message_id = [0u8; 16];
        message_id.copy_from_slice(&data[offset..offset + 16]);
        offset += 16;

        // Routing header (14 bytes)
        let mut recipient_hint = [0u8; 4];
        recipient_hint.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;

        let created_at = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let ttl_expiry = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let hop_count = data[offset];
        offset += 1;

        let priority = data[offset];
        offset += 1;

        // Crypto header (152 bytes)
        let mut sender_public_key = [0u8; 32];
        sender_public_key.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        let mut ephemeral_public_key = [0u8; 32];
        ephemeral_public_key.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        let mut nonce = [0u8; 24];
        nonce.copy_from_slice(&data[offset..offset + 24]);
        offset += 24;

        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[offset..offset + 64]);
        offset += 64;

        // Payload (2 + N bytes)
        let ciphertext_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if data.len() < offset + ciphertext_len {
            return Err(DriftError::BufferTooShort {
                need: offset + ciphertext_len,
                got: data.len(),
            });
        }

        let ciphertext = if compressed {
            super::compress::decompress(&data[offset..offset + ciphertext_len])?
        } else {
            data[offset..offset + ciphertext_len].to_vec()
        };
        offset += ciphertext_len;

        // Ratchet extension (backward compatible — older envelopes don't have this)
        let (ratchet_dh_public, ratchet_message_number) = if offset < data.len() {
            let ratchet_flag = data[offset];
            offset += 1;
            if ratchet_flag == 0x01 {
                if offset + 32 + 4 <= data.len() {
                    let mut dh_key = [0u8; 32];
                    dh_key.copy_from_slice(&data[offset..offset + 32]);
                    offset += 32;
                    let msg_num = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    offset += 4;
                    (Some(dh_key), Some(msg_num))
                } else {
                    return Err(DriftError::BufferTooShort {
                        need: offset + 32 + 4,
                        got: data.len(),
                    });
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // PQ extension (backward compatible — older envelopes don't have this)
        let (suite, pq_kem_ciphertext, pq_encaps_key, transcript_hash) = if offset < data.len() {
            let pq_flag = data[offset];
            offset += 1;
            if pq_flag == 0x01 {
                let suite = data[offset];
                offset += 1;

                // Helper function to read a blob (length + data)
                let read_blob =
                    |data: &[u8], offset: &mut usize| -> Result<Option<Vec<u8>>, DriftError> {
                        if *offset + 4 > data.len() {
                            return Err(DriftError::BufferTooShort {
                                need: *offset + 4,
                                got: data.len(),
                            });
                        }

                        let len = u32::from_le_bytes([
                            data[*offset],
                            data[*offset + 1],
                            data[*offset + 2],
                            data[*offset + 3],
                        ]) as usize;
                        *offset += 4;

                        if len == 0 {
                            Ok(None)
                        } else {
                            if *offset + len > data.len() {
                                return Err(DriftError::BufferTooShort {
                                    need: *offset + len,
                                    got: data.len(),
                                });
                            }

                            let blob = data[*offset..*offset + len].to_vec();
                            *offset += len;
                            Ok(Some(blob))
                        }
                    };

                let pq_kem_ciphertext = read_blob(data, &mut offset)?;
                let pq_encaps_key = read_blob(data, &mut offset)?;
                let transcript_hash = read_blob(data, &mut offset)?;

                (
                    Some(suite),
                    pq_kem_ciphertext,
                    pq_encaps_key,
                    transcript_hash,
                )
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        Ok(DriftEnvelope {
            version,
            envelope_type,
            compressed,
            message_id,
            recipient_hint,
            created_at,
            ttl_expiry,
            hop_count,
            priority,
            sender_public_key,
            ephemeral_public_key,
            nonce,
            signature,
            ciphertext,
            ratchet_dh_public,
            ratchet_message_number,
            suite,
            pq_kem_ciphertext,
            pq_encaps_key,
            transcript_hash,
        })
    }

    /// Get the recipient hint from a public key using blake3
    pub fn hint_from_public_key(public_key: &[u8; 32]) -> [u8; 4] {
        let hash = blake3::hash(public_key);
        let mut hint = [0u8; 4];
        hint.copy_from_slice(&hash.as_bytes()[..4]);
        hint
    }

    /// Increment hop count (for relay forwarding)
    ///
    /// Uses saturating arithmetic to prevent overflow.
    pub fn increment_hop(&mut self) {
        self.hop_count = self.hop_count.saturating_add(1);
    }

    /// Check if message has expired
    ///
    /// Returns false if ttl_expiry is 0 (never expires).
    pub fn is_expired(&self) -> bool {
        if self.ttl_expiry == 0 {
            return false;
        }

        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        now > self.ttl_expiry
    }

    /// Convert to a legacy message::Envelope for use with crypto::decrypt_message.
    pub fn to_legacy_envelope(&self) -> Envelope {
        Envelope {
            sender_public_key: self.sender_public_key.to_vec(),
            ephemeral_public_key: self.ephemeral_public_key.to_vec(),
            nonce: self.nonce.to_vec(),
            ciphertext: self.ciphertext.clone(),
            ratchet_dh_public: self.ratchet_dh_public.map(|k| k.to_vec()),
            ratchet_message_number: self.ratchet_message_number,
        }
    }
}

impl DriftEnvelope {
    /// Convert from a legacy message::Envelope to DriftEnvelope
    ///
    /// This converts the bincode-based envelope format to the optimized
    /// Drift Protocol binary format with additional routing metadata.
    pub fn from_legacy_envelope(
        legacy: Envelope,
        message_id: String,
        recipient_public_key: [u8; 32],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self, DriftError> {
        // Parse message ID as UUID bytes
        let uuid = Uuid::parse_str(&message_id)
            .map_err(|e| DriftError::IoError(format!("Invalid message ID: {}", e)))?;
        let message_id_bytes = *uuid.as_bytes();

        // Calculate recipient hint for routing
        let recipient_hint = Self::hint_from_public_key(&recipient_public_key);

        // Get current timestamp
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| DriftError::IoError(format!("Time error: {}", e)))?
            .as_secs() as u32;

        // TTL: 7 days by default
        let ttl_expiry = created_at + DEFAULT_TTL_SECS;

        // Convert Vec<u8> fields to fixed-size arrays
        let sender_public_key: [u8; 32] = legacy
            .sender_public_key
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid sender public key length".into()))?;

        let ephemeral_public_key: [u8; 32] = legacy
            .ephemeral_public_key
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid ephemeral public key length".into()))?;

        let nonce: [u8; 24] = legacy
            .nonce
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid nonce length".into()))?;

        // Create the envelope without signature first
        let ratchet_dh_public = legacy
            .ratchet_dh_public
            .map(|v| -> Result<[u8; 32], DriftError> {
                v.try_into()
                    .map_err(|_| DriftError::IoError("Invalid ratchet_dh_public length".into()))
            })
            .transpose()?;

        let mut drift_env = DriftEnvelope {
            version: DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            compressed: legacy.ciphertext.len() > COMPRESSION_THRESHOLD,
            message_id: message_id_bytes,
            recipient_hint,
            created_at,
            ttl_expiry,
            hop_count: 0,  // Initial hop count
            priority: 128, // Medium priority
            sender_public_key,
            ephemeral_public_key,
            nonce,
            signature: [0u8; 64], // Placeholder, will be filled
            ciphertext: legacy.ciphertext,
            ratchet_dh_public,
            ratchet_message_number: legacy.ratchet_message_number,
            suite: None,
            pq_kem_ciphertext: None,
            pq_encaps_key: None,
            transcript_hash: None,
        };

        // Sign the envelope
        let signature = drift_env.sign(signing_key);
        drift_env.signature = signature;

        Ok(drift_env)
    }

    /// Convert from a V2 envelope to DriftEnvelope
    pub fn from_v2_envelope(
        v2: crate::message::EnvelopeV2,
        message_id: String,
        recipient_public_key: [u8; 32],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self, DriftError> {
        // Parse message ID as UUID bytes
        let uuid = Uuid::parse_str(&message_id)
            .map_err(|e| DriftError::IoError(format!("Invalid message ID: {}", e)))?;
        let message_id_bytes = *uuid.as_bytes();

        // Calculate recipient hint for routing
        let recipient_hint = Self::hint_from_public_key(&recipient_public_key);

        // Get current timestamp
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| DriftError::IoError(format!("Time error: {}", e)))?
            .as_secs() as u32;

        // TTL: 7 days by default
        let ttl_expiry = created_at + DEFAULT_TTL_SECS;

        // Convert Vec<u8> fields to fixed-size arrays
        let sender_public_key: [u8; 32] = v2
            .sender_public_key
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid sender public key length".into()))?;

        let ephemeral_public_key: [u8; 32] = v2
            .ephemeral_public_key
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid ephemeral public key length".into()))?;

        let nonce: [u8; 24] = v2
            .nonce
            .try_into()
            .map_err(|_| DriftError::IoError("Invalid nonce length".into()))?;

        // Validate ratchet_dh_public if present
        let ratchet_dh_public = v2
            .ratchet_dh_public
            .map(|v| -> Result<[u8; 32], DriftError> {
                v.try_into()
                    .map_err(|_| DriftError::IoError("Invalid ratchet_dh_public length".into()))
            })
            .transpose()?;

        // Create the envelope without signature first
        let mut drift_env = DriftEnvelope {
            version: DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            compressed: v2.ciphertext.len() > COMPRESSION_THRESHOLD,
            message_id: message_id_bytes,
            recipient_hint,
            created_at,
            ttl_expiry,
            hop_count: 0,  // Initial hop count
            priority: 128, // Medium priority
            sender_public_key,
            ephemeral_public_key,
            nonce,
            signature: [0u8; 64], // Placeholder, will be filled
            ciphertext: v2.ciphertext,
            ratchet_dh_public,
            ratchet_message_number: v2.ratchet_message_number,
            suite: Some(v2.suite),
            pq_kem_ciphertext: v2.pq_kem_ciphertext,
            pq_encaps_key: v2.pq_encaps_key,
            transcript_hash: v2.transcript_hash,
        };

        // Sign the envelope
        let signature = drift_env.sign(signing_key);
        drift_env.signature = signature;

        Ok(drift_env)
    }

    /// Convert to a wire envelope based on presence of PQ fields
    pub fn to_wire_envelope(&self) -> crate::message::WireEnvelope {
        if self.suite.is_some()
            || self.pq_kem_ciphertext.is_some()
            || self.pq_encaps_key.is_some()
            || self.transcript_hash.is_some()
        {
            crate::message::WireEnvelope::V2(crate::message::EnvelopeV2 {
                suite: self.suite.unwrap_or(0x02),
                sender_public_key: self.sender_public_key.to_vec(),
                ephemeral_public_key: self.ephemeral_public_key.to_vec(),
                nonce: self.nonce.to_vec(),
                ciphertext: self.ciphertext.clone(),
                ratchet_dh_public: self.ratchet_dh_public.map(|k| k.to_vec()),
                ratchet_message_number: self.ratchet_message_number,
                pq_kem_ciphertext: self.pq_kem_ciphertext.clone(),
                pq_encaps_key: self.pq_encaps_key.clone(),
                transcript_hash: self.transcript_hash.clone(),
            })
        } else {
            crate::message::WireEnvelope::V1(self.to_legacy_envelope())
        }
    }

    /// Compute the blake3 hash of all envelope fields except the signature itself.
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Header and routing fields
        hasher.update(&[self.version]);
        hasher.update(&[
            self.envelope_type.as_u8() | if self.compressed { COMPRESSION_FLAG } else { 0 }
        ]);
        hasher.update(&self.message_id);
        hasher.update(&self.recipient_hint);
        hasher.update(&self.created_at.to_le_bytes());
        hasher.update(&self.ttl_expiry.to_le_bytes());
        hasher.update(&[self.hop_count]);
        hasher.update(&[self.priority]);

        // Crypto fields (except signature)
        hasher.update(&self.sender_public_key);
        hasher.update(&self.ephemeral_public_key);
        hasher.update(&self.nonce);

        // Payload
        hasher.update(&(self.ciphertext.len() as u16).to_le_bytes());
        hasher.update(&self.ciphertext);

        // Ratchet extension (if present)
        if let Some(ref dh_public) = self.ratchet_dh_public {
            hasher.update(&[0x01]); // ratchet flag
            hasher.update(dh_public);
            hasher.update(&self.ratchet_message_number.unwrap_or(0).to_le_bytes());
        } else {
            hasher.update(&[0x00]); // no ratchet
        }

        // PQ extension (if present)
        if self.suite.is_some()
            || self.pq_kem_ciphertext.is_some()
            || self.pq_encaps_key.is_some()
            || self.transcript_hash.is_some()
        {
            hasher.update(&[0x01]);
            hasher.update(&[self.suite.unwrap_or(0x02)]);
            let hash_blob = |h: &mut blake3::Hasher, v: &Option<Vec<u8>>| {
                let d = v.as_deref().unwrap_or(&[]);
                h.update(&(d.len() as u32).to_le_bytes());
                h.update(d);
            };
            hash_blob(&mut hasher, &self.pq_kem_ciphertext);
            hash_blob(&mut hasher, &self.pq_encaps_key);
            hash_blob(&mut hasher, &self.transcript_hash);
        }

        *hasher.finalize().as_bytes()
    }

    /// Sign the envelope contents with the sender's signing key.
    pub fn sign(&self, signing_key: &ed25519_dalek::SigningKey) -> [u8; 64] {
        let hash = self.signing_hash();
        let signature = signing_key.sign(&hash);
        signature.to_bytes()
    }

    /// Verify the Ed25519 signature over canonical envelope bytes against sender_public_key.
    pub fn verify(&self) -> Result<(), DriftError> {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        let verifying_key = VerifyingKey::from_bytes(&self.sender_public_key)
            .map_err(|e| DriftError::IoError(format!("Invalid sender public key: {}", e)))?;

        let signature = Signature::from_bytes(&self.signature);

        let hash = self.signing_hash();
        verifying_key
            .verify(&hash, &signature)
            .map_err(|e| DriftError::IoError(format!("Signature verification failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_envelope() -> DriftEnvelope {
        DriftEnvelope {
            version: DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            compressed: false,
            message_id: [1u8; 16],
            recipient_hint: [2u8; 4],
            created_at: 1234567890,
            ttl_expiry: 1234567900,
            hop_count: 5,
            priority: 10,
            sender_public_key: [3u8; 32],
            ephemeral_public_key: [4u8; 32],
            nonce: [5u8; 24],
            signature: [6u8; 64],
            ciphertext: b"test payload".to_vec(),
            ratchet_dh_public: None,
            ratchet_message_number: None,
            suite: None,
            pq_kem_ciphertext: None,
            pq_encaps_key: None,
            transcript_hash: None,
        }
    }

    #[test]
    fn test_envelope_type_conversion() {
        assert_eq!(EnvelopeType::EncryptedMessage.as_u8(), 0x01);
        assert_eq!(EnvelopeType::DeliveryReceipt.as_u8(), 0x02);
        assert_eq!(EnvelopeType::SyncRequest.as_u8(), 0x03);
        assert_eq!(EnvelopeType::SyncResponse.as_u8(), 0x04);
        assert_eq!(EnvelopeType::PeerAnnouncement.as_u8(), 0x05);
        assert_eq!(EnvelopeType::RouteAdvertisement.as_u8(), 0x06);
        assert_eq!(EnvelopeType::CoverTraffic.as_u8(), 0x07);
        assert_eq!(EnvelopeType::OnionMessage.as_u8(), 0x08);

        assert_eq!(
            EnvelopeType::from_u8(0x01).unwrap(),
            EnvelopeType::EncryptedMessage
        );
        assert_eq!(
            EnvelopeType::from_u8(0x02).unwrap(),
            EnvelopeType::DeliveryReceipt
        );
        assert_eq!(
            EnvelopeType::from_u8(0x07).unwrap(),
            EnvelopeType::CoverTraffic
        );
        assert_eq!(
            EnvelopeType::from_u8(0x08).unwrap(),
            EnvelopeType::OnionMessage
        );
        assert!(EnvelopeType::from_u8(0x99).is_err());
    }

    #[test]
    fn test_envelope_serialize_deserialize() {
        let original = make_test_envelope();
        let bytes = original.to_bytes().unwrap();

        assert_eq!(
            bytes.len(),
            DriftEnvelope::FIXED_OVERHEAD + original.ciphertext.len()
        );

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_envelope_empty_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![];

        let bytes = env.to_bytes().unwrap();
        assert_eq!(bytes.len(), DriftEnvelope::FIXED_OVERHEAD);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), 0);
    }

    #[test]
    fn test_envelope_large_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0xAB; 10000];

        let bytes = env.to_bytes().unwrap();
        assert_eq!(bytes.len(), DriftEnvelope::FIXED_OVERHEAD + 10000);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), 10000);
        assert!(restored.ciphertext.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_envelope_max_ciphertext() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0x42; DriftEnvelope::MAX_CIPHERTEXT];

        let bytes = env.to_bytes().unwrap();
        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.ciphertext.len(), DriftEnvelope::MAX_CIPHERTEXT);
    }

    #[test]
    fn test_envelope_ciphertext_too_large() {
        let mut env = make_test_envelope();
        env.ciphertext = vec![0x42; DriftEnvelope::MAX_CIPHERTEXT + 1];

        let result = env.to_bytes();
        assert!(matches!(result, Err(DriftError::CiphertextTooLarge(_))));
    }

    #[test]
    fn test_hint_from_public_key_deterministic() {
        let pk = [42u8; 32];
        let hint1 = DriftEnvelope::hint_from_public_key(&pk);
        let hint2 = DriftEnvelope::hint_from_public_key(&pk);

        assert_eq!(hint1, hint2);
    }

    #[test]
    fn test_hint_from_public_key_different_keys() {
        let pk1 = [1u8; 32];
        let pk2 = [2u8; 32];

        let hint1 = DriftEnvelope::hint_from_public_key(&pk1);
        let hint2 = DriftEnvelope::hint_from_public_key(&pk2);

        assert_ne!(hint1, hint2);
    }

    #[test]
    fn test_increment_hop() {
        let mut env = make_test_envelope();
        env.hop_count = 5;

        env.increment_hop();
        assert_eq!(env.hop_count, 6);

        env.hop_count = 255;
        env.increment_hop();
        assert_eq!(env.hop_count, 255); // saturating
    }

    #[test]
    fn test_is_expired_never_expires() {
        let mut env = make_test_envelope();
        env.ttl_expiry = 0;

        assert!(!env.is_expired());
    }

    #[test]
    fn test_is_expired_in_future() {
        let mut env = make_test_envelope();
        env.ttl_expiry = u32::MAX;

        assert!(!env.is_expired());
    }

    #[test]
    fn test_is_expired_in_past() {
        let mut env = make_test_envelope();
        env.ttl_expiry = 1;

        assert!(env.is_expired());
    }

    #[test]
    fn test_buffer_too_short() {
        let data = [0u8; 100];
        let result = DriftEnvelope::from_bytes(&data);

        match result {
            Err(DriftError::BufferTooShort { need, got }) => {
                assert_eq!(need, DriftEnvelope::FIXED_OVERHEAD);
                assert_eq!(got, 100);
            }
            other => panic!("Expected BufferTooShort, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_version() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD + 5];
        data[0] = 0xFF; // Invalid version

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(result, Err(DriftError::InvalidVersion(0xFF))));
    }

    #[test]
    fn test_invalid_envelope_type() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD + 5];
        data[0] = DRIFT_VERSION;
        data[1] = 0x19; // Invalid type (not 0x01-0x06, no compression flag)

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(result, Err(DriftError::InvalidEnvelopeType(_))));
    }

    #[test]
    fn test_ciphertext_length_exceeds_buffer() {
        let mut data = vec![0u8; DriftEnvelope::FIXED_OVERHEAD];
        data[0] = DRIFT_VERSION;
        data[1] = 0x01; // EncryptedMessage

        // Set ciphertext length to 1000 but don't provide that much data
        let offset = DriftEnvelope::FIXED_OVERHEAD - 2;
        data[offset] = 232; // (1000 & 0xFF)
        data[offset + 1] = 3; // (1000 >> 8)

        let result = DriftEnvelope::from_bytes(&data);
        assert!(matches!(result, Err(DriftError::BufferTooShort { .. })));
    }

    #[test]
    fn test_little_endian_timestamps() {
        let mut env = make_test_envelope();
        env.created_at = 0x12345678;
        env.ttl_expiry = 0xABCDEF00;

        let bytes = env.to_bytes().unwrap();

        // Check that timestamps are stored little-endian
        let created_at_offset = 18 + 4; // After version, type, message_id, recipient_hint
        assert_eq!(bytes[created_at_offset], 0x78);
        assert_eq!(bytes[created_at_offset + 1], 0x56);
        assert_eq!(bytes[created_at_offset + 2], 0x34);
        assert_eq!(bytes[created_at_offset + 3], 0x12);

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert_eq!(restored.created_at, 0x12345678);
        assert_eq!(restored.ttl_expiry, 0xABCDEF00);
    }

    #[test]
    fn test_compressed_envelope_roundtrip() {
        let mut env = make_test_envelope();
        // Use a large, compressible payload
        env.ciphertext = vec![0xABu8; 5000];
        env.compressed = true;

        let bytes = env.to_bytes().unwrap();

        // The serialized form should be smaller than uncompressed
        let uncompressed_size = DriftEnvelope::FIXED_OVERHEAD + 5000;
        assert!(
            bytes.len() < uncompressed_size,
            "Compressed {} < uncompressed {}",
            bytes.len(),
            uncompressed_size
        );

        // The type byte should have the compression flag set
        assert!(
            bytes[1] & COMPRESSION_FLAG != 0,
            "Compression flag should be set"
        );

        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert!(restored.compressed);
        assert_eq!(restored.ciphertext.len(), 5000);
        assert!(restored.ciphertext.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_compression_flag_preserved_on_roundtrip() {
        let mut env = make_test_envelope();
        env.compressed = false;
        let bytes = env.to_bytes().unwrap();
        assert!(bytes[1] & COMPRESSION_FLAG == 0, "No compression flag");
        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert!(!restored.compressed);

        env.compressed = true;
        let bytes = env.to_bytes().unwrap();
        assert!(bytes[1] & COMPRESSION_FLAG != 0, "Compression flag set");
        let restored = DriftEnvelope::from_bytes(&bytes).unwrap();
        assert!(restored.compressed);
    }

    #[test]
    fn test_drift_envelope_sign_and_verify() {
        use rand::RngCore;

        let mut secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let mut env = make_test_envelope();
        env.sender_public_key = verifying_key.to_bytes();
        let sig = env.sign(&signing_key);
        env.signature = sig;

        // Valid signature verifies successfully
        assert!(env.verify().is_ok());

        // Tampering with payload fails verification
        let mut tampered_payload = env.clone();
        tampered_payload.ciphertext.push(0xFF);
        assert!(tampered_payload.verify().is_err());

        // Tampering with sender public key fails verification
        let mut tampered_sender = env.clone();
        tampered_sender.sender_public_key[0] ^= 0xFF;
        assert!(tampered_sender.verify().is_err());

        // Zero / dummy signature fails verification
        let mut zero_sig = env.clone();
        zero_sig.signature = [0u8; 64];
        assert!(zero_sig.verify().is_err());
    }
}
