# V040 freebuff dispatch — tag-path wave (2026-09-20)

Status: OPEN — DISPATCHABLE once this file is on `origin/main`
Priority: P0 — queue hygiene before any implementation paste
Lane: Freebuff (docs/status only)
Scope: `HANDOFF/freebuff/**` status lines and README index only.
Do NOT touch `core/`, `cli/`, `android/`, workflows, or unrelated HANDOFF trees.

## Why this exists

`scripts/check_queue_status.py` exits 1 on clean main because ticket Status
lines advertise open PRs that already merged. The operator pastes from this
queue; a stale "awaiting review" costs a full Freebuff cycle on finished work
(exactly what happened when T7 was selected after #312 merged).

Evidence (orchestrator, 2026-09-20, worktree `tmp/wt-tag-path-20260920`):

```
python scripts/check_queue_status.py
...
[FAIL] V040_T7_ANDROID_PARITY_STAGING.md: claims PR #312 is open/awaiting review, but it is MERGED (2026-09-19T23:05:21Z)
[ERROR] 1 stale status statement(s)
```

## Design authority

Unified path: `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md`.
Do not invent new 0.4.0 work in this ticket.

## Work

1. For each ticket below, set Status to MERGED/RESOLVED with the merge evidence
   already verified by the orchestrator. Keep pre-merge history as a second
   line if useful; the Status line must stop claiming the PR is open.
2. Update `HANDOFF/freebuff/README.md`:
   - Last updated date
   - Mark DISPATCHABLE vs DONE vs POST-TAG
   - Remove or strike the obsolete "PASTE FIRST review dispatches" block
     (those reviews are filed; PRs largely merged)
   - Link `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md` as the order authority
3. Move files that are fully closed to `HANDOFF/freebuff/done/` when the
   orchestrator files the `git mv` in a later docs PR — **this lane may edit
   Status in place; file moves stay with the orchestrator** unless the PR
   only relocates freebuff-owned queue/inbox files.
4. Re-run:

```
python scripts/check_queue_status.py
```

Must print `[OK]` / exit 0 (or only remaining failures with a written reason
why the Status line is intentionally non-merge-shaped).

## Status corrections required (verified merges)

| File | New Status essence |
|---|---|
| `V040_T7_ANDROID_PARITY_STAGING.md` | MERGED — PR #312 merged 2026-09-19T23:05:21Z |
| `V040_T8_RESTORE_DIAGNOSTICS_FORMATTER_TEST.md` | MERGED — PR #271 merged 2026-09-03 |
| `V040_T6_TIER_A_CONFORMANCE_HARNESS.md` | MERGED — PR #311 merged 2026-09-19; harness remains for scoring |
| `V040_T11_CANONICAL_DOC_RECONCILE.md` | MERGED — PR #314 merged 2026-09-19 |
| `V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS.md` | MERGED — PRs #319/#328 merged 2026-09-19 |
| `V040_T14_EPHEMERAL_PORT_ADVERTISED_AS_EXTERNAL.md` | MERGED — PR #270 merged 2026-09-03 |
| `V040_T14_PREEXISTING_DHT_BUGS_TICKET_2026-09-01.md` | MERGED — PR #269 merged 2026-09-03 |
| `V040_T1_NODE_BOOT_SEED_DIAL.md` | CODE ON MAIN — seed_dial.rs + swarm boot dial; do not re-dispatch implementation |
| `V040_T2_UNIFY_PEER_LEDGER_STORES.md` | CODE ON MAIN — core ledger + peers.json migration/archival; residual is T13/live hygiene |
| `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` | IMPLEMENTED, D6 UNSCORED — premise contradicted; scoring is operator |

## Scope correction

- Do not "fix" code in this task.
- Do not close PRs.
- Do not claim D4/D6/D7 passed.
- Do not paste this task together with implementation tickets.

## Acceptance

1. Status lines above updated with command-backed merge SHAs/dates.
2. README dispatch table matches `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md` §3.
3. `python scripts/check_queue_status.py` exit 0.
4. `python scripts/rules_check.py` exit 0.
5. PR opened; **no self-merge**.

## Review gate

None (HANDOFF docs only).

## Rules

- No emojis.
- Evidence contract: command, run URL, or UNVERIFIED.
- Shared checkout: touch only freebuff queue/README files listed above.
