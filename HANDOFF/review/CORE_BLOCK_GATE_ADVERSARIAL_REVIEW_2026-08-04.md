# ADVERSARIAL REVIEW -- block-gate identifier fix (CORE_BLOCK_GATE_IDENTIFIER_FIX)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-08-04
Reviewer: qwen/qwen-max (THINK tier, free lane), dispatched via
scripts/delegate_task.py; raw transcript
tmp/BLOCK_GATE_ADVERSARIAL_REVIEW_response.md.
Subject: db4401d7 (fix(identity): block gate checks both identifier
flavors), core/src/iron_core.rs inbound receive path.

## VERDICT: FAIL (two valid findings; follow-up dispatched)

## Findings and orchestrator dispositions

- P2, fail-OPEN blocked+deleted on store error (`unwrap_or(false)`):
  VALID. Pre-existing (the minus-lines in the diff show the same
  unwrap_or(false)) but inside the broken gate's blast radius and contrary
  to the file's own FAIL CLOSED doctrine on the sibling check. DISPOSITION:
  fix now -- ticket HANDOFF/todo/CORE_BLOCK_GATE_HARDENING.md.
- P6, per-candidate blocked_manager read-lock re-acquisition (TOCTOU
  between the two sections): VALID (defense-in-depth). DISPOSITION: fix
  now, same ticket (single lock snapshot).
- P1, pre-canonicalization blocks missed if derivation fails: NOT VALID as
  stated. identity_id_from_public_key_hex fails only for non-32-byte-hex
  input; such input is retained as-is in the candidate set, so
  identity_id-keyed legacy blocks still match. Legacy senders carrying
  identity_id as sender_id are covered by candidate[0]. DISPOSITION:
  dismissed; the Phase-1 identifier audit keeps a watch item for any third
  identifier flavor in persisted stores.
- P3, device-id lookup cross-contamination via find_map over flavors:
  RESIDUAL RISK, LOW-MEDIUM. Would require a cross-namespace collision
  (one peer's public key equal to another peer's identity_id, both
  64-hex); the block check still iterates both candidates regardless.
  DISPOSITION: Phase-1 identifier-audit watch item (prefixing the two
  flavors in storage/comparison paths, per keys.rs PUBLIC_KEY_PREFIX /
  IDENTITY_ID_PREFIX groundwork).
- P4, ingress drop ordering: PASS (no change; Err(Blocked) precedes dedup/
  persistence/receipt/inbox).
- P5, scope creep / latency for unblocked senders: NOT VALID. Semantics
  unchanged; cost is one extra derivation + at most one extra lookup.
  DISPOSITION: dismissed.

## Status

RE-REVIEW PASS (2026-08-04): targeted re-review of the hardening diff
(e04b23f9) by qwen/qwen-max THINK tier -- raw transcript
tmp/BLOCK_GATE_HARDENING_REREVIEW_response.md -- verdict PASS on all five
probes: R1 P2 closed (every Err path in the blocked+deleted loop drops at
ingress), R2 P6 closed (single read guard serves both sections), R3
device-lookup move is behavior-neutral (pure read), R4 guard drops
correctly on early return and nothing depends on the removed
is_blocked_and_deleted value, R5 no new bypass/panic/behavior change for
unblocked senders.

Hardening landed as e04b23f9 (compile + fmt green locally). Merge of
PR #136 now blocked ONLY on full CI green at e04b23f9 (operator directive
2026-08-04: iterate to green, then merge).
