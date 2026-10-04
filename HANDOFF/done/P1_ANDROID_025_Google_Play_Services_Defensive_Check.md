# P1_ANDROID_025: Google Play Services Defensive Check for QR Scanning

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Objective

Add a defensive check for Google Play Services availability before using ML Kit's barcode scanner. On devices without GMS (Huawei, custom ROMs), the app currently crashes with a `GoogleApiAvailability` exception when the user taps "Scan QR Code".

## Background

`AddContactScreen.kt` uses `GmsBarcodeScanning.getClient()` without checking if Google Play Services is available. ML Kit's code scanner is a GMS-dependent API.

## Implementation Plan

### 1. `AddContactScreen.kt`  QRScanTab composable
Before calling `GmsBarcodeScanning.getClient()`, check availability:

```kotlin
val googleApiAvailability = GoogleApiAvailability.getInstance()
val result = googleApiAvailability.isGooglePlayServicesAvailable(context)
if (result != ConnectionResult.SUCCESS) {
    // Show error: "Google Play Services is required for QR scanning"
    qrError = "Google Play Services is not available on this device. Please use manual entry or paste the identity string."
    return@Button
}
```

Import needed:
```kotlin
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
```

### 2. Alternative: Disable QR button when GMS is unavailable
Instead of showing an error on click, disable the "Scan QR Code" button and show a helper text:
```kotlin
val isGmsAvailable = remember {
    GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS
}
Button(
    onClick = { /* scan */ },
    enabled = isGmsAvailable
) { ... }
if (!isGmsAvailable) {
    Text("QR scanning requires Google Play Services", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
}
```

## Acceptance Criteria

1. App does not crash on non-GMS devices when user attempts QR scan
2. User sees a helpful message explaining why QR scanning is unavailable
3. Manual entry and paste options remain fully functional
4. `./gradlew :app:compileDebugKotlin` passes

## Related

- P1_ANDROID_003 (Identity Import UI)
- P1_ANDROID_RELEASE_001 (build verification)

---

**Priority:** P1
**Type:** Defensive / Crash Prevention
**Estimated LoC Impact:** ~20
**Blocking:** No
