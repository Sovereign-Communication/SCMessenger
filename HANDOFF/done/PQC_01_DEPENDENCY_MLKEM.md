# TASK: PQC-01  Add ML-KEM-768 dependency and smoke-test module

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: nothing. Wave 0. Min tier: Haiku.

## Chosen Version and Justification

- **Crate:** `libcrux-ml-kem`
- **Version:** `0.0.9`
- **Justification:** This is the latest pre-release stable version of FIPS 203 ML-KEM implemented by Cryspen, which is formally verified and used by Signal. The default features are disabled, and only the `mlkem768` feature is enabled as required.

## Scope

Add `libcrux-ml-kem` to the workspace and prove it compiles and round-trips on every target this repo builds. No integration with any existing crypto path  that is PQC-05/06.

## Steps

1. Add to root `Cargo.toml` under `[workspace.dependencies]`: `libcrux-ml-kem = { version = "0.0.9", default-features = false, features = ["mlkem768"] }`. Recorded above.
2. Add `libcrux-ml-kem = { workspace = true }` to `core/Cargo.toml` `[dependencies]`.
3. Create `core/src/crypto/pq/mod.rs` (and register `pub mod pq;` in `core/src/crypto/mod.rs`) containing thin wrappers:
   - `MlKem768KeyPair` wrapping public key bytes (`[u8; 1184]`) and private key bytes wrapped in zeroizing type `MlKem768PrivateKey` (`[u8; 2400]`).
   - `generate()` generating from `OsRng` using 64-byte seeds.
   - `encapsulate()` validating key length (1184) and encapsulating with 32-byte randomness.
   - `decapsulate()` validating ciphertext length (1088) and decapsulating using private key bytes.
4. Unit tests in the same file:
   - `test_roundtrip`: round-trip test.
   - `test_wrong_lengths`: wrong length input checks.
   - `test_tampered_ciphertext`: implicit rejection validation.
5. Security rule audit: documented below.

## Definition of Done

- [x] Standard gates (PQC-00) all PASS (workspace check, wasm32 check, clippy, cargo fmt all pass)
- [x] Android NDK check (`cargo ndk -t arm64-v8a check -p scmessenger-core`) passes
- [x] New unit tests pass (`cargo test --lib -p scmessenger-core pq`)
- [x] Cargo.lock audit section written.
- [x] File moved to `HANDOFF/done/` + committed.

## Cargo.lock Audit

As part of the security audit rule, the dependency changes added to `Cargo.lock` have been audited. The following crates were added to the dependency tree (all expected and required transitive dependencies for `libcrux-ml-kem` and the required testing framework):

| Crate | Version | Role / Justification |
|-------|---------|----------------------|
| `libcrux-ml-kem` | 0.0.9 | Target post-quantum ML-KEM cryptographic library. |
| `libcrux-platform` | 0.0.3 | libcrux CPU feature detection platform library. |
| `libcrux-intrinsics` | 0.0.7 | SIMD intrinsics helper crate for libcrux. |
| `libcrux-secrets` | 0.0.5 | Secret/memory validation helper crate for libcrux. |
| `libcrux-sha3` | 0.0.9 | SHA-3 hash implementation used by FIPS 203 ML-KEM. |
| `libcrux-traits` | 0.0.7 | Cryptographic traits for libcrux. |
| `hax-lib` | 0.3.6 | Formal verification annotations crate used by libcrux. |
| `hax-lib-macros` | 0.3.6 | Macro helpers for formal verification. |
| `hax-lib-macros-types` | 0.3.6 | Type definitions for formal verification macros. |
| `core-models` | 0.0.6 | Core types library used by hax-lib. |
| `pastey` | 0.2.3 | String paste utility crate for core-models. |
| `proc-macro-error2` | 2.0.1 | Error reporting for procedural macros. |
| `proc-macro-error-attr2` | 2.0.0 | Attribute macro support for proc-macro-error2. |
| `rand` | 0.10.2 | Random number generation dependency (v0.10) for libcrux. |
| `rand_core` | 0.10.1 | Core random traits for rand v0.10. |
| `chacha20` | 0.10.1 | ChaCha20 cipher implementation for rand v0.10. |

No unexpected or untrusted packages were added. All packages map directly to the `libcrux-ml-kem` dependency chain.

## Do NOT

- Touch `encrypt.rs`, `ratchet.rs`, `session_manager.rs`, or any wire format.
- Add any other new dependency.
