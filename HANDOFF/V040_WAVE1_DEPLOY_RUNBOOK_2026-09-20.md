# Wave-1 fleet deploy notes (orchestrator)

Authority: `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`
Rule: CI artifacts only; binaries under `tmp/radio-<sha>/`, never `target/`.
Disk: prefer artifacts; `python scripts/disk_budget.py` before downloads.
Pixel: `adb install -r` allowed; **no UI driving**.

## CI queue hygiene (mandatory before waiting on artifacts)

Operator 2026-09-20: **always clear superseded GitHub Actions runs** so deploy
is not stuck behind work that can no longer land. Full policy:
`docs/rules/BUILD_AND_CI.md` ("CI queue hygiene").

Before/while waiting for deploy artifacts:

```
gh run list --limit 40 --status queued
gh run list --limit 40 --status in_progress
gh run cancel <run-id>   # superseded main SHAs, merged-PR branches, idle docs PRs
```

**Keep:** `CI` / `Mobile` / `Docker Publish` on the **target deploy SHA** only.
**Cancel:** older main-tip runs, pull_request runs on merged branches, non-candidate
docs CI when the queue is saturated.
**Do not cancel:** artifact-producing jobs on the deploy SHA.
Record run ids you cancelled in the deploy report (evidence contract).

## Target SHA

Deploy **one** SHA to all three nodes after Wave-1 merges that are green.
Record SHA in the deploy report to the operator.

## Windows CLI

1. `gh run download <run-id> -n windows-cli-<full-sha> -D tmp/radio-candidates/<sha>`
2. Confirm `cli-provenance.txt` git_hash matches target.
3. Capture pre-state: `/version`, `/api/identity`, PID.
4. Graceful stop: `POST http://127.0.0.1:9876/api/shutdown`
5. Start new exe from `tmp/radio-candidates/<sha>/scmessenger-cli.exe`
   with the same argv as the live node (`start -p 9001 --auto-reply ...`).
6. Confirm `/health` + `/version` + identity unchanged.

## AWS cloud node

Precondition: Docker Publish green on **main** for target SHA
(`testbotz/scmessenger:sha-<short>` or `:latest` for main-only).

```
scripts/aws_deploy.sh
```

Discovers host from EC2 tag `scm-always-on-node`. Preserves
`/opt/scm-relay-data`. Do not terminate instances. After deploy:
`docker logs scm-node` build string + `/health` + `/version`.

## Pixel 6a

1. Find Mobile/Android Debug APK run for target SHA.
2. `gh run download <run-id> -n android-debug-apk -D tmp/radio-candidates/<sha>-apk`
3. `adb connect <lan>:<port>` if needed.
4. `adb install -r app-debug.apk` (signing pinned since #324/#326).
5. Passive `adb logcat` / `run-as` log pull only — operator drives UI.

## Report to operator

After all three: table of node, SHA, identity, health, evidence paths.
No D4/D6/D7 scoring until the whole Wave 1 has landed (operator ruling).
