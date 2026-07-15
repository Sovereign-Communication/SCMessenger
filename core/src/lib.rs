//! SCMessenger Core Library
//! 
//! This crate provides the core functionality for the SCMessenger P2P messaging system.
//! It includes identity management, encryption, transport layer, message handling,
//! and persistent storage capabilities.

pub mod abuse;
pub mod crypto;
pub mod drift;
pub mod dspy;
pub mod error;
pub mod identity;
pub mod iron_core;
pub mod message;
pub mod mobile_bridge;
pub mod notification;
pub mod observability;
pub mod privacy;
pub mod relay;
pub mod routing;
pub mod settings;
pub mod store;
pub mod transport;
pub mod wasm_support;

// Re-export critical types from core modules
pub use message::types::{Receipt, MessageType};
pub use message::codec::decode_envelope;
pub use store::outbox::RetryPolicy;

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
                max_retries: 3,           // CLI baseline
                initial_delay_ms: 100,    // CLI baseline
                backoff_factor: 2,        // exponential: 100ms, 200ms, 400ms
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
            let power = (attempt - 2) as u32;
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
            assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(100)));
            assert_eq!(policy.delay_for_attempt(3), Some(Duration::from_millis(200)));
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

// UniFFI support - ensure UniFfiTag is available for all UniFFI macros
pub struct UniFfiTag;

// Re-export for convenience in modules using UniFFI
pub use uniffi::deps::anyhow;

// Include UniFFI scaffolding if the gen-bindings feature is enabled
#[cfg(feature = "gen-bindings")]
uniffi::include_scaffolding!("scmessenger");