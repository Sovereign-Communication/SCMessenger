# V040-T6 -- Tier A two-node continuous conformance harness

Status: OPEN (filed 2026-08-31, operator directive)
Priority: P1 -- this is what makes "never idle" real rather than aspirational
Lane: Freebuff / DeepSeek V4 Flash
Scope: new `scripts/tier_a_conformance.sh` plus a results file it writes. No
changes to `core/` or `cli/` source. Read-only against the live nodes.

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
| A6 | Custody live | `custody_audit_count` is present and non-decreasing across runs |
| A7 | **Ledger sanity** | `storage/ledger.json` entry count is non-zero on both. Today Windows reports **0** -- this row is expected to FAIL until T2 lands, and that is the point: it should fail loudly rather than be rediscovered |
| A8 | No self-entries | Neither node's peer store contains its own identity or its own external address (issue I-06) |
| A9 | Listener surface sane | Report the listener count. `[WARNING]` above 12. The AWS node currently binds **33**, including 80/443/8080/9090, and cross-dials its own listeners (issue I-12) |
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

**I-15: `HANDOFF_AUDIT/REPO_MAP.jsonl` lies about the codebase.** It contains
stale AI-generated `calls` entries asserting call sites that do not exist in
source; it already misled an agent into believing `routing_peer_seen` had
callers. Either regenerate it from source, or add a header line marking it
stale and untrustworthy with the date. An artifact agents trust and that is
wrong costs more than no artifact.

**I-14: `scratch/driver/watcher_run.cmd`** claims persistence via a
`SCMessengerDriverWatcher` ONLOGON scheduled task. That task is not registered;
persistence is actually a Startup-folder shortcut. Correct the comment. (This
file is untracked and owned by another session -- correct only the comment, do
not restructure it.)

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
