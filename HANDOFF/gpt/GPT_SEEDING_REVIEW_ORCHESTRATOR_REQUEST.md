# GPT HANDOFF -- unblock seeding review and grant branch autonomy

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ACTION REQUIRED BY WINDOWS ORCHESTRATOR
Created: 2026-07-28
Requester: GPT-5.6 Sol Codex desktop session on the operator's MacBook
Current task: `HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES.md`

## What was checked

The Mac session refreshed `origin/main` and all advertised remote branches at
2026-07-28 11:13 HST. The newest remote commit remained:

```text
2733de5c native: GPT lane rules -- branch/push/PR autonomy; seeding review packet
```

No remote branch contained the promised Wave 1b fixes for F10, F7(a), F7(b),
F13, or NEW-6. The review packet still contains the literal
`[ORCHESTRATOR: insert git diff ...]` placeholder, and the promised symbols
such as `MAX_LEDGER_ENTRIES` and `annotate_identities_batch` are absent at
`origin/main`. There is therefore no fix diff that GPT can honestly review yet.

## Action 1 -- publish an immutable review target

Please complete all of the following:

1. Push the compile-gated Wave 1b fixes to `main` or to a named remote review
   branch.
2. Update `HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES.md` from `AWAITING DIFF
   INSERTION` to `READY`.
3. Put the exact full commit SHA and its parent SHA in that packet.
4. Replace the placeholder with the exact diff for:
   - `core/src/store/ledger_entry.rs`
   - `core/src/mobile_bridge.rs`
   - `core/src/transport/swarm.rs`
5. State whether GPT should review the single commit, a commit range, or the
   final tree at a named SHA. The immutable SHA/range is authoritative if the
   embedded diff and remote tree ever disagree.
6. Notify the Mac session only after the named SHA is fetchable from GitHub.

Do not ask GPT to infer or reconstruct the fixes from their prose description.
The requested deliverable is an adversarial diff review, so the exact code is a
required input.

## Action 2 -- reconcile the repository rules for GPT branch autonomy

Commit `2733de5c` and the task packet say this Mac GPT session may commit, push
its own `gpt/*` branches, and open/manage its own PRs. The canonical
`AGENTS.md`, however, still says:

- hard rule 5: `NEVER git push`;
- `FOREIGN WORKER`: do not commit or push;
- `FOREIGN WORKER`: final output must use the short worker report format.

`AGENTS.md` declares itself canonical for non-Claude agents, so those rules
override the task-local permission and currently prevent the requested
branch/PR workflow. Please update the canonical policy, not only another GPT
handoff file.

Requested least-privilege lane:

```text
GPT DESKTOP (Codex on the operator-authorized MacBook)
- May fetch and create/switch local branches named gpt/*.
- May commit work produced by this session on its own gpt/* branch.
- May push only refs/heads/gpt/* and set upstream for those branches.
- May open and update PRs from gpt/* into main.
- May respond to review feedback with additional commits on the same branch.
- Must run the repository hook; never use --no-verify.
- Must never push directly to main, merge a PR, tag a release, move HANDOFF
  task files to done, or alter queue status.
- Core Rust changes remain subject to the repository security-review gate.
- Windows build and physical-device results remain the only authoritative
  verification.
```

Please also revise hard rule 5 so it does not contradict this scoped lane, for
example by prohibiting direct pushes except where a named capability class
explicitly grants branch-only push authority.

## Acceptance signal back to GPT

Reply with all four items:

```text
READY
REVIEW_TARGET: <full SHA or parent..tip range>
REMOTE_REF: <main or refs/heads/...>
GPT_POLICY_COMMIT: <full SHA that updates canonical AGENTS.md>
```

Once both commits are remotely fetchable, GPT can review, write
`HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES_VERDICT.md`, commit it on
`gpt/seeding-review`, push that branch, and open the PR without the operator
manually carrying files between computers.
