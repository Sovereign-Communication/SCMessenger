# TASK: PQC-02 — Envelope v2 wire format with suite tag and PQ fields

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: nothing. Wave 0. Min tier: Sonnet.

## Why this design

`Envelope` (`core/src/message/types.rs`, ~line 74) is serialized with bincode (see `sign_envelope`/`verify_envelope` in `core/src/crypto/encrypt.rs`). bincode is positional: appending fields breaks decoding of old bytes and old peers cannot decode new bytes. Therefore v2 is a NEW struct behind an explicit one-byte wire tag, with fallback decode — the same pattern `core/src/crypto/backup.rs` uses (FORMAT_TAG_ARGON2ID / FORMAT_TAG_BLAKE3).

## Target format

```
wire = 0x02 || bincode(EnvelopeV2)          // v2
wire = bincode(Envelope)                    // v1 (unchanged, no tag)
```

```rust
pub struct EnvelopeV2 {
    pub suite: u8,                          // from PQC-00 suite registry
    pub sender_public_key: Vec<u8>,         // 32 B Ed25519
    pub ephemeral_public_key: Vec<u8>,      // 32 B X25519 (DH ratchet public in ratcheted mode)
    pub nonce: Vec<u8>,                     // 24 B
    pub ciphertext: Vec<u8>,
    pub ratchet_dh_public: Option<Vec<u8>>,
    pub ratchet_message_number: Option<u32>,
    pub pq_kem_ciphertext: Option<Vec<u8>>, // 1088 B ML-KEM-768 ct (session init / PQ ratchet step)
    pub pq_encaps_key: Option<Vec<u8>>,     // 1184 B sender's next ML-KEM encaps key (PQ ratchet)
    pub transcript_hash: Option<Vec<u8>>,   // 32 B suite-negotiation binding (filled by PQC-04)
}
```

## Steps

1. Add `EnvelopeV2` and `WireEnvelope { V1(Envelope), V2(EnvelopeV2) }` to `core/src/message/types.rs`. Add `pub const WIRE_TAG_V2: u8 = 0x02;`.
2. Create ONE encode/decode choke point, e.g. `core/src/message/codec.rs`:
   - `encode_wire_envelope(&WireEnvelope) -> Vec<u8>` (V1 = plain bincode, V2 = tag byte + bincode)
   - `decode_wire_envelope(&[u8]) -> Result<WireEnvelope>`: if `buf[0] == WIRE_TAG_V2`, try V2 decode of `buf[1..]`; on failure, fall through to V1 decode of the whole buffer; else V1 directly. Strict length validation after decode (key/nonce/ct field lengths as commented above; reject otherwise).
3. Discovery step — find EVERY place envelope bytes hit a wire or sled, and route them through the choke point:
   ```bash
   rg -n "bincode::serialize|bincode::deserialize" core/src cli/src wasm/src --type rust | rg -i "envelope"
   rg -n "SignedEnvelope|Envelope" core/src/drift core/src/store core/src/relay --type rust -l
   ```
   List every call site you changed (or deliberately did not change, with reason) in this file. Sled-stored envelopes for inbox/outbox/custody keep their current format if they are local-only — only peer-facing bytes must go through the codec; state which is which.
4. Same treatment for `SignedEnvelope`: `SignedEnvelopeV2 { envelope: EnvelopeV2, signature: Vec<u8> }`. `sign_envelope`/`verify_envelope` v2 variants sign the TAGGED bytes (tag byte included in signed data, so the version itself is authenticated).
5. Tests (unit + proptest in `core/src/message/` and extend `core/tests/` where a wire test exists):
   - v1 bytes produced by the OLD path decode identically through the new codec (byte-for-byte fixture committed in the test).
   - v2 roundtrip for all Option-field combinations.
   - `test_v1_first_byte_disambiguation`: serialize a legacy envelope with real 32-byte keys and ASSERT `bytes[0] != WIRE_TAG_V2`. If this assertion fails on this repo's bincode version, the fallback ordering in step 2 still guarantees correctness — but you MUST then also write a test proving a v1 envelope decodes correctly via the fallback path.
   - proptest: arbitrary bytes into `decode_wire_envelope` never panic (errors only).

## Definition of Done

- [ ] Standard gates (PQC-00) PASS.
- [ ] `cargo test -p scmessenger-core --test integration_e2e` and `--test integration_relay_custody` PASS (proves no wire regression).
- [ ] Call-site inventory written into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Change the layout of the existing v1 `Envelope` struct in ANY way (field order, types, attributes).
- Populate the new pq_* fields anywhere yet (PQC-06/07 do that). They stay `None` in all production paths in this task.
