# V040 Merge-readiness runbook -- #272 to main at the final head e97c3f82

Date: 2026-09-04 (~11:40Z)
Purpose: one paste-ready runbook so that when the AWS leg lands, the stable
3-node evidence capture and the #272 merge are a single pass. The commands
below were verified live against the running node (PID 20688, Control API
127.0.0.1:9876) at 11:37Z -- they are not guesses.

## A. Gate ledger -- every #272-to-main gate and its evidence

| # | Gate | Status | Evidence (file / ledger) |
|---|------|--------|--------------------------|
| 1 | PR #272 head pinned | DONE | e97c3f8247b29dd344467e05137b24f0f110a10a, branch cto/v040-candidate-2026-09-02 |
| 2 | CI green at head | DONE (re-verify at go-time) | all checks COMPLETED, 0 failed; rerun via `gh pr checks 272` |
| 3 | Merge conflicts resolved | DONE | true merge commit e97c3f82 (parents 48672b18 + 45ab59f9); brief: HANDOFF/freebuff/inbox/V040_CANDIDATE_MERGE_CONFLICT_BRIEF_2026-09-04.md |
| 4 | Rule-8 resolution APPROVE | DONE | HANDOFF/review/V040_PR272_RESOLUTION_DELTA_RULE8_APPROVE_e97c3f82_2026-09-04.md (ledger seq 222-226) |
| 5 | FINAL APPROVE delta re-pin @ e97c3f82 | DONE | HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_DELTA_e97c3f82_HARNESS_2026-09-04.md (ledger seq 227-234) |
| 6 | FLAG-5 deferral (option c) re-pin @ e97c3f82 | DONE | HANDOFF/review/V040_CANDIDATE_272_DEFERRAL_APPROVE_e97c3f82_HARNESS_2026-09-04.md (ledger seq 243-250) |
| 7 | Artifact provenance @ e97c3f82 | DONE | .codebuff_deploy/wincli-e97c3f82/ (exe sha256 ac7fb51a..., cli-provenance.txt, merge-ref tree == e97c3f82 tree 94d9d7c0...); .codebuff_deploy/pixel-apk-e97c3f82/app-debug.apk (sha256 acb7bd88..., 4 ABIs) |
| 8 | Driver removal + /drive command | DONE | watcher PID killed, Startup lnk removed, no scheduled task; .claude/commands/drive.md |
| 9 | **3-node validation at final tree** | **PENDING -- ONLY REMAINING GATE** | this runbook section B |

## B. Stable 3-node evidence capture (run once AWS leg is live)

Precondition: CTO has pushed the docker image built at e97c3f82, redeployed
i-0b735c4f26aea42ed, and filed evidence in tmp/run-evidence/ -- including the
image SHA and whether node identity 12D3KooW9uRMQT survived (if it changed,
the mesh re-learns it; note the new ID).

Windows node (this machine, PID 20688, log
.codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log, Control API
127.0.0.1:9876):

```bash
# 1. SEED-DIAL evidence (already captured at launch; re-confirm)
grep '\[SEED-DIAL\] sweep' .codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log

# 2. Stable mesh -- all three peer IDs present together, steady for 2+ min
curl -s http://127.0.0.1:9876/api/peers          # expect AWS 12D3KooW9uRMQT + Pixel 12D3KooWBPdNE...
curl -s http://127.0.0.1:9876/api/external-address  # expect the public TCP addr
curl -s http://127.0.0.1:9876/api/listeners
curl -s http://127.0.0.1:9876/api/identity
curl -s http://127.0.0.1:9876/api/diagnostics > tmp/run-evidence/diag-e97c3f82-$(date -u +%Y%m%dT%H%M%SZ).json

# 3. Relay custody + audit (log lines)
grep 'Relay custody audit' .codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log | tail -5
grep 'Connected to'        .codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log | tail -10

# 4. Coordinated restart evidence: restart the Windows node (kill PID, relaunch
#    same exe), then confirm it re-joins AWS + Pixel within ~2 min and the
#    ledger retains both peers. Timestamped log lines are the evidence.
```

Android leg (user drives): app foregrounded, adb online
(adb-26261JEGR01896-6pHTac @ 192.168.0.129:42311 -- ephemeral; recover via
`adb mdns services` + reconnect if it drops). Join evidence = app showing
peers + Windows node log showing `Connected to 12D3KooWBPdNE...` after each
restart.

AWS leg (CTO): docker-publish at e97c3f82 -> pull + redeploy -> capture
container/identity continuity (12D3KooW9uRMQT kept or new ID noted) with
restart timestamps in tmp/run-evidence/.

## C. Merge sequence -- execute only after section B evidence is on file

```bash
# Per-merge gates, in order, before the squash
git fetch origin main cto/v040-candidate-2026-09-02
git rev-parse cto/v040-candidate-2026-09-02   # MUST still be e97c3f82
git diff --quiet origin/main...cto/v040-candidate-2026-09-02; echo $?  # 1 = diff present (expected)
git cherry origin/main cto/v040-candidate-2026-09-02 | grep '^+'         # unmerged commits
git merge-tree $(git merge-base origin/main cto/v040-candidate-2026-09-02) origin/main cto/v040-candidate-2026-09-02 | grep -c 'changed in both'; echo "0 expected"
gh pr checks 272   # all green
gh pr merge 272 --squash   # then verify main head
git fetch origin main && git rev-parse origin/main
```

Then append the run record (SHAs, evidence paths, verdict citations) to
HANDOFF/freebuff/inbox/V040_MERGE_EXECUTION_LOG_2026-09-03.md.

## D. Governance guards

- No self-merge, no tag, no release. Tag decision is the CEO's after #272
  lands (suggest v0.4.0 gate convention per SHIP_PLAN).
- Do not merge while the CTO is mid-deploy or any cargo/docker build is live
  (scripts/clean_target.sh serialization rule).
- Stop-and-report on: head movement off e97c3f82, any CI failure, merge-tree
  nonzero, or an AWS identity change that breaks the mesh.