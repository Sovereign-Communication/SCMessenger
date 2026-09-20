# CO-A-005 / doctrine copy -- operator planning record (2026-09-20)

Status: PLANNED (operator interview 2026-09-20)
Source findings: `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter1.md` (CO-A-002..005), `iter6` triage
Authority: AGENTS.md "nodes, not relays" doctrine
Owner lane: orchestrator + Freebuff for mechanical doc renames; Android strings need a scoped Android PR

## Operator decisions (interview answers)

1. **CO-A-005 rename method:** Rename only, update links. **No stub** at the old path.
2. **First merge PR scope:** Canonical audit package only (reports + freebuff index + queue task).
3. **Doctrine wording follow-on:** YES -- after audit docs land, fix operator-facing copy:
   - CO-A-004 README "internet relay" / "relays carry"
   - CO-A-002 Android `DiagnosticsReporter.kt` "relay servers may be down" strings
4. **Docs PR merge authority:** Orchestrator merges after CI green (docs-only; no Rule-8 surface).

## CO-A-005 rename plan

### Target

| From | To |
|---|---|
| `docs/RELAY_OPERATOR_GUIDE.md` | `docs/NODE_OPERATOR_GUIDE.md` |

Body already correct: title "Node Operator Guide"; states no dedicated relays; every node is a full relay. Filename is the outlier.

### Inbound references to update (verified this session via grep)

| File | Lines (observed) |
|---|---|
| `DOCUMENTATION.md` | 96 |
| `docs/BOOTSTRAP.md` | 11, 242 |
| `docs/BOOTSTRAP_GOVERNANCE.md` | 116, 137 |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` | 1327 |

Also re-grep after rename:

```
rg -n "RELAY_OPERATOR_GUIDE" --glob '!tmp/**' --glob '!target/**'
```

Zero hits required (or only historical archive paths that must keep the old name -- if any appear under `docs/historical/`, leave them and note why).

### Acceptance

1. `docs/NODE_OPERATOR_GUIDE.md` exists; first heading still "Node Operator Guide".
2. `docs/RELAY_OPERATOR_GUIDE.md` does **not** exist on the landing branch (operator: no stub).
3. `rg RELAY_OPERATOR_GUIDE` has no live non-historical references.
4. `DOCUMENTATION.md` link resolves to the new path.
5. `bash scripts/docs_sync_check.sh` exits 0 (capture `$?` without a pipe).
6. No product code changes in this rename PR.

### Out of scope for the rename PR

- Mass renames of code identifiers (`cmd_relay`, `RelayCustodyStore`, etc.) -- exception registry.
- Android string changes (CO-A-002) -- separate Android PR after docs.
- Wave 1 freebuff paste tickets -- separate from this inventory follow-on.

## Follow-on PR sequence (post docs-audit landing)

| Step | Branch (proposed) | Contents | Gate |
|---|---|---|---|
| 1 | `docs/canonical-outlier-audit-2026-09-19` | Canonical audit package only | CI green; orchestrator merge |
| 2 | `docs/node-operator-guide-rename` | CO-A-005 rename + link updates | docs_sync_check; orchestrator merge |
| 3 | `docs/doctrine-readme-copy` | CO-A-004 README wording | docs only; orchestrator merge |
| 4 | `android/diagnostics-node-copy` | CO-A-002 DiagnosticsReporter strings | Android CI; no Rule-8 if strings only |
| 5 | `chore/audit-lineage-rebase` | Remaining #335 non-docs lineage rebased onto main (AGENTS disk rules, scripts, MULTIDIM, CTO/runbook/todo) | Explicit path review; no force-push of #335 |

## WIP safety (shared checkout)

- Another MiMo session has dirty Android/`iron_core.rs`/`CTO_STATE.md` work in the main checkout.
- All audit-docs landing uses worktree `tmp/wt-canonical-audit-docs` on a branch from `origin/main`.
- Do not `git checkout --`, `reset --hard`, or commit others' dirty paths.
- Do not force-push `glm/canonical-outlier-audit` (open PR #335 head = shared branch).

## Open NEEDS-HUMAN carried from the audit (not decided here)

- CO-D-002: reconcile three 0.4.0 gate verdicts (CTO_STATE sealed vs SHADOW HALT vs MULTIDIM NOT COMPLETE).
- CO-G-002: disposition stale P1 outbox release-blocker ticket vs verified-fixed CLI path.
