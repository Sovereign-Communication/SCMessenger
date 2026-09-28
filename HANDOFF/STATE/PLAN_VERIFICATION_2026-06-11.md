# Plan Verification — 4-Section Architecture Status vs. Codebase HEAD

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Verifier:** Hermes Agent (overseer), session 2026-06-11
**Source plan:** the 4-section "platform status" claim pasted by Lucas
**Codebase state:** HEAD = `core/src/`, `mobile/`, `cli/`, `wasm/`, `android/`, `iOS/` on disk + `HANDOFF/ACTIVE_LEDGER.md` (2026-05-13 sweep) + `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md` + `docs/CURRENT_STATE.md` (last verified 2026-05-18)
**Verdict:** the plan is **substantially stale**. Three of its four "implemented / dormant / partial" labels are wrong about the current code, and one is half-right for the wrong reasons. Concrete corrections below.

---

## 1. Native Hardware & Proximity Transport Layer

### UniFFI mobile bindings — plan: "Active Development / Unstable"
**Correction: shipped and stable for the core scaffolding.**
- `core/src/lib.rs:93` — `uniffi::include_scaffolding!("api");` is live.
- `core/Cargo.toml` exposes the `gen-bindings` feature; `mobile/` crate builds `cdylib` + `staticlib`; `core/src/api.udl` has the full UDL surface.
- Cross-platform parity: still tracked in `[VALIDATED]_P1_IOS_001/002/003` (iOS side blocked on macOS) and `docs/historical/plans/PRODUCTION_ROADMAP_PRIORITIZED.md` ("Android: 30+ Kotlin compile errors, WASM: 28 cfg-gate errors"). So the **Rust UniFFI side is solid**; the platform-consumer sides (Android Kotlin / WASM / iOS Swift) still have compile gaps. The plan's framing is half-right but the blame is on the wrong layer.

### BLE & Wi-Fi Aware — plan: "Partially Implemented / bottleneck"
**Correction: more advanced than the plan suggests, but with the exact gaps the plan names.**
- `core/src/transport/ble/` = `beacon.rs`, `gatt.rs`, `l2cap.rs`, `scanner.rs`, `mod.rs` — all 5 files exist; `DutyCycleManager`, `BleScanConfig`, `GattWriteQueue`, `L2capFragmenter` are real, typed, serialized.
- `core/src/transport/wifi_aware.rs` is a real `WifiAwareConfig` + `WifiAwareError` module with `publish/subscribe/max_data_paths`. It is **not** a stub.
- **Bottleneck is real**: per `HANDOFF/ACTIVE_LEDGER.md` (2026-05-13), `add_discovered_peer` in `wifi_aware.rs` has **no external callers** (stub-only); BLE `gatt.rs:on_read/on_write` are **trait method definitions with no callers**. So the layers exist but the production dispatch path doesn't reach them yet.
- BLE duty-cycle management (`scanner.rs:105` `DutyCycleManager`, `BatteryState` enum) is real, not "existence-only" — but `HANDOFF/[VALIDATED]_P1_CLI_031` notes the daemon run-path is **not verifiable without hardware**.

### Acoustic / Ultrasonic Fallback — plan: "Not Implemented"
**Correction: confirmed.** Zero matches for `acoustic|ultrasonic|audio_modem|audible` across `core/` and `docs/`. No code, no docs, no tickets. This is the only claim in §1 that is accurate.

### Automated Radio Cycling — plan: "Partially Implemented"
**Correction: partially right, partially wrong.**
- **Right:** `scanner.rs` has adaptive duty cycles per `BatteryState`; `core/src/platform/auto_adjust.rs` profiles `battery_percent`, `is_charging`, `is_on_wifi`, `is_moving`, `screen_on`, `time_since_last_interaction_secs` and adjusts mesh params. Real code.
- **Wrong:** the "background daemon / aggressive OS battery managers" piece is **not solved** — there's no Android-side foreground service integration here (that lives in `android/`, separate code path). The plan is right that mobile-OS backgrounding is incomplete, but it understates what's already in `core/`.

---

## 2. Asynchronous Storage & Delay-Tolerant Networking

### Content-Addressed Append-Only Log — plan: "Implemented"
**Correction: half-right, terminology is wrong.**
- There is **no** "content-addressed append-only log" in the architecture. What exists:
  - `core/src/store/outbox.rs` — per-peer queues with `MAX_QUEUE_PER_PEER: 1000`, `MAX_TOTAL_QUEUED: 10_000`, sled-backed persistence. Plain FIFO outbox, not content-addressed, not append-only in the CAS sense.
  - `core/src/store/inbox.rs` — incoming store, not append-only.
- The outbox-first decoupling is real (iron_core stores envelopes first, swarm flushes on `PeerDiscovered`). But there is **no BLAKE3 / CID addressing** of stored messages. The plan's terminology is borrowed from IPFS / CAS stores and does not match this codebase. Should be relabeled "Sled-backed per-peer outbox with sled persistence, no content addressing."

### Drift Sneakernet / Data-Mule — plan: "Architected but Dormant"
**Correction: terminology is wrong; the module exists and is partially wired.**
- `core/src/drift/` is **not** a sneakernet / data-mule module. It is a **protocol/format module**: `envelope.rs` (DriftEnvelope, 186-byte fixed-width), `frame.rs` (DriftFrame + CRC32 + length), `compress.rs` (LZ4), `sketch.rs` (IBLT for set reconciliation), `sync.rs` (SyncSession state machine with `SYNC_SCHEMA_VERSION`), `store.rs` (MeshStore CRDT), `relay.rs` (NetworkState/RelayEngine), `policy.rs` (PolicyEngine), `rate_limit.rs` (SyncRateLimiter).
| `HANDOFF/[VALIDATED]_P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1.md` (verified gap, 2026-06-04): `/api/drift-status` returns `{"state":"Dormant","store_size":0}` on a running binary. The fields exist in `iron_core.rs:131-133` (`drift_active: false`, `drift_engine: None`, `drift_store: MeshStore::new()`). The toggle methods `IronCore::drift_activate()` / `drift_deactivate()` exist at `iron_core.rs:849, 858` — but **no caller invokes them anywhere in `core/src/`** (grep for `start_drift|stop_drift|enable_drift|disable_drift` and direct method searches return zero matches). So drift defaults to inactive and is never turned on by any internal path.
- **However**, `swarm.rs:383` defines `fn wrap_in_drift_frame(envelope_data: &[u8]) -> Vec<u8>` which constructs a `DriftFrame { frame_type: FrameType::Data, payload: … }`. That helper is invoked from 7 call sites in `swarm.rs`: 766, 1197, 3737, 3750, 3832, 4045, 4458 — and the corresponding `DriftFrame::from_bytes(...)` decodes appear at 2501, 4624. So the *frame format* is in production use across the live swarm path. What's dormant is the **relay-custody / sync session / IBLT reconciliation** part, not the framing.
- There is **no "data mule" / sneaker / store-and-forward carrier** code anywhere — `grep -rli "sneaker|store.and.forward|mule"` returns only docs (`docs/UNIFIED_GLOBAL_APP_PLAN.md`, `docs/historical/plans/DRIFTNET_MESH_BLUEPRINT.md`). This is **truly unimplemented**, not dormant.

### Anti-Entropy Reconciliation — plan: "Partially Implemented"
**Correction: more accurate than the other §2 claims, with a real gap.**
- `core/src/store/blocked.rs:85-110` — `BlockedManager` enforces single-device pairing (`blocked_devs:` device-registry prefix), auto-blocks new devices of a blocked peer. Real, production-wired.
- `core/src/drift/sketch.rs` (IBLT) + `core/src/drift/sync.rs` (3-phase SyncOffer/SyncResponse/SyncComplete using IBLT for O(d) reconciliation) — exist, unit-tested, but never invoked (per the drift-dormant gap above).
- The "merging isolated meshes" capability depends on Drift sync being active. It isn't. So anti-entropy is **architecturally present but functionally blocked** by the Drift wire gap.

---

## 3. Biological/Mycorrhizal Routing & Resource Heuristics

### Hardware-Aware Telemetry & Node Demotion — plan: "Implemented"
**Correction: confirmed, with a real battery-aware stack.**
- `core/src/platform/auto_adjust.rs` — `DeviceState { battery_percent, is_charging, is_on_wifi, is_moving, screen_on, ... }`.
- `core/src/transport/ble/scanner.rs:105` `DutyCycleManager` keyed on `BatteryState`.
- `core/src/relay/` — `RelayConfig.battery_floor_percent` (relay throttles below threshold) and `core/src/store/relay_custody.rs` (per the ledger, `can_bootstrap_others` at `mesh_routing.rs:615`).
- "Demote to leaf" is implemented via `RelayEngine` and `NetworkState` toggling. **Verified.**

### Epidemic Gossip & Mycorrhizal Routing — plan: "Architected but Dormant"
**Correction: the routing engine is ACTIVE in production. This is the biggest factual error in the plan.**
- `HANDOFF/ACTIVE_LEDGER.md` (2026-05-13) already shows: 12 files in `core/src/routing/`, ~5,170 LoC, all unit-tested, with 36+ symbols wired through `iron_core.rs → optimized_engine.rs → target module`.
- `docs/CURRENT_STATE.md` (2026-05-18) — **explicit verification note**: "Mycorrhizal Routing confirmed active in live send path. SwarmHandle::SendMessage calls `engine.route_message_optimized()` at `swarm.rs:3666-3716`, converts via `routing_decision_to_ranked_routes()`, dispatches via `dispatch_ranked_route()`. The routing engine is active in production send path."
- I verified independently: `swarm.rs:37` imports `OptimizedRoutingEngine`; `swarm.rs:3970` calls `engine.route_message_optimized(...)`; `swarm.rs:1794/1812/1849/2039` wire the engine handle; 30+ routing methods exposed on IronCore (`routing_peer_seen`, `routing_negative_cache_stats`, `routing_prefetch_stats`, `routing_timeout_budget_summary`, `routing_calculate_dynamic_ttl`, etc.).
- `core/tests/integration_mycorrhizal_routing.rs` (567 lines, 14 tests) — covers direct routing, transport propagation, negative cache, gateway routing, shared handle pattern, optimization tick, discovery phase advancement, app lifecycle prefetch. Active.
- **The plan's "dormant" label for Mycorrhizal Routing is stale and contradicted by both the ledger and CURRENT_STATE.md.** This is the single largest delta between the plan and reality.

### Mandatory Relaying (No Tit-for-Tat) — plan: "Implemented"
**Correction: confirmed, even stronger than the plan claims.**
- `core/src/drift/relay.rs` is explicit: "**ONE TOGGLE: ON = you can send messages AND relay for others. OFF = you can do neither.** This structurally prevents free-riding… There is no 'receive only' mode."
- `RelayConfig` has no per-message payment / reputation-gated fields — relaying is silent and mandatory.
- **Verified.**

---

## 4. Cryptographic Identity & Zero-Trust State Sync

### Zero-Status Architecture (Metadata Masking) — plan: "Fully Implemented"
**Correction: half-right. Receipts are partially deprecated, NOT fully removed.**
- `core/src/message/types.rs:6-12` — `MessageType::Receipt` **still exists** as a wire variant. The plan implies it's gone.
- `core/src/message/types.rs:22-31` — `DeliveryStatus::Read` is `#[deprecated(note = "Zero-Status Architecture: Read receipts are no longer emitted or displayed")]`. So read receipts are deprecated at the type level but the receipt type itself is still on the wire.
- `core/src/api.udl:14` — `"Receipt"` is still in the UDL `MessageType` enum.
- **The plan is correct that UI no longer shows them** (per the deprecation note + Android `MeshRepository.kt` references to `delivery_receipt_*` are all **internal audit log detail strings**, not user-facing UI — they are event tags for debug).
- So: "UI-stripped, deprecation-tagged, but wire-format still has Receipt variant and stores still receive them." Not "fully removed" but "fading out by deprecation."

### Pure Decentralized PKI — plan: "Fully Implemented"
**Correction: confirmed, with a strong addition not in the plan.**
- Ed25519 + X25519 + XChaCha20-Poly1305 — all in `core/src/crypto/encrypt.rs` and `core/src/identity/keys.rs`.
- **Additions the plan doesn't mention:**
  - `core/src/crypto/ratchet.rs` — **Double Ratchet** (DH Ratchet + Symmetric-Key Ratchet with Blake3 KDF) per `P0_SECURITY_003`. Forward secrecy IS implemented. The plan says "XChaCha20-Poly1305 ensures end-to-end encryption" but doesn't mention the ratcheting layer that gives forward secrecy.
  - `RATCHET_KDF_CONTEXT = "iron-core ratchet v1 2026-04-15"` — versioned, pinned.
- So the dPKI claim is correct AND stronger than stated (ratcheting is shipped).

### Offline Out-of-Band Key Exchange — plan: "Active Development"
**Correction: more accurate than the rest of §4, but framing is off.**
- `core/src/identity/` = `keys.rs`, `mod.rs`, `store.rs`. No `oob.rs`, no `pairing_qr.rs`, no `out_of_band.rs` — `grep` returns nothing under those names.
- `core/src/store/blocked.rs` has single-device pairing enforcement (device-registry prefix `blocked_devs:`). This is **blocklist-side** pairing, not **key-exchange-side** OOB pairing.
- `api.udl` exposes `IdentityInfo { public_key_hex, identity_id, device_id, ... }` — identity strings exist for OOB verification, but the **OOB verification ceremony itself** (QR scan, SAS, fingerprint compare) is not present in the Rust core. It would live in the platform clients (`android/`, `iOS/`, `wasm/`, `cli/`).
- The plan's "Active Development" is plausible but I cannot verify active work from the code; no ticket names OOB-key-exchange specifically. Closest is `P0_SECURITY_007_Identity_Backup_Encryption_V2` (different concern: backup, not OOB exchange).

---

## Summary of deltas (plan claim → reality)

| Plan claim | Reality | Severity |
|---|---|---|
| UniFFI "unstable, parity struggling" | Rust UniFFI stable; gaps are on Android/WASM/iOS consumer sides | low — framing issue |
| BLE/Wi-Fi Aware "partially implemented" | Code is real and substantial; production dispatch gaps confirmed (no callers for `add_discovered_peer`, `on_read`, `on_write`) | medium — understates code, correct on gap |
| Acoustic/Ultrasonic | Correct — not implemented | none |
| Radio Cycling "partially" | `DutyCycleManager` + `auto_adjust.rs` real; mobile-OS backgrounding still open | medium |
| "Content-Addressed Append-Only Log" | No CAS. Per-peer sled outbox with FIFO, no BLAKE3 addressing | high — terminology is wrong |
| "Drift sneakernet dormant" | Drift is a protocol/format module, NOT a data-mule module. `wrap_in_drift_frame()` is called from 7 sites in `swarm.rs` (766, 1197, 3737, 3750, 3832, 4045, 4458). Sync/relay is dormant (toggle methods exist at `iron_core.rs:849,858` but have no callers). Data-mule carrier is unimplemented (not just dormant). | high — module misidentified |
| "Anti-Entropy partially implemented" | BlockedManager pairing real; IBLT sync blocked by drift-dormant gap | medium |
| "Mycorrhizal Routing dormant" | **ACTIVE in production send path** per `CURRENT_STATE.md` 2026-05-18 + ACTIVE_LEDGER 2026-05-13 | **CRITICAL — most wrong claim in plan** |
| "Mandatory relaying" | Confirmed, stronger than stated (`drift/relay.rs` makes it structural, not policy) | none |
| "Zero-Status fully implemented" | Receipt variant still in wire format; `Read` deprecated but type still present; UI-stripped, not wire-removed | medium |
| "dPKI Ed25519/X25519/XChaCha" | Confirmed + Double Ratchet present (`crypto/ratchet.rs`, P0_SECURITY_003, 2026-04-15) | low — plan understates |
| "OOB key exchange active dev" | Single-device blocklist pairing exists. OOB verification ceremony (QR/SAS) absent from core. No specific ticket. | medium |

---

## Honest unknowns (things I could not verify without more tool budget)

1. **Compile state today:** I did NOT run `cargo check --workspace`. `HANDOFF/ACTIVE_LEDGER.md` (2026-05-13) reports `cargo check` passes with 1 warning; `cargo test --workspace --no-run` has 10 ICE failures, all in `integration_registration_protocol.rs` cascade. Per `HANDOFF/STATE/2026-06-10_BUILD_GATE_integration_cargo_test_norun.log` this is the most recent data point. **Status as of 2026-05-13: compile gate green, test gate red.**
2. **iOS specifics:** I did not re-scan `iOS/`. Per `CLAUDE.md` "iOS / Swift: Source-complete, needs macOS build. No `.a` binary, stale field build" — your phone is off-limits and Lucas-side mac build hasn't happened, so iOS verification is structurally impossible from this session.
3. **Platform-side compile errors:** `docs/historical/plans/PRODUCTION_ROADMAP_PRIORITIZED.md` (2026-04-21) lists 30+ Android Kotlin errors and 28 WASM cfg-gate errors. I did not re-verify those today — using that file as the most recent authoritative snapshot.
4. **Quota:** 7d at 84.1% per memory. Per Lucas's "low credits" framing, this verification used read-only ops (no dispatch, no edits outside this report file).

---

## Bottom line for the plan

- **Most wrong claim:** Mycorrhizal Routing is "dormant". It is **active** in the live send path as of 2026-05-18. The plan was written from a stale snapshot.
- **Most confused claim:** "Drift" is being used to mean "data-mule sneakernet". Drift is a protocol/format module (DriftFrame, DriftEnvelope, IBLT, SyncSession) and is currently dormant **at the sync/reconciliation layer only** — the frame wrapper is in production. Real sneakernet/data-mule is unimplemented (no code, no docs, no tickets).
- **Most understated claim:** dPKI. The plan omits the **Double Ratchet** implementation in `core/src/crypto/ratchet.rs` (P0_SECURITY_003, 2026-04-15), which provides true forward secrecy.
- **Plan is OK on:** acoustic gap, mandatory relaying, hardware-aware telemetry, the existence (not the depth) of the BLE/Wi-Fi Aware code.
