# TASK: Fix findings from PQC-09/10 adversarial security review

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO. Do NOT start until core message-delivery validation (current
session priority) is done and the operator says to return to PQC work.
Onion routing is not yet wired into any live path (`prepare_onion_message`
only builds classical hops), so none of this is exploitable in production
today - safe to defer.

Source: native `crypto-security-auditor` review of uncommitted PQC-09
(hybrid onion, `core/src/privacy/onion.rs`) + PQC-10 (ML-DSA-65,
`core/src/crypto/pq/mldsa.rs`, `core/src/identity/keys.rs`) changes,
2026-07-11. Verdict: NEEDS FIXES (no CRITICAL).

## HIGH - unauthenticated `is_destination` bit enables relay-triggered misdelivery/DoS

`core/src/privacy/onion.rs`: `HybridOnionLayer.is_destination` is a plaintext
sibling field to `payload`, not bound into the payload AEAD's AAD. A
malicious relay decrypts its own layer, flips `is_destination` on the
*next* hop's serialized struct before forwarding, and the next hop has no
cryptographic way to detect the tampering -> reliable message drop or
misinterpretation of plaintext/ciphertext as routing data, from a single
malicious relay, no crypto broken.

Fix: bind `is_destination` (and `hybrid_ct`) into the payload AEAD's AAD, or
better, encode the destination/relay signal as the first byte INSIDE the
AEAD-protected plaintext (mirroring how `ClassicalOnionLayer` already ties
this signal to whether `encrypted_routing_info` decrypts empty - tamper-evident
by construction). Add a test that a relay flipping this bit is detected
(AEAD failure), not silently misrouted.

## MEDIUM - unbounded bincode deserialization, pre-auth reachable

No `bincode::Options::with_limit(...)` anywhere in `core/src`. `IronCore::peel_onion_layer`
calls `bincode::deserialize(&onion_data)` on raw unauthenticated caller input
before any crypto check - attacker-controlled length prefixes can force
oversized allocation attempts. Compounded by nested deserialize calls added
this session (`ClassicalOnionLayer`/`HybridOnionLayer`/`PublicKeyBundle`
inside `layer_data`/routing info).

Fix: configure a bounded `bincode::Options` (realistic sizes per the diff's
own measurements: ~384B classical/hop, ~3744B all-hybrid/3-hop) for all
onion-related (de)serialization, plus an explicit max-length check on
`onion_data`/`envelope_data` before calling bincode at all.

## MEDIUM - `OnionEnvelope.version` tag unauthenticated

`core/src/privacy/onion.rs`: `version` (0x01/0x02) is not bound via AAD or
signed; a relay can lie about it. Confirmed this cannot silently downgrade/strip
PQ protection (struct shapes are incompatible, so mismatch = decode error or
the bincode issue above), but compounds the DoS surface.

Fix: authenticate `version` via AAD in the wrapping transport layer, or make
the layer format self-describing/length-checked before dispatch.

## MEDIUM - ML-DSA-65 key material never zeroized

`core/src/crypto/pq/mldsa.rs`: `MlDsa65KeyPair` derives only `Clone`, no
`Zeroize`/`ZeroizeOnDrop`, breaking this codebase's otherwise-consistent
zeroize discipline (Ed25519/X25519/ML-KEM seed/`IdentityKeysV2Raw`/V3Raw all
zeroize). Confirmed via Cargo.lock: none of ml-dsa's transitive deps
(module-lattice, hybrid-array, ctutils, cmov, pkcs8, spki, der, digest,
shake, keccak, sponge-cursor, const-oid, crypto-common, signature) pull in
`zeroize` either - no zeroization guarantee anywhere in this stack. Also:
`seed_bytes()`'s intermediate `ml_dsa::Seed` value is never explicitly
zeroized before return.

Fix: wrap key/seed bytes in a zeroizing container, or add a `[u8;32]` seed
field with `#[zeroize(drop)]` alongside the `SigningKey`. Explicitly zeroize
the intermediate `Seed` in `seed_bytes()`.

## MEDIUM - supply chain: pre-1.0 `ml-dsa` crate now load-bearing for identity trust

`core/Cargo.toml`: `ml-dsa = "0.1.1"`. Early-version FIPS 204 implementation,
now used in `verify_bundle`'s dual-signature trust decision. No suspicious
transitive deps found (`ctutils`/`cmov` presence is a good constant-time
signal). Not a defect - needs explicit human/security sign-off per this
repo's adversarial-review rule for new crypto deps, and version pinning +
advisory tracking given immaturity.

## LOW/MEDIUM - no Kani proofs for new crypto surface

`core/src/crypto/kani_proofs.rs` untouched; none of the 8 existing proofs
cover ML-DSA sign/verify length invariants, `HybridCiphertext` round-trip, or
`is_destination`/`version` decode-safety. Add harnesses for: ML-DSA-65
signature/public-key byte-length invariants (mirror the Ed25519 pattern),
and ideally a bounded-model proof that `peel_layer` never panics/over-allocates
on arbitrary `[u8]` input (would also catch the bincode finding above).

## LOW - placeholder `eprintln!` left in `verify_bundle`

`core/src/identity/keys.rs` ~line 335-347: `eprintln!(...)` with a literal
"(in real implementation, this would use tracing)" comment, inconsistent with
the rest of the codebase's `tracing::warn!` usage. Not exploitable (no key
material printed) but violates the no-placeholder repo convention and leaks
per-peer verification-failure detail to local logs at the wrong layer.

Fix: replace with `tracing::warn!`, remove the placeholder comment.

## Gate

Standard compile gate + `cargo test -p scmessenger-core --lib` +
`--test integration_pq_session --features test-utils` green. Re-run
adversarial review after fixes land (this file's fixes touch
`core/src/privacy/` and `core/src/crypto/` - mandatory per repo rules).
