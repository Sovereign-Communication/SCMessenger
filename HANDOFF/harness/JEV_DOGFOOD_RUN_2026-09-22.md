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

---

# Extension 2 (same day) — Completion Gate (jev-phase) + Live-Node Sorts

Second dogfood pass: the pass/fail-replacement completion gate
(`harness jev-phase`, jev_completion.py, merged Harness PR #39) exercised
for the first time here, plus a sort pass over genuinely live node logs.
Sync re-checked first: Harness main `f07c814` still == `origin/main`
(already newest). Freebuff MCP config surface re-checked once more: still
not file-discoverable (only `update-state.json` under `$APPDATA/Freebuff`)
— CLI path stands; the operator paste-step remains the one manual item.

## A. Completion-gate runs (jev-phase)

Surface as it actually exists (read from source before running):
`--phase <id> --repo-root <root> --evidence <json> --min-score N`, six hard
gates (pr_merged 25, origin_evidence 10, required_tests_present 20,
local_gates_green 15, ci_green 15, no_open_blockers 15 = 100),
`combined = 0.7*mechanical + 0.3*semantic` when all gates pass, else
`min(mechanical, semantic, min_score - 0.01)` — hard fail can never clear
the threshold regardless of prose. Native phase contracts read a
`docs/jev-roadmap.md` STATUS row; SCM has none, so SCM work items go
through the documented `--evidence` override surface (values below were
fact-checked this session: gh pr view 351 -> 0 fail / 33 total).

### A1. Native control run: Harness's own JEV-COMPLETION phase

`harness.exe jev-phase --phase JEV-COMPLETION --repo-root <Harness> --json`

- Hard gates: ALL PASS (pr_merged=true — PR #39 MERGED, roadmap STATUS row
  read natively). Mechanical score: 100.0.
- Keyed semantic (`jev-1.13.0`, is_fallback=false, cost 0.022176, conf
  0.76): **2.76** -> combined 70.83 < 85 -> `can_mark_complete: false`,
  exit 1, `[FATAL] ... do not mark complete`.

**Finding (upstream, Harness): the semantic-scale anomaly.** A phase that
is 100/100 mechanical and gate-green is blocked solely because the keyed
semantic score of 2.76 is blended as if it were 0-100. `jev_completion.py`
maps 0<=raw<=1 to x100 and takes anything >1 at face value — but the typed
score legend appears to run 0-5 (or similar), so a good answer scores ~2-4
"raw" and reads as catastrophic on the 0-100 blend. On a correct 0-100
semantic this phase would score ~97 and complete. Fail-closed direction is
right (nothing wrongly marked complete); the scale contract between
`_COMPLETION_PACK` and `jev-1.13.0` is the defect. Reported here for the
Harness maintainer; no SCM code involved.

### A2. SCM work item via override mode: PR #351 (green, unmerged)

`harness.exe jev-phase --phase JEV-P3 --repo-root <SCMessenger> --evidence
tmp/jev-dogfood/phase351_evidence.json --json` (phase id JEV-P3 = neutral
contract slot; the evidence JSON carries the real SCM facts: status_row
"PR #351 OPEN ... 33/33 green", pr_merged=false, ci_green=true,
local_gates_green=false).

- Hard gates: pr_merged=false, local_gates_green=false (honest: not merged,
  local gates not run per CI-primary doctrine), origin_evidence=true,
  required_tests_present=true, ci_green=true, no_open_blockers=true.
- Mechanical 60.0; keyed semantic 1.55 (conf 0.36, cost 0.022302);
  combined 1.55 -> `can_mark_complete: false`, blockers named:
  "hard gate failed: pr_merged", "hard gate failed: local_gates_green".

**Verdict: the gate's SCM verdict is CORRECT and useful** — PR #351 is
exactly "green but not complete" (merge review pending), and the gate says
so structurally, naming the two missing gates, immune to a green CI
looking like completion. The semantic number itself is unreliable pending
the A1 scale finding; the hard-gate layer carried the verdict here.

## B. Live-node sort pass (3 genuine lines from the running Windows node)

Source: `tmp/radio-candidates/56d66f7/node-stdout.log` + `node-stderr.log`
(this is the live node's own log, PID 5924, read this session). Health
endpoint `http://127.0.0.1:9876/health` -> `{"status":"healthy"}`;
`/peers` returned empty body at read time.

| line | tool verdict | keyed | agreement |
|---|---|---|---|
| 09:05:10Z WARN dial_policy `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts peer_id=12D3KooWMFSh...` | `backoff` conf (keyed) | yes, jev-1.13.0 | AGREE (manual: D2 mechanism) |
| recurring ERROR `warp::server::run: server connection error: hyper::Error(Parse(Method))` (~10s cadence through 09:06:53Z) | `unmatched`, "no declared pack keyword match" | structural-only | AGREE (honest; no API-noise bucket exists) |
| paired WARN `yamux::connection: ... (os error 10053)` | `capacity` conf 0.72, attention high | yes, jev-1.13.0 | DEFENSIBLE-BUT-QUESTIONED (no capacity keyword in text; see B1) |

**New live finding, D2 confirmation**: the 09:05:10Z DIAL-BACKOFF line is
D2 firing in real time on the Windows node, against a peer id
(`12D3KooWMFSh...`) not present in the OC audit's 15/4/2/1 counts — the
mechanism is continuous, not a one-day artifact. D2 ticket updated with
this live confirmation (no duplicate filed).

**B1 observation (bounded-judgment discipline, one data point each)**: the
warp line's full output shows the keyed model privately chose `capacity`
at conf 0.38 while the code-owned layer held the line to the pack and
returned honest `unmatched`; the yamux line was accepted as `capacity` at
0.72 with zero keyword support. Observed acceptance discipline therefore
sits somewhere between 0.38 (rejected) and 0.72 (accepted) — hypothesis
only, two data points, flagged as such. Worth pinning in Harness tests if
the threshold is meant to be deterministic.

## C. Read-only intel: the uncommitted Cargo.toml/Cargo.lock diff (rule 11)

Not mine, not touched. What it contains (read via `git diff` this
session): the OC session's **D9 fix in flight** — a
`[patch.crates-io] libp2p-swarm = { path = "vendor/libp2p-swarm-0.48.0" }`
fork whose vendored copy degrades the `unreachable!()` mismatches to
logged drops (`"D9-DEGRADE"`) instead of panicking, matched-side routing
unchanged; workspace.members gains the vendored crate; the lock's +273
lines are that crate's dev-dependencies (criterion, ciborium, env_logger,
...) entering via the new member. Note: this supersedes the earlier
"vendor/ is a pristine read-only research copy" disposition in the OC lane
audit — the session evolved from research to patch fork. Implication for
the operator: when this lands it changes transport behavior and needs the
Rule-8 adversarial review on merge, and the D9 ticket's fix-options list
should be read alongside this in-flight direction.

## Extension-2 verdicts

- Completion gate: real, usable on SCM work items via evidence overrides;
  correct structural verdict on PR #351; one upstream scale-contract
  defect found and documented (A1).
- Live sorts: the pack+keyed pipeline sorts real node logs correctly and
  refuses to invent buckets on out-of-pack noise; one live D2 confirmation
  captured; one borderline keyed acceptance (B1) flagged for Harness.
- Sync: already newest. MCP: CLI path; operator paste-step unchanged.

## Extension 3 — 2026-09-23: node restored on the D9 build, first live D9-DEGRADE observation, prober attributed, batch B sorted

Context: the Freebuff restart killed the unmanaged Windows node process AND
its staging binary (`tmp/radio-candidates/56d66f7/` retained logs only).
Restored per the CI-primary runbook instead of a local build:

- CI artifact `windows-cli-1ec0c2424c9cd2a292a43c7f9c99d5168610430c`
  (run 35787291590, success, PR #361 head `63f4a7d7` -> merge-sha build
  `1ec0c24`, which contains D9-degrade + D1) downloaded to
  `tmp/radio-candidates/1ec0c242/`; provenance printed by the binary:
  `CLI Version: 0.4.0 (1ec0c24 2026-09-22T21:35:37Z)`.
- Node relaunched under `scripts/run_node_supervised.ps1` (bounded restart,
  double-start refusal) — an upgrade over the old unmanaged process.
  `/health` -> `{"status":"healthy"}` at 08:00Z.

### First live D9-DEGRADE observation (third node with live proof)

Fresh log `scm.log.2026-09-23-08`, verbatim:

```
2026-09-23T08:00:27.111020Z  WARN libp2p_swarm::handler::either: D9-DEGRADE: Either on_behaviour_event event/handler side mismatch; dropping event
2026-09-23T08:02:26.185793Z  WARN libp2p_swarm::handler::either: D9-DEGRADE: Either on_behaviour_event event/handler side mismatch; dropping event
```

Two side-mismatch events in the first two minutes — the exact sites where
the old build panicked a tokio task — degraded to logged drops. Zero
`panicked` lines on the new build so far.

### Prober attribution closed -> new ticket D10

Corrected facts (commands on file this session): the old log's
Parse(Method) count is 96 over ~18h (~5/h, not the ~10s cadence extension 2
inferred from a compressed window); `warp::server::run` is the node's
**9001** HTTP+WS server (extension 2's "9876" attribution was wrong); the
node's multi-port adaptive listener binds 0.0.0.0:80/443/8080/9002/9090 —
internet/LAN TLS probes against those are the classic Parse(Method)
trigger. Filed: `HANDOFF/todo/D10_PARSE_METHOD_SCANNER_NOISE_PUBLIC_PORTS.md`
(exposure-scope question for the operator + noise-classification option;
Rule-8 if the binding policy changes in `core/src/transport/`).

### Batch B: three verbatim fresh-window lines, sorted

Evidence discipline note: an earlier draft of the batch used assumed lines
(zero actual Parse(Method) hits exist in the fresh window; today's
DIAL-BACKOFF lines are healthy `Reset backoff state after successful
connection` events, NOT dead-marks — D2's live confirmation remains
yesterday's 09:05:10Z line). Rebuilt verbatim before sorting:

| id | line (abbrev) | expected | sorted | conf | mode |
|---|---|---|---|---|---|
| LIVE-D9-DEGRADE-080027Z | D9-DEGRADE Either mismatch | supply_chain | **supply_chain** | 0.0 (fallback) | keyword, `is_fallback: true` |
| LIVE-BLE-HRESULT0-080025Z | BLE GATT error HRESULT(0x0) "completed successfully" | unmatched | **ble_lane** | 1.0 | keyed, `is_fallback: false` |
| LIVE-CUSTODY-SWEEP-INFO-080025Z | Custody retention sweep expired 0 of 115 (healthy) | unmatched | **storage** | 1.0 | keyed, attention=high |

### New findings

- **A2 (Harness, defect observed, handled):** the keyed model returned a
  malformed probability vector on the D9 line ("invalid TypeSafe response:
  bucket.probabilities must sum to 1"); Harness's TypeSafe layer caught it,
  fell back to the keyword path, and recorded the reason in
  `evidence_refs`. Honest degrade confirmed in production — the score
  itself is fine, the model's schema adherence is not. Same upstream
  maintainer audience as A1.
- **B2 (feature-scope finding):** the sorter ROUTES but does not judge — a
  healthy custody-sweep line landed in `storage` at confidence 1.0 with
  attention=high. Bucket membership is not severity, and the pack has no
  "benign/healthy" outcome. Either the pack needs an explicit healthy
  bucket or issue-sort consumers must triage on the line's own level
  (WARN/ERROR) before acting on the bucket. Recorded as a dogfood lesson;
  no Harness change requested (the tool does what it documents).
- **BLE HRESULT-0 line:** already known — evidence in
  `HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md` (line 25); no
  new ticket (the model's `ble_lane` @ 1.0 was the right triage route; my
  `unmatched` expectation was too literal).

### Merge-blocker flag (not acted on)

#359 and #360 are now CONFLICTING with main (parallel doctrine-file edits:
AGENTS.md rule 17, freebuff README rewrite, CTO_STATE appends). The local
working tree's Cargo.toml/Cargo.lock/vendor/ are byte-identical to
`origin/d9-libp2p-degrade` (#361), so resolving conflicts here would touch
foreign working-tree state (rules 11/12) — flagged for the operator/merge
seat; #361 itself is MERGEABLE and green.

### MCP status (cheap re-check)

Still no discoverable Freebuff MCP config surface — CLI path validated
end-to-end again this session; the operator paste-step remains the one
manual item (`docs/runbooks/HARNESS_MCP_SETUP.md`).

## Extension-3 verdicts

- CI-primary node restore worked exactly as canonized: artifact + provenance
  check + supervised relaunch, zero local build minutes spent.
- D9-degrade now has live third-node evidence; ticket already marked
  DEPLOYED (no update needed).
- The pack+keyed pipeline survived a malformed model response (A2) and an
  author over-expectation (BLE route) without lying once — both degrade
  paths visible in the saved outputs.
- Tickets reconciled: D10 filed (new finding); BLE/HRESULT-0 mapped to the
  existing P1 wedge ticket; D2 unchanged (fresh window healthy).
