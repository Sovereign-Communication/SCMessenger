# P1 -- `--version` reports a stale git hash, invalidating every cross-lane SHA claim

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Open
Filed: 2026-08-09 (Windows lane, during PR #139 CLI coordination)
Severity: P1 -- does not crash anything, but silently corrupts the evidence that
multi-lane fleet coordination depends on.

## Symptom (reproduced, not inferred)

A binary built from a clean worktree at `b295303009c3ae8482658ce31dfe43cd6147b34e`
self-reports a completely different commit:

```
CLI Version: 0.4.0 (33c16712 2026-08-09T10:43:06.024277300+00:00)
Core Provenance: 0.4.0 (33c16712:HEAD:1786272248)
```

`33c16712` is an unrelated older commit, and the embedded build time is ~3.5
hours before the binary was actually produced. Ground truth for the same binary:

- worktree HEAD `b2953030`, clean (`git status --porcelain` empty)
- `target/debug/scmessenger-cli.exe` mtime 2026-08-09 14:09:34Z, 27,353,600 bytes
- `cargo build` finished green at 14:09Z

Affects both the CLI version string and the Core Provenance line.

## Root cause

`cli/build.rs` declares only:

```rust
println!("cargo:rerun-if-env-changed=SCM_GIT_HASH");
println!("cargo:rerun-if-env-changed=SCM_BUILD_TIME");
```

There is no `cargo:rerun-if-changed=` for the source tree or for `.git/HEAD`.
Cargo therefore treats the build script's output as still valid across commits:
neither named env var changed, so the script is not re-run and the previously
embedded `SCM_GIT_HASH` / `SCM_BUILD_TIME` are carried forward into a genuinely
fresh binary. The script's `git rev-parse --short HEAD` fallback is correct --
it simply never executes again.

Every incremental build after the first inherits a stale stamp. Only a build in
a fresh target dir, or one where those env vars change, gets it right.

## Why this is worth fixing now rather than later

Multi-lane coordination (Windows / macOS / iOS / Android / AWS) currently
establishes "are we testing the same code?" by having each lane state its build
stamp. That protocol is unsound while this bug exists:

1. A lane can report the wrong SHA in complete good faith.
2. Two lanes can believe they are co-anchored while running different code, and
   then attribute the resulting behaviour to a network or transport defect.
3. It cuts the other way too -- a lane that really did rebuild can look stale.

This was found during the PR #139 Windows<->macOS CLI link work, where the exact
candidate SHA moved four times in under an hour and both lanes were relying on
self-reported stamps to stay in step.

## Workaround available immediately (no code change, use this today)

Because `rerun-if-env-changed` IS declared for both variables, setting them
explicitly forces the script to re-run and produces a correct stamp:

```bash
SCM_GIT_HASH=$(git rev-parse --short HEAD) \
SCM_BUILD_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ) \
CARGO_INCREMENTAL=0 cargo build -p scmessenger-cli -j12
```

Until the fix lands, **no lane should treat a self-reported version string as
provenance.** Report worktree HEAD + `git status --porcelain` (proving clean) +
binary mtime and size instead, or build with the env vars above.

## Acceptance criteria

1. `cli/build.rs` re-runs when HEAD moves. The conventional fix is to emit
   `cargo:rerun-if-changed=../.git/HEAD` plus the ref file HEAD points at;
   whoever implements this must handle the detached-HEAD and worktree cases
   (a linked worktree's `.git` is a FILE, not a directory, so a naive
   `../.git/HEAD` path does not resolve -- this repo uses linked worktrees
   heavily and the fix must be tested from one).
2. Same treatment wherever the Core Provenance string is produced -- it shows
   the identical stale value, so it has the same defect and must not be assumed
   fixed by the CLI change alone.
3. Regression evidence: build at commit A, commit B, build again in the SAME
   target dir without cleaning, and show the reported hash changed from A to B.
   A test that only ever builds once cannot catch this.
4. Confirm the fix does not force a full rebuild of the workspace on every
   `git commit` (over-broad `rerun-if-changed` on the source tree would trade
   this bug for a much slower build; scope it to the git ref files).

## Notes

- Not a regression from the PR #139 commits; `build.rs` has looked like this
  for some time. It became visible only because this exercise demanded exact
  SHA agreement between two machines.
- Low blast radius to fix, but it lives in build tooling that every lane
  depends on, so the change wants the rebuild-cost check in criterion 4.
