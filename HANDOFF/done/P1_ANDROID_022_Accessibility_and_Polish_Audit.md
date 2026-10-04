# P1_ANDROID_022: Accessibility and Polish Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Objective

Pass a basic accessibility scan and migrate deprecated APIs to align with Material 3 standards.

## Tasks

1. **Content Descriptions**
   - Audit all `Icon(..., contentDescription = null)` calls across the app
   - Add meaningful descriptions for interactive icons (copy, share, block, delete, edit)
   - Keep `contentDescription = null` for purely decorative icons only

2. **Material 3 Migration**
   - Replace `androidx.compose.material.Divider` with `androidx.compose.material3.HorizontalDivider` in:
     - `SettingsScreen.kt` (lines 318, 616, 646)
     - `ContactsScreen.kt`
     - Any other files
   - Migrate `androidx.compose.material.SwipeToDismiss` to `androidx.compose.material3.SwipeToDismissBox` in:
     - `ContactsScreen.kt`
     - `ConversationsScreen.kt`

3. **Color Contrast**
   - Verify `errorContainer.copy(alpha = 0.2f)` in `DataManagementSection` meets WCAG 4.5:1
   - Verify splash screen background `#FF1A1A2E` with white foreground meets contrast

4. **Text Scaling**
   - Ensure no hardcoded `Text(maxLines = 1)` truncates user-facing text without ellipsis
   - Ensure `OutlinedTextField` labels remain visible at 200% font scale

## Verification

Run Android Studio Layout Inspector with TalkBack enabled on the Pixel 6a, or use:
```bash
adb shell settings put secure enabled_accessibility_services com.google.android.marvin.talkback/com.google.android.marvin.talkback.TalkBackService
```

## Related

- P1_ANDROID_UI_001 (Chat empty/loading states)
- P1_ANDROID_003 (Identity import UI)

---

**Priority:** P1
**Type:** Polish / Compliance
**Estimated LoC Impact:** ~150
