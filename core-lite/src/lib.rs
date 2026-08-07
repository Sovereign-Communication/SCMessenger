//! scmessenger-core-lite — Minimal core for direct messaging + optional mesh relay
//!
//! Feature-gated modules:
//! - `lite` (default): identity, crypto, message, store, transport, direct routing
//! - `ratchet`: forward-secret ratchet sessions
//! - `sled-storage`: persistent sled backend
//! - `tcp`, `ble`, `wifi-aware`, `wifi-direct`, `multipeer`: transport backends
//! - `lite-mesh`: drift-core + relay-core + routing-mesh + relay-custody + ledger-sharing

#![allow(clippy::empty_line_after_outer_attr)]

// Core modules (always included)
pub mod identity;
pub mod crypto;
pub mod message;
pub mod store;
pub mod transport;
pub mod routing;
pub mod error;
pub mod settings;

// Optional modules (feature-gated)
#[cfg(feature = "ratchet")]
pub mod ratchet;

#[cfg(feature = "drift-core")]
pub mod drift;

#[cfg(feature = "relay-core")]
pub mod relay;

#[cfg(feature = "relay-custody")]
pub mod relay_custody;

// Re-export critical types
pub use error::{MeshError, MeshResult};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, TtlConfig, Receipt};
pub use message::codec::{decode_envelope, decode_message, encode_envelope, encode_message};
pub use store::{Contact, ContactManager, HistoryManager, Inbox, MessageDirection, MessageRecord, Outbox, QueuedMessage, ReceivedMessage, StorageBackend, StorageManager};
pub use transport::{TransportManager, TransportType, SwarmCommand, SwarmEvent, SwarmHandle};
pub use routing::LocalRouter;
pub use settings::MeshSettingsLite;

// Core coordinator
pub mod core;
pub use core::IronCoreLite;

// UniFFI scaffolding
#[cfg(not(target_arch = "wasm32"))]
uniffi::include_scaffolding!("api_lite");