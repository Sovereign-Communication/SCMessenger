<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / MT-07
Type: BLOCKED

This handoff is an SCMessenger-only record for Sovereign-Communication/SCMessenger.

## Current PR evidence

- `gh pr view 413 --json headRefOid,baseRefName,state,mergeStateStatus` returned OPEN, base `main`, head `5c80035f4d45d850abe409e54d53c97392161aa1`, CLEAN.
- `gh pr checks 413` reports 37 checks, all pass, including the five currently required contexts and the WASM job. This is CI evidence only; no local build ran.
- `git -C C:/Users/SCM/Documents/GitHub/wt-mt07 diff --stat origin/main...HEAD -- core/src/transport/swarm.rs` reports +41/-0 in the native loop. The branch's `core/src/transport/swarm.rs` diff contains no wasm changes.
- Ticket `HANDOFF/todo/OUTBOX_NO_PERIODIC_RETRY_SWEEP_2026-09-23.md` explicitly makes native sweep, wasm sweep, live TRI-040 R9 evidence, and mandatory Rule-8 review acceptance criteria. Its newly appended Fix text accurately records the wasm item unmet; it does not waive it.

## Premise found in the proposed wasm resume plan

The plan to implement wasm parity with the existing `Date::now()` inline-check convention is not an idle-period retry timer. In the actual event loop, `futures::select!` waits on `command_rx.recv()` or `swarm.select_next_some()`; the existing elapsed-time checks are after the select body. They execute only after a command or swarm event, so with no such event the 120-second outbox sweep cannot run. The ticket's stated failure is precisely a stranded entry on a stable connected peer that may not emit an event. Calling this equivalent to the native Tokio interval would not meet that behavior. No new timer dependency, architecture change, or silent weakening of the acceptance criterion was attempted.

The same source plan also proposes adding the F5 retained-bound connection trim to MT-07. The current branch does not implement it. `HANDOFF/review/D1_D9_TRANSPORT_ADVERSARIAL_FINDINGS_2026-09-22.md` defines F5 as an accepted, non-exploitable wasm parity follow-up; it does not establish that the F5 item belongs to MT-07's acceptance. The original MT-07 ticket asks for wasm sweep parity only. Do not expand scope by treating F5 as an already-authorized MT-07 requirement without resolving that boundary.

## Rule-8 review attempt and stop

The wasm proposed delta touches `core/src/transport/swarm.rs`, so A2 Rule-8 is mandatory. Prepared an independent, read-only Claude sign-off request and ran the documented isolated-review helper from the worktree:

`python <independent-review-helper> --model sonnet --budget 1.0 --prompt-file <read-only-review-packet> --cwd C:/Users/SCM/Documents/GitHub/wt-mt07`

Result: `{"ok": false, "result": "Failed to authenticate: OAuth session expired and could not be refreshed", "cost_usd": 0, "num_turns": 1, "permission_mode": "dontAsk", "is_error": true, "error": "Failed to authenticate: OAuth session expired and could not be refreshed"}`. No independent sign-off was produced. First-pass adversarial analysis was not run: the first A2 step cannot clear the mandatory second step, the work is not presently an acceptance-complete candidate, and I will not solicit a pass on the unchanged native-only patch.

Status: no Rule-8 approval, no wasm/F5 code change, no merge. Await operator direction on the wasm idle-wakeup mechanism and whether F5 is in or out of MT-07 scope, plus fresh Claude authentication for the required independent sign-off. The ticket's acceptance and A1 gates remain unchanged.