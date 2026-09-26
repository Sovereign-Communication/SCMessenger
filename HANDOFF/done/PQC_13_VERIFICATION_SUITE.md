# TASK: PQC-13 — PQC verification suite: Kani proofs, proptests, cross-version matrix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-05, PQC-06, PQC-07, PQC-08, PQC-09. Wave 5. Min tier: Sonnet.

## Scope

Consolidated verification hardening after the crypto tasks land. Individual tasks already carry their own tests; this task adds the cross-cutting layers the repo's rules require for crypto changes (`.claude/rules/rust.md`: unit + integration + property; `.claude/rules/security.md`: Kani proofs must compile and pass).

## Deliverables

1. **Kani proofs** (`core/src/crypto/kani_proofs.rs`, behind `kani-proofs` feature — follow existing proof style in that file):
   - hybrid combiner determinism: same inputs -> same output.
   - both-inputs-influence (bounded): for symbolic 32-byte `ss_x25519`/`ss_mlkem`, output differs when either differs (formulate as: combiner restricted to the KDF-input construction is injective in each argument position — if full proof is intractable for Kani, prove the concatenation layout is collision-free by construction: fixed widths, no ambiguity; document what was actually proven).
   - EnvelopeV2 codec: decode(encode(x)) == x for bounded field sizes; decode never panics on arbitrary bounded input.
2. **Proptest harness extension** (`core/src/crypto/proptest_harness.rs`): add hybrid encapsulate/decapsulate agreement, PQ-ratchet schedule survival (reuse PQC-07 generator, longer runs), negotiation determinism/symmetry, cross-suite envelope codec fuzz.
3. **Cross-version matrix integration test** (`core/tests/integration_pq_matrix.rs`, register in Cargo.toml):
   | initiator | responder | expected |
   |-----------|-----------|----------|
   | v1 | v1 | classical session, v1 envelopes |
   | v2 | v1 | classical session, v1 envelopes, audit event |
   | v1 | v2 | classical session (responder accepts) |
   | v2 | v2 | suite 0x02, hybrid root, PQ ratchet steps observed |
   | v2 strict | v1 | send error |
   | v2 | v2 (bundle stripped in transit) | establishment refused |
4. **Gate wiring:** confirm `.claude/skills/build_verify.sh` scopes pick up the new test binaries (compile gate `cargo test --workspace --no-run` covers registration; just verify, don't rewrite the script).

## Definition of Done

- [ ] Standard gates PASS.
- [ ] `cargo kani --features kani-proofs` (or the repo's documented kani invocation — check how existing proofs are run; if kani is not installed on the host, record that the proofs COMPILE under `cargo check --features kani-proofs` and escalate for a proof-run environment rather than skipping silently).
- [ ] `cargo test -p scmessenger-core --test integration_pq_matrix` green; full `cargo test -p scmessenger-core` green.
- [ ] Summary of what is proven vs tested vs assumed written into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Weaken or delete any existing proof or property test.
- Mark done with any matrix row skipped.
