# Bootstrap / Rendezvous Decision Record and Implementation Spec (issue #469)

Status: Active
Last updated: 2026-10-06
Set by: operator, 2026-10-06 (answers to the #469 decision questions)
Tracker: issue #469 (parent #455)

This record answers the "which rendezvous story ships in 1.0.0" question in
issue #469 and supersedes the options (a)/(b)/(c) framing there. Option (a)
("ship default bootstrap relays in the binary") is rejected outright.
Option (c) ("documented operator requirement") is rejected as the stock-build
story. The shipped answer is invite-seeded ledger sharing plus dynamic,
event-driven discovery, described below.

## 1. Binding decisions (operator, 2026-10-06, verbatim intent)

1. **Bootstrap source: invite QR / invite info ONLY.** No static seeds
   anywhere. For the 3-node rig: simulate the invite as coming from AWS to
   Windows; then when Windows and Android connect, we must see verified ledger
   sharing for node detection.
2. **Nodes are never static; nothing in the repo should ever be static.** This
   is a mesh P2P messenger; nodes are dynamically adjusted per the repo's
   planning. AWS (always on) earns a good reputation because it is always on
   and should naturally become predominant for store/forward, but it is NOT
   different from any other node. All nodes are full relays; the only
   distinction is whether a node has an identity or not; functionally for
   transport all are the same.
3. **Trust: ONLY QR/info invite; no static seeds; just ledger sharing plus the
   existing routing planning** (the mycorrhizal network routing and other
   nuance documented in the repo's philosophy canon).
4. **All-unreachable behaviour: keep retrying, but never show "no network
   peers"** (node indicators already exist). A fixed 120s bounded backoff is
   NOT acceptable: backoff must be dynamic and event-driven -- immediately retry
   on any network state change (e.g. user turns on BLE -> detect the change and
   aggressively poll BLE discovery at first, then slowly back off), same for
   Wi-Fi/cellular/LAN changes. Everything dynamic.

### Acceptance status of issue #469 checkboxes

- Decision recorded: this document.
- "Stock build can join the mesh with zero env configuration": redefined by
  decision 1 as "a stock build joins the mesh by redeeming an invite (QR or
  pasted info) or by LAN/BLE discovery; zero env configuration and zero
  shipped addresses". Delivered by tasks T1-T4 below.
- "Dead v0.2.1 relay retired or replaced": no replacement address is shipped
  (decision 1). Retire = remove every reference to the old address from
  shipped code/docs (task T9). The always-on AWS node is an ordinary node that
  issues invites; it is not compiled into anything.

## 2. Alignment with the philosophy canon

| Decision | Canon source | Alignment |
|---|---|---|
| 1, 3 (invite-only, ledger sharing) | `docs/BOOTSTRAP.md` "Invites Carry a Seed Ledger"; `docs/BOOTSTRAP_GOVERNANCE.md` "Decision"; `core/src/relay/invite.rs` (`InviteToken.seed_ledger`, signed, `SCI1:` QR, 2953-byte budget); `core/src/store/ledger_entry.rs` (`SeedLedgerEntry`, `import_seed_entries`, unproven tier promoted by first live dial) | Already the documented design. The code exists in core but has no production call sites (see gap G1-G3). |
| 2 (no node classes; AWS is a node) | `docs/rules/NODE_MODEL.md` ("All nodes are fully functional and identical. The only distinction between nodes is whether an identity is loaded or not"; fleet purposes are deployment, not node type); `AGENTS.md` "nodes, not relays" | Direct restatement. Use the term "always-on node", never "bootstrap node" or "relay node". |
| 2 (AWS predominant by reputation) | `core/src/store/ledger_entry.rs` `get_preferred_relays` (failure_count ascending, then `last_seen` descending, comment: "a 24/7 AWS relay has a recent last_seen"); `core/src/transport/reputation.rs` (0-100 score, decay); `core/src/transport/relay_health.rs` `priority_score` (uptime 0.4, latency 0.3, stability 0.3) | Behavioural scoring already exists: an always-on node wins on uptime/recency, not on a flag. One conflict: `relay_health.rs` adds `headless_bonus` (+0.1 when `is_headless`), a node-class bonus. See section 7. |
| 3 (mycorrhizal routing) | `core/src/routing/mod.rs` (Layer 1 Mycelium local cell, Layer 2 Rhizomorph neighborhood gossip, Layer 3 CMN global advertisements); `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md` (arteries/capillaries, dynamic clustering, "nodes do not maintain full global network maps") | Discovery after the first contact is ledger exchange (`/sc/ledger-exchange/1.0.0`) feeding Layer 1/2; no external directory. |
| 4 (event-driven, density-aware backoff) | `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md` pillar 4 "Density-Adaptive Heartbeats" and pillar 5 "Sneakernet & Mesh Synergies" (physical encounters/BLE as transport) | Adaptive duty cycles are canon; a constant 120s ceiling is not. |

## 3. Current state (verified against origin/main 3b4177be7)

- Compiled address lists are already empty: `CORE_BOOTSTRAP_NODES = &[]`
  (`core/src/transport/bootstrap.rs:28`), `DEFAULT_BOOTSTRAP_NODES = &[]`
  (`cli/src/bootstrap.rs:27`), CLI config default empty (`cli/src/config.rs:96`).
  No routable IP is compiled into Rust production code.
- Seed intake paths that still exist and contradict decision 1: the
  `SC_BOOTSTRAP_NODES` env var (runtime and `option_env!`), the CLI
  `config set bootstrap_node_add` pseudo-key, and docker/infra files.
- The invite token and seed-ledger machinery is implemented and tested in core
  (`InviteToken`, `to_qr_payload`, `from_qr_payload`, `build_seed_ledger`,
  `export_seed_entries`, `import_seed_entries`) but `to_qr_payload` /
  `from_qr_payload` / `build_seed_ledger` have no production caller outside
  `core/src/relay/invite.rs` and tests, and are not in `core/src/api.udl`. No CLI
  invite command exists (`cli/src/cli.rs` `Commands`). Android `JoinMeshScreen`
  uses a regex-parsed unsigned JSON "join bundle" and only calls `dialPeer`;
  `MeshRepository.importSeedAddresses` has no call site. iOS `JoinMeshView`
  imports `bundle.bootstrap_peers` from the same unsigned JSON.
- Ledger exchange runs automatically on connect in `core/src/transport/swarm.rs`
  and is verified-pair gated before reaching the DHT (`ledger_verified_pair`).
- Boot retry is the fixed `5/15/45/120s` ladder in `cli/src/seed_dial.rs`.
  Network-change callbacks exist (Android `NetworkDetector`,
  `AndroidPlatformBridge`; iOS `NWPathMonitor` in `IosPlatformBridge`) but only
  update the device profile (`MobileBridge::on_network_changed` sets
  `profile.has_wifi`); none triggers a dial or discovery reset. No Android
  `BluetoothAdapter.ACTION_STATE_CHANGED` receiver exists; BLE scan cadence is
  constant (`BleScanner.kt` 30s/60s duty cycle). iOS `centralManagerDidUpdateState`
  only handles a deferred first scan.

## 4. Gap list (remove or change)

Paths are relative to the repo root; line numbers are against 3b4177be7.

### Static seed intake (decision 1, 2)

| ID | Location | Issue | Action |
|---|---|---|---|
| S1 | `cli/src/bootstrap.rs:12-14,30-62` | `SC_BOOTSTRAP_NODES` runtime env and `option_env!` build-time seeding | Remove both reads; delete `default_bootstrap_nodes`, `DEFAULT_BOOTSTRAP_NODES` (`:27`) and the paragraph naming "Primary GCP/Secondary/Tertiary" (`:18-26`) |
| S2 | `cli/src/bootstrap.rs:77,100` | `promiscuous_bootstrap_addrs`, `merge_bootstrap_nodes` are the static-list merge | Remove; callers read the ledger only |
| S3 | `cli/src/main.rs:2527,4093` | `merge_bootstrap_nodes(config.bootstrap_nodes...)` feeds `web_ctx.bootstrap_nodes` (`:2533,:4126`) | Replace with ledger-derived peer list |
| S4 | `cli/src/config.rs:53,96,243-250,276,319,330-356` | `bootstrap_nodes` config field and `bootstrap_node_add/remove` | Remove field, pseudo-keys and `add_bootstrap_node`/`remove_bootstrap_node`; add one-time migration that imports existing values through `import_seed_entries` then drops them |
| S5 | `core/src/transport/bootstrap.rs:28` | `CORE_BOOTSTRAP_NODES` (empty but a static-list mechanism) | Remove constant and the union in `BootstrapManager::new` (`:114-150`) |
| S6 | `core/src/transport/bootstrap.rs:499-509` | `resolve_env_bootstrap_nodes` reads `SC_BOOTSTRAP_NODES` | Remove |
| S7 | `core/src/transport/bootstrap.rs:388-426,513-546` | `discover_dns_bootstrap`, `discover_websocket_bootstrap`, `discover_hardcoded_backup_relays` are empty stubs for static/DNS seed sources | Remove stubs and the `enable_dns_discovery`/`enable_websocket_fallback` flags tied to them (`:48-52,66-68`) |
| S8 | `docker/Dockerfile:14-15`, `docker/entrypoint.sh:30-45`, `docker/docker-compose*.yml`, `docker-compose.yml:17-18`, `docker/farm-sim-compose.yml`, `docker/docker-compose.network-test.yml`, `docker/docker-compose.test.yml`, `infra/cloudformation/farm-sim-stack.yaml:216-546`, `infra/ec2/launch-alpha-relay.sh:192`, `infra/ec2/launch-farm-sim.sh:78`, `infra/ec2/node-userdata-template.sh` | Deploy infra injects seed addresses via env | Replace with a harness step that has one node mint an invite and the others redeem it (`scm invite create` / `scm invite join`, task T3); simulation topologies are test fixtures, not shipped defaults |
| S9 | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:11082` | Literal AWS IP `18.234.62.247` in `isAddrHostConnected` | Remove; decide "connected" from the live peer set, not a host literal |
| S10 | `scripts/recovery_preflight.py:26` | `AWS_HOST = "18.234.62.247"` | Take host from a CLI argument or the ledger; do not store |
| S11 | `docs/BOOTSTRAP.md` Environment Variable, CLI, Private Networks sections; `docs/BOOTSTRAP_GOVERNANCE.md` "Resolution Order" | Document the removed mechanisms | Marked superseded in this change; rewritten when S1-S8 land |

Test-only addresses (`98.94.45.116`, `54.235.20.24`, `18.234.62.247` in
`ledger_entry.rs` tests, `swarm.rs` tests, Android unit tests) are fixtures and
may stay, but must be replaced by RFC 5737 documentation ranges when touched.

### Invite path not wired (decision 1, 3)

| ID | Location | Issue | Action |
|---|---|---|---|
| G1 | `core/src/api.udl`, `core/src/iron_core.rs:2572-2582` | No FFI to create/encode/decode/redeem an invite; only `invite_get_signable_data` exists | Add `create_invite_qr(ttl_secs) -> String` and `redeem_invite_qr(payload) -> RedeemReport` (verify signature, import `seed_ledger`, return count) |
| G2 | `cli/src/cli.rs` `Commands` | No invite command | Add `invite create [--ttl]` (prints `SCI1:` string) and `invite join <SCI1:...>` |
| G3 | `android/.../ui/join/JoinMeshScreen.kt:389-460`, `iOS/.../Views/Topics/JoinMeshView.swift:168` | Unsigned, regex-parsed JSON bundle; Android never imports seeds to the ledger (`MeshRepository.kt:6590` `importSeedAddresses` has no caller) | Replace both with `SCI1:` scan/paste -> `redeem_invite_qr`; delete the JSON bundle format |

### Fixed backoff and non-reactive discovery (decision 4)

| ID | Location | Issue | Action |
|---|---|---|---|
| B1 | `cli/src/seed_dial.rs:14-21` (`next_delay` 5/15/45/120), `:62-70,73-100` (steady 120s watch), `cli/src/main.rs:2671-2690` (spawn) | Fixed ladder with a hard 120s ceiling | Replace by the scheduler in section 6 |
| B2 | `core/src/transport/bootstrap.rs:56-70,362-373` | `max_backoff = 300s`, multiplier 1.5, `max_retries_per_node = 5` ("giving up") | Route through the same scheduler; no give-up state |
| B3 | `android/.../data/MeshRepository.kt:7815-7835` | Fixed 8s `delay` outbox/bootstrap loop | Event-driven wake plus decaying timer |
| B4 | `android/.../transport/ble/BleScanner.kt:99-105` | Constant scan windows/intervals | Scheduler-driven: aggressive on BLE-on / foreground / network event, decay after |
| B5 | `android/.../transport/ble/BleBackoffStrategy.kt:16-18` | `maxDelayMs = 30000` fixed cap | Take cap from the shared decay policy |
| B6 | `android/.../service/AndroidPlatformBridge.kt:216-234`, `.../transport/NetworkDetector.kt:122-142`, `iOS/.../Services/IosPlatformBridge.swift:232-245` | Network callbacks only update `has_wifi` | Also emit a `NetworkEvent` to core (section 6) |
| B7 | `core/src/mobile_bridge.rs:1633-1642` | `on_network_changed` ignores `_has_cellular`, no discovery trigger | Add cellular to profile; call `trigger_discovery(NetworkEvent)` |
| B8 | (missing) Android Bluetooth state receiver; `iOS/.../Transport/BLECentralManager.swift:361-372` | BLE on/off not an event source | Add `BluetoothAdapter.ACTION_STATE_CHANGED` receiver; extend `centralManagerDidUpdateState` to emit `BleStateChanged` on every transition |

### "No peers" UI strings (decision 4)

| ID | Location | String | Action |
|---|---|---|---|
| U1 | `android/app/src/main/res/values/strings.xml:299-300`, `ui/dashboard/PeerListScreen.kt:90,97` | `peer_list_no_peers` "No peers connected" / "Start the mesh service to discover peers" | Replace with neutral discovery-activity copy driven by scheduler state, no "no peers" wording |
| U2 | `strings.xml:244` | `dashboard_empty_state_discovered` "No nodes discovered yet" | Same |
| U3 | `strings.xml:437` | `add_contact_nearby_searching` | Keep (activity, not an absence claim); verify wording |
| U4 | `iOS/.../Views/Dashboard/MeshDashboardView.swift:589` | "No nodes discovered yet. Check transport status below." | Same as U1 |
| U5 | `cli/src/main.rs:4946` | "No peers discovered via local transports." | Replace with scheduler-state line |
| U6 | `cli/src/landing.html:911` | "No peers in ledger yet" | Same |
| U7 | `cli/src/seed_dial.rs:41,61` logs "0 candidate(s), peers=0" | Log text only | Keep as logs, not UI |

The node indicators referenced by the operator already exist and remain the
only "connectivity" presentation.

## 5. Ordered implementation tasks

Protected prefix means `core/src/{crypto,transport,routing,privacy}/`; any task
touching it is merge-blocked on adversarial review (Rule-8 /
`docs/rules/SECURITY_PROTOCOL.md`, `crypto-security-auditor`). Sizes: S < 150 lines,
M 150-500, L > 500. No local builds; CI only.

| # | Task | Files | Protected | Size | Acceptance |
|---|---|---|---|---|---|
| T1 | Invite FFI: `create_invite_qr`, `redeem_invite_qr` on `IronCore`; wire `build_seed_ledger` + signing + `to_qr_payload`; redeem = `verify_with_policy` then `import_seed_entries` | `core/src/iron_core.rs`, `core/src/api.udl`, `core/src/relay/invite.rs` (no protocol change) | No | M | Unit test: create -> redeem on a second ephemeral core imports N unproven seeds; tampered payload rejected; log `[INVITE] redeemed seeds=N inviter=<id8>` |
| T2 | CLI `invite create` / `invite join` | `cli/src/cli.rs`, `cli/src/main.rs` | No | S | `scm invite create` prints `SCI1:`; `scm invite join` imports and triggers immediate sweep (T5 event `InviteRedeemed`) |
| T3 | Mobile redeem: replace JSON join bundle with `SCI1:` scan/paste on Android and iOS | `JoinMeshScreen.kt`, `MeshRepository.kt`, `JoinMeshView.swift`, `MeshRepository.swift` | No | M | Scan of CLI-minted invite imports seeds into the ledger and dials; `importSeedAddresses` has a caller or is deleted |
| T4 | Remove static seed intake S1-S7, S9, S10; ledger-only dial candidates; config migration | `cli/src/bootstrap.rs`, `cli/src/config.rs`, `cli/src/main.rs`, `core/src/transport/bootstrap.rs`, `MeshRepository.kt`, `scripts/recovery_preflight.py` | **Yes** (`core/src/transport/bootstrap.rs`) | M | `rg SC_BOOTSTRAP_NODES` returns docs history only; stock CLI starts with empty ledger and logs `[DISCOVERY] cold: awaiting invite or LAN/BLE`; existing configs migrate once |
| T5 | Shared discovery scheduler (section 6) in core, with `NetworkEvent` intake | new `core/src/transport/discovery_scheduler.rs`, `core/src/transport/mod.rs`, `core/src/mobile_bridge.rs` | **Yes** | L | Unit tests with a fake clock: event resets to aggressive; decay is monotone with jitter; no give-up state; ceiling is a function of observed density/battery, not a literal |
| T6 | Replace `seed_dial` ladder with scheduler client; delete `next_delay` and its test | `cli/src/seed_dial.rs`, `cli/src/main.rs:2671-2690` | No | S | Test: simulated network-change event triggers a sweep within one tick; no `120` literal remains |
| T7 | Platform event sources: Android network + Bluetooth state receivers, iOS NWPathMonitor + CBManager state, CLI interface-change poll | `AndroidPlatformBridge.kt`, `NetworkDetector.kt`, new Bluetooth receiver, `IosPlatformBridge.swift`, `BLECentralManager.swift`, `cli/src/main.rs` | No | M | Turning BLE on logs `[DISCOVERY] event=BleOn transport=ble phase=aggressive`; switching Wi-Fi/cellular logs the same for `net` |
| T8 | BLE/mDNS/LAN scan cadence driven by scheduler; replace constants in `BleScanner`, `BleBackoffStrategy`, `MeshRepository` 8s loop | `BleScanner.kt`, `BleBackoffStrategy.kt`, `MdnsServiceDiscovery.kt`, `MeshRepository.kt`, `BLECentralManager.swift` | No | M | BLE-on -> scan windows at floor interval, widening afterward (log shows rising interval) |
| T9 | Remove "no peers" strings U1-U6; scheduler-state-driven neutral copy | `strings.xml`, `PeerListScreen.kt`, `MeshDashboardView.swift`, `cli/src/main.rs`, `landing.html` | No | S | `rg -i "no (network )?peers"` finds nothing in UI sources; `android-qa` string compliance passes |
| T10 | Infra: replace env-injected seeds with invite flow in harness scripts; remove S8 | `docker/*`, `docker-compose.yml`, `infra/**` | No | M | Farm-sim brings up N nodes where node 1 mints an invite and the rest redeem it; no `SC_BOOTSTRAP_NODES` in infra |
| T11 | Remove `headless_bonus` node-class term from relay scoring; scoring uses behaviour only | `core/src/transport/relay_health.rs` | **Yes** | S | `priority_score` has no `is_headless` input; always-on node still ranks first via uptime/recency in test |
| T12 | Docs: rewrite `BOOTSTRAP.md` and `BOOTSTRAP_GOVERNANCE.md` to the final state; update `CURRENT_STATE.md` | `docs/` | No | S | `bash scripts/docs_sync_check.sh` passes |
| T13 | 3-node acceptance run (section 7) | harness only | No | S | All markers below observed, evidence attached to #469 |

Dependency order: T1 -> T2, T3 -> T4; T5 -> T6, T7 -> T8, T9; T10 after T2; T11
independent; T12 after T4/T9; T13 last. T4 and T5 must not run concurrently with
other work in `core/src/transport/` and both need the adversarial review gate.

## 6. Event-driven discovery scheduler design (decision 4)

Goal: no fixed ceiling constant as the only behaviour; every retry decision is a
function of events and observed conditions.

### Structure

- One `DiscoveryScheduler` per transport class: `ble`, `lan` (mDNS/Wi-Fi
  Aware/Direct/Multipeer), `ledger_dial` (internet/cellular candidates from the
  ledger). Each owns its own phase state; one transport's failure never slows
  another.
- Inputs are `NetworkEvent`s: `BleStateChanged{on}`, `WifiChanged`,
  `CellularChanged`, `LanInterfaceChanged`, `AppForeground`, `InviteRedeemed`,
  `LedgerReceived{new_entries}`, `PeerDisconnected{count_now}`, `AllPeersLost`.
  Each event carries the transports it affects.
- On an affecting event the scheduler **resets that transport to aggressive**:
  `interval = floor` (floor is per-transport and platform-defined, e.g. ~1s for
  BLE scan restart, ~0.5s for a ledger dial), `attempts = 0`, and fires an
  immediate attempt (no wait for the pending timer; the pending timer is
  cancelled and re-armed).
- With no event, the interval decays: `interval_{n+1} = interval_n * g` with
  growth `g` in [1.5, 2.0], then full jitter (`uniform(0.5, 1.0) * interval`) so
  fleets of nodes do not synchronise. Decay is reset on every success that adds
  a new peer, not merely on a connection.
- **No give-up and no literal ceiling.** The ceiling is computed, not declared:
  `ceiling = clamp(base_ceiling * density_factor * power_factor)` where
  `density_factor` rises with currently connected peers (canon pillar 4:
  denser clusters lower ping frequency), `power_factor` rises on low battery or
  background (`DeviceProfile`), and `base_ceiling` is configuration with a
  documented default, never the only behaviour. With zero connected peers and a
  charging device the factors are minimal, so the node keeps probing at a short
  steady interval rather than going quiet.
- Per-candidate dial failure keeps using `DialPolicyManager` per-key backoff
  (`core/src/transport/dial_policy.rs`, unchanged); the scheduler decides *when
  to sweep*, the policy decides *which key is eligible*. A network event also
  calls `reset_peer_backoff` for ledger candidates, since the failure was
  recorded under the old network ("local-epoch" failures already demote rather
  than exclude, per `get_preferred_relays`).

### Platform wiring

- Android: `NetworkCallback.onAvailable/onLost/onCapabilitiesChanged` (already
  registered) plus a new `BluetoothAdapter.ACTION_STATE_CHANGED` receiver
  (`STATE_ON` -> `BleStateChanged{on:true}`).
- iOS: `NWPathMonitor.pathUpdateHandler` (already registered) and
  `centralManagerDidUpdateState` for every state transition.
- CLI/desktop: periodic interface-address snapshot diff (cheap poll of local
  addresses) emitting `LanInterfaceChanged`; the poll interval itself follows
  the same decay.
- Events cross FFI through one `MobileBridge::on_network_event` method;
  `on_network_changed` is kept as a thin shim.

### Logging contract

`[DISCOVERY] event=<kind> transport=<ble|lan|ledger> phase=<aggressive|decay>
interval_ms=<n> attempt=<n> peers=<n>` on every reset and every decayed
re-arm. These lines prove the behaviour in field logs.

### UI contract

The scheduler exposes `phase` and `last_attempt_at` for the existing node
indicators. No component renders an absence message ("no peers"); a node with
zero peers shows the same indicators with an "active/probing" state.

## 7. Three-node acceptance scenario (T13)

Nodes: AWS always-on node (ordinary node), Windows node (CLI), Android node
(Pixel 6a, wireless ADB). Windows and Android start with empty ledgers and no
seed configuration of any kind.

1. On the AWS node: `scm invite create --ttl 3600` -> `SCI1:...` string
   (the simulated "invite comes from AWS"). Marker: `[INVITE] created seeds=<n> inviter=<id8>`.
2. On Windows: `scm invite join <SCI1:...>`. Markers:
   - `[INVITE] redeemed seeds=<n> inviter=<id8>` (signature verified,
     seed_ledger covered by it)
   - `[DISCOVERY] event=InviteRedeemed transport=ledger phase=aggressive`
   - `Started core ledger exchange with newly connected peer <aws-peer-id>`
   - `Ledger exchange response from <aws-peer-id>: they learned <x> new peers, sent <y> back`
   - Windows ledger now holds the AWS entry promoted from the unproven tier
     (`success_count > 0`) - visible in `scm peers` / `ledger.json`.
3. On Android: scan or paste the same invite (or an invite minted by Windows).
   The Android node connects to AWS and/or Windows over LAN.
4. **Windows <-> Android connect and ledger sharing is verified.** Required
   evidence, all from logs of both nodes, within the same session:
   - Android: `Ledger exchange from <windows-peer-id>: offered <n> peer entries`
     or `Ledger exchange response from <windows-peer-id>: they learned <x> new
     peers, sent <y> back`
   - Windows: `Ledger exchange reply to <android-peer-id>: sending <n> peer entries`
   - Receiving side persisted entries: `SwarmEvent2::LedgerReceived` handled
     (`from_peer` = the other node) and the ledger gains an entry for the third
     node it did not previously know (Windows learns Android's address, or
     Android learns AWS's, via the other).
   - "Verified" per code semantics: an exchanged pair only reaches Kademlia when
     `ledger_verified_pair` holds (address locally dialed and bound to that
     peer id). Pass criterion: after the first direct dial between Windows and
     Android, a subsequent entry for the third node appears and is promoted only
     after a successful local dial, never directly from wire data.
5. Failure drill (decision 4): turn Android Bluetooth off then on while isolated
   from Wi-Fi. Marker: `[DISCOVERY] event=BleStateChanged transport=ble
   phase=aggressive interval_ms=<floor>` immediately at on-transition, then
   rising `interval_ms` lines; UI shows indicators, never "no peers".

Scoring follows the standing rule: receiver decrypt + durable ledger entries,
not transport ACKs alone.

## 8. Canon conflicts and supersessions

- `docs/BOOTSTRAP.md` (Environment Variable, CLI `config set bootstrap_node_add`,
  Private Networks build-time seeding, "Cold Start: user supplies one address",
  Mobile bundle section) and `docs/BOOTSTRAP_GOVERNANCE.md` ("Resolution Order
  for Seed Addresses") describe user/operator-supplied static seeds as legitimate.
  Decision 1 supersedes that: the only seed source is an invite. Marked
  superseded in this change (not deleted).
- `core/src/transport/relay_health.rs:52-55` `headless_bonus` is a node-class
  score term, in tension with `docs/rules/NODE_MODEL.md` and decision 2 (task T11).
- `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md` Implementation Mapping states
  `MAX_SEED_LEDGER_ENTRIES (64)`; the code constant is 16
  (`core/src/store/ledger_entry.rs:197`, sized to the QR byte budget). Doc is
  stale; the code and `BOOTSTRAP.md` are right.
- `core/src/transport/bootstrap.rs` header comments describe "Multi-node
  bootstrap with priority ordering" and "Environment variable override"; removed
  by T4/T5.
- `docs/CURRENT_STATE.md:1286-1287` references a GCP bootstrap VM
  (`scmessenger-bootstrap`); inconsistent with "no bootstrap node"; flagged for
  T12.
- `docs/rules/NODE_MODEL.md` and decision 2 agree; "AWS predominant for
  store/forward" is realised by reputation (uptime, recency, failure count), not
  configuration. No conflict.

No conflict was found with the philosophy canon's biological pillars; decision 4
is a stricter application of pillar 4.
