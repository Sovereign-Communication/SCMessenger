# IDENTIFIER GATE FOLLOW-UPS (from Phase 0b adversarial review)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- created 2026-08-04/05 from
HANDOFF/review/PHASE0B_MSGREQ_GATE_REVIEW_QWENPAID_2026-08-04.md.
Deadline: before v0.4.0 tag. Tier: THINK/MAX work via qwenpaid unless noted.

## T1 (HIGH, P1) -- mixed-fleet block bypass

Block stored under public key + inbound sender_id arriving as identity_id
(old pre-canonicalization build sending) misses both the direct and the
derived candidate, so the blocked peer's message is processed. Fix
direction: store BOTH identifier flavors at block-write time (block_peer /
block_and_delete_peer resolve the alternate flavor from the authenticated
public key when available), or add a reverse index. Requires design note
first (block-store schema / migration of existing block entries) --
dispatch design + implementation separately; core store change, so
adversarial review on file before merge (same bar as the block-gate
hardening).

## T2 (MEDIUM, P3) -- derivation helper type validation

identity_id_from_public_key_hex (core/src/identity/keys.rs) must return None
unless the input passes is_valid_public_key (Ed25519 curve point), preventing
identity_id double-hashing. Small mechanical change; core/src/identity/ is
not in the AGENTS.md rule-8 audit list, but include the change in the next
review bundle anyway. Dispatch: qwenpaid, diff mode, scoped to keys.rs.

## T3 (HIGH, P4) -- fail-closed pending-request listing

GetPendingMessageRequests (cli/src/server.rs) shows a request whenever the
sender cannot be proven known-or-blocked. Change the filter so an
UNRESOLVABLE sender identifier (derivation impossible or flavor unknown) is
suppressed from the listing or explicitly flagged, never shown as a clean
request. CLI-only change; no rule-8 gate. Dispatch: qwenpaid, diff mode.

## T4 (P2, P5 kernel) -- centralize flavor resolution

Move multi-flavor block resolution into BlockedManager (e.g.
is_blocked_resolved) so core ingress, CLI listing, WASM bridge, and Android
all consume one policy instead of reimplementing the expansion. Refactor
after T1 lands; schedule in v0.4.0 close-out or v0.5.0 prep.
