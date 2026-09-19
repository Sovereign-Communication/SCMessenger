# Finding -- #272 routing-feed removal vs merged PR #263 (T4): evidence-backed verdict

Status: **READ-ONLY INVESTIGATION 2026-09-03, filed for QWEN + CTO** (dispatch brief item C,
`V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md`). Nothing edited, deleted, staged,
committed, or merged. All claims verified against the candidate tree at `a759e0c7`
and `origin/main` `67d19d3c` this session.

## Reference points

- PR #263 (T4) squash merge in main: `bb253eaf` "V040-T4: feed routing engine on
  ConnectionEstablished (D6) (#263)". Added the feed at `swarm.rs:5571` (native) and
  `swarm.rs:8000` (wasm), `endpoint_transport_string`, the `TransportType::Circuit`
  tier, and three D6 pin tests at `iron_core.rs:5079/5128/5164`.
- Candidate `a759e0c7` removes all of the above. Its own commit note lists only
  retraction / ExpiredListenAddr / sync_external_address as the swarm.rs changes;
  the routing-feed removal is NOT accounted for in the note's "26 main-only lines".
- D6 exit criterion (SHIP_PLAN.md): field gate -- "delivery when the first-choice
  transport is unavailable, proving failover selects a working path". Unchanged by
  this PR; the deleted tests were the code-level pins of the mechanism, not the gate.

## Q1 -- After the candidate, how does a peer first enter LocalCell?

Trace of every admission entry point (`OptimizedRoutingEngine::peer_seen` is the only
entry into LocalCell; `LocalCell::peer_seen` creates the `PeerInfo`, `local.rs:123-145`):

1. Native: the Identify handler calls
   `engine.base_engine_mut().local_cell_mut().peer_seen(peer_id_bytes, transport_type)`
   at candidate `swarm.rs:5131` -- PRE-EXISTING in main (`swarm.rs:5141`), KEPT.
   Transport is always `transport_type_to_routing_transport(kad::Mode::Server)` = TCP
   (`swarm.rs:1548-1553`). Identify runs on connection establishment, so every
   connected native peer is admitted -- with transport pinned to TCP.
2. Wasm: NO feed. T4's wasm feed (main `swarm.rs:8000`) is deleted; grep of the
   candidate's wasm ConnectionEstablished region shows only `reported_peer_discoveries`
   and the smart-retry trigger -- no `peer_seen`, no `local_cell`, no
   `routing_peer_seen` anywhere in the wasm arm. The routing engine is installed
   unconditionally (`iron_core.rs:834`, not cfg-gated), so on wasm it stays
   PERMANENTLY EMPTY: `peers_for_hint`/`active_peers` always empty, every decision
   falls back to StoreAndCarry.
3. `routing_peer_seen` (`iron_core.rs:2710`) has ZERO production callers after this
   diff (grep of candidate core+cli: only the definition and doc comments) -- the
   claim in the dispatch brief is confirmed and now explained.
4. `record_message_activity` feeds only the adaptive TTL and negative cache
   (`optimized_engine.rs:308`), never LocalCell. `update_reliability` is a no-op for
   unadmitted peers (`local.rs:199-206` get_mut), so delivery confirmations
   (`swarm.rs:3896/3944`) mutate only peers the Identify feed already admitted.
5. `routing_update_peer_hints` has zero production callers in main AND in the
   candidate (only a test at main `iron_core.rs:5023`) -- the hint feed was already
   dead before this PR; the empty comment block at candidate `swarm.rs:5128-5135` is
   pre-existing scaffolding, not a candidate regression. Do not attribute it to #272.

## Q2 -- Deliberate tested replacement or silent regression?

Evidence: the three D6 tests exist in main at `iron_core.rs:5079/5128/5164` and are
deleted by the candidate diff (removed, not renamed, not replaced). The candidate's
iron_core test module retains `routing_hint_update_populates_local_cell` and
`reliability_success_updates_local_score_and_capability_uses_active_peers` but
contains NO test asserting confidence-after-connection or peer admission through the
surviving Identify feed, and no wasm-side routing test.

Verdict: **partially deliberate, partially silent.** The native duplication (Identify
feed + ConnectionEstablished feed both admitting the peer on connection) is a genuine
duplicate the architecture pass could legitimately collapse. But three consequences
are unaddressed and undocumented:

- **Wasm regression.** On wasm the Identify feed does not exist (it is native code
  under the `#[cfg(not(target_arch = "wasm32"))]` section -- the wasm arm is a
  separate event loop), so removing T4's wasm feed leaves the wasm engine empty.
  There was no duplication to collapse on wasm; this is a straight deletion.
- **Transport fidelity lost.** T4 deliberately recorded the endpoint's actual
  transport (tcp/quic/ws/relay) so LocalCell accumulated transports and the
  BLE < WiFi < Circuit < TCP < QUIC ladder could rank paths. The Identify feed pins
  every peer to TCP, so the QUIC bonus (`0.15`) and the ws distinction in
  `routing_decision_to_ranked_routes` are now unreachable in practice -- no producer
  ever records a non-TCP transport into LocalCell. The Circuit tier's removal is
  self-consistent only because its sole producer was deleted with it.
- **Negative-cache clear on sighting lost.** T4's own comment claimed "peer_seen
  clears the peer's negative-cache entry, so a reconnect after path loss restores
  routing confidence immediately". In the candidate, the only clear-on-sighting path
  (`iron_core.rs:2717` inside `routing_peer_seen`) is unreachable; the alternative
  clears (`routing_clear_unreachable_peer` `iron_core.rs:2872`,
  `record_reconnect_success_and_clear_cache` `iron_core.rs:3949`) have zero in-tree
  callers. Recovery now relies solely on TTL expiry (`negative_cache.rs` time-based
  expiry, entries `ttl`-stamped) -- bounded, but the immediate-restore behavior is
  gone. A peer recorded unreachable (`swarm.rs:3960` on outbound failure) stays
  avoided for the TTL even after it reconnects.

## Q3 -- Is PR #263's functionality lost or preserved through another path?

| T4 capability (merged bb253eaf) | After #272 | Where |
|---|---|---|
| Native peer admission on connection | PRESERVED (Identify feed) | candidate `swarm.rs:5131` |
| Wasm peer admission on connection | **LOST, no replacement** | wasm arm has no routing feed |
| Endpoint transport classification (tcp/quic/ws/relay) | **LOST** (all TCP) | `transport_type_to_routing_transport` Server -> TCP, `swarm.rs:1548-1553` |
| Relayed-circuit distinction (Circuit tier, 0.07 bonus) | **LOST** (variant deleted) | `local.rs` enum, ranking tables |
| Negative-cache immediate clear on sighting | **LOST** (TTL-only now) | `iron_core.rs:2717` unreachable; no other caller |
| Confidence-after-connection pins (3 D6 tests) | **LOST** (deleted, no replacement) | diff removes `iron_core.rs:5079/5128/5164` |
| Reliability updates on delivery | PRESERVED | `swarm.rs:3896/3944` (pre-existing, untouched) |
| D6 field gate itself | UNCHANGED (field proof) | SHIP_PLAN.md |

## Bottom line for the reviewer

Native-only: the removal is a defensible duplication collapse IF the transport-fidelity
loss is accepted as doctrine (relayed paths ranked as direct TCP) and the negative-cache
TTL is accepted as the recovery bound. But as committed, three things are wrong with
the PR: (1) the wasm feed deletion is a straight regression with no replacement and no
test -- wasm routing is permanently empty; (2) the three D6 pin tests were deleted
without replacement tests covering the surviving admission path; (3) the removal is
undocumented in the PR's own change note. Minimum to clear: restore or explicitly
retire the wasm feed with a test, add one test pinning the Identify-feed admission and
the negative-cache TTL recovery, and amend the PR note to document the transport-fidelity
decision. This is a REQUEST_CHANGES-grade item for #272, not an APPROVE-as-is.

## Evidence commands (rerun any of these)

- `git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate grep -n 'routing_peer_seen' -- core cli` (def only)
- `git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate grep -n 'local_cell' -- core cli | grep -v tests` (5131, 3896, 3944, 5184)
- `git show 67d19d3c:core/src/transport/swarm.rs | grep -n 'routing_peer_seen'` (5571 native, 8000 wasm)
- `git show 67d19d3c:core/src/iron_core.rs | grep -nE 'fn routing_peer_seen_raises|fn routing_peer_seen_distinguishes|fn parse_transport_type_distinguishes'`
- `git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate grep -n 'routing_clear_unreachable_peer\|record_reconnect_success_and_clear_cache' -- core cli` (zero callers)
- `git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate show HEAD:core/src/routing/local.rs | sed -n '199,206p'` (update_reliability is a no-op for unadmitted peers)