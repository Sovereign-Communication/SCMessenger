# Reconciliation and 3-node rollout runbook

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-10-04
Branch: `integrate/train-20261004` @ `200bfc54e`
Answering: "reconcile all the issues and get us rolled out for a 3-node test to
verify the changes once they have landed."

## Bottom line

The reconciliation is done and is written up. **The rollout cannot start yet,
and three of the four gates are operator-held.** The single most important thing
in this document is finding 2: the Pixel signing blocker is *not* the one the
prior checkpoint described, and the recorded remedy for it will not work.

---

## 1. What was reconciled in this pass

### 1.1 Quantum / PQC tracking (complete)

| Item | Action |
|---|---|
| `docs/QUANTUM_READINESS_AUDIT.md` carried 3 false claims | **Corrected in place** (docs-only, no Rule-8 exposure) |
| T2 (CRITICAL) had no ticket | **Created** `HANDOFF/todo/CRYPTO_T2_HYBRID_SEND_PATH_UNREACHABLE.md` |
| T4 (CRITICAL) had no ticket | **Created** `HANDOFF/todo/CRYPTO_T4_ENCRYPT_AT_REST.md` |
| PQC-09 marked `[DONE]` with no deliverable | **Annotated** (additive; nothing removed) |
| Review tracking ledger | `HANDOFF/review/QUANTUM_REVIEW_TRACKING_AUDIT_2026-10-04.md` |

The doc corrections: the remediation table no longer marks F1/F2/F3 CLOSED, the
"no post-quantum primitives anywhere in the workspace" claim is marked
superseded (ML-KEM and ML-DSA are in `Cargo.toml`), and Residual Exposure 1 --
which asserted payloads were protected by ML-DSA signatures and hybrid ratchet
encryption -- now states plainly that **no production traffic has PQ
protection today** and names what a send actually uses.

### 1.2 Merge train: two of the seven Wave A PRs must NOT be merged

This is a finding, not a status update. Merging them would cause a regression.

- `git diff --stat <#316 head> integrate/train-20261004 -- HANDOFF/freebuff/`
  reports **+2481 insertions**. The train is a strict superset of #316.
- #316 and #357 fail the required `Handoff ownership scope` check. The job log
  names three files with `found 0 begin markers`:
  `V040_OUTBOX_RETRY_DIAGNOSIS_2026-09-19.md`,
  `TRAIN_STATUS_2026-09-21.md`, `AND06_KOTLIN_CUTOVER_PREREQUISITE_2026-09-21.md`.
- Those three files **do not exist on `origin/main` at all**, and the **train
  already carries metadata-fixed versions of all three** (1 marker each).

So the PR branches hold pre-fix versions of files the train has already fixed.
Merging #316/#357 into `main` first would land the metadata-less copies, and the
train merge afterwards would then have to undo them. **Close #316 and #357 as
superseded by the train rather than merging them.**

#376, #386, #415, #425 are `CLEAN` (0 failing, 0 pending of 22 checks) but are
likewise very likely already contained in the train. Merging them is harmless
but probably unnecessary -- confirm with `pr_scope.sh` before spending merges on
them. #388 has `Analyze (rust)` still in flight.

### 1.3 The train itself

```
#451  head=200bfc54e  base=main  mergeable=MERGEABLE  state=BLOCKED
```

Of the 5 required contexts: `Repository Hygiene Checks` SUCCESS, `Lint`
SUCCESS, `Rust Linting` SUCCESS, `Handoff ownership scope` SUCCESS.
`Test (ubuntu-latest)` IN_PROGRESS. Twenty other checks are green or in flight.

```
git rev-list --count origin/main..integrate/train-20261004   -> 211
git rev-list --count integrate/train-20261004..origin/main   -> 0
git merge-base --is-ancestor integrate/train-20261004 origin/main -> NOT MERGED
```

**211 ahead, 0 behind.** `main` has nothing the train lacks, so this is a clean
fast-forward with no conflict resolution required. #451 is the only PR that
matters.

---

## 2. The Pixel signing blocker is different from the recorded one

The prior record says: set `SCMESSENGER_DEBUG_KEYSTORE_BASE64` and the CI APK
will install over the Pixel. **That remedy will not work.** Measured today:

```
# installed on the Pixel (pulled base.apk, apksigner verify --print-certs)
Signer #1 certificate SHA-256 digest:
  067d312c1470d9b36c39b54943e3d8c434243821c9874c03241e8668801fd672

# this machine's Android debug keystore
keytool -list -v -keystore ~/.android/debug.keystore -storepass android \
        -alias androiddebugkey
  SHA256: 1C:DE:F0:9C:D3:B8:0F:9B:68:6E:5F:9E:7B:76:0D:36:0B:1F:D3:38:C6:72:0B:CB:59:B1:5B:23:39:67:83:5F
```

**They do not match.** The Pixel app was not signed with this machine's
`debug.keystore`. Two further facts close off the easy explanations:

- `android/app/build.gradle:169-172` -- the `debug` build type sets **no**
  `signingConfig`, so a local `./gradlew assembleDebug` would produce
  `1cdef09c...`. A local build cannot produce `067d312c...` either.
- All five `release.yml` runs on record **failed**
  (`37116380941`, `37101943986`, `34996353889`, `34932614680`, `34789843685`),
  so there is no release APK artifact to compare against.

**Consequence: no artifact obtainable from this repository today can
`adb install -r` over the current Pixel install.** The key that signed it
(`067d312c...`) is not present here in readable form. A release keystore exists
at `/c/Users/SCM/kiee/scmessenger-release.jks` but is password-protected; its
certificate could not be read and **was not guessed at**. If that keystore's
certificate turns out to be `067d312c...`, then a *release* APK updates the
phone in place with no wipe -- but that is currently **UNVERIFIED**, and the
release path is separately blocked on the `SCMESSENGER_KEY_ALIAS` mismatch
documented in `HANDOFF/audit/ANDROID_SIGNING_LINEAGE_2026-10-03.md`.

### The three real options (operator decision)

| Option | Effect on the Pixel identity | Cost |
|---|---|---|
| A. Recover the `067d312c...` key | preserved | requires the operator to know where that key lives |
| B. Confirm `scmessenger-release.jks` is `067d312c...`, fix the alias secret, ship a **release** APK | preserved | needs the keystore password + correct alias; then also closes the release-blocker |
| C. `adb uninstall` then install a CI debug APK | **RESET** -- new node identity, ledger lost | zero operator effort |

Option B is the one worth 10 minutes of checking first, because it is the only
path that both preserves identity and unblocks the release pipeline.

---

## 3. Live three-node state, re-verified 2026-10-04

```
curl -s http://127.0.0.1:9876/health   -> {"status":"healthy"}
curl -s http://127.0.0.1:9876/version  -> 0.4.1  dcd67b94ecb4...  (provenance "v0.4.1:")
curl -s http://18.234.62.247:9876/health  -> {"status":"healthy"}
curl -s http://18.234.62.247:9876/version -> 0.4.1  dcd67b94ecb4...  (provenance "main:")
adb devices -> adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp   device
```

| Node | State | Verdict |
|---|---|---|
| Windows CLI | healthy, PID 26880 (from `~/.local/bin`, outside any `target/`) | PASS on `dcd67b94` |
| AWS cloud node | healthy, public `:9876` only; **no ssh key on this host** | PASS on version, identity UNVERIFIED |
| Pixel 6a | **reachable now** (was recorded offline), `0.4.0` vc15, PID 4917, `DEBUGGABLE` | BLOCKED on signing only |

The Pixel being back on the network is new since the PREFLIGHT checkpoint and is
good news -- option C above becomes available at any time.

**The critical sequencing fact:** both nodes run `dcd67b94`, which **is an
ancestor of the train head**. The 211 train commits are not on any node. A
three-node test performed right now would verify v0.4.1 again and prove nothing
about the changes. The rollout must follow the train.

---

## 4. The chain to a three-node test of the changes

Each step is gated by the one above it.

| # | Step | Gate | Who |
|---|---|---|---|
| 1 | `Test (ubuntu-latest)` on #451 goes green | in flight now | CI |
| 2 | #451 merges to `main` | **Rule-8 decision** -- 18 files under `core/src/{crypto,transport,routing,privacy}`, and the recorded independent verdict is BLOCK | operator |
| 3 | Tag the next release, release pipeline produces CLI + APK | 5/5 prior release runs failed; see below | CI + operator |
| 4 | Deploy CLI to Windows and AWS | needs the new artifact; AWS needs its deploy credential | operator / Windows lane |
| 5 | Install APK on Pixel | **finding 2** -- operator choice A/B/C | operator |
| 6 | Run the three-node matrix | all of the above | operator present |

Steps 4 and 5 must happen in that order only because both need a new build; they
are otherwise independent.

### Why step 2 is an operator decision and not mine

`HANDOFF/review/RULE8_OPUS_VERDICT_2026-10-04.md` records an independent
(non-author) **BLOCK**, with the HIGH finding since fixed in `a34c60017`. Per
`docs/rules/SECURITY_PROTOCOL.md:105`, a verdict is scoped to the commit
reviewed, and later commits touching those modules void it. The fix commit
`a34c60017` has **no** non-author review, and I authored it, so I am
disqualified from signing it off. `origin/main..train` touches
`core/src/crypto/encrypt.rs` and `core/src/routing/global.rs`.

Four routes, unchanged from `HANDOFF/review/RULE8_TRAIN_451_REVIEW_GAP_2026-10-04.md`:
fund an independent review of the current head; have a non-author human review
it; split the non-gated files out and merge those; or record an explicit
override.

### Why step 3 is not a formality

`release.yml` has failed 5 of 5 recorded runs. The v0.4.1 run `37116380941`
failed in `Build Android Release` preflight with keytool's own message:
`SCMESSENGER_KEY_ALIAS does not exist in the decoded keystore`. Until that is
fixed there is no signed release APK, so step 5 falls back to the debug path,
which per finding 2 cannot update the phone.

---

## 5. Operator actions, in the order they unblock the most

1. **Decide the Pixel signing route** (finding 2). Check whether
   `scmessenger-release.jks` is certificate `067d312c...` first -- if it is,
   option B fixes the phone *and* the release pipeline together.
2. **Rule-8 route for #451** (step 2). This gates everything downstream.
3. **Fix `SCMESSENGER_KEY_ALIAS`** so `release.yml` can produce a signed APK.
4. **Close #316 and #357 as superseded** -- do not merge them (section 1.2).
5. Optional, if #451's Rule-8 review needs a clean bill on the crypto fix:
   an independent reviewer needs to look at `a34c60017` specifically.

Nothing in the quantum/PQC set gates the three-node test. Per
`HANDOFF/review/QUANTUM_REVIEW_TRACKING_AUDIT_2026-10-04.md` section 3.2, the
production send path invokes neither ML-KEM nor ML-DSA, so a 3-node run will
validate confidentiality and delivery exactly as it would on a purely classical
build. The PQ tickets are security debt to schedule, not blockers to clear.

---

## 6. What this pass did NOT do

Stated plainly so nothing here is over-claimed:

- **No merge.** #451 and all Wave A PRs are untouched. Freebuff lane authority
  does not include merging.
- **No push.** Nothing was committed. All edits are in the working tree on
  `integrate/train-20261004`.
- **No node was started, stopped, installed, or messaged.** Every node
  interaction was read-only: `curl /health`, `/version`, `adb devices`,
  `dumpsys package`, `pidof`.
- **No device state was modified.** The APK pull is a read. The Pixel app was
  not stopped, not updated, not cleared.
- **No keystore password was guessed at**, and no release APK was forced.
- **No local build.** No `cargo`, no `gradlew`.
- **Rule-8 is not satisfied by this document.** It changes no file under
  `core/src/{crypto,transport,routing,privacy}`.
- **The `067d312c...` identity is unresolved.** Whether it is the release
  keystore is explicitly UNVERIFIED.

### Files changed by this pass

- `docs/QUANTUM_READINESS_AUDIT.md` (modified -- false claims corrected)
- `HANDOFF/todo/CRYPTO_T2_HYBRID_SEND_PATH_UNREACHABLE.md` (new)
- `HANDOFF/todo/CRYPTO_T4_ENCRYPT_AT_REST.md` (new)
- `HANDOFF/review/QUANTUM_REVIEW_TRACKING_AUDIT_2026-10-04.md` (new)
- `HANDOFF/IN_PROGRESS/PQC09_HYBRID_ONION_INVESTIGATION.md` (annotated, additive)
- `HANDOFF/review/RECONCILIATION_3NODE_RUNBOOK_2026-10-04.md` (new, this file)
- `tmp/pixel-cert/installed-base.apk` (temporary evidence, gitignored, reclaimable)