# TASK: Fix 8 call sites in core/tests/integration_pq_session.rs (PQC-08 signature change)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

`encrypt_with_ratchet_fallback` in `core/src/crypto/encrypt.rs:472` was
extended (PQC-08 gating work) from 7 to 9 parameters, adding two new
trailing parameters:

```rust
pub fn encrypt_with_ratchet_fallback(
    sender_signing_key: &SigningKey,
    recipient_bundle: Option<&crate::identity::PublicKeyBundle>,
    recipient_public_key_fallback: &[u8; 32],
    plaintext: &[u8],
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    peer_id: &str,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
    require_pq: bool,                                   // NEW
    audit_log: Option<&mut crate::observability::AuditLog>,  // NEW
) -> Result<crate::message::WireEnvelope>
```

Every call site in `core/tests/integration_pq_session.rs` (lines 28, 64,
106, 112, 144, 177, 206, 222) still uses the old 7-argument form and fails
to compile with E0061 ("this function takes 9 arguments but 7 arguments
were supplied").

## Fix

At each of the 8 call sites, append two trailing arguments:
- `require_pq`: pass `false` unless the surrounding test is specifically
  about strict-mode/require_pq behavior (grep the test function name and
  body for "require_pq" or "strict" to decide; default false).
- `audit_log`: pass `None` unless the test already has an
  `AuditLog`/`audit_log` variable in scope it should be threading through
  (grep the enclosing test function body first) -- if one exists, pass
  `Some(&mut that_variable)` instead of `None`.

Do NOT change `encrypt_with_ratchet_fallback` itself, its callers elsewhere
in the codebase, or any decrypt-path code. This is a test-file-only fix.

## Gate

`cargo test -p scmessenger-core --test integration_pq_session --no-run`
must exit 0.

## Output format (MANDATORY)

Return the FULL updated contents of
`core/tests/integration_pq_session.rs` in one fenced code block with
`// core/tests/integration_pq_session.rs` as the first line inside the
block.
