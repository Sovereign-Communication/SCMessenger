# Merge queue state (2026-10-04, end of session)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Owned by `Sovereign-Communication/SCMessenger` (this repository).

Branch `integrate/train-20261004` head: `363795e72`.
Both handoffs are pushed; the tree is clean and nothing is unpushed.

## Verified green on 6eef46750 (all 9 runs, read from job logs)

CI (10 jobs), Cross, Lint, Mobile (4 jobs), PR #451, Repository Hygiene,
Security Regression Tests, iOS Build & Test, Auto Label. Zero in flight,
zero red. Pushing 363795e72 cancelled nothing.

Required contexts on main are exactly `Repository Hygiene Checks`, `Lint`,
`Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope` (strict).
`Mobile` is not required and main runs no Mobile job.

## PR #451: blocked on governance, not on CI

Head is fully green. It cannot merge because 18 files under
`core/src/{crypto,transport,routing,privacy}/` have no live Rule-8 verdict and
the integration agent authored three of the commits that touch them. Full
analysis, the void-verdict proof, the independent audit, and the four options:
`HANDOFF/review/RULE8_TRAIN_451_REVIEW_GAP_2026-10-04.md`.

Open items from that audit:
- V2 unproven: no message-ID dedup inside `decrypt_with_ratchet_fallback`
  (`core/src/crypto/encrypt.rs:663-860`).
- N3 unverified, potentially High: sender-auth static-static DH term into the
  0x03 ratchet derivation, exempted from the perimeter check.
- N1 real: `max_gateways` in `core/src/routing/neighborhood.rs` became a soft
  cap, so a burst of fresh untrusted gateway IDs grows the table unbounded.

## Wave 2: ready now

Pre-verified conflict-free and documentation-only, 7 of 7:
#316 #357 #376 #386 #388 #415 #425.

Also conflict-free and Rule-8 clear, but code, so CI must verify:
#156 #170 #212 #214 #349 #352 #380 #381 #382 #398 #400 #401 #405 #409 #410.

Decide before merging: #409 contains cherry-picks of #401 #400 #381 #380.
Merge the umbrella or the parts, not both.

## Do not merge these 10 (known-poison)

#106 #107 #108 #210 #213 (AndroidX / Kotlin-metadata bumps)
#439 #441 #443 (Rust bumps that broke every Rust job)
#440 #442 are lockfile-only and unproven, not proven-bad.

Full reasoning and the corrected scan (a naive scan produced 6 false
positives): `HANDOFF/review/WAVE2_MERGE_READINESS_2026-10-04.md`.

## Known-good tooling notes for the next session

- `gh` needs `-R Sovereign-Communication/SCMessenger` on every call.
- `gh run list --commit` needs the FULL 40-char SHA. A short SHA returns `[]`,
  and `all([])` is `True`, which fakes an all-clear. Always guard with
  `bool(d) and all(...)`.
- Python `subprocess(shell=True)` uses `cmd.exe` on this host and splits jq
  filters on `[`. Pass argument lists with `shell=False`.
- `gh pr view --json files` caps at 100 files; #451 is at the cap, so its 18
  gated files are a floor.
- `scripts/pr_scope.sh` is a bash script; run it with `bash`, not `python`.
- Edit files under `C:\Users\SCM\Documents\GitHub\wt-int\`; the shared
  checkout must stay clean on `preserve/shared-checkout-20261003`.

## Disk

`python scripts/disk_budget.py` reports below the tight threshold (exit 1).
A 6.9 GB `target/debug` sits in the shared checkout, but
`reclaim_safe.py` rates that tree HOLD because its HEAD `e4050f84c` is a
"NEVER MERGE" snapshot commit that differs from main by 1798 files. It needs
an explicit operator decision; it was NOT deleted. The two worktrees the tool
rates SAFE (`wt-andss`, `wt-toolchain-pin`) have no `target/` at all.

The live node `scmessenger-cli.exe` (PID 26880) runs from
`C:\Users\SCM\.local\bin\`, outside every `target/`, so reclaim cannot
disturb it.

## Next actions, in order

1. Operator picks a Rule-8 route for #451 (fund a panel, human review, split
   the 112 non-gated files, or record an override).
2. Merge the 7 Tier A docs PRs to drain the board.
3. Wave the Tier B code PRs through CI.
4. Check V2 and N3 before any crypto merge.
5. 3-node log triangulation for 0.4.0, deferred until the tree settles.