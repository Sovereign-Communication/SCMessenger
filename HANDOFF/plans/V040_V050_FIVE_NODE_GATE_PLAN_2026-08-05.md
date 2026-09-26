# v0.4.0 / v0.5.0 Completion Plan -- Five-Node Fleet Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active -- operator-requested planning doc (2026-08-05)
Authority stack (unchanged, do not relitigate):
  Sequencing:  HANDOFF/V1_0_0_EXECUTION_PLAN.md (Section 0A)
  Slicing:     HANDOFF/plans/MILESTONE_RELEASE_PLAN.md
  Dispatch:    HANDOFF/todo/_QUEUE.md + docs/ORCHESTRATION.md
This doc adds ONE new construct on top of those: the five-node fleet test
is the release gate for v0.4.0, and its results carry forward into v0.5.0.

---

## North star

Five heterogeneous nodes, one fleet: any node can message any other node,
both directions, with honest delivery state, over every transport the
hardware supports -- LAN, BLE, and internet relay -- and the fleet view
converges on every node without re-pairing. Version tags follow proven
fleet behavior; fleet behavior does not follow version tags.

Guiding ideal: a stranger-level bar from the v1.0.0 plan still applies --
worst case (network drops, IP flips, app restarts, node offline-then-
returning), not the happy path.

## The fleet (five nodes)

| # | Node | Platform | Lane | Update path |
|---|------|----------|------|-------------|
| 1 | Windows dev node | CLI (scmessenger-cli.exe) | orchestrator (this host) | cargo build at main tip (debug cache is the gate env; no release cache locally) |
| 2 | Pixel 6a | Android app | orchestrator | CI `android-debug-apk` artifact (Mobile workflow, every main push), `adb install -r` in place -- identity/messages preserved |
| 3 | Christy's iPhone | iOS app | MAC LANE (GPT) | Xcode build/install from MacBook; handoff packets in HANDOFF/gpt/ |
| 4 | MacBook node | macOS CLI/desktop | MAC LANE (GPT) | same handoff; CLI via cargo or release artifact |
| 5 | AWS always-on node | cloud relay (Docker) | orchestrator via aws CLI | teardown + rebuild (NO SSH key exists locally -- in-place update is structurally blocked); user-data pulls `testbotz/scmessenger:latest` (Docker Publish pushes latest+sha tags on every main push). Playbook: HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md |

Known structural constraint: AWS public IP drifts on every rebuild (EIP
allocation is explicitly denied by IAM policy `SCMessengerRelayFreeTierOnly`).
After every rebuild: capture the new IP, update bootstrap configs on the
other four nodes, and update any doc that hardcodes it.

---

## v0.4.0 -- what is already done (verified at HEAD, not doc-claimed)

- Outbox Site-1 flush on reconnect (f521f142, 4 call sites)
- Receipt round-trip: core classify path + CLI serde fix (8f866bfc)
- Ledger choke-point (22b921ca); Android relay de-hardcode (f010a0f1)
- PR #136 (merged 68ef6256): identity canonicalization + block gate
- PR #137 (merged f9e0def5): transport liveness -- zombie transport
  reconciliation + auto-reconnect of stale peers (the fix for the
  field-observed mid-session halt)
- PR #138 (merged 6b2573fa): Android mDNS hardening Phase 1 -- permission
  gating, queryable lastFailureReason, idempotent registration/discovery,
  bounded retry, pinned `_p2p._udp` interop constant
- Field proof 2026-08-05: bidirectional Android<->iOS messaging CONFIRMED
  live on Pixel 6a + iPhone

## v0.4.0 -- what remains before tag

1. **Identifier-gate follow-ups P1/P3/P4** from the PR #136 phase-0b
   review -- explicitly flagged "open before the v0.4.0 tag":
   HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md
2. **Ledger visibility gap Phase 2** (trust-scoped LAN disclosure):
   ticket HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md,
   design HANDOFF/plans/TRUST_SCOPED_LAN_DISCLOSURE_DESIGN_2026-08-05.md.
   BLOCKED ON OPERATOR DECISION (disclosure policy = security trade-off,
   AGENTS.md rule 9). Phase 1 (PR #138) already gives queryable failure
   state so the gap is diagnosable, not silent.
3. **Transport BLE/LAN hiccup verification**:
   HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md --
   confirm PR #137's liveness fix closed the mid-session halt, and which
   transports actually carried traffic.
4. **Five-node rollout at one HEAD** (in progress this session): every
   node running the same commit, provenance stamps compared before test.
5. **The five-node gate itself** (below), twice reproducible.

Explicitly EXCLUDED from v0.4.0 (operator-confirmed, unchanged): iOS lane
features, P1-14/P1-18 hostile-network proofs, PQC-09, B1 DNS hardening,
farm-sim chain, KMP desktop, PQC waves 3-5 (E1 critical stays v1.0.0).

---

## THE FIVE-NODE GATE (v0.4.0 exit test)

Precondition: all five nodes on the same commit; provenance stamps (git
hash + build ts + libp2p version, P1-05) recorded side by side first.

- G1 PAIRWISE BIDIRECTIONAL. Every reachable node pair exchanges messages
  in BOTH directions with receipts. Hardware-blocked pairs get a recorded
  waiver, never a silent skip.
- G2 TRANSPORT COVERAGE. For each pair, every hardware-supported transport
  delivers: LAN (mDNS discovery + TCP/QUIC/WS), BLE (Android<->Windows),
  internet relay through the AWS node including CUSTODY proof (recipient
  offline at send time -> message held -> delivered + receipt on return).
  WiFi Aware / WiFi Direct Android<->Android cells stay waived to v1.1
  (BLOCKED-HW, one Android device).
- G3 DELIVERY TRUTH. Statuses reflect receipts end to end: no false
  failures, no checkmark without verified receipt, outbox flushes on
  reconnect (kill network -> restore -> queued traffic lands).
- G4 FLEET CONVERGENCE. Every node lists the full fleet (nodes + headless
  entries) within a bounded window; app restart re-converges WITHOUT
  re-pairing. This is the acceptance for the ledger-visibility ticket --
  Phase 2 disclosure policy must be settled by the operator first.
- G5 LIVENESS. No mid-session halt: disrupt the network for several
  minutes, restore, and peers auto-reconnect without restarting apps
  (fleet-level proof of PR #137).
- G6 PROVENANCE. All five nodes report the same git stamp -- the gate
  only counts when all nodes provably run the gated commit.

Evidence: recorded outputs/device logs appended to a dated ledger doc
(docs/release-readiness style). Run the gate TWICE, reproducibly, before
tag. One pass is luck; two passes is behavior.

## v0.4.0 tag mechanics

Tagging is MANUAL-ONLY (operator directive 2026-07-28): after the gate
passes twice, operator decides alpha (`v0.4.0-alpha.N`) vs stable
(`v0.4.0`), then Actions -> Auto Tag Release (tags whatever
[workspace.package] version says; currently 0.4.0). release.yml then
produces the install artifacts (CLI binaries all platforms, signed
APK/AAB) -- those become the fleet's canonical artifacts, replacing
per-push debug builds. CHANGELOG truthing + docs-sync land in the same
wave. Then bump workspace to 0.5.0.

---

## v0.5.0 -- Farm Simulation Release

Scope per MILESTONE_RELEASE_PLAN.md (unchanged): farm-sim infra (contact
provisioning, /api/identity), delivery truth (A4 outbox/drift single
ownership, F2 drift persistence), reach/anchor (B3 anchor deployment,
B4 cloud relays as secondary bootstrap, B5 P1-14 hostile-network + P1-18
WAN-relay proofs on the rig, B6 12-node soak), honesty/observability
(G1 network-error observability, G2 honest UI states), H1 onion seam
freeze, U5/U7 unifications.

Gate: all six farm scenarios pass in the 12-node Docker simulation on the
AWS rig, AND the five-node field gate still passes (regression, not
assumption). v0.4.0's gate evidence is the baseline; v0.5.0 re-runs it.

---

## Guidelines (ideals, not shackles)

1. Fleet convergence is the tie-breaker. When two tasks compete, the one
   that moves the five-node gate forward wins. Everything else queues.
2. Evidence over claims. A gate criterion closes with recorded output or
   a device log, never with "should work."
3. One HEAD during gate periods. All nodes on the same commit; compare
   stamps before testing. Artifact skew is a known root-cause class
   (P1-04 history) -- never retest on mismatched builds.
4. Process as settled: PR-first + green CI for functional changes;
   HANDOFF state files may go direct-to-main; CI-on-push is the full
   gate; Windows host is the authoritative build verifier; merges are the
   orchestrator's job.
5. Waivers are recorded, never silent. A matrix cell that cannot be
   tested gets an explicit, operator-signed waiver in the evidence doc.
6. Lane discipline: qwenpaid-first dispatch (routing directive
   2026-07-28 stands); MAC LANE (GPT) owns iOS/macOS and is API-budget
   constrained until ~2026-08-09 -- handoffs to it must be
   self-contained, step-by-step, near-zero-iteration; Claude Code stays
   locked out per its ticket.
7. Security gates are not negotiable: adversarial review on
   core/{crypto,transport,routing,privacy} changes; operator escalation
   on disclosure policy, release timing, API contracts.
8. Budget reality: prefer dispatching well-specified packets over long
   interactive exploration; this plan exists so ANY session can resume
   without re-deriving context.

## Open operator decisions

| # | Decision | Blocks |
|---|----------|--------|
| D1 | Ledger Phase 2 disclosure policy (trust-scoped LAN disclosure design) | G4 fleet convergence criterion as specified |
| D2 | v0.4.0 tag flavor (alpha.N vs stable) after gate passes twice | release artifacts |
| D3 | Accept AWS IP drift standing condition (EIP denied by IAM) or widen policy | unattended reconnect to cloud node; DDNS alternative referenced in provision-relay.sh |

---

## Resume state (rollout run of 2026-08-05/06, post-PR-138)

- main = 6b2573fa (PR #138 merged); full CI suite running on the push.
- Windows node: `cargo build -p scmessenger-cli` at main tip (debug).
- Android: Pixel 6a reachable over wireless adb (192.168.0.140:38351);
  APK source = Mobile workflow run for 6b2573fa, `adb install -r`
  (in-place, data preserved) -- this also validates the update path.
- AWS node: rebuild gated on Docker Publish for 6b2573fa (image tag
  latest+sha), then teardown/rebuild per AWS_RELAY_REBUILD_2026-08-04.md;
  new IP captured and propagated.
- iOS + macOS: self-contained install packet in HANDOFF/gpt/ (MAC LANE,
  API-constrained window) with post-install steps to rejoin the fleet
  at the current AWS IP.
- After rollout: run the five-node gate (first pass), file evidence,
  then dispatch IDENTIFIER_GATE_FOLLOWUPS P1/P3/P4 + ledger Phase 2
  decision as the remaining pre-tag work.
