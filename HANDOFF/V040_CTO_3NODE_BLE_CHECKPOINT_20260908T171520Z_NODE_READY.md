# V040 CTO three-node BLE checkpoint - NODE_READY

## Metadata

- Stage: `NODE_READY`
- Evidence capture stamp: `20260908T171520Z`; the first UTC command in the snapshot reported `2026-09-08T17:15:21Z`
- Operator/session: Freebuff `/cto` continuation; Windows/AWS owner lane
- CEO consumer: `HANDOFF/CEO_STATE.md`; this checkpoint is the coordination artifact and that file was not edited
- Overall verdict: `BLOCKED`

The Windows candidate node and AWS cloud node are live and freshly checked. The
Pixel remains operator-owned. Pixel Phase 2 service/BLE readiness, radio
isolation, transport correlation, and same-candidate certification are not
claimed.

## Exact commands and complete outputs

The complete Windows/AWS owner transcript, including command text, exit statuses,
and command output, is preserved at:

- `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`

The complete earlier fresh matrix, including the Pixel package baseline, is
preserved at:

- `tmp/cto/V040_CONTINUE_FRESH_REDERIVATION_20260908T165144Z.md`

The following commands were run in order in the owner snapshot. The linked
transcript is the complete output source; the individual JSON and text files
below preserve the machine-readable or large command outputs without excerpts.

1. `date -u '+%Y-%m-%dT%H:%M:%SZ'`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
   - Observed: `2026-09-08T17:15:21Z`
2. `git branch --show-current`; `git rev-parse HEAD`; `git rev-parse 'HEAD^{tree}'`; `git rev-parse origin/cto/v040-candidate-2026-09-02`; `git rev-parse 'origin/cto/v040-candidate-2026-09-02^{tree}'`
   - Exit: `0` for each
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
3. `sha256sum tmp/cto-win-build-85cb4c67/target/x86_64-pc-windows-msvc/release/scmessenger-cli.exe`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
4. `sha256sum tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe`; `sha256sum tmp/radio-85cb4c67/windows-cli-artifact/scmessenger-cli.exe`; `cat tmp/radio-85cb4c67/windows-cli-artifact/cli-provenance.txt`
   - Exit: `0` for each
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
5. `powershell.exe -NoProfile -NonInteractive -Command '$p=@(Get-CimInstance Win32_Process | Where-Object { $_.Name -eq "scmessenger-cli.exe" }); ...'`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
6. `netstat -ano -p tcp > tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_netstat.txt`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_netstat.txt`
7. `powershell.exe -NoProfile -NonInteractive -Command '$p=@(Get-Process -Name scmessenger-cli ...); ... Get-FileHash ...'`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
8. `curl -sS -m 15 http://127.0.0.1:9876/{health,version,api/identity,diagnostics,connection-path-state,discovery/status}`
   - Exit: `0` and HTTP `200` for each endpoint
   - Complete outputs:
     - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_health.json`
     - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_version.json`
     - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_identity.json`
     - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_diagnostics.json`
     - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md` for the path-state and discovery responses
9. `grep -aEn 'ble_daemon|ble_windows|ble_mesh|BLE|GATT|Service Provider|advertis|scanning|scan|adapter' tmp/cto-win-build-85cb4c67/windows-cli-candidate-live-20260908T162721Z.log > tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_ble_markers.txt`
   - Exit: `0`
   - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_ble_markers.txt`
10. Python EC2 `describe_instances` using `.codebuff_deploy/aws/scm_session.py`, filtered by the three approved node Name tags
    - Exit: `0`
    - Complete output: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`
11. `curl -sS -m 15 http://18.234.62.247:9876/{health,version,api/identity,diagnostics}`
    - Exit: `0` and HTTP `200` for each endpoint
    - Complete outputs:
      - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_health.json`
      - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_version.json`
      - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_identity.json`
      - `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_diagnostics.json`
12. Pixel baseline commands were run earlier in this session before the explicit Android ownership boundary; their complete output is `tmp/cto/V040_CONTINUE_FRESH_REDERIVATION_20260908T165144Z.md`. No ADB, Android UI, Android service, Bluetooth, Wi-Fi, or mobile-data command was issued in the final Windows/AWS owner snapshot.

## Repository provenance

- Shared checkout branch: `cto/t2-disk-ruling-2026-08-31`
- Shared checkout HEAD: `29a9b8e9bce6b69ff0c2671b420a531e38bbdeff`
- Shared checkout HEAD tree: `d6f374ae67af46a4acca04eda0747bfbd7cfdc90`
- Selected candidate ref: `origin/cto/v040-candidate-2026-09-02`
- Candidate commit/tree: `85cb4c67feb03d27fa004a2be6b1ce65b030eb06` / `0989624382eb06e3792babeaf7f60333202357b9`
- Candidate source worktree: `tmp/cto-win-build-85cb4c67`
- Candidate source worktree status: detached `HEAD`, clean in the owner snapshot
- Candidate source worktree commit/tree: `85cb4c67feb03d27fa004a2be6b1ce65b030eb06` / `0989624382eb06e3792babeaf7f60333202357b9`
- Relevant tags from the fresh listing: `freebuff-snapshot/245d21ce-450f-4f3b-90d5-eb6b7c119d89`, `freebuff-snapshot/d4ad9f34-e5c0-4bdd-8a18-1911adefd62a`, `v0.1.0`, `v0.1.1`, `v0.1.9`, `v0.2.1`, `v0.3.5`, `v0.4.0-rc.1`
- Final `v0.4.0` tag: absent from the fresh listing

### Artifact hashes

- Live Windows candidate executable, isolated exact-candidate build:
  `1a736ac97b74e7b2201cdcaebdfcd9d8990647be9ed01a382165954ffedce055`
  - Path: `tmp/cto-win-build-85cb4c67/target/x86_64-pc-windows-msvc/release/scmessenger-cli.exe`
- Live Windows process hash, freshly computed from the process path:
  `1A736AC97B74E7B2201CDCAEBDFCD9D8990647BE9ED01A382165954FFEDCE055`
- Prescribed older recorded Windows artifact:
  `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385`
  - Paths: `tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe` and `tmp/radio-85cb4c67/windows-cli-artifact/scmessenger-cli.exe`
  - Its recorded provenance file reports `git-sha: 94cfe7d565f9a5ececd1084ab8d96d958a2baf00` and runtime `94cfe7d`.
- Pixel installed APK from the fresh baseline:
  `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47`
  - The baseline command reported equality with `tmp/radio-85cb4c67/installed-base-2026-09-08.apk`.
- AWS executable hash: not collected through the read-only HTTP/EC2 owner checks; `UNVERIFIED`.

The live process is therefore the exact-candidate source build and does not use
the older `94cfe7d` artifact. The two Windows hashes are intentionally retained
as a provenance discrepancy; the older artifact was not overwritten or relabeled.

## Three-node matrix

| Node | Reachability | Version/commit/artifact | Identity | Verdict |
|---|---|---|---|---|
| AWS cloud node | EC2 inventory found one running tagged instance `i-0b41aab756eabd514` at `18.234.62.247`; `/health` HTTP `200` | `/version` HTTP `200`, version `0.4.0`, exact candidate commit `85cb4c67feb03d27fa004a2be6b1ce65b030eb06`; binary hash not collected | `identity_id=37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006`; `libp2p_peer_id=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`; public key `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c` | `PASS` for fresh API/version/identity; not a BLE or delivery pass |
| Windows CLI | One `scmessenger-cli.exe` process, PID `25856`; `127.0.0.1:9876`, `0.0.0.0:9001`, and `0.0.0.0:9002` are owned by PID `25856`; an established `192.168.0.222:9001` connection reaches `18.234.62.247:9001` | `/health` HTTP `200`; `/version` reports `0.4.0`, `git_hash=85cb4c67`, `core_provenance=0.4.0 (85cb4c67:HEAD:1788883797)`; live process hash `1a736ac9...`; older prescribed artifact remains `d2f75243...` and reports `94cfe7d` | `identity_id=985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826`; `libp2p_peer_id=12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`; public key `30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e` | `PASS` for live candidate node/API/identity/listener ownership; `BLOCKED` for recorded-artifact reconciliation and same-candidate certification |
| Pixel 6a | Fresh baseline found one authorized ADB device `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`, model `Pixel 6a`, device `bluejay`; package installed | Android `17`/API `37`; package `0.4.0`, versionCode `14`, targetSdk `35`; installed APK SHA256 `5090a835...` equal to local baseline artifact | Last identity observed in the prior tracked preflight: `identity_id=9a23057410a35584e79747fb0357821bf07bb6bd9a757d1c3e827edea3faab36`; `libp2p_peer_id=12D3KooWR9ioPPRJ2tGPbWj9NVKXAve2iwZX4csLbDd1Tn6Hpi3B`; public key `e3d4aaecf5b02fc7112a0d17b90d1dbe319a9ef7eef93bc7f099821e935ec7fa`. This identity was not re-queried in the Windows/AWS owner snapshot. | `PASS` for the fresh package baseline only; `UNVERIFIED` for current identity, app service/BLE readiness, and Phase 2 completion |

## Stage-specific evidence

- Android capture path: fresh baseline transcript `tmp/cto/V040_CONTINUE_FRESH_REDERIVATION_20260908T165144Z.md`; the operator owns the next service and radio capture.
- Windows capture path: live process log `tmp/cto-win-build-85cb4c67/windows-cli-candidate-live-20260908T162721Z.log`; complete owner transcript `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`; complete netstat `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_netstat.txt`.
- AWS capture/API output paths: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_health.json`, `_aws_version.json`, `_aws_identity.json`, `_aws_diagnostics.json`.
- Windows BLE startup markers: `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_ble_markers.txt` contains adapter probe `terminal_result="available"`, GATT Service Provider initialization, a Windows peripheral LE advertisement start marker, and BLE scan active. The same marker set says the CLI GATT central is used and peripheral advertising via btleplug is not enabled. Actual successful LE advertisement and transport ingress are therefore `UNVERIFIED`, not inferred from the start marker.
- Windows diagnostics: `running=true`, `connection_path_state=DirectPreferred`, `custody_audit_count=5058`, `outbox_count=0`, one AWS peer, and complete listeners are in `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_diagnostics.json`.
- AWS diagnostics: `running=true`, `connection_path_state=DirectPreferred`, `custody_audit_count=64`, `outbox_count=0`, one Windows peer, and complete listeners are in `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z_aws_diagnostics.json`.
- Message label/content: not created.
- Message ID: not created.
- BLE-specific ingress/decrypt evidence: not collected; no probe was sent.
- Competing TCP/mDNS/Wi-Fi/cellular/relay evidence: Windows is currently `DirectPreferred` and has a live AWS TCP connection. The Pixel baseline reported Wi-Fi enabled and connected to `KG` at `192.168.0.111`; its settings file reported BLE, Wi-Fi Aware, Wi-Fi Direct, and internet enabled. BLE-only isolation was not attempted by this owner lane.
- Radio state: Pixel baseline reported global Bluetooth `1` and airplane mode `0`; Wi-Fi was enabled and connected. Mobile-data state was not independently established. Current radio-isolation verdict: `UNVERIFIED`.
- Cleanup state: no node, Android service, contact, route, or radio cleanup action was issued by this owner lane. Windows PID `25856` and AWS instance remain running.

## Verdicts, blockers, and next action

- Windows live candidate node setup: `PASS` for one-process ownership, listener ownership, health, exact candidate runtime provenance, stable identity, AWS connection, and the recorded capture paths above.
- Windows BLE readiness: `UNVERIFIED`; initialization/start markers exist, but actual LE advertisement success is not proven and the log explicitly says peripheral advertising via btleplug is not enabled.
- AWS cloud node observer: `PASS` for fresh health, exact candidate version, identity, live Windows peer, and API diagnostics. AWS binary hash remains `UNVERIFIED`.
- Pixel Phase 2 service/BLE readiness: `OPERATOR_PENDING`; this controller did not start or stop the Android mesh service and did not issue a shell-forced service action.
- BLE-only isolation: `UNVERIFIED`; the Pixel baseline was still on Wi-Fi and had internet-enabled mesh settings.
- Same-candidate three-node certification: `BLOCKED`; the live Windows source build is exact candidate `85cb4c67`, but the older prescribed artifact remains a different hash with recorded `94cfe7d` provenance, and Pixel source provenance is not established by APK hash equality alone.
- Probe/correlation/cellular/release verdicts: `UNVERIFIED`; no probe was created or sent, no transport-specific correlation was attempted, and no final `v0.4.0` tag is present.

Exact next action: the operator completes Android Phase 2 through the real app
mesh-service control and supplies a fresh service identity/BLE readiness capture.
I will keep PID `25856` and AWS `18.234.62.247` running and re-query Windows/AWS
before any later package phase. No radio isolation or probe is authorized by
this checkpoint. The package final handoff is not created yet; it is required at
`CLEANUP` after the preceding gates complete.
