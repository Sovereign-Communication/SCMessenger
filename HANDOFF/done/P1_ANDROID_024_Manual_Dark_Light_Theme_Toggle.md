# P1_ANDROID_024: Manual Dark/Light Theme Toggle

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Objective

Add a manual theme toggle in Settings so users can override the system default dark/light mode.

## Background

The app already supports:
- `SCMessengerTheme(darkTheme: Boolean = isSystemInDarkTheme(), dynamicColor: Boolean = true)`
- `darkColorScheme()` and `lightColorScheme()` static palettes
- Dynamic colors on Android 12+

What's missing:
- No user preference stored for theme override
- No UI toggle in Settings to switch between System Default / Light / Dark

## Implementation Plan

### 1. PreferencesRepository.kt
Add theme preference storage:
```kotlin
enum class ThemeMode { SYSTEM, LIGHT, DARK }

val themeMode: Flow<ThemeMode>  // stored as string in DataStore
suspend fun setThemeMode(mode: ThemeMode)
```

### 2. MainViewModel.kt
Expose `themeMode` state flow.

### 3. MainActivity.kt
Pass `themeMode` to `SCMessengerTheme()`:
```kotlin
val themeMode by mainViewModel.themeMode.collectAsState()
val darkTheme = when (themeMode) {
    ThemeMode.LIGHT -> false
    ThemeMode.DARK -> true
    ThemeMode.SYSTEM -> isSystemInDarkTheme()
}
SCMessengerTheme(darkTheme = darkTheme) { ... }
```

### 4. SettingsScreen.kt
Add a "Theme" section with radio buttons or dropdown:
- System Default (follows OS)
- Light
- Dark

Place it in the App Preferences section or as its own card.

## Acceptance Criteria

1. Setting persists across app restarts
2. Toggle immediately changes theme without restart
3. Works correctly with dynamic colors on Android 12+
4. `./gradlew :app:compileDebugKotlin` passes

## Related

- P1_ANDROID_RELEASE_001 (build verification)

---

**Priority:** P1
**Type:** Feature / Polish
**Estimated LoC Impact:** ~80
**Blocking:** No
