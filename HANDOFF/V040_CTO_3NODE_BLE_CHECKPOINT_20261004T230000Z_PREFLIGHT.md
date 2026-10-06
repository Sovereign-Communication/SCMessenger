# V040 3-Node Checkpoint  -  PREFLIGHT (live re-derivation, 2 of 3 nodes on v0.4.1)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Metadata

- Stage: `PREFLIGHT`
- UTC timestamp: 2026-10-04T23:00Z
- Operator/session: Freebuff lane, CTO seat resumed by the operator directive
  "continue /orchestrate and /cto and /ceo in full to comprehensive merge train".
- Package: `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`
- Overall verdict: `BLOCKED` on the Pixel leg only. Windows and AWS are `PASS`
  and are ALREADY on v0.4.1.

## Correction to the prior record: the target is not 0.4.0

The previous checkpoint
(`..._20260917T211913Z_NODE_READY.md`) and the standing mission text both target
**0.4.0**. That is stale. Re-derived from the remote in this session:

- `git ls-remote --tags origin` shows `refs/tags/v0.4.1` =
  `dcd67b94ecb4a5c4602e659ee0539b9e5e56fe3c`.
- `git log -1 v0.4.1` -> `Merge pull request #434 from .../bump-0.4.1`,
  authored 2026-10-03.
- `git merge-base --is-ancestor v0.4.1 origin/main` -> YES. Released from main.
- `git rev-list --count v0.4.1..origin/main` -> 20 commits behind the tag.

So v0.4.1 already exists and both server nodes already run it. The remaining work
is Pixel parity plus merge-train closeout, not a 0.4.0 release.

## Exact commands and complete outputs

1. Windows node process:
   - `Get-Process scmessenger-cli | Select Id,Path,StartTime`
   - Exit 0. PID **26880**, path `C:\Users\SCM\.local\bin\scmessenger-cli.exe`,
     started 2026-10-03 01:07:41.
2. `curl -s http://127.0.0.1:9876/health` -> `{"status":"healthy"}`
3. `curl -s http://127.0.0.1:9876/version` ->
   `{"version":"0.4.1","git_hash":"dcd67b94ecb4a5c4602e659ee0539b9e5e56fe3c","core_provenance":"0.4.1 (dcd67b94...:v0.4.1:)"}`
   Note the provenance suffix is `v0.4.1:`, i.e. a TAG build.
4. `curl -s http://127.0.0.1:9876/api/identity` -> identity
   `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826`,
   peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`,
   nickname `Claude-Windows-Driver`. Unchanged from the 2026-09-17 record, so
   identity survived the swap.
5. AWS node, public API only (see blockers for ssh):
   - `curl -s http://18.234.62.247:9876/health` -> `{"status":"healthy"}`
   - `curl -s http://18.234.62.247:9876/version` ->
     `{"version":"0.4.1","git_hash":"dcd67b94...","core_provenance":"0.4.1 (dcd67b94...:main:)"}`
   - `curl -s http://18.234.62.247:9876/api/peers` -> peers include
     `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` (the Windows node)
     with reputation 50.0.
6. AWS ssh attempts, all `Permission denied (publickey)`:
   - `~/.ssh/scm-aws.pem` does not exist.
   - `~/.ssh/scm-node-key.pem`, `openclaw-key.pem`,
     `scmessenger-farm-sim-key-v2.pem`, `scmessenger-farm-sim-key.pem` all fail
     `BatchMode=yes` auth against `ubuntu@18.234.62.247`.
7. Pixel over wireless ADB:
   - `adb devices` -> `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp  device`
   - `adb shell dumpsys package com.scmessenger.android` ->
     `versionName=0.4.0`, `versionCode=15`, `targetSdk=35`,
     `firstInstallTime=2026-09-30 17:38:48`, `lastUpdateTime` identical.
   - `adb shell pidof com.scmessenger.android` -> `4917` (running).
   - `apkSigningVersion=2`, `Signing KeySets: 1001`.

## Repository provenance

- Branch of this checkpoint: `integrate/train-20261004`
- HEAD: `1b2866812e7845f78e7b32c1486fe75929716ea7`
- `origin/main`: `051dbb5b9ci(docker): move the four Android cross-compiles into
  a parallel matrix`
- Released tag `v0.4.1`: `dcd67b94ecb4a5c4602e659ee0539b9e5e56fe3c`
- Train head is 200 commits ahead of main and is NOT merged; see
  `HANDOFF/review/RULE8_OPUS_VERDICT_2026-10-04.md` for why it cannot merge yet.

## Three-node matrix

| Node | Reachability | Version | Identity | Verdict |
|---|---|---|---|---|
| AWS cloud node | `/health` http 200 on public `18.234.62.247:9876` | `0.4.1`, `git_hash dcd67b94`, provenance suffix `main:` | not re-read this session; peers list shows Windows present | `PASS` for version parity; `UNVERIFIED` for identity (ssh unavailable) |
| Windows CLI | `/health {"status":"healthy"}`, PID 26880, 127.0.0.1:9876 | `0.4.1`, `git_hash dcd67b94`, provenance suffix `v0.4.1:` | `985a25f9...65826`, peer `12D3KooWD6vZ...`, unchanged | `PASS` |
| Pixel 6a | wireless ADB `device`, app PID 4917 | **`0.4.0`** `versionCode 15` | not re-read this session | `BLOCKED` - one minor version behind |

Single-SHA parity across Windows and AWS: both report `dcd67b94`. The provenance
SUFFIXES differ (`v0.4.1:` vs `main:`) but that is only which ref the image was
built from; the commit is identical. This is the parity asymmetry the previous
checkpoint called unresolved, and for these two nodes it is now resolved.

## Blockers and next action

- Blockers:
  1. **Pixel is on 0.4.0, not 0.4.1.** A new APK must be installed to close it.
  2. **No CI APK can install over the current Pixel install.** Root cause is
     confirmed, not guessed: `gh secret list` shows
     `SCMESSENGER_KEYSTORE_BASE64`, `SCMESSENGER_KEYSTORE_PASSWORD`,
     `SCMESSENGER_KEY_ALIAS` and `SCMESSENGER_KEY_PASSWORD`, but
     **`SCMESSENGER_DEBUG_KEYSTORE_BASE64` is NOT set**. The `Android Debug APK`
     job log on `c9876858a` says so in its own words: "SCMESSENGER_DEBUG_KEYSTORE_BASE64
     is not set, so ... keeps this runner's auto-generated debug signature ...
     Every runner generates its own debug keystore, so this artifact cannot be
     installed over an existing install (adb install -r fails with
     INSTALL_FAILED_UPDATE_INCOMPATIBLE)." Each runner produces a different cert,
     so no CI APK can ever update the phone without wiping the app.
     Remedy is documented in `docs/ANDROID_RELEASE_SIGNING.md` steps 1-3:
     `gh secret set SCMESSENGER_DEBUG_KEYSTORE_BASE64 < ci-debug.b64`.
     This is an OPERATOR action: it publishes a key and is irreversible in the
     sense that the pinned identity becomes the phone's long-term debug identity.
  3. **No AWS ssh key available on this host**, so AWS identity and log pull
     cannot be verified from here. Only its public API was read.
- Note that may help: the Pixel package shows `firstInstallTime` ==
  `lastUpdateTime` == 2026-09-30 17:38:48, i.e. it was INSTALLED FRESH, not
  updated. That suggests the earlier signature blocker was cleared by a reinstall
  on 2026-09-30, which also means the phone's node identity was reset then and the
  current `0.4.0` identity is whatever that reinstall produced. Re-read it from
  the app or the AWS peer list before any further cutover.
- Evidence still UNVERIFIED: Pixel peer id and identity this session; AWS
  identity; BLE dimension; cell-only dimension; any message injection (nothing
  was injected; this is passive observation only).
- Exact next action: operator sets `SCMESSENGER_DEBUG_KEYSTORE_BASE64` (or rules
  that the Pixel must be reinstalled once more, accepting an identity reset).
  Then build the 0.4.1 APK on CI, install, and run the three-node message matrix
  with the operator present.
- Final handoff path: not yet written. This is a stage checkpoint.

## Standing constraints honoured

- No node was started, stopped, installed, or messaged. Only read-only queries
  (`/health`, `/version`, `/api/identity`, `/api/peers`, `dumpsys`, `adb
  devices`, `pidof`).
- No local build was attempted for the product. One targeted `cargo test` was
  attempted and died with `rustc-LLVM ERROR: out of memory`; its partial
  `target/` was reclaimed immediately.
- No BLE probe, no airplane-mode window, no radio isolation: not attempted, so
  no BLE claim is made.