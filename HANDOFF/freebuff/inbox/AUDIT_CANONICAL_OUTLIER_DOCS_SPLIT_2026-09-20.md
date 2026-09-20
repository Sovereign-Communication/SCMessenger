Task: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Type: DONE
Milestone: DOCS-FIRST SPLIT (operator 2026-09-20)

## Decision record

Operator interview answers (2026-09-20):
- CO-A-005: rename `docs/RELAY_OPERATOR_GUIDE.md` -> `docs/NODE_OPERATOR_GUIDE.md`, update links, **no stub**.
- First PR: **canonical audit package only**.
- Follow-on: README doctrine copy + Android diagnostics strings after audit docs land.
- Docs merge: **orchestrator merges after CI green**.

## Docs landing

- Worktree: `tmp/wt-canonical-audit-docs` (isolated; shared checkout WIP untouched)
- Branch: `docs/canonical-outlier-audit-2026-09-19`
- Base: `origin/main` @ `882e95db`
- Package: CANONICAL_OUTLIER iter0-6 + INDEX + freebuff queue task + inbox DONE/FINAL notes + README reference index on **main's rewritten** Freebuff README
- Planning ticket: `HANDOFF/todo/CO_A005_DOCTRINE_COPY_PLAN_2026-09-20.md`

## Superseded mixed PR

- https://github.com/Sovereign-Communication/SCMessenger/pull/335 (`glm/canonical-outlier-audit`)
- Contains audit docs PLUS non-docs lineage (AGENTS.md, disk scripts, MULTIDIMENSIONAL_AUDIT, CTO/runbook/todo)
- **Do not force-push** that branch (PR head is shared). Remaining lineage gets a new rebased branch after docs merge.

## WIP conflict avoidance

Other session dirty paths in main checkout (left untouched): Android Kotlin UI/viewmodels, `core/src/iron_core.rs`, `HANDOFF/CTO_STATE.md`, untracked inbox/review/test files.

PR number for this docs landing: filled after `gh pr create` on the docs branch.
