# TASK: PQC-09 — Hybrid Onion Routing Investigation (Spec + Design)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: READY FOR QWEN DELEGATION
Owner: Qwen (investigation + spec writing)
Scope: Phase 2 PQC depth work (low priority, informational)

## Objective

Investigate hybrid onion routing (X25519 + ML-KEM-768) privacy enhancement for v1.0.0. Current onion routing in `core/src/privacy/` is X25519-only (PQC-01 work). PQC-09 adds ML-KEM layer for PQ-resistant privacy.

## Current State

- `core/src/privacy/` — onion routing module exists, X25519-based
- PQC-01 completed (X25519 onion infrastructure)
- PQC-09 FROZEN until PQC-05/06/07 adversarial review completes (currently in-flight)
- Master plan: PQC-09 is wave 3 after PQC-07/08 land

## Investigation Scope

1. **Current onion routing (X25519):**
   - File: `core/src/privacy/onion_routing.rs` (or equivalent)
   - Read header comments + algorithm outline
   - Identify entry/exit functions (peel, wrap layers)
   - Output: 1-2 paragraph summary of current design

2. **Hybrid layer design (X25519 + ML-KEM-768):**
   - How to layer ML-KEM on top of X25519 onion hops?
   - Option A: hybrid encapsulation at each hop (ML-KEM ciphertext + X25519 ECDH)
   - Option B: post-quantum inner layer wrapping X25519 outer (trust boundary)
   - Option C: parallel paths (one X25519, one ML-KEM, select at runtime)
   - Recommend: which option fits the farm's threat model (no quantum computers today, but forward secrecy investment)?

3. **Integration points:**
   - Where does onion routing sit in the message flow? (message.rs? routing/?)
   - Does PQC-07 ratcheting affect onion paths? (should be independent)
   - New test scenarios: 3+ hops, mixed suite negotiation at each hop

4. **Risk analysis:**
   - Onion routing not yet wired into live paths (parked per execution plan)
   - Hybrid adds complexity: versioning, fallback, compatibility
   - Recommend: staged rollout (X25519-only first, opt-in hybrid later) or unified?

5. **Specification output:**
   - 3-4 page design note covering options A/B/C
   - Recommendation with rationale
   - Integration spec (which hops support hybrid, negotiation protocol)
   - Test scenarios (3+ hops, various suite combos)

## Acceptance Criteria

- [DONE] Current X25519 onion routing summarized (1-2 paragraphs)
- [DONE] Hybrid design options A/B/C outlined (1 page each)
- [DONE] Recommendation: chosen option + rationale (1 page)
- [DONE] Integration points identified (message flow, ratchet interaction)
- [DONE] Risk assessment (complexity, versioning, fallback)
- [DONE] Test scenarios outlined (3-4 examples)
- [DONE] Commit: `spec: PQC-09 hybrid onion routing design note (investigation, parked)`

## Output Format

Create file: `HANDOFF/plans/PQC_09_HYBRID_ONION_DESIGN_NOTE.md`

```markdown
# PQC-09 Hybrid Onion Routing Design Note

**Date:** 2026-07-17
**Status:** Investigation complete, parked pending PQC-05/06/07 adversarial pass
**Threat Model:** Forward secrecy + PQ resistance (low priority for v1.0.0)

## Current Design (X25519 Onion)

[1-2 paragraph summary of existing onion routing]

## Hybrid Options

### Option A: Hybrid Encapsulation at Each Hop
[Design, pros/cons]

### Option B: Post-Quantum Inner Layer
[Design, pros/cons]

### Option C: Parallel Paths
[Design, pros/cons]

## Recommendation

[Chosen option + rationale]

## Integration Spec

[Where in message flow, negotiation protocol, compatibility]

## Risk Analysis

[Complexity, versioning, fallback strategy]

## Test Scenarios

1. [3-hop path, all X25519]
2. [3-hop path, all ML-KEM]
3. [3-hop path, mixed suites]
4. ...

## Status

Parked pending PQC-05/06/07 adversarial review completion.
```

## Blocking/Blocked

**Blocked by:** PQC-05/06/07 adversarial review (standing rule: no wave 3 until wave 2 audited)
**Blocks:** PQC-10 (ML-DSA identity), implementation waves

## Time Estimate

60-90 minutes (code inspection + design writeup)

## Notes

- This is investigation + spec writing, not implementation
- Focus on design quality, not code
- Parked work: no commit required if design is incomplete (output to DRAFT status)
- If unable to reach recommendation: document open questions for future resolution
