# CRYPTO-T2 -- Make the hybrid ML-KEM-768 send path reachable

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- awaiting implementation
Severity: CRITICAL (finding F2 of `HANDOFF/audit/QUANTUM_CRYPTO_AUDIT_2026-10-03.md`)
Gate: touches `core/src/crypto/` and `core/src/store/` -- **Rule-8 adversarial
review required before merge.** The implementer cannot sign off their own change.
Raised: 2026-10-04, extracted from T2 of
`HANDOFF/audit/QUANTUM_CRYPTO_REMEDIATION_TASKS.md`, which existed only as prose
and therefore had no queue position.
Blocks: 3-node verification is NOT blocked by this (the runtime does not depend
on PQ). It blocks any claim that the product is post-quantum protected.

## Mechanism, re-verified 2026-10-04 against `integrate/train-20261004` @ `200bfc54e`

```
grep -rn "save_contact_bundle" core/src cli/src --include=*.rs
  core/src/store/contacts.rs:770:    pub fn save_contact_bundle(      <- definition
  core/src/store/contacts.rs:1594:  (test)
  core/src/store/contacts.rs:1628:  (test)
```

One definition, two test call sites, **zero production callers**. The only writer
of the `contact_bundle_key` keyspace is therefore never invoked in production,
so `get_contact_bundle` (`core/src/store/contacts.rs:787`, called from
`core/src/iron_core.rs:1049-1052` on the send path) always returns `None`.

That `None` decides the branch at `core/src/crypto/encrypt.rs:549-561`:

```rust
if let (Some(our_b), Some(their_b)) = (our_bundle, recipient_bundle) {
    ... manager.get_or_create_session_hybrid(...)   // ML-KEM-768 path -- NEVER TAKEN
} else {
    // Fallback to classical V1
    ... manager.get_or_create_session(...)         // classical suite-0x02 ratchet
}
```

**Consequence: ML-KEM-768 encapsulation is never invoked on the production send
path.** Additionally `require_pq` is passed as the literal `false` at
`core/src/iron_core.rs:1067`.

### Correction to the source audit's wording

The audit states `verify_bundle` has "zero production callers." It has exactly
one, at `core/src/store/contacts.rs:777` -- but it sits *inside*
`save_contact_bundle`, so it is unreachable for the same reason. The conclusion
is unchanged; the phrasing would send a fixer hunting for a second, phantom gap.

## Why this is not a one-line wiring job

There is no contact-establishment mechanism that could carry a bundle. Two prior
implementation attempts stopped on this. Established: `InviteToken::verify` /
`verify_with_policy` and `verify_bundle` are unreachable; `BootstrapManager::
accept_invite` is documented dead (`core/src/relay/bootstrap.rs:134`);
`InviteSystem` has zero users; contacts are created only by manual entry or
backup restore. `mlkem_encaps_key` cannot be derived from an Ed25519 key (1184
bytes of independent randomness), so a peer must *supply* one out of band.

## Operator decision already on file (2026-10-03)

Bundle announcement over the existing authenticated message channel.
**Key-change detection and pinning are deferred, and that is accepted risk, not
an oversight.** Consequence: a peer may re-advertise a different bundle at will,
so anyone able to write to the channel can substitute an ML-KEM key. This is
the same Ed25519-anchor weakness as F1, now reachable by substitution rather
than by CRQC.

## Coupling -- do not sequence these separately

T1/F1 (ML-DSA as the primary identity anchor, binding a PQ-verifiable
transcript) is coupled. The session root still folds a transcript hash over two
**Ed25519** public keys (`core/src/crypto/ratchet.rs:478-484`,
`core/src/crypto/negotiation.rs:52-64`). An attacker who breaks Ed25519
presents their own Ed25519, X25519 and ML-KEM keys, knows both DH secrets, and
decapsulates the ML-KEM themselves. Shipping T2 without T1 advertises
post-quantum protection that does not hold against a CRQC. They must not be
reported as two separate completed efforts.

## Acceptance criteria (each independently checkable)

- The transport decision is recorded on file naming the surface, the
  authentication, and the pinning / re-advertisement answer.
- `grep -rn "save_contact_bundle" core/src cli/src --include=*.rs` shows at least
  one **non-test** call site on the real production path. Today: none.
- A **new** integration test drives the production send path end to end --
  through the contact-establishment path, not the crypto layer directly and not
  a hand-seeded contact store -- and asserts the emitted envelope is
  `WireEnvelope::V2` with `suite == 0x03` and `pq_kem_ciphertext.is_some()`.
- Existing PQ suites (`integration_pq_verification_suite.rs`,
  `integration_pq_session.rs`, `test_v2_hybrid_envelope_forgery_is_rejected.rs`)
  pass **unmodified**. They hand-seed the contact store, so they prove the
  primitives work and prove nothing about reachability.
- `verify_bundle` gains a production caller and a test asserts a
  tampered-signature bundle is refused on the real path.
- Only then: `require_pq` (`core/src/iron_core.rs:1067`) becomes `true`, with a
  test asserting a peer with no bundle is **refused**, not silently downgraded.
- The reachability guard lands with this ticket (T10 in the source list): an
  end-to-end PQ assertion plus a symbol-level check that each public symbol in
  `core/src/crypto/pq/` has a non-test, non-`core/tests/` call site.

**Watch for:** a green PQ suite is **not** evidence this ticket is done. That is
the exact failure mode that let the defect survive since PQC-05.

## Adjacent, do not fold in

`HANDOFF/todo/CRYPTO_T4_ENCRYPT_AT_REST.md` is the other CRITICAL and is
**independent and more urgent**: it is exploitable today with no quantum
computer at all.