# V040 D10 — Relay-reservation base validation (mDNS LAN-discovery fix)

## Status

- Filed: 2026-09-10 (CTO seat, session D10)
- Reviewer verdict: **PENDING — no independent reviewer has approved this change**
- Gate status: branch work + branch deploys proceed per T14/D2 precedent; MERGE
  TO MAIN IS BLOCKED until an adversarial APPROVE from a reviewer who did not
  author the change is recorded in this directory.
- Author: CTO seat (self-authored; CANNOT self-approve per rule 8).
- File under review: `core/src/transport/swarm.rs` (rule-8 gated directory:
  `core/src/transport/`)

## Defect being fixed (evidence-first)

The Pixel could not rejoin the mesh after a drop test. Root-cause chain
(documented with live evidence in
`HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T225500Z_ANR_MAIN_FFI_FIX.md`):

1. libp2p-mdns 0.48.0 advertises the swarm's ListenAddresses verbatim
   (vendored source proof:
   `libp2p-swarm-0.47.1/src/behaviour/listen_addresses.rs` — only
   `NewListenAddr`/`ExpiredListenAddr` mutate the set; mdns forwards it).
2. The relay-reservation path called `swarm.listen_on()` with bases derived
   from a relay's identify listen_addrs. Two failure shapes reached it:
   - a base that is itself a `/p2p-circuit` route (nested relay
     registration), and
   - a base that is one of the node's OWN endpoints, reflected back during
     the startup race before binds are recorded (the string matcher
     `addr_filter::is_self_address` cannot catch the wildcard-bind vs
     concrete-reflection mismatch: bind is `/ip4/0.0.0.0/tcp/9001`, the
     reflected self address is `/ip4/192.168.0.222/tcp/9001`).
3. Observed poison listener on Windows (live node-out.log, 18:31-18:33Z):
   `/ip4/192.168.0.222/tcp/9001/p2p/<self>/p2p-circuit/p2p/<AWS>/p2p-circuit/p2p/<self>`
   — its `dnsaddr=` TXT exceeded the 255-byte mDNS record limit
   (`TxtRecordTooLong` warnings every ~90 s) and the mdns socket hit
   `os error 10040` datagram overflow.
4. Effect: Windows LAN advertisement degraded; the phone's NsdManager never
   received a usable response; with the phone's relay ledger empty (demoted
   by design after drop-test failures), every discovery path was dead.

## Change (minimal, at the root)

All in `core/src/transport/swarm.rs`:

1. `is_self_endpoint(addr, known_local_addrs)` — NEW, structural
   wildcard-aware self check: exact host:port match, plus wildcard-bind
   claims limited to loopback/private/link-local candidates (and IPv6
   conservatively). Deliberately does NOT claim public hosts on the same
   port — the standard Windows:9001 <-> AWS:9001 same-port topology must
   stay eligible (test-enforced).
2. `is_valid_reservation_base(addr, known_local_addrs)` — NEW gate:
   rejects circuit-containing bases, undiscoverable bases (existing
   `is_discoverable_multiaddr`), and self-endpoints against the CURRENT
   swarm listeners + external addresses.
3. Reservation call site (identify handler, relay reservation block):
   builds the known-local set from `swarm.listeners()` +
   `swarm.external_addresses()` at the moment of use; picks the first valid
   base among `routable_relay_addrs`; if none is valid, skips the
   reservation entirely (logged `[D10] ... reservation skipped`) — a later
   identify with complete binds retries. No reservation is ever created
   from an invalid base.
4. `is_canonical_reservation_addr(addr)` — NEW structural guarantee:
   exactly one `/p2p-circuit`, terminal position. Asserted (debug_assert)
   at the successful-reservation site; the existing normalization in
   `relay_reservation_multiaddr` already collapses any base to this form,
   so the assertion is a tripwire, not a filter.
5. `build_routable_relay_addrs` — unchanged (it already excluded circuit
   bases and self-addresses at identify time); the fix is the live
   re-validation at the moment of use, which closes the startup-race hole.

## Regression tests (8, all passing; `cargo test -p scmessenger-core --lib d10`)

- `d10_nested_circuit_base_is_rejected` — the exact poison shape.
- `d10_concrete_self_base_is_rejected` — private reflection vs wildcard bind.
- `d10_wildcard_bind_does_not_claim_public_hosts_on_same_port` — guards the
  standard same-port relay topology from over-blocking.
- `d10_exact_host_match_is_rejected_even_without_wildcard` — different-port
  same-host stays eligible; same host:port is self.
- `d10_empty_local_set_does_not_block_foreign_relay` — cold-start must not
  blanket-disable reservations.
- `d10_valid_foreign_base_is_accepted_and_reservation_is_canonical` — AWS
  endpoint accepted; canonical single trailing circuit; relay id embedded.
- `d10_reservation_builder_never_nests_circuit_from_circuit_base` —
  normalization collapse guarantee.
- `d10_loopback_base_is_rejected` — loopback can never anchor a reservation.

## Risk assessment (author's view, for the reviewer to attack)

- Over-blocking risk: the wildcard rule is limited to private/loopback
  candidates, so foreign relays on shared ports stay dialable. If a mesh
  legitimately ran a relay on a PRIVATE host sharing our wildcard port, D10
  would refuse a reservation to it during the race window (until binds are
  recorded, after which exact matching applies). Accepted trade-off: LAN
  discovery failure is network-wide and silent; a delayed reservation is
  self-healing on the next identify.
- Under-blocking risk: a self-reflection with a public IP (full-cone NAT
  reflection of our own public endpoint) would not be claimed by the
  wildcard rule. Mitigation: such an address typically also appears in the
  external-address set by reservation time, making the exact match hit; and
  the mDNS overflow requires the *nested-circuit* shape, which is rejected
  unconditionally.
- No protocol changes: only which `listen_on` calls are issued. Behavior
  with valid bases is byte-identical to before.

## Review focus requested

1. Is the wildcard-claims-private rule sound, or should wildcard binds claim
   nothing (delaying all reservations until concrete binds are known)?
2. Any path where `routable_relay_addrs` can still contain a circuit base
   that the new gate would not see (multiaddr parsing edge: `/p2p-circuit`
   vs `P2pCircuit` casing, ws/wss wrapping)?
3. The `continue` on no-valid-base skips `relay_peer_addrs` bookkeeping —
   confirm no state expectation elsewhere relies on that map being populated
   for a peer whose reservation was skipped.
4. Confirm the debug_assert cannot fire in production builds (it is a
   tripwire only).

## Verdict

- PENDING — awaiting independent adversarial review. Record verdict in this
  directory per the T14/D2 convention (`..._APPROVE_<reviewer>_<date>.md`).

## Finalize gate evidence (2026-09-09/10, authoritative Windows environment)

All gates run under `scripts/build_lock.py`; logs under `tmp/cto/`:

| Gate | Invocation | Exit | Log |
| --- | --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | 0 | `tmp/cto/d10_fmt_final.log` |
| D10 unit tests | `cargo test -p scmessenger-core --lib d10_` | 0 (8 passed / 0 failed) | `tmp/cto/d10_tests_final.log` |
| Full workspace suite | `cargo test --workspace` | 0 | `tmp/cto/D10_GATE_20260909T234503Z/workspace_tests.log` |
| clippy (CI-exact) | `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` (per `.github/workflows/lint.yml:38`) | 0 (clean, 39.9 s) | `tmp/cto/D10_CLIPPY_20260909T142328Z/clippy.log` |

Note: the earlier battery clippy failure (`tmp/cto/D10_GATE_20260909T234503Z/clippy.log`)
was an invocation artifact — the bare `-A empty_line_after_doc_comments`
shorthand trips `renamed-and-removed-lints` under `-D warnings` on this
toolchain; the CI-exact qualified form passes clean. A stale build lock from
that battery (`cto-d10-clippy`, dead pid) was released with the matching
holder name before the finalize runs.
