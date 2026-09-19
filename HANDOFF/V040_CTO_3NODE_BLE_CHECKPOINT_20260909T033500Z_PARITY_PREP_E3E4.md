# V040 CTO three-node checkpoint — PARITY PREP (E3/E4 closed, run tree frozen)

## Metadata

- Stage: `PARITY_PREP` (entry-gate closure; NOT a three-node completion stage)
- UTC timestamp: `2026-09-09T03:35:00Z`
- Session: CTO seat resumed via `/cto continue`; operator directive: "keep
  pushing to 3 node test - advise once we're ready"
- Frozen run commit: `d10ffda85e604557de5731c6654dea8c5492bda3` (branch
  `cto/t2-disk-ruling-2026-08-31`, pushed; `git rev-list origin..HEAD` = 0)

## What happened this session

1. **Windows node found DOWN at 02:47Z** (health 000, no listener, no
   process; last log line 02:23Z mid-normal-operation; no evidence of an
   authorized stop and none claimed by the CEO thread). Recovered: relaunched
   from the E6-proven no-env config path (bootstrap from `config.json`).
   Verified: identity byte-identical (`985a25f9...` /
   `12D3KooWD6vZQr...`), AWS peer reconnected, T14 pin live, custody 5073.
   Evidence: `tmp/cto/RESUME_20260909T024711Z/node-out-resume.log`.
2. **Run tree frozen**: E8 spec (`HANDOFF/V040_E8_ADVERTISEMENT_CONFIRMATION_SPEC_2026-09-09.md`)
   + `scripts/aws_deploy.sh` IMAGE_TAG override committed as `d10ffda8` and
   pushed. Android source is identical to the BLE-01 fix tree (`ba474a7a`
   span contains the android/ fix; core/src identical between the two).
3. **Docker image built from the frozen tree**: workflow_dispatch run
   `34305578702` SUCCESS (14m25s), pushed
   `testbotz/scmessenger:sha-d10ffda` + branch tag, image digest
   `sha256:133e5c673fee1d38acab88ef8d30b7109e677a4bad6c26a4e2c44389e11f882e`.
4. **E3 CLOSED - AWS redeployed at the run tree** via the tracked script
   mechanics with the identity-preserving `/opt/scm-relay-data:/data` mount
   (regression guard [OK]). AWS `/version` =
   `d10ffda85e604557de5731c6654dea8c5492bda3` exactly; identity PRESERVED
   (`12D3KooWGvCW...`, peerId unchanged across the swap).
5. **E4 CLOSED - AWS binary hash collected** (first time in campaign history;
   A2 closed): running binary sha256
   `c1257978bd3d80739c46ef281651379151fa10c310e298e2772d22d1b5c29730`
   (`/usr/local/bin/scm` in image `sha-d10ffda`). Pre-deploy record also
   captured: `94f04ecb5969...` at `sha-85cb4c6`
   (`tmp/cto/RESUME_20260909T024711Z/aws_binary_hash_pre.txt`).
6. **Windows cut over to the frozen tree**: release rebuild from `d10ffda8`
   (`cargo build --release -p scmessenger-cli`, 6m15s under build lock;
   required a stop-rebuild-relaunch because the live node held its own exe).
   New exe sha256 `26c4570938d8abdd96ef7a1779a93937d5d07eabf5221db9f65eee01ef82eff4`
   (rollback `829efe2c` still staged at `tmp/radio-829efe2c/rollback/`).
   Live `/version` reports `git_hash: d10ffda8`; `core_provenance` remains
   `ba474a7a` because core/src is byte-identical between the two commits
   (span touches tests/docs/cli only) - cargo correctly did not recompile it.

## Parity verdict (all from fresh commands, 03:30-03:35Z)

| Node | Artifact | Identity | State |
|---|---|---|---|
| Windows CLI | exe `26c45709...`, /version git_hash `d10ffda8` | `985a25f9...` / `12D3KooWD6vZQr...` (stable all campaign) | healthy, PID 444 owns 9876/9001/9002, AWS peer connected, external_addrs `["147.81.41.188:9001", ...]` configured-first, custody 5079 climbing |
| AWS cloud node | image `sha-d10ffda`, binary `c1257978...` | `37eb7561...` / `12D3KooWGvCW...` (preserved) | healthy, Windows peer connected, external `18.234.62.247:9001` |
| Pixel 6a | APK `09410285...` staged, NOT installed | (operator lane) | reachable via wireless adb; service control operator-held |

**Same-candidate parity: ACHIEVED on Windows + AWS (one commit, two nodes);
PENDING on Pixel (E5 install).**

## New findings recorded (not blocking, queued)

- **A4 (new RCA item): AWS custody audit history is container-ephemeral.**
  `custody_audit_count` went 151 -> 0 across the redeploy while
  `history_stats` (via `/data`) persisted. The custody audit store's default
  path resolves outside the mounted `/data`. No undelivered messages were
  lost (`undelivered_count` was 0). Fix (later, not on the frozen tree):
  point the custody store at `/data` via explicit config on the cloud node.
- Windows now advertises a secondary observed `147.81.41.188:8080` (a bound
  port; allowlist-consistent). Configured primary `:9001` still ranks first;
  watch during Phase 5 whether peers attempt the secondary.
- The 02:47Z silent node death is unexplained (no panic/shutdown in log).
  If it recurs, capture `Get-WinEvent` application errors around the window.

## Explicit verdicts

- E3 (AWS at run tree): **PASS**
- E4 (AWS binary hash): **PASS**
- E8 (evidence spec on file): **PASS** (spec written; per-node confirmation
  evidence still UNVERIFIED until the run)
- E9 (disk >= 20 GB): **PASS** (33 GB free)
- Windows/AWS connectivity + T14 primacy post-deploy: **PASS**
- E5 (Pixel BLE-01 APK install): **BLOCKED - operator action** (staged APK
  `09410285...`, path `android/app/build/outputs/apk/debug/app-debug.apk`)
- E1 (rule-8 review): **OPEN - operator dispatch**
  (`HANDOFF/V040_RULE8_REVIEW_PACKET_T14_ALLOWLIST_EXTERNAL_2026-09-09.md`)
- A3 (duplicate EC2 tag): **OPEN - operator housekeeping** (running
  `i-0b41aab756eabd514` is the correct target; stopped `i-0b735c4f26aea42ed`
  is the duplicate; deploy tooling resolves by running-state, so this does
  not block)
- BLE end-to-end, store-and-forward delivery, three-node completion:
  **UNVERIFIED - the test has not run**

## Operator actions to start the test

1. Install `android/app/build/outputs/apk/debug/app-debug.apk` (sha256
   `09410285974f3b0c198c8d5d026f62eaa1c5621eec43aa9d82e65eebcc829448`) on the
   Pixel, launch the app, leave mesh service running. Then tell the CTO
   "APK installed" - the CTO verifies via read-only adb (`pm path` hash,
   service state) and starts the package run at PREFLIGHT.
2. Optionally dispatch the E1 rule-8 review (merge gate, not a run gate).
3. Optionally retire/rename stopped instance `i-0b735c4f26aea42ed` (A3).

## Evidence index

- `tmp/cto/RESUME_20260909T024711Z/` - node-down recon + recovery logs,
  identity/diagnostics captures (before/after), AWS binary hashes (pre/post),
  deploy transcript, APK sha256, exe sha256, e3/e4 scripts
- `tmp/cto/E2_ANDROID_GATE_20260909T021316Z/` - android BLE gate 10/10 (prior)
- Docker Publish run: https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34305578702
