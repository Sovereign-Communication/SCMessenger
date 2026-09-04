# CTO DISPATCH PLAN -- 2026-08-21 (auditor-coordinated; operator-directed lane reassignment)

Status: ACTIVE -- supersedes the "Next seat" ordering in CTO_STATE section
0-latest for the remainder of this sprint. Written by the shadow auditor at
operator direction; the qwen CTO seat executes. Every fact cited was
verified this session (see CTO_SHADOW_AUDIT_2026-08-21.md in tmp/, to be
folded into HANDOFF/audit/ post-tag).

## 0. OPERATOR RULINGS BEING IMPLEMENTED (do not relitigate)

1. GPT/CAO lane is OUT OF OFFICE (API limit). Record an out-of-office
   event in the apple-windows journal per its own schema.
2. iOS/OSX DEPLOY BUILDS ONLY move to gemini-3.7-flash-high (agy) on the
   MacBook, dispatched through the existing gpt handoff lane protocol
   (HANDOFF/gpt/ rules of engagement still apply; paste xcodebuild output
   verbatim; xcodebuild on that machine remains the only iOS authority).
3. ALL other iOS/OSX work (planning, code, review) moves to the QWEN lane
   effective immediately.
4. Sprint goal A: v0.4.0 installed and working on the Pixel 6a, paired
   with Windows, full functionality (D4/D6/D7 scoring).
5. Sprint goal B (parallel): iOS/OSX updates + device pairing continue via
   the handoff lane.

## 1. DISPATCH A -- ANDROID/WINDOWS, THE 0.4.0 CRITICAL PATH (sequential)

**A1. Land #204 (APK native-lib regression).** It is MERGEABLE but red on
5 lanes (Test-windows, Bindings Kotlin/Swift, iOS Build x2).
- First: classify each red lane -- own-change vs environmental. Use the
  documented diagnostic (run the same check on a no-Rust PR; #194's
  clippy-1.98 incident on 2026-08-20 is the template).
- If own-change: the fix scope is the gradle Rust-profile wiring; keep the
  diff inside android/ gradle files + the packaging gate. Bindings/iOS
  lanes do not compile android gradle -- if they fail for a reason traceable
  to this PR's non-android files, get the diff narrowed instead of fixed
  broad.
- Merge gate: pr_scope.sh + full green, per standing rules. No red merges.

**A2. Install on the Pixel 6a from the CI artifact (no local gradle).**
- After A1 is green, download the android-debug-apk artifact from the
  green Mobile run (this is exactly the path that exposed the regression;
  it is also the cheapest install path -- local compute stays OFF).
- `adb install -r`, then verify: app launches past UnsatisfiedLinkError,
  identity creates, one message sends.
- Record device + build provenance (compare the build stamp / run URL) in
  docs/fieldtest/ per the existing evidence format.

**A3. Windows pairing field test -- D4/D6/D7 scoring.**
- Two nodes: Pixel 6a (A2 build) + Windows CLI from the #203 Windows
  release-binary artifact (already published by that PR).
- Scoring is fixed by SHIP_PLAN and repeated in every handoff:
  receiver-side decrypt + durable history + receipt. NOT transport ACKs,
  NOT UI counters, NOT BLE local acceptance.
- D6 variant: force first-choice transport unavailable, prove fallback
  delivery. D7 variant: no internet, proximity path.
- Record results in docs/fieldtest/ regardless of pass/fail. A documented
  failure that unblocks the next fix is worth more than an undocumented
  pass.

**A4. Operator gates (not CTO-executable):**
- Present A2/A3 evidence to the operator with the D-table filled.
- Operator approves -> publish v0.4.0-alpha.1 (D2 closes) -> tag v0.4.0.
- Four-node scoping ruling (AW4N gates 0.4.0 vs 0.5.0) is an OPERATOR
  decision -- ask for it in the same breath as the tag approval so the
  goalposts cannot move after the fact.

**A5. Post-tag hygiene (only after the tag):** renormalization PR,
verify_worker_commit.py, encoding sanity gate, dependabot disposition,
stale-branch cleanup on both remotes.

## 2. DISPATCH B -- IOS/OSX ON THE REASSIGNED LANES (parallel with A)

**B1. Journal the lane change FIRST** (the coordination contract requires
it before work moves):
- Append an out-of-office + delegation event to CAO_TO_CTO.md per the
  mandatory advisory event schema (item_id, sequence, APPROVAL/REQUEST
  type, scope APPLE, evidence refs).
- CTO also owes the reciprocal acknowledgment for AW-BILAT-0001 (bootstrap
  ledger says: "append reciprocal acknowledgment and nominate Windows
  preflight owner") -- do both journal writes in one commit.
- Nominate the Windows preflight owner in that same event.

**B2. Fresh planner for IOS-V050-1-REPAIR-2 -- QWEN LANE.**
- The redispatch requirements are already exhaustive in
  HANDOFF/gpt/IOS_V050_1_REPAIR_2_PLAN_STATUS_CONTINUITY_2026-08-21.md
  (on the fork branch gpt/apple-v1-cao-continuity-v2-2026-08-21 -- fetch
  pixiegirlchristy/SCMessenger; do NOT re-derive them).
- Non-negotiables from that spec: read-only planner; forward-application
  onto CURRENT main (the rejected base 5f052764 is 74 commits stale --
  re-derive the delta); no rebase/merge/push by the planner; conflict
  lattice + packet DAG; per-packet Apple-only vs core/FFI split decision
  (core/FFI packets are Windows-owned and need CAO/CTO + security gates).
- Planner output: plan path + SHA-256 + blockers + metadata, then STOP for
  review dispatch.

**B3. Gemini 3.7-flash-high on the MacBook -- DEPLOY BUILDS ONLY.**
- Scope strictly: xcodebuild archive/build, simulator + device install,
  test runs, paste output verbatim. NO implementation, NO plan approval,
  NO patch writing -- the BLOCK verdicts on the two prior patches stand
  until a fresh reviewed patch exists.
- Dispatch through the gpt handoff lane file protocol as normal; every
  build result lands as a handoff doc with commands + output quoted
  (R-Z2 evidence rule).

**B4. iOS code work -- QWEN LANE, gated on B2's plan.** No Swift
implementation before the fresh plan exists and its reviews pass. The two
rejected patches stay rejected; the fork's Swift files are continuity
state, not mergeable work.

**B5. PR #178 disposition** (11 days stale): decide revive-vs-fold after
B2's planner classifies the forward delta. Do not merge blind; its base
branch predates the entire August train.

## 3. EFFICIENCY RULES FOR THIS SPRINT (token/compute discipline)

- CI artifacts over local builds wherever a gate allows (A2 uses the CI
  APK; the Windows CLI uses the #203 artifact).
- One product-lane merge minimum per seat while any D-gate is open;
  process fixes only when they block product work.
- Cancel superseded run-sets on your own branch after every push
  (documented queue fix); do not re-fire a full run-set to test a hunch --
  use the cheapest decisive check first (the no-Rust-PR Lint diagnostic
  pattern).
- pr_scope.sh before every merge; never read checks as green through a
  failure (the script already fails closed).
- Free-lane tiering for any triage: gemini-3.7-flash-high implement /
  gemini-3.1-pro-high validate; glm-4.5-flash with thinking disabled for
  bulk reads; zai lane only via the fixed delegate.py path.

## 4. DONE MEANS

- Sprint goal A done = Pixel 6a runs a build containing #204, D4/D6/D7
  evidence recorded, operator tag approval requested with the D-table
  filled.
- Sprint goal B done = journals updated (out-of-office + reciprocal ack +
  preflight owner), fresh V050-1-REPAIR-2 plan produced and reviewed, and
  at least one gemini deploy-build cycle executed on the MacBook with
  verbatim evidence -- iOS code itself stays blocked pending its reviews.
