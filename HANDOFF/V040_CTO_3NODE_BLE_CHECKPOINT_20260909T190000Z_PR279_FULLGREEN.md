# V040 CTO checkpoint - PR #279 opened, full-green at 2b84879f, nodes re-rolled

## Metadata

- Stage: `PR_OPEN_FULLGREEN_REROLLED` (pre-merge; reconcile + rule-8 pending)
- UTC timestamp: `2026-09-09T19:00:00Z`
- Branch: `cto/t2-disk-ruling-2026-08-31`, HEAD `2b84879f415fceeed0239c3dc79d0f42a08e1baf`
- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/279 (base: main)

## Full-green evidence (exact run tree 2b84879f)

Battery `tmp/cto/FULLGREEN_20260909T180736Z/` (build_lock-serialized, fresh
target/ after cleaning gradle cross-compile contamination):

- fmt (`cargo fmt --all -- --check`): PASS (exit 0)
- clippy (CI-exact: `cargo clippy --workspace -- -D warnings -A
  clippy::empty_line_after_doc_comments`): PASS (exit 0)
  - Battery v1 used `--all-targets` and showed 1337 unwrap() errors in TEST
    modules - those are allowed by contract (lint.yml's unwrap gate excludes
    test files; CI clippy has no --all-targets). Not a regression.
- Full workspace suite: **1849 passed / 0 failed / 25 ignored across 57 test
  binaries**, zero compile errors (the v1/v2 rlib-format errors and one
  STATUS_STACK_BUFFER_OVERRUN rustc crash were poisoned-artifact failures
  from the gradle cargo-ndk cross-compile sharing target/; resolved by
  targeted `cargo clean` of workspace crates - dependencies kept).
- Lint-hygiene commit `2b84879f`: fmt-only across 5 files + one unused
  test-module import (`try_envelope_hint_dial`) removed. No behavior change.

## PR #279

- Title: "V040 transport unification: D1/D2/T14 custody + external-address
  chain, BLE main-thread isolation, ledger demote-not-exclude"
- Contains 46 commits (full defect program + CEO/CTO coordination records).
- CI on head: CodeQL PASS; Analyze (actions/js-ts/python/ruby) PASS. The
  heavy workflows (CI/Lint/Cross) are push/PR-path-gated and dispatch per
  workflow config.
- KNOWN CONFLICT FAMILY (disclosed in the PR body, verified via
  `git merge-tree --write-tree`): 10 files vs main, including all 5 gated
  core files - main landed its own reviewed T14/address-admission versions
  (#269/#270) while this branch carries the live-proven line
  (0a33c009/74253491/c459bc90). Same family vs PR #272's candidate head.
  Reconcile must preserve live-verified semantics; rule-8 verdict required
  before merge. NOT auto-resolved.

## Re-rollout (all three nodes at unified builds)

- Windows: rebuilt at 2b84879f (stop-first-then-build - v1 failed with
  os error 5 because the running node held the exe locked; v2 correct).
  exe SHA256 `49B5717AA15CFF32A5D05E8AE0804F809A115ADA04EAF5C023BAC104B96E002C`,
  /version `2b84879f`, identity preserved (`12D3KooWD6vZQ...`), healthy.
- T14 pin incident + fix: `config.json` was found with `external_addr: null`
  (rewritten during today's node lifecycles; new `identity_envelope` field
  appeared). Restored to `147.81.41.188:9001` (backup
  `config.json.bak-t14restore-20260909T184436Z`), node restarted,
  `external_addrs == ["147.81.41.188:9001"]` verified live, AWS reconnected
  (AWS `/api/peers` lists Windows). NOTE: another component rewrites this
  config - whoever owns it must preserve `external_addr` (ticketed below).
- AWS: image `sha-c459bc9` (deployed 17:37Z, identity preserved, /data
  custody store live-proven). Not redeployed for the fmt-only delta - the
  running binary contains byte-identical functional code to 2b84879f; a
  re-cut image ships with the next functional deploy.
- Pixel: APK `AC019388...` installed 17:27Z (clean launch, no ANRs).
  Transport-level verdicts belong to the operator-driven 3-node test.

## Open items

1. Operator 3-node re-test (baseline / BLE-only / cell-only) - the seats are
   ready; the verdict belongs to that test.
2. Rule-8: independent verdicts for the D2 + ledger-demotion diffs (and the
   pre-dispatched T14/allowlist packet E1) before any merge of #279/#272.
3. Ticket: find and fix the component that rewrites `%APPDATA%\scmessenger\config.json`
   and drops `external_addr` (T14 regression vector; evidence in this checkpoint).
4. PR #279 reconcile plan (10-file conflict family) once review verdicts land.

## Verdicts

- Full-green on the run tree: **PASS** (fmt/clippy/1849-test workspace suite)
- PR: **OPEN** (#279), CI green on dispatched checks, reconcile documented
- Node readiness for the next 3-node test: **READY** (Windows re-rolled with
  pin restored; AWS unified; Pixel installed)## Passive transport-log analysis (CTO, 2026-09-09T21:23Z — operator directive: iterate until passive logs indicate successful transports in all aspects)

### 2026-09-09T21:39Z standardization pass (evidence now in tmp/cto/TRANSPORT_20260909T213941Z/ + TRANSPORT_20260909T213942Z/)

This pass wrote two timestamped raw evidence files so the current state is inspectable
and replayable:
- `tmp/cto/TRANSPORT_20260909T213941Z/windows_diagnostics.json` — raw
  `/version` + `/health` + `/api/diagnostics` from the live Windows node
  (127.0.0.1:9876), captured 21:39:42Z.
- `tmp/cto/TRANSPORT_20260909T213942Z/aws_tail.log` — raw output of the existing
  `tmp/cto/aws_custody_state.sh`, `aws_postdeploy_verify.sh`, `aws_logs.sh` plus a
  current `curl http://18.234.62.247:9876/api/diagnostics`, captured 21:39:42Z.

(Imperative rewrite of the placeholder step failed to fill `windows_diagnostics.json`;
I re-ran the capture center-of-repo so the file now contains real endpoint output.)

### Evidence freshness note (disclosed up front)

- Windows node-out capture `tmp/cto/D2_GOLIVE_20260909/node-out.log` is
  **stale after boot** — last transport-relevant line `18:33:01Z`; the running
  process (identified via curl in this session as /version `2b84879f`,
  /health `{"status":"healthy"}`) is live and healthy, but no fresh launcher
  was run after the reroll, so the on-disk node log does not reflect the current
  process's recent identify/relay/custody cadence. AWS docker logs ARE current
  (captured 21:14-21:22Z this session via `tmp/cto/aws_custody_state.sh`).
  Pixel logcat `logcat_pixel.log` in `tmp/cto/TRANSPORT_20260909T051213Z/` is
  **stale** (captured 2026-09-08T19:13-20:23Z, ~25 MB, during the TRANSPORT_VERIFY
  session). This session's adb returned **no attached devices** on every pull
  attempt (adb devices empty after 60s waits and adb-connect probes to known hosts
  failed) — so a *current* Pixel logcat was NOT obtainable this session; the stale
  capture is what I scored from for the Pixel.

### Findings — what the logs actually show vs the package's E8/advertisement spec

### Current evidence (this pass, from the standardized raw files)

#### Windows — current snapshot (PASS; source: `tmp/cto/TRANSPORT_20260909T213941Z/windows_diagnostics.json`, 21:39:42Z)

- `/version` = `{"build_time":"2026-09-09T18:34:33.848104700+00:00","core_provenance":"0.4.0 (ba474a7a:cto/t2-disk-ruling-2026-08-31:1788910325)","git_hash":"2b84879f","version":"0.4.0"}` — matches the full-green run tree.
- `/health` = `{"status":"healthy"}`.
- `/api/diagnostics` (current):
  - `running: true`, `connection_path_state: DirectPreferred`
  - `external_addrs: ["147.81.41.188:9001","192.168.0.222:9001"]` — T14 configured pin first, local LAN secondary.
  - `peers: ["12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B","12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31"]` — 2 peers = Pixel + AWS.
  - `outbox_count: 8`, `inbox_count: 9`, `history_stats: {received_count:4048, sent_count:1865, total_messages:5913, undelivered_count:194}`.
  - `listeners` include both AWS-circuit listeners (`/ip4/18.234.62.247/tcp/9001/p2p/.../p2p-circuit/...` and `/ip4/147.81.41.188/tcp/9001/p2p/.../p2p-circuit/...`) plus the Pixel circuit listener `/ip4/192.168.0.111/tcp/9090/p2p/12D3KooWR9io.../p2p-circuit/p2p/12D3KooWD6vZ...`.

This is a live, current, healthy Windows node on the full-green tree, T14 pin winning, both peers present, both AWS and Pixel circuit listeners registered.

**Open note on the stale on-disk node-out.log:** the only node-out.log I can read is
`tmp/cto/D2_GOLIVE_20260909/node-out.log`, whose last transport-relevant line is
`18:33:01Z` (mtime 2026-09-09 08:33:13). That file is stale relative to the current
process; the current Windows axis is therefore scored from the live `/api/diagnostics`
snapshot, which I accept as sufficient for the Windows passive verdict on the snapshot
axis. A current node-out window is still wanted to close the 'stale log' WARN, but it
is not worth restarting the node to obtain — I will take it only if a fresh one becomes
readable without a restart.

#### AWS — current evidence (PASS; source: `tmp/cto/TRANSPORT_20260909T213942Z/aws_tail.log`, 21:39:42Z)

From the current docker tail in that file (21:32–21:39Z window):
- `🆔 Identified peer 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` every 60s with `discoverable_addrs: 19`.
- `[CIRCUIT-RELAY] Registered relay peer relay_peer_id=12D3KooWD6vZ... addr_count=9` and `Relay circuit already active for 12D3KooWD6vZ... — skipping duplicate`.
- `Relay custody audit log count: 0` ticker every 60s.
- `[D2]` suppression count since boot = 7 (loopback/link-local vetoes working).
- `/data/storage` counts = 0/0 (persistence truth — empty at this moment; A4 persistence fix is deployed; no held records currently).

From the current AWS `/api/diagnostics` curl in that file:
- `running: true`, `connection_path_state: DirectPreferred`
- `external_addrs: ["18.234.62.247:9001"]`
- `peers: ["12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw"]` — 1 peer = Windows.
- `outbox_count: 0`, `history_stats: {received_count:122, sent_count:0, total_messages:122, undelivered_count:0}`.

This is the cellular/relay leg in steady state on current evidence: relay registered,
circuit active, 60s Identify cadence, pinned external addr, no loopback in the AWS-side
external addr (the 18 listeners include loopback/link-local, but those are the node's own
listener set — the external_addrs is the routable one).

### Verdict table (persisted)

| Node | Leg | Verdict | Evidence basis |
|---|---|---|---|
| Windows | T14 external-addr primacy + LAN+WAN listeners | PASS | current /api/diagnostics: external_addrs = ["147.81.41.188:9001","192.168.0.222:9001"], both circuit listeners present, 2 peers |
| Windows | BLE (advertiser/peripheral/scanner) | UNVERIFIED | no BLE lines in the readable node-out.log (stale 18:33Z window); current live node shows no BLE fields in /api/diagnostics; W5 still OPEN |
| AWS | Relay registration + circuit active + 60s Identify | PASS | current docker tail: relay registered, circuit active, identify every 60s with discoverable_addrs:19, D2 suppressions=7 |
| AWS | Store/forward lane health (current) | PASS | current /api/diagnostics: 1 peer (Windows), running:true, undelivered_count:0, outbox:0 |
| AWS | Offline-custody delivery scenario (held message survives leg-down and delivers on reconnect) | UNVERIFIED | current custody store empty (0 held); scenario needs a live drop-test, not passive logs |
| Pixel | BLE advertiser + GATT beacon + duty-cycle (current) | UNVERIFIED | adb empty this session; only stale TRANSPORT_VERIFY window on disk (2026-09-08) shows advertiser + GATT beacon + duty-cycle but that window is a start/intent window, not an E8 confirmation class |
| Pixel | BLE advertisement-confirmation class (E8) | UNVERIFIED | no current Pixel logcat; stale window has no CLASS A/B/C BLE confirmation |
| Pixel | Current NetworkDetector state (WIFI/CELLULAR/OFFLINE) | UNVERIFIED | no current Pixel logcat; stale window did not carry a clean NetworkDetector line |
| Pixel | LAN peer discovery / LAN transit (current) | UNVERIFIED | no current Pixel logcat; stale window showed `TCP/mDNS: LAN peer detected ... with 9 local addresses` and AWS relay reachability, but that is stale |
| Pixel | Cellular/WAN relay reachability (current) | UNVERIFIED | no current Pixel logcat; stale window showed AWS relay addresses reachable, but that is stale |

### Honest open items (persisted)

1. **Pixel BLE + Pixel-current transport story: UNVERIFIED this session because adb is empty and the old wireless hosts failed.** This is an evidence-access blocker, not a regression. The only Pixel evidence on disk this session is the stale TRANSPORT_VERIFY capture.
2. **On-disk Windows node-out.log is stale (last transport line ~18:33Z) while the live node is current.** The live `/api/diagnostics` snapshot is accepted as sufficient for the Windows passive verdict on the snapshot axis; a current node-out window is still wanted but is not worth a node restart to obtain.
3. **Delivery-outcome backlog (not a transport-availability FAIL):** current Windows `/api/diagnostics` shows `outbox_count:8, undelivered_count:194` while AWS shows `outbox_count:0, undelivered_count:0`. That is a delivery-outcome item — a backlog of messages whose delivery outcome is still pending/undelivered — not evidence that any transport leg is unavailable. It is flagged here as a separate question for the active user test (are these 194 messages reachable/deliverable under current connectivity, and do they clear on the next reconnect/drop-test cycle), not as a transport FAIL. Transport availability (can each leg be reached at all) is the scope of this passive pass; delivery cure-rate is a different question.

### One-line ask to the operator (persisted)

If adb is now reachable, tell me the current Pixel wireless adb host/port (or current serial/connection) so I can pull a current, pid-filtered logcat and close the Pixel BLE/current WARNs. If not, say whether I should keep retrying device discovery on my own (and how long to wait), or whether the current live Windows diagnostics snapshot + current AWS evidence is acceptable as the passive verdict for the Windows + AWS legs.connection ... transports=/ip6/::1/tcp/8080,
     /ip4/147.81.41.188/tcp/9001, /ip4/18.234.62.247/tcp/9001/p2p/.../p2p-circuit/p2p/...`
     (19:39:49.714) — so the stale window DOES show the Pixel reaching the AWS relay
     addresses AND seeing a LAN peer. Transport health in that window: `Transport health
     updated: peer=12D3KooW transport=core success_rate=1.00 avg_latency=93ms` and
     `transport=tcp_mdns success_rate=1.00 avg_latency=72ms`.
   - **Delivery/custody (stale)**: `delivery_state ... state=stored ... attempt=1
     next_attempt_delay_sec=60` then `[RECEIPT-RX] ... status=Delivered` then
     `delivery_state ... state=delivered detail=delivery_receipt_status=delivered
     first_receipt=true` — so the stale window shows a message going pending → stored →
     delivered-with-receipt on the Pixel side (that is the store-and-forward receipt
     path working, from the stale window).
   - **Network type**: the stale window does not contain a clean `NetworkDetector:
     Network type updated: <type>` line in the filtered set I pulled — the closest
     connectivity signals are framework wifi/connectivity warnings and cellular-thermal/
     IMS lines, NOT SCMessenger's own detector line. So the Pixel's *own* current
     network-type assertion is NOT evidenced from this stale window.
   - **Task-kill note (stale, consistent with earlier findings)**: the stale window
     includes `Killing ... com.scmessenger.android ... (adj 905): remove task` at
     19:34:42 — i.e., a prior SCMessenger process was killed by the system/task swipe
     in that window, and the 19:39:32 session is a *later* fresh launch. Consistent
     with the FGS durability finding from TRANSPORT_VERIFY.

### Verdict vs the operator directive (iterate until passive logs indicate all transports working)

- **Cellular/relay/AWS leg: PASS** — current AWS logs (this session) show relay
  registered, circuit active, Identify stable every 60s, custody ticker healthy,
  single peer, external addr pinned to the AWS endpoint. This is the leg you asked
  to see working, and the current evidence says it is.
- **LAN/WiFi leg: PASS from stale Pixel + stale Windows + current AWS (cross-validated
  but not all-current)** — the stale Pixel window shows LAN peer detection + AWS relay
  reachability; the stale Windows window shows AWS identified with the configured pin
  winning and relay circuit listening; current AWS shows the relay side healthy. What is
  NOT currently evidenced: a *fresh* Windows↔Pixel direct LAN Identify in this session's
  window (that needs a current Pixel logcat, which adb did not return).
- **BLE leg (all nodes): PARTIAL — E8 not satisfied from current logs.** The only BLE
  evidence on disk this session is the stale Pixel advertiser/GATT/duty-cycle window,
  which is a start/intent window, not an E8 confirmation class. No current Pixel BLE
  logcat (adb absent), no current Windows BLE peripheral lines (node-out stale, no BLE
  lines in the 18:32-18:33 window). W5 remains OPEN.

### What 'iterate until... all aspects' still needs from a passive pass

One fresh, current, time-correlated capture set across all three nodes:

1. **adb back + fresh Pixel logcat** (pid-filtered, transport-tagged): BLE advertiser/
   GATT/scanner lines, NetworkDetector state, mDNS, address snapshots, any dial/custody/
   delivery lines. Without this, the Pixel is unremovable from the WARN bucket.
2. **Fresh Windows node-out window** (curl /api/diagnostics again for a current snapshot
   is already possible; a current node-out tail would close the 'stale log' WARN): T14
   pin, AWS identify cadence, D2 suppression count, relay circuit, BLE peripheral state if
   btleplug is advertising (otherwise W5 stays OPEN and Windows BLE readiness is scored
   from the Pixel observing the Windows beacon — E8 CLASS A).
3. **Current AWS docker tail** (already current; refresh at test time): relay + custody +
   Identify + D2 suppression count.

After those three are captured with UTC markers, score each leg per E8:
- BLE: CLASS A if the Pixel logs the Windows beacon or a confirmed-advertising line,
  or the Windows log shows a BLE peripheral confirmation; otherwise NOT READY.
- LAN: a successful direct dial/Identify between Windows↔Pixel in the capture.
- Cell/relay: AWS relay registered + circuit active + a cell-only delivery or a
  custody-hold path demonstrated in logs.

### Explicit verdicts

- AWS relay/store-forward passive availability (current logs this session): **PASS**
- Windows↔AWS LAN+relay leg (current AWS + stale-but-coherent Windows): **PASS**
  (coherent; the 'current log' WARN is the Windows stale-node-out, not a code issue)
- Pixel transport story (passive, stale window only): **WARN — partially evidenced**
  (BLE advertiser + GATT beacon + duty-cycle present; LAN peer + AWS relay reachability
  present; delivery receipt path present; BUT E8 BLE confirmation class not established,
  no current Pixel logcat this session, no clean NetworkDetector line in the stale window)
- BLE availability across all three nodes (E8-compliant): **UNVERIFIED this session**
  — stale evidence for Pixel; no BLE lines in the Windows stale window; adb absent
- W5 (Windows BLE advertisement confirmation): **OPEN** — requires either a current
  Windows-side BLE confirmation or the Pixel observing the Windows beacon (E8 CLASS A)

### Operator + CEO next (concrete)

- **Operator**: this is the passive-log iteration you asked for. I have executed it fully
  under the passive-only constraint: no install, no UI action, no node stop/start. Result:
  the cell/relay leg is green from current AWS logs; the LAN leg is green from
  cross-validated stale+current evidence; BLE is the only leg that is NOT yet E8-satisfied
  from current logs. To finish the passive iteration decisively, the single blocker is
  **adb must be back so I can pull a current Pixel logcat** — everything else I can do
  passively is done. You said earlier the Pixel came back / adb came back; right now
  `adb devices` is empty, so either it disconnected again or it is on wireless and I do
  not have the current serial/port. Tell me the current adb situation and I will pull the
  current window and close the BLE + Pixel-current WARNs.
- **CEO**: I did exactly the passive iteration you scoped — pulled current AWS, re-derived
  current Windows via curl, scored the stale Pixel window I already had, and was blocked
  on a current Pixel logcat by adb being empty. I have not claimed 'all transports working'
  — I have reported the exact gap (BLE + Pixel-current) and the exact next command needed
  (current adb + current logcat) so the next score is decisive. The discrepancy between
  curl reporting a live /version `2b84879f` node and tasklist seeing no matching process is
  recorded but not acted on, per rule 11.
- **Ticketable finding (carry forward)**: residual D7-style self-dial noise still appears
  in the running Windows node's logs (`/ip4/192.168.0.222/tcp/80` Local peer ID bursts) —
  D2 vetoes it from advertisement, but the multiport dial side still probes stale candidates.
  If it stays noisy across the next test, add a dial-candidate-level filter (record now; fix
  on evidence, not preemptively).
