# SCMessenger Schema Versioning Map

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Audit date:** 2026-07-17
**Audited by:** Qwen THINK (qwen3-235b-a22b-thinking-2507) + orchestrator gap-fill
**Ticket:** U7_SCHEMA_DRIFT_AUDIT.md (A-06, Phase 1 Investigation)
**Status:** Phase 1 complete. Phase 2 (implementation) pending operator review.

---

## Section 1: Persistence Format Inventory

| Format | File | Serialization | Version Tag? | Key Prefix |
|---|---|---|---|---|
| history (MessageRecord) | store/history.rs | serde_json | No | msg_ |
| contacts (Contact) | store/contacts.rs | serde_json | No | contact:, contact_bundle: |
| outbox (QueuedMessage) | store/outbox.rs | bincode | No | outbox_ |
| inbox (ReceivedMessage) | store/inbox.rs | bincode | No | inbox_seen_ids, inbox_msg_ |
| transport_memory | store/transport_memory.rs | serde_json | No | tmem: |
| receipt | message/types.rs:55 | serde_json (encode_receipt/decode_receipt) | No | N/A (wire-only) |
| message (Message) | message/types.rs | bincode | No | N/A (wire-only) |
| envelope V1 | message/types.rs:73 | Drift-binary (primary) + bincode (fallback) | Yes (WIRE_TAG_V2=0x02 dispatch) | N/A (wire-only) |
| envelope V2 | message/types.rs:111 | bincode with 0x02 tag prefix | Yes (suite: u8 field + wire tag) | N/A (wire-only) |
| signed envelope V2 | message/types.rs:136 | bincode with tag dispatch | Yes | N/A (wire-only) |
| ledger entry | mobile_bridge.rs:2772 | serde (derive) | Unknown -- scattered | Unknown |
| ledger exchange req/resp | transport/behaviour.rs:341 | serde (derive) | Unknown -- scattered | N/A (wire-only) |

**Key finding:** Wire-format envelopes (V1/V2) are the ONLY structures with
explicit version dispatch (WIRE_TAG_V2 + suite tag). All persisted-to-disk
formats (history, contacts, outbox, inbox, transport_memory) are unversioned.

---

## Section 2: Disk vs Wire Format Comparison

| Format | Disk Format | Wire Format | Divergence? | Notes |
|---|---|---|---|---|
| history | serde_json (pretty) | N/A (not exchanged on wire) | No | Local-only persistence |
| contacts | serde_json | N/A (not exchanged) | No | Local-only; contact_bundle: prefix for key bundles |
| outbox | bincode | envelope_data (binary, Drift-wrapped) | Yes | Outbox stores bincode QueuedMessage; wire payload is Drift envelope |
| inbox | bincode | decrypted payload bytes | Yes | Inbox stores bincode ReceivedMessage; wire payload is envelope ciphertext |
| receipt | N/A (not persisted to disk) | serde_json (encode_receipt) | N/A | Wire-only; Android may persist independently -- needs Kotlin audit |
| message | N/A (not persisted as raw) | bincode (serialize_message) | N/A | Envelope is what goes on wire, not raw Message |
| envelope V1 | N/A | Drift-binary or bincode fallback | No | Wire tag dispatch: 0x01 -> Drift V1, 0x02 -> V2, else bincode V1 |
| envelope V2 | N/A | bincode with 0x02 tag prefix | No | Versioned via wire tag + suite field |
| ledger | Unknown (scattered) | serde (LedgerExchangeRequest/Response) | Unknown | Structures spread across 3 files; needs consolidation audit |
| transport_memory | serde_json | N/A (not exchanged) | No | Local-only; per (peer, network_fingerprint) |

---

## Section 3: Migration Registry

### Existing Migrations

1. **history.rs `adjust_legacy_timestamps()`** (line 34)
   - Called at lines 121, 160, 253, 280, 339 (every read path)
   - Handles legacy records where `sender_timestamp` was 0; falls back to
     `received_timestamp`
   - Applied at read time (lazy migration, not write-time)

2. **contacts.rs `migrate_unprefixed_contacts()`** (line 88)
   - One-time migration for contacts stored under bare keys before prefixing
   - Called during ContactManager initialization
   - Idempotent; runs once on startup
   - Has test coverage (lines 547, 553, 566)

### Registry Assessment

- **No central migration registry exists.** Migrations are ad-hoc functions
  called at read time (history) or initialization (contacts).
- No version tracking in stored data -- migrations detect old format by
  data shape (zero timestamp, bare key), not by an explicit version field.
- History migration has no test. Contacts migration has test coverage.
- No mechanism for schema evolution of bincode formats (outbox, inbox) --
  bincode is brittle to field additions/removals.

---

## Section 4: Drift Assessment

| Format | Assessment | Details |
|---|---|---|
| history | **No drift** (with caveats) | JSON resilient to schema changes; lazy migration handles legacy timestamps; no version field but JSON's tolerance provides implicit forward-compat |
| contacts | **No drift** (with caveats) | JSON resilient; init-time migration handles key prefix change; test coverage exists |
| outbox | **Explicit drift risk** | Bincode is brittle (field addition breaks deserialization); no version tag; no migration; no test for cross-version compat |
| inbox | **Explicit drift risk** | Bincode brittle; `sender_public_key_hex` was added (per A-06 THINK analysis); no version tag; no migration |
| receipt | **No drift** (wire-only) | JSON via encode_receipt/decode_receipt is canonical; verify Android/iOS use same path via UniFFI |
| message | **No drift** (wire-only) | bincode serialize_message/deserialize_message; wire-only, not persisted to disk in core |
| envelope V1 | **No drift** | Versioned via wire tag + Drift-binary format; backward compat via bincode fallback |
| envelope V2 | **No drift** | Versioned via suite tag + wire tag; PQ fields optional (Option<>); E-00 added backward-compatible trailing PQ block |
| ledger | **High drift risk** | Structures scattered across mobile_bridge.rs, behaviour.rs, swarm.rs; no single source of truth; serialization format unverified |
| transport_memory | **No drift** (with critical caveat) | JSON resilient; BUT `get_network_fingerprint()` returns hardcoded "placeholder_network_fingerprint" (line 73) -- transport memory is non-functional until real fingerprint is implemented |

---

## Section 5: Recommendations

### Critical Priority

1. **Add versioning to bincode formats (outbox, inbox):**
   - Add a `version: u8` field to `QueuedMessage` and `ReceivedMessage`
   - Implement versioned deserialization: read version byte first, dispatch
   - Add migration tests: write v0 data, read with v1 code, verify success

2. **Consolidate ledger structures:**
   - Move `LedgerEntry`, `LedgerManager` from `mobile_bridge.rs` to a
     dedicated `store/ledger.rs` or `store/ledger_entry.rs`
   - Move `LedgerExchangeRequest/Response` wire format alongside it
   - Add explicit version tag to the wire format

3. **Implement transport_memory network fingerprint:**
   - `get_network_fingerprint()` returning a placeholder is a known
     gap -- transport memory (P1-12 "remember" path) is non-functional
   - Implement: hash of default-gateway MAC + subnet /24 (per the TODO
     comment at transport_memory.rs:72)

### Medium Priority

4. **Add version field to JSON formats (history, contacts):**
   - Add `schema_version: u8` to `MessageRecord` and `Contact`
   - Replace ad-hoc migrations with version-dispatched migration registry
   - Add tests for each migration path

5. **Cross-platform serialization audit:**
   - Verify Android/iOS use UniFFI-provided `encode_receipt`/`decode_receipt`
     (not independent serialization)
   - Verify no Kotlin/Swift code independently serializes Message/Envelope
   - Check Android MeshStore and iOS equivalent for local persistence format

### Testing Recommendations

6. **Cross-version serialization tests:**
   - Test reading v0.3.x data with v1.0.0 code (verify migration or error)
   - Test ledger_exchange round-trip: serialize -> deserialize -> compare
   - Test receipt round-trip across platforms

---

## Section 6: Open Questions (require Kotlin/Swift source verification)

1. **Android MeshStore persistence:** Does Android's MeshStore serialize
   messages/receipts independently of core's store module, or does it
   delegate to core via UniFFI? (Check `MeshRepository.kt`, any Room/SQLite
   schema definitions.)

2. **iOS persistence:** Does iOS have independent Core Data / SQLite
   persistence with its own schema, or does it use core via UniFFI?

3. **UniFFI boundary:** How are complex types (MessageRecord, Contact,
   Receipt) marshalled across the UniFFI boundary? Is there any
   transformation that could cause schema drift?

4. **Ledger exchange wire format:** Are `LedgerExchangeRequest/Response`
   serialized with the same serde config on all platforms? (They go through
   libp2p's protocol framing, which uses serde -- verify no platform-specific
   serde attributes.)

5. **Error handling for schema drift:** How do mobile platforms handle
   deserialization failures from older data? Is there graceful degradation
   or does it crash?

---

## Notes

- This audit covers core Rust source only. Kotlin/Swift persistence
  verification is a follow-up task (can be dispatched to Qwen CODER with
  the relevant Android/iOS source files).
- The `transport_memory` placeholder fingerprint is tracked separately
  as part of P1-12 (C-03 "remember" path) which is blocked on H-03.
- Phase 2 implementation (adding versioning, consolidation, migration
  registry) should be dispatched after this report is operator-reviewed.
