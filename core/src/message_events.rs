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

fn clamp_id(id: &str) -> String {
    id.chars().take(MAX_ID_CHARS).collect()
}

/// Truncate a peer id for logging (first `PEER_LOG_CHARS` chars).
pub fn short_peer(peer: &str) -> String {
    peer.chars().take(PEER_LOG_CHARS).collect()
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
        msg_id, detail
    )
}

/// G2: `rx_decrypt msg=<id> from=<peer16> type=<text|receipt|other> result=ok`
pub fn fmt_rx_decrypt(msg_id: &str, peer: &str, kind: &str) -> String {
    format!(
        "rx_decrypt msg={} from={} type={} result=ok",
        msg_id,
        short_peer(peer),
        kind
    )
}

/// G3: `rx_history msg=<id> from=<peer16> result=<ok|failed> dup=<bool> hidden=<bool>`
pub fn fmt_rx_history(msg_id: &str, peer: &str, ok: bool, dup: bool, hidden: bool) -> String {
    format!(
        "rx_history msg={} from={} result={} dup={} hidden={}",
        msg_id,
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
        msg_id,
        short_peer(from),
        short_peer(dest)
    )
}

/// G1: `ledger_address_learned peer=<peer16> via=<via> addr=<multiaddr>`
pub fn fmt_ledger_address_learned(peer: &str, via: &str, addr: &str) -> String {
    format!(
        "ledger_address_learned peer={} via={} addr={}",
        short_peer(peer),
        via,
        addr
    )
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
}
