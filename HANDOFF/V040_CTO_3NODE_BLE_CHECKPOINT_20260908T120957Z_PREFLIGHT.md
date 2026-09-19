# V040 CTO three-node BLE checkpoint — PREFLIGHT

## Metadata

- Stage: `PREFLIGHT`
- UTC timestamp: `2026-09-08T12:09:40Z` (fresh Git/process checks); final bounded device checks through `2026-09-08T12:09:57Z`
- Operator/session: Freebuff `/cto` continuation
- Overall verdict: `BLOCKED`

## Exact commands and complete outputs

The complete fresh preflight transcript is preserved at `tmp/cto/V040_PREFLIGHT_20260908T120957Z_COMPLETE_OUTPUT.md`. The following bounded commands were run in that transcript; each has its command, output, and exit status:

1. Command: `git status --short --branch`
   - Exit: `0`
   - Complete output: checkout `cto/t2-disk-ruling-2026-08-31...origin/cto/t2-disk-ruling-2026-08-31 [gone]`; the shared checkout contains pre-existing staged, modified, and untracked files, including application and deployment artifacts. Those files were not altered by this session.
2. Command: `git branch --show-current`
   - Exit: `0`
   - Complete output: `cto/t2-disk-ruling-2026-08-31`
3. Command: `git rev-parse HEAD`
   - Exit: `0`
   - Complete output: `0e0d54dab43a3ab375e8e1f799d8d4a4168033de`
4. Command: `git rev-parse HEAD^{tree}`
   - Exit: `0`
   - Complete output: `2f371cef5115bd79bd0e97e2240011586723775a`
5. Command: `git rev-parse origin/cto/v040-candidate-2026-09-02`
   - Exit: `0`
   - Complete output: `85cb4c67feb03d27fa004a2be6b1ce65b030eb06`
6. Command: `git rev-parse origin/cto/v040-candidate-2026-09-02^{tree}`
   - Exit: `0`
   - Complete output: `0989624382eb06e3792babeaf7f60333202357b9`
7. Command: `git tag -l | sort`
   - Exit: `0`
   - Complete output:
     - `freebuff-snapshot/245d21ce-450f-4f3b-90d5-eb6b7c119d89`
     - `freebuff-snapshot/d4ad9f34-e5c0-4bdd-8a18-1911adefd62a`
     - `v0.1.0`
     - `v0.1.1`
     - `v0.1.9`
     - `v0.2.1`
     - `v0.3.5`
     - `v0.4.0-rc.1`
     - No final `v0.4.0` tag is present in this output.
8. Command: `Python EC2 describe_instances` using `.codebuff_deploy/aws/scm_session.py`, filtered by Name tag `scm-always-on-node`, `scm-node`, `scmessenger`
   - Exit: `0`
   - Complete output:
     - `instance_id=i-0b735c4f26aea42ed name=scm-always-on-node state=stopped public_ip=- private_ip=172.31.31.151 launch_time=2026-08-31 03:40:10+00:00`
     - `instance_id=i-0b41aab756eabd514 name=scm-always-on-node state=running public_ip=18.234.62.247 private_ip=172.31.18.74 launch_time=2026-09-07 16:20:12+00:00`
9. Command: `curl -sS -m 15 -w '\nHTTP_STATUS=%{http_code}\n' http://18.234.62.247:9876/{health,version,api/identity}`
   - Exit: `0` for each request
   - Complete output:
     - `/health`: `{"status":"healthy"}`; `HTTP_STATUS=200`
     - `/version`: `{"build_time":"","core_provenance":"0.4.0 (85cb4c67feb03d27fa004a2be6b1ce65b030eb06:cto/v040-candidate-2026-09-02:)","git_hash":"85cb4c67feb03d27fa004a2be6b1ce65b030eb06","version":"0.4.0"}`; `HTTP_STATUS=200`
     - `/api/identity`: `device_id=8a6b20bb-5f9f-46c4-8a6c-6aece03ce427`, `identity_id=37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006`, `libp2p_peer_id=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, `public_key_hex=69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c`, `initialized=true`; `HTTP_STATUS=200`
10. Command: `powershell.exe -NoProfile -Command '$p=Get-Process -Name "scmessenger-cli" -ErrorAction SilentlyContinue; ...'`
    - Exit: `0`
    - Complete output: `NO_PROCESS scmessenger-cli`
11. Command: `netstat -ano -p tcp | grep ':9876'`
    - Exit: `0` after explicit no-match handling
    - Complete output: `NO_LOCAL_9876_LISTENER_MATCH`
12. Command: `curl -sS -m 5 -w '\nHTTP_STATUS=%{http_code}\n' http://127.0.0.1:9876/health`
    - Exit: `7`
    - Complete output: `curl: (7) Failed to connect to 127.0.0.1:9876 ...`; `HTTP_STATUS=000`
13. Command: `sha256sum tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe tmp/radio-85cb4c67/installed-base-2026-09-08.apk`
    - Exit: `0`
    - Complete output:
      - `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385  tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe`
      - `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47  tmp/radio-85cb4c67/installed-base-2026-09-08.apk`
14. Command: `adb devices -l`
    - Exit: `0`
    - Complete output: `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp device product:bluejay model:Pixel_6a device:bluejay transport_id:29`
15. Command: `adb -s <selected> shell getprop {ro.product.model,ro.product.device,ro.build.version.release,ro.build.version.sdk}`
    - Exit: `0` for each
    - Complete output: `Pixel 6a`, `bluejay`, `17`, `37`
16. Command: `adb -s <selected> shell pm path com.scmessenger.android`
    - Exit: `0`
    - Complete output: `package:/data/app/~~EwhaJ2zV_TeoXrrMByE1fw==/com.scmessenger.android-qByZxX6GRIGz-yJKRW8olA==/base.apk`
17. Command: `adb -s <selected> exec-out cat <installed APK> | sha256sum`
    - Exit: `0`
    - Complete output: `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47  -`
18. Command: `adb -s <selected> shell dumpsys package com.scmessenger.android` with package metadata selection
    - Exit: `0`
    - Complete output: `versionCode=14`, `versionName=0.4.0`, `targetSdk=35`, `lastUpdateTime=2026-09-07 17:58:29`, `firstInstallTime=2026-09-06 23:33:39`, `dataDir=/data/user/0/com.scmessenger.android`
19. Command: `adb -s <selected> shell run-as com.scmessenger.android cat files/mesh_settings.json`
    - Exit: `0`
    - Complete output: `relay_enabled=true`, `ble_enabled=true`, `wifi_aware_enabled=true`, `wifi_direct_enabled=true`, `internet_enabled=true`, `discovery_mode=Normal`
20. Command: bounded rotated diagnostics extraction and local grep for `SC_IDENTITY_OWN|getIdentityInfo: result|BLE GATT identity beacon updated`
    - Exit: `0`
    - Complete output includes Pixel self identity `p2p_id=12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B`, public key `e3d4aaecf5b02fc7112a0d17b90d1dbe319a9ef7eef93bc7f099821e935ec7fa`, identity ID `9a23057410a35584e79747fb0357821bf07bb6bd9a757d1c3e827edea3faab36`, and repeated GATT beacon updates for that peer.
21. Command: attempted broad PowerShell/CIM process and socket query
    - Exit: timed out after 60 seconds; this failed diagnostic is preserved in the session output and was not used as evidence.
22. Command: incorrectly quoted remote `adb shell run-as ... grep -E ...` command in the consolidated transcript
    - Exit: `127`
    - Complete output: `/system/bin/sh: ignoring: inaccessible or not found` and local grep errors caused by shell quoting. This was superseded by command 20 and is not used as positive evidence.

## Repository provenance

- Branch: `cto/t2-disk-ruling-2026-08-31`
- HEAD: `0e0d54dab43a3ab375e8e1f799d8d4a4168033de`
- HEAD tree: `2f371cef5115bd79bd0e97e2240011586723775a`
- Selected candidate ref: `origin/cto/v040-candidate-2026-09-02`
- Candidate commit/tree: `85cb4c67feb03d27fa004a2be6b1ce65b030eb06` / `0989624382eb06e3792babeaf7f60333202357b9`
- Relevant tags: `v0.4.0-rc.1` exists; final `v0.4.0` is absent.

## Three-node matrix

| Node | Reachability | Version/commit/artifact | Identity | Verdict |
|---|---|---|---|---|
| AWS cloud node | Reachable at `18.234.62.247:9876`; `/health` 200 | `/version` reports `0.4.0`, commit `85cb4c67feb03d27fa004a2be6b1ce65b030eb06`; no local AWS binary hash collected | `identity_id=37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006`; `libp2p=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`; public key `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c` | `PASS` for API reachability/version identity read; not a BLE evidence pass |
| Windows CLI | Local API unreachable (`curl` exit `7`, HTTP `000`); no matching process; no local `:9876` listener | Expected artifact `tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe`; SHA256 `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385`; runtime version/identity unavailable | Historical prior evidence only names peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`; this session does not treat it as fresh runtime identity | `BLOCKED` |
| Pixel 6a | Authorized ADB device `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`; app PID `11582`; package installed | Android 17/API 37, package `0.4.0`/versionCode `14`; installed APK SHA256 `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47`, matching local pulled-base artifact | `identity_id=9a23057410a35584e79747fb0357821bf07bb6bd9a757d1c3e827edea3faab36`; `libp2p=12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B`; public key `e3d4aaecf5b02fc7112a0d17b90d1dbe319a9ef7eef93bc7f099821e935ec7fa` from app-owned diagnostics | `PASS` for ADB/package/identity read; not radio-isolated |

## Stage-specific evidence

- Android capture path: live authorized ADB device and app-owned `files/mesh_diagnostics.log` plus rotated files; prior local artifacts under `tmp/radio-85cb4c67/` were not used as current BLE proof.
- Windows capture path: `tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe` hash verified; no fresh runtime log was started because the package requires stopping at unreachable Windows.
- AWS capture/API output path: live read-only HTTP output from `http://18.234.62.247:9876/{health,version,api/identity}`; the IP came from the EC2 API query in command 8.
- Message label/content: not created; no probe sent.
- Message ID: not created.
- BLE-specific ingress/decrypt evidence: not collected. Existing device diagnostics show historical self advertisement/GATT initialization, but that is insufficient for this run.
- Competing TCP/mDNS/Wi-Fi/cellular/relay evidence: Pixel `mesh_settings.json` reports Wi-Fi, Wi-Fi Aware, Wi-Fi Direct, and internet enabled; `cmd wifi status` reports Wi-Fi connected to `KG` at `192.168.0.111`; isolation was not attempted.
- Radio state: Bluetooth enablement was not independently queried in this preflight; app settings report `ble_enabled=true`. Airplane mode `0`; Wi-Fi enabled and connected.
- Cleanup state: no changes made; no capture or node process started; Pixel connectivity remains as found.

## Blockers and next action

- Blockers:
  1. Windows CLI is not running and `127.0.0.1:9876` is unreachable. This is the package's explicit Phase 0 stop condition.
  2. Same-candidate fleet provenance cannot be certified from this checkpoint because Windows runtime API/identity is unavailable, and the local checkout is not the candidate commit.
  3. Pixel is not BLE-isolated: Wi-Fi is enabled and connected; radio isolation was not attempted because the Windows stop condition fired first.
  4. No final `v0.4.0` tag exists in the fresh tag listing.
- Evidence still UNVERIFIED: Windows runtime identity/version, exact live Windows-to-candidate artifact provenance, BLE-only isolation, one-probe BLE transport correlation, cellular, same-candidate certification, final tag/release readiness.
- Exact next action: operator or authorized Windows lane must start exactly one approved candidate Windows CLI instance, then rerun Phase 0/Phase 1 fresh checks. Do not start it from this controller session; the package says `/cto` stops at the first failed precondition.
- Final handoff path: `HANDOFF/V040_CTO_3NODE_BLE_FINAL_HANDOFF_20260908T120957Z.md` (blocked preflight handoff).
