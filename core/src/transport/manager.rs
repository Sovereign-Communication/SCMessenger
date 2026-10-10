//! Transport Manager — multiplexes multiple transports
//!
//! This module coordinates transport abstraction and intelligently selects
//! the best transport for each peer based on capabilities and connection state.
//!
//! # Multi-Hop Routing Integration
//!
//! The TransportManager integrates with the DSPy multi-hop recall module for
//! intelligent path selection across multiple transport types.

use crate::dspy::modules::{DSPyModule, MultiHopRecall};
use crate::transport::abstraction::{
    TransportCapabilities, TransportError, TransportEvent, TransportType,
};
use crate::transport::escalation::EscalationEngine;
use crate::transport::health::TransportHealthMonitor;
use crate::transport::observation::AddressObserver;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};
use web_time::{Duration, Instant, SystemTime};

/// State of a registered transport
#[derive(Debug, Clone)]
pub struct TransportState {
    /// Whether this transport is currently running
    pub running: bool,
    /// Set of connected peer IDs via this transport
    pub connected_peers: HashSet<[u8; 32]>,
    /// Capabilities of this transport
    pub capabilities: TransportCapabilities,
}

impl TransportState {
    /// Create a new transport state
    pub fn new(capabilities: TransportCapabilities) -> Self {
        Self {
            running: false,
            connected_peers: HashSet::new(),
            capabilities,
        }
    }
}

/// Pending outgoing data
#[derive(Debug, Clone)]
pub struct PendingSend {
    /// Target peer ID
    pub peer_id: [u8; 32],
    /// Data to send
    pub data: Vec<u8>,
    /// Priority (0-255, higher is more important)
    pub priority: u8,
    /// Preferred transport for this send (if None, any is acceptable)
    pub preferred_transport: Option<TransportType>,
    /// When this was queued
    pub created_at: SystemTime,
}

/// Priority queue of outgoing data
#[derive(Debug, Clone)]
pub struct OutgoingQueue {
    items: Vec<PendingSend>,
}

impl OutgoingQueue {
    /// Create a new outgoing queue
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the queue (maintains priority order)
    pub fn enqueue(&mut self, item: PendingSend) {
        self.items.push(item);
        // Sort so highest priority items are first
        self.items
            .sort_by_key(|item| std::cmp::Reverse(item.priority));
    }

    /// Dequeue the highest priority item
    pub fn dequeue(&mut self) -> Option<PendingSend> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    /// Get the count of pending items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all pending sends
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for OutgoingQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RECONNECTION WITH EXPONENTIAL BACKOFF
// ============================================================================

/// Minimum backoff interval for reconnection attempts
const RECONNECT_BASE_INTERVAL: Duration = Duration::from_secs(1);

/// Scale for the stability-derived backoff ceiling. The live ceiling is this
/// value times a factor in [1, 4] that grows as THIS peer's recent reconnect
/// success rate falls (see `ceiling_for_success_rate`). Reconnection NEVER
/// gives up: the ceiling only spaces attempts, and wake credits pull them
/// forward.
const RECONNECT_CEILING_BASE: Duration = Duration::from_secs(60);

/// EWMA weight of the newest reconnect outcome in the stability estimate.
const RECONNECT_STABILITY_ALPHA: f64 = 0.2;

/// Consecutive failures after which a peer is reported Dormant (label only).
const RECONNECT_DORMANT_AFTER: u32 = 3;

/// Backoff multiplier per failed attempt
const RECONNECT_BACKOFF_MULTIPLIER: u32 = 2;

/// Number of consecutive ticks a peer must stay past the staleness window
/// before it is pruned and synthetically disconnected. Guards against
/// pruning idle-but-healthy links that see traffic between ticks (review F2,
/// TRANSPORT_FAILOVER_AUDIT_QWENPAID_2026-08-05).
const STALE_CONFIRM_TICKS: u32 = 3;

/// Minimum interval between successive reconnection dials (stagger)
const RECONNECT_STAGGER_INTERVAL: Duration = Duration::from_millis(200);

/// First 4 bytes of a peer id as hex, for log markers.
fn short_id(peer_id: &[u8; 32]) -> String {
    peer_id[..4].iter().map(|b| format!("{b:02x}")).collect()
}

/// Per-peer reconnection state with exponential backoff
#[derive(Debug, Clone)]
pub struct ReconnectionState {
    /// The peer we want to reconnect to
    pub peer_id: [u8; 32],
    /// Last known transport(s) for this peer
    pub last_transports: HashSet<TransportType>,
    /// Last known address bytes (opaque to manager, passed through to transport)
    pub last_addr: Vec<u8>,
    /// Number of consecutive failed reconnection attempts
    pub failures: u32,
    /// When the next reconnection attempt is allowed
    pub next_attempt_at: SystemTime,
    /// When this peer was first lost
    pub disconnected_at: SystemTime,
    /// Dormant: the next attempt waits for a wake event or the jittered
    /// backoff. Never terminal.
    pub dormant: bool,
    /// Per-peer EWMA of reconnect outcomes in [0, 1]. Starts optimistic: a
    /// queued peer was connected until it was lost. Only THIS peer's
    /// failures stretch its own ceiling.
    success_rate: f64,
    /// Wake-credit buckets, one per trigger (bounded by the trigger count).
    wake_buckets: HashMap<super::dial_policy::WakeTrigger, super::dial_policy::WakeBucket>,
}

impl ReconnectionState {
    fn new(peer_id: [u8; 32], transports: HashSet<TransportType>, addr: Vec<u8>) -> Self {
        Self {
            peer_id,
            last_transports: transports,
            last_addr: addr,
            failures: 0,
            next_attempt_at: SystemTime::now() + RECONNECT_BASE_INTERVAL,
            disconnected_at: SystemTime::now(),
            dormant: false,
            success_rate: 1.0,
            wake_buckets: HashMap::new(),
        }
    }

    /// Backoff ceiling for this peer from its own stability estimate.
    fn ceiling(&self) -> Duration {
        let rate = if self.success_rate.is_finite() {
            self.success_rate.clamp(0.0, 1.0)
        } else {
            0.5
        };
        RECONNECT_CEILING_BASE.mul_f64(1.0 + 3.0 * (1.0 - rate))
    }

    /// Fold a reconnect outcome into this peer's stability estimate.
    fn record_outcome(&mut self, success: bool) {
        let sample = if success { 1.0 } else { 0.0 };
        self.success_rate = (1.0 - RECONNECT_STABILITY_ALPHA) * self.success_rate
            + RECONNECT_STABILITY_ALPHA * sample;
    }

    /// Nominal backoff interval at the default ceiling.
    #[cfg(test)]
    fn backoff_interval(&self) -> Duration {
        self.backoff_interval_with(RECONNECT_CEILING_BASE)
    }

    /// Nominal backoff interval (1 s doubling) capped at `ceiling`.
    fn backoff_interval_with(&self, ceiling: Duration) -> Duration {
        let base = RECONNECT_BASE_INTERVAL.as_millis() as u64;
        let multiplier = RECONNECT_BACKOFF_MULTIPLIER
            .checked_pow(self.failures)
            .unwrap_or(u32::MAX) as u64;
        let interval_ms = base.saturating_mul(multiplier);
        Duration::from_millis(interval_ms).min(ceiling.max(RECONNECT_BASE_INTERVAL))
    }

    /// Record a failed reconnection attempt, advancing the backoff
    pub fn record_failure(&mut self) {
        self.record_failure_with_ceiling(RECONNECT_CEILING_BASE);
    }

    /// Record a failed attempt with a stability-derived ceiling. The next
    /// attempt is scheduled with full jitter (uniform 0.5..1.0 of nominal).
    /// There is no failure limit: the peer stays queued until it connects
    /// or is removed as a target.
    pub fn record_failure_with_ceiling(&mut self, ceiling: Duration) {
        self.failures = self.failures.saturating_add(1);
        let wait = super::dial_policy::jittered(self.backoff_interval_with(ceiling));
        self.next_attempt_at = SystemTime::now() + wait;
        if self.failures >= RECONNECT_DORMANT_AFTER && !self.dormant {
            self.dormant = true;
            info!(
                "[DIAL] dormant peer={} reason=reconnect_failures next_wake=event|backoff_ms={}",
                short_id(&self.peer_id),
                wait.as_millis()
            );
        }
    }

    /// Wake event: a ONE-SHOT credit. The next attempt is made due no later
    /// than one floor interval from now, but the backoff ladder is NOT reset
    /// (`failures` is kept), so a failed woken attempt resumes the ladder.
    /// Credits are limited by a per-trigger token bucket whose refill period
    /// is derived from the observed event frequency (see
    /// `dial_policy::WakeBucket`). Returns true if a credit was granted.
    pub fn wake(&mut self, trigger: super::dial_policy::WakeTrigger) -> bool {
        if self.failures == 0 && !self.dormant {
            return false;
        }
        let now = Instant::now();
        let ceiling = self.ceiling();
        let admitted = self
            .wake_buckets
            .entry(trigger)
            .or_insert_with(|| super::dial_policy::WakeBucket::new(now))
            .admit(now, ceiling);
        if !admitted {
            debug!(
                "[DIAL] wake rate-limited peer={} trigger={}",
                short_id(&self.peer_id),
                trigger.as_str()
            );
            return false;
        }
        let due = SystemTime::now() + RECONNECT_BASE_INTERVAL;
        if self.next_attempt_at > due {
            self.next_attempt_at = due;
        }
        let was_dormant = self.dormant;
        self.dormant = false;
        if was_dormant {
            info!(
                "[DIAL] wake peer={} trigger={}",
                short_id(&self.peer_id),
                trigger.as_str()
            );
        } else {
            debug!(
                "[DIAL] wake peer={} trigger={}",
                short_id(&self.peer_id),
                trigger.as_str()
            );
        }
        true
    }

    /// Whether enough time has passed to attempt reconnection
    pub fn is_ready(&self) -> bool {
        SystemTime::now() >= self.next_attempt_at
    }
}

/// Result of queuing a send — explicitly NOT a delivery confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendResult {
    /// Message queued for delivery via the specified transport.
    /// This does NOT mean the peer received it.
    Queued(TransportType),
}

/// Manages multiple transports and provides intelligent transport selection
pub struct TransportManager {
    /// Transport state per transport type.
    ///
    /// LOCK ORDER (never invert): transports -> peer_transports ->
    /// peer_last_seen -> target_peers -> reconnection_queue. `tick()` takes
    /// these in order and replays synthetic events only with NO locks held
    /// (review F1, TRANSPORT_FAILOVER_AUDIT_QWENPAID_2026-08-05).
    transports: Arc<RwLock<HashMap<TransportType, TransportState>>>,

    /// Maps peer IDs to available transports
    peer_transports: Arc<RwLock<HashMap<[u8; 32], HashSet<TransportType>>>>,

    /// Pending outgoing data
    outgoing: Arc<RwLock<OutgoingQueue>>,

    /// Last time each peer was seen
    peer_last_seen: Arc<RwLock<HashMap<[u8; 32], SystemTime>>>,

    /// Peers we want to stay connected to (survive disconnects)
    target_peers: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,

    /// Peers awaiting reconnection with backoff state
    reconnection_queue: Arc<RwLock<HashMap<[u8; 32], ReconnectionState>>>,

    /// Peers currently past the staleness window, with the count of
    /// consecutive ticks they have stayed stale (review F2 grace counter).
    stale_candidates: Arc<RwLock<HashMap<[u8; 32], u32>>>,

    /// Pending-dial headroom reported by the swarm layer; `usize::MAX` means
    /// unreported (batch size is then derived from the ready set).
    pending_dial_headroom: Arc<AtomicUsize>,

    /// Optional health monitor for stale connection cleanup
    health_monitor: Option<Arc<TransportHealthMonitor>>,

    /// Address observer for consensus-based address discovery and expiry
    address_observer: Arc<RwLock<AddressObserver>>,

    /// Multi-hop recall module for path selection across transports
    multi_hop_recall: Option<MultiHopRecall>,

    /// Escalation engine consulted to prefer fallback transports after a
    /// synthetic disconnect (set via `set_escalation_engine`).
    escalation_engine: Option<Arc<EscalationEngine>>,
}

impl TransportManager {
    /// Create a new transport manager
    pub fn new() -> Self {
        Self::new_with_multihop(None)
    }

    /// Create a new transport manager with multi-hop recall for intelligent path selection
    pub fn new_with_multihop(multi_hop_recall: Option<MultiHopRecall>) -> Self {
        Self {
            transports: Arc::new(RwLock::new(HashMap::new())),
            peer_transports: Arc::new(RwLock::new(HashMap::new())),
            outgoing: Arc::new(RwLock::new(OutgoingQueue::new())),
            peer_last_seen: Arc::new(RwLock::new(HashMap::new())),
            target_peers: Arc::new(RwLock::new(HashMap::new())),
            reconnection_queue: Arc::new(RwLock::new(HashMap::new())),
            pending_dial_headroom: Arc::new(AtomicUsize::new(usize::MAX)),
            stale_candidates: Arc::new(RwLock::new(HashMap::new())),
            health_monitor: None,
            address_observer: Arc::new(RwLock::new(AddressObserver::new())),
            multi_hop_recall,
            escalation_engine: None,
        }
    }

    /// Set the transport health monitor for stale connection cleanup.
    pub fn set_health_monitor(&mut self, monitor: Arc<TransportHealthMonitor>) {
        self.health_monitor = Some(monitor);
    }

    /// Set the escalation engine. When configured, a synthetic disconnect
    /// from `tick()` deescalates the peer so reconnects prefer a fallback
    /// transport over the one that went silent.
    pub fn set_escalation_engine(&mut self, engine: Arc<EscalationEngine>) {
        self.escalation_engine = Some(engine);
    }

    /// Register a transport with capabilities
    pub fn register_transport(
        &self,
        transport_type: TransportType,
        capabilities: TransportCapabilities,
    ) {
        let mut transports = self.transports.write();
        transports.insert(transport_type, TransportState::new(capabilities));
        info!("Transport registered: {}", transport_type);
    }

    /// Register a transport only if it is not already registered.
    ///
    /// Unlike `register_transport`, this never replaces an existing
    /// `TransportState`, so re-registering on every swarm connect does not
    /// wipe the connected_peers set of peers registered earlier (R1-A4:
    /// peer B connecting used to make peer A read as disconnected).
    pub fn ensure_transport_registered(
        &self,
        transport_type: TransportType,
        capabilities: TransportCapabilities,
    ) {
        let mut transports = self.transports.write();
        // R3-C4: log only when this call actually registers something --
        // this runs on every swarm connect, so unconditional info-level
        // logging would spam under reconnect churn.
        if let std::collections::hash_map::Entry::Vacant(e) = transports.entry(transport_type) {
            e.insert(TransportState::new(capabilities));
            debug!("Transport registered: {}", transport_type);
        }
    }

    /// Handle a transport event
    pub fn handle_event(&self, event: TransportEvent) {
        match event {
            TransportEvent::PeerDiscovered {
                peer_id, transport, ..
            } => {
                let mut peer_transports = self.peer_transports.write();
                peer_transports
                    .entry(peer_id)
                    .or_default()
                    .insert(transport);

                let mut last_seen = self.peer_last_seen.write();
                last_seen.insert(peer_id, SystemTime::now());

                // Wake event: a fresh sighting/address for a queued peer.
                // (reconnection_queue is last in the lock order.)
                if let Some(state) = self.reconnection_queue.write().get_mut(&peer_id) {
                    state.wake(super::dial_policy::WakeTrigger::AddressLearned);
                }

                debug!("Peer {:x?} discovered on {}", &peer_id[..8], transport);
            }
            TransportEvent::PeerDisconnected { peer_id, transport } => {
                // Drop the peer from this transport's connected set so silent
                // platform disconnects cannot leave zombie entries behind
                // (ConnectionEstablished inserts here; this is the mirror).
                {
                    let mut transports_state = self.transports.write();
                    if let Some(state) = transports_state.get_mut(&transport) {
                        state.connected_peers.remove(&peer_id);
                    }
                }

                let mut peer_transports = self.peer_transports.write();
                if let Some(transports) = peer_transports.get_mut(&peer_id) {
                    transports.remove(&transport);
                    if transports.is_empty() {
                        peer_transports.remove(&peer_id);

                        // If this was a target peer, queue for reconnection
                        let target_peers = self.target_peers.read();
                        if let Some(addr) = target_peers.get(&peer_id) {
                            let mut reconnect_queue = self.reconnection_queue.write();
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                reconnect_queue.entry(peer_id)
                            {
                                let mut known_transports = HashSet::new();
                                known_transports.insert(transport);
                                e.insert(ReconnectionState::new(
                                    peer_id,
                                    known_transports,
                                    addr.clone(),
                                ));
                                info!(
                                    "Peer {:x?} lost on {} — queued for reconnection",
                                    &peer_id[..8],
                                    transport
                                );
                            }
                        }
                    }
                }
                debug!("Peer {:x?} disconnected from {}", &peer_id[..8], transport);
            }
            TransportEvent::DataReceived { peer_id, .. } => {
                let mut last_seen = self.peer_last_seen.write();
                last_seen.insert(peer_id, SystemTime::now());
            }
            TransportEvent::ConnectionEstablished { peer_id, transport } => {
                {
                    let mut transports = self.transports.write();
                    if let Some(state) = transports.get_mut(&transport) {
                        state.connected_peers.insert(peer_id);
                    }
                }
                // Review F4: restore the optimal transport after reconnect
                // instead of staying pinned to the deescalated fallback.
                if let Some(ref engine) = self.escalation_engine {
                    if engine.should_escalate(peer_id) {
                        if let Ok(better) = engine.escalate(peer_id) {
                            debug!(
                                "Re-escalated peer {:x?} to {} after reconnect",
                                &peer_id[..8],
                                better
                            );
                        }
                    }
                }
                debug!(
                    "Connection established to {:x?} via {}",
                    &peer_id[..8],
                    transport
                );
            }
            TransportEvent::TransportError { .. } => {
                // Log and continue
            }
        }
    }

    /// Queue data for delivery to a peer via the best available transport.
    ///
    /// **Important:** Returns `SendResult::Queued`, which means the message is
    /// in the outgoing queue — NOT that the peer has received it. Actual delivery
    /// confirmation requires an application-level receipt (see `CoreDelegate::on_receipt_received`).
    pub fn send_to_peer(
        &self,
        peer_id: [u8; 32],
        data: Vec<u8>,
        priority: u8,
    ) -> Result<SendResult, TransportError> {
        // Wake event: an outbound message is queued for this peer.
        self.wake_reconnect(&peer_id, super::dial_policy::WakeTrigger::OutboundQueued);
        let best = self.best_transport_for_peer(peer_id)?;

        // Structured tracing: Log transport handoff to hardware layer
        tracing::info!(
            event = "transport_handoff",
            peer_id = %hex::encode(peer_id),
            transport = %best,
            priority = priority,
            payload_size = data.len()
        );

        let mut outgoing = self.outgoing.write();
        outgoing.enqueue(PendingSend {
            peer_id,
            data,
            priority,
            preferred_transport: Some(best),
            created_at: SystemTime::now(),
        });

        Ok(SendResult::Queued(best))
    }

    /// Determine the best transport for a peer
    pub fn best_transport_for_peer(
        &self,
        peer_id: [u8; 32],
    ) -> Result<TransportType, TransportError> {
        let peer_transports = self.peer_transports.read();
        let available = peer_transports
            .get(&peer_id)
            .ok_or(TransportError::PeerNotFound(format!(
                "{:x?}",
                &peer_id[..8]
            )))?;

        if available.is_empty() {
            return Err(TransportError::PeerNotFound(format!(
                "{:x?}",
                &peer_id[..8]
            )));
        }

        let transports = self.transports.read();

        // Score each transport and pick the best
        let best = available.iter().max_by_key(|&&transport_type| {
            let state = transports.get(&transport_type);
            let caps = state.map(|s| &s.capabilities);

            let mut score = 0u64;

            // Prefer connected transports
            if let Some(state) = state {
                if state.connected_peers.contains(&peer_id) {
                    score += 1000;
                }
            }

            // Prefer streaming capability
            if let Some(caps) = caps {
                if caps.supports_streaming {
                    score += 500;
                }

                // Bandwidth
                let bandwidth_score = std::cmp::min(100, caps.estimated_bandwidth_bps / 1_000_000);
                score += bandwidth_score * 5;

                // Prefer lower latency
                let latency_score = 100u64.saturating_sub(caps.estimated_latency_ms as u64);
                score += latency_score;
            }

            score
        });

        best.copied().ok_or(TransportError::PeerNotFound(format!(
            "{:x?}",
            &peer_id[..8]
        )))
    }

    /// Get all discovered peers
    pub fn connected_peers(&self) -> Vec<[u8; 32]> {
        let peer_transports = self.peer_transports.read();
        peer_transports.keys().copied().collect()
    }

    /// Get peers on a specific transport
    pub fn peers_on_transport(&self, transport: TransportType) -> Vec<[u8; 32]> {
        let transports = self.transports.read();
        if let Some(state) = transports.get(&transport) {
            state.connected_peers.iter().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a peer is connected on any transport
    pub fn is_peer_connected(&self, peer_id: [u8; 32]) -> bool {
        let peer_transports = self.peer_transports.read();
        peer_transports
            .get(&peer_id)
            .map(|transports| !transports.is_empty())
            .unwrap_or(false)
    }

    /// Get all transports where a peer is connected
    pub fn transports_for_peer(&self, peer_id: [u8; 32]) -> Vec<TransportType> {
        let peer_transports = self.peer_transports.read();
        peer_transports
            .get(&peer_id)
            .map(|transports| transports.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get the pending sends queue
    pub fn pending_sends(&self) -> Vec<PendingSend> {
        let outgoing = self.outgoing.read();
        outgoing.items.clone()
    }

    // --------------------------------------------------------------------
    // RECONNECTION MANAGEMENT
    // --------------------------------------------------------------------

    /// Register a peer as a "target peer" — one we want to stay connected to.
    /// When a target peer disconnects, it's automatically queued for reconnection
    /// with exponential backoff.
    pub fn add_target_peer(&self, peer_id: [u8; 32], addr: Vec<u8>) {
        self.target_peers.write().insert(peer_id, addr);
        debug!("Added target peer {:x?}", &peer_id[..8]);
    }

    /// Remove a peer from the target set. Stops reconnection attempts.
    pub fn remove_target_peer(&self, peer_id: &[u8; 32]) {
        self.target_peers.write().remove(peer_id);
        self.reconnection_queue.write().remove(peer_id);
        debug!("Removed target peer {:x?}", &peer_id[..8]);
    }

    /// Report how many more outbound dials the swarm can start right now
    /// (pending-dial headroom). Reconnect batches are sized from it.
    pub fn set_pending_dial_headroom(&self, headroom: usize) {
        self.pending_dial_headroom
            .store(headroom, Ordering::Relaxed);
    }

    /// Returns peers that are due for a reconnection attempt.
    /// The caller is responsible for actually dialing these peers and then
    /// calling `record_reconnect_success` or `record_reconnect_failure`.
    ///
    /// Rate-limited to prevent a Resume Storm (all peers reconnecting at
    /// once after app wake). The batch size is derived, not fixed: it is the
    /// pending-dial headroom reported via `set_pending_dial_headroom`, or,
    /// when none was reported, the ceiling of the square root of the ready
    /// set (small sets go mostly at once, large sets spread out). Peers over
    /// the batch are staggered across subsequent ticks. No peer is ever
    /// dropped: retries never stop.
    pub fn peers_needing_reconnect(&self) -> Vec<ReconnectionState> {
        let mut queue = self.reconnection_queue.write();

        let mut ready: Vec<[u8; 32]> = queue
            .iter()
            .filter(|(_, state)| state.is_ready())
            .map(|(id, _)| *id)
            .collect();

        let reported = self.pending_dial_headroom.load(Ordering::Relaxed);
        let batch = if reported == usize::MAX {
            ((ready.len() as f64).sqrt().ceil() as usize).max(1)
        } else {
            // Zero reported headroom still lets one peer through per tick:
            // never-give-up means a saturated swarm must not starve the
            // queue entirely (the swarm's own limits still apply).
            reported.max(1)
        };

        // If more peers are ready than the batch, stagger the excess
        if ready.len() > batch {
            // Sort by disconnected_at so longest-waiting peers go first
            ready.sort_by(|a, b| {
                let a_disc = queue
                    .get(a)
                    .map(|s| s.disconnected_at)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let b_disc = queue
                    .get(b)
                    .map(|s| s.disconnected_at)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                a_disc.cmp(&b_disc)
            });

            // Stagger the ones we're NOT returning this tick
            for (i, peer_id) in ready[batch..].iter().enumerate() {
                if let Some(state) = queue.get_mut(peer_id) {
                    state.next_attempt_at =
                        SystemTime::now() + RECONNECT_STAGGER_INTERVAL * (i as u32 + 1);
                }
            }

            ready.truncate(batch);
        }

        ready
            .iter()
            .filter_map(|id| queue.get(id).cloned())
            .collect()
    }

    /// Record a successful reconnection - removes from reconnect queue.
    pub fn record_reconnect_success(&self, peer_id: &[u8; 32]) {
        self.reconnection_queue.write().remove(peer_id);
        info!("Reconnected to peer {:x?}", &peer_id[..8]);
    }

    /// Record a failed reconnection attempt - advances the jittered backoff
    /// timer. There is no failure limit; the peer stays queued.
    pub fn record_reconnect_failure(&self, peer_id: &[u8; 32]) {
        let mut queue = self.reconnection_queue.write();
        if let Some(state) = queue.get_mut(peer_id) {
            state.record_outcome(false);
            let ceiling = state.ceiling();
            state.record_failure_with_ceiling(ceiling);
            debug!(
                "Reconnection to {:x?} failed (attempt {}), next try in {:?}",
                &peer_id[..8],
                state.failures,
                state.backoff_interval_with(ceiling)
            );
        }
    }

    /// Wake event for one queued peer (identify, inbound connection or dial,
    /// new address, outbound message queued). Returns true if it changed.
    pub fn wake_reconnect(
        &self,
        peer_id: &[u8; 32],
        trigger: super::dial_policy::WakeTrigger,
    ) -> bool {
        self.reconnection_queue
            .write()
            .get_mut(peer_id)
            .is_some_and(|state| state.wake(trigger))
    }

    /// Wake every queued peer. Returns how many changed.
    pub fn wake_all_reconnects(&self, trigger: super::dial_policy::WakeTrigger) -> usize {
        self.reconnection_queue
            .write()
            .values_mut()
            .map(|state| state.wake(trigger))
            .filter(|changed| *changed)
            .count()
    }

    /// Consume a discovery-scheduler event: genuine LOCAL change events wake
    /// queued peers; peer-loss events do not, and neither does
    /// `NewPeerConnected` (a remote party, including a Sybil, can trigger it
    /// cheaply).
    pub fn on_network_event(&self, event: &super::discovery_scheduler::NetworkEvent) -> usize {
        use super::discovery_scheduler::NetworkEvent;
        match event {
            NetworkEvent::PeerDisconnected { .. }
            | NetworkEvent::AllPeersLost
            | NetworkEvent::NewPeerConnected => 0,
            NetworkEvent::BleStateChanged { on } if !*on => 0,
            NetworkEvent::LedgerReceived { new_entries } if *new_entries == 0 => 0,
            _ => self.wake_all_reconnects(super::dial_policy::WakeTrigger::NetworkChange),
        }
    }

    /// How many peers are currently in the reconnection queue
    pub fn reconnection_queue_len(&self) -> usize {
        self.reconnection_queue.read().len()
    }

    /// Maintenance: clean up stale peer entries (reconnection entries are never
    /// pruned for failing; only `remove_target_peer` or success removes them).
    ///
    /// Stale peers (not seen for 5 minutes) are fully reconciled out of every
    /// state map -- including each transport's `connected_peers`, which the
    /// platform event path alone can leave behind when a transport dies
    /// silently (no disconnect callback ever fires). For every such peer a
    /// synthetic `PeerDisconnected` event is replayed through `handle_event`
    /// so target peers land in the reconnection queue exactly as they would
    /// on a real disconnect, and the escalation engine deescalates to a
    /// fallback transport.
    pub fn tick(&self) {
        let now = SystemTime::now();
        let stale_threshold = Duration::from_secs(300);

        // Phase A: identify peers past the staleness window, and require
        // STALE_CONFIRM_TICKS consecutive stale ticks before acting (review
        // F2: an idle-but-healthy link that sees traffic between ticks
        // resets naturally via the last_seen refresh on DataReceived).
        // NOTE: the health monitor tracks libp2p PeerIds while this manager is
        // keyed by 32-byte identity ids; bridging the two is follow-up work
        // (HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md).
        // Time-based staleness is the Phase-1 trigger.
        let time_stale: HashSet<[u8; 32]> = {
            let last_seen = self.peer_last_seen.read();
            last_seen
                .iter()
                .filter(|(_, seen_at)| {
                    now.duration_since(**seen_at)
                        .map(|elapsed| elapsed > stale_threshold)
                        .unwrap_or(false)
                })
                .map(|(peer_id, _)| *peer_id)
                .collect()
        };

        let confirmed: Vec<[u8; 32]> = {
            let mut stale_candidates = self.stale_candidates.write();
            // Peers that saw traffic again drop out of the candidate set.
            stale_candidates.retain(|peer_id, _| time_stale.contains(peer_id));
            for peer_id in &time_stale {
                *stale_candidates.entry(*peer_id).or_insert(0) += 1;
            }
            let confirmed: Vec<[u8; 32]> = stale_candidates
                .iter()
                .filter(|(_, count)| **count >= STALE_CONFIRM_TICKS)
                .map(|(peer_id, _)| *peer_id)
                .collect();
            for peer_id in &confirmed {
                stale_candidates.remove(peer_id);
            }
            confirmed
        };

        let stale_peers: Vec<([u8; 32], Vec<TransportType>)> = {
            let peer_transports = self.peer_transports.read();
            confirmed
                .iter()
                .map(|peer_id| {
                    let transports = peer_transports
                        .get(peer_id)
                        .map(|ts| ts.iter().copied().collect::<Vec<_>>())
                        .unwrap_or_default();
                    (*peer_id, transports)
                })
                .collect()
        };

        // Phase B: reconcile last_seen and each transport's connected set
        // under write locks. `peer_transports` is deliberately left intact:
        // the replayed PeerDisconnected events remove it through the normal
        // handler, which is also what populates the reconnection queue for
        // target peers.
        if !stale_peers.is_empty() {
            let mut last_seen = self.peer_last_seen.write();
            let mut transports = self.transports.write();
            for (peer_id, transport_list) in &stale_peers {
                last_seen.remove(peer_id);
                for t in transport_list {
                    if let Some(state) = transports.get_mut(t) {
                        state.connected_peers.remove(peer_id);
                    }
                }
            }
        }

        // Phase C: replay synthetic disconnects with NO locks held.
        // handle_event takes its own locks and re-queues target peers for
        // reconnection, identical to a platform-reported disconnect.
        for (peer_id, transport_list) in &stale_peers {
            if let Some(ref engine) = self.escalation_engine {
                if engine.deescalate(*peer_id).is_err() {
                    debug!(
                        "Deescalation skipped for stale peer {:x?}: untracked or no fallback",
                        &peer_id[..8]
                    );
                }
            }
            for t in transport_list {
                info!(
                    "Stale peer {:x?} pruned from {} -- emitting synthetic disconnect",
                    &peer_id[..8],
                    t
                );
                self.handle_event(TransportEvent::PeerDisconnected {
                    peer_id: *peer_id,
                    transport: *t,
                });
            }
        }

        // Review F3: stagger reconnects inserted by this prune batch so an
        // N-peer silence window does not turn into an N-peer dial burst on
        // the first ready tick.
        if !stale_peers.is_empty() {
            let mut queue = self.reconnection_queue.write();
            let mut idx = 0u32;
            for (peer_id, _) in &stale_peers {
                if let Some(state) = queue.get_mut(peer_id) {
                    state.next_attempt_at =
                        SystemTime::now() + RECONNECT_STAGGER_INTERVAL * (idx + 1);
                    idx += 1;
                }
            }
        }

        // Clean up stale connection stats from the health monitor
        if let Some(ref monitor) = self.health_monitor {
            monitor.cleanup_stale_connections(3600);
        }
    }

    /// Expire address observations older than `max_age_secs`.
    /// Should be called periodically from the maintenance loop to prune
    /// stale external address observations.
    pub fn expire_address_observations(&self, max_age_secs: u64) {
        self.address_observer
            .write()
            .expire_old_observations(max_age_secs);
    }

    /// Return the set of currently healthy peer IDs from the health monitor.
    /// Returns an empty list if no health monitor is configured.
    pub fn get_healthy_connections(&self) -> Vec<libp2p::PeerId> {
        self.health_monitor
            .as_ref()
            .map(|m| m.get_healthy_connections())
            .unwrap_or_default()
    }

    /// Return the set of currently unhealthy peer IDs from the health monitor.
    /// Returns an empty list if no health monitor is configured.
    pub fn get_unhealthy_connections(&self) -> Vec<libp2p::PeerId> {
        self.health_monitor
            .as_ref()
            .map(|m| m.get_unhealthy_connections())
            .unwrap_or_default()
    }

    /// Get all connection statistics from the health monitor.
    /// Returns an empty map if no health monitor is configured.
    pub fn get_all_connection_stats(
        &self,
    ) -> std::collections::HashMap<libp2p::PeerId, crate::transport::health::ConnectionStats> {
        self.health_monitor
            .as_ref()
            .map(|m| m.get_all_connection_stats())
            .unwrap_or_default()
    }

    /// Get global transport metrics from the health monitor.
    /// Returns empty metrics if no health monitor is configured.
    pub fn get_global_metrics(&self) -> crate::transport::health::GlobalTransportMetrics {
        self.health_monitor
            .as_ref()
            .map(|m| m.get_global_metrics())
            .unwrap_or_default()
    }

    /// Get all tracked connections from the address observer's connection tracker.
    /// Delegates to AddressObserver::all_observations() to surface observed
    /// connection endpoints for diagnostics.
    pub fn get_all_observed_connections(
        &self,
    ) -> Vec<crate::transport::observation::ConnectionEndpoint> {
        // The AddressObserver currently only holds address observations.
        // The ConnectionTracker is a separate type; expose through a
        // dedicated tracker if wired. For now, return tracked peers
        // from the address observer's observations.
        self.address_observer
            .read()
            .all_observations()
            .into_iter()
            .map(|obs| {
                crate::transport::observation::ConnectionEndpoint {
                    peer_id: obs.observer,
                    remote_addr: libp2p::Multiaddr::empty(), // observation records SocketAddr, not Multiaddr
                    local_addr: libp2p::Multiaddr::empty(),
                    connection_id: format!("obs-{}", obs.observer),
                    established_at: obs.timestamp,
                }
            })
            .collect()
    }

    /// Clean up stale connections in the health monitor.
    /// Removes entries for peers that have not been active for `max_age_secs`.
    /// No-op if no health monitor is configured.
    pub fn cleanup_health_stale_connections(&self, max_age_secs: u64) {
        if let Some(ref monitor) = self.health_monitor {
            monitor.cleanup_stale_connections(max_age_secs);
        }
    }

    /// Disable a transport type at runtime.
    /// Marks the transport as not running and removes all peers associated
    /// with it from the connected set. The transport will not be selected
    /// for new connections until it is re-registered via `register_transport`.
    pub fn disable_transport(&mut self, transport_type: &str) {
        let tt = match transport_type.to_lowercase().as_str() {
            "ble" => TransportType::BLE,
            "wifi_aware" | "wifiaware" => TransportType::WiFiAware,
            "wifi_direct" | "wifidirect" => TransportType::WiFiDirect,
            "internet" | "tcp" | "quic" => TransportType::Internet,
            "local" => TransportType::Local,
            _ => {
                warn!("Unknown transport type to disable: {}", transport_type);
                return;
            }
        };

        let mut transports = self.transports.write();
        if let Some(state) = transports.get_mut(&tt) {
            state.running = false;
            state.connected_peers.clear();
            info!("Transport {} disabled", tt);
        } else {
            warn!("Transport {} not registered, cannot disable", tt);
        }
    }

    // -----------------------------------------------------------------------
    // Multi-Hop Routing Integration
    // -----------------------------------------------------------------------

    /// Set the multi-hop recall module for path selection across transports.
    /// The multi-hop recall module retrieves relevant transport paths from
    /// multiple sources for intelligent routing decisions.
    pub fn set_multi_hop_recall(&mut self, recall: MultiHopRecall) {
        self.multi_hop_recall = Some(recall);
        info!("Multi-hop recall module configured for path selection");
    }

    /// Get a reference to the multi-hop recall module
    pub fn multi_hop_recall(&self) -> Option<&MultiHopRecall> {
        self.multi_hop_recall.as_ref()
    }

    /// Run multi-hop recall to retrieve transport paths for a peer.
    /// Returns a list of potential paths through available transports.
    // PERIMETER-ALLOW-UNDERSCORE: the DSPy multi-hop recall module is not yet
    // wired into production routing (recall() is a stub returning empty -- see
    // dspy/modules.rs); peer_id is reserved for peer-scoped recall when wired up.
    pub fn run_multi_hop_path_selection(
        &self,
        _peer_id: &[u8; 32],
        query: &str,
    ) -> Vec<TransportType> {
        let mut selected_transports = Vec::new();

        // Check if multi-hop recall is configured
        if let Some(recall) = self.multi_hop_recall.as_ref() {
            // Validate input
            if recall.validate_input(&query.to_string()) {
                // Run recall to get paths
                match recall.recall(query) {
                    Ok(paths) => {
                        // Process recalled paths and map to transport types
                        for path in paths {
                            if path.contains("ble") {
                                selected_transports.push(TransportType::BLE);
                            } else if path.contains("wifi") {
                                selected_transports.push(TransportType::WiFiDirect);
                            } else if path.contains("internet") {
                                selected_transports.push(TransportType::Internet);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Multi-hop recall failed for peer: {:?}", e);
                    }
                }
            }
        } else {
            // Fallback: use available transports in priority order
            let transports = self.transports.read();
            for transport_type in [
                TransportType::BLE,
                TransportType::WiFiAware,
                TransportType::WiFiDirect,
                TransportType::Internet,
                TransportType::Local,
            ] {
                if transports.get(&transport_type).is_some_and(|s| s.running) {
                    selected_transports.push(transport_type);
                }
            }
        }

        selected_transports
    }

    /// Add a reasoning step to the multi-hop chain-of-thought pipeline.
    /// This extends the multi-hop recall with additional routing heuristics.
    pub fn add_multi_hop_step(&mut self, step: &str) {
        if let Some(recall) = self.multi_hop_recall.as_mut() {
            recall.add_step(step);
        }
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_peer_id(val: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = val;
        id
    }

    #[test]
    fn test_transport_state_creation() {
        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        let state = TransportState::new(caps);
        assert!(!state.running);
        assert!(state.connected_peers.is_empty());
    }

    #[test]
    fn test_outgoing_queue_fifo_with_priority() {
        let mut queue = OutgoingQueue::new();

        let send1 = PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 5,
            preferred_transport: None,
            created_at: SystemTime::now(),
        };

        let send2 = PendingSend {
            peer_id: create_peer_id(2),
            data: vec![2],
            priority: 10,
            preferred_transport: None,
            created_at: SystemTime::now(),
        };

        queue.enqueue(send1);
        queue.enqueue(send2);

        // Higher priority should dequeue first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.priority, 10);

        let second = queue.dequeue().unwrap();
        assert_eq!(second.priority, 5);

        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_outgoing_queue_len() {
        let mut queue = OutgoingQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.enqueue(PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 1,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });

        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_outgoing_queue_clear() {
        let mut queue = OutgoingQueue::new();
        queue.enqueue(PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 1,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });

        assert_eq!(queue.len(), 1);
        queue.clear();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_transport_manager_creation() {
        let manager = TransportManager::new();
        assert_eq!(manager.connected_peers().len(), 0);
    }

    #[test]
    fn test_register_transport() {
        let manager = TransportManager::new();
        let caps = TransportCapabilities::for_transport(TransportType::BLE);

        manager.register_transport(TransportType::BLE, caps);

        let peers = manager.peers_on_transport(TransportType::BLE);
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_peer_discovered_event() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };

        manager.handle_event(event);

        let peers = manager.connected_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], peer_id);
    }

    #[test]
    fn test_peer_disconnected_event() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);

        assert_eq!(manager.connected_peers().len(), 1);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.connected_peers().len(), 0);
    }

    /// Force a peer's last-seen timestamp past the 300s staleness window.
    fn force_stale(manager: &TransportManager, peer_id: [u8; 32]) {
        let mut last_seen = manager.peer_last_seen.write();
        last_seen.insert(peer_id, SystemTime::now() - Duration::from_secs(301));
    }

    /// Tick until the STALE_CONFIRM_TICKS grace counter confirms the prune.
    fn confirm_stale(manager: &TransportManager) {
        for _ in 0..STALE_CONFIRM_TICKS {
            manager.tick();
        }
    }

    #[test]
    fn test_tick_stale_prune_clears_connected_peers() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id,
            transport: TransportType::BLE,
        });

        assert!(manager.is_peer_connected(peer_id));
        assert_eq!(manager.peers_on_transport(TransportType::BLE).len(), 1);

        force_stale(&manager, peer_id);
        confirm_stale(&manager);

        // Zombie cleanup: no trace of the peer in any state map.
        assert!(!manager.is_peer_connected(peer_id));
        assert_eq!(manager.peers_on_transport(TransportType::BLE).len(), 0);
        assert!(manager.peer_transports.read().get(&peer_id).is_none());
        assert!(manager.peer_last_seen.read().get(&peer_id).is_none());
    }

    #[test]
    fn test_tick_stale_prune_queues_target_peer_reconnect() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(2);

        // Target peers are the ones we want back; a synthetic disconnect
        // must land them in the reconnection queue like a real one would.
        manager.add_target_peer(peer_id, vec![1]);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id,
            transport: TransportType::BLE,
        });

        assert_eq!(manager.reconnection_queue_len(), 0);

        force_stale(&manager, peer_id);
        confirm_stale(&manager);

        assert_eq!(manager.reconnection_queue_len(), 1);
    }

    #[test]
    fn test_tick_stale_prune_deescalates_transport() {
        use crate::transport::escalation::{EscalationEngine, EscalationPolicy};

        let mut manager = TransportManager::new();
        let engine = Arc::new(EscalationEngine::new(EscalationPolicy::Balanced));
        manager.set_escalation_engine(engine.clone());

        let peer_id = create_peer_id(3);
        engine
            .init_peer(peer_id, vec![TransportType::WiFiDirect, TransportType::BLE])
            .expect("init_peer");
        let before = engine
            .current_transport(peer_id)
            .expect("transport initialized");

        let caps = TransportCapabilities::for_transport(before);
        manager.register_transport(before, caps);
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: before,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id,
            transport: before,
        });

        force_stale(&manager, peer_id);
        confirm_stale(&manager);

        assert!(!manager.is_peer_connected(peer_id));
        let after = engine
            .current_transport(peer_id)
            .expect("peer still tracked by engine");
        if before == TransportType::WiFiDirect {
            assert_eq!(
                after,
                TransportType::BLE,
                "deescalation must move a stale peer to the fallback transport"
            );
        } else {
            // Already on the lowest tier: deescalate is a no-op.
            assert_eq!(after, before);
        }
    }

    #[test]
    fn test_tick_stale_prune_concurrent_with_real_disconnect() {
        // Review F6: a real PeerDisconnected racing the confirming tick must
        // yield exactly one reconnection-queue entry and no panic, whichever
        // side wins the race.
        let manager = Arc::new(TransportManager::new());
        let peer_id = create_peer_id(4);

        manager.add_target_peer(peer_id, vec![1]);
        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id,
            transport: TransportType::BLE,
        });

        force_stale(&manager, peer_id);
        // Two warm-up ticks: grace counter reaches 2, no prune yet.
        manager.tick();
        manager.tick();
        assert_eq!(manager.reconnection_queue_len(), 0);

        let racer = manager.clone();
        let handle = std::thread::spawn(move || {
            racer.handle_event(TransportEvent::PeerDisconnected {
                peer_id,
                transport: TransportType::BLE,
            });
        });
        manager.tick();
        handle.join().expect("racer thread");

        // Vacant-entry guard + single prune => exactly one queue entry.
        assert_eq!(manager.reconnection_queue_len(), 1);
        assert!(!manager.is_peer_connected(peer_id));
    }

    #[test]
    fn test_multiple_transports_per_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr: vec![4, 5, 6],
        };
        manager.handle_event(event2);

        let transports = manager.transports_for_peer(peer_id);
        assert_eq!(transports.len(), 2);
    }

    #[test]
    fn test_best_transport_prefers_connected() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps_ble = TransportCapabilities::for_transport(TransportType::BLE);
        let caps_wifi = TransportCapabilities::for_transport(TransportType::WiFiDirect);

        manager.register_transport(TransportType::BLE, caps_ble);
        manager.register_transport(TransportType::WiFiDirect, caps_wifi);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);

        let discovered2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr: vec![4, 5, 6],
        };
        manager.handle_event(discovered2);

        let established = TransportEvent::ConnectionEstablished {
            peer_id,
            transport: TransportType::WiFiDirect,
        };
        manager.handle_event(established);

        let best = manager
            .best_transport_for_peer(peer_id)
            .expect("should have transport");
        assert_eq!(best, TransportType::WiFiDirect);
    }

    #[test]
    fn test_best_transport_prefers_streaming() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(2);

        let caps_ble = TransportCapabilities::for_transport(TransportType::BLE);
        let caps_aware = TransportCapabilities::for_transport(TransportType::WiFiAware);

        manager.register_transport(TransportType::BLE, caps_ble);
        manager.register_transport(TransportType::WiFiAware, caps_aware);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiAware,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let best = manager
            .best_transport_for_peer(peer_id)
            .expect("should have transport");
        assert_eq!(best, TransportType::WiFiAware);
    }

    #[test]
    fn test_best_transport_prefers_low_latency() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(3);

        let caps_internet = TransportCapabilities::for_transport(TransportType::Internet);
        let caps_local = TransportCapabilities::for_transport(TransportType::Local);

        manager.register_transport(TransportType::Internet, caps_internet);
        manager.register_transport(TransportType::Local, caps_local);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Internet,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Local,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let best = manager
            .best_transport_for_peer(peer_id)
            .expect("should have transport");
        assert_eq!(best, TransportType::Local);
    }

    #[test]
    fn test_best_transport_fails_for_unknown_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(99);

        let result = manager.best_transport_for_peer(peer_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_peer_connected() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        assert!(!manager.is_peer_connected(peer_id));

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        assert!(manager.is_peer_connected(peer_id));
    }

    #[test]
    fn test_peers_on_transport() {
        let manager = TransportManager::new();
        let peer1 = create_peer_id(1);
        let peer2 = create_peer_id(2);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id: peer1,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id: peer2,
            transport: TransportType::BLE,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let established1 = TransportEvent::ConnectionEstablished {
            peer_id: peer1,
            transport: TransportType::BLE,
        };
        manager.handle_event(established1);

        let established2 = TransportEvent::ConnectionEstablished {
            peer_id: peer2,
            transport: TransportType::BLE,
        };
        manager.handle_event(established2);

        let peers = manager.peers_on_transport(TransportType::BLE);
        assert_eq!(peers.len(), 2);
    }

    #[test]
    fn test_send_to_peer_queues_data() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let result = manager.send_to_peer(peer_id, vec![1, 2, 3], 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SendResult::Queued(TransportType::BLE));

        let pending = manager.pending_sends();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].priority, 5);
    }

    #[test]
    fn test_pending_sends_priority_ordering() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        manager.send_to_peer(peer_id, vec![1], 3).unwrap();
        manager.send_to_peer(peer_id, vec![2], 10).unwrap();
        manager.send_to_peer(peer_id, vec![3], 5).unwrap();

        let pending = manager.pending_sends();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].priority, 10);
        assert_eq!(pending[1].priority, 5);
        assert_eq!(pending[2].priority, 3);
    }

    #[test]
    fn test_tick_cleanup() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        assert_eq!(manager.connected_peers().len(), 1);

        // Manually set last seen to far in the past
        {
            let mut last_seen = manager.peer_last_seen.write();
            last_seen.insert(
                peer_id,
                SystemTime::now() - web_time::Duration::from_secs(301),
            );
        }

        confirm_stale(&manager);

        assert_eq!(manager.connected_peers().len(), 0);
    }

    #[test]
    fn test_transports_for_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Internet,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let transports = manager.transports_for_peer(peer_id);
        assert_eq!(transports.len(), 2);
        assert!(transports.contains(&TransportType::BLE));
        assert!(transports.contains(&TransportType::Internet));
    }

    #[test]
    fn test_connected_peers_deduplication() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let peers = manager.connected_peers();
        assert_eq!(peers.len(), 1);
    }

    // ====================================================================
    // RECONNECTION TESTS
    // ====================================================================

    #[test]
    fn test_target_peer_queued_for_reconnect_on_disconnect() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        // Register as target peer
        manager.add_target_peer(peer_id, vec![1, 2, 3]);

        // Discover and then disconnect
        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);
        assert_eq!(manager.connected_peers().len(), 1);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        // Should be in reconnection queue
        assert_eq!(manager.reconnection_queue_len(), 1);
    }

    #[test]
    fn test_non_target_peer_not_queued_for_reconnect() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        // Discover WITHOUT registering as target
        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        // Should NOT be in reconnection queue
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_reconnection_backoff_increases() {
        let mut state = ReconnectionState::new(create_peer_id(1), HashSet::new(), vec![]);

        let first = state.backoff_interval();
        state.record_failure();
        let second = state.backoff_interval();
        state.record_failure();
        let third = state.backoff_interval();

        // Each interval should be larger (exponential backoff)
        assert!(second > first);
        assert!(third > second);
    }

    #[test]
    fn test_reconnection_backoff_capped_at_max() {
        let mut state = ReconnectionState::new(create_peer_id(1), HashSet::new(), vec![]);

        // Hit it many times to saturate
        for _ in 0..20 {
            state.record_failure();
        }

        assert!(state.backoff_interval() <= RECONNECT_CEILING_BASE);
    }

    fn queue_target(manager: &TransportManager, peer_id: [u8; 32]) {
        manager.add_target_peer(peer_id, vec![1]);
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        });
    }

    #[test]
    fn test_reconnection_never_gives_up() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);
        queue_target(&manager, peer_id);

        // Far beyond the old 10-failure limit: still queued, still retried.
        for _ in 0..100 {
            manager.record_reconnect_failure(&peer_id);
        }
        assert_eq!(manager.reconnection_queue_len(), 1);
        manager.tick();
        assert_eq!(manager.reconnection_queue_len(), 1);

        // Jittered backoff never exceeds the stability-derived ceiling
        // (at most 4x the base) and the peer is due once it elapses.
        let due = {
            let queue = manager.reconnection_queue.read();
            let state = queue.get(&peer_id).expect("queued");
            assert!(state.failures >= 100);
            assert!(state.dormant);
            state.next_attempt_at
        };
        let wait = due
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO);
        assert!(wait <= RECONNECT_CEILING_BASE * 4);
        {
            let mut queue = manager.reconnection_queue.write();
            let state = queue.get_mut(&peer_id).expect("queued");
            state.next_attempt_at = SystemTime::now() - Duration::from_secs(1);
        }
        assert_eq!(manager.peers_needing_reconnect().len(), 1);
    }

    #[test]
    fn test_reconnect_wake_triggers() {
        use super::super::dial_policy::WakeTrigger;
        let triggers = [
            WakeTrigger::InboundConnection,
            WakeTrigger::InboundDial,
            WakeTrigger::Identify,
            WakeTrigger::AddressLearned,
            WakeTrigger::NetworkChange,
            WakeTrigger::OutboundQueued,
        ];
        for trigger in triggers {
            let manager = TransportManager::new();
            let peer_id = create_peer_id(1);
            queue_target(&manager, peer_id);
            for _ in 0..12 {
                manager.record_reconnect_failure(&peer_id);
            }
            assert!(manager.peers_needing_reconnect().is_empty());
            assert!(
                manager.wake_reconnect(&peer_id, trigger),
                "{}",
                trigger.as_str()
            );
            let queue = manager.reconnection_queue.read();
            let state = queue.get(&peer_id).expect("queued");
            assert!(!state.dormant);
            // One-shot credit: the ladder is kept.
            assert_eq!(state.failures, 12);
            let wait = state
                .next_attempt_at
                .duration_since(SystemTime::now())
                .unwrap_or(Duration::ZERO);
            assert!(wait <= RECONNECT_BASE_INTERVAL, "wake must bound the wait");
        }
    }

    #[test]
    fn test_send_to_peer_and_discovery_wake_queued_peer() {
        let manager = TransportManager::new();
        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);
        let peer_id = create_peer_id(1);
        queue_target(&manager, peer_id);

        // New sighting of the queued peer wakes it.
        for _ in 0..12 {
            manager.record_reconnect_failure(&peer_id);
        }
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![2],
        });
        assert!(!manager.reconnection_queue.read()[&peer_id].dormant);
        assert_eq!(manager.reconnection_queue.read()[&peer_id].failures, 12);

        // An outbound message wakes it even when no transport can carry it.
        for _ in 0..12 {
            manager.record_reconnect_failure(&peer_id);
        }
        manager.handle_event(TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        });
        let _ = manager.send_to_peer(peer_id, vec![1], 5);
        assert!(!manager.reconnection_queue.read()[&peer_id].dormant);
    }

    #[test]
    fn test_network_event_wakes_queue_but_peer_loss_does_not() {
        use super::super::discovery_scheduler::NetworkEvent;
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);
        queue_target(&manager, peer_id);
        for _ in 0..12 {
            manager.record_reconnect_failure(&peer_id);
        }
        assert_eq!(
            manager.on_network_event(&NetworkEvent::PeerDisconnected { count_now: 1 }),
            0
        );
        assert_eq!(manager.on_network_event(&NetworkEvent::AllPeersLost), 0);
        assert_eq!(manager.on_network_event(&NetworkEvent::NewPeerConnected), 0);
        assert_eq!(manager.on_network_event(&NetworkEvent::WifiChanged), 1);
        assert!(!manager.reconnection_queue.read()[&peer_id].dormant);
        // A second wake from the same source is rate limited (one-shot credit).
        assert_eq!(manager.on_network_event(&NetworkEvent::WifiChanged), 0);
    }

    #[test]
    fn test_reconnect_batch_follows_headroom() {
        let manager = TransportManager::new();
        for n in 1..=8u8 {
            queue_target(&manager, create_peer_id(n));
        }
        {
            let mut queue = manager.reconnection_queue.write();
            for state in queue.values_mut() {
                state.next_attempt_at = SystemTime::now() - Duration::from_secs(1);
            }
        }
        manager.set_pending_dial_headroom(5);
        assert_eq!(manager.peers_needing_reconnect().len(), 5);

        // Zero headroom still releases one peer (max(1)) and drops none.
        {
            let mut queue = manager.reconnection_queue.write();
            for state in queue.values_mut() {
                state.next_attempt_at = SystemTime::now() - Duration::from_secs(1);
            }
        }
        manager.set_pending_dial_headroom(0);
        assert_eq!(manager.peers_needing_reconnect().len(), 1);
        assert_eq!(manager.reconnection_queue_len(), 8);
    }

    #[test]
    fn test_dead_peer_does_not_stretch_other_peers_ceiling() {
        let manager = TransportManager::new();
        let dead = create_peer_id(1);
        let healthy = create_peer_id(2);
        queue_target(&manager, dead);
        queue_target(&manager, healthy);
        for _ in 0..40 {
            manager.record_reconnect_failure(&dead);
        }
        manager.record_reconnect_failure(&healthy);
        let queue = manager.reconnection_queue.read();
        assert!(queue[&dead].ceiling() > queue[&healthy].ceiling());
        assert!(queue[&healthy].ceiling() < RECONNECT_CEILING_BASE * 2);
    }

    #[test]
    fn test_reconnect_success_removes_from_queue() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        manager.add_target_peer(peer_id, vec![1]);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.reconnection_queue_len(), 1);

        manager.record_reconnect_success(&peer_id);
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_remove_target_peer_stops_reconnection() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        manager.add_target_peer(peer_id, vec![1]);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.reconnection_queue_len(), 1);

        manager.remove_target_peer(&peer_id);
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_send_result_is_queued_not_delivered() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let result = manager.send_to_peer(peer_id, vec![1, 2, 3], 5).unwrap();

        // Explicitly verify it's Queued, not some "Delivered" status
        match result {
            SendResult::Queued(transport) => assert_eq!(transport, TransportType::BLE),
        }
    }

    #[test]
    fn test_latency_score_underflow_prevention() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(42);

        // High latency transport (500ms > 100ms)
        let high_latency_caps = TransportCapabilities {
            max_payload_size: 65536,
            supports_streaming: false,
            is_bidirectional: true,
            estimated_bandwidth_bps: 1_000_000,
            estimated_latency_ms: 500,
        };
        manager.register_transport(TransportType::BLE, high_latency_caps);

        // Lower latency transport (200ms)
        let lower_latency_caps = TransportCapabilities {
            max_payload_size: 65536,
            supports_streaming: false,
            is_bidirectional: true,
            estimated_bandwidth_bps: 1_000_000,
            estimated_latency_ms: 200,
        };
        manager.register_transport(TransportType::WiFiDirect, lower_latency_caps);

        // Extreme latency transport (u32::MAX)
        let extreme_latency_caps = TransportCapabilities {
            max_payload_size: 65536,
            supports_streaming: false,
            is_bidirectional: true,
            estimated_bandwidth_bps: 1_000_000,
            estimated_latency_ms: u32::MAX,
        };
        manager.register_transport(TransportType::Internet, extreme_latency_caps);

        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        });
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr: vec![2],
        });
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Internet,
            addr: vec![3],
        });

        // Best transport selection must not panic from underflow
        let selected = manager.best_transport_for_peer(peer_id);
        assert!(selected.is_ok());
    }
}
