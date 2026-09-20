# CO-B-002 / CO-G-001 / CO-G-002 / CO-G-003 — docs dispositions (2026-09-20)

Status: OPEN — orchestrator package (this branch)
Source: CANONICAL_OUTLIER_AUDIT_2026-09-19 (main via #336)

## CO-B-002 — orphan ID doc

`docs/ID_UNIFICATION_IMPLEMENTATION.md` still says **Status: Active Implementation**
and that contact storage MUST use `libp2p_peer_id`. That contradicts
`core/src/api.udl` canonical = public_key_hex.

**Disposition (this package):** header rewritten to **SUPERSEDED / HISTORICAL**
with dated correction pointing at api.udl + UNIFICATION docs. File not deleted.

## CO-G-001 — unindexed freebuff queue files

FREEBUFF.md: unindexed queue files are invisible. Audit listed 18 files missing
from README.

**Disposition:** freebuff README DISPATCHABLE section is the paste authority;
historical review-dispatch and CTO handoff files listed in a **DO NOT PASTE /
ARCHIVE-INDEX** subsection (or moved later by orchestrator `git mv` to done/).
This package updates README to name the audit finding and keep paste set clean.

## CO-G-002 — stale P1 outbox ticket

`HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md`
still OPEN as 0.4.0 Release Blocker while CLI-03/CORE-02 are fixed on main
(#297 lineage + dual-drain).

**Disposition:** ticket Status updated: **CLI/core drain FIXED on main**;
**residual OPEN = CO-B-001 wasm/core single-form flush** (own ticket
`V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md`). Not a blind close.

## CO-G-003 — MULTIDIM still-open rows

Re-verified on `origin/main` `d7f169c1` / code tip evidence from `34b56d54` lineage:

| Row | Verdict 2026-09-20 |
|---|---|
| TRN-04 | **Closed in code** (#305 retention + admission). Audit grep used pre-#305 HEAD |
| TRN-07 | **Mechanism present** (per-peer ladder #305); global default still 200 — tuning/policy residual |
| AND-06 | Open — Wave-1 A1/A2 |
| SEC-03 | Open — operator (b) migration branch / dated waiver |

Recorded so the next session does not re-chase false blockers.
