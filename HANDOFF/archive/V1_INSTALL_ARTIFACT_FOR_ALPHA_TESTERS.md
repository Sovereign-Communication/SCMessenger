# Produce a real, shareable install link (not a local debug build)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY -- waiting on CI green + Lucas go-ahead to tag
Filed: 2026-07-20

## Why this ticket exists

Everything verified so far (Lucas's CLI and local emulator connecting to the
alpha relay) used artifacts sitting on Lucas's own machine: a locally-built
`target/debug/scmessenger-cli.exe` and a locally-built debug APK
(`android/app/build/outputs/apk/debug/app-debug.apk`). Per
`HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md`, the plan for Josh today is either
"build from source" (Windows CLI) or receive the debug APK by hand/file
transfer. Neither of these is "a link." There is currently no hosted,
clickable download for either platform.

`release.yml` (295 lines, in `.github/workflows/`, "Multi-platform CLI binary
builds", triggers on tag push or manual dispatch per
`HANDOFF/GITHUB_CI_CD_AUDIT_FINDINGS.md` Section 2.1) already exists and,
per that same audit, is listed as "Active" -- but nothing in the HANDOFF
history this session shows it having been actually exercised for a real
release, and it's unclear whether it currently covers the Android APK at all
(the audit's inventory describes it as CLI-binary-focused; `mobile.yml`
separately builds the Android APK but that's a CI artifact, not a public
release asset).

## Goal

Lucas should be able to hand Josh (or any future alpha tester) a single URL
that resolves to: a Windows CLI executable, and an installable Android APK --
both built from a commit that has passed CI (see
`CI_RED_ON_MAIN_ALL_FEATURES.md` -- that should land first or in parallel,
since a release cut from a red build isn't trustworthy).

## Open questions -- ANSWERED (2026-07-20)

- **Does release.yml cover Android APK?** YES. `build-android` job builds a debug
  APK unconditionally (no secrets needed) and a signed release APK+AAB if
  `SCMESSENGER_KEYSTORE_BASE64` secret is configured. Secrets are NOT set, so
  only the debug APK runs today. Debug APK is fine for Josh (trusted alpha tester,
  just needs "install from unknown sources" enabled on his phone).

- **GitHub Release vs workflow_dispatch artifact?** GitHub Release. `create-release`
  job uses `softprops/action-gh-release@v2` to create a real release page with
  the APK attached. This is a real clickable URL. Triggered by tag push or
  `workflow_dispatch`.

- **Signing?** Unsigned debug APK for this alpha. No keystore secret configured.
  For Josh specifically this is acceptable. When keystore secrets are added, the
  signed release APK path activates automatically (the gradle signing config
  checks for `SCMESSENGER_KEYSTORE_PATH` env var first).

- **Version tag?** Tag as `v1.0.0-alpha.1`. The `prerelease` flag in the workflow
  is set automatically when the tag contains "alpha". Workspace version stays at
  0.3.5 internally -- the git tag is the external-facing label.

- **iOS blocking the release?** Fixed. Removed `build-ios` from `create-release`'s
  `needs` list (iOS distribution requires Apple Developer account -- human decision
  pending). iOS job still runs so failures surface, but the release doesn't wait on it.

- **Relay address location:** hardcoded in
  `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:87`
  (`getBootstrapNodesForSettings()` returns `listOf("/ip4/100.56.248.69/tcp/9001")`).
  No change needed for this alpha -- just don't lose track of it for future relay
  updates.

## Acceptance

- A real URL exists (GitHub Release page, or equivalent) that Lucas can send
  Josh, from which Josh can download and install the app without needing to
  clone the repo or compile anything himself.
- The artifact was built from a commit where CI is green.
- The relay bootstrap address baked into the build is documented (done above).

## How to trigger (WHEN CI IS GREEN AND LUCAS GIVES GO-AHEAD)

```
git tag v1.0.0-alpha.1
git push origin v1.0.0-alpha.1
```

This triggers `release.yml` which:
1. Builds Windows/Linux/macOS CLI binaries
2. Builds Android debug APK
3. Builds WASM
4. Creates a GitHub Release at https://github.com/Sovereign-Communication/SCMessenger/releases
   with all artifacts attached, marked as pre-release

DO NOT push the tag without Lucas's explicit go-ahead (standing rule).

## Notes

- This does NOT require the crypto-security-auditor gate by itself (it's a
  packaging/CI task), unless the work uncovers a reason to touch signing
  material or crypto code, in which case the standard gate applies.
- Never push tags/releases without Lucas's explicit go-ahead, same as the
  standing "never push without operator go-ahead" rule for commits.
