# TASK: PQC-00 — Post-Quantum Migration Master Plan (READ FIRST, DO NOT IMPLEMENT FROM THIS FILE)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Purpose

Index and shared rules for the post-quantum cryptography (PQC) workstream. Source analysis: `docs/QUANTUM_READINESS_AUDIT.md` (2026-07-03). Every PQC task file references this one. This file itself contains no implementation work — do not move it to done/ until PQC-01 through PQC-14 are all done.

## Goal

Close the harvest-now-decrypt-later (HNDL) gap: all new message confidentiality must rest on hybrid X25519 + ML-KEM-768 (security holds if EITHER survives), and identity operations must be dual-signed Ed25519 + ML-DSA-65. Symmetric crypto (XChaCha20-Poly1305, Blake3, Argon2id) is already quantum-safe and MUST NOT be changed.

## Task graph

| Task | File | Depends on | Wave | Min model tier |
|------|------|-----------|------|----------------|
| PQC-01 ML-KEM dependency + smoke module | PQC_01_DEPENDENCY_MLKEM.md | — | 0 | Haiku |
| PQC-02 Envelope v2 wire format | PQC_02_ENVELOPE_V2.md | — | 0 | Sonnet |
| PQC-03 Identity v2 key bundle | PQC_03_IDENTITY_V2_KEYBUNDLE.md | PQC-01 | 0 | Sonnet |
| PQC-04 Suite negotiation + authenticated downgrade | PQC_04_SUITE_NEGOTIATION.md | PQC-02, PQC-03 | 1 | Sonnet |
| PQC-05 Hybrid KEM module | PQC_05_HYBRID_KEM_MODULE.md | PQC-01 | 1 | Sonnet + auditor |
| PQC-06 Hybrid session establishment | PQC_06_HYBRID_SESSION_INIT.md | PQC-02..05 | 2 | Sonnet + auditor |
| PQC-07 PQ ratchet | PQC_07_PQ_RATCHET.md | PQC-06 | 3 | Sonnet + auditor |
| PQC-08 Legacy path retirement | PQC_08_LEGACY_PATH_RETIREMENT.md | PQC-07 | 4 | Haiku |
| PQC-09 Hybrid onion routing | PQC_09_HYBRID_ONION.md | PQC-05, PQC-03 | 3 | Sonnet |
| PQC-10 ML-DSA identity signatures | PQC_10_MLDSA_IDENTITY_SIGNATURES.md | PQC-03 | 3 | Sonnet |
| PQC-11 Relay/invite hybrid auth | PQC_11_RELAY_INVITE_HYBRID_AUTH.md | PQC-10 | 4 | Haiku |
| PQC-12 Transport TLS PQ groups | PQC_12_TRANSPORT_TLS_PQ.md | — | 4 | Sonnet |
| PQC-13 Verification suite (Kani/proptest/matrix) | PQC_13_VERIFICATION_SUITE.md | PQC-05..09 | 5 | Sonnet |
| PQC-14 Docs + risk register closure | PQC_14_DOCS_AND_RISK_REGISTER.md | all | 5 | Haiku |

Tasks in the same wave with no dependency edge between them may run in parallel.

## Suite registry (canonical for this workstream)

| Suite ID | Meaning |
|----------|---------|
| 0x01 | CLASSICAL_X25519 — legacy: X25519 ECDH (+ optional DH ratchet), XChaCha20-Poly1305 |
| 0x02 | HYBRID_X25519_MLKEM768 — hybrid KEM (PQC-05) + PQ ratchet (PQC-07), XChaCha20-Poly1305 |

Reserved wire tag byte for Envelope v2 framing: 0x02 (see PQC-02).

## Fixed parameters (do not deviate without human sign-off)

- KEM: ML-KEM-768 (FIPS 203). Crate: `libcrux-ml-kem` (formally verified; used by Signal). Encapsulation key 1184 B, ciphertext 1088 B, shared secret 32 B.
- Signature (PQ): ML-DSA-65 (FIPS 204). Public key 1952 B, signature 3309 B.
- Combiner KDF: Blake3 `derive_key` with new, unique context strings per use (existing repo pattern, see `KDF_CONTEXT` in `core/src/crypto/encrypt.rs`).
- AEAD stays XChaCha20-Poly1305. Hashes stay Blake3/SHA-2. No cipher changes.

## Global rules (every PQC task)

1. **Hybrid, never pure.** A PQ secret is always combined with a classical secret. AND-verification for dual signatures: both must verify; never accept one.
2. **Never remove a decrypt or verify path for old data.** Old messages, backups, and invites must remain readable/verifiable. Only SENDING legacy formats gets restricted (PQC-08).
3. **Zeroize** all new secret material (`zeroize` crate; follow `RatchetKey` pattern in `core/src/crypto/ratchet.rs`).
4. **No `unsafe`.** If you believe you need it, stop and escalate.
5. **No emojis** anywhere (`.claude/rules/no-emojis.md`).
6. **Serialization discipline:** bincode is positional and NOT self-describing — never add/reorder fields of a struct that is bincode-serialized on the wire or in sled without an explicit format tag + fallback decode (pattern: `core/src/crypto/backup.rs` FORMAT_TAG). JSON-persisted state (e.g. ratchet sessions) tolerates new `#[serde(default)]` fields.
7. **Do not modify UniFFI-generated files.** Bridge exposure of new APIs is out of scope unless a task says otherwise.
8. **Escalate to the human operator** (stop work, write findings into the task file) if: a fixed parameter above cannot be used, a wire format must break compatibility, or a design in PQC-06/07 must deviate from its spec.

## Standard gates (referenced as "Standard gates" in every task's Definition of Done)

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core                       # plus task-specific scoped tests
cargo test --workspace --no-run
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
```

All gates must PASS and their output recorded in the task file before moving it to `HANDOFF/done/`. Commit per repo rules (`native: completed [task]` or `swarm: completed [task]`).

## Mandatory review

Changes under `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, `core/src/privacy/` require the adversarial review protocol (crypto-security-auditor) BEFORE the task is considered done, and release-gatekeeper before merge (see CLAUDE.md). Tasks marked "+ auditor" above are not optional.

## Acceptance criteria for the workstream as a whole

- [ ] Two v2 peers never negotiate below suite 0x02, and a stripped/downgraded offer is detected cryptographically (PQC-04).
- [ ] All new sessions between v2 peers derive every root/chain secret from BOTH an X25519 secret AND an ML-KEM-768 secret (PQC-05..07).
- [ ] Legacy static-ECDH sending is impossible between two v2 peers (PQC-08).
- [ ] Identity bundles, invites, and registration are dual-signed and dual-verified (PQC-10/11).
- [ ] Full cross-version test matrix green (PQC-13); docs and risk register updated (PQC-14).
