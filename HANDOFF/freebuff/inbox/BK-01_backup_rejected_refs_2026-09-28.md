<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / BK-01
Type: BLOCKED

This handoff is an SCMessenger-only record for Sovereign-Communication/SCMessenger.

Evidence captured 2026-09-28:

- `bash 'C:/Users/SCM/Documents/GitHub/wt-train-plan/scripts/backup_purge.sh' inventory` printed `C:` as 237G total, 233G used, 3.4G available (99%) at 20:59:32Z.
- `bash 'C:/Users/SCM/Documents/GitHub/wt-train-plan/scripts/backup_purge.sh' backup` exceeded the tool's 600-second timeout. Its output in `C:/Users/SCM/Documents/GitHub/scm-train-state/backup_purge/run.log` records 16 failed backup-ref pushes (non-fast-forward) and one network connection failure. `ledger.tsv` was incomplete at 20 rows (16 WIP, 1 index, 3 clean); no `[OK] backup done` or successful verification for this retry was recorded. Correction to the first #403 summary: the log contains 16 failed-push markers total, comprising 15 non-fast-forward rejections and 1 network failure (not 16 non-fast-forward rejections plus a network failure).
- `powershell.exe -NoProfile -Command 'Get-CimInstance Win32_Process ...'` confirmed the timed-out task-owned backup script was still running as PID 31100. I stopped only that process after the script's `[FAIL]` condition; a follow-up process query found no remaining `backup_purge.sh backup` process.
- I did not run `verify`, `purge --dry-run`, `purge`, or `report` for this retry. No purge or deletion was performed. Whether the current snapshots are all backed up is UNVERIFIED; the incomplete ledger cannot establish `0 missing`.

Stopped as required by the BK-01 instruction to stop on any `[FAIL]`. Await operator direction before retrying the backup. No secret or vault paths are included.

## Read-only reconciliation of the rejected refs for the SCMessenger merge train (2026-09-28)

After the stop, I compared every failed WIP capture against the current remote ref without writing to either side:

- `git ls-remote --heads origin 'backup/20260927/*'` returned 28 refs.
- `ledger.tsv` contains 16 WIP rows, 1 index row, and 3 clean rows. `run.log` has 16 failed-push markers: 15 non-fast-forward rejections and 1 network failure.
- For each of the 16 work-in-progress captures whose push did not complete, the named remote ref exists and `git show -s --format=%T <remote-sha>` equals the ledger's expected tree. The attempted commit SHA differs from the remote commit SHA for each row, and `git merge-base --is-ancestor <attempt-sha> <remote-sha>` returned false for each. Thus the captured Git tree content is already represented by the remote ref, but the backup verification script's exact-commit check is not satisfied. I did not run `verify`, rewrite the ledger, or retry a push.

All 16 comparisons (ref | attempted commit -> existing remote commit | tree):

| Ref | Attempt SHA | Remote SHA | Tree SHA |
|---|---|---|---|
| `backup/20260927/wip/SCMessenger/cbb450828a7d` | `362cd27cdc85f40eddc4323b3759507c26d90c2c` | `6f6aaeef55454ae0b17122f3799bb2f4fc26df23` | `cbb450828a7d53c2883cd8d27f46fc34766de851` |
| `backup/20260927/wip/scm-train-state--scratch--harness-plan-snapshot-20260924/0815e699693d` | `105b6c2e1ecc4f9c5b05113f30c27c98d7fa5491` | `7901a81bd75f446802b43885ebed805571b37996` | `0815e699693d55fe8ded12650d6db70904ec8a97` |
| `backup/20260927/wip/scm-train-state--scratch--repro-candidate/440722c48d10` | `fed4b21374b2e8e1299bd0ee743675581c31875e` | `dd9b62196a1070759a3dd33f5603c92e51abc943` | `440722c48d108811dd2610f35c2fa94d29b9eecb` |
| `backup/20260927/wip/scm-train-state--scratch--repro-candidate2/d8e170673d92` | `5a96b0a89c9e7659e3c87f556d1ad64b4a5a2fa1` | `fed354a47e7f980c9b9db4572ac2848d47cbdab0` | `d8e170673d92b8b4105a4ade1165ed17259664de` |
| `backup/20260927/wip/scm-train-state--scratch--repro-candidate3/9cab9880a856` | `34c511afdb8d6988213de23da99286354a565fff` | `c04d3caffc997a6d93aa65897afecffb2723a7e8` | `9cab9880a856e8a627d28f73958b66bd741a1011` |
| `backup/20260927/wip/scm-train-state--scratch--repro-candidate4/655edf9bef7e` | `d89f5a13a2773245a1b1b17d1ea44c7bd9c6ff40` | `2e7b9f864e29b063efb5a2ee87333f0405464718` | `655edf9bef7e854257aa5848043adc6f89611d35` |
| `backup/20260927/wip/scm-train-state--scratch--repro-candidate5/d22eac902336` | `0cab3fedb36237c8252444969b22928d3aee9042` | `dcb983a03b78c4e43b61f0cfd687c1ceffe08730` | `d22eac902336fe448d9f2750c1dbf9916ca438dd` |
| `backup/20260927/wip/scm-train-state--scratch--repro-clean/aaa240bc209f` | `8a83853a82bd286eb129222a461731def677fc7d` | `6fbd7136a2edc6ec015d2f4b46929dcc043f55ce` | `aaa240bc209f008ae2b09b69b477c89bd962fa38` |
| `backup/20260927/wip/scm-train-state--scratch--SCMessenger-v040-harness-plan/fc85095fa4af` | `a3caaf481f627cf109384ba043fb01126817658c` | `0427ce3a20c2b3d4aebb41669f90c4eda38d2cf9` | `fc85095fa4af29fe32264e7f6d481a7b208b2156` |
| `backup/20260927/wip/scm-train-state--scratch--swa-20260925/398bb4cf548e` | `720e16e8832ca67447528b21f04c7f741ca5f4c9` | `c6fcf86a765de9ad461c5b19701600e122ec9bed` | `398bb4cf548e4eda60cf6bf5f8473dbd5d91a5dc` |
| `backup/20260927/wip/SCMessenger--tmp--harness-plan-snapshot-20260924/0815e699693d` | `1822691330ef058e192ce8f57ea47971a8ea449d` | `646d7bd8874f28a13cfb15ee9a4529ac26e4cff7` | `0815e699693d55fe8ded12650d6db70904ec8a97` |
| `backup/20260927/wip/SCMessenger--tmp--repro-candidate/440722c48d10` | `6bae6eb3e74bc1a59b722cebfe2f9ad5b5b0db3f` | `4e4f4f038b47e0f4f25cc20e3fd460d2616a3431` | `440722c48d108811dd2610f35c2fa94d29b9eecb` |
| `backup/20260927/wip/SCMessenger--tmp--repro-candidate2/d8e170673d92` | `11ec3544c4275523c7e8ba4ba1b4284fb987adb1` | `c3fee8d4b83aeac74f7e6491026b42020b49feff` | `d8e170673d92b8b4105a4ade1165ed17259664de` |
| `backup/20260927/wip/SCMessenger--tmp--repro-candidate3/9cab9880a856` | `0ebd9bb2fa80c0d9ef591ff26e126cc5d40ce08c` | `82d0ab5441877c3fcc37bc302f47d99d04c23464` | `9cab9880a856e8a627d28f73958b66bd741a1011` |
| `backup/20260927/wip/SCMessenger--tmp--repro-candidate4/655edf9bef7e` | `dbe5935f9e1372313e49b7fa4a70adbbe9c847b0` | `7078cf159b1dd4a8b8fafa8737b2050dbe0e9a60` | `655edf9bef7e854257aa5848043adc6f89611d35` |
| `backup/20260927/wip/SCMessenger--tmp--repro-candidate5/d22eac902336` | `254d1185e2fb8384e78153a75281ad979be0dd96` | `9467a2bc9abeee375f52106ae090beb252042d7b` | `d22eac902336fe448d9f2750c1dbf9916ca438dd` |

No remote ref or local commit was modified in this reconciliation. The backup verification script compares exact commit SHAs, so it would still reject these 16 ledger entries despite matching trees. The prior run log and this read-only tree comparison do not establish the complete backup acceptance criteria, particularly the ignored-file release assets. The next step needs an operator ruling on whether to change the verification contract for tree-equivalent captures or use a newly approved, non-overwriting reference scheme; neither was attempted.

[OPERATOR GATE] BK-01-BACKUP-VERIFY: direct the permitted non-overwriting way to reconcile the 16 commit-SHA mismatches and complete backup verification, or STOP.
[WHY] All 16 expected Git trees match their existing remote refs, but none of the attempted commit SHAs is the remote tip and the prescribed verifier is SHA-exact; a network failure also occurred. A11 does not authorize overwriting backup refs. | [BLAST RADIUS] No remote ref was written or rewritten, no files were purged, and no worktree was touched; current local data remains in place.
Reply: GO BK-01-BACKUP-VERIFY | STOP

This gate concerns only reconciling the backup state. It does not authorize `purge --dry-run` or purge. Those steps remain subject to the separate conditions in task section 4/P0-8, including a verified 0-missing result, acceptance review of `decisions.tsv`, and the 14.4 operator-only session-pause prerequisite.

No secret or vault paths are included.
