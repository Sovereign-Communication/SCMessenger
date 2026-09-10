// Envelope-sourced dial-candidate store ("hint store").
//
// Identity envelopes (`scm.message.identity.v1`) carry the sender's fresh
// listeners / external_addresses / connection_hints. This module keeps the
// most recent set per libp2p PeerId so outbound paths can DIAL those hints
// when the routing table still only knows stale LAN addresses — the exact
// live-cell failure seen 2026-08-25 (phone on cellular, Windows CLI holding
// stale `route=direct relay=- candidate=1/1` candidates only).
//
// Deliberately process-global: envelope parsing happens on the CLI event
// loop while dial decisions happen inside the swarm task, and both live in
// one process per node (CLI, Android FFI, headless relay alike).

use crate::transport::addr_filter::{is_dialable_multiaddr_parsed, DnsPolicy, NetworkMode};
use libp2p::{Multiaddr, PeerId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Cap on stored candidates per peer, mirroring the envelope caps
/// (MAX_CONNECTION_HINTS = 6) with a little headroom.
pub const MAX_HINTS_PER_PEER: usize = 8;

/// Hints older than this are dropped: cellular/NAT mappings churn, and
/// dialing a day-old address is pure backoff noise.
pub const HINT_TTL_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone)]
struct PeerHintEntry {
    addrs: Vec<Multiaddr>,
    updated_unix: u64,
}

fn hint_entries() -> &'static RwLock<HashMap<PeerId, PeerHintEntry>> {
    static HINTS: OnceLock<RwLock<HashMap<PeerId, PeerHintEntry>>> = OnceLock::new();
    HINTS.get_or_init(|| RwLock::new(HashMap::new()))
}

fn now_unix() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Strip any trailing `/p2p/<peer>` component: the peer identity is pinned by
/// `DialOpts::peer_id` at dial time, and keeping it would duplicate the
/// component libp2p appends itself.
fn transport_only(addr: &Multiaddr) -> Multiaddr {
    addr.iter()
        .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
        .collect()
}

/// Record freshly learned route hints for a peer from an identity envelope.
///
/// Each hint must parse as a Multiaddr and pass the same dialability filter
/// used at every other remote-supplied-address boundary (SSRF gate:
/// `NetworkMode::Local` + `DnsPolicy::Reject`). Newer annotations replace the
/// previous set wholesale — stale addresses never outlive a fresher envelope.
pub fn annotate_peer_hints(peer_id: PeerId, hints: &[String]) {
    let mut addrs: Vec<Multiaddr> = Vec::with_capacity(hints.len());
    for hint in hints {
        let trimmed = hint.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(addr) = trimmed.parse::<Multiaddr>() else {
            continue;
        };
        let stripped = transport_only(&addr);
        if stripped.is_empty() {
            continue;
        }
        if !is_dialable_multiaddr_parsed(&stripped, NetworkMode::Local, DnsPolicy::Reject) {
            continue;
        }
        if !addrs.contains(&stripped) {
            addrs.push(stripped);
        }
        if addrs.len() >= MAX_HINTS_PER_PEER {
            break;
        }
    }
    if addrs.is_empty() {
        return;
    }
    hint_entries().write().insert(
        peer_id,
        PeerHintEntry {
            addrs,
            updated_unix: now_unix(),
        },
    );
}

/// Fresh, dialable envelope-sourced candidates for a peer, newest first.
/// Returns an empty vector when nothing was annotated or the entry expired.
pub fn peer_hint_dial_candidates(peer_id: PeerId) -> Vec<Multiaddr> {
    let now = now_unix();
    let mut entries = hint_entries().write();
    match entries.get(&peer_id) {
        Some(entry) if now.saturating_sub(entry.updated_unix) <= HINT_TTL_SECS => {
            entry.addrs.clone()
        }
        Some(_) => {
            // Expired: drop eagerly so repeated sends do not keep re-reading it.
            entries.remove(&peer_id);
            Vec::new()
        }
        None => Vec::new(),
    }
}

/// Whether any (unexpired) hints exist for this peer.
#[allow(dead_code)]
pub fn has_peer_hints(peer_id: PeerId) -> bool {
    !peer_hint_dial_candidates(peer_id).is_empty()
}

/// Drop every hint entry older than `max_age` (memory hygiene).
pub fn prune_older_than(max_age_secs: u64) {
    let now = now_unix();
    hint_entries()
        .write()
        .retain(|_, entry| now.saturating_sub(entry.updated_unix) <= max_age_secs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid() -> PeerId {
        libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id()
    }

    #[test]
    fn annotate_keeps_dialable_hints_and_strips_p2p_suffix() {
        let peer = pid();
        let other = format!("/ip4/10.42.0.19/tcp/45123/p2p/{}", pid());
        let hints = vec![
            other.clone(),
            "/ip4/203.0.113.7/tcp/4001".to_string(),
            "not-a-multiaddr".to_string(),
            "".to_string(),
            "/dns4/repointable.example/tcp/4001".to_string(),
        ];
        annotate_peer_hints(peer, &hints);

        let candidates = peer_hint_dial_candidates(peer);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"/ip4/10.42.0.19/tcp/45123".parse::<Multiaddr>().unwrap()));
        assert!(candidates.contains(&"/ip4/203.0.113.7/tcp/4001".parse::<Multiaddr>().unwrap()));
        for candidate in &candidates {
            assert!(!candidate
                .iter()
                .any(|p| matches!(p, libp2p::multiaddr::Protocol::P2p(_))));
        }
    }

    #[test]
    fn newer_annotation_replaces_previous_set() {
        let peer = pid();
        annotate_peer_hints(peer, &["/ip4/192.168.0.121/tcp/9090".to_string()]);
        assert_eq!(peer_hint_dial_candidates(peer).len(), 1);

        // Phone moved to cellular: fresh envelope carries only the new reachables.
        annotate_peer_hints(peer, &["/ip4/100.66.12.34/tcp/41234".to_string()]);
        let candidates = peer_hint_dial_candidates(peer);
        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&"/ip4/100.66.12.34/tcp/41234".parse::<Multiaddr>().unwrap()));
    }

    #[test]
    fn unknown_peer_has_no_candidates_and_cap_is_enforced() {
        assert!(!has_peer_hints(pid()));

        let peer = pid();
        let many: Vec<String> = (0..20)
            .map(|i| format!("/ip4/10.0.0.{}/tcp/{}", i, 1000 + i))
            .collect();
        annotate_peer_hints(peer, &many);
        assert_eq!(peer_hint_dial_candidates(peer).len(), MAX_HINTS_PER_PEER);
    }

    #[test]
    fn restricted_hosts_never_become_hints() {
        let peer = pid();
        annotate_peer_hints(
            peer,
            &[
                "/ip4/169.254.169.254/tcp/80".to_string(),
                "/ip4/127.0.0.1/tcp/8080".to_string(),
                "/ip6/::1/tcp/9090".to_string(),
            ],
        );
        // Loopback is filtered by the dialability gate; metadata endpoints too.
        assert!(peer_hint_dial_candidates(peer).is_empty());
    }
}
