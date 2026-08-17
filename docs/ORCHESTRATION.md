# SCMessenger Orchestration Protocol

Status: Active. Last updated: 2026-08-14. Section 2.2 and Section 3 still
route dispatch, parsing, handoff, and build serialization through
`dispatch_dial.py` / `parse_orchestration_footer.py` / `batch_handoff.py` /
`build_lock.py`; Section 3.1 adds task-sized dynamic resource admission.
The existing dispatch rules remain unchanged in substance; the resource
policy now records active reservations and uses fresh host telemetry -- full
audit, what was tested, and every edge case considered:
`HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md`.

This is the single canonical reference for orchestration. There is now ONE
orchestrator command -- `/orchestrate` (`.claude/commands/orchestrate.md`) -- and
it is a thin launcher that points back here. Any model, running anywhere, can
drive the v1.0.0 farm build by reading this document plus the shared state files
in Section 2. The superseded per-backend commands (scmorc, scm, scmqwen,
gemini-orchestrator, swarm) are archived under `.claude/archive/commands/`; their
behaviour is preserved below as selectable BACKENDS (Section 5), not as separate
commands.

---

## 0. Operating Contract (read first -- applies to every orchestrator, every model)

These five rules are absolute. They are written plainly so that even a small,
tool-poor but instruction-following model can orchestrate correctly.

1. **DELEGATION IS MANDATORY. The orchestrator never writes application code.**
   Every implementation, fix, test, or analysis task is dispatched to a lake
   (Section 1). You may directly edit ONLY: HANDOFF task files (state moves), the
   backlog tracker, prompt files under `tmp/`, and a surgical 1-3 line compile fix
   that is the sole thing blocking a build gate. Anything larger -> delegate. If
   you are about to type code into a `.rs/.kt/.java/.swift/.ts` file, STOP and
   dispatch instead.

2. **The canonical dispatch path is a script, so ANY model can run it.**
   `python scripts/delegate_task.py --task <file> --provider <lake> [--model <m>]
   --files <targets> --apply --verify "<gate>" --mode diff --max-rounds 3`.
   The native `Agent` tool and `claude -p` workers are OPTIONAL accelerators
   available only when the orchestrator is Claude; they are never required. A
   non-Claude orchestrator uses the script for 100% of dispatches.

3. **You are the only writer of builds, commits, and state.** Workers implement
   and report; they never run `cargo`/`gradlew`, never commit, never move HANDOFF
   files. You run the gate, you move the ticket, you commit. One build at a time
   (Windows rlib-lock safety, Section 9).

4. **Follow the loop in Section 2.2 for every task**, in order: read queue ->
   validate -> dial (tier/lake/model via `dispatch_dial.py`, which applies
   the 2.1 ladder automatically through `lake_route.py`) -> dispatch ->
   verify gate (structured footer parse, then the real gate yourself -- a
   worker's own claim is never a substitute) -> security gate if required
   -> mark complete + commit + record ledger in one `batch_handoff.py`
   call. No step is optional.

5. **Record every dispatch in the ledger** (`tmp/lakes/ledger.jsonl` via
   `scripts/lake_route.py --record ...`). The router is blind to what you do not
   record; unrecorded dispatches burn lakes twice.

Escalate to the operator before: architecture-direction changes, security/privacy
trade-offs, tech-stack changes, API-contract breaks, or release/versioning
decisions.

---

## 1. Lake Registry

All agent API lakes available to any orchestrator. Full endpoint + model + quota
registry, the ranked free-tier and tokens/$ comparison, and the rotation strategy:
**`docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md`**.

### Active lakes (wired in `scripts/delegate_task.py` today -- valid `--provider` values)

| Lake        | Provider          | Best For                                              | Tiers              |
|-------------|-------------------|-------------------------------------------------------|--------------------|
| qwenpaid    | Alibaba paid plan | PRIMARY for all dispatches (operator 2026-07-28)      | MAX (qwen3.8-max-preview) |
| qwen        | DashScope/Alibaba | Rust/Kotlin implementation, deep CODER/THINK capacity | FLASH/CODER/THINK/MAX |
| groq        | Groq Cloud        | Fast FLASH micro-tasks; small TPM, micro-chunk        | FLASH/CODER        |
| openrouter_direct | OpenRouter (dedicated key) | Backup lane for clearly scoped tasks; DeepSeek V4 Flash, USD 1/day cap | FLASH/CODER |
| openrouter  | OpenRouter        | Free-model spillover; 1,000 req/day (via $10 topup)   | FLASH/CODER        |
| gemini      | Google AI Studio  | Large-context review, whole-file analysis (key-gated) | FLASH/CODER/THINK  |
| ollama      | Ollama free tier  | Small overflow (a few tasks/week); air-gap fallback   | FLASH/CODER        |

### Candidate lakes (DOCUMENTED ONLY -- not yet wired; registry Section 6 has the exact add)

Do NOT pass these as `--provider` yet: `delegate_task.py` rejects any provider not
in its `choices` list, and each needs a `~/.config/scmorc/<lake>.env` key file
first. They are researched and ready to wire, nothing more.

| Lake        | Provider          | Best For                                              | Tiers              |
|-------------|-------------------|-------------------------------------------------------|--------------------|
| mistral     | Mistral (Plateforme+Codestral) | Best free code lake: 1B tok/mo, 500K TPM, Codestral | FLASH/CODER |
| nvidia      | NVIDIA NIM        | 100+ models (qwen3-coder, DeepSeek, GLM); no CC       | FLASH/CODER/THINK  |
| sambanova   | SambaNova Cloud   | Largest daily free budget; DeepSeek V3.2              | FLASH/CODER/THINK  |
| cerebras    | Cerebras          | Fastest inference; 8K free context -> mechanical only | FLASH              |
| modelscope  | Alibaba ModelScope| 2,000 calls/day free, separate from DashScope         | FLASH/CODER        |
| scaleway    | Scaleway (EU)     | qwen3-coder-30b, devstral; 1M free tokens             | FLASH/CODER        |
| deepseek    | DeepSeek (paid)   | Cheapest capable coder/$: V4 Flash, 98% cache discount | CODER/THINK       |

Note: full quotas, endpoints, key files, and the free vs paid tokens/$ comparison
live in `docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md`. Standing reality (2026-07-20):
Ollama Cloud Pro is NOT currently subscribed (purchase candidate); OpenRouter sits
at 1,000 req/day thanks to the one-time $10 lifetime topup. Groq's small per-minute
token cap means prompts over ~6K tokens must be micro-chunked (Section 6);
big-context lakes (qwen, mistral, nvidia, sambanova, gemini) do not.

---

## 2. Shared State Files

All orchestrators read and write these files. State lives in files, not in any
model's memory -- this is the unification property: any model can take over
orchestration by reading the queue and ledger.

| File                              | Purpose                                                  |
|-----------------------------------|----------------------------------------------------------|
| `HANDOFF/todo/_QUEUE.md`          | Live human-readable dispatch order                       |
| `scm_v1_farm_queue.jsonl`         | Machine-readable task queue (one JSON per line)          |
| `tmp/lakes/ledger.jsonl`          | Quota ledger -- append-only, one entry per dispatch      |
| `tmp/lakes/round_robin_state.json`| Per-lake per-tier model rotation counters                |
| `tmp/lakes/registry.json`         | Lake registry snapshot (seed from docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md) |
| `tmp/scmorc/dispatch_log.md`      | Human dispatch log (all orchestrators append here)       |

---

## 2.1 Cross-Lane Dispatch Ladder

For any task, try lanes in this order (first available with quota wins).
Where this prose order and `scripts/lake_route.py`'s `TIER_LADDERS`
disagree, the code is authoritative -- the dial (Section 2.2 step 3) always
follows the code, and the router skips lakes with no key file or active
cooldown automatically:

1. **Groq FLASH** (`delegate_task.py --provider groq --model llama-3.1-8b-instant`):
   mechanical tasks, docs, config. Fastest inference. Micro-chunk to <=6K
   tokens if prompt is large (see Section 6).
2. **Qwen FLASH** (`delegate_task.py --provider qwen --model qwen3-coder-flash`):
   mechanical tasks when Groq daily cap is hit.
3. **Groq CODER** (`delegate_task.py --provider groq --model qwen/qwen3-32b`):
   standard implementation on fresh daily window. Micro-chunk to <=6K tokens.
4. **Qwen CODER** (`delegate_task.py --provider qwen --model qwen3-coder-plus`):
   Rust/Kotlin implementation, 128K context, no size limit. Primary CODER lane.
5. **Gemini CODER** (`delegate_task.py --provider gemini --model gemini-2.5-flash`):
   large-context review, whole-file diffs. Secondary CODER lane. KEY-GATED:
   needs `~/.config/scmorc/gemini.env` (absent 2026-07-17; router skips it
   automatically -- the agy CLI sign-in does not cover this lane).
6. **OpenRouter CODER** (`delegate_task.py --provider openrouter --model deepseek/deepseek-chat-v3:free`):
   spillover when Qwen tiers saturate.
7. **Qwen THINK** (`delegate_task.py --provider qwen --model qwen3-235b-a22b-thinking-2507`):
   adversarial review, hard design, failed-CODER escalation.
8. **Gemini THINK** (`delegate_task.py --provider gemini --model gemini-2.5-pro`):
   large-context adversarial review. Same gemini.env key gate as lane 5.
9. **Fusion Lite** (`scripts/fusion_lite.py --max-cost 0.01`): planning,
   verification, and JUDGEMENT (Section 10). Caps: $0.01 default, $0.05 for
   hard problems (operator-settled 2026-07-17). Never implementation. Never
   raise caps without operator approval.
10. **Claude native**: [AUDIT-GATE] adversarial verdicts (fable), structural
    deadlocks, 2+ free-lane failures. Burns Anthropic subscription window.

---

## 2.2 The Orchestration Loop (run this for every task)

This is the whole job. It was previously duplicated across five command files;
it now lives here once. Revised 2026-08-03: this used to be 10 steps: read,
validate, write-prompt, pick-lake, dispatch, verify, security-gate,
mark-complete, commit, record. It is now 9 -- pick-lake became step 3
(DIAL, via `dispatch_dial.py`), and mark-complete/commit/record merged into
one step 8 (`batch_handoff.py`) -- because those three used to be separate
manual actions and now happen atomically in one call. See
`HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md` for the full audit, what was
tested, and every edge case considered. Every safety rule in Section 9
still applies unchanged; only the mechanism changed. Follow it in order.

1. **READ QUEUE.** Open `HANDOFF/todo/_QUEUE.md`; take the top actionable ticket.
   Group consecutive tickets by domain (rust-core / android / wasm / desktop /
   docs) to reuse worker context.
2. **PRE-DISPATCH VALIDATION** (cheap, orchestrator-local -- never spend a worker
   on a dead task). Read the ticket, identify the concrete target (symbol/file),
   grep for it and note the line range -- it feeds step 4's scoped `--files`:
   - FALSE_POSITIVE (target is a test/Kani/proptest/`GOLDEN_*` literal) -> move to
     `HANDOFF/done/` with a note; next ticket.
   - ALREADY_WIRED (the thing to "wire" already has callers) -> move to done/; next.
   - NEEDS_REVIEW (target missing/ambiguous) -> STOP, ask the operator.
   - VALID -> continue.
3. **DIAL.** `python scripts/dispatch_dial.py --tier <ticket tier> --files
   <targets> --description "<ticket text>" --retry-count <N>`. Returns the
   effective tier (auto-escalated to THINK if step 2's target falls under
   `core/src/{crypto,transport,routing,privacy}/`, per Section 4 -- never
   send analysis or judgement to a FLASH lake, Section 9.13), the
   `thinking` flag, `security_gate_required`/`delivery_gate_required`
   booleans, and the lake/model to dispatch to (applies the Section 2.1
   ladder automatically via `lake_route.py` -- never pick a lake by hand
   and never call `lake_route.py` directly from this step). Empty `lake`
   in the output means no quota anywhere on this tier: escalate to the
   operator or fall to the `native`/`agent` backends (Section 5) instead
   of guessing.
4. **WRITE the worker prompt** to `tmp/<slug>.prompt.md`: self-contained --
   requirement, exact target file paths (use step 2's scoped
   `path:Lstart-Lend` syntax when the target is a narrow slice of a large
   file), acceptance criteria, and the exact build-gate command. Include
   the Worker Contract header (Section 3) -- step 6 depends on the footer
   format being present, don't skip it. For a local direct worker or build
   lane, include the task kind, peak estimate, safety margin, reservation ID,
   and operator approval (if any) after obtaining the reservation from
   `scripts/resource_admission.py`; remote API-lake calls use quota accounting
   unless they launch local descendants.
5. **DISPATCH** (canonical, any model): `scripts/delegate_task.py --task <file>
   --provider <dial's lake> --model <dial's model> --files <targets, scoped
   if applicable> --apply --verify "<gate>" --mode diff --max-rounds
   <dial's max_rounds>`. Wrap the verify command with `scripts/build_lock.py
   --run "<gate>"` (Section 9.5 -- never run two verify jobs concurrently).
   Claude-only accelerators, if available: the `Agent` tool, `claude -p`
   workers, or the ollama pool via `orchestrator_manager.sh` (Section 5).
   Always `--mode diff` (Section 9.3).
6. **VERIFY.** `python scripts/parse_orchestration_footer.py
   tmp/<slug>_response.md` for the structured result (`result`, `files`,
   `notes`). A missing/degraded footer (`degraded: true`) is not an error by
   itself -- fall back to reading the response body directly and `git diff
   --stat` scoped to the ticket, exactly as before this revision. Either way:
   - ZERO-DIFF -> do not trust it; ticket stays in todo/, log `requeued`.
   - Real diff -> run the matching gate YOURSELF, under `build_lock.py`
     (Rust `cargo check --workspace`; Android `cd android && ./gradlew
     assembleDebug -x lint --quiet`; WASM `cargo check -p
     scmessenger-wasm --target wasm32-unknown-unknown`;
     `CARGO_INCREMENTAL=0` on Windows). Grep the diff for
     `simulate|mock|placeholder|in a real implementation` -- a clean compile
     is NOT completion (Section 9.1). A worker's own `VERIFICATION:` field
     is NEVER a substitute for running this yourself (Section 3: workers
     dispatched via `delegate_task.py` cannot execute code at all and must
     report `VERIFICATION: NONE`).
7. **SECURITY GATE** (Section 4). Step 3's `security_gate_required` (or a
   manual check: diff touches `core/src/{crypto,transport,routing,privacy}/`)
   -> mandatory adversarial review at THINK/MAX tier before commit.
   `delivery_gate_required` (outbox, receipt, custody, retry) -> triangulate:
   3 distinct verifier dispatches or one Fusion Lite panel (Section 10).
8. **MARK COMPLETE + COMMIT + RECORD**, in one call: `python
   scripts/batch_handoff.py --batch-file <batch.json for this ticket>
   --provider <lake> --commit-message "<task>"`. Moves the ticket to
   `HANDOFF/done/` (only for a real diff + passing gate + security pass
   where required -- a task is not complete until the file has moved),
   commits (`git add -A && git commit`; provenance `<prov>:` in the
   message, `native:` for Claude-worker completions, `swarm:` for
   foreign/pool completions; never push unless the operator asks), and
   records the ledger entry via `lake_route.py --record` in the same call
   -- the router is blind to what you do not record. Batching several
   tickets from one dispatch round into a single `batch_handoff.py` call
   (one commit instead of N) is preferred; a single-ticket batch works too.
9. Re-check quota/cooldowns and return to step 1. Stop when the queue is
   empty, a NEEDS_REVIEW/escalation is hit, or the operator interrupts.

---

## 3. Worker Contract

Revised 2026-08-03: field names and the RESULT vocabulary are now identical
to AGENTS.md's own REMOTE SANDBOX/FOREIGN WORKER report contract (AGENTS.md
lines 81-118), which states plainly it is "the canonical, model-agnostic
rules contract for ANY agent" -- so this contract extends it instead of
using a different vocabulary. The prior `PATCH:`/`VERDICT:` fields are
retired; `RESULT` + `FILES` + `NOTES` cover the same information.

Every worker response dispatched via `delegate_task.py` MUST end with this
footer (the diff/file content is the main payload, parsed separately by
`extract_diff_blocks`/`extract_file_blocks` -- this is a supplement at the
end, not a replacement for the first line of the response):

```
---ORCHESTRATION_METADATA---
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE
FILES: ["path/one.rs", "path/two.rs"]
NOTES: ["what changed", "anything the verifier must know before running gates"]
---END---
```

`VERIFICATION` MUST always be `NONE` for workers dispatched through
`delegate_task.py` -- they are pure text completion with no execution
environment, so a `CONTAINER(...)` or other claim of having run a gate is
false by construction and must never be trusted if one appears. (A
`CONTAINER(...)` claim is only ever meaningful from an actual REMOTE
SANDBOX-class agent with its own toolchain, and even then it is advisory
only, never authoritative -- Section 9.1 and AGENTS.md both apply
regardless of what a worker's footer says.)

`FILES`/`NOTES` accept a JSON list (preferred -- ask for it explicitly in
the dispatch prompt) or bare AGENTS.md-style comma-separated free text;
either parses correctly via `scripts/parse_orchestration_footer.py`. A
missing or unparseable footer (`degraded: true` in the parser's output) is
never treated as success -- Section 2.2 step 6 falls back to reading the
response body directly rather than assuming anything worked.

Workers NEVER: run builds (`cargo`, `gradlew`), commit, push, or move HANDOFF
files. The orchestrator owns ALL of those operations.

---

## 3.1 Dynamic Worker Resource Admission

The former fixed 384 MiB per-worker RSS ceiling is retired. Resource limits are
now task-sized and host-aware. A small read-only task should receive a small
reservation; a build or other approved heavy task may receive the memory its
measured process family requires.

Before every local direct-worker or build-lane launch, the orchestrator MUST:

1. Read the shared active-work registry at `tmp/lakes/active_workers.json` and
   reconcile completed/dead entries. The registry is shared across Prime
   sessions and terminal lanes; a local in-memory list is not sufficient.
2. Obtain fresh host telemetry: total physical RAM, currently available RAM,
   swap/memory-pressure state where the platform exposes it, and the worker's
   current process-tree baseline.
3. Estimate the peak RSS of the worker plus every descendant process for the
   exact task. Add the packet's safety margin (10% by default, or a larger
   margin when telemetry is uncertain) to obtain the requested reservation.
4. Admit the task only when the requested reservation plus every active
   reservation fits the current available RAM, the configured global utilization
   budget, and mandatory system headroom. Unknown or stale telemetry is
   `BLOCKED_RESOURCE_TELEMETRY`, not permission to guess.
5. Reserve before launch, bind the launched PID/process group, sample the full
   descendant tree during execution, update the observed peak, and release the
   reservation on completion or stop. A stopped worker's descendants must be
   confirmed gone before its reservation is released.

The canonical helper is:

```text
python3 scripts/resource_admission.py snapshot
python3 scripts/resource_admission.py reserve --task-id <id> --kind small|analysis|build --estimate-mib <peak> [--operator-approved --approval-note <text>]
python3 scripts/resource_admission.py bind --task-id <id> --pid <pid>
python3 scripts/resource_admission.py sample --task-id <id>
python3 scripts/resource_admission.py release --task-id <id>
```

`scripts/resource_manager.sh --admission` and `--status` expose the same
registry/telemetry snapshot; its legacy CPU and percentage checks are advisory
and cannot admit a worker without this gate.

`resource_admission.py` uses a 10% task margin, a 2 GiB minimum host-headroom
floor, and a 75% total worker-reservation budget by default. It also refuses a
fourth active local reservation. These are host safety bounds, not a per-worker
cap; they may be made stricter by the operator or platform telemetry, never
silently weaker. The one-build-at-a-time rule and `build_lock.py` remain
mandatory regardless of available RAM.

An explicit, authenticated human or terminal-operator directive may approve an
exception worker for any stated purpose. The approval removes the former
per-worker 384 MiB ceiling, but it does NOT bypass resource admission, active
reservation accounting, process-tree monitoring, host headroom, build
serialization, security gates, provenance separation, or the PR139 HARD
NO-GO. Record the approval, purpose, estimate, margin, reservation, and exact
operator wording in the dispatch packet and evidence report.

The three-direct-worker concurrency limit remains. Remote API-lake calls are
quota-tracked in `tmp/lakes/ledger.jsonl`; they are not treated as local RSS
workers unless they launch local descendant processes. Any dispatch that cannot
obtain a reservation remains queued with the measured required and available
values. Never start an unregistered worker or build and infer availability from
worker count alone.

## 4. Security Gates (mandatory -- no exceptions)

| Trigger                                               | Gate Required                                           |
|-------------------------------------------------------|---------------------------------------------------------|
| Any diff in `core/src/{crypto,transport,routing,privacy}/` | Adversarial review (THINK or MAX tier) before commit |
| Any WS-A delivery logic diff (outbox, receipt, custody, retry) | Fusion Lite 3-panel ($0.001 ceiling) OR 3 distinct Qwen verifier dispatches |
| E-01c dispatch                                        | E-01b must carry adversarial PASS on file first        |
| PQC-11/PQC-13 dispatch                                | E-01 (full chain) must be landed first                 |
| PQC-09 dispatch                                       | E-01 landed AND explicit AD-8 operator lift            |

---

## 5. Backends (HOW you dispatch -- not separate commands)

There is one command: `/orchestrate`. A "backend" is only the mechanism that
carries a given dispatch. Pick per task and mix freely within one run. All
backends share the Section 2 state files, so you can switch backend mid-sprint
with zero state loss. The old per-backend commands are archived under
`.claude/archive/commands/` and map onto this table.

| Backend | Invocation | Runs on | Use when | (archived command) |
|---------|------------|---------|----------|--------------------|
| Script lane (CANONICAL) | `scripts/delegate_task.py --provider <lake>` | Any free/paid API lake (Section 1) | Default for ~100% of tasks; the only path a non-Claude orchestrator needs | scmqwen, gemini-orchestrator |
| Native Claude worker | `claude -p ... --model <alias> --effort <lvl>` (background Bash) | Anthropic subscription window | AUDIT-GATE adversarial verdicts (fable), or a task with 2+ free-lane failures | scmorc |
| Native Agent subagent | `Agent` tool (`rust-implementer` / `android-qa` / `crypto-security-auditor` / `docs-sync-auditor` / `release-gatekeeper`) | Anthropic subscription window | Claude orchestrator wants isolated-context delegation without spawning a CLI | scm |
| Ollama pool (micro-swarm) | `orchestrator_manager.sh pool launch <agent> <task>` | Ollama free tier (small: a few tasks/week) + any cloud pool | Batch fan-out across pooled agents | orchestrate (old), swarm |

Rules that bind every backend: Free lanes first, always -- a native Claude
worker is the last resort, not the default (it burns the Anthropic window; the
Quota Governor tiers from the archived `scmorc` apply when you use that backend).
The DELEGATION-IS-MANDATORY rule (Section 0) holds identically no matter which
backend or which model is orchestrating.

---

## 6. Groq Micro-Chunking Rule

Groq free tier enforces ~12K tokens-per-minute. Any prompt exceeding 6K tokens
MUST be split before dispatch:

1. Identify the context-heavy section (usually embedded source code).
2. Split into <=6K-token chunks, each self-contained (repeat task header).
3. Dispatch chunk 1, receive response, then dispatch chunk 2 with the prior
   response inlined as context if needed.
4. Orchestrator merges partial patches before applying.

Use `scripts/lake_route.py --tier FLASH --probe-groq` to confirm current
Groq TPM headroom before a large dispatch.

---

## 7. State Machine

```
HANDOFF/todo/<ID>_*.md
  -> HANDOFF/IN_PROGRESS/<ID>_<lake>_<ts>.md   (when dispatched)
  -> HANDOFF/review/<ID>_evidence.md            (when gate evidence recorded)
  -> HANDOFF/done/<ID>_*.md                     (when all gates pass)
```

Every state transition requires the gate evidence named in the task packet.
Zero-diff worker responses are re-queued, not marked done.

---

## 8. Session Continuity

State is file-backed; resumption requires only: this document, the JSONL
queue, the ledger, and the HANDOFF tree. No model memory is required.
Follow `docs/historical/plans/API_LIMIT_MANAGEMENT_PLAN.md` and the routing/ledger sections of
`docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md` (Section 3) for per-lake exhaustion and
cooldown handling.

---

## 9. Lessons: 2026-07-17 Swarm Post-Mortem (READ before any batch dispatch)

Each rule below was paid for in a bad commit or a burned quota window.
Commits 71d02d4d/e298e9bf ("swarm: completed remaining queue") were reverted
by 23960b35/8da8cc90 after audit; do not repeat their failure modes.

1. **Compile-only verify is NOT a completion gate.** The reverted run's
   "passing" C-06 diff was 212 lines of simulated/mock dead code that
   compiled cleanly. After ANY exit-0 verify, grep the applied diff for
   `simulate|mock|placeholder|in a real implementation` before accepting,
   and give it an orchestrator quality pass.
2. **Know the delegate_task.py exit codes:** 0 = verified (still needs
   rule-1 quality pass), 2 = verify failed after all fix rounds, 3 =
   VACUOUS success (model returned no applicable file blocks -- treat as
   FAILED, never as done).
3. **Always dispatch with `--mode diff`.** Without it, flash-tier models
   emit prose summaries instead of applicable file blocks, producing
   vacuous successes (observed on E-02/E-04/D-05/D-01 in the reverted run).
4. **Platform-correct verify commands.** gradlew lives in `android\`, not
   the repo root (`gradlew.bat assembleDebug` from root fails with
   "Task 'assembleDebug' not found"). iOS targets CANNOT be verified on
   Windows -- xcodebuild does not exist here. Mark iOS packets
   BLOCKED-PLATFORM and route them to a macOS runner (H-01); never let a
   batch runner "fail" them against a nonexistent toolchain.
5. **One build at a time on Windows.** Never run two concurrent
   `delegate_task.py --verify` jobs (2 concurrent cargo/gradle builds risk
   rlib lock corruption; see .claude/rules/build.md). `scripts/run_tasks.ps1` v2 is
   strictly sequential for this reason. `scripts/build_lock.py --run "<gate
   command>"` (added 2026-08-03) enforces this with a tested advisory
   lockfile -- wrap every Section 2.2 step 5/6 verify command with it
   rather than relying on discipline alone; it also has a stale-lock
   recovery path and a `--wait-seconds` mode for a batch that would rather
   queue than fail.
6. **Batch runners NEVER auto-commit and NEVER move tickets.** Workers
   implement; the orchestrator reviews (adversarial gate for
   `core/src/{crypto,transport,routing,privacy}/`), moves tickets, and
   commits. `scripts/run_tasks.ps1` v2 writes `tmp/swarm_report.md` only.
7. **Hallucinated Target Files are real.** On D-03 the file-deducer emitted
   three nonexistent `SCMessengerTests/*.swift` paths, which would have
   become the worker's write allowlist. `scripts/deduce_files.py` now drops
   any emitted path not present in `git ls-files`. If a packet has no
   Target Files, re-run `scripts/fix_targets.py` before dispatch.
8. **Qwen non-stream 400** (`parameter.enable_thinking must be set to false
   for non-streaming calls`): fixed in delegate_task.py (all non-streaming
   DashScope calls send `enable_thinking=false`). If you see this error you
   are running an old script -- pull.
9. **Feed the ledger or the router goes blind.** After EVERY dispatch:
   `python scripts/lake_route.py --record --lake <lake> --model <model>
   --task <id> --result ok|429|403|413|error|timeout|vacuous`. The router
   skips lakes with no key file and honors cooldowns automatically -- but
   only knows what you record.
10. **Lane smoke results, 2026-07-17** (re-probe at sprint start):
    - LIVE: groq `llama-3.1-8b-instant`; qwen `qwen3-coder-flash`;
      ollama `gpt-oss:20b-cloud`; openrouter `morph/morph-v3-fast` (paid,
      routes fine).
    - DOWN: openrouter `:free` tiers (429 shared-pool saturation -- retry
      off-peak); ollama `qwen3.5:397b-cloud` (403 auth); gemini lane needs
      `GEMINI_API_KEY`/`GOOGLE_API_KEY` in `~/.config/scmorc/gemini.env`
      (the agy CLI's own sign-in does NOT cover delegate_task.py).
11. **Morph Lite** is for single-file surgical edits only (three lane bugs
    fixed 2026-07-17; see HANDOFF/MORPH_LITE_HANDOFF.md). **Fusion Lite** is
    planning triangulation only, on the spend-capped key at
    `~/.config/scmorc/openrouter_fusion.env`.
12. **`enable_thinking` must follow the model name.** DashScope non-thinking
    hybrids require `enable_thinking=false` for non-streaming; thinking models
    (qwen3-*-thinking-*) REQUIRE `true` (400 "restricted to True" otherwise).
    delegate_task.py now sets it from the model name. Symptom history: THINK
    dispatch 400'd and silently rotated down to a FLASH model -- a masked tier
    downgrade. A rotation that DOWNGRADES tier on an analysis/judgement task
    is a FAILED dispatch: fix the root cause, re-dispatch at the right tier.
13. **FLASH tier cannot do analysis.** On the E-00 pre-flight, a flash model
    ignored an explicit read-only instruction, emitted code blocks, and
    guessed constants instead of citing file:line evidence. Analysis and
    judgement dispatches: THINK tier minimum, never FLASH/CODER-flash.
14. **OpenRouter budgets (operator 2026-07-17):** `openrouter.env` =
    FREE-MODELS-ONLY (delegate_task.py refuses non-`:free` models);
    `openrouter_fusion.env` = shared paid Fusion+Morph key ($0.50 cap).
    Proven costs: Morph call $0.00086; Fusion 3-panel+judge $0.0013.
15. **OpenCode native agent map** (`.opencode/`): GLM-5.2 `orchestrator`
    (primary), kimi-k2.7-code `implementer`, deepseek-v4-flash explore +
    small_model, glm-5.1 general. Config loads at startup only -- RESTART
    opencode to activate; verify model IDs resolve (`opencode-go/<id>`).
16. **Orchestrator token overhead was the coordinator's problem, not the
    workers'.** 2026-08-03 audit found the orchestrator itself (not
    workers) was spending an estimated ~750-950 tokens/task on prompt
    construction, response-grepping, and one-commit-per-ticket state moves
    that a script can do for a fraction of that. `dispatch_dial.py`
    (tier/effort/lake/model decision, Section 2.2 step 3),
    `parse_orchestration_footer.py` (structured response parsing, step 6),
    and `batch_handoff.py` (batched state move + single commit + ledger
    record, step 8) now handle this. Full audit -- what was tested, the
    two real bugs testing found and fixed, and every edge case considered
    (concurrent builds, malformed worker output, capability-class limits):
    `HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md`. The same pass found
    `qwenpaid` was missing from `lake_route.py`'s `TIER_LADDERS` despite
    being the operator's stated primary lane since 2026-07-28 (fixed for
    CODER/THINK/MAX) -- and found that `tmp/lakes/registry.json` is
    gitignored, so that half of the fix does not survive a fresh checkout
    on its own; re-apply the `qwenpaid` block from
    `docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md` Section 1 (or
    write the regeneration script that document's Part 1.1 recommends) on
    any machine where `qwenpaid` stops being picked automatically.

---

## 10. Fusion Judgement Protocol (operator-settled 2026-07-17)

Judgement is DELEGATED, not done natively by the orchestrator. Acceptance of
any analysis, design, or non-trivial implementation diff requires a Fusion
Lite panel verdict of UNANIMOUS PASS. Anything less: re-iterate -- fix or
re-dispatch with the panel's dissent inlined, then re-judge.

1. **Panel:** 3 models, 70B+ class, different vendors. Proven set:
   `qwen/qwen3-235b-a22b-2507,deepseek/deepseek-chat-v3.1,meta-llama/llama-3.3-70b-instruct`,
   judge `qwen/qwen3-235b-a22b-2507`. Never 8B-class for design judgement.
2. **Command:**
   `python scripts/fusion_lite.py --prompt-file tmp/<item>.md --panel "<3>" --judge "<j>" --max-tokens 1000 --max-cost 0.05 --out tmp/<item>-verdict.md`
   (--max-tokens >=800 for design questions; 500 truncates 70B panelists).
3. **Unanimity rule:** every panelist must independently endorse AND the
   judge synthesis must record no unresolved dissent. One dissent = re-iterate.
4. **What gets judged:** pre-implementation analysis (PASS before writing the
   implementation packet), implementation diffs on gated paths (this is the
   adversarial-review gate for core/src/{crypto,transport,routing,privacy}/
   when the panel is given an adversarial prompt: probe for races, desync,
   downgrade, framing-compat, DoS), and any acceptance the orchestrator is
   unsure about. Compile-only verify is still NOT completion (Section 9.1).
5. **Record** every judgement in the ledger (`--record --lake openrouter
   --model fusion-panel-3x70b`) and cite the verdict file in the commit.
