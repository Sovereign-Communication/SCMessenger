# CRYPTO-T12 -- Malformed PQ envelope fields panic the receiver

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- not started
Severity: **HIGH** (remote denial of service). The independent review that
raised this rated it LOW and explicitly did not trace it; tracing it shows it is
worse than reported. See "Why the severity is raised" below.
Gate: touches `core/src/crypto/` and `core/src/message/` -- Rule-8 adversarial
review required before merge.
Raised: 2026-10-04

## Mechanism

`build_receiver_v2_session` (`core/src/crypto/encrypt.rs`, the hybrid-ciphertext
block) copies two attacker-controlled wire fields into FIXED-SIZE arrays:

```rust
let hct = if let Some(mlk) = &envelope_v2.pq_kem_ciphertext {
    let mut e = [0u8; 32];
    e.copy_from_slice(&envelope_v2.ephemeral_public_key);   // line 635
    let mut m = [0u8; 1088];
    m.copy_from_slice(mlk);                                // line 637
```

`copy_from_slice` **panics** when source and destination lengths differ.

## Why the severity is raised above the reviewer's LOW

The review noted the panic risk at `encrypt.rs:636-638` but wrote "I didn't
trace whether lengths are checked earlier," and rated it LOW. The trace:

```
core/src/message/types.rs:119   pub ephemeral_public_key: Vec<u8>,
core/src/message/types.rs:129   pub pq_kem_ciphertext: Option<Vec<u8>>,
```

Both are unconstrained `Vec<u8>` on the wire -- there is no fixed-size array
type, so serde accepts any length. Searching the receive path for validation:

```
grep -rn "pq_kem_ciphertext" core/src --include=*.rs | grep -iE "len\(\)|size|1088|check"
```

returns only test fixtures (`message/codec.rs`, all `vec![6u8; 1088]`) and one
positional parse in `drift/envelope.rs:351`. **No length check exists on the
inbound path.**

So the lengths are neither constrained by the type nor validated by any code.
The path is reached on every inbound V2 envelope that carries a
`pq_kem_ciphertext`, including the first-contact and divergence-recovery paths.

## Impact

A single crafted `EnvelopeV2` with `pq_kem_ciphertext` of any length other than
1088 bytes -- or `ephemeral_public_key` other than 32 bytes -- panics the
receiving node. In a mesh that relays and stores envelopes on behalf of others,
that is remotely triggerable and takes the node down. It is a denial of
service, not a confidentiality break, which is why this is HIGH and not
CRITICAL.

A peer does not need to be authenticated: the panic occurs while parsing
bootstrap material, before the transcript check that would reject an impostor.

## Pre-existing, not introduced by the recent crypto work

```
git log -1 -L 633,638:core/src/crypto/encrypt.rs
306e3149c fix(core): auto re-establish ratchet session on decrypt divergence
```

It arrived with the divergence-recovery feature, not with the Rule-8 fixes. The
recent work moved this code between functions but did not change it.

## Proposed fix

Validate before copying, and fail closed. Do not pad and do not truncate -- a
wrong length is malformed input and must be an error, because silently
normalising it would let a peer drive us into a KEM derivation with material we
invented.

1. `if envelope_v2.ephemeral_public_key.len() != 32 { bail!(...) }`
2. `if mlk.len() != 1088 { bail!(...) }`

Better still, express the expectation as a named constant next to the ML-KEM
size so the two sites cannot drift, and reuse it in the tests.

## Acceptance criteria (each independently checkable)

- A test builds a V2 envelope whose `pq_kem_ciphertext` is `vec![0u8; 1]` and
  asserts `decrypt_with_ratchet_fallback` returns `Err` rather than panicking.
- The same for a `vec![0u8; 1087]` and a `vec![0u8; 1089]` -- off-by-one on
  both sides.
- The same for `ephemeral_public_key` of length 31 and 33.
- Each test is a genuine `Err` assertion; if the current code panics, that is
  the proof the test is load-bearing.
- The existing PQ suites (`integration_pq_verification_suite.rs`,
  `integration_pq_session.rs`,
  `test_v2_hybrid_envelope_forgery_is_rejected.rs`) pass unmodified.

## While in this area

`core/src/crypto/encrypt.rs` has further unguarded `copy_from_slice` calls into
fixed-size buffers on the same inbound path -- lines 200, 679, 719, 945, 956 and
1001. They were not all traced. A single sweep that length-checks every inbound
copy before the array is filled is preferable to patching this one site and
leaving the rest.

## Related

Filed from the LOW finding in
`HANDOFF/review/RULE8_HAIKU_OPUS_VERDICT_a34c60017_2026-10-04.md`. That verdict
is otherwise ACCEPT-scoped: it covers `a34c60017` only.