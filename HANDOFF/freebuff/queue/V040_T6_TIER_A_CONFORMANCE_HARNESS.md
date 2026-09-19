# V040-T6 -- Tier A two-node continuous conformance harness

Status: MERGED -- the harness landed as PR #311 (main 1579b049, 2026-09-19), the
same day this status line was corrected. Status corrected 2026-09-19 by the
queue-reconcile pass; the ticket body below is unchanged.

## Hand-off (2026-09-18) -- implemented, in review as PR #311

Branch `freebuff/tier-a-conformance`, worktree `tmp/wt-t6-conformance`.
`scripts/tier_a_conformance.sh`, plus `scripts/aws_node_ip.sh` -- the EC2 tag
lookup extracted from `aws_deploy.sh` so it exists once, with `aws_deploy.sh`
consuming it instead of carrying its own copy (behaviour-preserving:
`aws_deploy.sh` -22/+9, `aws_node_ip.sh` new, +63).
Results: `scratch/driver/tier_a_conformance.json`, `current` plus `previous`;
a field a run could not measure carries the last known good value forward, so a
degraded run cannot erase the baseline.

Acceptance -- all four met, each exercised against the live rig on 2026-09-18:

1. Ten rows plus the churn `[SKIP]`, and a non-zero exit while any row is
   `[FAIL]`: a live run reported A2/A4/A8 `[FAIL]` and exited 1. `--help` exits 0.
2. Real live output is pasted into PR #311, including the known-bad rows.
3. Address change works both ways: `SCM_AWS_HOST=<ip>` is used verbatim, and the
   EC2 path resolves the current address (18.234.62.247).
4. Never reads `$?` after a pipe: the single occurrence is `local rc=$?`, imme-
   diately after two redirections in `fetch_remote_ledger`, never after a pipe.

### Corrections to this ticket's own expectations (evidence, per row)

- **A6's pass condition cannot hold on a healthy rig.** `custody_audit_count` is
  a key count, not a cumulative counter: `audit_count()` returns
  `count_prefix(CUSTODY_AUDIT_PREFIX)` (`core/src/store/relay_custody.rs:1260-1262`,
  prefix `relay_custody_audit_` at `:21`), and delivery (`remove_message`, `:1520`)
  and storage-pressure purge (`purge_oldest_by_policy`, `:1402`) both lower it.
  Measured `11239 -> 11207` across consecutive runs on a healthy node on
  2026-09-18, so a literal "non-decreasing" rule `[FAIL]`s a working rig.
  Implemented behaviour keeps the ticket's intent -- present, and `[FAIL]` only at
  zero (proved reachable: a stub with `custody_audit_count: 0` printed
  `[FAIL] A6 custody EMPTY on: windows`) -- with a decrease reported as
  `[WARNING]` carrying its delta. **The wording of A6 is the operator's decision,
  not the agent's; the code does not amend this ticket.**
- **A7 is stale -- it passes now.** T2 landed: measured `entries=133` (windows)
  and `entries=230` (cloud), where this ticket expected Windows to report 0 and
  the row to fail until T2 landed.
- **A9's listener count is stale.** The ticket says the AWS node binds 33; on
  2026-09-18 it reports 28 (windows: 17). The row warns above the threshold of 12
  on both, as designed.
- **A10 needs a caveat when run from a worktree.** The row resolves
  `scratch/driver/watcher.log` relative to the tree the script is run from, so a
  worktree without that file reports `[WARNING] not present` even when the main
  checkout's watcher is alive. Not changed here; noted so the output is not
  misread.
- Live findings still open (not harness defects): **A2** -- both nodes report
  `0fd69fb` while `origin/main` is `20cfb91`; **A4** -- neither node lists the
  other, both reporting only `12D3KooWD776...`; **A8** -- I-06 self-entries
  windows 7 self / 3 own-address, cloud 1 / 2.

### The two named extras -- neither landed, both are operator questions

**I-15 -- the premise does not survive contact with the repo.** The file this
ticket names, `HANDOFF_AUDIT/REPO_MAP.jsonl`, exists and is tracked, but it
carries **no `calls` key at all**: 258 entries whose keys are `chunk`, `file`,
`funcs`, `imports`, `structs_or_classes`, `summary`, with call claims inside
`funcs[].calls_out_to`. The `calls` schema described here exists only in four
chunk files under `HANDOFF_AUDIT/output/`. The demonstrably stale artifact is
instead `HANDOFF/discovery/REPO_MAP.jsonl`: **11 of its 25 entries name files that
do not exist** (`adb_extractor.py`, `count_braces.py`, `fix_contactmanager.swift`,
`fix_swift_generation.py`, `fix_swift_strings.py`, ...), and it begins with a UTF-8
BOM, so a plain `json.load` on it raises on line 1. Open questions: which artifact
should be marked or regenerated -- the audit map this ticket names, or the
discovery map that is provably stale -- and regenerate versus a dated stale
header? Deliberately not implemented, since either choice is the operator's.

**I-14 -- not deliverable by a pull request.** `scratch/driver/watcher_run.cmd`
exists (295 bytes) but `.gitignore:380` (`scratch/*`) ignores it, so a corrected
comment cannot be committed from a worktree and cannot reach a PR. It needs an
in-place edit in the shared checkout by the operator, or this item re-targeted at
a tracked file.

## Why

Operator directive 2026-08-31: the two always-available nodes (AWS Linux +
Windows CLI) must be driven to full v1.0.0 conformance continuously, and work
must not stall when the Android handset is away. Policy:
`docs/rules/CONTINUOUS_EXECUTION.md`.

Today there is no single command that answers "are the two nodes conformant right
now?" Every session re-derives it by hand with ad-hoc `curl` calls, which is how
a 13-hour `peers: []` outage and a 7-hour dead watcher both went unnoticed.

## What to build

One script, `scripts/tier_a_conformance.sh`, that probes both nodes and prints a
pass/fail matrix. It must be **safe to run at any time** -- read-only, no
restarts, no writes to node state.

Node discovery, in this order, so a changed AWS address never breaks it:

1. `$SCM_AWS_HOST` if set.
2. The EC2 API, exactly as `scripts/aws_deploy.sh` already does it (reuse that
   logic; do not duplicate a second copy of the lookup).
3. Fail with `[FAIL] could not locate the AWS node` -- never fall back to a
   hardcoded address. A hardcoded address is what produced issue I-02.

Windows node: `http://127.0.0.1:9876`.

## The conformance rows

Each row prints `[OK]`, `[FAIL]`, or `[WARNING]` with the observed value. Exit
non-zero if any row is `[FAIL]`.

| Row | Check | Pass condition |
|---|---|---|
| A1 | Both nodes reachable | `/health` returns `{"status":"healthy"}` on both |
| A2 | **SHA parity** | Both report the same git hash, and it matches `git rev-parse origin/main`. A result gathered across mismatched SHAs is not evidence |
| A3 | Identity stable | `/api/identity` `identity_id` on each matches the value recorded in the results file from the previous run. Changed identity = persistence regression (issue I-01) |
| A4 | Mesh formed | Each node lists the other in `/api/diagnostics.peers` |
| A5 | Connection path | `connection_path_state` is not `Bootstrapping` on either |
| A6 | Custody live | `custody_audit_count` is present and non-decreasing across runs (see Hand-off: as written this cannot pass on a healthy rig) |
| A7 | **Ledger sanity** | `storage/ledger.json` entry count is non-zero on both. Today Windows reports **0** -- this row is expected to FAIL until T2 lands, and that is the point: it should fail loudly rather than be rediscovered (see Hand-off: stale, T2 landed, the row passes) |
| A8 | No self-entries | Neither node's peer store contains its own identity or its own external address (issue I-06) |
| A9 | Listener surface sane | Report the listener count. `[WARNING]` above 12. The AWS node currently binds **33**, including 80/443/8080/9090, and cross-dials its own listeners (issue I-12) (see Hand-off: 28 on 2026-09-18) |
| A10 | Watcher alive | `scratch/driver/watcher.log`'s last line is under 2 hours old. It was dead for 7 hours on 2026-08-31 and nothing noticed (issue I-13) |

Write results to `scratch/driver/tier_a_conformance.json` -- current values plus
the previous run's, so A3 and A6 can compare. Keep it to one file; do not
accumulate per-run artifacts.

## Explicitly out of scope for this ticket

The churn row (redeploy the AWS node, take a new IP, confirm the mesh re-forms
unaided) is **not** automated here. It requires a redeploy, which is not
read-only. It stays operator-run as SHIP_PLAN G3-0. Print it as a `[SKIP]` row
naming the command, so its absence is visible rather than forgotten.

## Acceptance

Per-criterion status is recorded in the Hand-off above.

1. `bash scripts/tier_a_conformance.sh` prints all ten rows and exits non-zero
   while any is `[FAIL]`.
2. Run against the live rig, and paste the real output into the PR. Expect A7 to
   fail and A9 to warn today -- a run where everything passes on the first
   attempt means the checks are not actually reading live state, so verify
   against the known-bad rows.
3. It works when the AWS address changes: `SCM_AWS_HOST=<other> ... ` and the
   EC2 path both resolve.
4. Never reads `$?` after a pipe. `cmd > out.txt; rc=$?; head out.txt; exit $rc`.

## Also fix in this PR -- two cheap, high-leverage items from the ledger

Neither landed; the premise check for each is in the Hand-off above.

**I-15: `HANDOFF_AUDIT/REPO_MAP.jsonl` lies about the codebase.** It contains
stale AI-generated `calls` entries asserting call sites that do not exist in
source; it already misled an agent into believing `routing_peer_seen` had
callers. Either regenerate it from source, or add a header line marking it
stale and untrustworthy with the date. An artifact agents trust and that is
wrong costs more than no artifact. (Premise check: that file has no `calls` key;
the stale map is `HANDOFF/discovery/REPO_MAP.jsonl`. See Hand-off.)

**I-14: `scratch/driver/watcher_run.cmd`** claims persistence via a
`SCMessengerDriverWatcher` ONLOGON scheduled task. That task is not registered;
persistence is actually a Startup-folder shortcut. Correct the comment. (This
file is untracked and owned by another session -- correct only the comment, do
not restructure it.) (Premise check: it is also gitignored, so it cannot be
committed from a worktree. See Hand-off.)

## Not in this ticket -- flagged for the operator

**I-03: `.codebuff_deploy/aws/launch.py` still omits the `/data` mount.** That
omission is what caused the identity-loss incident, and it is still live: any
future instance replacement launched from it re-breaks persistence.
`scripts/aws_deploy.sh` is now the only correct path. The file is untracked and
owned by another session, so it is not edited here -- the operator should delete
it or point it at `aws_deploy.sh`.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`, `[SKIP]`.
- Read-only against live nodes. No restarts, no state writes, no redeploys.
- Shared checkout: touch only what this task requires.

### Status reconciliation (2026-09-19)

The status line still said "PR FILED -- #311 open". It is merged.

```
gh pr view 311 --json state,mergedAt,headRefOid,files
#  MERGED  head 3adee565  4 files (tier_a_conformance.sh, aws_node_ip.sh,
#  aws_deploy.sh, this ticket)
git show origin/main:scripts/tier_a_conformance.sh | grep -nE "^(measured|state_arg)\\(\\)"
#  356:measured() { ... }
#  363:state_arg() {
git ls-tree --name-only origin/main scripts/ | grep -E 'tier_a_conformance|aws_node_ip|aws_deploy'
#  all three present
```
