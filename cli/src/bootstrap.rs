// Bootstrap helpers (#469)
//
// There are NO static bootstrap seeds: no environment variable, no build-time
// option, no compiled-in or config-supplied address list. The only seed
// sources are an invite (`SCI1:` seed ledger) and live discovery (LAN/BLE).
// Dial candidates are always derived from the peer ledger.
//
// The single remaining legacy read is the one-time migration of a pre-#469
// config's `bootstrap_nodes` key into the ledger as UNPROVEN seed entries, so
// existing deployments keep their connectivity. After the import the key is
// dropped from the config file.

use crate::ledger;
use scmessenger_core::store::{LedgerManager, SeedLedgerEntry, MAX_SEED_LEDGER_ENTRIES};
use scmessenger_core::{TOPIC_LOBBY, TOPIC_MESH};

/// Upper bound on ledger-derived dial candidates handed to the swarm at boot.
const LEDGER_CANDIDATE_LIMIT: u32 = 64;

/// Outcome of the one-time legacy config migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyMigrationOutcome {
    /// Legacy addresses found in the config.
    pub found: usize,
    /// Addresses newly added to the ledger as unproven seed entries.
    pub imported: usize,
}

/// Import legacy config-supplied bootstrap addresses into the ledger as
/// unproven seeds, draining `legacy` in the process.
///
/// Seed import enforces the ledger's own safety filters (non-routable and DNS
/// forms are rejected, peer-id components are stripped, duplicates collapse),
/// so a hostile or stale config cannot do more than an invite could. Entries
/// are fed in batches of `MAX_SEED_LEDGER_ENTRIES` because the import caps each
/// call. The caller persists the config afterwards, which drops the key.
pub fn migrate_legacy_bootstrap_nodes(
    legacy: &mut Vec<String>,
    ledger_manager: &LedgerManager,
) -> LegacyMigrationOutcome {
    let addrs = std::mem::take(legacy);
    let found = addrs.len();
    let mut imported = 0usize;
    for batch in addrs.chunks(MAX_SEED_LEDGER_ENTRIES.max(1)) {
        let entries: Vec<SeedLedgerEntry> = batch
            .iter()
            .map(|a| SeedLedgerEntry {
                multiaddr: a.trim().to_string(),
            })
            .collect();
        imported += ledger_manager.import_seed_entries(entries) as usize;
    }
    LegacyMigrationOutcome { found, imported }
}

/// Dial candidates derived from the peer ledger only: proven peers first
/// (best-ranked), then unproven seeds. Peer-id components are not included.
pub fn ledger_candidate_addrs(ledger_manager: &LedgerManager) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in ledger_manager.get_preferred_relays(LEDGER_CANDIDATE_LIMIT) {
        if !out.contains(&e.multiaddr) {
            out.push(e.multiaddr);
        }
    }
    for e in ledger_manager.seed_addresses(LEDGER_CANDIDATE_LIMIT) {
        if !out.contains(&e.multiaddr) {
            out.push(e.multiaddr);
        }
    }
    out
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

/// Get all default topics that a node should subscribe to
pub fn default_topics() -> Vec<String> {
    vec![TOPIC_LOBBY.to_string(), TOPIC_MESH.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bootstrap_addr() {
        let (stripped, peer_id) = parse_bootstrap_addr(
            "/ip4/192.0.2.4/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",
        );
        assert_eq!(stripped, "/ip4/192.0.2.4/tcp/9001");
        assert_eq!(
            peer_id,
            Some("12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W".to_string())
        );

        let (stripped, peer_id) = parse_bootstrap_addr("/ip4/10.0.0.1/tcp/4001");
        assert_eq!(stripped, "/ip4/10.0.0.1/tcp/4001");
        assert_eq!(peer_id, None);
    }

    #[test]
    fn test_default_topics() {
        let topics = default_topics();
        assert!(topics.contains(&TOPIC_LOBBY.to_string()));
        assert!(topics.contains(&TOPIC_MESH.to_string()));
    }

    #[test]
    fn test_env_var_is_not_a_seed_source() {
        // SC_BOOTSTRAP_NODES is no longer read anywhere: candidates come from
        // the ledger alone, so a fresh ledger yields none regardless of env.
        let lm = LedgerManager::ephemeral();
        assert!(ledger_candidate_addrs(&lm).is_empty());
    }

    #[test]
    fn test_migration_imports_legacy_as_unproven_seeds_and_drains() {
        let lm = LedgerManager::ephemeral();
        let mut legacy = vec![
            "/ip4/10.1.0.1/tcp/9001".to_string(),
            // Same address with a peer id: must dedupe onto the first.
            "/ip4/10.1.0.1/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W"
                .to_string(),
            "/ip4/10.1.0.2/tcp/9001".to_string(),
            // DNS forms are rejected by seed import.
            "/dns4/relay.example/tcp/9001".to_string(),
        ];
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &lm);
        assert_eq!(out.found, 4);
        assert_eq!(out.imported, 2);
        assert!(legacy.is_empty(), "legacy list must be drained");

        // Unproven: never in the proven/dialable tier, present as seeds.
        assert!(lm.get_preferred_relays(10).is_empty());
        let seeds = lm.seed_addresses(10);
        assert_eq!(seeds.len(), 2);
        assert!(seeds
            .iter()
            .all(|e| e.success_count == 0 && !e.locally_verified));
        assert_eq!(ledger_candidate_addrs(&lm).len(), 2);
    }

    #[test]
    fn test_migration_is_idempotent_and_batches_past_the_import_cap() {
        let lm = LedgerManager::ephemeral();
        let total = MAX_SEED_LEDGER_ENTRIES * 2 + 3;
        let make = || -> Vec<String> {
            (0..total)
                .map(|i| format!("/ip4/10.2.{}.{}/tcp/9001", i / 200, i % 200 + 1))
                .collect()
        };
        let mut legacy = make();
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &lm);
        assert_eq!(out.found, total);
        assert_eq!(out.imported, total);

        // Second run over the same addresses adds nothing new.
        let mut again = make();
        let out2 = migrate_legacy_bootstrap_nodes(&mut again, &lm);
        assert_eq!(out2.imported, 0);

        // Empty legacy list is a no-op.
        let mut none: Vec<String> = Vec::new();
        assert_eq!(
            migrate_legacy_bootstrap_nodes(&mut none, &lm),
            LegacyMigrationOutcome::default()
        );
    }
}
