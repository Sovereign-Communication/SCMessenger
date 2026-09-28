# SCMessenger product-owned verifier and acceptance boundary

**Date:** 2026-09-24
**Owner:** SCMessenger
**Change boundary:** Documentation only. This handoff changes no source, tests, hooks, CI, vendor tree, runtime, device, node, message state, or legacy handoff. It is not implementation authorization.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Decision requested

Define one product-owned verifier boundary for message delivery and acceptance. The verifier must be the authority for translating transport events, application receipts, custody ownership, and message kind into an explicit delivery state. Platform callers may display that state, but they must not invent a second success policy.

The smallest safe implementation boundary is a pure, versioned core verifier plus focused contract tests and projections at existing CLI/Android call sites. Do not begin with a transport rewrite, vendor update, UI feature, device run, or storage migration.

This document does **not** authorize implementation, deployment, restart, sending, device action, legacy migration, or PR creation.

## Evidence inspected

The current SCMessenger checkout is `3f41005d1a6a2b9b608dd3960640a642c92f792e` and contains active source, test, vendor, and documentation WIP. The hashes below identify the inspected files; they do not represent a clean release.

| Boundary | Exact evidence location | Finding |
|---|---|---|
| Connection flush ownership | `core/src/transport/swarm.rs:1908-1960` | `register_and_flush_swarm_peer` records the peer and routes both connection paths through one egress helper. |
| Native periodic sweep | `core/src/transport/swarm.rs:3972-3982,4193-4220` | The current 120-second outbox sweep is visibly installed in the native event loop and calls the connection flush path for connected peers. |
| WASM retry arm | `core/src/transport/swarm.rs:8570-8770` | Retry response/failure handling is present in the WASM arm, but a matching periodic outbox sweep is not shown in the inspected range; parity is an explicit acceptance requirement, not an assumed fact. |
| Dispatch is not receipt | `core/src/iron_core.rs:3184-3190,3265-3291,3315-3337` | A successful swarm dispatch keeps the outbox entry enqueued with a grace deadline; the code explicitly distinguishes dispatch from application receipt. |
| Receipt ownership | `core/src/iron_core.rs:3764-3805` | Receipt classification is separated from ordinary content, and a delivered receipt is the event that clears sender retry state. |
| Outbox durability and due policy | `core/src/store/outbox.rs:442-522,524-618,898-962` | Draining, restoration, due checks, retry-now, and retry policy are spread across storage and core behavior; custody entries are excluded from local flush. |
| CLI transport-ack path | `cli/src/api.rs:894-921` | A successful swarm send currently calls `mark_message_sent` and returns `accepted`; this must be reconciled explicitly with the core receipt contract. |
| Alternate CLI path | `cli/src/api_axum.rs:291-304` | The alternate API path independently decides that swarm success releases the outbox entry, so it cannot remain an unverified second acceptance policy. |
| CLI receipt and text filtering | `cli/src/main.rs:3095-3129,3234-3247,3343-3357` | Text metadata filtering, receipt handling, and swarm-ack release are implemented in separate event branches. |
| Delivery regression tests | `core/tests/integration_outbox_flush_reconnect.rs:449-551` | The new tests cover grace expiry, custody exclusion, and non-enqueued no-op behavior, but do not by themselves prove a full text acceptance path. |
| Retry ownership tests | `core/tests/integration_retry_lifecycle.rs:54-323` | Persistence, no terminal drop, custody mutual exclusion, and single ownership are covered. |
| Receipt convergence tests | `core/tests/integration_receipt_convergence.rs:5-110` | Forwarder convergence and duplicate retry cleanup are covered. |
| Round-trip receipt test | `core/tests/integration_ironcore_roundtrip.rs:328-445` | A valid application receipt clears the matching outbox entry and does not enter the inbox. |

Relevant inspected SHA-256 values:

```text
core/src/transport/swarm.rs                         9416607f3265b5b61e5a2c80bff36cb3eab3c319b0b4b638e4293a110675afb3
core/src/iron_core.rs                              0e874b4e4c8c15c705c43f850ac2de036ace06ad85ba074798006bfada6fd77c
core/src/store/outbox.rs                           4b37e8a762f09ec3943db5dfc81b179535590e79c7fe48b096bbf12708aeb192
core/tests/integration_outbox_flush_reconnect.rs  ccb84f78d44d1c780101d7da86dbf949cc5729df98609022d7a8b8d4232cb9c3
core/tests/integration_retry_lifecycle.rs         728affd849e60db1c07e2d4199553c2073758fb14649a00720b1721abdd931e0
core/tests/integration_receipt_convergence.rs     e0b97831fa440e67e722b4fd0ff106d5fbc0e519df1861cd0c640277851f1591
core/tests/integration_ironcore_roundtrip.rs      3023a9a7b026c0aa2b9f14095b1de7367f4937148c7af7ef01fe75da67b44bb8
```

## Product-owned state contract

The owner must explicitly choose and document the meaning of each state before implementation. At minimum, the verifier must distinguish:

```text
queued
transport_acked
delivered
retryable
custody_owned
indeterminate
blocked
```

Required rules:

1. Preparing and queueing a message does not mean delivery.
2. A transport success is not silently equivalent to an application receipt. If the product intentionally treats a particular transport ACK as terminal, that choice must be explicit, versioned, and consistent across CLI, Android, core, and tests.
3. An application receipt is the normal delivery authority. A matching receipt clears retry ownership exactly once.
4. Custody ownership suppresses local outbox retries and must never create dual ownership.
5. A due retry is allowed only for the owning non-custody entry and only once per due epoch.
6. Machine envelopes such as identity or history synchronization cannot satisfy a real-text acceptance assertion.
7. A malformed, unmatched, or ambiguous receipt produces an indeterminate/blocked result; it must not promote a message to delivered.
8. A stale or unverifiable recipient identity fails before persistence or dispatch and cannot silently select another peer.

## Smallest implementation boundary

1. Add or identify one pure verifier in the core product layer. It accepts typed, redacted event facts and returns a typed state/decision; it must not perform network calls, mutate storage, or print message content.
2. Route the existing CLI, alternate API, Android/FFI projection, and receipt path through that decision. Remove duplicated policy only after the core contract is tested.
3. Keep the current wire format and receipt codec. Do not change message kinds, peer identity, or transport framing in this boundary.
4. Add one product-owned acceptance command or test fixture that exercises text send, transport acknowledgement, receipt convergence, and the no-receipt case using redacted IDs and counts.
5. Defer the larger native/WASM scheduler extraction, vendor fork decision, Android lifecycle refactor, and any UI work to separate owner-routed issues.

## Deterministic acceptance commands

These are proposed commands for a separately authorized implementation. They were not run for this documentation-only handoff.

```text
cargo test -p scmessenger-core --test integration_outbox_flush_reconnect test_sweep_reflushes_entry_after_grace_expiry_on_live_connection -- --exact
cargo test -p scmessenger-core --test integration_outbox_flush_reconnect test_sweep_never_drains_custody_entries -- --exact
cargo test -p scmessenger-core --test integration_retry_lifecycle -- --test-threads=1
cargo test -p scmessenger-core --test integration_receipt_convergence -- --test-threads=1
cargo test -p scmessenger-core --test integration_ironcore_roundtrip test_receipt_roundtrip_flips_state -- --exact
cargo test -p scmessenger-core queued_transport_send_remains_in_outbox_until_receipt -- --exact
cargo test -p scmessenger-cli --lib api_send_ignores_stale_contact_public_key -- --exact
cargo test --workspace --features test-utils -- --test-threads=1
cargo fmt --check
```

If the implementation touches Android projections, the separately authorized fixture must additionally run the owner-approved unit-test target for receipt and delivery-state mapping. A live text round trip is a later acceptance step, not a command to run from this handoff.

The acceptance evidence must be redacted: state names, counts, hashes, timestamps, and command exit codes are allowed; message bodies, credentials, private keys, contact identities, and raw peer identifiers are not.

## Compatibility and rollback notes

- Make the verifier additive. Preserve existing serialized outbox, history, receipt, and FFI fields and enum values.
- Unknown, malformed, or missing state must remain conservative (`pending`/`indeterminate`) and must never become `Delivered`.
- Preserve old records and forward-compatible reads. Do not delete, rewrite, or mass-migrate existing delivery state.
- If the owner changes the current transport-ACK meaning, require a versioned contract, migration note, compatibility projection, and independent review; this handoff does not authorize that change.
- Roll back by disabling the new verifier projection or reverting only its isolated implementation. Leave existing outbox/history/custody records untouched.
- Any FFI or Android compatibility work requires its own owner review and generated-binding checks.

## Explicit stop conditions

Stop before implementation if any of the following is true:

- the verifier would be implemented outside the product-owned core boundary;
- a machine envelope or transport ACK is counted as real-text delivery without an explicit owner decision;
- native and WASM retry decisions cannot be shown equivalent;
- a receipt, identity, or custody fact is ambiguous but the proposed result is `Delivered`;
- a stale recipient can be persisted or dispatched before preflight;
- the change requires a vendor update, storage migration, UI feature, device action, message send, or deployment under this handoff alone;
- rollback would require deleting or rewriting delivery records;
- foreign material or a non-owner status is needed to justify the result.

## Unresolved prerequisites and evidence limits

- Owner ruling on the exact terminal meaning of swarm transport acknowledgement versus application receipt.
- A clean isolated SCMessenger worktree and a bounded implementation branch.
- Adversarial security review before any change under `core/src/transport`.
- A deterministic native/WASM test target or an explicit documented platform limitation.
- A separately authorized, redacted real-device text round trip for final acceptance.
- A privacy classification for any operational evidence retained with the verifier.

No implementation, deployment, restart, send, migration, or publication is authorized by this document. It records only the SCMessenger-owned verifier and acceptance boundary.
