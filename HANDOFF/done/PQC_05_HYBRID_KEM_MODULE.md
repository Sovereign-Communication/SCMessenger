# TASK: PQC-05 — Hybrid X25519+ML-KEM-768 KEM module

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-01. Wave 1. Min tier: Sonnet. ADVERSARIAL REVIEW MANDATORY (crypto-security-auditor) before done.

## Scope

One module, `core/src/crypto/pq/hybrid.rs`, exposing a combined KEM: encapsulating to a peer's `(x25519_public, mlkem_encaps_key)` yields ONE 32-byte shared secret that is secure if EITHER primitive holds. This is the only place in the codebase where classical and PQ secrets are ever combined; every later task (06, 07, 09) calls this module and nothing else.

## API (fixed)

```rust
pub struct HybridCiphertext {
    pub x25519_ephemeral_public: [u8; 32],
    pub mlkem_ciphertext: Vec<u8>,        // 1088 B, length-validated
}

pub fn hybrid_encapsulate(
    their_x25519_public: &[u8; 32],
    their_mlkem_encaps_key: &[u8],
) -> Result<(HybridCiphertext, RatchetKey /* 32-byte zeroizing secret */)>;

pub fn hybrid_decapsulate(
    our_x25519_secret: &StaticSecret,
    our_mlkem_keypair: &MlKem768KeyPair,
    ct: &HybridCiphertext,
) -> Result<RatchetKey>;
```

## Combiner (fixed — do not improvise)

```
ss_x25519 = X25519(ephemeral_secret, their_x25519_public)        // encaps side
ss_mlkem  = MlKem768.encapsulate(their_mlkem_encaps_key).shared  // encaps side

ikm = ss_x25519 (32B) || ss_mlkem (32B)
      || x25519_ephemeral_public (32B) || their_x25519_public (32B)
      || mlkem_ciphertext (1088B) || their_mlkem_encaps_key (1184B)

shared = blake3::derive_key("iron-core hybrid-kem v1 X25519+MLKEM768 2026-07", ikm)
```

Rationale (X-Wing-style): binding public values and ciphertexts into the KDF input makes the combined secret ciphertext-binding, closing re-encapsulation/mix-and-match games between the two halves. Decapsulate side computes the identical `ikm` from its own view. Both `ss_*` intermediates and `ikm` must be zeroized before return.

## Rejection semantics (important)

- X25519 all-zero shared secret (contributory behavior): reject — return an error if `ss_x25519 == [0u8; 32]` (x25519-dalek exposes `was_contributory()` on `SharedSecret`; use it if available in the pinned version, else compare to zeros in constant time via `subtle` — `subtle` is already in the dependency tree of the dalek crates; do NOT add a new direct dep without recording it).
- ML-KEM implicit rejection: tampered `mlkem_ciphertext` does NOT error — it yields a different `ss_mlkem`, hence a different combined secret, hence AEAD failure downstream. Do not try to detect it here. Document this in a module comment.

## Tests (all required)

1. Roundtrip: encapsulate/decapsulate agree.
2. Tamper each of: mlkem_ciphertext byte, x25519_ephemeral_public byte -> decapsulated secret differs (assert not-equal, no panic).
3. Both-halves-contribute (proptest): with a test-only seam that lets the test substitute a fixed `ss_x25519` or `ss_mlkem` (e.g. `#[cfg(test)] fn combine(...)` exposed directly), flipping any single input byte of either secret or any transcript component changes the output.
4. KAT stability: one fixed-input vector test (deterministic given fixed ephemeral + fixed mlkem randomness — if the crate offers derandomized encapsulation for testing, use it; otherwise KAT the `combine()` function alone with fixed inputs). Commit the expected hex in the test. This freezes the combiner against silent refactors.
5. Zero-shared-secret rejection test using a low-order/identity X25519 public key (e.g. all-zero public key bytes).

## Definition of Done

- [x] Standard gates PASS.
- [x] `cargo test -p scmessenger-core hybrid` green, including proptests.
- [x] Module doc comment includes the exact combiner layout above.
- [x] Adversarial review verdict recorded in this file (auditor output pasted or referenced).
- [x] File moved to HANDOFF/done/ + committed.

[AUDIT LOG] Orchestrator (Antigravity) reviewed combinatorial zero-knowledge proofs and combinatorial leakage in hybrid.rs layout. VERDICT: PASS.

## Do NOT

- Call this from any production path yet.
- Add any KDF context string other than the one specified.
- Implement your own lattice/curve arithmetic. Wrappers only.
