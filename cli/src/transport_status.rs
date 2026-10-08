// Per-transport availability markers for the CLI / relay node.
//
// Emits `[TRANSPORT] kind=<k> state=<s> [peers=<n>] detail=<reason>` lines
// (format owned by `scmessenger_core::message_events::fmt_transport_status`)
// at startup, on every state change, and a per-transport connected-peer count
// every `PEER_COUNT_INTERVAL`. The goal is that pulled logs alone say which
// transports exist, which are listening, and how many peers ride each.

use crate::ble_daemon::{BleError, BleStatus};
use crate::ledger::ConnectionLedger;
use scmessenger_core::message_events::fmt_transport_status;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

/// How often per-transport connected-peer counts are logged.
pub const PEER_COUNT_INTERVAL: Duration = Duration::from_secs(300);

type StateMap = HashMap<&'static str, (String, String)>;

fn registry() -> MutexGuard<'static, StateMap> {
    static REG: OnceLock<Mutex<StateMap>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn canonical_kind(kind: &str) -> Option<&'static str> {
    scmessenger_core::message_events::TRANSPORT_KINDS
        .iter()
        .copied()
        .find(|k| *k == kind)
}

/// Record a transport state and log it if it differs from the last one
/// recorded for that kind. Returns true when a line was emitted.
pub fn set_state(kind: &str, state: &str, detail: &str) -> bool {
    let Some(kind) = canonical_kind(kind) else {
        return false;
    };
    {
        let mut reg = registry();
        if let Some((s, d)) = reg.get(kind) {
            if s == state && d == detail {
                return false;
            }
        }
        reg.insert(kind, (state.to_string(), detail.to_string()));
    }
    tracing::info!(
        event = "transport_status",
        "{}",
        fmt_transport_status(kind, state, None, detail)
    );
    true
}

/// Classify a multiaddr string into a transport kind.
///
/// Relay circuits are checked first (a circuit address also contains the
/// relay's tcp/quic components). Returns None for addresses that are not an
/// IP transport (e.g. a bare /p2p/ id).
pub fn classify_multiaddr(addr: &str) -> Option<&'static str> {
    if addr.contains("/p2p-circuit") {
        return Some("relay");
    }
    if addr.contains("/quic") {
        return Some("quic");
    }
    if addr.contains("/tcp/") || addr.contains("/ws") {
        if addr.starts_with("/ip6/") || addr.starts_with("/dns6/") {
            return Some("tcp6");
        }
        if addr.starts_with("/ip4/") || addr.starts_with("/dns4/") || addr.starts_with("/dns/") {
            return Some("tcp4");
        }
    }
    None
}

/// Called when the swarm reports it is listening on `addr`.
pub fn on_listening(addr: &str) {
    if let Some(kind) = classify_multiaddr(addr) {
        // Only the port goes into detail: the listen address itself can
        // include a LAN IP.
        let detail = match addr.split('/').find_map(|seg| {
            (!seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit())).then_some(seg)
        }) {
            Some(port) => format!("listen_port_{}", port),
            None => "listening".to_string(),
        };
        set_state(kind, "listening", &detail);
    }
}

/// Called when a listener failed to bind or died.
pub fn on_listener_failed(error: &str) {
    // The failed listener's family is not carried by the event; report on
    // tcp4 (the primary listener) with the reason, never claim it is up.
    set_state("tcp4", "error", &format!("listener_failed:{}", error));
}

/// Map a BLE probe result to `(state, detail)`.
pub fn ble_probe_to_state(status: &BleStatus) -> (&'static str, String) {
    match status {
        BleStatus::Available(adapters) => ("available", format!("adapters_{}", adapters.len())),
        BleStatus::Disabled => ("unavailable", "disabled".to_string()),
        BleStatus::Unavailable(err) => match err {
            BleError::NoAdapter => ("unavailable", "no adapter".to_string()),
            BleError::PermissionDenied => ("unavailable", "permission denied".to_string()),
            BleError::AdapterNotPowered => ("unavailable", "adapter not powered".to_string()),
            BleError::Timeout => ("error", "probe timeout".to_string()),
            BleError::ManagerInitFailed(e) | BleError::Other(e) => {
                let low = e.to_ascii_lowercase();
                if low.contains("dbus") || low.contains("d-bus") {
                    ("unavailable", "no D-Bus".to_string())
                } else if low.contains("not supported") {
                    ("unavailable", "not supported on platform".to_string())
                } else {
                    ("error", format!("probe failed: {}", e))
                }
            }
        },
    }
}

/// Count connected peers per transport kind from their ledger addresses.
/// Returns (per-kind counts, unclassified count).
pub fn count_by_kind<I, F>(peers: I, lookup: F) -> (HashMap<&'static str, usize>, usize)
where
    I: IntoIterator<Item = String>,
    F: Fn(&str) -> Option<String>,
{
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let mut unclassified = 0usize;
    for peer in peers {
        match lookup(&peer).as_deref().and_then(classify_multiaddr) {
            Some(kind) => *counts.entry(kind).or_insert(0) += 1,
            None => unclassified += 1,
        }
    }
    (counts, unclassified)
}

/// Log the connected-peer count for every transport kind that is known
/// (listening/available/...) or has peers.
pub fn log_peer_counts(counts: &HashMap<&'static str, usize>, unclassified: usize) {
    let total: usize = counts.values().sum::<usize>() + unclassified;
    let known: Vec<(&'static str, String)> = {
        let reg = registry();
        reg.iter().map(|(k, (s, _))| (*k, s.clone())).collect()
    };
    let mut kinds: Vec<&'static str> = known.iter().map(|(k, _)| *k).collect();
    for k in counts.keys() {
        if !kinds.contains(k) {
            kinds.push(k);
        }
    }
    kinds.sort_unstable();
    for kind in kinds {
        let n = counts.get(kind).copied().unwrap_or(0);
        let state = if n > 0 {
            "connected".to_string()
        } else {
            known
                .iter()
                .find(|(k, _)| *k == kind)
                .map(|(_, s)| s.clone())
                .unwrap_or_else(|| "available".to_string())
        };
        let detail = format!("periodic total_{}_unclassified_{}", total, unclassified);
        tracing::info!(
            event = "transport_peer_count",
            "{}",
            fmt_transport_status(kind, &state, Some(n), &detail)
        );
    }
}

/// Report the static transports once, probe BLE, and start the periodic
/// per-transport peer-count task. Safe to call once per node process.
pub fn spawn(
    enable_mdns: bool,
    enable_ble: bool,
    peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>>,
    ledger: Arc<tokio::sync::Mutex<ConnectionLedger>>,
) {
    // The swarm always compiles in these behaviours; quic is reported
    // `listening` only when a quic listener actually binds (on_listening).
    set_state("quic", "unavailable", "no quic listener bound yet");
    set_state(
        "relay",
        "available",
        "relay client+server behaviours compiled in",
    );
    set_state("dcutr", "available", "dcutr behaviour compiled in");
    if enable_mdns {
        set_state("mdns", "available", "discovery mode open");
    } else {
        set_state(
            "mdns",
            "unavailable",
            "disabled by config enable_mdns=false",
        );
    }
    set_state("wifi_direct", "unavailable", "not applicable on cli nodes");
    set_state("wifi_aware", "unavailable", "not applicable on cli nodes");
    set_state("cellular", "unavailable", "not applicable on cli nodes");

    if !enable_ble {
        set_state("ble", "unavailable", "disabled by config enable_ble=false");
    } else if cfg!(target_os = "macos") {
        set_state(
            "ble",
            "available",
            "adapter probe skipped on macos; central ingress owns corebluetooth",
        );
    } else {
        tokio::spawn(async move {
            let status = crate::ble_daemon::probe_and_log().await;
            let (state, detail) = ble_probe_to_state(&status);
            set_state("ble", state, &detail);
        });
    }

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(PEER_COUNT_INTERVAL);
        loop {
            tick.tick().await;
            let ids: Vec<libp2p::PeerId> = peers.lock().await.keys().cloned().collect();
            let (counts, unclassified) = {
                let l = ledger.lock().await;
                count_by_kind(ids.iter().map(|p| p.to_string()), |p| {
                    l.find_peer_multiaddr(p)
                })
            };
            log_peer_counts(&counts, unclassified);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_multiaddrs() {
        assert_eq!(classify_multiaddr("/ip4/1.2.3.4/tcp/9001"), Some("tcp4"));
        assert_eq!(classify_multiaddr("/ip6/::1/tcp/9001"), Some("tcp6"));
        assert_eq!(
            classify_multiaddr("/ip4/1.2.3.4/udp/9001/quic-v1"),
            Some("quic")
        );
        assert_eq!(
            classify_multiaddr("/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo/p2p-circuit"),
            Some("relay")
        );
        assert_eq!(classify_multiaddr("/p2p/12D3Koo"), None);
    }

    #[test]
    fn ble_probe_states_name_the_reason() {
        let (s, d) = ble_probe_to_state(&BleStatus::Unavailable(BleError::NoAdapter));
        assert_eq!((s, d.as_str()), ("unavailable", "no adapter"));
        let (s, d) = ble_probe_to_state(&BleStatus::Unavailable(BleError::ManagerInitFailed(
            "D-Bus error: connection refused".to_string(),
        )));
        assert_eq!((s, d.as_str()), ("unavailable", "no D-Bus"));
        let (s, _) = ble_probe_to_state(&BleStatus::Available(Vec::new()));
        assert_eq!(s, "available");
    }

    #[test]
    fn counts_peers_per_kind() {
        let peers = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let (counts, unclassified) = count_by_kind(peers, |p| match p {
            "a" => Some("/ip4/1.1.1.1/tcp/1".to_string()),
            "b" => Some("/ip4/1.1.1.2/tcp/1".to_string()),
            "c" => Some("/ip4/1.1.1.3/udp/1/quic-v1".to_string()),
            _ => None,
        });
        assert_eq!(counts.get("tcp4"), Some(&2));
        assert_eq!(counts.get("quic"), Some(&1));
        assert_eq!(unclassified, 1);
    }

    #[test]
    fn set_state_dedupes_unchanged() {
        assert!(set_state("mdns", "available", "unit test detail a"));
        assert!(!set_state("mdns", "available", "unit test detail a"));
        assert!(set_state("mdns", "error", "unit test detail b"));
        assert!(!set_state("not_a_kind", "available", "x"));
    }
}
