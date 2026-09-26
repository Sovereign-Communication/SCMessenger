# TASK: Fix iOS CI workflow so free public-repo macOS runners produce real signal

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO
Tier: [SONNET] (mechanical once decisions below are read)
Blocks: iOS parity lane (bindings regen + simulator tests without local Mac)

## Context

The repo is PUBLIC on GitHub, so standard hosted runners -- including macOS --
are free with unlimited minutes. This makes GitHub Actions the primary macOS
capacity for iOS work (bindings regen, XCFramework build, simulator tests).
But the existing workflow cannot produce trustworthy signal:

## Defects in .github/workflows/ios-build-test.yml (verified 2026-07-11)

1. Failure masking: xcodebuild is piped through `| xcpretty || true` (~lines
   101, 112) so the job can NEVER fail. Remove `|| true`; use
   `set -o pipefail` before piped xcodebuild.
2. Lowercase paths: uses `ios/` working-directory and paths (~lines 84-94).
   Repo rule is uppercase `iOS/` in ALL references (CI-enforced elsewhere;
   works only because APFS is case-insensitive). Fix all references.
3. xcodebuild invoked with -scheme but no -project/-workspace while the
   project lives at iOS/SCMessenger/SCMessenger.xcodeproj -- point it there.
4. Workflow is workflow_dispatch-only. Add pull_request/push triggers on
   paths: iOS/**, core/src/api.udl, mobile/** so FFI drift is caught.

## Additional steps for real value

5. Add a bindings-drift gate: regenerate Swift bindings in CI (cargo run
   --bin gen_swift with the gen-bindings feature) and diff against the
   committed iOS/.../Generated/api.swift -- fail on drift. This catches the
   current known drift: PQC-05 added `require_pq` to AppSettings
   (core/src/api.udl:263) AFTER the last Swift regen (2026-07-02).
   NOTE: do NOT regenerate-and-commit until after PQC-10 lands (it changes
   identity signatures; one regen cycle after PQC-10 avoids doing it twice).
6. scripts/verify_ios_bindings.sh only checks IdentityInfo + sendMessage and
   contains emoji (lines 17,24,35,42) violating the no-emoji rule -- either
   replace it with the CI drift gate above or extend it to full-surface diff
   and strip the emoji.
7. Run the existing XCTest suites on an iOS Simulator destination
   (NotificationVerificationTests, BackupPassphraseValidatorTests,
   MeshBackgroundServiceTests). Simulator CANNOT cover: CoreBluetooth,
   Multipeer/AWDL radios, APNs, true background scheduling -- record those
   as hardware-waived cells, do not fake them.

## Acceptance

- A push touching iOS/ or api.udl triggers the workflow on macos-14.
- A deliberately broken Swift file makes the job FAIL (prove failure works).
- Bindings drift (current state) makes the job FAIL until regen lands.
- No emoji, uppercase iOS/ paths throughout.
