# TASK [CRITICAL]: PQ ratchet cadence never mixes the shared secret into root_key

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## UPDATE 2026-07-14: THIRD attempt (DH-step-tied mix) reviewed -> BLOCKED

Status: STILL OPEN, THREE attempts blocked. Attempt 3 followed this ticket's own
synthesized guidance (tie the PQ mix to a DH ratchet step for self-sync via the
public envelope header) and was adversarially reviewed on a free lane
(qwen3.7-max, independent of the qwen-thinking drafter). VERDICT: BLOCK.
Artifacts: `HANDOFF/review/PQC_07_ATTEMPT3_DRAFT.md` +
`HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md`. Nothing was applied; core/
tree clean throughout.

**KEY NEW INSIGHT (invalidates part of the attempt-1/2 synthesis below):**
`handle_dh_ratchet` in `ratchet.rs` is a strictly RECEIVER-SIDE operation -
it fires on the DECRYPT path when the peer's DH public rotates. The sender
does NOT execute `handle_dh_ratchet` for the same message; it rotates its own
DH only on its next `encrypt`-triggered step. So tying the mix to
`handle_dh_ratchet` is STILL asymmetric - Bob (receiver of M100) mixes ss_pq
into root_key when he processes Alice's DH rotation, but Alice did not mix it
when she sent M100, so her sending chain for M100 derived from the un-mixed
root key -> Bob's receiving chain (mixed) can't authenticate M100 -> AEAD
failure -> session bricked on the very message carrying the PQ material. This
is attempt 1's failure class shifted by one round-trip, NOT a fix for it.
Therefore the DH-STEP anchor is insufficient by itself: attempt 4 needs a
trigger that BOTH the sender's `encrypt`/`perform_pq_ratchet_step` path AND the
receiver's `decrypt`/`handle_dh_ratchet` path provably reach for the SAME
message before deriving that message's chain key - likely mixing ss_pq at
symmetric DH-crossing points on BOTH sides (sender mixes its own encapsulated
_ss_pq at perform_pq_ratchet_step; receiver mixes the decapsulated ss_pq at the
matching step), proven equal by construction, NOT just at the receiver's
handle_dh_ratchet. The draft also had a compile error (stray `None };`) and
still shipped no Kani/unit symmetry proof (the DoD item that would catch this
whole class). Attempt 4 is real protocol design; do NOT single-shot it. This
is NOT beta-blocking (PQC-11/13 stay frozen behind it, farm V1.0.0 ships on the
bootstrap hybrid handshake without ongoing-cadence PQ security).

## UPDATE 2026-07-13: SECOND attempt (decoupled mix_pq_secret design) triangulated -> BLOCKED

Status: STILL OPEN, two attempts blocked now - this is a genuinely hard protocol
design problem, not a wiring gap. Attempt 2 introduced `mix_pq_secret(&mut self,
ss_pq: &[u8])`, decoupling PQ mixing from any DH-crossing entirely: both sender
(after encrypting the ciphertext-carrying message) and receiver (after decrypting
it) call mix_pq_secret with their independently-derived-but-identical (by KEM
construction) shared secret. This correctly fixed attempt 1's asymmetric-mixing
bug (verified: the mixing call is now symmetric, same derivation on both sides).

**Fusion Lite triangulated review (3-panel + judge, real spend ~$0.07 of the
$0.50 capped key) found a NEW fatal flaw, not present in attempt 1's bug class
but just as severe:** tying the mix trigger to a SPECIFIC message means if that
message is lost in transit (dropped, network partition, exactly the farm's
unreliable-connectivity reality this whole plan exists to serve), one side mixes
while the other never does - root_key desyncs again, just via packet loss instead
of reorder. 2 of 3 panelists plus the judge flagged this; judge's verdict: "the
cryptographic key derivation is secure, but the state-machine architecture is
fatally vulnerable to packet loss and requires structural revision before
implementation." Also unresolved: whether `chain.index` should reset to 0 on
chain-key replacement (standard Double Ratchet semantics, argued by 1 panelist)
or be preserved (argued by 2) - a genuine open design question, not just a bug.

**Per the verification protocol, this is torn down, not micro-patched** - the
recommended fix (tie PQ mixing to DH ratchet steps for self-synchronization via
public envelope headers, so the receiver only proceeds once it has processed the
same header the sender committed to) is itself a return toward attempt 1's
DH-tied approach, but done correctly this time - this needs real design work,
not another single-shot dispatch. Attempt 2's design + wiring diffs + full
triangulation verdict preserved at `HANDOFF/review/PQC_07_ATTEMPT2_*` for
whoever designs attempt 3. Nothing was ever applied to the real files - the
working tree was clean throughout this attempt.

**What attempt 3 needs to get right (synthesizing both blocked attempts):**
1. The mix must happen at a point BOTH sides are guaranteed to reach together,
   surviving message loss/reorder - a DH ratchet step (which already has
   built-in self-synchronization via the public key in the envelope header) is
   the natural anchor, NOT an arbitrary message-count cadence.
2. But attempt 1's bug was mixing asymmetrically AT that DH step (receiver
   mixed, sender didn't) - so DH-tied is fine as the trigger, the fix is making
   BOTH sides mix at that trigger, symmetrically, using attempt 2's sound KDF
   mechanism (mix_pq_secret's blake3 derivation, which passed review cleanly).
3. Resolve chain.index reset-vs-preserve explicitly as part of the design, not
   left implicit.
4. Add the Kani/unit proof of root-key-symmetry invariant that both attempts
   have skipped - this is exactly the kind of thing that would have caught
   both blocked attempts before they reached review.

---

## UPDATE 2026-07-12 (LATER): attempted wiring fix ADVERSARIALLY REVIEWED -> BLOCKED -> REVERTED

Status: STILL OPEN. The wiring fix described in the section below was submitted
to the mandatory `crypto-security-auditor` gate on 2026-07-12. **Verdict: BLOCK.**
It was reverted; main is restored to the (inert-but-not-bricking) prior state.
The attempted diff is preserved at
`HANDOFF/review/PQC_07_ATTEMPTED_PQ_SS_WIRING_REVERTED.patch` for the redesign.

Why it was blocked (confirmed by reading the code):
1. **HIGH / regression — asymmetric root-key mixing bricks a live session.** The
   fix made the RECEIVER mix `fresh_pq_ss` into the root key (in
   `handle_dh_ratchet`, both chain derivations) but the SENDER still discards
   its encapsulated secret (`perform_pq_ratchet_step`, ratchet.rs:685,
   `let (ct, _ss_pq) = encapsulate(...)`). When the receiver mixes, its root key
   diverges from the sender's. Concrete brick: suite 0x02, sender bursts >100
   messages one-directionally; msg #100 is a PQ-cadence message; under
   relay drop/reorder it is the FIRST message of that chain the receiver sees ->
   `dh_changed == true` AND pq fields present -> receiver mixes `pq_ss` while the
   sender derived that chain with `None` -> AEAD fails, root_key already
   overwritten (no rollback) -> session permanently bricked. The pre-change code
   (both sides `None`) decrypted this exact pattern fine.
2. **MEDIUM / fix inert in the common path.** `message_number % 100 == 0` only
   occurs mid-chain (chains reset index to 0), where the sender's DH is
   unchanged -> receiver `dh_changed == false` -> `fresh_pq_ss` is dropped
   without ever calling `handle_dh_ratchet`. So in normal operation the secret
   still reaches `root_key_ratchet_v2` on neither side. Matches
   `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md`.

**What the real fix must do (auditor guidance):** the sender must mix the SAME
secret at a deterministic, both-sides-synchronized ratchet point (not "receiver
mixes, sender discards", and not an independent per-side mix into differing
local root_key snapshots — that desyncs, exactly as this ticket's original
analysis below warned). Add a Kani/unit proof of root-key mix SYMMETRY as DoD.
This is a genuine double-ratchet protocol-design task; re-run the crypto gate on
the redesign before any merge.

---

## UPDATE 2026-07-12: literal wiring bug FIXED; deeper design question found, split out

The literal bug described below (item 3 - `handle_dh_ratchet` hardcoding
`pq_ss` to `None`) is fixed: `RatchetSession::decrypt` now takes a
`pq_ss: Option<Vec<u8>>` parameter, threaded from
`decrypt_message_ratcheted_v2`'s captured `handle_incoming_pq_fields(...)`
return value, through to `handle_dh_ratchet`, replacing both hardcoded
`None` branches. Verified: `cargo build --workspace` clean,
`cargo test -p scmessenger-core --lib` 1130/1130,
`cargo test -p scmessenger-core --features test-utils --test
integration_pq_session` 6/6. Not yet committed - mandatory adversarial
review still required before merge (see Gate).

**However, the coverage test's disabled assertion (Step 5,
`test_pq_ratchet_cadence_refreshes_shared_secret`) still fails, for a
DIFFERENT and deeper reason than the original bug**, confirmed by empirical
re-run after the fix: in that test's one-directional scenario (Alice sends
~105 messages, no reply from Bob after the initial handshake), Alice's own
DH public key never rotates after the handshake (only `encrypt()` runs,
which doesn't rotate DH keys), so `handle_dh_ratchet` is never called again
on her session at all - the PQ cadence trigger at message ~100 has no
accompanying DH crossing for the freshly-threaded `pq_ss` to mix into. The
wiring fix only activates `if dh_changed`; here it's never true.

The "obvious" alternative fix (mix `perform_pq_ratchet_step`'s own
`_ss_pq` directly into `self.root_key` immediately, independent of any DH
crossing) was investigated and is NOT safe to ship without dedicated
design + review: tracing the double-ratchet root-key handoff chain shows
Alice's and Bob's `root_key` values are never equal at arbitrary points
between DH crossings (each crossing leaves the receiver one preemptive
round ahead until the next reply) - an independent mix of the same
`ss_pq` into each side's differing local snapshot would desynchronize
their root keys and break chain-key derivation at the next real crossing.
This is a genuine protocol-design question, not a wiring gap. Split out as
its own ticket: `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md`. This ticket
(the literal wiring bug) can close once its own review passes; the split
ticket tracks the remaining, harder question and is NOT release-blocking
in the same way (one-directional message bursts without any reply are a
real but narrower scenario than the general PQ-ratchet-is-inert bug this
ticket originally described).

The disabled assertion in `integration_pq_session.rs` was re-enabled,
re-confirmed to still fail for the new/narrower reason, and re-disabled
with an updated comment explaining precisely why (distinguishing "wiring
bug: fixed" from "no-DH-crossing refresh: open design question, tracked
separately") rather than left permanently red or silently reverted to the
stale original comment.

**Also found, NOT fixed (same defect, out of this ticket's scope per
explicit instruction not to touch it unprompted):**
`RatchetSession::force_ratchet` (`core/src/crypto/ratchet.rs`, currently
~line 649-671) hardcodes `pq_ss: None` for its own `root_key_ratchet_v2`
call, identical shape to the bug this ticket fixed. `force_ratchet` is a
separate, manually-triggered ratchet path (not reached via `decrypt()`),
so the same threading approach doesn't automatically apply - needs its own
look. Tracked as `PQC_07_FORCE_RATCHET_SAME_DEFECT.md`.

Status: TODO. Found 2026-07-11 while adding test coverage for the PQC-07
cadence trigger (`PQC_07_CADENCE_TEST_COVERAGE.md`, done/). This is a
PRE-EXISTING bug in the original PQC-07 implementation, not introduced this
session. Mandatory adversarial review before merge (`core/src/crypto/`).

## Finding

The "PQ ratchet cadence" feature (every 100 messages, refresh PQ material for
ongoing forward secrecy) correctly transmits real ML-KEM-768 ciphertexts and
encapsulation keys over the wire, and both sides correctly encapsulate/
decapsulate them - but the resulting shared secret is **never actually mixed
into the session's root key, on either side, ever**:

1. `core/src/crypto/ratchet.rs::perform_pq_ratchet_step` (~line 680): the
   sender's own encapsulation produces `_ss_pq` (underscore-prefixed,
   deliberately unused) and only the ciphertext `ct` is kept/sent. This
   might be fine BY DESIGN if the sender is meant to derive its "fresh"
   secret later from the receiver's reply (see point 3) - but point 2 below
   means that path is also broken.
2. `core/src/crypto/encrypt.rs::decrypt_message_ratcheted_v2` (~line 313):
   `session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)?;` -
   the `Result<Vec<u8>>` return value (the decapsulated shared secret) is
   NOT captured (no `let ss = ...`). It's computed and immediately dropped.
3. `core/src/crypto/ratchet.rs::handle_dh_ratchet` (~line 550-591), called
   from `decrypt()` (~line 527) whenever the peer's DH position changes -
   this is THE function that actually calls `root_key_ratchet_v2(..., pq_ss)`
   to mix PQ material into the root key. But `pq_ss` here is:
   ```rust
   let pq_ss = if self.negotiated_suite == Some(0x02) {
       // Check if we have PQ fields available (should be provided externally)
       // For now, we'll assume the PQ handling is done in the calling code
       // This function just handles the DH part, PQ is handled separately
       None
   } else {
       None
   };
   ```
   BOTH branches return `None` unconditionally - the "should be provided
   externally" never happens anywhere in the current call graph. So every
   single DH ratchet step for suite 0x02 sessions calls
   `root_key_ratchet_v2(&root_key, dh_output, None)` - the PQ component is
   always absent.

**Net effect: the entire ongoing PQ ratchet cadence is cryptographically
inert.** It costs bandwidth and computation (real KEM operations, real wire
transmission) but contributes ZERO security benefit - a quantum-capable
attacker who breaks the classical X25519 DH fully compromises the session
regardless of the PQ material ever exchanged post-bootstrap. Only the
INITIAL bootstrap hybrid handshake (a separate code path, PQC-06) actually
achieves hybrid security; nothing after that does.

## Verification test (already added, correctly demonstrates this)

`core/tests/integration_pq_session.rs::test_pq_ratchet_cadence_refreshes_shared_secret`
was written to assert the root key changes after the cadence trigger. That
assertion is currently commented out / disabled (see that test's own
comments) specifically because it correctly fails against current code -
re-enable it as part of this fix's Definition of Done.

## What a real fix needs

1. Thread the decapsulated shared secret from `handle_incoming_pq_fields`'s
   return value through to wherever the NEXT `handle_dh_ratchet` call
   happens for that session, so `pq_ss` in `handle_dh_ratchet` is real
   material, not always `None`. This likely means:
   - `decrypt_message_ratcheted_v2` captures `let ss_pq = session.handle_incoming_pq_fields(...)?;`
     and stores it on the session (e.g. a new `pending_pq_ss: Option<Vec<u8>>`
     field) rather than discarding it.
   - `decrypt()`/`handle_dh_ratchet()` reads and clears that stored value
     when performing the next DH ratchet step, instead of hardcoding `None`.
2. Consider whether `perform_pq_ratchet_step`'s own `_ss_pq` (sender side)
   should also be used somehow, or whether the ping-pong design (sender
   only sends ciphertext + new encaps key, receiver decapsulates and stores
   the secret for ITS next DH step) is the intended shape - clarify design
   intent before implementing, this is a real architecture question, not
   just a wiring bug.
3. Update/re-enable the disabled assertion in the coverage test once fixed.
4. Add a Kani proof or additional unit test specifically asserting no DH
   ratchet step for a suite-0x02 session with pending PQ material ever
   produces the SAME root key transition as one without it (i.e. prove the
   PQ material is load-bearing, not just present).

## Do NOT

- Do not change the wire format (`pq_kem_ciphertext`/`pq_encaps_key` fields)
  - only the internal handling of the already-decapsulated secret.
- Do not weaken the cadence trigger condition (message-100 logic) - that
  part works correctly.

## Gate

Mandatory `crypto-security-auditor` (or equivalent) adversarial review
before merge - this is a live-path change to the double-ratchet root key
derivation. Standard compile gate + `cargo test -p scmessenger-core --test
integration_pq_session --features test-utils -j 2` green with the
previously-disabled assertion re-enabled and passing.
