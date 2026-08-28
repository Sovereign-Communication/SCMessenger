# Apple node install packet -- antigravity lane, INSTALL ONLY

**Status**: Ready to hand to the antigravity lane once `v0.4.0-rc.1` exists
**Date**: 2026-08-24
**From**: Windows CTO seat
**To**: Operator, for dispatch to the antigravity (Gemini) lane on the MacBook
**Coordination ID**: `AW-BILAT-0003` step 5
**Precondition**: the tag `v0.4.0-rc.1` is cut and its draft release carries the
Apple artifacts. **This packet is inert until then.** Do not dispatch it early.

---

## 0. Read this to the agent first, verbatim

> You are an INSTALLER, not a developer. Your entire job is to put an
> already-built, already-verified artifact onto this machine and prove which
> artifact it is. You are not here to improve anything.
>
> **You must not modify a single file in the repository.** Not source, not
> config, not a Gradle or Xcode setting, not a lockfile, not formatting, not a
> comment. Not even to fix an obvious error.
>
> If the build fails, that is a RESULT, not a task. Stop, capture the error,
> and report it. Do not fix it. Do not work around it. Do not try a different
> branch, a different toolchain version, or a different flag.

**Why this rule is absolute:** the entire premise of this gate is that every
node runs one identical frozen commit. An agent that quietly fixes a compile
error has created a fifth variant of the software and invalidated every result
the run produces -- and nobody would know, because the fix looks like success.
A failed install reported honestly costs one hour. A silent divergence costs
the whole gate and all confidence in it.

---

## 1. Absolute prohibitions

The agent MUST NOT:

1. Edit, create, or delete any file inside the repository.
2. Run `git commit`, `git push`, `git checkout -b`, `git merge`, `git rebase`,
   `git stash`, `git reset`, or `git clean`.
3. Run any formatter, linter, or codegen that rewrites files (`cargo fmt`,
   `swiftformat`, `swiftlint --fix`, `gradlew ktlintFormat`, or any xcodebuild
   invocation that writes settings back into the project).
4. Change Xcode project settings, signing identity, team ID, or build
   configuration to "make it work".
5. Upgrade, downgrade, or install toolchains, SDKs, or dependencies.
6. Modify `Cargo.lock`, `Package.resolved`, or any lockfile.
7. Retry a failed build with different flags.
8. Continue past any tripwire in section 4.

The agent MAY: check out the tag read-only, build, install, run read-only
verification commands, read logs, and report.

---

## 2. The tripwire -- run this before AND after every step

    git -C <repo> status --porcelain -- "*.rs" "*.kt" "*.kts" "*.swift" \
        "*.toml" "*.py" "*.sh" "*.gradle" "*.pbxproj" "*.plist"

**This must print NOTHING, every single time.** Empty output is the pass
condition.

If it ever prints a line, SOURCE has been modified. STOP IMMEDIATELY. Do not
attempt to undo it -- `git checkout --` and `git reset` have destroyed other
sessions' work on this project before. Report exactly what it printed and
wait for a human.

Capture this command at the start, after checkout, after build, and after
install. Four captures, all empty, are part of the evidence.

### KNOWN AND EXPECTED: dirty `.md` files. Do not "fix" them.

A plain `git status` in this repo reports roughly 110-190 modified `.md`
files in EVERY checkout and EVERY worktree, on a completely fresh clone.
This is expected and is NOT your doing.

Cause: those files are committed with CRLF line endings while
`.gitattributes` declares `*.md text eol=lf`, so git normalizes them to LF on
checkout and the working tree permanently disagrees with the stored blobs.
Verified 2026-08-24: `git diff --ignore-cr-at-eol` reports
**0 insertions, 0 deletions** -- there is no content difference at all, and
**no source file is affected**.

Therefore:

- The tripwire above deliberately filters to SOURCE extensions. Use it as
  written. Do not substitute a bare `git status --porcelain`.
- Do NOT run `git checkout`, `git restore`, `git stash`, or
  `git add --renormalize` to "clean up" those `.md` files. A repo-wide
  renormalization is scheduled as a separate change after the tag and would
  conflict with it.
- If a `git merge` refuses with "Your local changes to the following files
  would be overwritten", and every named file is a `.md`, that is this same
  issue. You do not need to merge anything for an install -- you check out a
  tag. Report it and stop.

---

## 3. Install procedure

### 3.1 Anchor to the tag

    git -C <repo> fetch --tags origin
    git -C <repo> status --porcelain
    git -C <repo> checkout --detach v0.4.0-rc.1
    git -C <repo> rev-parse HEAD

Record the SHA. **Every node in this gate must report this same SHA.** If it
differs from the SHA the CTO published with the go, stop and report.

Detached HEAD is correct and intended. Do not create a branch.

### 3.2 Build

Use the project's own documented build path for each target -- do not invent
one, and do not add flags that are not already in the repo scripts or the CI
workflow. If the repo has a script for it, use the script.

- **N5a, macOS CLI**: build the release CLI binary.
- **N5b, iOS app**: build and install to the device or simulator using the
  existing iOS build path.

Prefer downloading the artifact from the draft release over building locally,
if an equivalent signed artifact exists there -- a downloaded artifact is
provably the CI-built one, which is stronger evidence than a local rebuild.
State which of the two you did.

### 3.3 Capture per-node identity

For each installed node, capture and report:

- the git SHA the node reports at RUNTIME (not just the checkout SHA)
- the node PeerId / identity fingerprint
- its listening addresses
- the exact artifact used: local build, or release asset filename + checksum

---

## 4. Abort conditions -- stop and report, do not resolve

Stop immediately if ANY of these occur:

1. `git status --porcelain` prints anything at any point.
2. `git rev-parse HEAD` does not match the CTO-published tag SHA.
3. Any build or install command exits non-zero.
4. A code-signing, provisioning, or entitlement prompt appears that would
   require changing project settings.
5. A toolchain or SDK version mismatch is reported.
6. The runtime SHA the node reports does not match the checkout SHA.

For each, report the exact command and the exact output, and nothing else.
No diagnosis, no proposed fix, no attempt. Diagnosis is the CTO job, and an
agent guess actively contaminates it.

---

## 5. Log pulling and artifact exchange -- operator directive 2026-08-24

There are two channels and they carry different things. Do not mix them.

### 5.1 Full logs: SCMessenger itself is PRIMARY

Full, unredacted logs move between nodes **over SCMessenger**. This is
deliberate, and it doubles as the most honest product test we have: if the
messenger cannot move its own diagnostic payloads between the operator
machines, that is a genuine product finding and must be recorded as one --
not quietly worked around with a USB cable.

Rules:

- Full logs are exchanged node-to-node over SCMessenger.
- If SCMessenger CANNOT carry them -- size limit, transport failure, delivery
  never confirmed -- **that is a FINDING, log it as one** with the failure
  mode, then fall back to a direct file copy and clearly mark that leg as
  "fallback used, SCMessenger delivery failed" in the manifest.
- A fallback used silently is exactly the failure mode this gate exists to
  catch. Never let it pass unrecorded.

### 5.2 Redacted summaries: PR and HANDOFF

PRs and `HANDOFF/` documents carry **only redacted or summarized** results.

**Never commit raw logs to the repository.** Raw node logs contain PeerIds, IP
addresses, multiaddrs, timing, and identity material. A public repo makes
every one of those permanent and searchable.

What belongs in a PR or handoff doc:

- pass/fail per criterion
- counts, durations, rates
- error classes and fingerprint strings (for example `swarm_event_loop_died`)
- truncated identifiers only: first 8 characters of a PeerId, never the full
  value; never a public IP for a node that is still live
- quoted excerpts of failure output with identifiers redacted

If a reviewer needs the full log to judge something, the full log goes over
SCMessenger to that reviewer -- not into the PR.

---

## 6. What to report back

    CHECKOUT SHA:        <sha>   (must equal the published tag SHA)
    STATUS TRIPWIRE:     4 captures, all empty  Y/N -- paste them
    ARTIFACT SOURCE:     release asset <name> + checksum | local build
    N5a macOS CLI:       INSTALLED | FAILED -- <exact command + exact output>
      runtime SHA:       <sha>
      PeerId:            <id>
      listeners:         <addrs>
    N5b iOS app:         INSTALLED | FAILED -- <exact command + exact output>
      runtime SHA:       <sha>
      PeerId:            <id>
    FILES MODIFIED:      MUST be "none". Anything else is an abort.
    ABORTS HIT:          <list, or none>

Any claim without pasted command output is not accepted. "It built fine" is
not a result; the build output is the result.
