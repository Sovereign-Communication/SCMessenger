# GPT-MAC takeover packet -- PR #139 / v0.4.0 + v0.5.0

**From:** GPT-Windows (Codex, 5.6 Luna lane)
**To:** GPT-MAC / MAC LANE
**Date:** 2026-08-07
**Repository:** `Sovereign-Communication/SCMessenger`
**PR:** #139, `tracking/pre-v040-tag-work`
**Current PR head:** `5b8b8e7b68da62f5b20fa2401517e0f8a2763bd5`

> **Head re-pointed 2026-08-07 (Windows lane).** This packet originally named
> `57c5d6a4` and instructed a build from `cabc0473`. Both are now stale by
> several commits. Build from `5b8b8e7b` (or the then-current PR head), which
> additionally carries `d9099f3d` IronCore delivery/routing state handling,
> `abb32d45` relay reputation manager wiring on identity init, and `5b8b8e7b`
> the canonical peer id FFI surface snapshot. All 31 CI checks are green at
> this head, so no Windows follow-up commit is pending -- the Mac lane is not
> waiting on the implementation lane.

## Current Windows state

- Hermes has finished its shared-worktree run.
- T1 dual-flavor block storage, T2 curve-point identity derivation, T3
  fail-closed pending requests, and T4 centralized block resolution are in
  the PR head.
- Windows gates passed after the handoff: `cargo fmt --all -- --check`, 21
  targeted blocked-manager tests, and 4 CLI message-request integration tests.
- The Windows checkout has a small local CLI request-key correctness delta in
  `cli/src/server.rs`; do not use or modify that local delta from the Mac lane.
  Fetch the PR head above into the Mac checkout, then wait for the Windows lane
  to publish the follow-up commit if the platform result depends on it.
- No GPT-MAC Codex task is visible to GPT-Windows on this host. This file is
  the coordination handoff; write the result below when the Mac lane finishes.

## MAC LANE scope

Use a separate Mac checkout/branch. Do not edit Rust core, merge, tag, or
rewrite the Windows worktree. The Mac lane owns platform verification and
evidence:

1. Build the macOS CLI from PR head `5b8b8e7b`; launch it using the existing
   data directory and record the actual version/git provenance line.
2. Build and install the iOS app in place from the same PR head. Do not
   uninstall, wipe identity data, delete contacts/history, or re-pair.
3. Confirm identity, contacts, and history survive the update; confirm the
   outbound identity is the canonical public-key form.
4. Where the fleet is available, run both message directions with receipts,
   ledger/fleet visibility, and the BLE/LAN liveness checks. Record actual
   transport and restart/reconnect evidence rather than inferred success.
5. Use only the current relay address from
   `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`; treat every older IP as stale.

Write the result to:

`HANDOFF/gpt/IOS_MACOS_PR139_STATUS_2026-08-07.md`

Use this result format:

```text
RESULT: DONE|BLOCKED|FAILED
HEAD: <actual commit built>
MACOS_CLI: <provenance and startup evidence>
IOS: <build/install/provenance/data-preservation evidence>
IDENTITY: <canonical identity evidence>
FLEET: <message/receipt/ledger/transport evidence or explicit N/A>
BLOCKERS: <exact external blocker, if any>
```

If a platform or physical-device step is unavailable, stop at that step and
report the exact blocker. Do not substitute a stale CI artifact or a source
inspection claim for runtime evidence.
