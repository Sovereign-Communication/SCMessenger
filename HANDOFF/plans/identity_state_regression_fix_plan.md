# Design Plan: Identity State Regression and Recovery

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

[INFO] This plan addresses the latent state machine bug where the Android client's identity state cannot regress from `Ready` to `Uninitialized` (or `None`) when the identity is deleted, wiped, or uninitialized.

---

## 1. Issue Explanation and Root Cause

### 1.1 State Machine in IdentityCreationCoordinator
The `IdentityCreationCoordinator` class manages the state transitions of the local user identity (`IdentityState`) using a `MutableStateFlow<IdentityState>(_identityState)`. It initializes and updates this state by observing changes in `MeshRepository.identityInfo` inside its `init` block:

```kotlin
    init {
        // Observe MeshRepository identityInfo changes
        scope.launch {
            meshRepository.identityInfo.collect { info ->
                val initialized = info?.initialized == true
                if (initialized) {
                    _identityState.value = IdentityState.Ready
                } else if (_identityState.value != IdentityState.Ready) {
                    // Don't regress from Ready — the initial null emission from
                    // StateFlow construction must not override a valid cached state.
                    determineInitialState()
                }
            }
        }
        determineInitialState()
    }
```

### 1.2 The Regression Trap
The exact block trapping the regression is:
```kotlin
                } else if (_identityState.value != IdentityState.Ready) {
                    // Don't regress from Ready — the initial null emission from
                    // StateFlow construction must not override a valid cached state.
                    determineInitialState()
                }
```

* **The Trap:** When `initialized` is `false` (meaning the identity is uninitialized or wiped), the code checks if the current state is NOT `Ready`. If the current state is already `Ready`, the block is completely skipped.
* **The Reason:** This check was originally introduced to prevent a transient `null` or uninitialized emission during application startup (before the Rust core has hydrated the identity database) from overriding a valid cached identity state.
* **The Bug:** Because the transition is skipped when `_identityState.value == IdentityState.Ready`, any legitimate uninitialization event (e.g., a data wipe or factory reset) is ignored, trapping the coordinator in the `Ready` state permanently until the app is restarted.

### 1.3 Missing Event Propagation in MeshRepository.resetAllData()
In `MeshRepository.kt`, the `resetAllData()` function clears the identity backup and cache preferences:
```kotlin
        // 2. Clear identity backup SharedPreferences
        identityBackupPrefs.edit().clear().apply()

        // P0: Clear cached identity fields so reset is reflected on next launch
        identityCachePrefs.edit().clear().apply()
```
However, it fails to call `publishIdentityInfo(null)`. Consequently, the centralized `_identityInfo` StateFlow does not emit a `null` value, and downstream observers never receive any notification of the data wipe.

### 1.4 Unidirectional State Collection in MainViewModel
In `MainViewModel.kt`, the observer of `meshRepository.identityInfo` only updates `_isReady` to `true`:
```kotlin
        viewModelScope.launch {
            meshRepository.identityInfo.collect { info ->
                val initialized = info?.initialized == true
                if (initialized && !_isReady.value) {
                    _isReady.value = true
                }
            }
        }
```
If `initialized` drops to `false`, `_isReady.value` remains `true`, trapping the main app container in the logged-in state.

---

## 2. Proposed Code Changes (Diffs)

### 2.1 MeshRepository.kt
Propagate the identity deletion by publishing `null` to the centralized flow at the end of the `resetAllData()` method.

```diff
--- android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
+++ android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
@@ -5165,5 +5165,7 @@
         }
 
+        publishIdentityInfo(null)
         Timber.i("All application data reset successfully")
     }
```

### 2.2 IdentityCreationCoordinator.kt
Safely allow state regression to `None` if the identity is truly missing from the disk.

```diff
--- android/app/src/main/java/com/scmessenger/android/data/IdentityCreationCoordinator.kt
+++ android/app/src/main/java/com/scmessenger/android/data/IdentityCreationCoordinator.kt
@@ -48,8 +48,15 @@
                 val initialized = info?.initialized == true
                 if (initialized) {
                     _identityState.value = IdentityState.Ready
-                } else if (_identityState.value != IdentityState.Ready) {
-                    // Don't regress from Ready — the initial null emission from
-                    // StateFlow construction must not override a valid cached state.
-                    determineInitialState()
+                } else {
+                    val isReallyInitialized = meshRepository.isIdentityInitialized()
+                    if (!isReallyInitialized) {
+                        // Safe regression: no identity exists on disk.
+                        _identityState.value = IdentityState.None
+                        _progressStage.value = IdentityProgressStage.Idle
+                        _progressSubDetail.value = null
+                        _error.value = null
+                    } else if (_identityState.value != IdentityState.Ready) {
+                        determineInitialState()
+                    }
                 }
             }
```

### 2.3 MainViewModel.kt
Allow `_isReady` to transition back to `false` when the identity is wiped, forcing the UI to update.

```diff
--- android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt
+++ android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt
@@ -140,4 +140,4 @@
                 val initialized = info?.initialized == true
-                if (initialized && !_isReady.value) {
-                    Timber.d("P0_SHARED_IDENTITY: meshRepository.identityInfo reports initialized; updating _isReady")
-                    _isReady.value = true
-                }
+                if (_isReady.value != initialized) {
+                    Timber.d("P0_SHARED_IDENTITY: meshRepository.identityInfo reports initialized=$initialized; updating _isReady")
+                    _isReady.value = initialized
+                }
             }
```

---

## 3. Downstream UI and ViewModel Impact Analysis

### 3.1 SettingsViewModel
* **Identity Flow Collection:** `SettingsViewModel` mirrors `meshRepository.identityInfo` inside its `init` block and updates `_hasIdentity` and `cachedIdentityInfo` via `emitIdentityInfo()`.
* **Wipe Handling:** When `emitIdentityInfo(null)` is called:
  - `cachedIdentityInfo` becomes `null`.
  - `_hasIdentity` receives `meshRepository.isIdentityInitialized()`, which resolves to `false`.
* **Safety Evaluation:** [OK] `SettingsViewModel` handles `null` identity information correctly and updates its active state flow appropriately.

### 3.2 IdentityViewModel
* **Identity Flow Collection:** `IdentityViewModel` mirrors `meshRepository.identityInfo` into a local StateFlow `_identityInfo`.
* **QR Code Regeneration:** The collector on `_identityInfo` updates `_qrCodeData`:
  ```kotlin
          viewModelScope.launch(Dispatchers.Default) {
              _identityInfo.collect { info ->
                  if (info?.initialized == true) {
                      _qrCodeData.value = getQrCodeData()
                  } else {
                      _qrCodeData.value = null
                  }
              }
          }
  ```
  If `_identityInfo` becomes `null`, `_qrCodeData` correctly resets to `null`.
* **Safety Evaluation:** [OK] The internal progress states and error states will successfully clear when `IdentityCreationCoordinator` regresses to `IdentityState.None`.

### 3.3 Main Navigation Graph (MeshApp.kt)
* **Onboarding View Trigger:** `MeshApp` displays the onboarding/setup screen when `showOnboarding` resolves to `true`.
* **Onboarding Formula:** `showOnboarding` is defined as `!isReady && !installChoiceCompleted`.
* **Navigation Transition:** Wiping the identity will clear the preferences (setting `installChoiceCompleted` to `false`) and update `_isReady` in `MainViewModel` to `false`. As a result, `showOnboarding` transitions to `true` instantly, routing the user back to the onboarding interface.
* **Safety Evaluation:** [OK] The UI reacts safely and immediately to the state change, preventing access to the dashboard when no identity is present.
