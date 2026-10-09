//! Bounded in-memory trail of per-message lifecycle events, plus the stable
//! log-marker formats the 3-node triangulation verifier parses.
//!
//! Privacy: only message ids, truncated peer ids and a fixed event vocabulary
//! are ever stored or formatted here. Never message content, plaintext or keys.
//! Memory: a fixed-capacity ring (`MAX_EVENTS`); ids are clamped to
//! `MAX_ID_CHARS`, so the worst case is a few hundred KiB regardless of traffic.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::OnceLock;

/// Ring capacity (events, not messages).
pub const MAX_EVENTS: usize = 1024;
/// Message ids longer than this are truncated before storage.
pub const MAX_ID_CHARS: usize = 128;
/// Peer ids in log markers are truncated to this many chars (matches the
/// existing `peer_id[..16]` convention in iron_core.rs logging).
pub const PEER_LOG_CHARS: usize = 16;

/// Lifecycle stage of a message as observed by THIS node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageEventKind {
    /// Outbound envelope prepared (sender side).
    Sent,
    /// Relay custody accepted (relay side).
    Custody,
    /// Inbound envelope decrypted (receiver side).
    Decrypt,
    /// Inbound message history write attempted (receiver side).
    History,
    /// Delivery receipt processed (sender side).
    Receipt,
}

impl MessageEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MessageEventKind::Sent => "sent",
            MessageEventKind::Custody => "custody",
            MessageEventKind::Decrypt => "decrypt",
            MessageEventKind::History => "history",
            MessageEventKind::Receipt => "receipt",
        }
    }
}

/// One recorded event. Contains no content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEvent {
    pub msg_id: String,
    pub kind: MessageEventKind,
    /// false only for a failed attempt (e.g. history write error).
    pub ok: bool,
    pub ts_ms: u64,
}

/// Fixed-capacity ring of events.
#[derive(Debug)]
pub struct MessageEventRing {
    cap: usize,
    events: VecDeque<MessageEvent>,
}

impl MessageEventRing {
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(1);
        Self {
            cap,
            events: VecDeque::with_capacity(cap.min(MAX_EVENTS)),
        }
    }

    pub fn push(&mut self, event: MessageEvent) {
        while self.events.len() >= self.cap {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Events for the `max_messages` most recently active message ids, each id
    /// keeping its last `per_message` events in chronological order. Result is
    /// ordered most-recently-active id first.
    pub fn recent(
        &self,
        max_messages: usize,
        per_message: usize,
    ) -> Vec<(String, Vec<MessageEvent>)> {
        let mut order: Vec<String> = Vec::new();
        for ev in self.events.iter().rev() {
            if order.len() >= max_messages {
                break;
            }
            if !order.contains(&ev.msg_id) {
                order.push(ev.msg_id.clone());
            }
        }
        order
            .into_iter()
            .map(|id| {
                let mut evs: Vec<MessageEvent> = self
                    .events
                    .iter()
                    .rev()
                    .filter(|e| e.msg_id == id)
                    .take(per_message)
                    .cloned()
                    .collect();
                evs.reverse();
                (id, evs)
            })
            .collect()
    }
}

fn global() -> &'static RwLock<MessageEventRing> {
    static RING: OnceLock<RwLock<MessageEventRing>> = OnceLock::new();
    RING.get_or_init(|| RwLock::new(MessageEventRing::with_capacity(MAX_EVENTS)))
}

fn now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

const MAX_DETAIL_CHARS: usize = 128;

fn clamp_id(id: &str) -> String {
    sanitize_log_field(id, MAX_ID_CHARS)
}

/// Make an untrusted string safe to embed as a single `key=value` token in a
/// triangulation marker line: keep at most `max_chars` chars and replace
/// control characters, whitespace and `=` (the marker grammar delimiters)
/// with `_`, so a field can neither break the line nor add key=value pairs.
pub(crate) fn sanitize_log_field(s: &str, max_chars: usize) -> String {
    s.chars()
        .take(max_chars)
        .map(|c| {
            if c.is_control() || c.is_whitespace() || c == '=' {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// Truncate a peer id for logging (first `PEER_LOG_CHARS` chars, sanitized).
pub fn short_peer(peer: &str) -> String {
    sanitize_log_field(peer, PEER_LOG_CHARS)
}

/// Number of trailing peer-id chars used by `short_peer_tail`.
pub const PEER_TAIL_CHARS: usize = 8;

/// Last `PEER_TAIL_CHARS` chars of a peer id, sanitized. Peer ids share a
/// fixed multihash prefix, so the tail is the discriminating part; this is the
/// format used by the #520/#521 markers, for cross-marker correlation.
pub fn short_peer_tail(peer: &str) -> String {
    let total = peer.chars().count();
    let tail: String = peer
        .chars()
        .skip(total.saturating_sub(PEER_TAIL_CHARS))
        .collect();
    sanitize_log_field(&tail, PEER_TAIL_CHARS)
}

/// Record an event in the process-wide bounded ring.
pub fn record(msg_id: &str, kind: MessageEventKind, ok: bool) {
    global().write().push(MessageEvent {
        msg_id: clamp_id(msg_id),
        kind,
        ok,
        ts_ms: now_ms(),
    });
}

/// Snapshot for `/api/diagnostics`: see [`MessageEventRing::recent`].
pub fn recent_message_events(
    max_messages: usize,
    per_message: usize,
) -> Vec<(String, Vec<MessageEvent>)> {
    global().read().recent(max_messages, per_message)
}

// ---- stable marker formats (parsed by scripts/trinode/markers.py) ----------

/// G6: `delivery_state msg=<id> state=pending detail=<detail>`
pub fn fmt_delivery_state_pending(msg_id: &str, detail: &str) -> String {
    format!(
        "delivery_state msg={} state=pending detail={}",
        sanitize_log_field(msg_id, MAX_ID_CHARS),
        sanitize_log_field(detail, MAX_DETAIL_CHARS)
    )
}

/// G2: `rx_decrypt msg=<id> from=<peer16> type=<text|receipt|other> result=ok`
pub fn fmt_rx_decrypt(msg_id: &str, peer: &str, kind: &str) -> String {
    format!(
        "rx_decrypt msg={} from={} type={} result=ok",
        sanitize_log_field(msg_id, MAX_ID_CHARS),
        short_peer(peer),
        sanitize_log_field(kind, 32)
    )
}

/// G3: `rx_history msg=<id> from=<peer16> result=<ok|failed> dup=<bool> hidden=<bool>`
pub fn fmt_rx_history(msg_id: &str, peer: &str, ok: bool, dup: bool, hidden: bool) -> String {
    format!(
        "rx_history msg={} from={} result={} dup={} hidden={}",
        sanitize_log_field(msg_id, MAX_ID_CHARS),
        short_peer(peer),
        if ok { "ok" } else { "failed" },
        dup,
        hidden
    )
}

/// G4: `custody_accept msg=<id> from=<peer16> dest=<peer16>`
pub fn fmt_custody_accept(msg_id: &str, from: &str, dest: &str) -> String {
    format!(
        "custody_accept msg={} from={} dest={}",
        sanitize_log_field(msg_id, MAX_ID_CHARS),
        short_peer(from),
        short_peer(dest)
    )
}

/// G1: `ledger_address_learned peer=<peer16> via=<via> addr=<multiaddr>`
pub fn fmt_ledger_address_learned(peer: &str, via: &str, addr: &str) -> String {
    format!(
        "ledger_address_learned peer={} via={} addr={}",
        short_peer(peer),
        sanitize_log_field(via, 64),
        sanitize_log_field(addr, 256)
    )
}

/// `[ROUTING] peer_seen peer=<short> source=<transport>` -- emitted (rate
/// limited per peer by the caller) when the routing engine is told a peer was
/// seen on a transport. `source` is attacker-influenced (it arrives over the
/// FFI from platform glue), so it is sanitized and length-capped.
pub fn fmt_routing_peer_seen(peer: &str, source: &str) -> String {
    format!(
        "[ROUTING] peer_seen peer={} source={}",
        short_peer(peer),
        sanitize_log_field(source, 32)
    )
}

/// Canonical transport kinds allowed in `[TRANSPORT] kind=...`.
pub const TRANSPORT_KINDS: &[&str] = &[
    "tcp4",
    "tcp6",
    "quic",
    "circuit",
    "dcutr",
    "mdns",
    "ble",
    "wifi_direct",
    "wifi_aware",
    "cellular",
];

/// Canonical transport states allowed in `[TRANSPORT] ... state=...`.
pub const TRANSPORT_STATES: &[&str] = &[
    "unavailable",
    "available",
    "listening",
    "connected",
    "error",
];

/// `[TRANSPORT] kind=<kind> state=<state> [peers=<n>] detail=<reason>`.
///
/// `kind` and `state` must be members of [`TRANSPORT_KINDS`] /
/// [`TRANSPORT_STATES`]; anything else is rendered as `kind=invalid` /
/// `state=error` so a caller bug can never inject free text into the
/// grammar. `detail` is sanitized (whitespace and `=` become `_`) and capped.
pub fn fmt_transport_status(kind: &str, state: &str, peers: Option<usize>, detail: &str) -> String {
    let kind = if TRANSPORT_KINDS.contains(&kind) {
        kind
    } else {
        "invalid"
    };
    let state = if TRANSPORT_STATES.contains(&state) {
        state
    } else {
        "error"
    };
    let detail = if detail.is_empty() { "none" } else { detail };
    match peers {
        Some(n) => format!(
            "[TRANSPORT] kind={} state={} peers={} detail={}",
            kind,
            state,
            n,
            sanitize_log_field(detail, MAX_DETAIL_CHARS)
        ),
        None => format!(
            "[TRANSPORT] kind={} state={} detail={}",
            kind,
            state,
            sanitize_log_field(detail, MAX_DETAIL_CHARS)
        ),
    }
}

// ---- [RX-DROP] / [RX-STALL] markers ----------------------------------------

/// Minimum spacing between `[RX-DROP]` lines for one (stage, reason) key.
const RX_DROP_LOG_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);
/// Bound on distinct (stage, reason) keys tracked by the limiter.
const RX_DROP_LIMITER_MAX_KEYS: usize = 64;

type DropLimiter = std::collections::HashMap<String, (web_time::Instant, u64)>;

fn rx_drop_limiter() -> &'static parking_lot::Mutex<DropLimiter> {
    static LIMITER: OnceLock<parking_lot::Mutex<DropLimiter>> = OnceLock::new();
    LIMITER.get_or_init(|| parking_lot::Mutex::new(DropLimiter::new()))
}

/// Limiter decision: `None` = skip this line; `Some(n)` = emit, with `n`
/// lines suppressed since the last emission for the key.
fn rx_drop_gate_in(map: &mut DropLimiter, key: &str, now: web_time::Instant) -> Option<u64> {
    if let Some((last, suppressed)) = map.get_mut(key) {
        if now.saturating_duration_since(*last) < RX_DROP_LOG_INTERVAL {
            *suppressed = suppressed.saturating_add(1);
            return None;
        }
        let n = *suppressed;
        *last = now;
        *suppressed = 0;
        return Some(n);
    }
    if map.len() >= RX_DROP_LIMITER_MAX_KEYS {
        // Key space is attacker-influenced only through sanitized, fixed
        // stage/reason literals; if it is somehow full, stay quiet.
        return None;
    }
    map.insert(key.to_string(), (now, 0));
    Some(0)
}

/// `[RX-DROP] msg=<id|-> stage=<stage> reason=<reason>`
pub fn fmt_rx_drop(msg_id: Option<&str>, stage: &str, reason: &str) -> String {
    format!(
        "[RX-DROP] msg={} stage={} reason={}",
        msg_id
            .map(clamp_id)
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "-".to_string()),
        sanitize_log_field(stage, 32),
        sanitize_log_field(reason, 64)
    )
}

/// `[RX-DROP] stage=<stage> kind=<kind>` (payload kind with no message id).
pub fn fmt_rx_drop_kind(stage: &str, kind: &str) -> String {
    format!(
        "[RX-DROP] stage={} kind={}",
        sanitize_log_field(stage, 32),
        sanitize_log_field(kind, 64)
    )
}

/// `[RX-DROP] suppressed=<n> stage=<stage>` (matches PR #512 grammar).
pub fn fmt_rx_drop_suppressed(count: u64, stage: &str) -> String {
    format!(
        "[RX-DROP] suppressed={} stage={}",
        count,
        sanitize_log_field(stage, 32)
    )
}

/// `[RX-STALL] drain_loop idle_ms=<n> backlog=<n>`
pub fn fmt_rx_stall_drain(idle_ms: u64, backlog: usize) -> String {
    format!(
        "[RX-STALL] drain_loop idle_ms={} backlog={}",
        idle_ms, backlog
    )
}

/// Emit a rate-limited `[RX-DROP]` line (per stage+reason); folded lines are
/// reported as a following `[RX-DROP] suppressed=<n> stage=<s>` line.
pub fn log_rx_drop(msg_id: Option<&str>, stage: &str, reason: &str) {
    let key = format!(
        "{}/{}",
        sanitize_log_field(stage, 32),
        sanitize_log_field(reason, 64)
    );
    let gate = rx_drop_gate_in(
        &mut rx_drop_limiter().lock(),
        &key,
        web_time::Instant::now(),
    );
    if let Some(suppressed) = gate {
        if suppressed > 0 {
            tracing::info!("{}", fmt_rx_drop_suppressed(suppressed, stage));
        }
        tracing::info!("{}", fmt_rx_drop(msg_id, stage, reason));
    }
}

/// `[RX] forwarded kind=<k>` (relay control message not consumed by the swarm
/// handler; the frame is NOT dropped, it continues to normal handling).
pub fn fmt_rx_forwarded(kind: &str) -> String {
    format!("[RX] forwarded kind={}", sanitize_log_field(kind, 64))
}

/// `[RX] drift_frame type=<t> payload_len=<n> from=<peer8>`
pub fn fmt_rx_drift_frame(frame_type: &str, payload_len: usize, peer: &str) -> String {
    format!(
        "[RX] drift_frame type={} payload_len={} from={}",
        sanitize_log_field(frame_type, 32),
        payload_len,
        short_peer_tail(peer)
    )
}

/// Emit a rate-limited `[RX] forwarded kind=<k>` line (per kind). Folded
/// lines are reported as `[RX-DROP] suppressed=<n> stage=swarm_forwarded`.
pub fn log_rx_forwarded(kind: &str) {
    let key = format!("swarm_forwarded/{}", sanitize_log_field(kind, 64));
    let gate = rx_drop_gate_in(
        &mut rx_drop_limiter().lock(),
        &key,
        web_time::Instant::now(),
    );
    if let Some(suppressed) = gate {
        if suppressed > 0 {
            tracing::info!("{}", fmt_rx_drop_suppressed(suppressed, "swarm_forwarded"));
        }
        tracing::info!("{}", fmt_rx_forwarded(kind));
    }
}

/// Emit a rate-limited `[RX] drift_frame` line. The limiter key is the fixed
/// literal `drift_frame` (NOT peer- or type-derived), so a peer can neither
/// flood the log nor grow the limiter's key set.
pub fn log_rx_drift_frame(frame_type: &str, payload_len: usize, peer: &str) {
    let gate = rx_drop_gate_in(
        &mut rx_drop_limiter().lock(),
        "drift_frame",
        web_time::Instant::now(),
    );
    if let Some(suppressed) = gate {
        if suppressed > 0 {
            tracing::info!("{}", fmt_rx_drop_suppressed(suppressed, "drift_frame"));
        }
        tracing::info!("{}", fmt_rx_drift_frame(frame_type, payload_len, peer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(id: &str, kind: MessageEventKind) -> MessageEvent {
        MessageEvent {
            msg_id: id.to_string(),
            kind,
            ok: true,
            ts_ms: 0,
        }
    }

    #[test]
    fn sanitize_blocks_newline_and_kv_injection() {
        let evil = "a\nrx_history msg=x result=ok\r\tz";
        let line = fmt_rx_history(evil, "peer\nfrom=evil", true, false, false);
        assert_eq!(line.lines().count(), 1);
        assert!(!line.contains('\r') && !line.contains('\t'));
        // Exactly the 6 expected tokens: name + 5 key=value pairs.
        assert_eq!(line.split(' ').count(), 6);
        assert_eq!(line.matches('=').count(), 5);
        for l in [
            fmt_rx_decrypt(evil, evil, evil),
            fmt_custody_accept(evil, evil, evil),
            fmt_delivery_state_pending(evil, evil),
        ] {
            assert_eq!(l.lines().count(), 1);
            assert!(!l.contains('\r') && !l.contains('\t'));
        }
    }

    #[test]
    fn sanitize_caps_length_and_keeps_normal_ids() {
        let long = "x".repeat(1000);
        assert_eq!(sanitize_log_field(&long, 128).chars().count(), 128);
        assert_eq!(
            fmt_rx_history("abc-123", "p", true, false, true),
            "rx_history msg=abc-123 from=p result=ok dup=false hidden=true"
        );
        assert_eq!(
            fmt_custody_accept("m1", "a", "b"),
            "custody_accept msg=m1 from=a dest=b"
        );
    }

    #[test]
    fn ledger_marker_sanitizes_multiaddr() {
        let addr = format!(
            "/ip4/1.2.3.4/tcp/1\nrx_history msg=1 result=ok{}",
            "y".repeat(500)
        );
        let line = fmt_ledger_address_learned("peer", "via x=1\n", &addr);
        assert_eq!(line.lines().count(), 1);
        assert_eq!(line.matches('=').count(), 3);
        assert!(line.chars().count() < 400);
        assert_eq!(
            fmt_ledger_address_learned("p", "unknown", "/ip4/1.2.3.4/tcp/9"),
            "ledger_address_learned peer=p via=unknown addr=/ip4/1.2.3.4/tcp/9"
        );
    }

    #[test]
    fn routing_and_transport_markers_are_single_line_and_closed_grammar() {
        let l = fmt_routing_peer_seen(
            "abc
from=x",
            "ble
rx_history msg=1",
        );
        assert_eq!(l.lines().count(), 1);
        assert_eq!(l.matches('=').count(), 2);
        assert!(l.starts_with("[ROUTING] peer_seen peer="));

        assert_eq!(
            fmt_transport_status("ble", "unavailable", None, "no adapter"),
            "[TRANSPORT] kind=ble state=unavailable detail=no_adapter"
        );
        assert_eq!(
            fmt_transport_status("quic", "connected", Some(3), "periodic"),
            "[TRANSPORT] kind=quic state=connected peers=3 detail=periodic"
        );
        let bad = fmt_transport_status(
            "evil
kind",
            "up",
            None,
            "x=1
y",
        );
        assert_eq!(bad.lines().count(), 1);
        assert!(bad.contains("kind=invalid state=error"));
        assert_eq!(bad.matches('=').count(), 3);
    }

    #[test]
    fn ring_ids_are_sanitized() {
        assert_eq!(clamp_id("a\nb"), "a_b");
    }

    #[test]
    fn ring_is_bounded() {
        let mut ring = MessageEventRing::with_capacity(8);
        for i in 0..100 {
            ring.push(ev(&format!("m{i}"), MessageEventKind::Sent));
        }
        assert_eq!(ring.len(), 8);
        let all = ring.recent(100, 100);
        assert_eq!(all.len(), 8);
        assert_eq!(all[0].0, "m99");
    }

    #[test]
    fn recent_limits_ids_and_events_per_id() {
        let mut ring = MessageEventRing::with_capacity(64);
        ring.push(ev("a", MessageEventKind::Sent));
        ring.push(ev("b", MessageEventKind::Sent));
        ring.push(ev("a", MessageEventKind::Custody));
        ring.push(ev("a", MessageEventKind::Decrypt));
        ring.push(ev("a", MessageEventKind::History));
        let out = ring.recent(1, 2);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "a");
        let kinds: Vec<_> = out[0].1.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            vec![MessageEventKind::Decrypt, MessageEventKind::History]
        );
    }

    #[test]
    fn zero_capacity_is_clamped() {
        let mut ring = MessageEventRing::with_capacity(0);
        ring.push(ev("a", MessageEventKind::Sent));
        ring.push(ev("b", MessageEventKind::Sent));
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn global_record_clamps_id_length() {
        let long = "x".repeat(MAX_ID_CHARS * 4);
        record(&long, MessageEventKind::Receipt, true);
        let found = recent_message_events(MAX_EVENTS, 1)
            .into_iter()
            .any(|(id, _)| id.len() == MAX_ID_CHARS);
        assert!(found);
    }

    #[test]
    fn marker_formats_are_stable() {
        assert_eq!(
            fmt_delivery_state_pending("m1", "core_envelope_prepared"),
            "delivery_state msg=m1 state=pending detail=core_envelope_prepared"
        );
        assert_eq!(
            fmt_rx_decrypt("m1", "0123456789abcdef0123", "text"),
            "rx_decrypt msg=m1 from=0123456789abcdef type=text result=ok"
        );
        assert_eq!(
            fmt_rx_history("m1", "peer", false, true, false),
            "rx_history msg=m1 from=peer result=failed dup=true hidden=false"
        );
        assert_eq!(
            fmt_custody_accept("m1", "aaaaaaaaaaaaaaaaZZZ", "bbbb"),
            "custody_accept msg=m1 from=aaaaaaaaaaaaaaaa dest=bbbb"
        );
        assert_eq!(
            fmt_ledger_address_learned("p", "unknown", "/ip4/1.2.3.4/tcp/9"),
            "ledger_address_learned peer=p via=unknown addr=/ip4/1.2.3.4/tcp/9"
        );
    }

    #[test]
    fn rx_drop_formats_are_sanitized_and_single_line() {
        let evil = "a
b c=d";
        let l = fmt_rx_drop(Some(evil), evil, evil);
        assert_eq!(l.lines().count(), 1);
        assert_eq!(l.matches('=').count(), 3);
        assert_eq!(
            fmt_rx_drop(None, "inbox", "duplicate"),
            "[RX-DROP] msg=- stage=inbox reason=duplicate"
        );
        assert_eq!(
            fmt_rx_drop_kind("swarm_unhandled", "PeerExchange"),
            "[RX-DROP] stage=swarm_unhandled kind=PeerExchange"
        );
        assert_eq!(
            fmt_rx_forwarded("peer_exchange"),
            "[RX] forwarded kind=peer_exchange"
        );
        assert_eq!(
            fmt_rx_drift_frame("Envelope", 42, "12D3KooWabcdefgh01234567"),
            "[RX] drift_frame type=Envelope payload_len=42 from=01234567"
        );
        assert_eq!(short_peer_tail("abc"), "abc");
        assert_eq!(
            short_peer_tail(
                "a=b c
d1234567"
            ),
            "d1234567"
        );
        assert_eq!(
            fmt_rx_drop_suppressed(7, "inbox"),
            "[RX-DROP] suppressed=7 stage=inbox"
        );
        assert_eq!(
            fmt_rx_stall_drain(30000, 12),
            "[RX-STALL] drain_loop idle_ms=30000 backlog=12"
        );
    }

    #[test]
    fn rx_drop_gate_rate_limits_and_counts_suppressed() {
        let mut map = DropLimiter::new();
        let t0 = web_time::Instant::now();
        assert_eq!(rx_drop_gate_in(&mut map, "k", t0), Some(0));
        assert_eq!(rx_drop_gate_in(&mut map, "k", t0), None);
        assert_eq!(rx_drop_gate_in(&mut map, "k", t0), None);
        let later = t0 + RX_DROP_LOG_INTERVAL;
        assert_eq!(rx_drop_gate_in(&mut map, "k", later), Some(2));
        assert_eq!(rx_drop_gate_in(&mut map, "other", later), Some(0));
    }

    #[test]
    fn rx_drop_gate_bounds_key_space() {
        let mut map = DropLimiter::new();
        let t0 = web_time::Instant::now();
        for i in 0..RX_DROP_LIMITER_MAX_KEYS {
            assert!(rx_drop_gate_in(&mut map, &format!("k{}", i), t0).is_some());
        }
        assert_eq!(rx_drop_gate_in(&mut map, "overflow", t0), None);
        assert_eq!(map.len(), RX_DROP_LIMITER_MAX_KEYS);
    }
}
