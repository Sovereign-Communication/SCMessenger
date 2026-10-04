# Plan: Hermes Memory System — "Elongated Dreaming" for Distilled Lessons Learned

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

> **Status:** Proposed
> **Date:** 2026-08-06
> **Author:** Hermes (acting orchestrator)
> **Verdict up front: Yes, Hermes is the right harness.** It already ships three of the four "dreaming" pillars; the gap is a *periodic* consolidation pass over the full historical record, which we build on top of existing primitives.

---

## 1. Verdict: Is Hermes the right harness?

**Yes — and I verified this against the actual installed source** (`C:\Users\SCM\AppData\Local\hermes\hermes-agent`), not docs alone. Hermes already has the machinery for "learning from experience." The user's instinct ("get all lessons learned distilled effectively") is exactly what its memory/skill loop is designed for.

### What Hermes already ships (verified in source)

| Pillar | Mechanism | Verified location | On by default? |
|---|---|---|---|
| **Per-turn reflection ("dreaming")** | After every turn, a **forked agent** replays the conversation and asks *"should any skill/memory be saved or updated?"* — tool-whitelisted to memory+skills only; never touches the main prompt cache. | `agent/background_review.py` (`spawn_background_review_thread`), called from `agent/turn_finalizer.py:651` | Yes — nudge-triggered per turn |
| **Memory persistence** | `MEMORY.md` + `USER.md`, injected every turn; plus pluggable external providers (honcho, mem0, openviking, hindsight, holographic, retaindb, byterover) | `agent/*`, `hermes memory` CLI | Built-in always on |
| **Skill lifecycle (curator)** | Background maintenance: tracks usage, marks idle skills stale, archives, backs up. Optional LLM "consolidate overlapping skills into umbrellas" pass. | `agent/curator.py`, `hermes curator` | Inactivity sweep free; `consolidate` OFF by default |
| **Historical record + visualization** | `journey` / Star Map / Memory Graph: timeline of learned skills + memories over time. `session_search` reaches the full SQLite session DB (FTS5). | `hermes_cli/journey.py`, `agent/learning_graph.py` | Read-only view |

### The gap (what's missing that the user wants)
There is **no single "dream" command** that reads the *entire historical record at once* and distills an *elongated/consolidated* lessons-learned digest that then rewrites memory + skills. We have:
- per-turn review (background_review) — myopic, single-session
- skill consolidation (curator) — skills only, not memory, off by default
- visualization (journey) — shows the data, doesn't synthesize
- search (session_search) — retrieval, not consolidation

**The "elongated dreaming state" = a periodic consolidation pipeline** that: (1) pulls the whole historical record, (2) distills durable lessons, (3) writes them back to memory + skills in compact form. This we build.

---

## 2. Design: The Elongated Dreaming Loop

```
┌──────────────────────────────────────────────────────────────┐
│  TRIGGER (cron, periodic)  e.g. weekly "Sunday 00:00"        │
└─────────────────────────────┬────────────────────────────────┘
                              ▼
   STEP 1 — GATHER the historical record
     · session_search(browse all recent sessions / topic queries)
     · Read current MEMORY.md / USER.md
     · List current skills + usage (curator status / skill_view)
                              ▼
   STEP 2 — DISTILL (an LLM pass, "the dream")
     · Prompt: given ALL of the above, produce:
         a) New durable facts (user prefs, env quirks → memory)
         b) Lessons learned / pitfalls → skills (patch existing or new)
         c) Consolidation: overlapping memories/skills → merge
         d) Stale items → deprecate/archive
   STEP 3 — WRITE BACK (tool-whitelisted)
     · memory() batch op (atomic add/replace/remove)
     · skill_manage() create/patch/delete (via curator gate)
   STEP 4 — REPORT
     · Deliver a compact "dream digest" to the origin chat:
       what was learned, what changed, what was pruned
└──────────────────────────────────────────────────────────────┘
```

This maps ~70% onto existing machinery; the new bits are STEP 1 (batch gather) + STEP 2 (the distillation prompt) + the cron wiring.

---

## 3. Concrete implementation — three tiers (pick your appetite)

### Tier 1 — Zero-build, manual "dream" (do today)
A skill that encodes the gather → distill → write-back procedure so I (or any Hermes session) can run a dream on demand:
- `skill_manage(create "dreaming" ...)`: trigger conditions, the exact gather commands, the distillation prompt, the write-back rules, the digest format.
- Invoke manually after heavy work: `hermes chat -q "Run a dreaming pass over my session history."` (with the skill loaded).

### Tier 2 — Automated periodic dreaming (recommended)
The same skill, but scheduled as a **cron job** so it runs without a human remembering:
```
cronjob(
  action="create",
  schedule="0 2 * * 0",        # weekly, Sunday 02:00
  name="weekly-dream",
  skills=["dreaming"],          # loads the dreaming skill
  prompt="Run the Elongated Dreaming procedure...",
  enabled_toolsets=["file","web"],   # file to read memory/graph, web optional; memory+skill_manage are always available
  deliver="origin",             # auto-deliver digest back to this conversation/chat
)
```
Key config considerations (verified):
- Cron sessions pass `skip_memory=True` by default — fine, the dream *is* the memory writer.
- `attach_to_session=True` → the digest lands as a continuable thread you can reply to.
- Budget: one weekly run, model `deepseek-v4-flash-0731` (cheap) for the distillation pass.

### Tier 3 — Strengthen the per-turn and consolidation defaults (enable what's off)
- Enable skill consolidation: `hermes config set curator.consolidate true` → curator merges overlapping skills into umbrellas automatically.
- Optionally route `background_review` to a cheaper model: `hermes config set auxiliary.background_review.model deepseek-v4-flash-0731` (digest-based replay, cheaper than main model).
- Consider an external memory provider (mem0 / honcho) for true cross-session recall if the plaintext MEMORY.md ever gets large.

---

## 4. The `-j 12` nuance (and the general capture convention)

The user wants "a simple/clean/easy way to remember this stuff." Answer: **the memory tool is exactly that** — and I've already applied it:

- Added a consolidated memory entry: `-j 12` is optimal on this Windows host when nothing else heavy runs; default-parallel `cargo test --workspace` OOMs the paging file (`os error 1455`); **never trust exit code when piped through `tail`** (it's tail's exit, not cargo's) — capture cargo's real exit via `cargo ...; echo EXIT=$?` or grep the log for `test result: FAILED`.

**General convention going forward:** every time I (or a subagent) discover an environment quirk, a pitfall, or a cost-saving setting, immediately `memory(add ...)` in a one-liner. Durable facts live in memory; reusable *procedures* live in skills; *work logs* deliberately do NOT go to memory (they're recallable via session_search).

---

## 5. Files / artifacts produced by this plan

| Artifact | Path | Owner |
|---|---|---|
| The `dreaming` skill (procedure: gather → distill → write-back) | `~/.hermes/skills/` via `skill_manage` | Hermes |
| Weekly cron job | `hermes cron` / `cronjob` tool | Hermes |
| Consolidated memory entries (e.g. `-j 12`, tail-exit trap) | `MEMORY.md` | Already done this session |
| Optional: enable `curator.consolidate` | `config.yaml` via `hermes config set` | Hermes (needs operator OK) |

---

## 6. Risks / trade-offs

| Risk | Mitigation |
|---|---|
| Dreaming pass could write contradictory memory | Skill instructs: prefer EDIT/consolidate over blind ADD; batch ops are atomic; cap memory size |
| Cost of periodic runs | Cheap model + weekly cadence; digest-based replay keeps tokens low |
| A "bad dream" distills nonsense | The digest is delivered as a continuable thread (`attach_to_session`) so the operator can override/reject before it sticks; curation is non-destructive (backup before archive) |
| Historical record is huge | session_search is FTS5-indexed and scoped (recent N sessions / topic queries); the dream reads a bounded window |

---

## 7. Recommended next steps (in order)

1. **Create the `dreaming` skill** (Tier 1) — makes the procedure durable and reusable today. **I can do this now.**
2. **Schedule weekly cron** (Tier 2) — `weekly-dream`. **I can do this now.**
3. Verify one manual dream run end-to-end and inspect the digest before letting the cron repeat unattended.
4. Operator opt-in (AGENTS.md rule 9 — config change): `curator.consolidate true` + optional external memory provider.

---

*Decision points for the operator:* (a) cron cadence (weekly? daily?), (b) model for the distillation pass (`deepseek-v4-flash-0731` default), (c) whether to enable `curator.consolidate` + an external memory provider, (d) deliver target for the digest (this conversation? a dedicated channel?).
