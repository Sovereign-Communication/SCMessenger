# TASK: Add Contact screen has 2 bugs - button hidden on small screens, save doesn't persist

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO. Found 2026-07-11 while manually adding a test contact for the
Windows-CLI<->Android-emulator live delivery test. Both confirmed via direct
device interaction (adb + uiautomator), not speculation.

## Bug 1: Add Contact submit button hidden behind bottom nav on small screens

`android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
~line 201-294: the form's `Column` has `.verticalScroll(rememberScrollState())`,
with the "Add" `Button` (line 278) as the LAST element after Peer ID/Public
Key/Nickname/Notes fields. On the test AVD (320x640 logical resolution,
160dpi - confirmed via `adb shell wm size`/`wm density`), the scrollable
content's natural height exceeds the visible viewport by roughly one
button's-height, and the Column's scroll state reaches its maximum WITHOUT
ever fully revealing the button - it renders at/past the boundary where the
app's persistent bottom navigation bar (Chats/Contacts/Mesh/Settings) is
drawn, and the nav bar wins touch dispatch in that overlapping region.
Confirmed workaround: `adb shell wm size 320x960` (taller virtual display)
immediately reveals the button as a normal, tappable element - proving this
is a viewport/scroll-math issue, not a fundamentally broken button.

**Likely cause:** the outer `Scaffold` (wherever this screen is hosted,
check `MeshApp.kt` navigation) may not be passing its `bottomBar`
`PaddingValues` into this screen's content padding, so the scrollable
Column doesn't know it needs extra bottom space equal to the nav bar's
height - or the Column's `.verticalScroll` doesn't get a bottom
`contentPadding`/`Spacer` matching `WindowInsets` for the nav bar.

**Fix direction:** ensure `AddContactScreen`'s scrollable content reserves
bottom padding for the Scaffold's `innerPadding`/bottom bar height (or a
`Spacer(Modifier.navigationBarsPadding())` at the end of the Column), and
add a regression test/manual check on a small-screen device profile
specifically (this AVD's 320x640 config is a good repro case - don't assume
larger/default test devices will catch it).

## Bug 2: Contact save silently doesn't persist

After correctly filling both required fields (Peer ID, Public Key - verified
via uiautomator dump showing exact expected text in both fields) and tapping
the confirmed "Add Contact" button (verified via `content-desc="Add
Contact"` at the exact tapped coordinates), the app navigates back to the
Contacts list as if successful, but the list shows "Contacts (0)" / "No
contacts yet" - both immediately and after navigating away/back (ruling out
a simple stale-cache display issue). Direct inspection of
`/data/data/com.scmessenger.android/files/contacts.db` (via `adb shell
run-as ... ls -la`) showed the underlying `db` file's modification time
was NOT updated by the save attempt (older than the attempt itself) -
confirming the write genuinely never reached persistent storage, not just a
UI refresh gap.

**Investigate:** trace `onAdd` callback wiring from `AddContactScreen`
through to whatever ViewModel/repository method actually persists a new
contact (likely in `ContactsViewModel.kt` per earlier grep results this
session) - check for a silently-swallowed exception, a validation check
that fails without user-visible feedback, or a coroutine/async call whose
result is never awaited/observed. `adb logcat` around the tap showed
nothing under simple contact/error/exception filters - may need broader
logcat capture or added logging to pinpoint exactly where the flow drops
the write.

## Gate

Standard Android gates (`./gradlew assembleDebug`, existing instrumented
tests if any cover contact add). Manual verification: add a contact via UI
on the actual small-screen AVD profile, confirm `contacts.db`'s mtime
updates and the list shows "Contacts (1)".
