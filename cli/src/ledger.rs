// Connection Ledger — process-lifetime dial state over the CORE ledger store
//
// Philosophy: "A node is a node." IP is the source of truth.
//
// T2 unification (2026-08-31): the CLI's own persisted `peers.json` store is
// GONE. The core `LedgerManager` is the single peer store for the whole node;
// this module keeps only what is process-lifetime and inherently CLI-side:
// the per-peer/per-address dial scheduler state (backoff, in-flight claims,
// concurrent-connection caps) and the DialPolicyManager. Every piece of
// durable peer memory — addresses, verified status, topics, bootstrap flags —
// lives in the core store, which the swarm itself reads and writes
// (`record_connection` fires on outbound `ConnectionEstablished` in core).
//
// The one-time migration (`run_legacy_migration`) imports surviving entries
// from a pre-unification `peers.json` into the core store, preserving
// `locally_verified`, then archives the file. Nothing writes `peers.json`
// anymore.

use anyhow::{Context, Result};
use libp2p::{Multiaddr, PeerId};
use scmessenger_core::store::{LedgerManager, LedgerMigrationEntry};
use scmessenger_core::transport::dial_policy::DialPolicyManager;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Key for per-peer dial state: PeerId when known, else the stripped
/// multiaddr (address-only dials must NEVER be dropped).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DialKey {
    Peer(PeerId),
    Addr(String),
}

impl DialKey {
    /// Build a key from a target multiaddr and optional known PeerId.
    pub fn for_target(multiaddr_str: &str, peer_id: Option<PeerId>) -> Self {
        if let Some(pid) = peer_id {
            return Self::Peer(pid);
        }

        if let Some(idx) = multiaddr_str.find("/p2p/") {
            let remainder = &multiaddr_str[idx + "/p2p/".len()..];
            if let Ok(pid) = PeerId::from_str(remainder) {
                return Self::Peer(pid);
            }
        }

        Self::Addr(strip_peer_id(multiaddr_str))
    }
}

/// Process-lifetime per-peer dial state (NOT persisted anywhere).
#[derive(Debug, Clone, Default)]
pub struct PeerDialState {
    /// Consecutive dial failures this session (1st failure -> 5s delay).
    pub consecutive_failures: u32,

    /// Unix ts: next allowed dial attempt (0 = now).
    pub next_attempt_after: u64,

    /// A dial for this key is currently in flight.
    pub in_flight: bool,

    /// Number of established connections.
    pub connections: u32,

    /// Has a successful connection history (seeded from ledger, set on success).
    pub is_known_good: bool,
}

impl PeerDialState {
    /// Backoff ladder in seconds: 5s, 30s, 2m, 5m, 30m.
    pub const BACKOFF_LADDER: [u64; 5] = [5, 30, 120, 300, 1800];

    /// Whether a new dial may be started now.
    ///
    /// `connections == 0` is the per-peer concurrent-connection cap (P0,
    /// 2026-08-12): once a peer already has an established connection, no
    /// further dials to its OTHER addresses may start. Keyed on PeerId via
    /// DialKey::Peer, independent of address -- the prior address-level guard
    /// alone let N distinct addresses of one peer open N simultaneous
    /// connections, which is the trigger for the libp2p-request-response
    /// connection-bookkeeping panic. The slot is released by
    /// `record_disconnect`; saturating arithmetic keeps a missed release from
    /// ever underflowing into a negative/wedged state.
    pub fn ready(&self, now: u64) -> bool {
        now >= self.next_attempt_after && !self.in_flight && self.connections == 0
    }

    /// Reset state after a successful dial.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.next_attempt_after = 0;
        self.in_flight = false;
        self.is_known_good = true;
    }

    /// Back off after a failed dial.
    pub fn record_failure(&mut self, now: u64) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let idx = std::cmp::min(self.consecutive_failures.saturating_sub(1), 4) as usize;
        self.next_attempt_after = now.saturating_add(Self::BACKOFF_LADDER[idx]);
        self.in_flight = false;
    }
}

/// Process-lifetime ADDRESS-level dial guard (NOT persisted anywhere).
///
/// `peer_dial_states` alone is not enough to stop simultaneous connections to
/// one physical host:port. It is keyed by `DialKey`, which is `Peer(pid)`
/// whenever a PeerId is known (see `DialKey::for_target`) -- and this fleet's
/// nodes mint a new identity on every rebuild, so one address can accumulate
/// many stale PeerIds in the core ledger's `observed_peer_ids`. Each stale
/// identity produces a DIFFERENT `DialKey::Peer`, so the peer-level guard
/// sees N unrelated dials while the OS opens N concurrent connections to the
/// same address -- which is exactly what tripped a `debug_assert_eq!` inside
/// `libp2p-request-response` in production (three simultaneous connections
/// to the byte-identical multiaddr within 30ms).
///
/// This guard is keyed on the normalized address string instead (see
/// `ConnectionLedger::key_to_policy_args`, which resolves a `DialKey::Peer`
/// back to its known address via the core ledger and already reuses
/// `strip_peer_id` for normalization), so it catches the collision
/// regardless of which PeerId a given dial attempt happens to be keyed on.
#[derive(Debug, Clone, Default)]
pub struct AddrDialState {
    /// An address-level dial is currently in flight.
    pub in_flight: bool,

    /// Unix ts the in-flight claim was made. Used only to expire a claim
    /// that never got released via `complete_dial` (see `STALE_CLAIM_SECS`).
    /// This is a concurrency guard, not a ban list -- an address must never
    /// be permanently unreachable because one dial attempt never completed.
    pub claimed_at: u64,
}

impl AddrDialState {
    /// A claim older than this is treated as abandoned. There is no existing
    /// timeout/expiry mechanism on the in-flight bit this guard mirrors
    /// (`PeerDialState::in_flight` has none either), so this is new: without
    /// it, a dial that starts and never calls `complete_dial` (panic, task
    /// drop, etc.) would wedge the address closed for the rest of the
    /// process's life, reproducing exactly the "address-only dials must
    /// never be dropped" bug this file already warns about elsewhere.
    pub const STALE_CLAIM_SECS: u64 = 120;

    /// Whether a new dial may claim this address now.
    fn ready(&self, now: u64) -> bool {
        !self.in_flight || now.saturating_sub(self.claimed_at) >= Self::STALE_CLAIM_SECS
    }
}

/// The Connection Ledger — dial state only. All peer memory lives in the
/// shared core [`LedgerManager`] (same storage path as `IronCore`, which
/// makes this handle the SAME store the swarm reads and writes).
pub struct ConnectionLedger {
    /// Handle onto the node's single peer store.
    core: LedgerManager,

    /// Process-lifetime per-peer dial state. Never persisted.
    pub peer_dial_states: HashMap<DialKey, PeerDialState>,

    /// Process-lifetime per-address dial state, keyed by the normalized
    /// (stripped) address string. See `AddrDialState` for why this exists
    /// alongside `peer_dial_states`. Never persisted.
    pub addr_dial_states: HashMap<String, AddrDialState>,

    /// Global dial policy manager enforcing per-peer backoff and concurrent dial limits.
    pub dial_policy: DialPolicyManager,
}

impl ConnectionLedger {
    /// Wrap the node's core ledger. The manager must be constructed over the
    /// SAME storage path as the `IronCore` the swarm runs on (the core
    /// registry then returns the identical shared entry state).
    pub fn new(core: LedgerManager) -> Self {
        Self {
            core,
            peer_dial_states: HashMap::new(),
            addr_dial_states: HashMap::new(),
            dial_policy: DialPolicyManager::new(),
        }
    }

    /// One-time migration from the pre-unification CLI `peers.json`.
    ///
    /// If `data_dir/peers.json` exists, parse it in the legacy format, import
    /// the survivors into the core store via
    /// [`LedgerManager::import_legacy_cli_entries`] (which applies the same
    /// dialability/self/port filters the node now enforces everywhere; the
    /// legacy `locally_verified` flag is NOT trusted -- only operator
    /// bootstrap survives as verified, everything else is re-proven by the
    /// first live dial), then rename the file to
    /// `peers.json.migrated-<ts>` so it cannot run again. Invoked once per
    /// node startup from the bootstrap-dial sweep, where the node's own
    /// acquired addresses are already known.
    pub fn run_legacy_migration(
        &self,
        data_dir: &Path,
        local_peer_id: Option<&str>,
        my_addrs: &[String],
    ) -> Result<LegacyMigrationReport> {
        let peers_path = data_dir.join("peers.json");
        if !peers_path.exists() {
            return Ok(LegacyMigrationReport::default());
        }
        let contents = match std::fs::read_to_string(&peers_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "[WARNING] legacy peers.json unreadable ({}); leaving in place",
                    e
                );
                return Ok(LegacyMigrationReport::default());
            }
        };
        let file: LegacyLedgerFile = match serde_json::from_str(&contents) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(
                    "[WARNING] legacy peers.json is not valid legacy JSON ({}); leaving in place",
                    e
                );
                return Ok(LegacyMigrationReport::default());
            }
        };

        let rows: Vec<LedgerMigrationEntry> = file
            .entries
            .into_values()
            .map(|e| LedgerMigrationEntry {
                multiaddr: e.multiaddr,
                peer_id: e.last_peer_id,
                // V040-T13 F1: the legacy flag was never trustworthy --
                // record_identified_peer marked every advertised listen
                // address as verified. Only operator-configured bootstrap
                // nodes are verified by fiat; everything else migrates
                // unverified and is re-proven by the first live dial.
                locally_verified: e.is_bootstrap,
                is_bootstrap: e.is_bootstrap,
                // Legacy clock was UNIX seconds; the core converter
                // multiplies to millis on import.
                first_seen: (e.first_seen != 0).then_some(e.first_seen),
                last_seen: (e.last_seen != 0).then_some(e.last_seen),
                observed_peer_ids: e.observed_peer_ids,
                label: e.label,
                // Re-verification happens on the first live connection.
                consecutive_failures: 0,
            })
            .collect();

        let result = self
            .core
            .import_legacy_cli_entries(rows, local_peer_id, my_addrs);

        let archived = archive_legacy_peers_json(&peers_path);
        if result.offered > 0 || result.imported > 0 || result.rejected > 0 {
            tracing::info!(
                "[INFO] legacy peers.json migration: {}/{} imported, {} rejected, archived={}",
                result.imported,
                result.offered,
                result.rejected,
                archived
            );
        }
        Ok(LegacyMigrationReport {
            offered: result.offered,
            imported: result.imported,
            rejected: result.rejected,
            archived,
        })
    }

    /// Extract address key string and optional PeerId for DialPolicyManager calls.
    pub fn key_to_policy_args(&self, key: &DialKey) -> (String, Option<PeerId>) {
        match key {
            DialKey::Peer(pid) => {
                let addr_key = if let Some(addr) = self.find_peer_multiaddr(&pid.to_string()) {
                    strip_peer_id(&addr)
                } else {
                    pid.to_string()
                };
                (addr_key, Some(*pid))
            }
            DialKey::Addr(addr) => {
                let pid = if let Some(idx) = addr.find("/p2p/") {
                    let remainder = &addr[idx + "/p2p/".len()..];
                    PeerId::from_str(remainder).ok()
                } else {
                    None
                };
                (strip_peer_id(addr), pid)
            }
        }
    }

    /// Record a failed connection attempt.
    ///
    /// Entry-side failure accounting now lives in the core store
    /// (`LedgerManager::record_failure`); this wrapper keeps the process
    /// dial-policy bookkeeping that is CLI-specific.
    pub fn record_failure(&mut self, multiaddr: &str) {
        let stripped = strip_peer_id(multiaddr);
        let parsed_pid = if let Some(idx) = multiaddr.find("/p2p/") {
            let remainder = &multiaddr[idx + "/p2p/".len()..];
            PeerId::from_str(remainder).ok()
        } else {
            None
        };
        self.dial_policy.record_dial_failure(&stripped, parsed_pid);
        self.core.record_failure(stripped.clone());
        tracing::warn!("[WARNING] Connection failed to {}", stripped);
    }

    /// Apply an Identify `PeerIdentified` event to the ledger.
    ///
    /// HEARSAY, never verified: `listen_addrs` is whatever the REMOTE peer
    /// chose to advertise. The core store records the identity at the address
    /// without setting `locally_verified` (a `/dns4/...` name or a remote's
    /// self-claim is not evidence). Returns the number of addresses actually
    /// recorded.
    pub fn record_identified_peer(&self, peer_id: &str, listen_addrs: &[String]) -> usize {
        self.core.record_identified_peer(peer_id, listen_addrs)
    }

    /// Record a CONFIRMED connection in the core store, marking the address
    /// locally verified. Mirrors the core swarm's own firing on outbound
    /// `ConnectionEstablished`; exposed so callers without a live swarm (and
    /// the gate tests) can promote a hearsay address to proven.
    pub fn record_connection(&self, multiaddr: &str, peer_id: &str) {
        self.core
            .record_connection(multiaddr.to_string(), peer_id.to_string());
    }

    /// Drop a peer's stale ledger addresses once a NEWER address is CONFIRMED.
    ///
    /// CONFIRMED connections only (called from the dial-success path, never
    /// from advertisements). Bootstrap entries are exempt so no peer can ever
    /// reap the seeded discovery roots. Returns the number of entries removed
    /// (the core store's eviction, but reported the same way this facade's
    /// callers expect).
    pub fn reap_stale_addresses_for_peer(&self, peer_id: &str, confirmed_addr: &str) -> usize {
        self.core
            .reap_stale_addresses_for_peer(peer_id, confirmed_addr)
    }

    /// Record a topic observed from a peer.
    pub fn record_topic(&self, multiaddr: &str, topic: &str) {
        self.core.record_topic(multiaddr, topic);
    }

    /// Get all addresses that should be dialed now, excluding the local node.
    ///
    /// Delegates to the core store's proven-and-alive candidate build
    /// (`success_count > 0`, under the dead threshold, self-filtered,
    /// node-reachable, prioritised). Per-key backoff is enforced separately
    /// by `try_begin_dial`, so the candidate list here needs no backoff gate.
    pub fn dialable_addresses(
        &self,
        local_peer_id: Option<&str>,
        my_addrs: &[String],
    ) -> Vec<(String, Option<String>)> {
        self.core
            .dialable_addresses_for_node(my_addrs, local_peer_id)
            .into_iter()
            .map(|e| (e.multiaddr, to_base58_peer_id(e.peer_id)))
            .collect()
    }

    /// Get all known topics from connected peers.
    pub fn all_known_topics(&self) -> Vec<String> {
        self.core.all_known_topics()
    }

    /// Resolve a PeerId to the transport multiaddr currently bound to it, if
    /// the core store knows the peer.
    pub fn find_peer_multiaddr(&self, peer_id: &str) -> Option<String> {
        self.core.find_by_peer_id(peer_id).map(|e| e.multiaddr)
    }

    /// Number of known peers carrying at least one gossipsub topic.
    pub fn entry_count_with_known_topics(&self) -> usize {
        self.core.entry_count_with_known_topics()
    }

    /// Total number of entries in the unified core store.
    pub fn entry_count(&self) -> usize {
        self.core.entry_count()
    }

    /// Merge wire-shared peer entries (ledger exchange) into the core store.
    /// Returns the number of NEW addresses added.
    pub fn merge_shared_entries(
        &self,
        entries: &[scmessenger_core::transport::SharedPeerEntry],
    ) -> usize {
        self.core.merge_shared_entries(entries)
    }

    /// A summary string for display.
    pub fn summary(&self) -> String {
        self.core.summary()
    }

    /// Add or update a bootstrap entry (operator configuration) in the core
    /// store. Skips a multiaddr that embeds this node's own PeerId.
    pub fn add_bootstrap(&mut self, multiaddr: &str, local_peer_id: Option<&str>) {
        if let Some(local) = local_peer_id {
            if multiaddr.contains(local) {
                return;
            }
        }
        self.core.add_bootstrap(multiaddr, None);
    }

    pub fn try_begin_dial(&mut self, key: DialKey, now: u64, relay_healthy: bool) -> bool {
        let is_circuit = Self::is_circuit_key(&key);
        let is_bootstrap = self.is_bootstrap_key(&key);

        if let Some(state) = self.peer_dial_states.get(&key) {
            if !state.ready(now) {
                return false;
            }
            if relay_healthy && !state.is_known_good && !is_circuit && !is_bootstrap {
                return false;
            }
        } else {
            let is_known_good = self.is_known_good_key(&key);
            if relay_healthy && !is_known_good && !is_circuit && !is_bootstrap {
                return false;
            }
        }

        // Enforce DialPolicyManager backoff and concurrent dial limits. This
        // is the first provisional claim; every early return below it must
        // release it via `complete_dial_attempt`.
        let (addr_key, pid_opt) = self.key_to_policy_args(&key);
        if !self.dial_policy.register_dial_attempt(&addr_key, pid_opt) {
            return false;
        }

        // Cap process-lifetime dial state at 4096 keys. Drop the entry
        // with the smallest next_attempt_after (least urgent) in a single
        // pass.
        if self.peer_dial_states.len() >= 4096 {
            if let Some(evict_key) = self
                .peer_dial_states
                .iter()
                .min_by_key(|(_, state)| state.next_attempt_after)
                .map(|(k, _)| k.clone())
            {
                self.peer_dial_states.remove(&evict_key);
            }
        }

        // Claim the peer-level slot (second provisional claim). Track
        // whether we mutated a pre-existing entry or inserted a fresh one,
        // so a rollback below restores exactly the prior state instead of
        // leaving a stale entry behind.
        let peer_slot_pre_existed = self.peer_dial_states.contains_key(&key);
        let is_known_good = self.is_known_good_key(&key);
        {
            let state = self
                .peer_dial_states
                .entry(key.clone())
                .or_insert_with(|| PeerDialState {
                    is_known_good,
                    ..Default::default()
                });
            state.in_flight = true;
        }

        // Claim the address-level slot -- the actual fix for the crash. Two
        // `DialKey::Peer` values for different (often stale) PeerIds can
        // resolve to the SAME `addr_key` above via `key_to_policy_args`,
        // which is exactly the fleet scenario that produced N simultaneous
        // connections to one host:port.
        let addr_already_in_flight = self
            .addr_dial_states
            .get(&addr_key)
            .is_some_and(|s| !s.ready(now));
        if addr_already_in_flight {
            // Release both provisional claims made above. Do not leak a
            // half-claim: the DialPolicyManager slot must be returned, and
            // the peer-level slot must go back to exactly what it was
            // before this call (removed if it did not exist, or left
            // `in_flight = false` if it did -- it could not have been
            // `in_flight = true` already, since the readiness check at the
            // top of this function would have returned `false` before we
            // ever reached here).
            self.dial_policy.complete_dial_attempt(&addr_key);
            if peer_slot_pre_existed {
                if let Some(state) = self.peer_dial_states.get_mut(&key) {
                    state.in_flight = false;
                }
            } else {
                self.peer_dial_states.remove(&key);
            }
            return false;
        }

        // Cap process-lifetime address dial state at 4096 keys too, mirroring
        // the peer_dial_states eviction above (least-recently-claimed entry).
        if self.addr_dial_states.len() >= 4096 {
            if let Some(evict_key) = self
                .addr_dial_states
                .iter()
                .min_by_key(|(_, state)| state.claimed_at)
                .map(|(k, _)| k.clone())
            {
                self.addr_dial_states.remove(&evict_key);
            }
        }

        let addr_state = self.addr_dial_states.entry(addr_key).or_default();
        addr_state.in_flight = true;
        addr_state.claimed_at = now;

        true
    }

    /// Record the outcome of a dial previously started with `try_begin_dial`.
    ///
    /// Releases BOTH slots claimed by `try_begin_dial` on every path through
    /// this function -- success and failure alike -- because the
    /// address-level release happens once, unconditionally, before the
    /// success/failure branch below.
    pub fn complete_dial(
        &mut self,
        key: &DialKey,
        success: bool,
        now: u64,
        learned_peer_id: Option<PeerId>,
    ) {
        let (addr_key, pid_opt) = self.key_to_policy_args(key);
        self.dial_policy.complete_dial_attempt(&addr_key);

        // Release the address-level slot claimed in try_begin_dial. Runs on
        // every path through this function (see doc comment above).
        if let Some(addr_state) = self.addr_dial_states.get_mut(&addr_key) {
            addr_state.in_flight = false;
        }

        if success {
            let target_pid = learned_peer_id.or(pid_opt);
            self.dial_policy
                .reset_on_connection_established(&addr_key, target_pid);
            if let Some(pid) = target_pid {
                self.dial_policy.reset_peer_backoff(pid);
            }

            // P0 stale-address reaping (2026-08-12): this is a CONFIRMED
            // connection to `addr_key`, so drop this peer's other ledger
            // addresses now. Runs before the early-return below so both
            // success exits get it. Advertisements never reach this path.
            if let Some(pid) = target_pid {
                self.reap_stale_addresses_for_peer(&pid.to_string(), &addr_key);
            }

            let mut state = self.peer_dial_states.remove(key).unwrap_or_default();
            state.record_success();

            if let DialKey::Addr(_) = key {
                if let Some(pid) = learned_peer_id {
                    let peer_key = DialKey::Peer(pid);
                    let peer_state = self.peer_dial_states.entry(peer_key).or_insert(state);
                    peer_state.connections = peer_state.connections.saturating_add(1);
                    peer_state.is_known_good = true;
                    return;
                }
            }

            // Count the established connection against this key's slot. For
            // Peer keys this is the release target of `record_disconnect`
            // (fired per dropped connection). For Addr keys with no learned
            // peer id the counter stays 1 until eviction -- acceptable: a
            // successful connection records the peer id into the ledger, so
            // all future scheduler dials of this address key on `DialKey::Peer`
            // instead of the Addr key.
            state.connections = state.connections.saturating_add(1);
            self.peer_dial_states.insert(key.clone(), state);
        } else {
            self.dial_policy.record_dial_failure(&addr_key, pid_opt);
            if let Some(state) = self.peer_dial_states.get_mut(key) {
                state.record_failure(now);
            }
        }
    }

    /// Record a disconnected peer, releasing its concurrent connection slot.
    ///
    /// Fired from the `SwarmEvent::PeerDisconnected` handler (one event per
    /// dropped connection -- see core/src/transport/swarm.rs), which keeps the
    /// `connections` counter incremented in `complete_dial` balanced.
    /// Saturating subtraction means a missed or extra release can never
    /// underflow; the worst case is a stale slot that a subsequent successful
    /// dial repairs via `complete_dial`.
    pub fn record_disconnect(&mut self, peer_id: PeerId) {
        let key = DialKey::Peer(peer_id);
        if let Some(state) = self.peer_dial_states.get_mut(&key) {
            state.connections = state.connections.saturating_sub(1);
        }
    }

    /// Borrow a tracked dial state, if any.
    pub fn dial_state(&self, key: &DialKey) -> Option<&PeerDialState> {
        self.peer_dial_states.get(key)
    }

    fn is_circuit_key(key: &DialKey) -> bool {
        matches!(key, DialKey::Addr(addr) if addr.contains("/p2p-circuit"))
    }

    fn is_bootstrap_key(&self, key: &DialKey) -> bool {
        match key {
            DialKey::Peer(pid) => self
                .core
                .find_by_peer_id(&pid.to_string())
                .is_some_and(|e| e.is_bootstrap),
            DialKey::Addr(addr) => self
                .core
                .entry_for_multiaddr(addr)
                .is_some_and(|e| e.is_bootstrap),
        }
    }

    // Review triage (qwen3.8-max-0902, finding 4): this bar is intentionally
    // STRICTER than the DHT disclosure gate (`failure_count <
    // LEDGER_DEAD_FAILURE_THRESHOLD`). The dial scheduler conservatively
    // avoids peers with ANY recorded failure; the gate's < 3 expresses
    // publishability-until-dead, a different decision. Documented, not
    // aligned -- widening the dial set to include flaky peers has no benefit.
    fn is_known_good_key(&self, key: &DialKey) -> bool {
        match key {
            DialKey::Peer(pid) => self
                .core
                .find_by_peer_id(&pid.to_string())
                .is_some_and(|e| e.locally_verified && e.peer_id.is_some() && e.failure_count == 0),
            DialKey::Addr(addr) => self
                .core
                .entry_for_multiaddr(addr)
                .is_some_and(|e| e.locally_verified && e.peer_id.is_some() && e.failure_count == 0),
        }
    }
}

/// Report from [`ConnectionLedger::run_legacy_migration`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LegacyMigrationReport {
    /// Entries present in the legacy `peers.json`.
    pub offered: usize,
    /// Entries the core store accepted.
    pub imported: usize,
    /// Entries rejected by the node-dialability/port/self filters.
    pub rejected: usize,
    /// Whether the legacy file was renamed aside (idempotence marker).
    pub archived: bool,
}

/// Legacy `peers.json` envelope (CLI writes predating the unification).
#[derive(Debug, Deserialize)]
struct LegacyLedgerFile {
    #[serde(default)]
    entries: HashMap<String, LegacyLedgerEntry>,
    #[serde(default)]
    version: u32,
    #[serde(default)]
    last_saved: u64,
}

/// A legacy CLI ledger entry. Only the fields the migration consumes; the
/// backoff bookkeeping (`consecutive_failures` etc.) is re-derived live.
#[derive(Debug, Deserialize)]
struct LegacyLedgerEntry {
    multiaddr: String,
    #[serde(default)]
    last_peer_id: Option<String>,
    #[serde(default)]
    observed_peer_ids: Vec<String>,
    #[serde(default)]
    last_seen: u64,
    #[serde(default)]
    first_seen: u64,
    #[serde(default)]
    locally_verified: bool,
    #[serde(default)]
    is_bootstrap: bool,
    #[serde(default)]
    label: Option<String>,
}

/// Rename the legacy file aside so a crash or restart cannot re-import it.
fn archive_legacy_peers_json(peers_path: &Path) -> bool {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let archived = peers_path.with_file_name(format!("peers.json.migrated-{}", ts));
    match std::fs::rename(peers_path, &archived) {
        Ok(()) => {
            tracing::info!(
                "[INFO] archived legacy peers.json to {}",
                archived.display()
            );
            true
        }
        Err(e) => {
            tracing::warn!(
                "[WARNING] could not archive legacy peers.json ({}); will retry next start",
                e
            );
            false
        }
    }
}

// Address filtering lives in the core crate now (adversarial review F3): core's
// ledger-seed import, its seed-dial candidate build and its ledger-exchange
// response all need the same rules, and having two definitions of "dialable" in
// one workspace is how core ended up with none. These re-exports keep every
// existing `ledger::is_dialable_multiaddr` / `ledger::NetworkMode` call site
// working unchanged.
pub use scmessenger_core::transport::addr_filter::{
    is_dialable_multiaddr, is_self_address, strip_peer_id, DnsPolicy, NetworkMode,
};

/// Extract the first `/ip4/x.x.x.x/` component of a multiaddr, if any.
fn extract_ipv4(multiaddr: &str) -> Option<std::net::Ipv4Addr> {
    let parts: Vec<&str> = multiaddr.split('/').collect();
    for i in 0..parts.len() {
        if parts[i] == "ip4" && i + 1 < parts.len() {
            if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv4Addr>() {
                return Some(ip);
            }
        }
    }
    None
}

/// Which RFC1918 private-address class an IPv4 address falls in, if any.
/// `None` means the address is not a private (RFC1918) address at all.
fn rfc1918_class(ip: &std::net::Ipv4Addr) -> Option<u8> {
    let o = ip.octets();
    if o[0] == 10 {
        Some(0) // 10.0.0.0/8
    } else if o[0] == 172 && (16..=31).contains(&o[1]) {
        Some(1) // 172.16.0.0/12
    } else if o[0] == 192 && o[1] == 168 {
        Some(2) // 192.168.0.0/16
    } else {
        None
    }
}

/// Returns true iff `candidate` is worth dialing given this node's own known
/// addresses: rejects self-dials outright, and (in `NetworkMode::Local`)
/// rejects a private-range (RFC1918) address unless this node itself holds
/// an address in the SAME private-range class -- e.g. a node on
/// `192.168.0.121` should not promiscuously dial an advertised
/// `10.0.2.16` (a different private class it has no route to), but should
/// still dial other `192.168.x.x` peers on its own LAN. This does not
/// replace `is_dialable_multiaddr` -- callers should still apply that
/// filter first (it rejects unconditionally-unroutable things like
/// loopback/link-local); this is an additional, node-aware layer on top.
pub fn is_dialable_for_this_node(multiaddr: &str, mode: NetworkMode, my_addrs: &[String]) -> bool {
    if is_self_address(multiaddr, my_addrs) {
        return false;
    }
    // A /p2p-circuit address's leading /ip4/.../ component is the RELAY
    // hop's address, not the final target peer's -- applying RFC1918
    // class-awareness to the relay's own address would incorrectly reject
    // the only path to a NAT'd peer whenever the relay's IP happens to
    // differ in private-range class from this node's own address. Mirrors
    // the same unconditional-allow exemption is_dialable_multiaddr already
    // gives circuit addresses.
    if multiaddr.contains("/p2p-circuit") {
        return true;
    }
    // Port-stripped self-dial gap (P1 follow-up, 2026-08-12): is_self_address
    // compares stripped strings EXACTLY, so a candidate carrying this node's
    // own IP with NO port component ("/ip4/192.168.0.121") does not equal
    // "/ip4/192.168.0.121/tcp/9001" and slips past the check above -- and
    // is_dialable_multiaddr still marks a bare /ip4/ component as having a
    // transport, so such a candidate reaches the dialer. A portless candidate
    // aimed at our own IP is either a self-dial intent or a malformed ledger
    // entry; neither is worth a dial slot. Candidates on our IP that DO carry
    // a port are left alone: a different port on the same host is a
    // legitimate co-located node, and the exact-match check above already
    // catches our own port. Placed after the circuit exemption so circuit
    // addresses keep their unconditional-allow semantics.
    if let Some(candidate_ip) = extract_ipv4(multiaddr) {
        let has_transport_port = multiaddr.contains("/tcp/") || multiaddr.contains("/udp/");
        if !has_transport_port
            && my_addrs
                .iter()
                .filter_map(|a| extract_ipv4(a))
                .any(|ip| ip == candidate_ip)
        {
            return false;
        }
    }
    if mode == NetworkMode::Local {
        if let Some(candidate_ip) = extract_ipv4(multiaddr) {
            if let Some(candidate_class) = rfc1918_class(&candidate_ip) {
                let my_ipv4s: Vec<std::net::Ipv4Addr> =
                    my_addrs.iter().filter_map(|a| extract_ipv4(a)).collect();
                let on_same_range = my_ipv4s
                    .iter()
                    .any(|m| rfc1918_class(m) == Some(candidate_class));
                if !on_same_range {
                    return false;
                }
            }
        }
    }
    true
}

/// Returns true when a multiaddr contains the local PeerId in any `/p2p/`
/// component. This catches self-targeted and self-relayed circuit paths that
/// cannot be detected by comparing the transport socket alone.
pub fn contains_peer_id_component(multiaddr: &str, peer_id: &str) -> bool {
    let Ok(local_peer_id) = peer_id.parse::<PeerId>() else {
        return false;
    };
    let Ok(addr) = multiaddr.parse::<Multiaddr>() else {
        return false;
    };

    addr.iter().any(|protocol| {
        matches!(protocol, libp2p::multiaddr::Protocol::P2p(candidate) if candidate == local_peer_id)
    })
}

/// Prefer directly useful local candidates without discarding global
/// fallbacks. Phones often advertise carrier IPv6 addresses alongside their
/// Wi-Fi address; those global addresses can consume the dial budget before a
/// same-LAN path is attempted.
pub fn prioritize_dial_candidates(
    mut candidates: Vec<(String, Option<String>)>,
) -> Vec<(String, Option<String>)> {
    candidates.sort_by_key(|(multiaddr, _)| {
        let priority = multiaddr
            .parse::<libp2p::Multiaddr>()
            .ok()
            .and_then(|addr| {
                addr.iter().find_map(|protocol| match protocol {
                    libp2p::multiaddr::Protocol::Ip4(ip) => {
                        Some(if ip.is_private() || is_cgnat(&ip) {
                            0u8
                        } else {
                            1u8
                        })
                    }
                    libp2p::multiaddr::Protocol::Ip6(ip) => {
                        Some(if is_ula(&ip) { 0u8 } else { 2u8 })
                    }
                    _ => None,
                })
            })
            .unwrap_or(3);
        (priority, multiaddr.clone())
    });
    candidates
}

fn is_cgnat(ip: &std::net::Ipv4Addr) -> bool {
    let value = u32::from_be_bytes(ip.octets());
    (u32::from_be_bytes([100, 64, 0, 0])..=u32::from_be_bytes([100, 127, 255, 255]))
        .contains(&value)
}

fn is_ula(ip: &std::net::Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

/// Extract IP:Port from a multiaddr string for human-readable display
pub fn extract_ip_port(multiaddr: &str) -> Option<String> {
    // Parse /ip4/1.2.3.4/tcp/9001 -> 1.2.3.4:9001
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut ip = None;
    let mut port = None;

    for i in 0..parts.len() {
        if (parts[i] == "ip4" || parts[i] == "ip6") && i + 1 < parts.len() {
            ip = Some(parts[i + 1]);
        }
        if (parts[i] == "tcp" || parts[i] == "udp") && i + 1 < parts.len() {
            port = Some(parts[i + 1]);
        }
    }

    match (ip, port) {
        (Some(ip), Some(port)) => Some(format!("{}:{}", ip, port)),
        _ => None,
    }
}

/// The store keeps identities in canonical hex (a completed connection writes
/// the canonical form); the CLI's dial scheduler and address-reflection
/// callers parse libp2p base58 `PeerId`s. Convert canonical hex back to
/// base58, passing through anything that is not canonical hex (e.g. legacy
/// base58 identifiers imported from a pre-unification `peers.json`).
pub fn to_base58_peer_id(stored: Option<String>) -> Option<String> {
    let stored = stored?;
    if stored.len() == 64 && stored.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Some(base58) = scmessenger_core::store::peer_id_from_public_key_hex(&stored) {
            return Some(base58);
        }
    }
    Some(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_peer_id() -> String {
        PeerId::random().to_string()
    }

    fn ledger() -> ConnectionLedger {
        ConnectionLedger::new(LedgerManager::ephemeral())
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    #[test]
    fn test_strip_peer_id() {
        let pid = test_peer_id();
        let addr = format!("/ip4/192.168.0.5/tcp/9001/p2p/{}", pid);
        assert_eq!(strip_peer_id(&addr), "/ip4/192.168.0.5/tcp/9001");
    }

    #[test]
    fn test_strip_peer_id_preserves_circuit_path() {
        let target = PeerId::random();
        let relay = PeerId::random();
        let addr = format!(
            "/ip4/198.51.100.1/tcp/443/p2p/{}/p2p-circuit/p2p/{}",
            relay, target
        );
        // The relay hop (before the circuit) is routing information and stays;
        // only the explicit target identity after the circuit is stripped.
        let stripped = strip_peer_id(&addr);
        assert!(stripped.contains(&relay.to_string()));
        assert!(!stripped.contains(&target.to_string()));
        assert!(stripped.contains("/p2p-circuit"));
    }

    #[test]
    fn test_extract_ip_port() {
        assert_eq!(
            extract_ip_port("/ip4/1.2.3.4/tcp/9001"),
            Some("1.2.3.4:9001".to_string())
        );
        // Display format; brackets are a rendering concern, not this helper's.
        assert_eq!(
            extract_ip_port("/ip6/::1/tcp/9002"),
            Some("::1:9002".to_string())
        );
        assert_eq!(extract_ip_port("/p2p-circuit/p2p/x"), None);
    }

    #[test]
    fn test_dial_key_for_target() {
        let pid = PeerId::random();
        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", Some(pid));
        assert!(matches!(key, DialKey::Peer(p) if p == pid));

        let key = DialKey::for_target(&format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", pid), None);
        assert!(matches!(key, DialKey::Peer(p) if p == pid));

        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", None);
        assert!(matches!(key, DialKey::Addr(a) if a == "/ip4/1.2.3.4/tcp/9001"));
    }

    #[test]
    fn test_peer_dial_state_backoff_ladder() {
        let mut state = PeerDialState::default();
        let t = 1_000_000u64;
        // Rungs: 5s, 30s, 2m, 5m, then 30m cap.
        for rung in [5u64, 30, 120, 300, 1800, 1800] {
            state.record_failure(t);
            assert_eq!(state.next_attempt_after, t + rung, "rung {rung}");
        }
        // The ladder is capped: further failures stay at 30m.
        state.record_failure(t);
        assert_eq!(state.next_attempt_after, t + 1800);
    }

    #[test]
    fn test_peer_dial_state_success_reset() {
        let mut state = PeerDialState::default();
        state.record_failure(1_000_000);
        state.in_flight = true;
        state.record_success();
        assert_eq!(state.consecutive_failures, 0);
        assert!(!state.in_flight);
        assert!(state.is_known_good);
        assert!(state.ready(0));
    }

    #[test]
    fn test_try_begin_dial_blocks_in_flight_reuse() {
        let mut l = ledger();
        let pid = PeerId::random();
        let key = DialKey::Peer(pid);
        assert!(l.try_begin_dial(key.clone(), now(), false));
        // Second dial with an in-flight claim is refused even without relay.
        assert!(!l.try_begin_dial(key, now(), false));
        // But a different address is fine.
        let other = DialKey::Addr("/ip4/10.0.0.8/tcp/9001".to_string());
        assert!(l.try_begin_dial(other, now(), false));
    }

    #[test]
    fn test_try_begin_dial_suppresses_unknown_when_relay_healthy() {
        let mut l = ledger();
        let pid = PeerId::random();
        let key = DialKey::Peer(pid);
        assert!(!l.try_begin_dial(key, now(), true));
    }

    #[test]
    fn test_try_begin_dial_allows_circuit_when_relay_healthy() {
        let mut l = ledger();
        let key = DialKey::Addr("/ip4/198.51.100.1/tcp/443/p2p/x/p2p-circuit/p2p/y".to_string());
        assert!(l.try_begin_dial(key, now(), true));
    }

    #[test]
    fn test_try_begin_dial_allows_bootstrap_when_relay_healthy() {
        let mut l = ledger();
        l.core
            .add_bootstrap("/ip4/198.51.100.10/tcp/9001", Some("Bootstrap 1"));
        let key = DialKey::Addr("/ip4/198.51.100.10/tcp/9001".to_string());
        assert!(l.try_begin_dial(key, now(), true));
    }

    #[test]
    fn test_try_begin_dial_allows_known_good_when_relay_healthy() {
        let mut l = ledger();
        let pid = PeerId::random();
        // A verified connection in the core store classifies the key known-good.
        l.core
            .record_connection("/ip4/198.51.100.20/tcp/9001".to_string(), pid.to_string());
        let key = DialKey::Peer(pid);
        assert!(l.try_begin_dial(key, now(), true));
    }

    #[test]
    fn test_complete_dial_failure_enforces_backoff() {
        let mut l = ledger();
        let pid = PeerId::random();
        let key = DialKey::Peer(pid);
        let t = now();
        assert!(l.try_begin_dial(key.clone(), t, false));
        l.complete_dial(&key, false, t, None);
        let state = l.peer_dial_states.get(&key).expect("state kept");
        assert!(state.next_attempt_after > t);
        assert!(!state.in_flight);
    }

    #[test]
    fn test_complete_dial_migrates_addr_to_peer() {
        let mut l = ledger();
        let learned = PeerId::random();
        let key = DialKey::Addr("/ip4/198.51.100.30/tcp/9001".to_string());
        let t = now();
        assert!(l.try_begin_dial(key.clone(), t, false));
        l.complete_dial(&key, true, t, Some(learned));
        // The learned peer now owns the connection slot.
        let peer_state = l
            .peer_dial_states
            .get(&DialKey::Peer(learned))
            .expect("peer state created");
        assert_eq!(peer_state.connections, 1);
    }

    #[test]
    fn test_try_begin_dial_blocks_same_address_different_peer_ids() {
        let mut l = ledger();
        let addr = "/ip4/198.51.100.40/tcp/9001";
        let pid_a = PeerId::random();
        let pid_b = PeerId::random();
        // `a` is the proven incumbent; `b` is a newer claimant via Identify
        // (hearsay). Both identities resolve to the SAME address through the
        // shared core store -- the P0 stale-identity fleet scenario.
        l.core
            .record_connection(addr.to_string(), pid_a.to_string());
        l.core
            .record_identified_peer(&pid_b.to_string(), &[addr.to_string()]);
        assert_eq!(
            l.find_peer_multiaddr(&pid_a.to_string()).as_deref(),
            Some(addr)
        );
        assert_eq!(
            l.find_peer_multiaddr(&pid_b.to_string()).as_deref(),
            Some(addr)
        );

        let ka = DialKey::Peer(pid_a);
        let kb = DialKey::Peer(pid_b);
        let t = now();
        assert!(l.try_begin_dial(ka.clone(), t, false));
        // Same host:port under a different key is refused while in flight.
        assert!(!l.try_begin_dial(kb.clone(), t, false));
        // Success on `ka` releases the shared address slot AND resets the
        // dial-policy backoff, so the other stale identity can now claim it.
        l.complete_dial(&ka, true, t, Some(pid_a));
        assert!(l.try_begin_dial(kb, t + 1, false));
    }

    #[test]
    fn test_try_begin_dial_address_guard_releases_after_complete_dial() {
        let mut l = ledger();
        let key = DialKey::Addr("/ip4/198.51.100.50/tcp/9001".to_string());
        let t = now();
        assert!(l.try_begin_dial(key.clone(), t, false));
        assert!(!l.try_begin_dial(key.clone(), t, false));
        assert!(
            l.addr_dial_states["/ip4/198.51.100.50/tcp/9001"].in_flight,
            "address slot claimed while a dial is in flight"
        );
        l.complete_dial(&key, false, t, None);
        // The address-level claim is released on EVERY completion path,
        // success and failure alike. (Missed release = wedge-forever; this is
        // the exact regression the guard comment warns about.) The dial
        // policy's own wall-clock backoff still gates a re-dial, which is
        // enforced separately and is not what this test pins.
        assert!(
            !l.addr_dial_states["/ip4/198.51.100.50/tcp/9001"].in_flight,
            "address slot stuck after complete_dial"
        );
    }

    #[test]
    fn test_try_begin_dial_allows_concurrent_dials_to_different_addresses() {
        let mut l = ledger();
        let ka = DialKey::Addr("/ip4/198.51.100.60/tcp/9001".to_string());
        let kb = DialKey::Addr("/ip4/198.51.100.61/tcp/9001".to_string());
        let t = now();
        assert!(l.try_begin_dial(ka, t, false));
        assert!(l.try_begin_dial(kb, t, false));
    }

    #[test]
    fn test_peer_dial_states_eviction_caps_at_4096() {
        let mut l = ledger();
        for i in 0..5000u32 {
            let key = DialKey::Addr(format!("/ip4/198.51.100.1/tcp/{}", 1000 + i));
            assert!(l.try_begin_dial(key, now(), false));
        }
        assert!(l.peer_dial_states.len() <= 4096);
        assert!(l.addr_dial_states.len() <= 4096);
    }

    #[test]
    fn test_identify_is_hearsay_never_verified() {
        let mut l = ledger();
        let pid = PeerId::random().to_string();
        let addrs = vec!["/ip4/198.51.100.70/tcp/9001".to_string()];
        let recorded = l.record_identified_peer(&pid, &addrs);
        assert_eq!(recorded, 1);
        let entry = l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.70/tcp/9001")
            .expect("advertised address recorded");
        assert!(!entry.locally_verified, "advertisement must not verify");
        // And a hearsay-only peer never appears among dial candidates.
        assert!(l.dialable_addresses(None, &[]).is_empty());
    }

    #[test]
    fn test_recorded_connection_is_dialable_candidate() {
        let mut l = ledger();
        let pid = PeerId::random();
        l.core
            .record_connection("/ip4/198.51.100.80/tcp/9001".to_string(), pid.to_string());
        let addrs = l.dialable_addresses(None, &[]);
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].0, "/ip4/198.51.100.80/tcp/9001");
        assert_eq!(addrs[0].1.as_deref(), Some(pid.to_string().as_str()));
    }

    #[test]
    fn test_dialable_addresses_excludes_self_and_hearsay() {
        let mut l = ledger();
        let me = PeerId::random();
        let other = PeerId::random();
        // A success on the peer's address + our own address in the store.
        l.core
            .record_connection("/ip4/198.51.100.90/tcp/9001".to_string(), other.to_string());
        l.core
            .record_connection("/ip4/198.51.100.91/tcp/9001".to_string(), me.to_string());
        let addrs = l.dialable_addresses(
            Some(&me.to_string()),
            &["/ip4/198.51.100.91/tcp/9001".to_string()],
        );
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].0, "/ip4/198.51.100.90/tcp/9001");
    }

    #[test]
    fn test_merge_shared_entries_and_topics_flow() {
        let mut l = ledger();
        let pid = PeerId::random();
        l.core
            .record_connection("/ip4/198.51.100.100/tcp/9001".to_string(), pid.to_string());
        let shared = scmessenger_core::transport::SharedPeerEntry {
            multiaddr: "/ip4/198.51.100.100/tcp/9001".to_string(),
            last_peer_id: Some(pid.to_string()),
            last_seen: now(),
            known_topics: vec!["scm/test".to_string()],
        };
        let added = l.merge_shared_entries(std::slice::from_ref(&shared));
        assert_eq!(added, 0, "known address updates rather than adds");
        l.record_topic("/ip4/198.51.100.100/tcp/9001", "scm/test");
        assert_eq!(l.entry_count_with_known_topics(), 1);
        assert!(l.all_known_topics().contains(&"scm/test".to_string()));
        assert!(l.summary().contains("1"));
    }

    #[test]
    fn test_find_peer_multiaddr_resolves_core_entry() {
        let mut l = ledger();
        let pid = PeerId::random();
        l.core
            .record_connection("/ip4/198.51.100.110/tcp/9001".to_string(), pid.to_string());
        assert_eq!(
            l.find_peer_multiaddr(&pid.to_string()).as_deref(),
            Some("/ip4/198.51.100.110/tcp/9001")
        );
        assert_eq!(l.find_peer_multiaddr(&PeerId::random().to_string()), None);
    }

    #[test]
    fn test_stale_address_reaping_via_wrapper() {
        let mut l = ledger();
        let pid = PeerId::random().to_string();
        l.core
            .record_connection("/ip4/198.51.100.120/tcp/9001".to_string(), pid.clone());
        l.core
            .record_connection("/ip4/198.51.100.120/tcp/9002".to_string(), pid.clone());
        // Confirmed connection on .120:9001 reaps the other address.
        let removed = l.reap_stale_addresses_for_peer(&pid, "/ip4/198.51.100.120/tcp/9001");
        assert_eq!(removed, 1);
        assert!(l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.120/tcp/9002")
            .is_none());
        assert!(l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.120/tcp/9001")
            .is_some());
    }

    #[test]
    fn test_legacy_migration_strips_untrusted_verified_flag_and_archives() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pid_verified = PeerId::random().to_string();
        let pid_hearsay = PeerId::random().to_string();
        let pid_bootstrap = PeerId::random().to_string();
        let legacy = serde_json::json!({
            "version": 1,
            "last_saved": 1700000000,
            "entries": {
                "/ip4/198.51.100.130/tcp/9001": {
                    "address": "198.51.100.130:9001",
                    "multiaddr": "/ip4/198.51.100.130/tcp/9001",
                    "last_peer_id": pid_verified,
                    "observed_peer_ids": [pid_verified],
                    "last_seen": 1700000000,
                    "first_seen": 1699999000,
                    "consecutive_failures": 0,
                    "backoff_seconds": 0,
                    "next_attempt_after": 0,
                    "locally_verified": true,
                    "is_bootstrap": false,
                    "known_topics": ["scm/legacy"],
                    "label": null
                },
                // Junk port: the migration filter must drop it.
                "/ip4/198.51.100.131/tcp/49152": {
                    "address": "198.51.100.131:49152",
                    "multiaddr": "/ip4/198.51.100.131/tcp/49152",
                    "last_peer_id": pid_hearsay,
                    "observed_peer_ids": [pid_hearsay],
                    "last_seen": 1700000100,
                    "first_seen": 1699999100,
                    "consecutive_failures": 4,
                    "backoff_seconds": 300,
                    "next_attempt_after": 1700000300,
                    "locally_verified": false,
                    "is_bootstrap": false,
                    "known_topics": [],
                    "label": null
                },
                // Operator-configured bootstrap: verified by fiat even though
                // the legacy flag says otherwise (F1: `is_bootstrap` is the
                // only trusted input).
                "/ip4/198.51.100.155/tcp/9001": {
                    "address": "198.51.100.155:9001",
                    "multiaddr": "/ip4/198.51.100.155/tcp/9001",
                    "last_peer_id": pid_bootstrap,
                    "observed_peer_ids": [pid_bootstrap],
                    "last_seen": 1700000000,
                    "first_seen": 1699999000,
                    "consecutive_failures": 0,
                    "backoff_seconds": 0,
                    "next_attempt_after": 0,
                    "locally_verified": false,
                    "is_bootstrap": true,
                    "known_topics": [],
                    "label": null
                }
            }
        });
        std::fs::write(
            dir.path().join("peers.json"),
            serde_json::to_string_pretty(&legacy).unwrap(),
        )
        .expect("write peers.json");

        let mut l = ledger();
        let report = l
            .run_legacy_migration(dir.path(), None, &[])
            .expect("migration runs");
        assert_eq!(report.offered, 3);
        assert_eq!(report.imported, 2);
        assert_eq!(report.rejected, 1);
        assert!(report.archived);

        // V040-T13 F1: the legacy `locally_verified` flag was never
        // trustworthy -- `record_identified_peer` marked every advertised
        // listen address as verified. The entry migrates UNVERIFIED and is
        // re-proven by the first live dial.
        let entry = l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.130/tcp/9001")
            .expect("legacy entry imported");
        assert!(
            !entry.locally_verified,
            "legacy verified flag is not trusted"
        );
        assert!(!entry.is_bootstrap);
        assert_eq!(entry.peer_id.as_deref(), Some(pid_verified.as_str()));

        // Operator bootstrap survives as verified by fiat.
        let bootstrap = l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.155/tcp/9001")
            .expect("bootstrap legacy entry imported");
        assert!(bootstrap.locally_verified);
        assert!(bootstrap.is_bootstrap);

        // Idempotence: file gone -> nothing offered on a second run.
        let report2 = l
            .run_legacy_migration(dir.path(), None, &[])
            .expect("second run");
        assert_eq!(report2.offered, 0);
        assert!(std::fs::read_dir(dir.path())
            .expect("read dir")
            .any(|e| e.as_ref().is_ok_and(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with("peers.json.migrated-"))));
    }

    #[test]
    fn test_legacy_migration_skips_self_and_ephemeral() {
        let dir = tempfile::tempdir().expect("tempdir");
        let me = PeerId::random().to_string();
        let legacy = serde_json::json!({
            "entries": {
                "/ip4/198.51.100.140/tcp/9001/p2p/".to_string() + &me: {
                    "multiaddr": "/ip4/198.51.100.140/tcp/9001/p2p/".to_string() + &me,
                    "last_peer_id": me,
                    "observed_peer_ids": [],
                    "last_seen": 1700000000,
                    "first_seen": 1699999000,
                    "locally_verified": true,
                    "is_bootstrap": false,
                    "label": null
                },
                // Peer on our own private-range class.
                "10.0.0.9/tcp/9001": {
                    "multiaddr": "/ip4/10.0.0.9/tcp/9001",
                    "last_peer_id": PeerId::random().to_string(),
                    "observed_peer_ids": [],
                    "last_seen": 1700000000,
                    "first_seen": 1699999000,
                    "locally_verified": true,
                    "is_bootstrap": false,
                    "label": null
                }
            }
        });
        std::fs::write(
            dir.path().join("peers.json"),
            serde_json::to_string_pretty(&legacy).unwrap(),
        )
        .expect("write peers.json");

        let mut l = ledger();
        let report = l
            .run_legacy_migration(
                dir.path(),
                Some(&me),
                &["/ip4/10.0.0.5/tcp/9001".to_string()],
            )
            .expect("migration runs");
        // Self entry rejected by identity; same-class LAN peer accepted.
        assert_eq!(report.imported, 1);
        assert_eq!(report.rejected, 1);
    }

    #[test]
    fn test_add_bootstrap_marks_verified_and_skips_self() {
        let mut l = ledger();
        let me = PeerId::random().to_string();
        l.add_bootstrap(
            &format!("/ip4/198.51.100.150/tcp/9001/p2p/{}", me),
            Some(&me),
        );
        assert!(l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.150/tcp/9001")
            .is_none());

        l.add_bootstrap("/ip4/198.51.100.160/tcp/9001", None);
        let entry = l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.160/tcp/9001")
            .expect("bootstrap recorded");
        assert!(entry.is_bootstrap);
        assert!(entry.locally_verified);
    }

    #[test]
    fn test_record_failure_bookkeeping_still_runs() {
        let mut l = ledger();
        let pid = PeerId::random();
        l.core
            .record_connection("/ip4/198.51.100.170/tcp/9001".to_string(), pid.to_string());
        l.record_failure(&format!("/ip4/198.51.100.170/tcp/9001/p2p/{}", pid));
        let entry = l
            .core
            .entry_for_multiaddr("/ip4/198.51.100.170/tcp/9001")
            .expect("entry kept");
        assert_eq!(entry.failure_count, 1);
        // Dial-policy backoff ran too: a same-address dial attempt right now
        // is refused by register_dial_attempt.
        let key = DialKey::Addr("/ip4/198.51.100.170/tcp/9001".to_string());
        let t = now();
        assert!(!l.try_begin_dial(key, t, false));
    }
}
