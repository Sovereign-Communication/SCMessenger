# GPT VERDICT -- v0.4.0 completion and v0.5.0 execution plan

Status: READY FOR ORCHESTRATOR INTAKE
Release verdict at review baseline: v0.4.0-alpha.1 NO-SHIP
Baseline: `origin/main` `74a6808d`; staging ref
`refs/heads/wip/v040-seeding-fixes` `068972f2`
Prepared: 2026-07-28
Owner model: GPT-5.6 Sol, Mac lane

## Executive decision

The release order remains v0.4.0-alpha.1, then v0.5.0, then v1.0.0.
v0.4.0 is not ready to tag. The ledger-seeding remediation has two accepted
NO-SHIP review rounds, its full Windows gate is still pending, and the staged
packet list does not yet explicitly close every finding the operator mandated
for this tag. A current-head real delivery and receipt proof is also still
required.

v0.5.0 can start without an idle planning gap: prepare its iOS repair branch
and exact farm-sim re-cut while v0.4 operational evidence is collected, but do
not add either to the v0.4 release scope. Merge v0.5 work only after the alpha
tag unless the Windows orchestrator records a collision-free exception.

Two sequence corrections are mandatory:

1. The current iOS XCTest target does not compile at `74a6808d`. The prior
   "47/47" handoff is not reproducible from the committed tree and is not
   release evidence.
2. `.github/workflows/auto-tag-release.yml` tags every main push that changes
   the workspace version. Merging a `0.3.5 -> 0.4.0` bump before the terminal
   gate would create `v0.4.0`, not the locked `v0.4.0-alpha.1`. Hold the version
   commit until the end and resolve that workflow/tag mismatch before merging
   it.

The untracked `tmp/v040-completion-wave.md` named by the queue and planning
packet is absent in this checkout and cannot be an auditable authority.
Current tracked queue headers, fetched commit trees, review verdicts, and
recorded command/device evidence govern this plan.

---

## 1. v0.4.0-alpha.1 completion plan

### 1.1 Scope and current truth

The Josh release is deliberately narrow:

- Windows CLI and Android app only.
- Hawaii to Pennsylvania through the live AWS relay.
- A real message in both directions and a real receipt round trip.
- Unattended reconnect behavior, matching build provenance, installable
  artifacts, and no false delivery state.

Already landed work such as outbox Site 1, receipt classification, Android
retry suppression, ledger choke-point consolidation, adaptive dialing, and
relay de-hardcoding is an input, not a substitute for the current-head proof.

The following are explicitly outside v0.4.0:

- iOS implementation or distribution.
- P1-14/P1-18 hostile-network farm rig work beyond what is needed for the
  literal Josh WAN proof.
- Farm simulation, Meeting Mode, KMP desktop, PQC-09 onion wiring, WiFi Direct
  Android-to-Android, and B1 DNS hardening.
- General backlog cleanup or cosmetic documentation work.

### 1.2 Critical-path work

| ID | Owner | Depends on | Size | DONE evidence |
|---|---|---|---|---|
| 040-G0 baseline freeze | Windows orchestrator | none | S | Record immutable main, staging parent/tip, dirty-state status, and the exact commits entering the release. No result may depend on an untracked Windows-only plan or uncommitted tree. |
| 040-S1a staged seeding remediation | qwen implementation via Windows, serial single-writer lane | 040-G0 | L | Land v2a, v2c, v2b, 1c, and packet 2: bounded/validated load; serialized atomic persistence; anchor/deterministic eviction; one production batch save; F7 dial-policy and failure wiring; F13 dialer-only completion; global anti-Sybil bucket. Required race, cap, persistence, determinism, and production-caller tests pass. |
| 040-S1b complete original finding closure | qwen implementation via Windows, GPT review | 040-S1a | M | The terminal verdict explicitly names F2, F3, F6, F7, F10, F12, F13, F16, and NEW-6. F2 signed-vs-live provenance, F3 DNS/private-address policy, F6 disclosure/filter/rate-limit ordering, F12 ranking poison at every sink, and F16 bindings/FFI drift cannot disappear merely because packet 2 fixes a subset. Every operator-mandated open finding is FIXED or has an operator-signed release decision; implicit deferral is forbidden. |
| 040-S2 independent adversarial verdict | GPT second opinion, Windows gatekeeper | 040-S1a and 040-S1b | M | Review the exact parent..tip range plus final tree. Verdict contains one evidence-backed disposition per finding and a final SHIP/NO-SHIP line. Re-run after any remediation; no self-review-only acceptance. |
| 040-S3 Windows compile, test, and FFI gates | Windows orchestrator | final S1 tree | M | Authoritative fmt, clippy, workspace build/test/compile gates, Android unit/build gates, rules/docs checks, and P6 FFI snapshot all pass at one SHA. Any UDL/public API change has regenerated Kotlin/Swift outputs or an explicit proof that none changed. |
| 040-S4 current-head local delivery truth | Windows orchestrator, Android emulator | final S1 tree; may run parallel with S3 | M | Matching build-provenance stamps; target-scoped outbound `ConnectionEstablished`; unique message ID stored/decrypted at receiver; authentic receipt returns; sender transitions to Delivered only from that receipt; restart preserves the state; repeat in both directions. Dial-queued, peer-discovered, transport-ACK-only, or UI-only evidence fails. |
| 040-S5 literal Josh WAN proof | operator plus Josh/Lucas, Windows orchestrator captures evidence | final S1 tree and healthy relay; may prepare in parallel | M | Hawaii and Pennsylvania endpoints use the candidate artifacts and matching SHA. AWS relay health and public reachability are captured. Both directions deliver and receipt-confirm over the WAN without hand-editing state. A disconnect/reconnect arm drains a queued message unattended. |
| 040-S6 release truth and tag | Windows orchestrator plus operator only | S2, S3, S4, S5 | S | Version, CHANGELOG, docs, artifacts, checksums, install smoke, CI, and tag all point to the same reviewed SHA. Operator controls the final merge/tag decision. |

### 1.3 Parallel tracks and dependency discipline

```text
Track A, serial security:
  G0 -> S1a -> S1b -> S2

Track B, release gates:
  final S1 -> S3

Track C, evidence:
  relay/port/DDNS preparation now
  final S1 -> S4 and S5 in parallel

Terminal:
  S2 + S3 + S4 + S5 -> S6

v0.5 pre-stage, no v0.4 scope change:
  iOS repair spec/branch + V050-P2-00 evidence inventory
```

`swarm.rs`, `mobile_bridge.rs`, and `ledger_entry.rs` remain single-writer
hotspots. Do not overlap packet 2, caller swap, or another transport fix.
Operational relay checks, release-note drafting, and v0.5 read-only re-cutting
can run while that lane is occupied.

### 1.4 Josh-test proof protocol

For each direction:

1. Record the Git SHA/build provenance of the installed CLI and APK. They must
   match the reviewed release candidate.
2. Record sender identity, intended recipient identity, relay endpoint, and
   one newly generated message ID.
3. Capture an outbound connection establishment for the intended peer/address.
   A command reply or dial queue entry is not enough.
4. Send once. Capture receiver decryption and durable history insertion for
   that exact message ID and payload hash.
5. Capture receiver receipt generation, sender receipt callback, pending
   outbox removal, and sender history/UI Delivered transition for the same ID.
6. Restart the sender and receiver and show that the message remains exactly
   once and does not re-enter retry.
7. Disconnect one endpoint, send a second unique message, reconnect without
   manual state edits, and prove queued delivery plus receipt.
8. Repeat the complete test in the reverse direction.

The evidence bundle must include UTC timestamps, sanitized logs, candidate
artifact checksums, build provenance, command transcript, and a short
pass/fail manifest. Redact keys, tokens, device serials, and personal content.

### 1.5 Final tag checklist

- [ ] Terminal ledger-seeding verdict is SHIP and covers every named finding.
- [ ] All authoritative Windows/Rust/Android/FFI gates pass at the candidate
      SHA; working tree is clean.
- [ ] Required GitHub checks for that SHA are green without a manual rerun or
      hidden local patch.
- [ ] CLI and APK provenance strings match the candidate SHA.
- [ ] Current-head CLI-to-emulator delivery and receipt proof passes both ways.
- [ ] Literal Hawaii-to-Pennsylvania Josh proof passes both ways through the
      healthy AWS relay, including queued reconnect.
- [ ] AWS service health, public endpoint, firewall/security-group policy, and
      deployment image digest are recorded.
- [ ] Lucas verifies TCP 443, TCP 80, UDP 443, and the DDNS record from an
      external network if the home/farm anchor is part of the declared alpha
      topology. If Josh is AWS-only, the operator records that explicit alpha
      waiver rather than silently treating untested home forwarding as done.
- [ ] Cargo workspace version and Android versionName are consistent; Android
      versionCode is monotonically advanced and both clean-install and upgrade
      install behavior are smoke-tested.
- [ ] CHANGELOG/release notes describe what was actually proven and list
      exclusions without claiming iOS or farm readiness.
- [ ] Windows CLI and Android APK are generated, checksummed, downloadable,
      installed from the release artifacts, and smoke-tested.
- [ ] `.github/workflows/auto-tag-release.yml` cannot create an early or wrong
      `v0.4.0` tag. Chosen execution: in the terminal release PR, remove its
      automatic main-push trigger while retaining an inert manual definition;
      set the source versions to `0.4.0`, advance Android versionCode, merge only
      after all preceding boxes pass, and let the operator create
      `v0.4.0-alpha.1`. The tag then triggers `release.yml`. Do not rely on an
      automatically generated stable tag for this prerelease.
- [ ] The operator, not the Mac lane, merges and tags. The resulting GitHub
      prerelease points at the reviewed SHA and exposes the expected artifacts.

---

## 2. v0.5.0 plan: farm simulation plus iOS-Android parity

### 2.1 Exit definition and zero-idle handoff

v0.5.0 is complete when:

- The 12-node, three-group farm rig executes all six topology scenarios with
  authentic contacts, encrypted messages, receipts, custody, failure
  injection, recovery, and measured results.
- Android and iOS implement the same delivery-truth contract and the intended
  platform-equivalent transport behavior.
- The committed iOS app and XCTest targets build and test at the same SHA,
  generated bindings are proven current, and physical-device evidence covers
  radio, seed/restart, relay, and background boundaries.

Immediately after the v0.4 tag, the Windows orchestrator opens
`V050-P2-00/FARM-SIM-REBASE` and the Mac lane promotes its already-prepared
iOS repair branch. Do not wait to rediscover the stale milestone statuses.

PR #114 is a focused iOS tooling prerequisite for physical-device work. It
replaces the useful portion of PR #111 with exact stable-ID resolution and
fail-closed tests. PR #111 must not be merged.

### 2.2 Current iOS ground truth

| Area | Current tree truth | v0.5 action |
|---|---|---|
| XCTest target | Registered, but does not compile. `ReceiptUnificationTests.swift` calls nonexistent component helpers; `MeshBackgroundServiceTests.swift` uses `Task<Void, Error>` for `Task<Void, Never>` results. | P0 repair first; record exact test count at current SHA. Prior "47/47" evidence is invalidated. |
| Receipt codec | Generated API exposes `encodeReceipt(receipt:)` and `decodeReceipt(data:)`; stale tests target a different API. `core/src/iron_core.rs` currently maps both Read and Failed through its wildcard Delivered arm, while the Swift callback can emit Failed and the repository ignores it. | Use the real generated types in tests, then settle exact Sent/Delivered/Read/Failed semantics across core, Android, and iOS. |
| Outbox retry | Seven-day ceiling exists, but iOS initially retries a transport-acked/no-receipt message after 8 seconds while Android waits 60 seconds. | Align or explicitly justify timing and prove no failure-state downgrade or duplicate churn. |
| Ledger/bootstrap | Preferred and dialable ledger candidates feed automatic/manual `startSwarm`; no hard-coded iOS relay remains. | Evidence-only until fresh-install, learned-seed, restart, and physical dial are recorded. |
| Local transport | BLE, Multipeer, and TCP/mDNS code exists as the intentional iOS equivalent of Android BLE/Aware/Direct/LAN. | Do not invent WiFi Aware/Direct on iOS. Prove payload and receipt over physical routes and fallback. |
| Settings | iOS displays WiFi Aware and WiFi Direct toggles even though those transports are unsupported. | Replace with truthful Multipeer behavior/control or remove/disable unsupported controls. No fake preference. |
| Background | BG task and notification scaffolding exists; tests inject no-op work. | Physical lock/background/wake evidence is still required; do not claim an OS scheduling SLA. |
| Binding drift | A checksum mismatch occurred previously; CI currently treats binding verification as non-blocking. | Make drift verification blocking before the Xcode build. |

### 2.3 Ordered iOS parity lane

| ID | Owner | Depends on | Size | Gate |
|---|---|---|---|---|
| V050-I0 restore committed test truth | Mac Swift lane | v0.4 branch point only | S | Fix the two stale test files against the generated API. App build and complete `SCMessengerTests` scheme pass at one SHA; count and `.xcresult` recorded. |
| V050-I1 bindings ratchet | Mac lane plus Windows FFI gate | I0 | S/M | `iOS/copy-bindings.sh`, `iOS/assert-generated-path.sh`, `scripts/verify_ios_bindings.sh`, P6 FFI snapshot, and Xcode build/test pass. Remove `continue-on-error` from the binding-drift CI step. Commit Swift, C header, module map, and related generated outputs atomically. |
| V050-I2 retry-timing parity | Mac Swift lane | I0 | M | Deterministic XCTest covers the initial acknowledged/no-receipt delay, adaptive schedule, age ceiling, and no downgrade to Failed/Corrupted. Simulator build/test pass. |
| V050-I3 truthful transport settings | Mac Swift lane; operator only if a new cross-platform setting is proposed | I0 | S/M | Every visible iOS transport control changes real service behavior and one persisted source of truth. Unsupported Android-only toggles are absent. Simulator UI/service tests pass. |
| V050-I4 receipt state-machine contract | GPT-think for contract review, qwen/Windows for any core change, Mac for Swift | I0 and S1 security lane free | M | Exact Sent/Delivered/Read/Failed semantics are specified. Only an authenticated delivery receipt may mark Delivered or clear delivery retry. Duplicate and out-of-order receipts are idempotent. Windows core/FFI gates, Android tests, iOS tests, and protected-tree review run where applicable. |
| V050-I5 physical parity matrix | Mac lane plus operator devices | I1-I4 | L evidence | Physical iOS-to-Android and iOS-to-iOS: BLE, TCP/mDNS or Multipeer as applicable, relay, route-loss fallback, receipt, exactly-once history, fresh seed/restart, and background/wake persistence. Record unsupported cells honestly. |

Exact Mac simulator gates for every iOS implementation task:

```text
xcodebuild build -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 17 Pro' CODE_SIGNING_ALLOWED=NO
mkdir -p tmp/xcode-results
xcodebuild test -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessengerTests -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 17 Pro' -resultBundlePath tmp/xcode-results/<candidate-sha>-SCMessengerTests.xcresult CODE_SIGNING_ALLOWED=NO
```

Use a resolved simulator UDID when the named destination is ambiguous.
Simulator success never waives physical BLE, Multipeer, relay, or background
evidence.

### 2.4 Farm-simulation and delivery-truth lane

| ID | Owner | Depends on | Size | Gate |
|---|---|---|---|---|
| V050-P2-00 re-cut | qwen MAX analysis, Windows orchestrator accepts | v0.4 tag | S | Reconcile queue, done/todo tickets, current code, rig availability, and exact 12-node matrix. Publish one collision-aware pick list. |
| V050-FS1 contacts and identity | qwen implementation via Windows | P2-00 | M | Deterministic or securely provisioned identities and contacts; live API exposes only the minimum authenticated identity material needed. `/api/send` sends an encrypted message instead of returning 404. Unit/integration tests plus a clean-container proof. |
| V050-FS2 fault-injection capability | qwen implementation via Windows | P2-00; parallel with FS1 | S/M | Add a bounded, least-privilege partition mechanism to the test image/topology instead of assuming absent `iptables`. Automated probes prove latency, loss, full partition, healing, crash/restart, and metrics capture without granting production containers unnecessary network authority. |
| V050-A4 single ownership | qwen high via Windows | P2-00; parallel with FS1 | M | Test traces a message through outbox and Drift custody and proves exactly one active owner; receipt clears every applicable queue. Protected-tree fixes receive adversarial review. |
| V050-F2 MeshStore decision | qwen high investigation, Windows accepts | P2-00 | S/M | RelayCustodyStore persistence is not reimplemented. Prove whether in-memory MeshStore holds unique data. Persist only if process-death testing demonstrates loss; otherwise close with evidence. |
| V050-B1/B2 DNS and bootstrap contract | qwen high via Windows plus adversarial review | P2-00 | M/L | Mid-session DNS IP change reconnects without restart; hostname survives in ledger/negative-cache identity; one documented bootstrap precedence feeds every platform. Required before the IP-flip rig gate. |
| V050-B3/B4 rig activation | Windows/orchestrator and operator; qwen scripts | P2-00 and operator reopens paused cloud work; parallel with FS1/A4/F2 | M | Verify existing `--http-bind` and `/health` implementation rather than redoing it. Activate approved AWS/Alibaba resources, firewall/DDNS, image digest, monitoring, and three-group topology. |
| V050-B5 hostile/WAN proof | Windows/orchestrator | FS1, FS2, B1/B2, and B3/B4 | M evidence | P1-14/P1-18 profiles prove WAN relay/custody, carrier-filter fallback, packet loss/latency, partition recovery, and IP flip with authentic messages and receipts. |
| V050-B6 12-node six-scenario soak | Windows/orchestrator | B5, A4, F2 decision | L evidence | All S1-S6 profiles use 12 nodes, nonzero contacts, real encrypted sends, delivery/receipt metrics, restart/failure arms, and a multi-hour resource soak. Command success without application delivery fails. |
| V050-G2 honest UI states | qwen platform implementation, Mac mirror | I4 and A4 | M | Queued -> InCustody -> Sent(unconfirmed) -> Delivered(receipt-verified) is consistent on Android/iOS and in persistence. |
| V050-G1 diagnostics | qwen implementation | core delivery flow stable; parallel before dogfood | M | A user-exportable sanitized report preserves actionable dial/routing failure detail without secrets or social-graph leakage. |

### 2.5 Do not redispatch completed or disproven work

- U7 schema drift/versioning is done. Verify its tests; do not estimate it as
  new v0.5 work.
- U5 Android receipt unification has a done ticket but its acceptance boxes and
  real round trip must be re-proven in the v0.4/v0.5 evidence runs.
- The WiFi Aware "orphan" is a closed false positive. Its loopback TCP path is
  intentional.
- B3's `--http-bind`/health code and H1's
  `core/tests/seam_freeze_onion.rs` already exist. Treat both as verify-first
  plus runbook/evidence work, not fresh implementation.
- Meeting Mode, PQC depth, KMP desktop, and full field rollout do not enter the
  v0.5 critical path.
- Apple Developer/TestFlight is a human gate for the farm pilot, not a reason
  to delay simulator or locally signed physical parity work. Record the
  distribution decision before pilots begin.

### 2.6 Binding-drift prevention discipline

1. Any UDL/core public-surface PR declares whether bindings change.
2. Windows lands the reviewed core/API source first; the Mac lane regenerates
   once from that immutable SHA.
3. Run `iOS/copy-bindings.sh`, `iOS/assert-generated-path.sh`, and
   `scripts/verify_ios_bindings.sh`; then run the app build and full XCTest
   suite.
4. Run the Windows P6 FFI surface snapshot and Kotlin/Android build against the
   same API SHA.
5. Commit all generated Swift/C/module-map outputs together. A partial
   generated diff is a hard failure.
6. CI runs binding verification before Xcode and does not use
   `continue-on-error`.
7. A launch smoke must reach core initialization. Compile-only success cannot
   catch a UniFFI checksum trap.

---

## 3. Verification and evidence standards

### 3.1 Universal evidence rules

- Every result names commit SHA, platform/host, exact command, UTC time, exit
  code, and artifact/log location.
- A behavior is proven only at the layer that matters. API 200, queued dial,
  transport ACK, and UI animation are not message delivery.
- Build provenance must match across peers before any interoperability run.
- Use unique message IDs and payload hashes to correlate sender, relay,
  receiver, receipt, persistence, and UI evidence.
- State survives a restart and remains exactly once.
- Every positive path has a negative control: disconnect or black-hole the
  route and confirm the UI remains queued/unconfirmed rather than lying.
- Two reproducible cold-start passes are required for release-gating device or
  farm scenarios.
- Evidence is sanitized. Never commit credentials, private keys, device
  serials, provisioning data, or personal message content.

### 3.2 What counts as connection, delivery, and receipt

| Claim | Minimum evidence | Non-evidence |
|---|---|---|
| Connected | `ConnectionEstablished` for the intended peer/address and correct outbound/inbound role, plus matching provenance | dial queued, discovery count, socket-open attempt, generic "connected" UI |
| Delivered | Receiver decrypts the unique envelope and durably stores exactly one history row | transport ACK, relay accepted custody, HTTP success, sender-side log only |
| Receipt round trip | Receiver creates a receipt for that message ID; sender core classifies it; platform callback updates history; pending retry is removed; Delivered appears only then | locally calling `markDelivered`, synthetic callback alone, receipt parse test only |
| Recovered | After forced disconnect/restart, queued/custody state drains unattended and converges without duplicates | manual redial, manual DB edit, restarting until it happens |

If current logs do not expose connection direction or the intended address,
add narrowly scoped structured diagnostics before claiming the gate. Do not
infer outbound establishment from a successful command reply.

### 3.3 Per-PR gate matrix

| Change class | Required gate before merge |
|---|---|
| Documentation/tooling only | rules check, `git diff --check`, link/path validation, factual review against current tree |
| Rust outside protected paths | Windows fmt, clippy with warnings denied, workspace build/test/compile, focused tests |
| `core/src/crypto`, `transport`, `routing`, or `privacy` | All Rust gates plus independent adversarial verdict and release-gatekeeper review |
| UDL/FFI/public API | Rust gates, P6 snapshot, regenerated Kotlin/Swift, Android build/tests, Mac binding drift/build/XCTest |
| Android/Kotlin behavior | Windows Gradle unit/lint/assemble gates plus emulator/device behavior evidence |
| iOS Swift/project | Mac binding check as applicable, app build, full XCTest scheme, and physical evidence for radios/background |
| Docker/farm/relay | Config/build validation, immutable image digest, health/readiness, authentic contact/send/receipt test, resource and failure metrics |
| Cross-platform delivery | Both platform build gates, matching provenance, and the complete delivery/receipt protocol |

GitHub Actions supplements the authoritative Windows and Mac gates. A
cancelled, skipped, or `continue-on-error` job is not a pass.

### 3.4 Version exit gates

v0.4.0-alpha.1 requires all of Section 1.5.

v0.5.0 additionally requires:

- 12-node S1-S6 simulated matrix with real application delivery, receipts,
  custody, failure injection, and soak metrics.
- Current-SHA Android and iOS state-machine conformance tests.
- Current-SHA Mac app build and complete XCTest pass.
- Physical iOS-to-Android plus iOS-to-iOS evidence for every applicable farm
  pillar, with unsupported cells recorded rather than simulated.
- Blocking binding-drift and FFI snapshot checks.
- Independent reviews for every protected core change.

---

## 4. Top five "looks done but is not" risks

### Risk 1 -- queued dial reported as a connection

A reply channel or dial queue can return success before the target connection
exists, and an inbound simultaneous connection can satisfy the wrong pending
outbound attempt. This ships a client that says connected but cannot create the
required NAT path.

Killer verification: target-scoped `ConnectionEstablished` with endpoint role
and address, then a unique payload plus receipt over that connection. Run the
simultaneous-open case explicitly.

### Risk 2 -- transport success or malformed receipt becomes Delivered

A transport ACK proves only that one carrier accepted bytes. Receipt enum
fallbacks and platform state machines can mark Delivered, clear retry, or emit
conflicting Failed/Delivered UI events without an authentic delivery receipt.

Killer verification: one ID traced receiver-decrypt -> receipt-create ->
sender-classify -> history/outbox/UI. Inject Sent, Delivered, Read, Failed,
duplicate, malformed, and out-of-order receipts and assert the exact contract.

### Risk 3 -- green compile hides an incomplete or regressed security fix

Stage 1b passed `cargo check` while introducing same-instance lost updates and
non-atomic JSON corruption. A subset packet can also leave F2/F3/F6/F12 open
while the branch prose says "seeding fixed."

Killer verification: race and crash/persistence tests, sabotage-and-restore for
each remediation, full Windows gates, and an independent final verdict naming
every original and newly introduced finding.

### Risk 4 -- tested source differs from the installed artifacts

The CLI, APK, relay container, or generated bindings can come from different
commits. Everything can pass independently while interoperability fails or iOS
traps on a checksum mismatch.

Killer verification: compare build provenance/image digest before testing,
install only checksummed release-candidate artifacts, run a core-init smoke,
and retain the artifact hashes in the evidence manifest.

### Risk 5 -- version merge creates the wrong release before the gates

The current auto-tag workflow turns a main version change into a tag
immediately. The planned `0.4.0` bump would create stable `v0.4.0` before the
operator's locked `v0.4.0-alpha.1` decision and could publish artifacts from an
ungated SHA.

Killer verification: keep the version bump terminal, resolve auto-tag behavior
in the same reviewed release packet, inspect the exact ref before push, and
have the operator verify the prerelease tag and artifact SHA after creation.

---

## 5. GPT budget allocation

Planned GPT use: 40% of the remaining weekly window. Contingency: 10%.
Total reserved/allocated: 50%.

| Allocation | Mode | Work |
|---|---|---|
| 10% | xhigh | Final ledger-seeding remediation re-review, including races, persistence, SSRF/disclosure, and simultaneous-open semantics |
| 8% | xhigh | Pre-tag delivery-truth sweep over the exact release candidate and evidence manifest |
| 8% | xhigh | v0.5 P2-00 re-cut plus B1/B2 DNS/bootstrap contract review |
| 7% | xhigh | Cross-platform receipt state-machine and bindings-ratchet review; Swift implementation stays in the Mac lane |
| 7% | xhigh, held | Meeting Mode D1 design only after the v0.5 delivery/rig critical path clears |
| 10% | untouched contingency | Repeat NO-SHIP remediation, a binding/checksum contradiction, or a farm-rig evidence contradiction |

GPT review-only, not implementation: A4/F2 conclusions, farm-sim evidence
verdict, and final v0.5 scope audit.

Do not spend GPT implementation tokens on deterministic contact provisioning,
Docker/compose mechanics, routine G1/G2 platform work, H1 verification, U7,
generic build triage, or ordinary ticket movement. Those belong to the paid
Qwen/Windows lane. Do not spend GPT tokens re-reviewing an unchanged staging
tip; the local ref watcher should wake this task only after a remote change.

## Final handoff

Windows orchestrator:

1. Accept or amend the sequencing correction for the version/tag workflow.
2. Expand the seeding terminal checklist to explicitly cover every
   operator-mandated open finding, not only the current staged subset.
3. Finish the v0.4 gates and evidence in Section 1 without importing v0.5
   scope.
4. Immediately after `v0.4.0-alpha.1`, dispatch V050-P2-00 and promote the
   prepared iOS I0/I1 lane.
5. Merge and tag only from the authoritative Windows/operator lane.
