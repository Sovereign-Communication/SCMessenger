# Runbook: Pixel / Android passive verification

Status: Active
Owner: operator + CTO/controller seats
Rule date: 2026-09-11
Applies to: every agent (MiMo Desktop, Claude, Freebuff, MAC lane)

## Why

Seats that actively drive the Pixel UI (taps, Settings toggles, phone-side
sends) invent evidence the operator did not observe, race onboarding dialogs,
and have already cost identity wipes (`pm clear`). Install + log-pull is the
only seat-driven Pixel action. The **operator** performs the human path.

## Seat checklist (every APK install)

1. Preflight if destructive: `$env:MIMO_PYTHON scripts/recovery_preflight.py --action pm_clear`
   (almost always BLOCK — do not wipe).
2. `adb devices` — confirm serial.
3. `adb -s <serial> install -r <apk>` (replace-install; preserves identity).
4. `adb -s <serial> shell am start -n com.scmessenger.android/.ui.MainActivity`
   only if the process is not already up.
5. **Stop.** Notify operator: "APK installed; please open the app / peers list
   and exercise the path under test."
6. Wait for operator confirmation, then pull logs (below).

## Log pull (passive)

```powershell
$ser = "<serial>"
$out = "tmp/evidence/<runid>"
adb -s $ser logcat -d -v time > "$out/logcat.txt"
adb -s $ser exec-out run-as com.scmessenger.android cat files/logs/scmessenger-mesh.log > "$out/mesh.log"
adb -s $ser exec-out run-as com.scmessenger.android cat files/mesh_diagnostics.log > "$out/diag.log"
adb -s $ser exec-out run-as com.scmessenger.android cat files/ledger.json > "$out/ledger.json"
```

`mesh.log` is often **UTF-16** (`FF FE`). Decode as UTF-16 before grepping.
FileLoggingTree carries Timber tags: `GHOST-IDENTITY-001`, `UNIFICATION
loadPeers`, `NODE-RETENTION-001`.

## What the seat must never do on Pixel

- `input tap` / `input swipe` / `input keyevent` for app navigation
- send messages via UI or `am broadcast` to fake traffic
- toggle mesh / Bluetooth / WiFi from the app Settings
- `pm clear com.scmessenger.android` without identity export +
  `recovery_preflight.py --action pm_clear` + explicit operator approval
- force Dashboard `loadPeers` — ask the operator to open the peers list

## What the seat may do

- install APKs (`install -r`)
- start/stop the app process for a clean launch (`am start`, `force-stop`)
- pull logs / ledger / diagnostics read-only
- install then wait

## Windows / AWS (not Pixel)

Actively drivable: CLI, HTTP API (`:9876/api/*`), ssh + docker on AWS.
Still gate destructive actions via `recovery_preflight.py`.

## Verification of a fix (pattern)

1. Seat builds + installs APK.
2. Operator exercises the path (or just opens the mesh/peers list).
3. Seat pulls logs and records PASS/FAIL with file:line evidence.
4. Incomplete log coverage = **UNVERIFIED**, never PASS.

Example (GHOST-IDENTITY-001, 2026-09-11):

```
GHOST-IDENTITY-001 skip auto-subscribe ghost peer topic: /scmessenger/peer/577fd171…/v1
Auto-subscribing to discovered topic: /scmessenger/peer/30d0fa67…/v1
```
Ghost skipped; proven Windows topic still subscribed. Evidence:
`tmp/rca_577fd/phone_meshlog_gate.txt`.
