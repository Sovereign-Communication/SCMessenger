# BLE OFFLINE PROXIMITY DEMO PLAYBOOK (SHIP_PLAN D7)

Status: READY-FOR-OPERATOR (radio-readiness pre-verified 2026-08-25, see Adjacency Findings below)
Scope: Prove Windows <-> Pixel 6a message delivery over BLE GATT with NO WiFi/LAN/Internet path available.

## HARD SAFETY RULES

1. NEVER touch Windows network adapters or the Windows daemon process. The daemon stays up for the entire demo.
2. Only toggle **WiFi** on the Pixel. Bluetooth stays ON at all times (BLE transport + no radio surprises).
3. WIRELESS ADB DIES WHEN PHONE WIFI GOES DOWN. Before step 4 you MUST have USB adb connected and verified, or you lose all remote control of the phone until WiFi returns.
4. Do not enable airplane mode (kills BT as well as WiFi). Toggle WiFi only.

## PRE-VERIFIED STATE (2026-08-25, agent-verified — do not redo unless suspicious)

- Pixel 6a (`bluejay`, serial `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`): BT State ON, uptime stable, 0 crashes. Identity address `24:29:34:8B:84:22`.
- Windows daemon: BLE central scan ACTIVE (`scmessenger_cli::ble_mesh: BLE found matching peripheral ...` continuously). It sees the Pixel's rotating RPAs (e.g. `4F:73:8C:BD:F2:C0`, `51:4B:37:37:36:8D`) advertising the SCM service UUID. Android rotates its RPA every ~15 min — MAC churn in the logs is NORMAL, do not chase specific MACs. The identity MAC will never appear in scans; that is by design.
- One full inbound GATT delivery already observed in logs: `route="ble_windows_gatt" terminal_result="windows_notification_complete"` (12:47:33Z).
- Config `%APPDATA%\scmessenger\config.json`: `enable_ble=true`, `enable_mdns=true`, `network.enable_relay=true`.
- Drift baseline: `GET http://127.0.0.1:9876/api/drift-status` -> `{"state":"Dormant","store_size":0}`.

## HOW SEND PATH SELECTION WORKS (why this demo proves BLE)

`cli/src/api_axum.rs` `handle_send_message` (and the legacy `api.rs` equivalent):
1. Try `ble_mesh::send_ble_message(...)` FIRST (Windows GATT notification -> else GATT-central write to char 0xDF03, fragmented via `GattFragmenter`).
2. Only on BLE failure, fall back to libp2p swarm (`swarm_handle.send_message`) — LAN/mDNS/TCP/relay/bootstrap.

There is NO runtime "force BLE" flag. `config set enable_mdns false` (+ clearing bootstrap, disabling relay) would hard-disable the fallback but REQUIRES a daemon restart — out of scope here. Instead, this demo removes the fallback physically: with phone WiFi off there is no LAN/Internet route, so any successful delivery is BLE-only by construction. Corroborate with the `route=` field in the daemon log.

## PROCEDURE

### Phase 0 — Baselines (both radios up)

```powershell
# Windows side
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:9876/api/drift-status | Select -Expand Content   # expect Dormant, store_size small/0
Get-Content "$env:LOCALAPPDATA\scmessenger\logs\scm.log.2026-08-25-*" -Tail 5   # note current log file name
```

```powershell
# Phone side — verify BOTH transports
adb devices                       # wireless entry present
adb usb                           # SWITCH TO USB ADB NOW (keeps control through WiFi-off)
adb devices                       # USB serial 26261JEGR01896 must appear
adb shell cmd wifi status | Select-String "Wi-Fi is"
adb shell dumpsys bluetooth_manager | Select-Object -First 8   # State: ON
adb logcat -c                     # clear logcat ring buffer
```

CAPTURE EVIDENCE A: screenshot/copy of drift-status output + `adb devices` showing the USB serial.

### Phase 1 — Partition the phone (WiFi OFF)

```powershell
adb shell cmd wifi set-wifi-enabled disabled
adb shell cmd wifi status | Select-String "enabled"    # confirm disabled
adb devices                                            # wireless entry drops; USB entry remains — if not, STOP and re-enable WiFi
ping 192.168.0.129 -n 3                                # should FAIL now (phone unreachable on LAN)
```

Note the exact timestamp T_off. Verify the Windows daemon itself is still healthy:
```powershell
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:9876/api/drift-status | Select -Expand Content
```

### Phase 2 — BLE-only send while partitioned

```powershell
# Send from Windows to a known contact (use the exact recipient id/nickname already in contacts)
$body = '{"recipient":"<CONTACT_PEER_ID_OR_NICKNAME>","message":"D7-BLE-OFFLINE <utc-now>"}'
Invoke-WebRequest -UseBasicParsing -Method Post -ContentType application/json -Body $body http://127.0.0.1:9876/api/send
```

Then immediately:

```powershell
# Drift ledger check — should LEAVE Dormant and store_size should grow vs Phase 0
Start-Sleep 20
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:9876/api/drift-status | Select -Expand Content

# Windows-side BLE evidence (the load-bearing artifact)
Select-String -Path "$env:LOCALAPPDATA\scmessenger\logs\scm.log.2026-08-25-*" -Pattern "ble_gatt_central|ble_windows_gatt|windows_notification_complete|gatt_write_complete|decode_or_decrypt_error" | Select-Object -Last 30
```

What counts as SUCCESS on the Windows side, in order of strength:
1. `route="ble_windows_gatt" terminal_result="windows_notification_complete"` or `route="ble_gatt_central" terminal_result="gatt_write_complete"` stamped AFTER T_off — bytes physically left over the radio.
2. No `swarm` dial-success lines after T_off (fallback provably dead).
3. drift-state != `Dormant` and store_size increased (message parked in drift store pending sync).

Phone-side evidence (USB adb still works):
```powershell
adb logcat -d | Select-String -Pattern "scmessenger|Ble|BLE|GATT" | Select-Object -Last 40 > d7_phone_logcat.txt
adb shell dumpsys bluetooth_manager | Select-String -Pattern "State|Gatt|App" | Select-Object -First 15 >> d7_phone_logcat.txt
```

If `/api/send` returns 500 ("Failed to send message via BLE and Swarm") that is an honest negative result — record the log tail verbatim; do NOT retry-spam more than 3 times.

### Phase 3 — Restore

```powershell
adb shell cmd wifi set-wifi-enabled enabled
adb shell cmd wifi status | Select-String "connected"      # wait for Kana5G reassociation (~30s)
```

Re-establish wireless adb (USB can then be removed):
```powershell
adb tcpip 5555          # if needed; device may auto-re-advertise _adb-tls-connect
adb mdns services       # find adb-26261JEGR01896-..._adb-tls-connect._tcp
adb connect <phone-ip>:<port>
adb devices             # confirm wireless entry back
```

Post-restore convergence check (drift should drain/sync toward Dormant):
```powershell
foreach ($i in 1..6) { Start-Sleep 30; (Invoke-WebRequest -UseBasicParsing http://127.0.0.1:9876/api/drift-status).Content }
```

## EVIDENCE PACKAGE (collect into HANDOFF/evidence/D7/)

1. Phase 0 drift-status JSON (baseline) + timestamps.
2. Full daemon log excerpt from T_off-60s to restore+120s (the `route=` diagnostic lines are self-documenting).
3. `d7_phone_logcat.txt`.
4. Post-restore drift-status time series.
5. One-line verdict: DELIVERED_OVER_BLE / STORED_IN_DRIFT_ONLY / SEND_FAILED, with the terminal_result lines quoted.

## KNOWN BLOCKERS / RISKS (pre-identified)

- B1 (medium): Many recent inbound payloads log `route="ble_gatt_ingress" terminal_result="decode_or_decrypt_error"` (including malformed fragment_index/count pairs). Ingress decrypt/decode of phone->Windows payloads is flaky. This affects the REVERSE direction (phone->Windows); the Windows->phone direction (`send_ble_message`) is independent. If demo direction Windows->phone works, B1 does not block D7; log B1 as a follow-up bug either way.
- B2 (structural): No runtime BLE-only override exists; forcing it via config needs a daemon restart (forbidden mid-session). Mitigated by physical partition making swarm fallback unreachable.
- B3 (operational): Wireless adb shares the phone's WiFi — hence the mandatory `adb usb` in Phase 0. If USB is unavailable, the operator loses remote control while WiFi is off and must re-enable WiFi on-device manually.
- B4 (minor): RPA rotation means a fresh scan-connect cycle may take up to ~15 min if the peripheral cache goes stale mid-demo; be patient rather than toggling BT.
