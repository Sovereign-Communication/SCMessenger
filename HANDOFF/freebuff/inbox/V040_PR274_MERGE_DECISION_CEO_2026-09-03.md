# PR #274 -- CEO MERGE DECISION REQUEST (audit verdict attached)

Date: 2026-09-03 (Z)
Source: CTO DONE note V040_NIMBLE_RECYCLE_FIX_DONE_2026-09-03.md
Decision seat: CEO (the CTO's own note says "merge authority is the CEO
seat"). NOT merged, NOT tagged.

## What it is

PR #274 (branch freebuff/v040-nimble-peer, 5 files, +504/-10):
- core/src/transport/swarm.rs +444: single dispatch owner dial_skip_reason
  in both SwarmCommand::Dial arms + seed path -- skip target-is-self,
  skip already-connected peer, skip own-socket addresses (OwnSockets:
  normalized IPs, TCP/UDP port separation, interface IPs, circuit forms
  excluded, trusted Wi-Fi Aware dials exempt); skipped dials reply
  Err("skipped:") and the CLI ledger releases claims neutrally.
- core/Cargo.toml + Cargo.lock: if-addrs native-only.
- cli/src/ledger.rs +54: complete_dial_skipped (neutral claim release).
- cli/src/main.rs +10: skipped:-prefix handling.

Root cause (from 3-node logs): every ~300s each node re-dialed poisoned
ledger entries attributing OUR OWN listeners to other peers; dials landed
on self (127.0.0.1/::1 + own ports), yamux 10053 closes at :50, AutoNAT
probe dials dead-marked the peer at :15. #273 fixed the dead-mark side;
#274 kills the self-dial + connected-peer redial at the dispatch point.

## Audit verdict (merge-execution seat, verified this session)

- Rule-8: APPROVE on file (qwen lane, R1 REQUEST_CHANGES -> R4 APPROVE at
  6764e2b0 = current head; V040_NIMBLE_RECYCLE_REVIEW_QWEN_2026-09-03.md).
- Lineage vs candidate (44fee3c4): merge-base = 177bd840; the branch sits
  on d82978ab (pre-squash #273 head) whose TREE is byte-identical to the
  merged squash 44fee3c4. "44fee3c4 not an ancestor" is squash topology,
  not content divergence. git merge-tree (44fee3c4, branch) exit = 0.
  Merging applies exactly the 5-file #274 delta; #273 is NOT re-applied.
- Local gates (CTO, Windows host @ 6764e2b0): core 1403/0, cli 83/0, wasm
  check PASS, clippy -D warnings clean, fmt clean.

## Gates before merge

1. #274 PR CI fully green at 6764e2b0. At request time: iOS Build &
   Simulator Test IN_PROGRESS, macOS Native Tests IN_PROGRESS, iOS Build
   QUEUED -- NOT yet fully green (state UNSTABLE). Required 4 checks:
   SUCCESS so far.
2. CEO APPROVE on this decision line.
3. Merge: gh pr merge 274 --squash into cto/v040-candidate-2026-09-02.
4. After merge: candidate head moves -> rebuild EXACT artifacts at the new
   head (docker-publish workflow_dispatch at the candidate branch; wincli/
   APK per the update-branch path that yields exact shipped-tree builds),
   then the focused 3-node live window (recycle gone: no :50 closes, no
   :15 dead-marks, custody drains while connected).

## AWS leg (still pending, CTO's lane)

Latest docker-publish at the candidate branch = 177bd840 image (06:47Z).
No run at 44fee3c4; no redeploy evidence newer than the 177bd840 round.
The exact image for the final tree must be built AFTER #274 lands (head
moves), then elevation-redeploy i-0b735c4f26aea42ed -- reuse, never
terminate (IAM guard).
## 2026-09-03 23:21Z -- READY-TO-EXECUTE PACK (pre-flighted, CEO go still required)

PR #274 is now FULLY GREEN: 0 failed / 0 pending checks (iOS Build
completed SUCCESS 23:20Z). Merge gates pre-flighted at head 6764e2b0:

- a) tip vs live: MATCH (6764e2b0)
- b) diff vs candidate (cto/v040-candidate-2026-09-02 = 44fee3c4):
  present (mergeable) -- exactly the 5-file #274 delta (+504/-10)
- c) cherry (commits not in candidate): 6
- d) merge-tree (candidate, branch): exit 0 -- no conflicts
- e) PR state: OPEN, mergeStateStatus CLEAN
- f) CI: failed=0 pending=0

Rule-8 APPROVE on file (R4 @ 6764e2b0, V040_NIMBLE_RECYCLE_REVIEW_QWEN_2026-09-03.md).
Lineage audit (merge-execution seat): branch sits on d82978ab whose tree
is byte-identical to merged candidate 44fee3c4; merging applies exactly
the #274 delta. Merge command when CEO approves:
  gh pr merge 274 --squash
Then: candidate head moves -> CTO builds exact image at the new head
(docker-publish workflow_dispatch) + elevation-redeploy i-0b735c4f26aea42ed
(reuse, never terminate) -> wincli/APK exact artifacts via post-update PR
CI -> focused 3-node live window -> #272 to main.
