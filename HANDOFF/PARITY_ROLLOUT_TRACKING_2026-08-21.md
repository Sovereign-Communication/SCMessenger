# Parity rollout tracking -- v0.4.0 / v0.5.0 four-node gate

Status: Active
Created: 2026-08-21 (UTC)
Owner: CTO seat (Windows)
Audience: auditor (ox alpha), CAO / Mac lane (GPT-MAC), operator
Directive: operator 2026-08-21 -- perfect v0.4.0 and v0.5.0 in unison, deploy
4 of 5 nodes (Ubuntu/AWS excluded), verify full Windows/Android/macOS/iOS
mesh; Windows and OSX pairing stable first for the four-node test.

## 0. Why this document exists

One tracking surface for the parity rollout so the auditor, the Mac lane, and
the Windows seat read a single record instead of reconstructing state from
session logs. Standing update rule (CTO_STATE.md 0-rule): update at every
important change, dated, with the evidence next to the claim. Superseded
rows get marked, never deleted.

Coordinates, not duplicates:
- Gate contract (passes 0-4): HANDOFF/gpt/WINDOWS_V040_V050_FOUR_NODE_PARITY_KICKOFF_2026-08-21.md
- v0.5.0 lane order V050-I0..I5: HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md
- Windows/macOS pairing history: HANDOFF/gpt/MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md
  and HANDOFF/gpt/WINDOWS_GATE_EVIDENCE_AND_BRIDGE_RCA_2026-08-11.md
- Ship criteria D1-D7: SHIP_PLAN.md

## 1. Rollout board -- v0.4.0

| Item | State (2026-08-21) | Evidence | Owner |
|---|---|---|---|
| D1 main green | GREEN at 8b3ecfe5; moves when #204 merges | main CI runs on #200/#201/#202/#203 merges | CI |
| D2 signed APK downloadable | BLOCKED on tag; signing proof on main | #154 merged 2026-08-20 | operator at tag |
| D3 README | DONE | SHIP_PLAN board | -- |
| D4 two-device message+receipt | UNPROVEN since DUAL_BIND fix; folds into Pass 2 | ROLLOUT_2026-08-18_FIELD_EVIDENCE.md (broken by DUAL_BIND at b4ccd30a); #180 fixed, never re-proven | operator + seats |
| D5 no integration branch | DONE | #139 merged | -- |
| D6 transport racing | UNPROVEN; folds into Pass 3 | none yet | operator + seats |
| D7 offline proximity | UNPROVEN; folds into Pass 3; BLE gaps below | none yet | operator + seats |
| APK packaging regression | FIX IN FLIGHT as #204 (gate green twice on PR) | PR #204 + CI logs; RCA in PR body | CTO seat |
| Windows deploy artifact | LIVE on main | #203 merged as 8b3ecfe5; job ran green on #203/#204 | CI |
| Freeze SHA | NOT SET -- after #204 merges | this board, section 2 | CTO seat |

## 2. Windows-OSX pairing stability worklist (first pair to stabilise)

The four-node gate starts with the Windows<->macOS edge. Known state from the
2026-08-11 gate evidence: a p2p pairing existed (Windows saw only the Mac
node), but every artifact is now stale -- the Windows soak binary is
053fd137-era, pre-DUAL_BIND -- and the bootstrap-mismatch stop condition from
the AWS role switch was never closed.

| # | Step | State | Owner |
|---|---|---|---|
| W1 | Merge #204 (APK native-lib fix + gate) | IN CI, required lanes green so far | CTO seat watch |
| W2 | Set freeze SHA = the #204 merge commit | NOT SET | CTO seat |
| W3 | Windows node redeploy at freeze SHA from the windows-cli-<sha> CI artifact, identity preserved | BLOCKED on W2; no current binary on the host (old soak binary pre-DUAL_BIND, do not reuse) | CTO seat + operator |
| W4 | macOS node at the same SHA | BLOCKED on W2 | CAO / Mac lane |
| W5 | Bootstrap alignment: 2026-08-11 stop condition (all nodes still name the OLD AWS PeerId; Windows config updated but never activated). NOT needed for a LAN four-node test (mDNS/LAN discovery); must close before any WAN claim | OPEN, deferred by the AWS exclusion | CTO seat + CAO |
| W6 | Pairing evidence: Windows<->macOS edge first -- Pass 1 ConnectionEstablished at BOTH ends, then Pass 2 delivery truth both directions | BLOCKED on W3/W4 | both seats |
| W7 | Honest gaps carried into all evidence: macOS BLE scan is SIMULATED; Windows WinRT GATT peripheral unverified; BLE-only matrix cases stay gated per the 2026-08-11 parity queue until both sides are real. TCP/LAN is the stability baseline | STANDING | both seats |
| W8 | Mesh coordination channel: the Windows inbox_bridge allow-list currently carries the Android and GPT-MAC identities only. Any new lane (including the auditor) that coordinates OVER THE MESH must supply its peer identity for allow-listing; git HANDOFF files need no allow-list | OPEN | CTO seat |

## 3. Rollout board -- v0.5.0 (iOS parity, in unison per directive)

| Lane | Task | State | Owner |
|---|---|---|---|
| REBASE | gpt/v050-ios-release-ready (18 ahead, pre-#139 base) onto current main, then PR | REQUESTED via #202 kickoff; awaiting CAO ACK | CAO / Mac lane |
| V050-I0 | restore committed iOS test truth (XCTest target must compile at the SHA) | PENDING REBASE | CAO |
| V050-I1 | bindings ratchet (copy/assert/verify scripts blocking, no continue-on-error) | PENDING I0 | CAO + Windows FFI gate |
| V050-I2 | retry-timing parity (8s vs 60s initial retry) | PENDING I0 | CAO |
| V050-I3 | truthful transport settings (remove unsupported WiFi Aware/Direct toggles) | PENDING I0 | CAO |
| V050-I4 | receipt state-machine contract (Sent/Delivered/Read/Failed across core/Android/iOS) | PENDING I0; core changes route through Windows + rule-8 review | GPT-think + seats |
| V050-I5 | physical parity matrix (iOS<->Android, iOS<->iOS; BLE/Multipeer/relay/fallback) | PENDING I1-I4; folds into four-node Pass 4 | CAO + operator devices |

## 4. Auditor contract (ox alpha)

You are asked to ACK this document and align on scope before the gate runs.

1. ACK mechanism: reply file in HANDOFF/ (name it for yourself, e.g.
   HANDOFF/OX_ALPHA_*.md) plus a comment on this document's PR. If you
   coordinate over the mesh instead, provide your peer identity first (W8).
2. Evidence standard is GPT_PLANNING_040_050_VERDICT.md section 3: receiver-
   side decrypt + durable history + receipt. Sender status, CI green, peer
   tables, and simulated scans are NOT delivery evidence. Provenance claims
   bind artifact -> SHA -> node; the APK stamp is self-attested, not
   cryptographic (recorded caveat, 2026-08-11 gate evidence).
3. Recommended audit scope, in risk order:
   a. Provenance chain of every deployed artifact at the freeze SHA.
   b. Gate integrity: the APK native-lib gate (#204), the wiring gate,
      pr_scope.sh -- each has failed open before; verify they now fail closed.
   c. The four-node run's evidence bundle as it lands (Passes 0-4).
4. Findings land in HANDOFF/audit/ with severity and the exact evidence;
   BLOCK verdicts stop the gate until dispositioned by the operator.

## 5. Change log (newest first)

- 2026-08-21: created. #202 kickoff merged (48303050); #203 Windows CLI
  artifact job merged (8b3ecfe5), job already green on PR CI; #204 APK fix +
  gate in CI with the gate green twice on the PR; Pixel carries the crashing
  pre-fix APK as evidence until the post-#204 install.
