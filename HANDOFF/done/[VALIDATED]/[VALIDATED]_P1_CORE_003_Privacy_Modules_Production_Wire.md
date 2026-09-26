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
# BUDGET: 2400
# token_budget: 24000

# P1_CORE_003_Privacy_Modules_Production_Wire

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 2400s (LARGE tier)
**Phase:** v0.2.1 P1 wire dormant modules
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md Module Status Matrix (Privacy:  Dormant) + planfromclaudeforhermes 2 Phase C.3
**Depends on:** P1_CORE_001 (Drift wire provides the message envelope to wrap)

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md`: "Privacy modules dormant  Onion routing (`privacy/onion.rs`), cover traffic (`privacy/cover.rs`), message padding (`privacy/padding.rs`), and timing obfuscation (`privacy/timing.rs`) all exist with tests but are NEVER called from any production code path."

Privacy module: 6 files totaling ~2,350 LoC (circuit 529, cover 527, onion 508, padding 320, timing 397, mod 69). All unit-tested, none called.

`iron_core.rs` already has `prepare_onion_message` (L1537), `peel_onion_layer` (L1563) per MASTER_SPRINT_PLAN.md sprint 1  these are wired in `iron_core.rs` but not called from the swarm send path.

## Scope (~250 LoC across 5 files)

### Part A: Wire onion routing in send path (LOC: ~80)

In `core/src/transport/swarm.rs` `send_message()`:
- Check `privacy_config.onion_enabled` (already exposed via `get_privacy_config`/`set_privacy_config` at iron_core.rs:1271/1164)
- If true, wrap envelope via `iron_core.prepare_onion_message(envelope, circuit_hops=3)` before sending
- On relay side, `peel_onion_layer` called automatically when forwarding (add to relay forwarding path in `core/src/relay/server.rs`)

### Part B: Wire cover traffic (LOC: ~60)

In `core/src/transport/swarm.rs` background loop:
- If `privacy_config.cover_traffic_enabled`, periodically send decoy messages to random peers
- Rate: 1 cover msg per 30s when idle, scaled by `cover_traffic_intensity` (0-100)
- Cover messages are indistinguishable from real messages to outside observers

### Part C: Wire padding (LOC: ~50)

In `core/src/transport/swarm.rs` `send_message()`:
- If `privacy_config.padding_enabled`, pad envelope to next power-of-2 size using `privacy::padding::pad_to_bucket(envelope, bucket_sizes)`
- On receive, strip padding via `padding::strip_padding(envelope)`

### Part D: Wire timing obfuscation (LOC: ~60)

In `core/src/transport/swarm.rs` `send_message()`:
- If `privacy_config.timing_obfuscation_enabled`, add random delay before send
- Use `privacy::timing::jitter_send(delay_range_ms, urgency=message.priority)`
- High-priority messages have tighter delay bounds

## File Targets

- `core/src/transport/swarm.rs` [EDIT  primary wire, ~150 LoC across 4 hooks]
- `core/src/iron_core.rs` [EDIT  verify prepare_onion_message is called; add cover traffic scheduler if not]
- `core/src/privacy/onion.rs` [EDIT  verify API; no new code]
- `core/src/privacy/cover.rs` [EDIT  verify API; no new code]
- `core/src/privacy/padding.rs` [EDIT  verify API; no new code]
- `core/src/privacy/timing.rs` [EDIT  verify API; no new code]
- `core/src/relay/server.rs` [EDIT  peel_onion_layer call in forwarding path]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib privacy
cargo test -p scmessenger-core --lib transport
cargo test --workspace --no-run

# Integration
cargo test -p scmessenger-core --test integration_all_phases -- --nocapture privacy

# CLI smoke with privacy enabled
cargo run -p scmessenger-cli -- config set privacy.onion_enabled true
cargo run -p scmessenger-cli -- config set privacy.cover_traffic_enabled true
cargo run -p scmessenger-cli -- daemon &
sleep 2
cargo run -p scmessenger-cli -- send "private test"
# Should see "onion_wrapped" and "cover_traffic_sent" in log
grep "onion_wrapped\|cover_traffic\|padding_applied\|timing_jitter" /e/.hermes/logs/daemon-*.log
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: onion wrap+peel roundtrip, cover traffic rate matches config, padding bucket selection, timing jitter bounds per priority
3. `grep "prepare_onion\|peel_onion" core/src/transport/swarm.rs` returns  1 hit each
4. `grep "cover::\|cover_traffic" core/src/transport/swarm.rs` returns  1 hit
5. `grep "padding::\|pad_to_bucket" core/src/transport/swarm.rs` returns  1 hit
6. `grep "timing::\|jitter_send" core/src/transport/swarm.rs` returns  1 hit
7. Privacy config toggles work via `scm config set privacy.<flag>`
8. Commit: `feat(wire): v0.2.1 Privacy modules  onion, cover traffic, padding, timing in production`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P1_CORE_001] [PARALLEL_WITH: P1_CORE_002, P1_CORE_004, P1_PLATFORM_001]
