# Build & CI Rules

Status: Active
Last updated: 2026-09-24 (Harness immutable-tag admission and bounded canary policy)

Loaded on demand. The always-on summary lives in `CLAUDE.md`; this file holds
the detail. Prefer the `build-verify` skill over running these commands by hand.

## CI queue hygiene (standing practice — operator 2026-09-20)

GitHub Actions runners are a **shared finite resource**. A merge train that
leaves superseded runs queued makes every later gate wait for work that can no
longer land. **Common practice, not an emergency measure:**

1. **After every merge to `main` (or update-branch on a PR), cancel superseded
   runs** that cannot contribute evidence for the SHA you still care about:
   - push/workflow runs on **older `main` SHAs** that are no longer tip
   - pull_request runs on **branches whose PR is already MERGED or CLOSED**
   - docs-only PR runs when the PR is not being merged this hour (optional;
     free the slot for the candidate SHA)
2. **Keep only the live candidate.** For fleet deploy that is usually one
   `main` tip SHA. Required contexts + artifact jobs (CI Windows CLI, Mobile
   Android APK, Docker Publish) on that SHA are what you wait on — not the
   full history of the merge train.
3. **Cancel commands (orchestrator / host agents):**

```
gh run list --limit 40 --status queued
gh run list --limit 40 --status in_progress
gh run cancel <run-id>
```

   List **all** live runs (do not assume the API default page is the total —
   rule 15). Cancel by run id; a cancel that returns "already completed" is
   fine.
4. **Zombie runs:** a run stuck `queued` for days on a deleted workflow branch
   (e.g. historical `fix/h2-rustsec-*` Hygiene) may be uncancellable
   (`Cannot cancel a workflow run that is completed` or never dispatches).
   Record it; it does not consume runners when it never starts. Do not treat
   it as a reason to wait.
5. **Never cancel required contexts on the SHA you are about to merge** or
   artifact jobs you need for deploy (`windows-cli-<sha>`, `android-debug-apk`,
   Docker Publish on the deploy SHA).
6. **When waiting on CI, say what you are waiting for** (workflow name +
   head SHA + run id), not "CI is slow."
7. Same rule applies **before** a long `update-branch` storm: one PR at a
   time when the queue is saturated; cancel that PR's old head runs after the
   new push.

Related: branch protection `strict: true` serializes merges and re-queues
every other PR (SHIP_PLAN I-31). Queue hygiene does not remove that cost; it
stops you paying it twice for dead work.

## JEV / harness verification (orchestrator + implementers — 2026-09-21)

Standing practice for SCMessenger completion work:

1. **Toolchain:** **SCMessenger-local** harness only —
   `python scripts/update_local_harness.py` refreshes
   `vendor/sovereign-harness` from the official harness GitHub remote.
   Do **not** edit `Documents/GitHub/Harness` product trees.
   Callers use `scripts/local_harness.py` (override: `HARNESS_REPO`).
2. **Insight / sentiment batches:** `python scripts/jev_repo_insights.py --mode full`
   (includes issue-sort via `JevPolicy.evaluate_issue_sort` + frozen pack
   `scripts/scmessenger_issue_sort_pack.json`).
3. **WP / canonical DONE:** mechanical CI + greps **and**
   `python scripts/jev_canonical_check.py --wp WPn --state-file ...`
   (`is_passing` at min_confidence 0.70). Unkeyed fallback → `UNVERIFIED-JEV`.
4. **Clarification:** if confidence <99% on a claim/design, run harness verify
   or a typed JEV question pack — do not invent a new root-cause plan.
5. Full design: `HANDOFF/V040_JEV_HARMESS_INTEGRATION_2026-09-21.md`.

### Harness admission and update gate (2026-09-24)

The production Harness source of truth is the immutable release tag `v0.4.1`
and its peeled commit, not a moving branch and not a local checkout. The
verified baseline is:

- Harness `v0.4.1` -> `ad4a30052955e1f574c90f426edfc7a274e16ebf`.
- Harness `origin/main` -> `6d5a2f818d3029b10635566a8131c6e8be234371`,
  untagged and 55 commits beyond `v0.4.1`; it is a bounded canary source only.
- The local Harness checkout is 30 commits behind `origin/main` and dirty;
  it is diagnostic context, never production evidence.

Admission is fail-closed. A candidate is admitted only after an exact remote
ref is resolved, the source is clean, the package and public/private SCMessenger
imports are checked, the Harness CLI/report contract is checked, and the
upstream CI run for the exact SHA is green. `origin/main` may be tested in a
separate `tmp/` staging path for 0.5.0 compatibility, but a moving branch must
never be used directly by a release gate. If the branch advances, the old
canary is invalid and a new exact SHA must be admitted.

SCMessenger owns the consumer wrapper, admission manifest, and rollback
procedure. The Harness maintainer owns upstream tags, API compatibility, and
Harness CI. The SCMessenger orchestrator owns the 0.4.0 freeze and merge
sequence; platform owners own Android, CLI, cloud, and native behavior. No
external Harness worktree is edited by SCMessenger.

The local update flow below is the required target behavior. The current
floating updater is not yet compliant and must not be treated as admission
until the modes and exact-SHA checks are implemented.

1. `bootstrap`: create a clean consumer copy from the admitted tag in
   `tmp/harness-admission/<tag>-<sha>`; never use the system temp directory.
2. `admit-tag`: fetch the tag, verify the peeled commit, run the hermetic
   contract probe, and record the exact SHA before promotion.
3. `canary-main`: resolve one `origin/main` SHA, run the same probe in a
   separate staging path, and label the result `CANARY`, never production.
4. `rollback`: restore the previous admitted tag/SHA and its manifest; leave a
   failed candidate in place for evidence rather than overwriting the active
   consumer copy.

A dirty vendor copy, unknown version, missing imported symbol, wrong remote,
wrong SHA, report-schema mismatch, or unavailable required key is a refusal,
not a warning. Structural JEV fallback is never a canonical pass. The existing
consumer scripts (`local_harness.py`, `jev_canonical_check.py`,
`jev_repo_insights.py`, `harness_gate.py`, and `bod_governance.py`) must all
resolve the same admitted source; direct installed-package fallback is not
allowed.

## Windows parallelism (measured on this box)

Machine: AMD Ryzen 7 7730U, 16 logical / 8 physical cores, 11.8 GB RAM. The
binding constraint for cargo is **RAM, not core count**.

- **Default: `-j12`** when the box is otherwise idle. This is the operator's
  standing default -- do not silently downgrade it. Applying `-j2` to every
  build "to be safe" once turned routine gates into 20+ minute waits.
- **RAM contended** (another cargo/gradle/java session live): `-j6`.
- **Cold post-`cargo clean` full-workspace build:** start at `-j4`, drop to
  `-j2` only if rustc actually dies.
- Keep `CARGO_INCREMENTAL=0` (set for Windows in `.cargo/config.toml`); also
  `export CARGO_INCREMENTAL=0` in the shell before cargo commands.
- **Shared warm target directory:** Export
  `CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target` across all
  worktrees to avoid duplicate target/ directories (see details below).
- **A rustc crash is resource exhaustion, not corruption.**
  `STATUS_STACK_BUFFER_OVERRUN` (0xc0000409), "can't find crate" for a crate
  that just built, or "import resolution is stuck" all mean memory pressure.
  Retry the identical command at lower `-j` before concluding a code or
  toolchain problem. This workspace pulls libp2p, quinn, ring, libcrux-ml-kem
  and uniffi concurrently with debuginfo=2.
- `cargo clippy` and `cargo build`/`test` use separate artifact caches
  (clippy-driver vs rustc), and the two clippy variants (default-features vs
  `--all-features`) do not share artifacts either. Sequence gates once at the
  end rather than after each edit; during iteration `cargo check -p <crate>`
  is far cheaper.
- Never run two build-tool invocations concurrently. Multiple agent sessions
  share this repo, and Gradle can spawn cargo-ndk upstream.

## CI-Primary Build Doctrine (operator directive, 2026-09-22)

**CI is the primary verifier. Local builds are the failover.** A push that
carries the applicable gates is how work is verified; a local build is the
exception, not the default.

- **Default loop:** commit scoped work -> push -> `gh run watch <run-id>` ->
  triage failures from `gh run view <run-id> --log-failed` -> download any
  needed artifact (`gh run download <run-id> -n <name> -D tmp/<dir>`, always
  under `tmp/`). Do not build locally what CI is already building.
- **Local build is justified only by:** CI unavailable/red for infra reasons,
  a failover debugging session CI cannot reproduce, or a gate the workflows
  do not run. Everything else waits for CI.
- **If you do build locally, reclaim immediately afterwards.** The build is
  not done when the command exits -- it is done when the disk is given back:
  `python scripts/disk_budget.py` before, `python scripts/reclaim_safe.py --reclaim`
  (or `scripts/clean_target.sh` for scoped output) after, same session.
  Holding a warm `target/` "for later" is a rules violation, not a convenience.
- **Fail closed on BLOCKED disk.** `scripts/disk_budget.py` exit 2 means no
  local build, no exceptions: pull the artifact from CI instead. A build that
  would fill the disk is how 2026-09-17 happened.
- **Prefer committing over building.** An uncommitted fix has no provenance
  and cannot be deployed or verified by CI. When in doubt: commit to a branch,
  push, let CI run the wide sweep. (This lesson was paid for 2026-09-22: the
  V040-T-CONN-04 fix was cross-built and deployed to two live nodes from an
  uncommitted working tree before anyone committed it.)
- The one-build-at-a-time rule, `build_lock.py`, and `CARGO_INCREMENTAL=0`
  apply to failover builds exactly as before.

## Build Verification (Mandatory)

Scoped to what changed, before finalizing any run (prefer the `build-verify`
skill):

1. Rust edits: `cargo build --workspace` (record output in HANDOFF notes).
2. Android edits: `cd android && ./gradlew assembleDebug -x lint --quiet`.
3. WASM edits: `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`.
4. Format: `cargo fmt --all -- --check`.
5. Lint: `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`.

Compile gate: `cargo test --workspace --no-run` must pass before any task is
considered complete.

## Checks that fail as *plausible emptiness*

The pipe trap below is one instance of a recurring class: **a check that did not
run looks identical to a check that found nothing.** Every one of these has
produced a wrong conclusion in this repo. When a command returns no output,
confirm it *executed* before concluding the thing is absent.

**Never read `$?` after a pipe.** A pipeline's exit status is the LAST
command's, so `cargo fmt --check | head; echo $?` always reports 0 and the gate
cannot fail. Capture the status of the command itself. Piping a gate through
`tail` for readability silently disarms it -- re-run unpiped to get a real exit
code.

**`git show <rev>:<path>` mangles dotted paths under Git Bash.** MSYS path
conversion rewrites `<rev>:.github/workflows/x.yml` into
`<rev>;.github\workflows\x.yml`; git then errors with `fatal: ambiguous
argument`. With stderr redirected this reads as an empty or truncated file.
Paths under `core/src/` usually survive, which makes the failure look random.
Prefix with `MSYS_NO_PATHCONV=1`, and sanity-check that the output looks like
file content.

**`tasklist /FO CSV /NH` returns nothing under Git Bash.** It mangles `/FO` into
`C:/Program Files/Git/FO`, tasklist errors, and you get a false "process not
running". Use plain `tasklist | grep -i <name>`.

**A `:latest` container tag is not a version.** `docker-publish.yml` applies
`latest` only via `enable={{is_default_branch}}`, so a dispatch from any branch
publishes branch and sha tags and leaves `latest` pointing at the last `main`
build. Pulling `:latest` to deploy a candidate silently ships the wrong commit
while the node reports healthy. **Deploy by immutable digest**, then gate on the
node's `/version` reporting the expected commit -- treat that as a gate, not a
formality.

**Multi-model helper output can be empty while reporting success.** Reasoning
models spend the completion budget on hidden reasoning tokens and return
`finish_reason="length"` with `content=""`. `scripts/fusion_lite.py` now caps
reasoning effort and falls back to the reasoning trace, but check the character
count of what came back rather than trusting the exit status.

## Disk Space Preflight (Windows & macOS)

Before running a full gate sweep, verify free disk space via
`scripts/preflight_disk.sh` (or `.ps1` on Windows). Not CI-enforced, but
mandatory for local and agent-driven builds to prevent OOM/disk-full crashes.

**Constraint:** A full five-gate sweep (fmt, clippy default, clippy
--all-features, `cargo test --workspace --no-run`, wasm release) regrows
`target/` to ~40-47 GB. Measured evidence: three consecutive runs reclaimed
42.7 GB, 35.7 GB, and 47.2 GB. Threshold: 25 GB minimum free space. The C:
drive on this box runs near 97% full -- check `df -h /c` before assuming a
build failure is a code problem.

**Critical traps:**

1. `cargo clean --target <triple>` does NOT scope to a single target -- it wipes
   ALL of `target/`. Verified: intended to reclaim ~4 GB, deleted 44.7 GB.
2. `scripts/ffi_surface.sh` silently depends on `core/target/generated-sources/`.
   Note the path: that directory lives under `core/target/`, which is SEPARATE
   from the workspace `target/`. Measured 2026-07-27: a plain `cargo clean` from
   the workspace root removed 22,557 files / 47.1 GiB from `target/` and left
   `core/target/generated-sources/` intact, so a root clean does NOT require
   regenerating bindings.
   What DOES destroy it: `cargo clean` run from inside `core/`, `cargo clean
   --target <triple>` (see trap 1 -- it wipes everything), or deleting
   `core/target` directly. After any of those, regenerate bindings (`gen_swift`,
   `gen_kotlin`) and verify the files exist before running `ffi_surface.sh
   --update` -- skipping that check produced a vacuous "Updated Swift snapshot"
   with exit 0 and no bindings, twice.
   Cheap insurance before any clean: `cp -r core/target/generated-sources <tmp>`
   (1.2 MB).

**Use the script, not raw commands.** Both traps above are handled by
`scripts/clean_target.sh`, which never invokes `cargo clean` at all -- it removes
directories by explicit path, which is the only way to actually scope the
operation. It also backs up and verifies `core/target/generated-sources/`, and
refuses to run while a build tool is live (deleting objects under a running
cargo corrupts the build in ways that look like source errors).

```bash
scripts/clean_target.sh --dry-run --all   # always look first
scripts/clean_target.sh --triples         # cross-compile outputs only
scripts/clean_target.sh --deps            # debug intermediates, KEEPS binaries
scripts/clean_target.sh --all
```

`--deps` preserves built binaries in `target/debug/`, so a running CLI node
survives the clean and does not need a rebuild to restart. Measured 2026-08-03:
`--all` reclaimed ~51 GB (30 GB of `target/debug/deps` plus 20 GB of Android
triples) with the node still running.

Do NOT reach for `cargo clean --target <triple>` to reclaim one triple. It does
not do that. Use `--triples`, or delete the specific `target/<triple>/` path.

## Shared Warm Cargo Cache (`CARGO_TARGET_DIR`)

All dispatched workers and agent sessions on this host must reuse the shared warm build cache by exporting `CARGO_TARGET_DIR`:

```bash
export CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target
```

In PowerShell:
```powershell
$env:CARGO_TARGET_DIR = "C:\Users\SCM\Documents\GitHub\.scm-shared-target"
```

- **Why it exists:** Each isolated worktree previously initialized and built its own `target/` directory. With heavy workspace dependencies (libp2p, quinn, ring, uniffi, etc.), a single target directory rapidly expands to 16-45 GB. On 2026-08-15, isolated worktrees filled a 237 GB drive to 99%, triggering compiler crashes and disk-full errors.
- **Shared Target Dir:** `C:/Users/SCM/Documents/GitHub/.scm-shared-target` acts as a unified shared warm cache across all worktrees. This eliminates redundant dependency compilation, speeds up incremental builds, and saves tens of gigabytes of disk space.
- **Concurrency Caveat:** Concurrent cargo builds against a shared target directory will block on the cargo file lock. This is a feature here, not a bug -- it naturally serializes cargo invocations and reinforces the rule that two build tools must never execute concurrently.

## Docs Sync

Run `./scripts/docs_sync_check.sh` (or the `.ps1`) after any documentation
change; resolve failures before finalizing. The `docs-sync` skill wraps this.

## Path Conventions (CI Enforced)

Enforced by the `Repository Hygiene` workflow (`.github/workflows/hygiene.yml`):

- `iOS/` uppercase-I in ALL path references; XCFramework at
  `iOS/SCMessengerCore.xcframework/` (step: `Verify path governance rules`).
- No `.py` in repo root (use `scripts/`); no build artifacts committed
  (`git ls-files "*.log" "*.pid" "*.logcat"` must be empty) (step:
  `Verify root directory layout`).
- Keep the repo root minimal. Documentation belongs under `docs/` (with
  historical material in `docs/historical/`), executable scripts under
  `scripts/`. Only tooling-mandated files and GitHub community-health files
  belong at the root.

## Reading CI failures

`gh` is authenticated as Treystu. Read failures with
`gh run view <id> --log-failed` -- do not guess at causes. The repo lives at
`Sovereign-Communication/SCMessenger` (public); macOS runners execute, so the
iOS lane is unblocked.

## Windows shell notes

- Shell scripts need Git Bash/WSL. CI is ubuntu/macos only. Since
  2026-09-22 CI is the PRIMARY verifier and local Windows builds are the
  FAILOVER -- see "CI-Primary Build Doctrine" above and
  `docs/runbooks/CI_PRIMARY_BUILD.md`.
- `python3` is a shim at `~/.local/bin/python3.exe`; orchestrator scripts
  hardcode `python3` but only `python` exists natively.

## Model Availability Check (the `swarm` backend ONLY)

Only when using the `swarm` backend (ollama pool): verify the target ollama model
via `bash .claude/model_validation_template.sh` or `https://ollama.com/api/tags`.
Not applicable to the `lanes`, `native`, or `agent` backends -- for `native` the
model truth is `claude --help` aliases; for `lanes` it is the lake registry
`docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md`.
