# TASK: P1-CORE-BLE-GATT  `GattServer`/`GattClient` traits have zero implementations (CORE_SWEEP_03) + malformed `GATT_SERVICE_UUID`

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] (a delete-with-decision or a wire-up; the design call is small and mechanical once the direction is picked).
**Gates:** [AUDIT-GATE] (touches `core/src/transport/ble/`).

## Source

P1-15 transport-matrix ground-truth audit (`HANDOFF/plans/P1-15_transport_matrix_audit.md`, section (c)), 2026-07-04 [V-READ]. Resolves the long-standing `[NEEDS PLANNING] CORE_SWEEP_03_ble_gatt_traits_never_implemented.md` (referenced in `HANDOFF/RESUME_STATE_2026-07-04.md:86`).

## Problem (exact, verified)

Two findings, both confirmed by grep/read this session:

1. **Dead traits (CORE_SWEEP_03 holds).** `core/src/transport/ble/gatt.rs:279-313` defines `pub trait GattServer` and `pub trait GattClient`. A repo-wide grep for `impl GattServer` / `impl GattClient` / `dyn GattServer` / `dyn GattClient` returns **zero matches**. `core/src/transport/ble/mod.rs:27` re-exports `GattServer` but nothing implements it. Neither live BLE path uses these traits:
   - CLI uses `btleplug` directly (`cli/src/ble_mesh.rs`), not the trait.
   - Android uses the Android BLE stack directly (`BleGattServer.kt`), not the trait.
   The fragmentation/reassembly/write-queue helpers in the same file (`GattFragmenter`, `GattReassembler`, `GattWriteQueue`, lines 107-276) ARE unit-tested but are not called by any live path either.

2. **Malformed service-UUID constant.** `core/src/transport/ble/gatt.rs:10`:
   ```rust
   pub const GATT_SERVICE_UUID: u128 = 0x0000_0DF0_1000_1000_8000_0080_5F9B_34FB;
   ```
   The nibbles are shifted: this yields `0DF01000-...` in the top 32 bits, NOT the intended base-UUID form `0000DF01-0000-1000-8000-00805F9B34FB`. The live paths all use the CORRECT form:
   - `cli/src/ble_mesh.rs:25` `0x0000_DF01_0000_1000_8000_0080_5F9B_34FB`
   - `android/.../ble/BleGattServer.kt:541` `0000df01-0000-1000-8000-00805f9b34fb`
   - `core/src/transport/ble/beacon.rs:15` `BLE_BEACON_SERVICE_UUID: u16 = 0xDF01`
   Currently harmless (the malformed constant is unused), but it is a live landmine: the moment anyone wires `GattServer`/`GattClient` (e.g. the companion BLE-TX ticket), advertising/scanning on `gatt.rs`'s constant would silently not match any real SCM peer.

## Root Cause

The GATT trait layer was scaffolded (SOVEREIGN_MESH_PLAN era, see `docs/historical/SOVEREIGN_MESH_PLAN.md:387`) as a platform abstraction, but both platforms ended up talking to their native BLE stacks directly, so the abstraction was never implemented. The UUID typo predates any consumer, so it was never exercised.

## Blast Radius

- `core/src/transport/ble/gatt.rs` only, plus its re-export in `mod.rs`. No live path depends on the traits, so removal is low-risk; a wire-up is contained to the trait file + whichever consumer adopts it.
- Touches `core/src/transport/` -> `crypto-security-auditor` gate mandatory before done.
- **Decision fork (record the choice in the task file, don't silently pick):**
  - **(A) Delete-with-decision:** remove `GattServer`/`GattClient` traits (keep the tested `GattFragmenter`/`GattReassembler`/`GattWriteQueue` if the BLE-TX ticket will reuse them; otherwise remove those too) and the malformed constant. Cleanest if BLE stays native-per-platform.
  - **(B) Wire-and-fix:** if the CLI BLE-TX ticket (`P1_CLI_BLE_Outbound_TX_Path_Missing.md`) will implement `GattClient` for the btleplug central path, keep the trait, FIX the UUID constant to `0x0000_DF01_...`, and reconcile the characteristic map: `gatt.rs` currently maps `0xDF02=Write, 0xDF03=Notify, 0xDF04=Status` (`:29-38`), while the live Android server maps `0xDF02=Identity(read), 0xDF03=Message(write+notify), 0xDF04=Sync` (`BleGattServer.kt:544-546`). These MUST be reconciled to the live layout or the trait will target the wrong characteristics.

## Files to Touch

- `core/src/transport/ble/gatt.rs` (traits + `GATT_SERVICE_UUID` + characteristic enum)
- `core/src/transport/ble/mod.rs` (re-exports, line 27)
- Coordinate with `P1_CLI_BLE_Outbound_TX_Path_Missing.md`  these two must not both edit the framing layer blind.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib transport::ble
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
# If (A) delete: grep for any lingering references
grep -rn "GattServer\|GattClient" core/ cli/ mobile/ --include=*.rs
```

## Do NOT

- Do NOT "fix" the malformed UUID in isolation and leave the dead traits  pick (A) or (B) explicitly; a half-measure leaves a fixed-but-still-dead constant that invites future confusion.
- Do NOT change the live UUIDs (`ble_mesh.rs:25`, `BleGattServer.kt:541`, `beacon.rs:15`)  they are correct and interoperate; only `gatt.rs`'s constant is wrong.
- Do NOT remove the `GattFragmenter`/`GattReassembler` helpers if the BLE-TX ticket intends to reuse them for the CLI write path  confirm with that ticket's implementer first.
- Mandatory `crypto-security-auditor` review before done (transport module). `release-gatekeeper` before merge.
