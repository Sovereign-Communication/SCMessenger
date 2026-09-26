# P1_IOS_002: iOS Notification Verification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1 (High)
**Platform:** iOS
**Status:** Open
**Routing Tags:** [REQUIRES: UI_VISION] [REQUIRES: FINALIZATION]

## Objective
Comprehensive verification of iOS notification functionality including permission flows, delivery reliability, and user interaction handling. Notifications exist in code but need real-world testing on physical devices.

## Background
From REMAINING_WORK_TRACKING.md:
- iOS notifications code exists and is documented as complete
- `NotificationManager.swift` implementation is ready
- Marked as " Needs verification" in the tracking document
- Requires testing on physical iOS devices

## Implementation Plan

### 1. Permission Flow Testing
**File:** `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift`
```swift
func verifyPermissionFlow() async -> Bool {
    do {
        let settings = await UNUserNotificationCenter.current().notificationSettings()
        
        switch settings.authorizationStatus {
        case .notDetermined:
            // Request permission
            let granted = try await requestNotificationPermission()
            return granted
            
        case .denied:
            // Show guidance for denied permissions
            showPermissionGuidance()
            return false
            
        case .authorized, .provisional, .ephemeral:
            // Already granted
            return true
            
        @unknown default:
            return false
        }
    } catch {
        print("Permission verification failed: \(error)")
        return false
    }
}

func showPermissionGuidance() {
    // Guide users to enable notifications in Settings
    let alert = UIAlertController(
        title: "Notifications Disabled",
        message: "Please enable notifications in Settings to receive messages",
        preferredStyle: .alert
    )
    
    alert.addAction(UIAlertAction(title: "Open Settings", style: .default) { _ in
        if let url = URL(string: UIApplication.openSettingsURLString) {
            UIApplication.shared.open(url)
        }
    })
    
    alert.addAction(UIAlertAction(title: "Cancel", style: .cancel))
    
    // Present the alert
    if let topVC = UIApplication.shared.topViewController() {
        topVC.present(alert, animated: true)
    }
}
```

### 2. Notification Delivery Testing
**File:** `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift`
```swift
func testNotificationDelivery() async -> NotificationTestResults {
    var results = NotificationTestResults()
    
    // Test basic notification
    results.basicNotification = await testBasicNotification()
    
    // Test message notification
    results.messageNotification = await testMessageNotification()
    
    // Test group chat notification
    results.groupNotification = await testGroupNotification()
    
    // Test with app in background
    results.backgroundNotification = await testBackgroundNotification()
    
    // Test with app terminated
    results.terminatedNotification = await testTerminatedNotification()
    
    return results
}

struct NotificationTestResults {
    var basicNotification: Bool = false
    var messageNotification: Bool = false
    var groupNotification: Bool = false
    var backgroundNotification: Bool = false
    var terminatedNotification: Bool = false
    var deliveryTime: TimeInterval = 0
    var reliability: Double = 0
}
```

### 3. User Interaction Testing
**File:** `iOS/SCMessenger/SCMessenger/AppDelegate.swift`
```swift
// Handle notification actions
extension AppDelegate: UNUserNotificationCenterDelegate {
    
    func userNotificationCenter(_ center: UNUserNotificationCenter,
                              didReceive response: UNNotificationResponse,
                              withCompletionHandler completionHandler: @escaping () -> Void) {
        
        let userInfo = response.notification.request.content.userInfo
        
        // Handle different action types
        switch response.actionIdentifier {
        case UNNotificationDefaultActionIdentifier:
            // User tapped the notification
            handleNotificationTap(userInfo)
            
        case "REPLY_ACTION":
            // User used reply action
            if let textResponse = response as? UNTextInputNotificationResponse {
                handleReplyAction(userInfo, text: textResponse.userText)
            }
            
        case "MARK_READ_ACTION":
            // User marked as read
            handleMarkReadAction(userInfo)
            
        default:
            break
        }
        
        completionHandler()
    }
    
    private func handleNotificationTap(_ userInfo: [AnyHashable: Any]) {
        // Extract conversation info and open appropriate screen
        if let conversationId = userInfo["conversationId"] as? String {
            openConversation(conversationId)
        } else if let requestId = userInfo["requestId"] as? String {
            openRequest(requestId)
        }
    }
}
```

### 4. Background Processing Verification
**File:** `iOS/SCMessenger/SCMessenger/Background/NotificationBackgroundProcessor.swift` (NEW)
```swift
class NotificationBackgroundProcessor {
    
    func verifyBackgroundProcessing() async -> BackgroundTestResults {
        var results = BackgroundTestResults()
        
        // Test background fetch
        results.backgroundFetch = await testBackgroundFetch()
        
        // Test silent notifications
        results.silentNotification = await testSilentNotifications()
        
        // Test background processing time
        results.processingTime = await measureProcessingTime()
        
        // Test reliability under constraints
        results.constraintHandling = await testConstraintHandling()
        
        return results
    }
    
    struct BackgroundTestResults {
        var backgroundFetch: Bool = false
        var silentNotification: Bool = false
        var processingTime: TimeInterval = 0
        var constraintHandling: Bool = false
    }
}
```

### 5. Comprehensive Test Suite
**File:** `iOS/SCMessenger/SCMessengerTests/NotificationVerificationTests.swift` (NEW)
```swift
import XCTest
@testable import SCMessenger

class NotificationVerificationTests: XCTestCase {
    
    var notificationManager: NotificationManager!
    
    override func setUp() {
        super.setUp()
        notificationManager = NotificationManager()
    }
    
    func testPermissionFlow() async {
        let granted = await notificationManager.verifyPermissionFlow()
        XCTAssertTrue(granted, "Notification permission should be granted")
    }
    
    func testNotificationDelivery() async {
        let results = await notificationManager.testNotificationDelivery()
        
        XCTAssertTrue(results.basicNotification, "Basic notifications should work")
        XCTAssertTrue(results.messageNotification, "Message notifications should work")
        XCTAssertTrue(results.backgroundNotification, "Background notifications should work")
        
        XCTAssertLessThan(results.deliveryTime, 2.0, "Notifications should deliver within 2 seconds")
        XCTAssertGreaterThan(results.reliability, 0.95, "Notification reliability should be >95%")
    }
    
    func testUserInteractions() {
        // Test notification tap handling
        let userInfo: [AnyHashable: Any] = ["conversationId": "test123"]
        
        // Simulate notification tap
        let expectation = self.expectation(description: "Notification tap handled")
        
        // Verify conversation is opened
        DispatchQueue.main.async {
            // Check if conversation view is presented
            expectation.fulfill()
        }
        
        waitForExpectations(timeout: 5.0)
    }
}
```

## Files to Modify/Create
1. `iOS/SCMessenger/SCMessenger/Utils/NotificationManager.swift` - Enhanced permission flows
2. `iOS/SCMessenger/SCMessenger/AppDelegate.swift` - Notification action handling
3. `iOS/SCMessenger/SCMessenger/Background/NotificationBackgroundProcessor.swift` (NEW) - Background processing
4. `iOS/SCMessenger/SCMessengerTests/NotificationVerificationTests.swift` (NEW) - Test suite
5. `iOS/SCMessenger/SCMessenger/Views/NotificationGuidanceView.swift` (NEW) - Permission guidance UI
6. `iOS/SCMessenger/SCMessenger/Utils/NotificationLogger.swift` (NEW) - Verification logging

## Test Plan
1. **Permission Flow**: Test grant/deny/default states on physical devices
2. **Delivery Testing**: Verify notifications deliver in all app states (foreground, background, terminated)
3. **Interaction Testing**: Test tap actions, reply actions, custom actions
4. **Background Processing**: Verify background fetch and silent notifications
5. **Performance Testing**: Measure delivery times and reliability
6. **Cross-Device Testing**: Test on different iOS devices and versions

## Success Criteria
-  Notifications work on physical iOS devices
-  Permission flow handles all user choices gracefully
-  Tap actions open correct conversations/requests
-  Background processing works reliably
-  Comprehensive verification report generated

## Priority: HIGH
iOS notification functionality is critical for user engagement and must work reliably on physical devices.

**Estimated LOC:** ~400-500 LOC across 6 files
**Time Estimate:** 4-5 hours implementation + 3 hours testing