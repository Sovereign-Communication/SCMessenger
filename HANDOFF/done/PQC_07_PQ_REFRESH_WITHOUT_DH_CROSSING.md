# TASK [HIGH, design question]: PQ ratchet refresh when no DH crossing occurs

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO. Split out 2026-07-12 from
`PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` after that ticket's literal
wiring bug was fixed and re-verified, but a narrower, deeper issue survived.

## The scenario

The PQ ratchet cadence fix (see the ticket above) correctly mixes a fresh
ML-KEM shared secret into the root key WHEN a DH ratchet step happens to
coincide with the cadence trigger (`handle_dh_ratchet` now receives the
real `pq_ss` instead of a hardcoded `None`). But `handle_dh_ratchet` is
only ever called from `RatchetSession::decrypt` when the peer's DH public
key has changed (`dh_changed`) - i.e., only when the OTHER side has sent a
reply carrying a new DH key.

A session with prolonged one-directional traffic (e.g. Alice sends many
messages in a row with no reply from Bob) never triggers a DH crossing on
Alice's own session state during that burst - `encrypt()` doesn't rotate
DH keys, only `decrypt()` does, in response to the peer's key changing.
So even though the PQ cadence trigger (every ~100 messages) fires and
`perform_pq_ratchet_step`/`handle_incoming_pq_fields` run correctly and
transmit/receive real ML-KEM material, there's no DH crossing for that
fresh secret to be mixed into - it's computed, transmitted, received, and
then has nowhere to go.

Confirmed empirically: `core/tests/integration_pq_session.rs
::test_pq_ratchet_cadence_refreshes_shared_secret`'s scenario is exactly
this (one-directional burst), and its root-key-changed assertion still
fails after the wiring fix, for this reason specifically (not the original
hardcoded-`None` bug, which is independently confirmed fixed).

## Why the obvious fix isn't safe

The natural next idea - mix `perform_pq_ratchet_step`'s own `_ss_pq`
(currently discarded, underscore-prefixed) directly into `self.root_key`
immediately, independent of waiting for a DH crossing - was investigated
and rejected without a design decision first: Alice's and Bob's `root_key`
values are NOT equal at arbitrary points between DH crossings in this
double-ratchet design (each crossing leaves the receiver one preemptive
round ahead of the sender until the next reply flips it back). Each side
independently mixing the "same" `ss_pq` into their own, different local
root-key snapshot would desynchronize them, breaking chain-key derivation
at the next real DH crossing - a correctness bug potentially worse than
the one being fixed.

## What needs deciding (design, not implementation)

1. Should the PQ ratchet cadence be allowed to trigger a root-key refresh
   independent of DH crossings at all, or is "PQ material only refreshes
   alongside a DH crossing" an acceptable design constraint (i.e., a
   one-directional burst session simply doesn't get PQ forward-secrecy
   refresh until the next reply, but DOES get it at that point, since the
   wiring fix now correctly threads it through the next crossing)? If the
   latter, the real fix might just be: update the test to reflect a
   two-directional exchange (which the wiring fix DOES correctly handle,
   confirmed by the 6/6 passing tests in `integration_pq_session.rs`
   covering other scenarios) rather than requiring a no-crossing refresh at
   all - re-scope down to "document this as an accepted limitation" if the
   security team agrees it's acceptable.
2. If independent-of-crossing refresh IS required, design a symmetric
   mechanism that keeps both sides' root keys converged without
   depending on DH-crossing timing - e.g. a transcript-bound or
   ratchet-generation-numbered scheme so both sides can independently
   derive a matching update. This needs real protocol design + adversarial
   review before any implementation, not a quick patch.
3. Either way, decide what should happen to `perform_pq_ratchet_step`'s
   currently-discarded `_ss_pq` sender-side value - is it dead weight (the
   sender never needs its own encapsulation's shared secret, only the
   ciphertext it sends), or is it meant to be used somewhere that never
   got wired?

## Gate

This is a genuine cryptographic protocol design question requiring
operator/security-lead input before implementation - do not let an
automated worker unilaterally decide the design and ship it. Once designed,
implementation touches `core/src/crypto/ratchet.rs`/`encrypt.rs` and
requires the standard mandatory adversarial review.
