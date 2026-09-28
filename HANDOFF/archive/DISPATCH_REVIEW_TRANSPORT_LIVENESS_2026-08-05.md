# DISPATCH: ADVERSARIAL REVIEW -- Transport Liveness Phase 1

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Mode: ADVERSARIAL SECURITY/CORRECTNESS REVIEW. Read-only. No code changes.
Verdict required in the report format below. This review is the AGENTS.md
rule 8 gate before the change can merge.

## The change under review

Branch fix/transport-liveness-failover-2026-08-05, commit 7af3bb4e, file
core/src/transport/manager.rs (attached in full). Summary of what changed:

1. PeerDisconnected handler (search "Drop the peer from this transport's
   connected set"): now also removes the peer from
   transports[transport].connected_peers -- previously only
   ConnectionEstablished touched that set, so every disconnect path leaked
   zombie entries.
2. tick() rewritten in three phases:
   - Phase A (read locks): collect peers whose last_seen exceeds 300s and
     the transports they occupy.
   - Phase B (write locks): remove them from peer_last_seen and from each
     transport's connected_peers. peer_transports is DELIBERATELY left
     intact.
   - Phase C (no locks): for each stale peer, call
     escalation_engine.deescalate(peer_id) if configured, then replay
     TransportEvent::PeerDisconnected through handle_event for each
     occupied transport. The normal handler then removes peer_transports
     entries and queues target peers for reconnection.
3. New field/setter: escalation_engine: Option<Arc<EscalationEngine>> +
   set_escalation_engine.
4. Three new tests: test_tick_stale_prune_clears_connected_peers,
   test_tick_stale_prune_queues_target_peer_reconnect,
   test_tick_stale_prune_deescalates_transport.

Known, documented deferral: the health monitor tracks libp2p PeerIds while
the manager keys on [u8; 32] identity ids; health-triggered staleness is
Phase 2 (ticket reference in the code comment).

## Review brief -- attack these specifically

1. LOCK DISCIPLINE: Phase A read -> Phase B write -> Phase C unlocked. Can
   any interleaving deadlock (handle_event re-takes these locks; parking_lot
   RwLock is NOT reentrant)? Can a concurrent event between phases produce
   inconsistent state (e.g., ConnectionEstablished for a peer mid-prune)?
   Is the outcome at worst benign (one extra tick to converge)?
2. EVENT REPLAY SEMANTICS: replaying PeerDisconnected for a peer whose
   platform connection might STILL be alive at the transport layer (false
   staleness -- e.g., an idle-but-healthy BLE link with no traffic for 5
   minutes). What breaks? Reconnection churn? Duplicate connections?
   Message duplication? Assess against the reconnection queue backoff
   (ReconnectionState) and target_peers semantics.
3. RECONNECT STORM: N stale peers pruned in one tick -> N synthetic events
   -> queue population. Is peers_needing_reconnect's
   RECONNECT_MAX_CONCURRENT + stagger sufficient, or can tick amplify load?
4. DEESCALATION INTERACTION: deescalate mutates engine state for peers that
   may reconnect on the SAME transport moments later. Does the engine
   re-escalate correctly on reconnect (check should_escalate/escalate in
   escalation.rs, also attached if needed)? Any state where a peer is
   pinned to a worse transport permanently?
5. BEHAVIORAL NEUTRALITY: confirm zero wire-format change, zero change to
   event enum semantics, zero behavioral change for healthy/active peers
   (last_seen refreshed on DataReceived -- verify the refresh path exists).
6. TEST ADEQUACY: do the three tests actually prove the claims? What
   missing test would you require before merge (name it, don't write it)?
7. Anything else a hostile reviewer would flag.

## Report format (mandatory final block)

VERDICT: PASS|CONDITIONAL_PASS|FAIL
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (read-only review)
FILES: <files examined>
NOTES: <max 8 lines: verdict rationale + blocking findings if any>

Before the final block, list findings as F1..Fn with severity
(CRITICAL/HIGH/MEDIUM/LOW), evidence (file:line), and required action.
