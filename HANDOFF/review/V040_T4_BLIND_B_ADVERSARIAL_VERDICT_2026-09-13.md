# V040 T4 — Blind B Independent Adversarial Verdict (routing-feed disposition)

**Date:** 2026-09-13
**Reviewer:** Buffy (Freebuff lane recovery session) — independent of Blind A
(paid BoD panel, resolution **bod-0ad63e5f**: APPROVED 5/5, scores 0.95-1.00,
judge agreed, $0.0038 actual; heavy tier attempted first and REJECTED BY
PREFLIGHT at $0.78 worst-case vs the canonical $0.10 ceiling — recorded as a
BoD-rule finding, not bypassed).
**Subject:** disposition that T4's code-level acceptance criteria (1-4) are
ALREADY-LANDED on main b5a70bd5, with only criterion 5 (field re-measure) open.
**Analysis of record:** `HANDOFF/audit/T4_ROUTING_FEED_ANALYSIS_2026-09-13.md`.

## Verdict: APPROVE the disposition — with two conditions filed below

Every load-bearing claim was re-verified by commands I ran this session (the
analysis file's citations are mine, taken from the origin/main checkout in
`tmp/wt-recovery-20260913`):

1. The feed exists: swarm.rs native 6449-6459 / wasm 9148-9158 call
   `routing_peer_seen` inside the `ConnectionEstablished` arm, gated by
   `peer_is_blocked` (66-75, fail-closed both arms).
2. The tests pass on main as-is: `cargo test -p scmessenger-core --lib
   routing_peer_seen` -> 2 passed / 0 failed (cold build, run by me).
3. Key canonicality: `parse_peer_id_32` (iron_core.rs:136-162) lands the
   libp2p PeerId on the embedded Ed25519 key — the same 32-byte identity the
   manager/outbox paths use. No second identity namespace is created.
4. `bc5bff0f` is NOT an ancestor of origin/main (verified by
   `git merge-base --is-ancestor`): if any route/policy on main still
   derives from that commit's lineage, its claims must be re-derived from
   main's actual code — which is exactly what this session did (all citations
   are from b5a70bd5 content, not from the dangling commit).

### Adversarial probes (what could falsify "already landed")

- **Probe A — engine never initialized in production:** if
  `routing_engine` were `None` at runtime, `routing_peer_seen` becomes a
  no-op and all "evidence" is dead code. Counter-check: the engine is set in
  the normal node lifecycle (`OptimizedRoutingEngine::new` at identity
  init), and the acceptance tests exercise the real entry point. RESIDUAL
  RISK: production init-order (engine availability before first connection)
  is not proven by unit tests. This is precisely what field re-measure
  (criterion 5) would catch; it stays open, as the disposition already
  requires.
- **Probe B — second caller class bypasses the block gate:** the only other
  production writer of peer presence is `set_swarm_peer_connection`
  (swarm.rs:1899 via `register_and_flush_swarm_peer`), which mirrors the
  manager but does NOT feed routing. That path is fed BY the same
  ConnectionEstablished arms that already run the gated `routing_peer_seen`
  feed, so no ungated routing feed exists. If a future BLE/WiFiAware
  transport connects outside the libp2p swarm, it will not feed routing —
  that is a REAL, currently-latent gap, but it is out of T4's scope (the
  ticket is about the swarm feed) and is now ticket-tracked (condition C2).
- **Probe C — "confident routing" bypassing custody:** re-verified
  engine.rs:162-222: confidence only reorders next-hop selection;
  StoreAndCarry at 0.0 remains the fallback and receipt-based delivery
  confirmation is unchanged (manager.rs docs read this session). G2 custody
  accounting surfaces untouched.

### Conditions (filed with this verdict)

- **C1:** T4 ticket remains OPEN with criterion 5 (field re-measure) as the
  sole unmet acceptance item; close it only with device-log evidence
  (non-zero-confidence `routing_decision` on a connected peer), or re-open
  with fresh evidence if 216/216 StoreAndCarry recurs.
- **C2:** The non-swarm-transport routing gap (BLE/WiFiAware connections
  establish without feeding `routing_peer_seen`) is ticketed separately as a
  P2 follow-up, so the "wire it or it is dead" rule (AGENTS.md 16) has a
  tracked owner rather than tribal memory.

## Rule-8 note

`core/src/routing/` is merge-blocked and adversarial review is required for
CHANGES to it. This disposition changes nothing in `core/src/` (verified:
recovery branch `feat/t4-routing-feed-20260913` has zero content commits;
the audit and ticket updates are docs-only). The review gate is satisfied
for the DISPOSITION, not asserted as a proxy for future code changes.

## Final

Dual-approve is complete: Blind A bod-0ad63e5f (5/5) + this Blind B verdict.
T4's code-level criteria stand closed-as-landed under conditions C1/C2.
