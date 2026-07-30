// Mesh Routing: Relay, Reputation, and Retry Logic (Phases 3-6)
//
// Implements the sovereign mesh routing system where:
// - Every node can relay messages for others (Phase 3)
// - Nodes track relay performance and reputation (Phase 5)
// - Message delivery uses multi-path retry with continuous adaptation (Phase 6)
// - Any node can bootstrap from any other node (Phase 4)

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// Route reason: direct-first policy candidate.
pub const ROUTE_REASON_DIRECT_FIRST: &str = "DIRECT_FIRST";
/// Route reason: relay chosen by recipient-recency and success score policy.
pub const ROUTE_REASON_RELAY_RECENCY_SUCCESS: &str = "RELAY_RECENCY_SUCCESS";
/// Route reason: relay chosen by success score when no recipient-recency signal exists.
pub const ROUTE_REASON_RELAY_SUCCESS_SCORE: &str = "RELAY_SUCCESS_SCORE";
/// Route reason: relay ordering required latest-success tie-break.
pub const ROUTE_REASON_RELAY_TIEBREAK_LAST_SUCCESS: &str = "RELAY_TIEBREAK_LAST_SUCCESS";
/// Route reason: relay ordering fell back to deterministic peer-id tie-break.
pub const ROUTE_REASON_RELAY_TIEBREAK_PEER_ID: &str = "RELAY_TIEBREAK_PEER_ID";

/// Ranked route candidate with deterministic metadata for trace logging.
#[derive(Debug, Clone)]
pub struct RankedRoute {
    pub path: Vec<PeerId>,
    pub reason_code: &'static str,
    pub recipient_recency: u64,
    pub relay_success_score: f64,
    pub latest_success_order: u64,
}

/// Output of advancing to the next route candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteCursorAdvance {
    pub next_index: usize,
    pub wrapped_pass: bool,
}

/// Advance to the next route in a pass; wraps to index 0 when exhausted.
pub fn advance_route_cursor(current_index: usize, candidate_count: usize) -> RouteCursorAdvance {
    if candidate_count == 0 {
        return RouteCursorAdvance {
            next_index: 0,
            wrapped_pass: false,
        };
    }

    let next_index = current_index.saturating_add(1);
    if next_index >= candidate_count {
        RouteCursorAdvance {
            next_index: 0,
            wrapped_pass: true,
        }
    } else {
        RouteCursorAdvance {
            next_index,
            wrapped_pass: false,
        }
    }
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Tolerated clock skew when accepting a `last_seen`/`seen_at` recency signal
/// (review F12). Anything beyond `now + this` is clamped down to it.
pub const RECENCY_MAX_CLOCK_SKEW_SECS: u64 = 5 * 60;

/// Oldest recency signal a wire peer may assert, matching the 7-day horizon the
/// CLI ledger already prunes against (`cli/src/ledger.rs`).
pub const RECENCY_MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

/// Upper bound on tracked `(relay, recipient)` recency pairs. The key is built
/// entirely from remote-supplied peer ids, so it needs a ceiling.
pub const RECENCY_MAX_TRACKED_ROUTES: usize = 4096;

/// Low-water mark the pruner drops to once [`RECENCY_MAX_TRACKED_ROUTES`] is
/// exceeded.
///
/// HYSTERESIS, NOT A TRIM (re-review NEW-3). The first remediation pass pruned
/// on EVERY insert and, past the ceiling, collected the whole map into a `Vec`
/// and sorted it -- to remove exactly ONE entry. That is O(n log n) per insert
/// with zero amortisation, driven directly from the ledger-exchange handler on
/// the `select!` thread that also owns the swarm poll and the dial sweep: the
/// F12 fix reintroduced the F4 event-loop DoS it was written next to. Dropping
/// to a low-water mark amortises the work over
/// `RECENCY_MAX_TRACKED_ROUTES - RECENCY_PRUNE_TARGET_ROUTES` inserts, and the
/// pruner no longer sorts at all.
pub const RECENCY_PRUNE_TARGET_ROUTES: usize = 3072;

/// Maximum `(relay, recipient)` routes a SINGLE relay peer may occupy.
///
/// Re-review NEW-4: without this, one peer can fill all
/// [`RECENCY_MAX_TRACKED_ROUTES`] slots and flush every honest route out of the
/// map, making `recipient_recency_by_route` -- the primary descending sort key
/// in [`MultiPathDelivery::ranked_routes`] -- entirely its own. 64 is symmetric
/// with the ledger-exchange response cap, i.e. exactly one full honest exchange
/// fits, and a peer that exceeds it evicts only ITS OWN oldest route.
pub const RECENCY_MAX_ROUTES_PER_RELAY: usize = 64;

// ============================================================================
// PHASE 3: RELAY CAPABILITY
// ============================================================================

/// Relay statistics for a peer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelayStats {
    /// Total messages relayed through this peer
    pub messages_relayed: u64,
    /// Total bytes relayed
    pub bytes_relayed: u64,
    /// Messages successfully delivered
    pub successful_deliveries: u64,
    /// Messages that failed or timed out
    pub failed_deliveries: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// When this peer was last used as a relay
    pub last_used: u64,
}

// ============================================================================
// PHASE 5: REPUTATION TRACKING
// ============================================================================

/// Reputation score for a relay peer
#[derive(Debug, Clone)]
pub struct RelayReputation {
    /// Peer ID
    pub peer_id: PeerId,
    /// Statistics
    pub stats: RelayStats,
    /// Calculated reputation score (0-100)
    pub score: f64,
    /// Is this peer currently considered reliable?
    pub is_reliable: bool,
}

impl RelayReputation {
    /// Calculate reputation score based on statistics
    pub fn calculate_score(&mut self) {
        if self.stats.messages_relayed == 0 {
            self.score = 50.0; // Neutral score for new peers
            self.is_reliable = true;
            return;
        }

        let success_rate =
            self.stats.successful_deliveries as f64 / self.stats.messages_relayed as f64;

        // Score factors:
        // - Success rate (70% weight)
        // - Latency (20% weight - lower is better)
        // - Recency (10% weight - recent usage preferred)

        let success_score = success_rate * 70.0;

        let latency_score = if self.stats.avg_latency_ms < 100 {
            20.0
        } else if self.stats.avg_latency_ms < 500 {
            15.0
        } else if self.stats.avg_latency_ms < 1000 {
            10.0
        } else {
            5.0
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();
        let age_secs = now.saturating_sub(self.stats.last_used);
        let recency_score = if age_secs < 60 {
            10.0
        } else if age_secs < 300 {
            7.0
        } else if age_secs < 3600 {
            5.0
        } else {
            2.0
        };

        self.score = success_score + latency_score + recency_score;
        self.is_reliable = self.score >= 50.0;
    }
}

/// Tracks reputation of all known relay peers
#[derive(Debug, Clone)]
pub struct ReputationTracker {
    reputations: HashMap<PeerId, RelayReputation>,
}

impl Default for ReputationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationTracker {
    pub fn new() -> Self {
        Self {
            reputations: HashMap::new(),
        }
    }

    /// Record a relay attempt
    pub fn record_relay_attempt(
        &mut self,
        peer_id: PeerId,
        success: bool,
        latency_ms: u64,
        bytes: u64,
    ) {
        let rep = self.reputations.entry(peer_id).or_insert(RelayReputation {
            peer_id,
            stats: RelayStats::default(),
            score: 50.0,
            is_reliable: true,
        });

        rep.stats.messages_relayed += 1;
        rep.stats.bytes_relayed += bytes;

        if success {
            rep.stats.successful_deliveries += 1;
        } else {
            rep.stats.failed_deliveries += 1;
        }

        // Update average latency (moving average)
        rep.stats.avg_latency_ms = (rep.stats.avg_latency_ms + latency_ms) / 2;

        rep.stats.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();

        rep.calculate_score();
    }

    /// Get best relay peers (sorted by reputation)
    pub fn best_relays(&self, count: usize) -> Vec<PeerId> {
        let mut peers: Vec<_> = self.reputations.values().collect();
        peers.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.peer_id.to_string().cmp(&b.peer_id.to_string()))
        });

        peers
            .into_iter()
            .filter(|r| r.is_reliable)
            .take(count)
            .map(|r| r.peer_id)
            .collect()
    }

    /// Add a peer as a potential relay (neutral reputation)
    pub fn add_relay(&mut self, peer_id: PeerId) {
        self.reputations
            .entry(peer_id)
            .or_insert_with(|| RelayReputation {
                peer_id,
                stats: RelayStats::default(),
                score: 50.0,
                is_reliable: true,
            });
    }

    /// Check if we have any known relays
    pub fn is_empty(&self) -> bool {
        self.reputations.is_empty()
    }

    /// Get all reputations
    pub fn all_reputations(&self) -> Vec<RelayReputation> {
        self.reputations.values().cloned().collect()
    }
}

// ============================================================================
// PHASE 6: CONTINUOUS RETRY LOGIC
// ============================================================================

/// Retry strategy for message delivery
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Optional maximum number of retry attempts.
    /// `None` means unbounded retries (default).
    pub max_attempts: Option<u32>,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to use exponential backoff
    pub use_exponential_backoff: bool,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: None, // WS1: no terminal retry cap
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
            use_exponential_backoff: true,
        }
    }
}

impl RetryStrategy {
    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if !self.use_exponential_backoff {
            return self.initial_delay;
        }

        let delay_ms =
            self.initial_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);

        let delay = Duration::from_millis(delay_ms as u64);

        delay.min(self.max_delay)
    }

    /// Should we retry after this many attempts?
    pub fn should_retry(&self, attempt: u32) -> bool {
        self.max_attempts.map(|max| attempt < max).unwrap_or(true)
    }
}

/// Tracks ongoing delivery attempts
#[derive(Debug, Clone)]
pub struct DeliveryAttempt {
    /// Message ID
    pub message_id: String,
    /// Target peer
    pub target_peer: PeerId,
    /// Attempt number (0-indexed)
    pub attempt: u32,
    /// Paths tried so far (direct or via relays)
    pub paths_tried: Vec<Vec<PeerId>>,
    /// Last attempt timestamp
    pub last_attempt: u64,
    /// Retry strategy
    pub strategy: RetryStrategy,
}

impl DeliveryAttempt {
    pub fn new(message_id: String, target_peer: PeerId) -> Self {
        Self {
            message_id,
            target_peer,
            attempt: 0,
            paths_tried: Vec::new(),
            last_attempt: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_secs(),
            strategy: RetryStrategy::default(),
        }
    }

    /// Get next retry delay
    pub fn next_retry_delay(&self) -> Duration {
        self.strategy.calculate_delay(self.attempt)
    }

    /// Should we retry?
    pub fn should_retry(&self) -> bool {
        self.strategy.should_retry(self.attempt)
    }

    /// Record a failed attempt via a specific path
    pub fn record_failure(&mut self, path: Vec<PeerId>) {
        self.paths_tried.push(path);
        self.attempt += 1;
        self.last_attempt = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs();
    }
}

/// Multi-path delivery manager
#[derive(Debug)]
pub struct MultiPathDelivery {
    /// Active delivery attempts
    attempts: HashMap<String, DeliveryAttempt>,
    /// Reputation tracker for selecting best paths
    reputation: ReputationTracker,
    /// Recipient-recency signals keyed by (relay, recipient)
    recipient_recency_by_route: HashMap<(PeerId, PeerId), u64>,
    /// Live recipients per relay peer, in FIRST-INSERTION order.
    ///
    /// This is both the per-relay quota index ([`RECENCY_MAX_ROUTES_PER_RELAY`])
    /// and the sole eviction order (see [`Self::prune_recipient_recency`]).
    /// Never holds tombstones.
    ///
    /// Re-review NEW-4: eviction must not be driven by any value the attacker
    /// supplies. An earlier pruner sorted ascending by `seen_at`, which is wire
    /// data clamped only to `now + RECENCY_MAX_CLOCK_SKEW_SECS`. Insertion order
    /// is chosen by US. Re-observing an existing route deliberately does NOT
    /// refresh its position -- if it did, a relay could hold its own entries at
    /// the back of the queue by re-asserting them, which is the same
    /// steerability through a different door.
    ///
    /// Round 4: a GLOBAL first-insertion queue used to sit alongside this one and
    /// drive the ceiling pruner. That is what made the NEW-4 defence
    /// insufficient -- see [`Self::prune_recipient_recency`] -- so it is gone and
    /// eviction is per-relay only.
    recency_routes_by_relay: HashMap<PeerId, VecDeque<PeerId>>,
    /// Latest successful relay path order keyed by (relay, recipient)
    latest_success_by_route: HashMap<(PeerId, PeerId), u64>,
    /// Monotonic sequence for deterministic "latest successful path" tie-breaks
    success_sequence: u64,
}

impl Default for MultiPathDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPathDelivery {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
            reputation: ReputationTracker::new(),
            recipient_recency_by_route: HashMap::new(),
            recency_routes_by_relay: HashMap::new(),
            latest_success_by_route: HashMap::new(),
            success_sequence: 0,
        }
    }

    /// Start a delivery attempt
    pub fn start_delivery(&mut self, message_id: String, target_peer: PeerId) {
        let attempt = DeliveryAttempt::new(message_id.clone(), target_peer);
        self.attempts.insert(message_id, attempt);
    }

    /// Register a potential relay node
    pub fn add_relay(&mut self, peer_id: PeerId) {
        self.reputation.add_relay(peer_id);
    }

    /// Record a recipient-recency signal for a relay candidate.
    ///
    /// `seen_at` must be a unix timestamp (seconds). Newer timestamps overwrite
    /// older values.
    ///
    /// FUTURE-CLAMP (review F12): `recipient_recency` is the PRIMARY descending
    /// sort key in [`Self::ranked_routes`], and the merge here is a monotone
    /// `max`, so a single `u64::MAX` would pin a route at the front of the
    /// ranking permanently -- it can never be lowered by the passage of time,
    /// by honest observation, or by a peer restart. Values beyond
    /// `now + RECENCY_MAX_CLOCK_SKEW_SECS` are clamped rather than rejected so
    /// an honest peer with a slightly fast clock still registers.
    pub fn record_recipient_seen_via_relay(
        &mut self,
        relay_peer: PeerId,
        recipient_peer: PeerId,
        seen_at: u64,
    ) {
        let now = unix_now_secs();
        if seen_at == 0 || seen_at.saturating_add(RECENCY_MAX_AGE_SECS) < now {
            return;
        }
        let ceiling = now.saturating_add(RECENCY_MAX_CLOCK_SKEW_SECS);
        let clamped = seen_at.min(ceiling);
        let key = (relay_peer, recipient_peer);

        // Updating a route we already track changes no structure: no new key,
        // no quota consumption, no prune. If the existing timestamp is in the future
        // (due to past clock skew or injection), reset it to the clamped value.
        if let Some(existing) = self.recipient_recency_by_route.get_mut(&key) {
            if *existing > now {
                *existing = clamped;
            } else {
                *existing = (*existing).max(clamped);
            }
            return;
        }

        self.recipient_recency_by_route.insert(key, clamped);

        // Per-relay quota (NEW-4): a relay that exceeds its allowance evicts its
        // OWN oldest route. Bounded at RECENCY_MAX_ROUTES_PER_RELAY, so this
        // loop runs at most once per insert.
        let per_relay = self.recency_routes_by_relay.entry(relay_peer).or_default();
        per_relay.push_back(recipient_peer);
        while per_relay.len() > RECENCY_MAX_ROUTES_PER_RELAY {
            let Some(evicted) = per_relay.pop_front() else {
                break;
            };
            self.recipient_recency_by_route
                .remove(&(relay_peer, evicted));
        }

        self.prune_recipient_recency();
    }

    /// [`Self::record_recipient_seen_via_relay`] for a timestamp that arrived
    /// over the wire, i.e. one chosen by whoever sent it.
    ///
    /// Returns `true` if the signal was recorded. In addition to the
    /// future-clamp, this drops values older than [`RECENCY_MAX_AGE_SECS`] --
    /// the same 7-day horizon the CLI ledger uses -- and the `0` that an entry
    /// with no `last_seen` serialises to. A week-old sighting is not evidence
    /// about a route's current usefulness, and accepting it only lets a peer
    /// inject ranking weight for routes it has no live knowledge of.
    pub fn record_recipient_seen_via_relay_from_wire(
        &mut self,
        relay_peer: PeerId,
        recipient_peer: PeerId,
        seen_at: u64,
    ) -> bool {
        let now = unix_now_secs();
        if seen_at == 0 || seen_at.saturating_add(RECENCY_MAX_AGE_SECS) < now {
            return false;
        }
        self.record_recipient_seen_via_relay(relay_peer, recipient_peer, seen_at);
        true
    }

    /// Bound `recipient_recency_by_route`, which is keyed by a pair of
    /// remote-supplied peer ids and previously grew without limit.
    ///
    /// WHY THIS IS PROPORTIONAL AND NOT FIFO (re-review round 4). The previous
    /// version popped a GLOBAL first-insertion queue down to the low-water mark.
    /// That made the per-relay quota decorative at the scale that matters: the
    /// quota bounds one relay at 64 routes, but `max_established_incoming` is 64
    /// (`behaviour.rs`), so 64 attacker identities hold 64 * 64 = 4096 routes --
    /// exactly [`RECENCY_MAX_TRACKED_ROUTES`]. The honest route, being the
    /// OLDEST, was then the FIRST thing a global FIFO evicted. The NEW-4 test
    /// was sized at 8 identities (512 slots) and so never crossed the ceiling it
    /// claimed to test; at 64 it fails against the old pruner.
    ///
    /// So eviction is max-min fair: repeatedly take from whoever holds the most.
    /// Concretely, water-fill to a level `t` such that trimming every relay down
    /// to `t` (plus a remainder taken one-each from relays sitting exactly at
    /// `t + 1`) removes precisely the number of routes needed. A relay holding
    /// one route is only ever touched when EVERY relay holds one route, i.e.
    /// when there are [`RECENCY_PRUNE_TARGET_ROUTES`] distinct relay peer ids --
    /// and a relay peer id costs a Noise handshake, so that is 3072 handshakes
    /// against a 64-slot inbound limit, not a message flood. Within a relay,
    /// order is its own first-insertion order.
    ///
    /// Properties this must keep, both learned the hard way:
    ///
    /// 1. AMORTISED (NEW-3). It runs on the swarm `select!` thread, once per
    ///    inserted wire entry. Crossing the ceiling drops all the way to
    ///    [`RECENCY_PRUNE_TARGET_ROUTES`], so the cost is spread over the next
    ///    1024 inserts. The histogram is `O(relays)` with counts bounded by
    ///    [`RECENCY_MAX_ROUTES_PER_RELAY`], and there is no sort.
    /// 2. NOT ATTACKER-STEERABLE (NEW-4). Nothing here reads `seen_at` or any
    ///    other value that arrived over the wire. The only inputs are how many
    ///    routes each relay holds and the order that relay inserted them.
    fn prune_recipient_recency(&mut self) {
        let total = self.recipient_recency_by_route.len();
        if total <= RECENCY_MAX_TRACKED_ROUTES {
            return;
        }
        let mut to_drop = total - RECENCY_PRUNE_TARGET_ROUTES;

        // Histogram of per-relay route counts. Counts are bounded by
        // RECENCY_MAX_ROUTES_PER_RELAY, so this is a fixed-width array.
        let mut histogram = [0usize; RECENCY_MAX_ROUTES_PER_RELAY + 1];
        for recipients in self.recency_routes_by_relay.values() {
            let count = recipients.len().min(RECENCY_MAX_ROUTES_PER_RELAY);
            histogram[count] += 1;
        }

        // `excess_above(t)` = how many routes disappear if every relay is
        // trimmed down to `t`. Non-increasing in `t`, and `excess_above(0)` is
        // the whole map, which always exceeds `to_drop`.
        let excess_above = |level: usize| -> usize {
            (level + 1..=RECENCY_MAX_ROUTES_PER_RELAY)
                .map(|c| histogram[c] * (c - level))
                .sum()
        };
        // Largest level whose excess still covers `to_drop`. Trimming to
        // `t_star + 1` therefore removes strictly fewer than `to_drop`, and the
        // shortfall is made up one route at a time from relays at that level --
        // which is what makes the eviction count exact instead of overshooting
        // by a whole level.
        let mut t_star = 0usize;
        for t in (0..=RECENCY_MAX_ROUTES_PER_RELAY).rev() {
            if excess_above(t) >= to_drop {
                t_star = t;
                break;
            }
        }
        let trim_to = (t_star + 1).min(RECENCY_MAX_ROUTES_PER_RELAY);

        let mut relays_at_trim_level: Vec<PeerId> = Vec::new();
        let mut evicted: Vec<(PeerId, PeerId)> = Vec::new();

        for (relay, recipients) in self.recency_routes_by_relay.iter_mut() {
            while recipients.len() > trim_to && to_drop > 0 {
                let Some(recipient) = recipients.pop_front() else {
                    break;
                };
                evicted.push((*relay, recipient));
                to_drop -= 1;
            }
            if recipients.len() == trim_to && trim_to > 0 {
                relays_at_trim_level.push(*relay);
            }
        }

        // Remainder: one route each from relays still at the trim level.
        //
        // Sorted by peer id purely for DETERMINISM. `HashMap` iteration order
        // differs between two maps even in the same process (each `RandomState`
        // gets its own seed), and the NEW-4 property test asserts that the
        // surviving key set depends on the key sequence alone -- an unordered
        // remainder would make that property flap. The ordering is not a
        // security property: it decides at most `to_drop - excess_above(trim_to)`
        // evictions, every candidate is already at the same route count, and the
        // most a ground peer id could buy is keeping one of its own 64 routes.
        relays_at_trim_level.sort_unstable();
        for relay in relays_at_trim_level {
            if to_drop == 0 {
                break;
            }
            if let Some(recipients) = self.recency_routes_by_relay.get_mut(&relay) {
                if let Some(recipient) = recipients.pop_front() {
                    evicted.push((relay, recipient));
                    to_drop -= 1;
                }
            }
        }

        for key in &evicted {
            self.recipient_recency_by_route.remove(key);
        }
        self.recency_routes_by_relay
            .retain(|_, recipients| !recipients.is_empty());
    }

    /// Recency value currently held for `(relay_peer, recipient_peer)`, if any.
    ///
    /// Read-only observability for the bounds and eviction-order invariants of
    /// `recipient_recency_by_route` (re-review NEW-3/NEW-4), which are otherwise
    /// only visible through `ranked_routes` and therefore only for peers that
    /// also have a reputation entry.
    pub fn recipient_recency(&self, relay_peer: &PeerId, recipient_peer: &PeerId) -> Option<u64> {
        self.recipient_recency_by_route
            .get(&(*relay_peer, *recipient_peer))
            .copied()
    }

    /// Number of `(relay, recipient)` routes currently tracked. Bounded by
    /// [`RECENCY_MAX_TRACKED_ROUTES`].
    pub fn tracked_recency_routes(&self) -> usize {
        self.recipient_recency_by_route.len()
    }

    /// Number of tracked routes attributed to one relay peer. Bounded by
    /// [`RECENCY_MAX_ROUTES_PER_RELAY`].
    pub fn tracked_recency_routes_for_relay(&self, relay_peer: &PeerId) -> usize {
        self.recency_routes_by_relay
            .get(relay_peer)
            .map(|recipients| recipients.len())
            .unwrap_or(0)
    }

    /// Record a "seen now" recipient-recency signal.
    pub fn record_recipient_seen_now(&mut self, relay_peer: PeerId, recipient_peer: PeerId) {
        self.record_recipient_seen_via_relay(relay_peer, recipient_peer, unix_now_secs());
    }

    /// Deterministic ranked routes: direct-first, then relay ranking policy.
    pub fn ranked_routes(&self, target: &PeerId, count: usize) -> Vec<RankedRoute> {
        if count == 0 {
            return Vec::new();
        }

        let mut routes = Vec::with_capacity(count);
        routes.push(RankedRoute {
            path: vec![*target],
            reason_code: ROUTE_REASON_DIRECT_FIRST,
            recipient_recency: 0,
            relay_success_score: 0.0,
            latest_success_order: 0,
        });

        #[derive(Debug)]
        struct RelayCandidate {
            relay_peer: PeerId,
            relay_key: String,
            recipient_recency: u64,
            relay_success_score: f64,
            latest_success_order: u64,
        }

        let mut relays: Vec<RelayCandidate> = self
            .reputation
            .reputations
            .values()
            .filter(|rep| rep.is_reliable && rep.peer_id != *target)
            .map(|rep| {
                let relay_peer = rep.peer_id;
                RelayCandidate {
                    relay_peer,
                    relay_key: relay_peer.to_string(),
                    recipient_recency: self
                        .recipient_recency_by_route
                        .get(&(relay_peer, *target))
                        .copied()
                        .unwrap_or(0),
                    relay_success_score: rep.score,
                    latest_success_order: self
                        .latest_success_by_route
                        .get(&(relay_peer, *target))
                        .copied()
                        .unwrap_or(0),
                }
            })
            .collect();

        relays.sort_by(|a, b| {
            b.recipient_recency
                .cmp(&a.recipient_recency)
                .then_with(|| b.relay_success_score.total_cmp(&a.relay_success_score))
                .then_with(|| b.latest_success_order.cmp(&a.latest_success_order))
                .then_with(|| a.relay_key.cmp(&b.relay_key))
        });

        for relay in relays.into_iter().take(count.saturating_sub(1)) {
            let reason_code = if relay.recipient_recency > 0 {
                ROUTE_REASON_RELAY_RECENCY_SUCCESS
            } else if relay.latest_success_order > 0 {
                ROUTE_REASON_RELAY_TIEBREAK_LAST_SUCCESS
            } else if relay.relay_success_score > 0.0 {
                ROUTE_REASON_RELAY_SUCCESS_SCORE
            } else {
                ROUTE_REASON_RELAY_TIEBREAK_PEER_ID
            };

            routes.push(RankedRoute {
                path: vec![relay.relay_peer, *target],
                reason_code,
                recipient_recency: relay.recipient_recency,
                relay_success_score: relay.relay_success_score,
                latest_success_order: relay.latest_success_order,
            });
        }

        routes
    }

    /// Get best paths to try (direct + relay options)
    pub fn get_best_paths(&self, target: &PeerId, count: usize) -> Vec<Vec<PeerId>> {
        self.ranked_routes(target, count)
            .into_iter()
            .map(|route| route.path)
            .collect()
    }

    /// Record delivery success
    pub fn record_success(&mut self, message_id: &str, path: Vec<PeerId>, latency_ms: u64) {
        // Remove from active attempts
        self.attempts.remove(message_id);

        // Update reputation for relays in the path
        if path.len() > 1 {
            self.success_sequence = self.success_sequence.saturating_add(1);
            let latest_success_order = self.success_sequence;
            let target_peer = *path.last().unwrap_or(&path[0]);
            for relay in &path[..path.len() - 1] {
                self.reputation
                    .record_relay_attempt(*relay, true, latency_ms, 1024);
                self.latest_success_by_route
                    .insert((*relay, target_peer), latest_success_order);
            }
        }
    }

    /// Record delivery failure
    pub fn record_failure(&mut self, message_id: &str, path: Vec<PeerId>) {
        if let Some(attempt) = self.attempts.get_mut(message_id) {
            attempt.record_failure(path.clone());

            // Update reputation for relays that failed
            if path.len() > 1 {
                for relay in &path[..path.len() - 1] {
                    self.reputation
                        .record_relay_attempt(*relay, false, 10000, 0);
                }
            }
        }
    }

    /// Converge delivery state for a message once a final delivery marker is observed.
    ///
    /// Returns `true` when an active retry attempt was cleared.
    pub fn converge_delivery(&mut self, message_id: &str) -> bool {
        self.attempts.remove(message_id).is_some()
    }

    /// Get a specific pending delivery attempt by message id.
    pub fn delivery_attempt(&self, message_id: &str) -> Option<&DeliveryAttempt> {
        self.attempts.get(message_id)
    }

    /// Get pending delivery attempts
    pub fn pending_attempts(&self) -> Vec<&DeliveryAttempt> {
        self.attempts.values().collect()
    }

    /// Get reputation tracker
    pub fn reputation(&self) -> &ReputationTracker {
        &self.reputation
    }

    /// Get best relay peers (sorted by reputation)
    pub fn best_relays(&self, count: usize) -> Vec<PeerId> {
        self.reputation.best_relays(count)
    }
}

// ============================================================================
// PHASE 4: MESH-BASED DISCOVERY
// ============================================================================

/// Bootstrap capability - any node can help others join the network
#[derive(Debug, Clone)]
pub struct BootstrapCapability {
    /// Peers we know about (potential bootstrap candidates)
    pub known_peers: Vec<PeerId>,
    /// Last time we updated our peer list
    pub last_update: u64,
}

impl Default for BootstrapCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl BootstrapCapability {
    pub fn new() -> Self {
        Self {
            known_peers: Vec::new(),
            last_update: 0,
        }
    }

    /// Add a peer as a potential bootstrap node
    pub fn add_peer(&mut self, peer_id: PeerId) {
        if !self.known_peers.contains(&peer_id) {
            self.known_peers.push(peer_id);
            self.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_secs();
        }
    }

    /// Get bootstrap candidates (all stable peers)
    pub fn get_bootstrap_candidates(&self) -> &[PeerId] {
        &self.known_peers
    }

    /// Can this node help others bootstrap?
    pub fn can_bootstrap_others(&self) -> bool {
        !self.known_peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_calculation() {
        let mut rep = RelayReputation {
            peer_id: PeerId::random(),
            stats: RelayStats {
                messages_relayed: 100,
                successful_deliveries: 95,
                failed_deliveries: 5,
                avg_latency_ms: 50,
                ..Default::default()
            },
            score: 0.0,
            is_reliable: false,
        };

        rep.calculate_score();

        assert!(
            rep.score > 80.0,
            "High success rate should yield high score"
        );
        assert!(rep.is_reliable, "Should be marked as reliable");
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy::default();

        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        assert!(strategy.calculate_delay(1) > Duration::from_millis(100));
        assert!(strategy.calculate_delay(5) < strategy.max_delay);

        assert!(strategy.should_retry(5));
        assert!(strategy.should_retry(100));

        let bounded = RetryStrategy {
            max_attempts: Some(3),
            ..RetryStrategy::default()
        };
        assert!(bounded.should_retry(2));
        assert!(!bounded.should_retry(3));
    }

    #[test]
    fn test_multi_path_delivery() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let message_id = "test-message-123".to_string();

        delivery.start_delivery(message_id.clone(), target);

        let paths = delivery.get_best_paths(&target, 3);
        assert!(!paths.is_empty(), "Should provide at least direct path");
        assert_eq!(paths[0], vec![target], "First path should be direct");

        delivery.record_failure(&message_id, vec![target]);

        let pending = delivery.pending_attempts();
        assert_eq!(pending.len(), 1, "Should have one pending attempt");
    }

    #[test]
    fn test_converge_delivery_clears_pending_retry_attempt() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let message_id = "converge-message-123".to_string();

        delivery.start_delivery(message_id.clone(), target);
        assert_eq!(delivery.pending_attempts().len(), 1);

        let cleared = delivery.converge_delivery(&message_id);
        assert!(cleared);
        assert!(delivery.delivery_attempt(&message_id).is_none());
        assert_eq!(delivery.pending_attempts().len(), 0);
    }

    // ------------------------------------------------------------------
    // F12 -- wire `last_seen` is attacker-controlled ranking input
    // ------------------------------------------------------------------

    /// Without the clamp, `u64::MAX` merged with a monotone `max` pins the
    /// attacker's route at the head of `ranked_routes` forever: it can never be
    /// lowered by time, honest observation or a peer restart.
    #[test]
    fn future_recency_cannot_pin_a_route_forever() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let attacker_relay = PeerId::random();
        let honest_relay = PeerId::random();

        delivery.record_recipient_seen_via_relay_from_wire(attacker_relay, target, u64::MAX);
        delivery.record_recipient_seen_now(honest_relay, target);

        let attacker_recency = delivery
            .recipient_recency_by_route
            .get(&(attacker_relay, target))
            .copied()
            .unwrap_or(0);
        let honest_recency = delivery
            .recipient_recency_by_route
            .get(&(honest_relay, target))
            .copied()
            .unwrap_or(0);

        assert_ne!(
            attacker_recency,
            u64::MAX,
            "u64::MAX was stored verbatim as a recency score"
        );
        assert!(
            attacker_recency <= unix_now_secs() + RECENCY_MAX_CLOCK_SKEW_SECS,
            "recency {} exceeds now + max skew",
            attacker_recency
        );
        assert!(
            honest_recency + RECENCY_MAX_CLOCK_SKEW_SECS >= attacker_recency,
            "an honest live sighting is still hopelessly behind the clamped attacker value"
        );
    }

    /// A modest clock skew from an honest peer is tolerated, not discarded.
    #[test]
    fn small_clock_skew_is_clamped_not_dropped() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let relay = PeerId::random();
        let slightly_ahead = unix_now_secs() + 30;

        assert!(delivery.record_recipient_seen_via_relay_from_wire(relay, target, slightly_ahead));
        let stored = delivery
            .recipient_recency_by_route
            .get(&(relay, target))
            .copied()
            .unwrap_or(0);
        assert_eq!(stored, slightly_ahead);
    }

    /// Anything older than the 7-day horizon (or the `0` an entry with no
    /// `last_seen` serialises to) carries no information about a route's
    /// current usefulness and must not enter the ranking at all.
    #[test]
    fn stale_wire_recency_is_rejected() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let relay = PeerId::random();
        let ancient = unix_now_secs() - RECENCY_MAX_AGE_SECS - 60;

        assert!(!delivery.record_recipient_seen_via_relay_from_wire(relay, target, ancient));
        assert!(!delivery.record_recipient_seen_via_relay_from_wire(relay, target, 0));
        assert!(delivery.recipient_recency_by_route.is_empty());
    }

    /// The map is keyed entirely by remote-supplied peer ids, so it needs a
    /// ceiling.
    #[test]
    fn recipient_recency_map_is_bounded() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        for i in 0..(RECENCY_MAX_TRACKED_ROUTES + 500) {
            delivery.record_recipient_seen_via_relay(
                PeerId::random(),
                PeerId::random(),
                now - (i as u64 % 1000),
            );
        }
        assert!(delivery.recipient_recency_by_route.len() <= RECENCY_MAX_TRACKED_ROUTES);
    }

    // ------------------------------------------------------------------
    // NEW-3 -- the F12 pruner reintroduced the F4 event-loop DoS
    // ------------------------------------------------------------------

    /// Past the ceiling, the previous pruner collected the entire map into a
    /// `Vec` and sorted it on EVERY insert, to remove exactly one entry. This
    /// runs on the swarm `select!` thread that also owns the swarm poll and the
    /// dial sweep.
    ///
    /// The bound is deliberately loose (a wall-clock assertion in a debug-build
    /// test suite on a shared machine cannot be tight); the point is the
    /// difference between amortised-constant and n log n per insert, which at
    /// this size is roughly two orders of magnitude.
    #[test]
    fn pruning_is_amortised_not_per_insert() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let inserts = RECENCY_MAX_TRACKED_ROUTES * 8;

        let started = std::time::Instant::now();
        for i in 0..inserts {
            // Distinct relay per insert so the per-relay quota is not what is
            // being measured here.
            delivery.record_recipient_seen_via_relay(
                PeerId::random(),
                PeerId::random(),
                now - (i as u64 % 600),
            );
        }
        let elapsed = started.elapsed();

        assert!(
            delivery.recipient_recency_by_route.len() <= RECENCY_MAX_TRACKED_ROUTES,
            "map exceeded its ceiling"
        );
        assert!(
            delivery.recency_routes_by_relay.len() <= RECENCY_MAX_TRACKED_ROUTES,
            "the per-relay eviction index grew without bound: {}",
            delivery.recency_routes_by_relay.len()
        );
        assert_eq!(
            delivery
                .recency_routes_by_relay
                .values()
                .map(|r| r.len())
                .sum::<usize>(),
            delivery.recipient_recency_by_route.len(),
            "the per-relay eviction index drifted out of sync with the map"
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "{inserts} inserts took {elapsed:?} on the event-loop thread"
        );
    }

    /// Crossing the ceiling must drop to the low-water mark, so the next
    /// `RECENCY_MAX_TRACKED_ROUTES - RECENCY_PRUNE_TARGET_ROUTES` inserts do no
    /// pruning work at all.
    #[test]
    fn prune_drops_to_the_low_water_mark() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        for _ in 0..=RECENCY_MAX_TRACKED_ROUTES {
            delivery.record_recipient_seen_via_relay(PeerId::random(), PeerId::random(), now);
        }
        assert_eq!(
            delivery.recipient_recency_by_route.len(),
            RECENCY_PRUNE_TARGET_ROUTES,
            "pruner trimmed to the ceiling instead of the low-water mark, so \
             every subsequent insert prunes again"
        );
    }

    // ------------------------------------------------------------------
    // NEW-4 -- eviction must not be steerable by wire data
    // ------------------------------------------------------------------

    /// The previous pruner sorted ascending by `seen_at` and evicted the front.
    /// `seen_at` is wire data, clamped only to `now + 300s` and kept verbatim
    /// below that, so a handful of connected peers asserting `now + 300` filled
    /// the map and evicted every honest route -- and `recipient_recency_by_route`
    /// is the PRIMARY descending sort key in `ranked_routes`, so the ranking
    /// became entirely theirs. That is cheaper than the `u64::MAX` pin F12 was
    /// written to stop.
    ///
    /// The defence is the per-relay quota PLUS proportional eviction: every
    /// route key needs a relay peer id, a peer id costs a Noise handshake, and
    /// when the global ceiling is crossed the routes come off whoever holds the
    /// most.
    ///
    /// SIZING IS THE TEST (re-review round 4). This used to run 8 attacker
    /// identities x 64 quota = 512 slots against a 4096 ceiling, so it never
    /// crossed the threshold it claimed to test and passed against a global-FIFO
    /// pruner that evicted the honest route first. `max_established_incoming` is
    /// 64 (`behaviour.rs`), so the real bound on concurrent attacker identities
    /// is 64, and 64 x 64 = 4096 = `RECENCY_MAX_TRACKED_ROUTES` exactly. At that
    /// size the honest route is the oldest key in the map and a FIFO pruner
    /// evicts it on the very first prune.
    #[test]
    fn future_dated_flood_from_bounded_identities_cannot_evict_honest_routes() {
        // One identity per inbound connection slot. 64 * 64 == the ceiling.
        const ATTACKER_IDENTITIES: usize = 64;
        assert_eq!(
            ATTACKER_IDENTITIES * RECENCY_MAX_ROUTES_PER_RELAY,
            RECENCY_MAX_TRACKED_ROUTES,
            "the flood must be sized to actually reach the ceiling, or this test \
             cannot fail"
        );

        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let target = PeerId::random();

        // An honest, live sighting recorded first -- i.e. the one an
        // insertion-ordered pruner would reach first, so this is not passing by
        // accident of ordering.
        let honest_relay = PeerId::random();
        delivery.record_recipient_seen_now(honest_relay, target);
        assert!(delivery
            .recipient_recency_by_route
            .contains_key(&(honest_relay, target)));

        // Every identity sends far more than the whole map could hold, all at
        // the maximum timestamp the clamp permits.
        let attackers: Vec<PeerId> = (0..ATTACKER_IDENTITIES).map(|_| PeerId::random()).collect();
        let future = now + RECENCY_MAX_CLOCK_SKEW_SECS;
        for i in 0..(RECENCY_MAX_TRACKED_ROUTES * 4) {
            delivery.record_recipient_seen_via_relay_from_wire(
                attackers[i % attackers.len()],
                PeerId::random(),
                future,
            );
        }

        assert!(
            delivery
                .recipient_recency_by_route
                .contains_key(&(honest_relay, target)),
            "the honest route was evicted by a future-dated flood; the primary \
             ranking key is now attacker-controlled"
        );
        assert!(
            delivery.recipient_recency_by_route.len() <= RECENCY_MAX_TRACKED_ROUTES,
            "{ATTACKER_IDENTITIES} identities occupied {} slots",
            delivery.recipient_recency_by_route.len()
        );
        // The honest relay holds its single route; no attacker holds more than
        // its fair share of what is left.
        let honest_held = delivery
            .recipient_recency_by_route
            .keys()
            .filter(|(relay, _)| *relay == honest_relay)
            .count();
        assert_eq!(honest_held, 1);
    }

    /// The same attack with a flood that is only just over the ceiling, so the
    /// pruner runs exactly once. Guards against a fix that only works because
    /// repeated prunes eventually rebalance.
    #[test]
    fn a_single_prune_pass_does_not_sacrifice_the_smallest_holder() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let honest_relay = PeerId::random();
        let target = PeerId::random();
        delivery.record_recipient_seen_now(honest_relay, target);

        let attackers: Vec<PeerId> = (0..64).map(|_| PeerId::random()).collect();
        let mut inserted = 1usize;
        'outer: for attacker in &attackers {
            for _ in 0..RECENCY_MAX_ROUTES_PER_RELAY {
                delivery.record_recipient_seen_via_relay(*attacker, PeerId::random(), now);
                inserted += 1;
                if inserted > RECENCY_MAX_TRACKED_ROUTES {
                    break 'outer;
                }
            }
        }

        assert_eq!(
            delivery.recipient_recency_by_route.len(),
            RECENCY_PRUNE_TARGET_ROUTES,
            "one prune pass must reach the low-water mark"
        );
        assert!(
            delivery
                .recipient_recency_by_route
                .contains_key(&(honest_relay, target)),
            "the single-route honest relay was evicted before relays holding 64"
        );
    }

    /// One relay must not be able to occupy the whole map, however many
    /// recipients it claims to have seen.
    #[test]
    fn one_relay_cannot_exceed_its_route_quota() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let greedy = PeerId::random();

        for _ in 0..(RECENCY_MAX_ROUTES_PER_RELAY * 20) {
            delivery.record_recipient_seen_via_relay_from_wire(greedy, PeerId::random(), now);
        }

        let held = delivery
            .recipient_recency_by_route
            .keys()
            .filter(|(relay, _)| *relay == greedy)
            .count();
        assert_eq!(
            held, RECENCY_MAX_ROUTES_PER_RELAY,
            "one relay holds {held} of {RECENCY_MAX_TRACKED_ROUTES} route slots"
        );
        assert_eq!(
            delivery
                .recency_routes_by_relay
                .get(&greedy)
                .map(|d| d.len()),
            Some(RECENCY_MAX_ROUTES_PER_RELAY),
            "the per-relay index drifted out of sync with the map"
        );
    }

    /// A relay at quota evicts only its own routes, never a neighbour's.
    #[test]
    fn relay_quota_eviction_does_not_touch_other_relays() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let victim_relay = PeerId::random();
        let victim_target = PeerId::random();
        delivery.record_recipient_seen_via_relay(victim_relay, victim_target, now);

        let greedy = PeerId::random();
        for _ in 0..(RECENCY_MAX_ROUTES_PER_RELAY * 5) {
            delivery.record_recipient_seen_via_relay_from_wire(greedy, PeerId::random(), now);
        }

        assert!(delivery
            .recipient_recency_by_route
            .contains_key(&(victim_relay, victim_target)));
    }

    /// Re-asserting a route we already hold must not move it to the back of the
    /// eviction queue -- that would restore steerability through the update
    /// path instead of the insert path.
    #[test]
    fn reasserting_a_known_route_does_not_refresh_its_eviction_position() {
        let mut delivery = MultiPathDelivery::new();
        let now = unix_now_secs();
        let relay = PeerId::random();
        let target = PeerId::random();

        let other = PeerId::random();
        delivery.record_recipient_seen_via_relay(relay, target, now - 10);
        delivery.record_recipient_seen_via_relay(relay, other, now - 10);
        let order_len_before = delivery
            .recency_routes_by_relay
            .get(&relay)
            .map(|d| d.len())
            .unwrap_or_default();
        for _ in 0..50 {
            delivery.record_recipient_seen_via_relay(relay, target, now);
        }

        assert_eq!(
            delivery
                .recency_routes_by_relay
                .get(&relay)
                .map(|d| d.len())
                .unwrap_or_default(),
            order_len_before,
            "a repeated observation queued a duplicate eviction-order entry"
        );
        assert_eq!(
            delivery
                .recency_routes_by_relay
                .get(&relay)
                .and_then(|d| d.front()),
            Some(&target),
            "re-asserting moved the route out of the eviction front"
        );
        // The value still updates -- this must not have broken the recency
        // signal itself.
        assert_eq!(
            delivery
                .recipient_recency_by_route
                .get(&(relay, target))
                .copied(),
            Some(now)
        );
    }
}
