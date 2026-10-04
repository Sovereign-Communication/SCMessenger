# Orchestration Token Strategy: The Delegation Dial

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active -- consolidates and supersedes ORCHESTRATOR_TOKEN_AUDIT_AND_REDESIGN.md,
ORCHESTRATION_TOKEN_REDUCTION_PLAN.md, and ORCHESTRATION_IMPLEMENTATION_GUIDE.md
(all three kept for history, banners added, do not use them for reference).
Last updated: 2026-08-03.
Author: Claude (Cowork, REMOTE SANDBOX capability class per AGENTS.md -- see Part 8).

This is the third and final pass on this work. The first pass produced an
audit and two untested scripts. The second pass reorganized the same
untested ideas into a tighter document. Neither was run against the real
repo. This pass does four things the first two did not: reads the actual
routing code instead of only the docs describing it, runs every new script
against real or realistically-isolated test data before calling it done,
fixes two concrete bugs those tests found, and separates what was measured
from what was estimated from what is merely projected.

---

## Part 0. What problem this solves

The orchestrator role (whichever model is playing it -- Claude, or any
other model per ORCHESTRATION.md's model-agnostic design) spends tokens on
four things that do not require judgment and can be made close to free:
constructing dispatch prompts from a template, parsing a worker's response
to find out what happened, moving HANDOFF files and committing, and
deciding which lake/model/effort-level a task should get. The first three
are mechanical. The fourth needs to be principled but is still a small,
deterministic decision once the inputs are known.

The redesign in this document turns those four things into three small,
independently-tested scripts plus one policy function, so the orchestrator
either doesn't do them at all (they run as subprocess calls) or does them
in a handful of tokens (reading a JSON result). It does not change what
gets built, how workers implement things, or the security/verification
bar -- ORCHESTRATION.md's loop, Operating Contract, and Section 4 security
gates are unchanged and still authoritative. This document is about the
coordinator's own token consumption, nothing else.

---

## Part 1. Ground truth found during this pass (read before trusting anything below)

Everything in this part was confirmed by reading the actual code/config in
the repo, not inferred from what the docs say the code does. Three of the
five findings were inconsistencies between documentation and reality that
the first two passes missed because they reasoned from ORCHESTRATION.md's
prose instead of opening the scripts it references.

| # | Finding | Where | Status |
|---|---|---|---|
| 1 | `scripts/lake_route.py` already implements quota-aware, cooldown-aware, round-robin tier routing. It is fully working code with real entries already in `tmp/lakes/ledger.jsonl`. The first-pass drafts reimplemented a worse version of this inline instead of calling it. | `scripts/lake_route.py` | Now used, not duplicated (Part 3). |
| 2 | `qwenpaid` -- the operator's stated PRIMARY lane for all non-FLASH dispatches since 2026-07-28 -- was entirely absent from `tmp/lakes/registry.json` and `lake_route.py`'s `TIER_LADDERS`. Every dispatch that used `--provider qwenpaid` directly (as the V0.4.0 session did) worked, but nothing routed through `lake_route.py` would ever have picked it automatically. | `tmp/lakes/registry.json`, `scripts/lake_route.py` | Fixed and tested (Part 1.1). |
| 3 | Two different, incompatible worker report contracts exist today: `ORCHESTRATION.md` Section 3 (`RESULT: DONE` / `PATCH: <n>` / `VERDICT: PASS\|FAIL\|...`) and `AGENTS.md`'s REMOTE SANDBOX / FOREIGN WORKER format (`RESULT: DONE\|BLOCKED\|FAILED` / `VERIFICATION:` / `FILES:` / `NOTES:`). AGENTS.md states plainly it is "the canonical, model-agnostic rules contract for ANY agent" -- so it should win, and any new machine-parseable contract should extend it rather than add a third vocabulary. | `docs/ORCHESTRATION.md` Section 3 vs. `AGENTS.md` lines 81-118 | Footer contract in Part 2 uses AGENTS.md's field names; Section 3 of ORCHESTRATION.md still needs a human edit to point at this doc (Part 6, Day 1 checklist -- not done automatically here since ORCHESTRATION.md is a heavily cross-referenced document better edited deliberately). |
| 4 | `delegate_task.py --tier` (lowercase `thinking\|max\|standard\|plus\|flash`) only resolves for `provider == "qwen"`, and three of its five values (`standard`, `thinking`, `flash`) construct model name strings (`qwen-standard`, `qwen-thinking`, `qwen-flash`) that do not appear anywhere in the script's own `MODEL_TOKEN_LIMITS` table. It is very likely broken for those three values and was never exercised by any of the sessions read for this audit -- every real dispatch found in session history used an explicit `--model`. | `scripts/delegate_task.py` `_resolve_max_tokens`, tier-to-model logic in `main()` | Not fixed (would need a live API key to verify the actual DashScope model names, which is outside what this session can test). Documented as a trap: never use `--tier`; the dial always emits an explicit `--model` sourced from `lake_route.py`. |
| 5 | `tmp/lakes/registry.json`'s own `tier_ladders` JSON key is dead data -- `lake_route.py`'s `route()` function reads a separate, hardcoded module-level `TIER_LADDERS` dict and never looks at the registry's copy. | `scripts/lake_route.py` line ~50 vs `registry.json` | Both copies were updated for consistency (Part 1.1) but be aware if a future change touches only one of them, the hardcoded one is the one that actually executes. |

### 1.1 The qwenpaid fix, specifically

Added a `qwenpaid` block to `tmp/lakes/registry.json` (same schema as the
existing lakes, model `qwen3.8-max-preview` under `CODER`/`THINK`/`MAX` --
it is a single thinking-hybrid model serving all three roles, the same
one-model-multiple-tiers pattern the registry already uses for `ollama`).
Prepended `qwenpaid` to `lake_route.py`'s `TIER_LADDERS` for `CODER`,
`THINK`, and `MAX` only (not `FLASH` -- the paid budget is reserved for
real work, and qwenpaid carries no FLASH-tier model; not `MORPH`, which is
OpenRouter-specific).

Verified in an isolated scratch copy of the registry (never the live
`tmp/lakes/round_robin_state.json` or `ledger.jsonl`):

- No `qwenpaid` key anywhere -> `CODER`/`FLASH` cascade correctly to the
  next available lake (all the way to `ollama`, which needs no key);
  `MAX` correctly reports "no lake available" since its ladder has no
  keyless fallback. This is pre-existing, correct, unchanged behavior.
- `QWEN_PAID_API_KEY` set -> `CODER`, `THINK`, and `MAX` all now correctly
  return `qwenpaid qwen3.8-max-preview` first.
- `FLASH` with the same key set -> unaffected, still routes to the next
  available free lane. Confirms the fix is properly scoped.

**Persistence warning found during the final sanity pass, not before:**
`tmp/` is gitignored (`.gitignore` lines 6 and 94; confirmed with
`git check-ignore`). `scripts/lake_route.py`'s `TIER_LADDERS` fix is a
normal tracked-file change and will commit and persist normally. The
`tmp/lakes/registry.json` half of the fix will NOT -- it is local to
whatever checkout has it, and a fresh clone, a CI runner, or a different
machine will not have the `qwenpaid` block added here unless someone adds
it again. This is not a new problem this session created: `registry.json`
carries its own header (`"source": "SCM_UNIFIED_LAKE_ORCHESTRATION.md"`)
and `ORCHESTRATION.md`'s own state-files table (Section 2) already
describes it as a "seed from `docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md`"
-- i.e. it was always meant to be a regenerated/reseeded snapshot, not a
hand-maintained tracked file, and no regeneration script for it was found
in this repo (searched; only readers exist, `lake_route.py` and
`dispatch_dial.py`). The markdown source of truth
(`docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md` Section 1) already
had the correct `qwenpaid` JSON block before this session started -- only
the generated snapshot had drifted out of sync with it.

Practical consequence: `lake_route.py`'s `route()` silently `continue`s
past any ladder entry with no matching block in `registry.json`'s `lakes`
dict (confirmed by reading the exact code path) -- so on a machine without
this session's local `registry.json` edit, the `TIER_LADDERS` change is
inert (harmless, but does nothing) rather than broken. Whoever adopts this
fix for real should either write a small `regenerate_registry.py` that
derives `tmp/lakes/registry.json` from the markdown doc's JSON block (the
durable fix -- makes this reproducible on any checkout), or, at minimum,
manually re-apply the same `qwenpaid` block to `tmp/lakes/registry.json`
on every machine that runs `lake_route.py`. Filed here rather than quietly
left for someone to rediscover.

---

## Part 2. The worker response contract (extends AGENTS.md, does not replace it)

A worker dispatched through `delegate_task.py` gets a footer requirement
appended to its prompt. The field names and RESULT/VERIFICATION vocabulary
are copied verbatim from AGENTS.md's existing REMOTE SANDBOX/FOREIGN WORKER
format so a worker (or a human) who already knows AGENTS.md does not have
to learn anything new:

```
---ORCHESTRATION_METADATA---
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(<what ran, exact commands>)
FILES: ["core/src/transport/swarm.rs", "core/src/audit.rs"]
NOTES: ["line 42: added observability_event() call", "48 tests pass"]
---END---
```

The delimiter is the one real addition beyond AGENTS.md, and it exists for
a structural reason, not a stylistic one: AGENTS.md's REMOTE/FOREIGN format
assumes the report IS the entire message. `delegate_task.py` workers return
a diff or full file as their MAIN payload (parsed separately by
`extract_diff_blocks`/`extract_file_blocks`), so the footer is a supplement
appended at the end and needs a marker to be found reliably inside a much
longer response.

`FILES`/`NOTES` accept either a JSON list (preferred -- ask for it in the
dispatch prompt) or bare AGENTS.md-style free text
(`FILES: a.rs, b.rs`) -- both parse into the same structured output, so a
worker that only knows AGENTS.md's plain-text convention still produces
something the parser can use. This was a deliberate design change from the
first-pass draft, which invented `TOUCHED_FILES`/`EVIDENCE` field names
that matched nothing already in the repo; renamed after finding AGENTS.md's
existing contract during this pass's ground-truth check (Part 1, finding 3).

A missing or unparseable footer is never treated as success. The parser
returns `degraded: true` and `result: "UNKNOWN"`, and the caller is
expected to fall back to `delegate_task.py`'s existing diff-block
extraction rather than assume anything worked.

---

## Part 3. The delegation dial: precise model/effort/scope per task

Three tools, one job each, none of them duplicating another:

```
dispatch_dial.py     WHAT effort level and scope does this task need
                      (reads: queue tier, target files, description, retry count)
                      -- pure logic, no network calls, no ledger writes

lake_route.py         WHICH lake/model currently has quota for that tier
                      (existing, unmodified in its decision logic -- see
                      Part 1.1 for the one data fix applied to it)
                      -- called BY dispatch_dial.py as a subprocess, never
                         reimplemented

delegate_task.py      HOW the API call itself is made, including the new
                      scoped-file-targeting capability (Part 4)
```

`dispatch_dial.py`'s decision rules, mirrored exactly from
`ORCHESTRATION.md` Section 4 so the two documents can be diffed against
each other rather than drifting apart:

- Any target file under `core/src/{crypto,transport,routing,privacy}/`
  forces the effective tier up to at least THINK (Section 4 row 1 already
  mandates adversarial review at THINK+ before commit -- this makes the
  first-pass implementation meet that bar rather than under-shoot it and
  cost a round trip) and sets `security_gate_required: true` so the caller
  knows a separate adversarial dispatch is still owed regardless of which
  tier implemented the change.
- A description or file path containing `outbox`, `receipt`, `custody`, or
  `retry` (Section 4 row 2, "WS-A delivery logic") sets
  `delivery_gate_required: true`. This does not change tier by itself.
- Two or more prior failures (`--retry-count >= 2`) escalates the tier one
  step, matching `ORCHESTRATION.md` Section 2.2 step 6 ("Two failures ->
  escalate tier"). Escalations compose: a CODER task that is both
  gated and on its third attempt goes CODER -> THINK (gate) -> MAX
  (retry), and this composition was specifically tested (Part 5).
- Tier never escalates past MAX (clamped).
- `thinking` is `true` iff the effective tier is THINK or MAX.
- Retry budget (`max_rounds`) is 2 for ungated FLASH work (fail fast on
  mechanical tasks) and 3 otherwise (the existing default).
- The queue's own `tier` field is always the starting point and is never
  silently downgraded -- a "this looks mechanical" heuristic exists but
  only ever adds an advisory note, never lowers an explicitly-assigned
  tier.

Division of responsibility is strict: `dispatch_dial.py` never makes a
network call and never touches the ledger, so calling it costs the
orchestrator close to nothing. `lake_route.py` owns all quota/cooldown
state and is the only thing that reads `ledger.jsonl` for routing
purposes. Neither one executes the dispatch; that is still
`delegate_task.py`'s job, unchanged except for Part 4.

---

## Part 4. Scope per dispatch: delegate_task.py patch

`--files` now accepts `path:Lstart-Lend` (1-indexed, inclusive) alongside
plain paths, which is unchanged. This is additive: `parse_scoped_files()`
strips the range suffix into a side table before anything else in the
script sees `args.files`, so the write-allowlist, chunk-token estimation,
and retry-prompt logic all continue to operate on real paths exactly as
before and require no other changes. `_format_scoped_file()` builds the
prompt block, slicing to the requested range when one exists and falling
back to the full file (never to nothing) if the range is missing, invalid,
or past end-of-file.

This is what lets the orchestrator's existing pre-dispatch validation step
(`ORCHESTRATION.md` Section 2.2 step 2: grep the target before dispatching)
directly narrow what gets sent, instead of validating with a grep and then
sending the whole file anyway. A task that only needs to touch 40 lines of
a 700-line file can now be dispatched with `--files
core/src/transport/swarm.rs:40-90` and the model never sees the other 650
lines.

Proven backward-compatible by direct comparison, not by inspection: for
any path without a `:Lstart-Lend` suffix, `_format_scoped_file()`'s output
was asserted byte-identical to the old `f.read()`-based format in a unit
test. All four call sites in the script (main dispatch, and both the
diff-mode and full-mode retry prompts) now go through the same function,
where previously each had its own copy of the same read-and-format logic.

---

## Part 5. Verification evidence (what was actually run, not just claimed)

Every script below was executed in this session, either directly against
real repo data (read-only) or inside a fully isolated scratch git
repository under `/tmp` (never the mounted SCMessenger repo's actual git
history). None of the testing below moved a real `HANDOFF/` file, wrote to
the real `tmp/lakes/ledger.jsonl`, or committed to the real repo.

**`parse_orchestration_footer.py`** -- 5 cases: well-formed JSON footer,
AGENTS.md-bare-style free text (`FILES: a.rs, b.rs`, no brackets/quotes at
all), malformed near-JSON (single quotes, trailing commas), a response
with no footer at all, and a `BLOCKED` result with an empty `FILES` list.
All five produced exactly the expected structured output and exit code
(0=DONE, 1=BLOCKED/FAILED, 2=UNKNOWN/degraded).

**`batch_handoff.py`** -- built a scratch git repo with four task fixtures
covering every branch: a normal move, an already-at-destination no-op, a
missing source, an invalid destination string, and a requeue. Dry-run
first (confirmed zero filesystem changes), then a real run (confirmed
exactly one commit for the whole batch, not one per task), then an
identical re-run to prove idempotency. The re-run surfaced a real bug: the
original "nothing to commit" detection string-matched git's message, but
git phrases that message differently ("nothing added to commit but
untracked files present") when unrelated untracked files exist elsewhere
in the repo -- true in this sandbox because of ordinary scratch files.
Fixed by replacing the string match with `git diff --cached --quiet`,
which checks the actual staged state instead of parsing git's variable
wording, and re-verified the fix against the exact scenario that broke it.
Also verified full integration with the real `lake_route.py --record`
(ledger entry correctly written to the scratch directory; confirmed the
real repo's `tmp/lakes/ledger.jsonl` line count was unchanged afterward).

**`dispatch_dial.py`** -- 7 scenarios built from real queue/task data, not
synthetic examples: `A-01` (real CODER task, Android retry suppression),
`A-03` (real THINK task, MeshStore persistence), `PQC-09`'s actual target
file `core/src/privacy/onion.rs` (confirms the security-gate escalation
fires on a real gated path, not a hypothetical one), a delivery-logic
task, a mechanical FLASH task (confirms the fast-fail `max_rounds=2`),
a double-escalation case (gate then retry-count, confirming they compose
in the right order), and a MAX-tier task with a high retry count
(confirms the tier ceiling clamps rather than erroring).

**`build_lock.py`** -- 7 scenarios including one genuine concurrency test
(a backgrounded 3-second "build" racing against a foreground acquire
attempt 0.5 seconds later, which correctly observed BUSY) and a
poll-until-free mode. Testing found a second real bug: the initial design
authorized `--release` by matching the process ID recorded at `--acquire`
time, which cannot work for the script's primary intended use (acquire and
release as two separate CLI invocations from a shell script are two
different processes by construction). Fixed by switching ownership to the
`--holder` name instead, keeping pid as diagnostic-only metadata, and
re-verified with an explicit wrong-holder-cannot-release test.

**`delegate_task.py` (scoped targeting)** -- full-file syntax check
(`py_compile`), an end-to-end argv-to-parsed-scope test, `--help`
regression check, and direct unit tests of the new functions against a
real 20-line fixture file: exact line-range extraction with no bleed into
adjacent lines, the note text present, out-of-range and missing-file
fallback behavior, and -- the load-bearing guarantee -- byte-identical
output to the pre-patch format for any unscoped path.

**`orchestrate_strict.py`** -- rewritten during this pass to compose the
tested pieces above instead of the first draft's inline reimplementation
of routing and response-parsing. `py_compile` clean; a `--dry-run` pass
against the real `scm_v1_farm_queue.jsonl` correctly read the queue,
evaluated real dependency state against the real `HANDOFF/done/` tree,
called the (now-fixed) real `lake_route.py` for each candidate task, and
correctly reported that no dispatch-prompt files exist yet for those
specific IDs rather than fabricating one -- dispatched nothing, wrote
nothing.

**Mechanical hygiene** -- every file touched or created in this pass (13
files: 4 documents, 7 scripts, 1 registry, 1 router) was run through the
repo's own `scripts/rules_check.py` (the actual pre-commit gate, not a
approximation of it). Final result: exit 0, zero violations. This caught
a real, pre-existing issue: all four first-pass documents used a checkmark
character (U+2705) that violates AGENTS.md rule 1; fixed by stripping it
and, since the rule requires stripping emoji from any file being edited
anyway, using the same edit to add supersession banners.

**What was not run:** no live API dispatch (would require real keys this
sandbox does not have and would spend real money/quota to test
infrastructure code); no `cargo`/`gradlew` build gate (out of scope -- see
Part 8, nothing in this change set touches Rust/Kotlin/Swift source); no
test of `orchestrate_strict.py`'s non-dry-run path against a real worker
response end-to-end (would require the live dispatch above).

---

## Part 6. Token accounting -- measured, modeled, and projected, kept separate

**Measured (from session transcripts, not estimated):** the V0.4.0 batch
(5 tasks: P1 backoff, P2 outbox, P3 receipt, P4 receipt unification, P6
FFI) reported by that session itself as "~3,700 LOC, 48 test cases, 409K
tokens, $0.012 cost" total. That figure is the ONLY hard number available
from real usage; it is a whole-session total, not broken down by
orchestrator-vs-worker, so it cannot by itself prove or disprove the
per-task overhead claims below.

**Modeled (reasoned estimate of typical prompt/response sizes, not
measured token-by-token):** a per-task orchestrator overhead breakdown of
roughly 750-950 tokens under the OLD pattern (read queue, construct a
prompt from scratch, grep a response for keywords, move a HANDOFF file
with a separate Edit + commit per task), versus roughly 70-150 tokens
under the NEW pattern (one `dispatch_dial.py` call, one
`parse_orchestration_footer.py` call, one shared batch commit across N
tasks instead of N commits). This is a defensible model given what each
step actually does, but it was not instrumented with real token counts in
either pattern -- treat the "~85-90% reduction in orchestrator-specific
overhead" figure as the size of the effect this design targets, not a
verified measurement.

**Projected (depends on adoption, not yet observed):** for a 50-task
sprint, modeled orchestrator overhead drops from an estimated ~40-48K
tokens to an estimated ~4-8K tokens. Worker-side token cost (the P1-P6
style implementation work itself) is unchanged by anything in this
document -- this redesign only touches the coordinator's own consumption,
never the worker's.

The honest summary: this pass fixed two real, tested bugs and closed one
real, tested gap (qwenpaid missing from the router) in infrastructure that
already existed and already worked for its narrow original purpose. The
token-savings figures are a reasoned projection based on removing
duplicated/manual steps, not a before/after measurement, because doing a
real before/after measurement requires running the same batch twice
against live workers under both patterns, which this session did not have
the API access to do. Part 9's rollout plan includes that measurement as
an explicit next step rather than assuming the projection is correct.

---

## Part 7. Risk register (issues considered and how each is handled)

| Risk | Handling |
|---|---|
| Worker hallucinates the footer format, wraps it in a code fence, uses single quotes, or omits it entirely | Parser strips fences, falls back through JSON -> quoted-item scrape -> bare comma-split, and treats a missing/unparseable footer as `degraded: true` / `UNKNOWN`, never as success. Tested against all of these cases directly (Part 5). |
| A worker's `RESULT` value is garbage (e.g. `MAYBE_KINDA`) despite the footer otherwise parsing fine | Fails closed to `UNKNOWN` even though `degraded` is `false` (structure was found, but the one field that gates "did this succeed" was invalid) -- tested explicitly. |
| Two verification runs (e.g. two batched tasks' gates) execute concurrently on Windows, risking the rlib-lock corruption `build.md` and `ORCHESTRATION.md` Section 9.5 both warn about | `build_lock.py`: advisory lockfile, tested under real concurrency (backgrounded process + racing foreground acquire), with a wait-and-retry mode for a batch orchestrator that would rather queue than fail. |
| A crashed or killed process leaves the build lock held forever | Stale-lock detection (default 30 minutes) force-acquires with a loud warning, never silently. |
| `batch_handoff.py` run twice on the same batch (e.g. a retry after a partial failure) | Idempotent by design: already-at-destination is a skip, not an error; re-running an already-committed batch correctly reports nothing to commit. Tested by literally re-running the same batch file twice. |
| The orchestrator's own routing duplicates or drifts from `lake_route.py`'s quota/cooldown logic | Eliminated by construction: `dispatch_dial.py` calls `lake_route.py` as a subprocess and contains zero quota/cooldown logic of its own. |
| Security-relevant diffs slip through at a tier too low for the review they'll need, wasting a round trip | `requires_security_gate()` floors the tier at THINK for any of the four gated path prefixes, independent of what the queue originally said, tested against a real gated file (`core/src/privacy/onion.rs`, PQC-09's actual target). |
| Scoped file targeting sends a model an incomplete/misleading excerpt | Falls back to the FULL file (never to nothing, never to silence) whenever the requested range is invalid, out-of-bounds, or the file can't be read -- a broken scope costs the tokens scoping was meant to save, it never costs correctness. Tested directly. |
| This session claims a build/test gate passed when it didn't run one that's actually authoritative | Did not happen: nothing in this change set touches Rust/Kotlin/Swift/WASM source (Part 8), so no cargo/gradlew gate applies; the applicable gate (`rules_check.py`) was actually run, not assumed. |
| This session, running as REMOTE SANDBOX class, oversteps its authority (moves real HANDOFF files, commits, or claims container-green as authoritative) | Did not happen -- see Part 8 for the explicit self-check. |
| The footer/dial design quietly reinvents a third worker contract instead of extending AGENTS.md | Checked directly against AGENTS.md's text (Part 1 finding 3, Part 2) and field names changed to match once the divergence was found. |
| A stale `registry.json`/`lake_route.py` edit breaks existing dispatches that don't use `qwenpaid` | The FLASH ladder (the only one most mechanical dispatches use) is untouched; CODER/THINK/MAX changes are additive (new lake prepended, existing lakes and their order unchanged) and the no-key fallback path was re-verified to behave exactly as before. |
| The `tmp/lakes/registry.json` half of the qwenpaid fix is in a gitignored path and will not survive a fresh checkout, silently making the (committed) `lake_route.py` ladder change inert rather than broken elsewhere | Found and documented in Part 1.1's persistence warning, with the exact code path confirming "inert, not broken" and a concrete recommendation (a small regen script deriving the JSON from the already-correct markdown source) rather than assuming the one-time edit is durable. |

---

## Part 8. Capability class self-check (AGENTS.md)

This session is a Claude Cowork sandbox, which AGENTS.md classifies as
REMOTE SANDBOX: "container-green cargo check/clippy/fmt/test is USEFUL
ADVISORY SIGNAL but never authoritative... Do NOT move HANDOFF task files
to done/. Do NOT update _QUEUE.md statuses. Do NOT claim any gate passed
unless you name the environment it ran in."

What this session actually did, checked against those rules:

- Did not move any real `HANDOFF/` file, anywhere. Every `batch_handoff.py`
  move tested in Part 5 ran inside `/tmp/handoff_scratch_test*`, an
  isolated scratch git repository created solely for that test, never the
  mounted SCMessenger repo.
- Did not commit to the real repo's git history. The registry/router/
  delegate_task.py edits are working-tree changes only.
- Did not claim any build gate passed. The applicable, actually-run gate
  (`scripts/rules_check.py`) is named explicitly everywhere it's cited in
  this document, and no Rust/Kotlin/Swift file was touched, so no
  cargo/gradlew claim was made or implied.
- Did edit tracked source files (`scripts/*.py`) and one gitignored data
  file (`tmp/lakes/registry.json`) directly rather than only proposing a
  patch. This is within what a REMOTE SANDBOX session can do per the
  "Best-fit work" list (mechanical refactors with clear acceptance
  criteria, pre-dispatch validation-style sweeps). The two are not
  equivalent, though: the `scripts/*.py` changes are normal uncommitted
  working-tree edits and should be reviewed and committed from the Windows
  host like any other change. The `registry.json` edit is NOT committable
  by anyone through normal git -- it is gitignored (Part 1.1) -- so
  "review and commit" does not apply to it; what applies is the
  regenerate-or-manually-reapply action Part 1.1 and Part 9 step 2
  describe.

Practical consequence for the operator: everything in this document
describes verified BEHAVIOR of the new/changed scripts (they were actually
run and their output actually checked), not a verified BUILD (nothing
required one) and not a committed change (nothing has been committed).

---

## Part 9. Rollout plan

**Now (this session's output):** working-tree changes only, described
above. Nothing committed, nothing pushed, no real HANDOFF state touched.

**Integration status (updated same session, after the operator chose "wire
it into /orchestrate now"):** `docs/ORCHESTRATION.md` Section 0 rule 4,
Section 2.2 (the loop, now 9 steps instead of 10), Section 3 (worker
contract), and Section 9 (lessons 5 and a new 16) have been edited directly
-- `/orchestrate` now instructs the orchestrator to use `dispatch_dial.py`,
`parse_orchestration_footer.py`, `batch_handoff.py`, and `build_lock.py`
rather than doing those steps by hand, the next time it is invoked. Every
existing safety rule (compile-only-is-not-completion, zero-diff-not-
trusted, one-build-at-a-time, mandatory security gates, workers-never-
commit) is unchanged, re-stated inline at each step that touches it, and
`rules_check.py` was re-run clean on the edited file. `.claude/commands/orchestrate.md`
was NOT editable from this session (protected location) -- non-blocking,
since it already instructs "read docs/ORCHESTRATION.md in full," which
covers the new Section 2.2/3 content regardless; only its own summary
bullet list (a minor emphasis aid, not load-bearing) is stale until someone
with write access adds a one-line pointer to Section 3.

**Day 1 (human or Windows-host Claude Code session):**
1. Review the diff on `docs/ORCHESTRATION.md`, `scripts/lake_route.py`, and
   `scripts/delegate_task.py` (the loop rewrite, the qwenpaid ladder fix,
   and the scoping patch -- all normal tracked files, all show in `git
   diff` normally).
2. Separately handle `tmp/lakes/registry.json`: it is gitignored (Part 1.1
   persistence warning) so it will NOT appear in `git status`/`git diff`
   at all. Confirm it has the `qwenpaid` block on whatever machine will
   actually run dispatches -- either write the small regeneration script
   recommended in Part 1.1, or manually copy the block from
   `docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md` Section 1 (it
   was already correct there before this session started).
3. Optionally add the one-line Section 3 pointer to
   `.claude/commands/orchestrate.md`'s "must internalise" bullet list
   (this session could not write to that path -- see integration status
   above).
4. Run `python scripts/rules_check.py --staged` after staging, as a second
   confirmation beyond this session's own run.
5. Since no Rust/Kotlin/Swift files changed, the `build.md` cargo/gradlew
   gates are not applicable to this change set -- confirm that's still
   true (`git diff --stat -- scripts/ HANDOFF/ docs/ORCHESTRATION.md`
   should show only `.py`/`.md` files; note plain `git diff --stat` in
   this sandbox also shows dozens of unrelated files as fully rewritten --
   that is a pre-existing CRLF/LF line-ending artifact of this sandbox's
   checkout, confirmed with `git diff -w`, not a real change and not
   something this session caused; scope any review to the files listed in
   Part 10) before skipping the build gates.
6. Commit.

**Day 2 (real validation, requires live API keys this session did not
have):** dispatch a real 5-task batch through `orchestrate_strict.py`
(not `--dry-run`) against `qwenpaid` or another live lake, with real
dispatch-prompt files pre-written under `tmp/tasks/`. This is the first
point at which the Part 6 token projections can become measurements
instead of a model -- capture actual before/after token counts if
possible (e.g. by running one batch the old way and one the new way
against comparably-sized tasks).

**Ongoing:** `dispatch_dial.py`, `parse_orchestration_footer.py`, and
`batch_handoff.py` are additive and backward-compatible -- nothing forces
their adoption. A task can still be dispatched the old way at any time;
the new path is faster for the coordinator, not mandatory.

---

## Part 10. File inventory

| File | State | Purpose |
|---|---|---|
| `scripts/dispatch_dial.py` | New, tested | Task properties -> tier/thinking/scope/gates decision |
| `scripts/parse_orchestration_footer.py` | New, tested, reconciled with AGENTS.md | Worker response -> structured result |
| `scripts/batch_handoff.py` | New, tested, one real bug fixed | Batch HANDOFF move + single commit + ledger delegation |
| `scripts/build_lock.py` | New, tested, one real bug fixed | Serialize verification runs (Windows rlib-lock safety) |
| `scripts/orchestrate_strict.py` | Rewritten this pass | Composition of the four pieces above into the pure-delegator loop |
| `scripts/supervisor.py` | Unchanged code, superseded-note added | First-pass script; batch_handoff.py + build_lock.py now cover its job with more testing |
| `scripts/lake_route.py` | Patched (qwenpaid added to 3 ladders) | Existing quota-aware router, unmodified in its core logic |
| `scripts/delegate_task.py` | Patched (scoped-file targeting, additive) | Existing dispatch script, unmodified for every pre-existing call pattern |
| `tmp/lakes/registry.json` | Patched (qwenpaid block added) | Lake registry, now matches documented operator policy |
| `docs/ORCHESTRATION.md` | Patched (Section 0/2.2/3/9 wired to the new scripts) | `/orchestrate`'s actual brain -- this is what makes the slash command itself behave differently, not just the scripts existing |
| `HANDOFF/ORCHESTRATOR_TOKEN_AUDIT_AND_REDESIGN.md` | Superseded, emoji fixed | First-pass audit, kept for history |
| `HANDOFF/ORCHESTRATION_TOKEN_REDUCTION_PLAN.md` | Superseded, emoji fixed | Second-pass plan, kept for history |
| `HANDOFF/ORCHESTRATION_IMPLEMENTATION_GUIDE.md` | Superseded, emoji fixed | Second-pass guide, kept for history |
| `HANDOFF/ORCHESTRATOR_AUDIT_EXECUTIVE_SUMMARY.md` | Superseded, emoji fixed | First-pass summary, kept for history |
| `HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md` | This document | Authoritative reference |

---

## Part 11. Quick reference: dispatching one task the new way

```bash
# 1. Dial: what effort level, thinking flag, and lake/model does this need?
python scripts/dispatch_dial.py \
  --tier CODER \
  --files core/src/store/outbox.rs \
  --description "Wire outbox flush on reconnect" \
  > tmp/tasks/P2.dial.json

# 2. Dispatch (lake/model/thinking come straight from the dial's output;
#    scope syntax available on --files if the pre-dispatch grep found a
#    narrow target, e.g. core/src/store/outbox.rs:40-90)
python scripts/delegate_task.py \
  --task tmp/tasks/P2.dispatch.md \
  --provider qwenpaid --model qwen3.8-max-preview \
  --files core/src/store/outbox.rs \
  --apply --verify "cargo check --workspace" --mode diff --max-rounds 3

# 3. Read the structured result instead of grepping the response
python scripts/parse_orchestration_footer.py tmp/P2_response.md

# 4. Batch move + one commit + ledger record (repeat step 1-3 for the rest
#    of the batch first, then call this once)
python scripts/batch_handoff.py \
  --batch-file tmp/batch.json --provider qwenpaid \
  --commit-message "P1-P5 backoff/outbox/receipt batch"
```

Or run the whole loop as one composed call:

```bash
python scripts/orchestrate_strict.py \
  --queue scm_v1_farm_queue.jsonl --max-tasks 5 --provider qwenpaid
```
