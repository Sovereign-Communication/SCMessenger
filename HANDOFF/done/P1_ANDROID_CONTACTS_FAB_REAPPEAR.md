# TASK: Fix Android ContactsScreen "+" FAB Disappeared (Add Contact)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Agent Role
Agent 4: Android UI Bug Fix (single-screen, high-impact, P1)

## User Report (verbatim)
> "contact add button disappeared - add '+' in contact tab for manual entry"

## Context
File: `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`

The FloatingActionButton (FAB) IS defined in code at lines 74-78:
```kotlin
floatingActionButton = {
    FloatingActionButton(onClick = onNavigateToAddContact) {
        Icon(Icons.Default.Add, contentDescription = stringResource(R.string.contacts_action_add))
    }
}
```

And the call site in `MeshApp.kt:157-163` correctly passes:
```kotlin
onNavigateToAddContact = {
    navController.navigate(Screen.AddContact.route)
}
```

**So the code is present. The FAB is invisible/broken at runtime.** Most likely cause: **double/nested Scaffold**  `MeshApp.kt:121` wraps the NavHost in a `Scaffold`, and `ContactsScreen` adds its OWN `Scaffold` inside. Nested Scaffolds cause FAB positioning and padding issues (inner FAB can be hidden behind outer Scaffold's bottom bar, or positioned outside the visible area).

The `AddContactScreen` itself supports manual entry (QR + manual + nearby)  so the fix is just making the button reachable.

## Acceptance Criteria
- [ ] FAB (the circular "+" button) is visible on the Contacts tab at all times.
- [ ] Tapping the FAB navigates to `AddContactScreen` (which already has a manual entry tab).
- [ ] Manual entry of peer_id + public_key works end-to-end and persists.
- [ ] No regression to existing Nearby Peers, swipe-to-delete, or contact detail flows.
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` succeeds.

## Implementation Plan (try in order)

### Attempt 1  Remove the inner Scaffold (preferred)
In `ContactsScreen.kt`, change lines 68-245: remove the outer `Scaffold { paddingValues -> ... }` and inline the TopAppBar + content directly. Move the FAB to the **outer** Scaffold in `MeshApp.kt` (or use a global FAB pattern via a Composable wrapper that has its own Scaffold). Quick diff:

```kotlin
// BEFORE (ContactsScreen.kt line 68):
Scaffold(
    topBar = { TopAppBar(...) },
    floatingActionButton = { FloatingActionButton(...) }
) { paddingValues -> Column { ... } }

// AFTER:
Column(modifier = Modifier.fillMaxSize().padding(paddingValues)) { ... }
// TopAppBar is now in the outer Scaffold in MeshApp
// FAB moves to MeshApp's Scaffold keyed by current route == "contacts"
```

Then in `MeshApp.kt`:
```kotlin
Scaffold(
    floatingActionButton = {
        if (currentRoute == "contacts") {
            FloatingActionButton(onClick = { navController.navigate(Screen.AddContact.route) }) {
                Icon(Icons.Default.Add, contentDescription = "Add contact")
            }
        }
    }
) { paddingValues -> NavHost(...) }
```

### Attempt 2  If outer Scaffold is needed for something else
Use `Box(Modifier.fillMaxSize())` inside the inner Scaffold, and explicitly position the FAB using `Modifier.align(Alignment.BottomEnd).padding(16.dp)` instead of relying on `floatingActionButton` slot.

### Attempt 3  Add a fallback entry point
If the FAB can't be made to work for some reason, ALSO add a "+ Add" `TextButton` to the top app bar's `actions` slot, or a button inside the empty-state Box (lines 127-159). The user gets manual entry either way.

## Files to Touch
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` (required)
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt` (required if Attempt 1)

## Verification
```bash
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger/android
./gradlew :app:assembleDebug -x lint --quiet
```
Expected: BUILD SUCCESSFUL.

Manual test:
1. Install fresh APK on Pixel 6a (or current test device)
2. Open app  tap Contacts tab
3. FAB should be visible at bottom-right of the screen
4. Tap FAB  AddContactScreen opens with Manual/QR/Nearby tabs
5. Manual tab: enter peer_id (any 32+ char hex) + public_key (any 32+ char hex) + nickname  tap Add  returns to Contacts list with the new contact visible

## Related
- The `AddContactScreen.kt` already supports manual entry  no changes needed there for THIS task. (Though if manual entry has bugs, file a separate task.)
- Sibling task: `P1_ANDROID_LAN_DISCOVERY_REPAIR`  addresses why no nearby peers show up, which compounds the empty Contacts screen problem.
