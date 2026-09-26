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
# BUDGET: 1200
# token_budget: 12000

# P1_ANDROID_023_History_Persistence_Regression_Test

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1 Android stability
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md 1.2 (Verify message history persistence across app restarts) + planfromclaudeforhermes 2 Phase D.5
**Depends on:** P0_BUILD_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` 1.2: "Verify message history persistence across app restarts"  listed but not done.

No existing regression test for the "create message, force-stop app, restart, verify message present" flow.

## Scope (~100 LoC across 1-2 files)

### Part A: Compose UI test for persistence (LOC: ~60)

In `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt` (NEW):

```kotlin
@RunWith(AndroidJUnit4::class)
class MeshRepositoryHistoryTest {
    @get:Rule val rule = createAndroidComposeRule<MainActivity>()
    
    @Test
    fun messageHistory_persistsAcrossAppRestart() {
        // 1. Create identity + send message to self
        rule.onNodeWithText("Get Started").performClick()
        rule.onNodeWithTag("consent_checkbox").performClick()
        rule.onNodeWithText("Continue").performClick()
        val nickname = "TestUser_${System.currentTimeMillis()}"
        rule.onNodeWithTag("nickname_field").performTextInput(nickname)
        rule.onNodeWithText("Create").performClick()
        rule.onNodeWithTag("message_input").performTextInput("Persistence test message")
        rule.onNodeWithTag("send_button").performClick()
        
        // Verify message visible
        rule.onNodeWithText("Persistence test message").assertIsDisplayed()
        
        // 2. Force-stop the app
        val activity = rule.activity
        val packageName = activity.packageName
        val uiAutomation = InstrumentationRegistry.getInstrumentation().uiAutomation
        uiAutomation.executeShellCommand("am force-stop $packageName")
        Thread.sleep(2000)
        
        // 3. Restart the app
        val intent = activity.packageManager.getLaunchIntentForPackage(packageName)
        activity.startActivity(intent)
        rule.waitForIdle()
        
        // 4. Verify message still visible
        rule.onNodeWithText("Persistence test message").assertIsDisplayed()
    }
}
```

### Part B: Helper for restart (LOC: ~40)

In `android/app/src/androidTest/java/com/scmessenger/android/util/AppRestartHelper.kt` (NEW):

```kotlin
object AppRestartHelper {
    fun forceStopAndRestart(packageName: String) {
        val uiAutomation = InstrumentationRegistry.getInstrumentation().uiAutomation
        uiAutomation.executeShellCommand("am force-stop $packageName")
        Thread.sleep(2000)
        val intent = InstrumentationRegistry.getInstrumentation().context
            .packageManager.getLaunchIntentForPackage(packageName)
        intent?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        InstrumentationRegistry.getInstrumentation().context.startActivity(intent)
    }
}
```

## File Targets

- `android/app/src/androidTest/java/com/scmessenger/android/data/MeshRepositoryHistoryTest.kt` [NEW]
- `android/app/src/androidTest/java/com/scmessenger/android/util/AppRestartHelper.kt` [NEW]

## Build Verification Commands

```bash
cd android
./gradlew :app:assembleDebugAndroidTest
./gradlew :app:connectedDebugAndroidTest --tests "*MeshRepositoryHistoryTest"
# Requires: Pixel 6a (or any API 33+ device) connected via adb
```

## Acceptance Gates

1. `./gradlew :app:assembleDebugAndroidTest` succeeds
2. `MeshRepositoryHistoryTest.messageHistory_persistsAcrossAppRestart` passes on real device
3. Test reliably reproduces the persistence behavior (run 3x, all pass)
4. Test takes < 30 seconds (restart + assertion)
5. Commit: `android: v0.2.1 message history persistence regression test`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## REQUIRES_USER_ACTION
This test must be run on a real Android device (Pixel 6a per backlog). User connects device via adb, then subagent runs the test. If device unavailable, defer to manual verification (user reports result).

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: GEMMA_4_31B] [DEPENDS_ON: P0_BUILD_001] [REQUIRES_REAL_DEVICE]
