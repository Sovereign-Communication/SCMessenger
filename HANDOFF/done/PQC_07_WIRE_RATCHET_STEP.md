# TASK: Wire PQ ratchet step + incoming PQ fields into the live encrypt/decrypt path

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO -- HIGH PRIORITY, [AUDIT-GATE]. Confirmed via adversarial
review (`HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md`, CRITICAL
finding). `RatchetSession::perform_pq_ratchet_step()` and
`handle_incoming_pq_fields()` exist, compile, and pass their own isolated
unit tests in `core/src/crypto/ratchet.rs`, but NEITHER is ever called from
the real message send/receive path in `core/src/crypto/encrypt.rs`. Result:
the ML-KEM shared secret is fixed at session bootstrap and never refreshed
-- PQC-07's intended PQ-forward-secrecy property is not actually active.

## Current (broken) state -- read before changing anything

`encrypt_message_ratcheted` (`core/src/crypto/encrypt.rs:330-378`):
```rust
if session.negotiated_suite == Some(0x02) {
    let (pq_kem_ciphertext, pq_encaps_key) = if !session.peer_confirmed {
        if let Some(hct) = &session.bootstrap_hct {
            (Some(hct.mlkem_ciphertext.clone()), None) // pq_encaps_key comes in later tasks (PQC-07)
        } else {
            (None, None)
        }
    } else {
        (None, None)   // <-- every message after the first sends nothing
    };
    ...
}
```

`decrypt_message_ratcheted_v2` (`core/src/crypto/encrypt.rs:276-313`) never
reads `envelope.pq_kem_ciphertext`/`envelope.pq_encaps_key` at all; it goes
straight to `session.decrypt(dh_public, message_number, nonce, ciphertext, aad)`.

## Fix

### Sender side: `encrypt_message_ratcheted`

Decide and implement a PQ ratchet cadence. Simplest correct option (do this
unless it conflicts with something you find in `RatchetSession::encrypt` --
read it first): call `session.perform_pq_ratchet_step()` whenever a NEW DH
ratchet step happens on the sending side (i.e., whenever `dh_step_count`
would increment / a new `our_dh_public` is generated -- check how the
existing DH ratchet decides this, e.g. `force_ratchet()` or the sending-side
analogue, and hook the PQ step to the same trigger). If sending-side DH
ratchet steps are not currently triggered by `encrypt()` itself (Signal-style
protocols usually only ratchet DH on the RECEIVING side, in response to a
new incoming DH key), then instead: call `perform_pq_ratchet_step()` once
per N messages (pick N consistent with any existing rekey-interval constant
if one exists -- grep for `MAX_RATCHET_STEPS`/rekey/interval constants
first) OR once per outgoing DH public key change if the sender ever changes
its own DH key. Populate `pq_kem_ciphertext`/`pq_encaps_key` in the envelope
with the FRESH values from `perform_pq_ratchet_step()`'s return whenever a
step was performed -- not just pre-confirmation.

Keep the existing pre-confirmation bootstrap behavior (first message still
carries the bootstrap ciphertext) -- do not remove that; add the ongoing
ratchet-step behavior alongside it, not instead of it.

### Receiver side: `decrypt_message_ratcheted_v2`

Before calling `session.decrypt(...)`, check if
`envelope.pq_kem_ciphertext.is_some()`. If so, call
`session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)` and
ensure the returned/updated shared secret is what `session.pq_ss` holds
when `session.decrypt(...)` subsequently triggers `handle_dh_ratchet`
internally (check whether `handle_incoming_pq_fields` already updates
`self.pq_ss` as a side effect -- per the function's current body, it does:
`self.pq_ss = Some(ss_pq.to_vec());` -- so calling it BEFORE `decrypt()` in
the right order should be sufficient; verify this is actually true by
tracing the call order and confirm `handle_dh_ratchet` reads the UPDATED
`self.pq_ss`, not a stale clone taken earlier in the same call frame).

Call `session.validate_pq_fields_present(envelope.pq_kem_ciphertext.is_some())`
at the appropriate point (this function already exists and is also
currently orphaned -- wire it in as the anti-downgrade/anti-stripping check
it was designed to be).

## Constraints

- Do NOT change `perform_pq_ratchet_step`, `handle_incoming_pq_fields`,
  `validate_pq_fields_present`, or any other function signature in
  `ratchet.rs` -- only change CALL SITES in `encrypt.rs`.
- Do NOT change suite 0x01 (classical) behavior at all.
- Preserve backward decrypt compatibility: old stored/in-flight envelopes
  without PQ fields must still decrypt via the classical-only path.
- Add unit/integration test coverage proving the fix: a full round trip
  where (a) a message N steps into a session has a PQ ciphertext in its
  envelope (not just the first message), (b) the receiver's derived root
  key differs from what it would be if the PQ step were skipped (i.e.
  actually prove `pq_ss` changes root key derivation across a ratchet
  step, not just at bootstrap).

## Gate

```
cargo test -p scmessenger-core --lib
cargo test -p scmessenger-core --test integration_pq_session --test integration_e2e --test integration_ironcore_roundtrip
```
All must pass, including new tests added for this fix.

## Post-fix requirement

This touches `core/src/crypto/` -- mandatory adversarial re-review before
this task is considered done (per repo security rules). Do not skip.

## Output format (MANDATORY)

Return the FULL updated contents of `core/src/crypto/encrypt.rs` (and
`core/src/crypto/ratchet.rs` ONLY if you added test coverage there) in
fenced code blocks, filename as first line inside each block.
