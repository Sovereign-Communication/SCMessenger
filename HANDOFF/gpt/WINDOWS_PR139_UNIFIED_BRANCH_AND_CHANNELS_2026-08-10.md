# Windows lane -> GPT-MAC: unified branch strategy and coordination channels

Status: Active. Written 2026-08-10 by the Windows `/orchestrate` session.
Supersedes the branch anchors in `HANDOFF/gpt/PR139_ORCHESTRATION_STATE_2026-08-10.md`
lines 30-32, which are stale. Everything else in that document still stands.

This file needs a GPT-MAC ACK before either lane executes against it. Reply in
PR 139 comments or amend this file directly.

## 1. Branch strategy -- RESOLVED, one branch

The GPT-MAC state doc names `codex/pr139-five-node-gate-fixes` as "the candidate
branch". That is no longer true and following it will split the lanes.

Verified on this host, 2026-08-10, against freshly fetched `origin`:

| Branch | Tip | Contained in PR 139 head? |
|---|---|---|
| `tracking/pre-v040-tag-work` | `e5284b7b` (remote) | this IS the PR 139 head branch |
| `codex/pr139-five-node-gate-fixes` | `49bc3f56` | [OK] fully merged, no commits ahead |
| `android/pr139-transport-durability` | `96194e06` | [WARNING] NOT merged, 4 commits ahead |

`gh pr view 139 --json headRefName` returns `tracking/pre-v040-tag-work`. Every
candidate SHA the Mac lane has been citing -- `68fcc3f1`, `4083e59b`, `acda09df`,
`c6420b3a`, `e873ed4a` -- is already an ancestor of it. There is no divergence to
reconcile on the Codex side.

**THE RULE, both lanes, from now on:**

- One PR: **#139**. One integration branch: **`tracking/pre-v040-tag-work`**.
- All work branches merge INTO that branch. No lane merges to `main`.
- Handoff and orchestration state (`HANDOFF/**`, queue, audits, reviews, plans)
  commits directly to the integration branch. It carries no runtime risk.
- Functional code lands via a work branch merged into the integration branch,
  never pushed straight to `main`.
- Cite the integration-branch SHA in every evidence report, not a work-branch SHA.

## 2. Stranded work -- ACTION REQUIRED

`android/pr139-transport-durability` (worktree `C:/Users/SCM/Documents/GitHub/scm-android-gate`)
holds four commits that are NOT in PR 139, two of which are real code:

```
96194e06 handoff: P1 contact recovery writes the PeerId into the public_key field
d6ae8490 test(android): cover onServiceLost self-guard and disconnect regression
fd7655fa fix(android): guard mDNS service-lost against the local peer id
caa8be18 handoff: file Android transport defects and the unified master merged plan
```

`fd7655fa` guards mDNS service-lost against the node's own peer id. That is
plausibly load-bearing for the address-churn and self-address symptoms both lanes
have been chasing (the Mac lane's `[DIAL-BACKOFF]` observations, the PR-thread P1
finding that the ledger binds peer identities to addresses that are not theirs).

Windows lane proposal: merge this branch into the integration branch, gated on a
local Android build, BEFORE any further five-node gate attempt. Reason: running a
gate against a head that is missing a known transport fix burns the run.

GPT-MAC: confirm or object before the merge. This is a consensus item.

## 3. Coordination channels

Two channels, both required. Neither alone is trusted.

**Channel A -- SCMessenger CLI (primary, currently DOWN).**
Windows CLI node is not running as of this writing. Scheduled task
`\SCMessengerSoak` last ran 2026-08-10 06:17:09 local and exited
`-1073741510` (`0xC000013A`, `STATUS_CONTROL_C_EXIT`). Restoration is in flight
in the Windows lane. Until `/version` answers on this host, treat every SCM CLI
send from Mac to Windows as UNDELIVERED, including
`8a16beb5-4f2a-4844-b913-70c4cd35a726`. Do not resend it; do not count it.

**Channel B -- PR 139 comments (fallback, always on).**
Every confirmed result gets mirrored here regardless of Channel A health. A PR
comment is the durable record; the CLI is the fast path.

Evidence standard is unchanged and applies to both channels: a CLI `accepted`
response is not delivery. A handoff is confirmed only by receiver-side
`inbox_receive` plus a delivery ACK carrying the exact message ID.

## 4. No CI -- local gates only

The repo no longer has GitHub Actions runners available. `gh pr checks 139` is
not a gate any more and neither lane should wait on it or cite it. All gates run
locally on the lane that owns the platform:

- Rust: `cargo test --workspace --no-run`, `cargo fmt --all -- --check` (Windows).
- Android: `cd android && ./gradlew assembleDebug -x lint --quiet` (Windows).
- iOS/macOS: owned by the Mac lane; Windows cannot verify these and will not try.

Disk discipline: this host is at 94% on C: with roughly 17 GB free. Builds are
serialized, one at a time, and only when a gate actually requires one. No
speculative or duplicate builds. Never `cargo clean --target <triple>` here -- it
wipes the whole `target/` tree.

## 5. Consensus protocol (operator directive, 2026-08-10)

Neither lane executes a plan the other has not signed off on.

1. Proposing lane writes the plan to `HANDOFF/gpt/` and mirrors a summary to PR 139.
2. Reviewing lane returns explicit `CONCUR` or `OBJECT` with reasons.
3. On `OBJECT`, the plan is revised and re-proposed. No execution in between.
4. Work executes only after `CONCUR`.
5. The lane that did NOT do the work validates the result against the stated
   acceptance criteria. Self-validation does not close an item.

## 6. Open items for GPT-MAC

1. `CONCUR`/`OBJECT` on the single-integration-branch rule in section 1.
2. `CONCUR`/`OBJECT` on merging `android/pr139-transport-durability` (section 2).
3. Confirm you have dropped `codex/pr139-five-node-gate-fixes` as an anchor and
   are reading `tracking/pre-v040-tag-work`.
4. State the current live iOS PeerId, or state that iOS remains unconfirmed. The
   roster in the previous state doc still lists node 5 as pending.
5. Acknowledge that CI is unavailable and that your lane's gates are local.
