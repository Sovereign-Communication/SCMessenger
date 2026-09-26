# HANDOFF: E-01b Design Spec — PQ-secret -> root-key mixing (FABLE ESCALATION)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** DRAFT design from Qwen THINK produced 2026-07-17; needs Fable
(Claude native) validation/refutation against actual code structure before
any adversarial review or E-01c dispatch.
**Priority:** CRITICAL — blocks entire Wave B PQC depth (B-01..B-07)
**Operator decision:** E-01b/c still needed (confirmed 2026-07-17). E-00's
IronCore-layer wiring does NOT satisfy the ratchet-level desync-safety
requirement.
**Escalation reason:** 3 prior attempts failed on 3 different desync modes.
This is the hardest design problem in the v1.0.0 backlog. Qwen THINK
produced a draft but it has suspected structural mismatches with the actual
code that file-level access is needed to validate.

---

## 1. The problem in one paragraph

`handle_dh_ratchet` in `core/src/crypto/ratchet.rs` hardcodes `pq_ss = None`
(lines 581-588) even for suite 0x02 (PQ hybrid) sessions. The PQ ratchet
cadence transmits real ML-KEM-768 ciphertexts and encapsulation keys over
the wire, both sides correctly encapsulate/decapsulate them, but the
resulting shared secret is never mixed into the root key. The entire
ongoing PQ ratchet cadence is cryptographically inert — a quantum-capable
attacker who breaks classical X25519 DH fully compromises the session
regardless of PQ material exchanged post-bootstrap. Only the initial
bootstrap hybrid handshake (PQC-06) achieves hybrid security; nothing
after that does.

E-00 (committed 2026-07-17, 6059038c) wired the ratchet/PQ subsystem into
IronCore's production path at the IronCore layer
(`encrypt_with_ratchet_fallback` / `decrypt_with_ratchet_fallback`), but
the ratchet-level `handle_dh_ratchet` still has `pq_ss = None`. The
operator confirmed that ratchet-level mixing is a separate concern that
still needs the E-01b design + E-01c implementation.

---

## 2. Three failed attempts — READ BEFORE DESIGNING

### Attempt 1: Asymmetric mixing at DH crossing — REORDER DESYNC
- Receiver mixed `fresh_pq_ss` into root_key in `handle_dh_ratchet` (both
  chain derivations). Sender still discarded its encapsulated secret
  (`perform_pq_ratchet_step` ratchet.rs:699: `let (ct, _ss_pq) =
  encapsulate(...)` — underscore-prefixed, unused).
- Result: receiver's root_key diverged from sender's. When messages arrived
  out of order, AEAD failed, session permanently bricked.
- Artifact: `HANDOFF/review/PQC_07_ATTEMPTED_PQ_SS_WIRING_REVERTED.patch`

### Attempt 2: Symmetric per-message mixing — PACKET LOSS DESYNC
- Introduced `mix_pq_secret(&mut self, ss_pq: &[u8])`, decoupled from DH
  crossing. Both sender (after encrypt) and receiver (after decrypt) called
  it with their independently-derived-but-identical (by KEM construction)
  shared secret.
- Result: if the triggering message was LOST, one side mixed while the
  other never did — root_key desynced via packet loss.
- Fusion Lite 3-panel verdict: "cryptographically secure but state-machine
  fatally vulnerable to packet loss."
- Artifacts: `HANDOFF/review/PQC_07_ATTEMPT2_*`

### Attempt 3: DH-step-tied mixing — STILL ASYMMETRIC (one round-trip shifted)
- Followed E-01a guidance: tie pq_ss mixing to `handle_dh_ratchet` (the DH
  ratchet step, self-synchronizing via public envelope header).
- KEY INSIGHT THAT KILLED IT: `handle_dh_ratchet` is a RECEIVER-SIDE
  operation — it fires on the DECRYPT path when the peer's DH public
  rotates. The sender does NOT execute `handle_dh_ratchet` for the same
  message. So tying the mix to `handle_dh_ratchet` is STILL asymmetric:
  Bob (receiver of M100) mixes ss_pq when processing Alice's DH rotation,
  but Alice did not mix when she sent M100 — her sending chain for M100
  derived from the un-mixed root key -> Bob's receiving chain (mixed) can't
  authenticate M100 -> AEAD failure -> session bricked.
- Also had a compile error (stray `None };`) and no Kani/unit symmetry proof.
- Artifacts: `HANDOFF/review/PQC_07_ATTEMPT3_DRAFT.md` +
  `HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md`

### Root pattern across all three
Both attempts failed due to unsynchronized mixing events where network
unreliability (reorder or loss) or sender/receiver asymmetry caused the
parties to apply pq_ss at different logical points in the ratchet.

---

## 3. E-01a Constraints (attempt 4 must satisfy ALL)

1. SYNCHRONIZATION: Mixing at identical logical points for both parties,
   determined solely by observable protocol state (not local timing or
   message content).
2. LOSS-SAFETY: If the triggering message is lost, neither party mixes;
   protocol recovers on subsequent messages without desync.
3. REORDER-SAFETY: Message reordering does not affect mixing timing.
4. KDF SOUNDNESS: Identical symmetric KDF inputs on both sides.
5. RATCHET EPOCH BINDING: Tied to a DH ratchet epoch boundary, using the
   public DH component from the envelope header as the synchronization
   anchor.
6. SYMMETRY PROOF: A Kani proof or unit test asserting root-key symmetry
   (both attempts 1-3 skipped this — it would have caught all three).

---

## 4. Current code structure (post-E-00) — CRITICAL for design validity

### The Double Ratchet flow in THIS codebase (NOT textbook Signal):

- `encrypt()` (ratchet.rs:485) does NOT do DH rotation. It just advances
  the sending chain and produces a ciphertext with the current DH public
  in the envelope header.
- `decrypt()` (ratchet.rs:524) checks if the peer's DH public changed
  (`dh_changed`, line 539). If yes, calls `handle_dh_ratchet()` (line 545).
- `handle_dh_ratchet()` (ratchet.rs:568-620) is RECEIVER-SIDE ONLY:
  1. DH with peer's new DH public using identity secret (first DH) ->
     derives (new_root_key, receiving_chain_key) via root_key_ratchet_v2
  2. Generates NEW DH keypair
  3. DH with peer's new DH public using new secret (second DH) ->
     derives (new_root_key_2, sending_chain_key) via root_key_ratchet_v2
  4. Updates root_key, receiving_chain, sending_chain, dh_step_count
- `perform_pq_ratchet_step()` (ratchet.rs:688-710) is a SEPARATE method:
  encapsulates with peer's PQ key, rotates own PQ keypair, stores pending
  ciphertext. Currently called from... (Fable must verify the call site in
  encrypt.rs — it may be called from `encrypt_message_ratcheted` or from
  the cadence trigger logic, NOT from `encrypt()` directly).
- `handle_incoming_pq_fields()` (ratchet.rs:713-753) decapsulates PQ
  ciphertext, returns `ss_pq: Vec<u8>`. Called from
  `decrypt_with_ratchet_fallback` in encrypt.rs.

### CRITICAL: handle_dh_ratchet has TWO root_key_ratchet_v2 calls

```rust
fn handle_dh_ratchet(&mut self, their_new_dh: &X25519PublicKey) -> Result<()> {
    // First DH (with identity secret)
    let dh_output = first_dh_secret.diffie_hellman(their_new_dh);
    let pq_ss = if self.negotiated_suite == Some(0x02) { None } else { None }; // INERT
    let (new_root_key, receiving_chain_key) = if self.negotiated_suite == Some(0x02) {
        root_key_ratchet_v2(&self.root_key, dh_output.as_bytes(), pq_ss.clone())
    } else { root_key_ratchet_v1(...) };

    // Generate new DH keypair
    let new_dh_secret = X25519StaticSecret::from(new_secret_bytes);
    let new_dh_public = X25519PublicKey::from(&new_dh_secret);

    // Second DH (with new secret)
    let dh_output_2 = new_dh_secret.diffie_hellman(their_new_dh);
    let (new_root_key_2, sending_chain_key) = if self.negotiated_suite == Some(0x02) {
        root_key_ratchet_v2(&new_root_key, dh_output_2.as_bytes(), pq_ss) // same pq_ss
    } else { root_key_ratchet_v1(...) };

    self.root_key = new_root_key_2;
    // ...
}
```

Any design that threads `pq_ss` into `handle_dh_ratchet` must account for
BOTH calls. The Qwen THINK draft only showed one derivation.

### root_key_ratchet_v2 (ratchet.rs:782) — the KDF

```rust
fn root_key_ratchet_v2(
    root_key: &RatchetKey, dh_output: &[u8], pq_ss: Option<Vec<u8>>,
) -> (RatchetKey, RatchetKey) {
    let mut input = vec![root_key.as_bytes().to_vec(), dh_output.to_vec()];
    if let Some(ss_pq) = pq_ss { input.push(ss_pq); }
    let combined = blake3::derive_key(ROOT_KDF_CONTEXT_V2, &input.concat());
    let new_root = blake3::derive_key(&format!("{}:root", ROOT_KDF_CONTEXT_V2), &combined);
    let chain_key = blake3::derive_key(&format!("{}:chain", ROOT_KDF_CONTEXT_V2), &combined);
    (RatchetKey::from_bytes(new_root), RatchetKey::from_bytes(chain_key))
}
```

---

## 5. Qwen THINK draft design (starting point — VALIDATE OR REFUTE)

The Qwen THINK response (tmp/E-01b-design-spec_response.md) proposed
"reply-based deferred mixing":
- Sender stores ss_pq in `pending_pq` during `perform_pq_ratchet_step`
- Receiver stores decapsulated ss_pq in `pending_pq` during `handle_incoming_pq_fields`
- Mixing occurs when: sender receives a reply with new DH public
  (`handle_dh_ratchet`), receiver sends a reply with new DH public
  (`encrypt`)

### SUSPECTED FLAWS the Fable session must verify:

1. **encrypt() does not do DH rotation.** The design says "receiver mixes
   pending_pq in encrypt() when sending DH-step reply" — but encrypt()
   (ratchet.rs:485) just advances the sending chain. DH rotation happens
   in handle_dh_ratchet on the RECEIVE path. So where does the receiver
   mix? If it's in handle_dh_ratchet (when receiving the NEXT message
   after the reply), then the timing is different from what the design
   describes.

2. **Two root_key_ratchet_v2 calls in handle_dh_ratchet.** The design's
   code snippet shows one derivation. The actual code has two (first DH
   for receiving chain, second DH for sending chain). pending_pq must be
   threaded through both, and both sides must use it identically.

3. **perform_pq_ratchet_step call site.** The design assumes it's called
   during the encrypt path. Verify where it's actually called from
   (encrypt.rs — likely encrypt_message_ratcheted or the cadence trigger).
   If it's not called from encrypt(), the "sender stores ss_pq during
   perform_pq_ratchet_step" flow may not align with the message that
   carries the PQ ciphertext.

4. **Root-key convergence under the reply-based scheme.** Trace through
   the ACTUAL code paths (not abstract descriptions) for:
   - Alice sends M (DH step + PQ) — which functions execute, in what order?
   - Bob receives M — which functions execute?
   - Bob sends R (reply) — which functions execute?
   - Alice receives R — which functions execute?
   At each step, what is root_key on each side? Are they identical before
   the mixing point?

---

## 6. What Fable must produce

A DESIGN SPEC (markdown, NOT code) that:

1. **Validates or refutes the Qwen THINK draft.** If the draft is sound
   after fixing the structural mismatches, produce the corrected version.
   If it's fundamentally flawed (like attempt 3), explain why and propose
   a different approach.

2. **Traces through the ACTUAL code paths.** Not abstract descriptions.
   Cite file:line for every function call in the encrypt -> send ->
   receive -> decrypt -> handle_dh_ratchet flow. Show what root_key is
   on each side at each step.

3. **Satisfies all 6 E-01a constraints.** Walk through loss and reorder
   scenarios using the actual code structure.

4. **Addresses the two-pass DH derivation.** handle_dh_ratchet calls
   root_key_ratchet_v2 TWICE. The design must specify what pq_ss value
   goes into each call, and prove both sides use the same values.

5. **Includes a symmetry proof outline.** What test/Kani proof would
   catch the failure modes of all 3 prior attempts?

6. **Does NOT change the wire format.** pq_kem_ciphertext / pq_encaps_key
   fields stay as-is. Only internal handling of the decapsulated secret.

7. **Resolves chain.index reset-vs-preserve.** With justification.

---

## 7. Files Fable must read (in priority order)

1. `core/src/crypto/ratchet.rs` — THE file. Read in full (1048 lines).
   Focus on: encrypt() (485), decrypt() (524), handle_dh_ratchet() (568),
   perform_pq_ratchet_step() (688), handle_incoming_pq_fields() (713),
   root_key_ratchet_v2() (782), struct RatchetSession fields.
2. `core/src/crypto/encrypt.rs` — encrypt_with_ratchet_fallback (503),
   decrypt_with_ratchet_fallback (581), encrypt_message_ratcheted,
   decrypt_message_ratcheted_v2. Find where perform_pq_ratchet_step is
   called and where handle_incoming_pq_fields return value goes.
3. `core/src/iron_core.rs` — prepare_message_internal (~715) and
   receive_message (~2836) to see the IronCore-layer wiring.
4. `core/tests/integration_pq_session.rs` — the test that asserts PQ
   ratchet cadence refreshes shared secret (currently has disabled
   assertion). Read the test to understand what "correct" looks like.
5. `HANDOFF/review/E01a_attempt_constraints.md` — the constraints.
6. `HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` — full
   failure history of attempts 1-3.
7. `HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md` — why attempt 3
   was blocked.
8. `tmp/E-01b-design-spec_response.md` — the Qwen THINK draft being
   validated.

---

## 8. Gate

This design spec MUST pass adversarial review before E-01c (implementation)
is dispatched. The adversarial review will probe for:
- Root-key desync under message loss
- Root-key desync under message reorder
- Sender/receiver asymmetry (the trap that killed attempts 1 and 3)
- Whether the symmetry proof would actually catch the known failure modes

E-01c may NOT be dispatched until E-01b carries an adversarial PASS on file
in `HANDOFF/review/`.

---

## 9. What this is NOT

- NOT beta-blocking: the farm v1.0.0 ships on the bootstrap hybrid
  handshake (PQC-06) without ongoing-cadence PQ security. This is
  post-bootstrap forward-secrecy depth.
- NOT a wire-format change: only internal handling of the already-
  decapsulated secret.
- NOT a KDF redesign: use the existing root_key_ratchet_v2.
- NOT an excuse to skip the symmetry proof: that is a DoD item, not
  optional.
