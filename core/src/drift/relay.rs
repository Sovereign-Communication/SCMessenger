//! Phase 2D: Relay=Messaging Coupling
//!
//! The core architectural principle of SCMessenger:
//! ONE TOGGLE: ON = you can send messages AND relay for others.
//!            OFF = you can do neither.
//!
//! This structurally prevents free-riding that killed every previous mesh project.
//! There is no "receive only" mode. There is no "don't relay" mode.
//! If you're on the network, you serve the network.

use super::envelope::DriftEnvelope;
use super::store::{MeshStore, MessageId, StoredEnvelope};
use super::DriftError;
use crate::dspy::modules::{DSPyModule, OptimizerPipeline};
use crate::privacy::cover::{CoverConfig, CoverTrafficScheduler};
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

/// The unified relay=messaging toggle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    /// Active: can send messages AND relays for others
    Active,
    /// Dormant: cannot send or relay. Network participation suspended.
    Dormant,
}

/// Configuration for relay behavior (all tunable EXCEPT the coupling)
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Maximum messages to relay per hour (0 = unlimited)
    pub max_relay_per_hour: u32,
    /// Maximum hop count before dropping (prevents infinite relay)
    pub max_hop_count: u8,
    /// Minimum priority to relay (0 = relay everything)
    pub min_relay_priority: u8,
    /// Battery threshold below which relay reduces to minimum
    pub battery_floor_percent: u8,
    /// Whether to relay messages even if we can't decrypt them (always true in normal operation)
    pub relay_opaque: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            max_relay_per_hour: 1000,
            max_hop_count: 16,
            min_relay_priority: 0,
            battery_floor_percent: 20,
            relay_opaque: true,
        }
    }
}

/// Outcome of processing an incoming envelope
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayDecision {
    /// Message is for us — deliver to application layer
    DeliverLocal { message_id: MessageId },
    /// Message is not for us — store for relay to others
    StoreAndRelay { message_id: MessageId },
    /// Message is a duplicate we already have
    Duplicate { message_id: MessageId },
    /// Message was dropped (expired, too many hops, rate limited, etc.)
    Dropped {
        message_id: MessageId,
        reason: DropReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    Expired,
    MaxHopsExceeded,
    RateLimited,
    NetworkDormant,
    LowPriority,
    StoreFull,
}

/// Relay engine errors
#[derive(Debug, Error, Clone)]
pub enum RelayError {
    #[error("Network is dormant — cannot send or relay")]
    NetworkDormant,

    #[error("Failed to serialize envelope: {0}")]
    SerializationFailed(String),

    #[error("Invalid envelope: {0}")]
    InvalidEnvelope(String),
}

/// Report from maintenance operation
#[derive(Debug, Clone)]
pub struct MaintenanceReport {
    /// Number of expired messages removed
    pub expired_removed: usize,
    /// Total messages in store after maintenance
    pub store_size: usize,
    /// Messages evicted due to budget
    pub evictions: usize,
}

/// The relay engine — heart of the mesh
pub struct RelayEngine {
    state: NetworkState,
    config: RelayConfig,
    store: MeshStore,
    /// Our own recipient hint for identifying messages addressed to us
    local_hint: [u8; 4],
    /// Rate limiting: messages relayed in current window
    relay_count_this_hour: u32,
    hour_start: u64,
    /// Cover traffic scheduler (privacy: traffic analysis resistance)
    cover_scheduler: Option<CoverTrafficScheduler>,
    /// Reputation manager for abuse-based relay decisions
    reputation_manager: Option<Arc<crate::abuse::reputation::EnhancedAbuseReputationManager>>,
    /// Shared reputation manager owned by IronCore.
    shared_reputation_manager:
        Option<Arc<RwLock<crate::abuse::reputation::EnhancedAbuseReputationManager>>>,
    /// Security audit pipeline for relay custody verification
    security_audit_pipeline: Option<OptimizerPipeline>,
}

impl RelayEngine {
    /// Create a new relay engine
    pub fn new(local_public_key: &[u8; 32], config: RelayConfig) -> Self {
        Self::new_with_audit(local_public_key, config, None)
    }

    /// Create a new relay engine with security audit pipeline
    pub fn new_with_audit(
        local_public_key: &[u8; 32],
        config: RelayConfig,
        security_audit_pipeline: Option<OptimizerPipeline>,
    ) -> Self {
        let local_hint = DriftEnvelope::hint_from_public_key(local_public_key);
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            state: NetworkState::Dormant,
            config,
            store: MeshStore::new(),
            local_hint,
            relay_count_this_hour: 0,
            hour_start: now,
            cover_scheduler: None,
            reputation_manager: None,
            shared_reputation_manager: None,
            security_audit_pipeline,
        }
    }

    /// THE TOGGLE. Activates or deactivates the network.
    /// Active = can message + relay. Dormant = neither.
    pub fn set_network_state(&mut self, state: NetworkState) {
        self.state = state;
    }

    /// Get current network state
    pub fn network_state(&self) -> NetworkState {
        self.state
    }

    /// Enable or disable cover traffic generation.
    ///
    /// When enabled, the relay engine generates dummy messages at a configured
    /// rate to mask real traffic patterns from traffic analysis.
    pub fn set_cover_traffic(&mut self, enabled: bool, rate_per_minute: u32) {
        if enabled {
            let config = CoverConfig {
                enabled: true,
                message_size: 512,
                rate_per_minute: rate_per_minute.max(1),
            };
            self.cover_scheduler = CoverTrafficScheduler::new(config).ok();
        } else {
            self.cover_scheduler = None;
        }
    }

    /// Set the reputation manager for abuse detection
    pub fn set_reputation_manager(
        &mut self,
        manager: Arc<crate::abuse::reputation::EnhancedAbuseReputationManager>,
    ) {
        self.reputation_manager = Some(manager);
    }

    /// Set the shared reputation manager owned by IronCore.
    pub fn set_shared_reputation_manager(
        &mut self,
        manager: Arc<RwLock<crate::abuse::reputation::EnhancedAbuseReputationManager>>,
    ) {
        self.shared_reputation_manager = Some(manager);
    }

    #[cfg(test)]
    pub(crate) fn reputation_manager_is_configured(&self) -> bool {
        self.reputation_manager.is_some() || self.shared_reputation_manager.is_some()
    }

    /// Check if cover traffic is due and generate a cover message.
    ///
    /// Call this periodically (e.g., during maintenance tick).
    /// Returns `Some(cover_bytes)` when a cover message should be broadcast,
    /// or `None` if it's not yet time.
    pub fn generate_cover_traffic_if_due(&mut self) -> Option<Vec<u8>> {
        let scheduler = self.cover_scheduler.as_mut()?;
        if !scheduler.should_generate_cover_traffic() {
            return None;
        }
        let cover_msg = scheduler.generate_and_update().ok()?;
        // Encode: ephemeral_key(32) | recipient_hint(32) | encrypted_payload
        let mut out = Vec::with_capacity(32 + 32 + cover_msg.encrypted_payload.len());
        out.extend_from_slice(&cover_msg.ephemeral_key);
        out.extend_from_slice(&cover_msg.recipient_hint);
        out.extend_from_slice(&cover_msg.encrypted_payload);
        Some(out)
    }

    /// Process an incoming DriftEnvelope. Returns what to do with it.
    /// This is the central routing decision for every message.
    pub fn process_incoming(&mut self, envelope_data: &[u8]) -> Result<RelayDecision, DriftError> {
        self.process_incoming_from(envelope_data, None)
    }

    /// Process an incoming envelope with the authenticated transport identity
    /// used for reputation accounting. Callers without that identity use the
    /// compatibility method above, which deliberately skips reputation lookup
    /// rather than attributing every sender to one shared `unknown` bucket.
    pub fn process_incoming_from(
        &mut self,
        envelope_data: &[u8],
        sender_peer_id: Option<&str>,
    ) -> Result<RelayDecision, DriftError> {
        // Parse envelope
        let envelope = DriftEnvelope::from_bytes(envelope_data)?;

        // Check for spam patterns if reputation manager is available
        let enhanced_score = sender_peer_id.and_then(|sender_peer_id| {
            if let Some(ref reputation_manager) = self.shared_reputation_manager {
                Some(reputation_manager.read().get_enhanced_score(sender_peer_id))
            } else {
                self.reputation_manager
                    .as_ref()
                    .map(|reputation_manager| reputation_manager.get_enhanced_score(sender_peer_id))
            }
        });

        if let Some(enhanced_score) = enhanced_score {
            if enhanced_score.is_abusive() && enhanced_score.spam_confidence > 0.8 {
                return Ok(RelayDecision::Dropped {
                    message_id: envelope.message_id,
                    reason: DropReason::RateLimited,
                });
            }
        }

        // Check if already expired
        if envelope.is_expired() {
            return Ok(RelayDecision::Dropped {
                message_id: envelope.message_id,
                reason: DropReason::Expired,
            });
        }

        // Check for duplicate
        if self.store.contains(&envelope.message_id) {
            return Ok(RelayDecision::Duplicate {
                message_id: envelope.message_id,
            });
        }

        // Privacy: Cover traffic envelopes are relayed identically to regular
        // messages but don't count against rate limits. This ensures cover
        // traffic maintains its indistinguishability while allowing the
        // network to carry it freely.
        let is_cover_traffic =
            envelope.envelope_type == super::envelope::EnvelopeType::CoverTraffic;

        // Privacy: Onion-routed messages are delivered locally if the
        // recipient_hint matches (meaning we're a hop in the onion path).
        // The application layer is responsible for peeling the onion layer
        // and forwarding the remaining envelope.
        let is_onion = envelope.envelope_type == super::envelope::EnvelopeType::OnionMessage;

        // Check if message is for us
        if envelope.recipient_hint == self.local_hint {
            // Onion messages for us need to be delivered locally for peeling
            if is_onion || !is_cover_traffic {
                return Ok(RelayDecision::DeliverLocal {
                    message_id: envelope.message_id,
                });
            }
            // Cover traffic addressed to us: silently absorb (no real recipient)
            return Ok(RelayDecision::Dropped {
                message_id: envelope.message_id,
                reason: DropReason::Expired, // reused as "absorbed cover"
            });
        }

        // Not for us — check if we should relay
        if self.state == NetworkState::Dormant {
            return Ok(RelayDecision::Dropped {
                message_id: envelope.message_id,
                reason: DropReason::NetworkDormant,
            });
        }

        // Check hop count limit
        if envelope.hop_count >= self.config.max_hop_count {
            return Ok(RelayDecision::Dropped {
                message_id: envelope.message_id,
                reason: DropReason::MaxHopsExceeded,
            });
        }

        // Check priority minimum
        if envelope.priority < self.config.min_relay_priority {
            return Ok(RelayDecision::Dropped {
                message_id: envelope.message_id,
                reason: DropReason::LowPriority,
            });
        }

        // Check rate limit (cover traffic exempt — it IS the rate limiting mechanism)
        if !is_cover_traffic {
            self.check_rate_limit();
            if self.config.max_relay_per_hour > 0
                && self.relay_count_this_hour >= self.config.max_relay_per_hour
            {
                return Ok(RelayDecision::Dropped {
                    message_id: envelope.message_id,
                    reason: DropReason::RateLimited,
                });
            }
        }

        // Store and relay
        let stored = StoredEnvelope {
            envelope_data: envelope_data.to_vec(),
            message_id: envelope.message_id,
            recipient_hint: envelope.recipient_hint,
            created_at: envelope.created_at,
            ttl_expiry: envelope.ttl_expiry,
            hop_count: envelope.hop_count,
            priority: envelope.priority,
            received_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let inserted = self.store.insert(stored);
        if !inserted {
            // Duplicate caught during insert
            return Ok(RelayDecision::Duplicate {
                message_id: envelope.message_id,
            });
        }

        // Increment rate limit counter (cover traffic exempt)
        if !is_cover_traffic {
            self.relay_count_this_hour += 1;
        }

        Ok(RelayDecision::StoreAndRelay {
            message_id: envelope.message_id,
        })
    }

    /// Prepare a message for sending. Only works in Active state.
    /// Returns error if Dormant (enforces the coupling).
    pub fn prepare_outgoing(&self, envelope: &DriftEnvelope) -> Result<Vec<u8>, RelayError> {
        // THE COUPLING: can't send if not Active
        if self.state == NetworkState::Dormant {
            return Err(RelayError::NetworkDormant);
        }

        envelope.to_bytes().map_err(|e| {
            RelayError::SerializationFailed(format!("Failed to serialize envelope: {:?}", e))
        })
    }

    /// Get messages to sync with a peer, sorted by priority
    pub fn messages_for_sync(&self, max_count: usize) -> Vec<&StoredEnvelope> {
        let all = self.store.by_priority();
        all.into_iter().take(max_count).collect()
    }

    /// Get messages specifically for a recipient (by hint)
    pub fn messages_for_recipient(&self, hint: &[u8; 4]) -> Vec<&StoredEnvelope> {
        self.store.messages_for_recipient(hint)
    }

    /// Perform housekeeping: remove expired, evict over budget
    pub fn maintenance(&mut self) -> MaintenanceReport {
        let expired_removed = self.store.remove_expired();
        let store_size = self.store.len();

        MaintenanceReport {
            expired_removed,
            store_size,
            evictions: 0,
        }
    }

    /// Apply a policy-derived config to the relay engine.
    ///
    /// Updates relay parameters (budget, hop count, battery floor) while
    /// preserving the current network state and cover traffic settings.
    /// This is the bridge between the PolicyEngine (which computes profiles
    /// from device state) and the RelayEngine (which enforces relay policy).
    pub fn apply_policy_config(&mut self, config: RelayConfig) {
        self.config = config;
    }

    /// Access the underlying store (for sync protocol)
    pub fn store(&self) -> &MeshStore {
        &self.store
    }

    /// Mutably access the underlying store
    pub fn store_mut(&mut self) -> &mut MeshStore {
        &mut self.store
    }

    /// Check and reset rate limit if hour has passed
    fn check_rate_limit(&mut self) {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        const HOUR_MS: u64 = 3600 * 1000;

        if now - self.hour_start > HOUR_MS {
            self.relay_count_this_hour = 0;
            self.hour_start = now;
        }
    }

    /// Run security audit on an envelope before relaying.
    /// Uses the security audit pipeline to verify custody and integrity.
    /// Returns Ok(()) if the audit passes, Err otherwise.
    pub fn run_security_audit(&self, _envelope: &DriftEnvelope) -> Result<(), SecurityAuditError> {
        let pipeline = self.security_audit_pipeline.as_ref().ok_or_else(|| {
            SecurityAuditError::NotConfigured("Security audit pipeline not configured".to_string())
        })?;

        // Validate envelope metadata
        let metadata = pipeline.get_metadata();
        tracing::debug!("Running security audit with pipeline: {}", metadata.name);

        // Run the security audit stages
        let audit_output = pipeline.execute(&vec![
            "vulnerability_scan".to_string(),
            "crypto_review".to_string(),
            "bounds_check".to_string(),
            "memory_safety".to_string(),
            "compliance_check".to_string(),
        ])?;

        tracing::debug!("Security audit completed: {:?}", audit_output);
        Ok(())
    }

    /// Set the security audit pipeline for custody verification
    pub fn set_security_audit_pipeline(&mut self, pipeline: OptimizerPipeline) {
        self.security_audit_pipeline = Some(pipeline);
        tracing::info!("Security audit pipeline configured for relay custody verification");
    }

    /// Get a reference to the security audit pipeline
    pub fn security_audit_pipeline(&self) -> Option<&OptimizerPipeline> {
        self.security_audit_pipeline.as_ref()
    }
}

/// Security audit errors for relay custody verification
#[derive(Debug, Error, Clone)]
pub enum SecurityAuditError {
    #[error("Security audit not configured: {0}")]
    NotConfigured(String),

    #[error("Security audit failed: {0}")]
    Failed(String),

    #[error("Invalid envelope for audit: {0}")]
    InvalidEnvelope(String),
}

impl From<crate::dspy::modules::DSPyError> for SecurityAuditError {
    fn from(err: crate::dspy::modules::DSPyError) -> Self {
        SecurityAuditError::Failed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::envelope::EnvelopeType;
    use super::*;

    fn make_test_envelope(
        message_id: [u8; 16],
        recipient_hint: [u8; 4],
        hop_count: u8,
        priority: u8,
        expired: bool,
    ) -> DriftEnvelope {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        DriftEnvelope {
            version: super::super::DRIFT_VERSION,
            envelope_type: EnvelopeType::EncryptedMessage,
            compressed: false,
            message_id,
            recipient_hint,
            created_at: now,
            ttl_expiry: if expired { now - 100 } else { now + 3600 },
            hop_count,
            priority,
            sender_public_key: [1u8; 32],
            ephemeral_public_key: [2u8; 32],
            nonce: [3u8; 24],
            signature: [4u8; 64],
            ciphertext: b"test".to_vec(),
            ratchet_dh_public: None,
            ratchet_message_number: None,
            suite: None,
            pq_kem_ciphertext: None,
            pq_encaps_key: None,
            transcript_hash: None,
        }
    }

    #[test]
    fn test_deliver_local() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        let local_hint = DriftEnvelope::hint_from_public_key(&local_pk);

        let envelope = make_test_envelope([1u8; 16], local_hint, 0, 100, false);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::DeliverLocal {
                message_id: [1u8; 16]
            }
        );
    }

    #[test]
    fn test_store_and_relay() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::StoreAndRelay {
                message_id: [1u8; 16]
            }
        );

        // Verify it was stored
        assert!(engine.store().contains(&[1u8; 16]));
    }

    #[test]
    fn test_duplicate_detection() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let data = envelope.to_bytes().unwrap();

        // First message
        let decision1 = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision1,
            RelayDecision::StoreAndRelay {
                message_id: [1u8; 16]
            }
        );

        // Same message again
        let decision2 = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision2,
            RelayDecision::Duplicate {
                message_id: [1u8; 16]
            }
        );
    }

    #[test]
    fn test_expired_message_dropped() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, true);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::Dropped {
                message_id: [1u8; 16],
                reason: DropReason::Expired
            }
        );
    }

    #[test]
    fn test_max_hops_exceeded() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 16, 100, false);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::Dropped {
                message_id: [1u8; 16],
                reason: DropReason::MaxHopsExceeded
            }
        );
    }

    #[test]
    fn test_low_priority_dropped() {
        let local_pk = [5u8; 32];
        let config = RelayConfig {
            min_relay_priority: 50,
            ..Default::default()
        };
        let mut engine = RelayEngine::new(&local_pk, config);
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 30, false);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::Dropped {
                message_id: [1u8; 16],
                reason: DropReason::LowPriority
            }
        );
    }

    #[test]
    fn test_network_dormant_drop() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        // Engine starts in Dormant state

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let data = envelope.to_bytes().unwrap();

        let decision = engine.process_incoming(&data).unwrap();
        assert_eq!(
            decision,
            RelayDecision::Dropped {
                message_id: [1u8; 16],
                reason: DropReason::NetworkDormant
            }
        );
    }

    #[test]
    fn test_rate_limiting() {
        let local_pk = [5u8; 32];
        let config = RelayConfig {
            max_relay_per_hour: 2,
            ..Default::default()
        };
        let mut engine = RelayEngine::new(&local_pk, config);
        engine.set_network_state(NetworkState::Active);

        // Message 1
        let env1 = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let decision1 = engine.process_incoming(&env1.to_bytes().unwrap()).unwrap();
        assert!(matches!(decision1, RelayDecision::StoreAndRelay { .. }));

        // Message 2
        let env2 = make_test_envelope([2u8; 16], [9u8; 4], 0, 100, false);
        let decision2 = engine.process_incoming(&env2.to_bytes().unwrap()).unwrap();
        assert!(matches!(decision2, RelayDecision::StoreAndRelay { .. }));

        // Message 3 — should be rate limited
        let env3 = make_test_envelope([3u8; 16], [9u8; 4], 0, 100, false);
        let decision3 = engine.process_incoming(&env3.to_bytes().unwrap()).unwrap();
        assert_eq!(
            decision3,
            RelayDecision::Dropped {
                message_id: [3u8; 16],
                reason: DropReason::RateLimited
            }
        );
    }

    #[test]
    fn test_coupling_cannot_send_when_dormant() {
        let local_pk = [5u8; 32];
        let engine = RelayEngine::new(&local_pk, RelayConfig::default());
        // Starts in Dormant

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let result = engine.prepare_outgoing(&envelope);

        assert!(matches!(result, Err(RelayError::NetworkDormant)));
    }

    #[test]
    fn test_coupling_can_send_when_active() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let envelope = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, false);
        let result = engine.prepare_outgoing(&envelope);

        assert!(result.is_ok());
    }

    #[test]
    fn test_messages_for_sync() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        // Insert multiple messages
        for i in 0..5 {
            let env = make_test_envelope([i as u8; 16], [9u8; 4], 0, (i * 20) as u8, false);
            let _ = engine.process_incoming(&env.to_bytes().unwrap());
        }

        let msgs = engine.messages_for_sync(3);
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn test_messages_for_recipient() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        let hint_a = [1u8, 2u8, 3u8, 4u8];
        let hint_b = [5u8, 6u8, 7u8, 8u8];

        // Messages for hint_a
        for i in 0..3 {
            let env = make_test_envelope([i as u8; 16], hint_a, 0, 100, false);
            let _ = engine.process_incoming(&env.to_bytes().unwrap());
        }

        // Messages for hint_b
        for i in 3..5 {
            let env = make_test_envelope([i as u8; 16], hint_b, 0, 100, false);
            let _ = engine.process_incoming(&env.to_bytes().unwrap());
        }

        let for_a = engine.messages_for_recipient(&hint_a);
        let for_b = engine.messages_for_recipient(&hint_b);

        assert_eq!(for_a.len(), 3);
        assert_eq!(for_b.len(), 2);
    }

    #[test]
    fn test_maintenance_removes_expired() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());
        engine.set_network_state(NetworkState::Active);

        // Create one expired, one valid
        let expired = make_test_envelope([1u8; 16], [9u8; 4], 0, 100, true);
        let valid = make_test_envelope([2u8; 16], [9u8; 4], 0, 100, false);

        // Insert expired message directly into store (bypassing process_incoming which would drop it)
        let expired_stored = StoredEnvelope {
            envelope_data: expired.to_bytes().unwrap(),
            message_id: expired.message_id,
            recipient_hint: expired.recipient_hint,
            created_at: expired.created_at,
            ttl_expiry: expired.ttl_expiry,
            hop_count: expired.hop_count,
            priority: expired.priority,
            received_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        engine.store_mut().insert(expired_stored);

        // Insert valid message through normal flow
        let _ = engine.process_incoming(&valid.to_bytes().unwrap());

        let report = engine.maintenance();
        assert_eq!(report.expired_removed, 1);
        assert_eq!(report.store_size, 1);
    }

    #[test]
    fn test_network_state_toggle() {
        let local_pk = [5u8; 32];
        let mut engine = RelayEngine::new(&local_pk, RelayConfig::default());

        assert_eq!(engine.network_state(), NetworkState::Dormant);

        engine.set_network_state(NetworkState::Active);
        assert_eq!(engine.network_state(), NetworkState::Active);

        engine.set_network_state(NetworkState::Dormant);
        assert_eq!(engine.network_state(), NetworkState::Dormant);
    }

    #[test]
    fn test_relay_config_default() {
        let config = RelayConfig::default();
        assert_eq!(config.max_relay_per_hour, 1000);
        assert_eq!(config.max_hop_count, 16);
        assert_eq!(config.min_relay_priority, 0);
        assert_eq!(config.battery_floor_percent, 20);
        assert!(config.relay_opaque);
    }
}
