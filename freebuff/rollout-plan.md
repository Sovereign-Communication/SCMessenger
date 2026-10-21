# SCMessenger 0.5.1 — Unified Rollout Plan (2026-10-21)

**Status:** FINAL — pending Lucas's launch word.
**Scope:** SCMessenger only. Cross-repo matters go to main chat, never into lane prompts or repo issues.

## 1. Objective and end state

Get every SCMessenger work item — on GitHub and on the Freebuff laptop — landed, verified, and consolidated into **one unified app on main**, tagged **v0.5.1**, with the laptop reclaimed to **one clean main clone**, **zero** only-local work product anywhere, and Linux CI served from Ubicloud's free tier.

Done when ALL of these hold (verified by the Overseer against GitHub, never from lane self-reports):
- Every 0.5.1-scope PR merged or closed with a documented disposition; branches deleted.
- Main green; v0.5.1 tagged.
- Zero commits exist only-locally on any machine (SHA-verified on GitHub).
- The laptop holds one clean main clone; all other worktrees removed; trash emptied on operator confirm.
- A real CI job has run on a Ubicloud runner on the free credit.

## 2. Lane roster — five agents, crisp scopes

| Lane | Name | Owns | Does NOT touch |
|------|------|------|----------------|
| 0 | WIP Audit + Burn-Down | Sweep the laptop for ALL SCMessenger work product; back it up to GitHub (additive only); purge local copies only when SHA-verified; burn down backup branches one at a time into PRs/handoffs | Merging to main (hands PRs to Lane 1); other lanes' claimed worktrees/branches; Ubicloud |
| 1 | Merge Train | The #595 merge queue and ALL merging to main; the v0.5.1 tag | Writing new code (rebase-only); other lanes' worktrees |
| 2 | Fix-Forwards | Non-transport red/broken PRs (#597, #601, #508): diagnose, fix, verify green, hand to Lane 1 | Transport files (`core/src/transport/` → hand to Lane 3); merging |
| 3 | Transport | `core/src/transport/` and ALL transport work: #521 + #599 fix-forwards, #623/#624/#625 new implementation | Merging; non-transport PRs |
| 4 | Infra | Ubicloud runners on the free tier only (PAT → verify → workflow PR → confirm free-credit burn) | Local backup/disk (that's Lane 0 — do not duplicate); merging (workflow PR goes through Lane 1) |

**Supersessions (explicit):** Lane 0 absorbs and replaces the backup/disk sections of draft Lane 4 and rev 4 Outcomes 1–2, which assumed 4 known worktrees on a known machine. The rev 4 single-operator mission is retired as an execution vehicle — its Outcomes 3 (Ubicloud) and 4 (land everything + tag) live on as Lane 4 and Lane 1 scopes. The 2026-10-21 09:54 multi-lane plan's Lane 4 backup/disk scope is withdrawn in favor of Lane 0.

## 3. Rollout sequence — phased, dependencies explicit

**Phase A — Audit discovery (Lane 0, Phases 0–1). READ-ONLY.**
Lane 0 sweeps the laptop, builds the full inventory (every repo root, worktree, branch, stash, dangling commit; local-vs-GitHub for every ref), posts it on #602. Nothing is modified. Lanes 1–4 do NOT start yet — the inventory is the map they work from.

**Phase B — Lane launch.**
Lanes 1–4 launch, each in its own worktree, and immediately register their claimed worktree paths + branches as a comment on #602. From that moment those paths/branches are off-limits to Lane 0's purge. Lane 1 verifies current main head from GitHub (never trust a pinned SHA).

**Phase C — Backup + verified purge (Lane 0, Phases 2–3).**
Lane 0 pushes every only-local ref to NEW `backup/` branches (zero CI burned), the Overseer SHA-verifies each on GitHub, and only then Lane 0 removes the unclaimed worktrees (`git worktree remove`, never `rm -rf`). Lane-claimed worktrees are never touched.

**Phase D — Burn-down (Lane 0 Phase 4 + Lanes 1–3).**
Lane 0 works backup branches one at a time: dedupe → open PR / hand to Lane 1 / cherry-pick / disposition. Lane 2 and Lane 3 work their queues; all green PRs flow to Lane 1 via "green and ready" handoffs. Lane 1 drains the queue with batch merging.

**Phase E — Tag + stand-down.**
Lane 1 tags v0.5.1 after the queue is empty and main is green. Then all lanes remove their worktrees; Lane 0 (or the operator) confirms one clean main clone remains; trash emptied on operator confirm. Overseer runs the end-state checklist (§1).

Ubicloud (Lane 4) runs in parallel with Phases A–D — it has no dependencies on them.

## 4. Coordination protocol

- **#595** — the merge queue and lane coordination point. Source of truth for what Lane 1 merges (the prompt's embedded list is the 2026-10-21 snapshot; #595 is authoritative). Lane 0's burn-down handoffs feed into it.
- **#602** — the WIP reporting hub. Lane 0's inventory, backup report, and purge report go here; lanes register claimed worktrees/branches here.
- **PR comments** — "green and ready" handoffs (PR number, exact head SHA, green CI run link, Jev verdict; + Rule-8 sign-off where required). Lane 1 verifies every claim against GitHub before merging — never trust the comment alone.
- **Overseer thread ("Muse Role — Overseer Seat")** — adoption/verification tracking. Lanes don't post here; the Overseer reads #595/#602/PRs and verifies.
- **Escalation:** lane → Overseer (via issue comment tagging the blocker) → Lucas, and only for genuine blockers or access Lucas alone can unlock. Lanes never DM Lucas technical questions.
- **Idle lanes:** report to the Overseer, take the next highest-priority item from #595. Never invent work or expand scope.
- **Sole merger:** Lane 1 is the ONLY lane that merges into main or into queued PR branches. Lane 0's "merge as the situation calls for" means: merging main INTO backup branches to keep them current, and PRs — never merging to main itself. This invariant is what the batch-merge / halt-and-bisect safety model depends on.

## 5. Standing rules (consolidated, all lanes)

- **Jev gate v1** applies to every lane. If `~/workspace/jev-gate-policy-v1.md` is not present on this machine, use this summary: *Decision — `is_destructive` ≥ 0.5 → escalate; disposition lead < 0.20 → escalate; `advances_goal` < 0.95 → investigate (not auto-escalate); choice confidence is telemetry, never a gate. Assessment — 5 aspects (completeness, confidence, correctness, no-regression, evidence quality), relevance-first; 4/5 ≥ 90 with 5th at 85–89 → auto-investigate (no real risk → pass with note; real risk → block); a single aspect at 85+ never blocks alone; below 85 → fix and re-assess. Below bar starts investigation automatically — escalate only on actual risk. Stop after 3 consecutive stalled rounds.*
- **Purge rule:** 100% certainty before purging anything — work product proven landed-or-superseded AND swept for importable snippets first (cherry-pick anything worth keeping). Never purge on supersede alone. Lane 0 purges only SHA-verified content.
- **Merge boundary:** lanes act under Lucas's direct authorization for this rollout. Lane 1 merges green PRs; no lane merges red CI, force-pushes main/shared branches, or rewrites tags.
- **Backup hygiene:** new `backup/` branches only; never push over an existing branch or PR head; backup pushes must not burn CI (cancel runs if they trigger; `[skip ci]` on new backup commits).
- **WIP safety:** never `rm -rf`, never `reset --hard`, never `checkout -- .` on work in progress. Dirty work is committed to new backup branches first, leaving original branch pointers untouched.
- **No technical questions to Lucas** — ramifications and end-state only.
- **File ownership:** Transport owns `core/src/transport/`. One worktree per lane; never edit another lane's files. Lane 0 never touches lane-claimed worktrees/branches (registry on #602).
- **CI lane policy:** a previously-green lane stays green — investigate failures, never accept red.
- **Carried hard constraints:** local copies for PRs #489, #521, #550, #554, #556, #562 do NOT match their GitHub heads — never push over those heads. #489 stays held per the 2026-10-18 disposition (a backup branch duplicating it dispositions as "held", not "superseded").
- **Known ops risk:** #624's always-up node redeploy needs SSH/deploy access — Lane 3 verifies FIRST; if blocked, escalates to the Overseer and proceeds with #623/#625. Code never waits on ops.

## 6. Inconsistencies flushed (what was caught and resolved)

1. *Lane 4 vs WIP audit double-ownership of backup/disk* → Lane 0 owns it; Lane 4 is Ubicloud-only. Lane 4 prompt amended.
2. *Audit purging worktrees while lanes work in them* → phased rollout (audit inventory first, lane worktree registry on #602, purge excludes claimed paths).
3. *"One clean main clone" as a mid-mission done-criteria* → moved to Phase E post-tag; lanes need their worktrees until then.
4. *Sole-merger vs WIP lane "merge as needed"* → Lane 0 never merges to main; PRs/handoffs to Lane 1.
5. *Lane 1's embedded queue going stale* → #595 is authoritative; prompt list is a snapshot.
6. *Pinned main SHA in Lane 1 prompt* → verify current head from GitHub at launch.
7. *`~/workspace/...` paths invalid on the laptop* → inline Jev v1 summary fallback in every prompt.
8. *Lane 4's backup section assumed the 4 known worktrees at `~/workspace/scmessenger-work/`* → replaced by Lane 0's unknown-environment sweep (that path is a different machine).
9. *WIP backup branches had no merge path* → they feed #595 via "green and ready" handoffs; Rule-8 applies where the files require it.

## 7. Overseer verification checkpoints

1. After Lane 0 Phase 1: inventory posted on #602 is complete (spot-check SHAs on GitHub).
2. After each Lane 0 backup push: SHA resolvable via GitHub API; no CI ran on backup branches.
3. Before any purge: 100% of the worktree's content confirmed on GitHub.
4. Per handoff: PR head SHA matches, CI green on the exact head, Jev verdict present.
5. End state: §1 checklist, all from primary sources.

## 8. Launch checklist (Lucas)

1. Sync to the laptop: this plan + the 5 lane prompt files + `jev-gate-policy-v1.md` (or rely on the inline summary).
2. Post the two issue comments (#602 mission-issued, #595 coordination) — drafts ready, awaiting your word.
3. Paste Lane 0 prompt into its mission tab → wait for the Phase 0–1 inventory on #602.
4. Paste Lanes 1–4 prompts into their mission tabs → each registers worktrees on #602.
5. Lane 0 proceeds Phases 2–4 on your word after inventory review (or auto-continues per its prompt — your call at launch).

Prompt files: `scmessenger-wip-audit-burndown-prompt-2026-10-21.md` (Lane 0), `scmessenger-lane-1-merge-train.md`, `scmessenger-lane-2-fix-forwards.md`, `scmessenger-lane-3-transport.md`, `scmessenger-lane-4-infra.md`.

## 9. Rule-8 Review Protocol (Jev-driven, model-agnostic) — Lucas 2026-10-21

**Eligibility:** any model/agent that did not author the PR. Independence means non-author — the specific model does not matter.

**The bar:** Jev gate v1 PASS. Results-driven: the verdict is what counts, not who produced it.

**Flow:**
1. Authoring lane posts "review requested" on the PR: PR number, head SHA, what changed, CI link.
2. Reviewer (a different lane's model) pulls the actual code, verifies every claim against primary sources, flushes it out completely as a fresh objective viewer — then posts findings + Jev verdict as a PR comment.
3. Gaps known → findings go back to the authoring lane → iterate → re-request review. This is the normal loop, not a failure.
4. Truly stalled with no direction → Jev deep-dive diagnosis: which change direction would push the score higher? Identify the gap drivers, then iterate on that direction.
5. Incomplete review → reviewer posts findings-so-far → a secondary model picks it up and completes it.
6. **True pass = two independent non-author reviews, both Jev PASS.** The second reviewer re-verifies from the code — never a rubber stamp of the first.
7. True-passed PRs get the "green and ready" handoff to Lane 1 (for Rule-8 PRs, the handoff carries both review verdicts).

**Fallback chain:** lane model → another lane's model → Muse (final). Muse does the full fresh-objective review: pull the work, verify the claims, flush out completely.

**Default review swaps:** Lane 2 ⇄ Lane 3 (each reviews the other's); Lane 0's burn-down PRs → Lane 2 or Lane 3; Lane 4's workflow PR → Lane 2. If a request sits unclaimed or a review stalls, the Overseer assigns it.

**Distribution:** every lane prompt carries this protocol — any lane can act as reviewer, and the consciousness is the same everywhere: verify from the code, gate on Jev, iterate on known gaps, deep-dive on stalls, double-review for a true pass.

## 10. Smart launch + token discipline (Lucas 2026-10-21)

**Do NOT launch all lanes at once.** Activate on demand:

1. **Lane 0 first, alone** (Phase A, read-only audit). Wait for the #602 inventory.
2. **Overseer triages the inventory + #595 queue**, then Lucas activates only lanes with real pending work:
   - Lane 1 if green-mergeable PRs exist; Lane 2 if red non-transport PRs exist;
   - Lane 3 if transport work is pending; Lane 4 whenever (independent, bounded).
3. **Standby rule:** a lane whose queue empties reports to the Overseer and goes DORMANT — no hot polling loops (polling burns tokens for nothing). Reactivation = Lucas pastes the prompt into a fresh tab (fresh context, no stale state). The Overseer flags "Lane X has N new items" on #595 when work reappears.
4. **Collision:** on-demand activation *reduces* collision surface (fewer concurrent writers). At each activation the Overseer confirms non-overlapping queues. Standing protections still apply: file ownership, one worktree per lane, sole merger (Lane 1), #602 worktree registry.

**Harness MCP for cheap decisions:** every lane routes judgment calls through the LOCAL harness MCP (Jev score/choice/noul) per the sovereign-harness skill — plan assessments, self-reviews, triage, deferral rationale, model/provider selection. MCP calls cost a fraction of a cent vs full lane sessions. Do NOT burn lane tokens on what a Jev call answers. Still lane-local: code writing, file ops, git ops, and claim verification (pull the code yourself; route only the verdict math through MCP).
