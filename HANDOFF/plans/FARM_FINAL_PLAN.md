# FARM FINAL PLAN — Puna Seed Deployment (v1.0.0 field target)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active — supersedes nothing; refines the Farm Use Case directive in
`HANDOFF/todo/_QUEUE.md` (2026-07-11) into the definitive deployment plan.
Last updated: 2026-07-13
Author: Claude Fable 5 (native Cowork session), on operator direction (Lucas)
Sequencing authority upstream: `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (Phase 1
COMPLETE per P1-19, 2026-07-10). This plan governs WHAT ships to the farm and
in what order; the queue (`HANDOFF/todo/_QUEUE.md`) remains the live pick list
and was re-ranked 2026-07-13 to match Section 7.

---

## 0. Operator-settled inputs (2026-07-13 session — do not relitigate)

1. **The farm is the seed.** Physical deployment: Hawaii Island, Puna district
   (Pahoa / Kalapana area). ~28 acres, ~12 users. Roughly half the residents
   are on the farm WiFi mesh (distributed APs across acres, one fiber uplink);
   the other half are cellular-only or both. One physical choke point everyone
   drives/walks past. Community meetings of 6-10 people. Regular trips to
   Pahoa/Hilo (town WiFi + cellular). Unreliable internet, power, and finances
   are the adoption driver, not an edge case.
2. **Device mix: half or more iPhone.** iOS parity is therefore IN scope for
   the farm seed. This RESOLVES open decision point 4 in `_QUEUE.md`
   (2026-07-11): the execution plan is amended — iOS is v1.0.0-blocking for
   farm rollout. Consequence: the GitHub billing unlock (decision 3) and
   `TASK_CI_IOS_MACOS_RUNNER_FIX.md` graduate from "nice to have" to
   farm-gating infrastructure.
3. **Relay backbone:** the Windows dev machine runs the farm anchor (24/7 CLI
   relay on the farm fiber, behind the operator-managed firewall), plus AWS
   free tier AND Alibaba Cloud free tier (both fresh) as WAN relays /
   secondary bootstrap. The cloud boxes double as the docker farm-sim rig
   (extends the P1-14/P1-18 rig already approved 2026-07-11 — same rig, not a
   second provisioning effort). A cheap dedicated on-farm box (Pi/mini PC) is
   recommended later so the anchor survives dev-machine reboots [HUMAN,
   hardware, non-blocking].
4. **WAN addressing: DNS-name-first is MANDATORY.** Farm fiber is Hawaiian
   Telcom residential 1 Gbps/600 Mbps with a dynamic-but-stable public IP and
   an existing dynamic-DNS hostname. The system MUST survive an IP flip
   unattended — this is a hard requirement, not an optimization (Section 3,
   AD-2).
5. **Onion routing: seam preserved, not wired.** Regular routing gets
   perfected first. `core/src/privacy/onion.rs` + PQC-09 hybrid onion stay
   compiled and unit-tested with ZERO live call sites until after farm
   stability (Section 3, AD-8). `PQC_09_SECURITY_REVIEW_FIXES.md` stays
   parked, exactly as the queue already records.
6. **WiFi Aware/Direct: kept, deprioritized** — unchanged from the 2026-07-11
   directive (Aware is Android<->Android [BLOCKED-HW]; Direct waived to v1.1).
   Note for the farm's device mix: iPhones have neither API. iOS proximity =
   Multipeer (iOS<->iOS) + BLE (cross-platform). This makes BLE the ONLY
   universal proximity transport at the farm — it gets first-class treatment
   (AD-6).

## 1. Vision and design doctrine (farm seed -> island)

The app must be loved on the farm before it can spread. It spreads by serving
sovereign communication needs that centralized apps fail: no dependence on
any single network, honest delivery status, cryptographic identity owned by
the user, and a mesh that gets STRONGER as more neighbors join.

**Doctrine (each maps to architecture decisions in Section 3):**

- **D1 — Work on whatever network you are on.** A node on farm WiFi, a node
  on cellular in a coffee shop in Hilo, and a node with no connectivity at a
  meeting must all participate. Reach ladder: LAN-direct (mDNS) ->
  anchor-assisted WAN (DNS-named, port-laddered) -> relay circuit ->
  store-and-carry custody. Every rung already exists in code; this plan makes
  the LADDER work as one continuous flow (the P0 in the 2026-07-11 directive).
- **D2 — One ledger, many carriers (the mycorrhizal consciousness).** Every
  discovery surface — mDNS, `ledger_exchange` (peer lists shared on connect,
  `transport/behaviour.rs`), relay `peer_exchange`, Kademlia DHT, identify
  observed-addresses, BLE beacons — feeds ONE superset peer ledger
  (`LedgerManager`, persisted `ledger.json`). Nodes share their ledger gladly
  and unconditionally on every connect. Staleness is NOT distrusted at share
  time; it is handled downstream by dial outcomes feeding the routing engine's
  negative cache and reliability scores. The 3-layer mycorrhizal routing
  engine (local/neighborhood/global cells) consumes this ledger and is already
  live in the production send path (verified 2026-05-18).
- **D3 — Delivery truth.** The UI never claims delivery that is not
  receipt-confirmed, and never claims failure for a message that arrived.
  Both directions of this are broken today (Section 4, WS-FARM-A) and are the
  top-ranked work in the queue.
- **D4 — Degrade and recover without a human.** Power outage, fiber cut, IP
  flip, phone dead for two weeks — the mesh reconverges unattended when
  conditions restore. Section 5's drills make each of these a tested, repeated
  scenario rather than a hope.
- **D5 — The island inherits everything.** Nothing farm-specific is
  hardcoded. Anchors are configuration (a DNS name + a port ladder), not code.
  Hilo users with fiber become voluntary anchors/relays; off-grid Puna users
  lean on custody and proximity. Same binary, same ledger doctrine, N anchors
  instead of one. CORE_BOOTSTRAP_NODES is empty in code today and STAYS empty
  — farm defaults ship as config, so other communities seed their own.

## 2. Farm topology as six engineering scenarios

Each scenario states what must work, the verdict on today's code, and where
the gaps are ledgered (Section 4).

**S1 — Farmhouse cluster (farm WiFi mesh, ~half the residents).**
Nodes on the same WLAN discover via mDNS and talk direct TCP/QUIC. LAN
discovery, dial, and E2E messaging were device-validated in Phase 1
(P1-06/07/09, NEXT_ITER_04 retest); adaptive ports landed (P1-11/12/13).
Multi-AP roaming across acres means a phone hops APs — same L2 network
assumed; if the farm mesh segments into multiple subnets, mDNS won't cross
segments and nodes fall back to the anchor (S2 path). VERDICT: closest to
done. Gaps: outbox flush on reconnect (WS-FARM-A), ledger convergence test
(WS-FARM-F), AP-roam soak drill (FD-1/FD-9).

**S2 — Cellular-only nodes on the farm (~half the residents).**
No LAN path to the farmhouse cluster. They reach farm peers through the
anchor: cellular node dials `dns4/<farm-ddns>/...` -> anchor relays or
custody-holds. Carrier NAT means inbound dialing to these nodes is
impossible; they must maintain an outbound connection (or reservation) to an
anchor. QUIC-on-443 default and WSS-on-443/80 carrier-filter escape already
exist (`relay/client.rs`). VERDICT: architecture present, never live-proven
end-to-end (P1-18 WAN arm is open verification debt). Gaps: WS-FARM-B
(DDNS end-to-end, anchor deployment), FD-2 drill.

**S3 — Town roamer (Pahoa cafe WiFi, Hilo, any foreign network).**
Same mechanism as S2: the roamer's node re-dials the anchor by DNS name from
whatever network it lands on, re-learns current peer state via
ledger_exchange, sends/receives queued messages. Must also work when the farm
IP flipped while the roamer was away (AD-2). VERDICT: same plumbing as S2
plus DNS-first discipline. Gaps: WS-FARM-B, FD-3 drill (a literal
sit-in-a-Pahoa-cafe test).

**S4 — Community meeting (6-10 people, one room, foreground use).**
People quietly message each other DURING the meeting: screens on, app
foregrounded — this is explicitly NOT the background-BLE worst case, which
makes it winnable. Mixed iPhone/Android. Required: any pair of the 6-10
devices exchanges messages within seconds, sustained for a 30+ minute
meeting. Transport reality: BLE GATT/L2CAP is the only universal carrier
(iOS<->iOS pairs may ride Multipeer opportunistically). Android supports
roughly 4-7 concurrent GATT connections depending on chipset; 10 devices
all-pairs needs a connection budget + rotation or relay-through-strongest
strategy — a real design decision (WS-FARM-D design note). BLE MAC rotation
already keyed off identity handshake (P1-16 done). VERDICT: pairwise BLE
proven Android<->Windows; N-way room topology and Android<->iOS BLE at the
farm's mix are unproven. Gaps: WS-FARM-D, WS-FARM-C (iOS), FD-4 drill.

**S5 — Choke-point drive-by (BLE sneakernet).**
Everyone passes one choke point; contact windows are seconds. Drift/DTN
custody with IBLT sync is live in the swarm path; BLE contact-window
efficiency (sync completes in a ~10 s window) has an existing test target in
the backlog (fable5plan T2.2 scenario). VERDICT: opportunistic bonus for
v1.0.0, architecture already supports it; NOT a farm gate (slim BLE window at
driving speed is physics-limited). FD-7 drill is stretch, not gating.

**S6 — Outage mode (power out, fiber cut, or both — Puna reality).**
Fiber cut: farm WiFi keeps working on battery/solar APs -> S1 continues
untouched (mDNS/LAN has zero internet dependency); cellular nodes lose the
anchor but keep cellular-relay via cloud relays (AWS/Alibaba) if configured
as secondary bootstrap. Power out at the anchor: cloud relays hold WAN
custody; LAN continues peer-to-peer. Recovery: anchor comes back, DDNS
updates, custody drains. VERDICT: each piece exists; the DEGRADE-AND-RECOVER
sequence has never been exercised as one flow. Gaps: WS-FARM-B (cloud
relays), WS-FARM-F (custody convergence), FD-5/FD-6 drills.

## 3. Architecture decisions (AD-1 .. AD-8)

**AD-1 — Hub-assisted cooperative mesh (not hub-dependent).**
The farm anchor makes WAN reach deterministic: every node, wherever it is,
can always dial ONE well-known DNS name. AutoNAT + DCUtR + relay-client are
already in the behaviour stack (`transport/behaviour.rs`) and remain enabled
— direct hole-punched connections are an optimization the mesh takes when it
can. But the DESIGN never depends on hole punching succeeding: anchor relay +
custody is the guaranteed floor. On the LAN, the anchor is just another peer;
losing it does not degrade S1 at all.

**AD-2 — DNS-name-first addressing (the IP-flip mandate).**
Every reference to the anchor — bootstrap config on all platforms, ledger
entries, relay registry, peer_exchange records — uses `/dns4/<ddns-host>/...`
multiaddrs, never the raw IP. libp2p `dns` feature is already in the
workspace pin; Android (which excludes DNS from the libp2p transport per
`.claude/rules/rust.md`) resolves via the custom DNS fallback landed
2026-07-10 (`ESC_ANDROID_DNS_RESOLVER_FIX`, Google Public DNS nameservers).
Required behaviors to implement/verify (WS-FARM-B):
- Dial failure on a DNS-named addr triggers re-resolution, not just backoff
  (a flipped IP must not poison the negative cache against the HOSTNAME).
- `LedgerManager` entries for DNS-named peers store the name; observed raw
  IPs are hints, never replacements.
- One integration test simulates a resolution change mid-session and asserts
  reconnection without restart; one live drill (FD-6) forces a real flip.
- iOS dial path verified with dns4 multiaddrs (unknown today — WS-FARM-C).

**AD-3 — Anchor port posture.**
Anchor listens on the laddered ports that already exist in code
(`multiport.rs`, P1-11/12/13 landed): TCP 443 + 80 + fallback, QUIC on
udp/443 (the `relay/client.rs` default), WS/WSS for browser + carrier-filter
escape. Firewall forwards exactly: tcp/443, tcp/80, udp/443 to the anchor.
This matches the "if 443 gets through, use 443" acceptance standard from
P1-14 and gives cellular-carrier-filtered farm-mates their escape hatch.
[HUMAN: Lucas configures the port forwards + DDNS record; runbook in
WS-FARM-B.]

**AD-4 — One-ledger superset merge, with a convergence contract.**
Today: `ledger_exchange` shares peer lists on every connect (automatic since
2026-03-20), `peer_exchange` propagates relay-known addresses, kad supplies
WAN lookup, `LedgerManager` persists and ranks dialable addresses. What is
MISSING is a stated, tested contract: from any connected graph of N nodes,
every node's ledger converges to the superset of live peer records within
one gossip round-trip per hop, and a returning stale node (2 weeks offline)
reconverges on first contact with any one peer. WS-FARM-F adds
`integration_ledger_convergence.rs` encoding exactly that, plus an audit that
all four surfaces actually WRITE into the one ledger (the CLI's promiscuous
bootstrap dialing in `cli/src/bootstrap.rs` is a third bootstrap mechanism —
fold or formally document, see the SC_BOOTSTRAP_NODES finding).

**AD-5 — Delivery truth chain (the two live CRITICALs).**
The farm-killer bugs, both root-caused with fix paths already written:
1. Outbox never flushes on reconnect — Site 2 fixed and live-verified
   (5-11 ms deliveries); Site 1+3 re-scoped in
   `OUTBOX_FLUSH_ON_CONNECT_RETRY.md` with a 95%-complete reference patch
   (`HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch`) and the exact
   failing-test diagnosis. Highest-priority implementation task in the queue.
2. Receipts cannot round-trip — `on_receipt_received` is dead-on-arrival
   (nothing in core classifies incoming receipts) and the CLI's own
   recognition uses bincode against a JSON payload
   (`CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`, definitive
   root cause). Consequence today: Android tells the sender a DELIVERED
   message failed, then deletes it after 12 retries — trust poison for farm
   adoption. Fix = core classification + CLI serde fix + Android retry
   suppression (transport-success is never escalated to "failed", only to
   "sent, unconfirmed").
Plus the single-ownership rule between outbox retry and drift custody
(exactly-one system owns a message at a time) — audit task WS-FARM-A4.

**AD-6 — Meeting Mode is a foreground proximity profile.**
Scope the S4 scenario deliberately: app foregrounded, screens on, 6-10
devices, one room. Design note (WS-FARM-D1, [OPUS+]) settles: connection
budget and rotation policy for Android's concurrent-GATT ceiling; whether
the strongest device (charging/newest) acts as an in-room star hub using the
existing GO-intent-style election logic; Multipeer offload for iOS<->iOS
pairs; gossipsub topic per-room as the fan-out layer once links exist
(behaviour.rs already runs gossipsub in PERMISSIVE mode for topic
auto-negotiation). 1:1 messaging in-room is the v1.0.0 gate; a shared
meeting-room group thread rides the same links and is scoped in the design
note as fast-follow if it threatens the timeline.

**AD-7 — iOS is a farm gate now (device-mix consequence).**
The Swift transport layer already exists (BLECentral/Peripheral/L2CAP,
Multipeer, mDNS, SmartTransportRouter) and the app was code-complete at last
build; what is missing is a BUILD-AND-VERIFY lane: GitHub billing unlock
[HUMAN] -> free macOS runners (public repo) -> `TASK_CI_IOS_MACOS_RUNNER_FIX`
-> single XCFramework/Swift-bindings regen AFTER PQC-10 lands (per the
queue's own note) -> iOS parity verification of exactly the farm pillars:
receipts, outbox flush, dns4 dialing, mDNS LAN, BLE meeting drill.
Distribution [HUMAN decision]: TestFlight needs the USD 99/yr Apple
Developer account; free-account sideloading expires weekly — unacceptable
for farm-mates. Budget the account or scope an alternative before F3 pilot
(Section 6).

**AD-8 — Onion seam freeze (build later without rebuilding).**
The seam already exists as clean module boundaries: `privacy/onion.rs`,
`prepare_onion_message` in iron_core, PQC-09 hybrid onion implemented but
unwired, relay-hop machinery in `relay/`. Freeze contract, enforced by one
new test (WS-FARM-H1): privacy module compiles and its unit tests pass in
every build; ZERO call sites from the live send path into onion construction
(assert by grep-style test or cfg-gated wiring point); the wiring point is
documented (one function, one decision) so v1.1 completion is a wire-up plus
adversarial review, not a redesign. Cover traffic / padding / timing stay
out of the UI (removed 2026-04-20, stays removed until real).

## 4. Gap ledger — all work between today and the farm seed

Tier tags follow the repo convention and route to models per
`docs/ORCHESTRATION.md` / the `/scmorc` routing table:
- [HAIKU] -> haiku(low) native / [FLASH] qwen — mechanical, spec-in-ticket.
- [SONNET] -> sonnet(medium, escalate to high) / [CODER] qwen — standard
  implementation; the DEFAULT for this plan. Most tasks below are
  deliberately specified tightly enough for this tier.
- [OPUS+] -> opus(high) / [THINK|MAX] qwen — design notes, root-cause,
  anything whose failure mode is "confidently wrong".
- [AUDIT] -> mandatory adversarial review (fable(high) native read-only /
  deepseek in ollama swarm) — every diff in crypto/transport/routing/privacy.
- [DEVICE] real hardware/live network; [HUMAN] operator action.

### WS-FARM-A — Delivery truth (P0, blocks everything user-facing)

- **A1** Outbox enqueue-on-disconnect + flush-on-connect with retry.
  Ticket: `OUTBOX_FLUSH_ON_CONNECT_RETRY.md` (re-scoped 2026-07-13, reuse the
  296-line reference patch; the exact failing-test fix is written in the
  ticket). [SONNET/CODER]. Delivery logic — careful review, no formal audit
  gate (store/ tree), verification protocol per ticket.
- **A2** Receipt round-trip: core incoming-message classification fires
  `on_receipt_received`; fix CLI `bincode` vs JSON mismatch (standardize on
  serde_json both ends). Ticket: `CRITICAL_ANDROID_FALSE_DELIVERY...md` steps
  1-2. [SONNET/CODER][AUDIT — touches transport/swarm.rs incoming path].
- **A3** Android retry suppression: transport-success is terminal-ish
  ("sent, unconfirmed"), never escalates to failed/corrupted; widen receipt
  window; Kotlin regression test (mock success, no receipt). Ticket steps
  3-4. [SONNET/CODER], Kotlin-side.
- **A4** Outbox <-> drift custody single-ownership audit: verify (test-first)
  that a message is owned by exactly one queue at a time and a receipt clears
  both; wire the StoreAndCarry -> custody handoff if the trace shows a gap.
  [SONNET(high)], becomes [AUDIT] if fixes land in routing/. New task, cut
  from this plan.
- **A5** iOS parity for A1-A3 semantics once the iOS lane opens (same state
  machine, `MessageStatus`-style honest states). [SONNET], after C-lane.

### WS-FARM-B — Reach every node from anywhere (DDNS + anchors)

- **B1** DNS-name-first hardening: re-resolve on dial failure; negative-cache
  keys must not poison hostnames on IP flip; ledger stores hostnames for
  DNS-named peers; integration test with mid-session resolution change.
  [SONNET(high)][AUDIT — transport/ + routing/]. New task, cut from this plan.
- **B2** Bootstrap unification: wire `SC_BOOTSTRAP_NODES` through the CLI
  `start` path or retire it explicitly; document/fold the CLI promiscuous
  bootstrap (`cli/src/bootstrap.rs`) vs core `BootstrapManager` vs config.json
  into ONE documented precedence. Finding recorded in the outbox CRITICAL
  ticket. [SONNET/CODER].
- **B3** Farm anchor deployment runbook + config: CLI relay on the Windows
  box; listen ladder per AD-3; DDNS multiaddr advertised; autostart on boot;
  `--http-bind` flag + `/health` route (rig prerequisites already flagged in
  the queue 2026-07-11). [SONNET for the flag/health work; HUMAN for firewall
  port-forwards and DDNS record].
- **B4** Cloud relays: deploy CLI relay to AWS free tier + Alibaba free tier
  as secondary bootstrap/relay; both entries ship in farm config after the
  anchor. Doubles as the docker sim rig host (P1-14/P1-18 + the 3-group farm
  topology already scoped in the queue). [SONNET + HUMAN provisioning].
- **B5** P1-14 hostile-network + P1-18 relay/WAN live proof — the standing
  post-exit verification debt, now executed on the B4 rig with the farm
  topology (Group A farmhouse/mDNS, Group B far-field/internet-relay, Group C
  dead-zone/BLE-offline). [SONNET driving][DEVICE].
- **B6** 12-node farm simulation soak on the rig: all six scenarios of
  Section 2 encoded as compose profiles + netem; runs before every farm APK/
  TestFlight push. [SONNET after B4/B5].

### WS-FARM-C — iOS lane (device-mix gate)

- **C1** GitHub billing unlock (or repo transfer into the trial org).
  [HUMAN] — the single cheapest unblock in the whole plan.
- **C2** iOS CI runner fix — the task file lives at
  `HANDOFF/done/TASK_CI_IOS_MACOS_RUNNER_FIX.md` but its own header still
  says TODO (premature done-move pattern this repo has hit before).
  Verify-first whether the ios-build-test.yml fixes (failure masking,
  iOS/ casing, -project flag, triggers, bindings-drift gate) actually
  landed; re-open and implement if not. [SONNET/CODER, verify-first].
- **C3** XCFramework + Swift bindings regen — AFTER PQC-10 lands (single
  regen cycle, per the queue's standing note). [SONNET, mechanical parts
  HAIKU]. [DEVICE: at least one farm iPhone for smoke test].
- **C4** iOS farm-pillar verification pass: dns4 dial-back, mDNS LAN, BLE
  pairwise to Android, receipts/outbox semantics (A5), background-BLE
  staleness honesty (fable5plan T1.6 hardening items fold in here).
  [SONNET][DEVICE: real iPhones — the farm-mates' phones are the fleet].
- **C5** Distribution decision: Apple Developer account (USD 99/yr) +
  TestFlight vs alternatives. Required before F3 pilot. [HUMAN].

### WS-FARM-D — Meeting Mode (S4)

- **D1** Design note: connection budget/rotation for 6-10 devices vs
  Android concurrent-GATT ceiling; in-room star-hub election (reuse
  GO-intent/charging heuristics); Multipeer offload for iOS pairs; gossipsub
  room-topic fan-out; group-thread scope call. 2-3 pages in HANDOFF/plans/,
  decomposes into D2/D3 sized [SONNET]. [OPUS+/THINK — this is the one
  genuinely novel design in the plan].
- **D2** Implementation per D1 (Rust link budget + Kotlin/Swift session
  management). [SONNET/CODER][AUDIT — transport/ble/].
- **D3** L2CAP fragmentation hardening under loss (reassembly timeout,
  per-peer memory cap, CRC assert, 10k-stream proptest — fable5plan T1.5
  spec is still accurate). [SONNET/CODER].
- **D4** Room drill FD-4 execution + evidence. [DEVICE][HUMAN: needs 6-10
  actual humans — schedule against a real community meeting dry-run].

### WS-FARM-E — Crypto soundness before the seed ships

The farm pitch is security; shipping known-broken PQ crypto to the seed
community is not an option. All are existing tickets, all [AUDIT] by
definition:
- **E1** `PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` — CRITICAL: the PQ
  ratchet is cryptographically inert post-bootstrap today. [SONNET impl,
  fable adversarial review]. PQC-11/13 stay frozen until this lands (standing
  queue rule).
- **E2** `PQC_07_FORCE_RATCHET_SAME_DEFECT.md` + 
  `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md` — same defect family. [SONNET].
- **E3** `PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED.md` — restart mid-
  conversation must not brick decryption (farm reality: phones die daily).
  [SONNET].
- **E4** `PQC_08_LEGACY_PATH_RETIREMENT.md` — finish remaining scope (in
  progress). [SONNET/CODER].
- Explicitly NOT farm-gating: PQC-09 wiring (AD-8 freeze), PQC-10..14 depth
  work — they proceed in the background lane and C3 waits on PQC-10.

### WS-FARM-F — Cooperative ledger + custody convergence

- **F1** `integration_ledger_convergence.rs`: N in-process nodes, arbitrary
  connect graph -> superset convergence; stale-node (simulated 2-week-old
  ledger) rejoin -> full reconverge on one contact; all discovery surfaces
  write into the one ledger (audit included). [SONNET(high)][AUDIT if fixes
  touch transport/]. New task, cut from this plan.
- **F2** Drift custody persistence audit: does `MeshStore` custody survive
  process death on mobile (fable5plan flagged in-memory construction at
  `iron_core.rs:264` — verify-first, fix with `StorageBackend`/sled if
  confirmed, cap + eviction with DropReason). [SONNET(high), verify-first].
- **F3** Contact-window efficiency: custody sync completes useful transfer in
  a 10 s BLE contact (S5); rate limiter must not starve short windows.
  Existing T2.2-shaped test scenario. [SONNET]. Stretch, not gating.

### WS-FARM-G — Field ops, honesty, adoption

- **G1** `NETWORKERROR_OBSERVABILITY_GAP.md` (existing ticket) + routing
  telemetry ring buffer in the diagnostics report (fable5plan T3.4 spec):
  when a farm-mate says "it didn't send", the device must be able to say WHY
  without adb. [SONNET/CODER].
- **G2** Honest message states surfaced in UI on BOTH platforms:
  Queued -> InCustody -> Sent(unconfirmed) -> Delivered(receipt-verified),
  no checkmark without a verified receipt (depends on A2). [SONNET].
- **G3** Onboarding path for non-technical farm-mates: QR contact exchange
  verified end-to-end, first-run defaults pointing at the farm anchor config,
  one-page paper quickstart (English, plain). [SONNET + HAIKU docs].
- **G4** Battery honesty soak: overnight foreground-service soak on 2+ real
  phones with the mesh idle-connected; measure and record drain; tune
  keepalives if >5%/night class. [DEVICE].
- **G5** Install/update path: Android APK direct (or Obtainium-friendly
  release), iOS per C5; update cadence documented (farm gets stable tags
  only, never main). [HAIKU docs + HUMAN release discipline].

### WS-FARM-H — Onion seam freeze

- **H1** Seam-freeze test per AD-8: privacy module compiles + unit tests in
  every gate run; zero live-path call sites asserted; wiring point
  documented in ARCHITECTURE/docs. [HAIKU/FLASH — mechanical]. Keeps
  `PQC_09_SECURITY_REVIEW_FIXES.md` parked with a clear conscience.

## 5. Farm Readiness Drills — what "perfect" means, made literal

Repo standard applies: every gating drill passes TWICE, reproducibly,
cold-start included, evidence logged to the dated ledger doc
(`docs/release-readiness-YYYY-MM-DD.md` style). Simulated first on the B4/B6
rig, then live at the farm. GATE = blocks farm rollout; STRETCH = recorded,
not blocking.

| ID | Drill | Pass criteria | Gate? |
|---|---|---|---|
| FD-1 | LAN pairwise (S1) | Any two farm-WiFi nodes: discovery + E2E delivery + verified receipt, both directions, cold start included | GATE |
| FD-2 | Cellular-to-farm (S2) | Cellular-only node <-> farm-WiFi node via anchor: delivery + receipt both ways; carrier-filtered path lands via 443/WSS ladder | GATE |
| FD-3 | Town dial-back (S3) | Real Pahoa/Hilo WiFi + cellular: roamer reaches farm peers, drains queued messages both directions within 60 s of connectivity | GATE |
| FD-4 | Meeting room (S4) | 6+ mixed iOS/Android devices, foreground, one room, no WiFi/internet required: all-pairs message delivery, 30-minute soak, zero lost messages | GATE |
| FD-5 | Fiber-cut (S6) | Kill WAN at the firewall: S1 unaffected; cellular nodes fail over to cloud relays; restore fiber -> full reconvergence unattended <15 min | GATE |
| FD-6 | IP-flip (AD-2) | Force WAN IP change + DDNS update while a remote node is active: remote node reconnects unattended, no restart, <15 min | GATE |
| FD-7 | Drive-by sneakernet (S5) | Slow drive-past at the choke point flushes queued custody between two nodes | STRETCH |
| FD-8 | Stale rejoin (D2/AD-4) | Node offline 14 days (simulated clock) rejoins on ONE contact: ledger reconverges, queued custody drains, ratchet still decrypts (E3) | GATE |
| FD-9 | Overnight soak (G4) | 2+ real phones idle-connected overnight: no ANR, no service death, battery drain recorded and acceptable | GATE |
| FD-10 | Delivery-truth audit (D3) | During FD-1..4: zero false "failed" for delivered messages, zero checkmarks without verified receipt | GATE |

FD-4 and FD-9 need real farm-mate hardware [DEVICE][HUMAN]; everything else
has a rig-simulated first pass.

## 6. Rollout sequence (the seed planting)

- **F0 — Stabilize (now):** WS-FARM-A (A1-A4) + WS-FARM-E (E1-E4) + B1/B2.
  Exit: compile/test/clippy/fmt gates green; A-fixes live-verified
  CLI<->emulator; adversarial verdicts on file.
- **F1 — Infrastructure up:** B3 anchor live on farm fiber [HUMAN firewall],
  B4 cloud relays live, B5/B6 rig proofs green. Exit: FD-2/FD-5/FD-6 pass on
  the rig; FD-2 passes live with Lucas's own devices.
- **F2 — Daily-drive (dogfood):** Lucas + household devices run it as the
  actual messenger for a week. G1/G2 land here. Exit: FD-1/FD-3/FD-10 live,
  plus a week of honest daily use with zero silent losses.
- **F3 — Pilot (2-3 friendly farm-mates):** C-lane must be done for iPhone
  pilots (C1-C5); G3 onboarding + G5 install paths ready. Exit: pilots
  message daily without hand-holding; FD-9 on their hardware.
- **F4 — Full farm:** everyone in; FD-4 executed at a real community meeting
  (dry-run first); FD-8 verified on a genuinely-lapsed device. Exit: the
  farm uses it because it works, not because Lucas asks them to.
- **F5 — Island seeds:** publish the anchor-kit runbook (B3 generalized: any
  community = one DNS name + one always-on box + config); Hilo early adopters
  as voluntary relay nodes; WASM/browser client via CLI bridge for
  desktop-only users. No code changes required by design (D5).

## 7. Queue re-rank + honesty ledger

`HANDOFF/todo/_QUEUE.md` EXECUTION ORDER re-ranked 2026-07-13 to: A1 -> A2/A3
-> E1 (with E2/E3 following) -> B1/B2 -> C1/C2 [HUMAN+SONNET] -> B3/B4/B5 ->
D1 -> F1/F2 -> remainder per Section 4 dependencies. PQC-09..14 depth work
stays behind E1 and the C3 regen point exactly as already queued. New tasks
(A4, B1, B3, B4, B6, D1, F1, F2, G2, G3, H1) are cut into task files by the
next orchestrator session directly from this plan's specs — each WS entry
above is written to be ticket-ready.

**Honesty ledger (what this plan trusts vs verified):**
- No cargo/gradle/device command was run in this session (planning session,
  Windows toolchain untouched). All code-state claims trace to: the queue and
  execution plan (operator-maintained), tickets with recorded live evidence
  (outbox/receipt CRITICALs, 2026-07-12 live tests), and direct source reads
  this session (behaviour.rs stack, bootstrap.rs empty defaults + dns flag,
  ledger_exchange/LedgerManager, relay 443 rationale, iOS transport files,
  privacy module presence, scmorc routing table).
- Prior "COMPLETE" claims in this repo have failed live re-verification
  before (NEXT_ITER_04 history). This plan therefore treats NOTHING as
  farm-ready until its FD drill passes — including things marked done.
- Unverified assumptions flagged in-plan: farm WiFi is one L2 segment (S1),
  Android concurrent-GATT ceiling on farm-mates' actual devices (D1 measures
  it), MeshStore custody persistence (F2 verify-first), iOS dns4 dialing
  (C4), Alibaba free-tier egress limits for relay duty (B4 confirms).
- The single most leveraged unknowns for the farm: A2 receipt round-trip
  (delivery truth) and B1 IP-flip behavior (mandatory requirement). Both get
  drills, not just tests.

