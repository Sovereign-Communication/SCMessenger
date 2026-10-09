// Boot seed-dial sweep (V040-T1 HALF 2, rescheduled by #469 T6).
//
// A node whose public address changed can never rejoin the mesh: nobody can
// dial it at its old address, and it never dials out. This module fires
// `SwarmHandle::connect_to_seed_peers` on boot and keeps sweeping, forever,
// on the schedule of the shared event-driven `DiscoveryScheduler`
// (core/src/transport/discovery_scheduler.rs). There is no fixed ladder and
// no give-up: a network event (interface change, peer lost, ...) resets the
// schedule to aggressive and wakes the loop immediately, then the interval
// decays with jitter toward a ceiling derived from peer density and power.
// The candidate list comes from the core ledger (proven + unproven seed
// tiers).

use crate::platform_signals;
use scmessenger_core::transport::{
    DiscoveryInputs, DiscoveryScheduler, NetworkEvent, PowerState, SwarmHandle, TransportClass,
};
use scmessenger_core::IronCore;
use std::collections::BTreeSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Why a [`SeedDialClient::wait`] returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitOutcome {
    /// The scheduled delay elapsed with no event.
    Elapsed,
    /// An affecting network event arrived; sweep now.
    Woken,
}

/// Client of the shared discovery scheduler for the internet/ledger dial.
/// Owns the scheduler plus the wake-up used to cancel a pending timer when a
/// network event resets the schedule.
pub struct SeedDialClient {
    scheduler: DiscoveryScheduler,
    wake: Notify,
    /// Signalled when an event is coalesced, so a wait in flight re-reads the
    /// scheduler's pending-reset due time.
    rearm: Notify,
    /// Swarm handle (set once by `run`) so network events also wake dormant
    /// peers in the dial policy, not only the seed-dial schedule.
    swarm: std::sync::OnceLock<SwarmHandle>,
}

impl SeedDialClient {
    pub fn new() -> Arc<Self> {
        Self::with_scheduler(DiscoveryScheduler::with_system_time(
            TransportClass::Internet,
        ))
    }

    /// Client over a caller-supplied scheduler (injected clock in tests).
    pub fn with_scheduler(scheduler: DiscoveryScheduler) -> Arc<Self> {
        Arc::new(Self {
            scheduler,
            wake: Notify::new(),
            rearm: Notify::new(),
            swarm: std::sync::OnceLock::new(),
        })
    }

    /// Feed a network event. If it affects the ledger dial the schedule is
    /// reset to aggressive and any pending wait is cancelled.
    pub fn emit(&self, event: NetworkEvent) -> bool {
        let affected = self.scheduler.on_event(event);
        if let Some(swarm) = self.swarm.get() {
            swarm.notify_network_event(event);
        }
        if affected {
            // notify_one stores a permit when nobody is waiting yet, so an
            // event that lands mid-sweep still wakes the next wait.
            self.wake.notify_one();
        } else if self.scheduler.pending_reset_due_in().is_some() {
            // Coalesced, not lost: the wait must also end when the deferred
            // reset falls due (a stored permit covers a not-yet-started wait).
            self.rearm.notify_one();
        }
        affected
    }

    /// Poll interval for platforms with no OS network-change notification.
    /// Computed from the live scheduler state (its current interval, which
    /// is the floor right after a change and decays toward the density/power
    /// ceiling), never a literal: detection is no slower than the next sweep.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.scheduler.snapshot().interval_ms.max(1))
    }

    /// Sleep for `delay`, or return early when an affecting event arrives or
    /// when a coalesced reset falls due: the wait is effectively
    /// `min(delay, pending reset due time)`, so a real change absorbed by flap
    /// damping is applied within its window instead of after the full sleep.
    pub async fn wait(&self, delay: Duration) -> WaitOutcome {
        let sleep = tokio::time::sleep(delay);
        tokio::pin!(sleep);
        loop {
            let rearmed = self.rearm.notified();
            tokio::pin!(rearmed);
            // Registered before the due time is read so a coalesce landing in
            // between is not missed.
            rearmed.as_mut().enable();
            let pending = self.scheduler.pending_reset_due_in();
            let due = async {
                match pending {
                    Some(d) => tokio::time::sleep(d).await,
                    None => std::future::pending::<()>().await,
                }
            };
            tokio::select! {
                _ = &mut sleep => return WaitOutcome::Elapsed,
                _ = self.wake.notified() => return WaitOutcome::Woken,
                _ = due => {
                    if self.scheduler.apply_pending_reset() {
                        return WaitOutcome::Woken;
                    }
                }
                _ = &mut rearmed => {}
            }
        }
    }
}

/// Event implied by a change in the connected peer count between sweeps.
pub fn peer_transition(previous: usize, now: usize) -> Option<NetworkEvent> {
    if now > previous {
        Some(NetworkEvent::NewPeerConnected)
    } else if now == 0 && previous > 0 {
        Some(NetworkEvent::AllPeersLost)
    } else if now < previous {
        Some(NetworkEvent::PeerDisconnected {
            count_now: u32::try_from(now).unwrap_or(u32::MAX),
        })
    } else {
        None
    }
}

/// Candidate count the seed dial can draw on: the proven tier plus the
/// unproven seed tier of the core ledger. Both are bounded by the store's
/// own caps, so the count is cheap and safe to read on every sweep.
pub fn candidate_count(lm: &scmessenger_core::store::LedgerManager) -> usize {
    lm.get_preferred_relays(u32::MAX).len() + lm.seed_addresses(u32::MAX).len()
}

/// What a single sweep should do, given the live state. Extracted as a pure
/// decision so the boot-dial behaviour is unit-testable without a running
/// swarm (V040-T1 acceptance: the startup path issues a seed dial when the
/// seed list is non-empty and the peer count is zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepAction {
    /// Peers are already connected -- no dial, just keep watching.
    WatchOnly,
    /// Zero candidates to dial (nothing in the core ledger yet).
    Wait,
    /// Non-empty seed list and zero peers -- issue one seed dial.
    Dial,
}

pub fn sweep_decision(peer_count: usize, candidates: usize) -> SweepAction {
    if peer_count > 0 {
        SweepAction::WatchOnly
    } else if candidates == 0 {
        SweepAction::Wait
    } else {
        SweepAction::Dial
    }
}

/// Run one boot seed-dial sweep and return the connected peer count observed
/// at the start of the sweep (the scheduler input for density).
///
/// - connected: no dial, just watch (the schedule re-arms aggressively the
///   moment the count drops)
/// - zero candidates: log the empty sweep
/// - candidates present, zero peers: issue `connect_to_seed_peers` (one dial
///   per sweep -- the swarm command itself waits for a real outcome and
///   dials only one candidate per call, so callers retry)
pub async fn sweep_once(swarm: &SwarmHandle, core: &IronCore, sweep: u32) -> usize {
    let peer_count = swarm.get_peers().await.unwrap_or_default().len();
    match sweep_decision(peer_count, candidate_count(&core.ledger_manager)) {
        SweepAction::WatchOnly => {
            tracing::debug!("[SEED-DIAL] peers={} -- connected; watching", peer_count);
        }
        SweepAction::Wait => {
            tracing::info!(
                "[SEED-DIAL] sweep {}: 0 candidate(s), peers=0 -- nothing to dial yet",
                sweep
            );
        }
        SweepAction::Dial => {
            let count = candidate_count(&core.ledger_manager);
            match swarm.connect_to_seed_peers().await {
                Ok(()) => tracing::info!(
                    "[SEED-DIAL] sweep {}: {} candidate(s), peers=0 -- connected",
                    sweep,
                    count
                ),
                Err(e) => tracing::info!(
                    "[SEED-DIAL] sweep {}: {} candidate(s), peers=0 -- dial outcome: {}",
                    sweep,
                    count,
                    e
                ),
            }
        }
    }
    peer_count
}

/// Long-lived seed-dial loop. Never returns: sweeps, then waits the
/// scheduler's delay or an affecting network event, whichever comes first.
pub async fn run(swarm: SwarmHandle, core: Arc<IronCore>, client: Arc<SeedDialClient>) {
    let _ = client.swarm.set(swarm.clone());
    let mut sweep: u32 = 0;
    let mut previous_peers: usize = 0;
    // No battery (or an unread sample) means mains powered.
    let mut power = PowerState::Charging;
    loop {
        sweep = sweep.saturating_add(1);
        let peers = sweep_once(&swarm, &core, sweep).await;
        // Peer-count changes are events too; they reset the schedule (subject
        // to flap coalescing) but need no wake-up because the next delay is
        // computed right below.
        if let Some(event) = peer_transition(previous_peers, peers) {
            client.scheduler.on_event(event);
        }
        // A peer still connected across two consecutive sweeps is a proven
        // connection: restore normal reset responsiveness.
        if previous_peers > 0 && peers > 0 {
            client.scheduler.record_success();
        }
        previous_peers = peers;
        // Power is read from the OS where cheap (Linux sysfs, Windows power
        // status); with no battery the computed default is mains powered. The
        // CLI is always foreground. Density comes from the live peer count.
        // The sysfs reads are blocking file I/O: keep them off the async
        // worker. A failed join keeps the previous sample.
        power = tokio::task::spawn_blocking(platform_signals::detect_power_state)
            .await
            .unwrap_or(power);
        client.scheduler.set_inputs(DiscoveryInputs {
            connected_peers: u32::try_from(peers).unwrap_or(u32::MAX),
            power,
            foreground: true,
        });
        let delay = client.scheduler.next_delay();
        if client.wait(delay).await == WaitOutcome::Woken {
            tracing::info!("[SEED-DIAL] network event -- sweeping now");
        }
    }
}

/// Non-loopback local interface addresses. An enumeration failure yields an
/// empty set (treated as "no addresses"), never a panic.
pub fn interface_snapshot() -> BTreeSet<IpAddr> {
    if_addrs::get_if_addrs()
        .map(|ifaces| {
            ifaces
                .into_iter()
                .filter(|iface| !iface.is_loopback())
                .map(|iface| iface.ip())
                .collect()
        })
        .unwrap_or_default()
}

/// Interface-set monitor. Detection is driven by OS change notifications
/// (Windows `NotifyUnicastIpAddressChange`, Linux netlink); each wake-up is
/// diffed against the last interface set so only a real change emits
/// `LanInterfaceChanged`. On platforms with no notification source, or if the
/// source dies, it falls back to polling at an interval computed from the
/// dial scheduler's live state (`SeedDialClient::poll_interval`), not a
/// literal. Notification storms are absorbed by the scheduler's reset
/// coalescing.
pub async fn run_interface_monitor(client: Arc<SeedDialClient>) {
    let watch = platform_signals::start_change_watch();
    let mut previous = interface_snapshot();
    loop {
        match &watch {
            Some(w) if w.alive() => w.changed().await,
            _ => tokio::time::sleep(client.poll_interval()).await,
        }
        let current = interface_snapshot();
        if current != previous {
            tracing::info!(
                "[DISCOVERY] event=LanInterfaceChanged detected interfaces={}",
                current.len()
            );
            previous = current;
            client.emit(NetworkEvent::LanInterfaceChanged);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T6 acceptance: a simulated network-change event wakes the pending wait
    /// within one tick instead of sitting out the scheduled delay.
    #[tokio::test]
    async fn network_event_wakes_sweep_within_one_tick() {
        let client = SeedDialClient::new();
        assert!(client.emit(NetworkEvent::LanInterfaceChanged));
        let outcome = tokio::time::timeout(
            Duration::from_secs(5),
            client.wait(Duration::from_secs(3600)),
        )
        .await
        .expect("wait must return promptly after an affecting event");
        assert_eq!(outcome, WaitOutcome::Woken);
    }

    /// An event that does not affect the ledger dial must not wake it.
    #[tokio::test]
    async fn unaffecting_event_does_not_wake() {
        let client = SeedDialClient::new();
        assert!(!client.emit(NetworkEvent::BleStateChanged { on: true }));
        let outcome = client.wait(Duration::from_millis(50)).await;
        assert_eq!(outcome, WaitOutcome::Elapsed);
    }

    /// An event delivered while a sweep is running is not lost: the next
    /// wait returns immediately.
    #[tokio::test]
    async fn event_during_sweep_is_not_lost() {
        let client = SeedDialClient::new();
        client.emit(NetworkEvent::CellularChanged);
        let outcome = client.wait(Duration::from_secs(3600)).await;
        assert_eq!(outcome, WaitOutcome::Woken);
    }

    /// The schedule resets to the aggressive floor on an event and then
    /// decays; the ceiling is a function of density, not one literal.
    #[tokio::test]
    async fn schedule_resets_then_decays_without_fixed_ladder() {
        let client = SeedDialClient::new();
        for _ in 0..40 {
            client.scheduler.next_delay();
        }
        let plateau = client.scheduler.snapshot().interval_ms;
        client.emit(NetworkEvent::WifiChanged);
        let snap = client.scheduler.snapshot();
        assert!(snap.interval_ms < plateau);
        assert_eq!(snap.attempts, 0);
        client.scheduler.set_inputs(DiscoveryInputs {
            connected_peers: 0,
            power: PowerState::Charging,
            foreground: true,
        });
        let sparse = client.scheduler.ceiling_ms();
        client.scheduler.set_inputs(DiscoveryInputs {
            connected_peers: 16,
            power: PowerState::Charging,
            foreground: true,
        });
        assert!(client.scheduler.ceiling_ms() > sparse);
    }

    struct ManualClock(std::sync::atomic::AtomicU64);
    impl scmessenger_core::transport::SchedulerClock for ManualClock {
        fn now_ms(&self) -> u64 {
            self.0.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    /// Rule-8 review F1: a flap followed by a real change 300 ms later is
    /// coalesced, but a long sleep already in flight must still end by the
    /// window end (500 ms floor), not after the full delay.
    #[tokio::test]
    async fn coalesced_real_change_ends_long_sleep_by_window_end() {
        let clock = Arc::new(ManualClock(std::sync::atomic::AtomicU64::new(0)));
        let scheduler = DiscoveryScheduler::new(
            TransportClass::Internet,
            scmessenger_core::transport::SchedulerConfig::for_transport(TransportClass::Internet),
            clock.clone(),
            Arc::new(scmessenger_core::transport::ThreadRngJitter),
        );
        let client = SeedDialClient::with_scheduler(scheduler);
        assert!(client.emit(NetworkEvent::WifiChanged));
        // The reset's own wake permit belongs to the sweep that just ran.
        assert_eq!(
            client.wait(Duration::from_secs(3600)).await,
            WaitOutcome::Woken
        );
        clock.0.store(300, std::sync::atomic::Ordering::SeqCst);
        let waiter = {
            let c = Arc::clone(&client);
            tokio::spawn(async move { c.wait(Duration::from_secs(3600)).await })
        };
        tokio::task::yield_now().await;
        // Coalesced while the long sleep is in flight.
        assert!(!client.emit(NetworkEvent::CellularChanged));
        let outcome = tokio::time::timeout(Duration::from_secs(10), waiter)
            .await
            .expect("coalesced reset must end the wait by the window end")
            .expect("waiter task");
        assert_eq!(outcome, WaitOutcome::Woken);
        assert_eq!(client.scheduler.snapshot().resets, 2);
        assert_eq!(client.scheduler.pending_reset_due_in(), None);
    }

    /// The fallback poll interval follows scheduler state, not a literal.
    #[test]
    fn poll_interval_follows_scheduler_state() {
        let client = SeedDialClient::new();
        let at_floor = client.poll_interval();
        for _ in 0..40 {
            client.scheduler.next_delay();
        }
        assert!(client.poll_interval() > at_floor);
        assert!(client.poll_interval() <= Duration::from_millis(client.scheduler.ceiling_ms()));
    }

    /// Back-to-back interface events: the first resets, the second is
    /// coalesced and does not wake the dial loop again.
    #[test]
    fn back_to_back_interface_events_are_coalesced_at_the_client() {
        let client = SeedDialClient::new();
        assert!(client.emit(NetworkEvent::LanInterfaceChanged));
        assert!(!client.emit(NetworkEvent::LanInterfaceChanged));
        assert_eq!(client.scheduler.snapshot().coalesced, 1);
    }

    #[test]
    fn peer_transition_maps_count_changes_to_events() {
        assert_eq!(peer_transition(0, 0), None);
        assert_eq!(peer_transition(2, 2), None);
        assert_eq!(peer_transition(0, 1), Some(NetworkEvent::NewPeerConnected));
        assert_eq!(peer_transition(1, 3), Some(NetworkEvent::NewPeerConnected));
        assert_eq!(peer_transition(1, 0), Some(NetworkEvent::AllPeersLost));
        assert_eq!(
            peer_transition(3, 1),
            Some(NetworkEvent::PeerDisconnected { count_now: 1 })
        );
    }

    #[test]
    fn interface_snapshot_excludes_loopback() {
        let snap = interface_snapshot();
        assert!(snap.iter().all(|ip| !ip.is_loopback()));
    }

    /// V040-T1 HALF 2 acceptance: the startup path issues a seed dial when
    /// the seed list is non-empty and the peer count is zero.
    #[test]
    fn sweep_decision_dials_when_seeds_nonempty_and_peers_zero() {
        assert_eq!(sweep_decision(0, 1), SweepAction::Dial);
        assert_eq!(sweep_decision(0, 16), SweepAction::Dial);
    }

    /// No candidates -> nothing to dial: the sweep waits and re-checks.
    #[test]
    fn sweep_decision_waits_when_no_candidates() {
        assert_eq!(sweep_decision(0, 0), SweepAction::Wait);
    }

    /// Connected peers -> no dial; the sweep only re-checks (re-arms when
    /// the count returns to zero).
    #[test]
    fn sweep_decision_watches_when_connected() {
        assert_eq!(sweep_decision(1, 0), SweepAction::WatchOnly);
        assert_eq!(sweep_decision(3, 16), SweepAction::WatchOnly);
    }

    /// The candidate count reflects the core ledger: zero for an empty
    /// ledger, non-zero once seeds are imported (the Half 1 bridge in action).
    #[test]
    fn candidate_count_reflects_seeded_core_ledger() {
        let lm = scmessenger_core::store::LedgerManager::ephemeral();
        assert_eq!(candidate_count(&lm), 0);
        let imported = lm.import_seed_entries(vec![scmessenger_core::store::SeedLedgerEntry {
            multiaddr: "/ip4/98.94.45.116/tcp/9001".to_string(),
        }]);
        assert_eq!(imported, 1);
        assert!(candidate_count(&lm) >= 1);
    }
}
