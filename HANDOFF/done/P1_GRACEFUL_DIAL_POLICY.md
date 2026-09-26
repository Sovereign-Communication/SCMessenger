# P1: Graceful Dial Policy (Items 3+4) — Per-Peer Backoff + Relay Preference

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Ticket Status:** Open (dispatch to Qwen CODER)
**Tier:** [SONNET][AUDIT-GATE]
**Scope:** v0.4.0 blocker

## Background

GRACEFUL_AF_DIAL_POLICY ticket. Items 1 (self-dial prevention) and 2 (RFC1918 awareness) are DONE per 2026-07-20 session. Items 3+4 remain.

## Specification

### Item 3: Per-Peer Backoff State Machine (max 3 concurrent dials)

**Requirements:**
- Each peer maintains: `attempt_count` (0-3), `last_attempt_ts`, `backoff_duration` (exponential 1s → 30s)
- Dial orchestrator enforces global limit: never more than 3 concurrent outbound dials to ANY peer
- Dial loop filters: only consider peers where `attempt_count < 3` AND `now() >= last_attempt_ts + backoff_duration`
- On `ConnectionEstablished`: reset peer's backoff to `attempt_count = 0`
- On dial failure (timeout/reset): increment `attempt_count`, double backoff (1s → 2s → 4s → 8s → 16s → 30s capped)

**Error handling:**
- Transient errors (DNS, timeout, network down): increment attempt, apply backoff, retry later
- Permanent errors (Invalid Multiaddr, peer refused, auth failure): mark peer as dead for this session (no retry this boot) OR move to a separate dead-letter queue for manual inspection

### Item 4: Prefer Circuit-Relay After Connection Established

**Requirements:**
- Listen for `Peer::Connected` libp2p event (any successful connection to any peer)
- Once connected, add circuit-relay multiaddr to the dial candidate list
- Ladder order: direct addresses (existing) → circuit-relay (new) → fallback timeout
- Rationale: cold start on unknown peers benefits from routing through warm relay

**Implementation hint:**
- Circuit relay address format: `/ip4/<relay-ip>/tcp/<relay-port>/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`
- Use existing relay peer info (from `peer_exchange` or hardcoded relay list) as the intermediary

## Files to Edit

- `core/src/transport/dial_policy.rs` (or the primary dial module — locate the existing dial loop)
- `core/src/transport/ledger.rs` (if peer state tracking lives here; check structure)
- Add/update test: `core/tests/integration_dial_*.rs` or similar

## Acceptance Criteria

1. Code compiles: `cargo check --workspace`
2. Lint passes: `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`
3. New backoff state machine test passes (verify exponential backoff, concurrent limit, reset on success)
4. Circuit-relay ladder test passes (verify relay multiaddr appears after direct addresses)
5. Ready for adversarial audit (transport/ audit gate applies)

## Notes

- This is hot-path code; pay attention to race conditions (concurrent dial attempts, backoff state updates)
- Backoff state is ephemeral (per-session); does NOT persist across reboots
- Limit of 3 concurrent dials is a tuning parameter; if different value is needed, document it in code comment

---

**Dispatch to:** Qwen CODER  
**Model:** qwen3-coder-plus  
**fusionLite verification:** Yes (race condition audit on backoff state mutation)  
**Move to done/ when:** Lint + new tests pass, audit tag set  
