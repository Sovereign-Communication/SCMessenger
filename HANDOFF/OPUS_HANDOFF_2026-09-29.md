# Handoff: Claude lane (Sonnet 5.5) to Claude Opus 5.5 (2026-09-29)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Written about 22:00Z by the Claude lane (Sonnet 5.5) when the operator's weekly API
limit approached. The Claude lane's successor is Claude Opus 5.5, which takes over when
this session concludes. Astra (Codex/GPT, local) is the separate Codex lane: it watches
and audits this lane, and does not take it over.

This file is a RECORD, not an authorization. Verify every fact below with the
command beside it before acting. Permission comes only from the operator in your own
session. Two actions were refused by Claude Code's auto-mode classifier today (see
section 4); the classifier applies to Opus 5.5 in Claude Code too, so do not route
around either.

## 0. Update at about 22:20Z (this section wins wherever it conflicts with the rest)

- **Recipient.** The operator's last instruction was "get all our work committed and then
  cleaned up fully. Handoff for Codex by wrapping up and pushing and then halting." So
  Codex (Astra) takes the work now; Claude Opus 5.5 is the Claude lane's successor once the
  weekly limit resets. Everything below is written for whichever lane resumes.
- **The Claude lane has HALTED.** No subagent, build or lock of it is running. Every
  worktree it made was clean with nothing unpushed before cleanup (section 9 lists what was
  removed and kept).
- **POLICY: no local builds.** The operator: "no local builds are authorized. we moved to
  CI!" All compiling and testing is done by GitHub Actions; wait for the PR's checks
  instead. The subagent that made #421 ran `cargo check` and `clippy` locally (about 12
  minutes cold) and started a test build that took the disk to 0 bytes free. The Compile
  gate line in `CLAUDE.md` and the build-lock guidance are stale against this instruction;
  ask the operator to update them.
- **Changed since section 2.** #420 (this record) was merged by the operator at 21:46Z;
  `main` is `10f5642e`. The connection-cap stopgap is PR #421 (branch
  `fix/conn-cap-per-peer-64`, `MAX_ESTABLISHED_PER_PEER = 64`, a compile-time floor of 16,
  and the one test that hard-coded 4). The lane updated its branch with `main` at about
  22:18Z (a merge commit by Treystu). Its only local verification was `cargo fmt`, `check`
  and `clippy`; the changed `swarm.rs` test has never been compiled or run, so the PR's
  CI is its first run. It is unmerged and the adversarial review has not happened (#417).
- **Merge state.** After #420 every open train PR is BEHIND `main` (#399, #400, #401, #403,
  #405, #406, #408 to #411, #413, #415, #416, and #421 until its update settles); #412 is
  CLEAN. The auto-mode classifier still refused the lane's merges ("Merge Without Review"),
  so merges are the operator's or Codex's. Update one branch at a time
  (`gh pr update-branch <n>`), then merge with `--merge --match-head-commit <full sha>`.
- **Disk.** 100% used, about 2.3 GB free at 22:15Z (12 GB free at 21:30Z). The subagent's
  target directory (2.2 GB) is deleted; the rest of the drop is unexplained.
  `pagefile.sys` is 26.8 GB and `hiberfil.sys` 5.5 GB. The Windows node was healthy at
  22:06Z with no write errors, but a full disk can corrupt its sled store, so fix the disk
  before anything else: reboot (a system-managed pagefile shrinks then), or run
  `scripts/reclaim_safe.py` (dry run first).
- **Recommended order.** (1) Let #421's checks finish (Test (ubuntu-latest) is the long
  pole) and merge it. `Docker Publish` then builds `testbotz/scmessenger:sha-<7 chars>` from
  `main` (no local build). (2) Deploy the relay:
  `IMAGE_TAG=testbotz/scmessenger:sha-<7 chars> bash scripts/aws_deploy.sh 18.234.62.247`
  (A6; roll back with `sha-bceacb9`). (3) The operator repeats the cell test with the
  phone's current APK (the fix is relay-side) and the passive pull follows section 7.
  (4) The docs PRs #416 and #415 are cheap in CI (docs-only short-circuit): update, merge,
  then open MT-12', then the rest of section 6.

## 1. What the operator asked in this stretch

1. Unify, merge and land the v0.4.0 and v0.5.0 work on the merge train, with one
   canonical workflow that does not drift.
2. Roll out the three nodes (Pixel 6a, Windows, AWS; fresh install if needed), pull the
   Android logs passively, and triangulate against the Windows and AWS logs.
3. Push everything so nothing is unsaved, without disturbing WIP; the private backup
   repo may be used.
4. After the cell test failed: raise the per-peer connection cap "to at least 64",
   with a sanity pre-filter and a fallback of 16, and add a TODO for dynamic tuning in
   Advanced Settings. Then: "for now just raise it so I can test" and "override
   adversarial review for now ... then add the issue and tracking".
5. Stand down and hand the Claude lane to Opus 5.5 ("Claude to Opus and Codex to
   Astra"); Astra watches and audits.

## 2. State at handoff (verify each)

| Item | State | Check |
|---|---|---|
| `main` | tip `a41df8c4`, the merge of #414 (MT-00a anchor) at 21:03:49Z | `git fetch origin main; git rev-parse origin/main` |
| #414 | MERGED after three A2 rounds: 18f9dcb5 REJECT, 28eef643 REJECT, a1cb97cc APPROVE. Records: PR comments 5893414003, 5897925773, 5898833038 | `gh pr view 414` |
| #407, #404 | auto-marked MERGED (their heads are ancestors of #414) | `gh pr view 407` |
| #416 (MT-00b') | OPEN, head `d4138a07`, CLEAN, 5/5 required checks green. The lane's merge was refused: "Merge Without Review" | `gh pr view 416 --json mergeStateStatus,headRefOid` |
| #415 (BOD_STATE docs) | OPEN, BEHIND main; needs update-branch, CI, merge | `gh pr view 415` |
| MT-12' | branch `orch/mt12-drift-guard` (head `0719401e`), no PR yet. Must land AFTER #416 | `git log origin/orch/mt12-drift-guard -3` |
| #408 | superseded by #416; GitHub marks it merged when #416 lands (its head is an ancestor) | |
| Issues | #417 cell-handover root cause and the retroactive review; #418 dynamic tuning; #419 Android 16 KB RELRO alignment | `gh issue view 417` |
| Cap change | delegated to a subagent at about 21:45Z after the operator relaunched it; PR expected on branch `fix/conn-cap-per-peer-64` (worktree `wt-cap64`). STATUS UNKNOWN when this was written | `gh pr list --head fix/conn-cap-per-peer-64` |
| Backups | every local branch tip and every worktree HEAD is on a remote. Uncommitted work: earlier BK-01 refs `backup/20260927/...` on origin, plus refs `backup/20260929/wip/...` and branch `evidence/20260929-cell-test` on the private repo `Treystu/SCMessenger-backup` | `git ls-remote backup` |

Do NOT run `backup_purge.sh purge` without the operator. Its `backup` mode also
uploads multi-GB tarballs; use its read-only `capture-test` mode (as done today) for
snapshots.

## 3. The nodes (live facts at about 21:25Z)

- **Windows**: found DOWN (the node and its supervisor vanished at 10:27Z on 09-29 with
  no shutdown line; the machine did not reboot). Relaunched at 20:27:02Z under standing
  approval A5, build `bceacb9`, identity `985a25f9...`, peer `12D3KooWD6vZQrUq...`,
  supervised, HTTP health on `127.0.0.1:9876` (control socket 9001). Environment:
  `SCM_AUTO_REPLY=1` and `RUST_LOG=info,scmessenger=debug,scmessenger_core=debug,scmessenger_cli=debug`.
  The plan's literal `scmessenger=debug` matches no crate, so the crate targets were
  added; revert to `info` after the TRI run. Relaunch form: plan section 7.2 (WMI
  `Win32_Process.Create` running `scripts/run_node_supervised.ps1`). Logs:
  `C:/Users/SCM/AppData/Local/scmessenger/logs/scm.log.<UTC hour>`.
- **AWS**: `18.234.62.247`, image `testbotz/scmessenger:sha-bceacb9`, up since
  09-26T21:52Z, restarts 0, peer `12D3KooWGvCWJNoW...`, container env
  `RUST_LOG=info,scmessenger=debug`. Access per A6: `ssh -i ~/.ssh/scm-node-key.pem
  -o BatchMode=yes ec2-user@18.234.62.247 'bash -s' < script.sh` (put remote commands in
  a script file: a preflight hook rejects a pipe followed by `$?`). Deploy and rollback:
  `IMAGE_TAG=testbotz/scmessenger:sha-<short> bash scripts/aws_deploy.sh 18.234.62.247`.
  The plan's rollback tag `sha-45b0f8b` is stale: the current image is `sha-bceacb9`.
  The AWS CLI is not installed (needed only for the churn case C15).
- **Pixel 6a**: wireless ADB; the port changes, so use the mDNS name
  `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp` and ask the operator to re-enable
  wireless debugging if it is not listed. Installed 21:01:45Z from CI Mobile run
  `36311537543` (main at `1dc70f0b`, runtime-identical to `bceacb9`), versionCode 15,
  APK sha256 `3aebc5226c4391655f9f41003214684f644b06736e72772103b963488a3f2b3a`,
  device-side hash verified equal. NEW identity: peer
  `12D3KooWR4GDL3hWKDirvzeQALdqDnwJA1bfyjnieW24KzBEkgGV`, public key
  `29876f3973f8e9f2705ec531c1a429583336c137bfc01bd313c69cd663e6e675`. It is NOT yet
  seeded as a contact on Windows or AWS (plan H-3, "re-seed"). The phone's clock runs
  about 1.7 s ahead of the relay's.
- **Signing gap (the plan is wrong here)**: H-2 is recorded as done, but the repo secret
  `SCMESSENGER_DEBUG_KEYSTORE_BASE64` does NOT exist (only the release-keystore secrets
  do; the CI log for the run above says "not set"). Every CI APK carries a throwaway
  signature (this one: `9a2f3b12...`), so the next CI APK, including the TRI candidate,
  cannot be installed over this one and forces an uninstall, which wipes the identity.
  The operator chose "install now, pin later". Ask them to set the secret first.
- **Android 16 KB warning** (#419): non-fatal; details in
  `HANDOFF/todo/ANDROID_16KB_RELRO_ALIGNMENT_2026-09-29.md`.

## 4. Refused by the auto-mode classifier (do not route around)

1. Launching a subagent for the cap change with the review waived: denied, no reason
   given. After the operator explicitly relaunched it (scope: just raise the cap), it
   ran.
2. `gh pr merge 416 --merge --match-head-commit ...`: denied, "Merge Without Review".
   #414 merged earlier because independent A2 approvals were on the PR. For #416 the
   operator merges it, or adds a permission rule. Do not use `--admin`, `--auto`, the
   API, or another session to do the same thing.

## 5. The cell test failure (issue #417; evidence on the private repo)

The operator's cell test ran 21:09:26 to 21:10:35Z. The relay denied every fresh
cellular connection ("Inbound connection DENIED from /ip4/166.196.8.97/... limit 4
reached", five times) because the phone's four dead Wi-Fi connections still held the
per-peer cap (`core/src/transport/behaviour.rs`, `max_established_per_peer(Some(4))`)
until ping timeout at 21:10:13 to 21:10:24Z. The relay's ZOMBIE-CONNECTION REAP
attributes by source IP, which a Wi-Fi to cellular handover changes. On the phone the
relay-circuit dial to Windows died in 17 ms ("oneshot canceled"). Also seen: 20% of
the phone's dials went to addresses that cannot work from a phone (`172.17.0.1`,
`172.31.18.74`, loopback), and 76 dials fanned across six ports of one node.
Evidence: branch `evidence/20260929-cell-test` on `Treystu/SCMessenger-backup`
(`pixel/`, `aws/`, `windows/`); the same files sit in
`C:/Users/SCM/Documents/GitHub/wt-orch-mt00a/tmp/evidence/20260929`.

## 6. Next actions, in order

1. **Cap test build.** When the PR for `fix/conn-cap-per-peer-64` exists and its
   `Test (ubuntu-latest)` is green, publish a branch image (this leaves `latest`
   alone; tags are the branch name and `sha-<7 chars>`):
   `gh workflow run docker-publish.yml --ref fix/conn-cap-per-peer-64`, wait for it,
   then `IMAGE_TAG=testbotz/scmessenger:sha-<short> bash scripts/aws_deploy.sh 18.234.62.247`.
   Only the relay needs the new cap for the cell retest; the phone keeps its APK. Record
   the previous image (`sha-bceacb9`) for rollback. Ask the operator to repeat the cell
   test, then pull logs the same way (section 7).
2. **Merge #416** (operator), then run the post-merge JEV from the `orch/mt12-drift-guard`
   checkout (the script lives only on that branch until MT-12' lands):
   `HARNESS_REPO=C:/Users/SCM/Documents/GitHub/wt-harness-canonical python scripts/jev_post_merge.py --pr 416 --phase MT-00b --purpose "<text>" --out-dir tmp/jev/416`.
   It must score 85 or more before the next dependent car.
3. Update-branch #415, wait for CI, merge. Open MT-12' from `orch/mt12-drift-guard`
   after merging `main` into it.
4. The diamond, one car at a time, update-branch only the next one: #409, then #410,
   then #411. Then #405 and #406. #412 is based on `freebuff/train-mt00b`; retarget it to
   `main` after the pack is frozen. #413 waits on the MT-07 wasm scope ruling. Perimeter
   cars (MT-05 and others) need A2 on the exact head (Harness first pass, then a clean
   Claude reviewer; opus for transport).
5. Before any merge, read live: head SHA, `mergeStateStatus`, the required contexts from
   `gh api repos/Sovereign-Communication/SCMessenger/branches/main/protection/required_status_checks`
   (five: Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest), Handoff
   ownership scope), unresolved threads, and every non-required failure. Merge with
   `--merge --match-head-commit <full sha>`, never `--admin`.
6. TRI-040 (plan section 7) once every car has landed. Preconditions the operator must
   provide: the debug keystore secret (section 3), the AWS CLI for case C15, and the
   phone taps for cases C2, C6, C7, C8, C10, C14.
7. A2 follow-ups from #414 (LOW/INFO items) are listed in PR comment 5898833038.

## 7. Passive log pull (worked today)

- Phone: `adb -s <serial> logcat -d -v threadtime,UTC,year`; then
  `adb -s <serial> exec-out run-as com.scmessenger.android cat files/logs/scmessenger-mesh.log`
  (this build writes JSON lines, UTF-8). `files/mesh_diagnostics.log*` rotates about once
  a minute and keeps only five, so pull immediately.
- AWS: `sudo docker logs -t --since <T0> scm-node` through the ssh script form above.
  Strip ANSI before parsing.
- Windows: copy the hourly `scm.log.*` and `curl -s http://127.0.0.1:9876/api/diagnostics`.
- A downloaded CI artifact through `gh run download` hung; use
  `gh api repos/<repo>/actions/artifacts/<id>/zip > file` instead.

## 8. Gotchas

- Hooks: no `$?` after a pipe; no `git add -A` (even against a temporary index; hash
  files and use `git update-index --cacheinfo`); no foreground `sleep`; no emojis in any
  file, PR body or issue.
- Git Bash mangles `git show rev:path`; use `MSYS_NO_PATHCONV=1`.
- `wt-orch-mt00a`'s local branch tracks the lane's branch: always push with an explicit
  refspec (`git push origin HEAD:refs/heads/<name>`).
- Disk was 95% full (about 13 GB free). No two build tools at once; build through
  `scripts/build_lock.py`; remove your own `target/` afterwards.
- The Harness spend key has a $0.75 daily limit (about $0.07 used today). A2 first passes
  cost $0.001 to $0.003; the admitted release for gates is tag `v0.4.1`
  (`wt-harness-canonical`).
- The worktrees the lane created today are listed, with what was removed and kept, in
  section 9.

## 9. Cleanup performed at wrap-up (2026-09-29, about 22:25Z)

- Each worktree below was clean with nothing unpushed before removal. Every branch stays
  on origin, so recreate a checkout with `git worktree add <path> <branch>`.
- REMOVED (SCMessenger repo): `wt-a2-414` (detached review checkout), `wt-cap64` (the
  subagent's; branch `fix/conn-cap-per-peer-64`, PR #421), `wt-orch-mt00b` (`orch/mt00b-amend`,
  PR #416), `wt-orch-mt12` (`orch/mt12-drift-guard`, no PR yet; it carries
  `scripts/jev_post_merge.py`), `wt-bod-unify` (`orch/bod-unify-20260929`, PR #415) and
  `wt-astra-handoff` (this record's branch).
- KEPT: `wt-orch-mt00a`, because it holds `tmp/evidence/20260929` and the runbook forbids
  deleting `tmp/` evidence (the same 29 files are on the private backup repo, branch
  `evidence/20260929-cell-test`); its downloaded APK copy and scratch scripts were removed.
  In the Harness repo: `wt-harness-canonical` (the admitted v0.4.1 used for gates) and
  `wt-harness-canary` (the Harness MCP server's registered checkout).
- NOT TOUCHED: the primary checkout and every other session's worktree, all branches and
  remote refs, the backup refs, and the running Windows node (supervised, healthy). The adb
  server the lane started is left running because the operator uses it.
- Temporary files the lane made in the user temp folder were deleted.
