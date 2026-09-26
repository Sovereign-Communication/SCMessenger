<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: glm-5.1:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_SECURITY_009_Sled_Compaction_And_Monitoring

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 security
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P0.2 (No bounded retention enforcement) + planfromclaudeforhermes 2 Phase B.3
**Depends on:** P0_BUILD_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P0.2: "StorageManager exists but `enforce_retention()` is never called automatically. sled databases will grow without bound. No compaction, no size limits, no corruption recovery."

`StorageManager` is in `core/src/store/storage_manager.rs`. `enforce_retention()` exists but is dormant.

## Scope (~120 LoC across 3 files)

### Part A: Sled compaction call (LOC: ~30)

In `core/src/store/storage_manager.rs`:

```rust
impl StorageManager {
    pub fn compact_all(&self) -> Result<CompactionReport> {
        let mut report = CompactionReport::default();
        for tree_name in self.trees.keys() {
            let tree = self.trees.get(tree_name).unwrap();
            let before = tree.len();
            tree.flush()?;
            // sled doesn't have explicit compact()  but flush + clear cache forces page merge
            report.report(tree_name, before, tree.len());
        }
        Ok(report)
    }
}

pub struct CompactionReport {
    pub trees_compacted: Vec<(String, usize, usize)>,  // (name, before, after)
    pub bytes_reclaimed: u64,
    pub duration_ms: u64,
}
```

### Part B: Hook into IronCore::stop() (LOC: ~30)

In `core/src/iron_core.rs`:

```rust
pub fn stop(&self) -> Result<()> {
    // ... existing shutdown logic ...
    
    // Compact before shutdown
    let report = self.storage_manager.compact_all()
        .map_err(|e| warn!("Sled compaction failed: {}", e))
        .unwrap_or_default();
    info!("Sled compaction on shutdown: {} trees, {} bytes reclaimed in {}ms",
          report.trees_compacted.len(), report.bytes_reclaimed, report.duration_ms);
    
    *self.running.write() = false;
    Ok(())
}
```

### Part C: Size monitoring with low-disk graceful degradation (LOC: ~60)

In `core/src/store/storage_manager.rs`:

```rust
pub struct StorageHealth {
    pub total_bytes: u64,
    pub disk_free_bytes: u64,
    pub tree_sizes: BTreeMap<String, u64>,
    pub critical: bool,  // true if disk_free < 100MB or total > 10GB
}

impl StorageManager {
    pub fn health(&self) -> StorageHealth {
        let total_bytes: u64 = self.trees.values().map(|t| t.len() as u64 * AVG_ENTRY).sum();
        let disk_free = fs2::available_space_bytes(&self.data_dir).unwrap_or(u64::MAX);
        let critical = disk_free < 100 * 1024 * 1024 || total_bytes > 10 * 1024 * 1024 * 1024;
        StorageHealth { total_bytes, disk_free_bytes: disk_free, tree_sizes: ..., critical }
    }
}
```

Add periodic check in `perform_maintenance()` (every 100th tick). If `critical=true`, call `enforce_retention()` automatically and emit warn log.

## File Targets

- `core/src/store/storage_manager.rs` [EDIT  add compact_all, health, enforce_retention hook]
- `core/src/store/compaction.rs` [NEW if separating concerns; optional]
- `core/src/iron_core.rs` [EDIT  wire compact_all into stop(), health into perform_maintenance()]
- `core/Cargo.toml` [EDIT  verify `fs2` or alternative disk-free crate]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib store
# Create a 1M-entry tree
cargo run -p scmessenger-cli -- store populate --entries 1000000
# Measure before/after
cargo run -p scmessenger-cli -- diagnostics storage
# Shutdown triggers compaction
cargo run -p scmessenger-cli -- daemon stop
# Check logs
grep "Sled compaction" /e/.hermes/logs/*.log
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: compact_all runs without error on empty store, health() returns expected values, critical flag set when disk free < 100MB (mocked), enforce_retention called automatically on critical state
3. Manual: after populate+stop, log shows "Sled compaction: N trees, X bytes reclaimed"
4. Storage health exposed via existing `diagnostics export` JSON
5. Commit: `security: v0.2.1 sled compaction on shutdown + size monitoring`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_BUILD_001]
