# TASK: D-05 Unwrap/Panic Hardening (v1.0.0 Robustness)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: DISPATCH-READY (parallel x2: FFI/startup + crypto/storage)
Model: Qwen CODER tier
Scope: Bounded hardening (~60 sites, Result/logged-default pattern)

## Objective

Replace ~60 `unwrap()` / `panic!()` calls with proper error handling (Result types + logged defaults). Priority zones: FFI boundary, startup path, crypto, storage.

## Strategy

**Zone 1 (FFI boundary + Startup):** mobile_bridge.rs + iron_core.rs init paths
- FFI boundary violations = return error to caller (Result)
- Startup errors = log + default (non-critical) or bail (critical path)
- Use anyhow::Context for error wrapping

**Zone 2 (Crypto + Storage):** crypto module + store module  
- Critical invariant failures = keep panic, add `// SAFETY:` comment explaining why
- Non-critical decoding errors = Result + log
- Cryptographic operations with fallback logic = Result + default

## Implementation Pattern

```rust
// Before:
let key = something().unwrap(); // UNSAFE

// After (non-critical):
let key = match something() {
    Ok(k) => k,
    Err(e) => {
        tracing::warn!("failed to load key: {}", e);
        DEFAULT_KEY // or return Err
    }
};

// After (critical invariant):
let key = something().expect("SAFETY: key must exist at this point (verified by load_keys)")
// OR if panic truly required:
// SAFETY: This path is reached only after initialization_guard check (line 123)
panic!("key missing after guard");
```

## Scope

Identify all `unwrap()` / `panic!()` via grep:
```bash
grep -rn "unwrap()\|panic!" core/src/ --include="*.rs" | grep -v test | grep -v "// SAFETY"
```

Categorize by zone:
1. **mobile_bridge.rs** + iron_core.rs init (~20 sites) → Result/logged-default
2. **crypto/** (~15 sites) → Result or documented panic
3. **store/** (~10 sites) → Result or documented panic  
4. **Other** (~15 sites) → Context-dependent (read each)

## Success Criteria

- Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- `cargo test --workspace --no-run` green (compile gate)
- All existing tests pass (no functional regressions)
- Critical panics documented with `// SAFETY:` comment
- Non-critical paths return Result or logged-default

## Files to Modify

- `core/src/mobile_bridge.rs` (primary FFI boundary)
- `core/src/iron_core.rs` (startup/initialization)
- `core/src/crypto/*.rs` (as needed)
- `core/src/store/*.rs` (as needed)

## Estimate
600 LOC, ~90 min total (dispatch as 2x parallel if large)

## Parallel Sharding (Optional)
If diff becomes large (>300 lines):
- **Worker 1:** FFI boundary + startup (mobile_bridge.rs + iron_core.rs init)
- **Worker 2:** Crypto + storage (crypto/ + store/ modules)

Each worker operates independently on their zone; diffs merge cleanly (no overlap).

## Review Gate
None (hardening task). Verify compile + all tests pass.

## Execution
Dispatch to Qwen CODER with `--mode diff --apply --verify "cargo check --workspace"`.

Single dispatch or split if needed.

## Handoff
Move this file to `HANDOFF/done/` ON COMPLETION via `mv` command.

## RE-SCOPE FINDING (2026-07-19)

Two bulk qwen dispatches attempted (Zone 1: mobile_bridge.rs+iron_core.rs
combined; Zone 2: crypto/+store/ files combined) both failed SAFELY at the
input stage: qwen-max's 30720-token input ceiling was exceeded when bundling
multiple large files, the retry model's diff didn't apply against real line
numbers, and delegate_task.py's vacuous-success guard correctly refused to
claim success with zero changes applied. No corruption occurred.

Real grep count is far above the ticket's "~60 sites" estimate:
- crypto/: 80 real sites (backup.rs 24, encrypt.rs 26, negotiation.rs 5,
  ratchet.rs 15, session_manager.rs 10) -- not ~15.
- store/: 266 sites across 9 files (blocked.rs 89(!), contacts.rs 33,
  outbox.rs 41, relay_custody.rs 56, history.rs 18, logs.rs 14, inbox.rs 8,
  storage.rs 6, dedup.rs 1) -- not ~10.
- mobile_bridge.rs: 42, iron_core.rs: 9 -- roughly matches the ~20 estimate
  for Zone 1 combined (51 total).

Given crypto/ and store/ sites require individual judgment (is this a
genuine invariant panic that should stay documented, or a real error path
being silently masked into a swallowed Result?) rather than a mechanical
pattern substitution, and given the crypto module's mandatory adversarial
review gate, bulk-dispatching Zone 2 to qwen is NOT safe as originally
scoped -- risk of silently changing crypto failure semantics without per-site
review. Deferring Zone 2 pending a smaller, file-by-file, judgment-reviewed
approach (one file per dispatch, human/careful-agent review of each
before/after diff, not a bulk "replace all unwrap()" pass).

Zone 1 (mobile_bridge.rs, iron_core.rs) is lower-stakes -- neither file is in
security.md's mandatory-audit module list (crypto/transport/routing/privacy)
-- proceeding with per-file (not combined) qwen dispatch for Zone 1 only.

## FURTHER FINDING (2026-07-19, continued)

Third attempt: mobile_bridge.rs dispatched ALONE (single file, 4379 lines,
no combined-file bundling) still exceeds qwen-max's 30720-token input limit
on the primary model call; the automatic fallback model
(qwen3-next-80b-a3b-instruct) produced a diff, but it did not apply cleanly
against the real file (git apply rejected at line 1762 -- line-number drift,
a known failure mode when an LLM reconstructs a diff against a large file
from memory rather than exact context). All three attempts failed SAFELY
(delegate_task.py's vacuous-success guard correctly refused to claim success
with zero applied changes) -- no corruption occurred at any point.

Conclusion: whole-file dispatch (even single-file) does not work for this
task given mobile_bridge.rs/iron_core.rs's size. This needs either (a) a
surgical per-site approach -- grep each unwrap()/panic! site, dispatch only
a small window of surrounding context per site or per small cluster of
sites, not the whole file, or (b) direct hands-on implementation rather than
LLM bulk dispatch. Not attempted further this session given the real scope
(51 sites in Zone 1 alone, ~400+ across the full original ticket scope) would
require dozens of individually-scoped dispatches or a substantial direct
implementation effort -- deferring to a dedicated future session rather than
rushing a partial, inconsistent pass under time pressure. This ticket is
NOT blocking for Josh-sim/Farm-sim core P2P functionality (it is a
robustness/hardening item, not a functional correctness gap) so deprioritizing
relative to items that directly gate the live connectivity test.


## COMPLETION NOTE (2026-07-21, Kimi-K3 orchestrator)
The v0.4.0-scope slice was completed and documented in HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md. Compile gates verified on the authoritative Windows host: `cargo clippy --workspace --all-features -- -D warnings` exit 0; `cargo test --workspace --no-run` exit 0. Broader v1.0.0-scale sweep remains tracked by HANDOFF/todo/D-05_unwrap_panic_hardening_v1.md (out of v0.4.0 scope).
