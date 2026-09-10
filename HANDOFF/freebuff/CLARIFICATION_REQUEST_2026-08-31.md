# Freebuff lane -- clarification request to CEO

Status: ANSWERED 2026-08-31 -- ruling in `inbox/RULING_2026-08-31_clarification_response.md`.
        Order changed to T5 -> T2 -> T1 (Half 2 only); T1 Half 1 withdrawn.
        Item 2 was correct and corrected T1's premise.
From: Freebuff lane agent (this session)
Date: 2026-08-31
Re: `HANDOFF/freebuff/README.md` queue (V040-T1/T2/T4/T5) and SHIP_PLAN section 6.4 G2

This file is the first outbound report on the Freebuff lane's bidirectional
channel, written at the operator's instruction: confirm anything I am less
than 99% confident about before starting the v0.4.0 queue. The repo is the
reliable channel (CEO_RULINGS_2026-08-16: "The repo is the reliable channel;
session messages are not.").

## Verified this session (>= 99%, commands run on main@69a8ba57)

- T1: `connect_to_seed_peers` has exactly one caller repo-wide --
  `core/src/mobile_bridge.rs:862`. The CLI node never dials. CONFIRMED.
- T4: `routing_peer_seen` has zero callers in real source. The "calls" in
  `HANDOFF_AUDIT/REPO_MAP.jsonl` are a stale AI-generated artifact, not call
  sites. CONFIRMED.
- T4: helpers `parse_transport_type` / `parse_peer_id_32` exist as private fns
  at `core/src/iron_core.rs:110` and `:126` (task said "module scope" -- they
  are module-scope in iron_core.rs, consistent with routing the feed through
  `IronCore::routing_peer_seen`; no second copy required).
- T5: `bash scripts/docs_sync_check.sh` exits 1 on clean main with exactly the
  filed error (`DiagnosticsBundleFormatterTest.kt` broken link). CONFIRMED.

## Items below 99% confidence -- each with my default if unanswered

### 1. First task order (~90%)

The docs file T1 first, but T1's Half-1 (seed promotion from `peers.json`) is
superseded by T2's real migration, and T5 is a 5-30 LoC fix that unblocks every
agent's red finalize gate. The operator deferred this decision to the CEO.

Options: T1-first (filed order) | T5-first | T2-first | T4-first
Default if unanswered: T1 first, per the filed queue order.

### 2. Live-evidence standard for T1/T4 acceptance (~70%)

The rig has moved since the task files were written: the local Windows node
(127.0.0.1:9876) now reports `connection_path_state: DirectPreferred` with 1
peer (the AWS node's libp2p id), not the filed `Bootstrapping` / `peers: []`
regression state. The AWS node is unreachable from this seat at the filed
address `54.235.20.24:9001` (address churn is expected per the 2026-08-31
ruling; its current address is unknown to me). Re-establishing the filed
regression case (fresh restarts at `69a8ba57` with cleared stores) is
operator/hardware territory.

Default if unanswered: unit tests are the acceptance gate; live evidence is
attempted where reachable and marked `UNVERIFIED` where it is not, per the
evidence contract.

### 3. Loop mechanics (~85%)

Confirm: the lane agent implements on a dedicated worktree (SHIP_PLAN 3.5) and
opens one PR per task; the CEO/operator merges (the lane may not self-merge)
and moves the task file to `done/` with the PR number on its Status line;
status reports land in the conversation thread. The agent does not edit
operator-created files (task files, queue README) without permission.

Default if unanswered: as stated above.

### 4. T5 evidence disposition (~60%)

If no surviving equivalent of the deleted `DiagnosticsBundleFormatterTest`
exists, I will downgrade/reopen the affected residual-risk row or mark it
`UNVERIFIED` per the task file's authorization, and state in one sentence what
happened to the evidence. Confirm the `UNVERIFIED` path is acceptable if the
coverage is genuinely gone.

Default if unanswered: the `UNVERIFIED` path is acceptable.

## Notes (informational, not questions)

- T2 and T4 PRs are Rule-8 merge-blocked until a fresh adversarial reviewer
  that did not author the change records APPROVE. The lane will flag each PR's
  review need; the review itself is a native-seat responsibility.
- Cross-node churn evidence (SHIP_PLAN G3-0) is operator/hardware territory;
  the lane cannot reach the AWS node from its seat.

REPLY WITH: T1-first | T5-first | T2-first | T4-first (item 1), plus any vetoes
on items 2-4 defaults. A directive file in `HANDOFF/DIRECTIVES/` or a ruling
appended here is equally fine -- the repo is the channel.
