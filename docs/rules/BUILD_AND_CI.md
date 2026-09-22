# Build & CI Rules

Status: Active
Last updated: 2026-08-08 (extracted from `.claude/rules/build.md` for Tier 1
on-demand loading; Windows parallelism section added)

Loaded on demand. The always-on summary lives in `CLAUDE.md`; this file holds
the detail. Prefer the `build-verify` skill over running these commands by hand.

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
