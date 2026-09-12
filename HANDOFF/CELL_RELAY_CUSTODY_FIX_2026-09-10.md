# Cell/AWS relay RCA + custody fix + emulator 3-node status

UTC: 2026-09-10 ~20:09–20:40Z
Fix commit: `441a0214` on `unified/v040-3node-parity`
PR: https://github.com/Sovereign-Communication/SCMessenger/pull/281

## Cell/AWS relay defect (root cause)

Symptom (operator field test): WiFi delivery works; **cell/AWS forwarding does not**.
AWS logs showed:

```
WARN Relay request rejected by custody enforcement
  from 12D3KooWFhvg… (new phone) -> 12D3KooWD6vZ… (Windows)
  : identity_registration_missing
```

**Cause:** Android registers its identity with the relay on Identify
(`mobile_bridge.rs` + `build_registration_request`). The **CLI never did**.
AWS therefore had no Active registration for the Windows identity_id, so any
relay *to* Windows (phone→AWS→Windows) failed custody.

Phone→AWS direct `inbox_receive` still worked (phone registered itself).
Windows→Phone via AWS relay also worked (relay accepted custody *for* the
phone). Only the **to-Windows** path was broken.

## Fix

`cli/src/main.rs`: on `SwarmEvent::PeerIdentified`, call
`register_identity_with_relay()` → `build_registration_request()` +
`SwarmHandle::register_identity`. INFO log:

```
[CUSTODY] Registered local identity with peer 12D3KooWGvCW… (relay-ready)
```

Unit/compile: `cargo check -p scmessenger-cli` PASS.

## Live proof after fix

- Windows `441a0214` logs `[CUSTODY] Registered … 12D3KooWGvCW…` and phone.
- **Zero** `identity_registration_missing` on AWS after 20:36:00Z.
- Win→Emulator probe `a52112da` delivered **6ms** + inbound TRANSPORT-LANE.

## 3-node triangle (emulator replacing Pixel)

| Node | git_hash | peers |
|---|---|---|
| Windows | `441a0214` | Emulator + AWS |
| AWS | `e8c8f52b…` | Windows + Emulator |
| Emulator (`scm_test_34`) | app `f9a1f60b` | `peersDiscovered=2` |

Emulator triad:
- p2p `12D3KooWAv9RxQfYfbQDCLZtabfctRjM8AfyLFAWnxE1BXTt1CSx`
- pk `1056c4e93cbcd1ffa5030ba6ee8d8feb9b0647d973696a569c3d0fdaf0b88d5d`

Android log pull (Pixel, before unavailability):
`tmp/android_logpull_field_20260910T101148/`

## Remaining for perfect 0.4.0 (logs)

1. **Cell-only relay retest** — phone/emulator with no LAN path to Windows;
   expect relay via AWS **without** `identity_registration_missing`.
2. **BLE** — still hardware-gated on Windows MT7921.
3. AWS image should be rebuilt at `441a0214` for exact one-candidate parity
   (registration protocol already present on `e8c8f52b` image).
4. Rule-8 APPROVE for merge-to-main.

## Emulator

- AVD `scm_test_34` (API 34 google_apis x86_64), headless boot
- APK `app-debug.apk` installed (`10C43A79…` family / `f9a1f60b`)
- Use only when Pixel is offline
