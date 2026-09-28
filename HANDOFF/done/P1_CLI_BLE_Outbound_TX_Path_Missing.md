# TASK: P1-CLI-BLE-TX  CLI has no BLE outbound (write/TX) path; Android<->Windows BLE is half-duplex (Android->CLI only)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] implementation (may need an [OPUS+] design pass first if peripheral-advertising is chosen  see below).
**Gates:** [AUDIT-GATE] (touches `core/src/transport/`), [DEVICE] (BLE radios required to prove).

## Source

P1-15 transport-matrix ground-truth audit (`HANDOFF/plans/P1-15_transport_matrix_audit.md`, section (c)), 2026-07-04. [V-READ] this session  grep/read only, no cargo/BLE hardware run. Blocks the plan's BLE worst-case exit cell (2.6) and P1-16.

## Problem (exact, verified)

BLE is a **half-duplex data path** today: Android -> CLI works in code, CLI -> Android does not exist.

- **Android -> CLI (works):** Android's `BleGattServer` notifies subscribers on the message characteristic `0000df03` (`android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt:545` `MESSAGE_CHAR_UUID`, notify send at `:178-223`). The CLI's `run_ble_central_ingress` (`cli/src/ble_mesh.rs:128`) scans service `0xDF01`, subscribes to notify `0xDF03` (`:91-106`), decodes via `IronCore::receive_message` (`:36-57`), pushes to the Web UI (`:120-124`).
- **CLI -> Android (missing):** `cli/src/` has **zero** BLE write/TX. Grep across `cli/src/` for `.write(`, `write_without_response`, `write_char`, `send_ble` returns nothing. `run_ble_peripheral_advertising` (`cli/src/ble_mesh.rs:317`) is a **deliberate no-op stub** (btleplug is central-only on desktop; documented `:299-316`). So the CLI can receive BLE frames but cannot originate one.

Net effect: the worst-case exit test in the plan ("WiFi off, no internet, both radios on  message composed on phone arrives on Windows via BLE, **and vice versa**") can pass phone->Windows but structurally cannot pass Windows->phone today.

## Root Cause

btleplug (the CLI's BLE lib) is central-oriented on desktop OSes. The Android side is wired as the peripheral (it advertises + notifies); the CLI is the central (it scans + subscribes). A central *can* write to a peer's writable characteristic, but the CLI code never does  it only subscribes to notify. Android's message characteristic `0000df03` is declared `WRITE | WRITE_NO_RESPONSE | NOTIFY` (`BleGattServer.kt:108-114`) and Android reassembles inbound writes (`reassemblyBuffers`, `:46`), so the peripheral side is ready to receive a central write  the CLI just doesn't issue one.

## Blast Radius

- New code in `cli/src/ble_mesh.rs` (add a TX/write path as GATT central) and likely a small surface in `core/src/transport/ble/` if fragmentation/framing is shared. Touches `core/src/transport/` -> `crypto-security-auditor` gate is mandatory before done.
- Must reuse the existing Drift-frame + `IronCore` encrypt path so the bytes the CLI writes are decodable by Android's reassembly + `onDataReceived` -> `meshService.onDataReceived` chain. Do NOT invent a second framing.
- Independent of the P1-04 libp2p negotiation bug: BLE messaging is app-level frame relay (`PlatformBridge`/proximity + `receive_message`), not libp2p. So this can proceed on the BLE lane without waiting on P1-04.
- Directional design choice with different blast radii:
  - **(A) CLI writes to Android's `0xDF03` write characteristic as a central**  smaller, stays within btleplug's central capabilities; needs Android's GATT server to route inbound writes on `0xDF03` to `onDataReceived` (verify `BleGattServer` `onCharacteristicWriteRequest` handling exists and reaches the Rust `on_ble_data_received`/proximity path).
  - **(B) CLI advertises as a peripheral so Android writes to it**  larger, needs a per-OS peripheral GATT server (WinRT `GattServiceProvider`) that `run_ble_peripheral_advertising` explicitly declined to build; higher effort, [DEVICE]-heavy. Prefer (A) unless (A) proves infeasible.

## Files to Touch

- `cli/src/ble_mesh.rs`  add a GATT-central write/TX function (mirror `subscribe_ingress_for_peripheral`'s connect/discover, then write to `0xDF02`/`0xDF03` per the chosen direction), wire it to the CLI's outbound message path (`cli/src/api.rs` / `cli/src/server.rs` send flow).
- `core/src/transport/ble/`  only if fragmentation/reassembly needs a shared helper (the fragmenter already exists in `gatt.rs:107` but is currently unused  see companion ticket `P1_CORE_BLE_GATT_Traits_Dead_And_Malformed_UUID.md`; coordinate so you don't wire a malformed-UUID/dead trait).
- `android/.../transport/ble/BleGattServer.kt`  verify (do not rewrite) that inbound central writes on the message characteristic reach `onDataReceived` -> Rust; add routing only if missing.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-cli
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
# Device (P1-16): WiFi off + no internet on both ends; compose on Windows CLI -> confirm arrival on Pixel; then reverse.
```

## Do NOT

- Do NOT re-enable `run_ble_peripheral_advertising` as a partial WinRT implementation without an explicit [OPUS+] design pass  that stub is intentional and its doc (`ble_mesh.rs:299-316`) explains why. Choose direction (A) first.
- Do NOT introduce a second framing/encryption for BLE writes  reuse `DriftFrame` + `IronCore` so Android's existing reassembly/`receive_message` decodes it.
- Do NOT wire the dead `core/src/transport/ble/gatt.rs` `GattClient` trait or its malformed `GATT_SERVICE_UUID` (`gatt.rs:10`)  resolve the companion ticket first or use the correct `0xDF01` form from `ble_mesh.rs:25` / `BleGattServer.kt:541`.

---
**DEFERRED 2026-07-07 (orchestrator, /scmorc lean-mode session):** attempted
twice via agy-Gemini (headless `-p` dispatch) and NOT completed either time -
(1) first attempt drifted into editing unrelated already-closed HANDOFF
tickets instead of touching any BLE-TX file (reverted, zero-diff on the actual
task); (2) second attempt (strict file-scope guard added) hung 16+ minutes
past its own `--print-timeout`, zero CPU activity, zero file changes (agy's
`--print-timeout` flag does not reliably kill a stuck child - had to
`Stop-Process -Force` manually both times this session). No repo state was
lost either time; git tree was clean before and after both attempts.
Assessment: this task's scope (new GATT-central write path in
`cli/src/ble_mesh.rs`, verifying Android's `BleGattServer.kt` inbound-write
routing, coordinating with the currently-unused fragmenter in
`core/src/transport/ble/gatt.rs`, plus the [AUDIT-GATE] transport-touching
requirement) needs either a human/native-Claude-driven exploration pass or a
staged approach (orchestrator does the multi-file reads itself, then hands
Qwen a curated context per sub-piece, mirroring how F2/F5/F7/the rate-limit
signal were landed this session) - NOT a single blind agy dispatch. Left
untouched in `todo/`; no work lost. Next session should NOT retry a bare
`agy -p` dispatch on this ticket a third time.
- Mandatory `crypto-security-auditor` review before this is considered done (transport path). `release-gatekeeper` before merge.
