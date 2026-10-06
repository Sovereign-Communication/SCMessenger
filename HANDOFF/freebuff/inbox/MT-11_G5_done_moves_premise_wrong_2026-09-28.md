Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / MT-11
Type: PREMISE-WRONG

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
Repository: scmessenger

G5's second instruction -- "Move to `HANDOFF/done/` with an evidence citation each"
-- names ten tickets as verified-done. **Most of them are not done.** I moved
none of them. The G5 address correction *was* implemented and is staged as PR
#406; this report is only about the ticket moves.

## Evidence -- each named ticket's own Status line on main

```
$ for t in <each name>; do f=$(ls HANDOFF/todo/${t}*.md | head -1); \
    grep -m1 -iE '^\*\*Status|^Status' "$f"; done

V040_LEDGER_SEEDING_AND_GOSSIP            Status: PARTIAL -- items 1, 2, 3 (response half), 6
                                                implemented; items 4...
P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT      (not in todo/)
P0_ANDROID_FINITE_RETRY_ABANDONMENT       Status: Active -- DISPOSITIONED 2026-08-24:
                                                ACCEPTED FOR rc.1, remains v...
D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING  Status: OPEN (filed 2026-08-30, scoped out of
                                                PR #244 per adversarial re...
RECEIPT_MARKER_ID_FLAVOR_MISMATCH         Status: Active
P0_ANDROID_SELF_RATCHET_RESET             Status: FIXED -- dispositioned 2026-08-24
                                                against main ceabdbd4
P0_DEEPLINK_PARSES_BUT_NEVER_DIALS        (not in todo/)
P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE  Status: FIXED -- dispositioned 2026-08-24
                                                against main ceabdbd4
RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN      Status: Ready for implementation
```

Against "move to done/":

| Ticket | Status | Move? |
|---|---|---|
| `V040_LEDGER_SEEDING_AND_GOSSIP` | **PARTIAL** -- items 4+ outstanding | **no** |
| `P0_ANDROID_FINITE_RETRY_ABANDONMENT` | **Active**, accepted for rc.1, still open | **no** -- and it is car **V5-10** |
| `D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING` | **OPEN**, scoped out of PR #244 | **no** |
| `RECEIPT_MARKER_ID_FLAVOR_MISMATCH` | **Active** | **no** |
| `RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN` | **Ready for implementation** | **no** |
| `P0_ANDROID_SELF_RATCHET_RESET` | FIXED | maybe, needs re-verification |
| `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE` | FIXED | **no** -- contradicted below |
| `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT` | not in `todo/` | already moved, or elsewhere |
| `P0_DEEPLINK_PARSES_BUT_NEVER_DIALS` | not in `todo/` | already moved, or elsewhere |

Four are unambiguously not done and one is "Ready for implementation", which is
the opposite of done. Moving those would put a false completion record in
`HANDOFF/done/` -- the exact falsification AGENTS.md rules 13 and 15 forbid, and
what the JEV `status_honesty` axis exists to catch.

## The one that is contradicted by the train's own plan

`P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE` reads `Status: FIXED`. But this
train's own car list still carries it as open work:

- task-file section 6, **MT-09m**: "async delivery receipts convergence | P1;
  verify after MT-03 + MT-09d"
- TRAIN_STATE ledger: `MT-09m | async delivery receipts convergence | P1; verify
  after MT-03 + MT-09d`

So the ticket's own FIXED line and the train's plan disagree. The plan is the
newer, more specific statement and explicitly defers the verdict to after
MT-03 and MT-09d, neither of which has run. **The FIXED line is 2026-08-24 and
predates the whole train.** I did not move it.

The other FIXED line, `P0_ANDROID_SELF_RATCHET_RESET`, has the same
2026-08-24 vintage and the same problem: it was dispositioned against
`main ceabdbd4`, which is many hundreds of commits behind the current
`faf22a57`. "FIXED against an old main" is not evidence of fixed now.

## G5's other two counts are also stale

- **"Archive the 11 `INBOX_2026-08-11*` files"** -- zero remain, not 10 or 11.
  Sub-item is **DONE-VERIFIED**; nothing to do.
  ```
  $ git ls-tree -r origin/main --name-only HANDOFF/todo/ | grep -c "INBOX_2026-08-11"
  0
  ```
- **"Target: `HANDOFF/todo` under 10 files"** -- there are **46**. G5 was written
  on 2026-08-31 and the queue has grown since. Not reachable by the INBOX
  archiving alone.

## What I did instead

Implemented the first G5 item in full and staged it as **PR #406**: the dead
`54.226.67.101` correction in the three files G5 names, pointing at
`scripts/aws_node_ip.sh` rather than substituting a new literal address.
`scripts/docs_sync_check.sh` -> `docs-sync-check: PASS` (rc=0).

Also closed **#227** under A4 (CONTAINED proven with Appendix A's exact test --
`rc -eq 0 && tree == $T`; note the ancestry test misleads here, the commit was
applied then superseded).

## What I need

One line for the ticket moves:

- `G5 MOVES CONFIRMED` -- you know of state I do not; name which tickets and I
  will move exactly those, with a citation each.
- `G5 MOVES PARTIAL` -- I move only the two `FIXED` tickets after re-verifying
  each against current `main`, and record the re-verification in the citation.
- `G5 MOVES DROP` -- retire the instruction; the train's own car list already
  tracks this work and is the more accurate ledger.

The dependabot half of G5 (merge #214/#212/#211, rebase-or-close #141/#103,
close #156, defer #213/#210) is unblocked only by the JEV-ORDER ruling, since
every one of those is a merge.
