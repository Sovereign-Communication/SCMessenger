# BATCH: Android UI Wiring  Priority 1 (12 tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are a worker implementing Android/Kotlin wiring tasks. Each task requires you to:
1. Find the target Composable function in the specified file
2. Identify where it should be called in the app's navigation/UI graph
3. Wire it into the production call path (add navigation routes, import statements, etc.)
4. Verify compilation with `cd android && ./gradlew assembleDebug -x lint --quiet`
5. Move each completed task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After ALL wiring is done, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

## Android-Specific Rules
- All user-facing text MUST come from strings.xml  no hardcoded strings
- Navigation routes should follow the existing NavHost pattern in the app
- Import paths must match the project's package structure: `com.scmessenger.android.ui.*`
- Use Hilt DI where appropriate (`@HiltViewModel`, `@AndroidEntryPoint`)

## UI Component Tasks (Compose Screens & Components)

### Screen-Level Components (wire into navigation graph)

1. **task_wire_ContactDetailScreen.md**  `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt`  Wire ContactDetailScreen into the navigation graph. It should be navigable from the Contacts list.
2. **task_wire_MeshSettingsScreen.md**  MeshSettingsScreen  Wire into Settings navigation. This is a sub-screen of the main Settings.
3. **task_wire_PeerListScreen.md**  PeerListScreen  Wire into the mesh/network navigation. Shows connected peers.
4. **task_wire_PowerSettingsScreen.md**  PowerSettingsScreen  Wire into Settings. Controls battery/power optimization settings.
5. **task_wire_TopologyScreen.md**  TopologyScreen  Wire into mesh/network navigation. Shows mesh topology visualization.

### Reusable UI Components (wire into their parent screens/layouts)

6. **task_wire_MessageInput.md**  `android/app/src/main/java/com/scmessenger/android/ui/chat/MessageInput.kt`  Wire MessageInput into the Chat/Conversation screen where messages are composed.
7. **task_wire_ErrorState.md**  ErrorState composable  Wire into screens that need error display (ConversationsScreen, DashboardScreen, etc.).
8. **task_wire_IdenticonFromHex.md**  IdenticonFromHex  Wire into contact list items, conversation headers, anywhere a contact avatar is shown.
9. **task_wire_InfoBanner.md**  InfoBanner  Wire into screens that need informational banners (Settings, Dashboard, etc.).
10. **task_wire_LabeledCopyableText.md**  LabeledCopyableText  Wire into Settings, ContactDetail, or any screen showing key fingerprints/identifiers.
11. **task_wire_TruncatedCopyableText.md**  TruncatedCopyableText  Wire into message list items, contact cards, anywhere long text needs truncation with copy ability.
12. **task_wire_WarningBanner.md**  WarningBanner  Wire into screens needing warning displays (Settings, onboarding, permissions).

## Execution Strategy

1. Start with screen-level components (tasks 1-5)  these require navigation graph changes
2. Then reusable components (tasks 6-12)  these replace placeholder UI in existing screens
3. After each component is wired, move its task file to HANDOFF/done/
4. Run `./gradlew assembleDebug -x lint --quiet` after every 3-4 tasks to catch issues early
5. Final verification build must pass clean

## Navigation Graph Pattern
The app uses Jetpack Compose Navigation. Follow the existing pattern in the NavHost setup:
```kotlin
composable("route_name") { backStackEntry ->
    ComponentScreen(viewModel = hiltViewModel())
}
```

When you've completed all 12 tasks and the build passes, report STATUS: SUCCESS_STOP


# REPO_MAP Context for Task: BATCH_ANDROID_WIRING_P1
