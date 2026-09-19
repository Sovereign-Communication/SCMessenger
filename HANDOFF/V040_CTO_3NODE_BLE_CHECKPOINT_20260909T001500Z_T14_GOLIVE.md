# V040 CTO three-node checkpoint - T14 + BLE-01 go-live (Windows node live, APK staged)

## Metadata

- Stage: `GOLIVE_T14_BLE01` (live node cutover + artifact staging; NOT three-node completion)
- UTC timestamp: `2026-09-09T00:15:00Z`
- Operator/session: Freebuff `/cto`; operator rulings in force: all work locally
  (no dispatch/worktrees), no commit yet, no firewall changes, operator drives
  Android/Pixel (no install, no Pixel contact this session)
- Branch: `cto/t2-disk-ruling-2026-08-31`, HEAD `ba474a7a1cd1b5e7ef87cdbc1c4a7069e091784b`
  (uncommitted working tree: T14 diff + BLE-01 diff + records)

## 1) Release build + provenance (exe hash is the authority; /version recorded)

- New Windows CLI: `target/release/scmessenger-cli.exe`
  - SHA256: `829efe2cd1bab2cd33ec36ccdf6dc622abeee296f54b181f1d1b2b5067bc1a5d`
  - /version: `0.4.0 (ba474a7a:cto/t2-disk-ruling-2026-08-31:1788910325)`, build_time
    `2026-09-08T23:32:05Z`, git_hash `ba474a7a`
  - Build: `cargo build --release -p scmessenger-cli` under `scripts/build_lock.py`
    (holder `cto-t14-golive`), log `tmp/cto/T14_GOLIVE/build_release_20260908T233157Z.log`
- Previous (replaced) node binary: `tmp\cto-win-build-85cb4c67\target\x86_64-pc-windows-msvc\release\scmessenger-cli.exe`
  - SHA256: `1a736ac97b74e7b2201cdcaebdfcd9d8990647be9ed01a382165954ffedce055`
  - /version was `85cb4c67`; process was PID 25856, started 06:27:22Z, argv `start`
- The old binary is UNTOUCHED at its recorded path (it lives in a separate build
  tree; the repo release build did not clobber it) and remains the rollback target.

## 2) Cutover (stop/relaunch authorized by operator)

Sequence: pre-stop captures (health/version/identity/diagnostics into
`tmp/cto/T14_GOLIVE/`) -> `taskkill /PID 25856 /F` (old logs preserved:
`tmp/cto-win-build-85cb4c67/windows-cli-candidate-live-20260908T162721Z.log`, 812KB) ->
launch exactly one new instance from `target\release\scmessenger-cli.exe`.

- Config edit BEFORE stop: `%APPDATA%\scmessenger\config.json` gained
  `"external_addr": "147.81.41.188:9001"`; backup
  `config.json.bak-t14golive-20260908T234155Z`; all other fields unchanged.
- Transparency record: FIRST relaunch came up healthy (PID 12852) but with
  `peers: []` - the old launch shell carried `SC_BOOTSTRAP_NODES` env
  (`bootstrap.rs::default_bootstrap_nodes` reads it before build-time overrides;
  `config.json bootstrap_nodes` is empty). Root-caused from the old log line
  "Dialing 2 bootstrap node(s)"; restarted the new instance ONCE with the exact
  env replicated. First-launch log preserved at `tmp/cto/T14_GOLIVE/new-node-out.log`.
- Final instance: PID **16548**, one process owning 127.0.0.1:9876 +
  0.0.0.0:9001 + 0.0.0.0:9002 (+ [::]:9001), exe
  `C:\Users\SCM\Documents\GitHub\SCMessenger\target\release\scmessenger-cli.exe`.
- Rollback path was armed but never needed (auto-rollback script logic was in
  the first cutover run; the only restart was the bootstrap-env correction).

### EXACT relaunch configuration (recorded for reproducibility)

- Exe: `C:\Users\SCM\Documents\GitHub\SCMessenger\target\release\scmessenger-cli.exe`
- Argv: `start`
- Working dir: `C:\Users\SCM\Documents\GitHub\SCMessenger`
- Env: `SC_BOOTSTRAP_NODES="/ip4/127.0.0.1/tcp/19001,/ip4/18.234.62.247/tcp/9001"`
  (order as observed in the old node's log; 19001 currently has no listener -
  dial fails fast, AWS dial unaffected)
- Config: `%APPDATA%\scmessenger\config.json` with `external_addr=147.81.41.188:9001`
- Data dir: default `%LOCALAPPDATA%\scmessenger` (no SCMESSENGER_DATA_DIR)
- LAUNCHER script (env-capable, reusable):
  `tmp/cto/T14_GOLIVE/launch_node_env.ps1`
- WARNING: the AWS bootstrap link depends on the `SC_BOOTSTRAP_NODES` env var,
  NOT on persistent config. Any future restart without it will boot healthy but
  peerless (observed and documented in the first relaunch attempt).

## 3) Live verification - ALL 10 CHECKS PASS (evidence in tmp/cto/T14_GOLIVE/)

| Check | Result | Evidence |
| --- | --- | --- |
| /health 200 healthy | PASS | `health_final.json` |
| /version git_hash ba474a7a (new tree) | PASS | `version_final.json` |
| /api/identity stable vs pre-stop | PASS | `identity_before.json` == `identity_final.json` (`985a25f9...` / `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`) |
| Configured external address wins | PASS | `diagnostics_final.json`: `external_addrs == ["147.81.41.188:9001"]` exactly |
| No ephemeral port advertised | PASS | single entry, port 9001 only (pre-cutover had 2 entries incl. :443 artifact) |
| AWS peer connected | PASS | peers == [`12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`]; log shows "Connected to ... via /ip4/18.234.62.247/tcp/9001" (`node-out-20260908T235102Z.log`) |
| AWS node healthy | PASS | `aws_health_final.json` (`18.234.62.247:9876`) |
| AWS version parity | PASS | `aws_version_final.json`: 85cb4c67 (cloud node intentionally still on candidate build) |
| Persistent store continuity | PASS | `custody_audit_count` 5073 across cutover; identical history_stats |
| Exactly one node process, ports owned | PASS | netstat + CIM probe, PID 16548 |

## 4) APK with BLE-01 scanner fix (STAGED ONLY - no install, no Pixel contact)

- Path: `android/app/build/outputs/apk/debug/app-debug.apk`
- SHA256: `2f07ed917179463d7b774ff4731d4feed0844e6965c76fb2fa3ead9a3547372b`
- Size: 71,568,015 bytes; versionName `0.4.0`, versionCode `14`
  (`output-metadata.json`, applicationId `com.scmessenger.android`)
- Build: `gradlew.bat assembleDebug` under build lock (holder `cto-ble01-apk2`),
  `BUILD SUCCESSFUL in 5m 2s`, log `tmp/cto/T14_GOLIVE/apk_build_bg.log`
- Source inputs into this APK:
  - `BleScanner.kt` SHA256 `32b68d7d34c58c08e2901e374fb296272ee97beb49798495deccd95c9f121961`
  - `BleScannerTest.kt` SHA256 `f8899bb150c58dab77aa28a83e3d128ef1a8a530120d3a5246671a8a7916da37`
- Prior (installed on Pixel) APK hash for comparison: `5090a835...` (recorded in
  earlier checkpoints); the new hash differs as expected.
- NO adb commands were issued this stage; the Pixel was not touched.

## Verdicts (explicit)

- T14 configured-external-address primacy LIVE on the Windows node: **PASS**
  (diagnostics prove the configured endpoint is the only advertised address).
- Windows node health/identity/persistence after cutover: **PASS**.
- AWS connectivity + cloud-node health: **PASS** (re-established via env-var
  bootstrap replication; config.json alone does NOT carry the bootstrap link).
- BLE scanner fix live on the Pixel: **UNVERIFIED** - APK staged, install is the
  operator's action.
- Store-and-forward delivery to offline/other-network peers: **UNVERIFIED** -
  requires a live delivery test with a second peer after Pixel redeploy.
- Rule-8 adversarial review of the T14 diff (core/src/transport): **STILL
  OUTSTANDING** - merge gate, not a live-node gate; this node is a deployment,
  not a merge.
- Repo commit: **NOT DONE** (operator ruling: no commit yet).

## Evidence index (tmp/cto/T14_GOLIVE/)

- `build_release_20260908T233157Z.log` - release build under lock
- `identity_before.json` / `identity_after.json` / `identity_final.json` - identity stability
- `version_new.json` / `version_final.json`, `health_final.json` - new node API
- `diagnostics_new.json` (first relaunch, no env) / `diagnostics_reconnect.json`
  / `diagnostics_final.json` - external_addr + peers progression
- `aws_health_final.json`, `aws_version_final.json` - cloud node parity
- `new-node-out.log` (first relaunch), `node-out-20260908T235102Z.log` +
  `node-err-20260908T235102Z.log` (FINAL live instance), old-node log preserved
  at `tmp/cto-win-build-85cb4c67/windows-cli-candidate-live-20260908T162721Z.log`
- `launch_node_env.ps1` - reusable launcher with SC_BOOTSTRAP_NODES
- `apk_build_bg.log`, `apk_sha256.txt` - APK staging evidence
- `probe_node.ps1` - process probe used for PID/exe/argv evidence
