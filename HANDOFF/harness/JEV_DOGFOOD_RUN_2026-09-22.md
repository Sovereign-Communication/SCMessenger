# JEV Dogfood Run 001 — Bucketed Issue-Sort on Real SCM Failure Data

Status: Complete
Date: 2026-09-22
Executor: Buffy (Freebuff lane, Windows host)
Feature under test: Harness JEV-P5 `issue-sort` (buckets replace pass/fail
for completion/quality checks; never invents buckets or actions)
Pack: `HANDOFF/harness/packs/scm-ops-issues-v1.json` (validated through
`harness.jev_packs.validate_operator_pack` before and after the run)
Harness version: main `f07c814` (verified == origin/main immediately before
the run; editable install tracks that checkout)
Runner: installed console script `harness.exe issue-sort --issue <line>
--pack <pack>` invoked per line; full JSON outputs preserved in the session
transcript and summarized in full below.

## Sync verdict (step 1 of the dogfood loop)

Harness main `f07c814` == `origin/main`, 0/0 ahead/behind at run time.
Verdict: **already newest**. No pull, no reinstall needed.

## The five real failure lines and their sorts

Every input line is genuine SCM failure evidence quoted from sources read
this session (sources cited per line). Expected buckets are the manual
analysis; the tool had to agree.

| # | line id | bucket (tool) | conf | expected | attention | path_id from pack |
|---|---|---|---|---|---|---|
| 1 | D9-panic | supply_chain | 0.97 | supply_chain | high | Cargo.lock |
| 2 | D1-cap-refusal | capacity | 0.73 | capacity | high | core/src/transport |
| 3 | D1-inbound-denied | capacity | 1.0 | capacity | high | core/src/transport |
| 4 | MSG-STORE-LOCK-001 | storage | 0.99 | storage | high | core/src/store |
| 5 | AAPT2-XML-358 | android_build | 1.0 | android_build | medium | android/app/src/main |

**Result: 5/5 agreement with manual analysis.**

## Line sources (all read this session, per rule 13)

1. `D9-panic`: OC `SCM_NODES_AUDIT.md` node journal — "thread 'tokio-rt-worker'
   panicked at libp2p-swarm-0.48.0/src/handler/either.rs:110: internal error:
   entered unreachable code", 12 occurrences since 2026-09-21T21:01:17Z.
   Ticket: `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md`.
2. `D1-cap-refusal`: OC audit D1 section — 173x "connection_limits: limit 4
   reached", the phone peer refused 169 times. Fix landed as V040-T-CONN-04,
   PR #359.
3. `D1-inbound-denied`: OC audit D1 section — "Inbound connection DENIED from
   /ip4/13.217.204.112/tcp/56864 -> /ip4/172.31.18.74/tcp/9001", last seen
   03:45:35Z.
4. `MSG-STORE-LOCK-001`: RCA doc
   `HANDOFF/freebuff/inbox/MESSAGE_STORE_LOCK_STOP_START_RCA_2026-09-21.md`
   (on PR #351) — "Message Store Unavailable" after Stop->Start; FileLoggingTree
   retained the core past teardown so the sled lock survived.
5. `AAPT2-XML-358`: CI run 35681051803 (Mobile, 2026-09-22T03:09:49Z) —
   "network_security_config.xml:25:49: The string \"--\" is not permitted
   within comments" (aapt2/XMLStreamException). PR #358 was red on exactly
   this; fixed by `24e6d93d` in wt-canon, re-run green.

## What the run surfaced (two real defects found by dogfooding)

These are the point of the exercise — the tool feature worked AND earned its
keep immediately:

1. **Pack defect: bare `ble` keyword substring-matched inside
   "unreacha-ble"** (D9's panic text). First-pass sort landed D9 in
   `ble_lane` with confidence 0.0 — a false positive caused by our pack
   keyword, not the tool. `match_keywords` (harness/jev_packs.py:382) is
   documented substring matching with hit-count scoring; a bare 3-letter
   keyword is a footgun. **Fix applied to the pack**: BLE keywords made
   distinctive (`ble transport`, `ble gatt`, `ble advertise`, ...); re-run
   sorted D9 correctly into `supply_chain` at conf 0.97.
2. **Pack gap: the buckets the runbook documented did not exist in the
   pack** — no android-build bucket, no supply-chain bucket, and the
   runbook's "attention" fallback bucket does not exist (unmatched is a
   result status, not a bucket; the honest `status: "unmatched"` +
   "no declared pack keyword match" handling is correct tool behavior).
   **Fix applied**: real `android_build` and `supply_chain` buckets added
   (10 buckets total); runbook corrected in the same push.

## Execution-mode note (recorded honestly)

First pass ran the structural-only fallback (`is_fallback: true`, model
`jev-latest`, cost 0.0, reason "keyed Jev evaluation requires the shared
spend governor") — that pass surfaced defects 1 and 2 above. The post-fix
pass ran **fully keyed**: `is_fallback: false`, model `jev-1.13.0`,
confidence 0.73-1.0 per line, total cost ~0.12. Both passes agreed on all
five buckets after the pack fix. The keyed pass is the feature's real
capability; the structural pass is the honest degrade path, and both are
visible in the outputs by design.

## Ticket reconciliation (per docs/runbooks/HARNESS_MCP_SETUP.md loop step 4)

- D9-panic -> existing `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md`
  (updated with the run cross-reference; no duplicate filed).
- D1 both lines -> fix already landed (PR #359, V040-T-CONN-04); no ticket
  needed, outcome recorded here.
- MSG-STORE-LOCK-001 -> fix already landed on PR #351; no ticket needed.
- AAPT2-XML-358 -> already fixed (`24e6d93d`); no ticket needed.
- No new tickets created: every sorted item mapped to an existing ticket or
  a landed fix, exactly what "update, don't duplicate" demands.

## Verdict

The bucket feature is production-usable on SCM data: deterministic pack
contract, honest fallback/keyed distinction, per-line confidence, actionable
`path_id` + `suggested_next_action` straight from the pack into ticket
triage. The pack encodes SCM doctrine and now matches the runbook; changes
to it follow `docs/runbooks/HARNESS_MCP_SETUP.md` (edit -> validate ->
commit).

## Remaining operator step

One line: paste the `harness` MCP server into Freebuff desktop config
(exact values in `docs/runbooks/HARNESS_MCP_SETUP.md`) so future sessions
call `issue_sort` as an MCP tool instead of shelling the CLI.
