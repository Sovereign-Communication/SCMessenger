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
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_ANDROID_Identity_Generation_From_Settings_Missing_Entropy_And_Hangs_30s

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P1 Android UX stability
**Priority:** P1 (user-blocking  every user who skips onboarding hits this)
**Source:** Lucas 2026-06-08, Telegram DM: "issue for Android where identity generation isn't showing the (entropy canvas / finger-move) from within settings menu. It works from fresh install, but when in settings it's bugging out and not using the full flow. Plus I hit generate identity and it doesn't indicate that it's doing anything, so it seems hung, but then works after like 30 seconds."
**Depends on:** none (independent)
**Branch:** `fix/p1-android-settings-identity-entropy` (NEW, off origin/main `dd109707`)
**Worktree:** TBD  pick from `E:\SCMessenger-build-p0-024\` or fresh

---

## Verified Gap (with line citations)

### Bug 1  Settings  Identity path skips EntropyCanvas entirely

The finger-move "Entropy Gathering Box" (`com.scmessenger.android.ui.components.EntropyCanvas`, 280dp Canvas, drag gesture  SHA-256 salt) is wired up **only in the onboarding flow**. The Settings  Identity path goes through a completely different ViewModel and a stub `IdentityNotInitializedView` that has **no entropy collection at all**.

Code path when the user taps "Create Identity" in Settings:

1. `SettingsScreen.kt:158-160`  `IdentityUnavailableSection(onCreateIdentity = onNavigateToIdentity)` (just navigates away)
2. `MeshApp.kt:246-249`  `onNavigateToIdentity = { navController.navigate(Screen.Identity.route) }`
3. `MeshApp.kt:282-286`  `composable(Screen.Identity.route) { IdentityScreen(onNavigateBack = ...) }`
4. `IdentityScreen.kt:38-93`  `IdentityScreen` uses **`IdentityViewModel`**, NOT `MainViewModel`
5. `IdentityScreen.kt:87-92`  when `identityInfo.initialized != true`, renders `IdentityNotInitializedView`
6. `IdentityScreen.kt:113-147`  `IdentityNotInitializedView` is a **bare** nickname field + "Create" button. **No EntropyCanvas. No salt collection. No "Move your finger" UI.**

Compare to the onboarding path (the working one):

7. `OnboardingScreen.kt:65`  `val isCreating by viewModel.isCreatingIdentity.collectAsState()` (on `MainViewModel`)
8. `OnboardingScreen.kt:241-248`  `if (nickname.trim().isNotEmpty()) { EntropyCanvas(onEntropyComplete = { salt -> touchEntropySalt = salt }) }`
9. `OnboardingScreen.kt:255-264`  button enabled only when `touchEntropySalt != null && !isCreating`, calls `viewModel.createIdentity(nickname, touchEntropySalt)`

**The fix:** the Settings path must render the **same** `EntropyCanvas` flow as onboarding, or both must be refactored to share a single `IdentityCreationFlow` composable.

### Bug 2  "Generate Identity" silently hangs for ~30 seconds with no UI feedback

The user reports the button "doesn't indicate that it's doing anything, so it seems hung, but then works after like 30 seconds."

Two compounding causes:

**Cause 2a  `IdentityViewModel.createIdentity` flips `_isLoading` AFTER launching, not before.** `IdentityViewModel.kt:91-119`:

```kotlin
fun createIdentity(nickname: String? = null) {
    viewModelScope.launch(Dispatchers.IO) {   // <-- the IO jump happens first
        try {
            _isLoading.value = true            // <-- only set true INSIDE the coroutine
            _error.value = null
            meshRepository.createIdentity()    // <-- this is the heavy FFI call
            ...
```

Compare to the working `MainViewModel.createIdentity` at `MainViewModel.kt:182-187`:

```kotlin
if (_isCreatingIdentity.value) {              // <-- guard BEFORE the launch
    Timber.d("createIdentity: ignored re-entrant call (already in progress)")
    return
}
viewModelScope.launch(Dispatchers.IO) {
    _isCreatingIdentity.value = true          // <-- flipped inside, but UI gates on isCreating
    ...
```

The IdentityViewModel flow doesn't even have a re-entrancy guard at all. A second tap during the ~30s would race the same way the P0_ANDROID_024 race did.

**Cause 2b  `IdentityNotInitializedView` doesn't render progress at all.** `IdentityScreen.kt:113-147`:

```kotlin
Button(onClick = { onCreateIdentity(nickname) }) {  // <-- no isLoading check
    Text(stringResource(R.string.identity_action_create))
}
```

There is no `CircularProgressIndicator`, no "Generating keys..." text, no disabled-while-busy state, no overlay. The button just sits there pressed until the FFI call returns. Lucas's "30 seconds" is consistent with Ed25519 keygen + sled write + identity federation setup + the implicit `getIdentityInfoNonBlocking` reload. That is heavy work, and the UI gives zero signal that it's happening.

**Cause 2c  The `isLoading` state IS collected in `IdentityScreen` (line 44) but only used to gate the `CircularProgressIndicator` on the top-level `when` block (line 81-85), which is inside the `Scaffold` body and only shows when `isLoading == true` for the **whole screen**. So when `IdentityNotInitializedView` is showing, the loading spinner on the parent `Box` is *behind* the view, not in front of the button.**

### Bug 3  `IdentityViewModel.createIdentity` does not accept or use a touch-entropy salt

`IdentityViewModel.kt:91`:

```kotlin
fun createIdentity(nickname: String? = null) {  // <-- no salt parameter
    ...
    meshRepository.createIdentity()              // <-- salt is null
```

But `MeshRepository.createIdentity(customSalt: ByteArray? = null)` **does** accept a salt (`MeshRepository.kt:4373`). The onboarding path passes the salt from `EntropyCanvas.onEntropyComplete`. The Settings path silently drops the entropy-collection step entirely and calls with `null`. The user generates an identity that has **lower entropy than the onboarding path produces**  security regression, not just UX.

---

## Scope (~80 LoC across 3 files)

### Part A: Extract a shared `IdentityCreationFlow` composable (~60 LoC)

New file: `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityCreationFlow.kt`

This composable holds the nickname field, the `EntropyCanvas`, the generate button, the loading state, and the salt pass-through. Both `OnboardingScreen` and `IdentityScreen.IdentityNotInitializedView` consume it.

```kotlin
@Composable
fun IdentityCreationFlow(
    isCreating: Boolean,
    onCreate: (nickname: String, salt: ByteArray?) -> Unit,
    onImport: () -> Unit = {},  // for the onboarding import button
    showImportButton: Boolean = false,
    modifier: Modifier = Modifier
) {
    var nickname by remember { mutableStateOf("") }
    var touchEntropySalt by remember { mutableStateOf<ByteArray?>(null) }

    Column(modifier = modifier, ...) {
        OutlinedTextField(
            value = nickname,
            onValueChange = {
                nickname = it
                touchEntropySalt = null
            },
            label = { Text(stringResource(R.string.onboarding_label_nickname)) },
            singleLine = true,
            ...
        )

        if (nickname.trim().isNotEmpty()) {
            Spacer(modifier = Modifier.height(16.dp))
            EntropyCanvas(
                onEntropyComplete = { salt -> touchEntropySalt = salt }
            )
        }

        Spacer(modifier = Modifier.height(12.dp))
        Button(
            onClick = {
                onCreate(nickname.trim(), touchEntropySalt)
            },
            enabled = nickname.trim().isNotEmpty() && touchEntropySalt != null && !isCreating,
            modifier = Modifier.fillMaxWidth().height(56.dp)
        ) {
            if (isCreating) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Spacer(modifier = Modifier.size(8.dp))
                Text(stringResource(R.string.onboarding_generating_keys))
            } else {
                Text(stringResource(R.string.identity_action_create))
            }
        }

        // Optional: Skip-for-Relay and Import buttons (onboarding-only)
        if (showImportButton) {
            Spacer(modifier = Modifier.height(8.dp))
            OutlinedButton(onClick = onImport, ...) {
                Text(stringResource(R.string.onboarding_button_import_join))
            }
        }
    }
}
```

**Key UX improvements baked in:**
- Button shows an inline spinner + "Generating keys..." text when `isCreating == true`. **No more silent 30-second hang.**
- Button is disabled while `isCreating` (re-entrancy defense).
- The `EntropyCanvas` is **always** shown when nickname is non-empty  same as onboarding.
- Salt is passed through to `onCreate`  entropy is preserved.

### Part B: Wire `IdentityViewModel` to accept and use the salt (~10 LoC)

`IdentityViewModel.kt:91`:

```kotlin
fun createIdentity(nickname: String? = null, customSalt: ByteArray? = null) {
    if (_isLoading.value) {                       // <-- NEW: re-entrancy guard
        Timber.d("createIdentity: ignored re-entrant call (already in progress)")
        return
    }
    viewModelScope.launch(Dispatchers.IO) {
        _isLoading.value = true
        _error.value = null
        try {
            meshRepository.createIdentity(customSalt)   // <-- pass salt through
            if (nickname != null && nickname.isNotBlank()) {
                meshRepository.setNickname(nickname)
            }
            val info = meshRepository.getIdentityInfoNonBlocking()
            if (_identityInfo.value != info) {
                _identityInfo.value = info
            }
            _successMessage.value = "Identity created successfully"
        } catch (e: Exception) {
            _error.value = "Failed to create identity: ${e.message}"
            Timber.e(e, "Failed to create identity")
        } finally {
            _isLoading.value = false
        }
    }
}
```

### Part C: Use `IdentityCreationFlow` in both places

**In `IdentityScreen.IdentityNotInitializedView`** (`IdentityScreen.kt:113-147`):

```kotlin
@Composable
private fun IdentityNotInitializedView(
    isCreating: Boolean,
    onCreateIdentity: (nickname: String, salt: ByteArray?) -> Unit,   // <-- now takes salt
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(stringResource(R.string.identity_not_initialized_title), ...)
        Text(stringResource(R.string.identity_not_initialized_description), ...)
        IdentityCreationFlow(
            isCreating = isCreating,
            onCreate = onCreateIdentity,
            showImportButton = false
        )
    }
}
```

And in the parent `IdentityScreen.kt:87-92`:

```kotlin
identityInfo == null || identityInfo?.initialized != true -> {
    IdentityNotInitializedView(
        isCreating = isLoading,                                       // <-- pass through
        onCreateIdentity = { nickname, salt ->
            viewModel.createIdentity(nickname, customSalt = salt)     // <-- pass salt
        },
        modifier = Modifier.align(Alignment.Center)
    )
}
```

**In `OnboardingScreen.kt:220-264`:** replace the inline nickname/canvas/button block with `IdentityCreationFlow(isCreating = isCreating, onCreate = { nickname, salt -> viewModel.createIdentity(nickname, salt) }, onImport = { showImportDialog = true }, showImportButton = true)`. The "Skip for Relay-Only Install" button stays outside the shared composable since it's onboarding-specific.

---

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityCreationFlow.kt` [NEW  shared composable]
- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt` [EDIT  use IdentityCreationFlow, pass isLoading, pass salt]
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt` [EDIT  accept customSalt, add re-entrancy guard]
- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt` [EDIT  use IdentityCreationFlow, remove inline duplicate]

**Reuse (no edits needed):**
- `android/app/src/main/java/com/scmessenger/android/ui/components/EntropyCanvas.kt`  already correct, no changes
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4373`  already accepts `customSalt: ByteArray? = null`

---

## Build Verification Commands

```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
export CARGO_INCREMENTAL=0
cd android
./gradlew assembleDebug -x lint --quiet 2>&1 | tee ../tmp/build_logs/gradle_settings_identity_$(date +%s).log
./gradlew :app:testDebugUnitTest 2>&1 | tee ../tmp/build_logs/gradle_test_settings_identity_$(date +%s).log
```

## Acceptance Gates

1. **From a fresh install:** identity generation flow unchanged (still shows EntropyCanvas, still works).
2. **From Settings  Identity (no identity yet):** the user sees a nickname field, the EntropyCanvas appears once nickname is entered, the "Create" button is disabled until entropy is collected, an inline spinner + "Generating keys..." text appears on the button during the FFI call, and the button is disabled until the call returns. **No 30-second silent hang.**
3. **Tap "Generate Identity" twice rapidly:** second tap is a no-op (re-entrancy guard), identity is still created exactly once.
4. **Generated identity bytes match** the onboarding path's output format (entropy is preserved through `MeshRepository.createIdentity(customSalt)`).
5. **No regression** in `OnboardingScreen`  same flow, same UX, same tests pass.

## Tests to Add

- `android/app/src/test/java/com/scmessenger/android/ui/identity/IdentityViewModelTest.kt`:
  - `test createIdentity with custom salt passes salt to repository` (use a fake `MeshRepository`)
  - `test createIdentity re-entrancy guard prevents second concurrent call`
  - `test createIdentity sets isLoading before launching coroutine` (assert on initial state right after invocation)

- `android/app/src/test/java/com/scmessenger/android/ui/identity/IdentityCreationFlowTest.kt` (Compose UI test):
  - `test entropy canvas appears after nickname is entered`
  - `test generate button is disabled when salt is null`
  - `test generate button shows spinner and Generating Keys text when isCreating is true`
  - `test clicking generate with valid input calls onCreate with (nickname, salt)`

## Pre-flight Reads (for the worker)

- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt` lines 214-264  the existing working flow to preserve
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt` lines 175-240  the existing working re-entrancy guard pattern to copy
- `android/app/src/main/java/com/scmessenger/android/ui/components/EntropyCanvas.kt`  the entropy widget, no changes
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4373`  `createIdentity(customSalt: ByteArray? = null)` signature

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags

[REQUIRES: ANDROID] [REQUIRES: COMPOSE] [REQUIRES: KOTLIN] [REQUIRES: QWEN3_CODER_NEXT_CLOUD] [INDEPENDENT_OF: P0_BUILD_001] [INDEPENDENT_OF: P0_ANDROID_024] [TIER: 2]
