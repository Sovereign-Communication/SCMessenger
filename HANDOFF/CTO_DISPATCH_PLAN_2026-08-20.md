# CTO dispatch plan -- 2026-08-20 (post-unification seat)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Supersedes: HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md (merge train completed)
Entry point: `/CTO`. State file: `HANDOFF/CTO_STATE.md` section 0 + 2026-08-20 addendum.
Doctrine: controller plans and validates; workers implement in isolated
worktrees; CTO merges under the section 0b confidence test.

## 0. What this seat verified before writing this plan

Every claim below was obtained by command in THIS session (AGENTS.md rule 13).

| Check | Command | Result |
|---|---|---|
| Open PRs | `gh pr list --state open` | #193, #188, #178, #170, #156, + dependabot |
| #188 content | `gh pr diff 188 --name-only` | 1 file: HANDOFF/CTO_STATE.md (+244/-4 pre-merge) |
| main head | `git log --oneline -8 origin/main` | eaf9ba73 (#191 merge) |
| Branch protection | `gh api .../branches/main/protection` | strict:false; contexts: Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest) |
| Hooks | `git config core.hooksPath` | .githooks (pre-commit + pre-push present) |
| Orchestration contract | `python scripts/orchestration_contract.py --print-version` | 2.0.0 |
| Kernel | `python scripts/orchestrate_strict.py --dry-run` | exit 0 -- but see finding A |
| Gate scripts | glob scripts/ | all 13 present (pr_scope, check_wiring, orchestrator_guard, apply_branch_protection, agy_run, delegate, lane_probe, session_orchestration_audit, reclaim_safe, rules_check, orchestrate_strict, orchestration_contract, orchestration_worktree) |
| Session hooks | .claude/hooks/ | model_gate.sh, session_orientation.sh, preflight_guard.py (+test), check_no_emoji.py |
| Lane roster | `python scripts/delegate.py --list-lanes` | 16 free lanes OK, probed 2026-08-19; qwenpaid DEAD |
| agy auth | `agy models` | [OK] WORKING -- full roster (gemini-3.7-flash-high ... gpt-oss-120b-medium). The 2026-08-20 OAuth expiry recorded in the addendum is cleared |
| Frontend adapters | orchestration/manifest.yaml | claude, codex, qwen, gemini, bob, opencode, portable, script -- all wired for orchestrate; CTO seat existed ONLY in .claude/commands (gap, now being fixed) |

## 1. Gate audit findings (this seat)

| ID | Finding | Severity | Disposition |
|---|---|---|---|
| A | `orchestrate_strict.py --dry-run` plans provider=qwenpaid, which the roster lists DEAD and the operator has ruled off limits ("Qwen paid remains off limits", 2026-08-19) | HIGH -- kernel can route authority onto a forbidden/dead lane | Lane L8: kernel must consult scripts/lanes.json status + operator policy before planning |
| B | scripts/lane_probe.py missing the zai `thinking:disabled` fix that #181 applied to delegate.py (recorded in CTO_STATE section 0) | MED -- silent vacuous success on probe | Lane L6: fix dispatched to isolated worker (in flight) |
| C | session_orchestration_audit.py STATUS column untrustworthy (7 false "Stalled", empty VERIFICATION marked valid; token/step accounting OK) | MED -- audit gate lies about delegation health | Lane L7 |
| D | apply_branch_protection.sh hardcoded strict:true + removed "Android JVM Unit Tests" context | MED -- script reality mismatch | #193 fixes; rerun in flight |
| E | HANDOFF/CTO_STATE.md on main carries ~110 double-encoded UTF-8 sequences (75 em dashes, 21 section signs, quotes) introduced by the #185 merge; the #188 branch was byte-clean | LOW-MED -- handoff doc corrupted; passed CI because hygiene does not check encoding | Lane L4: repair PR with byte-level acceptance criteria |
| F | model_gate.sh emitted MODEL GATE BLOCKED with continue:false but the session ran anyway (hook exit 0; observed in session 23881d4b) | MED -- the session-launch gate fails OPEN; a misconfigured session then spends premium tokens | Lane L9: SCANNER RCA + hard-fail mechanism |
| G | Repository Hygiene "new blank line at EOF" caught on #188 merge push -- gate works in CI; local pre-commit hook lacks the same check | INFO | Candidate: add EOF-blank-line check to .githooks/pre-commit so the lesson fails locally, not after a push |

## 2. Merge train state

Executed by THIS seat (merge authority per CTO_STATE section 0b):

1. #188: merged origin/main into docs/cto-handoff-2026-08-19 in isolated
   worktree .claude/worktrees/cto-188-merge; conflict resolved keeping all
   sections newest-first per the addendum's own instruction; pushed 50209900.
   Repository Hygiene flagged "new blank line at EOF" (the addendum's trailing
   blank line -- latent since 25607354, first hygiene pass over the diff);
   fixed and pushed ce581698. MERGE when checks are green (lane L1).
2. #193: reran the single failed Test (ubuntu-latest) lane via
   `gh run rerun 32408344178 --failed`. MERGE when green (lane L2).

Note: the scm-handoff worktree (C:/Users/SCM/Documents/GitHub/scm-handoff)
holds the #188 branch plus 28 UNSTAGED modified .md files (the known
renormalization backlog). NOT touched this seat. Do not commit, stash, or
revert them; they belong to whoever ran that renormalization pass.

## 3. Dispatch lanes -- ordered

Delegation doctrine: docs/ORCHESTRATION.md + scripts/orchestrator_guard.py.
CONTROLLER writes nothing but HANDOFF/ and tmp/orchestration/. Every writer
gets an isolated worktree. Every shell-needing task goes to agy (the only
free lane with a shell; auth confirmed working this seat). Review of anything
under core/src/{crypto,transport,routing,privacy} requires an independent
CRITICAL_VALIDATOR on a DIFFERENT, STRONGER model than the implementer
(measured tiering: gemini-3.7-flash-high implements, gemini-3.1-pro-high
reviews).

| Lane | Task | Role/tier | Acceptance (machine-verifiable) |
|---|---|---|---|
| L1 | Merge #188 when green | CTO (no delegation) | `scripts/pr_scope.sh 188` clear; all required contexts pass; merge commit on main |
| L2 | Merge #193 when rerun is green | CTO (no delegation) | same gate; then verify `bash scripts/apply_branch_protection.sh --dry-run` output matches live API state |
| L3 | Branch protection strict:true | OPERATOR DECISION -- CTO presents evidence only | Evidence packet: every open PR merged or explicitly deferred; required contexts all run on every PR shape (docs/scripts/Rust/Android); no runner-hang incident in flight. Then operator runs/approves `scripts/apply_branch_protection.sh --apply --strict` |
| L4 | CTO_STATE.md encoding repair | IMPLEMENTER, isolated worktree, base = post-#188 main | Zero bytes of the mojibake patterns (C3 82 C2 A7 / C3 A2 E2 82 AC E2 80 9D / C3 A2 E2 80 A0 E2 80 99 / C3 A2 E2 80 A0 E2 80 9D / C3 A2 E2 82 AC E2 80 9C); sections 1-8 + 0b/0c byte-equal to `git show c1708f58:HANDOFF/CTO_STATE.md` where semantically unchanged; valid UTF-8; no BOM; hygiene green |
| L5 | Wiring PR: .qwen/commands/CTO.md + .agents/skills/onboard + .claude/skills/onboard + this plan | IMPLEMENTER draft (dispatched this seat), CTO integrates | Files match conventions of .claude/commands/CTO.md and .agents/skills/orchestrate/SKILL.md; no emoji; hygiene green; `python scripts/rules_check.py` clean |
| L6 | lane_probe.py zai thinking fix | IMPLEMENTER (dispatched this seat) | py_compile exit 0; zai path sends thinking:disabled, mirrors delegate.py #181 fix; no live probes during verification |
| L7 | session_orchestration_audit.py STATUS fix | IMPLEMENTER via agy (shell tier -- must rerun the audit against recorded dispatch logs) | STATUS column agrees with hand-checked ground truth for at least the 7 previously misreported dispatches; empty VERIFICATION never marked valid |
| L8 | orchestrate_strict.py lane policy | IMPLEMENTER via agy or scoped HTTP lane | `--dry-run` never plans a lane marked dead in scripts/lanes.json; never plans qwenpaid (operator-banned); failure mode is explicit BLOCKED, not silent fallback |
| L9 | model_gate fail-open RCA | SCANNER (read-only) | Written finding in HANDOFF/ or docs/: why exit 0 + continue:false did not stop session 23881d4b; proposed hard-fail mechanism (hook exit 2 or launcher enforcement) |
| L10 | U-C2: swarm.rs 11 topic literals -> core constants | IMPLEMENTER agy + CRITICAL_VALIDATOR gemini-3.1-pro-high (rule 8: transport tree) | Brief at tmp/unify-c2/BRIEF.md; validator verdict committed to docs/security/; CRYPTO_TOUCHED verdict recorded; CI green |
| L11 | Two-Commands enum unification (withdrawn from #191) | PLANNER first (design note), then IMPLEMENTER | Design note answers: migrate cli/tests/integration.rs to the lib enum, or make main.rs consume it; one definition survives; consumer census includes cli/tests/ (the #191 lesson) |
| L12 | Rank-4: two LedgerManager handles over one file | PLANNER design note (UniFFI accessor) before any implementation | Design note committed; implementation is a SEPARATE later lane |
| L13 | U1 escalation single-authority + U2 WiFi-Aware send() no-op | IMPLEMENTER (zai backlog per addendum) | Wiring proven by check_wiring.py delta + targeted test, not prose |
| L14 | Two-node LAN field test (D6/D7) + v0.4.0 tag | OPERATOR + hardware | Scoring: receiver-side decrypt + durable history + receipt. Not transport ACKs, not UI counters, not BLE local acceptance. Tag only after D6/D7 pass and #154's proof is on the tag commit |

Critical path: L1 + L2 (parallel, CI-bound) -> L3 (operator) -> L14
(operator/hardware) -> tag. L4-L13 run concurrently beside the path; L10 is
the only one touching a merge-blocked tree and needs its review artifact
BEFORE merge scheduling.

## 4. Delegation executed directly by this seat (in flight)

- W1: wiring files draft (L5 content) -- isolated worktree worker, running.
- W2: lane_probe.py fix (L6) -- isolated worktree worker, running.
- Pending W3: L4 encoding repair -- dispatch AFTER L1 lands (base changes).

End-of-session duties for whichever seat closes this out: update
CTO_STATE.md (section 0-rule standing rule), run
`python scripts/session_orchestration_audit.py`, record its output with the
known STATUS-column caveat (finding C) until L7 lands.

## 5. Lessons re-affirmed this seat (gates, not prose)

1. A lesson stored as prose gets re-learned; a lesson stored as a gate does
   not (section 0c governing finding). Evidence this seat: the EOF-blank-line
   lesson existed in section 8 as prose AND as a CI step; CI caught it in 8
   seconds. The residual gap is local (finding G).
2. Derive from origin/main, never the shared checkout (it was on a merged
   branch with a gone upstream this whole seat).
3. Machine-verify fixtures and READ the output (57-char peer ID lesson);
   this seat applied it: byte-level census before and after the merge
   resolution, not a visual diff read.
4. Worker-produced artifacts are not exempt from gates: the #185 merge
   proves it -- a worker-touched CTO_STATE.md landed with ~110 mojibake
   sequences and zero checks caught it (finding E). Encoding sanity belongs
   in Repository Hygiene (candidate gate addition beside G).

## 6. Seat updates -- appended in-session as they happened

- #193 MERGED as 520e26ea (pr_scope clean, 26/26 checks). Merged ahead of
  #188 because it was fully green and file-disjoint; the handoff ordering
  assumed #188 would be green first.
- #188 gated on its non-required Test (windows-latest) lane; pr_scope.sh
  failed closed correctly. Merge as soon as it clears.
- L5 wiring: PR #195 (CTO.md + onboard skills + this plan + .qwen/.gitignore).
- L6 lane_probe fix: PR #196.
- L7 audit STATUS fix: PR #197. Worker commit split before PR -- see H.
- L8 kernel lane policy: PR #198. Worker commit split before PR -- see H.
  Dry-run now plans provider=cerebras model=gemma-4-31b; qwenpaid and
  dashscope rejected as operator-banned; dead lanes excluded; fail-closed
  on empty roster.
- L9 model-gate RCA: COMPLETE and independently verified (24-session count
  reproduced by the CTO's own grep). Fix mechanism: exit 2 + stderr.
  model_gate.sh fix dispatched (W4).
- agy auth: WORKING this seat (full roster), clearing the addendum's
  2026-08-20 OAuth-expiry blocker. U-C2 is dispatchable.

## 7. Finding H -- worker dispatch swept worktree line-ending noise (x2)

Both agy IMPLEMENTER commits (L7, L8) swept the repo's ~190-file
.gitattributes CRLF-to-LF renormalization backlog into their commits --
any fresh worktree checkout materializes that backlog as dirty files, and
both workers staged everything despite packet text saying "stage explicit
paths only." Caught at integration; split via reset + explicit-path recommit
before PR. The prose instruction failed; per section 0c rule 7, the fix is
mechanism:

1. Gate candidate: `scripts/verify_worker_commit.py <commit> <allowed
   path>...` -- exit non-zero if the commit touches anything outside the
   packet's file list. Run before any worker commit is cherry-picked onto a
   PR branch. Until it exists, the manual gate stands: `git show --stat` on
   every worker commit before integration (run twice this seat, caught both).
2. Same failure family: `scripts/agy_stream_watch.py` reported "[RESULT]
   ERROR" for the L9 and L8 runs even though both workers delivered DONE +
   artifacts + passing verifications. The watcher classification needs the
   same fail-closed-but-accurate treatment finding C demands of the audit
   script -- a delivered output contract is a success, whatever the wrapper
   exit code. New lane for the orchestrator queue.
3. The renormalization backlog itself is still uncommitted (dirty in the
   L7/L8 worktrees and in scm-handoff). It collides with every .md-touching
   change, so land it as its own dedicated PR after this merge train, not
   inside one.
