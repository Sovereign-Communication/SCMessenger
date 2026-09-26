# E-02 Adversarial Review: force_ratchet deletion

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Date:** 2026-07-17
**Reviewer:** Qwen THINK (qwen3-235b-a22b-thinking-2507)
**Verdict:** PASS
**Fusion Judgement:** UNANIMOUS PASS (3-panel: qwen3-235b, deepseek-v3.1, llama-3.3-70b; judge qwen3-235b)

## Diff reviewed

Pure deletion of `RatchetSession::force_ratchet` (ratchet.rs:662-685) and
`IronCore::force_ratchet` wrapper (iron_core.rs:3476-3487). Comment header
updated from "force ratchet, receiver session creation" to "receiver session
creation."

## Findings

1. **Regression risk:** NONE. Exhaustive grep across .rs/.kt/.swift/.udl
   confirmed zero callers. 10/10 ratchet tests pass post-removal.
2. **Semantic safety:** No orphan fields or broken implementations. All
   struct fields remain fully utilized in other ratchet operations.
3. **Crypto soundness:** No impact on state machine integrity. The method
   was never invoked in production. dh_step_count behavior remains
   consistent with normal DH step processing.
4. **Comment accuracy:** Updated comment correctly reflects remaining
   functionality (create_receiver_session).

## Pre-removal evidence

- grep `force_ratchet` across all .rs: 3 matches (all definition sites)
- grep in cli/src/: 0 hits
- Not in core/src/api.udl (not UniFFI-exposed)
- cargo check --workspace: clean
- integration_e00_drift_v2_diag: 2/2 PASS
- integration_e00_ratchet_wiring: 2/2 PASS
- integration_pq_session: 6/6 PASS

## Fusion Lite panel verdict

All three panelists unanimously recommended REMOVE. Rationale: dead code
with latent PQ defect (hardcoded None for pq_ss in suite 0x02), functionality
already covered by perform_pq_ratchet_step + encrypt_with_ratchet_fallback
(E-00 wiring), removal reduces attack surface for v1.0.0.
