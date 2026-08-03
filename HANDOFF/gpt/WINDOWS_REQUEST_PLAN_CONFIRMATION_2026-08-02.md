# Windows -> GPT: confirm the post-green plan (AFTER PR 129 merges)

Status: Queued -- do NOT start until PR 129 is merged to main
Last updated: 2026-08-02
Requested tier: **GPT-5.6 Sol Ultra** -- this is design judgement on a
dependency order and a still-unsolved concurrency bug, which is the one thing
worth spending the scarce tier on. Do NOT burn Sol Ultra on the mechanical
items in the plan; route those to 5.4 mini or to the Qwen lane.

Operator directive: confirm the plan only AFTER the merge lands, so it is
validated against the post-merge tree rather than a moving target.

## What to review

`HANDOFF/POST_GREEN_PLAN.md` (408 lines, authored by the Qwen lane, spot-checked
by Windows against source -- line references verified accurate).

## The specific judgement calls we want a second opinion on

1. **Is the dependency order right?** The plan puts
   `eprintln! -> tracing` FIRST, ahead of the critical BLE bug, on the grounds
   that 15 `eprintln!` calls in `core/src/mobile_bridge.rs` never reach Android
   logcat (stderr is discarded), so every diagnostic inside `on_data_received`
   is invisible and the bug cannot be localised without them. We think that is
   correct and non-obvious. Challenge it if you disagree.

2. **The BLE wedge root cause is still UNSOLVED.** This is the real ask.
   Evidence, all verified on device:
   - `mesh_ble_forward` logged immediately before `onDataReceived(...)`,
     `mesh_ble_forward_return` immediately after. Device shows 264 entries / 0
     returns, then 46 / 0 on a fresh buffer. It never returns.
   - Synchronous UniFFI call on the BLE GATT callback thread:
     `MeshRepository.kt:2836` -> `mobile_bridge.rs:1385` ->
     `iron_core.rs:2994 IronCore::receive_message`.
   - The block is strictly between `iron_core.rs:2994` and `:3162`. Proven: the
     Kotlin delegate's first statement logs "Message from" and that string
     appears ZERO times.
   - Exactly ONE message got through earlier, then everything wedged.

   ALREADY REFUTED -- do not re-propose:
   - Re-entrant deadlock via the delegate callback. The Kotlin delegate does
     `repoScope.launch{}` and returns immediately, and it is never reached.
   - Simple ABBA between send and receive: both take the same order, and the
     receive path RELEASES `ratchet_sessions` at :3055 before taking
     `audit_log` at :3154, whereas the send path holds both together at
     :754-755.

   ALREADY ELIMINATED by Windows:
   - Sled disk-full. The device has 25 GB free (79% used), so a full-disk stall
     inside `inbox.write()` / `audit_log.write()` is not it.

   The plan's three candidates are: (A) `identity.read()` starvation by a
   pending `identity.write()` -- parking_lot is writer-preferring, so a queued
   writer blocks new readers; (B) `ratchet_sessions.write()` held by the send
   path across `encrypt_with_ratchet_fallback` while the outbox retries every
   ~8s; (C) sled I/O contention (non-disk-full variants).

   **We want your ranking and, more importantly, any FOURTH candidate we have
   missed.** Candidate A is the one we find most plausible, because it is the
   only one that explains a PERMANENT block rather than a transient stall -- but
   we have not proven the writer that would be queued.

3. **Is anything in the plan already stale or wrong post-merge?** Especially
   items that assume files PR 129 touched (`core/src/transport/addr_filter.rs`,
   `core/src/transport/swarm.rs`, `core/src/mobile_bridge.rs`,
   `MeshRepository.kt`).

## Also queued for the Mac lane (tier 5.4 mini, mechanical)

Still outstanding from the earlier handoff and not yet answered:
- iOS-side logs for the BLE test window.
- macOS CLI node running and LOGGING, with a verified port BIND (netstat/ss
  matched to PID plus the real log line). Do not accept exit code 0 as proof:
  this repo has eight confirmed cases of code reporting success for work it
  never performed, including a CLI catch-all that printed "executed
  successfully" for unimplemented commands.

## Redaction

Repo is PUBLIC. No peer ids, public keys, BLE MACs or IP addresses in anything
you commit. Message ids and timestamps are fine.

## Reply

`HANDOFF/gpt/GPT_RESPONSE_PLAN_CONFIRMATION_2026-08-02.md`.
