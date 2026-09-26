# TASK: Android  Hardcoded User-Facing Strings in ContactsScreen and SettingsScreen

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

`.claude/rules/android.md` Pre-Merge Checklist mandates: "No hardcoded strings
in UI  all user-facing text in `strings.xml`." A fresh sweep of
`android/app/src/main/java/com/scmessenger/android/` (2026-07-04, independent
of the T8-T13 items already tracked in
`docs/release-readiness-2026-07-02.md` section 7) found two files that
violate this rule extensively, alongside otherwise-compliant sibling code in
the same files (e.g. `SettingsScreen.kt` already uses
`stringResource(R.string.identity_field_unavailable)` a few lines away from
literal `Text("Public Key")`), which shows the rule is being followed
inconsistently rather than never.

Not covered by T8-T13 (those are QR cache/perf, safety-number memoization,
import dialog state leak, UTF-16 length bug, WiFi Aware loopback proxy,
test package name  none touch string localization).

## Scope (confirmed via `grep`, 2026-07-04)

`ui/screens/ContactsScreen.kt`  `AddContactDialog` composable (~lines
596-762), all hardcoded:
- Dialog title: `Text("Add Contact")`
- Buttons: `Text("Paste Identity Export")`, `Text("Scan Contact QR")`,
  `Text("Add")` (x2, one in `AddContactDialog` confirmButton, one in the
  peer-list row at line 590), `Text("Chat")`, `Text("Cancel")`
- Field labels: `Text("Peer ID")`, `Text("Public Key")`,
  `Text("Nickname (Optional)")`
- Error string: `parseError = "Unable to scan QR code."`
- `contentDescription`: `"Add contact"`, `"Paste"`, `"Scan QR code"`

`ui/screens/SettingsScreen.kt`  identity section (~lines 925-1161):
- Field labels: `Text("Nickname")`, `Text("Peer ID (Network)")`,
  `Text("Identity Hash")`, `Text("Public Key")`
- Buttons: `Text("Save Nickname")`, `Text("Copy Full Identity Export")`,
  `Text("Show Identity QR")`, `Text("Import Identity")`,
  `Text("Create Identity")`, `Text("Manage Blocked Peers")`,
  `Text("Privacy Policy")`
- Error string: `Text("Failed to export backup: ${error.message}")`
- `contentDescription`: `"Copy Peer ID"`, `"Copy Identity Hash"`,
  `"Copy Key"`, `"Share identity export"`
- `ClipData.newPlainText` labels (`"Identity Export"`, `"Identity Backup"`,
  `"Peer ID"`, `"Identity Hash"`, `"Public Key"`) are lower severity  these
  are the clipboard *description* metadata, not visible UI text on most
  devices/Android versions  but are visible in the system clipboard
  suggestion strip on some OEM skins and API 33+ clipboard-access
  toasts, so should be localized too while the file is being touched.

`ui/screens/OnboardingScreen.kt:275`  one instance:
`Text("Skip for Relay-Only Install")`.

## Acceptance Criteria

- [ ] Every hardcoded string identified above (and any sibling ones found
      during implementation in the same composables) is moved to
      `strings.xml` with a descriptive key following the existing naming
      convention in that file (e.g. `contacts_dialog_title_add`,
      `settings_label_peer_id_network`).
- [ ] All `Text(...)`, `contentDescription = ...`, and error-message string
      literals in the touched composables are replaced with
      `stringResource(R.string.xxx)`.
- [ ] `ClipData.newPlainText` first-argument labels in `SettingsScreen.kt`
      are also sourced from `strings.xml` (label text does not need to
      match the visible field label 1:1, but should be equally localized).
- [ ] No behavior change  dialogs, buttons, and copy actions work exactly
      as before, just with resource-backed text.
- [ ] `./gradlew assembleDebug -x lint --quiet` succeeds.

## Implementation Plan

1. Grep the three files for the literal patterns above to get exact line
   numbers on the current tree (they may have shifted slightly since this
   audit).
2. Add new `<string name="...">` entries to
   `android/app/src/main/res/values/strings.xml`, grouped near existing
   `contacts_*` / `settings_*` keys for discoverability.
3. Replace each literal with `stringResource(R.string.<key>)`, importing
   `androidx.compose.ui.res.stringResource` if not already imported in that
   file (both files already import it elsewhere, so this should be a
   no-op import-wise).
4. For `ClipData.newPlainText(...)` calls, either inline
   `stringResource(...)` at the call site (these are inside `@Composable`
   scope so this is legal) or hoist to a `val` above the `onClick` lambda
   if Compose complains about calling `stringResource` inside a
   non-composable lambda (likely  `onClick` lambdas are not
   `@Composable`). In that case, resolve the string via
   `stringResource(...)` in the composable body and capture it in a `val`
   that the lambda closes over, mirroring how `identityInfo` fields are
   already captured.
5. Re-check the two files (and `OnboardingScreen.kt`) with the same grep
   patterns used in this audit to confirm zero hits remain:
   ```
   grep -n 'Text("[A-Za-z]' ui/screens/ContactsScreen.kt ui/screens/SettingsScreen.kt ui/screens/OnboardingScreen.kt
   grep -n 'contentDescription = "[A-Za-z]' ui/screens/ContactsScreen.kt ui/screens/SettingsScreen.kt
   ```

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt`
- `android/app/src/main/res/values/strings.xml`

## Verification

```bash
cd android
./gradlew assembleDebug -x lint --quiet
```
Expected: BUILD SUCCESSFUL. Then re-run the grep commands in step 5 above
and confirm no matches.

## Related

- Not a duplicate of T8-T13 (`docs/release-readiness-2026-07-02.md` 7
  Android)  those cover QR perf, safety-number memoization, import dialog
  state leak, UTF-16 length, WiFi Aware loopback, test package name. None
  touch string localization.

---
**CLOSED 2026-07-06 (orchestrator): DUPLICATE of ANDROID_SWEEP_01_hardcoded_strings_contacts_settings.md**, which is the superset (same Contacts/Settings/Onboarding scope plus NetworkStatusDialog.kt and the 8-file contentDescription sweep). SWEEP_01 is the survivor and was dispatched to an agy/Gemini foreign worker. The only divergence: this ticket asked to localize ClipData.newPlainText labels; SWEEP_01 deliberately leaves them (system clipboard metadata, not user-visible UI) - accepted as the correct scope.
