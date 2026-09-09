// Relay custody store — durable relay-side store-and-forward state.
//
// This stores messages accepted by a relay on behalf of offline recipients
// and records an auditable transition log for custody lifecycle changes.

use crate::dspy::modules::DSPyModule;
#[cfg(not(target_arch = "wasm32"))]
use crate::store::backend::SledStorage;
use crate::store::backend::{MemoryStorage, StorageBackend};
use serde::{Deserialize, Serialize};
#[cfg(all(not(target_arch = "wasm32"), unix))]
use std::ffi::CString;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

const CUSTODY_MSG_PREFIX: &str = "relay_custody_msg_";
const CUSTODY_AUDIT_PREFIX: &str = "relay_custody_audit_";
const REGISTRATION_STATE_PREFIX: &str = "relay_registration_state_";
const REGISTRATION_AUDIT_PREFIX: &str = "relay_registration_audit_";
const MAX_PENDING_PER_DESTINATION: usize = 10_000;
const DEVICE_USAGE_CEILING_PERCENT: u64 = 90;
const FALLBACK_STORAGE_TOTAL_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const REGISTRATION_STALE_TAKEOVER_MS: u64 = 15 * 24 * 60 * 60 * 1000;
const HANDOVER_STALE_COLLAPSE_MS: u64 = 15 * 24 * 60 * 60 * 1000;

static CUSTODY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePressureBand {
    UpTo20Pct,
    From20To50Pct,
    From50To70Pct,
    From70To80Pct,
    From80To90Pct,
    EmergencyOver90Pct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStorageSnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoragePressureState {
    pub band: StoragePressureBand,
    pub hard_ceiling_bytes: u64,
    pub target_quota_bytes: u64,
    pub scm_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StoragePressureReport {
    pub emergency_mode: bool,
    pub hard_ceiling_bytes: u64,
    pub target_quota_bytes: u64,
    pub scm_bytes_before: u64,
    pub scm_bytes_after: u64,
    pub purged_records: usize,
    pub purged_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationState {
    Active {
        device_id: String,
        seniority_timestamp: u64,
    },
    Handover {
        from_device_id: String,
        to_device_id: String,
        initiated_at: u64,
    },
    Abandoned {
        device_id: String,
        abandoned_at: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RegistrationRecord {
    identity_id: String,
    state: RegistrationState,
    updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationTransition {
    pub identity_id: String,
    pub from_state: Option<RegistrationState>,
    pub to_state: RegistrationState,
    pub reason: String,
    pub at_ms: u64,
    #[serde(default)]
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationStateInfo {
    pub state: String,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustodyError {
    DeviceMismatch,
    AbandonedIdentity,
    NoRegistration,
}

impl CustodyError {
    pub fn as_code(&self) -> &'static str {
        match self {
            CustodyError::DeviceMismatch => "identity_device_mismatch",
            CustodyError::AbandonedIdentity => "identity_abandoned",
            CustodyError::NoRegistration => "identity_registration_missing",
        }
    }
}

impl std::fmt::Display for CustodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_code())
    }
}

/// WS13 compatibility enforcement phase.
///
/// Phase A (current): Legacy requests missing `intended_device_id` are accepted
/// without enforcement. WS13+ clients with both fields are strictly enforced.
/// Phase B (future): WS13+ clients strictly enforced; legacy clients still
/// accepted but log deprecation warnings. Phase C (deferred): all requests
/// require `intended_device_id`; legacy rejected. Transition to Phase B requires
/// >80% client upgrade penetration; Phase C requires >95%.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustodyCompatMode {
    #[default]
    PhaseA,
    PhaseB,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustodyEnforcement {
    Active {
        identity_id: String,
        device_id: String,
    },
    Redirected {
        identity_id: String,
        from_device_id: String,
        to_device_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistrySideEffect {
    None,
    Migrate {
        identity_id: String,
        from_device_id: String,
        to_device_id: String,
    },
    Purge {
        identity_id: String,
        device_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationUpdateOutcome {
    state: RegistrationState,
    side_effect: RegistrySideEffect,
}

#[derive(Clone)]
pub struct RelayRegistry {
    backend: Arc<dyn StorageBackend>,
}

impl StoragePressureState {
    pub fn emergency_mode(self) -> bool {
        self.band == StoragePressureBand::EmergencyOver90Pct
    }
}

trait StoragePressureProbe: Send + Sync {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot>;
}

#[derive(Debug, Default)]
struct NoopStoragePressureProbe;

impl StoragePressureProbe for NoopStoragePressureProbe {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct FilesystemStoragePressureProbe {
    root: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl FilesystemStoragePressureProbe {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StoragePressureProbe for FilesystemStoragePressureProbe {
    fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
        filesystem_usage_bytes(&self.root).map(|(total_bytes, used_bytes)| DeviceStorageSnapshot {
            total_bytes,
            used_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StoragePressureContext {
    total_bytes: u64,
    non_scm_used_bytes: u64,
}

impl StoragePressureContext {
    fn from_snapshot(snapshot: DeviceStorageSnapshot, scm_bytes: u64) -> Option<Self> {
        if snapshot.total_bytes == 0 {
            return None;
        }
        let used_bytes = snapshot.used_bytes.min(snapshot.total_bytes);
        let non_scm_used_bytes = used_bytes.saturating_sub(scm_bytes);
        Some(Self {
            total_bytes: snapshot.total_bytes,
            non_scm_used_bytes,
        })
    }

    fn state_for_scm_bytes(self, scm_bytes: u64) -> StoragePressureState {
        let used_bytes = self.non_scm_used_bytes.saturating_add(scm_bytes);
        let used_percent_basis_points =
            ((used_bytes as u128 * 10_000u128) / self.total_bytes as u128) as u64;
        let free_bytes = self.total_bytes.saturating_sub(used_bytes);
        let ninety_percent_total =
            ((self.total_bytes as u128 * DEVICE_USAGE_CEILING_PERCENT as u128) / 100u128) as u64;
        let hard_ceiling_bytes = ninety_percent_total.saturating_sub(self.non_scm_used_bytes);

        let (band, band_percent) = if used_percent_basis_points <= 2_000 {
            (StoragePressureBand::UpTo20Pct, 70u64)
        } else if used_percent_basis_points <= 5_000 {
            (StoragePressureBand::From20To50Pct, 45u64)
        } else if used_percent_basis_points <= 7_000 {
            (StoragePressureBand::From50To70Pct, 25u64)
        } else if used_percent_basis_points <= 8_000 {
            (StoragePressureBand::From70To80Pct, 10u64)
        } else if used_percent_basis_points <= 9_000 {
            (StoragePressureBand::From80To90Pct, 3u64)
        } else {
            (StoragePressureBand::EmergencyOver90Pct, 0u64)
        };

        let dynamic_target = if band == StoragePressureBand::EmergencyOver90Pct {
            hard_ceiling_bytes
        } else {
            ((free_bytes as u128 * band_percent as u128) / 100u128) as u64
        };
        let target_quota_bytes = dynamic_target.min(hard_ceiling_bytes);

        StoragePressureState {
            band,
            hard_ceiling_bytes,
            target_quota_bytes,
            scm_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyState {
    Accepted,
    Dispatching,
    Delivered,
}

impl CustodyState {
    fn as_str(self) -> &'static str {
        match self {
            CustodyState::Accepted => "accepted",
            CustodyState::Dispatching => "dispatching",
            CustodyState::Delivered => "delivered",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyMessage {
    pub custody_id: String,
    pub relay_message_id: String,
    pub source_peer_id: String,
    pub destination_peer_id: String,
    #[serde(default)]
    pub recipient_identity_id: Option<String>,
    #[serde(default)]
    pub intended_device_id: Option<String>,
    pub envelope_data: Vec<u8>,
    pub state: CustodyState,
    pub accepted_at_ms: u64,
    pub updated_at_ms: u64,
    pub delivery_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransition {
    pub custody_id: String,
    pub relay_message_id: String,
    pub source_peer_id: String,
    pub destination_peer_id: String,
    pub from_state: Option<CustodyState>,
    pub to_state: CustodyState,
    pub reason: String,
    pub at_ms: u64,
    #[serde(default)]
    pub sequence: u64,
}

#[derive(Clone)]
pub struct RelayCustodyStore {
    backend: Arc<dyn StorageBackend>,
    registry: RelayRegistry,
    local_identity: Option<String>,
    pressure_probe: Arc<dyn StoragePressureProbe>,
    /// Security audit pipeline for cryptographic and protocol verification.
    pub(crate) security_audit_pipeline: Arc<crate::dspy::modules::OptimizerPipeline>,
    /// WS13 compatibility enforcement phase. Controls how legacy requests
    /// (missing `intended_device_id`) are handled at the store level.
    compat_mode: CustodyCompatMode,
    /// WS13+ adoption telemetry for Phase B transition decisions.
    ws13_requests: Arc<std::sync::atomic::AtomicU64>,
    legacy_requests: Arc<std::sync::atomic::AtomicU64>,
}

impl RelayCustodyStore {
    pub fn in_memory() -> Self {
        let backend = Arc::new(MemoryStorage::new());
        Self::new_with_backends(
            backend.clone(),
            backend,
            None,
            Arc::new(NoopStoragePressureProbe),
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
        )
    }

    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self::new_with_backends(
            backend.clone(),
            backend,
            None,
            Arc::new(NoopStoragePressureProbe),
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
        )
    }

    fn new_with_backends(
        custody_backend: Arc<dyn StorageBackend>,
        registry_backend: Arc<dyn StorageBackend>,
        local_identity: Option<String>,
        pressure_probe: Arc<dyn StoragePressureProbe>,
        security_audit_pipeline: Arc<crate::dspy::modules::OptimizerPipeline>,
    ) -> Self {
        Self {
            backend: custody_backend,
            registry: RelayRegistry::new(registry_backend),
            local_identity,
            pressure_probe,
            security_audit_pipeline,
            compat_mode: CustodyCompatMode::default(),
            ws13_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            legacy_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    #[cfg(test)]
    fn in_memory_with_probe(
        local_identity: Option<String>,
        pressure_probe: Arc<dyn StoragePressureProbe>,
    ) -> Self {
        let backend = Arc::new(MemoryStorage::new());
        Self::new_with_backends(
            backend.clone(),
            backend,
            local_identity,
            pressure_probe,
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_local_peer(local_peer_id: &str) -> Self {
        let base = custody_base_dir();
        let dir = base.join(local_peer_id);
        let _ = std::fs::create_dir_all(&dir);

        let path = dir.to_string_lossy().to_string();
        match SledStorage::new(&path) {
            Ok(backend) => {
                let backend = Arc::new(backend);
                Self::new_with_backends(
                    backend.clone(),
                    backend,
                    Some(local_peer_id.to_string()),
                    Arc::new(FilesystemStoragePressureProbe::new(dir)),
                    Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
                )
            }
            Err(_) => {
                let backend = Arc::new(MemoryStorage::new());
                Self::new_with_backends(
                    backend.clone(),
                    backend,
                    Some(local_peer_id.to_string()),
                    Arc::new(NoopStoragePressureProbe),
                    Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
                )
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_service_storage(storage_path: Option<&str>, local_peer_id: &str) -> Self {
        let Some(storage_path) = storage_path else {
            return Self::for_local_peer(local_peer_id);
        };

        let storage_root = PathBuf::from(storage_path);
        let custody_dir = storage_root.join("relay_custody");
        let registry_dir = storage_root.join("root");
        let _ = std::fs::create_dir_all(&custody_dir);
        let _ = std::fs::create_dir_all(&registry_dir);

        let custody_path = custody_dir.to_string_lossy().to_string();
        let registry_path = registry_dir.to_string_lossy().to_string();
        let custody_backend = SledStorage::new(&custody_path)
            .map(|backend| Arc::new(backend) as Arc<dyn StorageBackend>);
        let registry_backend = SledStorage::new(&registry_path)
            .map(|backend| Arc::new(backend) as Arc<dyn StorageBackend>);

        match (custody_backend, registry_backend) {
            (Ok(custody_backend), Ok(registry_backend)) => Self::new_with_backends(
                custody_backend,
                registry_backend,
                Some(local_peer_id.to_string()),
                Arc::new(FilesystemStoragePressureProbe::new(custody_dir)),
                Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
            ),
            _ => {
                let backend = Arc::new(MemoryStorage::new());
                Self::new_with_backends(
                    backend.clone(),
                    backend,
                    Some(local_peer_id.to_string()),
                    Arc::new(NoopStoragePressureProbe),
                    Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()),
                )
            }
        }
    }

    pub fn registry(&self) -> &RelayRegistry {
        &self.registry
    }

    pub fn compat_mode(&self) -> CustodyCompatMode {
        self.compat_mode
    }

    pub fn set_compat_mode(&mut self, mode: CustodyCompatMode) {
        self.compat_mode = mode;
    }

    pub fn adoption_stats(&self) -> (u64, u64) {
        (
            self.ws13_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            self.legacy_requests
                .load(std::sync::atomic::Ordering::Relaxed),
        )
    }

    pub fn adoption_pct(&self) -> f64 {
        let (ws13, legacy) = self.adoption_stats();
        let total = ws13 + legacy;
        if total == 0 {
            return 0.0;
        }
        (ws13 as f64 / total as f64) * 100.0
    }

    pub fn register_identity(
        &self,
        identity_id: String,
        device_id: String,
        seniority_timestamp: u64,
    ) -> Result<RegistrationState, String> {
        let outcome = self
            .registry
            .register(identity_id, device_id, seniority_timestamp)?;
        self.apply_registry_side_effect(&outcome.side_effect)?;
        Ok(outcome.state)
    }

    pub fn deregister_identity(
        &self,
        identity_id: String,
        from_device_id: String,
        target_device_id: Option<String>,
    ) -> Result<RegistrationState, String> {
        let outcome = self
            .registry
            .deregister(identity_id, from_device_id, target_device_id)?;
        self.apply_registry_side_effect(&outcome.side_effect)?;
        Ok(outcome.state)
    }

    pub fn get_registration_state(
        &self,
        identity_id: &str,
    ) -> Result<Option<RegistrationState>, String> {
        self.registry.get_state(identity_id)
    }

    pub fn get_registration_state_info(&self, identity_id: &str) -> RegistrationStateInfo {
        self.registry.get_state_info(identity_id)
    }

    pub fn enforce_custody(
        &self,
        identity_id: &str,
        device_id: &str,
    ) -> Result<CustodyEnforcement, CustodyError> {
        self.registry.enforce_custody(identity_id, device_id)
    }

    pub fn registration_transitions_for_identity(
        &self,
        identity_id: &str,
    ) -> Vec<RegistrationTransition> {
        self.registry.transitions_for_identity(identity_id)
    }

    fn apply_registry_side_effect(&self, side_effect: &RegistrySideEffect) -> Result<(), String> {
        match side_effect {
            RegistrySideEffect::None => Ok(()),
            RegistrySideEffect::Migrate {
                identity_id,
                from_device_id,
                to_device_id,
            } => {
                let _ = self.migrate_pending_identity_device(
                    identity_id,
                    from_device_id,
                    to_device_id,
                    "identity_handover_queue_migrated",
                )?;
                Ok(())
            }
            RegistrySideEffect::Purge {
                identity_id,
                device_id,
            } => {
                let _ = self.purge_pending_identity_messages(
                    identity_id,
                    device_id.as_deref(),
                    "identity_queue_purged",
                )?;
                Ok(())
            }
        }
    }

    pub fn accept_custody(
        &self,
        source_peer_id: String,
        destination_peer_id: String,
        relay_message_id: String,
        envelope_data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Result<CustodyMessage, String> {
        if let Some(existing) = self.find_existing(&destination_peer_id, &relay_message_id)? {
            return Ok(existing);
        }

        let normalized_recipient_identity_id =
            recipient_identity_id.as_deref().and_then(|identity_id| {
                self.registry
                    .normalize_lookup_identity(identity_id)
                    .ok()
                    .flatten()
            });
        let normalized_intended_device_id = intended_device_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        // WS13 compat mode enforcement at the store level.
        // When both identity_id and device_id are present, enforce custody strictly.
        // When device_id is absent, apply compat mode policy.
        let mut legacy_reason = None;
        if let Some(ref identity_id) = normalized_recipient_identity_id {
            if let Some(ref device_id) = normalized_intended_device_id {
                // WS13+ client: strictly enforce custody
                match self.enforce_custody(identity_id, device_id) {
                    Ok(CustodyEnforcement::Active { .. }) => {}
                    Ok(CustodyEnforcement::Redirected { to_device_id, .. }) => {
                        // Redirect accepted: update intended_device_id to the handover target
                        // The redirect is already handled at the transport layer by
                        // resolve_custody_metadata; here we just allow it through.
                        let _ = to_device_id;
                    }
                    Err(error) => {
                        return Err(error.to_string());
                    }
                }
                self.ws13_requests
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                // Legacy client: no device_id provided
                match self.compat_mode {
                    CustodyCompatMode::PhaseA => {
                        tracing::debug!(
                            identity_id,
                            "relay custody accepted in Phase A compat mode (no device enforcement)"
                        );
                        legacy_reason = Some("custody_accepted_compat_phase_a");
                    }
                    CustodyCompatMode::PhaseB => {
                        tracing::warn!(
                            identity_id,
                            "relay custody accepted in Phase B compat mode (legacy client, deprecation warning)"
                        );
                        legacy_reason = Some("custody_accepted_compat_phase_b");
                    }
                }
                self.legacy_requests
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let pending_count = self
            .pending_for_destination(&destination_peer_id, usize::MAX)
            .len();
        if pending_count >= MAX_PENDING_PER_DESTINATION {
            return Err(format!(
                "custody queue full for destination {}",
                destination_peer_id
            ));
        }

        let now_ms = now_ms();
        let sequence = CUSTODY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let custody_id = format!("{}-{}-{}", relay_message_id, now_ms, sequence);

        // Wire into security audit pipeline for relay custody verification.
        // Performs a vulnerability scan and crypto review of the incoming envelope.
        let _ = self.security_audit_pipeline.execute(&vec![
            format!("relay_custody_verify: {}", custody_id),
            format!("source: {}", source_peer_id),
            format!("destination: {}", destination_peer_id),
        ]);

        let message = CustodyMessage {
            custody_id,
            relay_message_id,
            source_peer_id,
            destination_peer_id,
            recipient_identity_id: normalized_recipient_identity_id,
            intended_device_id: normalized_intended_device_id,
            envelope_data,
            state: CustodyState::Accepted,
            accepted_at_ms: now_ms,
            updated_at_ms: now_ms,
            delivery_attempts: 0,
        };

        self.enforce_storage_pressure_for_write(&message)?;
        self.put_message(&message)?;
        let reason = legacy_reason.unwrap_or("custody_accepted");
        self.record_transition(&message, None, CustodyState::Accepted, reason)?;
        Ok(message)
    }

    pub fn storage_pressure_state(&self) -> Option<StoragePressureState> {
        let scm_bytes = self.current_scm_storage_bytes().ok()?;
        let snapshot = self
            .pressure_probe
            .snapshot()
            .or_else(|| synthetic_storage_snapshot(scm_bytes))?;
        let context = StoragePressureContext::from_snapshot(snapshot, scm_bytes)?;
        Some(context.state_for_scm_bytes(scm_bytes))
    }

    pub fn enforce_storage_pressure(&self) -> Result<StoragePressureReport, String> {
        self.enforce_storage_pressure_internal(None)
    }

    pub fn pending_for_destination(
        &self,
        destination_peer_id: &str,
        limit: usize,
    ) -> Vec<CustodyMessage> {
        let prefix = destination_prefix(destination_peer_id);
        let mut records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(prefix.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.state == CustodyState::Accepted)
            .collect();
        records.sort_by_key(|record| (record.accepted_at_ms, record.custody_id.clone()));
        records.into_iter().take(limit).collect()
    }

    pub fn mark_dispatching(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Dispatching {
            return Ok(());
        }
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        // Max delivery attempts guard — prevent infinite cycling
        const MAX_DELIVERY_ATTEMPTS: u32 = 12;
        if record.delivery_attempts >= MAX_DELIVERY_ATTEMPTS {
            return Err(format!(
                "Max delivery attempts ({}) exceeded for custody {}",
                MAX_DELIVERY_ATTEMPTS, custody_id
            ));
        }
        let from_state = record.state;
        record.state = CustodyState::Dispatching;
        record.updated_at_ms = now_ms();
        record.delivery_attempts = record.delivery_attempts.saturating_add(1);
        self.put_message(&record)?;
        self.record_transition(&record, Some(from_state), CustodyState::Dispatching, reason)?;
        Ok(())
    }

    /// Re-arm dispatch attempts for every pending custody record addressed to
    /// `destination_peer_id` (delivery_attempts -> 0).
    ///
    /// Rationale (D1, live 2026-09-09): dispatch attempts consumed at the
    /// periodic-pull cadence (~15s) exhaust the 12-attempt guard in ~5
    /// minutes. Without a re-arm, a custody entry whose destination is merely
    /// RESTARTING an app (or briefly dropping cell) gets wedged forever -
    /// "Max delivery attempts (12) exceeded" refused on every later periodic
    /// pull while the destination sat connected. Store-and-forward must
    /// outlive short destination absences, so a FRESH connection episode
    /// re-arms the counter; the per-episode cap still prevents infinite
    /// retry churn against a connected-but-not-accepting destination.
    pub fn reset_delivery_attempts_for_destination(&self, destination_peer_id: &str) -> usize {
        let prefix = destination_prefix(destination_peer_id);
        let records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(prefix.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| {
                record.state != CustodyState::Delivered && record.delivery_attempts > 0
            })
            .collect();

        let mut reset = 0usize;
        for mut record in records {
            record.delivery_attempts = 0;
            if self.put_message(&record).is_ok() {
                reset += 1;
            }
        }
        if reset > 0 {
            tracing::info!(
                "[CUSTODY-REARM] reset delivery_attempts for {} pending custody record(s) to {}",
                reset,
                destination_peer_id
            );
        }
        reset
    }

    pub fn mark_dispatch_failed(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Accepted {
            return Ok(());
        }
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        let from_state = record.state;
        record.state = CustodyState::Accepted;
        record.updated_at_ms = now_ms();
        self.put_message(&record)?;
        self.record_transition(&record, Some(from_state), CustodyState::Accepted, reason)?;
        Ok(())
    }

    pub fn mark_delivered(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
        reason: &str,
    ) -> Result<(), String> {
        let mut record = self.require_record(destination_peer_id, custody_id)?;
        if record.state == CustodyState::Delivered {
            return Ok(());
        }
        let from_state = record.state;
        record.state = CustodyState::Delivered;
        record.updated_at_ms = now_ms();
        self.record_transition(&record, Some(from_state), CustodyState::Delivered, reason)?;
        self.remove_message(destination_peer_id, custody_id)?;
        Ok(())
    }

    /// Mark all duplicate custody records for a relay message as delivered and
    /// remove them from the pending queue.
    pub fn converge_delivered_for_message(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
        reason: &str,
    ) -> Result<usize, String> {
        let prefix = destination_prefix(destination_peer_id);
        let records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(prefix.as_bytes())?
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.relay_message_id == relay_message_id)
            .collect();

        if records.is_empty() {
            return Ok(0);
        }

        let mut converged = 0usize;
        for mut record in records {
            if record.state == CustodyState::Delivered {
                continue;
            }
            let from_state = record.state;
            record.state = CustodyState::Delivered;
            record.updated_at_ms = now_ms();
            self.record_transition(&record, Some(from_state), CustodyState::Delivered, reason)?;
            self.remove_message(destination_peer_id, &record.custody_id)?;
            converged += 1;
        }

        Ok(converged)
    }

    pub fn transitions_for_custody(&self, custody_id: &str) -> Vec<CustodyTransition> {
        let mut transitions: Vec<CustodyTransition> = self
            .backend
            .scan_prefix(CUSTODY_AUDIT_PREFIX.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyTransition>(&value).ok())
            .filter(|transition| transition.custody_id == custody_id)
            .collect();
        transitions.sort_by_key(|transition| (transition.at_ms, transition.sequence));
        transitions
    }

    pub fn audit_count(&self) -> usize {
        self.backend
            .count_prefix(CUSTODY_AUDIT_PREFIX.as_bytes())
            .unwrap_or(0)
    }

    fn enforce_storage_pressure_for_write(&self, incoming: &CustodyMessage) -> Result<(), String> {
        let report = self.enforce_storage_pressure_internal(Some(incoming))?;
        if let Some(snapshot) = self.pressure_probe.snapshot() {
            let write_bytes = serialized_record_bytes(incoming)?;
            let mut scm_bytes = report.scm_bytes_after;
            let context = StoragePressureContext::from_snapshot(snapshot, scm_bytes)
                .ok_or_else(|| "invalid_storage_snapshot".to_string())?;
            let mut state = context.state_for_scm_bytes(scm_bytes);

            if state.emergency_mode() && !self.is_identity_related_record(incoming) {
                return Err("emergency_mode_non_critical_rejected".to_string());
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.target_quota_bytes {
                let need = projected.saturating_sub(state.target_quota_bytes);
                let (_, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_target_quota")?;
                scm_bytes = scm_bytes.saturating_sub(purged_bytes);
                state = context.state_for_scm_bytes(scm_bytes);
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.hard_ceiling_bytes {
                let need = projected.saturating_sub(state.hard_ceiling_bytes);
                let (_, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_hard_ceiling")?;
                scm_bytes = scm_bytes.saturating_sub(purged_bytes);
                state = context.state_for_scm_bytes(scm_bytes);
            }

            let projected = scm_bytes.saturating_add(write_bytes);
            if projected > state.target_quota_bytes || projected > state.hard_ceiling_bytes {
                return Err(format!(
                    "storage_pressure_capacity_exceeded: projected={} target={} hard_ceiling={}",
                    projected, state.target_quota_bytes, state.hard_ceiling_bytes
                ));
            }
        }
        Ok(())
    }

    fn enforce_storage_pressure_internal(
        &self,
        incoming: Option<&CustodyMessage>,
    ) -> Result<StoragePressureReport, String> {
        let scm_before = self.current_scm_storage_bytes()?;
        let snapshot = self
            .pressure_probe
            .snapshot()
            .or_else(|| synthetic_storage_snapshot(scm_before))
            .ok_or_else(|| "invalid_storage_snapshot".to_string())?;

        let context = StoragePressureContext::from_snapshot(snapshot, scm_before)
            .ok_or_else(|| "invalid_storage_snapshot".to_string())?;
        let state = context.state_for_scm_bytes(scm_before);
        let mut report = StoragePressureReport {
            emergency_mode: state.emergency_mode(),
            hard_ceiling_bytes: state.hard_ceiling_bytes,
            target_quota_bytes: state.target_quota_bytes,
            scm_bytes_before: scm_before,
            scm_bytes_after: scm_before,
            ..StoragePressureReport::default()
        };

        if state.emergency_mode()
            && incoming
                .map(|record| !self.is_identity_related_record(record))
                .unwrap_or(false)
        {
            let need = report
                .scm_bytes_after
                .saturating_sub(state.hard_ceiling_bytes);
            if need > 0 {
                let (purged_records, purged_bytes) =
                    self.purge_oldest_by_policy(need, "storage_pressure_emergency")?;
                report.purged_records += purged_records;
                report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
                report.scm_bytes_after = report.scm_bytes_after.saturating_sub(purged_bytes);
            }
            return Ok(report);
        }

        let mut scm_bytes = report.scm_bytes_after;
        let mut current_state = context.state_for_scm_bytes(scm_bytes);
        let mut required = scm_bytes.saturating_sub(current_state.target_quota_bytes);
        if required > 0 {
            let (purged_records, purged_bytes) =
                self.purge_oldest_by_policy(required, "storage_pressure_target_quota")?;
            scm_bytes = scm_bytes.saturating_sub(purged_bytes);
            report.purged_records += purged_records;
            report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
            current_state = context.state_for_scm_bytes(scm_bytes);
            required = scm_bytes.saturating_sub(current_state.hard_ceiling_bytes);
        } else {
            required = scm_bytes.saturating_sub(current_state.hard_ceiling_bytes);
        }

        if required > 0 {
            let (purged_records, purged_bytes) =
                self.purge_oldest_by_policy(required, "storage_pressure_hard_ceiling")?;
            scm_bytes = scm_bytes.saturating_sub(purged_bytes);
            report.purged_records += purged_records;
            report.purged_bytes = report.purged_bytes.saturating_add(purged_bytes);
            current_state = context.state_for_scm_bytes(scm_bytes);
        }

        report.scm_bytes_after = scm_bytes;
        report.target_quota_bytes = current_state.target_quota_bytes;
        report.hard_ceiling_bytes = current_state.hard_ceiling_bytes;
        report.emergency_mode = current_state.emergency_mode();
        Ok(report)
    }

    fn current_scm_storage_bytes(&self) -> Result<u64, String> {
        let records = self.backend.scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?;
        Ok(records
            .into_iter()
            .map(|(_, value)| value.len() as u64)
            .sum::<u64>())
    }

    fn load_stored_records(&self) -> Result<Vec<StoredCustodyRecord>, String> {
        let records = self.backend.scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?;
        let mut parsed = Vec::with_capacity(records.len());
        for (_, value) in records {
            if let Ok(record) = bincode::deserialize::<CustodyMessage>(&value) {
                parsed.push(StoredCustodyRecord {
                    record,
                    serialized_bytes: value.len() as u64,
                });
            }
        }
        Ok(parsed)
    }

    fn purge_oldest_by_policy(
        &self,
        mut required_bytes: u64,
        reason: &str,
    ) -> Result<(usize, u64), String> {
        if required_bytes == 0 {
            return Ok((0, 0));
        }
        let mut candidates = self.load_stored_records()?;
        candidates.sort_by_key(|candidate| {
            (
                candidate.identity_related(self),
                candidate.delivery_priority(),
                candidate.record.accepted_at_ms,
                candidate.record.custody_id.clone(),
            )
        });

        let mut purged_records = 0usize;
        let mut purged_bytes = 0u64;
        for candidate in candidates {
            if required_bytes == 0 {
                break;
            }
            let purge_reason = format!("{}_purged", reason);
            self.record_transition(
                &candidate.record,
                Some(candidate.record.state),
                candidate.record.state,
                &purge_reason,
            )?;
            self.remove_message(
                &candidate.record.destination_peer_id,
                &candidate.record.custody_id,
            )?;
            purged_records += 1;
            purged_bytes = purged_bytes.saturating_add(candidate.serialized_bytes);
            required_bytes = required_bytes.saturating_sub(candidate.serialized_bytes);
            tracing::warn!(
                "purged custody {} due to {} ({} bytes)",
                candidate.record.custody_id,
                reason,
                candidate.serialized_bytes
            );
        }
        Ok((purged_records, purged_bytes))
    }

    fn is_identity_related_record(&self, record: &CustodyMessage) -> bool {
        self.is_identity_related_ids(&record.source_peer_id, &record.destination_peer_id)
    }

    fn is_identity_related_ids(&self, source_peer_id: &str, destination_peer_id: &str) -> bool {
        let Some(local_identity) = self.local_identity.as_deref() else {
            return false;
        };
        source_peer_id == local_identity || destination_peer_id == local_identity
    }

    fn find_existing(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
    ) -> Result<Option<CustodyMessage>, String> {
        let prefix = destination_prefix(destination_peer_id);
        for (_, value) in self.backend.scan_prefix(prefix.as_bytes())? {
            if let Ok(record) = bincode::deserialize::<CustodyMessage>(&value) {
                if record.relay_message_id == relay_message_id {
                    return Ok(Some(record));
                }
            }
        }
        Ok(None)
    }

    pub fn has_message_for_destination(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
    ) -> bool {
        self.find_existing(destination_peer_id, relay_message_id)
            .ok()
            .flatten()
            .is_some()
    }

    fn require_record(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
    ) -> Result<CustodyMessage, String> {
        self.get_message(destination_peer_id, custody_id)?
            .ok_or_else(|| format!("custody record not found: {}", custody_id))
    }

    fn get_message(
        &self,
        destination_peer_id: &str,
        custody_id: &str,
    ) -> Result<Option<CustodyMessage>, String> {
        let key = message_key(destination_peer_id, custody_id);
        if let Some(bytes) = self.backend.get(key.as_bytes())? {
            let record = bincode::deserialize::<CustodyMessage>(&bytes)
                .map_err(|e| format!("deserialize custody record failed: {}", e))?;
            return Ok(Some(record));
        }
        Ok(None)
    }

    fn put_message(&self, record: &CustodyMessage) -> Result<(), String> {
        let key = message_key(&record.destination_peer_id, &record.custody_id);
        let bytes = bincode::serialize(record)
            .map_err(|e| format!("serialize custody record failed: {}", e))?;
        self.backend.put(key.as_bytes(), &bytes)?;
        self.backend.flush()?;
        Ok(())
    }

    fn remove_message(&self, destination_peer_id: &str, custody_id: &str) -> Result<(), String> {
        let key = message_key(destination_peer_id, custody_id);
        self.backend.remove(key.as_bytes())?;
        self.backend.flush()?;
        Ok(())
    }

    fn migrate_pending_identity_device(
        &self,
        identity_id: &str,
        from_device_id: &str,
        to_device_id: &str,
        reason: &str,
    ) -> Result<usize, String> {
        let Some(identity_id) = self.registry.normalize_lookup_identity(identity_id)? else {
            return Ok(0);
        };

        let records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.state != CustodyState::Delivered)
            .filter(|record| record.recipient_identity_id.as_deref() == Some(identity_id.as_str()))
            .filter(|record| record.intended_device_id.as_deref() == Some(from_device_id))
            .collect();

        let mut migrated = 0usize;
        for mut record in records {
            record.intended_device_id = Some(to_device_id.to_string());
            record.updated_at_ms = now_ms();
            self.put_message(&record)?;
            self.record_transition(&record, Some(record.state), record.state, reason)?;
            migrated += 1;
        }
        Ok(migrated)
    }

    fn purge_pending_identity_messages(
        &self,
        identity_id: &str,
        device_id: Option<&str>,
        reason: &str,
    ) -> Result<usize, String> {
        let Some(identity_id) = self.registry.normalize_lookup_identity(identity_id)? else {
            return Ok(0);
        };

        let records: Vec<CustodyMessage> = self
            .backend
            .scan_prefix(CUSTODY_MSG_PREFIX.as_bytes())?
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<CustodyMessage>(&value).ok())
            .filter(|record| record.state != CustodyState::Delivered)
            .filter(|record| record.recipient_identity_id.as_deref() == Some(identity_id.as_str()))
            .filter(|record| {
                device_id
                    .map(|expected| record.intended_device_id.as_deref() == Some(expected))
                    .unwrap_or(true)
            })
            .collect();

        let mut purged = 0usize;
        for record in records {
            self.record_transition(&record, Some(record.state), record.state, reason)?;
            self.remove_message(&record.destination_peer_id, &record.custody_id)?;
            purged += 1;
        }
        Ok(purged)
    }

    fn record_transition(
        &self,
        record: &CustodyMessage,
        from_state: Option<CustodyState>,
        to_state: CustodyState,
        reason: &str,
    ) -> Result<(), String> {
        let at_ms = now_ms();
        let sequence = CUSTODY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let key = format!(
            "{}{:020}_{:06}_{}_{}",
            CUSTODY_AUDIT_PREFIX,
            at_ms,
            sequence,
            record.custody_id,
            to_state.as_str()
        );
        let transition = CustodyTransition {
            custody_id: record.custody_id.clone(),
            relay_message_id: record.relay_message_id.clone(),
            source_peer_id: record.source_peer_id.clone(),
            destination_peer_id: record.destination_peer_id.clone(),
            from_state,
            to_state,
            reason: reason.to_string(),
            at_ms,
            sequence,
        };
        let bytes = bincode::serialize(&transition)
            .map_err(|e| format!("serialize custody transition failed: {}", e))?;
        self.backend.put(key.as_bytes(), &bytes)?;
        self.backend.flush()?;
        Ok(())
    }
}

impl Default for RelayCustodyStore {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl RelayRegistry {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Return the count of registered identities in the registry.
    pub fn len(&self) -> usize {
        self.backend
            .count_prefix(REGISTRATION_STATE_PREFIX.as_bytes())
            .unwrap_or(0)
    }

    /// Return whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn register(
        &self,
        identity_id: String,
        device_id: String,
        seniority_timestamp: u64,
    ) -> Result<RegistrationUpdateOutcome, String> {
        let identity_id = normalize_identity_id(&identity_id)?;
        let device_id = normalize_uuid_v4(&device_id, "registration_device_id_invalid")?;
        if seniority_timestamp == 0 {
            return Err("registration_seniority_invalid".to_string());
        }

        let now = now_ms();
        let current = self.get_record_by_identity_id(&identity_id)?;
        let previous_state = current.as_ref().map(|record| record.state.clone());
        let current_updated_at_ms = current.as_ref().map(|record| record.updated_at_ms);
        let (state, side_effect, reason) = match previous_state.as_ref() {
            None => (
                RegistrationState::Active {
                    device_id: device_id.clone(),
                    seniority_timestamp,
                },
                RegistrySideEffect::None,
                "registration_activated",
            ),
            Some(RegistrationState::Active {
                device_id: active_device_id,
                ..
            }) if active_device_id == &device_id => (
                RegistrationState::Active {
                    device_id: device_id.clone(),
                    seniority_timestamp,
                },
                RegistrySideEffect::None,
                "registration_refreshed",
            ),
            Some(RegistrationState::Active {
                device_id: active_device_id,
                ..
            }) => {
                let Some(updated_at_ms) = current_updated_at_ms else {
                    return Err("registration_state_missing".to_string());
                };
                if now.saturating_sub(updated_at_ms) > REGISTRATION_STALE_TAKEOVER_MS {
                    (
                        RegistrationState::Active {
                            device_id: device_id.clone(),
                            seniority_timestamp,
                        },
                        RegistrySideEffect::Purge {
                            identity_id: identity_id.clone(),
                            device_id: Some(active_device_id.clone()),
                        },
                        "registration_stale_takeover",
                    )
                } else {
                    return Err("registration_active_conflict".to_string());
                }
            }
            Some(RegistrationState::Handover { to_device_id, .. })
                if to_device_id == &device_id =>
            {
                (
                    RegistrationState::Active {
                        device_id: device_id.clone(),
                        seniority_timestamp,
                    },
                    RegistrySideEffect::None,
                    "registration_handover_completed",
                )
            }
            Some(RegistrationState::Handover { .. }) => {
                return Err("registration_handover_conflict".to_string());
            }
            Some(RegistrationState::Abandoned { .. }) => (
                RegistrationState::Active {
                    device_id: device_id.clone(),
                    seniority_timestamp,
                },
                RegistrySideEffect::None,
                "registration_reactivated",
            ),
        };

        self.persist_state(identity_id, previous_state, state.clone(), reason, now)?;
        Ok(RegistrationUpdateOutcome { state, side_effect })
    }

    fn deregister(
        &self,
        identity_id: String,
        from_device_id: String,
        target_device_id: Option<String>,
    ) -> Result<RegistrationUpdateOutcome, String> {
        let identity_id = normalize_identity_id(&identity_id)?;
        let from_device_id =
            normalize_uuid_v4(&from_device_id, "deregistration_from_device_id_invalid")?;
        let target_device_id = match target_device_id {
            Some(target_device_id) => Some(normalize_uuid_v4(
                &target_device_id,
                "deregistration_target_device_id_invalid",
            )?),
            None => None,
        };

        if target_device_id.as_deref() == Some(from_device_id.as_str()) {
            return Err("deregistration_target_matches_source".to_string());
        }

        let now = now_ms();
        let current = self
            .get_record_by_identity_id(&identity_id)?
            .ok_or_else(|| "registration_not_found".to_string())?;
        let previous_state = current.state.clone();

        let (state, side_effect, reason) = match &previous_state {
            RegistrationState::Active { device_id, .. } if device_id == &from_device_id => {
                if let Some(target_device_id) = target_device_id.clone() {
                    (
                        RegistrationState::Handover {
                            from_device_id: from_device_id.clone(),
                            to_device_id: target_device_id.clone(),
                            initiated_at: now,
                        },
                        RegistrySideEffect::Migrate {
                            identity_id: identity_id.clone(),
                            from_device_id: from_device_id.clone(),
                            to_device_id: target_device_id,
                        },
                        "registration_handover_started",
                    )
                } else {
                    (
                        RegistrationState::Abandoned {
                            device_id: from_device_id.clone(),
                            abandoned_at: now,
                        },
                        RegistrySideEffect::Purge {
                            identity_id: identity_id.clone(),
                            device_id: None,
                        },
                        "registration_abandoned",
                    )
                }
            }
            RegistrationState::Active { .. } => {
                return Err("registration_device_mismatch".to_string());
            }
            RegistrationState::Handover {
                from_device_id: current_from,
                to_device_id: current_to,
                initiated_at,
            } if current_from == &from_device_id
                && target_device_id.as_deref() == Some(current_to.as_str()) =>
            {
                (
                    RegistrationState::Handover {
                        from_device_id: current_from.clone(),
                        to_device_id: current_to.clone(),
                        initiated_at: *initiated_at,
                    },
                    RegistrySideEffect::None,
                    "registration_handover_reaffirmed",
                )
            }
            RegistrationState::Handover { .. } => {
                return Err("registration_handover_conflict".to_string());
            }
            RegistrationState::Abandoned { device_id, .. } if device_id == &from_device_id => (
                RegistrationState::Abandoned {
                    device_id: device_id.clone(),
                    abandoned_at: now,
                },
                RegistrySideEffect::None,
                "registration_abandon_reaffirmed",
            ),
            RegistrationState::Abandoned { .. } => {
                return Err("registration_device_mismatch".to_string());
            }
        };

        self.persist_state(
            identity_id,
            Some(previous_state),
            state.clone(),
            reason,
            now,
        )?;
        Ok(RegistrationUpdateOutcome { state, side_effect })
    }

    pub fn get_state(&self, identity_id: &str) -> Result<Option<RegistrationState>, String> {
        let Some(identity_id) = self.normalize_lookup_identity(identity_id)? else {
            return Ok(None);
        };
        Ok(self
            .get_record_by_identity_id(&identity_id)?
            .map(|record| record.state))
    }

    pub fn get_state_info(&self, identity_id: &str) -> RegistrationStateInfo {
        match self.get_state(identity_id).ok().flatten() {
            Some(RegistrationState::Active {
                device_id,
                seniority_timestamp,
            }) => RegistrationStateInfo {
                state: "active".to_string(),
                device_id: Some(device_id),
                seniority_timestamp: Some(seniority_timestamp),
            },
            Some(RegistrationState::Handover { to_device_id, .. }) => RegistrationStateInfo {
                state: "handover".to_string(),
                device_id: Some(to_device_id),
                seniority_timestamp: None,
            },
            Some(RegistrationState::Abandoned { device_id, .. }) => RegistrationStateInfo {
                state: "abandoned".to_string(),
                device_id: Some(device_id),
                seniority_timestamp: None,
            },
            None => RegistrationStateInfo {
                state: "none".to_string(),
                device_id: None,
                seniority_timestamp: None,
            },
        }
    }

    pub fn enforce_custody(
        &self,
        identity_id: &str,
        device_id: &str,
    ) -> Result<CustodyEnforcement, CustodyError> {
        let Some(identity_id) = self
            .normalize_lookup_identity(identity_id)
            .map_err(|_| CustodyError::NoRegistration)?
        else {
            return Err(CustodyError::NoRegistration);
        };
        let device_id = normalize_uuid_v4(device_id, "registration_device_id_invalid")
            .map_err(|_| CustodyError::NoRegistration)?;

        match self
            .get_state(&identity_id)
            .map_err(|_| CustodyError::NoRegistration)?
        {
            Some(RegistrationState::Active {
                device_id: active_device_id,
                ..
            }) if active_device_id == device_id => Ok(CustodyEnforcement::Active {
                identity_id,
                device_id,
            }),
            Some(RegistrationState::Active { .. }) => Err(CustodyError::DeviceMismatch),
            Some(RegistrationState::Handover {
                from_device_id,
                to_device_id,
                ..
            }) if device_id == from_device_id || device_id == to_device_id => {
                Ok(CustodyEnforcement::Redirected {
                    identity_id,
                    from_device_id,
                    to_device_id,
                })
            }
            Some(RegistrationState::Handover { .. }) => Err(CustodyError::DeviceMismatch),
            Some(RegistrationState::Abandoned { .. }) => Err(CustodyError::AbandonedIdentity),
            None => Err(CustodyError::NoRegistration),
        }
    }

    pub fn transitions_for_identity(&self, identity_id: &str) -> Vec<RegistrationTransition> {
        let Some(identity_id) = self.normalize_lookup_identity(identity_id).ok().flatten() else {
            return Vec::new();
        };

        let mut transitions: Vec<RegistrationTransition> = self
            .backend
            .scan_prefix(REGISTRATION_AUDIT_PREFIX.as_bytes())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, value)| bincode::deserialize::<RegistrationTransition>(&value).ok())
            .filter(|transition| transition.identity_id == identity_id)
            .collect();
        transitions.sort_by_key(|transition| (transition.at_ms, transition.sequence));
        transitions
    }

    fn normalize_lookup_identity(&self, identity_hint: &str) -> Result<Option<String>, String> {
        let trimmed = identity_hint.trim().to_lowercase();
        if trimmed.is_empty() || !is_hex_64(&trimmed) {
            return Ok(None);
        }
        if self.record_exists(&trimmed)? {
            return Ok(Some(trimmed));
        }

        if let Some(derived) = derive_identity_id_from_public_key_hex(&trimmed) {
            if self.record_exists(&derived)? {
                return Ok(Some(derived));
            }
        }

        Ok(Some(trimmed))
    }

    fn record_exists(&self, identity_id: &str) -> Result<bool, String> {
        Ok(self
            .backend
            .get(registration_key(identity_id).as_bytes())?
            .is_some())
    }

    fn get_record_by_identity_id(
        &self,
        identity_id: &str,
    ) -> Result<Option<RegistrationRecord>, String> {
        let key = registration_key(identity_id);
        let Some(bytes) = self.backend.get(key.as_bytes())? else {
            return Ok(None);
        };
        let record = bincode::deserialize::<RegistrationRecord>(&bytes)
            .map_err(|e| format!("deserialize registration state failed: {}", e))?;
        if let RegistrationState::Handover {
            from_device_id,
            initiated_at,
            ..
        } = &record.state
        {
            if now_ms().saturating_sub(*initiated_at) > HANDOVER_STALE_COLLAPSE_MS {
                let collapsed = RegistrationState::Abandoned {
                    device_id: from_device_id.clone(),
                    abandoned_at: now_ms(),
                };
                self.persist_state(
                    identity_id.to_string(),
                    Some(record.state),
                    collapsed.clone(),
                    "registration_handover_stale_timeout",
                    now_ms(),
                )?;
                return Ok(Some(RegistrationRecord {
                    identity_id: identity_id.to_string(),
                    state: collapsed,
                    updated_at_ms: now_ms(),
                }));
            }
        }
        Ok(Some(record))
    }

    fn persist_state(
        &self,
        identity_id: String,
        from_state: Option<RegistrationState>,
        state: RegistrationState,
        reason: &str,
        updated_at_ms: u64,
    ) -> Result<(), String> {
        let record = RegistrationRecord {
            identity_id: identity_id.clone(),
            state: state.clone(),
            updated_at_ms,
        };
        let bytes = bincode::serialize(&record)
            .map_err(|e| format!("serialize registration state failed: {}", e))?;
        self.backend
            .put(registration_key(&identity_id).as_bytes(), &bytes)?;
        self.record_transition(identity_id, from_state, state, reason, updated_at_ms)?;
        self.backend.flush()?;
        Ok(())
    }

    fn record_transition(
        &self,
        identity_id: String,
        from_state: Option<RegistrationState>,
        to_state: RegistrationState,
        reason: &str,
        at_ms: u64,
    ) -> Result<(), String> {
        let sequence = CUSTODY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let key = format!(
            "{}{:020}_{:06}_{}",
            REGISTRATION_AUDIT_PREFIX, at_ms, sequence, identity_id
        );
        let transition = RegistrationTransition {
            identity_id,
            from_state,
            to_state,
            reason: reason.to_string(),
            at_ms,
            sequence,
        };
        let bytes = bincode::serialize(&transition)
            .map_err(|e| format!("serialize registration transition failed: {}", e))?;
        self.backend.put(key.as_bytes(), &bytes)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct StoredCustodyRecord {
    record: CustodyMessage,
    serialized_bytes: u64,
}

impl StoredCustodyRecord {
    fn identity_related(&self, store: &RelayCustodyStore) -> bool {
        store.is_identity_related_record(&self.record)
    }

    fn delivery_priority(&self) -> u8 {
        if self.record.state == CustodyState::Delivered {
            0
        } else {
            1
        }
    }
}

fn serialized_record_bytes(record: &CustodyMessage) -> Result<u64, String> {
    let bytes = bincode::serialize(record)
        .map_err(|e| format!("serialize custody record failed: {}", e))?;
    Ok(bytes.len() as u64)
}

fn is_hex_64(value: &str) -> bool {
    value.len() == 64 && value.as_bytes().iter().all(|byte| byte.is_ascii_hexdigit())
}

fn normalize_identity_id(identity_id: &str) -> Result<String, String> {
    let normalized = identity_id.trim().to_lowercase();
    if !is_hex_64(&normalized) {
        return Err("registration_identity_id_invalid".to_string());
    }
    Ok(normalized)
}

fn normalize_uuid_v4(value: &str, error: &str) -> Result<String, String> {
    let normalized = value.trim();
    let uuid = Uuid::parse_str(normalized).map_err(|_| error.to_string())?;
    if uuid.get_version_num() != 4 {
        return Err(error.to_string());
    }
    Ok(normalized.to_string())
}

fn derive_identity_id_from_public_key_hex(value: &str) -> Option<String> {
    if !is_hex_64(value) {
        return None;
    }
    let bytes = hex::decode(value).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    Some(hex::encode(blake3::hash(&bytes).as_bytes()))
}

fn registration_key(identity_id: &str) -> String {
    format!("{}{}", REGISTRATION_STATE_PREFIX, identity_id)
}

fn synthetic_storage_snapshot(scm_bytes: u64) -> Option<DeviceStorageSnapshot> {
    let total_bytes = std::env::var("SCM_STORAGE_TOTAL_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(FALLBACK_STORAGE_TOTAL_BYTES);
    let used_bytes = std::env::var("SCM_STORAGE_USED_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.max(scm_bytes))
        .unwrap_or(scm_bytes);
    if total_bytes == 0 {
        return None;
    }
    Some(DeviceStorageSnapshot {
        total_bytes,
        used_bytes: used_bytes.min(total_bytes),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn custody_base_dir() -> PathBuf {
    if let Ok(path) = std::env::var("SCM_RELAY_CUSTODY_DIR") {
        return PathBuf::from(path);
    }
    if let Some(path) = dirs::data_local_dir() {
        return path.join("scmessenger").join("relay_custody");
    }
    if let Some(path) = dirs::home_dir() {
        return path.join(".scmessenger").join("relay_custody");
    }
    std::env::temp_dir().join("scmessenger_relay_custody")
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(unix)]
fn filesystem_usage_bytes(path: &std::path::Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: std::mem::zeroed() is safe here because `libc::statvfs` is a C struct
    // with only primitive numeric fields; zeroing is the required initialization
    // before passing it to `statvfs(3)`.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a valid NUL-terminated C string created by `CString`,
    // and `stat` is a mutable reference to a properly zeroed `libc::statvfs`.
    // These invariants satisfy the preconditions of `statvfs(3)`.
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        return None;
    }

    let block_size = if stat.f_frsize > 0 {
        stat.f_frsize
    } else {
        stat.f_bsize
    };
    if block_size == 0 {
        return None;
    }

    let total_bytes = (stat.f_blocks as u128)
        .saturating_mul(block_size as u128)
        .min(u64::MAX as u128) as u64;
    let available_bytes = (stat.f_bavail as u128)
        .saturating_mul(block_size as u128)
        .min(u64::MAX as u128) as u64;
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    Some((total_bytes, used_bytes))
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(unix))]
fn filesystem_usage_bytes(_path: &std::path::Path) -> Option<(u64, u64)> {
    None
}

fn now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn destination_prefix(destination_peer_id: &str) -> String {
    format!("{}{}_", CUSTODY_MSG_PREFIX, destination_peer_id)
}

fn message_key(destination_peer_id: &str, custody_id: &str) -> String {
    format!("{}{}", destination_prefix(destination_peer_id), custody_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    use web_time::Duration;

    #[derive(Default)]
    struct TestPressureProbe {
        snapshot: RwLock<Option<DeviceStorageSnapshot>>,
    }

    impl TestPressureProbe {
        fn set(&self, snapshot: DeviceStorageSnapshot) {
            *self.snapshot.write().expect("snapshot lock poisoned") = Some(snapshot);
        }
    }

    impl StoragePressureProbe for TestPressureProbe {
        fn snapshot(&self) -> Option<DeviceStorageSnapshot> {
            *self.snapshot.read().expect("snapshot lock poisoned")
        }
    }

    #[test]
    fn custody_transitions_are_recorded() {
        let store = RelayCustodyStore::in_memory();
        let accepted = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-1".to_string(),
                vec![1, 2, 3],
                None,
                None,
            )
            .unwrap();

        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "reconnect_pull")
            .unwrap();
        store
            .mark_dispatch_failed(
                "destination-peer",
                &accepted.custody_id,
                "temporary_failure",
            )
            .unwrap();
        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "retry")
            .unwrap();
        store
            .mark_delivered("destination-peer", &accepted.custody_id, "recipient_ack")
            .unwrap();

        let transitions = store.transitions_for_custody(&accepted.custody_id);
        assert_eq!(transitions.len(), 5);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].to_state, CustodyState::Dispatching);
        assert_eq!(transitions[2].to_state, CustodyState::Accepted);
        assert_eq!(transitions[3].to_state, CustodyState::Dispatching);
        assert_eq!(transitions[4].to_state, CustodyState::Delivered);
        assert!(store
            .pending_for_destination("destination-peer", 100)
            .is_empty());
    }

    /// D1 regression (live failure 2026-09-09): dispatch attempts burned at the
    /// periodic-pull cadence while the destination was restarting, the guard
    /// tripped, and the entry was refused FOREVER afterwards ("Max delivery
    /// attempts (12) exceeded" every 5s while the destination sat connected).
    /// A fresh connection episode must re-arm the counter so store-and-forward
    /// outlives short destination absences; the per-episode cap must still stop
    /// runaway retries within one episode.
    #[test]
    fn fresh_connection_re_arms_exhausted_custody_dispatch_attempts() {
        let store = RelayCustodyStore::in_memory();
        let accepted = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-d1".to_string(),
                vec![9, 9, 9],
                None,
                None,
            )
            .unwrap();

        // Burn through the per-episode guard exactly as the periodic pull did.
        for _ in 0..12 {
            store
                .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
                .unwrap();
            store
                .mark_dispatch_failed(
                    "destination-peer",
                    &accepted.custody_id,
                    "no_response",
                )
                .unwrap();
        }
        // Guard tripped: the 13th dispatch is refused.
        assert!(store
            .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
            .is_err());

        // Fresh connection episode re-arms the destination's pending custody.
        let re_armed = store.reset_delivery_attempts_for_destination("destination-peer");
        assert_eq!(re_armed, 1, "the one pending entry must be re-armed");

        // Dispatch succeeds again after the re-arm, and the per-episode cap
        // still holds after 12 more attempts within the SAME episode.
        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
            .expect("post-re-arm dispatch must succeed");
        // 12 more failed attempts within the SAME episode (the first
        // mark_dispatching above consumed attempt 1 while already Dispatching
        // the record was left Accepted by each failure, so iterations here
        // burn attempts 2..=12).
        for _ in 0..11 {
            store
                .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
                .expect("attempts 2..=12 must be accepted within the episode");
            store
                .mark_dispatch_failed(
                    "destination-peer",
                    &accepted.custody_id,
                    "no_response",
                )
                .unwrap();
        }
        // Attempt 12 is the last allowed; the 13th must be refused.
        store
            .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
            .expect("attempt 12 must still be accepted");
        store
            .mark_dispatch_failed("destination-peer", &accepted.custody_id, "no_response")
            .unwrap();
        assert!(
            store
                .mark_dispatching("destination-peer", &accepted.custody_id, "periodic_pull")
                .is_err(),
            "per-episode cap must still stop runaway retries after re-arm"
        );
    }

    #[test]
    fn custody_rearm_skips_delivered_records() {
        let store = RelayCustodyStore::in_memory();
        let accepted = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-delivered".to_string(),
                vec![1, 1, 1],
                None,
                None,
            )
            .unwrap();
        store
            .mark_delivered("destination-peer", &accepted.custody_id, "recipient_ack")
            .unwrap();
        assert_eq!(
            store.reset_delivery_attempts_for_destination("destination-peer"),
            0,
            "delivered records must not be resurrected by a re-arm"
        );
    }

    #[test]
    fn custody_deduplicates_same_destination_and_message_id() {
        let store = RelayCustodyStore::in_memory();
        let first = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-dedupe".to_string(),
                vec![9, 9, 9],
                None,
                None,
            )
            .unwrap();
        let second = store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-dedupe".to_string(),
                vec![9, 9, 9],
                None,
                None,
            )
            .unwrap();

        assert_eq!(first.custody_id, second.custody_id);
        assert_eq!(
            store.pending_for_destination("destination-peer", 100).len(),
            1
        );
    }

    #[test]
    fn converge_delivered_for_message_removes_matching_pending_records() {
        let store = RelayCustodyStore::in_memory();
        let _ = store
            .accept_custody(
                "source-peer-a".to_string(),
                "destination-peer".to_string(),
                "relay-msg-converge".to_string(),
                vec![1, 2, 3],
                None,
                None,
            )
            .unwrap();
        let other = store
            .accept_custody(
                "source-peer-b".to_string(),
                "destination-peer".to_string(),
                "relay-msg-other".to_string(),
                vec![4, 5, 6],
                None,
                None,
            )
            .unwrap();

        let converged = store
            .converge_delivered_for_message(
                "destination-peer",
                "relay-msg-converge",
                "delivery_converged",
            )
            .unwrap();
        assert_eq!(converged, 1);

        let pending = store.pending_for_destination("destination-peer", 100);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].relay_message_id, "relay-msg-other");

        let transitions = store.transitions_for_custody(&other.custody_id);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
    }

    #[test]
    fn storage_pressure_quota_bands_follow_locked_policy() {
        let scm_bytes = 50_000u64;
        let ctx20 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 200_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s20 = ctx20.state_for_scm_bytes(scm_bytes);
        assert_eq!(s20.band, StoragePressureBand::UpTo20Pct);
        assert_eq!(s20.target_quota_bytes, 560_000);

        let ctx50 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 500_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s50 = ctx50.state_for_scm_bytes(scm_bytes);
        assert_eq!(s50.band, StoragePressureBand::From20To50Pct);
        assert_eq!(s50.target_quota_bytes, 225_000);

        let ctx70 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 700_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s70 = ctx70.state_for_scm_bytes(scm_bytes);
        assert_eq!(s70.band, StoragePressureBand::From50To70Pct);
        assert_eq!(s70.target_quota_bytes, 75_000);

        let ctx80 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 800_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s80 = ctx80.state_for_scm_bytes(scm_bytes);
        assert_eq!(s80.band, StoragePressureBand::From70To80Pct);
        assert_eq!(s80.target_quota_bytes, 20_000);

        let ctx90 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 900_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s90 = ctx90.state_for_scm_bytes(scm_bytes);
        assert_eq!(s90.band, StoragePressureBand::From80To90Pct);
        assert_eq!(s90.target_quota_bytes, 3_000);

        let ctx_over_90 = StoragePressureContext::from_snapshot(
            DeviceStorageSnapshot {
                total_bytes: 1_000_000,
                used_bytes: 910_000,
            },
            scm_bytes,
        )
        .unwrap();
        let s_over_90 = ctx_over_90.state_for_scm_bytes(scm_bytes);
        assert_eq!(s_over_90.band, StoragePressureBand::EmergencyOver90Pct);
        assert!(s_over_90.emergency_mode());
        assert_eq!(s_over_90.target_quota_bytes, s_over_90.hard_ceiling_bytes);
    }

    fn seed_purge_order_records(store: &RelayCustodyStore) {
        let payload = vec![7u8; 256];
        let _ = store
            .accept_custody(
                "peer-non-old-src".to_string(),
                "peer-non-old-dst".to_string(),
                "msg-non-old".to_string(),
                payload.clone(),
                None,
                None,
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "local-peer".to_string(),
                "peer-id-old-dst".to_string(),
                "msg-id-old".to_string(),
                payload.clone(),
                None,
                None,
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "peer-non-new-src".to_string(),
                "peer-non-new-dst".to_string(),
                "msg-non-new".to_string(),
                payload.clone(),
                None,
                None,
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(1));
        let _ = store
            .accept_custody(
                "peer-id-new-src".to_string(),
                "local-peer".to_string(),
                "msg-id-new".to_string(),
                payload,
                None,
                None,
            )
            .unwrap();
    }

    #[test]
    fn storage_pressure_purge_prioritizes_non_identity_then_identity() {
        let store = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        seed_purge_order_records(&store);

        let records = store.load_stored_records().unwrap();
        assert_eq!(records.len(), 4);
        let non_identity_bytes = records
            .iter()
            .filter(|entry| !entry.identity_related(&store))
            .map(|entry| entry.serialized_bytes)
            .sum::<u64>();

        let (purged_records, _) = store
            .purge_oldest_by_policy(non_identity_bytes, "test_non_identity_first")
            .unwrap();
        assert_eq!(purged_records, 2);

        let mut remaining: Vec<String> = store
            .load_stored_records()
            .unwrap()
            .into_iter()
            .map(|entry| entry.record.relay_message_id)
            .collect();
        remaining.sort();
        assert_eq!(
            remaining,
            vec!["msg-id-new".to_string(), "msg-id-old".to_string()]
        );

        let store2 = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        seed_purge_order_records(&store2);

        let records2 = store2.load_stored_records().unwrap();
        let non_identity_bytes2 = records2
            .iter()
            .filter(|entry| !entry.identity_related(&store2))
            .map(|entry| entry.serialized_bytes)
            .sum::<u64>();

        let (purged_records2, _) = store2
            .purge_oldest_by_policy(non_identity_bytes2 + 1, "test_identity_when_required")
            .unwrap();
        assert_eq!(purged_records2, 3);

        let remaining2 = store2.load_stored_records().unwrap();
        assert_eq!(remaining2.len(), 1);
        assert_eq!(remaining2[0].record.relay_message_id, "msg-id-new");
    }

    #[test]
    fn storage_pressure_purge_records_audit_transition_before_delete() {
        let store = RelayCustodyStore::in_memory_with_probe(
            Some("local-peer".to_string()),
            Arc::new(NoopStoragePressureProbe),
        );
        let accepted = store
            .accept_custody(
                "peer-src".to_string(),
                "peer-dst".to_string(),
                "relay-msg-audit-purge".to_string(),
                vec![9u8; 64],
                None,
                None,
            )
            .unwrap();

        let (purged_records, purged_bytes) =
            store.purge_oldest_by_policy(1, "test_pressure").unwrap();
        assert_eq!(purged_records, 1);
        assert!(purged_bytes > 0);

        let transitions = store.transitions_for_custody(&accepted.custody_id);
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].from_state, Some(CustodyState::Accepted));
        assert_eq!(transitions[1].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].reason, "test_pressure_purged");
    }

    #[test]
    fn storage_pressure_emergency_mode_rejects_non_critical_and_recovers() {
        let probe = Arc::new(TestPressureProbe::default());
        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 50_000,
        });
        let store =
            RelayCustodyStore::in_memory_with_probe(Some("local-peer".to_string()), probe.clone());

        let _ = store
            .accept_custody(
                "peer-pre-emergency-src".to_string(),
                "peer-pre-emergency-dst".to_string(),
                "msg-pre-emergency".to_string(),
                vec![1u8; 256],
                None,
                None,
            )
            .unwrap();

        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 95_000,
        });
        let report = store.enforce_storage_pressure().unwrap();
        assert!(report.emergency_mode);
        assert_eq!(
            store.storage_pressure_state().unwrap().band,
            StoragePressureBand::EmergencyOver90Pct
        );

        let rejected = store.accept_custody(
            "peer-emergency-src".to_string(),
            "peer-emergency-dst".to_string(),
            "msg-rejected-emergency".to_string(),
            vec![2u8; 256],
            None,
            None,
        );
        assert!(rejected
            .unwrap_err()
            .contains("emergency_mode_non_critical_rejected"));

        probe.set(DeviceStorageSnapshot {
            total_bytes: 100_000,
            used_bytes: 85_000,
        });
        let accepted = store.accept_custody(
            "peer-post-emergency-src".to_string(),
            "peer-post-emergency-dst".to_string(),
            "msg-post-emergency".to_string(),
            vec![3u8; 256],
            None,
            None,
        );
        assert!(accepted.is_ok());
        assert_ne!(
            store.storage_pressure_state().unwrap().band,
            StoragePressureBand::EmergencyOver90Pct
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn custody_audit_persists_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("relay_custody_store");
        let path_str = path.to_string_lossy().to_string();

        let custody_id = {
            let backend = Arc::new(SledStorage::new(&path_str).unwrap());
            let store = RelayCustodyStore::persistent(backend);
            let accepted = store
                .accept_custody(
                    "source-peer".to_string(),
                    "destination-peer".to_string(),
                    "relay-msg-persist".to_string(),
                    vec![7, 7, 7],
                    None,
                    None,
                )
                .unwrap();
            store
                .mark_dispatching("destination-peer", &accepted.custody_id, "reconnect_pull")
                .unwrap();
            accepted.custody_id
        };

        let backend = Arc::new(SledStorage::new(&path_str).unwrap());
        let reloaded = RelayCustodyStore::persistent(backend);
        let transitions = reloaded.transitions_for_custody(&custody_id);
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to_state, CustodyState::Accepted);
        assert_eq!(transitions[1].to_state, CustodyState::Dispatching);
        assert_eq!(
            reloaded
                .pending_for_destination("destination-peer", 100)
                .len(),
            0
        );
    }

    #[test]
    fn storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable() {
        let store = RelayCustodyStore::in_memory();
        store
            .accept_custody(
                "source-peer".to_string(),
                "destination-peer".to_string(),
                "relay-msg-snapshot-fallback".to_string(),
                vec![1u8; 64],
                None,
                None,
            )
            .unwrap();
        let state = store.storage_pressure_state();
        assert!(state.is_some(), "expected synthetic snapshot fallback");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn for_local_peer_prefers_explicit_custody_dir_override() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("custody-base");
        #[allow(clippy::disallowed_methods)]
        std::env::set_var("SCM_RELAY_CUSTODY_DIR", &base);
        let peer = "peer-override";
        let _store = RelayCustodyStore::for_local_peer(peer);
        assert!(
            base.join(peer).exists(),
            "expected custody dir override to be created"
        );
        std::env::remove_var("SCM_RELAY_CUSTODY_DIR");
    }

    // --- WS13.6 custody enforcement state machine tests ---

    #[test]
    fn custody_enforcement_accepts_active_device_match() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "a".repeat(64);
        let device_id = Uuid::new_v4().to_string();
        let seniority = 1700000000u64;

        store
            .register_identity(identity_id.clone(), device_id.clone(), seniority)
            .unwrap();

        let result = store.enforce_custody(&identity_id, &device_id);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CustodyEnforcement::Active {
                identity_id,
                device_id
            }
        );
    }

    #[test]
    fn custody_enforcement_rejects_device_mismatch() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "b".repeat(64);
        let device_id = Uuid::new_v4().to_string();
        let other_device = Uuid::new_v4().to_string();
        let seniority = 1700000000u64;

        store
            .register_identity(identity_id.clone(), device_id, seniority)
            .unwrap();

        let result = store.enforce_custody(&identity_id, &other_device);
        assert_eq!(result.unwrap_err(), CustodyError::DeviceMismatch);
    }

    #[test]
    fn custody_enforcement_rejects_no_registration() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "c".repeat(64);
        let device_id = Uuid::new_v4().to_string();

        let result = store.enforce_custody(&identity_id, &device_id);
        assert_eq!(result.unwrap_err(), CustodyError::NoRegistration);
    }

    #[test]
    fn custody_enforcement_redirects_during_handover() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "d".repeat(64);
        let from_device = Uuid::new_v4().to_string();
        let to_device = Uuid::new_v4().to_string();
        let seniority = 1700000000u64;

        store
            .register_identity(identity_id.clone(), from_device.clone(), seniority)
            .unwrap();
        store
            .deregister_identity(
                identity_id.clone(),
                from_device.clone(),
                Some(to_device.clone()),
            )
            .unwrap();

        // The from_device should get a redirect
        let result = store.enforce_custody(&identity_id, &from_device);
        assert!(result.is_ok());
        match result.unwrap() {
            CustodyEnforcement::Redirected {
                from_device_id,
                to_device_id,
                ..
            } => {
                assert_eq!(from_device_id, from_device);
                assert_eq!(to_device_id, to_device);
            }
            other => panic!("expected Redirected, got {:?}", other),
        }
    }

    #[test]
    fn custody_enforcement_rejects_abandoned_identity() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "e".repeat(64);
        let device_id = Uuid::new_v4().to_string();
        let seniority = 1700000000u64;

        store
            .register_identity(identity_id.clone(), device_id.clone(), seniority)
            .unwrap();
        store
            .deregister_identity(identity_id.clone(), device_id, None)
            .unwrap();

        let result = store.enforce_custody(&identity_id, &Uuid::new_v4().to_string());
        assert_eq!(result.unwrap_err(), CustodyError::AbandonedIdentity);
    }

    #[test]
    fn compat_mode_defaults_to_phase_a() {
        assert_eq!(CustodyCompatMode::default(), CustodyCompatMode::PhaseA);
    }

    #[test]
    fn store_compat_mode_defaults_to_phase_a() {
        let store = RelayCustodyStore::in_memory();
        assert_eq!(store.compat_mode(), CustodyCompatMode::PhaseA);
    }

    #[test]
    fn store_compat_mode_can_transition_to_phase_b() {
        let mut store = RelayCustodyStore::in_memory();
        store.set_compat_mode(CustodyCompatMode::PhaseB);
        assert_eq!(store.compat_mode(), CustodyCompatMode::PhaseB);
    }

    #[test]
    fn phase_a_accepts_legacy_request_without_device_id() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "a".repeat(64);

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-a-legacy".to_string(),
            vec![1, 2, 3],
            Some(identity_id),
            None,
        );

        assert!(
            result.is_ok(),
            "Phase A must accept legacy requests without device_id"
        );
    }

    #[test]
    fn phase_b_accepts_legacy_request_without_device_id() {
        let mut store = RelayCustodyStore::in_memory();
        store.set_compat_mode(CustodyCompatMode::PhaseB);
        let identity_id = "b".repeat(64);

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-b-legacy".to_string(),
            vec![4, 5, 6],
            Some(identity_id),
            None,
        );

        assert!(
            result.is_ok(),
            "Phase B must still accept legacy requests without device_id (with deprecation warning)"
        );
    }

    #[test]
    fn phase_b_enforces_ws13_device_mismatch() {
        let mut store = RelayCustodyStore::in_memory();
        store.set_compat_mode(CustodyCompatMode::PhaseB);
        let identity_id = "c".repeat(64);
        let registered_device = Uuid::new_v4().to_string();
        let wrong_device = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), registered_device, 1700000000)
            .unwrap();

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-b-mismatch".to_string(),
            vec![7, 8, 9],
            Some(identity_id),
            Some(wrong_device),
        );

        assert!(
            result.is_err(),
            "Phase B must enforce device mismatch for WS13+ clients"
        );
        assert!(result.unwrap_err().contains("identity_device_mismatch"));
    }

    #[test]
    fn phase_a_enforces_ws13_device_mismatch() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "d".repeat(64);
        let registered_device = Uuid::new_v4().to_string();
        let wrong_device = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), registered_device, 1700000000)
            .unwrap();

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-a-mismatch".to_string(),
            vec![10, 11, 12],
            Some(identity_id),
            Some(wrong_device),
        );

        assert!(
            result.is_err(),
            "Phase A must also enforce device mismatch for WS13+ clients with device_id"
        );
        assert!(result.unwrap_err().contains("identity_device_mismatch"));
    }

    #[test]
    fn phase_a_accepts_ws13_active_device_match() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "e".repeat(64);
        let device_id = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
            .unwrap();

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-a-match".to_string(),
            vec![13, 14, 15],
            Some(identity_id),
            Some(device_id),
        );

        assert!(
            result.is_ok(),
            "Phase A must accept WS13+ active device match"
        );
    }

    #[test]
    fn phase_b_accepts_ws13_active_device_match() {
        let mut store = RelayCustodyStore::in_memory();
        store.set_compat_mode(CustodyCompatMode::PhaseB);
        let identity_id = "f".repeat(64);
        let device_id = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
            .unwrap();

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-phase-b-match".to_string(),
            vec![16, 17, 18],
            Some(identity_id),
            Some(device_id),
        );

        assert!(
            result.is_ok(),
            "Phase B must accept WS13+ active device match"
        );
    }

    #[test]
    fn compat_mode_allows_handover_redirect() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "1".repeat(64);
        let from_device = Uuid::new_v4().to_string();
        let to_device = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), from_device.clone(), 1700000000)
            .unwrap();
        store
            .deregister_identity(
                identity_id.clone(),
                from_device.clone(),
                Some(to_device.clone()),
            )
            .unwrap();

        // WS13+ request with from_device during handover should redirect
        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-handover-redirect".to_string(),
            vec![19, 20, 21],
            Some(identity_id),
            Some(from_device),
        );

        assert!(
            result.is_ok(),
            "Handover redirect must be accepted at store level"
        );
    }

    #[test]
    fn test_adoption_stats_start_at_zero() {
        let store = RelayCustodyStore::in_memory();
        let (ws13, legacy) = store.adoption_stats();
        assert_eq!(ws13, 0, "ws13_requests should start at 0");
        assert_eq!(legacy, 0, "legacy_requests should start at 0");
        assert_eq!(
            store.adoption_pct(),
            0.0,
            "adoption_pct should be 0.0 with no requests"
        );
    }

    #[test]
    fn test_ws13_request_increments_counter() {
        let store = RelayCustodyStore::in_memory();
        let identity_id = "a".repeat(64);
        let device_id = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
            .unwrap();

        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-ws13-count".to_string(),
            vec![1, 2, 3],
            Some(identity_id),
            Some(device_id),
        );
        assert!(result.is_ok(), "WS13+ custody accept should succeed");

        let (ws13, legacy) = store.adoption_stats();
        assert_eq!(ws13, 1, "ws13_requests should be 1 after one WS13+ accept");
        assert_eq!(legacy, 0, "legacy_requests should remain 0");
    }

    #[test]
    fn test_legacy_request_increments_counter() {
        let mut store = RelayCustodyStore::in_memory();
        store.set_compat_mode(CustodyCompatMode::PhaseA);
        let identity_id = "b".repeat(64);

        // Accept custody without a device_id (legacy client)
        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-legacy-count".to_string(),
            vec![4, 5, 6],
            Some(identity_id),
            None,
        );
        assert!(
            result.is_ok(),
            "Legacy custody accept should succeed in Phase A"
        );

        let (ws13, legacy) = store.adoption_stats();
        assert_eq!(ws13, 0, "ws13_requests should remain 0");
        assert_eq!(
            legacy, 1,
            "legacy_requests should be 1 after one legacy accept"
        );
    }

    #[test]
    fn test_adoption_pct_calculation() {
        let store = RelayCustodyStore::in_memory();
        let identity_id_ws13 = "c".repeat(64);
        let device_id = Uuid::new_v4().to_string();

        store
            .register_identity(identity_id_ws13.clone(), device_id.clone(), 1700000000)
            .unwrap();

        // First: 3 WS13+ requests
        for i in 0..3 {
            let result = store.accept_custody(
                "source-peer".to_string(),
                "dest-peer".to_string(),
                format!("msg-ws13-pct-{}", i),
                vec![1],
                Some(identity_id_ws13.clone()),
                Some(device_id.clone()),
            );
            assert!(result.is_ok());
        }

        // Then: 1 legacy request (no device_id)
        let identity_id_legacy = "d".repeat(64);
        let result = store.accept_custody(
            "source-peer".to_string(),
            "dest-peer".to_string(),
            "msg-legacy-pct".to_string(),
            vec![2],
            Some(identity_id_legacy),
            None,
        );
        assert!(result.is_ok());

        let (ws13, legacy) = store.adoption_stats();
        assert_eq!(ws13, 3);
        assert_eq!(legacy, 1);

        let pct = store.adoption_pct();
        assert!(
            (pct - 75.0).abs() < f64::EPSILON,
            "adoption_pct should be 75.0, got {}",
            pct
        );
    }
}
