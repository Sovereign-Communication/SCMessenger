<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# V040 Safe Work Ahead -- coordination plan (single implementer, 2026-09-25)

Owner of this document: SCMessenger. This is the coordination seat's plan for
the SCMessenger 0.4.0 burndown. The SCMessenger main lane is the sole
implementer; this seat wrote this plan and the files indexed below, read-only
against `origin/main` and the in-flight PRs. Nothing here was written to the
shared checkout.

## 1. Ownership model

| Seat | May do | May not do |
|---|---|---|
| Main implementation lane (single writer) | Write code, open/close PRs, land the merge train, commit and push SCMessenger docs | Self-sign Rule-8; improvise on architecture/security/API-contract decisions |
| Coordination seat (this document) | Read-only audits, task files, gate specs, Rule-8 input packs, disposition ledgers | Edit the shared checkout, edit any PR head, merge, tag, deploy, or sign a review |

Reserved (never written by this seat, and single-writer during the train):
`core/src/transport/swarm.rs`, `core/tests/integration_outbox_flush_reconnect.rs`,
`android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`,
`android/app/src/test/java/com/scmessenger/android/service/StopStartFloodTest.kt`,
`Cargo.toml`, `Cargo.lock`, `docker/Dockerfile`, `vendor/libp2p-swarm-0.48.0/**`,
`.github/workflows/ci.yml`, `AGENTS.md`, `scripts/rules_check.py`,
`scripts/validate_handoff_scope.py`, `tests/test_handoff_scope.py`,
`HANDOFF/review/SCOPE_INVENTORY_2026-09-24.md`,
`HANDOFF/review/SCOPE_OWNERSHIP_RCA_2026-09-24.md`, and the shared checkout
branch `glm/canonical-outlier-audit`. The main lane's own isolated worktrees
(such as the one holding PR #359's head `3f41005d`) are not present in this
seat's view and are not touched.

The coordination seat's own work lives in a registered git worktree at
`tmp/swa-20260925` (branch `safe/work-ahead-20260925`, off `origin/main`
`d1c4a173`). It is gitignored under `tmp/` and is not part of the shared
checkout.

## 2. Verified state (2026-09-25; re-verify before acting)

Commands used this session: `git status --short --branch`, `git log --oneline -1
origin/main`, `gh pr list` (45 open, paginated), `gh pr view <n> --json ...`,
`gh pr checks <n>` for all 45, `git fetch origin pull/372/head:pr372` plus
`git diff origin/main...pr372` in the isolated worktree,
`python scripts/disk_budget.py`, `python scripts/pr_disposition_ledger.py`,
`python scripts/test_pr_disposition_ledger.py`, `python
scripts/jev_canonical_check.py --wp WP1 --state-file scripts/wp_state_template.json
--no-openrouter` (read-only tool import), `python tests/test_handoff_scope.py`
(9 tests OK), `python scripts/validate_handoff_scope.py --document ...` (this
worktree), `bash scripts/docs_sync_check.sh` (PASS), `python
scripts/check_queue_status.py` (PASS), `python scripts/check_wiring.py` (OK),
and `git grep`/`git show` against `origin/main`.

- `origin/main` = `d1c4a173` (PR #371, Android diagnostics). 45 open PRs, 250
  merged. Branch protection: strict, required checks `Repository Hygiene`,
  `Lint`, `Rust Linting`, `Test (ubuntu)`, 0 approvals.
- Disk: `disk_budget.py` = BLOCKED (4.27 GB free at the start of the
  coordination pass, 2.36 GB / 99.0% used by its last run; ~27 GB reclaimable
  but `reclaim_safe` refuses the dirty shared tree). CI is the only builder;
  no local cargo/gradle; no device actions.
- Per-PR check state, gated-file lists, and dispositions live in the
  generated ledger `tmp/pr_disposition_ledger.md` (2026-09-25T09:52:06Z) and
  the human summary `HANDOFF/review/WORKAHEAD_DEPENDABOT_DISPOSITION_2026-09-25.md`.
  The bullets below name only the facts the landing order turns on:
  - PR #372 `fix/361-review-blockers@a76d7d66` replaces #361: MERGEABLE/CLEAN,
    33 pass / 0 skip / 0 fail, 71 changed files vs main (git), 5 gated
    `core/src/transport/*` files (+873/-11 lines), 35 vendored `libp2p-swarm`
    0.48.0 files (+13,368), plus docs, one Android lifecycle fix, one
    routing-feed fix, and a new `cli/src/bin/conn-fanout.rs`. Rule-8 OPEN:
    the author also authored the D1/D9 work. The PR body says so itself.
    Input pack for the independent reviewer:
    `HANDOFF/review/RULE8_INPUT_PACK_PR372_2026-09-25.md`.
  - #361 and #364: 32/0/1 each (`Android JVM Unit Tests` fails); #364 carries
    4 gated files inherited from #361 and must be split, not merged whole.
  - #369: 26/0/1 (`Repository Hygiene Checks` fails). Close.
  - #103: 21/1/2, CONFLICTING (`FFI Surface Contract`, `iOS Build & Simulator
    Test`; CodeQL skipped). Reopen after the train. #141: 11/0/16, CONFLICTING
    (stale branch). The five Android dependency bumps are red (10-13 failures
    each) on unrebased August branches, not green; they belong to the 0.5.0
    Android toolchain lane.
  - #227 is OPEN (draft, CONFLICTING, 33/0/0), not merged. #228 is the
    deepest-red PR in the set (12/2/20 across 14 gated files).
- `deny.toml` still ignores `RUSTSEC-2026-0285` (line 32); `Cargo.lock`
  already resolves rustls 0.23.45 (patched). `cargo deny check` runs in the
  required Lint job and in `security.yml`.
- The handoff-scope gate WIP rejects the literal foreign-product alias in any
  file under `HANDOFF/`; a full-tree run of the gate's own validator against
  `origin/main` reports 1,579 of 1,591 pre-existing HANDOFF files failing.
  Operator ruled: keep the gate, no grandfathering, re-run the train with
  scope blocks. The alias policy is the gate owner's call with the operator.
  Two consequences for this plan's files: the alias word is absent from every
  body entirely (the gate flags it inside code identifiers, file names, and
  paths too), and machine-readable artifacts that must stay valid JSON live
  under `scripts/`, not `HANDOFF/`, because the gate treats every file under
  `HANDOFF/` as a handoff and a JSON document cannot carry the gate's
  comment-style block.
- JEV tooling is on main: `scripts/jev_canonical_check.py`,
  `jev_repo_insights.py`, `jev_packs.py`, the local-import helper, the refresh
  script, and the wrapper. The WP1/WP2 state files live only in open PR #354.
  The state template `scripts/wp_state_template.json` was exercised through
  the real `jev_canonical_check.py` entry point against the checked-out tool
  (read-only import, `--no-openrouter`): the template loads, a keyed answer
  is returned (model `jev-1.13.0`, cost $0.028), and the gate correctly fails
  a placeholder state (`instruction_matches` 0.37 < 0.70). That run is the
  evidence that the template and the 0.70 threshold are live, not theoretical.
- The external JEV evaluation product checked out next to this repository
  (a sibling directory whose name contains the gated word) is at 0.4.1 plus an
  unreleased JEV-BAR layer: per-axis sentiment buckets,
  `packs/phase_completion.pack.json` (6 axes, 5 levels, default threshold 85,
  hard-gate weights 25/10/20/15/15/15, combined `0.7*mechanical +
  0.3*semantic`), `can_mark_complete`, `improvements`, `--all`, `--min-score`.
  SCMessenger docs still name 0.3.3 (PR #360 never landed). The wrapper
  accepts only `verify spend ledger trust lint-claims smoke`; the tool's CLI
  offers more. The local-import helper imports two private `jev`-module
  symbols (`_validate_questions`, `_parse_answer`) -- a 0.x semver break
  risk. The tool has no `--version` flag; the package `__version__` falls
  back to `pyproject.toml`.

## 3. Workstreams and landing order

| # | Workstream | First artifact | Gate |
|---|---|---|---|
| 0 | JEV tool upgrade to the newest version (independent PR, first) | `HANDOFF/freebuff/queue/V040_WS0_JEV_TOOL_UPGRADE_2026-09-25.md` | hermetic compat test; pin file; CI |
| 0b | Handoff-scope gate landing | gate owner's WIP | operator alias ruling; in-flight docs compliant |
| 1 | Per-item JEV completeness gates | `HANDOFF/freebuff/queue/V040_WS1_JEV_PHASE_BAR_GATE_2026-09-25.md` | `jev_phase_check.py` exit 0; hermetic tests |
| 2 | Transport leg | #372 | independent Rule-8 verdict, then squash-merge |
| 3 | Security waiver | `deny.toml` one-line removal | Lint job cargo-deny |
| 4 | #364 split (outbox sweep / Docker vendor copy; Kotlin half closes) | re-based PRs | tests + Rule-8 for the sweep half |
| 5 | #351 rework | re-based PR | tests |
| 6 | WP legs #349 #352 #355 #356 | re-based PRs | Rule-8 each; keyed JEV exit 0 each |
| 7 | #359 docs residue (transport commit dropped) | re-based PR | scope blocks on every touched HANDOFF doc |
| 8 | #368 | merge | checks; add WS queue rows in the same edit |
| 9 | D8, D2, D3, D5, D6 | `HANDOFF/freebuff/queue/V040_WP4_D8_CUSTODY_RECEIPT_RETURN_2026-09-25.md` and the #372 tickets | Rule-8 each (core custody/transport); JEV exit 0 |
| 10 | Android test-only reproductions (Compose crash, chat order) | tickets on main | tests; no code WIP until released |
| 11 | WP5 3-node proof | evidence on the P0 umbrella ticket | operator phone session |
| 12 | Tag 0.4.0 | operator call | release checklist |

Dependabot (#211 #212 #214, then #103 and #141) lands after the train, each
rebased first. The five Android dependency bumps (#106 #107 #108 #210 #213)
go with the 0.5.0 Android toolchain lane, rebased, because their current
check states are red on stale branches. PR #360 (the old 0.3.3 version-floor
change) is superseded by WS0. Per-PR dispositions, check states, and
gated-file lists: `tmp/pr_disposition_2026-09-25.md` (first run) and
`tmp/pr_disposition_ledger.md` (latest), both generated by
`scripts/pr_disposition_ledger.py`.

## 4. JEV completeness gate (definition of DONE per item)

Every work item closes only when BOTH gates pass on the item's final head:

1. Mechanical gates: the project's typecheck/tests/clippy, `rules_check`,
   `pr_scope.sh`, the required CI checks, and (for gated trees) a recorded
   adversarial APPROVE from a reviewer who did not author it.
2. Keyed JEV canonical gate:
   `python scripts/jev_canonical_check.py --wp WPn --state-file <state.json>`,
   three keyed questions (`canon_identity`, `canon_routing_feed`,
   `instruction_matches`), `is_passing(0.70)`, exit 0. `--allow-fallback`
   prints `UNVERIFIED-JEV` and is never DONE. WP1/WP2 state files require #354
   to have landed. Template: `scripts/wp_state_template.json`.
3. JEV-BAR phase gate (WS1): `python scripts/jev_phase_check.py --phase <id>
   --evidence <evidence.json>`; all hard gates (`pr_merged`, `origin_evidence`,
   `required_tests_present`, `local_gates_green`, `ci_green`,
   `no_open_blockers`) true, combined score >= 85, no blocking sentiment axis,
   `can_mark_complete` true. Any `improvements:` entry means fix-and-iterate.

A JEV verdict is a completeness judgement, not a security verdict, and never
substitutes for Rule-8.

## 5. Per-leg report format

For every landed leg record: leg, PR, head SHA reviewed, Rule-8 verdict file,
JEV state file and exit code, JEV-BAR evidence and exit code, squash SHA, tree
proof (`git diff --name-only <pre>..<post>`), new `main` tip, CI run URL, and
what stayed untouched. Stop and report rather than improvise on any blocker.

## 6. Files produced by this coordination seat

All under the worktree `tmp/swa-20260925` (branch `safe/work-ahead-20260925`).

- `HANDOFF/plans/V040_SAFE_WORK_AHEAD_COORDINATION_PLAN_2026-09-25.md` (this file)
- `HANDOFF/freebuff/queue/V040_WS0_JEV_TOOL_UPGRADE_2026-09-25.md`
- `HANDOFF/freebuff/queue/V040_WS1_JEV_PHASE_BAR_GATE_2026-09-25.md`
- `HANDOFF/freebuff/queue/V040_WS0B_PER_ITEM_GATE_ENFORCEMENT_2026-09-25.md`
- `HANDOFF/freebuff/queue/V040_WP4_D8_CUSTODY_RECEIPT_RETURN_2026-09-25.md`
- `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md` (WP state-file contract)
- `HANDOFF/jev/JEV_TOOL_PIN.md` (pin record format; filled by WS0)
- `HANDOFF/jev/PHASE_BAR_GATES.md` (JEV-BAR axis definitions and evidence schema)
- `HANDOFF/audit/D8_CUSTODY_RECEIPT_CONVERGENCE_AUDIT_2026-09-25.md`
- `HANDOFF/audit/D2_D3_DIAGNOSIS_2026-09-25.md`
- `HANDOFF/review/WORKAHEAD_DEPENDABOT_DISPOSITION_2026-09-25.md`
- `HANDOFF/review/RULE8_INPUT_PACK_PR372_2026-09-25.md` (candidate-finding input for the independent reviewer; not a verdict)
- `scripts/pr_disposition_ledger.py` (read-only open-PR disposition ledger generator; output under `tmp/`)
- `scripts/test_pr_disposition_ledger.py` (hermetic tests for the ledger; not yet wired to CI -- the gate owner's CI edit)
- `scripts/wp_state_template.json` (JEV canonical state-file template; under `scripts/` so it stays valid JSON outside the HANDOFF gate)

`HANDOFF/freebuff/README.md` is NOT edited here because PR #368 already
modifies it. The main lane adds the four rows above to its README edit when it
lands #368.
