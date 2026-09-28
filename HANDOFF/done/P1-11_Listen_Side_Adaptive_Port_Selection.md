# TASK: P1-11  Listen-side adaptive port selection (default-on `MultiPortConfig`, laddered WS, bound-addr export)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] [AUDIT-GATE]
**Phase:** v1.0.0 Phase 1, Stage C (deliverability workstream).
**Design source:** `HANDOFF/plans/P1-10_adaptive_port_selection_design.md` 1.1, 3.1 (read it first).
**Depends on / BLOCKED BY:**
- **P1-04** (transport negotiation root-cause) MUST land first. P1-04 owns the `transport/` hotspot lane per plan 1.4; this ticket queues behind it even though the design (P1-10) is done. DO NOT start editing `swarm.rs` until P1-04 releases the lane.
- P1-01 (swarm.rs test-module import fix) so the compile gate is green.

## Source

`HANDOFF/V1_0_0_EXECUTION_PLAN.md` P1-11 (Stage C). Ground truth verified in P1-10 design note this session ([V-READ], no toolchain in sandbox  re-verify with a real build on the Windows box).

## Problem (exact, verified)

The multi-port ladder engine already exists (`core/src/transport/multiport.rs`: `MultiPortConfig`, `generate_listen_addresses`, `analyze_bind_results`) and `start_swarm_with_config` already has the multi-port listen branch (`core/src/transport/swarm.rs:1876-1901`). But it is **dormant**: both spawn call sites pass `None` for `multiport_config`:
- CLI: `cli/src/main.rs:1395`
- Mobile: `core/src/mobile_bridge.rs:725`

So every node runs single-port. The CLI forces `/ip4/0.0.0.0/tcp/{p2p_port}` with `p2p_port = ws_port + 1`, `ws_port` default 9000 (`cli/src/main.rs:723-728, 1373`)  effective hardcode 9001. Additionally the WebSocket listener is bound TWICE: a **literal** `/ip4/0.0.0.0/tcp/9002/ws` inside `start_swarm_with_config` (`swarm.rs:1938`), and a derived `/ip4/0.0.0.0/tcp/{p2p_port+1}/ws` in `cli/src/main.rs:1406-1408`  colliding on 9002 under defaults.

There is also no accessor that returns the node's **actually-bound** address set. `NewListenAddr` (`swarm.rs:3674-3677`) only logs and emits `SwarmEvent2::ListeningOn`. The advertise slice (P1-12) needs a canonical bound-addr set.

## Root Cause

The ladder was built but never wired on by default, and the WS listener predates it (literal 9002). No "my bound addrs" registry was ever collected from `NewListenAddr` events.

## Scope / What to do

1. **Default-on ladder, both spawn paths.**
   - Change `cli/src/main.rs:1395` and `core/src/mobile_bridge.rs:725` to pass `Some(MultiPortConfig::...)`.
   - Preserve explicit-port intent: if the user passed `--port N` / a non-default `listen_port`, that port becomes the **preferred first** rung, not the only rung. Implement by extending `MultiPortConfig` with `preferred_port: Option<u16>` (Default `None`) prepended in `generate_listen_addresses` before `COMMON_PORTS`. (Additive public-API change  flagged for audit in P1-10 4 item 4.)
2. **Kill the literal WS hardcode.** Remove/replace `swarm.rs:1938` `/ip4/0.0.0.0/tcp/9002/ws`. WS must bind via the ladder (add a `/ws` variant per bound TCP port, or bind `/ip4/0.0.0.0/tcp/0/ws` ephemeral). Ensure the `cli/src/main.rs:1406-1408` WS bind and the in-swarm WS bind no longer double-bind or collide.
3. **Export bound-addr set.** Collect addresses from `NewListenAddr` into a shared set inside the swarm task and add a `SwarmHandle` accessor (e.g. `get_bound_addresses()`), mirroring the existing `GetExternalAddresses` command pattern (`swarm.rs:4099`). This is the input P1-12 consumes.

## Blast Radius

`core/src/transport/swarm.rs` (hotspot), `core/src/transport/multiport.rs`, `cli/src/main.rs`, `core/src/mobile_bridge.rs`. Touches the live listen path on every node  behavioral. Firewall/privilege edge cases: port 443/80 bind will fail without privilege on Unix (already detected by `requires_elevated_privileges`); failure is non-fatal (ladder continues), but confirm the CLI log surfaces it.

## Adversarial Review Requirement

**[AUDIT-GATE].** Touches `core/src/transport/`. Mandatory `crypto-security-auditor` pass before done, `release-gatekeeper` before merge, per `.claude/rules/security.md` and plan 1.1. Not test-only  no skip.

## Files to Touch

- `core/src/transport/multiport.rs`  add `preferred_port: Option<u16>`, prepend in `generate_listen_addresses`.
- `core/src/transport/swarm.rs`  enable ladder path already present; remove literal WS 9002 (1938); collect `NewListenAddr` into a bound-addr set; add `SwarmHandle` accessor.
- `cli/src/main.rs`  pass `Some(MultiPortConfig)` at 1395; reconcile WS bind (1406-1408); map `--port`/`listen_port` to `preferred_port`.
- `core/src/mobile_bridge.rs`  pass `Some(MultiPortConfig)` at 725.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib multiport
cargo test --workspace --no-run
```
Manual (Windows box, device optional): start CLI with 9001 blocked by firewall; confirm it still binds and the startup log reports 443/80/ephemeral; confirm no double-bind error on 9002.

## Acceptance Tests (per-slice)

1. `MultiPortConfig { preferred_port: Some(9001), .. }`  `generate_listen_addresses` yields 9001 FIRST, then 443/80/8080/9090, then 0. (unit test)
2. With the default ladder on, a node whose 9001 is firewalled still reports at least one successful bind (ephemeral) and does not exit. (integration/manual)
3. `SwarmHandle::get_bound_addresses()` returns the real bound set (non-empty, contains the ephemeral addr). (unit/integration)
4. No `Address already in use` for 9002 at startup on default config (single WS listener). (manual/log)

## Do NOT

- Do NOT delete the single-port else-branch (`swarm.rs:1902-1910`)  it is still the correct path when a caller passes an explicit `listen_addr` with no ladder.
- Do NOT change `MultiPortConfig` defaults for `enable_common_ports`/`enable_random_port` (already correct).
- Do NOT start before P1-04 releases the `transport/` lane (plan 1.4).
- Do NOT skip the audit gate  this is production transport code.
