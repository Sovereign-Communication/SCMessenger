# GPT 5.6 Sol Ultra -- scoped design request: PQ/DH root-ratchet desync

Status: OPEN REQUEST
Raised: 2026-08-01 (Windows Claude, orchestrator)
Priority: CRITICAL -- blocks Wave B (B-01..B-07) and 1.0.0
Prior art: `HANDOFF/todo/E01B_FABLE_DESIGN_HANDOFF.md` (3 failed attempts)

## Why you

Three prior fix attempts failed, each on a DIFFERENT desync mode. This is a
protocol-design question, not an implementation question -- the code compiles
and the primitives are correct. We want a design that is provably synchronized
before anyone writes more code. Scope is deliberately narrow: one function
family in one file.

## IMPORTANT CORRECTION to the existing ticket

`E01B_FABLE_DESIGN_HANDOFF.md` states the PQ secret "is never mixed into
root_key". That premise is WRONG as of today's code, and designing against it
would waste your time. Verified 2026-08-01 at `core/src/crypto/ratchet.rs`:

```rust
// ratchet.rs:1080-1096
fn root_key_ratchet_v2(
    root_key: &RatchetKey,
    dh_output: &[u8],
    pq_ss: Option<Vec<u8>>,
) -> (RatchetKey, RatchetKey) {
    let mut input = vec![root_key.as_bytes().to_vec(), dh_output.to_vec()];
    if let Some(ss_pq) = pq_ss {
        input.push(ss_pq);
    }
    let combined = blake3::derive_key(ROOT_KDF_CONTEXT_V2, &input.concat());
    let new_root  = blake3::derive_key(&format!("{}:root",  ROOT_KDF_CONTEXT_V2), &combined);
    let chain_key = blake3::derive_key(&format!("{}:chain", ROOT_KDF_CONTEXT_V2), &combined);
    (RatchetKey::from_bytes(new_root), RatchetKey::from_bytes(chain_key))
}
```

The mixing EXISTS and is correct in isolation. The defect is in WHEN and WITH
WHICH secret each peer invokes it.

## The actual structural problem (verified)

`handle_dh_ratchet` (ratchet.rs:821) performs TWO sequential root advances in
one call, each with a DIFFERENT PQ secret:

```rust
// ratchet.rs:835-843  -- advance #1, receiving side, uses pq_pending_recv
let pq_ss = if <suite 0x02> { self.pq_pending_recv.clone() } else { None };
let (new_root_key, receiving_chain_key) = root_key_ratchet_v2(
    &self.root_key, dh_output.as_bytes(), pq_ss.as_ref().map(|k| k.as_bytes().to_vec()));

// ratchet.rs:858-869  -- advance #2, sending side, uses pq_pending_sent
let pq_ss_2 = if <suite 0x02> { self.pq_pending_sent.as_ref().map(|p| p.ss.clone()) } else { None };
let (new_root_key_2, sending_chain_key) = root_key_ratchet_v2(
    &new_root_key, dh_output_2.as_bytes(), pq_ss_2.as_ref().map(|k| k.as_bytes().to_vec()));
```

`handle_dh_ratchet_trial` (ratchet.rs:695) mirrors this with its own pair of
calls at :729 and :753.

Two properties make this fragile:

1. **Asymmetric optionality.** `pq_ss` is `Option`. If peer A has `Some` and
   peer B has `None` for the same logical ratchet step -- because the pending
   PQ state arrived on a different schedule -- both sides derive successfully
   and SILENTLY produce different root keys. There is no negotiation failure,
   no error: just divergence discovered later as undecryptable messages.
2. **Two advances, two different secrets, one message boundary.** Each peer
   must perform the same two advances, in the same order, pairing the same
   `pq_pending_recv` / `pq_pending_sent` with the same DH outputs. Any
   reordering, skip, or re-entry desynchronizes the root chain permanently.

Relevant state (ratchet.rs:174-176, 246-247): `pq_our_keypair`,
`pq_prev_keypair` are `Option<MlKem768KeyPair>`; suite is gated on
`self.negotiated_suite == Some(0x02)`.

## What we want from you

1. **A synchronization contract.** State precisely, as a protocol invariant,
   when each peer MUST mix a PQ secret and which one, such that both peers
   perform an identical sequence of root advances. Cover: initiator vs
   responder, simultaneous ratchet (both sides advance at once), out-of-order
   delivery, and a peer that has no PQ material yet.
2. **A resolution for asymmetric optionality.** Should a `None` on one side be
   a hard protocol error rather than a silent different-key derivation? If you
   keep `Option`, explain how both sides provably agree on `None`.
3. **A desync-detection mechanism.** All three prior attempts failed silently.
   We want the design to make divergence LOUD -- e.g. a transcript/epoch
   binding committed into the KDF so a mismatch fails authentication
   immediately instead of surfacing as lost messages later.
4. **A test oracle.** Concretely: what property test or state-machine test
   would have caught all three prior failure modes? The prior attempts skipped
   this and that is why they kept failing. This item is as important as the fix.
5. **Migration.** Suite 0x02 is negotiated; v1 (`root_key_ratchet_v1`) still
   exists. Say whether your design needs a new suite id and how in-flight
   sessions behave.

## Constraints

- Design only. Do NOT write the implementation; we will dispatch that
  separately, and it must pass adversarial review per
  `.claude/rules/security.md` before merge.
- Do not modify X25519 ECDH or XChaCha20-Poly1305 usage.
- Kani proofs live behind the `kani-proofs` feature and must still hold.
- Deliverable: a design note we can turn into a dispatch packet, with explicit
  invariants and the test oracle. Reply as
  `HANDOFF/gpt/GPT_SOL_ULTRA_PQC07_RATCHET_DESYNC_RESPONSE.md`.

## Reference

- `core/src/crypto/ratchet.rs` (1346 lines) -- the whole surface in question
- `core/src/crypto/pq/` -- ML-KEM-768 primitives
- `HANDOFF/todo/E01B_FABLE_DESIGN_HANDOFF.md` -- the 3 failed attempts, with
  the premise correction noted above
