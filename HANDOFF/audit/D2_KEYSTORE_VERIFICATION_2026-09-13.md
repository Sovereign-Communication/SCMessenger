# D2 — Release keystore verification at main ccce98cf (evidence, not inference)

**Date:** 2026-09-13
**Author:** Buffy (Freebuff recovery session), Phase 4 of the approved plan
**Question:** does the release build actually sign, with which alias and
certificate? The D2 blocker (SHIP_PLAN / V040_3NODE_RCA chain) was an alias
mismatch; neither the stale claim nor its denial was trusted — everything
below was executed this session.

## A. What the release build reads (build config, verified live)

`android/app/build.gradle` `signingConfigs.release` (lines 97-129) resolves in
strict priority: (1) env `SCMESSENGER_KEYSTORE_PATH/_PASSWORD/_ALIAS/_PASSWORD`,
(2) legacy env names, (3) `android/keystore.properties` (gitignored, template
only), (4) ELSE prints `"Release signing is not configured; release tasks must
fail."` and leaves the config incomplete so AGP rejects release tasks — it
never substitutes the debug key. Verified live on this tree:

```
$ ./gradlew help
Release signing is not configured; release tasks must fail.
```

Release variant sets `signingConfig = signingConfigs.release`, R8 minify on,
`debuggable = false` (lines 155-167).

CI (`.github/workflows/release.yml`): decodes secret
`SCMESSENGER_KEYSTORE_BASE64` to `android/app/release.keystore` (step 12),
runs a keytool alias preflight ("Verify SCMESSENGER_KEY_ALIAS present in
keystore", fails the job with `::error::` on mismatch), then builds with the
four `SCMESSENGER_*` envs — the same env path as above.

## B. The keystore that exists (local, operator-created)

- File: `C:/Users/SCM/kiee/scmessenger-release.jks` — 2,266 bytes, mtime
  2026-08-15 07:05:07 (-10:00). Genuine JKS v2 (magic `fe ed fe ed`, version 2,
  1 entry).
- `C:/Users/SCM/kiee/scmessenger-release.b64` (3,024 chars) decodes
  **byte-identical** to the `.jks` — it is the exact upload source for the
  GitHub secret (matches `docs/ANDROID_RELEASE_SIGNING.md` line 101/104).
- Timeline closes: keystore created 2026-08-15T17:05:07Z (embedded creation
  timestamp) -> GitHub secrets set 17:07:21-17:08:01Z (`gh secret list`).
- Alias (from the JKS entry header, plaintext by format): **`scmessenger`**
  (u2-length 11 bytes; matches `setup_android_signing.sh` line 28).
- Certificate (extracted from the entry's X.509 chain — certificates are
  plaintext in JKS; only the private key is encrypted). The 908-byte DER carve
  was validated by an independent SSL parser (`ssl._ssl._test_decode_cert`),
  not just by my struct reading:
  - Subject = Issuer: C=Unknown, ST=Unknown, L=Unknown, O=Unknown, OU=Unknown,
    **CN=Lucas Ballek** (self-signed)
  - Validity: notBefore 2026-08-15T17:05:06Z, notAfter 2053-12-31T17:05:06Z
  - Serial: `9C20F627A64D1904`
  - **SHA-256: `8220e4cc70a5bae2846d88a1bd11d2c353daab18457bfd0ae4c9d915ec418251`**
  - SHA-1: `f5fabd43a6788007eca3b1eae465896886661a4d`
  - Carved DER archived at `tmp/review/scmessenger_release_cert.{der,pem}`
    (session-local; not committed).
- Store/key passwords: **NOT held by this lane.** No record found in the
  bounded sweep (`kiee/`, repo, Desktop, Documents, Downloads, SCM-Progress).
  keytool refuses full enumeration without them (JKS integrity check), which
  is why the certificate was extracted by carving + parser validation rather
  than `keytool -list`.

## C. The rc.1 failure, forensically (run 32817839477, 2026-08-25)

Job "Build Android Release" steps: `Decode release keystore` SUCCESS (step 12)
-> `Build signed release AAB + APK` FAILURE (step 13). The alias preflight
step **did not exist in that revision** — steps jump 12 -> 13 -> 14. Log:

```
> Task :app:packageRelease FAILED
com.android.ide.common.signing.KeytoolException: Failed to read key *** from
store ".../android/app/release.keystore": No key with alias '***' found in
keystore ...
```

(2026-08-25T07:40Z.) The preflight step and `scripts/verify_release_keystore.sh`
were the post-incident fixes (signing doc updated 2026-08-31). All four
SCMESSENGER_* secrets are timestamped 2026-08-15 and were never updated
(`gh secret list` — values are unreadable by GitHub design), so the condition
that failed rc.1 has not changed since. The keystore decoded fine on CI, so
the base64 secret is good; **the open doubt is exactly the alias (or key
password) secret value vs the keystore contents** — unprovable from here
without the store password, and provable by CI itself, which is what is
running now.

## D. The live experiment (decisive)

`release.yml` `workflow_dispatch` with `artifacts_only=true` builds checksummed
signed candidate artifacts **without creating a GitHub Release and without a
tag** — the pipeline's own safe mode. Dispatched 2026-09-13T23:29Z on `main`
(= ccce98cf):

**Run 34789843685** — https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34789843685

Decision table:
- Preflight step fails -> `SCMESSENGER_KEY_ALIAS` secret does not match the
  decoded keystore (whose true alias is `scmessenger`, section B). Fix is
  `gh secret set SCMESSENGER_KEY_ALIAS` = `scmessenger` exactly — an operator
  30-second action, then re-dispatch.
- Preflight passes, build fails at packageRelease on key password -> the
  `SCMESSENGER_KEY_PASSWORD` secret is wrong (JKS key password may differ
  from store password). Operator re-sets that secret from their records.
- Everything green -> **D2 verified end-to-end at the CI level**: alias,
  store password, and key password all resolve; signed APK/AAB artifacts are
  the Phase 6 package input.

## E. What this lane could NOT do (and did not improvise)

A locally signed release build requires the store/key passwords, which are
operator-held and were not found in any bounded location. No password was
guessed, cracked, reset, or bypassed; no keystore was regenerated (regenerating
is still free per the signing doc only because nothing was ever published —
but that decision is the operator's alone, per the doc's warning about the
first-signing lock-in).

## F. Documentation defect found (not blocking, worth one PR)

The on-disk keystore is genuine JKS while `docs/ANDROID_RELEASE_SIGNING.md`
(2026-08-31 correction) documents `-storetype PKCS12`. Gradle and keytool
autodetect the format from the magic bytes, so builds and the preflight are
unaffected — but anyone running the documented verify command with an explicit
`-storetype PKCS12` against this JKS gets a confusing failure. If the operator
confirms this JKS is the release key, the doc's format note should say
"format: JKS (autodetected); do not force -storetype".
