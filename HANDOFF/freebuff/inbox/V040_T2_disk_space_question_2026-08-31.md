Task: V040_T2_UNIFY_PEER_LEDGER_STORES.md (blocked by disk, needs a ruling)
Type: QUESTION
Date: 2026-08-31

## What happened

The T2 build hit `error: failed to build archive ... There is not enough
space on the disk. (os error 112)` (C: is 237G, was 51M free). The earlier
LNK1318 linker errors were the same root cause, not the toolchain.

## What I already cleared -- 100% mine, no approval needed (already done)

- `rm -rf scm-t1-boot-seed-dial/target` (my own parked worktree's build
  output; AGENTS.md rule 12 exempts target/) -- freed 5.1 GB. C: now 5.1G
  free, 98%.

This is enough for the current core test binary, but NOT enough to also run
the CLI tests, `cargo test --workspace`, and clippy for T2's gates. I need
roughly 4-8 GB more headroom.

## The one big lever -- NOT safe for me to take unilaterally

`SCMessenger/target` (the shared checkout's build tree) is **41,128 MB**
(41 GB -- by far the largest consumer on C:). Every other candidate is empty
(scm-mailbox, scm-secutils, scm-t5-docs-sync have no target dir).

Why it is not a 100%-safe delete by me right now:

1. The always-on local node is **running from that build tree right now**:
   `tasklist` -> `scmessenger-cli.exe` PID 2456; `/health` on 127.0.0.1:9876
   returns `{"status":"healthy"}`. Deleting under a live process hits Windows
   file locks and leaves a half-gone tree under a running node.
2. The Windows host serializes builds; another lane's gate (Android/iOS/core)
   may be mid-flight in that same target -- a delete would corrupt their
   cargo lock state and fail their run.

The delete is 100% reversible (source is untouched; target/ is pure rebuild
output), but 100% *safe* only under both conditions being true:
(a) the node (PID 2456) is stopped by whoever owns it, and
(b) no build gate is in flight on the Windows host.

## What I need from the CEO seat (pick one)

A. Authorize the operator to stop the node + confirm no build in flight, then
   tell me to delete `SCMessenger/target` (41 GB). I will `rm -rf` it and
   report the freed space + disk state. Rebuild cost for the next gate
   runner: a couple of hours.
B. Tell me there is a build in flight / the node must stay up -- I will then
   shrink my own `scm-t2-unify-ledgers/target` via partial `cargo clean` to
   fit the remaining T2 gates in the ~5 GB I have, and flag any gate I
   cannot run as UNVERIFIED rather than claim it.
C. Another path you prefer (e.g., a different disk to build on).

Default if no ruling arrives within one paste cycle: B (least impact, zero
risk to the node or other lanes), with gates that do not fit reported
UNVERIFIED -- never claimed passed.

## Evidence run this turn

- `df -h /c` -> `C: 237G 237G 51M 100%` (before) -> `C: 237G 232G 5.1G 98%`
  (after clearing my own target)
- `du -sm SCMessenger/target` -> 41128 MB; `scm-t2-unify-ledgers/target` ->
  3631 MB (still needed for the in-flight build)
- `tasklist` -> `scmessenger-cli.exe 2456`; GET /health -> `{"status":"healthy"}`