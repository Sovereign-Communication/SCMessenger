# TASK: Fix 41 pre-existing compile errors in core lib unit tests (stale struct shapes)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO -- HIGH PRIORITY. This blocks the mandatory compile gate
(`cargo test --workspace --no-run`) for the ENTIRE workspace, not just these
two files. Confirmed pre-existing (predates today's PQC-08 work) via a full
error capture.

## Root cause

The `#[cfg(test)]` modules in `core/src/crypto/encrypt.rs` and
`core/src/crypto/ratchet.rs` were written against an OLD shape of
`crate::identity::PublicKeyBundle` and `crate::observability::AuditEvent`.
Both structs were changed (as part of the PQC identity-v2 work) and the
test modules were never updated. This is test-only code; no production
logic needs to change.

## Ground truth: current REAL struct definitions (do not guess)

```rust
// core/src/identity/keys.rs
pub struct PublicKeyBundle {
    pub ed25519_public: [u8; 32],
    pub x25519_public: [u8; 32],
    pub mlkem_encaps_key: Vec<u8>,   // NOT [u8; 32] -- variable length, 1184 bytes typical
    pub created_at: u64,
    #[serde(default = "default_supported_suites")]
    pub supported_suites: Vec<u8>,   // e.g. vec![0x01, 0x02]
    pub signature: Vec<u8>,
}

// core/src/observability.rs
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub timestamp_unix_secs: u64,
    pub identity_id: Option<String>,
    pub peer_id: Option<String>,
    pub details: Option<String>,
    pub prev_hash: String,
}
```

There is NO `identity_id`/`device_id`/`x25519_public_key`/`mlkem768_public_key`/
`timestamp` field on `PublicKeyBundle` -- those names do not exist. There is
NO `actor_id` field on `AuditEvent` -- use `identity_id` or `peer_id` instead
depending on what the test is asserting (read the surrounding test to judge
which; `peer_id` is almost always correct for "who sent/received this event").

For a valid signature is not required by these unit tests (they test
`should_use_ratcheted_encryption`/ratchet mechanics, not signature
verification) -- a dummy `signature: vec![]` or `vec![0u8; 64]` is
acceptable UNLESS the specific test is exercising signature verification
(check first).

## Fix, file by file

### core/src/crypto/encrypt.rs (~10 occurrences, lines ~761-855 per current
compiler output -- re-run to get exact current line numbers, they may have
shifted slightly from other work landing on this file today)

Every `PublicKeyBundle { identity_id: ..., device_id: ..., x25519_public_key:
..., mlkem768_public_key: ..., timestamp: ... }` literal must become:
```rust
PublicKeyBundle {
    ed25519_public: <appropriate [u8;32]>,
    x25519_public: <appropriate [u8;32]>,
    mlkem_encaps_key: <vec![0u8; 32] or similar, as Vec<u8>>,
    created_at: 0,
    supported_suites: vec![0x01, 0x02],
    signature: vec![],
}
```
Adapt field VALUES to match what each specific test needs (some construct a
v1-only bundle with `supported_suites: vec![0x01]` -- check the surrounding
test's intent, e.g. a test named `*_v1_peer*` should NOT advertise 0x02).

Also fix the one `AuditEvent`/audit-log assertion using `actor_id` (around
line 1106): change `audit_log.events[0].actor_id` to
`audit_log.events[0].peer_id` (confirm by reading what the audit call site
being tested actually populates -- it should be consistent with whichever
field `AuditLog::append(...)` writes the peer identifier into).

### core/src/crypto/ratchet.rs (~4-5 occurrences in the `#[cfg(test)]`
module, around lines 891-967 per current compiler output)

- `x25519_public: bob_x25519` / `x25519_public: alice_x25519` where
  `bob_x25519`/`alice_x25519` are `x25519_dalek::PublicKey` values: the
  field wants `[u8; 32]`, so use `.to_bytes()`, e.g.
  `x25519_public: bob_x25519.to_bytes()`.
- `mlkem_encaps_key: [0u8; 32]` -> `mlkem_encaps_key: [0u8; 32].to_vec()`
  (or `vec![0u8; 32]`).
- Missing fields `created_at`, `ed25519_public`, `signature`,
  `supported_suites` in `PublicKeyBundle { ... }` literals -- add them
  (dummy/zero values are fine for these tests; `ed25519_public` can reuse
  the relevant signing key's public bytes if convenient, or `[0u8; 32]`).
- `bob_bundle`/`alice_bundle` not found in scope (E0425 at ratchet.rs:920)
  -- a bundle variable is referenced before it's constructed, or was
  renamed; read the surrounding test function fully and either construct
  it earlier in the function or fix the reference to match whatever
  variable name actually holds the bundle in that scope.
- `hybrid_encapsulate(&bob_x25519, &alice_bundle.mlkem_encaps_key)` --
  `hybrid_encapsulate` expects `&[u8; 32]` for the first arg but
  `bob_x25519` is a `PublicKey` -- pass `&bob_x25519.to_bytes()` (or a
  `[u8; 32]` local bound to a variable first, since `.to_bytes()` returns
  an owned array and you cannot take a reference to a temporary in some
  call shapes -- bind it to a `let` first if the borrow checker complains).
- `init_as_receiver_hybrid(&bob_key, &alice_bundle, &hct, transcript_hash)`
  is missing 2 required arguments (`our_x25519_secret: &StaticSecret`,
  `our_mlkem_keypair: &MlKem768KeyPair`) per its real signature (see
  `core/src/crypto/ratchet.rs` around line 387-401 for the actual param
  list) -- read the enclosing test to find or construct the appropriate
  `bob`-side X25519 secret and ML-KEM keypair (there is likely a
  `bob_key`/keypair already in scope, or generate one via
  `crate::crypto::pq::generate()` and the signing key's derived X25519
  secret -- check how OTHER passing tests in this same file construct
  these, e.g. `init_as_sender_hybrid` callers, for the established pattern).

## Do NOT

- Do not change `PublicKeyBundle`, `AuditEvent`, or any production
  (non-`#[cfg(test)]`) code.
- Do not change `sign_bundle()` or any other non-test function.
- Do not weaken any test's actual assertions to make it "pass easier" --
  only fix construction/argument mismatches so the code compiles and the
  test still checks what its name says it checks.

## Gate (this exact command currently fails with 41 errors -- must reach 0)

```
cargo test -p scmessenger-core --lib --no-run
```
Then also run (must pass, not just compile):
```
cargo test -p scmessenger-core --lib
```

## Output format (MANDATORY)

Return the FULL updated contents of BOTH files, each in its own fenced code
block, filename as the first line inside the block:
`// core/src/crypto/encrypt.rs` and `// core/src/crypto/ratchet.rs`.
