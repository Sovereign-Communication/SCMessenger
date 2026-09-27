<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## E-01 Attempt 3 -- Required Constraints

Status: AWAITING_OPERATOR_REVIEW
Written: 2026-07-17
Source: Qwen THINK (qwen3-235b-a22b-thinking-2507) analysis of attempt 1/2 failure modes.

### What Failed

- Attempt 1: ROOT KEY DESYNC under message reorder. Asymmetric mixing
  (sender/receiver mixing on their respective DH steps) caused divergent root
  keys when messages arrived out of order.
- Attempt 2: ROOT KEY DESYNC under packet loss. Symmetric mixing tied to a
  specific message (carrying KEM ciphertext) caused desync when that message
  was lost, as sender mixed while receiver never received the trigger.

### Root Pattern

Both attempts failed due to unsynchronized mixing events where network
unreliability (reorder or loss) caused sender and receiver to apply pq_ss at
different logical points in the ratchet.

### Constraints Attempt 3 Must Satisfy

1. SYNCHRONIZATION: Mixing must occur at identical logical points for both
   parties, determined solely by observable protocol state (not local timing
   or message content).
2. LOSS-SAFETY: If the triggering message is lost, neither party must mix
   pq_ss; the protocol must recover synchronization on subsequent messages
   without desync.
3. REORDER-SAFETY: Message reordering must not affect mixing timing; both
   parties must derive identical root keys regardless of message arrival
   sequence.
4. KDF SOUNDNESS: Mixing must use identical symmetric KDF inputs (root key +
   pq_ss) on both sides with no asymmetric parameters.
5. RATCHET EPOCH BINDING: Mixing must be irrevocably tied to a specific DH
   ratchet epoch boundary, using the public DH component from the envelope
   header as the synchronization anchor.

### Recommended Design Direction

Tie pq_ss mixing to the DH ratchet step: when a new DH public key appears in
the envelope header (triggering `handle_dh_ratchet`), both sides mix pq_ss
into the root key during that step. This leverages the DH ratchet's
self-synchronizing property (both derive identical new DH keys from public
headers) to ensure epoch agreement. Message loss skips the epoch cleanly,
while reordering does not affect epoch boundaries since DH steps are
determined by public headers.

### What Attempt 3 Must NOT Do

- Mix pq_ss based on sender/receiver role asymmetry (e.g., sender mixes on
  send, receiver on receive).
- Bind mixing to message content or sequence numbers that may be
  lost/reordered (e.g., "mix when processing message N").
- Allow pq_ss mixing outside a DH ratchet step boundary (e.g., during
  message encryption/decryption).

### OPERATOR ACTION REQUIRED

Review this constraints document. If accepted, the next step is E-01b:
design the full spec (THINK/MAX tier, adversarial review mandatory before
any code is dispatched for E-01c).

Gate: E-01c may NOT be dispatched until E-01b carries an adversarial PASS
on file in HANDOFF/review/.
