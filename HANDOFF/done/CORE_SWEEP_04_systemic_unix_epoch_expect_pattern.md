# TASK: CORE-SWEEP-04  Systemic `SystemTime::now().duration_since(UNIX_EPOCH).expect(...)` pattern across 20 files (informational / low-priority)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Context

Found during a comprehensive gap sweep of `core/src/` (2026-07-04). This is
a **systemic pattern**, not a single bug  flagging it as one consolidated
task rather than 67 separate findings so it's tracked but not overweighted.

A repo-wide grep of `core/src/` for `duration_since(UNIX_EPOCH)` finds 67
occurrences across 20 files (`drift/envelope.rs`, `identity/store.rs`,
`iron_core.rs`, `message/ephemeral.rs`, `relay/bootstrap.rs`,
`relay/client.rs`, `relay/invite.rs`, `relay/peer_exchange.rs`,
`relay/server.rs`, `routing/smart_retry.rs`, `transport/ble/beacon.rs`,
`transport/ble/scanner.rs`, `transport/health.rs`, `transport/internet.rs`,
`transport/mesh_routing.rs`, `transport/nat.rs`, `transport/observation.rs`,
`transport/peer_broadcast.rs`, `transport/relay_health.rs`,
`transport/reputation.rs`, `transport/swarm.rs`), almost all following the
form:

```rust
SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock before UNIX_EPOCH")  // or "Time went backwards"
    .as_secs() // or .as_millis()
```

This only panics if the system clock reads before 1970-01-01, which is not
attacker-controlled and essentially unreachable on any real device/server
(would require a badly misconfigured or actively tampered system clock).
This is a much weaker finding than CORE-SWEEP-01/02 (which are reachable
via ordinary disk/filesystem failure)  it's included for completeness
since the sweep brief asked for `.expect()`/`.unwrap()` calls that could
panic on real (non-programmer-error) input, and a sufficiently broken or
adversarially-set device clock is *technically* real (not compile-time)
input, even though it's an extreme edge case.

## Recommendation

**Low priority. Do not spend a dedicated session on 67 individual
call-site edits.** Options, roughly in order of cost/benefit:

1. **(Recommended) No code change**  add a single repo-wide helper (e.g.
   `crate::util::unix_time_ms() -> u64` / `unix_time_secs() -> u64`) in a
   shared location that does the `unwrap_or(Duration::ZERO)` fallback
   (clock-before-epoch clamps to 0 rather than panicking) once, and
   opportunistically route NEW code through it  do not do a mass find/replace
   across all 67 existing call sites in one PR (that would be a large,
   low-value diff touching `transport/`, `routing/`, `relay/` all at once,
   which is exactly the kind of sweeping cross-cutting change the mandatory
   adversarial review protocol exists to catch, for very little real
   robustness gain).
2. If a future session touches one of these files anyway for an unrelated
   reason, swap that file's calls to the shared helper as a drive-by
   improvement rather than a dedicated pass.
3. Do NOT treat this as blocking for v1.0.0 release readiness  it is
   categorically different from CORE-SWEEP-01/02 above.

## Acceptance Criteria (if ever picked up)

- New shared helper added (suggest `core/src/util.rs` or similar existing
  small-utility module  check for one before creating a new file).
- No behavior change to existing call sites unless a file is independently
  being touched for another task.
- If `transport/`, `routing/`, `relay/`, or `privacy/` files are edited as
  part of adopting the helper, mandatory crypto-security-auditor review
  applies per `.claude/rules/security.md` before merge (most of the 20
  files listed above fall under `transport/`, `routing/`, or `relay/`).

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test --workspace --no-run
```
