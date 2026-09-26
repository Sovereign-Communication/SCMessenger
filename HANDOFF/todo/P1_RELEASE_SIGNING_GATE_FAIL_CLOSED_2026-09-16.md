# P1: Release Signing Gate Fail-Closed Enforcement (SEC-01 & SEC-02)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** OPEN
**Priority:** P1 (v0.4.0 Release Blocker)
**Target Branch:** `feat/v040-multi-transport-store-forward`
**Components:** `.github/workflows/release.yml`, `scripts/verify_release_keystore.sh`
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
In `.github/workflows/release.yml`, all signing and verification steps in `build-android` run conditionally under `if: ${{ env.HAS_KEYSTORE == 'true' }}`, but `assembleDebug` runs unconditionally. If `SCMESSENGER_KEYSTORE_BASE64` or `SCMESSENGER_KEY_ALIAS` is missing or mismatched:
1. `build-android` succeeds without signing release artifacts.
2. `create-release` downloads all artifacts and packages `app-debug.apk` (`CN=Android Debug`) into the public release with zero `.aab` bundles.
3. In addition, rehearsal 34996353889 failed with `SCMESSENGER_KEY_ALIAS is not present in the decoded keystore` due to alias case mismatch.

## Acceptance Criteria
1. Make `HAS_KEYSTORE == 'true'` mandatory when triggered by a release version tag (`startsWith(github.ref, 'refs/tags/v')`), failing the build immediately if false.
2. In `create-release`, explicitly filter out `*debug*.apk` from public release asset uploads.
3. Validate and correct secret `SCMESSENGER_KEY_ALIAS` matching the keystore entry (`scmessenger`).
