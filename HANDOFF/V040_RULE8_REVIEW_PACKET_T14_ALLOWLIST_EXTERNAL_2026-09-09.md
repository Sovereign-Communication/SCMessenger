# V040 rule-8 adversarial review packet — gated transport diffs 0a33c009 + 74253491

Written by the CEO seat, 2026-09-09T02:00Z, at operator direction (next-run
package gate E1, RCA X1). This is the dispatch packet; the reviewer's verdict
must be filed under `HANDOFF/review/` referencing both commit SHAs.

## Gate

AGENTS.md rule 8: changes under `core/src/{crypto,transport,routing,privacy}/`
are not done until an adversarial security review is on file by a reviewer who
did not author the change. The authoring session was the CTO seat (Freebuff
`/cto`, 2026-09-08 evening) and cannot self-approve. This packet was prepared
by the CEO seat, which also did not author the code, but preparation is not
review — an independent reviewer must produce the verdict.

## Scope — exactly two commits

1. `0a33c009` "fix(transport): gate advertised addresses on actually-bound
   listen ports; unify BLE stack ownership on Android" (8 files, +745/-760)
   - Gated portions: `core/src/transport/observation.rs` (listen-port
     allowlist: observations are accepted only for ports this node currently
     binds; empty allowlist fails closed), `core/src/transport/swarm.rs`
     (+5: observer wiring at the `NewListenAddr` event arm), plus non-gated
     companions `core/src/routing/local.rs`, `core/src/routing/
     optimized_engine.rs`, `core/src/iron_core.rs`, `cli/Cargo.toml`,
     `android/` (TransportManager restructure, MeshRepository wiring).
2. `74253491` "fix(transport): T14 configured-external-address primacy; BLE
   scanner duty-cycle fix" (10 files, +693/-2)
   - Gated portions: `core/src/transport/observation.rs` (+52:
     `configured_external: Option<SocketAddr>` + `set_configured_external()`;
     `recalculate_consensus()` pins the configured address first and
     deduplicates), `core/src/transport/swarm.rs` (+28:
     `SwarmCommand::SetConfiguredExternalAddress` handled in native loop with
     `swarm.add_external_address` registration and in the wasm loop;
     `SwarmHandle::set_configured_external_address()`), plus non-gated
     companions `cli/src/config.rs` (external_addr knob, fail-closed
     validation), `cli/src/main.rs` (wired in `cmd_start` and `cmd_relay`),
     `core/tests/test_configured_external_address.rs` (new), `android/`
     BleScanner duty-cycle fix + test.

Reviewer method: read the two diffs directly
(`git show 0a33c009`, `git show 74253491`), not this packet's summary.

## Threat model and review focus

The change surface is: which addresses a node ADVERTISES to peers. A wrong
advertisement is an availability and privacy issue (peers dial unusable or
wrong endpoints; custody delivery fails). The T14 defect class was: an
ephemeral/NAT-observed port winning observation consensus, advertised
fleet-wide.

Mandatory review questions (from the package + author's own flagged focus,
independently re-derived):

1. Primacy bypass: the configured external address intentionally MAY differ
   from any bound listen port (that is the port-forwarded-NAT case). Confirm
   this cannot be abused to make a node advertise an arbitrary third-party
   address (spam/amplification/attribution concerns) — who can set it, and is
   it operator-only config?
2. Retraction semantics: clearing the knob (`set_configured_external(None)`)
   restores pure observation consensus, but `swarm.add_external_address` in
   the native loop is NOT retracted — the address stays in libp2p's registry.
   Assess the staleness window and whether any code path keeps advertising a
   cleared configured address.
3. Allowlist interaction in `0a33c009`: observations for non-bound ports are
   rejected; confirm the configured address is not accidentally filtered by
   the allowlist it must coexist with, and that the empty-allowlist fail-closed
   path is unreachable in the configured case.
4. Unbounded state: both diffs add bounded fields only (`Option<SocketAddr>`,
   one Vec dedup) — confirm no new unbounded collection or per-observation
   growth.
5. Dual-loop consistency: the wasm loop records observer state but cannot
   register `add_external_address` — confirm diagnostics-only parity is the
   intended and safe behavior (matches existing wasm design per the author's
   checkpoint).
6. Regression coverage: `configured_external_wins_over_consensus` (observer
   unit), `test_configured_external_address.rs` (swarm-level through the real
   command channel), `dutyCycleStop_clearsIsScanningFlag` (Android JVM).
   Assess: is the "ask where the hole went" question covered — does any test
   pin that a configured address SURVIVES a peer-observation flood (5x wrong
   observations in the unit test) at the swarm level, not just the observer
   level?

## Verification status at packet time

- Targeted gates green on the Windows host under build lock (logs preserved):
  observer 6/6 (`tmp/cto/T14_GATE_1_OBSERVER_CONFIG_20260908T230550Z.log`),
  swarm regression 1/1 (T14_GATE_2), cli config 4/4 (T14_GATE_3), wasm check
  clean (T14_GATE_4), BleScannerTest PASSED
  (`tmp/cto/BLE01_GATE_FINAL_20260908T224500Z.log`).
- Live proof: Windows node (PID 23508, binary 829efe2c built from HEAD
  ba474a7a-family) reports `external_addrs == ["147.81.41.188:9001", ...]`
  with the configured address pinned first; go-live 10/10 checks PASS
  (`HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T001500Z_T14_GOLIVE.md`).
- Full core regression suite: first rerun (disk now healthy) found exactly one
  failure — `test_consensus_with_multiple_observations` in
  `test_address_observation.rs` recorded observations without arming the
  listen-port allowlist, which 0a33c009 made fail-closed by design. Test
  contract updated in `6c007b47`. Full rerun after the fix: 41 binaries,
  1653 passed / 0 failed / 24 ignored, zero compile errors
  (`tmp/cto/E2_REGRESSION_RERUN_20260909T0210Z.log`, under build_lock). The
  reviewer should read that log's `test result:` lines; a suite failure inside
  the touched modules remains a REJECT condition for the merge verdict.

## Related review history (for context, not a substitute)

`HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md` and
`V040_T14_EPHEMERAL_CONFIRM_APPROVE_HARNESS_2026-09-03.md` reviewed the
EARLIER T14 approach (pre-0a33c009 code) and are stale for these diffs.
This packet supersedes them for gate purposes; both commits are newer trees.

## Verdict requirements

File `HANDOFF/review/V040_RULE8_T14_ALLOWLIST_EXTERNAL_<REVIEWER>_<DATE>.md`
with: reviewer identity/lane (must differ from the authoring CTO seat), each
focus question answered with line-cited evidence, explicit APPROVE or REJECT
for EACH commit (they may split), and any MAJOR findings with suggested
dispositions. A bare "LGTM" does not satisfy the gate. On APPROVE, E1 closes
and the merge path per the ship plan reopens; on REJECT, fixes route back
through the CTO.
