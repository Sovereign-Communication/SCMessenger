# CircleCI wave-1 port - what was written, and what was checked

Date: 2026-11-06
Lane: `lane/burndown` (base `main` `ba7fa778`)
Spec: PR #653 comment 6675798232 (determined wave-1 spec), P4 addendum 6675498503

## Why the port is the first item

Main has no verifier that can return a verdict. The five required contexts are
produced by Actions workflows whose hosted jobs are refused by the account
billing lock, and whose self-hosted pool is two slots. Until a second rail
exists, `main` is UNVERIFIABLE and no green claim about it is admissible. The
port therefore outranks feature and verdict work.

## What was written

`.circleci/config.yml` (new, version 2.1), wave 1 = exactly the five
branch-protection contexts, job display names copied from the protection list,
commands copied from the Actions originals:

| Display name (required context) | Source | Jobs / commands carried |
|---|---|---|
| `Repository Hygiene Checks` | `hygiene.yml` | 13 hygiene steps + `check_required_contexts.py`, `check_toolchain_pin.py` (and `--self-test`), `check_workflow_yaml.py` (and `--self-test`), `check_queue_status.py` |
| `Lint` | `ci.yml` job `lint` | `cargo clippy --workspace --all-features -- -D warnings -A clippy::empty_line_after_doc_comments`, cargo-deny 0.20.2 pinned, `cargo update --workspace --locked --dry-run` |
| `Rust Linting` | `lint.yml` job `rust` | `cargo fmt --all -- --check`, `check_perimeter_underscore_params.py`, unwrap/println unit test + scan |
| `Test (ubuntu-latest)` | `ci.yml` job `test`, ubuntu leg | `cargo test --workspace --features test-utils` |
| `Handoff ownership scope` | `ci.yml` job `handoff-scope` | `validate_handoff_scope.py --changed-from <base>`, `--self-test` |

Executor: `cimg/rust:1.99.0` (Rust jobs, `resource_class: large`) and
`cimg/python:3.11` (git/python gates, `small`). System packages mirror
`.github/actions/linux-build-deps`: `libdbus-1-dev`, `pkg-config`,
`protobuf-compiler`, plus `libssl-dev` for image parity with hosted runners.
No secrets are used.

A preflight step asserts `rustc --version` is exactly `1.99.0`, so a moved
image tag cannot silently change the lint set under a green repository (the
2026-09-30 `clippy::double_must_use` incident, recorded in `rust-toolchain.toml`).

## Parity notes (the places CircleCI differs from Actions, handled explicitly)

1. CircleCI runs steps under `bash -eo pipefail`; Actions runs `bash -e`. The
   trailing-whitespace step reads the exit status of `git diff | grep -v`, so
   under pipefail a CLEAN tree makes `grep` exit 1 and the step would fail. The
   step disables pipefail for its own body rather than changing the check.
2. Actions checks out a synthetic merge commit on `pull_request`, making
   `HEAD^..HEAD` the whole PR diff. CircleCI checks out the branch tip, so the
   compared range is `GATE_BASE` = merge-base with `origin/main` for branch
   builds and `HEAD^` for `main` builds. Same changed-line set, never smaller,
   and the job fails closed (exit 2) when a base cannot be computed.
3. `check_queue_status.py` needs the `gh` CLI and a token. Wave 1 has neither,
   and the script fails OPEN on infrastructure by design, so the step is
   branched: it prints that the gate is SKIPPED as a wave-1 gap instead of
   passing quietly. Wave 2 enables the enforcing branch.

## Status names this rail publishes - CORRECTION

An earlier revision of this record (and the matching PR comment) claimed
CircleCI publishes `<workflow>/<job display name>`. That was written before the
read and is wrong in form. CircleCI's own documentation ("Status updates
overview", read 2026-11-06) says:

- GitHub OAuth connection, regular job: commit status `ci/circleci: <job name>`.
  Only approval, no-op and release jobs carry the `<workflow_name>/<job_name>`
  form this record previously generalised from.
- GitHub App connection: statuses/checks are posted per workflow, so the
  per-job granularity branch protection needs is not what arrives.

The job `name:` overrides in the workflow are what make `<job name>` read as
`Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)` and
`Handoff ownership scope`. The authoritative statement of the five published
strings is the first pipeline read plus the connection type oversight
activates with - not a prediction in this file.

Branch protection currently requires the Actions names. Updating the
required-context list to the strings the first pipeline actually publishes is
the merge-time governance item already named on PR #653 (oversight/owner); this
port does not change protection, and no protection change should be made from
this record alone.

## Local verification (this session)

- YAML parse with the repository's duplicate-key loader
  (`scripts/check_workflow_yaml.DuplicateKeyLoader`): OK; five jobs, five
  workflow entries with the exact display names.
- Every python gate the config calls, run on this head through
  `scripts/gate.sh` (real exit codes, logs in `tmp/gate-logs/`):
  `check_required_contexts.py` exit 0; `check_toolchain_pin.py` exit 0;
  `check_workflow_yaml.py` exit 0 (27 files); `check_perimeter_underscore_params.py`
  exit 0 (63 files, 2108 fn items); unwrap/println unit 19 tests OK and scan
  exit 0; `validate_handoff_scope.py --changed-from <merge-base>` exit 0;
  `--self-test` exit 0 (24 classification + 27 waiver cases).
- The adapted trailing-whitespace step was extracted from the config itself and
  run under `bash -eo pipefail` against a scratch repository in
  `tmp/cc-twtest/`, three cases: unchanged range exit 0, clean added line
  exit 0, added line with a trailing space exit 1 with the diff printed.

Not verified locally, by design: `circleci` CLI is not installed on this host,
so no `circleci config validate` run was made. The authoritative validation is
the first pipeline read, which is the next step.

## Next step

STEERING REQUEST on PR #653: oversight activates the CircleCI project
(`gh/Treystu/SCMessenger`, id `033e97e6`) and posts the first pipeline read.
The branch-protection required-context switch stays a merge-time governance
item and is not performed by the lane.

## Addendum (same session): the lane's pre-existing hygiene red, cured and CI-verified

The first push of the port exposed that `Repository Hygiene Checks` had been
red since `91917dab0` / `72719a008` (check-run `127320201813` on `a85ebea8`,
`Process completed with exit code 1`, step 4 `Check for trailing whitespace`).
That red was not caused by the port but it would have made wave 1 impossible to
call green, so it was cured in a second commit (`31f698e27`):

- `docs/audit/C2_C5_WAVE_VERDICTS_2026-11-05.md` - one trailing space in a
  wrapped `runs-on` quotation removed (content unchanged).
- `docs/audit/LANE_FLUSH_2026-11-05.md` - one blank line at EOF removed.
- `*.patch` excluded from the changed-file list in BOTH `hygiene.yml` and the
  CircleCI job. A unified diff renders the context line for a blank target line
  as a single space, so the check was reporting diff FORMAT as trailing
  whitespace in `docs/audit/red_head_remediation/*.patch`; those bytes must
  survive intact because `git apply` depends on them.

Verified by CI on `31f698e27`: `Repository Hygiene Checks` -> `success`,
`Lint` -> `success`, `Rust Linting` -> `success`, `Handoff ownership scope` ->
`success`. `Test (ubuntu-latest)` on that head (`128166639873`) ended
`cancelled`, not failed: the next push started a newer run in the same
concurrency group. On the current head `47266ed093` the read is
`Repository Hygiene Checks` success, `Rust Linting` success, `Handoff ownership
scope` success, with `Lint` and `Test (ubuntu-latest)` queued on the two-slot
pool. Cancellations are recorded as they read - they are neither passes nor
blocks (clause 8).

## Addendum (2026-11-07): pipeline #1's only red - the PyYAML environment gap, cured

Pipeline #1 on `41fed8ac7` produced the rail's first verdict and it was a real
finding, not a port defect: four of the five contexts read `success` and
`Repository Hygiene Checks` read `failure` at the step `Verify workflow YAML has
no duplicate mapping keys`, with

    [ERROR] PyYAML is not installed; cannot check workflow YAML.
            Install it rather than skipping the check.

The cause is an environment gap the Actions original never had to state:
`check_workflow_yaml.py` imports `yaml` and exits **2** when the import fails,
and GitHub's `ubuntu-latest` image happens to preinstall PyYAML while
`cimg/python:3.11` does not. The gate therefore did not run at all on this rail
- and a gate that cannot run is not a gate with nothing to check.

Cure: a named command `install_python_gate_deps` in `.circleci/config.yml`,
invoked in the hygiene job directly after `checkout`, which

- installs `pyyaml==6.0.2` pinned (a moved dependency cannot silently change the
  parse underneath a green repository), and
- imports it back and echoes the version, so the log shows the gate HAS a
  parser before the gate runs.

Skipping the step was explicitly rejected: it would leave the duplicate-key
check unexecuted while the job read green, which is exactly the shape
`check_workflow_yaml.py --self-test` exists to prevent. The remaining four
hygiene gates (`check_required_contexts.py`) did pass on pipeline #1 through
their documented `yq` fallback, which is why the failure surfaced at this step
and not earlier; with PyYAML importable that script now selects PyYAML and names
it in its output.

Local verification before push: `.circleci/config.yml` parses under the
repository's duplicate-key loader (the same loader the gate uses), all
top-level keys intact, hygiene step order `checkout` ->
`install_python_gate_deps` -> `fetch_full_history` -> `compute_gate_base` ->
the 13 checks. The fresh exact-head pipeline is the authoritative check and is
read, not predicted, on the pushed head.
