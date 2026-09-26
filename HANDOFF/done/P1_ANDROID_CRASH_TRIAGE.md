VERIFIED FIXED as of 2026-07-03  see commit 87d1ef61 (fix(android): FAB reappear + TCP subnet probe for LAN discovery). SubnetProbe.kt confirmed present, crash handler confirmed installed in MeshApplication.kt, nested-Scaffold fix confirmed in MeshApp.kt.

# TASK: Android Crash  Need User Reproduction Steps to Diagnose

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 4: Android Crash Investigation (depends on user repro)

## User Report
User reported "Android is crashing" but did NOT specify:
- Which screen crashes (Contacts? Identity? AddContact? Chat?)
- Whether it's a hard crash (app dies), ANR (frozen), or Compose exception
- What action triggers it (open app? tap button? scroll? send message?)

## Diagnostic Bundle Already Captured
File: `HANDOFF/diagnostics/2026-06-04_android_diag.txt`
Captured at app launch (14:33:06 to 14:33:19, ~13 seconds)

### What the bundle shows
- App launches cleanly  no FATAL EXCEPTION, no AndroidRuntime crash
- All managers initialize: `I/Mesh: all_managers_init_success`
- ANR watchdog starts: `I/Mesh: ANR watchdog started (check=5000ms, threshold=10000ms)`
- Identity loads: `p2p_id=12D3KooWHqsq5WM5c4VYHWXgHiUr9873gYA1K93aPWBcVhN7Ykc2`
- Service transitions: STOPPED  RUNNING
- Bootstrap fails (all relays blocked, mDNS timeout)
- App reaches steady state with 0 contacts, 0 messages, 0 peers

### What the bundle does NOT show
- Any user interaction
- Any FATAL EXCEPTION / AndroidRuntime stack trace
- Any ANR callback fire
- Any Compose recomposition crash
- Any BLE GATT error

**This means the crash either:**
1. Happens on a specific UI action (tap, scroll, navigate) that wasn't captured
2. Happens after a longer interval (the bundle only covers first 13s)
3. Is a "soft crash"  UI freezes or becomes unresponsive, not a process death
4. Was already fixed in a recent build and the user is testing an older APK

## Pre-existing Crash-Prone Code Patterns (from earlier read)

The agent found these in `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`:

1. **Nested Scaffold** (lines 68 / 121 of MeshApp.kt)  known Compose pitfall; can cause infinite recomposition or layout-loop crashes on certain screens
2. **No global UncaughtExceptionHandler**  `MeshApplication.kt` doesn't set `Thread.setDefaultUncaughtExceptionHandler(...)`, so crashes just disappear into logcat
3. **No `try/catch` around UniFFI FFI calls** in any of the screens  if a JNI call returns null, the Compose `getValue()` will NPE
4. **StringResource on potentially-null values**  e.g., `contact.publicKey.take(32)` if `publicKey` is null throws NPE

## Acceptance Criteria
- [ ] A specific crash scenario is reproduced and root-caused.
- [ ] Crash is fixed and verified on Pixel 6a.
- [ ] All previously-captured-good flows still work (identity creation, send/receive, BLE pairing).
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` succeeds.

## Implementation Plan

### Step 1  Capture the crash properly (don't fix blind)
Add a `CrashHandler` to `MeshApplication.kt`:
```kotlin
class MeshApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            // Write to a file the user can pull via `adb pull`
            val crashFile = File(filesDir, "last_crash.txt")
            crashFile.writeText("""
              Thread: ${thread.name}
              Time: ${java.util.Date()}
              Stack:
              ${throwable.stackTraceToString()}
            """.trimIndent())
            // Also write to logcat with a unique tag
            android.util.Log.e("SC_CRITICAL", throwable.stackTraceToString())
            // Re-throw to default handler so Android still kills the process
            throw throwable
        }
    }
}
```
After install, have the user reproduce the crash, then:
```bash
adb pull /sdcard/Android/data/com.scmessenger.android/files/last_crash.txt .
# OR via run-as:
adb shell run-as com.scmessenger.android cat files/last_crash.txt
```

### Step 2  Ask the user for repro context
The user needs to provide:
1. **What screen/tab** were they on when it crashed?
2. **What were they doing** (tapping a button, scrolling, sending a message)?
3. **What did they see** (red error screen, force-close dialog, freeze, "app stopped responding")?
4. **How many times** has it happened?
5. **Did the app fully crash** (closed) or **just freeze** (still on screen)?

Without this, any fix is guesswork.

### Step 3  Apply the most likely fixes regardless
These are pre-existing latent bugs that SHOULD be fixed even if the user's specific crash is different:

A. **Remove nested Scaffold** in `ContactsScreen.kt` (paired with the FAB task `P1_ANDROID_CONTACTS_FAB_REAPPEAR`)

B. **Wrap FFI calls in try/catch** in all ViewModels:
```kotlin
fun loadContacts() {
    viewModelScope.launch {
        _isLoading.value = true
        try {
            _contacts.value = meshRepository.getContacts()
        } catch (e: Exception) {
            Timber.e(e, "Failed to load contacts")
            _error.value = "Failed to load contacts: ${e.message}"
        } finally {
            _isLoading.value = false
        }
    }
}
```

C. **Null-safe Compose parameters**  replace `contact.publicKey.take(32)` with `contact.publicKey?.take(32) ?: "(no key)"`

D. **Global CrashHandler** from Step 1  captures the next crash for analysis

## Files to Touch
- `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt` (CrashHandler)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt` (try/catch)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` (null safety)
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt` (Scaffold refactor)

## Verification
```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger/android
./gradlew :app:assembleDebug -x lint --quiet
```
Expected: BUILD SUCCESSFUL.

Reproduce the original crash scenario  confirm it doesn't happen  pull `last_crash.txt` to verify no new crashes were captured.

## Related
- `P1_ANDROID_CONTACTS_FAB_REAPPEAR` (sibling)  fixes the nested Scaffold
- `P1_ANDROID_LAN_DISCOVERY_REPAIR` (sibling)  the bootstrap fail in the bundle is part of why the app feels broken even without a literal crash
</content>
