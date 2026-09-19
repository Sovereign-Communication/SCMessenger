# V040 Scoping note -- UDP/QUIC inbound admission vs the #272 TCP-only allowlist

Date: 2026-09-03
Status: READ-ONLY analysis, candidate tree @ 177bd840 (worktree
scm-v040-candidate, clean). Input to the directive's re-review
(V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md).
Conclusion up front: for the v0.4.0 gate the CEO-sanctioned DEFERRAL
(directive option c) is the right call -- the widening change (option a) is
small in file count but amends the arch pass's port-only observation model
AND the address-reflection wire protocol, which is more surface than a gate
deferral warrants. Nothing in the current shape forecloses UDP/QUIC later.

## 1. Current reject sites (verified in the candidate tree)

1. `listen_port_from_bound_addr` -- core/src/transport/swarm.rs:464-474.
   Returns None for `Protocol::Udp(_) | Protocol::Quic | Protocol::QuicV1`.
   The design comment states the reason: "observations carry no transport and
   promotion always reconstructs /tcp/ -- admitting a UDP port could
   otherwise advertise a TCP endpoint for a listener that never speaks TCP."
   The rejection is therefore a CONSEQUENCE of the transport-less model, not
   an independent choice.
2. `sync_external_address` -- swarm.rs:477-495. Promotes the consensus
   primary to a /tcp/ multiaddr only (format! "/ip4|x/tcp/{port}" /
   "/ip6|x/tcp/{port}").
3. The model behind both: `AddressObservation { observer, address: SocketAddr,
   timestamp, confirmation_count }` (observation.rs:14-20) -- ip+port only,
   NO transport. `extract_socket_addr` (observation.rs:255-279) parses BOTH
   Tcp and Udp ports, so the parse layer is not the blocker; the exclusion
   happens at allowlist construction (site 1) and promotion (site 2).
4. Allowlist semantics: `set_listen_ports` (observation.rs:38-46) + the gate
   in `record_observation` (observation.rs:48-60) are PORT-only and
   fail-closed on empty -- transport-agnostic as written; they do not need to
   change for a UDP listener to be ADMITTED, only for it to be promoted
   correctly.

## 2. Routing ladder QUIC sites (NO change needed -- already live)

- Transport bonus map at swarm.rs:1622-1629 (and mirrored at 1650, 1712,
  1740): `RoutingTransportType::QUIC => 0.15, TCP => 0.10, Circuit => 0.07,
  WiFiAware/WiFiDirect => 0.05, BLE => 0.0`, comment "BLE < WiFi < TCP < QUIC".
- `RoutingTransportType::QUIC` exists; `endpoint_transport_string` already
  maps quic -> "quic" (pinned by test at swarm.rs:8428:
  `assert_eq!(endpoint_transport_string(&quic), "quic")`).
- So OUTBOUND QUIC classification + routing rank are done. The gap is purely
  inbound: a UDP/QUIC listener port is never admitted into the observer, so
  a QUIC endpoint is never advertised, so inbound QUIC is unreachable.
- Infrastructure note: the always-on relay node binds tcp+udp/9001 with a
  QUIC listener (security group opens udp/9001 -- HANDOFF/audit/AWS_RELAY_REBUILD).
  When UDP lands, inbound QUIC to the cloud node is the first real user.

## 3. Tests that pin the current behavior (named)

NO test pins the UDP/QUIC rejection in `listen_port_from_bound_addr` itself
(verified: no test call site for the function; the only quic assertion in the
swarm tests is the endpoint_transport_string one at 8428). The TCP-only
behavior is pinned INDIRECTLY by port-only observer tests:

- observation.rs unit tests: `address_observer_contract` (286),
  `non_listen_port_observations_are_rejected` (304),
  `removing_a_listen_port_removes_its_observations` (321),
  `test_extract_socket_addr` (344) -- all TCP ports, port-only semantics.
- core/tests/test_address_observation.rs: `test_address_observation_between_peers`
  (10), `test_consensus_with_multiple_observations` (92 -- the file the
  177bd840 test-only fix touched; set_listen_ports([1234, 5678])),
  `test_connection_tracking` (132), `test_address_extraction_from_multiaddr` (177).
- D6 confidence tests -- NOT admission tests, unaffected by any option:
  `routing_peer_seen_raises_confidence_after_connection_established`
  (iron_core.rs:5043) and `routing_peer_seen_distinguishes_circuit_from_direct_tcp`
  (iron_core.rs:5092) pin the routing FEED, which already carries quic/ws/tcp.

## 4. Option (a) change surface -- widen with transport-carrying promotion

Honest estimate: 2 core files + 1 integration test + ~6 new/adapted tests,
PLUS one wire-protocol touch. Detailed:

1. observation.rs: AddressObservation gains a transport field (enum
   TransportKind { Tcp, Quic, Ws, ... } or protocol family); record_observation
   signature takes transport (or &Multiaddr); set_listen_ports becomes
   per-(port, transport) -- otherwise admitting UDP ports while the allowlist
   stays port-only would admit a UDP observation of port 9001 and promote it
   as /tcp/9001 (the exact mislabeling the design comment warns about);
   consensus primary carries transport.
2. swarm.rs: listen_port_from_bound_addr returns (port, transport) and accepts
   Udp/Quic/QuicV1 (map Quic/QuicV1 -> Quic); adapt the two filter_map call
   sites (5343, 6002); sync_external_address promotes by transport
   (/udp/<ip>/quic-v1 for Quic, /tcp/ for Tcp -- and note the pre-existing
   ws nuance: /tcp/9002/ws listeners are ALREADY promoted as /tcp/9002 today,
   which the widening would either fix or leave documented).
3. WIRE-PROTOCOL TOUCH -- the two reflection-response recording sites get the
   address as a parsed SocketAddr STRING with no transport: swarm.rs:4097 and
   :7602 (`response.observed_address.parse::<SocketAddr>()`). Carrying
   transport through those paths requires the address-reflection request/
   response to carry it (or those sites to be excluded from QUIC promotion).
   The other four recording sites have the Multiaddr in scope and are cheap:
   :5196 (Identify observed_addr), :6739/:6745 (RegisterEndpoint/TouchEndpoint
   -- note: these arms exist in the candidate because #267 is unmerged; they
   are removed by fdht-gate), :7985 (wasm Identify).
4. New tests: (i) udp/quic listen-port admission; (ii) per-transport allowlist
   rejection (UDP observation rejected when only TCP listens); (iii) QUIC
   primary promotes to /udp/<ip>/quic-v1 not /tcp/; (iv) sync_external_address
   lockstep with a QUIC primary; (v) ephemeral UDP source port rejected (#270
   class per-transport); (vi) integration case in test_address_observation.rs.
   D6 tests unchanged; observer contract tests mostly unchanged.

Net: option (a) is ~3 files + one protocol contract. Small in file count,
but it amends the arch pass's deliberate port-only design decision (the F3
disposition) and the reflection protocol -- more than a gate deferral needs.

## 5. Option (c) -- the recommended path: CEO-sanctioned deferral

For the v0.4.0 gate: keep the TCP-only admission, zero code change, head
stays 177bd840, no CI re-run. The lane re-review ends in plain APPROVE at
177bd840 with the verdict file RECORDING the deferral explicitly:
(i) UDP/QUIC inbound admission deferred, not excluded forever; (ii) nothing
in the current shape structurally forecloses the later change -- the observer
allowlist and promotion are the only two touch points and both are
additive-friendly (Section 4); (iii) QUIC is already live in the routing
ladder (0.15 bonus) and outbound classification, so only inbound
admission/advertisement is deferred; (iv) deferral is CEO-sanctioned for the
v0.4.0 gate only. The F3 disposition in the existing FINAL APPROVE is then
re-issued as "approved-with-deferral-recorded" by the lane, and FLAG-5
resolves.

## 6. Consequence (per the directive)

Any code change (option a/b) moves the head -> CI must run -> the lane
re-approves at the new head. The deferral (option c) avoids that entirely.
If UDP lands post-gate, the Section-4 surface is the exact work order.