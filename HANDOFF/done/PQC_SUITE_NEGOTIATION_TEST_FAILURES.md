# TASK: Diagnose why 3 integration_pq_session tests get V1 envelope instead of V2

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: read-only diagnosis first, no fix yet - need root cause before touching
anything (this is core/src/crypto/ - mandatory adversarial review on any fix).

## Symptom

`cargo test -p scmessenger-core --test integration_pq_session --features test-utils -j 2`
now compiles cleanly but 3 of 6 tests FAIL at runtime:

```
test_pq_session_full_send_receive (panicked at: Expected V2 envelope)
test_pq_session_transcript_mismatch (panicked at: assertion failed: res.is_err())
test_pq_ratchet_cadence_refreshes_shared_secret (panicked at: Expected V2 envelope for message #1)
```

3 tests PASS: `test_pq_session_lost_first_envelope`, `test_v2_initiator_to_v1_peer`,
`test_pq_session_persistence`.

This is the FIRST time this specific test file has ever actually executed
(it was auto-discovered by cargo before, but this session added a
`required-features = ["test-utils"]` gate to `core/Cargo.toml` so it only
just started compiling+running for real). So it's unclear whether these 3
tests EVER passed, or whether something in this session's broader PQC-09/10
identity/keys.rs changes (ML-DSA fields added to `PublicKeyBundle`,
`IdentityKeys` gained `mldsa_keypair`) broke suite negotiation.

## What's already confirmed (do not re-derive)

- `sign_bundle()` (`core/src/identity/keys.rs` ~line 333) sets
  `supported_suites = vec![0x01, 0x02, 0x03]` - includes 0x02, so
  `should_use_ratcheted_encryption` (`core/src/crypto/encrypt.rs` ~line 428)
  SHOULD return `Ok(true)` for any bundle from `sign_bundle()`.
- The `true` branch of `encrypt_with_ratchet_fallback` (~line 502-524), when
  both `our_bundle`/`recipient_bundle` are `Some`, calls
  `manager.get_or_create_session_hybrid(peer_id, sender_signing_key, our_b, their_b)`
  then `encrypt_message_ratcheted(...)`, which constructs `WireEnvelope::V2`
  whenever `session.negotiated_suite == Some(0x02)` - so IF this path runs
  correctly, a V2 envelope should always result.
- The failing test (`test_pq_session_full_send_receive`) passes
  `Some(&bob_bundle)`/`Some(&alice_bundle)` explicitly for both bundle
  parameters at every call site - not a test-side "forgot to pass the
  bundle" bug on its face.

## Questions to answer

1. Trace `RatchetSessionManager::get_or_create_session_hybrid`
   (`core/src/crypto/session_manager.rs`) - under what conditions does it
   fail hybrid negotiation and fall back to a session with
   `negotiated_suite != Some(0x02)` (or return an Err that gets `?`-propagated,
   in which case the failure would show as a `.unwrap()` panic, not "Expected
   V2 envelope" - so more likely it succeeds but negotiates something other
   than 0x02)?
2. Does anything about the ML-DSA fields added to `PublicKeyBundle` this
   session (`mldsa_public: Option<Vec<u8>>`, `mldsa_signature: Option<Vec<u8>>`)
   affect bundle verification/negotiation in a way that could cause hybrid
   negotiation to silently fall through to classical/V1? Check
   `verify_bundle` (`core/src/identity/keys.rs`) and anywhere
   `get_or_create_session_hybrid` might call bundle verification before
   accepting suite 0x02.
3. Is there a difference between how `test_pq_session_lost_first_envelope`
   (PASSES) and `test_pq_session_full_send_receive` (FAILS) construct their
   bundles/sessions that would explain why one negotiates hybrid correctly
   and the other doesn't, given both call the same `generate_identities()`
   helper in this test file?
4. State plainly: is this a test-construction bug (test itself does
   something subtly wrong) or a production-code regression (real bug in
   `get_or_create_session_hybrid`/verification path)? If production, this
   blocks PQC-07's actual claimed "hybrid session establishment" guarantee
   and is more serious than a test-only issue.

## Output format

Plain-text diagnosis with file:line evidence, ending with a clear verdict:
TEST BUG (describe the fix) or PRODUCTION BUG (describe the fix, and flag
severity). Do not write a code fix in this pass unless the diagnosis is
completely unambiguous - if there's any doubt, stop at the diagnosis and let
the orchestrator decide the fix.
