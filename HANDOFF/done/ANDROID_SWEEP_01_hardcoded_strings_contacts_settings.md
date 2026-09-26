# TASK: Android  Extract Hardcoded UI Strings in ContactsScreen/SettingsScreen/others to strings.xml

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Independent sweep of `android/app/src/main/java/com/scmessenger/android/` (2026-07-04),
scoped explicitly to find gaps NOT already covered by
`docs/release-readiness-2026-07-02.md` (T8-T13) or the existing HANDOFF backlog.
T8-T13 cover QR composable perf, safety-number memoization, import dialog state
leak, UTF-16 length bug, WiFi Aware loopback defects, and a test package name
mismatch  none of them touch string externalization.

`.claude/rules/android.md` Pre-Merge Checklist states: "No hardcoded strings in
UI  all user-facing text in `strings.xml`." This is currently violated in
several screens. Grep evidence (`Text("...")` literals, excluding
`stringResource`/`R.string` usages):

```
14 ui/screens/SettingsScreen.kt
10 ui/screens/ContactsScreen.kt
 3 ui/dialogs/NetworkStatusDialog.kt
 1 ui/screens/OnboardingScreen.kt
```

Plus hardcoded `contentDescription` string literals (also user-facing, read by
TalkBack/accessibility services) in at least: `CopyableText.kt`,
`StorageWarningBanner.kt`, `MeshApp.kt`, `ChatScreen.kt`, `ContactsScreen.kt`,
`ConversationsScreen.kt`, `RequestsInboxScreen.kt`, `SettingsScreen.kt`.

The worst offender is `AddContactDialog` in `ContactsScreen.kt` (lines
~596-745): the entire "Add Contact" dialog  title, both action buttons,
paste/scan buttons, and all three text field labels  is hardcoded English
with zero `stringResource` calls. This dialog is also completely
non-localizable today, unlike the rest of the app which does use
`strings.xml` properly (confirmed: most other screens already call
`stringResource(R.string....)`, so this is a localized gap, not a
project-wide practice).

## Confidence

Ready-to-implement. This is mechanical string extraction with no product
decision involved  the target strings and their English values are already
fully determined by the existing hardcoded text.

## Acceptance Criteria
- [ ] No literal user-facing `Text("...")` string arguments remain in
      `ContactsScreen.kt`, `SettingsScreen.kt`, `NetworkStatusDialog.kt`,
      `OnboardingScreen.kt` (the four files above the count threshold).
- [ ] No literal `contentDescription = "..."` remain in the eight files
      listed above.
- [ ] All new strings added to
      `android/app/src/main/res/values/strings.xml` with names following the
      existing naming convention in that file (e.g. `contacts_*`,
      `settings_*`, `dialog_*` prefixes  check existing keys for the
      established pattern before adding new ones).
- [ ] `./gradlew assembleDebug -x lint --quiet` succeeds.
- [ ] No functional/behavioral change  this is a pure string externalization
      pass.

## Implementation Plan

1. Enumerate every hardcoded string precisely:
   ```bash
   grep -rn 'Text("' android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt
   grep -rn 'Text("' android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt
   grep -rn 'Text("' android/app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt
   grep -rn 'Text("' android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt
   grep -rnE 'contentDescription\s*=\s*"[A-Za-z]' android/app/src/main/java/com/scmessenger/android/ui/
   ```
2. For each literal, add a `<string name="...">...</string>` entry to
   `android/app/src/main/res/values/strings.xml`, grouped near related
   existing keys (e.g. contact-dialog strings near other `contacts_*` keys).
   Use format placeholders (`%1$s`) rather than string concatenation where the
   original code built strings dynamically (none of the ones found do, but
   verify for `NetworkStatusDialog.kt`'s `"  - $domain"` /
   `"  - $relay"` / `"  - $rec"` lines  these need a formatted string
   resource, e.g. `network_status_domain_item` = `"  - %1$s"`).
3. Replace each literal in the Kotlin source with
   `stringResource(R.string.<new_key>)` (add
   `import androidx.compose.ui.res.stringResource` if the file doesn't already
   have it  check first, several of these files already import it for other
   strings).
4. Do not touch `Text("Failed to export backup: ${error.message}")` in
   `SettingsScreen.kt:419` by simple literal replacement of the whole
   string  that one has a dynamic `error.message` suffix; use a formatted
   string resource (`%1$s`) the same way.
5. Leave `ClipData.newPlainText("Identity Export", export)` and similar
   `ClipData` label arguments alone  those are the system clipboard's
   internal label, not user-visible UI text, and are out of scope for this
   rule.

## Files to Touch
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/components/CopyableText.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/components/StorageWarningBanner.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/RequestsInboxScreen.kt`
- `android/app/src/main/res/values/strings.xml`

## Verification
```bash
cd android
./gradlew assembleDebug -x lint --quiet
grep -rn 'Text("' app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt \
  app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt \
  app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt \
  app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt
# expect: no output (all remaining Text() calls use stringResource/variables, not string literals)
```
Manually smoke-test the Add Contact dialog, Settings identity/nickname
section, and Network Status dialog on a device/emulator to confirm the UI
text renders identically.
