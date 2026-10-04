# TASK: Android IdentityScreen Scroll Broken After QR Loads

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 4: Android UI Bug Fix (single-file, low-risk)

## Context
SCMessenger Android app. File:
`android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`

The Identity screen displays public key, peer ID, identity hash, identicon, and a QR code. User reports the screen does NOT scroll after the QR code image loads (which happens with a visible delay because QR generation is done in a coroutine via `viewModel.getQrCodeData()`  see line 50-56).

## The Bug
- Before QR loads: screen scrolls fine (just text + identicon).
- After QR bitmap renders: scroll locks up, user cannot reach the export/share buttons at the bottom.

## Likely Root Cause
Two issues likely compound:

1. **Scroll modifier on wrong element**: `IdentityContent` has `Modifier.verticalScroll(rememberScrollState())` on a `Column` inside a `Box` (`Scaffold { paddingValues -> Box { ... } }` at lines 75-78). The `Box` does not propagate a fillMaxSize+scroll-friendly height, and `padding(paddingValues)` is applied to the Box, not the Column. The Column tries to scroll within the Box, but the QR's `Modifier.align(Alignment.CenterHorizontally)` inside a scrollable Column can break measurement when the QR bitmap is large.

2. **QR generation blocks layout**: `getQrCodeData()` is called from a `LaunchedEffect` on the Main dispatcher, so it freezes UI thread. Once it returns, the QR Image (a 256256+ bitmap) is added to the scrollable Column, and the recomposition may invalidate the scroll state.

## Acceptance Criteria
- [ ] Screen scrolls smoothly before AND after the QR code image is rendered.
- [ ] QR generation does not block the Main thread.
- [ ] No regression to existing identicon / copy-public-key / export flows.
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` succeeds.
- [ ] Optional: Pre-render the QR (run `getQrCodeData()` on `Dispatchers.Default` IMMEDIATELY at screen open, cache the result in ViewModel so it's ready by the time the user looks at the screen  addresses user's "pre render if possible" comment).

## Implementation Hints

### Quick fix (scroll only)
In `IdentityContent` (line 150-...), change:
```kotlin
Column(
    modifier = Modifier
        .fillMaxSize()
        .verticalScroll(rememberScrollState())
        .padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(24.dp)
)
```
to put `verticalScroll` AFTER `padding` (Modifier order matters) and add `weight(1f)` if inside a `Box`  or better, wrap the whole content in a `Box(modifier = Modifier.fillMaxSize().padding(paddingValues))` and put the `verticalScroll` Column INSIDE that Box with explicit fillMaxSize.

Better: move `.verticalScroll` to the outermost scrollable container. Don't nest scroll.

### Proper fix (scroll + pre-render)
1. In `IdentityViewModel`, add:
   ```kotlin
   private val _qrCodeData = MutableStateFlow<String?>(null)
   val qrCodeData: StateFlow<String?> = _qrCodeData.asStateFlow()

   init {
       viewModelScope.launch(Dispatchers.Default) {
           // Generate QR off the Main thread, store as state
       }
   }
   ```
2. In the Composable, collect from ViewModel directly (no `LaunchedEffect` on Main thread).
3. Apply the scroll modifier fix from above.

## Files to Touch
- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt` (required)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt` (for pre-render option)

## Verification
```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger/android
./gradlew :app:assembleDebug -x lint --quiet
```
Expected: BUILD SUCCESSFUL.

Manual test plan (record for HANDOFF/done/):
1. Open app, navigate to Identity screen
2. Immediately try to scroll  should work
3. Wait for QR to render (typically 1-3s)
4. Scroll again  should still work
5. Scroll to bottom  should reach export buttons
