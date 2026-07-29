//! SCMessenger Core Library
//!
//! This crate provides the core functionality for the SCMessenger P2P messaging system.
//! It includes identity management, encryption, transport layer, message handling,
//! and persistent storage capabilities.

pub mod abuse;
#[cfg(not(target_arch = "wasm32"))]
pub mod blocked_bridge;
#[cfg(not(target_arch = "wasm32"))]
pub mod contacts_bridge;
pub mod crypto;
pub mod drift;
pub mod dspy;
pub mod error;
pub mod identity;
pub mod iron_core;
pub mod message;
#[cfg(not(target_arch = "wasm32"))]
pub mod mobile_bridge;
pub mod notification;
pub mod notification_defaults;
pub mod observability;
pub mod privacy;
pub mod relay;
pub mod routing;
pub mod settings;
pub mod store;
pub mod transport;
pub mod wasm_support;

// Re-export critical types from core modules
pub use error::MeshError;
pub use iron_core::{CoreDelegate, IronCore};

// IronCoreError — defined in Rust rather than generated from UDL
// because the UDL-based scaffolding requires interface types to have
// matching Rust implementations (which are wired in Phase 1B).
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum IronCoreError {
    #[error("Identity not initialized")]
    NotInitialized,
    #[error("Service already running")]
    AlreadyRunning,
    #[error("Storage error")]
    StorageError,
    #[error("Cryptographic error")]
    CryptoError,
    #[error("Network error")]
    NetworkError,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Peer is blocked")]
    Blocked,
    #[error("Consent required")]
    ConsentRequired,
    #[error("Internal error")]
    Internal,
    #[error("Data corruption detected")]
    CorruptionDetected,
    #[error("Dial self")]
    DialSelf,
    #[error("No addresses")]
    NoAddresses,
    #[error("Connection limit reached")]
    ConnectionLimit,
    #[error("Multiaddress not supported")]
    MultiaddrNotSupported,
    #[error("IO error")]
    IoError,
    #[error("Onion routing disabled")]
    OnionRoutingDisabled,
pub use crypto::{decrypt_message, encrypt_message};
pub use error::{
    MeshResult, SerializationError, SerializationResult, TransportError, TransportResult,
};
pub use identity::IdentityManager;
pub use message::codec::decode_envelope;
pub use message::types::Receipt;
// FFI-facing receipt codec wrappers (owned args, IronCoreError) — these are what
// the UniFFI scaffolding binds; the canonical serializers live in message::types.
#[cfg(not(target_arch = "wasm32"))]
pub use iron_core::{decode_receipt, encode_receipt};
#[cfg(target_arch = "wasm32")]
pub use message::types::{decode_receipt, encode_receipt};
pub use message::MessageType;
pub use message::{DeliveryStatus, Envelope, Message, TtlConfig};
pub use notification::{
    classify_notification as classify_notification_policy, NotificationDecision,
    NotificationEndpoint, NotificationEndpointCapabilities, NotificationEndpointError,
    NotificationEndpointRegistry, NotificationKind, NotificationMessageContext,
    NotificationPlatform, NotificationUiState,
};
pub use observability::{AuditEvent, AuditEventType};
pub use settings::{DiscoveryMode, MeshSettings};
pub use store::ledger_entry::{LedgerEntry, SeedLedgerEntry, MAX_SEED_LEDGER_ENTRIES};
pub use store::outbox::RetryPolicy;
pub use transport::{start_swarm, start_swarm_with_config, SwarmCommand, SwarmEvent, SwarmHandle};

// Mobile bridge exports for UniFFI
#[cfg(not(target_arch = "wasm32"))]
pub use blocked_bridge::{
    blocked_identity_new, blocked_identity_with_device_id, blocked_identity_with_notes,
    blocked_identity_with_reason, BlockedIdentity, BlockedManager,
};
#[cfg(not(target_arch = "wasm32"))]
pub use contacts_bridge::{Contact, ContactManager};
#[cfg(not(target_arch = "wasm32"))]
pub use mobile_bridge::*;

#[derive(Default)]
pub struct IdentityInfo {
    pub identity_id: Option<String>,
    pub public_key_hex: Option<String>,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
    pub initialized: bool,
    pub nickname: Option<String>,
    pub libp2p_peer_id: Option<String>,
}

pub struct SignatureResult {
    pub signature: Vec<u8>,
    pub public_key_hex: String,
}

pub struct PreparedMessage {
    pub message_id: String,
    pub envelope_data: Vec<u8>,
}

pub struct PeelResult {
    pub next_hop: Option<Vec<u8>>,
    pub remaining_data: Vec<u8>,
}

pub struct MessageRequest {
    pub peer_id: String,
    pub nickname: Option<String>,
    pub message_preview: String,
    pub message_timestamp: u64,
    pub message_count: u32,
}

pub struct RegistrationStateInfo {
    pub state: String,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
}

// Build provenance information
pub fn get_build_provenance() -> String {
    option_env!("SCM_GIT_HASH")
        .map(|hash| format!("{} ({})", env!("CARGO_PKG_VERSION"), hash))
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// Retry policy module - centralized retry logic for all platforms
pub mod retry_policy {
    use std::time::Duration;

    /// Retry configuration for message delivery.
    ///
    /// This is the ONLY place retry policy is defined. All platforms
    /// (CLI, Android, iOS, WASM) use this struct. Changes to backoff
    /// strategy apply everywhere automatically.
    #[derive(Debug, Clone)]
    pub struct RetryPolicy {
        /// Maximum number of retry attempts (including initial attempt).
        pub max_retries: u32,
        /// Initial delay in milliseconds before first retry.
        pub initial_delay_ms: u64,
        /// Backoff multiplier (2 = exponential, 1 = fixed).
        pub backoff_factor: u32,
    }

    impl Default for RetryPolicy {
        fn default() -> Self {
            Self {
                max_retries: 3,        // CLI baseline
                initial_delay_ms: 100, // CLI baseline
                backoff_factor: 2,     // exponential: 100ms, 200ms, 400ms
            }
        }
    }

    impl RetryPolicy {
        /// Compute the delay before the given attempt (1-indexed).
        ///
        /// Returns None if attempt exceeds max_retries (delivery should be abandoned).
        pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
            if attempt > self.max_retries {
                return None;
            }
            if attempt == 1 {
                // No delay for initial attempt
                return Some(Duration::from_millis(0));
            }
            // exponential: delay = initial * (backoff ^ (attempt - 2))
            // attempt 2: delay = initial * 1 = 100ms
            // attempt 3: delay = initial * 2 = 200ms
            // attempt 4: delay = initial * 4 = 400ms
            let power = attempt - 2;
            let multiplier = (self.backoff_factor as u64).saturating_pow(power);
            let delay_ms = self.initial_delay_ms.saturating_mul(multiplier);
            Some(Duration::from_millis(delay_ms))
        }

        /// Whether another retry is possible.
        pub fn can_retry(&self, attempt: u32) -> bool {
            attempt < self.max_retries
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_default_retry_delays() {
            let policy = RetryPolicy::default();
            assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(0)));
            assert_eq!(
                policy.delay_for_attempt(2),
                Some(Duration::from_millis(100))
            );
            assert_eq!(
                policy.delay_for_attempt(3),
                Some(Duration::from_millis(200))
            );
            assert!(policy.delay_for_attempt(4).is_none()); // exceeds max_retries
        }

        #[test]
        fn test_can_retry() {
            let policy = RetryPolicy::default();
            assert!(policy.can_retry(1));
            assert!(policy.can_retry(2));
            assert!(!policy.can_retry(3)); // 3 is the max
        }
    }
}

// Re-export for convenience in modules using UniFFI
pub use uniffi::deps::anyhow;

/// Canonical gossipsub topic names, shared across CLI/Android/iOS so no
/// platform can drift from a hand-copied string literal. Values match the
/// wire-format strings already live in core/src/transport/swarm.rs and
/// cli/src/bootstrap.rs -- do not change these without a coordinated
/// migration, mismatched values silently partition the mesh.
pub const TOPIC_LOBBY: &str = "sc-lobby";
pub const TOPIC_MESH: &str = "sc-mesh";
pub const TOPIC_RECEIPT_CONVERGENCE: &str = "sc-receipt-convergence";

// Include UniFFI scaffolding
#[cfg(not(target_arch = "wasm32"))]
uniffi::include_scaffolding!("api");
