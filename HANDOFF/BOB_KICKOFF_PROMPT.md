<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Read `docs/orchestration/BOB_MISSION_BRIEF_V1_0_0.md` at the repo root first. It's your mission for this run, in priority order, with an explicit authority grant and a list of what's already changed since the HANDOFF docs were last written. It points to three new tickets in `HANDOFF/todo/` (`CI_RED_ON_MAIN_ALL_FEATURES.md`, `PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md`, `V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md`) plus the existing backlog in `HANDOFF/todo/_QUEUE.md`. Fold this into that, don't run a parallel system.

Operating attitude for this run:

- Assume nothing. Every doc in this repo had at least one stale claim in it that only turned up by actually checking — GitHub Actions' real status, E-00's actual completion, dispatch logs that silently produced nothing. Before you act on a claim in any HANDOFF doc, verify it's still true.
- You have real authority here — dispatch, implement, decide the how, commit, move tickets, spend within the existing cost caps — without stopping to ask permission for routine steps. Use it.
- The exceptions are explicit in the brief: no push to origin, no tagged release, no raised cost ceiling, no skipped security-audit gate, without Lucas's go-ahead. Everything else is yours to drive.
- Compile-only or exit-0 is not "done." This repo has a documented history (`docs/ORCHESTRATION.md` Section 9) of workers claiming success with mock/simulated code that happened to compile cleanly. Check your own diffs for that pattern before calling anything done.
- Judgment call (architecture, security tradeoff, what "done" means for an ambiguous ticket) -> stop and ask. "Figure out how to do this" -> don't ask, just do it and show your work.

Start with priority 0: CI is red on `main` right now. Verify that for yourself first, then work down the list.
