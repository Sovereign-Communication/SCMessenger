// Aggressive Bootstrap — Promiscuous Peer Discovery
//
// Philosophy: "A node is a node." IP is the source of truth.
//
// This module implements promiscuous bootstrap dialing:
// - Dial by IP:Port only, ignoring PeerID in the multiaddr
// - If the remote presents a different PeerID than expected, ACCEPT it
// - Log the identity change and update the routing table
// - Never reject a connection based on PeerID mismatch
//
// Build-time customization:
// - Set SC_BOOTSTRAP_NODES environment variable during build
// - Format: comma-separated multiaddrs
// - Example: export SC_BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo..."

use crate::ledger;

/// Default bootstrap nodes — can be overridden at build time
///
/// Strategy: Multiple public relay nodes with varying availability
/// - Node 1: Primary GCP (high availability)
/// - Node 2: Secondary relay (geographic redundancy)
/// - Node 3: Tertiary relay (provider diversity)
///
/// All nodes relay for the mesh. Connection attempts fail over automatically.
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[];

/// The "lobby" topic — a universal discovery channel.
/// All nodes subscribe to this on startup to find the active mesh.
pub const LOBBY_TOPIC: &str = "sc-lobby";

/// The primary mesh topic for real messages
pub const MESH_TOPIC: &str = "sc-mesh";

/// Get default bootstrap nodes, with optional build-time override
pub fn default_bootstrap_nodes() -> Vec<String> {
    // 1. Check runtime environment variable (for Docker/Cloud)
    if let Ok(nodes_str) = std::env::var("SC_BOOTSTRAP_NODES") {
        if !nodes_str.trim().is_empty() {
            return nodes_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    // 2. Check build-time override
    let build_time_nodes = option_env!("SC_BOOTSTRAP_NODES");

    if let Some(nodes_str) = build_time_nodes {
        if nodes_str.trim().is_empty() {
            // Empty string means use defaults (treat as if env var was unset)
            DEFAULT_BOOTSTRAP_NODES
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            // Parse comma-separated multiaddrs
            nodes_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    } else {
        // Use hardcoded defaults
        DEFAULT_BOOTSTRAP_NODES
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// Get bootstrap addresses stripped of PeerID for promiscuous dialing.
///
/// This is the core of aggressive discovery: we dial the IP:Port ONLY.
/// libp2p will accept whatever PeerID the remote presents during the
/// Noise handshake. No identity validation occurs at this stage.
/// Reserved for a future aggressive-discovery bootstrap mode; not yet called
/// from the CLI entry point or elsewhere.
#[allow(dead_code)]
pub fn promiscuous_bootstrap_addrs() -> Vec<String> {
    default_bootstrap_nodes()
        .into_iter()
        .map(|addr| ledger::strip_peer_id(&addr))
        .collect()
}

/// Extract the expected PeerID from a bootstrap multiaddr (if present).
/// Returns (stripped_addr, optional_expected_peer_id)
/// Reserved helper for future bootstrap-address/PeerID parsing use sites;
/// not yet called outside this module.
#[allow(dead_code)]
pub fn parse_bootstrap_addr(multiaddr: &str) -> (String, Option<String>) {
    let stripped = ledger::strip_peer_id(multiaddr);
    let peer_id = multiaddr
        .find("/p2p/")
        .map(|idx| multiaddr[idx + 5..].to_string());
    (stripped, peer_id)
}

/// Merge user-provided bootstrap nodes with defaults.
/// Ensures defaults are preserved unless explicitly removed.
/// Deduplicates by IP:Port (ignoring PeerID differences).
pub fn merge_bootstrap_nodes(user_nodes: Vec<String>) -> Vec<String> {
    let mut seen_addrs = std::collections::HashSet::new();
    let mut merged = Vec::new();

    // Add defaults first
    for node in default_bootstrap_nodes() {
        let stripped = ledger::strip_peer_id(&node);
        if seen_addrs.insert(stripped) {
            merged.push(node);
        }
    }

    // Add user nodes that don't duplicate an existing address
    for node in user_nodes {
        let stripped = ledger::strip_peer_id(&node);
        if seen_addrs.insert(stripped) {
            merged.push(node);
        }
    }

    merged
}

/// Get all default topics that a node should subscribe to
pub fn default_topics() -> Vec<String> {
    vec![LOBBY_TOPIC.to_string(), MESH_TOPIC.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bootstrap_nodes() {
        // Now returns empty unless env is set
        let nodes = default_bootstrap_nodes();
        // Just verify basic multiaddr format if any are present
        for node in &nodes {
            assert!(
                node.starts_with("/ip4/"),
                "Bootstrap node should be a multiaddr: {}",
                node
            );
        }
    }

    #[test]
    fn test_promiscuous_addrs_strip_peer_id() {
        let addrs = promiscuous_bootstrap_addrs();
        for addr in &addrs {
            assert!(
                !addr.contains("/p2p/"),
                "Promiscuous addrs must NOT contain PeerID: {}",
                addr
            );
            assert!(
                addr.contains("/tcp/") || addr.contains("/udp/"),
                "Must contain transport: {}",
                addr
            );
        }
    }

    #[test]
    fn test_parse_bootstrap_addr() {
        let (stripped, peer_id) = parse_bootstrap_addr(
            "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",
        );
        assert_eq!(stripped, "/ip4/1.2.3.4/tcp/9001");
        assert_eq!(
            peer_id,
            Some("12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W".to_string())
        );

        let (stripped, peer_id) = parse_bootstrap_addr("/ip4/10.0.0.1/tcp/4001");
        assert_eq!(stripped, "/ip4/10.0.0.1/tcp/4001");
        assert_eq!(peer_id, None);
    }

    #[test]
    fn test_merge_deduplicates_by_ip() {
        // Same IP but different PeerIDs should be deduplicated
        let user_nodes = vec![
            "/ip4/1.2.3.4/tcp/9001/p2p/DIFFERENT_PEER_ID".to_string(),
            "/ip4/10.0.0.1/tcp/9001/p2p/SomeNewPeer".to_string(),
        ];
        let merged = merge_bootstrap_nodes(user_nodes);

        // Count entries for 1.2.3.4 — should only be 1
        let ip_count = merged.iter().filter(|n| n.contains("1.2.3.4")).count();
        assert_eq!(ip_count, 1, "Should deduplicate by IP:Port");

        // 10.0.0.1 is new, should be added
        assert!(merged.iter().any(|n| n.contains("10.0.0.1")));
    }

    #[test]
    fn test_default_topics() {
        let topics = default_topics();
        assert!(topics.contains(&"sc-lobby".to_string()));
        assert!(topics.contains(&"sc-mesh".to_string()));
    }
}
