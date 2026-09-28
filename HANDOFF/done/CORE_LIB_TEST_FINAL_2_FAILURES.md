# TASK: Fix the last 2 real test failures (1121 passed, 2 failed)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

The previous compile-fix round got `cargo test -p scmessenger-core --lib`
from 41 compile errors down to 2 runtime failures. Both root causes are
already diagnosed below -- apply the exact fixes, do not re-diagnose.

## Fix 1 (PRODUCTION BUG): core/src/crypto/encrypt.rs line 508-513

```rust
audit.append(
    crate::observability::AuditEventType::LegacyStaticEcdhSend,
    Some(peer_id.to_string()),
    None,
    None,
);
```

`AuditLog::append`'s real signature (core/src/observability.rs:242-248) is
`append(event_type, identity_id: Option<String>, peer_id: Option<String>,
details: Option<String>)`. This call puts the peer identifier into the
`identity_id` slot, leaving the real `peer_id` field `None` -- a genuine
production bug (audit events for legacy ECDH sends currently misattribute
the peer). Fix: swap the argument so the peer id lands in the `peer_id`
slot:
```rust
audit.append(
    crate::observability::AuditEventType::LegacyStaticEcdhSend,
    None,
    Some(peer_id.to_string()),
    None,
);
```
This makes `crypto::encrypt::tests::test_audit_log_legacy_static_ecdh_send`
(which asserts `audit_log.events[0].peer_id == Some("test_peer".to_string())`)
pass, and fixes the real audit-trail correctness bug.

## Fix 2 (TEST BUG): core/src/crypto/ratchet.rs, test
`test_init_as_receiver_hybrid_then_decrypt` (around lines 913-957)

The test currently does:
```rust
let (hct, _) = crate::crypto::pq::hybrid::hybrid_encapsulate(&bob_x25519.to_bytes(), &bob_bundle.mlkem_encaps_key).unwrap();
...
let mut alice_session = RatchetSession::init_as_sender_hybrid(&alice_key, &bob_bundle, transcript_hash).unwrap();
let bob_x25519_secret = super::super::encrypt::ed25519_to_x25519_secret(&bob_key);
let mut bob_session = RatchetSession::init_as_receiver_hybrid(&bob_key, &bob_x25519_secret, &bob_mlkem_keypair, &alice_bundle, &hct, transcript_hash).unwrap();
```

Bug: the standalone `hybrid_encapsulate(...)` call at the top computes its
OWN independent hybrid ciphertext (`hct`) with a fresh random ephemeral
X25519 key. But `init_as_sender_hybrid` internally calls
`hybrid_encapsulate` AGAIN itself and stores the result in
`alice_session.bootstrap_hct` (this is a public field on `RatchetSession` --
see `core/src/crypto/ratchet.rs` struct definition, field
`pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>`).
These are two DIFFERENT hybrid ciphertexts (different ephemeral keys ->
different derived shared secrets). Bob decapsulates the wrong one, so his
root key never matches Alice's, and decryption fails.

Fix: delete the standalone `let (hct, _) = hybrid_encapsulate(...)` line
entirely, and instead use `alice_session`'s own bootstrap ciphertext:
```rust
let mut alice_session = RatchetSession::init_as_sender_hybrid(&alice_key, &bob_bundle, transcript_hash).unwrap();
let hct = alice_session.bootstrap_hct.clone().expect("sender_hybrid sets bootstrap_hct");
let bob_x25519_secret = super::super::encrypt::ed25519_to_x25519_secret(&bob_key);
let mut bob_session = RatchetSession::init_as_receiver_hybrid(&bob_key, &bob_x25519_secret, &bob_mlkem_keypair, &alice_bundle, &hct, transcript_hash).unwrap();
```
(Reorder so `alice_session` is created BEFORE `hct` is read from it. Check
if `bootstrap_hct`'s type `HybridCiphertext` derives `Clone` -- it is used
with `.clone()` elsewhere in ratchet.rs already, e.g. `bootstrap_hct:
Some(hct.clone())`, so this is safe.)

Check `test_pq_ratchet_step` and any OTHER test in the same file for the
identical mistake (a standalone `hybrid_encapsulate` call whose result is
later fed into a receiver-side init instead of using the sender session's
own `bootstrap_hct`) -- fix every occurrence of this pattern, not just the
one named above.

## Do NOT

- Do not change `AuditLog::append`'s signature, `RatchetSession` fields, or
  `hybrid_encapsulate`/`init_as_sender_hybrid`/`init_as_receiver_hybrid`
  themselves.
- Do not touch any other test or production logic.

## Gate

```
cargo test -p scmessenger-core --lib
```
Must show 0 failed (currently 1121 passed, 2 failed).

## Output format (MANDATORY)

Return the FULL updated contents of BOTH files, each in its own fenced code
block, filename as the first line inside the block:
`// core/src/crypto/encrypt.rs` and `// core/src/crypto/ratchet.rs`.
