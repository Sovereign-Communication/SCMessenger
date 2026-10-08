// Bootstrap helpers (#469)
//
// There are NO static bootstrap seeds: no environment variable, no build-time
// option, no compiled-in or config-supplied address list. The only seed
// sources are an invite (`SCI1:` seed ledger) and live discovery (LAN/BLE).
// Dial candidates are always derived from the peer ledger.
//
// The single remaining legacy read is the one-time migration of a pre-#469
// config's `bootstrap_nodes` key into the ledger as UNPROVEN seed entries, so
// existing deployments keep their connectivity. Addresses the ledger refuses
// are logged and preserved in `legacy_bootstrap_nodes_rejected`; only then is
// the original key dropped from the config file.

use crate::ledger;
use scmessenger_core::store::{
    LedgerManager, SeedLedgerEntry, MAX_LEDGER_ENTRIES, MAX_SEED_LEDGER_ENTRIES,
};
use scmessenger_core::{TOPIC_LOBBY, TOPIC_MESH};

/// Upper bound on ledger-derived dial candidates handed to the swarm at boot.
const LEDGER_CANDIDATE_LIMIT: u32 = 64;

/// Hard cap on legacy entries migrated in one pass: the ledger's own retention
/// limit. Anything beyond it is recorded as rejected, never silently dropped.
pub const LEGACY_MIGRATION_MAX_ENTRIES: usize = MAX_LEDGER_ENTRIES;

/// Outcome of the one-time legacy config migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyMigrationOutcome {
    /// Legacy addresses found in the config.
    pub found: usize,
    /// Addresses newly added to the ledger as unproven seed entries.
    pub imported: usize,
    /// Addresses refused by the ledger (or over the migration cap) and
    /// recorded in the rejected list instead of being imported.
    pub rejected: usize,
}

/// Record a rejected legacy address (once) and log why.
fn record_rejected(rejected: &mut Vec<String>, addr: &str, reason: &str) {
    tracing::warn!(
        "[MIGRATION] legacy bootstrap address rejected: {} ({})",
        addr,
        reason
    );
    if !rejected.iter().any(|r| r == addr) {
        rejected.push(addr.to_string());
    }
}

/// Import legacy config-supplied bootstrap addresses into the ledger as
/// unproven seeds, draining `legacy` in the process.
///
/// Seed import enforces the ledger's own safety filters (non-routable and DNS
/// forms are rejected, peer-id components are stripped, duplicates collapse),
/// so a hostile or stale config cannot do more than an invite could. Entries
/// are fed in batches of `MAX_SEED_LEDGER_ENTRIES` because the import caps each
/// call, and at most `LEGACY_MIGRATION_MAX_ENTRIES` are considered in total.
///
/// Nothing is lost: every address that is neither imported nor already known
/// (a duplicate) is appended to `rejected` with a logged reason, so the caller
/// can persist both lists and only then drop the original key.
pub fn migrate_legacy_bootstrap_nodes(
    legacy: &mut Vec<String>,
    rejected: &mut Vec<String>,
    ledger_manager: &LedgerManager,
) -> LegacyMigrationOutcome {
    let addrs = std::mem::take(legacy);
    let found = addrs.len();
    let mut imported = 0usize;
    let mut newly_rejected = 0usize;

    let mut candidates: Vec<String> = Vec::with_capacity(found.min(LEGACY_MIGRATION_MAX_ENTRIES));
    for addr in &addrs {
        let trimmed = addr.trim();
        if trimmed.is_empty() {
            record_rejected(rejected, addr, "empty address");
            newly_rejected += 1;
        } else if candidates.len() >= LEGACY_MIGRATION_MAX_ENTRIES {
            record_rejected(
                rejected,
                trimmed,
                "legacy migration cap reached (ledger retention limit)",
            );
            newly_rejected += 1;
        } else {
            candidates.push(trimmed.to_string());
        }
    }

    for batch in candidates.chunks(MAX_SEED_LEDGER_ENTRIES.max(1)) {
        let entries: Vec<SeedLedgerEntry> = batch
            .iter()
            .map(|a| SeedLedgerEntry {
                multiaddr: a.clone(),
            })
            .collect();
        imported += ledger_manager.import_seed_entries(entries) as usize;
        // Known afterwards means imported or an existing duplicate; unknown
        // means the ledger refused it.
        for addr in batch {
            if ledger_manager.entry_for_multiaddr(addr).is_none() {
                record_rejected(
                    rejected,
                    addr,
                    "refused by ledger: non-routable, DNS form, or no transport component",
                );
                newly_rejected += 1;
            }
        }
    }
    LegacyMigrationOutcome {
        found,
        imported,
        rejected: newly_rejected,
    }
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
        let mut rej = Vec::new();
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &mut rej, &lm);
        assert_eq!(out.found, 4);
        assert_eq!(out.imported, 2);
        // The DNS form is not lost: it is recorded as rejected.
        assert_eq!(out.rejected, 1);
        assert_eq!(rej, vec!["/dns4/relay.example/tcp/9001".to_string()]);
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
        let mut rej = Vec::new();
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &mut rej, &lm);
        assert_eq!(out.found, total);
        assert_eq!(out.imported, total);

        // Second run over the same addresses adds nothing new.
        let mut again = make();
        let out2 = migrate_legacy_bootstrap_nodes(&mut again, &mut rej, &lm);
        assert_eq!(out2.imported, 0);

        // Empty legacy list is a no-op.
        let mut none: Vec<String> = Vec::new();
        assert_eq!(
            migrate_legacy_bootstrap_nodes(&mut none, &mut rej, &lm),
            LegacyMigrationOutcome::default()
        );
    }

    #[test]
    fn test_migration_caps_total_and_records_overflow_as_rejected() {
        let lm = LedgerManager::ephemeral();
        let extra = 5usize;
        let total = LEGACY_MIGRATION_MAX_ENTRIES + extra;
        let mut legacy: Vec<String> = (0..total)
            .map(|i| format!("/ip4/10.3.{}.{}/tcp/9001", i / 200, i % 200 + 1))
            .collect();
        let mut rej = Vec::new();
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &mut rej, &lm);
        assert_eq!(out.found, total);
        assert_eq!(out.imported, LEGACY_MIGRATION_MAX_ENTRIES);
        assert_eq!(out.rejected, extra);
        assert_eq!(rej.len(), extra);
        assert!(legacy.is_empty());
    }

    #[test]
    fn test_migration_records_empty_and_duplicate_rejections_once() {
        let lm = LedgerManager::ephemeral();
        let mut legacy = vec![
            "   ".to_string(),
            "/dns4/a.example/tcp/1".to_string(),
            "/dns4/a.example/tcp/1".to_string(),
        ];
        let mut rej = Vec::new();
        let out = migrate_legacy_bootstrap_nodes(&mut legacy, &mut rej, &lm);
        assert_eq!(out.imported, 0);
        assert_eq!(out.rejected, 3);
        // Deduplicated in the persisted list.
        assert_eq!(rej.len(), 2);
    }
}
