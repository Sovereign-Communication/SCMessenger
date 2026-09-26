<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: gemma4:31b:cloud
# BUDGET: 600
# token_budget: 6000

# P1_IOS_001_Build_Verification_And_Smoke_Test_Plan

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 600s (MICRO tier)
**Phase:** v0.2.1 P1 iOS verification
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md 1.3 (iOS Stability) + planfromclaudeforhermes 2 Phase E.1-E.2
**Depends on:** P0_BUILD_001 (cross-platform compilation check)

---

## Verified Gap

Per `ALPHA_BURNDOWN_V0.2.1.md` "iOS" row: " PASS  `verify-test.sh` auto-generates bindings; `@MainActor` isolation fixed". This is the CI matrix. The actual user-side smoke test on a real iOS device is not automated.

`HANDOFF/backlog/P1_IOS_002_NOTIFICATION_VERIFICATION.md` and `P1_IOS_*` items list real-device verification as the open gap.

## Scope (~30 LoC of harness, plus user-side test)

### Part A: Verify build script exists and works (LOC: ~10)

In `iOS/verify-test.sh`:
- Confirm file exists; if missing, create it (basic Xcode test invocation)

### Part B: Document smoke test plan (LOC: ~20)

Create `iOS/PHYSICAL_DEVICE_SMOKE_TEST.md`:

```markdown
# iOS Physical Device Smoke Test Plan

## Prerequisites
- Xcode 15+ installed
- iOS device (iPhone X or newer) connected via USB
- Apple Developer account signed in
- `iOS/SCMessenger.xcworkspace` builds clean

## Steps
1. Open `iOS/SCMessenger.xcworkspace` in Xcode
2. Select real device (not simulator)
3. Set signing team: SCMessenger  Signing & Capabilities  Team
4. Build: `xcodebuild -workspace ios/SCMessenger.xcworkspace -scheme SCMessenger -configuration Debug -sdk iphoneos -destination 'platform=iOS,name=<iPhone Name>'`
5. Install via Xcode  Run
6. Walk through:
   - [ ] Onboarding flow completes (6 steps)
   - [ ] Identity created
   - [ ] Send message to CLI daemon (paired via Android or another device)
   - [ ] Receive message from CLI daemon
   - [ ] Notification permission prompted
   - [ ] Background app, send message from another device  notification received
   - [ ] Kill app, relaunch  message history visible
   - [ ] No crashes in Console.app

## Expected Results
- All checkboxes ticked
- No `EXC_BAD_ACCESS` or `Fatal Error` in Console
- Memory usage stable (< 200MB)

## Report Format
Run script: `scripts/ios_smoke_test.sh` (created in this task) and append output to `iOS/SMOKE_TEST_RESULTS_<date>.md`
```

## File Targets

- `iOS/verify-test.sh` [VERIFY EXISTS, EDIT if missing]
- `iOS/PHYSICAL_DEVICE_SMOKE_TEST.md` [NEW]
- `scripts/ios_smoke_test.sh` [NEW, ~10 lines]

## Build Verification Commands

```bash
# macOS only  Windows host cannot run xcodebuild
# On Windows: confirm files exist, defer execution to macOS CI or user
ls iOS/verify-test.sh
ls iOS/PHYSICAL_DEVICE_SMOKE_TEST.md
ls scripts/ios_smoke_test.sh
```

## Acceptance Gates

1. `iOS/verify-test.sh` exists
2. `iOS/PHYSICAL_DEVICE_SMOKE_TEST.md` exists with 9-step checklist
3. `scripts/ios_smoke_test.sh` exists (even if not run on this host)
4. CI matrix shows iOS job  PASS
5. Commit: `ios: v0.2.1 verify-test.sh + physical device smoke test plan`

## REQUIRES_USER_ACTION
User must run the smoke test on a real iOS device and report results. Subagent cannot execute xcodebuild on Windows host.

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: SWIFT_DOCS] [REQUIRES: GEMMA_4_31B] [DEPENDS_ON: P0_BUILD_001] [REQUIRES_USER_DEVICE]
