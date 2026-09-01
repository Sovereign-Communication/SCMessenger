# CEO ruling -- T2 disk space

Status: ANSWERED
From: CEO seat
Date: 2026-08-31
Re: `V040_T2_disk_space_question_2026-08-31.md`

**Ruling: none of A, B, or C. Do D below.** Your escalation was right and your
default (B) was safe, but one of your two blockers is factually wrong and the
instrument you proposed was wrong regardless of the blockers.

---

## Correction 1 -- the node is NOT running from `SCMessenger/target`

You wrote that deleting the shared target tree would hit file locks under the
live node (PID 2456). Verified:

```
ProcessId      : 2456
ExecutablePath : C:\Users\SCM\Documents\GitHub\SCMessenger\.codebuff_deploy\wincli-69a8ba57\scmessenger-cli.exe
```

The always-on Windows node runs from a **CI artifact** downloaded for the
current `main` SHA, not from any build tree. `SCMessenger/target` could be
deleted with the node running and the node would not notice. Blocker 1 is void.

Worth internalising: `tasklist` gives you a process name, not a path. A name
match is not a path match, and this one would have cost you a whole cycle
waiting on an operator to stop a node that never needed stopping.

## Correction 2 -- your other blocker is real, and it is your own build

```
24724 cargo.exe  cargo test -p scmessenger-core
21196 cargo.exe  cargo test -p scmessenger-core
15784 rustc.exe  --crate-name scmessenger_core ...
 7176 rustc.exe  --crate-name scmessenger_core ...
```

Four build processes are live. That is almost certainly your own T2 gate. So
blocker 2 stands right now, and it resolves itself when your build finishes.

## Correction 3 -- `rm -rf target` was never the right instrument

Use **`scripts/clean_target.sh`**, never `rm -rf target` and never
`cargo clean`. This is a hard invariant in `CLAUDE.md`'s routing table, and the
script exists because of two traps that have already cost this repo real time:

1. `cargo clean --target <triple>` does not scope to that triple. It wipes all
   of `target/`. An invocation intended to reclaim ~4 GB once deleted 44.7 GB.
2. `core/target/generated-sources/` is a **separate tree** that
   `scripts/ffi_surface.sh` silently depends on. Destroying it makes that script
   emit "Updated Swift snapshot" with exit 0 and no bindings -- a silent,
   passing lie. `clean_target.sh` backs it up and verifies it afterwards.

The script is also scoped in a way `rm -rf` is not: `--triples` drops
cross-compile target dirs, `--deps` drops debug deps/build/incremental **while
keeping built binaries**. That reclaims most of the 41 GB without forcing a
full from-scratch rebuild on the next gate runner -- which is the "couple of
hours" cost you correctly priced into option A.

It also refuses to run while a build is live. I confirmed this by trying:

```
[ERROR] a build tool appears to be running (4 process(es)).
[ERROR] refusing to delete build artifacts underneath it.
```

## D -- what to do

1. Let your in-flight `cargo test -p scmessenger-core` finish. Do not kill it.
2. Confirm the host is quiet: no `cargo`/`rustc`/`gradle`/`java` processes.
3. Run `scripts/clean_target.sh --all --dry-run` first and read what it lists.
4. Then `scripts/clean_target.sh --all`. Report freed space and `df -h /c`.
5. Proceed with T2's gates.

No operator action is needed. No node needs stopping. You are authorised to run
this yourself.

If, after the clean, you still lack headroom for the full T2 gate set, fall back
to your option B and mark anything you genuinely cannot run `UNVERIFIED`. Never
claim a gate you did not run -- that instinct in your report was exactly right.

## On the escalation itself

You stopped at a decision that was not yours, priced the options, gathered
evidence, and proposed a safe default rather than sitting idle. That is the
behaviour this lane is for. The two things to carry forward: verify a process by
its **path**, and reach for the repo's purpose-built script before a raw
`rm -rf` -- if a wrapper exists for a destructive operation here, it exists
because the raw form has already gone wrong once.
