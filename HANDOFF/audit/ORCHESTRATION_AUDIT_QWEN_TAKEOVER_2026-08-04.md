# ORCHESTRATION AUDIT -- QWEN CODE TAKEOVER (2026-08-04)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Final. Author: Qwen Code (qwen3.8-max) session, Windows host,
operator-approved audit + setup package. Scope: the unified orchestration
setup (command surface, docs, dispatch tooling, state files, lane
economics) and readiness to delegate the remaining v0.4.0/v0.5.0 work from
this Qwen Code installation.

## 1. Version state

- GitHub (`origin/main`) still carries the PREVIOUS orchestration version:
  10-step loop, manual lake picking, RESULT/PATCH/VERDICT worker contract.
- Local HEAD carries the 2026-08-03 token-efficiency overhaul (commits
  9121fd3e, fd3fb712, 7cd7d4f2 and successors): 9-step loop via
  dispatch_dial.py / parse_orchestration_footer.py / batch_handoff.py /
  build_lock.py, AGENTS.md-aligned worker footer, qwenpaid ladder fix.
- WARNING: 15 commits ahead of origin/main and unpushed (AGENTS.md rule 5).
  ALL new orchestration tooling exists only on this machine. Operator
  decision on push timing is pending.

## 2. What was verified green

- docs/ORCHESTRATION.md Section 2.2 matches every script's actual CLI on
  all load-bearing arguments (no name/flag/default mismatches).
- qwenpaid wired end-to-end on this host: qwenpaid.env present,
  registry.json block present, lake_route.py ladders CODER/THINK/MAX.
- tmp/lakes/{registry.json, ledger.jsonl, round_robin_state.json} present.
- Git rules hook active (core.hooksPath = .githooks), Python 3.14.6.
- dispatch_dial.py live: plain CODER spec correct; FLASH request touching
  core/src/crypto/ auto-escalates to THINK with security_gate_required.
- Cooldown-aware routing live: after recording groq vacuous + qwen 403,
  the dial rotated models and escalated FLASH -> CODER at retry-count 2.
- parse_orchestration_footer.py: good footer -> DONE/exit 0; missing footer
  -> degraded:true, non-zero exit.
- batch_handoff.py --dry-run: plans moves, touches nothing.
- build_lock.py: acquire OK; contended --run refuses and does NOT execute
  the wrapped gate; release by holder name OK; exit codes non-zero as
  documented.
- scripts/rules_check.py exit 0 on all files created/edited by this audit.

## 3. Findings (ordered by impact)

1. No /orchestrate entry point existed for Qwen Code. FIXED today:
   .qwen/commands/orchestrate.md created (thin launcher, {{args}},
   lanes-only backend guidance).
2. .agents/skills/build-verify/SKILL.md pointed at nonexistent
   .Codex/skills/build_verify.sh. FIXED today: now .claude/skills/
   build_verify.sh plus a Windows Git-Bash note.
3. Scoped --files targeting in delegate_task.py is broken end to end:
   apply-side allowlist compares against the raw path incl. the
   :Lstart-Lend suffix, so every scoped diff is REJECTED, full-file
   fallback finds nothing, and exit code is 0 instead of vacuous.
   Evidence: tmp/ORCH_E2E_GITIGNORE_LINE_REFS_2026-08-04_response.md.
   Ticket: HANDOFF/todo/DELEGATE_SCOPED_FILES_ALLOWLIST_FIX.md.
4. The free DashScope lane is DEAD: ~/.config/scmorc/dashscope.env returns
   401 Invalid API-key for every qwen model (11 observed). Router cooldowns
   are per-model, so a lake-level auth failure only cools one model per
   record and the dial keeps returning the dead lake. Operator: refresh the
   key or retire the lane; the per-lake cooldown behavior is noted in the
   allowlist ticket for a future router fix.
5. tmp/lakes/registry.json is gitignored with no regeneration script --
   the qwenpaid routing fix does not survive a fresh checkout. Ticket:
   HANDOFF/todo/REGENERATE_LAKE_REGISTRY.md.
6. orchestrate_strict.py is not gate-safe (no build_lock wrap, ignores
   security/delivery gate flags, mixed: provenance, needs tmp/tasks/).
   Manual 9-step loop remains the safe path. Ticket:
   HANDOFF/todo/ORCHESTRATE_STRICT_HARDENING.md.
7. Governance: AGENTS.md capability classes did not name Qwen Code.
   Proposed class text in Section 9 below -- operator ratification
   pending. Also: AGENTS.md FOREIGN WORKER exception still references
   HANDOFF/todo/GEMINI_SCMORC_DRIVER_2026-07-07.md; that ticket lives in
   HANDOFF/done/ (complete since c371065f) and the exception can go.
8. Doc drift, FIXED today: CLAUDE_REFERENCE.md gained the Section 5
   script inventory; .claude/commands/orchestrate.md internalise list
   gained Section 3; ORCHESTRATION.md Section 2.1 now states the code
   ladders are authoritative over the prose order.
9. Stale artifacts inventoried (no action taken; operator may archive):
   .bob/skills/* (swarm-era, macOS paths), scripts/supervisor.py
   (self-declared legacy), 7 IN_PROGRESS tickets untouched since
   07/17-25, three 0-byte placeholder tickets, two retire-on-sight
   tickets (PQC_10_MLDSA_MODULE_MISSING, V040_FREE_LANE_DISPATCH_READY).
10. Environment: C: has 17.9 GB free vs ~25 GB recommended for full
    gates; Groq hard TPM limit 6000 confirmed (full-file prompt of 9602
    tokens 413'd); gemini.env absent (documented; router skips);
    concurrent sessions were observed committing/pushing to the shared
    branch mid-audit -- single-writer discipline must hold for the sprint
    (orchestrator owns the worktree; other sessions use worktrees or
    HANDOFF packets).
11. Groq FLASH (llama-3.1-8b-instant) hallucinated diff context lines on
    a scoped slice even for a 2-character mechanical fix -- consistent
    with Section 9 lesson 13; FLASH is acceptable only for tasks whose
    whole context fits the prompt, and orchestrator-side gate + diff
    inspection remain mandatory.

## 4. Lane status at audit time (2026-08-04 ~21:00 local)

| Lane | State |
|---|---|
| qwenpaid (qwen3.8-max-preview) | LIVE, operator PRIMARY for CODER/THINK/MAX; same paid plan as this Qwen Code session -- budget accordingly |
| groq | LIVE but 6K TPM hard cap; in cooldown until UTC midnight after test dispatches |
| qwen (DashScope free) | KEY INVALID (401 all models) but QUOTA ALIVE per operator console 2026-08-04 (1M-token/model allocations, most expiring 2026-10-06). Key swap pending; registry model lists pruned to quota-confirmed models (qwen3-coder-flash carries no free quota -- absent from the console table) |
| openrouter :free | DOCUMENTED SATURATED (429s on 2026-07-17); untested today |
| openrouter morph/fusion (paid keys) | presumed live (not probed; spend-capped) |
| openrouter_direct (DeepSeek V4 Flash) | ADDED 2026-08-04 (operator): backup lane for clearly scoped tasks, USD 1/day cap; registry block live, provider wiring dispatched via ticket DISPATCH_LAKE_OPENROUTER_DIRECT.md; key file to be placed by operator |
| gemini | no key file -- skipped by router (documented) |
| ollama cloud | free tier, a few tasks/week; not probed today |

Vision models (qwen3-vl-* on the same console, ~1M calls each): not a
dispatch lane (delegate_task.py flow is text/diff-only) -- use as an
ORCHESTRATOR capability: device/UI evidence verification (Android
fresh-install screenshots, 5-node run 2 receipt screens, error-dialog
transcription). This Qwen Code session accepts image input natively.

Sprint economics rule (unchanged): free lanes first; qwenpaid for real
work; never burn the paid window on orchestrator-side fan-out when a
Coding-Plan model can read instead.

## 5. Setup package implemented this session

- .qwen/commands/orchestrate.md -- Qwen-native /orchestrate launcher.
- .agents/skills/build-verify/SKILL.md -- path fix + Windows note.
- docs/ORCHESTRATION.md -- Section 2.1 code-authoritative note.
- .claude/commands/orchestrate.md -- Section 3 in internalise list.
- docs/CLAUDE_REFERENCE.md -- new Section 5 script inventory.
- HANDOFF/todo/: REGENERATE_LAKE_REGISTRY.md,
  ORCHESTRATE_STRICT_HARDENING.md,
  DELEGATE_SCOPED_FILES_ALLOWLIST_FIX.md,
  ORCH_E2E_GITIGNORE_LINE_REFS_2026-08-04.md (live test ticket; still
  todo -- its fix is a real 2-character doc correction).
- tmp/tasks/ created.
- This audit doc.

## 6. Test evidence summary

Dry chain: dial (3 scenarios) [OK], footer parser (good + degraded) [OK],
batch_handoff --dry-run [OK], build_lock acquire/contention/release [OK],
rules_check on all touched files [OK].

Live e2e (free lanes only, per operator): 3 dispatches -- groq scoped
(rejected: bug #3), groq full-file (413 TPM), qwen (401 dead key, bug #4).
Router/ledger/cooldown/escalation behavior proven live on all three.
The apply -> gate -> handoff arc completes its first live pass on the
first real sprint dispatch (ticket ORCH_E2E... or next queue item).

## 7. Sprint state (from remaining-work audit, same day)

Authoritative order: HANDOFF/todo/_NEXT_ORCHESTRATE_KICKOFF.md supersedes
_QUEUE.md body. Phases: 0 PR #136 merge (CI re-running on ea0de26f;
adversarial review of cli/src/server.rs gating owed) -> 1 identifier
unification audit -> 2 five-node run 2 (relay 34.203.213.35, re-verify IP
pre-run; GPT lane armed on the merge) -> 3 v0.4.0 close-out (DNS-first
hardening urgent -- no Elastic IP; seeding findings fix-all; bump
0.3.5->0.4.0) -> 4 v0.5.0 farm sim. Human gates: operator Elastic-IP
decision, Lucas port-forwards + DDNS, operator push/tag. Verify-don't-
trust: grep-confirm every doc-claimed DONE at HEAD before closing.

## 8. Open escalations for the operator

1. Push timing for the 15 local commits (new orchestration tooling is
   single-machine until pushed).
2. Refresh or retire the dead DashScope free key
   (~/.config/scmorc/dashscope.env).
3. Elastic IP: widen IAM policy or accept drift + DDNS (blocks DNS-first
   bootstrap root cause).
4. Disk cleanup approval (~7 GB more needed for comfortable full gates).
5. Ratify the AGENTS.md capability-class text below.

## 9. Proposed AGENTS.md amendment (PENDING RATIFICATION -- not applied)

Add after the MAC LANE block in "Capability classes":

    ### FULL-QWEN (Qwen Code on the Windows host, toolchain available)
    Same authority and duties as FULL: may run build gates, move HANDOFF
    files, and commit per CLAUDE.md's finalize-checklist rules (which
    Qwen Code loads via .qwen/commands + .agents/skills instead of
    CLAUDE.md itself). The Windows host's build results are authoritative
    regardless of which FULL-class tool produced them. Its /orchestrate
    entry point is .qwen/commands/orchestrate.md. Push remains forbidden
    (rule 5); the MAC LANE push exception does not extend to this class.

And in the FOREIGN WORKER heading, drop the stale exception: remove
'except "HANDOFF/todo/GEMINI_SCMORC_DRIVER_2026-07-07.md"' (ticket is in
HANDOFF/done/ since c371065f).
