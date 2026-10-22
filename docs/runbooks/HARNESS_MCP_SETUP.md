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
- **Bucketed issue sort verified for real on genuine SCM failure data**:
  first real JEV dogfood run 2026-09-22 — 5/5 agreement on real failure
  lines (D9 panic, D1 refusals, MESSAGE-STORE-LOCK, aapt2 XML), keyed
  mode (`jev-1.13.0`, is_fallback false). Full record:
  `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md`.

## The bucket feature (replaces pass/fail for completion/quality)

Harness JEV no longer just passes/fails a phase: unmatched findings are
**sorted into operator-declared buckets** (operator-declared JSON, code owns
matching). Buckets carry `suggested_action` and `path_id` so each item
becomes a dispatchable task rather than a red X. SCM's pack lives in-repo at
`HANDOFF/harness/packs/scm-ops-issues-v1.json` with these buckets:

Current buckets (ids exactly as declared in the pack; unmatched input
returns honest `status: "unmatched"` — there is no catch-all bucket):

| bucket | label |
|---|---|
| `capacity` | connection, rate, or budget saturation |
| `backoff` | dial/retry policy starving reconnection |
| `poison_queue` | self-addressed or stuck queue entries |
| `crypto_noise` | ratchet, decrypt, or envelope anomalies |
| `ble_lane` | BLE / proximity transport health |
| `storage` | sled / persistence / degraded store |
| `identity_device` | identity or device registry anomalies |
| `orchestration` | agent-orchestration / harness process issue |
| `android_build` | Android build, resource, or signing failure |
| `supply_chain` | upstream crate or dependency defect |

Pack keyword caution (learned the hard way in dogfood run 001): the code-
owned matcher is substring-based (`kw.lower() in text.lower()`), so never
declare short generic keywords — bare `ble` once matched inside
"unreacha-ble". Write distinctive phrases.

To extend: edit the JSON, validate it through the real validator, commit.
The code owns matching; the pack only declares buckets.

```bash
cd "$USERPROFILE/Documents/GitHub/Harness" && python -c "
import sys, json; sys.path.insert(0, '.')
from harness.jev_packs import validate_operator_pack
validate_operator_pack(json.load(open('C:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/harness/packs/scm-ops-issues-v1.json')))
print('pack OK')"
```

## The one manual step (Freebuff desktop MCP config)

The Freebuff desktop app reads its MCP server config from a config surface
that is not file-discoverable from the agent side (no `mcp*.json` found under
`$USERPROFILE` for this app). Configure it in the Freebuff UI:

- **name**: `harness`
- **command**: `C:\Users\SCM\AppData\Roaming\Python\Python314\Scripts\harness-mcp.exe`
- **args**: `[]`
- **env**: `HARNESS_REPO=C:/Users/SCM/Documents/GitHub/Harness`, `HARNESS_PACK=C:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/harness/packs/scm-ops-issues-v1.json`

After saving, a new Freebuff session should show `issue_sort` and the other
11 tools as callable MCP tools. If they do not appear, re-check the exe path
(`python -m pip show sovereign-harness` → Location).

## Claude Code local MCP (orchestrator sessions), 2026-09-29

Two dedicated, exact-SHA worktrees of the Harness repo. Never the shared pip
editable install (other lanes use it) and never a moving branch:

| Purpose | Worktree | Ref | Why |
|---|---|---|---|
| Release-gate scoring (`jev-phase` CLI, `scripts/jev_post_merge.py`) and the Board (`scripts/bod_governance.py`) | `wt-harness-canonical` | admitted tag `v0.4.1` (`ad4a300`) | BUILD_AND_CI.md: production is the immutable tag |
| Interactive MCP registration in Claude Code | `wt-harness-canary` | exact SHA `6961c09` (CANARY) | the tag's MCP server rejects Claude Code's protocol version (`-32602 unsupported MCP protocol version '2025-11-25'; supported: 2025-06-18`); `6961c09` negotiates it and exposes 15 tools |

    git -C C:/Users/SCM/Documents/GitHub/Harness worktree add --detach C:/Users/SCM/Documents/GitHub/wt-harness-canonical v0.4.1
    git -C C:/Users/SCM/Documents/GitHub/Harness worktree add --detach C:/Users/SCM/Documents/GitHub/wt-harness-canary 6961c09
    claude mcp add harness --scope local \
      -e PYTHONPATH=C:/Users/SCM/Documents/GitHub/wt-harness-canary \
      -e HARNESS_REPO=C:/Users/SCM/Documents/GitHub/wt-harness-canary \
      -e HARNESS_PACK=C:/Users/SCM/Documents/GitHub/SCMessenger/HANDOFF/harness/packs/scm-ops-issues-v1.json \
      -e PYTHONIOENCODING=utf-8 -- C:/Python314/python.exe -m harness.mcp

Registration is local scope (user config only, no repo change). Verified
2026-09-29: `claude mcp list` reports `Connected` at `6961c09` and
`Failed to connect` at the tag. The tools appear in a NEW session; a running
session does not hot-load a newly registered server.

Facts that matter when you use it:

- Results from the canary MCP are canary evidence, never a release gate. Score
  cars with the admitted tag: the tag has the `jev-phase` CLI (its MCP server has
  no `jev_phase` tool), so run `HARNESS_REPO=<wt-harness-canonical> python
  scripts/jev_post_merge.py ...` or `PYTHONPATH=<wt-harness-canonical> python -m
  harness.cli jev-phase --local-only ...`. If the canary advances, its old SHA is
  invalid and a new exact SHA must be admitted (BUILD_AND_CI.md).
- Spend: the OpenRouter key has a daily limit (`spend_status` shows it) and the
  harness session ceiling is $0.05 unless `HARNESS_MAX_COST` is raised (the
  operator cap is $0.10 per gated PR, A9). A worst-case preflight refusal costs
  nothing; a panelist that returns only reasoning text costs money and casts no
  vote; a panel with fewer than 5 valid votes is a DEFERRED Board result.
- Do not redirect or print the server's or the CLI's stderr: it carries the
  key-identity line. Read results from `--out` JSON and typed progress from
  `--events FILE`.
- Never repoint the shared pip editable install to fix this locally.

## Dogfood loop for SCM (stay in sync with newest Harness dev)

1. **Sync check** (weekly or before any JEV run): SUPERSEDED 2026-09-29. Do
   NOT pull `origin/main` to upgrade. `docs/rules/BUILD_AND_CI.md` ("Harness
   admission and update gate") makes the immutable tag `v0.4.1` the
   production source and `origin/main` a bounded canary that a release gate
   must never use directly; the old pull-to-upgrade habit is how the shared
   editable install drifted off any admitted source. Re-admit by exact tag or
   SHA, then re-validate the pack and smoke the MCP server.
2. **Smoke after any pull**:
   `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | harness-mcp.exe`
   (expects the 12-tool list).
3. **Run JEV on SCM work**: `harness.exe issue-sort --issue <line> --pack
   <pack>` (or the MCP `issue_sort` tool once configured) with recent SCM
   log lines (CI failure logs, node logs, OC audit findings) -> items land
   in buckets -> convert each bucket item into a HANDOFF ticket referencing
   the pack's `suggested_next_action`. Update existing tickets; do not
   duplicate. Record every real run as a dated doc under
   `HANDOFF/harness/` (see `JEV_DOGFOOD_RUN_2026-09-22.md` for the
   template: per-line outputs in full, sources cited, defects surfaced).
4. **Pack changes** follow the table above; keep pack edits in the same PR
   as any doctrine change they reflect (the pack encodes repo doctrine).

## Known limits

- Freebuff-side MCP config is a manual paste step (see above); everything
  else is file-managed in-repo.
- The installed CLI is not on PATH by default; call the full exe path
  (`%APPDATA%\Python\Python314\Scripts\harness.exe`). `python -m harness`
  does NOT work (the package has no `__main__`); use `python -m harness.cli`
  or `python -m harness.mcp` with `PYTHONPATH` set to the worktree.
- The bucket pack encodes SCM doctrine as of 2026-09-22 — review it when
  doctrine changes (it is versioned `scm-ops-issues-v1` for this reason).
  Run history: `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md`.
