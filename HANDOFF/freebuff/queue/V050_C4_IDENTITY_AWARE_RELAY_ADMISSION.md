# Task: c4 fix -- identity-aware relay admission (known/unknown classification + aggregate unknown pool)

Lane: Freebuff (0.5.0). Priority: next implementation task after the current
mission. Review gate: **Rule-8 mandatory** (touches `core/src/transport`).

## Decision on file

Operator ruling 2026-09-18 (via agent decision capture, after the rule-9
escalation in the PR #305 review): adopt **Option A** for the 0.5.0 lane.
Source: `HANDOFF/audit/C4_IDENTITY_ADMISSION_DECISION_BRIEF_2026-09-18.md`
(read it fully before implementing). Interim posture until this ships: option E
-- the residual is documented in the PR #305 verdict's follow-ups and bounded by
the node-global 200/hr ceiling; measured natural inbound relay peak ~15/hr.

## What it fixes

c4 from the PR #305 rule-8 review: TRN-07's per-peer relay share is keyed by the
connection's libp2p PeerId. A peer holding N keypairs draws N individual shares
and can reconstitute the full relay budget, denying relay to every other peer
for the remainder of the hour window. Identity multiplication, same defect class
as TRN-07. Panel tally: real, 2R/1NR, judge medium severity.

## Implementation (from the brief's Option A)

1. At admission, derive the requester's `identity_id` from the
   noise-authenticated PeerId (recover public key, `blake3(pubkey)`); classify
   against contacts + device registry (`identity_id_idx`,
   `register(identity_id, device_id, seniority_timestamp)`). Zero wire change:
   noise already proves possession of the key that hashes to the PeerId.
2. **Known** identities: individual share (today's `max(global/4, 25)`).
   **Unknown** identities: one aggregate pool (default `global/4` per hour for
   all unknowns combined -- operator chose the brief's default; make it a
   config knob).
3. Newcomer grace window: small individual share for first contact, so the
   invite-first mesh stays welcoming. Shape is implementer's choice between the
   brief's two candidates (fixed minutes per PeerId, or first-N-admissions);
   record the choice and rationale in the PR.
4. Extend the already-extracted pure admission ladder so the whole
   classification + pool logic is a pure function with the
   `relay_per_peer_budget_tests` test pattern. Regression tests must fail
   without the fix (N identities must NOT sum to N shares once unknown).
5. Native and wasm loops stay in step; both compile clean.

## Verification gates (all required before "done")

- `cargo fmt`, `cargo clippy -D warnings` clean; targeted lib tests green
  locally, wide sweep left to CI.
- Rule-8 harness panel on the new admission windows (3/3 seats, <= $0.10, same
  structured-claims mode as the TRN-07 and zombie-fix gates).
- wasm build compiles with no new warnings.

## Explicitly out of scope

Signed relay requests (brief Option B) -- groundwork if relay abuse
attribution ever becomes a requirement, NOT a c4 fix (operator confirmed out of
scope absent that requirement). Proof-of-work (D). Raising any cap.

## Evidence trail

- Brief: `HANDOFF/audit/C4_IDENTITY_ADMISSION_DECISION_BRIEF_2026-09-18.md`
- c4 verdict row: `HANDOFF/review/RULE8_PR305_VERDICT_2026-09-18.md`
- Live-measured baseline: `HANDOFF/audit/LIVE_VERIFICATION_305_2026-09-18.md`
