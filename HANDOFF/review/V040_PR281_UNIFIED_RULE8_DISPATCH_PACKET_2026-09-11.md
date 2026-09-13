# V040 PR #281 — UNIFIED RULE-8 ADVERSARIAL REVIEW DISPATCH PACKET

**Date:** 2026-09-11  
**Target PR:** PR #281 (\unified/v040-3node-parity\)  
**Commit Range:** \origin/main\..\72cf7035\  
**Perimeter:** \core/src/transport/\, \core/src/routing/\ (Rule-8 gated)

---

## 1. Scope of Gated Changes

Under AGENTS.md Rule 8 and \docs/rules/SECURITY_PROTOCOL.md\, changes to \core/src/{crypto,transport,routing,privacy}/\ require an independent adversarial security review on file before merging into \main\.

The 7 files touching the security perimeter in PR #281 are:
1. \core/src/transport/swarm.rs\:
   - D10 relay-reservation multiaddr validation: \is_valid_reservation_base()\ strips nested circuit/p2p segments to prevent mDNS TXT record overflow.
   - D10b poison-listener event-loop guard: \is_poison_circuit_listener()\ intercepts and removes illegitimate circuit listener registrations in \SwarmEvent::NewListenAddr\.
   - Observability logging for lane visibility and relay custody handoffs.
2. \core/src/transport/dial_policy.rs\:
   - Single-owner address admission; suppression of self-referential / broadcast loop dial attempts.
3. \core/src/transport/manager.rs\:
   - Unified connection and transport lifecycle management.
4. \core/src/transport/observation.rs\:
   - Bidirectional transport lane observation telemetry.
5. \core/src/routing/local.rs\:
   - Unified local ordering for message custody.
6. \core/src/routing/optimized_engine.rs\:
   - Routing engine optimizations with peer liveness integration.
7. \core/src/routing/resume_prefetch.rs\:
   - Prefetch cache management for resumed peer connections.

---

## 2. Reviewer Independence & Prior Art

* Prior detailed review packets filed and analyzed:
  - \HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md\
  - \HANDOFF/review/V040_D10B_POISON_LISTENER_GUARD_REVIEW_2026-09-10.md\
  - \HANDOFF/review/V040_PR272_RESOLUTION_DELTA_RULE8_APPROVE_e97c3f82_2026-09-04.md\
* Author Conflict Clearance:
  - As author/integrator of the unified parity merge, the active orchestrator cannot sign its own Rule-8 gate.
  - Review assignment is routed to the independent reviewer (Mac Lane / GPT or designated adversarial reviewer).

---

## 3. Adversarial Assessment Checklist for Reviewer

1. Multiaddr Parsing Edge Cases:
   - Does \is_valid_reservation_base()\ properly reject all malformed, loopback, or nested \/p2p-circuit\ multiaddrs without panicking?
2. Event-Loop Listener Eviction:
   - In \swarm.rs\, does removing an illegitimate listener via \swarm.remove_listener(listener_id)\ cause any dangling state or race conditions with ongoing active reservations?
3. Cryptographic Authority:
   - Verify zero modifications to cryptographic primitives (\core/src/crypto/\). Ensure all key exchanges and AEAD operations remain strictly in IronCore.
