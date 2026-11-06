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

## Status names this rail publishes

CircleCI publishes `<workflow>/<job display name>`:

```
ci/Repository Hygiene Checks
ci/Lint
ci/Rust Linting
ci/Test (ubuntu-latest)
ci/Handoff ownership scope
```

Branch protection currently requires the Actions names. Updating the
required-context list to those five strings is the merge-time governance item
already named on PR #653 (oversight/owner); this port does not change
protection.

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
`success`, `Test (ubuntu-latest)` -> running at the read.
