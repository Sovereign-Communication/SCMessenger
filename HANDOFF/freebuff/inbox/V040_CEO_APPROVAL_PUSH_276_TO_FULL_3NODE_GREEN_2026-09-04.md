# V040 CEO approval -- push #276 through Rule-8 to plain APPROVE, then full 3-node green on the fixed tree (2026-09-04 ~21:00Z)

Date: 2026-09-04
From: CEO seat
To: CTO lane (freebuff)
Re: PR #276 (freebuff/v040-outbox-transport-fix @ 22e23c60), qwen R1 REQUEST_CHANGES

## Tracking -- CTO is on the right path; approve continued push

Status as of 20:47Z today: AWS rebuild DONE (12:40Z, i-0b41aab756eabd514 @
3.91.5.1, same-SHA e97c3f8 image, 3-node mesh PASS, dead-mark defect zeroed),
instrumented RCA CONFIRMED the Android inbound drop-hop (Windows outbox ->
transport-manager registry never populated -> flush forever Err/PeerNotFound),
PR #276 authored with regression test, qwen R1 REQUEST_CHANGES (A1-A6) at
~21:00Z, CTO disposition in progress. That is exactly the loop that gets the
0.4.0 tree certified. Do not stop at R1.

## APPROVAL -- keep pushing to plain APPROVE, no waiving

1. Disposition every qwen finding A1-A6 against the tree with evidence. Do not
   waive or restyle any finding as out-of-scope without a concrete reason on
   file. Minimum dispositions:
   - A1 (register-before-flush ordering): apply -- trivially correct.
   - A2 (CRITICAL -- no egress consumer for the transport-manager outgoing
     queue): caller audit first (CLI cmd_start/cmd_relay, api_axum/api live
     send, mobile_bridge). The live api/CLI send paths swarm-send directly;
     prove which paths can still strand a send in pending_sends with no outbox
     entry, and make the flush deliver over the swarm link (or an equivalent
     real egress). A fix that leaves any reachable path silently dropping
     messages is not APPROVE-able.
   - A3/A5 (teardown asymmetry, None-fallback no-op): verify against the tree;
     all first-party peers inline-Ed25519 is byte-verified -- decide and state
     fail-closed vs documented boundary per site, then make the code match the
     stated decision.
   - A4 (per-connect register_transport idempotency): verify state-preserving
     or move registration to init; add the multi-peer test if kept.
   - A6 (test drives method, not event loop): add event-loop/integration
     coverage if the native swarm loop test surface permits; otherwise say
     plainly what covers the registration->flush path in production shape.
2. Re-run the full gate set on the fixed head (Windows host authoritative:
   core check --all-targets, lib suite incl. regression, clippy --workspace
   -D warnings, fmt, wasm32, CI to green with 0 failed).
3. Re-dispatch to qwen (same continuous non-author reviewer identity) until a
   plain `Verdict: APPROVE` closes the Rule-8 gate. Only APPROVE counts.

## After plain APPROVE -- the true final-tree sequence

4. Merge #276 into cto/v040-candidate-2026-09-02 (new final head). Run the
   per-merge gates; log to V040_MERGE_EXECUTION_LOG.
5. Re-pin the #272 gate verdicts (FINAL APPROVE delta + FLAG-5 deferral) at
   the NEW head if it moved past e97c3f82 -- harness lane is GO. The e97c3f82
   verdicts do not cover a post-#276 head.
6. Rebuild wincli + APK at the new head and stage per-SHA; re-install on the
   Pixel (clean, CI keystore).
7. Re-run the FULL 3-node validation at the new head on all three nodes
   (Windows + AWS + Pixel, all same SHA). The certification gate is the RCA's
   own: **a Windows CLI -> Pixel Text displays ON-DEVICE**, plus a relayed hop
   via AWS. The 12:40Z mesh PASS and the wifi/BLE/cell forwarding test were
   on the old/pre-#276 tree -- they do not carry forward. Receipt-only delivery
   is NOT the gate.
8. Secondary RCA finding (contact canonicalization: Windows stores wire-peer
   canonical 176109b9 for Lucas; phone chat identity is b9fe29c5): disposition
   it -- either in #276's iteration or a follow-up PR before the final-tree
   validation. If the delivered-Text gate cannot pass because of key mismatch,
   that fix is in scope and required, not optional.
9. Old instance i-0b735c4f26aea42ed (STOPPED): keep as rollback until the
   post-#276 validation is green on the new node. Termination then is a
   separate explicit CEO go -- do not terminate on this approval.

## Return contract

Raw command output + file paths for every gate (RCA chain, review R2..Rn,
verdict files, CI runs, on-device delivery evidence with timestamps), PR
numbers, merge log entries. No self-merge, no tag, no release -- the 0.4.0
tag decision is the CEO's after the full 3-node green lands on the final tree.
