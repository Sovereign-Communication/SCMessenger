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

/// Maximum characters of a message id kept in an `[RX-DROP]` marker.
const MAX_DROP_ID_CHARS: usize = 64;

/// Reduce an untrusted id/reason fragment to `[A-Za-z0-9_.:-]` and bound its
/// length so a hostile envelope cannot inject log lines or spaces into the
/// stable `[RX-DROP]` marker.
fn sanitize_drop_field(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
        .take(MAX_DROP_ID_CHARS)
        .collect();
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

/// RX-DROP: `[RX-DROP] msg=<id> stage=<stage> reason=<reason>`
///
/// Emitted at INFO for every inbound frame that is discarded before it is
/// persisted, so a silently dropped message is always attributable.
pub fn fmt_rx_drop(msg_id: &str, stage: &str, reason: &str) -> String {
    format!(
        "[RX-DROP] msg={} stage={} reason={}",
        sanitize_drop_field(msg_id),
        sanitize_drop_field(stage),
        sanitize_drop_field(reason)
    )
}

/// Log an inbound drop at INFO using the stable [`fmt_rx_drop`] format.
pub fn log_rx_drop(msg_id: &str, stage: &str, reason: &str) {
    tracing::info!("{}", fmt_rx_drop(msg_id, stage, reason));
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
    fn rx_drop_marker_is_stable_and_sanitized() {
        assert_eq!(
            fmt_rx_drop("5e7b311b-ef47", "receive", "decrypt_failed"),
            "[RX-DROP] msg=5e7b311b-ef47 stage=receive reason=decrypt_failed"
        );
        let hostile = fmt_rx_drop("id\n[RX-DROP] msg=evil", "st age", "");
        assert!(!hostile.contains('\n'));
        assert_eq!(hostile.matches("msg=").count(), 1);
        assert!(hostile.ends_with("reason=unknown"));
        let long = fmt_rx_drop(&"a".repeat(500), "s", "r");
        assert!(long.len() < 200);
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
}
