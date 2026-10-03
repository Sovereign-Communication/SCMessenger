# Android signing lineage: why the Pixel cannot take the 0.4.0 release build

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-10-03
**Author:** Buffy (Freebuff session), 0.4.0 rollout closeout
**Supersedes the diagnosis in:** `HANDOFF/audit/D2_KEYSTORE_VERIFICATION_2026-09-13.md`
(its decision table assumed the alias was wrong; the keystore proves it is not)
**Question:** why does `INSTALL_FAILED_UPDATE_INCOMPATIBLE` block the Pixel 6a,
and what exactly is missing before it can be unblocked?

## Short answer

Two independent faults, previously conflated into one:

1. **CI cannot produce a signed release APK.** The signing preflight fails. The
   cause was misreported for the entire history of the workflow (see below).
2. **Even a correctly signed release APK could not install over the Pixel's
   current app**, because the device is on a *different signing lineage*
   entirely. This is the part that was not previously identified, and it means
   fixing (1) alone does not complete the rollout.

## Fault 1 — the preflight reported a cause it had not verified

`release.yml` decoded the keystore and then ran a check whose output was
discarded, asserting a fixed conclusion:

```sh
keytool -list ... -alias "$ALIAS" >/dev/null 2>&1 \
  || { echo "::error::SCMESSENGER_KEY_ALIAS is not present in the decoded
keystore"; exit 1; }
```

`keytool` exits non-zero for a wrong **store password** and for a wrong
**alias**. The two are indistinguishable once stderr is thrown away, so the
message named a cause that had not been established. Reproduced locally:

| Fault | keytool output |
|---|---|
| wrong store password | `java.io.IOException: Keystore was tampered with, or password was incorrect` |
| wrong alias | `java.lang.Exception: Alias <X> does not exist` |
| truncated keystore | `java.io.EOFException` |
| empty file | `Keystore file exists, but is empty` |

Consequence: every release run in project history
(`37101943986`, `34996353889`, `34932614680`, `34789843685`, `32817839477`,
`32801758551`) failed with one unreadable message, and the recorded
follow-up actions all pointed at `SCMESSENGER_KEY_ALIAS`.

**The alias is not wrong.** Read directly from the JKS plaintext header
(aliases are unencrypted in JKS v2):

```
alias_len_field = 11
alias_bytes     = b'scmsessenger'
IS lowercase scmessenger: True
```

This matches `docs/ANDROID_RELEASE_SIGNING.md` exactly, including case.
`~/kiee/scmessenger-release.jks` is a genuine JKS v2 (magic `feedfeed`,
2266 bytes) and decodes byte-identical to `~/kiee/scmessenger-release.b64`,
the upload source for `SCMESSENGER_KEYSTORE_BASE64`.

**Therefore the outstanding secret is the password, not the alias.**

## Fault 2 — the device is on a third signing lineage

`apksigner verify --print-certs` against each APK:

| Source | SHA-1 signer digest | Key identity |
|---|---|---|
| App installed on the Pixel 6a | `46efdb19f328711880e94f2fdffceba1c1f9b7b1` | throwaway debug |
| `~/.android/debug.keystore` (this machine) | `8044d2844a176bc4cc3e6a6e9394f80072ef7e3f` | debug |
| CI `android-debug-apk` artifact (run `37101943986`) | `9ba2cbed6c6987bf9d05a9a23b7753e7e03681dd` | debug |
| `~/kiee/scmessenger-release.jks` | `f5fabd43a6788007eca3b1eae465896886661a4d` | release, `CN=Lucas Ballek` |

All four differ. Two consequences:

- A CI **debug** APK cannot install over the device build (lineage B vs A).
  This is the failure already recorded in
  `HANDOFF/todo/ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md`.
- A CI **release** APK cannot install over it either (lineage D vs A).

`dumpsys package` shows `firstInstallTime == lastUpdateTime ==
2026-09-30 17:38:48`, i.e. the app was **uninstalled and re-sideloaded**, not
updated. That is why the device key differs from the `8044d284` recorded in
the 2026-08-09 ticket: the install path changed since then.

So the device currently carries a build from a lineage that exists nowhere
else and is not reproducible from this repository.

## Operator checklist to unblock

Secrets required (only these two are in question; the alias is correct):

```bash
gh secret set SCMESSENGER_KEYSTORE_PASSWORD -R Sovereign-Communication/SCMessenger
gh secret set SCMESSENGER_KEY_PASSWORD        -R Sovereign-Communication/SCMessenger
```

`SCMESSENGER_KEYSTORE_BASE64` and `SCMESSENGER_KEY_ALIAS` are already set and
already correct. After the passwords are set, re-dispatch the release pipeline
and the preflight should pass; the new step will say precisely which secret is
at fault if it does not.

### The identity-loss tradeoff — this decision is the operator's

Installing a **release-signed** build over the device's current
**debug-signed** build is not an in-place upgrade. Android refuses it
(`INSTALL_FAILED_UPDATE_INCOMPATIBLE`), and the only ways forward both cost
data:

| Option | Data on device | Cost |
|---|---|---|
| `adb uninstall` then install release APK | **destroyed** — `contacts.db`, `history.db`, `ledger.json`, identity key | the Pixel re-registers as a new node; its ledger and peer trust are gone |
| Keep debug lineage, sign CI with a pinned debug keystore | preserved | the fleet stays on a throwaway key; not a distribution story |

The device holds real node state (`files/db` 524,287 B, `files/history.db`,
`files/ledger.json` 12,564 B, `relay_network_key.pb`). **Nothing was installed,
uninstalled, or modified on the device during this investigation** — verified
by unchanged `firstInstallTime`/`lastUpdateTime` and intact state files.

`docs/ANDROID_RELEASE_SIGNING.md` already reaches this conclusion ("Moving the
fleet from debug-signed to release-signed already requires an uninstall-and-
reinstall on every test device"); this document supplies the measurement that
makes the choice concrete.

## F-2 — the Pixel cannot corroborate the tagged commit

The device reports `versionName=0.4.0`, `versionCode=15`. It is **not** the
tagged build: the release job for `58c8970b` never produced a signed APK, and
the device's build predates the AND-SS-001 fix (`30caacc3`, 2026-10-02) by
three days. Its mesh log contains no `lifecycleMutex` evidence.

The Pixel is a live third log source for topology and health, but it cannot
corroborate version/commit against Windows and AWS. See
`HANDOFF/audit/ANDROID_RELEASE_SIGNING_LINEAGE_2026-10-03.md` companion
`V040_3NODE_ROLLOUT_STATE_2026-10-03.md` for the three-way correlation.
