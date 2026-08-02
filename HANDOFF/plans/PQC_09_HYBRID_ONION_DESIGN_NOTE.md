# PQC-09 Hybrid Onion Routing Design Note

**Date:** 2026-08-01
**Status:** Investigation complete, parked pending PQC-05/06/07 adversarial pass
**Threat Model:** Forward secrecy + PQ resistance (no quantum computers today; harvest-now-decrypt-later mitigation)

---

## Current Design (X25519 + Hybrid Onion)

The existing onion routing implementation in `core/src/privacy/onion.rs` (863 lines, fully tested) already supports both classical (X25519-only) and hybrid (X25519 + ML-KEM-768) onion circuits. The design is a standard Sphinx-like layered encryption scheme: given a path `[hop1, hop2, ..., hopN, destination]`, the sender constructs the onion in reverse order, encrypting the payload for the destination first, then wrapping with each relay's key material. Each relay peels exactly one layer, learns only the next hop address, and forwards the remaining ciphertext.

Two layer formats coexist. `ClassicalOnionLayer` (version 0x01) carries an ephemeral X25519 public key, XChaCha20-Poly1305 encrypted routing info, and an encrypted payload. `HybridOnionLayer` (version 0x02) replaces the ephemeral key with a `HybridCiphertext` (X25519 ephemeral + 1088-byte ML-KEM-768 ciphertext) and adds an explicit `is_destination` flag. Key derivation uses Blake3 (`SCMessenger-onion-layer-key-v1` context) over the shared secret, and nonces are deterministic per-layer via a counter-based Blake3 derivation. The `HopAddress` enum discriminates classical hops (32-byte X25519 key) from hybrid hops (full `PublicKeyBundle` including ML-KEM encapsulation key), and `construct_onion` accepts a `require_pq` flag that rejects mixed paths when strict PQ mode is requested. Measured envelope sizes for 3-hop circuits: classical-only approximately 384 bytes, hybrid-only approximately 3744 bytes, mixed approximately 2624 bytes.

The hybrid KEM combiner in `core/src/crypto/pq/hybrid.rs` concatenates both shared secrets (X25519 ECDH + ML-KEM-768 decapsulation) plus all public material into an IKM, then derives a 32-byte key via `blake3::derive_key("iron-core hybrid-kem v1 X25519+MLKEM768 2026-07", ikm)`. ML-KEM implicit rejection means tampered ML-KEM ciphertexts do not error at decapsulation but yield a different shared secret, causing AEAD authentication failure downstream.

---

## Hybrid Options

### Option A: Hybrid Encapsulation at Each Hop (IMPLEMENTED)

**Design.** Each hop's layer uses the hybrid KEM combiner: the sender generates an ephemeral X25519 keypair, performs X25519 ECDH with the hop's static X25519 key, and simultaneously encapsulates to the hop's ML-KEM-768 public key. Both shared secrets feed the Blake3 combiner to produce the per-layer encryption key. The `HybridCiphertext` (ephemeral X25519 public key + ML-KEM ciphertext) is included in the layer. Each hop is independently chosen: a path may be all-classical, all-hybrid, or mixed. The `construct_onion` function already supports this, with `require_pq` gating whether mixed paths are permitted.

**Pros.**
- Already implemented and tested (8 unit tests including mixed-path peel, tamper detection, strict-mode rejection).
- Per-hop independence: each relay can upgrade to PQ at its own pace; classical relays still function in mixed paths.
- The security guarantee is conjunctive: the layer key is secure if EITHER X25519 ECDH OR ML-KEM-768 remains unbroken, because the Blake3 combiner binds both shared secrets.
- Backward compatible: classical-only circuits produce version 0x01 envelopes identical to pre-PQC behavior.

**Cons.**
- Per-hop overhead: each hybrid layer adds approximately 1120 bytes (1088 ML-KEM ciphertext + 32 X25519 ephemeral) compared to classical. A 5-hop all-hybrid circuit would be approximately 6144 bytes of envelope alone.
- ML-KEM encapsulation is performed at construction time for every hop, meaning the sender does N encapsulations per message (acceptable for N <= 5).
- No information-theoretic separation between classical and PQ security domains; both feed the same KDF.

### Option B: Post-Quantum Inner Layer Wrapping X25519 Outer

**Design.** The message payload is first encrypted under an ML-KEM-768 derived key (the "PQ inner layer"), then the resulting ciphertext is wrapped in N classical X25519 onion layers. The inner layer is only decryptable by the destination, which holds the ML-KEM decapsulation key. Relays process classical X25519 layers as normal and never see PQ key material.

**Pros.**
- Clean separation of concerns: relays need no PQ awareness; only the destination needs ML-KEM keys.
- Lower per-hop overhead: relay layers remain 32-byte-ephemeral classical format.
- Simpler relay implementation: no changes to relay software required.

**Cons.**
- Does NOT protect routing metadata against a quantum adversary. A quantum-capable passive observer who records the classical X25519 key exchange at each hop can later recover all per-hop shared secrets, reconstruct the routing path, and correlate sender to receiver. Only the payload content is PQ-protected.
- Defeats the primary purpose of onion routing (hiding the communication graph) against a harvest-now-decrypt-later adversary.
- The destination must perform ML-KEM decapsulation in addition to X25519 ECDH, but this is a single operation regardless of hop count.

### Option C: Parallel Paths

**Design.** The sender constructs two independent onion circuits: one all-X25519 (classical) and one all-hybrid (PQ). The message is split or duplicated across both paths. At runtime, the sender selects which path to use based on peer capability advertisement, or sends on both for redundancy.

**Pros.**
- Maximum deployment flexibility: classical path works today with all peers; PQ path activates when both endpoints and sufficient relays support it.
- Fallback is trivial: if the PQ path fails (missing ML-KEM keys at a relay), the classical path still delivers.
- Clean versioning: the two envelope versions (0x01 and 0x02) already exist.

**Cons.**
- Sending on both paths doubles bandwidth and creates two traffic patterns that a global passive adversary can correlate, weakening anonymity.
- If the message is split (secret-shared) rather than duplicated, reliability drops: losing either circuit loses the message.
- Does not actually provide hybrid security for a single message: the classical path remains vulnerable to harvest-now-decrypt-later, and the PQ path's anonymity depends on whether enough PQ-capable relays exist to form diverse circuits.
- Circuit construction complexity: the sender must discover and maintain two separate relay pools.

---

## Recommendation

**Option A (Hybrid Encapsulation at Each Hop) is recommended** for the farm threat model.

**Rationale.**

1. **The threat model is harvest-now-decrypt-later.** The adversary has no quantum computer today but records encrypted traffic for future decryption. Option A directly addresses this: every hop's key exchange includes ML-KEM-768, so a future quantum adversary cannot recover any per-hop shared secret, and therefore cannot reconstruct the routing path or the message. Option B fails this threat model because the classical outer layers leak routing metadata. Option C fails if the classical path is used (which it must be, given current relay deployment).

2. **It is already implemented.** The code in `core/src/privacy/onion.rs` is complete, tested, and handles mixed classical/hybrid paths. The hybrid KEM combiner in `core/src/crypto/pq/hybrid.rs` is verified with roundtrip, tamper, and contributory-check tests. Choosing Option A means zero additional cryptographic design work.

3. **Forward secrecy is preserved.** Each hop uses a fresh ephemeral X25519 keypair. Compromising a relay's long-term X25519 key does not compromise past sessions (the ephemeral keys are discarded after construction). The ML-KEM component adds PQ resistance without weakening the classical forward secrecy property.

4. **The overhead is acceptable.** An additional 1120 bytes per hop yields approximately 3744 bytes for a 3-hop all-hybrid circuit. For a messaging application with typical payload sizes of 1-16 KB, this 10-30% overhead is a reasonable trade-off for PQ-resistant anonymity. The `MAX_ONION_HOPS` constant of 5 bounds worst-case envelope size to approximately 6 KB.

5. **Mixed-path support enables incremental deployment.** The `require_pq` flag allows operators to choose between strict mode (all hops must be PQ-capable) and opportunistic mode (use PQ where available). This is essential during the transition period when not all relays have published ML-KEM keys.

---

## Integration Spec

### Position in Message Flow

Onion routing sits between the ratchet-encrypted message payload and the transport layer:

```
Plaintext message
  -> PQC-07 double ratchet encryption (end-to-end)
    -> Onion construction (hop-by-hop layered encryption)
      -> Transport (libp2p swarm / BLE / WiFi relay)
```

The ratchet produces an end-to-end encrypted ciphertext. That ciphertext becomes the `payload` argument to `construct_onion()`. The onion layers provide routing anonymity; the ratchet provides end-to-end confidentiality and forward secrecy. They are independent: the ratchet session is between sender and destination, while onion layers are between sender and each relay (and between consecutive relays).

### Integration with PQC-07 Ratcheting

The PQC-07 double ratchet (`core/src/crypto/ratchet.rs`) already has PQ extensions:
- `SessionState` carries `pq_our_keypair`, `pq_prev_keypair`, `pq_their_encaps_key`, and `pq_pending_ct` for ML-KEM-768.
- A "trial adoption" pattern holds `PendingPqSecret` values until the peer confirms mixing.
- V2 root chain context (`iron-core root-chain v2 hybrid 2026-07`) binds PQ material into the ratchet.

**Independence property.** The ratchet and onion routing operate at different layers and MUST remain independent:
- The ratchet provides end-to-end security between communication partners. It runs once per message.
- Onion routing provides hop-by-hop anonymity. It runs once per relayed message, at the sender only.
- The ratchet's PQ key exchange uses the peer's ML-KEM key from their `PublicKeyBundle`. Onion routing uses each relay's ML-KEM key from the same bundle type. The keys are the same type but serve different purposes.
- A PQ ratchet session does NOT require onion routing, and vice versa.

**Interaction point: key distribution.** Both systems depend on `PublicKeyBundle` (`core/src/identity/keys.rs:305`) carrying ML-KEM encapsulation keys. The `supported_suites` field in the bundle advertises whether a peer supports suite 0x02 (hybrid). The circuit builder MUST check this field when selecting relays for PQ-capable paths.

### Negotiation Protocol

1. **Peer discovery:** Each node publishes its `PublicKeyBundle` via the DHT or peer exchange. The bundle includes `mlkem_encaps_key` (1184 bytes) and `supported_suites` (e.g., `[0x02]` for hybrid).
2. **Circuit construction:** `CircuitBuilder` filters eligible peers by `min_reliability` and network diversity. For PQ circuits, it additionally filters by `supported_suites.contains(0x02)`.
3. **Strict vs. opportunistic:** The caller passes `require_pq=true` to `construct_onion` for sensitive messages (rejects mixed paths) or `require_pq=false` for best-effort PQ (allows mixed paths with classical fallback).
4. **Version signaling:** The outermost layer determines the envelope version byte (0x01 if the first hop is classical, 0x02 if hybrid). Each relay reads the version to determine how to peel its layer. Mixed paths require the relay to re-wrap the remaining layers with the correct version for the next hop -- this is handled by the `(is_hybrid, next_hop_data)` tuple in the routing info.

### Compatibility

- Classical-only nodes (no ML-KEM keys) can still relay messages. They produce and consume version 0x01 envelopes.
- A hybrid envelope arriving at a classical relay is a protocol error (the relay lacks ML-KEM keys). The `peel_layer` function returns `OnionError::MissingPqKeys`.
- Circuit construction MUST NOT select a classical relay for a hop that will receive a version 0x02 envelope. The `next_hop_info` encoding handles this: it carries `(is_hybrid: bool, data)` so each relay knows the next hop's type before forwarding.

---

## Risk Analysis

### Complexity

**Risk: HIGH for deployment, LOW for code.** The cryptographic implementation is complete and tested. The deployment complexity comes from key distribution: every relay must publish ML-KEM-768 keys, and the circuit builder must discover enough PQ-capable relays to form diverse 3-5 hop paths. With fewer than `min_hops` PQ relays, strict PQ mode (`require_pq=true`) will always fail.

**Mitigation:** Staged rollout. Phase 1: deploy ML-KEM key generation to all nodes (already done in `IdentityKeys::generate()`). Phase 2: enable opportunistic hybrid onion (`require_pq=false`) so mixed paths work. Phase 3: switch to strict mode once sufficient relay density exists.

### Versioning

**Risk: LOW.** The version byte in `OnionEnvelope` cleanly separates classical (0x01) from hybrid (0x02). The `next_hop_info` tuple in routing info allows a classical relay to forward to a hybrid hop and vice versa. The main versioning concern is envelope size: hybrid envelopes are 10x larger than classical. Transport framing must accommodate the maximum possible envelope (5 hybrid hops approximately 6144 bytes).

**Mitigation:** The transport layer already handles variable-length messages. The `MAX_ONION_HOPS` constant (5) provides a hard bound. Document the maximum envelope size in the transport framing spec.

### Fallback

**Risk: MEDIUM.** If a relay's ML-KEM key is stale or missing, hybrid circuit construction fails for that hop. The fallback path is to use a classical hop instead (opportunistic mode) or abort (strict mode). There is no runtime fallback once the onion is constructed: if a hybrid envelope reaches a node that has lost its ML-KEM decapsulation key, decryption fails and the message is dropped.

**Mitigation:**
- ML-KEM keys are part of `IdentityKeys` and persisted alongside X25519 keys. Key rotation uses the same mechanism as X25519 (generate new bundle, publish, keep old keypair for one rotation period).
- The `pq_prev_keypair` field in `SessionState` (from PQC-07 ratchet) provides one-step-backward tolerance for the ratchet; onion routing needs a similar mechanism. Recommend adding `previous_mlkem_keypair` retention to the onion peeling path.
- Dead-letter reporting: when `peel_layer` returns `MissingPqKeys`, the relay should send a delivery failure notification back through the circuit (if possible) or log the event for circuit health monitoring.

### Performance

**Risk: LOW.** ML-KEM-768 encapsulation takes approximately 0.1 ms per operation on modern ARM. A 5-hop hybrid circuit requires 5 encapsulations at construction time (approximately 0.5 ms total) and 1 decapsulation per relay (approximately 0.15 ms each). This is negligible compared to network latency in a mesh (BLE/WiFi hop times are 10-100 ms).

**Mitigation:** None needed. Benchmark on target Android hardware during implementation phase.

### Envelope Size Traffic Analysis

**Risk: MEDIUM.** A hybrid envelope (approximately 3744 bytes for 3 hops) is trivially distinguishable from a classical envelope (approximately 384 bytes). An observer can identify which messages use PQ onion routing, potentially correlating PQ users as a distinct anonymity set.

**Mitigation:** Padding. The `padding.rs` module already provides `pad_to_next_standard_size`. Define standard size buckets that accommodate both classical and hybrid envelopes (e.g., pad classical envelopes to the hybrid size when PQ mode is active, or define a unified bucket at 4096 bytes). When onion routing is enabled, always pad to the PQ bucket size regardless of whether the circuit is hybrid or classical.

---

## Test Scenarios

### Scenario 1: Three-Hop All-Hybrid Circuit Construction and Peel

**Setup:** Generate 3 hybrid `HopAddress` entries (each with X25519 + ML-KEM keys). Construct an onion with `require_pq=true`.
**Action:** Peel each layer sequentially, simulating relay processing.
**Expected:** Each peel reveals the correct next hop as `HopAddress::Hybrid`. The final peel returns `(None, original_payload)`. All peels succeed without error.
**Already tested:** `test_construct_onion_all_hybrid`, `test_peel_layer_hybrid_destination`, `test_peel_layer_hybrid_relay`.

### Scenario 2: Mixed Classical-Hybrid Path with Relay Transition

**Setup:** Construct a 3-hop path: `[Classical, Hybrid, Classical]`. Build onion with `require_pq=false`.
**Action:** Peel layer 1 (classical relay). Verify it reveals hop 2 as `HopAddress::Hybrid` with full `PublicKeyBundle`. Peel layer 2 (hybrid relay). Verify it reveals hop 3 as `HopAddress::Classical`. Peel layer 3 (classical destination).
**Expected:** Cross-version transitions work correctly. The classical relay correctly deserializes the hybrid next-hop info. The hybrid relay correctly deserializes the classical next-hop info. Final payload matches original.
**Already tested:** `test_construct_onion_mixed_path`, `test_peel_layer_mixed_path`.
**Gap to fill:** End-to-end 3-hop mixed peel test (existing test only does 3 hops with 2 peels; add a test that peels all 3 layers to plaintext).

### Scenario 3: Tampered ML-KEM Ciphertext Detection

**Setup:** Construct a 2-hop hybrid circuit. Flip a bit in the ML-KEM ciphertext of the outer layer's `HybridCiphertext`.
**Action:** Attempt to peel the tampered layer.
**Expected:** `peel_layer` returns `OnionError::DecryptionFailed`. The ML-KEM implicit rejection causes decapsulation to produce a different shared secret, which leads to AEAD authentication failure when decrypting the payload.
**Already tested:** `test_tampered_hybrid_layer_fails_cleanly`.
**Gap to fill:** Tamper the X25519 ephemeral public key instead of the ML-KEM ciphertext; verify same failure mode. Also test: tamper inner layer while outer layer is intact; verify outer peel succeeds but inner peel fails.

### Scenario 4: Strict PQ Mode Rejects Mixed Paths

**Setup:** Create a path with 2 hybrid hops and 1 classical hop. Call `construct_onion` with `require_pq=true`.
**Action:** Verify construction fails.
**Expected:** Returns `OnionError::MixedHopsNotAllowed`. No envelope is produced.
**Already tested:** `test_construct_onion_strict_mode_rejects_mixed`.
**Gap to fill:** Property-based test: for any random path containing at least one classical hop, `require_pq=true` always rejects.

### Scenario 5 (NEW): Envelope Size Bounding and Padding Compatibility

**Setup:** Construct onions for all combinations: 3-hop classical, 3-hop hybrid, 5-hop hybrid, 1-hop classical.
**Action:** Measure serialized envelope sizes. Apply `pad_to_next_standard_size` from the padding module.
**Expected:** All padded envelopes fall into defined size buckets. The 5-hop hybrid envelope (approximately 6144 bytes before padding) does not exceed transport MTU limits. Classical envelopes padded to PQ bucket sizes are indistinguishable from hybrid envelopes by size alone.
**Gap to fill:** This test does not exist yet. It requires defining the standard size buckets and integrating padding into the onion construction path.

### Scenario 6 (NEW): Missing ML-KEM Key at Relay

**Setup:** Construct a 2-hop hybrid circuit. At peel time, pass `None` for the relay's ML-KEM keypair.
**Action:** Call `peel_layer` with the version 0x02 envelope but no ML-KEM key.
**Expected:** Returns `OnionError::MissingPqKeys`. The relay does not attempt partial decryption with X25519 alone.
**Partially tested:** The error variant exists but no test exercises it directly.
**Gap to fill:** Dedicated test for `MissingPqKeys` error path.

---

## Open Questions

1. **Key rotation for onion relays.** The PQC-07 ratchet has `pq_prev_keypair` for one-step-backward tolerance. Onion routing has no equivalent. Should a relay retain its previous ML-KEM decapsulation key for a grace period? Recommended: yes, retain one previous keypair, matching the ratchet pattern.

2. **Padding integration point.** Should `construct_onion` apply padding internally, or should the caller (in `IronCore` or the message pipeline) pad after construction? Recommended: the caller, because padding policy depends on privacy config, not on the onion algorithm itself.

3. **Circuit builder PQ awareness.** `CircuitBuilder` currently selects hops by reliability and network diversity. It does not filter by `supported_suites`. Should there be a `require_pq_hops` flag on `CircuitConfig`? Recommended: yes, add this as a config field alongside the existing `min_reliability`.

4. **ML-KEM ciphertext malleability.** ML-KEM implicit rejection means a tampered ciphertext produces a different (but valid-looking) shared secret. The AEAD layer catches this, but the error message is `DecryptionFailed` rather than a more specific `PqIntegrityFailed`. Is this distinction important for diagnostics? Recommended: no, the generic error prevents information leakage about which cryptographic component failed.

---

## Status

Parked pending PQC-05/06/07 adversarial review completion.

**Blocked by:** PQC-05/06/07 adversarial review (standing rule: no wave 3 until wave 2 audited)
**Blocks:** PQC-10 (ML-DSA identity), implementation waves

## Implementation Dependencies

When this work is un-parked, the following changes are needed (in order):

1. Add `previous_mlkem_keypair` retention to the onion peeling path (mirrors PQC-07 ratchet).
2. Add `require_pq_hops` field to `CircuitConfig` and filtering logic to `CircuitBuilder`.
3. Define standard padding size buckets that accommodate hybrid envelope sizes.
4. Wire `construct_onion` / `peel_layer` into the live message pipeline in `IronCore` (currently only called from tests).
5. Add the 4 new test scenarios identified above.
6. Adversarial review of the hybrid KEM combiner and onion construction/peel logic.
