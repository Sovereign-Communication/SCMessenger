# V040 D10 — REVIEWER DISPATCH PACKET (rule-8 independent adversarial review)

Filed: 2026-09-10T03:20Z by the CTO seat (author of the change under review;
this packet deliberately does NOT contain the verdict — that belongs to an
independent reviewer).

## Assignment

- Change under review: commit `7ff317f0` — "D10: validate relay-reservation
  bases to fix mDNS LAN-discovery death" on branch
  `cto/t2-disk-ruling-2026-08-31` (PR #279).
- Rule-8 perimeter: `core/src/transport/` (gated directory).
- Review packet (the document under review):
  `HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md`
  — read it FIRST; it contains the defect narrative, the exact change list,
  the 8 regression tests, the author's own risk assessment, and four
  explicit review-focus questions.
- Diff to review: `git show 7ff317f0 -- core/src/transport/swarm.rs`
  (+360/−4). Everything else in that commit is documentation.
- Tree to review against: current PR head (docs-only deltas since; the
  `swarm.rs` bytes are identical to `7ff317f0`).

## Reviewer eligibility (rule 8 + PR #235 governance precedent)

- MUST NOT be the author (the CTO seat authored it — this disqualifies every
  session in this Freebuff conversation thread).
- Prefer a different model family than the author (PR #235 precedent:
  same-lane self-review "has failed twice and is not a credential").
- Eligible: MAC lane (GPT/Codex on the operator's MacBook), or any
  operator-designated reviewer with repo read access and shell capability.
- HTTP-only lanes are NOT eligible (they cannot run the verification
  commands; per standing operator rules, only shell-capable lanes may assert
  build/test results).

## What the reviewer must do (procedure)

1. Read the review packet end to end.
2. Read the diff. Attack the four "Review focus requested" questions in the
   packet — especially (1) whether wildcard binds should claim NOTHING
   instead of private-only, and (2) multiaddr parsing edges
   (`/p2p-circuit` casing, ws/wss wrapping) that could smuggle a circuit
   base past `is_valid_reservation_base`.
3. Verify the regression tests actually pin the claims: run
   `python scripts/build_lock.py --holder <your-name> --run "cargo test -p
   scmessenger-core --lib d10_ 2>&1"` (Windows authoritative environment;
   expect 8 passed / 0 failed).
4. Optional but valued: attempt to construct a poison address that passes
   the new gate but would still overflow mDNS (the adversarial ask).
5. Record the verdict as a NEW file in `HANDOFF/review/` named
   `V040_D10_RESERVATION_BASE_<APPROVE|REJECT>_<reviewer>_<date>.md`
   (T14 precedent naming), containing: verdict, per-question answers to the
   packet's review-focus items, any commands run with their output (or
   UNVERIFIED marks), and any conditions attached to an APPROVE.
6. Commit the verdict file with an explicit path and push to
   `cto/t2-disk-ruling-2026-08-31` (PR #279) — or hand it to the operator /
   CTO seat to file verbatim.

## Consequences (what the verdict gates)

- APPROVE (with or without conditions): unblocks PR #279 merge to main once
  the operator accepts; the CTO seat will record the verdict reference in
  CTO_STATE.md and CEO_STATE.md.
- REJECT or CONDITIONS-MAJOR: the change is reworked on the branch; a new
  review packet revision is filed; merge stays blocked.
- No verdict: merge stays blocked indefinitely (current state since
  2026-09-10T00:00Z).

## Author's disclosures (for the reviewer's benefit, not to bias)

- The gate was live-deployed to both desktop nodes before independent review
  (branch-deploy precedent T14/D2); live evidence: zero `TxtRecordTooLong`
  after relaunch vs 206 before, canonical single-circuit listener set,
  successful phone rejoin. This is OPERATIONAL evidence, not review
  evidence — the rule-8 gate is about the code, not the rollout.
- One live observation the author wants reviewed: the reservation once
  selected a link-local IPv6 base from AWS's identify set (accepted; the
  canonical public base is what registered). The author assesses this as an
  ordering nit, not a validity defect — reviewers may disagree.
- The author did NOT exhaustively audit other `listen_on` call sites beyond
  the reservation path and `SwarmCommand::Listen` (recon note: the latter's
  senders were checked; a second pair of eyes is welcome).
