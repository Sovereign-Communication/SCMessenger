# V040 REVIEW DISPATCH -- #272 multi-transport deferral re-review @ 44fee3c4 (qwen free lane)

Date: 2026-09-03
Status: **STAGED -- NOT DISPATCHED** (lane dispatch is CTO-driven; the harness-lane
option additionally awaits the CEO's explicit go. This file is the paste-ready
brief for the CTO's next lane round.)
Supersedes the head pin of: V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md
section 2 (which pinned 177bd840); the acceptance contract is unchanged.
Evidence appendix: V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md
(inbox) -- its analysis is grounded in the candidate tree and TRANSFERS
unchanged to 44fee3c4, for the reason recorded below.
Grounding PRs: #272 (architecture candidate, branch cto/v040-candidate-2026-09-02),
#273 (nimble-peer fix, MERGED into the candidate 2026-09-03, squash 44fee3c4).

## Why the head moved: 177bd840 -> 44fee3c4

PR #273 (nimble-peer fix: kill the 5-minute dead-mark cycle) was merged into the
candidate branch by squash on 2026-09-03 (merge commit 44fee3c4, parent
177bd840, message "fix(core): kill the 5-minute dead-mark cycle -- liveness
beats stale dial-policy state (#273)"). The candidate head that the FINAL
APPROVE verdict covered (177bd840) is no longer the branch tip.

## Why #273 does NOT touch the UDP/QUIC admission question (evidence)

Diff 177bd840..44fee3c4 touches exactly three files:

- core/src/routing/resume_prefetch.rs   |  33 +++++++---
- core/src/transport/dial_policy.rs     | 127 +++++++++++++++++++++++++++++++++++-
- core/src/transport/swarm.rs           |  77 ++++++++++++++++++++--

Verified against that diff:

1. ZERO changed lines touch `listen_port_from_bound_addr` or
   `sync_external_address` (grep of the +/- lines for those names and for
   `Udp|Quic|/tcp/|/udp/` returns nothing).
2. The swarm.rs hunks land at lines 3437, 5166, 5851 -- the dial-policy /
   dead-mark region, far from the 464-495 admission+promotion region.
3. `core/src/transport/observation.rs` and the address-reflection wire
   protocol are untouched.

Conclusion: the TCP-only admission allowlist and /tcp/-only promotion are
BYTE-IDENTICAL between 177bd840 and 44fee3c4. The scoping analysis in
V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md (the transport-less
SocketAddr observation model, the wire-protocol touch required for option (a),
the additive-friendly shape of the later change) transfers unchanged.

## Task for the lane (one round, one verdict file)

Verify at head 44fee3c4 (git ls-remote origin cto/v040-candidate-2026-09-02)
and issue a plain "Verdict: APPROVE" under the directive's option (c) --
CEO-sanctioned deferral -- RESTATED for the moved head. The verdict file must
record, in the reviewer's own words:

(i) UDP/QUIC inbound admission is DEFERRED for the v0.4.0 gate, not excluded
    forever -- "we are not never going to use UDP".
(ii) Nothing in the current shape structurally forecloses the later
     transport-carrying change (additive-friendly; see the scoping note).
(iii) QUIC is already live in the routing ladder (bonus 0.15) and outbound
      classification; only inbound admission/advertisement is deferred.
(iv) The deferral is CEO-sanctioned for the v0.4.0 gate only.

Option (d) -- "TCP-only is fine" on CTO authority with no lane verdict --
remains NOT acceptable. Option (a)/(b) remain open if the lane finds a defect,
in which case the change lands on the candidate, CI runs, and the lane
re-approves at the resulting NEW head.

## Head-move delta to verify in the same round

The candidate now differs from the approved 177bd840 by exactly one squash:
44fee3c4 (parent 177bd840). Verify with:

    git diff --quiet d82978ab 44fee3c4 && echo IDENTICAL

`git diff --quiet` between the #273 reviewed head (d82978ab) and the candidate
tip (44fee3c4) exits 0 -- the trees are BYTE-IDENTICAL. The #273 content
carried into the candidate is therefore exactly the content already carrying a
plain Rule-8 APPROVE (HANDOFF/review/V040_NIMBLE_PEER_REVIEW_QWEN_2026-09-03.md,
continuous non-author reviewer, verdict at d82978ab). The lane should record
this tree identity in the verdict file so the delta addendum for #272's FINAL
APPROVE (see the companion delta request in this inbox) is grounded.

Reviewer constraint: MUST NOT be the author of the candidate or of #273.
Read-only. Do NOT post to the PR (standing no-posting decision). Model routing
per the quota ledger. Verdict file: HANDOFF/review/V040_CANDIDATE_272_DEFERRAL_DELTA_44fee3c4_QWEN_2026-09-03.md
(or the ledger-consistent equivalent).

## Return contract

One inbox file naming: PR #272, head SHA 44fee3c4 reviewed, round number,
verdict file path, and the next action (FLAG-5 resolve -> #272 merge sequence
can proceed after the final-head validation).
