Task: V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md (Mission 2 -- focused architecture pass verification)
Type: DONE (gate record on file; AWS leg still BLOCKED on the operator)

# DONE -- Mission 2: architecture pass verified, workspace no-run gate artifact-state (FAIL-reproduced -> PASS), evidence on file

## Result
The prior `cargo test --workspace --no-run` failure is CLASSIFIED:
artifact-state, NOT a source defect. Reproduced the documented corruption
family, then PASSED (exit=0) after the sanctioned scoped repair -- on the
identical tree with zero source changes. Full gate table, exact commands,
SHAs, and raw logs:
tmp/run-evidence/v040-arch-verify-e97c3f82-20260904/RUN-RECORD.md
(+ workspace-norun-shared-repro.log with 4 dated runs, fmt-check.log,
test-observation(-r2).log, test-local.log, test-optimized_engine.log).

## Gates
- G1 shared-checkout head on candidate line: FAIL (HEAD 0e0d54da on
  cto/t2-disk-ruling-2026-08-31, a deliberate stale mix; origin candidate =
  e97c3f82, correct). Shared checkout untouched.
- G2 517-file posture: STRAY -- 507 untracked (evidence/records) + 10 stale
  arch working copies; not v0.4.0 release state.
- G3 cargo fmt --check: PASS.
- G4/G5/G6 focused suites (observation/local/optimized_engine): PASS 5/34/6.
  G4 first FAILED on stale cached UniFFI bindings from a newer tree
  (artifact-state evidence); scoped build-script-dir removal -> PASS.
- G7 cargo test --workspace --no-run: run-1 FAIL exit=1 (exact documented
  signatures: metadata stubs, missing scmessenger_core, can't-find-std) ->
  run-2 PASS exit=0 (no repair) -> post-clean_target PASS exit=0. Repair =
  scripts/clean_target.sh --deps (scoped, ~22 GB; never cargo clean;
  core/target/generated-sources backed up + verified intact).
- e97c3f82 tree compile health stands independently: PR #272 CI fully green
  at e97c3f82 (all ~30 checks incl. Test matrix) + resolution-session local
  verification (lib 1411/0, check/clippy/fmt clean).

## Disk consequence (flag for other sessions)
Host was at 1.6 GB free. Freed ~48 GB non-destructively (regenerable target/
of the superseded scm-v040-candidate worktree + clean_target.sh --deps). The
shared checkout's debug deps are GONE: the next cargo build there is a full
~20-30 min dep rebuild. 33+ GB free at record time.

## AWS leg re-probed 2026-09-04 ~11:35Z -- STILL BLOCKED (operator action not yet taken)
Read-only ssh probe (no sudo, no docker):
- /opt/scm-relay-data/scm-node.pre-e97c3f82.inspect.json: ABSENT (the deploy
  block creates it as its first step).
- Newest logs scm.log.2026-09-04-10/-11: continuous old-image operation
  (rollover 11:00:09Z, no 0.4.0 (e97c3f82)/sha-e97c3f8 provenance anywhere;
  -11 live at 11:31Z, peers=1). Container still the 177bd840-era image.

## Single next CEO decision (unchanged)
Run the probe, then the deploy block from
tmp/run-evidence/aws-redeploy-e97c3f82-20260904/OPERATOR-DEPLOY-BLOCK.md
(paste back the docker ps line + first ~15 lines of `docker logs scm-node`),
then post-state capture runs (tmp/run-evidence/aws-redeploy-e97c3f82-20260904/
capture-post.sh) and the before/after verification proceeds. No merges, no
tags; no HANDOFF/.codebuff_deploy/source changes beyond this note.
