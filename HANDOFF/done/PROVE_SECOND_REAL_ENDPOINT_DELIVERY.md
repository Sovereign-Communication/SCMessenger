# Prove message delivery between two INDEPENDENT endpoints, not just Lucas's own two clients

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: DONE -- proven 2026-07-20, both directions confirmed.
Filed: 2026-07-20
Closed: 2026-07-19 (Bob planning session) -- moved to done/ per PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md.
Evidence: Two CLI identities (Alice pubkey b6ffcd97, Bob pubkey 94c1f6cb), both directions confirmed.
Bug fixed as part of proof: handle_send_message peer_id parse (commit 29d01e5b, fusion_lite PASS).

## Why this ticket exists

Read `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` and
`HANDOFF/ALPHA_TEST_SESSION_FINDINGS_2026-07-19.md` in full before starting --
they're the ground truth for what has and hasn't been proven.

Summary: the dial-establishment bug is fixed (commit `f283145`) and verified
-- but ONLY between Lucas's own Windows CLI and Lucas's own local Android
emulator, both dialing the same relay. Every attempt to get a genuinely
SECOND, independent endpoint (an AWS-hosted emulator standing in for Josh)
failed for reasons unrelated to the dial fix itself -- two different missing-
system-library crash loops in the Android emulator image on a no-KVM AWS
instance. The operator accepted Lucas's solo evidence as sufficient proof the
*code fix* works, which is fair, but it is not the same as proving two
strangers' devices can actually message each other. Contact exchange (public
key swap) and an actual delivered-and-acknowledged message have never
happened between two different identities.

## Goal

Demonstrate, end to end, with TWO independent identities that are not both
controlled from the same session:

1. Both connect to the alpha relay (`/ip4/100.56.248.69/tcp/9001`) and both
   show up simultaneously in the relay's live connection state.
2. Contacts are exchanged (deep link or in-app Add Contact -- whichever path
   is actually easiest to drive non-interactively/scriptably).
3. A message is sent from one identity to the other and its delivery +
   receipt is confirmed on the receiving side, not just "send returned 200."
4. Repeat in the other direction.

## Your choice of substitute for "Josh" -- pick whatever actually works

The AWS emulator path is not mandated -- it already failed twice for
infrastructure reasons unrelated to the app. Options include (not
exhaustive, and not a ranked recommendation -- assess and pick):

- A fresh Android emulator with a different system image (the session notes
  suspect the `android-34/google_apis/x86_64` image itself may be
  incomplete/corrupted -- a plain `default` (non-`google_apis`) image has
  fewer HAL/vendor-module dependencies and was flagged as worth trying).
- A KVM-capable AWS instance type instead of the no-KVM `m7i-flex.large`
  (cost tradeoff -- your call, or flag it back to Lucas if it needs spend
  approval).
- One of the farm-sim Docker CLI containers (already proven to work as a
  standalone CLI identity) pointed at the ALPHA relay instead of the
  farm-sim relay -- this sidesteps the Android-emulator problem entirely and
  may be the fastest path to proving the PROTOCOL works between two
  independent identities, even if it doesn't exercise the Android app
  specifically. Worth doing even if you ALSO keep trying the emulator path,
  since it isolates "does two-endpoint delivery work at all" from "does the
  Android emulator boot."
- A second physical Android device, if one becomes available.

## Acceptance

Delivery + receipt confirmed both directions, between two identities that
were provisioned/driven independently (not both from one script/session
pretending to be two people, and not reusing Lucas's own already-connected
identity as one side of the pair -- that's the thing that still hasn't been
proven).

## Notes

- This doesn't require the crypto-security-auditor gate by itself (it's a
  test/proof exercise using existing, already-reviewed code paths), unless
  it surfaces a real bug in `core/src/transport/` or the crypto path, in
  which case the standard gate applies to the fix.
- If the graceful-AF dial-loop backoff/dedup work (see the existing
  `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`, items 3-4, still open) is not
  done first, a second real endpoint joining the relay's ledger will
  reproduce the same promiscuous-dial noise Lucas's single client already
  hit -- not a blocker for this ticket's goal, but expect the noise and
  don't mistake it for a new bug.
