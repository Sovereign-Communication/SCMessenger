# TASK: PQC-10 — ML-DSA-65 dual signatures for identity operations

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-03. Wave 3. Min tier: Sonnet.

## Why

Audit finding F5: Ed25519 signatures become forgeable once a CRQC exists (not retroactive, but identity keys are long-lived and seniority raises the value of old identities). Identity-level operations get dual signatures: Ed25519 + ML-DSA-65, verified with AND logic.

## Dependency selection (do this first, carefully)

1. Candidates, in preference order: `libcrux` ML-DSA implementation if published as a usable crate at implementation time; RustCrypto `ml-dsa`.
2. Check on crates.io/docs.rs: version, FIPS 204 final (NOT a pre-standard Dilithium API), wasm32 compatibility, no_std/pure-Rust (needed for Android via cargo-ndk), release recency.
3. Record in this file: chosen crate + version + the checks above. Run the Cargo.lock audit (as in PQC-01).
4. If NO candidate satisfies FIPS 204 final + wasm32 + pure Rust: STOP and escalate with findings. Do not hand-roll, do not pick an unmaintained crate silently.

## Design (fixed)

- `IdentityKeys` gains `mldsa_keypair` (PQC-03 pattern: migrate-on-load for existing identities, zeroize, backup payload inclusion).
- `PublicKeyBundle` (PQC-03/04) gains `mldsa_public: Vec<u8>` (1952 B) inside the signed bytes and a SECOND signature field `mldsa_signature: Vec<u8>` (3309 B) over the SAME domain-separated bytes. Bundle format tag bumps again; older tags remain decodable with `mldsa_* = None` semantics.
- Verification rule: if a bundle carries ML-DSA fields, BOTH signatures must verify or the bundle is invalid. A v2-suite peer whose bundle lacks ML-DSA fields is accepted this release (rollout window) but logged; a bundle where ONE of two present signatures fails is REJECTED (never fall back to single-sig).
- Sizes: dual-signed bundle is ~5.5 KB — fine for contact exchange/QR-out-of-band flows that already carry bundles; per-message envelope signatures explicitly STAY Ed25519-only this release (audit rationale: envelope forgery needs a real-time CRQC, no HNDL; revisit when transports comfortably carry +3.3 KB per envelope).

## Steps

1. Dependency selection protocol above.
2. Thin wrapper `core/src/crypto/pq/mldsa.rs` (sign/verify/keygen, length validation, zeroize) + unit tests incl. tamper and KAT-style fixed-vector if the crate provides deterministic signing (ML-DSA has hedged/deterministic modes — use the crate default, record which).
3. Identity + bundle changes, migration, backup roundtrip tests (old backup -> load -> has ML-DSA keys after migration).
4. AND-verification tests: both valid -> OK; ed valid + mldsa tampered -> REJECT; mldsa valid + ed tampered -> REJECT; fields absent -> accepted-with-log.

## Definition of Done

- [ ] Standard gates PASS (wasm32 gate critical again).
- [ ] `cargo test -p scmessenger-core mldsa` and bundle/identity tests green; `--test test_persistence_restart` green.
- [ ] Dependency-selection record + Cargo.lock audit written into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Add ML-DSA signatures to per-message envelopes or relay custody paths (explicitly out of scope this release).
- Implement OR-fallback verification anywhere.

## Dependency selection: PRE-VERIFIED (2026-07-11, via crates.io API, do not re-research)

`libcrux-ml-dsa` does NOT exist on crates.io as of this date (libcrux only
publishes `libcrux-ml-kem`, no ML-DSA crate yet). Use the RustCrypto option
instead: crate `ml-dsa`, version `0.1.1`, repository
https://github.com/RustCrypto/signatures/tree/master/ml-dsa. Confirmed via
crates.io metadata: description states "FIPS-204 (final)" explicitly (not a
pre-standard Dilithium API) -- this satisfies the task's hard requirement.
Pure Rust, rust_version 1.85 (well within this workspace's toolchain).
Verify no_std/wasm32 compatibility empirically via the standard gates
(the wasm32 gate is already required in Definition of Done) rather than
re-researching -- if it fails to compile for wasm32-unknown-unknown, stop
and escalate rather than substituting a different crate silently.
Record `ml-dsa = "0.1.1"` in `core/Cargo.toml` and run the Cargo.lock audit
as instructed in Steps item 1.
