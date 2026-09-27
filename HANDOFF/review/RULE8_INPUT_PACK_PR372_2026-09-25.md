<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# Rule-8 input pack for PR #372 (candidate-finding input, NOT a verdict)

Owner of this document: SCMessenger. Prepared 2026-09-25 by the coordination
seat for the INDEPENDENT reviewer the PR requires. The reviewer must be a seat
that did not author #372, #361, or the D1/D9 work. This document is a packet of
claims to verify and questions to answer; it is not a review, and it does not
satisfy Rule-8 on its own. The PR body itself states that no sign-off is
claimed and that the author cannot self-sign.

## Head under review

| Field | Value |
|---|---|
| PR | #372 `fix/361-review-blockers` |
| head | `a76d7d66d3c29bde722931847fceb4c5311b68d4` |
| base | `main` @ `d1c4a173` (merge base `d1c4a173`) |
| mergeable / state | MERGEABLE / CLEAN |
| checks | 33 pass, 0 skip, 0 fail, 0 pending (`gh pr checks 372`; `tmp/pr_disposition_ledger.md` 2026-09-25T09:52:06Z) |
| commits | 34 |
| files changed vs main | 71 (`git diff --name-only origin/main...pr372`) |
| gated files | 5 (below) |
| HANDOFF docs touched | 14 (all need scope blocks under the WIP gate) |

Gated files (`core/src/transport/`), with line deltas:
`behaviour.rs` +20/-5, `dial_policy.rs` +98/-3, `mod.rs` +1/-0,
`per_peer_cap.rs` +594/-0 (new), `swarm.rs` +160/-3.

Non-gated code also on the head: `core/src/mobile_bridge.rs` +32/-4 (routing
feed from platform data links), `cli/src/bin/conn-fanout.rs` (new, 220 lines,
a fan-out test client), `MeshServiceViewModel.kt` +19/-2 and its test +38
(lifecycle toggle fix). Vendored `libp2p-swarm` 0.48.0: 35 files, +13,368 (new
vendored crate). Docs: `AGENTS.md`, `SHIP_PLAN.md`, `docs/rules/*`,
`docs/runbooks/*`, 14 HANDOFF docs.

## What the change claims (verify each against the head)

1. Two-tier per-peer cap: admission ceiling 8, retained bound 4.
   `per_peer_cap::ADMISSION_MAX_ESTABLISHED_PER_PEER = 8`,
   `RETAINED_MAX_ESTABLISHED_PER_PEER = 4`; `behaviour.rs` sets
   `with_max_established_per_peer(Some(8))`.
2. The admission ceiling is exactly the synthesised dial ladder width:
   `DIRECT_LADDER_ADDRS = 3 + 1` (ports 443/80/8080 plus `last_good`) plus
   `MAX_RELAY_LADDER_ADDRS = 4` = 8; a unit test pins the relationship in both
   directions.
3. The relay ladder is bounded: `RELAY_TRACKING_CAP = 8` tracked relays
   (newest-registration-wins) and at most 4 relays considered per target
   (recency-ranked), so a node with many relays cannot produce an unbounded
   candidate ladder.
4. The retained bound is enforced in BOTH the native and wasm event loops via
   the same helpers (`note_established_path`, `release_path`, `release_peer`),
   closing the least-recently-active path first (ties by establishment order).
5. Activity stamps: native stamps on request/response messages, ledger
   exchanges, and `Ping::Success`; wasm stamps on request/response messages,
   ledger exchanges, and `identify::Event::Received`. Every `ConnectionClosed`
   arm (partial and last) reclaims the path's activity entry.
6. The Android lifecycle toggle fix reads `meshRepository.serviceState.value`
   (the source) rather than the ViewModel's `WhileSubscribed` cache, and treats
   a tap during STARTING as "stop" and during STOPPING as "start".

## Questions the reviewer must answer with evidence

1. Is 8 the right admission ceiling given the ladder derivation, or does the
   dial path have another candidate source (ledger peers, mdns, DHT) that can
   push a legitimate fan-out past 8 and re-create the lockout at a higher
   number? Check `swarm.rs` dial construction beyond the ladder.
2. Does closing the least-recently-active path first ever close the path a
   handover depends on (Wi-Fi -> cellular) in a way that loses custody? Trace
   `ConnectionClosed` handling of the closed path: does the ledger exchange
   re-fire, and does a close during an in-flight request leave the peer with
   fewer than `DIRECT_LADDER_ADDRS` usable paths?
3. Is the newest-first relay cap exploitable: can a node register 8+ relay
   peers to displace legitimate relays, and what is the registration path
   (`add_relay` callers) and its authentication requirement?
4. The `mobile_bridge.rs` routing-feed change calls `routing_peer_seen` from
   every platform data link (BLE, Wi-Fi Aware, Wi-Fi Direct) after a
   block-status lookup. Does this change the semantics of routing confidence
   (does every received frame now raise a peer's confidence), and is the
   lookup fail-closed on error (it is: `Err` skips the feed)?
5. The Android STARTING/STOPPING toggle behaviour: is "tap during STARTING
   stops" a product decision or an implementation convenience, and does the
   foreground service serialize the two commands as the comment claims? The
   comment asserts serialization; the diff does not show it. Check
   `MeshForegroundService.kt` on the head.
6. The vendored crate: is `libp2p-swarm` 0.48.0 a version the project can
   support (license, MSRV, upstream CVEs), and are the vendored edits (tests
   and benches removed, doc-tests disabled) complete and auditable?
7. `conn-fanout.rs` ships as a CLI binary; is a 220-line test client in the
   release binary set acceptable, or should it be behind a feature or moved?

## Evidence commands (run in a worktree, not the shared checkout)

```
git fetch origin pull/372/head:pr372
git diff --name-only origin/main...pr372
git diff origin/main...pr372 -- core/src/transport/per_peer_cap.rs core/src/transport/dial_policy.rs
git diff origin/main...pr372 -- core/src/transport/behaviour.rs core/src/transport/mod.rs
git diff origin/main...pr372 -- core/src/transport/swarm.rs
gh pr checks 372 --repo Sovereign-Communication/SCMessenger
```

CI is the verifier (disk budget BLOCKED on this host; no local cargo). The
verdict file must record: reviewer seat, head SHA reviewed, each of the seven
questions with its answer, and APPROVE or BLOCK. A JEV panel or any candidate
tool is input, never the verdict.
