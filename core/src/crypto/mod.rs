// Cryptography module — message encryption and key exchange

// CI hardening (see docs/rules/SECURITY_PROTOCOL.md): a truly-unused,
// non-underscore-prefixed parameter here is now a hard compile error, not
// just a CI-only `-D warnings` clippy failure. This is deliberately narrow
// (module-scoped, not workspace-wide) and only covers half the threat model
// -- rustc's own escape hatch means a parameter renamed `_foo` is exempt
// from `unused_variables` by design, at any deny level. The other half
// (underscore-prefixed ignored parameters, the actual shape of the historical
// sender-impersonation bug in RatchetSession) is enforced separately by
// scripts/check_perimeter_underscore_params.py in CI, which this attribute
// cannot replace.
#![deny(unused_variables)]

pub mod backup;
pub mod encrypt;
pub mod negotiation;
pub mod pq;
pub mod ratchet;
pub mod session_manager;

#[cfg(test)]
mod proptest_harness;

#[cfg(feature = "kani-proofs")]
mod kani_proofs;

pub use encrypt::{
    decrypt_message, decrypt_message_ratcheted, decrypt_with_ratchet_fallback,
    ed25519_public_to_x25519, ed25519_to_x25519_secret, encrypt_message, encrypt_message_ratcheted,
    encrypt_with_ratchet_fallback, is_ratcheted_envelope, sign_envelope, sign_envelope_v2,
    validate_ed25519_public_key, verify_envelope, verify_envelope_v2,
};
pub use ratchet::{RatchetEncryptResult, RatchetKey, RatchetSession};
pub use session_manager::{RatchetSessionManager, SerializableRatchetSession};

// Re-export DSPy signature functions for crypto signature verification integration
pub use crate::dspy::signatures::{blake3_hash, get_signature, signature_fingerprint};
