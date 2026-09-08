# V040 THREE-NODE ROLLOUT PLAN -- candidate 3891d11c (paste-ready execution plan)

Task: V040_3NODE_ROLLOUT_PLAN_2026-09-03.md
Type: PLAN -- execute only after the GO-GATE below passes
Date: 2026-09-03

Grounding (read these first, they win on conflict):
- Work order + evidence contract: `HANDOFF/freebuff/queue/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md`
- Standing directive: `HANDOFF/freebuff/inbox/V040_CEO_CTO_SYNC_2026-09-03.md` (items 3, 6)
- Recovery/disposition: `HANDOFF/freebuff/queue/V040_WORKTREE_RECOVERY_DISPOSITION_2026-09-03.md` (G.1-G.10)
- Review dispatch: `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md`

## GO-GATE (nothing below starts until this passes)

The external qwen free-lane APPROVE verdict is the explicit go-gate. Both
verdict files must exist, be dated today or later, and say plain APPROVE:
- `HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md` -- APPROVE of 3891d11c
- `HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md` -- APPROVE of 80197ef5

RE-verify the SHA at go-time, do not trust this sheet:
```bash
git rev-parse origin/cto/v040-candidate-2026-09-02   # expect 3891d11c...
git ls-remote origin refs/pull/272/head refs/pull/267/head
git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate status --porcelain  # expect empty
```
Validation SHA for ALL THREE NODES: 3891d11c. Any drift stops the run.

## Already done (verified 2026-09-03, do not redo)

1. **Candidate reviewed through two qwen rounds, fixes committed and pushed.**
   3891d11c = `origin/cto/v040-candidate-2026-09-02` = `refs/pull/272/head`;
   candidate worktree clean at that SHA. Gates claimed passing on 3891d11c
   (core 1400/0, cli 82/0, wasm32 clean, fmt clean) -- CTO-run on Windows.
2. **Pixel 6a ONLINE via wireless adb.** `adb devices -l` ->
   `192.168.0.113:34123 device product:bluejay model:Pixel_6a` (serial
   26261JEGR01896, Android 17). Endpoint is EPHEMERAL (changes on
   wireless-debugging toggle/reboot); recovery is two commands:
   `adb mdns services` (read the `_adb-tls-connect._tcp` line) then
   `adb connect <ip:port>`. `android/install-clean.sh` ALREADY implements
   this fallback automatically (no-device branch runs mdns + connect).
3. **No emulator on this machine** (SDK has no `emulator/` dir, no AVDs).
   Physical Pixel is the only Android path. Do not plan around an AVD.
4. **AWS node is RUNNING**: `i-0b735c4f26aea42ed` (scm-always-on-node,
   t3.micro, public IP 54.235.20.24, ami-0db1c5c6dc64eb019) -- from
   `.codebuff_deploy/aws/check.py` raw output.
5. **Per-SHA artifact pattern exists**: `.codebuff_deploy/wincli-<sha>/`
   holds `scmessenger-cli.exe` + `cli-provenance.txt` (records git-sha +
   version line). Rollout makes `wincli-3891d11c/`.
6. **Tooling paths**: AWS `.codebuff_deploy/aws/{launch.py,check.py,capt.py,
   scm_session.py}` (creds `~/.config/scmorc/aws.env`); Pixel APK dir
   `.codebuff_deploy/pixel-apk/`; docker build env `docker/` (build.Dockerfile,
   pinned Rust 1.75.0); Android install `android/install-clean.sh` (honors
   `ANDROID_SERIAL`, auto wireless-reconnect).

## Blocked / watch items (the plan stops or marks UNVERIFIED here)

- **B1 -- AWS new launches IAM-DENIED.** `check.py` raw output:
  `run_instances dry-run: DENIED`, `terminate_instances dry-run: ALLOWED`
  (caller `scmessenger-relay-orchestrator`). CONSEQUENCE: the AWS leg MUST
  reuse the running instance; do NOT terminate it (relaunch would be denied,
  killing the node with no replacement). The handoff's "AWS redeploy/IP churn
  and unaided re-mesh" gate cannot produce IP churn until the IAM policy
  allows RunInstances -- mark that sub-gate UNVERIFIABLE and say why.
- **B2 -- docker image push unverified.** launch.py userdata pulls
  `testbotz/scmessenger:latest`; someone must build the 3891d11c image from
  `docker/build.Dockerfile` and push it. Confirm docker + hub creds for
  `testbotz` BEFORE starting the AWS leg.
- **B3 -- Windows exe at 3891d11c does not exist yet.** Existing
  `.codebuff_deploy/wincli*/` exes are older SHAs (provenance shows 9b3980b).
  Fresh build required (step 3 below).
- **B4 -- Pixel APK at 3891d11c does not exist yet.** `.codebuff_deploy/
  pixel-apk/app-debug.apk` is 2026-08-30 (pre-candidate). Fresh build required
  (step 4 below).

## Execution sequence (end to end)

### 1. GO-GATE
Verify both APPROVE verdict files exist (paths above), SHA re-verify commands
return 3891d11c, candidate worktree clean. If any fail: STOP, report BLOCKED.

### 2. Confirm no other build is live
`tasklist | grep -iE 'cargo|rustc|gradle'` must be empty before any build
(shared checkout serializes builds; clean_target.sh rule).

### 3. Windows CLI artifact (operator-driven, authoritative on this host)
```bash
WT=/c/Users/SCM/Documents/GitHub/scm-v040-candidate   # at 3891d11c, clean
cd $WT
cargo build --release -p scmessenger-cli
mkdir -p /c/Users/SCM/Documents/GitHub/SCMessenger/.codebuff_deploy/wincli-3891d11c
cp target/release/scmessenger-cli.exe /c/Users/SCM/Documents/GitHub/SCMessenger/.codebuff_deploy/wincli-3891d11c/
# provenance: run the exe once, capture the version line (git-sha 3891d11c),
# write .codebuff_deploy/wincli-3891d11c/cli-provenance.txt per existing format
```
Note: builds land in the main checkout's `target/` -- do NOT clean it; the
release-gate builds are the authoritative Windows signal for this SHA.

### 4. Android APK artifact
```bash
WT=/c/Users/SCM/Documents/GitHub/scm-v040-candidate
cd $WT/android && ./gradlew :app:assembleDebug    # per AGENTS_ANDROID.md
# output: android/app/build/outputs/apk/debug/app-debug.apk (3891d11c)
cp app/build/outputs/apk/debug/app-debug.apk /c/Users/SCM/Documents/GitHub/SCMessenger/.codebuff_deploy/pixel-apk/
```

### 5. Docker image build + push (CTO)
```bash
cd docker && ./build.sh build-image   # per docker/README.md, pinned toolchain
# build the 3891d11c release image from build.Dockerfile
docker push testbotz/scmessenger:latest   # the tag launch.py pulls -- B2 watch
```

### 6. AWS node deploy + evidence (CTO) -- reuse the RUNNING instance
```bash
# do NOT run launch.py (B1: IAM DENIED). Reuse i-0b735c4f26aea42ed.
python .codebuff_deploy/aws/check.py        # confirm still running, get IP
# ssh/SSM to 54.235.20.24 (key scm-node-key per capt.py):
#   docker pull testbotz/scmessenger:latest
#   docker rm -f scm-node && docker run -d --name scm-node --network host \
#     --restart unless-stopped -e RUST_LOG=info,scmessenger=debug \
#     testbotz/scmessenger:latest scm --http-bind 0.0.0.0:9876 start
# capture: node identity, IP, startup timestamp, restart timestamp
```
IP-churn/unaided-re-mesh sub-gate: UNVERIFIABLE until B1 is fixed; record the
dry-run DENIED output as the reason. Re-mesh WITHOUT churn still testable:
restart the container and confirm the other two nodes re-find the node via
ledger sharing.

### 7. Windows leg -- [SEED-DIAL] evidence (operator)
Run `.codebuff_deploy/wincli-3891d11c/scmessenger-cli.exe`, capture startup
log: the `[SEED-DIAL]` lines (boot-time seed dial with bounded backoff, merged
in main at 67d19d3c #266, present in the candidate). Capture:
- exact exe + provenance (git-sha 3891d11c)
- startup timestamp
- `[SEED-DIAL]` output verbatim
- the node's advertised/observed external_addrs (see gate 2 below)

### 8. Android leg -- install + join (CTO capture, verification-only)
```bash
# device should already be online; if dropped, re-derive the endpoint:
adb mdns services            # read the _adb-tls-connect._tcp ip:port
adb connect <ip:port>        # ephemeral endpoint recovery
ANDROID_SERIAL=$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')
cd /c/Users/SCM/Documents/GitHub/SCMessenger/android && \
  ANDROID_SERIAL=$ANDROID_SERIAL UNINSTALL_FIRST=1 ./install-clean.sh
adb shell am start -n com.scmessenger.android/.MainActivity   # adjust to real launcher activity
adb logcat --pid=$(adb shell pidof -s com.scmessenger.android)   # capture join evidence
```
Record: install exit status, app start, peer/ledger join lines from logcat.
No code authoring on the handset (verification-only per work order).

### 9. Mesh propagation evidence (all three legs)
Once all three nodes are up with 3891d11c:
- each node sees the other two as peers (peer/ledger propagation);
- a message sent node-to-node is delivered (per delivery gate of the work
  order, where the environment permits);
- diagnostics `external_addrs` on each node show NO non-listen/ephemeral
  source-port entry (the T14/observation gate);
- coordinated restart: restart each node in turn and confirm the mesh
  re-forms unaided (ledger-sharing discovery).

## Three evidence gates (explicit PASS / FAIL / UNVERIFIED each)

1. **Windows [SEED-DIAL] + clean start** at 3891d11c -- evidence: exe
   provenance, startup log with [SEED-DIAL] lines.
2. **Diagnostics external_addrs** on all nodes -- no non-listen/ephemeral
   source-port entry.
3. **Three-node mesh**: peer/ledger propagation, delivery, coordinated
   restart re-mesh, Android install/start/join. AWS IP-churn sub-gate =
   UNVERIFIED (B1), re-mesh-without-churn still testable.

## Report contract (per the work order section 4)

One timestamped run record under `.codebuff_deploy/` (the existing evidence
area: windows_node.log etc. live there), linking raw artifacts:
- candidate SHA + build provenance for EACH node (all three must say 3891d11c)
- node identity and dynamically discovered address (AWS IP from check.py;
  Windows/Android local peer identity from logs)
- startup and coordinated-restart timestamps
- Windows [SEED-DIAL] output; peer/ledger propagation lines; external_addrs
  diagnostics; Android install/start/join evidence
- EVERY command used and its exit status, captured before any pipe/filter
- explicit PASS, FAIL, or UNVERIFIED for each of the three gates above
- the B1 IAM evidence (dry-run DENIED) attached verbatim

Completion note to `HANDOFF/freebuff/inbox/` beginning with:
`Task: V040_3NODE_ROLLOUT_PLAN_2026-09-03.md` / `Type: DONE | BLOCKED`,
naming the run record path, each node's observed state, and the next decision
required from the CEO seat (merge sequence start vs tag decision).

## Ownership

- Operator/CEO: Windows driver (step 7), build authority, merge/tag decisions.
- CTO: AWS deploy + capture (steps 5-6, 8-9 Android capture), run record.
- qwen free lane: already dispatched for the GO-GATE verdicts -- do NOT
  re-dispatch; launch the pending dispatch when ready.
