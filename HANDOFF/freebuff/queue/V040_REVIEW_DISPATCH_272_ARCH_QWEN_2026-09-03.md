# V040 REVIEW DISPATCH -- #272 V040 architecture candidate (adversarial, qwen free lane)

Status: **AUTHORIZED 2026-09-03 (CEO input: `V040_CEO_INPUT_272_REVIEW_DISPATCH_2026-09-03.md`) -- AWAITING DISPATCH**
Priority: P0 -- the ONLY open v0.4.0 PR with no Rule-8 evidence on file; last dependency in the merge order
Lane: **Qwen free** -- model per CEO input (strongest funded bucket; 267 precedent `qwen3.8-max-0902`, else soonest-expiring funded large-general per quota ledger). Send the exact dated model code.
Target: PR **#272** `cto/v040-candidate-2026-09-02` @ `a759e0c7` (7 files, +222/-327 vs origin/main `67d19d3c`)
Reviewer constraint: MUST NOT be the author of the PR or of the earlier architecture work. Draft the verdict BEFORE reading the candidate note (`HANDOFF/freebuff/inbox/V040_candidate_commit_2026-09-02.md`) or the PR body. Read-only: do NOT edit code; findings go back to the lane author (CTO) as triage items.
Rule-8 applies: touches `core/src/transport/{observation,swarm}.rs` and `core/src/routing/{local,optimized_engine}.rs` -- the APPROVE must be explicitly recorded, non-author, and state the attacks actually tried.

## Why this review exists

The candidate makes `AddressObserver` the single source of truth for what this
node advertises: a listen-port allowlist gates every observed address, the
swarm's confirmed external-address set is synced to the observer's consensus
primary (with retraction of stale promotions), and -- the large behavioral
change -- the ConnectionEstablished routing feed, `endpoint_transport_string`,
and the whole `TransportType::Circuit` tier are REMOVED, along with three D6
acceptance tests. The author's own summary asserts these are reconciliations,
not regressions. Your job is to attack that claim.

## The diff (read it first)

- `core/src/transport/observation.rs` (+103/-): `listen_ports` allowlist on
  `AddressObserver`; `set_listen_ports` replaces the port set and drops
  ineligible observations; `record_observation` fails closed and invalidates a
  peer's prior observation on a non-listen port; deterministic sort tiebreak.
- `core/src/transport/swarm.rs` (+218/-): new `sync_external_address` helper
  (retracts every confirmed address that is not the current primary, re-adds
  primary); `NewListenAddr` feeds `set_listen_ports`; new `ExpiredListenAddr`
  arm; `ListenerClosed` retracts ports and re-syncs; ConnectionEstablished
  `routing_peer_seen` feed DELETED (native + wasm); `endpoint_transport_string`
  deleted; `Circuit` routing bonus removed at all four ranking sites.
- `core/src/iron_core.rs`: `parse_transport_type` no longer maps
  "circuit"/"p2p_circuit"/"relay" (relay now -> TCP); `get_forwarding_capability`
  drops the circuit arm; three D6 tests deleted; ownership doctrine comment added.
- `core/src/routing/local.rs`: `TransportType::Circuit` variant REMOVED;
  `peers_for_hint`/`active_peers` share `sort_by_reliability` with a peer-id
  tiebreak (deterministic order).
- `core/src/routing/optimized_engine.rs`: dead `local_id`/`local_hint` fields removed.
- `cli/Cargo.toml`: `default-run` added. `docs/ARCHITECTURE_SCOPE_V040.md`: +27.
- Emoji stripped from log strings throughout (hook compliance, not behavior).

## Attack checklist (attack each; reject hearsay claims)

### A. `sync_external_address` single-primary retraction invariant (highest risk)
The helper deletes EVERY `swarm.external_addresses()` entry that is not the
current consensus primary. The code comment claims "the two promotion sites
below are the only add_external_address callers in the tree". VERIFY by grepping
the entire candidate tree, not just swarm.rs: `add_external_address` and
`remove_external_address` in core (incl. wasm arm, mesh_routing.rs,
mobile_bridge.rs), cli, and tests. Any other confirm site means this helper
silently deletes an address some other subsystem legitimately confirmed. Also
check who READS `external_addresses()` (e.g. the DCUtR site that the 267
review's F-3 found publishing them) -- does the single-primary policy break a
consumer that expects multiple confirmed addresses? Note the base is origin/main
(67d19d3c), which does NOT include the unmerged #267/#268/#269/#270; flag
interactions the merge order will have to reconcile, but do not gate on
unmerged content.

### B. Listen-port allowlist: port-only strength
The allowlist is a set of PORTS, not address:port. `bound_addresses` will
contain wildcard listeners (`/ip4/0.0.0.0/tcp/P`, `/ip6/::/tcp/P`) and LAN
addresses -- the set is collapsed to ports. So ANY peer observation of
`<anything>:P` passes the gate; only the port is checked, the IP is consensus's
problem. Is that the intended strength (the doc says "whether an observed
address can be advertised" -- the port gate cannot prove the IP is ours)?
Additionally: non-socket listeners (dns4/ws/wss relay, p2p-circuit) are dropped
by `extract_socket_addr` filter_map -- if a node's only listeners are
non-socket, the allowlist is empty and the node fails closed (never advertises).
Verify every supported native config has at least one socket listener, and that
`set_listen_ports` being REPLACED (not merged) on every NewListenAddr event is
safe under multi-listener startup ordering.

### C. Routing-feed removal vs merged PR #263 (T4) -- the big behavioral question
Verified in the candidate tree at a759e0c7: `routing_peer_seen`
(iron_core.rs:2710) has ZERO production callers after this diff; the only hits
are the definition, its doc comment, and an unrelated comment. The
ConnectionEstablished feed (native + wasm) is deleted, and with it the D6
acceptance test `routing_peer_seen_raises_confidence_after_connection_established`
which asserted a peer's routing confidence goes from zero to non-zero on
connection. `update_reliability` also has zero production callers (test-only).
`record_message_activity` IS still fed (iron_core.rs:2716/2726/2771/2954 and
swarm.rs:3893 custody dispatch). Questions:
1. After this PR, how does a peer first enter LocalCell in production? If only
   via message activity, routing falls back to StoreAndCarry (zero confidence)
   for every peer until a message is delivered -- is that a deliberate,
   TESTED design, or a silent drop of the D6 acceptance criterion? Grep the
   candidate tree for any remaining site that feeds `peer_seen`/`routing_peer_seen`
   in production (not tests) and say which it is.
2. The Circuit-tier deletion is coherent only if no producer remains.
   `endpoint_transport_string` was the sole producer and is deleted -- confirm
   no OTHER producer passes a "circuit"/"relay" string to the parser, and that
   the parser's catch-all -> BLE cannot silently absorb a surviving literal
   (grep "circuit" and "relay" as string literals in production paths, incl.
   cli and wasm).
3. The deleted test `routing_peer_seen_distinguishes_circuit_from_direct_tcp`
   and `parse_transport_type_distinguishes_direct_from_circuit` pinned the
   string contract. With the tier gone, does anything still WRITE
   `TransportType::Circuit` or match on it (wasm, mobile_bridge, tests,
   serialization)? The enum variant is removed -- any leftover reference is a
   compile error caught by the gates, but semantic leftovers (a "relay" string
   now classifying as TCP) need eyes.

### D. Listener lifecycle retraction
`ExpiredListenAddr` (new arm) and `ListenerClosed` both retain bound_addresses,
recompute the allowlist, and re-sync. Verify:
1. libp2p-tcp emits ExpiredListenAddr per-address when an interface goes down;
   the listener may retain other addresses -- retain removes only the expired
   one; correct? (Two listeners on the SAME port, different IPs: port-set cannot
   distinguish them -- expired 192.168.1.5:9001 keeps port 9001 allowlisted if
   0.0.0.0:9001 remains. Conservative, but confirm the comment claims nothing
   stronger.)
2. `ListenerClosed` with `reason.is_err()` still sends `ListenerFailed`
   (preserved -- confirm the retraction code did not move past the event_tx send).
3. After retraction, sync removes the dead address from confirmed set -- the
   node stops advertising a dead port. Verify the ordering: set_listen_ports
   drops the observations BEFORE sync_external_address reads the primary, so a
   retracted port cannot leave a stale primary behind.

### E. observation.rs consensus
1. `record_observation` on a non-listen port REMOVES the observer's entire prior
   observation (which may have been for a valid port) -- deliberate per the
   comment; the test `non_listen_port_observations_are_rejected` covers
   re-observe. Check the invalidation is not too eager: one bad observation from
   a peer drops its good one, and nothing re-admits it until that peer re-observes.
2. Deterministic tiebreak `(Reverse(count), address)` -- SocketAddr Ord; confirm
   no remaining nondeterminism (HashMap iteration) in the consensus path.
3. `set_listen_ports([])` fails closed -- test covers; confirm no caller can
   ever pass an empty set while a listener genuinely exists (startup ordering:
   first NewListenAddr fires after first observations? if observations arrive
   before any NewListenAddr event they are rejected -- safe, but confirm the
   node eventually advertises once listening).

### F. Ownership claim in the docs
`docs/ARCHITECTURE_SCOPE_V040.md` asserts AddressObserver is a single source of
truth. The 09-02 audit flagged two candidate instances (manager.rs:238 owns one;
swarm.rs event handling uses another). Trace which instance the swarm's event
handler actually mutates, whether the manager.rs instance is live or dead, and
whether two observers could ever feed different consensus sets to different
consumers.

## Method

- Worktree `scm-v040-candidate` is on disk at a759e0c7 (clean, pushed).
  `cd /c/Users/SCM/Documents/GitHub/scm-v040-candidate` for the full tree, or
  `git show a759e0c7:<path>` from the main checkout. `gh pr diff 272` also works.
- Run greps (rg/git grep) for every caller/consumer claim above -- the verdict
  must cite command + line for each attack.
- Read-only. No code edits, no commits, no pushes. Cargo run/check is permitted
  for evidence only (lane rule: confirm no other build is live first); `cargo
  test -p scmessenger-core --lib` focused suites are useful signal but the
  Windows host is authoritative.

## Output contract

1. Verdict to `HANDOFF/review/V040_ARCH_CANDIDATE_REVIEW_QWEN_2026-09-03.md`
   (attack tried -> verdict -> evidence command+line; Rule-8 APPROVE must list
   the attacks actually tried, and must be non-author).
2. One comment on PR #272 with the same verdict (APPROVE / REQUEST_CHANGES +
   findings).
3. Reply in `HANDOFF/freebuff/inbox/` with a 3-line note (verdict, file path,
   anything the author must fix).

Do NOT pad. If nothing holds, list the attacks tried and say so plainly.