# Runbook: CI-Primary Build and Verification

Status: Active
Created: 2026-09-22 (operator directive: CI primary, local builds failover)
Authority: `docs/rules/BUILD_AND_CI.md` section "CI-Primary Build Doctrine"

## The default loop (every change)

```bash
# 1. Commit scoped work (explicit paths, never -A)
git add <your files>
git commit -m "..."

# 2. Push the branch; CI runs the gates
git push origin <branch>

# 3. Watch the run
gh run list --branch <branch> --limit 1
gh run watch <run-id>

# 4. Triage failures from evidence, never from guessing
gh run view <run-id> --log-failed
```

Green run = verified. The run URL is the evidence line in any report.

## Getting artifacts instead of building them

```bash
gh run download <run-id> -n android-debug-apk -D tmp/ci-apk/<run-id>/
```

Artifacts always land under `tmp/` (rule 2). Known artifact names:
`android-debug-apk` (mobile.yml), release APKs and CLI artifacts
(release.yml), docker images via `docker-publish.yml`.

## When a local build is justified

Only: CI unavailable/red for infra reasons; failover debugging CI cannot
reproduce; a gate the workflows do not run. Everything else waits for CI.

## Failover build procedure (all steps mandatory)

```bash
# 0. Preflight -- BLOCKED (exit 2) means no build, use CI
python scripts/disk_budget.py

# 1. Serialize (this host runs one build at a time)
python scripts/build_lock.py --run "cargo test -p scmessenger-core --lib <one_test>"

# 2. Export the incremental-off and shared-target env (see BUILD_AND_CI.md)
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target

# 3. ... build/test ...

# 4. RECLAIM IMMEDIATELY (same session, not later)
python scripts/reclaim_safe.py            # survey first
python scripts/reclaim_safe.py --reclaim  # then delete what is safe
```

Skipping step 4 is a rules violation. The build is not done when the command
exits; it is done when the disk is given back.

## Verification chain for gated code (core/src/{crypto,transport,routing,privacy})

Local or CI, the sequence is: unit tests -> wiring gate
(`python scripts/check_wiring.py`) -> push -> CI wide sweep -> Rule-8
adversarial review on file before merge. A green CI run does not replace the
review; the review does not replace CI.

## Provenance rule (paid for 2026-09-22)

Never deploy from an uncommitted tree. Commit first, push, let CI build, then
deploy the CI artifact (or a locally built binary whose source commit is
recorded in the deployment note). The V040-T-CONN-04 fix ran on two live
nodes for hours before its source existed in a commit — that is the failure
shape this rule closes.
