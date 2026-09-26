# P1_ANDROID_023: Deep Linking for Identity Sharing

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Objective

Enable deep links so users can share their SCMessenger identity via URL (e.g., `https://scmessenger.net/add?public_key=...` or `scmessenger://add?public_key=...`). When another user taps the link, the app opens directly to the Add Contact screen with fields pre-filled.

## Background

The app already has:
- Identity export JSON with `public_key`, `peer_id`, `nickname`, `identity_id`
- `ContactsScreen`  `AddContactDialog` with prefilled fields
- `MainActivity` handles `Intent.ACTION_SEND` for text sharing

What's missing:
- No `<intent-filter>` for `scmessenger://` or `https://scmessenger.net/add` URLs
- `MainActivity` and `MeshApp` do not process `intent.data` or `intent.extras` from deep links
- No routing from deep link to `AddContactScreen` or showing an "Add Contact" dialog

## Implementation Plan

### 1. AndroidManifest.xml
Add an `<intent-filter>` to `MainActivity` for the custom scheme and HTTPS path:
```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="scmessenger" android:host="add" />
</intent-filter>
<intent-filter android:autoVerify="true">
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="https" android:host="scmessenger.net" android:pathPrefix="/add" />
</intent-filter>
```

### 2. MainActivity.kt
In `onCreate`, after `setContent`, check `intent?.action == Intent.ACTION_VIEW` and extract query parameters:
- `public_key` (required)
- `peer_id` / `libp2p_peer_id` (optional)
- `nickname` (optional)
- `identity_id` (optional)

Store these in a `MutableStateFlow` or pass them to `MeshApp` via a ViewModel.

### 3. MeshApp.kt / MainViewModel.kt
Add a `pendingDeepLink` flow in `MainViewModel`. When a deep link is detected:
- If onboarding is not complete: queue it and process after identity creation
- If onboarding is complete: navigate to `AddContactScreen` with prefilled data

### 4. AddContactScreen
Ensure `AddContactScreen` accepts optional prefilled parameters and populates the form.

## URL Format

```
scmessenger://add?
  public_key=64char_hex
  &peer_id=QmBase58PeerId
  &nickname=alice
  &identity_id=blake3_hash
```

HTTPS equivalent:
```
https://scmessenger.net/add?public_key=...&peer_id=...&nickname=...
```

## Acceptance Criteria

1. `adb shell am start -a android.intent.action.VIEW -d "scmessenger://add?public_key=abc123&nickname=alice" com.scmessenger.android` opens the app and shows Add Contact with fields pre-filled
2. Same works for `https://scmessenger.net/add?...`
3. If app is not onboarded, deep link is queued and shown after onboarding completion
4. `./gradlew :app:compileDebugKotlin` passes
5. `./gradlew :app:bundleRelease` passes

## Related

- P1_ANDROID_003 (Identity Import UI)
- P0_ANDROID_PLAYSTORE_001 (compliance)
- P1_ANDROID_RELEASE_001 (build verification)

---

**Priority:** P1
**Type:** Feature / UX
**Estimated LoC Impact:** ~120
**Blocking:** No
