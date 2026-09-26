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

# P2_ANDROID_IDENTITY_QR_PRERENDER_AND_SCROLL

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P2 Android polish
**Source:** Tickets `HANDOFF/todo/P2_ANDROID_IDENTITY_QR_PRERENDER.md` and `HANDOFF/todo/P2_ANDROID_IDENTITY_SCROLL_FIX.md` (same root cause, same file pair)
**Depends on:** none
**Branch:** fix/p2-android-identity-qr-prerender (off origin/main dd109707)
**Worktree:** create a new worktree at `E:\SCMessenger-build-p2-qr\` based on origin/main
**Assignee:** worker

---

## Verified Gap (combined from two tickets)

User reported: "won't scroll after the QR loads (which is slow - pre render if possible)"

Two issues, same root cause, same fix:

1. **QR generation blocks Main thread**  `getQrCodeData()` is called from a `LaunchedEffect` on the Main dispatcher (line 50-56 of `IdentityScreen.kt`). This freezes the UI thread and the user sees an empty space where the QR will appear.
2. **Scroll locks up after QR loads**  `IdentityContent` has `Modifier.verticalScroll(rememberScrollState())` on a `Column` inside a `Box` (Scaffold  Box  Column structure). The QR Image is added to the scrollable Column and the recomposition invalidates the scroll state.

## Scope (~80 LoC across 2 files)

### Part A: ViewModel pre-warm (LOC: ~30)

In `E:\SCMessenger-build-p2-qr\android\app\src\main\java\com\scmessenger\android\ui\identity\IdentityViewModel.kt`:

Add a `StateFlow<String?>` for QR data and pre-warm it on init:

```kotlin
private val _qrCodeData = MutableStateFlow<String?>(null)
val qrCodeData: StateFlow<String?> = _qrCodeData.asStateFlow()

init {
    viewModelScope.launch(Dispatchers.Default) {
        if (identityInfo.value?.initialized == true) {
            val data = getQrCodeData()  // existing suspend or wrap in withContext
            _qrCodeData.value = data
        }
    }
}

fun refreshQrCode() {
    viewModelScope.launch(Dispatchers.Default) {
        _qrCodeData.value = getQrCodeData()
    }
}
```

Read the actual `IdentityViewModel.kt` first to match its style and existing identity-loading logic. Adjust names if it already has different state holders (e.g. `identityInfo` may be named differently  search for the actual field).

### Part B: Composable observes StateFlow + scroll fix (LOC: ~50)

In `E:\SCMessenger-build-p2-qr\android\app\src\main\java\com\scmessenger\android\ui\identity\IdentityScreen.kt`:

1. Replace the `LaunchedEffect` + local `remember { mutableStateOf<String?>(null) }` QR loading (lines 49-56) with:
   ```kotlin
   val qrCodeData by viewModel.qrCodeData.collectAsState()
   ```
2. Fix the scroll modifier. The current structure is `Scaffold { paddingValues -> Box { ... Column(modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(paddingValues)) } }`. The `padding(paddingValues)` on the Box outside the Column is wrong for scroll behavior. Change to:
   - The `Box` outside should have `Modifier.fillMaxSize().padding(paddingValues)` (no scroll on it)
   - The `Column` inside should have `Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(16.dp)` (scroll on the inner Column, with content padding)
   - The QR Image's `Modifier.align(Alignment.CenterHorizontally)` should be removed (use a wrapper Box if centering is needed, but inside a scrollable Column the alignment is by default horizontal)
3. Cite both P2 tickets in a comment at the top of the file.

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityViewModel.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt` [EDIT]

## Build Verification Commands

```bash
# In the WSL build copy
export JAVA_HOME=/home/scemessenger/.local/jdk/jdk-17.0.12+7
export ANDROID_NDK_HOME=/home/scemessenger/android-sdk/ndk/26.1.10909125
export CARGO_TARGET_DIR=/home/scemessenger/.cargo-target
export CARGO_INCREMENTAL=0
export GRADLE_USER_HOME=/home/scemessenger/.gradle
cd /home/scemessenger/scmessenger-build/android
./gradlew :app:assembleDebug -x lint --no-daemon --offline
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` succeeds
2. QR generation happens on `Dispatchers.Default`, not Main
3. Screen scrolls smoothly BEFORE and AFTER the QR loads
4. No regression: existing refresh button, error handling, copy-to-clipboard still work
5. NO commits, NO pushes  leave changes uncommitted for user review
6. The worktree at `E:\SCMessenger-build-p2-qr\` must be left ready for `git diff` review

## Out of scope

- DO NOT commit. The user owns the commit step.
- DO NOT push. The user owns the push step.
- DO NOT touch any of the other worktrees (`E:\SCMessenger-Github-Repo\SCMessenger\`, `E:\SCMessenger-build-p0-024\`).
- DO NOT touch `main` or `origin/main`.
- DO NOT install the APK. User installs manually.
- DO NOT touch the BLE / onboarding / repository tickets  those are separate.

## OODA Reporting

After completing the work, the agent must report:
- List of files modified and the diff stat
- Confirmation that the build succeeded (or full error if it failed)
- Any deviations from this spec (e.g. if `getQrCodeData()` is not suspend and needs to be wrapped in `withContext(Dispatchers.Default)`)

If the build fails, stop and report. Do not silent-fix.
