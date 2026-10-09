//! Connection admission: evict, never deny.
//!
//! [`AdmissionBehaviour`] replaces libp2p's static `connection_limits`
//! behaviour. It is kept FIRST in the `IronCoreBehaviour` derive struct, so
//! the derive macro consults it before any child behaviour. Its
//! `handle_established_*_connection` hooks ALWAYS accept: an honest peer is
//! never refused because of a count. Instead, when a peer ends up holding
//! more paths than its derived budget (see [`super::path_budget`]), the
//! behaviour queues `ToSwarm::CloseConnection` for the worst surplus path.
//!
//! The only remaining refusal is the operating system's own: when the OS runs
//! out of file descriptors or handles, accept/dial fail below libp2p. That is
//! detected by [`is_os_exhaustion`] and logged as `[CONN] hard-ceiling`
//! (the budget source for this arrives with the resource-aware follow-up).
//!
//! The swarm loop feeds the behaviour liveness and traffic stamps (the
//! behaviour cannot see other behaviours' events) and asks it whether a
//! close was an eviction ([`AdmissionBehaviour::take_eviction`]) so an
//! eviction does not trigger the failover ledger re-exchange.

use super::path_budget::{
    format_budget_marker, format_evict_marker, short_peer, Eviction, PathClass, PathLedger,
};
use libp2p::core::transport::PortUse;
use libp2p::core::Endpoint;
use libp2p::swarm::behaviour::{ConnectionClosed, ConnectionEstablished, FromSwarm};
use libp2p::swarm::{
    dummy, CloseConnection, ConnectionDenied, ConnectionId, NetworkBehaviour, THandler,
    THandlerInEvent, THandlerOutEvent, ToSwarm,
};
use libp2p::{Multiaddr, PeerId};
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::task::{Context, Poll, Waker};
use web_time::{Duration, Instant};

/// Cadence of the periodic `[CONN] budget=` marker.
pub const MARKER_INTERVAL: Duration = Duration::from_secs(30);

/// OS error codes that mean the process or machine is out of descriptors or
/// socket buffers: EMFILE (24) and ENFILE (23) on unix, ENOBUFS (105 on
/// Linux/Android), WSAEMFILE (10024) and WSAENOBUFS (10055) on Windows.
const OS_EXHAUSTION_CODES: [i32; 5] = [23, 24, 105, 10024, 10055];

/// True when `err` (or anything in its source chain) is an `io::Error` for
/// genuine OS resource exhaustion.
pub fn is_os_exhaustion(err: &(dyn std::error::Error + 'static)) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = current {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            if io
                .raw_os_error()
                .is_some_and(|code| OS_EXHAUSTION_CODES.contains(&code))
            {
                return true;
            }
        }
        current = e.source();
    }
    false
}

/// Log `[CONN] hard-ceiling` and return true when `err` is OS exhaustion.
pub fn log_if_hard_ceiling(context: &str, err: &(dyn std::error::Error + 'static)) -> bool {
    if is_os_exhaustion(err) {
        tracing::warn!(
            "[CONN] hard-ceiling source=os-descriptors where={} err={}",
            context,
            err
        );
        true
    } else {
        false
    }
}

/// Admission control that evicts instead of denying.
pub struct AdmissionBehaviour {
    ledger: PathLedger<PeerId, ConnectionId>,
    /// Handshake start per connection id, to derive the grace window.
    pending: HashMap<ConnectionId, Instant>,
    /// Closes to hand to the swarm on the next poll.
    to_close: VecDeque<(PeerId, ConnectionId)>,
    /// Connections that closed because we evicted them, until the swarm loop
    /// has looked at the matching `ConnectionClosed` event.
    evicted_closed: HashMap<ConnectionId, Instant>,
    last_eval: Instant,
    last_marker: Instant,
    waker: Option<Waker>,
}

impl Default for AdmissionBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

impl AdmissionBehaviour {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            ledger: PathLedger::new(),
            pending: HashMap::new(),
            to_close: VecDeque::new(),
            evicted_closed: HashMap::new(),
            last_eval: now,
            last_marker: now,
            waker: None,
        }
    }

    /// A connection attempt started (inbound or outbound).
    pub fn begin_handshake(&mut self, id: ConnectionId, now: Instant) {
        self.pending.entry(id).or_insert(now);
    }

    /// A connection finished its handshake. Recomputes the peer's budget and
    /// queues closes for surplus paths. Never refuses the new connection.
    pub fn path_established(
        &mut self,
        peer: PeerId,
        id: ConnectionId,
        remote_addr: &Multiaddr,
        now: Instant,
    ) {
        let handshake = self
            .pending
            .remove(&id)
            .map_or(Duration::ZERO, |start| now.saturating_duration_since(start));
        let class = PathClass::of_addr(remote_addr);
        let evictions = self.ledger.on_established(peer, id, class, handshake, now);
        self.queue(evictions);
    }

    /// The swarm reported `ConnectionClosed`.
    pub fn path_closed(
        &mut self,
        peer: &PeerId,
        id: &ConnectionId,
        remaining_established: usize,
        now: Instant,
    ) {
        self.pending.remove(id);
        let outcome = self.ledger.on_closed(peer, id, remaining_established);
        if outcome.was_eviction {
            self.evicted_closed.insert(*id, now);
        }
    }

    /// Periodic liveness proof on a path (ping success, identify).
    pub fn stamp_liveness(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_liveness(peer, id, now);
    }

    /// Protocol traffic on a path (message protocol, ledger exchange, ...).
    pub fn stamp_traffic(&mut self, peer: &PeerId, id: &ConnectionId, now: Instant) {
        self.ledger.stamp_traffic(peer, id, now);
    }

    /// A ping failed on a path.
    pub fn note_ping_failed(&mut self, peer: &PeerId, id: &ConnectionId) {
        self.ledger.note_ping_failed(peer, id);
    }

    /// True (once) if `id` closed because this behaviour evicted it. The
    /// caller skips the failover ledger re-exchange for such closes.
    pub fn take_eviction(&mut self, id: &ConnectionId) -> bool {
        self.evicted_closed.remove(id).is_some()
    }

    /// Next close request waiting for the swarm, if any.
    pub fn next_close_request(&mut self) -> Option<(PeerId, ConnectionId)> {
        self.to_close.pop_front()
    }

    /// Periodic work: re-evaluate grace expiry, re-issue stalled closes, and
    /// emit the budget marker. Rate-limited, so it is cheap to call from poll.
    pub fn tick(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_eval) >= super::path_budget::EVAL_QUANTUM {
            self.last_eval = now;
            let evictions = self.ledger.evaluate_all(now);
            self.queue(evictions);
            for (peer, id) in self.ledger.due_reissue(now) {
                self.to_close.push_back((peer, id));
            }
            // Bound the failover-skip memory: the swarm loop consumes entries
            // when it sees the ConnectionClosed event, so anything this old
            // was missed (for example a final-close arm) and can go.
            self.evicted_closed
                .retain(|_, at| now.saturating_duration_since(*at) < MARKER_INTERVAL);
        }
        if now.saturating_duration_since(self.last_marker) >= MARKER_INTERVAL {
            self.last_marker = now;
            let stats = self.ledger.stats(now);
            if stats.used > 0 {
                tracing::info!("{}", format_budget_marker(&stats));
            }
        }
    }

    fn queue(&mut self, evictions: Vec<Eviction<PeerId, ConnectionId>>) {
        if evictions.is_empty() {
            return;
        }
        for ev in evictions {
            tracing::info!(
                "{}",
                format_evict_marker(&ev.id, &short_peer(&ev.peer), ev.reason, ev.class)
            );
            self.to_close.push_back((ev.peer, ev.id));
        }
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl NetworkBehaviour for AdmissionBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;
    type ToSwarm = Infallible;

    fn handle_pending_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        _: &Multiaddr,
        _: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        self.begin_handshake(connection_id, Instant::now());
        Ok(())
    }

    fn handle_established_inbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        _: Option<PeerId>,
        _: &[Multiaddr],
        _: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        self.begin_handshake(connection_id, Instant::now());
        Ok(Vec::new())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: Endpoint,
        _: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        let now = Instant::now();
        match event {
            FromSwarm::ConnectionEstablished(ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                ..
            }) => {
                self.path_established(peer_id, connection_id, endpoint.get_remote_address(), now);
            }
            FromSwarm::ConnectionClosed(ConnectionClosed {
                peer_id,
                connection_id,
                remaining_established,
                ..
            }) => {
                self.path_closed(&peer_id, &connection_id, remaining_established, now);
            }
            FromSwarm::DialFailure(failure) => {
                self.pending.remove(&failure.connection_id);
            }
            FromSwarm::ListenFailure(failure) => {
                self.pending.remove(&failure.connection_id);
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        match event {}
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        self.tick(Instant::now());
        if let Some((peer_id, id)) = self.to_close.pop_front() {
            return Poll::Ready(ToSwarm::CloseConnection {
                peer_id,
                connection: CloseConnection::One(id),
            });
        }
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::core::ConnectedPoint;

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    fn lan(port: u16) -> Multiaddr {
        format!("/ip4/192.168.0.121/tcp/{port}")
            .parse()
            .expect("valid multiaddr")
    }

    fn wan() -> Multiaddr {
        "/ip4/8.8.8.8/tcp/9001".parse().expect("valid multiaddr")
    }

    fn cid(n: usize) -> ConnectionId {
        ConnectionId::new_unchecked(n)
    }

    #[test]
    fn established_hooks_always_accept() {
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        // A flood far beyond any historical static cap: every hook accepts.
        for n in 0..500usize {
            let a = lan(10_000 + (n % 1000) as u16);
            assert!(b.handle_pending_inbound_connection(cid(n), &a, &a).is_ok());
            assert!(b
                .handle_established_inbound_connection(cid(n), peer, &a, &a)
                .is_ok());
            assert!(b
                .handle_pending_outbound_connection(
                    cid(n + 1000),
                    Some(peer),
                    &[],
                    Endpoint::Dialer
                )
                .is_ok());
            assert!(b
                .handle_established_outbound_connection(
                    cid(n + 1000),
                    peer,
                    &a,
                    Endpoint::Dialer,
                    PortUse::Reuse
                )
                .is_ok());
        }
    }

    #[test]
    fn handover_ghosts_are_closed_through_swarm_events_and_the_new_inbound_survives() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        // Two Wi-Fi sockets, pinged every 15 s.
        b.begin_handshake(cid(1), base);
        b.path_established(peer, cid(1), &lan(9001), at(base, 20));
        b.begin_handshake(cid(2), at(base, 100));
        b.path_established(peer, cid(2), &lan(9002), at(base, 120));
        for t in [15_000u64, 30_000] {
            b.stamp_liveness(&peer, &cid(1), at(base, t));
            b.stamp_liveness(&peer, &cid(2), at(base, t + 100));
        }
        assert!(b.next_close_request().is_none());
        // Handover: cellular connection arrives, the Wi-Fi ones went silent.
        b.begin_handshake(cid(3), at(base, 99_900));
        b.path_established(peer, cid(3), &wan(), at(base, 100_000));
        let mut closes = Vec::new();
        while let Some(req) = b.next_close_request() {
            closes.push(req);
        }
        assert_eq!(closes.len(), 2);
        assert!(closes.contains(&(peer, cid(1))) && closes.contains(&(peer, cid(2))));
        assert!(!closes.iter().any(|(_, id)| *id == cid(3)));

        // The swarm confirms the closes; they are recognised as evictions
        // exactly once, the new inbound is untouched.
        b.path_closed(&peer, &cid(1), 2, at(base, 100_010));
        b.path_closed(&peer, &cid(2), 1, at(base, 100_020));
        assert!(b.take_eviction(&cid(1)));
        assert!(!b.take_eviction(&cid(1)), "reported once");
        assert!(b.take_eviction(&cid(2)));
        assert!(!b.take_eviction(&cid(3)));
        b.tick(at(base, 130_000));
        assert!(b.next_close_request().is_none());
    }

    #[test]
    fn natural_close_is_not_reported_as_an_eviction() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        b.path_established(peer, cid(1), &wan(), base);
        b.path_closed(&peer, &cid(1), 0, at(base, 10));
        assert!(!b.take_eviction(&cid(1)));
    }

    #[test]
    fn grace_expiry_is_picked_up_by_tick_not_only_by_new_connections() {
        let base = Instant::now();
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        b.path_established(peer, cid(1), &wan(), base);
        b.path_established(peer, cid(2), &wan(), at(base, 1));
        assert!(b.next_close_request().is_none());
        b.tick(at(base, 60_000));
        assert_eq!(b.next_close_request(), Some((peer, cid(1))));
        assert!(b.next_close_request().is_none());
    }

    #[test]
    fn swarm_event_path_registers_and_clears_connections() {
        let mut b = AdmissionBehaviour::new();
        let peer = PeerId::random();
        let addr = wan();
        let endpoint = ConnectedPoint::Dialer {
            address: addr,
            role_override: Endpoint::Dialer,
            port_use: PortUse::Reuse,
        };
        b.on_swarm_event(FromSwarm::ConnectionEstablished(ConnectionEstablished {
            peer_id: peer,
            connection_id: cid(1),
            endpoint: &endpoint,
            failed_addresses: &[],
            other_established: 0,
        }));
        assert_eq!(b.ledger.live_paths(&peer), 1);
        b.on_swarm_event(FromSwarm::ConnectionClosed(ConnectionClosed {
            peer_id: peer,
            connection_id: cid(1),
            endpoint: &endpoint,
            cause: None,
            remaining_established: 0,
        }));
        assert_eq!(b.ledger.tracked_paths(&peer), 0);
    }

    #[test]
    fn os_exhaustion_is_detected_through_the_source_chain() {
        let emfile = std::io::Error::from_raw_os_error(24);
        assert!(is_os_exhaustion(&emfile));
        let refused = std::io::Error::from_raw_os_error(111);
        assert!(!is_os_exhaustion(&refused));
        let plain = std::io::Error::new(std::io::ErrorKind::Other, "custom");
        assert!(!is_os_exhaustion(&plain));
        assert!(log_if_hard_ceiling("test", &emfile));
        assert!(!log_if_hard_ceiling("test", &refused));
    }
}
