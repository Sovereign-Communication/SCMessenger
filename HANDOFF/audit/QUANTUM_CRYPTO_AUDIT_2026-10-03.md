# Quantum-Resistance Crypto Audit — SCMessenger

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-10-03
Scope: read-only audit. No fixes applied.
Branch observed: `preserve/shared-checkout-20261003` @ `e4050f84`
Revision: **2** (2026-10-03) — findings F1-F4 re-verified by call-path tracing and
corrected. See "Revision note" at the end for what changed and why.

---

## BUILD STATE — READ FIRST

**`scmessenger-core` does not compile on the snapshot branch this audit was run
against** (`preserve/shared-checkout-20261003` @ `e4050f84`).

```
error[E0603]: function `canonical_ledger_peer_id` is private
  --> core\src\store\outbox.rs:196:33
note: the function `canonical_ledger_peer_id` is defined here
  --> core\src\store\ledger_entry.rs:489:1
```

`core/src/store/outbox.rs:196` calls a function that `core/src/store/ledger_entry.rs:489`
declares without `pub` on that branch. The working tree was clean, so it is committed
state, not local WIP. `git diff --stat main..HEAD -- core/src/store/ledger_entry.rs`
reports the snapshot branch is **294 lines short of main**.

**This break is specific to the snapshot branch. It does not exist on `main`:**
`git show origin/main:core/src/store/ledger_entry.rs` declares the function
`pub(crate)` at line 498. Anyone reading this report against `main` should not go
looking for a compile failure there — the remedy is to run the audit and the task
list from a tree that builds.

Two consequences for this report:

1. **No finding below is runtime-verified.** Every claim is settled by call-path
   tracing and grep, not by execution. A scratch test was written to settle F2 and
   F3 empirically and deleted; it could not run. The static traces are short and
   unambiguous, but a build that compiles should re-run the PQ test suites before
   anyone relies on this document as more than a code reading.
2. **The PQ integration tests cannot currently run**, so there is no green suite
   contradicting or corroborating anything here.

This build break is unrelated to the cryptography and is not developed further
here, but it should be tracked separately **against the snapshot branch only**.

---

## Method

Every claim is backed by a command executed in this session. Where a tool-reported
`referencedBy` index disagreed with the code, the grep was treated as authoritative
(two index claims were stale — see "Corrections" below).

Searched: `core/src/**`, `cli/src/**`, `wasm/src/**`, `core/tests/**`,
`iOS/**/*.swift`, `android/**/*.kt`, `mobile/`, `desktop_bridge/`. Excluded
`target/`, `tmp/`, `vendor/`.

---

## 1. Primitive inventory

| Primitive | Impl / crate | Where referenced | Key size |
|---|---|---|---|
| ML-KEM-768 (KEM) | `libcrux-ml-kem` 0.0.10 | `crypto/pq/mod.rs`; `crypto/pq/hybrid.rs`; `crypto/ratchet.rs:463,552,640,714,1694`; `privacy/onion.rs:175,273,515`; `identity/keys.rs:506` | pk 1184 B, sk 2400 B, ct 1088 B, ss 32 B |
| ML-DSA-65 (signature) | `ml-dsa` 0.1.1 | `crypto/pq/mldsa.rs`; `identity/keys.rs:249,290,321,554,605`; `relay/invite.rs:362` | vk 1952 B, sk-seed 32 B, sig 3309 B |
| Ed25519 (signature) | `ed25519-dalek` 2.1 | `crypto/encrypt.rs`; `identity/keys.rs`; `drift/envelope.rs:720,728`; `message/codec.rs:449,515,619`; libp2p PeerId (`identity/keys.rs:42-45,455,467-479`) | 32 B |
| X25519 (ECDH) | `x25519-dalek` 2.0 | `crypto/encrypt.rs:130-138`; `crypto/ratchet.rs` (DH ratchet); `crypto/pq/hybrid.rs:67`; `crypto/session_manager.rs`; `iron_core.rs:1866-1871` | 32 B |
| Curve25519 point conversion | `curve25519-dalek` 4.1 | `crypto/encrypt.rs:64,101` | — |
| libp2p Noise XX | `libp2p` 0.57 `noise` | `transport/swarm.rs:3418,3421,3423,3472,3482,8105,8113` | X25519 + Ed25519 static |
| TLS 1.3 (QUIC) | `quinn` 0.11 / `rustls` | `relay/client.rs:428,436` — `try_with_platform_verifier()`, no client cert | X25519 (default group) |
| XChaCha20-Poly1305 (AEAD) | `chacha20poly1305` 0.10 | `crypto/encrypt.rs:149,217`; `crypto/ratchet.rs:803,846,879,980`; `crypto/backup.rs:110,189`; `privacy/onion.rs:7` | key 32 B, nonce 24 B, tag 16 B |
| BLAKE3 (hash + KDF) | `blake3` 1.5 | `crypto/encrypt.rs:107`; `crypto/ratchet.rs:1273-1312`; `crypto/pq/hybrid.rs:86,127`; `crypto/negotiation.rs:62`; `crypto/backup.rs:93`; `iron_core.rs:1872`; `dspy/signatures.rs:134` | 32 B out |
| SHA-512 | `sha2` | `crypto/encrypt.rs:40` | — |
| SHA-256 / PBKDF2-HMAC-SHA256 | `sha2`, `pbkdf2` 0.12.2 | `crypto/backup.rs:24,29,102` — legacy decrypt only | 600,000 iters |
| Argon2id | `argon2` 0.5 | `crypto/backup.rs:66-79` — user-facing exports, 19 MiB / t=2 / p=1 | 32 B out |
| CSPRNG | `rand` 0.8 `OsRng`, `getrandom` 0.3 | all key/nonce/salt generation | — |

**Not present anywhere:** AES (0 occurrences), RSA, ECDSA, 3DES, RC4, MD5, custom
cipher constructions.

RNG audit is clean: every key, nonce, ephemeral secret, ML-KEM seed and encapsulation
randomness comes from `OsRng`. Two `rand::random()` calls exist
(`crypto/backup.rs:176`, `:429`); `rand::random()` uses ThreadRng, a CSPRNG seeded
from the OS — **not a vulnerability**, only a style inconsistency.

**Quantum-resistant:** ML-KEM-768, ML-DSA-65, XChaCha20-Poly1305 (Grover leaves
~128-bit effective), BLAKE3, SHA-2, Argon2id, PBKDF2.
**Quantum-vulnerable (Shor-broken):** Ed25519, X25519, libp2p Noise XX, QUIC/TLS 1.3
key exchange. Detail in F1.

---

## 2. Findings, ranked by corrected severity

Severity order is live-exploitable-today before forward-looking. The original
revision ranked F1 and F2 as the two CRITICALs; tracing changed both the mechanism
of F2 and the ranking, so the CRITICALs are now F2 and F4.

---

### F2 — CRITICAL — The hybrid ML-KEM-768 / ML-DSA-65 path is unreachable from the production send path

> **Corrected in revision 2.** Revision 1 attributed this to `require_pq = false`.
> That was wrong about the mechanism. The real defect is that peer key bundles are
> never persisted, so the hybrid branch is never entered.

**The finding:** `encrypt_with_ratchet_fallback` selects its hybrid PQ path only
when **both** `our_bundle` and `recipient_bundle` are `Some`. In production
`recipient_bundle` is **always `None`**, because nothing ever writes a peer bundle
into the contact store.

**Trace from the real entry point:**

1. `IronCore::prepare_message_internal` builds `our_bundle` locally
   (`iron_core.rs:992`, `crate::identity::sign_bundle(keys).ok()` — always `Some`).
2. It fetches `recipient_bundle` via
   `contact_manager.get_contact_bundle(recipient_id)` (`iron_core.rs:994-998`).
3. `get_contact_bundle` (`store/contacts.rs:760`) reads a **separate sled
   keyspace**, `contact_bundle_key` (`store/contacts.rs:85`). It is not a field of
   `Contact` — the struct (`store/contacts.rs:25-38`) carries only
   `peer_id / nickname / local_nickname / public_key / added_at / last_seen / notes /
   last_known_device_id`. No bundle.
4. That keyspace is written by exactly one function, `save_contact_bundle`
   (`store/contacts.rs:746`).
5. **`save_contact_bundle` has zero production callers.** Every call site outside its
   own definition and its own unit test is in a test file:
   - `store/contacts.rs:1507` (the function's own `#[cfg(test)]`)
   - `core/tests/integration_e00_ratchet_wiring.rs:47,50`
   - `core/tests/test_v1_bincode_downgrade_forgery.rs:74`
   - `core/tests/test_v2_hybrid_envelope_forgery_is_rejected.rs:194,434`

   Verified across Rust, Kotlin and Swift (`grep -rn "save_contact_bundle" --include=*.rs --include=*.kt --include=*.swift`).
6. So `recipient_bundle` is `None` at `encrypt_with_ratchet_fallback`
   (`encrypt.rs:528`, called from `iron_core.rs:1003`).

**Where it lands — two different classical paths:**

- **No existing session:** `should_use_ratcheted_encryption` (`encrypt.rs:456`) takes
  the `None` arm (`encrypt.rs:489-501`): `session_exists` false, `require_pq` false,
  so it returns `Ok(false)` → the `false` arm at `encrypt.rs:580` sends **legacy static ECDH**
  (`encrypt_message`, per-message ephemeral X25519 + XChaCha20-Poly1305). No PQ, and
  no forward secrecy.
- **Existing session:** returns `Ok(true)`, but the inner fork at `encrypt.rs:549`
  requires `(Some(our_b), Some(their_b))`. `their_b` is `None`, so it falls to
  `get_or_create_session` → `RatchetSession::init_as_sender`
  (`session_manager.rs:142`) — the **classical X25519 suite-0x01 ratchet**. No PQ.

Either way ML-KEM-768 is never invoked on the send path. The PQ code is real,
well-built and unit-tested; it simply has no production caller.

**Why the passing tests do not contradict this:** the PQ integration suites
(`core/tests/integration_pq_verification_suite.rs`,
`integration_pq_session.rs`, `test_v2_hybrid_envelope_forgery_is_rejected.rs`) call
the crypto layer directly and **hand-seed the contact store** with
`save_contact_bundle` before exercising the hybrid path — precisely the step
production never performs. They prove the primitives work. They prove nothing about
reachability. (They also cannot run at present; see BUILD STATE.)

**Minor, separate note — `require_pq = false`.** The literal at
`iron_core.rs:1012` is real, but it is **not** the operative gate and revision 1
over-weighted it. For any peer holding a current bundle,
`should_use_ratcheted_encryption` returns `Ok(true)` at `encrypt.rs:468-476`
*before* `require_pq` is ever consulted. The flag only decides whether a
bundle-less or pre-suite peer is refused or allowed to proceed. It is a
defense-in-depth / fail-closed policy gap worth closing, not a live downgrade in
its own right, and fixing it alone would change nothing while F2 stands.

**Migration:** wire bundle propagation — exchange and verify `PublicKeyBundle` on
contact add / peer discovery (the machinery already exists: `sign_bundle`,
`verify_bundle`, `save_contact_bundle`), then flip `require_pq` to `true` once
bundles are reliably present. Treat this as the top priority: until it lands, the
project has no post-quantum protection in production regardless of the other
findings.

---

### F4 — CRITICAL — Identity secrets are persisted to sled in the clear

> **Corrected in revision 2.** The ratchet-session half of this finding is
> unreachable in production and has been removed from the claim.

**Live path:**

1. `IronCore::with_storage` → `IdentityManager::with_backend(backend)`
   (`iron_core.rs:~430`).
2. That constructs `IdentityStore::persistent(backend)` (`identity/mod.rs:37-39`).
3. `initialize` and `import_key_bytes` call `store.save_keys(&keys)`
   (`identity/mod.rs:96`, `:167`).
4. `save_keys` (`identity/store.rs:53-63`) does
   `db.put(IDENTITY_KEY, &keys.to_bytes())` — raw bytes. The `zeroize()` applied
   afterwards touches the local `Vec` only, not the bytes already written.
5. `IdentityKeys::to_bytes()` (`identity/keys.rs:314-345`) serialises, unencrypted:
   the Ed25519 signing key, the X25519 secret, the ML-KEM 64-byte seed (from which
   the full ML-KEM-768 private key is derivable) and the ML-DSA-65 secret seed.
6. `core/src/store/backend.rs` contains no encryption code
   (`grep -n "encrypt|Cipher|Aes|Secret"` returns nothing).

Anyone with read access to the sled directory holds the complete identity.

**Removed claim — ratchet sessions are not at risk this way.** Revision 1 stated
that root keys and chain keys sit in `ratchet_sessions_v1` in the clear. They do
not, in production, because **nothing ever writes that key**:
`RatchetSessionManager::with_backend` is called only from `#[cfg(test)]` modules and
test files (`core/tests/integration_backup.rs`, `integration_ratchet_persistence.rs`).
All three `IronCore` constructors use `RatchetSessionManager::new()`
(`iron_core.rs:385`, `:561`, `:692`), which sets `backend: None`, so
`RatchetSessionManager::save` (`session_manager.rs:40-41`) is a no-op.

Consequently the doc comment at `session_manager.rs:307` — *"When loading, the caller
must ensure the storage is encrypted at rest"* — is currently unviolated, but only
vacuously. It is a live hazard **if and only if** someone later wires a backend to
the session manager, which is exactly the sort of change this comment exists to
constrain. Flagging it as a trap for that future change, not as a current
vulnerability.

**Migration:** wrap the sled backend in an XChaCha20-Poly1305 envelope keyed by a
device-bound key from the platform keystore (Keystore / Secure Enclave). The
`derive_key_blake3` device-bound pattern in `backup.rs` is an existing precedent.

---

### F3 — HIGH — Backup restore silently degrades a hybrid session to classical

> **Corrected in revision 2.** The mechanism holds; the trigger does not.
> Revision 1 said "after any restart that reloads sessions from disk". There is no
> such path. The sole trigger is **backup restore**.

**Mechanism (confirmed):** `SerializableRatchetSession::into_session` passes
`negotiated_suite` through to `reconstruct` but sets every PQ field to `None`:
`session_manager.rs:539-545` (`None, // pq_our_keypair` … `None, // pq_last_mixed_fp`).
`is_pq_hybrid()` is computed solely from `negotiated_suite` (`ratchet.rs:781`), so a
restored session still reports itself hybrid while holding no ML-KEM secret.

**Downstream is silent, not fatal:**
- `perform_pq_ratchet_step` (`ratchet.rs:1129`) passes the `is_pq_hybrid()` gate,
  then fails on `pq_their_encaps_key` (None).
- The caller discards the error: `if let Ok((ct, encaps_key)) =
  session.perform_pq_ratchet_step()` (`encrypt.rs:384`). No PQ fields are attached.
- The receiver-side anti-stripping guard does not trip either:
  `validate_pq_fields_present` (`ratchet.rs:1240`) requires
  `self.pq_their_encaps_key.is_some()`, which is false, so it returns `Ok`.

The session keeps working, silently without PQ refresh. Confirmed.

**Trigger (corrected):** production never persists sessions, so a plain restart
loses them entirely and re-establishes fresh hybrid sessions — which is fine. The
only production hydration path is `import_identity_backup` →
`deserialize_sessions_strict` (`iron_core.rs:2030-2033`), which reaches
`into_session()`. So this is a **backup-restore** defect with a narrow trigger.

**Migration:** persist the ML-KEM keypair (available as a 64-byte seed via
`pq::from_seed`) plus `pq_their_encaps_key` and `pq_last_mixed_fp` in
`SerializableRatchetSession`, or force re-establishment on restore. Note this
becomes materially worse if F4's sibling concern is ever addressed by giving the
session manager a real backend, since restore would then also inherit plaintext
root keys.

---

### F1 — HIGH — PQ coverage stops at the Ed25519 identity boundary

> **Scoped in revision 2.** This is a **forward-looking CRQC / harvest-now-decrypt-
> later risk**, not a live classical defect. Nothing is broken today. It is ranked
> here rather than as CRITICAL for that reason, but it is the largest migration in
> the report and it gates the value of fixing F2.

**The finding:** the message layer is genuinely hybrid and well built; the identity
and transport layers it depends on are entirely classical, and identity is the
anchor.

`RatchetSession::init_as_sender_hybrid` builds the session root key from three terms
(`ratchet.rs:478-484`):

```
root_key_0 = derive_key("iron-core session-root v3 2026-08",
                        ss_hybrid || dh_static || transcript_hash)
```

- `ss_hybrid` — ML-KEM-768 + X25519, genuine PQ (`pq/hybrid.rs:81-86`).
- `dh_static` — X25519 static-static, classical.
- `transcript_hash` — blake3 over **the two Ed25519 public keys**
  (`negotiation.rs:52-64`).

A CRQC that breaks Ed25519 lets an attacker present a bundle containing *their own*
Ed25519 key, *their own* X25519 key and *their own* ML-KEM encapsulation key. They
know both DH secrets and can decapsulate the ML-KEM themselves. Full session
compromise.

**ML-DSA-65 does not mitigate this.** `verify_bundle` (`identity/keys.rs:605-620`)
requires **both** signatures to verify. That is not protection against an Ed25519
break: a forger simply supplies a valid ML-DSA keypair of their own, and the
"both must pass" rule passes trivially. Revision 1 stated the dual-signature is "the
right pattern"; that was too generous — as an *authentication* control it is sound
against a classical attacker, and useless against a quantum one.

**Transport, also classical:** libp2p Noise XX authenticates with the Ed25519
identity key and derives transport keys from X25519 (`transport/swarm.rs:3472`);
QUIC to relay uses TLS 1.3 with a platform verifier and no client certificate
(`relay/client.rs:436`).

**Migration:** make ML-DSA-65 primary rather than corroborating — (1) accept a
bundle if **either** signature verifies, logging the other; (2) fold ML-DSA into
`negotiate_suite`'s transcript material so the session root binds a PQ-verifiable
value; (3) treat the Noise/QUIC layer as a confidentiality-only tunnel and never as
the authenticity anchor, since libp2p has no PQ Noise suite. Requires an operator
decision (AGENTS.md rule 9: architecture direction).

**Coupling warning:** fixing F2 without F1 would advertise post-quantum protection
that does not hold against a CRQC. Sequence them together.

---

### F5 — MEDIUM — Ed25519 signing key reused as an X25519 ECDH key

`crypto/encrypt.rs:40-48` (`ed25519_to_x25519_secret`) feeds the legacy per-message
path, the classical ratchet (`ratchet.rs:349,404`) and Wi-Fi Aware
(`iron_core.rs:1866`). Suite 0x03 deliberately avoids this, with an explicit comment
at `ratchet.rs:466-470` ("avoid cross-protocol key reuse"); the classical and
Wi-Fi Aware paths were not updated.

**Migration:** point `iron_core.rs:1866` and the classical `init_as_sender` /
`init_as_receiver` initial-DH at `keys.x25519_encryption_secret`, which already
exists and is already published in the bundle. Retire
`ed25519_to_x25519_secret` once the legacy path is gone.

### F6 — MEDIUM — `crypto` re-exports the hashing primitive from the LLM layer

`crypto/mod.rs:24` — `pub use crate::dspy::signatures::{blake3_hash, ...}`, so the
crypto module's public API depends on `dspy`, the DSPy prompt-schema layer.
Inverted dependency. Reaches production as FFI: `IronCore::dspy_blake3_hash` /
`dspy_get_signature`, consumed by `cli/src/server.rs` and `wasm/src/lib.rs`.

`dspy/signatures.rs:107-131` — `GOLDEN_XCHACHA20_ENCRYPTION` is a **string literal
containing insecure example code**: raw ChaCha20 keystream plus a Poly1305 tag
computed over the ciphertext *with the same key*, the textbook misuse pattern (and
it would not compile — `ChaCha20::new` takes a 12-byte nonce). It is prompt
material for an AI coding agent, i.e. a live path for an agent to copy an
unauthenticated cipher into the codebase. `dspy/signatures.rs:88-104`
(`GOLDEN_CURVE25519_KEYGEN`) mixes `ring` and `x25519_dalek` and returns the secret
key alongside the public key.

**Migration:** move `blake3_hash` into `crypto/`, have `dspy` import from `crypto`;
replace both golden examples with correct AEAD usage or delete them.

### F7 — MEDIUM — Ed25519 signing duplicated in three non-crypto layers

`drift/envelope.rs:720,728` — its own `signing_hash()` canonicalization (blake3,
`:666-717`) plus Ed25519 sign/verify, entirely outside `crypto/`.
`message/codec.rs:449,515,619` — Ed25519 signing in the serialization layer.
`iron_core.rs:1872` — a `blake3::derive_key` KDF in the facade.

**Migration:** route drift envelope signing through `crypto::`; three
canonicalizations are a maintenance hazard for any future PQ signature rollout.

### F8 — LOW — 128-bit fingerprint of an ML-KEM shared secret retained in memory

`ratchet.rs:67-72` — `pq_ss_fingerprint` = first 16 bytes of BLAKE3(ss), stored in
`pq_last_mixed_fp` for deduplication. Not transmitted and not persisted
(`session_manager.rs:545` passes `None`), so exposure is memory-local. Still, a
truncated hash of a fresh secret is derived material that should not be retained;
compare a full 32-byte value or a session-scoped non-reversible tag instead.

### F9 — LOW — Test and fixture key material: no real secrets found

Hardcoded hex in tests is all obviously synthetic: `[1u8;32]`, `1234...`, `[3u8;32]`
(`store/contacts.rs:1533+`, `drift/envelope.rs:760`, etc.).
`self_certifying_keypair(seed_tag)` (`identity/keys.rs:38-45`) deterministically
derives a keypair from a tag such as `b"wp1-add-canonical"`. All 11 call sites are in
`#[cfg(test)]` modules (`store/contacts.rs:1022`, `contacts_bridge.rs:482`).
Verified not reachable from production.
`scmessenger-farm-sim-key-v2.pem` exists in the working tree but is **gitignored**
(`.gitignore:252`), and `git ls-files` returns no `.pem`/`.key` files.
`crypto/backup.rs:412` is an Argon2id known-answer vector, not a key.

**No hardcoded production credentials identified.**

---

## 3. What is genuinely solid

- Hybrid KEM combiner (`pq/hybrid.rs:81-86`) correctly binds *both* shared secrets,
  both public keys and the ciphertext.
- Domain separation is disciplined: nine distinct BLAKE3 context strings
  (`encrypt.rs:31`, `ratchet.rs:29-31`, `hybrid.rs:86`, `negotiation.rs:62`,
  `backup.rs:93`, `iron_core.rs:1872`). No collisions.
- Suite 0x02/0x03 freeze discipline (`negotiation.rs:16-27`) is a careful response
  to a real redefinition bug; `keys.rs:515-538` documents why 0x02 is no longer
  advertised. Worth keeping as precedent for the F1 migration.
- X25519 all-zero (non-contributory) shared secrets are rejected on both hybrid
  sides (`hybrid.rs:74,120`).
- Zeroizing wrappers on key material throughout (`RatchetKey`, `MlKem768KeyPair`,
  `MlDsa65PrivateKey`), with a correct known-answer test replacing a flaky timing
  assertion (`backup.rs:399-417`).
- Mobile surfaces correctly delegate all cryptography to the Rust core; the only
  platform-side crypto is SHA-256 for BLE payload fingerprints.

---

## 4. Corrections made during verification

**Stale index data, discarded in favour of greps (AGENTS.md rule 13):**
- `crypto/pq/mldsa.rs` appeared to have zero callers; it is in fact wired into
  `identity/keys.rs` (keybundle dual-signing) and `relay/invite.rs` (invite tokens).
- `crypto/pq/hybrid.rs` appeared reachable only from `privacy/onion.rs`; it is also
  the ratchet's session-establishment KEM (`ratchet.rs:463,552,640,714`).
- A suspected duplicate `blake3_hash` definition in `dspy/signatures.rs` was a false
  positive: the second occurrence is inside a `const` string literal (`:147`) and is
  not compiled.

**Findings corrected by call-path tracing in revision 2:**

| Finding | Verdict | Correction |
|---|---|---|
| F2 | **wrong** | Mechanism was `require_pq=false`. Actually: `save_contact_bundle` has no production caller, so `recipient_bundle` is always `None` and the hybrid branch is never entered. `require_pq=false` demoted to a minor defense-in-depth note. |
| F3 | overstated | Mechanism holds; trigger was "restart reloads from disk". Sessions are never persisted in production — the sole trigger is `import_identity_backup`. |
| F4 | overstated | Identity-at-rest half is live. Ratchet-session half removed: `RatchetSessionManager::with_backend` is test-only, so `ratchet_sessions_v1` is never written. |
| F1 | rescope | Confirmed but forward-looking (CRQC), not live; ranked HIGH. Added that ML-DSA's "both must verify" rule provides no protection against an Ed25519 break, and that F2 must not be fixed without F1. |
| — | new | `scmessenger-core` does not compile on the audited snapshot branch (`preserve/shared-checkout-20261003`); no finding is runtime-verified. `main` is unaffected — the function is `pub(crate)` there. |

Ranking changed accordingly: revision 1 led with F1 and F2 as the two CRITICALs;
revision 2 leads with F2 and F4, because "never wired" is a live defect affecting
every message while "no at-rest encryption" is a live defect affecting anyone with
disk access, and the CRQC-bound F1 is neither.

---

## 5. Recommended order

1. **F2** — wire bundle propagation. Until this lands, there is no PQ in production
   at all. Note the F1 coupling: sequence F1 alongside it, or the result is PQ
   assurance that a CRQC invalidates.
2. **F4** — encrypt the sled backend; caps what any other fix is worth against a
   disk-level attacker.
3. **F3** — persist PQ state on restore; worsens if the session manager ever gains a
   real backend.
4. **F5, F6, F7** — layer hygiene, low risk, do alongside the above.
5. **F1** — the long migration: ML-DSA-primary identity, PQ-bound transcript, then
   transport. Operator decision required (rule 9).

Separately: the compile break described under BUILD STATE should be tracked on its
own, against the snapshot branch it was observed on.