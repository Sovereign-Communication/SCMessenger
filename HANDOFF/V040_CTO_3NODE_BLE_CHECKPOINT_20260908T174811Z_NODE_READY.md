# V040 CTO three-node BLE checkpoint - NODE_READY owner recheck

## Metadata

- Stage: `NODE_READY`
- Evidence capture timestamp: `2026-09-08T17:48:11Z`
- Operator/session: Freebuff `/cto` continuation; Windows/AWS owner lane
- CEO consumer: `HANDOFF/CEO_STATE.md`; that file was not edited
- Overall verdict: `BLOCKED`

This is a new immutable owner recheck. The Windows candidate node and AWS cloud
node are live and freshly queried. The operator owns the Pixel and has not
provided a Phase 2 completion capture in this checkpoint. No Android service,
Bluetooth, Wi-Fi, mobile-data, or probe action was issued by this owner lane.

## Complete evidence

The complete command text and output for this recheck is preserved at:

- `tmp/cto/V040_WINDOWS_AWS_OWNER_RECHECK_20260908T174811Z_COMPLETE_OUTPUT.md`

The prior tracked `NODE_READY` checkpoint remains preserved at:

- `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T171520Z_NODE_READY.md`

The prior complete Windows/AWS owner transcript, including the candidate build,
listener diagnostics, and startup markers, is preserved at:

- `tmp/cto/V040_WINDOWS_AWS_FINAL_OWNER_20260908T171520Z.md`

No evidence file was overwritten or relabeled.

## Fresh Windows result

The command `powershell.exe -NoProfile -NonInteractive` querying
`Win32_Process`, executable path, and SHA256 reported exactly one process:

- Process: `scmessenger-cli.exe`
- PID: `25856`
- Executable:
  `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\cto-win-build-85cb4c67\target\x86_64-pc-windows-msvc\release\scmessenger-cli.exe`
- Live process SHA256:
  `1A736AC97B74E7B2201CDCAEBDFCD9D8990647BE9ED01A382165954FFEDCE055`

The filtered `netstat -ano -p tcp` output reported that PID `25856` owns all
required local listeners and has established AWS connections:

- `0.0.0.0:9001` LISTENING
- `0.0.0.0:9002` LISTENING
- `127.0.0.1:9876` LISTENING
- `192.168.0.222:9001 -> 18.234.62.247:9001` ESTABLISHED
- `192.168.0.222:49365 -> 18.234.62.247:9001` ESTABLISHED

Fresh local API responses, all HTTP `200` with curl exit `0`:

- `/health`: `{"status":"healthy"}`
- `/version`: version `0.4.0`, `git_hash=85cb4c67`,
  `core_provenance=0.4.0 (85cb4c67:HEAD:1788883797)`
- `/api/identity`:
  - `identity_id=985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826`
  - `libp2p_peer_id=12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`
  - `public_key_hex=30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e`
  - `device_id=3069a066-2532-448d-a795-6e3a8b3392b2`
- `/api/diagnostics`: `running=true`, `connection_path_state=DirectPreferred`,
  `outbox_count=0`, one AWS peer, and the complete listener list is in the
  linked transcript.

The route is `/api/diagnostics`. A previous `/diagnostics` request returning
`404` was a wrong-route query, not a node failure.

## Fresh AWS result

The read-only Python EC2 inventory reported two instances matching the approved
`Name` tag filter:

- `i-0b735c4f26aea42ed`, `scm-always-on-node`, `stopped`, no public IP
- `i-0b41aab756eabd514`, `scm-always-on-node`, `running`, `18.234.62.247`

The running cloud node returned HTTP `200` with curl exit `0` for each endpoint:

- `/health`: `{"status":"healthy"}`
- `/version`: version `0.4.0`, exact full candidate
  `git_hash=85cb4c67feb03d27fa004a2be6b1ce65b030eb06`, and
  `core_provenance=0.4.0 (85cb4c67feb03d27fa004a2be6b1ce65b030eb06:cto/v040-candidate-2026-09-02:)`
- `/api/identity`:
  - `identity_id=37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006`
  - `libp2p_peer_id=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`
  - `public_key_hex=69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c`
  - `device_id=8a6b20bb-5f9f-46c4-8a6c-6aece03ce427`
- `/api/diagnostics`: `running=true`, `connection_path_state=DirectPreferred`,
  `outbox_count=0`, one Windows peer, and the complete listener list is in the
  linked transcript.

AWS executable SHA256 was not collected by this read-only HTTP/EC2 recheck and
remains `UNVERIFIED`.

## Artifact and provenance matrix

- Exact candidate source ref previously recorded:
  `origin/cto/v040-candidate-2026-09-02` at
  `85cb4c67feb03d27fa004a2be6b1ce65b030eb06`, tree
  `0989624382eb06e3792babeaf7f60333202357b9`.
- Live Windows executable is the isolated exact-candidate build and hashes to
  `1a736ac97b74e7b2201cdcaebdfcd9d8990647be9ed01a382165954ffedce055`.
- The older prescribed artifact remains untouched at
  `tmp/radio-85cb4c67/scmessenger-cli-85cb4c67.exe` and
  `tmp/radio-85cb4c67/windows-cli-artifact/scmessenger-cli.exe`; its recorded
  SHA256 is
  `d2f75243c0ff07d6b433068001ce5fddcf2df33066c32e2367f3be12ddfb8385` and its
  provenance file reports runtime `94cfe7d565f9a5ececd1084ab8d96d958a2baf00`.
- Pixel installed APK SHA256 from the earlier tracked baseline is
  `5090a835698c57e1e75def9dc70abf7597acda67bcba35fa9d681aa0da2daa47`; it was
  not queried in this Windows/AWS-only recheck.

## Three-node verdicts

| Node or gate | Verdict | Basis |
|---|---|---|
| Windows live candidate node | `PASS` | One process, exact candidate runtime, expected live hash, required listeners, healthy API, stable identity, and established AWS connection |
| AWS cloud node | `PASS` | One running approved tagged instance, healthy API, exact candidate version, stable identity, Windows peer, and direct-preferred diagnostics |
| Windows BLE startup/readiness | `UNVERIFIED` | Prior owner evidence has initialization/start markers; successful LE advertisement and transport ingress are not proven |
| Pixel Phase 2 service/BLE readiness | `OPERATOR_PENDING` | Android actions and evidence are outside this owner lane |
| BLE-only radio isolation | `UNVERIFIED` | No Android radio action or current Pixel radio capture was performed here |
| Same-candidate three-node certification | `BLOCKED` | Older prescribed Windows artifact mismatch remains explicit, AWS binary hash is unverified, and Pixel source provenance is not established here |
| Probe, correlation, cellular, cleanup, release | `UNVERIFIED` | No probe or later package phase was performed |

## Coordination handoff

Keep Windows PID `25856` and AWS instance `i-0b41aab756eabd514` at
`18.234.62.247` running. The next package gate is operator-owned Android Phase
2: use the app's real mesh-service control and provide a fresh service identity
and BLE initialization/advertisement/GATT-registration/active-scan capture.
After that evidence is supplied, this owner lane will re-query Windows and AWS
before any later package phase. No radio isolation or probe is being advanced by
this checkpoint.
