# SCMessenger Quantum Readiness Audit

Status: Active
Last updated: 2026-07-03
Last verified: 2026-07-03 (audit of source at commit state on this date)
Scope: Post-quantum threat assessment of all cryptographic surfaces; migration plan

---

## Executive Verdict

**SCMessenger is NOT quantum-proof.** Every asymmetric cryptographic operation in the codebase — which gates all message confidentiality, all authentication, and all routing privacy — is built on Curve25519 (X25519 ECDH and Ed25519 signatures). Both are broken by Shor's algorithm on a cryptographically relevant quantum computer (CRQC).

The symmetric layer (XChaCha20-Poly1305, Blake3, SHA-2, Argon2id) is quantum-resistant and needs no changes.

**Grounding, as of July 2026:** No CRQC exists. The largest quantum computers field dozens of error-corrected logical qubits; breaking Curve25519 requires thousands of logical qubits (millions of physical qubits). Nothing sent today can be decrypted today. However:

1. Qubit resource estimates for breaking RSA/ECC have fallen sharply over 2024-2026, compressing consensus timelines.
2. **Harvest-now-decrypt-later (HNDL)** is the operative threat for a messenger: ciphertext recorded today is decrypted retroactively whenever a CRQC arrives. For an app whose value proposition is sovereign long-term-confidential communication, the exposure window starts at the first recorded packet, not at Q-day.
3. NIST IR 8547 deprecates quantum-vulnerable public-key crypto by 2030 and disallows it by 2035. UK NCSC guidance targets migration completion by 2035, with high-priority systems earlier. Signal (Triple Ratchet / SPQR) and Apple (PQ3) have already shipped hybrid post-quantum messaging.

A mesh messenger with relay custody is a *worse-than-average* HNDL target: relays hold ciphertext by design, and any relay operator (or anyone who can stand up a relay) can archive envelopes passively.

---

## Cryptographic Inventory

| # | Surface | Location | Primitives | Quantum status |
|---|---------|----------|------------|----------------|
| 1 | E2E message encryption (legacy path) | `core/src/crypto/encrypt.rs` (`encrypt_message`) | Ephemeral X25519 x recipient static X25519 (derived from Ed25519 identity) -> Blake3 KDF -> XChaCha20-Poly1305 | **BROKEN** (Shor) |
| 2 | E2E message encryption (ratcheted path) | `core/src/crypto/ratchet.rs` | Double Ratchet: X25519 DH steps + Blake3 chain KDF + XChaCha20-Poly1305 | **BROKEN** (DH steps) |
| 3 | Ratchet session establishment | `core/src/crypto/session_manager.rs` (`init_as_sender`) | X25519 ECDH from identity keys | **BROKEN** |
| 4 | Identity + signatures | `core/src/identity/`, `encrypt.rs` (`sign_envelope`/`verify_envelope`) | Ed25519 | **BROKEN** (forgeable post-CRQC) |
| 5 | Onion routing | `core/src/privacy/onion.rs` | Per-hop ephemeral X25519 ECDH + AEAD layers | **BROKEN** |
| 6 | Transport hop encryption | `core/src/transport/swarm.rs` | libp2p Noise XX (X25519), QUIC (TLS 1.3, classical groups), WSS (native-tls / rustls-webpki) | **BROKEN** (outer layer) |
| 7 | Identity backup encryption | `core/src/crypto/backup.rs` | Argon2id / Blake3 derive_key / legacy PBKDF2 -> XChaCha20-Poly1305 | **SAFE** (symmetric only) |
| 8 | KDFs and hashing | Blake3 `derive_key`, SHA-512 (key conversion), SHA-256 (PBKDF2), crc32 (integrity) | Hash-based | **SAFE** |
| 9 | AEAD everywhere | XChaCha20-Poly1305, 256-bit keys | Symmetric | **SAFE** (Grover reduces to ~128-bit effective; acceptable) |

No RSA, NIST P-curves, or secp256k1 anywhere in the workspace. No post-quantum primitives anywhere in the workspace (verified by grep for ML-KEM/Kyber/ML-DSA/Dilithium/SPHINCS/PQ across all `.rs` and `.toml`).

---

## Findings

### F1 (CRITICAL): Legacy envelope path retro-decrypts in bulk under HNDL

`encrypt_message` does static-ephemeral ECDH against the recipient's *identity-derived* X25519 key. The recipient's Ed25519 public key is the canonical, published, cross-platform identifier (`public_key_hex`). A CRQC running Shor on that single public key recovers the recipient's X25519 static secret, which decrypts **every legacy-path envelope ever recorded for that recipient, forever**. This path also lacks classical forward secrecy (one static key protects all history), so it is the single worst surface in the app: one public identifier + one quantum computation = full historical plaintext.

### F2 (CRITICAL): Ratcheted path also fully HNDL-exposed

The Double Ratchet's forward secrecy is classical-only. Every envelope carries its `ratchet_dh_public`; an adversary with a transcript can Shor each X25519 ratchet public key, replay the root/chain KDF cascade (Blake3, public algorithm), and decrypt the whole session. Symmetric-ratchet steps do not help because all entropy injections come from DH.

### F3 (HIGH): Single keypair for signing and encryption; no crypto agility

The Ed25519 identity key *is* the encryption key via the Ed25519-to-X25519 conversion (`ed25519_to_x25519_secret`). Consequences:

1. Identity and confidentiality share one secret; no independent rotation.
2. ML-KEM keys cannot be derived from Ed25519 keys — PQ migration structurally forces separate encapsulation keys, so the identity format must change regardless.
3. The `Envelope` wire struct (`core/src/message/`) has no version or cipher-suite field. There is currently **no mechanism to negotiate or deploy any new cryptography**. This blocks every other fix.

### F4 (HIGH): Onion routing privacy retro-fails

Each onion layer is ephemeral X25519 against a relay's public key (`privacy/onion.rs`). Recorded onion traffic yields full route reconstruction post-CRQC: retroactive metadata dragnet (who talked to whom, when) — precisely what the privacy layer exists to prevent.

### F5 (MEDIUM): Ed25519 signatures forgeable post-CRQC; seniority amplifies impact

Signature forgery is not retroactive (no HNDL), but identity keys are long-lived and the seniority system increases the value of old identities. Post-CRQC, an attacker who Shors a published identity key can impersonate it: forge envelopes past relay verification (`verify_envelope`), forge invites, hijack contact relationships. Relay/bootstrap registration signed with Ed25519 inherits the same exposure.

### F6 (MEDIUM): Transport hop encryption classical end-to-end

libp2p Noise is X25519-only upstream (no PQ support in rust-libp2p as of mid-2026); QUIC and WSS hops use classical TLS groups by default. Hop encryption is the outer layer — E2E content is what matters — but hop crypto also shields protocol chatter, peer exchange, and presence metadata from passive HNDL collection.

### What is already quantum-safe (no action)

- XChaCha20-Poly1305 with 256-bit keys (all paths)
- Blake3 KDF chains, SHA-2 usage
- Backup encryption (`backup.rs`): Argon2id/Blake3 + XChaCha20-Poly1305, no asymmetric material
- sled at-rest storage model (symmetric/local)

---

## Migration Plan

Hybrid (classical + PQ) is the industry consensus — Signal SPQR, Apple PQ3, Chrome/rustls X25519MLKEM768 all pair ML-KEM with X25519 so that security holds if *either* assumption survives. Do not go pure-PQ; lattice cryptanalysis is young.

### Phase 0 — Prerequisites (unblocks everything; no new crypto)

1. **Versioned envelope + suite negotiation.** Add `suite: u8` (or version) to `Envelope`/`SignedEnvelope`, advertise supported suites in contact exchange/identify, and make downgrade *cryptographically authenticated* (transcript-bind the offered suite list, as Signal does) so an attacker cannot strip PQ.
2. **Decouple encryption keys from identity.** Introduce dedicated per-identity key bundles (X25519 today, +ML-KEM next), signed by the Ed25519 identity key. Retires the Ed25519-to-X25519 conversion. Identity format v2 must carry: Ed25519 (continuity), X25519 encaps key, ML-KEM-768 encaps key, later ML-DSA verify key — cross-signed by the legacy key to preserve seniority/trust continuity.

### Phase 1 — Confidentiality (kills HNDL; highest priority)

3. **Hybrid initial key agreement**: X25519 + ML-KEM-768, PQXDH-style — KDF over the concatenation of both shared secrets (or adopt X-Wing as the combiner). Crate: `libcrux-ml-kem` (formally verified, portable + AVX2/NEON, what Signal uses) or RustCrypto `ml-kem`. Both are pure Rust: Android (cargo-ndk) and wasm32 targets should compile cleanly — verify wasm32-unknown-unknown in CI.
4. **Hybrid ratchet**: add an ML-KEM ratchet alongside the DH ratchet (Signal Triple Ratchet / SPQR model). ML-KEM-768 sizes: encapsulation key 1184 B, ciphertext 1088 B. For BLE-constrained mesh links, adopt SPQR's chunking approach (encapsulation keys erasure-coded across messages) rather than inflating every envelope; `signalapp/SparsePostQuantumRatchet` is the reference implementation (check AGPL license compatibility before vendoring — use as design reference otherwise).
5. **Retire the legacy non-ratchet path** (F1). Make ratchet+hybrid mandatory for new sessions; keep legacy decrypt-only for old stored messages. This is worth doing even independently of PQ — the path has no forward secrecy today.
6. **Hybrid onion hops**: each layer encapsulates to the relay's hybrid bundle (X25519 + ML-KEM-768 ct per hop). Adds ~1.1 KB per hop; acceptable for relay-grade links, budget for BLE paths.

### Phase 2 — Authenticity

7. **Hybrid signatures on identity operations**: Ed25519 + ML-DSA-65 (FIPS 204) dual-sign registration, invites, key bundles, relay registration. ML-DSA-65: ~1.9 KB public key, ~3.3 KB signature — fine for infrequent identity operations.
8. **Per-envelope relay-auth signatures** can stay Ed25519 short-term: forgery requires a real-time CRQC (no HNDL), so this lags Phase 1 safely. Plan hybrid envelope signatures once sizes are workable for BLE (or gate by transport).

### Phase 3 — Transport and hygiene

9. **QUIC/WSS hops**: rustls >= 0.23.27 ships X25519MLKEM768 (TLS codepoint 0x11EC); enable `prefer-post-quantum` where both ends are project-controlled (bootstrap/relay infrastructure). Track upstream rust-libp2p for PQ Noise; app-layer E2E (Phase 1) already covers content, so this is metadata hardening.
10. **Verification requirements** (per `.claude/rules/security.md`): extend the Kani proof set and `proptest_harness.rs` to hybrid KDF combination (both-secrets-contribute property), run the repo's mandatory adversarial review on every Phase 1-2 change, and property-test cross-version envelope compatibility.

### Priority order if resources are constrained

Phase 0 -> item 3 (hybrid session init) -> item 4 (hybrid ratchet) -> item 5 (legacy retirement) -> item 6 (onion) -> Phase 2 -> Phase 3. Items 1-5 eliminate the overwhelming majority of quantum risk because they close HNDL on message content.

---

## Timeline Recommendation

For "secure for years to come" claims to be truthful, hybrid confidentiality (Phase 0 + Phase 1) belongs in the next protocol-breaking release line, not after 1.0 hardening: every day of classical-only operation permanently exposes that day's traffic to future decryption. Signatures (Phase 2) and transport (Phase 3) can follow on the NIST 2030 curve.

Escalation note (per CLAUDE.md): this plan changes identity format, wire format, and protocol version — architectural and security-posture decisions that require human sign-off before implementation.

---

## Sources

- NIST IR 8547 (transition to PQC; 2030 deprecate / 2035 disallow): https://csrc.nist.gov/pubs/ir/8547/ipd
- NIST PQC standards (FIPS 203 ML-KEM, 204 ML-DSA, 205 SLH-DSA): https://csrc.nist.gov/projects/post-quantum-cryptography
- UK NCSC PQC migration timelines: https://www.ncsc.gov.uk/guidance/pqc-migration-timelines
- Signal Triple Ratchet / SPQR: https://signal.org/blog/spqr/ and https://github.com/signalapp/SparsePostQuantumRatchet
- SPQR analysis: https://pqshield.com/diving-into-signals-new-pq-protocol/
- CRQC status 2026: https://quantumzeitgeist.com/cryptographically-relevant-quantum-computer/
- rustls ML-KEM hybrid support: https://docs.rs/rustls-post-quantum/latest/rustls_post_quantum/
- libcrux-ml-kem: https://crates.io/crates/libcrux-ml-kem
- RustCrypto ml-kem: https://crates.io/crates/ml-kem
- rust-libp2p Noise (X25519-only as of audit date): https://docs.rs/libp2p/latest/libp2p/noise/index.html

---

## Remediation Status (2026-07-29 Snapshot)

| Finding | Severity | Landed Remediation Tasks | Status |
|---|---|---|---|
| F1 Legacy envelope path HNDL | CRITICAL | `PQC_08_LEGACY_PATH_RETIREMENT.md` (gated legacy sends, require_pq enforcement) | **CLOSED** |
| F2 Ratcheted path HNDL | CRITICAL | `PQC_05_HYBRID_KEM_MODULE.md`, `PQC_06_HYBRID_SESSION_INIT.md`, `PQC_07_PQ_RATCHET.md` (root key mixing with ML-KEM-768 shared secret) | **CLOSED** |
| F3 Single keypair / no agility | HIGH | `PQC_02_ENVELOPE_V2.md`, `PQC_03_IDENTITY_V2_KEYBUNDLE.md`, `PQC_04_SUITE_NEGOTIATION.md` | **CLOSED** |
| F4 Onion routing retro-privacy | HIGH | `PQC_09_HYBRID_ONION.md` (spec and modules landed; parked until onion layer activation) | **PARKED** |
| F5 Ed25519 signatures forgeable | MEDIUM | `PQC_10_MLDSA_IDENTITY_SIGNATURES.md`, `PQC_11_RELAY_INVITE_HYBRID_AUTH.md` | **CLOSED** |
| F6 Transport hop encryption classical | MEDIUM | `PQC_12_TRANSPORT_TLS_PQ.md` (rustls PQ enabled; libp2p Noise remains classical) | **ACCEPTED** |

Residual Exposures:
1. **Relay Transit Signatures**: Envelope transit hop signatures use Ed25519 (64B) to preserve low overhead across mesh relay hops; payload authenticity and confidentiality are protected by E2E dual signatures (Ed25519 + ML-DSA-65) and hybrid ratchet encryption.
2. **libp2p Noise Transport**: Peer connection establishment uses classical Noise XX (X25519+ChaCha20Poly1305) until upstream libp2p releases standard post-quantum Noise extensions.

