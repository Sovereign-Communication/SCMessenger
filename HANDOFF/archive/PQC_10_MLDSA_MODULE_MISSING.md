# TASK: Create the missing core/src/crypto/pq/mldsa.rs module (PQC-10 completion)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO -- BLOCKING compile gate. The previous PQC-10 dispatch modified
`core/src/identity/keys.rs` and `core/Cargo.toml` to reference
`crate::crypto::pq::mldsa::MlDsa65KeyPair` and
`crate::crypto::pq::mldsa::verify(...)` extensively, but NEVER created the
actual `core/src/crypto/pq/mldsa.rs` file, and never added `pub mod mldsa;`
to `core/src/crypto/pq/mod.rs`. This is why the crate does not compile
(7 "cannot find `mldsa` in `pq`" errors).

## Real dependency (already added to core/Cargo.toml, verify it's there)

`ml-dsa = "0.1.1"` (RustCrypto/signatures, FIPS-204 final, pure Rust). If
missing from Cargo.toml, add it.

## Real crate API (verified via docs.rs, use this -- do not guess)

- Parameter set type for ML-DSA-65: `ml_dsa::MlDsa65`.
- `ml_dsa::SigningKey<P>` where `P: MlDsaParams` (e.g. `SigningKey<MlDsa65>`):
  implements `KeyInit`, `KeyExport`, `Keypair`, and RustCrypto `signature`
  crate's `Signer` trait. Has `.verifying_key()` returning the associated
  `VerifyingKey<P>`. Generation via the `Generate` trait (needs the crate's
  `rand_core`-backed feature; check `ml-dsa`'s Cargo features -- may need
  `--features rand_core` or similar enabled in `core/Cargo.toml`'s
  `ml-dsa` dependency line; if generation needs an explicit RNG argument,
  use `rand::rngs::OsRng` per this repo's existing pattern, e.g.
  `core/src/crypto/pq/hybrid.rs` or `core/src/identity/keys.rs`'s existing
  Ed25519/X25519 generation).
- `ml_dsa::VerifyingKey<P>`: implements `KeyInit`, `KeyExport`, and
  RustCrypto `signature::Verifier` trait (`.verify(msg, &signature)`).
- `ml_dsa::Signature<P>` (or similar; check the crate's actual export name
  via `cargo doc --no-deps -p ml-dsa -p scmessenger-core --open` is not
  available headlessly -- instead read the crate's source directly via
  `~/.cargo/registry/src/*/ml-dsa-0.1.1/src/lib.rs` if present after
  `cargo fetch`, or trust the compiler's error messages during the verify
  loop to correct type names): implements `SignatureEncoding` for
  byte conversion.
- Byte<->key conversion goes through the `KeyExport` trait (encoded
  key/signature byte types). Use whatever methods that trait actually
  exposes on `SigningKey`/`VerifyingKey`/`Signature` -- if the exact
  method name is unclear from documentation, try the RustCrypto
  convention (`to_bytes()`/`from_bytes()` or `EncodedX::from(&key)`) and
  let the compiler's error guide the correction.
- Sign via `signature::Signer::sign(&signing_key, msg)` ->
  `Signature<MlDsa65>`. Verify via
  `signature::Verifier::verify(&verifying_key, msg, &signature)` ->
  `Result<(), signature::Error>`.

## Required public API for core/src/crypto/pq/mldsa.rs (exact shape keys.rs needs)

```rust
pub struct MlDsa65KeyPair {
    pub inner: /* the actual ml_dsa::SigningKey<MlDsa65>, or a wrapper
                  around it -- keys.rs does `bincode::serialize(&mldsa_kp.inner)`
                  so `inner` must implement Serialize (check ml-dsa's own
                  serde feature flag -- if the crate doesn't support serde
                  directly, wrap raw bytes instead: store `inner` as the
                  encoded byte Vec<u8> from KeyExport, and reconstruct the
                  real SigningKey on demand inside generate()/sign() --
                  whichever is simpler and still lets keys.rs's existing
                  bincode::serialize(&mldsa_kp.inner) call compile) */
}

impl MlDsa65KeyPair {
    pub fn generate() -> Self { ... }
    pub fn public_key(&self) -> Vec<u8> { ... }  // 1952 bytes, ML-DSA-65 verifying key encoded
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> { ... }  // 3309-byte signature encoded
}

pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
    // Decode public_key into VerifyingKey<MlDsa65>, decode signature into
    // Signature<MlDsa65>, call Verifier::verify. Return Ok(true) on valid,
    // Ok(false) on signature mismatch (do NOT bail!/error on a normal
    // "signature invalid" result -- keys.rs treats a false return as a
    // rejected signature, not a crash). Only Err() for malformed input
    // (wrong-length bytes, undecodable key/signature).
}
```

Cross-check every method your implementation exposes against the ACTUAL
call sites in `core/src/identity/keys.rs` (provided in context) --
`MlDsa65KeyPair::generate()`, `.public_key()`, `keys.sign_mldsa()` (already
exists on `IdentityKeys`, calls into your `sign()`), and
`crate::crypto::pq::mldsa::verify(&sig_input, mldsa_signature, mldsa_public)`
returning something usable as a bool in `if ed_verified && mldsa_verified`.

## Also wire it in

Add `pub mod mldsa;` to `core/src/crypto/pq/mod.rs` (currently only has
`pub mod hybrid;`).

## Also fix in core/src/identity/keys.rs (separate small bug, same file)

Line ~351: `error[E0277]: the size for values of type [u8] cannot be known
at compilation time`. Read the surrounding code (`sig_input.extend_from_slice(mldsa_pub)`
context) -- likely `mldsa_pub` needs a `&` or `.as_slice()`/`.as_ref()` fix,
or a `Vec<u8>` vs `[u8]` deref mismatch. Fix minimally, do not restructure.

## Do NOT

- Do not change `IdentityKeys`, `PublicKeyBundle`, or `sign_mldsa()`'s
  existing logic/signatures beyond what's needed to compile against your
  new `mldsa` module -- the design (dual Ed25519+ML-DSA AND-verification,
  bundle format) is already correct per PQC_10_MLDSA_IDENTITY_SIGNATURES.md
  in HANDOFF/done/ or HANDOFF/todo/ -- read it for the full design intent
  if anything here is ambiguous.
- Do not add ML-DSA to per-message envelopes (out of scope, unchanged).

## Gate

```
cargo check -p scmessenger-core -j 2
cargo test -p scmessenger-core --lib -j 2
```

## Output format (MANDATORY)

Return the FULL updated contents of `core/src/crypto/pq/mldsa.rs` (new
file), `core/src/crypto/pq/mod.rs`, and `core/src/identity/keys.rs` (with
the line-351 fix), each in its own fenced code block, filename as the
first line inside each block.
