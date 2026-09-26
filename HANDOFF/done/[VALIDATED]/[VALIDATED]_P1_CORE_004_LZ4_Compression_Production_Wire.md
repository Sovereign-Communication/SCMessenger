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

# P1_CORE_004_LZ4_Compression_Production_Wire

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1 wire dormant modules
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P2.15 (No bandwidth-adaptive compression) + planfromclaudeforhermes 2 Phase C.5
**Depends on:** P1_CORE_001 (Drift wire)

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P2.15: "No bandwidth-adaptive compression  `drift/compress.rs` exists (LZ4) but is never used in the production send path. Messages are sent uncompressed."

`drift::compress::lz4_encode` is unit-tested but not called from `swarm.rs`.

## Scope (~70 LoC across 2 files)

### Part A: Wire LZ4 into DriftEnvelope send (LOC: ~50)

In `core/src/drift/envelope.rs`:

Add a `compressed: bool` field to `DriftEnvelope`, and helper:
```rust
impl DriftEnvelope {
    pub fn from_message_with_compression(msg: Message, policy: &Policy) -> Result<Self> {
        let bytes = serialize_message(&msg)?;
        let (payload, compressed) = if bytes.len() > 256 {
            let compressed = compress::lz4_encode(&bytes)?;
            (compressed, true)
        } else {
            (bytes, false)
        };
        Ok(DriftEnvelope { payload, compressed, ..Default::default() })
    }
}
```

In `core/src/transport/swarm.rs` `send_message()`:
- Replace `DriftEnvelope::from_message(msg, &policy)` with `DriftEnvelope::from_message_with_compression(msg, &policy)`
- On receive, check `envelope.compressed` and decompress before parsing

### Part B: Add compression metrics (LOC: ~20)

In `core/src/drift/compress.rs`:
- Track `compression_stats()`: total bytes_in, bytes_out, ratio, count_compressed, count_skipped
- Expose via `IronCore::diagnostics()` JSON

## File Targets

- `core/src/drift/envelope.rs` [EDIT  add compressed field, from_message_with_compression]
- `core/src/drift/compress.rs` [EDIT  add compression_stats()]
- `core/src/transport/swarm.rs` [EDIT  use from_message_with_compression, decompress on receive]
- `core/src/iron_core.rs` [EDIT  expose compression_stats in diagnostics]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib drift::compress
cargo test --workspace --no-run

# CLI smoke
cargo run -p scmessenger-cli -- daemon &
sleep 2
cargo run -p scmessenger-cli -- send "a message that is definitely longer than 256 bytes so compression kicks in and saves bandwidth on the wire"
# Check diagnostics
cargo run -p scmessenger-cli -- diagnostics
# Should show compression_stats with non-zero count_compressed
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: small messages (256 bytes) skip compression, large messages compress + decompress roundtrip, compression_stats() returns expected counts
3. `grep "lz4_encode\|from_message_with_compression" core/src/transport/swarm.rs` returns  1 hit
4. `grep "lz4_encode" core/src/drift/compress.rs` is called from envelope.rs
5. Manual: send 1KB+ message, diagnostics show compression ratio < 1.0 (i.e., compressed)
6. Commit: `feat(wire): v0.2.1 LZ4 compression in production send path`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P1_CORE_001] [PARALLEL_WITH: P1_CORE_002, P1_CORE_003, P1_PLATFORM_001]
