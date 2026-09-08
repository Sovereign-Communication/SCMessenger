# Beach-join audit and plan -- QR APK share to cloud resume

Status: Active
Date: 2026-09-05
Author: Windows orchestrator seat (native), evidence from three read-only explore passes + direct source reads this session
Decisions (operator, 2026-09-05): auto-hotspot flow; full stranger-grade trust up front (verify_bundle wiring ships in the same package, not as a follow-up)

## Use case

Two strangers meet with no WiFi and no internet. Phone A (installed) shares the
app to phone B (fresh) by QR scan. B installs, joins the mesh, and learns A's
always-on cloud node address. While together they communicate over local
transports. When they part and both regain cellular, both resume via the cloud
node (direct dial where reachable, store-and-forward relay where not) with no
manual re-seeding.

Reference scenario: beach, WiFi off, BLE/localhop while together, cloud relay
after departure.

## Verdict

The QR/APK pieces exist and are wired (`scripts/check_wiring.py`: `[OK]`, run
2026-09-05). But the end-to-end flow has four P0 breaks: the QR download URL is
unusable without WiFi, the join path never persists a seed, BLE never gossips
the ledger, and joining is cryptographically unauthenticated in both
directions. A stranger's phone would get the app yet never reach the cloud
node. This document inventories each link, then phases the fix.

## 1. What works today (verified this session)

| # | Link | Evidence |
|---|---|---|
| 1 | APK HTTP host on ephemeral port, QR render, Settings entry | `android/.../utils/ApkShareManager.kt:109` (`ServerSocket(0)`), `ui/dialogs/ApkShareDialog.kt:46`, `ui/components/QrCode.kt:51`, `ui/screens/SettingsScreen.kt:262,456-458` |
| 2 | QR join-mesh scan screen, route registered | `ui/join/JoinMeshScreen.kt:161-184` (ML Kit), route `ui/MeshApp.kt:410`, from Dashboard `:314-316` |
| 3 | BLE messaging with no internet | GATT server/client + L2CAP carry full Drift envelopes; phone-to-Windows PASS in the 3-node test (`tmp/run-evidence/3NODE-TEST-2026-09-03-WIFI-BLE-CELL.md:38`) |
| 4 | Cloud resume machinery (relay re-dial with backoff, custody pull, Android `onNetworkChanged` outbox flush) | `core/src/transport/swarm.rs:3406-3480,3333-3346,4311-4320`, `MeshRepository.kt:4260-4266` |
| 5 | Boot seed dial on Android | `core/src/mobile_bridge.rs:861-871` spawns `connect_to_seed_peers()` at swarm start; evidenced live (3-node Gate 1 PASS) |
| 6 | Boot auto-restart, FileProvider system share | `service/BootReceiver.kt:23`, `AndroidManifest.xml:111-120,153-161` |

## 2. The breaks

### G1 -- P0. BLE never teaches the stranger the cloud node

BLE ingress runs `mobile_bridge.rs:1582 on_ble_data_received` into
`core.receive_message` only. That path produces no `ConnectionEstablished`, no
ledger exchange, no routing feed (P1-15 audit,
`HANDOFF/plans/P1-15_transport_matrix_audit.md:107-108`). Ledger gossip fires
solely on swarm TCP connections (`core/src/transport/swarm.rs:5497-5527`,
gated on `!peer_is_blocked && !ledger_exchanged_peers.contains`). BLE payloads
over 512 B are dropped at ingress (`mobile_bridge.rs:68`, `gatt.rs:10`), so
ledger-over-BLE is not even representable today.

Consequence: "BLE across the beach, then auto-swap to cloud" fails at the
handoff. The stranger's ledger stays empty over a pure-BLE encounter.

Resolution (operator-chose combination): put the cloud seed in the QR bundle
(Phase 2, covers every QR flow) AND treat hotspot-LAN libp2p gossip as the
top-up path (automatic if mDNS + `ConnectionEstablished` + ledger exchange
fire on the hotspot subnet -- UNVERIFIED, device proof in Phase 2). Pure
proximity ledger-gossip-as-message is deferred to Phase 4; it needs replay
protection and disclosure-filter reuse and is not required for the QR flow.

Do not "fix" by making BLE ingress synthesize `ConnectionEstablished`: the
swarm connection bookkeeping (dial state, backoff, ledger dedupe) assumes a
libp2p connection that does not exist on the BLE side path.

### G2 -- P0. The QR download URL is dead without WiFi

`getLocalIpAddress()` returns the FIRST non-loopback IPv4
(`ApkShareManager.kt:81-99`). With no WiFi that is the cellular CGNAT address:
unreachable, and serving the APK on it leaks presence to the carrier network.
Nothing in the repo creates a hotspot (zero `LocalOnlyHotspot` hits; only two
comments mention hotspots: `transport/SubnetProbe.kt:337`,
`ApkShareManager.kt:79`). The comment at `ApkShareManager.kt:128-130`
documents a past incident where this URL seeded an unrelated node.

Fix (Phase 1): sender opens a `LocalOnlyHotspotReservation`, binds the
existing `ServerSocket` host to the hotspot interface IP (replacing
first-IPv4-pickup), QR encodes standard `WIFI:S:<ssid>;P:<pass>;;` plus
`http://<hotspot-ip>:<port>/scmessenger.apk`. Reservation loss must stop the
host. Manual-hotspot remains the fallback path, not the primary.

### G3 -- P0. A learned address is never persisted as a seed

`JoinMeshScreen.parseAndJoin` parses only legacy `"bootstrap_peers"` JSON and
dials -- it never imports seeds (`JoinMeshScreen.kt:42,348,359-419`; the
referenced `server.rs handle_join_bundle` does not exist).
`importSeedEntries`/`exportSeedEntries` have zero Android callers.
`MeshRepository.kt:3848` starts the swarm with `listOf()` bootstrap, and the
code states the consequence plainly: fresh-install seed dial runs with zero
candidates (`MeshRepository.kt:5768,6543`, `ensureBootstrapRelayConnected`
`MeshRepository.kt:5761-5777` logs "fresh install; ledger fills in via
invite/QR or LAN discovery").

Fix (Phase 2): define join-bundle v1 (cloud multiaddrs, inviter contact
bundle, bundle signature, APK SHA-256, format version), persist cloud
multiaddrs to the seed/ledger store at ingest. The existing seed dial, relay
re-dial, and custody pull then carry the resume with no new transport logic.

### G4 -- P0 security. Joining is unauthenticated in both directions

`verify_bundle` (`core/src/identity/keys.rs:452`) has zero production callers
(only tests). `save_contact_bundle` (`core/src/store/contacts.rs:669`) has
zero production callers (only tests). This is the deferred post-tag item from
CTO_STATE 2026-08-24; for random-stranger joining it is a ship-blocker, not
deferrable: without it an attacker plants a bundle pairing the victim's real
ed25519 key with attacker-controlled x25519/mlkem keys, defeating the #221
sender-auth fix for that victim. Separately, the APK has no hash in the QR and
no receiver-side verification: a beach attacker on the same hotspot can serve
a trojaned APK to the stranger.

Fix (Phase 1 + 3): SHA-256 in the QR, verify-before-install fail closed
(Phase 1); wire `verify_bundle` at every bundle-ingestion site with
reject-unsigned fail closed, plus a signature/provenance story for the APK,
plus a Rule-8 adversarial APPROVE of the delta (Phase 3, touches
`core/src/crypto|identity|transport`).

### G5 -- P1. No in-app APK-receive path

The stranger scans with the system camera, downloads in a browser, approves
unknown-sources, installs manually. Workable but lossy and untested. Phase 1
delivers an in-app fetch flow or a rehearsed, scripted camera flow with
pass/fail. `MainActivity.kt:348-365` ignoring `ACTION_SEND` and the
`ShareReceiver.kt:113,128-151` broadcast-context dialog are adjacent rough
edges on the system-share path; fix or explicitly disposition in Phase 1.

### G6 -- P1 (watch item). Hotspot-local addresses vs T14 disclosure rules

The phone will observe hotspot-subnet addresses; the RFC1918 same-subnet
filter (`ledger_entry.rs` `addr_filter`, `is_disclosable_multiaddr`) should
contain them to the hotspot. Confirm on device that hotspot IPs do not
pollute other nodes' stores; add a regression test if the filter needs it.

## 3. Phased implementation plan

Operator rulings baked in: auto-hotspot; trust ships in the same package.

### Phase 0 -- formats first (spec, no code)

Join-bundle v1 and the single-QR payload (`WIFI:` + URL + `#bundle=` +
`#sha256=` + mandatory format version). Wire-format timing applies: cheap
before any installed base, compatibility matrix after. Exit: spec reviewed;
version field mandatory; Phase 1/2 implement exactly it.

### Phase 1 -- hotspot share + verified install (Android-only, no Rule-8)

1. LOH sender flow, hotspot-interface IP selection, QR WIFI+URL+hash+bundle.
2. Receiver fetch + SHA-256 verify-before-install, fail closed; unknown-sources
   handoff tested on the Pixel 6a.
3. G5/G6 disposition (receive path, hotspot-IP disclosure hygiene + test).
4. Scope guard: do not touch `connect_to_seed_peers`, the `is_dialer()` ledger
   guard (`swarm.rs:5397` region), or BLE ingress semantics.

Exit: two-device proof with WiFi/cell off -- Pixel A shares to a wiped second
handset, hash-verified install, app opens. `UNVERIFIED` without the second
handset; state which leg is missing.

### Phase 2 -- seed import + cloud resume (core store/transport, Rule-8 review)

1. Bundle ingest: parse v1, persist cloud multiaddrs to the seed/ledger store
   (the missing import at the `MeshRepository.kt:3848` site), persist inviter
   contact through `verify_bundle`.
2. Prove hotspot-LAN gossip top-up: mDNS + `ConnectionEstablished` + ledger
   exchange between the two phones on the hotspot subnet. If it does not fire,
   file the ticket with log evidence; do not redesign around it silently.
3. Depart-and-return proof: both phones leave hotspot range, regain cell,
   re-mesh via the cloud node unaided; custody delivers a message sent while
   one side was offline. Score from receiver-side display + durable history +
   receipt, and from the swarm audit log -- not the API custody counter alone
   (split-brain history: `iron_core.rs:398` vs `swarm.rs:3019`; re-verify which
   store the API reads before trusting it).

Exit: the depart-and-return proof on the reviewed head.

### Phase 3 -- trust wiring + adversarial gate (blocks beach use)

`verify_bundle` at every ingestion site, APK provenance story, Rule-8
adversarial APPROVE of the Phase 1-3 delta from a non-author reviewer. Only a
plain `Verdict: APPROVE` closes the gate. Then merge per the standing
no-self-merge rule.

### Phase 4 -- deferred (not this package)

Ledger-gossip-over-proximity for the no-QR discovery path; D7 offline
proximity gate; `MeshVpnService` orphan disposition (unrelated to this flow).

## 4. Work order and sizing

Phase 0, then Phase 1 and the Phase 2 ingest in parallel (disjoint files:
`android/` vs `core/src/store` + ingest call sites), then Phase 2 device
proofs, then Phase 3 review, then merge. Project-measured rate is ~215
insertions per defect commit, roughly half tests: expect ~2 PRs (Phase 1),
~2 PRs (Phase 2), ~1 PR + review (Phase 3). Largest unknown is
hotspot-subnet gossip behavior -- genuinely device-observable, not derivable
from source.

Lane: Freebuff per `docs/rules/FREEBUFF.md`. Continuation brief:
`HANDOFF/freebuff/queue/V040_BEACH_JOIN_CONTINUATION_2026-09-05.md`.
Device proofs need the Pixel 6a + a second handset (Josh's phone doubles as
handset 2 once the 3-node You-Pixel-AWS gate is green).

## 5. What this audit did not verify

No builds ran; no device was touched. `check_wiring.py` passes, which proves
registration, not runtime behavior (it cannot see the hotspot-IP or
seed-persistence gaps -- both are logic, not wiring). The 3-node
You-Pixel-AWS validation (PR #276 follow-through) is the prerequisite: it
exercises the same cloud node and custody paths Phase 2 depends on.
