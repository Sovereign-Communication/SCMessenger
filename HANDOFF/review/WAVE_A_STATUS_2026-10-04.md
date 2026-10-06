# Merge train execution state (2026-10-04)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Owned by `Sovereign-Communication/SCMessenger` (this repository).

## Wave A: 7 documentation PRs, all gates passed, branches updated

`#316 #357 #376 #386 #388 #415 #425`

Every one was re-verified at execution time with `bash scripts/pr_scope.sh <n>`:
**0 blockers**, "no reasons not to merge were found". Each is documentation-only,
0 files under `core/src/{crypto,transport,routing,privacy}/`, and
`mergeable=MERGEABLE`.

### Why they are not merged yet

The first merge attempt failed on all 7. The real error, read from `gh`:

```
X Pull request ...#316 is not mergeable: the head branch is not up to
  date with the base branch.
```

That is `BEHIND`, not a conflict and not a red check. GitHub refuses an
immediate merge until the branch contains the current base. All 7 branches were
then updated with `gh pr update-branch`, which succeeded on every one. They now
show `mergeStateStatus=BLOCKED` only because a fresh CI run is in progress on
the updated branches.

**Action when those runs settle: re-run `bash tmp/merge_wave_a.sh`.** The script
re-verifies every prerequisite before merging, so it is safe to re-run.

### Correction to an earlier claim in this session

The first version of the merge script reported failures as "likely a required
context is not green". That was a guess made by a script that discarded
`gh`'s output. It was wrong: the contexts were green and the cause was the BEHIND
state. The script now captures and reports the real error. A failure line is
never a pass.

## PR #451: still must not merge

`bash scripts/pr_scope.sh 451` returns STOP with 5 reasons. Two are structural,
one is the standing rule:

- 207 commits. (False positive as a base error: `merge-base == origin/main` and
  `origin/main` IS an ancestor. It is flagged as an unreviewable size.)
- 18 files under the merge-blocked paths.
- "requires a crypto-security-auditor verdict before merge".
- Failing checks at the time of reading: `Test (macos-latest)`,
  `Test (ubuntu-latest)` -- these were the test-premise failures described below.

The independent Opus verdict is `BLOCK` with one HIGH latent finding, now fixed
on the branch by `a34c60017`. Remaining governance gap: that fix was authored by
the integration agent, so under `SECURITY_PROTOCOL.md:90` it still needs a
non-author review before merge. Detail:
`HANDOFF/review/RULE8_OPUS_VERDICT_2026-10-04.md`.

## CI history on the train, and two self-inflicted test failures

Recorded so the next session does not re-derive it:

1. `a34c60017` -- CI red on all three test platforms. Cause: my own new test
   asserted that a replayed envelope must fail to decrypt. It decrypted, because
   `get_message_key` (`ratchet.rs:1104-1106`) removes and returns the cached
   skipped key for an already-consumed message number. That is required so a
   duplicated delivery can still be decrypted once per key.
2. `1b2866812` -- CI red again on the same platforms, 1536 tests passing and one
   failing. Cause: the rewritten test still asserted a decrypt outcome, assuming
   advancing 300 messages would push a replay outside the servable window. Wrong
   again.
3. `0b9aa0c6c` -- current head. The test now asserts ONLY the property the
   production fix is about: a replayed delivery must not delete the peer session,
   and the contact must remain usable afterwards. The ratchet's serve-vs-reject
   decision is deliberately not asserted, because two attempts to pin it were
   both wrong.

**The production fix was never in question.** Both failures were in test
premises, not in `encrypt.rs` behaviour.

Not verified locally: a test-profile compile of `scmessenger-core` dies with
`rustc-LLVM ERROR: out of memory` on this host. CI is the only verifier here.

## Disk

Reclaimed and verified non-destructive. Freed 7.8 GB from the shared checkout's
`target/debug` after proving: preserve commit `e4050f84c` is byte-identical on
two remotes with all 3419 tracked files, zero uncommitted work in that checkout,
zero tracked or durable files under `target/`, and no process running from it.
The live node (PID 26880) runs from `C:\Users\SCM\.local\bin\`, outside every
`target/`, and was unaffected. A later 2.2 GB partial test build was reclaimed
the same way. Free space went 7.2 GB to 19 GB.

`Cargo.lock` was rewritten by that partial build (workspace version 0.4.0 to
0.4.1, matching `Cargo.toml`, no dependency change). Committed forward as
`a799e9041` rather than reverted, since I did not author the original state.

## Exact next actions

1. Re-run `bash tmp/merge_wave_a.sh` once the Wave A branch CI settles. Expect 7
   merges.
2. Operator routes Rule 8 for #451: fund a panel, human review, split the
   non-gated files, or record an explicit override.
3. Operator sets `SCMESSENGER_DEBUG_KEYSTORE_BASE64` per
   `docs/ANDROID_RELEASE_SIGNING.md` so a CI APK can install over the Pixel.
4. Then the three-node matrix for the 0.4.1 parity run.