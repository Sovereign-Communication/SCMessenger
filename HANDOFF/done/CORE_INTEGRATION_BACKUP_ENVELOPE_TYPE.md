# TASK: Fix WireEnvelope/Envelope type mismatch in integration_backup.rs (last workspace compile error)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

`core/tests/integration_backup.rs` lines 326 and 389 call:
```rust
let plaintext = decrypt_message_ratcheted(session, &first_envelope)
```
and
```rust
let plaintext = decrypt_message_ratcheted(session, &next_envelope)
```

`first_envelope`/`next_envelope` are produced by `encrypt_message_ratcheted`
(`core/src/crypto/encrypt.rs:330`), which returns
`Result<crate::message::WireEnvelope>` -- an enum:
```rust
pub enum WireEnvelope {
    V1(Envelope),
    V2(EnvelopeV2),
}
```
But `decrypt_message_ratcheted` (`core/src/crypto/encrypt.rs:245`) takes
`envelope: &crate::message::Envelope` -- the INNER type of the `V1` variant,
not the `WireEnvelope` wrapper. This test only ever creates classical
(non-hybrid) sessions (`get_or_create_session`/`create_receiver_session`,
no bundles involved), so `encrypt_message_ratcheted` will always return
`WireEnvelope::V1(...)` here.

## Fix

At both call sites (lines ~311-321 and ~372-384, where `first_envelope` and
`next_envelope` are bound), destructure the `WireEnvelope::V1` variant to
get the inner `Envelope` before calling `decrypt_message_ratcheted`. Two
acceptable approaches -- pick whichever reads more cleanly in context:

Option A (destructure at binding time):
```rust
let first_envelope = {
    ...
    let wire_envelope = encrypt_message_ratcheted(...).expect("alice encrypts first message");
    match wire_envelope {
        crate::message::WireEnvelope::V1(env) => env,
        crate::message::WireEnvelope::V2(_) => panic!("expected classical V1 envelope in this test"),
    }
};
```
(same pattern for `next_envelope`)

Option B (destructure at the call site):
```rust
let crate::message::WireEnvelope::V1(ref first_envelope_inner) = first_envelope else {
    panic!("expected classical V1 envelope in this test")
};
let plaintext = decrypt_message_ratcheted(session, first_envelope_inner)
    .expect("bob decrypts first message");
```

Use whichever keeps the rest of the test's existing structure and variable
names most intact -- do not rename `first_envelope`/`next_envelope` if
avoidable, and do not change any other logic, assertion, or test in this
file.

## Do NOT

- Do not change `encrypt_message_ratcheted`, `decrypt_message_ratcheted`,
  `WireEnvelope`, or `Envelope` themselves.
- Do not change any other test in this file.

## Gate (IMPORTANT: use -j 2 -- this machine's rustc crashes at default
parallelism on a cold/large rebuild; -j 2 is confirmed stable)

```
cargo test -p scmessenger-core --test integration_backup --no-run -j 2
```
Must succeed (currently the only 2 errors blocking the full workspace
compile gate).

## Output format (MANDATORY)

Return the FULL updated contents of `core/tests/integration_backup.rs` in
one fenced code block with `// core/tests/integration_backup.rs` as the
first line inside the block.
