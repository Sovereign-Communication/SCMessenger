# SCMessenger Orchestration Control Plane

Status: Active
Last updated: 2026-10-18
Protocol version: 2.0.0

This is SCMessenger's canonical human-readable orchestration authority. Its
machine-readable counterpart is `orchestration/manifest.yaml`; both versions
must match. `scripts/orchestrate_strict.py` is the common deterministic kernel.
No frontend command or model profile may redefine this protocol.

## Authority and boundaries

The controller coordinates workflow. It may inspect state, classify work,
create scoped packets, dispatch fresh workers, collect evidence, trigger
serialized verification, route independent review, integrate verified
worker-produced changes, update durable orchestration/HANDOFF state, and report
to the operator. It may not author application source, tests as implementation,
generated source patches, compile fixes, or architecture decisions. There is no
small-fix exception. A failed worker always causes re-brief, re-dispatch,
review, planner routing, or operator escalation.

The operator retains product, release, API-break, architecture, and material
security/privacy authority. A controller routes those decisions; it never
answers them on the operator's behalf.

## Universal roles

| Role | Purpose | May not become |
| --- | --- | --- |
| CONTROLLER | Durable coordination, evidence collection, routing, integration | An implementer or substantive decision-maker |
| SCANNER | Fresh read-only factual investigation | A planner or writer |
| EVIDENCE | Prescribed test/log/artifact evidence collection | A requirement interpreter |
| IMPLEMENTER / PLATFORM_IMPLEMENTER | Isolated, approved-scope implementation | An architect or release authority |
| PLANNER | Fresh technical design/replanning proposal | Product owner |
| VALIDATOR | Independent approved-requirement validation | Silent architecture rewriter |
| CRITICAL_VALIDATOR | Independent security/protocol/delivery review | Implementer |
| SECOND_OPINION | Resolve meaningful technical disagreement | Product owner |
| RELEASE_GATEKEEPER | Evidence-based release gate | Persistent controller |
| OPERATOR | Human authority for consequential choices | Delegated automation |

Every substantive investigation and implementation uses a fresh scoped worker.
Writer workers are isolated by default. The canonical packet contains: role,
task ID, objective, approved requirements, target files/line ranges, interfaces,
acceptance criteria, gates, writable scope, isolation ID, output schema, retry
evidence, stop conditions, and escalation conditions.

## Durable lifecycle

State is stored under `tmp/orchestration/state/<task-id>.json`, not chat
history. Valid transitions are defined in the manifest:

`INTAKE -> CLASSIFIED -> PACKET_READY -> DISPATCHED -> WORKER_DONE -> VERIFY -> REVIEW -> INTEGRATE -> COMPLETE`

Non-happy states are `BLOCKED`, `RETRY`, `REVIEW_REQUIRED`, `PLAN_REQUIRED`,
`OPERATOR_REQUIRED`, and `FAILED`. A fresh controller resumes by reading this
state, the queue/HANDOFF state, and evidence references. `UNKNOWN`, missing, or
malformed worker data is never success.

A resumed non-terminal record retains its original task snapshot, base SHA,
provider, and model; it is never reinitialized from a later queue read.
`REVIEW_REQUIRED` remains blocked until a controller-recorded reviewer
assignment and its complete structured footer are both present. The assignment
must bind the task, required independent role, distinct reviewer isolation ID,
provider/model/reasoning and dispatch reference, plus the expected writer patch
SHA-256 and base SHA. A provider that is unavailable is retained explicitly as
`UNAVAILABLE`, never as accepted review evidence. Record the dispatch artifact
with `--record-review-assignment TASK_ID=PATH`, then its result with
`--record-review-evidence TASK_ID=PATH`. A hand-written role-labelled footer,
or a footer without the dispatched assignment ID, cannot advance integration.
The kernel then advances eligible work to `INTEGRATE`; `--complete-integration
TASK_ID` runs the authoritative gate under the repository-wide build lock before
recording `COMPLETE`.

## Worker result contract

Workers append this metadata footer. Values must be complete and match the
packet; malformed metadata creates `RETRY`, `PLAN_REQUIRED`, or an escalation.

```text
---ORCHESTRATION_METADATA---
RESULT: DONE | BLOCKED | FAILED
ROLE: IMPLEMENTER
TASK: <task-id>
FILES: ["relative/path"]
VERIFICATION: NONE | CONTAINER(<actual command>)
SPEC_STATUS: SATISFIED | NOT_SATISFIED | AMBIGUOUS
ESCALATION: NONE | PLANNER | VALIDATOR | CRITICAL_VALIDATOR | SECOND_OPINION | OPERATOR
NOTES: ["concise evidence or blocker"]
---END---
```

Review-capable workers additionally return `ASSIGNMENT_ID: <durable reviewer
assignment id>`. The field mechanically binds their result to the independently
dispatched assignment; it is not a substitute for provider execution evidence.

`RESULT: DONE` is only eligible for review when `SPEC_STATUS: SATISFIED`,
`ESCALATION: NONE`, the role/task/files match, and an isolated non-zero diff is
in the packet's scope. A worker request that the controller repair code is a
re-dispatch event, never permission to edit.

## Gates, review, and disagreement

Worker-local checks are advisory. Authoritative integration verification runs
once through `scripts/build_lock.py`; workers never own completion. Any change
under `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, or
`core/src/privacy/` requires a fresh CRITICAL_VALIDATOR before integration.
Delivery-sensitive changes (outbox, receipts, custody, retry) require the
manifest-defined independent delivery review. Review outstanding means no
integration or commit.

The build lock is anchored at the repository root (`tmp/.build.lock`), so a
controller checkout and every detached writer worktree contend for the same
verification resource.

Route disagreement mechanically:

`implementation defect -> IMPLEMENTER repair -> fresh VALIDATOR`

`missing direction or plan defect -> PLANNER`

`security/protocol/delivery ambiguity -> CRITICAL_VALIDATOR`

`planner and critical-validator disagreement -> SECOND_OPINION -> OPERATOR if unresolved`

## Isolation and integration

`scripts/orchestration_worktree.py` creates a uniquely named detached writer
tree under `tmp/orchestration/worktrees/` from a recorded base SHA. A worker
cannot mutate the shared controller checkout before review. The controller may
apply an already verified worker-produced patch only after guard checks and
required review; source conflict resolution requiring authorship is dispatched
to a new implementer.

`scripts/orchestrator_guard.py` is the queryable policy layer. It answers role
write permission, writer-isolation requirement, review requirement, lifecycle
transition, and integration permission from the manifest. Controller writes are
limited to orchestration/HANDOFF state and packets, never application source.
An IMPLEMENTER or PLATFORM_IMPLEMENTER must also supply its packet's exact
`--packet-files` scope; a broad role grant never authorizes another path.

## Adapter compatibility

All active adapters load the same manifest and invoke the same kernel. Their
native subagent support changes isolation transport only, not semantic authority.

The manifest names `SHIP_PLAN.md` as the execution-pointer document. With no
`--queue` override, `orchestrate_strict.py --dry-run` resolves its authoritative
pointer and previews every car in that release-train document. The release-train
Markdown is a coordination plan, not a worker packet queue; dispatch therefore
requires an explicit JSONL worker queue via `--queue`.

| Frontend / entrypoint | Current behavior | Canonical protocol used | Native isolation/subagents | Role/model mapping | Source-edit escape hatch | Migration |
| --- | --- | --- | --- | --- | --- | --- |
| Claude | Command bootstrap | v2 manifest + strict kernel | Native agents when available | Capability mapping | No | Thin adapter |
| Codex | Project agent profiles | v2 manifest + strict kernel | Worktrees/agent contexts | GPT-5.6 capability mapping | No | Thin profiles |
| Qwen | Command bootstrap | v2 manifest + strict kernel | Qwen workers or worktrees | Provider capability mapping | No | Thin adapter |
| Gemini | Bootstrap document | v2 manifest + strict kernel | External workers/worktrees | Provider capability mapping | No | Thin adapter |
| Bob | Skill bootstrap | v2 manifest + strict kernel | Bob workers/worktrees | Provider capability mapping | No | Thin adapter |
| OpenCode | Agent bootstrap | v2 manifest + strict kernel | Native subagents/worktrees | Provider capability mapping | No | Thin adapter |
| `.agents` | Portable skill | v2 manifest + strict kernel | Repo worktree fallback | Semantic role only | No | New portable bootstrap |
| Direct scripts | Kernel CLI | v2 manifest | Repo worktree manager | Explicit provider provenance | No | Hardened kernel |

Model/provider availability may select a different backend, but never transfers
Planner, Validator, or Operator authority to the Controller. Codex uses the
GPT-5.6 Luna/Terra/Sol role preferences when the installation supports them;
the adapter must select the declared fallback rather than invent an authority.

## Required controller workflow

1. Validate `orchestration/manifest.yaml` with `scripts/orchestration_contract.py`.
2. Read durable non-terminal state and claim/classify one task.
3. Create a minimal packet and an isolated writer worktree.
4. Dispatch a fresh role-appropriate worker and record actual provider/model.
5. Fail closed on timeout, malformed report, `UNKNOWN`, zero diff, or
   out-of-scope edit; create a retry/review/planning route instead of repairing.
6. Run the serialized authoritative gate and independent review when required.
7. Integrate only worker-produced, reviewed output; then mark COMPLETE and
   record provenance. Never claim a worker's self-report is sufficient.

## Live-provider validation status

The deterministic suite verifies ledger shape, isolation separation, patch
binding, and rejection of local footers. It does not prove that an installed
provider actually dispatched and returned an independently isolated reviewer
result. That live-provider validation remains an open release blocker; no task
or draft PR status may be marked complete from the local fixture alone.

Historical orchestration records remain historical evidence. This document and
the manifest supersede contradictory active controller-fix and controller-gatekeeper
instructions in older governance surfaces.
