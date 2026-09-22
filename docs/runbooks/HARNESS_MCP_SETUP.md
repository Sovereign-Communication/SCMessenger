# Harness MCP in Freebuff Desktop — Setup + SCM Dogfood Runbook

Status: Active
Created: 2026-09-22
Owner: Buffy (Freebuff lane)
Source session: harness WIP handoff (PR #360 branch, pack `scm-ops-issues-v1`)

## What is installed and verified (this host, 2026-09-22)

- **Sovereign-Harness** at main `f07c814` (v0.4.0, includes the newest JEV
  work: `jev_completion.py` phase gate from merged PR #39, `jev_policy.py`
  issue buckets, `mcp.py` server) — editable pip install re-anchored to the
  stable main checkout:
  `C:/Users/SCM/Documents/GitHub/Harness` (was previously pointed at a
  Freebuff worktree, which would break on worktree rotation).
- **MCP server smoke-tested over stdio**: initialize handshake OK, **12
  tools** advertised, including `issue_sort`, `log_judgment`,
  `panel_verify`, `apply_edit`, `plan_and_execute`, `ledger_status`,
  `trust_status`.
- **Bucketed issue sort verified end-to-end against SCM's own pack**:
  `HANDOFF/harness/packs/scm-ops-issues-v1.json` — 7/7 realistic log lines
  sorted into `capacity`, `supply-chain`, `storage`, `android-build`
  buckets with path_id + suggested action, and honest no-match fallback
  (`attention` bucket).

## The bucket feature (replaces pass/fail for completion/quality)

Harness JEV no longer just passes/fails a phase: unmatched findings are
**sorted into operator-declared buckets** (operator-declared JSON, code owns
matching). Buckets carry `suggested_action` and `path_id` so each item
becomes a dispatchable task rather than a red X. SCM's pack lives in-repo at
`HANDOFF/harness/packs/scm-ops-issues-v1.json` with these buckets:

| bucket | matches | suggested action |
|---|---|---|
| `capacity` | connection_limits, per-peer cap, max peers | file D-series ticket, assign transport |
| `supply-chain` | libp2p, crate panic, vendor | vendor-source read, upstream issue, D9-style fix |
| `storage` | sled, lock, store, message store unavailable | RCA first (MESSAGE-STORE-LOCK pattern), then fix |
| `android-build` | aapt2, gradle, XML parse, keystore | fix on lane worktree, push, CI re-run |
| `attention` | fallback for anything unmatched | human triage |

To extend: edit the JSON, run the pack validator
(`python -c "import json,sys; sys.path.insert(0,'C:/Users/SCM/Documents/GitHub/Harness'); from harness.src.jev_policy import validate_pack; validate_pack(json.load(open('HANDOFF/harness/packs/scm-ops-issues-v1.json'))); print('pack OK')"`),
commit. The code owns matching; the pack only declares buckets.

## The one manual step (Freebuff desktop MCP config)

The Freebuff desktop app reads its MCP server config from a config surface
that is not file-discoverable from the agent side (no `mcp*.json` found under
`$USERPROFILE` for this app). Configure it in the Freebuff UI:

- **name**: `harness`
- **command**: `C:\Users\SCM\AppData\Roaming\Python\Python314\Scripts\harness-mcp.exe`
- **args**: `[]`
- **env**: `HARNESS_REPO=C:/Users/SCM/Documents/GitHub/Harness`, `HARNESS_PACK=SCMessenger/HANDOFF/harness/packs/scm-ops-issues-v1.json`

After saving, a new Freebuff session should show `issue_sort` and the other
11 tools as callable MCP tools. If they do not appear, re-check the exe path
(`python -m pip show sovereign-harness` → Location).

## Dogfood loop for SCM (stay in sync with newest Harness dev)

1. **Sync check** (weekly or before any JEV run):
   `git -C "$USERPROFILE/Documents/GitHub/Harness" log --oneline -3` vs
   `python -m pip show sovereign-harness` (editable install tracks the
   checkout automatically, so a `git pull` in the Harness repo IS the
   upgrade; re-run the smoke below after pulling).
2. **Smoke after any pull**:
   `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | harness-mcp.exe`
   (expects the 12-tool list) — or the fuller handshake driver in this
   session's transcript.
3. **Run JEV on SCM work**: use `issue_sort` with recent SCM log lines
   (CI failure logs, node logs, OC audit findings) -> items land in buckets
   -> convert each bucket item into a HANDOFF ticket referencing the pack's
   `suggested_action`.
4. **Pack changes** follow the table above; keep pack edits in the same PR
   as any doctrine change they reflect (the pack encodes repo doctrine).

## Known limits

- Freebuff-side MCP config is a manual paste step (see above); everything
  else is file-managed in-repo.
- The installed CLI is not on PATH by default; scripts should call
  `python -m harness ...` or the full exe path.
- The bucket pack encodes SCM doctrine as of 2026-09-22 — review it when
  doctrine changes (it is versioned `scm-ops-issues-v1` for this reason).
