# CRYPTO-T4 -- Encrypt the sled backend at rest

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- awaiting implementation
Severity: CRITICAL (finding F4 of `HANDOFF/audit/QUANTUM_CRYPTO_AUDIT_2026-10-03.md`)
Gate: touches `core/src/store/` and `core/src/identity/` -- Rule-8 review applies
if the diff reaches `core/src/crypto/`.
Raised: 2026-10-04, extracted from T4 of
`HANDOFF/audit/QUANTUM_CRYPTO_REMEDIATION_TASKS.md`, which existed only as prose
and therefore had no queue position.

**This is the more urgent of the two CRITICAL crypto findings.** T2 is a
forward-looking harvest-now-decrypt-later concern. T4 is exploitable **today, by
anyone with read access to the node's data directory, with no quantum computer
involved at all.**

## Mechanism

Identity secrets are persisted to sled in the clear. `save_keys` performs
`db.put(IDENTITY_KEY, &keys.to_bytes())` with raw bytes; the `zeroize()` applied
afterwards touches only the local `Vec`. The complete identity -- Ed25519
signing key, X25519 encryption secret, ML-KEM and ML-DSA material -- is
recoverable from a disk image, a backup, or a copied data directory.

**Files/lines:** `core/src/iron_core.rs` (~430) ->
`core/src/identity/mod.rs:37-39` -> `core/src/identity/mod.rs:96`, `:167` ->
`core/src/identity/store.rs:53-63` -> `core/src/identity/keys.rs:314-345`;
backend at `core/src/store/backend.rs`.

## The claim that must NOT be re-added

The ratchet-session half of this finding is **out of scope**. Revision 1 of the
audit asserted plaintext root keys sat in sled; that was wrong.
`RatchetSessionManager::with_backend` is test-only, so `ratchet_sessions_v1` is
never written in production. The doc comment at
`core/src/store/session_manager.rs:307` ("the caller must ensure the storage is
encrypted at rest") is currently unviolated **only vacuously**. Treat it as a
trap for whoever later wires a session-manager backend -- and note that landing
this ticket makes that comment a real obligation rather than a latent one.

## Migration

Wrap the sled backend in an XChaCha20-Poly1305 envelope keyed by a device-bound
key from the platform keystore (Android Keystore / macOS Keychain / Windows DPAPI
or equivalent). The `derive_key_blake3` device-bound pattern already in
`core/src/crypto/backup.rs` is the existing precedent -- follow it rather than
inventing a second one.

## Acceptance criteria (each independently checkable)

- `grep -n "encrypt\|Cipher\|Aes\|Secret" core/src/store/backend.rs` returns
  encryption code. It returns **nothing** today.
- A test writes an identity, reads the raw stored bytes for `IDENTITY_KEY`, and
  asserts they differ from `keys.to_bytes()` **and** that `IdentityKeys::
  from_bytes` on the raw stored blob **fails**. Full identity recovery from a
  disk image must become impossible.
- **A migration path for existing plaintext installs is written and tested.**
  Without this, landing the change silently bricks every node that already has a
  store. This is the single highest-risk part of the ticket.
- The key comes from the platform keystore. Device-unlock and keystore-loss
  behaviour is specified and covered by a test -- including what a user sees and
  what they can recover when the keystore key is gone.
- Migration is idempotent and resumable: an interrupted migration must not leave
  a half-encrypted store.

## Sequencing note

Independent of T2 and safe to run in parallel with it. If only one CRITICAL is
funded first, fund this one.