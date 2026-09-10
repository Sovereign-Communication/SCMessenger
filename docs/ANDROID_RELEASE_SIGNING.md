# Android Release Signing Setup

Status: Ready to configure (release.yml already wired, needs one-time keystore setup)
Last updated: 2026-08-31 -- storetype corrected to PKCS12, local verification step added

## Why This Matters -- Read Before Generating Anything

Whatever key signs the **first** release uploaded to a given Play Store app
listing is the **only** key that can ever sign updates to that listing.
There is no recovery path from Google if it's lost -- you cannot re-verify
ownership and get a new key issued for an existing listing. Losing this
keystore means the app can never be updated again under this package name
(`com.scmessenger.android`); the only fallback is publishing as a brand-new
listing, which throws away all reviews/install history/ratings.

**Before generating the keystore, decide your backup plan.** At minimum:
- The `.jks` file backed up in two places you control (e.g. a password
  manager's file-attachment feature, plus an encrypted drive/backup)
- The store password, key alias, and key password saved in a password
  manager -- NOT in a plain text file, NOT in this repo, NOT pasted into
  any chat (including this one)

## Regenerating Is Free Right Now -- And Only Right Now

**No Android APK has ever been published from this repo.** Verified 2026-08-31:
every public release (`v0.1.0`, `v0.1.1`, `v0.1.9`, `v0.2.1`) carries CLI
binaries only -- `scm-linux-amd64`, `scm-macos-amd64`, `scm-macos-arm64`,
`scm-windows-amd64.exe`. Zero Android assets, ever.

```bash
gh release view v0.1.9 --json assets --jq '.assets[].name'
```

That means there is **no signing lineage to preserve**. Generating a fresh
keystore today costs nothing. The moment a signed APK reaches a real user, that
key is locked in for the life of the app: a different key cannot upgrade an
install in place, so every user would have to uninstall -- destroying their
identity, contacts and message history, which for this product is the whole of
their data.

So if there is any doubt about the current keystore -- where it is, what its
alias is, whether it is backed up -- **regenerate it before the v0.4.0 tag.**
Archaeology on an unverifiable keystore is more expensive than a new one.

**The test fleet is not an argument against regenerating.** The Pixel 6a
currently runs a *debug*-signed build (see
`HANDOFF/todo/ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md`),
and D4/D6/D7 must be scored on the *released* APK. Moving the fleet from
debug-signed to release-signed already requires an uninstall-and-reinstall on
every test device. Regenerating the release keystore adds **zero incremental
cost** to that transition, as long as it happens before the fleet migrates.

## Step 1: Generate the Keystore (run this yourself, not through an agent)

```bash
keytool -genkeypair -v \
  -keystore scmessenger-release.jks \
  -alias scmessenger \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storetype PKCS12
```

**Use PKCS12, not JKS.** Earlier revisions of this document said
`-storetype JKS`. Modern `keytool` (JDK 9+) treats JKS as a legacy format and
warns on it, and the two differ in a way that has already cost this project a
failed release: **PKCS12 looks up the alias case-sensitively, where JKS did
not.** An alias that "looks right" but differs in case fails at
`:app:packageRelease` roughly 24 minutes into the release build with
`KeytoolException: No key with alias '***' found in keystore` -- exactly how the
`v0.4.0-rc.1` release build failed (run `32817839477`).

Pick a lowercase alias with no spaces, and record it verbatim.

(The `.jks` file extension below is only a filename -- the format is whatever
`-storetype` says. Keeping the name avoids churn in the docs and CI that already
reference it.)

`keytool` will interactively prompt for:
- A keystore password (this becomes `SCMESSENGER_KEYSTORE_PASSWORD`)
- Your name/org details for the certificate (cosmetic, shown in the cert, not security-sensitive)
- A key password (press Enter to reuse the keystore password, or set a
  separate one -- this becomes `SCMESSENGER_KEY_PASSWORD`)

`-validity 10000` is roughly 27 years -- Play Store requires the signing
cert to remain valid through the year 2033 minimum; this comfortably clears
that with margin.

The alias `scmessenger` becomes `SCMESSENGER_KEY_ALIAS` below (change it if you
prefer, just keep it consistent with what you set as the secret -- **including
its case**, per the PKCS12 note above).

**Immediately back up `scmessenger-release.jks`** (see the note above)
before doing anything else with it.

## Step 2: Base64-Encode the Keystore

```bash
# Windows (PowerShell):
[Convert]::ToBase64String([IO.File]::ReadAllBytes("scmessenger-release.jks")) | Set-Content -NoNewline scmessenger-release.b64

# macOS/Linux:
base64 -w0 scmessenger-release.jks > scmessenger-release.b64
```

## Step 2.5: Verify the Keystore Locally BEFORE Setting Any Secret

Do not skip this. It is the step whose absence cost this project the
`v0.4.0-rc.1` release build.

```bash
scripts/verify_release_keystore.sh scmessenger-release.jks scmessenger
```

It prompts once for the store password (never echoed, never written to disk,
never placed on the command line where shell history would capture it), then
runs the **exact** alias check that `release.yml` runs in CI. On failure it
lists the aliases actually present so you can compare their case. On success it
prints the certificate SHA-1/SHA-256 fingerprints.

**Record the SHA-256 fingerprint in your password manager next to the keystore.**
It is how you later prove a given APK came from this key, and how you would
detect a silent key swap.

Only once this prints `[OK]` should you continue.

## Step 3: Set the 4 GitHub Repo Secrets

Using `gh` (run these yourself from a terminal where you can see the
values momentarily -- avoid pasting the actual password strings into any
chat, including this one):

```bash
gh secret set SCMESSENGER_KEYSTORE_BASE64 < scmessenger-release.b64
gh secret set SCMESSENGER_KEYSTORE_PASSWORD   # paste the keystore password when prompted
gh secret set SCMESSENGER_KEY_ALIAS           # paste "scmessenger" (or your chosen alias)
gh secret set SCMESSENGER_KEY_PASSWORD        # paste the key password
```

`gh secret set NAME` (no `<`) prompts interactively and reads from stdin
without echoing it to the terminal history -- safer than putting the value
directly on the command line where it could land in shell history.

Or via the GitHub web UI: repo -> Settings -> Secrets and variables ->
Actions -> New repository secret, same 4 names.

## Step 4: Delete the Local Plaintext Copies

Once the secrets are set:

```bash
rm scmessenger-release.b64
# Keep scmessenger-release.jks itself -- that's your backup copy, just
# make sure it's also saved somewhere OTHER than this working directory
# (password manager attachment, encrypted external backup, etc.)
```

## What Happens Next

Once all 4 secrets exist, the next `v*` tag push triggers `release.yml`'s
`build-android` job to also produce:
- `android/app/build/outputs/bundle/release/*.aab` -- upload this to Play
  Console (internal testing track, then production when ready)
- `android/app/build/outputs/apk/release/*.apk` -- a signed APK, useful for
  direct/sideload distribution (e.g. handing a build straight to an alpha
  tester without waiting on Play Store review)

Both get attached to the resulting GitHub Release automatically (the
`create-release` job's file glob already includes `**/*.aab` and
`**/*.apk`).

If the secrets are NOT set, `build-android` still succeeds -- it just
produces the debug APK only, same as it does today. The signed-build steps
are conditional (`if: secrets.SCMESSENGER_KEYSTORE_BASE64 != ''`), so
there's no way to accidentally break CI by not having gotten to this setup
yet.

## Play Store Upload (Manual, By Design)

Per the "CI builds a signed AAB, you upload manually" choice: this repo
does NOT auto-publish to Play Store. You download the `.aab` from the
GitHub Release and upload it to Play Console yourself each time. This
keeps Play Store publishing credentials (a service account JSON with
publish rights) out of GitHub Secrets entirely, and keeps a manual human
gate before anything reaches real users via the Play Store.
