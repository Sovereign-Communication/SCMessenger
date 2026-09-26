# Windows CTO seat -> Mac lane (CAO): v0.4.0 + v0.5.0 four-node parity kickoff

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Date: 2026-08-21 (UTC)
From: CTO seat, Windows (Qwen FULL)
To: Mac lane / CAO (Chief Apple Officer)
Extends: MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md (handoff rule carries over)

## 1. Operator directive (2026-08-21), verbatim intent

Coordinate with the gpt/v050 parity branch; watch for handoff files from the
Mac lane; get the code perfected for v0.4.0 and v0.5.0 in unison; deploy to
4 of the 5 nodes (Windows, Android, macOS, iOS -- Ubuntu/AWS explicitly
excluded for now); verify Windows/Android/OSX/iOS compatibility such that all
4 nodes see each of the other three, and all features within scope work
between all the nodes.

The operator owns SHIP_PLAN.md; this directive is recorded as superseding its
section 4 hold on iOS-parity work for the parity workstream only. v0.4.0 tag
criteria D1-D7 stand unchanged, and the four-node gate is designed to
discharge D4/D6/D7 (section 4, Pass 3).

## 2. Current truth (verified by this seat on 2026-08-21; commands on record)

- main = 8663a149 (#201 merged 2026-08-21T06:01Z). All required contexts
  green; branch protection strict:true, 4 contexts, enforce_admins.
- DUAL_BIND fix (#180), all-nine Android wiring (#183), APK signing
  verification (#154) are ON main. Two-node LAN messaging was last observed
  broken at b4ccd30a by DUAL_BIND; nobody has re-proven messaging on
  hardware since the fix landed. That proof is now folded into Pass 1/2.
- Android deploy artifact for CURRENT main code exists: Mobile run
  32408319696 (merge of #194 at 9c80c597) uploaded android-debug-apk
  (21.4 MB, not expired); the path-filtered git log of
  9c80c597..origin/main over android/ iOS/ core/ Cargo.* mobile/ and the
  binding/wiring scripts is EMPTY, so that artifact is exactly current
  main's code. Release-signed APK ships with the tag (release.yml; #154
  verifies the signing path).
- Windows CLI: CI uploads NO Windows binary artifact today. This seat is
  dispatching a CI job to publish one per push; until it lands there is no
  deployable current-SHA Windows binary (local builds remain OFF by
  operator instruction; the last local binary was b4ccd30a, pre-fix).
- The gpt/v050 branches (v050-ios-release-ready 18 ahead, -readiness 7,
  -device-install 16) carry the July/August iOS parity work on a pre-#139
  base. They need a rebase onto current main before merge.

## 3. What this seat asks of the Mac lane / CAO

1. ACK this file per the MAC_WINDOWS_BLE_PARITY_QUEUE handoff rule: a
   response file in HANDOFF/gpt/ plus a comment on this seat's PR.
2. Rebase gpt/v050-ios-release-ready onto current main, resolve against the
   merged #139/#180/#183 world, and open a PR. Mac lane owns the rebase;
   xcodebuild on the Mac is the authoritative iOS gate. V050-I0..I5 in
   GPT_PLANNING_040_050_VERDICT.md still govern the lane order.
3. macOS node: bring up at the freeze SHA (section 4) once this seat
   publishes it; report listening multiaddrs + self-reported git hash.
4. iOS: build/install at the same freeze SHA; physical evidence per the
   VERDICT section 3 evidence rules -- receiver-side decrypt + durable
   history + receipt; never transport ACKs, UI counters, or BLE local
   acceptance.
5. The Windows seat runs a 20-minute watch on HANDOFF/gpt/ and gpt/* branch
   tips; new handoff files will be picked up and answered without prompting.

## 4. Four-node gate contract (DRAFT -- freezes on CAO ACK)

Nodes: Windows CLI (Windows host), Android (Pixel 6a), macOS, iOS.
AWS/Ubuntu node excluded per operator directive; its bootstrap entry stays
configured but its participation is not scored.

- Pass 0, provenance: all four nodes report the SAME git hash (the freeze
  SHA), captured as evidence before any messaging.
- Pass 1, mesh visibility: each of the 4 nodes discovers and connects to
  each of the other 3 -- 12 directed edges, each evidenced by
  ConnectionEstablished at sender AND receiver.
- Pass 2, delivery truth: messaging in both directions across a
  representative pair set, scored per GPT_PLANNING_040_050_VERDICT.md
  section 1.4 (steps 1-8): receiver-side decrypt, durable history, receipt
  round trip, restart stability, queued-delivery on reconnect.
- Pass 3, D6/D7 absorption: transport racing (D6) and offline proximity
  (D7) demonstrated within the same run so the v0.4.0 tag criteria are
  discharged by this gate rather than a separate two-node run.
- Pass 4, in-scope feature parity: the VERDICT section 2.2/2.3 matrix run
  per pair class; unsupported cells recorded honestly, never silently
  skipped or simulated (macOS BLE scan is currently simulated -- that must
  be labelled as such in any evidence).

Freeze SHA: chosen when (a) the Windows CI artifact job lands and (b) the
v050 rebase PR is green. Every node rebuilds/reinstalls to it, per the
PR139_FIVE_NODE_GATE_STATUS procedure minus the AWS arms.

## 5. Windows-side queue running beside this (context, not asks)

U-C2 swarm topic literals (rule 8 review pending before merge scheduling),
two-Commands enum unification, Rank-4 LedgerManager design note, U1/U2
wiring fixes, CRLF renormalization PR (post-tag), verify_worker_commit.py,
agy_stream_watch classification fix. None block the four-node gate.

## 6. Handoff rule (unchanged, binding)

Per MAC_WINDOWS_BLE_PARITY_QUEUE_2026-08-11.md: lanes acknowledge by file
and PR comment; each lane independently validates the other's evidence;
agreement is required before any shared-state claim. Receiver-backed
evidence only; no claim is green from sender status, CI, a peer table, or a
simulated scan.
