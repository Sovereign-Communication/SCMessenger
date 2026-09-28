# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1080

# Task: Fix P0 Theme.kt Regression  Restore Status Bar Color

**Priority:** P0
**Model:** qwen3-coder-next:cloud
**Budget:** 1080
**Assigned to:** implementer
**Created:** 2026-05-13
**Status:** FAILED  prematurely moved to done/ without code changes. Theme.kt still does not set status bar color.
**Source:** P0 Android Play Readiness regression audit

## Summary

The P0 deprecation fix in `android/app/src/main/java/com/scmessenger/android/ui/theme/Theme.kt` replaced the deprecated `window.statusBarColor` with `WindowCompat.setDecorFitsSystemWindows(window, false)`, but **lost** the status bar color tinting. The status bar must still be colored with `colorScheme.primary`.

## What To Do

In `Theme.kt` (SCMessengerTheme composable, SideEffect block):

1. Restore the status bar color using a non-deprecated API. Options:
   - Keep `setDecorFitsSystemWindows(false)` and use `window.statusBarColor` with a targeted `@Suppress("DEPRECATION")` + comment explaining API 35 has no direct replacement and the color tint is functionally required
   - OR use `EdgeToEdge` / system bar insets APIs if they provide color control

2. Ensure BOTH color tinting AND `isAppearanceLightStatusBars` work correctly.

Current broken code (line 48-51):
```kotlin
WindowCompat.setDecorFitsSystemWindows(window, false)
WindowCompat.getInsetsController(window, view).apply {
    isAppearanceLightStatusBars = !darkTheme
}
```

Must also set status bar color to `colorScheme.primary.toArgb()`.

## Verification

- `cd android && ./gradlew assembleDebug -x lint --quiet` must pass
- Status bar color must be visible at runtime (primary color tint)
- No unaddressed deprecation warnings for `statusBarColor` (single targeted suppression is acceptable)

## Files Expected to Change

- `android/app/src/main/java/com/scmessenger/android/ui/theme/Theme.kt`
