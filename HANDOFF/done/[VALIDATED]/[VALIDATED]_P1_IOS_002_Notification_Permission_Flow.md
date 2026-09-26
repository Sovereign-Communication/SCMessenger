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
# MODEL: gemma4:31b:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_IOS_002_Notification_Permission_Flow

**Status:** VERIFIED REMAINING WORK
**Agent:** worker
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1 iOS verification
**Source:** HANDOFF/backlog/P1_IOS_002_NOTIFICATION_VERIFICATION.md + planfromclaudeforhermes 2 Phase E.3
**Depends on:** P1_IOS_001 (smoke test plan establishes test framework)

---

## Verified Gap

Per `HANDOFF/backlog/P1_IOS_002_NOTIFICATION_VERIFICATION.md`: "iOS notifications code exists and is documented as complete. `NotificationManager.swift` implementation is ready. Marked as ' Needs verification' in the tracking document. Requires testing on physical iOS devices."

`iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift` exists with `verifyPermissionFlow()` async function per the backlog. The gaps are: (1) denied-state recovery, (2) settings deep-link, (3) iOS 17+ API compatibility verification.

## Scope (~80 LoC across 1-2 files)

### Part A: Denied-state recovery (LOC: ~40)

In `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift`:

```swift
func handleDeniedState() {
    guard let settingsURL = URL(string: UIApplication.openSettingsURLString) else { return }
    
    let alert = UIAlertController(
        title: "Notifications Disabled",
        message: "To receive message alerts, enable notifications in Settings.",
        preferredStyle: .alert
    )
    alert.addAction(UIAlertAction(title: "Open Settings", style: .default) { _ in
        UIApplication.shared.open(settingsURL)
    })
    alert.addAction(UIAlertAction(title: "Later", style: .cancel))
    
    // Find the active window's root view controller
    if let root = UIApplication.shared.connectedScenes
        .compactMap({ ($0 as? UIWindowScene)?.keyWindow }).first?.rootViewController {
        // Find the topmost presented controller
        var top = root
        while let presented = top.presentedViewController { top = presented }
        top.present(alert, animated: true)
    }
}
```

### Part B: Permission rationale view (LOC: ~40)

Create `iOS/SCMessenger/SCMessenger/Views/PermissionRationaleView.swift` (NEW if doesn't exist):

```swift
struct PermissionRationaleView: View {
    let onRequest: () -> Void
    let onSkip: () -> Void
    
    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: "bell.badge")
                .font(.system(size: 64))
                .foregroundColor(.accentColor)
            Text("Stay Connected")
                .font(.title)
            Text("Enable notifications to know when you receive messages, even when the app is in the background.")
                .multilineTextAlignment(.center)
                .padding(.horizontal)
            HStack {
                Button("Not Now", action: onSkip)
                    .buttonStyle(.bordered)
                Button("Enable", action: onRequest)
                    .buttonStyle(.borderedProminent)
            }
        }
        .padding()
    }
}
```

Add to `OnboardingFlow.swift` as a step between identity creation and main app.

## File Targets

- `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift` [EDIT  add handleDeniedState]
- `iOS/SCMessenger/SCMessenger/Views/PermissionRationaleView.swift` [NEW]
- `iOS/SCMessenger/SCMessenger/OnboardingFlow.swift` [EDIT  add rationale view as step]

## Build Verification Commands

```bash
# On macOS only
cd iOS
xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Debug -sdk iphonesimulator
# Or for device:
# xcodebuild -workspace SCMessenger.xcworkspace -scheme SCMessenger -configuration Debug -sdk iphoneos -destination 'platform=iOS,name=iPhone 15'
```

## Acceptance Gates

1. `xcodebuild` for simulator passes (CI check)
2. New unit test: `NotificationManagerTests` covers authorized, denied, provisional, ephemeral states
3. Manual: deny permission  see "Open Settings" alert  tap  settings app opens
4. Manual: iOS 17+ provisional authorization requested for quieter first-time UX
5. Commit: `ios: v0.2.1 notification permission flow  denied recovery + rationale view`

## REQUIRES_USER_ACTION
User must build on macOS and verify on real device. Windows host cannot run xcodebuild.

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: SWIFT] [REQUIRES: GEMMA_4_31B] [DEPENDS_ON: P1_IOS_001] [REQUIRES_MACOS_TO_BUILD]
