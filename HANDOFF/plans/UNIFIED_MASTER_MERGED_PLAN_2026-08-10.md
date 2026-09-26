# Unified master merged plan -- PR #139 field gate + Android transport lane

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Written: 2026-08-10 (HST)
Supersedes: nothing. **Reconciles** three existing documents; it does not
replace any of them.

## 0. What this document is

The operator asked for one reconciled plan so the Windows lane, the GPT-MAC
lane, and this Android lane cannot drift. This file maps the three existing
authorities onto one ownership table and records where they disagree.

| Source | Role | Precedence |
|---|---|---|
| `HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md` | **What/acceptance authority.** Pre-freeze scope PF-1..PF-12, delivery philosophy, G1-G6 semantics, GO/NO-GO. | Highest for acceptance semantics |
| `docs/ORCHESTRATION.md` + `AGENTS.md` | **How/delegation authority.** Lanes, worker contract, security gates. | Highest for delegation |
| `HANDOFF/plans/FIVE_NODE_UNIFIED_TEST_PLAN_2026-08-09.md` | Readiness gate, node inventory, per-node prep, scoring protocol. | Operational detail |
| `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-10_WINDOWS_LANE.md` | Current live state: transport contract, access, blockers, gotchas. | Newest situational truth |

Where the field-gate reference and the older test plan conflict, the reference
wins (it says so explicitly, and it is dated later). Where the takeover packet
describes *current machine state*, it wins over both, because state changes
faster than plans.

## 1. Reconciliation: conflicts found and resolved

### 1.1 "Must run twice" vs "two matrices + one soak"

The PR #139 body and `V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` say the gate
must "run twice". The field-gate reference Section 1.1 locks a stricter bar:
**two complete G1-G6 matrix passes plus one continuous 60-minute five-node
soak**, on a hard-frozen runtime SHA. **The reference wins.**

### 1.2 AWS node role

The older plan treats AWS as a fifth peer. The reference Section 5.0 scores it
as **headless infrastructure only** -- relay/rendezvous/custody -- and excludes
it from the G1 pairwise user-endpoint matrix (six pairs, twelve directional
flows across the four user endpoints). **The reference wins**, and it is
explicit that this is a test-role definition, not a change to product doctrine.

### 1.3 Anchor drift

- Field-gate reference records PR #139 head `e5284b7b`, last runtime candidate
  `7e527df0`, **freeze NOT declared**.
- Takeover packet says the running anchor is `68fcc3f1`.
- Verified this session: `68fcc3f1` is the merge commit of PR #144, contained in
  `tracking/pre-v040-tag-work` and `codex/pr139-five-node-gate-fixes`, **not on
  `main`**. This is intentional per the test plan ("land PR #144 into the PR
  branch").
- The installed Pixel APK (versionCode 14, `lastUpdateTime` 2026-08-09 16:00
  HST = 2026-08-10T02:00Z) matches `68fcc3f1`. `MdnsServiceDiscovery.kt` and
  `MeshRepository.kt` are byte-identical between `68fcc3f1` and `e5284b7b`.

**Resolution:** no runtime freeze is in effect. This Android lane's work
therefore lands *before* freeze, as pre-freeze scope, which is the correct
side of the gate.

### 1.4 `main` vs PR #139 divergence

The reference Section 2.2.A forbids merging `main` wholesale into the PR
branch. Each `main`-only finding must be classified. Section 3 below does that
for every finding this lane touches.

## 2. Android lane findings, classified against pre-freeze scope

Evidence base: full logcat, `files/mesh_diagnostics.log*`, and the 12 MB Rust
core JSON log pulled read-only from the Pixel 6a, split at the 2026-08-10T02:00Z
reinstall so only current-build behaviour is scored.

| ID | Finding | Disposition | Gate item | Owner |
|---|---|---|---|---|
| A1 | `onServiceLost` lacks a self-peer guard; 88/88 ratchet resets targeted the **local** peer id | **STILL OPEN -> fixed this lane** | PF-11 / G3 | Android lane |
| A2 | Four terminal paths abandon accepted undelivered messages; one marks them *corrupted* | **STILL OPEN -> fixed this lane** | **PF-1, PF-12** | Android lane |
| A3 | `routing_peer_seen` has zero callers; 216/216 decisions StoreAndCarry @ confidence 0.0 | **STILL OPEN -> analysis dispatched** | G2 / PF-10 | Android lane (AUDIT-GATE) |
| A5 | 123 `wire envelope: unexpected end of file` on current build | **STILL OPEN -> analysis pending** | G3 | Android lane |
| -- | mDNS fabricated `mdns-*` peer ids (2,404 phantom identities, 2,317-reset storm) | **ALREADY FIXED + REVERIFIED** -- zero occurrences post-`68fcc3f1` | PF-11 | closed by PR #144 |
| -- | Outbound receipt never clears sender state | **SUPERSEDED** -- Android symptom of `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE` | PF-2 | **Windows lane** |
| -- | 2,814 dials to tcp/80 | **SUPERSEDED** -- `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT`; Android is plain-TCP on 80 only | PF-10 | **Windows lane** |
| -- | 1,636 dial failures to `12D3KooWJUJ1ko` | **SUPERSEDED** -- that is the roaming iPhone; `P0_NO_RELAY_FALLBACK_FOR_ROAMING_PEERS` | PF-4 | **Windows lane** |
| -- | Malformed `/p2p/<peer>/p2p-circuit` (missing destination) | **SUPERSEDED** -- `P1_NESTED_CIRCUIT_ADDRESSES_STILL_FORMED` | PF-10 | **Windows lane** |
| -- | LAN dial failures to 192.168.0.121/.136 | **MEASUREMENT ARTEFACT** -- those hosts' ports were closed; the CLI node was down | n/a | not a defect |

Four findings that looked like Android defects are owned by other lanes. Filing
them again here would have duplicated work -- exactly what Section 2.2.A warns
against.

## 3. Ownership boundary (the thing that keeps lanes from colliding)

**This Android lane owns:** `android/app/src/main/**`, plus the analysis for
A3/A5. It does **not** touch `core/src/iron_core.rs` receipt handling,
`core/src/transport/` dial ordering, relay candidate construction, or address
filtering.

**Windows lane owns:** PF-2 receipt convergence, PF-3 request-response panic,
PF-4 relay fallback, PF-10 candidate ordering, and verification of GPT-MAC's
`4083e59b`.

**GPT-MAC lane owns:** nodes 4 and 5 (macOS CLI, physical iPhone), and its own
`4083e59b` claims until the Windows lane verifies them.

Android must behave correctly **whether or not a receipt ever arrives**. That
is the contract that lets A2 land without waiting on PF-2.

## 4. Sequence for this lane

1. A1 self-guard fix -- **applied**, pending gate + unit test.
2. A2 durable-delivery rework -- design panel first (Section 10 of
   `docs/ORCHESTRATION.md` requires a design PASS before the implementation
   packet), then implementation, then the **delivery gate** (Fusion Lite
   3-panel or 3 distinct verifier dispatches) because it touches
   outbox/receipt/retry.
3. A3 -- analysis, then adversarial review at THINK/MAX before any commit
   (`core/src/routing/` is merge-blocked).
4. A5 -- analysis, then fix per whichever verdict the analysis reaches.
5. One consolidated Android build gate, then field re-measure on the Pixel.

## 5. Field re-measure contract (how this lane proves it worked)

Per the reference Section 6.3, a collector must be shown capable of observing
what it scores. Before the next Pixel run:

- `adb shell logcat -G 16M` first. The 256 KiB default already destroyed one
  field test's evidence.
- Wake the device and whitelist it from Doze
  (`dumpsys deviceidle whitelist +com.scmessenger.android`); Doze blocks
  inbound connections while the app looks healthy.
- Capture full multiaddrs including `/p2p-circuit` suffixes; a regex that stops
  at `/tcp/<port>` already produced one false "zero relay attempts" conclusion.
- Do not use `dumpsys activity services` as a readiness check -- it reports no
  ServiceRecord for a mesh service that is demonstrably running.

Acceptance for this lane specifically:

| Finding | Passing measurement |
|---|---|
| A1 | Zero ratchet resets naming the local peer id over a full run |
| A2 | No accepted message removed by age or attempt count; no `markMessageCorrupted` from a delivery-failure path; bounded log volume |
| A3 | At least one `routing_decision` with non-zero confidence while a peer is connected |
| A5 | Truncated-frame count per peer, not per occurrence, and no connection teardown |

Scoring rule inherited from the takeover packet: **score on receiver-side
`inbox_receive` plus receipt, never on sender-side status or transport ACK.**

## 6. Known open risk

The CryptoError count (509 on the current build) has **not** collapsed despite
the receipt-marker work, which the existing ticket predicted would reduce it.
A1 is a strong candidate cause, since wiping the local ratchet session would
corrupt inbound decrypt for every peer. **This must be re-measured after A1
lands, before concluding anything about an independent decrypt defect.** A1
also contaminates any earlier CryptoError measurement taken on this build.
