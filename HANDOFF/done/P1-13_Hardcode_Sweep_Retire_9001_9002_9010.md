# TASK: P1-13  Hardcode sweep (retire 9001/9002/9010 literals)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Tier:** [HAIKU]
**Phase:** v1.0.0 Phase 1, Stage C (deliverability workstream).
**Design source:** `HANDOFF/plans/P1-10_adaptive_port_selection_design.md` 3.3.
**Depends on / BLOCKED BY:**
- **P1-11 and P1-12** must land first  this ticket sweeps the hardcodes that the negotiated/laddered-port machinery from P1-11/12 replaces (plan 2.5). Sweeping before the machinery exists would break dialing.
- Not itself [AUDIT-GATE], but if the sweep ends up editing production logic in `transport/` (beyond mechanical literal swaps), escalate to the P1-11/12 audit owner rather than editing silently.

## Source

`HANDOFF/V1_0_0_EXECUTION_PLAN.md` P1-13 (Stage C). Literals located by grep this session ([V-READ]).

## Problem (exact, verified)

After P1-11/12 make listen/advertise/dial adaptive, residual hardcoded port literals remain and must be retired or repointed:
- `core/src/mobile_bridge.rs:1398`  WiFi Direct client dials `/ip4/{group_owner_ip}/tcp/9001` (fixed by P1-12 to negotiated/laddered port; this ticket confirms the literal is gone).
- `core/src/relay/client.rs:52,65`  `quic_port` default `9002`.
- `cli/src/cli.rs:189` and `cli/src/main.rs:184`  `Commands::Relay` clap default literal `/ip4/0.0.0.0/tcp/9001` (decide: keep as documented preferred-first default, or move to ladder).
- Any remaining `9001|9002|9010` occurrences outside tests/docs surfaced by the repo-wide grep below.
- `docs/` references to the old fixed ports.

Note: `cli/src/ledger.rs` and `cli/src/bootstrap.rs` contain `9001` only in **test fixtures / example strings**  leave those; they are not runtime hardcodes.

## Root Cause

Legacy fixed-port assumptions predating the ladder. P1-11/12 remove the need; this is the cleanup pass.

## Scope / What to do

1. Repo-wide grep for `9001|9002|9010` outside `**/tests/**`, `*_test.rs`, `#[cfg(test)]` blocks, and `docs/`. Triage each: runtime hardcode  repoint to negotiated/laddered/ephemeral; test fixture  leave.
2. `relay/client.rs` `quic_port: 9002` default  derive from config or ladder (coordinate with P1-12 dial ladder; if the relay QUIC port is genuinely a well-known default, keep but document  record the decision).
3. Confirm `mobile_bridge.rs:1398` no longer contains the `tcp/9001` literal (P1-12 owns the fix; this ticket verifies).
4. Update `docs/` references to describe adaptive ports, not fixed 9001/9002.

## Blast Radius

Mostly mechanical literal swaps. `relay/client.rs` and `mobile_bridge.rs` are runtime; the rest are defaults/docs. Low risk IF P1-11/12 landed first.

## Adversarial Review Requirement

Not [AUDIT-GATE] by default (mechanical). BUT: `mobile_bridge.rs` and `relay/client.rs` are in/adjacent to `transport/`-class code  if a change is more than a literal swap, route it through the P1-11/12 audit owner. Record the skip decision explicitly if the sweep is purely mechanical.

## Files to Touch

- `core/src/relay/client.rs`  `quic_port` default (52, 65).
- `core/src/mobile_bridge.rs`  verify 1398 literal removed (P1-12 does the fix).
- `cli/src/cli.rs` (189) + `cli/src/main.rs` (184)  Relay clap default decision.
- `docs/**`  port references.
- Anything else the grep surfaces (triage runtime vs fixture).

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
# Grep must be clean of runtime hardcodes (fixtures/docs excluded intentionally):
grep -rnE '9001|9002|9010' core/src cli/src wasm/src \
  --include='*.rs' | grep -viE 'test|example|fixture' || echo "grep clean"
cargo build --workspace
```

## Acceptance Tests

1. `grep -rnE '9001|9002|9010'` over `core/src` + `cli/src` (excluding tests/fixtures/docs) returns no runtime port hardcodes. (command)
2. `cargo build --workspace` green. (command)
3. WiFi Direct client dial path contains no `tcp/9001` literal. (grep + code read)
4. `docs/` no longer promises fixed ports 9001/9002 as the connectivity contract. (docs-sync)

## Do NOT

- Do NOT touch `cli/src/ledger.rs` / `cli/src/bootstrap.rs` test-fixture 9001 strings  they are not runtime.
- Do NOT start before P1-11/12 land the negotiated-port machinery  sweeping first breaks dialing.
- Do NOT silently rewrite production dial/listen logic here  that belongs to P1-11/12 under the audit gate.
