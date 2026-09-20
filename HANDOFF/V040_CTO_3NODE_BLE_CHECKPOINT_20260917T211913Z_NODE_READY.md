# V040 3-Node Checkpoint — NODE_READY (candidate cutover, 2 of 3 nodes)

## Metadata

- Stage: `NODE_READY`
- UTC timestamp: 2026-09-17T21:19:13Z (commands run 20:55Z-21:19Z)
- Operator/session: Freebuff lane, CTO seat. Operator directive: "go for 3 node as
  soon as the new artifacts are ready and prove the working build in all
  dimensions"; candidate chosen by the operator = `1005da14` (#295 lane).
- Overall verdict: `BLOCKED` (AWS + Windows swapped to candidate and PASS;
  Pixel leg BLOCKED on a debug-keystore signature mismatch that cannot be
  cleared without destroying the phone's on-device identity)
- Package: `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`
- Runbook amended this stage: `HANDOFF/UNIFIED_V040_3NODE_DEPLOY_RUNBOOK_2026-09-10.md`
  (operator install/log-pull ruling; non-root AWS recreate recipe; keystore precondition)

## Exact commands and complete outputs

1. Identify the candidate's artifact set (nothing in flight anywhere in the repo):
   - `gh run list --limit 60 --json ... | select(status != completed)` -> empty
   - `gh run list --branch main --limit 40` -> main tip `eb557569` lanes:
     Repository Hygiene / Lint / CI / Docker Publish / CodeQL = success;
     Docker Integration Suite = failure (known non-blocking item, PR #156 open);
     Mobile + Cross + iOS did not run (docs-only path filters).
   - `gh api .../compare/eb557569...7becf8a0 --jq '.files[]'` -> 9 files, and
     `git diff --stat origin/main FETCH_HEAD -- core cli docker Cargo.toml Cargo.lock
     mobile wasm desktop_bridge` -> EMPTY.
   - Complete output: this checkpoint, and the artifact HASHES below.

2. Bind artifacts to the candidate:
   - `gh run download 35265161728 -n windows-cli-1005da142c7e7cae94801241ac834a04824db99d`
     -> `tmp/radio-candidates/7becf8a0/scmessenger-cli.exe`
     sha256 `fc82cbe1e14cc9b7a8345ba478f86711ccbc4f458207d7d7f322fda0f1a74375`
     provenance file: `git-sha: 1005da142c7e7cae94801241ac834a04824db99d`,
     `Core Provenance: 0.4.0 (1005da1:HEAD:1789674861)`
   - `gh run download 35265161795 -n android-debug-apk`
     -> `tmp/radio-candidates/7becf8a0-apk/app-debug.apk`
     sha256 `878008ea243cb1820417f939ba6b41ba21f12abad49b8620828b42d2c21835a3`
     `versionCode 15`, `versionName 0.4.0`
   - `gh run download 35265122137 -n windows-cli-eb557569...`
     -> `tmp/radio-candidates/eb557569/scmessenger-cli.exe`
     sha256 `489fedfe8de8ad7ee782641d8ef43dea859d5c8d890ed7210a92a65d0be09a3e`
   - `gh api .../commits/1005da142c...` -> `Merge 7becf8a0 into eb557569`,
     parents `[eb557569, 7becf8a0]`, tree `8025288f43f489a5ff8729ee03a42c49cc04c684`
     = the tree of `7becf8a0`. The CI artifact is the PR merge commit, not the head.

3. Disk reclaim (runbook stop condition was met at 10 GB free / 96%):
   - `python scripts/reclaim_safe.py` -> only `wt-297-outbox` and `wt-p1-curve`
     SAFE; every other worktree UNKNOWN/HOLD.
   - `python scripts/reclaim_safe.py --reclaim` -> `tmp/wt-p1-curve/target`
     2.04 GB freed. `df -h .` -> 14 GB free, 95% used.
   - The main checkout was correctly held (dirty), so the live node's
     `target/release/` was left intact.

4. Windows CLI cutover (identity-preserving):
   - pre-state capture -> `tmp/radio-candidates/win-node-prestate-20260917.txt`
     `/version` = `git_hash 7a2c16c3`, `core_provenance 0.4.0 (597e2c72:...)`;
     `/api/identity` = peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`,
     identity `985a25f9...5826`, nickname `Claude-Windows-Driver`
   - rollback staged: `tmp/radio-candidates/rollback-597e2c72/scmessenger-cli.exe`
     sha256 `9b251c276d6b3f5167bd6c2ab159d007a42c151dd4957e4ec0ce1798f2d6eecb` + SHA256SUMS
   - `curl -X POST http://127.0.0.1:9876/api/shutdown` -> `Stopping...`, process
     exited in 2 s (graceful, no kill needed)
   - `cp tmp/radio-candidates/7becf8a0/scmessenger-cli.exe target/release/scmessenger-cli.exe`
     -> file sha256 becomes `fc82cbe1...`
   - relaunch with the identical argv recorded from the live process:
     `nohup ./target/release/scmessenger-cli.exe start -p 9001 --auto-reply
     "Node is on but unattended - I will read your message when back."`
     -> new PID 3968; stdout `tmp/radio-candidates/win-node-1005da14.out`
   - post-state: `/health {"status":"healthy"}`;
     `/version git_hash 1005da1`, `core_provenance 0.4.0 (1005da1:HEAD:1789674861)`;
     identity and peer id UNCHANGED.

5. AWS cloud node cutover (identity-preserving):
   - pre-state: container `scm-node`, image `testbotz/scmessenger:sha-6acaa23`,
     `/version git_hash 6acaa2317f08b8095316c7775e482ef965dbc913`,
     identity `37eb7561...d006`, peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`
   - `docker pull testbotz/scmessenger:sha-eb55756` ->
     `Digest: sha256:82257d24244391afb87cfb96cefe3a63631509e6f88c97e60cddab841ea47000`
   - first recreate FAILED and is recorded deliberately: carrying the legacy env
     `SCM_CONFIG_DIR=/root/.config/scmessenger` into the post-#293 non-root image
     (`USER scm`, uid 10001) produced
     `mkdir: cannot create directory '/root': Permission denied` in a restart loop.
   - fix: `/opt/scm-relay-data` was root-owned; chowned to uid 10001 through a
     throwaway root container (no host privilege wrapper needed, docker group is enough):
     `docker run --rm --user root -v /opt/scm-relay-data:/d testbotz/scmessenger:sha-eb55756 chown -R 10001:10001 /d`
   - recreate with the image's own config dir and the previous env otherwise
     (full command in the runbook "Recreate recipe" section)
   - post-state: `docker inspect` -> `RestartCount=0`, started 2026-09-17T21:13:50Z;
     `/health` http 200; `/version git_hash eb55756957e2d01f558321b374258a3c05750181`,
     `core_provenance 0.4.0 (eb55756:main:)`;
     identity `37eb7561...d006` and peer `12D3KooWGvCW...` UNCHANGED, proving the
     identity lives in the mounted `/data` sled store, not the config dir.

6. Pixel candidate install attempt:
   - signer comparison via apksigner: CI APK cert SHA-256
     `47a84596e934e98293252e3874ecae4c2e2a9c3715a8ea7ce64963357c9a6097`
     vs local `gradlew` APK cert SHA-256
     `1cdef09cd3b80f9b686e5f9e7b760d360b1fd338c6720bcb59b15b233967835f`
   - `adb install -r tmp/radio-candidates/7becf8a0-apk/app-debug.apk` ->
     `Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE: Existing package
     com.scmessenger.android signatures do not match newer version; ignoring!]`
     exit 1. App verified untouched afterwards (`versionCode=15`, still running).
   - no uninstall was performed: that would change the node's identity.

7. Authorised log pull (no device driving):
   - `adb exec-out run-as com.scmessenger.android sh -c 'tail -c 300000
     files/logs/scmessenger-mesh.log'` -> `tmp/pixel_3node_20260917/pixel-mesh-tail.log`
     (300,000 bytes; the full log is 103,001,659 bytes and is not pulled)

## Repository provenance

- Branch: `feat/v040-multi-transport-store-forward`
- HEAD: `629a3eefa2c7eeca2c3d3af3d0cb49bd2bc6626f`
- HEAD tree: `8428b1f4a55e10aa9f64ccab0fb587db371df268`
- `origin/main`: `eb55756957e2d01f558321b374258a3c05750181`
- Selected candidate ref: `refs/pull/295/head` = `7becf8a0ac87bd9d7c3058723e702d4933377c22`
  (NOT merged: `git merge-base --is-ancestor FETCH_HEAD origin/main` -> no)
- Candidate build commit (what CI actually compiled): `1005da142c7e7cae94801241ac834a04824db99d`
- Candidate tree: `8025288f43f489a5ff8729ee03a42c49cc04c684`
- Relevant tags: `v0.4.0-rc.1` is the newest; there is still no final `v0.4.0`

## Three-node matrix

| Node | Reachability | Version/commit/artifact | Identity | Verdict |
|---|---|---|---|---|
| AWS cloud node | `/health` http 200 over ssh; `i-0b41aab756eabd514`, public 18.234.62.247 | `git_hash eb55756957e2d01f558321b374258a3c05750181`, image `testbotz/scmessenger:sha-eb55756` (digest `sha256:82257d24...`) | identity `37eb7561...d006`, peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, nickname `null` | `PASS` (updated from sha-6acaa23; identity preserved) |
| Windows CLI | `/health {"status":"healthy"}`, PID 3968, control API 127.0.0.1:9876 | `git_hash 1005da1`, `core_provenance 0.4.0 (1005da1:HEAD:1789674861)`; exe sha256 `fc82cbe1...` | identity `985a25f9...5826`, peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`, nickname `Claude-Windows-Driver` | `PASS` (swapped from 597e2c72; identity preserved) |
| Pixel 6a (`bluejay`, Android 17 / API 37) | wireless ADB `device`; app PID 22189; APK sha256 `4a0972b29556c4dadd0739667e2935aaededb5366ab6bed7e7c5a9f046ffe86b` (2026-09-16 build) | app `0.4.0`, `versionCode 15` — the CANDIDATE is NOT installed | identity `f83ab16319ca5b801f1c088935f2215c6aae9aa246b992f01c0f27f06b03cbe5`, peer `12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn`, pubkey `30dce2bb...`, nickname `Lucas` | `BLOCKED` — signature mismatch, see above |

Provenance asymmetry, stated plainly: Windows reports `1005da1` and AWS reports
`eb55756`. The node code is identical between them — `git diff --stat origin/main
FETCH_HEAD -- core cli docker Cargo.toml Cargo.lock mobile wasm desktop_bridge` is
EMPTY, and the whole diff between main tip and the candidate is 9 files, all
`android/**` plus one HANDOFF ticket. The two strings differ only because the
images were built from different refs (`HEAD` detached PR-merge build vs a `main`
push build). True single-SHA parity needs the candidate merged so CI publishes an
image, an APK and a CLI from one SHA; that is an operator merge decision, not a
lane action.

## Stage-specific evidence

- Android capture path: `tmp/pixel_3node_20260917/pixel-mesh-tail.log`
- Windows capture path: `tmp/radio-candidates/win-node-prestate-20260917.txt`,
  `tmp/radio-candidates/win-node-1005da14.out`
- AWS capture/API output: inline in section 5 of this checkpoint
- Rollback artifacts retained: `tmp/radio-candidates/rollback-597e2c72/`;
  previous AWS image `testbotz/scmessenger:sha-6acaa23` still present locally on
  the instance
- Message label/content: none injected — the traffic below is the fleet's own
  keepalive/probe path, observed passively
- Message ID: n/a
- Live IP-transport evidence from the Pixel app log, AFTER both swaps
  (`tmp/pixel_3node_20260917/pixel-mesh-tail.log`):
  - 21:16:20.077Z `[OK] Message delivered successfully to 12D3KooWD6vZQrUqpy... (13ms)` — Pixel -> Windows, direct
  - 21:17:58.630Z `[OK] Message delivered successfully to 12D3KooWD6vZQrUqpy... (28ms)` — Pixel -> Windows, direct
  - 21:17:38.529Z `ROUTE_DECISION ... route=direct destination=12D3KooWGvCWJNoWn...` then attempt 2
    `route=relay relay=12D3KooWD6vZQrUqpy... reason=RETRY_NEXT_CANDIDATE`
    — Pixel -> AWS with live relay fallback through the Windows node
  - 21:18:39.032Z `[OK] Message delivered successfully to 12D3KooWGvCWJNoWn... (273ms)` — Pixel -> AWS, direct
  - AWS side 21:18:36-37Z: `Identified peer 12D3KooWD776... - agent:
    scmessenger/0.4.0/full/relay/...`, `[CUSTODY] Registered local identity with
    peer ... (relay-ready)` for both Windows and Pixel, `inbox_receive` message
    from `f83ab163...` (the Pixel)
  - Windows `/api/peers` = the Pixel only; AWS `/api/peers` = Windows + Pixel
    (the two stable nodes see the phone; the AWS<->Windows edge is carried through
    the phone's ledger view as shown above)
- BLE-specific ingress/decrypt evidence: NONE — BLE not run this stage
- Competing TCP/mDNS/Wi-Fi evidence: yes, and it is all IP transport —
  ledger canonicalisation writes carry `/ip4/192.168.0.121/tcp/{9002/ws,8080,9090,80,443}`
- Radio state: not isolated (no airplane-mode / radio-off window was taken)
- Cleanup state: no teardown; rollback artifacts and the previous image retained

## Blockers and next action

- Blockers:
  1. Pixel candidate install is impossible without either a local build signed by
     the device's debug keystore, or an operator-approved `adb uninstall`
     (which resets the phone node identity `f83ab163...` / peer `12D3KooWD776...`).
  2. Provenance asymmetry `1005da1` vs `eb55756` (node code proven identical).
     Perfect single-SHA parity requires merging the candidate.
  3. BLE dimension remains hardware-gated; cell-only dimension needs the operator
     to move the phone off the LAN.
- Evidence still UNVERIFIED: BLE-only isolation, cell-only/AWS-custody matrix row,
  Pixel running the candidate build, Pixel peer id read from the app UI (it was
  read from the AWS node's logs instead).
- Exact next action: operator decision on the Pixel route (local build vs approved
  identity reset) and on merging the candidate for single-SHA artifacts; then
  re-run the base/cell/BLE matrix with the operator present.
- Final handoff path: not yet written — this is a stage checkpoint, not the
  `V040_CTO_3NODE_BLE_FINAL_HANDOFF_*`.
