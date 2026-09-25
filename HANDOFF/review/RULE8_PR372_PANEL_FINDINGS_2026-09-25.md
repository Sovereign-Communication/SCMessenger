# Rule-8 panel findings -- PR #372 (D9 vendored graceful degrade + CONN-CAP blockers)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-09-25
Reviewed head (round 2): `fix/361-review-blockers` @ `ec5f0135`
Reviewed head (round 1): `fix/361-review-blockers` @ `08c3482f`
Base for the transport delta: `a76d7d66` (3 files, +757/-43)

## STATUS: NOT A RULE-8 CLEARANCE. The gate remains OPEN.

This records what an adversarial panel found. It does not substitute for a
verdict from a reviewer who did not author D1/D9
(`docs/rules/SECURITY_PROTOCOL.md`, "Reviewer independence"; panels are
explicitly not a substitute). The author of the reviewed work also assembled
this pack, which is disclosed here rather than hidden.

## Method and provenance

- Panel: Harness `verify` (fusion_lite), 3/3 seats, 0 failures both rounds.
  Seats: `deepseek/deepseek-v4-flash`, `inclusionai/ling-3.0-flash`,
  `deepseek/deepseek-v4.1-flash`; judge `z-ai/glm-5.3-flash`.
- Cost: round 1 $0.002797, round 2 $0.005334
  (free-tier OpenRouter pool, `use_free: true`; no paid route used).
- Round 1 first attempt failed (all seats reasoning-only); re-run with
  `--reasoning-effort off`, which is the harness's own documented rotation
  trigger for that failure class.
- Vendor provenance proven byte-exact: pristine `libp2p-swarm 0.48.0` fetched
  from crates.io, SHA-256 `57ccbe1baeaef036ffde4b265871e11c64d29464036bba635378f356bcdca854`,
  matching the crates.io sparse-index checksum. The vendored tree at `08c3482f`
  differs in exactly 12 files / 456 diff lines
  (`tmp/rule8-pr372/pack/d9-vendor-delta.patch`).

## Round 1 verdict (head `08c3482f`): BLOCK

Agreement `medium`, confidence 0.75.

| Panel must-fix | Verified against the code | Disposition |
|---|---|---|
| Every drop site logs at `debug!` | **Partly false.** 6 of 8 were `debug!`; `on_behaviour_event` and `behaviour/either.rs` were already `warn!`. The concern about `FullyNegotiatedInbound::transpose` returning `None` silently was valid. | Fixed at `ec5f0135`: all 8 now `warn!`. |
| No in-tree regression test for the desync path | **False.** `handler/either.rs` has `#[cfg(test)] mod tests` under the DEFAULT feature set with `transpose_desync_returns_none_instead_of_panicking`, `transpose_matched_sides_still_route`, `on_behaviour_event_side_desync_must_not_panic`. The panel could not see the test module and said so. | No change needed. |
| `ProtocolsChange::Added` panic inconsistent with the drops | **True but not a production inconsistency:** it is inside `#[cfg(all(test, feature = "upstream-tests"))]` and does not compile in this workspace. | Classification documented at `ec5f0135`. |

## Change made in response (`ec5f0135`)

Six `tracing::debug!` -> `tracing::warn!` (the only executable change; verified
by diffing every changed line: the rest are comments), a "VENDORED PATCH POLICY"
block stating the fatal-vs-drop classification once, and an inline note on the
test-only panic. `rustfmt --edition 2024 --check` clean; `rules_check.py` exit 0.

## Round 2 verdict (head `ec5f0135`)

Agreement `medium`, confidence 0.62, defer
false.

> No critical or security-regression blocker; the panel converges that Q2's Option conversion is sound, Q4's ProtocolsChange::Added panic is cfg-gated/non-operational, and Q5's in-tree regression tests exist (residual risk is coverage shape, not absence). Merge only after closing two medium findings: (1) Q1 — drop-site logging must be at warn/error with messages an operator can triage (remote-induced desync vs internal mis-pairing), and (2) Q3 — the patch must demonstrate per-site acquire/release balance-neutrality at each of the eight converted drop sites, since drop-instead-of-panic at event-combinator boundaries can silently leak substream reservations/upgrade state that a panic's unwind would have released.

### Recorded disagreements (not resolved by the panel)

- Q1 log level (factual): Models 1 and 2 state drops log at debug! and rate HIGH for insufficient visibility; Model 3, applying the round-1 corrections, states all eight sites log at warn! and rates only MEDIUM (telemetry/triage ambiguity, not level).
- Q1 severity: HIGH (Models 1-2) vs MEDIUM (Model 3). Model 1 additionally speculates a race/interior-mutability path that could mis-pair the Either variants; neither other model finds evidence for this path.
- Q2 severity: NONE (Models 1 and 3, which affirm structural soundness of the Some arms) vs MEDIUM (Model 2, whose own reasoning concedes the pairing 'cannot mis-pair' — its MEDIUM effectively re-attributes the Q1 silent-None concern, not a Q2 defect).
- Q3 severity: HIGH (Model 1 asserts a resource leak with confidence) vs MEDIUM (Models 2-3; Model 3 explicitly declines to assert a confirmed leak, framing it as an unproven leak-shaped pattern with burden of proof on the patch).
- Q4 severity: MEDIUM (Model 1, 'principled inconsistency' concern if test code is ever un-gated) vs LOW (Models 2-3, non-operational since cfg-gated out of the build) — though all three agree there is no live security weakening in shipped code.
- Q5 residual: NONE (Model 2, coverage adequate; residual is only the documented open question about remote-induced desync) vs LOW (Models 1 and 3, which flag that not all eight converted arms are demonstrably exercised by the three named tests).

### Still open, and deliberately not closed here

1. **Acquire/release balance at the drop sites (round 2 finding 2, MEDIUM).**
   A drop at an event-combinator boundary may leave a substream reservation or
   upgrade future unreleased, where the original panic's unwind would have
   released it. One seat asserts a leak with confidence; the other two decline
   to assert it and put the burden of proof on the patch. This needs either a
   per-site resource-accounting argument in the review, or an instrumented
   run. NOT VERIFIED either way.
2. **Can a remote peer induce a desync (round 1 residual, round 2
   `Q5` residual)?** If yes, the drop class must be re-derived and Q1/Q3 move
   toward critical. This is the open question in
   `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md` and cannot be settled by
   reading code.
3. **Coverage shape:** the three named tests exercise the desync path, but not
   demonstrably all eight converted arms.

## Evidence locations (this checkout, `tmp/`, not committed)

- `tmp/rule8-pr372/pack/prompt_d9_vendor.txt` (round 2 prompt, with corrections)
- `tmp/rule8-pr372/pack/d9-vendor-delta.patch` (upstream -> vendored, 456 lines)
- `tmp/rule8-pr372/pack/d9_panel_result.json`, `d9_panel_result_v2.json`
- `tmp/rule8-pr372/source/vendor_handler_either_v2.rs` (the window reviewed)
- `tmp/rule8-pr372/upstream/` (the verified pristine crate)
