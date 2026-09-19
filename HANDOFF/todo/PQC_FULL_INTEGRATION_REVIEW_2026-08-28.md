# PQC Full-Integration Review (post-0.5.0)

Status: QUEUED -- operator request 2026-08-28: "after 0.5.0 look at PQC to ensure
it's implemented fully also."
Priority: P1 (security completeness, no release blocker for v0.4.0/v0.5.0)

## Scope

Verify that post-quantum cryptography (PQC) is implemented *fully* across the
stack, not just present in isolated spots. The hybrid suite (0x03, ML-KEM-768 +
X25519, "iron-core session-root v3 2026-08") exists in `core/src/crypto/`; this
review must confirm end-to-end coverage:

- [ ] **Ratchet/negotiation**: both suites (0x02 legacy / 0x03 hybrid) negotiate
      correctly; suite downgrade paths behave (see `crypto/negotiation.rs`,
      `HYBRID_SUITE_IDS`); no path silently falls back to non-PQC-only keys.
- [ ] **Envelope/transport**: PQC material (ML-KEM ciphertexts, hybrid
      encapsulations) survives every transport (LAN/TCP, WS relay, BLE,
      Wi-Fi Aware/Direct, mDNS) -- check for truncation/transcoding in the
      wire envelope (see `P2_WIRE_ENVELOPE_TRUNCATION_2026-08-10.md`).
- [ ] **Identity/signing**: hybrid signatures / PQC-capable identity bundles
      are used on the sender-auth term end-to-end; no Ed25519-only fallback in
      attribution paths (see the `_our_signing_key` perimeter findings --
      `scripts/check_perimeter_underscore_params.py` rationale).
- [ ] **Android/iOS/CLI FFI**: hybrid session init/decrypt reachable from all
      three frontends (udl API surface); no frontend bypasses PQC when the peer
      advertises 0x03.
- [ ] **Key storage**: ML-KEM keypairs persisted + exported/imported with
      identity backups (no regenerated-on-restart drift breaking sessions).
- [ ] **Test coverage**: hybrid round-trip + downgrade + tamper tests in CI;
      negative-path tests (wrong-key decrypt, ciphertext tamper) on all suites.

## Evidence pointers

- `core/src/crypto/pq/`, `core/src/crypto/encrypt.rs`
  (`decrypt_with_ratchet_fallback`, `create_receiver_session`)
- `core/src/crypto/negotiation.rs` (`HYBRID_SUITE_IDS`)
- Unification V3 plan: `HANDOFF/plans/UNIFICATION_V3_DELIVERY_CONVERGENCE_PLAN.md`
- Perimeter scan: `scripts/check_perimeter_underscore_params.py` (from #228)

## Definition of done

A written verdict per bullet above with source pointers and executed test
evidence, filed in `HANDOFF/review/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md`,
plus any fixes landed as PRs.
