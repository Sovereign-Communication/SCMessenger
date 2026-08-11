// Connection Ledger — Persistent peer discovery storage
//
// Philosophy: "A node is a node." IP is the source of truth.
//
// The ledger stores every successful IP:Port pair we've connected to.
// On startup, we load the ledger and attempt to reconnect to all known peers.
// If a peer presents a different PeerID (e.g., after restart), we accept it,
// update the ledger, and carry on. Unreachable peers enter exponential backoff
// but are never deleted — they may come back.

use anyhow::{Context, Result};
use libp2p::{Multiaddr, PeerId};
use scmessenger_core::transport::dial_policy::DialPolicyManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single entry in the connection ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// The IP:Port address (source of truth)
    pub address: String,

    /// The multiaddr we used to dial (without /p2p/ suffix)
    pub multiaddr: String,

    /// Last observed PeerID at this address (may change on restart)
    pub last_peer_id: Option<String>,

    /// All PeerIDs ever observed at this address
    pub observed_peer_ids: Vec<String>,

    /// Unix timestamp of last successful connection
    pub last_seen: u64,

    /// Unix timestamp of first discovery
    pub first_seen: u64,

    /// Number of consecutive failed connection attempts
    pub consecutive_failures: u32,

    /// Current backoff delay in seconds (doubles on each failure)
    pub backoff_seconds: u64,

    /// Unix timestamp of when we can next attempt connection
    pub next_attempt_after: u64,

    /// Whether this node has personally verified the address (successful
    /// local connection, or operator-trusted bootstrap). Defaults to false for
    /// entries loaded from disk that predate this field, so old peers.json
    /// entries classify as unknown until re-verified locally.
    #[serde(default)]
    pub locally_verified: bool,

    /// Whether this is a hardcoded bootstrap node (never remove)
    pub is_bootstrap: bool,

    /// Gossipsub topics this peer was subscribed to
    pub known_topics: Vec<String>,

    /// Human-readable label (e.g., "GCP Primary", "Community Relay")
    pub label: Option<String>,
}

impl LedgerEntry {
    /// Create a new entry for a discovered address
    pub fn new(multiaddr: String, is_bootstrap: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Extract IP:Port from multiaddr for the address field
        let address = extract_ip_port(&multiaddr).unwrap_or_else(|| multiaddr.clone());

        Self {
            address,
            multiaddr,
            last_peer_id: None,
            observed_peer_ids: Vec::new(),
            last_seen: now,
            first_seen: now,
            consecutive_failures: 0,
            backoff_seconds: 0,
            next_attempt_after: 0,
            locally_verified: false,
            is_bootstrap,
            known_topics: Vec::new(),
            label: None,
        }
    }

    /// Record a successful connection
    pub fn record_success(&mut self, peer_id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check if PeerID changed
        if let Some(ref old_id) = self.last_peer_id {
            if old_id != peer_id {
                tracing::warn!(
                    "[WARNING] PeerID changed at {}: {} -> {} (accepting new identity)",
                    self.address,
                    old_id,
                    peer_id
                );
            }
        }

        self.last_peer_id = Some(peer_id.to_string());

        // Track all observed PeerIDs
        if !self.observed_peer_ids.contains(&peer_id.to_string()) {
            self.observed_peer_ids.push(peer_id.to_string());
        }

        self.last_seen = now;
        self.consecutive_failures = 0;
        self.backoff_seconds = 0;
        self.next_attempt_after = 0;
        self.locally_verified = true;
    }

    /// Record a failed connection attempt with exponential backoff
    pub fn record_failure(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.consecutive_failures = self.consecutive_failures.saturating_add(1);

        // Exponential backoff: 5s, 10s, 20s, 40s, 80s, 160s, 300s (cap at 5 min).
        // Clamp exponent before shifting to avoid overflow under long-lived failure streaks.
        let exponent = self.consecutive_failures.saturating_sub(1).min(6);
        let uncapped_backoff = 5u64.saturating_mul(1u64 << exponent);
        self.backoff_seconds = std::cmp::min(uncapped_backoff, 300);

        self.next_attempt_after = now.saturating_add(self.backoff_seconds);
    }

    /// Check if we should attempt connection now
    pub fn should_attempt(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now >= self.next_attempt_after
    }

    /// Record a topic observed from this peer
    pub fn add_topic(&mut self, topic: &str) {
        if !self.known_topics.contains(&topic.to_string()) {
            self.known_topics.push(topic.to_string());
        }
    }
}

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

/// Process-lifetime per-peer dial state (NOT serialized to peers.json).
#[derive(Debug, Clone, Default)]
pub struct PeerDialState {
    /// Consecutive dial failures this session (1st failure -> 5s delay).
    pub consecutive_failures: u32,

    /// Unix ts: next allowed dial attempt (0 = now).
    pub next_attempt_after: u64,

    /// A dial for this key is currently in flight.
    pub in_flight: bool,

    /// Has a successful connection history (seeded from ledger, set on success).
    pub is_known_good: bool,
}

impl PeerDialState {
    /// Backoff ladder in seconds: 5s, 30s, 2m, 5m, 30m.
    pub const BACKOFF_LADDER: [u64; 5] = [5, 30, 120, 300, 1800];

    /// Whether a new dial may be started now.
    pub fn ready(&self, now: u64) -> bool {
        now >= self.next_attempt_after && !self.in_flight
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

/// Process-lifetime ADDRESS-level dial guard (NOT serialized to peers.json).
///
/// `peer_dial_states` alone is not enough to stop simultaneous connections to
/// one physical host:port. It is keyed by `DialKey`, which is `Peer(pid)`
/// whenever a PeerId is known (see `DialKey::for_target`) -- and this fleet's
/// nodes mint a new identity on every rebuild, so one address can accumulate
/// many stale PeerIds in `LedgerEntry::observed_peer_ids`. Each stale
/// identity produces a DIFFERENT `DialKey::Peer`, so the peer-level guard
/// sees N unrelated dials while the OS opens N concurrent connections to the
/// same address -- which is exactly what tripped a `debug_assert_eq!` inside
/// `libp2p-request-response` in production (three simultaneous connections
/// to the byte-identical multiaddr within 30ms).
///
/// This guard is keyed on the normalized address string instead (see
/// `ConnectionLedger::key_to_policy_args`, which resolves a `DialKey::Peer`
/// back to its known address via `find_by_peer_id` and already reuses
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

/// The Connection Ledger — persistent storage for all known peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLedger {
    /// All known peer entries, keyed by multiaddr (without /p2p/ suffix)
    pub entries: HashMap<String, LedgerEntry>,

    /// Version for future migrations
    pub version: u32,

    /// Last save timestamp
    pub last_saved: u64,

    /// Process-lifetime per-peer dial state. Never persisted to peers.json.
    #[serde(skip)]
    pub peer_dial_states: HashMap<DialKey, PeerDialState>,

    /// Process-lifetime per-address dial state, keyed by the normalized
    /// (stripped) address string. See `AddrDialState` for why this exists
    /// alongside `peer_dial_states`. Never persisted to peers.json.
    #[serde(skip)]
    pub addr_dial_states: HashMap<String, AddrDialState>,

    /// Global dial policy manager enforcing per-peer backoff and concurrent dial limits.
    #[serde(skip)]
    pub dial_policy: DialPolicyManager,
}

impl Default for ConnectionLedger {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
            last_saved: 0,
            peer_dial_states: HashMap::new(),
            addr_dial_states: HashMap::new(),
            dial_policy: DialPolicyManager::new(),
        }
    }
}

impl ConnectionLedger {
    /// Load the ledger from disk, or create a new one
    pub fn load(data_dir: &Path) -> Result<Self> {
        let ledger_path = data_dir.join("peers.json");

        if ledger_path.exists() {
            let contents =
                std::fs::read_to_string(&ledger_path).context("Failed to read peers.json")?;
            let ledger: ConnectionLedger =
                serde_json::from_str(&contents).context("Failed to parse peers.json")?;
            tracing::info!(
                "[INFO] Loaded connection ledger: {} known peers",
                ledger.entries.len()
            );
            Ok(ledger)
        } else {
            tracing::info!("[INFO] No existing ledger found, starting fresh");
            Ok(Self::default())
        }
    }

    /// Save the ledger to disk
    pub fn save(&mut self, data_dir: &Path) -> Result<()> {
        let ledger_path = data_dir.join("peers.json");

        self.last_saved = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let contents = serde_json::to_string_pretty(self).context("Failed to serialize ledger")?;
        std::fs::write(&ledger_path, contents).context("Failed to write peers.json")?;

        tracing::debug!("[INFO] Saved ledger ({} entries)", self.entries.len());
        Ok(())
    }

    /// Add or update a peer entry from a bootstrap multiaddr
    pub fn add_bootstrap(&mut self, multiaddr: &str, local_peer_id: Option<&str>) {
        if let Some(local) = local_peer_id {
            if multiaddr.contains(local) {
                return;
            }
        }
        let stripped = strip_peer_id(multiaddr);
        let label = format!("Bootstrap {}", self.entries.len() + 1);

        self.entries
            .entry(stripped.clone())
            .and_modify(|e| {
                e.is_bootstrap = true;
                e.locally_verified = true;
            })
            .or_insert_with(|| {
                let mut entry = LedgerEntry::new(stripped.clone(), true);
                entry.label = Some(label);
                entry.locally_verified = true;
                entry
            });
    }

    /// Extract address key string and optional PeerId for DialPolicyManager calls.
    pub fn key_to_policy_args(&self, key: &DialKey) -> (String, Option<PeerId>) {
        match key {
            DialKey::Peer(pid) => {
                let addr_key = if let Some(entry) = self.find_by_peer_id(&pid.to_string()) {
                    strip_peer_id(&entry.multiaddr)
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

    /// Add or update a peer after successful connection.
    ///
    /// INGESTION CHOKE POINT (re-review round 4, F3/NEW-1). `dns` is a REQUIRED
    /// parameter, and that is the entire point of this signature: the gate used
    /// to live in the callers with `AllowLocallyConfigured` hardcoded here, so
    /// `cmd_relay`'s `PeerIdentified` handler in `main.rs` grew a
    /// `DnsPolicy::Reject` check citing re-review NEW-1 while `cmd_start`'s
    /// byte-identical handler forty lines away did not. Both now have to name
    /// their provenance to call this at all, and both of them go through
    /// [`record_identified_peer`] anyway, which is the only place either handler
    /// still exists.
    ///
    /// [`DnsPolicy::Reject`] is the [`Default`], so a future caller who does not
    /// think about provenance fails closed.
    /// Returns true iff the address passed the gate and was recorded.
    pub fn record_connection(&mut self, multiaddr: &str, peer_id: &str, dns: DnsPolicy) -> bool {
        let stripped = strip_peer_id(multiaddr);
        if !is_dialable_multiaddr(&stripped, NetworkMode::Local, dns) {
            return false;
        }

        let parsed_pid = PeerId::from_str(peer_id).ok();
        self.dial_policy
            .reset_on_connection_established(&stripped, parsed_pid);

        self.entries
            .entry(stripped.clone())
            .and_modify(|e| {
                e.record_success(peer_id);
                e.locally_verified = true;
            })
            .or_insert_with(|| {
                let mut entry = LedgerEntry::new(stripped.clone(), false);
                entry.record_success(peer_id);
                entry.locally_verified = true;
                entry
            });
        true
    }

    /// Apply an Identify `PeerIdentified` event to the ledger.
    ///
    /// THE ONE HANDLER (choke-point refactor 2026-07-26). `cmd_start` and
    /// `cmd_relay` each carried their own inline copy of this loop. They were
    /// byte-identical until re-review NEW-1, at which point the DNS gate was
    /// added to `cmd_relay`'s copy -- with a comment citing the review -- and
    /// not to `cmd_start`'s. Both call sites now call this, so there is nothing
    /// left to keep in sync, and the behaviour is unit-testable without a
    /// running swarm (there was no CLI test covering either handler, which is
    /// why the miss survived a full review round).
    ///
    /// `listen_addrs` is whatever the REMOTE peer chose to advertise. It is not
    /// evidence of anything, so it enters the ledger under
    /// [`DnsPolicy::Reject`]: a `/dns4/evil.example/tcp/80` entry would
    /// otherwise be stored as locally verified and later dialed, with the zone
    /// owner picking the destination IP -- and re-picking it between probes.
    ///
    /// Returns the number of addresses actually recorded, so a test can assert
    /// the rejection rather than inferring it.
    pub fn record_identified_peer(&mut self, peer_id: &str, listen_addrs: &[String]) -> usize {
        listen_addrs
            .iter()
            .filter(|addr| self.record_connection(addr, peer_id, DnsPolicy::Reject))
            .count()
    }

    /// Record a topic observed from a peer
    pub fn record_topic(&mut self, multiaddr: &str, topic: &str) {
        let stripped = strip_peer_id(multiaddr);
        if let Some(entry) = self.entries.get_mut(&stripped) {
            entry.add_topic(topic);
        }
    }

    /// Record a failed connection attempt
    pub fn record_failure(&mut self, multiaddr: &str) {
        let stripped = strip_peer_id(multiaddr);
        let parsed_pid = if let Some(idx) = multiaddr.find("/p2p/") {
            let remainder = &multiaddr[idx + "/p2p/".len()..];
            PeerId::from_str(remainder).ok()
        } else {
            None
        };
        self.dial_policy.record_dial_failure(&stripped, parsed_pid);

        if let Some(entry) = self.entries.get_mut(&stripped) {
            entry.record_failure();
            tracing::warn!(
                "[WARNING] Connection failed to {} (attempt #{}, backoff {}s)",
                stripped,
                entry.consecutive_failures,
                entry.backoff_seconds
            );
        }
    }

    /// Get all addresses that should be dialed now, excluding the local node.
    ///
    /// DOES NOT DEPEND ON THE INGESTION INVARIANT (re-review round 4). This used
    /// to pass `AllowLocallyConfigured` for EVERY entry, justified by a comment
    /// asserting that "a DNS-form address may enter the CLI ledger only through
    /// `add_bootstrap`". That assertion was false -- `record_connection` accepted
    /// names from `cmd_start`'s `PeerIdentified` handler -- and a filter whose
    /// soundness rests on a documented invariant somewhere else is the exact
    /// failure mode this refactor exists to remove.
    ///
    /// So the permission is now derived from the entry itself: only an entry
    /// flagged `is_bootstrap` (which only [`Self::add_bootstrap`] sets, from
    /// operator configuration) may be a name. Every other entry is judged with
    /// [`DnsPolicy::Reject`]. Even if some future path did smuggle a name into
    /// the store, it would not be dialed.
    pub fn dialable_addresses(&self, local_peer_id: Option<&str>) -> Vec<(String, Option<String>)> {
        self.entries
            .values()
            .filter(|e| e.should_attempt())
            .filter(|e| {
                let dns = if e.is_bootstrap {
                    DnsPolicy::AllowLocallyConfigured
                } else {
                    DnsPolicy::Reject
                };
                is_dialable_multiaddr(&e.multiaddr, NetworkMode::Local, dns)
            })
            .filter(|e| {
                if let (Some(local), Some(last)) = (local_peer_id, &e.last_peer_id) {
                    local != last && !contains_peer_id_component(&e.multiaddr, local)
                } else if let Some(local) = local_peer_id {
                    !contains_peer_id_component(&e.multiaddr, local)
                } else {
                    true
                }
            })
            .map(|e| (e.multiaddr.clone(), e.last_peer_id.clone()))
            .collect()
    }

    /// Get all known topics from connected peers
    pub fn all_known_topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = self
            .entries
            .values()
            .flat_map(|e| e.known_topics.clone())
            .collect();
        topics.sort();
        topics.dedup();
        topics
    }

    /// Find entry by PeerID (lookup across all entries)
    pub fn find_by_peer_id(&self, peer_id: &str) -> Option<&LedgerEntry> {
        self.entries.values().find(|e| {
            e.last_peer_id.as_deref() == Some(peer_id)
                || e.observed_peer_ids.contains(&peer_id.to_string())
        })
    }

    // `to_shared_entries()` USED TO LIVE HERE AND IS GONE ON PURPOSE
    // (re-review NEW-2, choke-point refactor 2026-07-26). It was the SECOND
    // disclosure door: the ledger-exchange RESPONSE went through
    // `LedgerManager::exchange_response_entries`, which caps at 64, requires
    // `success_count > 0`, filters every address through
    // `is_disclosable_multiaddr` and blanks `known_topics` -- while the REQUEST
    // went through this function, which had no cap, no proven-peer filter, no
    // address filter, and copied `known_topics` verbatim. It fired on every peer
    // connection from three sites in `main.rs`.
    //
    // Both directions of the protocol now build their payload from
    // `exchange_response_entries` inside the swarm, so there is one door. The
    // CLI no longer produces wire records at all; see
    // `SwarmCommand::ShareLedger`.

    /// Merge peer entries received from a remote peer.
    ///
    /// New addresses are added with is_bootstrap=false.
    /// Existing addresses get their last_seen updated if the remote has
    /// a more recent timestamp. Returns the number of new peers learned.
    pub fn merge_shared_entries(
        &mut self,
        entries: &[scmessenger_core::transport::SharedPeerEntry],
    ) -> usize {
        let mut new_count = 0;

        for entry in entries {
            let stripped = strip_peer_id(&entry.multiaddr);

            // `DnsPolicy::Reject` (re-review NEW-1): this is the wire path, and
            // it is the gate that lets `dialable_addresses` keep allowing names.
            if !is_dialable_multiaddr(&stripped, NetworkMode::Local, DnsPolicy::Reject) {
                continue;
            }

            if let Some(existing) = self.entries.get_mut(&stripped) {
                // Update last_seen if the remote has fresher data
                if entry.last_seen > existing.last_seen {
                    existing.last_seen = entry.last_seen;
                }
                // Update PeerID if we didn't have one
                if existing.last_peer_id.is_none() {
                    existing.last_peer_id = entry.last_peer_id.clone();
                }
                // Merge topics
                for topic in &entry.known_topics {
                    existing.add_topic(topic);
                }
            } else {
                // Brand new peer — add it
                let mut new_entry = LedgerEntry::new(stripped.clone(), false);
                new_entry.last_peer_id = entry.last_peer_id.clone();
                new_entry.last_seen = entry.last_seen;
                new_entry.known_topics = entry.known_topics.clone();
                new_entry.label = Some("Discovered via peer".to_string());

                // Track the PeerID in observed list
                if let Some(ref pid) = entry.last_peer_id {
                    if !new_entry.observed_peer_ids.contains(pid) {
                        new_entry.observed_peer_ids.push(pid.clone());
                    }
                }

                self.entries.insert(stripped, new_entry);
                new_count += 1;
            }
        }

        if new_count > 0 {
            tracing::info!(
                "[INFO] Merged {} new peers from ledger exchange (total: {})",
                new_count,
                self.entries.len()
            );
        }

        new_count
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        let total = self.entries.len();
        let bootstrap = self.entries.values().filter(|e| e.is_bootstrap).count();
        let reachable = self
            .entries
            .values()
            .filter(|e| e.consecutive_failures == 0)
            .count();
        let backoff = self
            .entries
            .values()
            .filter(|e| e.consecutive_failures > 0)
            .count();

        format!(
            "Ledger: {} peers ({} bootstrap, {} reachable, {} in backoff)",
            total, bootstrap, reachable, backoff
        )
    }

    /// Decide whether a dial may be started for `key` right now.
    ///
    /// Returns true only when the key is ready, backoff eligible, under concurrent limit,
    /// and the dial is not suppressed by a healthy relay path. When the key is new, it is seeded
    /// from the persistent ledger so known-good peers are never suppressed.
    ///
    /// Claims TWO slots on success: the per-peer slot in `peer_dial_states`
    /// (unchanged from before this guard existed) and the per-address slot in
    /// `addr_dial_states` (see `AddrDialState`). Returns `false` if EITHER is
    /// already claimed. The peer-level slot is claimed first and the
    /// DialPolicyManager registration before it; if the address-level check
    /// then finds the address already in flight under a DIFFERENT `DialKey`,
    /// both of those provisional claims are released before returning `false`
    /// -- this function must never leak a half-claim, or the address is
    /// wedged closed until process restart.
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

            let mut state = self.peer_dial_states.remove(key).unwrap_or_default();
            state.record_success();

            if let DialKey::Addr(_) = key {
                if let Some(pid) = learned_peer_id {
                    let peer_key = DialKey::Peer(pid);
                    self.peer_dial_states.entry(peer_key).or_insert(state);
                    return;
                }
            }

            self.peer_dial_states.insert(key.clone(), state);
        } else {
            self.dial_policy.record_dial_failure(&addr_key, pid_opt);
            if let Some(state) = self.peer_dial_states.get_mut(key) {
                state.record_failure(now);
            }
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
                .find_by_peer_id(&pid.to_string())
                .map(|e| e.is_bootstrap)
                .unwrap_or(false),
            DialKey::Addr(addr) => self
                .entries
                .get(addr)
                .map(|e| e.is_bootstrap)
                .unwrap_or(false),
        }
    }

    fn is_known_good_key(&self, key: &DialKey) -> bool {
        match key {
            DialKey::Peer(pid) => self.find_by_peer_id(&pid.to_string()).is_some_and(|e| {
                e.locally_verified && e.last_peer_id.is_some() && e.consecutive_failures == 0
            }),
            DialKey::Addr(addr) => self.entries.get(addr).is_some_and(|e| {
                e.locally_verified && e.last_peer_id.is_some() && e.consecutive_failures == 0
            }),
        }
    }
}

// Address filtering lives in the core crate now (adversarial review F3): core's
// ledger-seed import, its seed-dial candidate build and its ledger-exchange
// response all need the same rules, and having two definitions of "dialable" in
// one workspace is how core ended up with none. These re-exports keep every
// existing `ledger::is_dialable_multiaddr` / `ledger::NetworkMode` call site
// working unchanged.
//
// Behavioural deltas versus the previous CLI-local implementation, all
// strictly-tightening: multicast, broadcast, 0.0.0.0/8, 192.0.0.0/24,
// IPv4-mapped IPv6 (`::ffff:127.0.0.1`) and IPv6 unique-local (`fc00::/7`, in
// Public mode) are now rejected, and a string with no transport component at
// all -- including `""`, which `Multiaddr` happily parses -- is no longer
// reported as dialable.
//
// Re-review NEW-1: `is_dialable_multiaddr` also takes a [`DnsPolicy`], because a
// `/dns4/...` address resolves to whatever its zone owner chooses at dial time,
// so none of the IP rules can be applied to it.
//
// CHOKE-POINT REFACTOR (2026-07-26). This file used to maintain the invariant
// "a DNS-form address may enter the CLI ledger only through `add_bootstrap`",
// and `dialable_addresses` DEPENDED on it -- it passed `AllowLocallyConfigured`
// for every entry. The invariant was false: `record_connection` hardcoded
// `AllowLocallyConfigured` and `cmd_start`'s `PeerIdentified` handler fed it the
// remote's advertised `listen_addrs` unfiltered. Two changes remove the
// dependency instead of restating the invariant:
//
//   1. `record_connection` takes `DnsPolicy` as a REQUIRED parameter, and the
//      only production callers reach it through `record_identified_peer`, which
//      passes `Reject`.
//   2. `dialable_addresses` derives the policy per entry from `is_bootstrap`
//      (set only by `add_bootstrap`), so it is sound even if some future path
//      does store a name.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A syntactically valid peer id.
    ///
    /// `strip_peer_id` now parses the multiaddr instead of truncating the
    /// string at the first `/p2p/` (core review F8: the truncation collapsed
    /// `/ip4/A/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` to the relay's
    /// bare address). Short made-up ids like `12D3KooWSpoof` are not valid
    /// multihashes and never occur on the wire, so the fixtures have to be
    /// real.
    fn test_peer_id() -> String {
        PeerId::random().to_string()
    }

    #[test]
    fn test_strip_peer_id() {
        let pid = test_peer_id();
        assert_eq!(
            strip_peer_id(&format!("/ip4/1.2.3.4/tcp/9001/p2p/{pid}")),
            "/ip4/1.2.3.4/tcp/9001"
        );
        assert_eq!(
            strip_peer_id("/ip4/1.2.3.4/tcp/9001"),
            "/ip4/1.2.3.4/tcp/9001"
        );
    }

    /// Core review F8, asserted from the CLI side too: the two ledgers are
    /// documented to dedupe on identical keys, so the CLI's key function must
    /// have the same circuit behaviour as the core's.
    #[test]
    fn test_strip_peer_id_preserves_circuit_path() {
        let relay = test_peer_id();
        let target = test_peer_id();
        let stripped = strip_peer_id(&format!(
            "/ip4/1.2.3.4/tcp/443/p2p/{relay}/p2p-circuit/p2p/{target}"
        ));
        assert_eq!(
            stripped,
            format!("/ip4/1.2.3.4/tcp/443/p2p/{relay}/p2p-circuit")
        );
        assert!(!stripped.contains(&target));
    }

    #[test]
    fn test_extract_ip_port() {
        assert_eq!(
            extract_ip_port("/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW"),
            Some("1.2.3.4:9001".to_string())
        );
        assert_eq!(
            extract_ip_port("/ip4/10.0.0.1/tcp/4001"),
            Some("10.0.0.1:4001".to_string())
        );
    }

    #[test]
    fn test_ledger_entry_backoff() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), false);
        assert!(entry.should_attempt());

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 1);
        assert_eq!(entry.backoff_seconds, 5);

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 2);
        assert_eq!(entry.backoff_seconds, 10);

        entry.record_failure();
        assert_eq!(entry.consecutive_failures, 3);
        assert_eq!(entry.backoff_seconds, 20);

        // Success resets everything
        entry.record_success("12D3KooWTest");
        assert_eq!(entry.consecutive_failures, 0);
        assert_eq!(entry.backoff_seconds, 0);
        assert_eq!(entry.last_peer_id, Some("12D3KooWTest".to_string()));
    }

    #[test]
    fn test_ledger_entry_backoff_overflow_safety() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), false);
        entry.consecutive_failures = u32::MAX;

        entry.record_failure();

        assert_eq!(entry.consecutive_failures, u32::MAX);
        assert_eq!(entry.backoff_seconds, 300);
        assert!(entry.next_attempt_after >= entry.backoff_seconds);
    }

    #[test]
    fn test_ledger_entry_peer_id_tracking() {
        let mut entry = LedgerEntry::new("/ip4/1.2.3.4/tcp/9001".to_string(), true);

        entry.record_success("PeerA");
        assert_eq!(entry.last_peer_id, Some("PeerA".to_string()));
        assert_eq!(entry.observed_peer_ids, vec!["PeerA".to_string()]);

        // Peer restarts with new key
        entry.record_success("PeerB");
        assert_eq!(entry.last_peer_id, Some("PeerB".to_string()));
        assert_eq!(
            entry.observed_peer_ids,
            vec!["PeerA".to_string(), "PeerB".to_string()]
        );
    }

    #[test]
    fn test_ledger_crud() {
        let mut ledger = ConnectionLedger::default();

        ledger.add_bootstrap(
            &format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", test_peer_id()),
            None,
        );
        assert_eq!(ledger.entries.len(), 1);

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.is_bootstrap);

        ledger.record_connection("/ip4/1.2.3.4/tcp/9001", "NewPeerId", DnsPolicy::Reject);
        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert_eq!(entry.last_peer_id, Some("NewPeerId".to_string()));
    }

    #[test]
    fn test_ledger_topic_tracking() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001", None);
        ledger.record_topic("/ip4/1.2.3.4/tcp/9001", "sc-mesh");
        ledger.record_topic("/ip4/1.2.3.4/tcp/9001", "sc-lobby");

        let topics = ledger.all_known_topics();
        assert!(topics.contains(&"sc-mesh".to_string()));
        assert!(topics.contains(&"sc-lobby".to_string()));
    }

    #[test]
    fn test_is_dialable_multiaddr() {
        use DnsPolicy::{AllowLocallyConfigured, Reject};
        use NetworkMode::{Local, Public};
        // Non-routable: rejected regardless of mode.
        assert!(!is_dialable_multiaddr(
            "/ip4/127.0.0.1/tcp/9001",
            Local,
            Reject
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/0.0.0.0/tcp/9001",
            Local,
            Reject
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/169.254.1.2/tcp/9001",
            Local,
            Reject
        ));
        assert!(!is_dialable_multiaddr("/ip6/::1/tcp/9001", Local, Reject));
        assert!(!is_dialable_multiaddr(
            "/ip6/fe80::1897:a8ff:fec5:3d16/tcp/443",
            Local,
            Reject
        ));
        assert!(!is_dialable_multiaddr(
            "/ip6/fec0::1/tcp/9001",
            Local,
            Reject
        ));
        // Globally routable: accepted.
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/9001",
            Local,
            Reject
        ));
        assert!(is_dialable_multiaddr(
            "/ip6/2606:4700:4700::1111/tcp/9001",
            Local,
            Reject
        ));
        // Private/LAN: kept in Local, dropped in Public.
        assert!(is_dialable_multiaddr(
            "/ip4/10.0.2.16/tcp/9001",
            Local,
            Reject
        ));
        assert!(is_dialable_multiaddr(
            "/ip4/192.168.1.5/tcp/9001",
            Local,
            Reject
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/10.0.2.16/tcp/9001",
            Public,
            Reject
        ));
        assert!(!is_dialable_multiaddr(
            "/ip4/192.168.1.5/tcp/9001",
            Public,
            Reject
        ));
        // p2p-circuit always allowed (relay path).
        assert!(is_dialable_multiaddr(
            "/ip4/1.2.3.4/tcp/9001/p2p-circuit",
            Local,
            Reject
        ));
        // A name is only as trustworthy as whoever supplied it.
        assert!(!is_dialable_multiaddr(
            "/dns4/relay.example/tcp/443",
            Local,
            Reject
        ));
        assert!(is_dialable_multiaddr(
            "/dns4/relay.example/tcp/443",
            Local,
            AllowLocallyConfigured
        ));
    }

    /// Re-review NEW-1, CLI half: the wire merge path must not let a peer put a
    /// DNS name into the ledger, because `dialable_addresses` deliberately
    /// allows names (they can only have come from `add_bootstrap`) and the
    /// dial scheduler resolves whatever it is handed.
    #[test]
    fn merge_shared_entries_rejects_dns_forms() {
        let mut ledger = ConnectionLedger::default();
        let hostile: Vec<scmessenger_core::transport::SharedPeerEntry> = [
            "/dns4/evil.example/tcp/80",
            "/dns6/evil.example/tcp/80",
            "/dns/evil.example/tcp/80",
            "/dnsaddr/evil.example",
        ]
        .iter()
        .map(|addr| scmessenger_core::transport::SharedPeerEntry {
            multiaddr: addr.to_string(),
            last_peer_id: None,
            last_seen: 0,
            known_topics: Vec::new(),
        })
        .collect();

        assert_eq!(ledger.merge_shared_entries(&hostile), 0);
        assert!(ledger.dialable_addresses(None).is_empty());
    }

    #[test]
    fn test_is_self_address() {
        let my_addrs = vec![
            "/ip4/192.168.0.121/tcp/9001".to_string(),
            format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", test_peer_id()),
        ];
        // Exact match (own LAN address) -> self-dial.
        assert!(is_self_address("/ip4/192.168.0.121/tcp/9001", &my_addrs));
        // Own address with a peer-id suffix attached still matches after stripping.
        assert!(is_self_address(
            &format!("/ip4/192.168.0.121/tcp/9001/p2p/{}", test_peer_id()),
            &my_addrs
        ));
        // Own public address matches regardless of which side carries the peer-id.
        assert!(is_self_address("/ip4/1.2.3.4/tcp/9001", &my_addrs));
        // A different address is not a self-dial.
        assert!(!is_self_address("/ip4/10.0.2.16/tcp/9001", &my_addrs));
    }

    #[test]
    fn test_is_dialable_for_this_node() {
        use NetworkMode::Local;
        // Node is on a 192.168.x.x home LAN.
        let my_addrs = vec!["/ip4/192.168.0.121/tcp/9001".to_string()];

        // Self-dial rejected even though it would otherwise be dialable.
        assert!(!is_dialable_for_this_node(
            "/ip4/192.168.0.121/tcp/9001",
            Local,
            &my_addrs
        ));
        // Another peer on the SAME private range (192.168.x.x) is fine.
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.0.55/tcp/9001",
            Local,
            &my_addrs
        ));
        // A DIFFERENT private range (10.x.x.x, e.g. an emulator's internal
        // address) is not reachable from a 192.168.x.x-only node.
        assert!(!is_dialable_for_this_node(
            "/ip4/10.0.2.16/tcp/9001",
            Local,
            &my_addrs
        ));
        // Globally routable addresses are unaffected by range-awareness.
        assert!(is_dialable_for_this_node(
            "/ip4/1.2.3.4/tcp/9001",
            Local,
            &my_addrs
        ));

        // A node with no private addresses of its own (e.g. cellular-only)
        // should not dial ANY private-range address.
        let public_only: Vec<String> = vec!["/ip4/1.2.3.4/tcp/9001".to_string()];
        assert!(!is_dialable_for_this_node(
            "/ip4/192.168.1.5/tcp/9001",
            Local,
            &public_only
        ));

        // Dual-homed node (has addresses in TWO different private classes):
        // both classes should be dialable, not just the first one found.
        let dual_homed = vec![
            "/ip4/192.168.0.121/tcp/9001".to_string(),
            "/ip4/10.5.5.5/tcp/9001".to_string(),
        ];
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.1.5/tcp/9001",
            Local,
            &dual_homed
        ));
        assert!(is_dialable_for_this_node(
            "/ip4/10.9.9.9/tcp/9001",
            Local,
            &dual_homed
        ));
        // Still not the third RFC1918 class (172.16.0.0/12).
        assert!(!is_dialable_for_this_node(
            "/ip4/172.16.0.5/tcp/9001",
            Local,
            &dual_homed
        ));

        // A relay-circuit address's leading /ip4/.../ is the RELAY hop, not
        // the final target -- it must NOT be subject to RFC1918
        // class-matching against that hop's own address, or the only path
        // to a NAT'd peer behind a cross-class relay would be silently
        // dropped. Regression test for the exact shape used by this
        // project's own test fixtures (core/src/transport/swarm.rs).
        let my_addrs = vec!["/ip4/192.168.0.121/tcp/9001".to_string()];
        assert!(is_dialable_for_this_node(
            "/ip4/172.26.144.1/tcp/9101/p2p/12D3KooWRelay/p2p-circuit/p2p/12D3KooWTarget",
            Local,
            &my_addrs
        ));
        // A circuit address whose relay hop happens to share this node's IP
        // is NOT treated as a self-dial: is_self_address does an exact
        // string match after stripping at the first "/p2p/", and the
        // "/p2p-circuit" suffix makes that stripped string differ from a
        // plain "/ip4/.../tcp/9001" self-address, so this is correctly
        // treated as "unconditionally allowed circuit address", not "self".
        // (Genuinely self-targeted circuit dials are a degenerate case the
        // ledger shouldn't produce in practice; libp2p itself also rejects
        // dialing one's own PeerId at the connection layer as a backstop.)
        assert!(is_dialable_for_this_node(
            "/ip4/192.168.0.121/tcp/9001/p2p-circuit/p2p/12D3KooWTarget",
            Local,
            &my_addrs
        ));
    }

    #[test]
    fn self_peer_components_are_rejected_from_ledger_dials() {
        let local = test_peer_id();
        let self_target = format!("/ip4/192.168.0.121/tcp/9001/p2p-circuit/p2p/{local}");
        let remote_target = format!(
            "/ip4/192.168.0.121/tcp/9001/p2p-circuit/p2p/{}",
            PeerId::random()
        );

        assert!(contains_peer_id_component(&self_target, &local));
        assert!(!contains_peer_id_component(&remote_target, &local));
    }

    #[test]
    fn local_candidates_precede_carrier_ipv6_candidates() {
        let candidates = vec![
            (
                "/ip6/2600:381:9b57:6b48:e125:d55a:4e95:7896/tcp/8080".to_string(),
                None,
            ),
            ("/ip4/192.168.0.136/tcp/9001".to_string(), None),
            ("/ip4/198.51.100.10/tcp/443".to_string(), None),
        ];

        let ordered = prioritize_dial_candidates(candidates);
        assert!(ordered[0].0.starts_with("/ip4/192.168."));
        assert!(ordered[1].0.starts_with("/ip4/198.51."));
        assert!(ordered[2].0.starts_with("/ip6/2600:"));
    }

    #[test]
    fn test_dial_key_for_target() {
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_id_str = peer_id.to_string();

        // Explicit peer id wins.
        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", Some(peer_id));
        assert_eq!(key, DialKey::Peer(peer_id));

        // Parsed from /p2p/ suffix.
        let addr_with_p2p = format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", peer_id_str);
        let key = DialKey::for_target(&addr_with_p2p, None);
        assert_eq!(key, DialKey::Peer(peer_id));

        // Address-only falls back to stripped multiaddr.
        let key = DialKey::for_target("/ip4/1.2.3.4/tcp/9001", None);
        assert_eq!(key, DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string()));
    }

    #[test]
    fn test_peer_dial_state_backoff_ladder() {
        let mut state = PeerDialState::default();
        let now = 1_000_000;
        let expected = [5, 30, 120, 300, 1800, 1800];

        for (i, &delay) in expected.iter().enumerate() {
            state.record_failure(now);
            assert_eq!(state.consecutive_failures, (i + 1) as u32);
            assert_eq!(state.next_attempt_after.saturating_sub(now), delay);
            assert!(!state.ready(now + delay - 1));
            assert!(state.ready(now + delay));
        }
    }

    #[test]
    fn test_peer_dial_state_success_reset() {
        let mut state = PeerDialState::default();
        let now = 1_000_000;

        state.record_failure(now);
        state.record_failure(now);
        assert!(!state.ready(now));

        state.record_success();
        assert!(state.ready(now));
        assert_eq!(state.consecutive_failures, 0);
        assert_eq!(state.next_attempt_after, 0);
        assert!(state.is_known_good);
    }

    #[test]
    fn test_try_begin_dial_blocks_in_flight_reuse() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key.clone(), 0, false));
        assert!(!ledger.try_begin_dial(key.clone(), 0, false));
    }

    #[test]
    fn test_try_begin_dial_suppresses_unknown_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(!ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_circuit_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001/p2p-circuit".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_bootstrap_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/ip4/1.2.3.4/tcp/9001", None);
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_try_begin_dial_allows_known_good_when_relay_healthy() {
        let mut ledger = ConnectionLedger::default();
        ledger.record_connection(
            "/ip4/1.2.3.4/tcp/9001",
            "12D3KooWTestPeerId",
            DnsPolicy::Reject,
        );
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_complete_dial_failure_enforces_backoff() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        assert!(ledger.try_begin_dial(key.clone(), 0, false));
        ledger.complete_dial(&key, false, 0, None);

        let state = ledger.dial_state(&key).unwrap();
        assert!(!state.ready(4));
        assert!(state.ready(5));
    }

    #[test]
    fn test_complete_dial_migrates_addr_to_peer() {
        let mut ledger = ConnectionLedger::default();
        let addr_key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        assert!(ledger.try_begin_dial(addr_key.clone(), 0, false));
        ledger.complete_dial(&addr_key, true, 0, Some(peer_id));

        assert!(ledger.dial_state(&addr_key).is_none());
        let peer_key = DialKey::Peer(peer_id);
        let state = ledger.dial_state(&peer_key).unwrap();
        assert!(state.is_known_good);
    }

    // ------------------------------------------------------------------
    // Address-level dial guard (crash fix). Live evidence: three
    // simultaneous connections to the byte-identical multiaddr
    // /ip4/192.168.0.142/tcp/51251 within 30ms, tripping a
    // `debug_assert_eq!` inside libp2p-request-response. Root cause: nodes
    // on this fleet mint a new PeerId on every rebuild, so a single address
    // accumulates many stale PeerIds in `LedgerEntry::observed_peer_ids`.
    // Each stale identity produces a different `DialKey::Peer`, so the
    // peer-level guard alone sees N unrelated dials while the OS opens N
    // concurrent connections to the same host:port.
    // ------------------------------------------------------------------

    /// REGRESSION TEST for the crash. Two dials to the same address under
    /// DIFFERENT PeerIds -- exactly the fleet scenario -- must not both be
    /// allowed in flight simultaneously, even though the peer-level guard
    /// alone (pre-fix) sees them as two unrelated keys and allows both.
    #[test]
    fn test_try_begin_dial_blocks_same_address_different_peer_ids() {
        let mut ledger = ConnectionLedger::default();
        let addr = "/ip4/192.168.0.142/tcp/51251";
        let peer_a = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_b = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        // Simulate the fleet scenario: this node connected to the same
        // address under two different (stale) identities, so both PeerIds
        // land in `observed_peer_ids` on the SAME ledger entry.
        ledger.record_connection(addr, &peer_a.to_string(), DnsPolicy::Reject);
        ledger.record_connection(addr, &peer_b.to_string(), DnsPolicy::Reject);

        let key_a = DialKey::for_target(addr, Some(peer_a));
        let key_b = DialKey::for_target(addr, Some(peer_b));
        assert_ne!(key_a, key_b, "test setup must use two distinct DialKeys");

        assert!(ledger.try_begin_dial(key_a, 0, false));
        assert!(!ledger.try_begin_dial(key_b, 0, false));
    }

    /// After `complete_dial` releases the first dial, a dial to the SAME
    /// address under a THIRD PeerId must succeed. Proves the release path
    /// works and the address is a concurrency guard, not a permanent ban --
    /// the existing "address-only dials must NEVER be dropped" comment
    /// documents a real prior bug this must not reintroduce.
    #[test]
    fn test_try_begin_dial_address_guard_releases_after_complete_dial() {
        let mut ledger = ConnectionLedger::default();
        let addr = "/ip4/192.168.0.142/tcp/51251";
        let peer_a = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_b = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_c = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        ledger.record_connection(addr, &peer_a.to_string(), DnsPolicy::Reject);
        ledger.record_connection(addr, &peer_b.to_string(), DnsPolicy::Reject);
        ledger.record_connection(addr, &peer_c.to_string(), DnsPolicy::Reject);

        let key_a = DialKey::for_target(addr, Some(peer_a));
        let key_b = DialKey::for_target(addr, Some(peer_b));
        let key_c = DialKey::for_target(addr, Some(peer_c));

        assert!(ledger.try_begin_dial(key_a.clone(), 0, false));
        assert!(!ledger.try_begin_dial(key_b, 0, false));

        ledger.complete_dial(&key_a, true, 0, Some(peer_a));

        assert!(ledger.try_begin_dial(key_c, 0, false));
    }

    /// Dials to two DIFFERENT addresses under different PeerIds must both
    /// succeed. Proves the address-level guard did not collapse into a
    /// global dial lock.
    #[test]
    fn test_try_begin_dial_allows_concurrent_dials_to_different_addresses() {
        let mut ledger = ConnectionLedger::default();
        let addr_x = "/ip4/192.168.0.10/tcp/9001";
        let addr_y = "/ip4/192.168.0.20/tcp/9001";
        let peer_x = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_y = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        ledger.record_connection(addr_x, &peer_x.to_string(), DnsPolicy::Reject);
        ledger.record_connection(addr_y, &peer_y.to_string(), DnsPolicy::Reject);

        let key_x = DialKey::for_target(addr_x, Some(peer_x));
        let key_y = DialKey::for_target(addr_y, Some(peer_y));

        assert!(ledger.try_begin_dial(key_x, 0, false));
        assert!(ledger.try_begin_dial(key_y, 0, false));
    }

    /// Half-claim case: `key_b`'s peer-level slot is fresh (no prior state),
    /// so `try_begin_dial` provisionally claims it before the address-level
    /// check discovers `addr` is already in flight under `key_a` and
    /// rejects the dial. That provisional peer-level claim (and the
    /// DialPolicyManager registration made just before it) must be released,
    /// not leaked -- a leaked claim would wedge `key_b` at
    /// `in_flight = true` forever, which is indistinguishable from the
    /// address being permanently dead. Assert both: the half-claim is not
    /// left dangling immediately, and `key_b` itself can still dial once the
    /// address frees up.
    #[test]
    fn test_try_begin_dial_releases_half_claim_when_address_already_in_flight() {
        let mut ledger = ConnectionLedger::default();
        let addr = "/ip4/192.168.0.142/tcp/51251";
        let peer_a = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let peer_b = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        ledger.record_connection(addr, &peer_a.to_string(), DnsPolicy::Reject);
        ledger.record_connection(addr, &peer_b.to_string(), DnsPolicy::Reject);

        let key_a = DialKey::for_target(addr, Some(peer_a));
        let key_b = DialKey::for_target(addr, Some(peer_b));

        assert!(ledger.try_begin_dial(key_a.clone(), 0, false));
        assert!(!ledger.try_begin_dial(key_b.clone(), 0, false));

        // The half-claim must not be left dangling: key_b's peer-level slot
        // must not still report in_flight after the address-level guard
        // rejected it.
        match ledger.dial_state(&key_b) {
            Some(state) => assert!(
                !state.in_flight,
                "half-claimed peer-level slot was left in_flight after rejection"
            ),
            None => {}
        }

        // Once the address frees up, key_b -- the one that was
        // half-claimed and released -- must still be able to dial. If the
        // release above had not happened, key_b's peer-level slot would
        // report ready() == false forever and this would fail.
        //
        // complete_dial(..., success = true, ...) here, not false: a false
        // completion runs DialPolicyManager::record_dial_failure, which
        // applies a real wall-clock backoff to `addr_key` (unrelated to
        // this guard) that would then block the very next attempt on its
        // own and produce a false failure here.
        ledger.complete_dial(&key_a, true, 0, Some(peer_a));
        assert!(ledger.try_begin_dial(key_b, 0, false));
    }

    #[test]
    fn test_peer_dial_states_eviction_caps_at_4096() {
        let mut ledger = ConnectionLedger::default();

        for i in 0..4096u64 {
            let key = DialKey::Addr(format!("/ip4/1.2.3.4/tcp/{}", i));
            assert!(ledger.try_begin_dial(key.clone(), i, false));
            ledger.complete_dial(&key, false, i, None);
        }
        assert_eq!(ledger.peer_dial_states.len(), 4096);

        let new_key = DialKey::Addr("/ip4/9.9.9.9/tcp/9999".to_string());
        assert!(ledger.try_begin_dial(new_key.clone(), 5000, false));
        assert_eq!(ledger.peer_dial_states.len(), 4096);
        assert!(ledger.peer_dial_states.contains_key(&new_key));

        let evicted_key = DialKey::Addr("/ip4/1.2.3.4/tcp/0".to_string());
        assert!(!ledger.peer_dial_states.contains_key(&evicted_key));
    }

    #[test]
    fn test_shared_entry_does_not_seed_known_good_until_locally_verified() {
        let mut ledger = ConnectionLedger::default();
        let spoof = test_peer_id();
        let shared = scmessenger_core::transport::SharedPeerEntry {
            multiaddr: format!("/ip4/1.2.3.4/tcp/9001/p2p/{spoof}"),
            last_peer_id: Some(spoof.clone()),
            last_seen: 1_700_000_000,
            known_topics: vec![],
        };
        ledger.merge_shared_entries(&[shared]);

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.last_peer_id.is_some());
        assert_eq!(entry.consecutive_failures, 0);
        assert!(!entry.locally_verified);

        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        assert!(!ledger.try_begin_dial(key.clone(), 0, true));

        ledger.record_connection(
            &format!("/ip4/1.2.3.4/tcp/9001/p2p/{spoof}"),
            &spoof,
            DnsPolicy::Reject,
        );
        assert!(
            ledger
                .entries
                .get("/ip4/1.2.3.4/tcp/9001")
                .unwrap()
                .locally_verified
        );
        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_add_bootstrap_seeds_known_good() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap(
            &format!("/ip4/1.2.3.4/tcp/9001/p2p/{}", test_peer_id()),
            None,
        );

        let entry = ledger.entries.get("/ip4/1.2.3.4/tcp/9001").unwrap();
        assert!(entry.locally_verified);
        assert!(entry.is_bootstrap);

        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());
        assert!(ledger.try_begin_dial(key, 0, true));
    }

    #[test]
    fn test_locally_verified_defaults_false_on_deserialize() {
        let json = r#"{
            "address": "1.2.3.4:9001",
            "multiaddr": "/ip4/1.2.3.4/tcp/9001",
            "last_peer_id": "12D3KooWTest",
            "observed_peer_ids": [],
            "last_seen": 1700000000,
            "first_seen": 1700000000,
            "consecutive_failures": 0,
            "backoff_seconds": 0,
            "next_attempt_after": 0,
            "is_bootstrap": false,
            "known_topics": [],
            "label": null
        }"#;
        let entry: LedgerEntry = serde_json::from_str(json).unwrap();
        assert!(!entry.locally_verified);
    }

    // ------------------------------------------------------------------
    // Round 4 -- the ingestion choke point, CLI half (F3 / NEW-1)
    // ------------------------------------------------------------------

    /// THE ROUND-2 MISS, as a test. `cmd_start`'s `PeerIdentified` handler fed
    /// the remote's advertised `listen_addrs` straight into `record_connection`,
    /// which hardcoded `AllowLocallyConfigured`, so `/dns4/evil.example/tcp/80`
    /// was stored as locally verified and `dialable_addresses` -- which
    /// deliberately allowed names -- handed it to the dial scheduler. The
    /// desktop swarm wires a real resolver, so `A evil.example -> 169.254.169.254`
    /// becomes a dial, and the zone can be re-pointed between probes.
    ///
    /// There was NO CLI test over either handler, which is why the miss survived
    /// a full review round. `record_identified_peer` is now the only copy of the
    /// handler and this is a direct test of it.
    #[test]
    fn peer_identified_dns_listen_addr_never_reaches_dialable_addresses() {
        let mut ledger = ConnectionLedger::default();
        let pid = test_peer_id();
        let advertised: Vec<String> = [
            "/dns4/evil.example/tcp/80",
            "/dns6/evil.example/tcp/80",
            "/dns/evil.example/tcp/80",
            "/dnsaddr/evil.example",
            "/dns4/evil.example/tcp/443/p2p-circuit",
            "/dns4/metadata.google.internal/tcp/80",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        assert_eq!(ledger.record_identified_peer(&pid, &advertised), 0);
        assert!(
            ledger.entries.is_empty(),
            "a remote-advertised DNS name entered the ledger: {:?}",
            ledger.entries.keys().collect::<Vec<_>>()
        );
        assert!(
            ledger.dialable_addresses(None).is_empty(),
            "a remote-advertised DNS name became a dial target: {:?}",
            ledger.dialable_addresses(None)
        );

        // The same handler must still accept a normal advertised IP address, so
        // this is a filter and not an outage.
        let good = vec!["/ip4/198.51.100.4/tcp/9001".to_string()];
        assert_eq!(ledger.record_identified_peer(&pid, &good), 1);
        assert_eq!(ledger.dialable_addresses(None).len(), 1);
    }

    /// The same assertion one layer down, so a future refactor that bypasses
    /// `record_identified_peer` still cannot get a name in: the policy is a
    /// REQUIRED argument of `record_connection` itself.
    #[test]
    fn record_connection_honours_the_required_dns_policy() {
        let dns = "/dns4/relay.example/tcp/443";
        let pid = test_peer_id();

        let mut wire = ConnectionLedger::default();
        assert!(!wire.record_connection(dns, &pid, DnsPolicy::Reject));
        assert!(wire.entries.is_empty());

        let mut configured = ConnectionLedger::default();
        assert!(configured.record_connection(dns, &pid, DnsPolicy::AllowLocallyConfigured));
        assert_eq!(configured.entries.len(), 1);

        // Fail-closed: a caller that does not think about provenance gets the
        // strict answer.
        let mut defaulted = ConnectionLedger::default();
        assert!(!defaulted.record_connection(dns, &pid, DnsPolicy::default()));
    }

    /// `dialable_addresses` must not depend on the "names can only come from
    /// `add_bootstrap`" invariant being true elsewhere in the file -- that
    /// invariant was false for two review rounds. It derives the policy from the
    /// entry's own `is_bootstrap` flag instead.
    #[test]
    fn dialable_addresses_allows_dns_only_for_bootstrap_entries() {
        let mut ledger = ConnectionLedger::default();
        ledger.add_bootstrap("/dns4/relay.example/tcp/443", None);
        assert_eq!(ledger.dialable_addresses(None).len(), 1);

        // Simulate a name reaching the store through some path that is not
        // add_bootstrap (this is what the choke point now prevents, but the
        // dial filter must be sound even if it did happen).
        let smuggled = LedgerEntry::new("/dns4/evil.example/tcp/80".to_string(), false);
        ledger
            .entries
            .insert("/dns4/evil.example/tcp/80".to_string(), smuggled);

        let dialable = ledger.dialable_addresses(None);
        assert_eq!(
            dialable.len(),
            1,
            "a non-bootstrap DNS entry was dialable: {dialable:?}"
        );
        assert!(dialable[0].0.contains("relay.example"));
    }

    /// Round 4, NAT64: `/ip6/64:ff9b::a9fe:a9fe/tcp/80` IS 169.254.169.254, and
    /// the CLI re-exports the same predicate core uses, so the CLI must reject
    /// it too. Guards against the CLI ever growing its own copy again.
    #[test]
    fn peer_identified_nat64_wrapped_metadata_address_is_rejected() {
        let mut ledger = ConnectionLedger::default();
        let pid = test_peer_id();
        let advertised: Vec<String> = [
            "/ip6/64:ff9b::a9fe:a9fe/tcp/80",
            "/ip6/64:ff9b::7f00:1/tcp/8080",
            "/ip6/2002:a9fe:a9fe::/tcp/80",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        assert_eq!(ledger.record_identified_peer(&pid, &advertised), 0);
        assert!(ledger.dialable_addresses(None).is_empty());
    }

    #[test]
    fn test_dial_policy_manager_integration() {
        let mut ledger = ConnectionLedger::default();
        let key = DialKey::Addr("/ip4/1.2.3.4/tcp/9001".to_string());

        // First attempt allowed
        assert!(ledger.try_begin_dial(key.clone(), 0, false));

        // Complete dial with success -> resets policy
        ledger.complete_dial(&key, true, 0, None);

        // Complete dial with failure -> records failure in policy
        assert!(ledger.try_begin_dial(key.clone(), 0, false));
        ledger.complete_dial(&key, false, 0, None);
    }
}
