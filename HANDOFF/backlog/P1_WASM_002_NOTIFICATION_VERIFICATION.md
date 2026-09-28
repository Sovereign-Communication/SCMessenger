# P1_WASM_002: WASM Notification Verification

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1 (High)
**Platform:** WASM/Web
**Status:** Open
**Routing Tags:** [REQUIRES: UI_VISION] [REQUIRES: FINALIZATION]

## Objective
Comprehensive verification of WASM browser notification functionality across different browsers and platforms. Notifications exist in code but need real-world testing to ensure they work as users expect.

## Background
From REMAINING_WORK_TRACKING.md:
- WASM notifications code exists and is documented as complete
- Browser API integration is implemented
- However, functionality needs real-world verification across browsers
- Marked as " Needs verification" in the tracking document

## Implementation Plan

### 1. Cross-Browser Testing Matrix
**File:** `wasm/test/notification_test.js` (NEW)
```javascript
// Test notification functionality across browsers
export async function testNotificationsCrossBrowser() {
    const browsers = ['chrome', 'firefox', 'safari', 'edge'];
    const results = {};
    
    for (const browser of browsers) {
        try {
            results[browser] = await testBrowserNotifications(browser);
        } catch (error) {
            results[browser] = { success: false, error: error.message };
        }
    }
    
    return results;
}

async function testBrowserNotifications(browser) {
    // Test basic notification permission
    const permission = await Notification.requestPermission();
    
    // Test notification display
    const notification = new Notification('SCMessenger Test', {
        body: 'Test notification from SCMessenger',
        icon: '/icon.png'
    });
    
    // Test click handling
    notification.onclick = () => {
        console.log('Notification clicked');
        window.focus();
    };
    
    return {
        success: true,
        permission,
        features: {
            basic: true,
            click: true,
            close: true
        }
    };
}
```

### 2. Notification Permission Flow
**File:** `wasm/src/notification_manager.js`
```javascript
export class NotificationManager {
    
    async requestPermission() {
        try {
            const permission = await Notification.requestPermission();
            
            if (permission === 'granted') {
                this.savePermissionState('granted');
                return true;
            } else if (permission === 'denied') {
                this.savePermissionState('denied');
                this.showPermissionHelp();
                return false;
            } else {
                // default - ask again later
                return false;
            }
        } catch (error) {
            console.error('Notification permission error:', error);
            return false;
        }
    }
    
    savePermissionState(state) {
        localStorage.setItem('notificationPermission', state);
    }
    
    showPermissionHelp() {
        // Show help for users who denied permissions
        if (this.isPermissionDenied()) {
            this.displayPermissionGuide();
        }
    }
}
```

### 3. Browser-Specific Workarounds
**File:** `wasm/src/browser_compatibility.js` (NEW)
```javascript
export function getBrowserSpecificNotificationOptions() {
    const userAgent = navigator.userAgent.toLowerCase();
    
    if (userAgent.includes('chrome')) {
        return {
            requireInteraction: false,
            badge: '/badge.png',
            vibrate: [200, 100, 200]
        };
    } else if (userAgent.includes('firefox')) {
        return {
            requireInteraction: true, // Firefox may dismiss too quickly
            tag: 'scmessenger' // Use tag to replace notifications
        };
    } else if (userAgent.includes('safari')) {
        return {
            // Safari has stricter requirements
            tag: 'scmessenger',
            renotify: true
        };
    } else {
        return {}; // Default options
    }
}
```

### 4. Service Worker Integration
**File:** `wasm/src/service_worker.js`
```javascript
// Service worker for background notifications
self.addEventListener('push', function(event) {
    const data = event.data.json();
    
    const options = {
        body: data.body,
        icon: data.icon || '/icon.png',
        badge: '/badge.png',
        tag: data.tag || 'scmessenger',
        data: data
    };
    
    event.waitUntil(
        self.registration.showNotification(data.title, options)
    );
});

self.addEventListener('notificationclick', function(event) {
    event.notification.close();
    
    // Focus the app or open specific conversation
    event.waitUntil(
        clients.matchAll({ type: 'window' }).then(function(clientList) {
            if (clientList.length > 0) {
                return clientList[0].focus();
            } else {
                return clients.openWindow('/');
            }
        })
    );
});
```

### 5. Verification Test Suite
**File:** `wasm/test/verification_test.js` (NEW)
```javascript
import { testNotificationsCrossBrowser } from './notification_test.js';

export async function runVerificationSuite() {
    const results = {
        browsers: await testNotificationsCrossBrowser(),
        features: await testNotificationFeatures(),
        performance: await testNotificationPerformance(),
        reliability: await testNotificationReliability()
    };
    
    // Generate verification report
    generateVerificationReport(results);
    
    return results;
}

async function testNotificationFeatures() {
    return {
        basic: testBasicNotifications(),
        click: testClickHandling(),
        close: testCloseHandling(),
        group: testNotificationGrouping(),
        sound: testNotificationSound()
    };
}
```

## Files to Modify/Create
1. `wasm/test/notification_test.js` (NEW) - Cross-browser testing
2. `wasm/src/notification_manager.js` - Permission flow improvements
3. `wasm/src/browser_compatibility.js` (NEW) - Browser-specific workarounds
4. `wasm/src/service_worker.js` - Service worker integration
5. `wasm/test/verification_test.js` (NEW) - Comprehensive test suite
6. `wasm/src/lib.rs` - Rust-side notification bridging

## Test Plan
1. **Cross-Browser Testing**: Chrome, Firefox, Safari, Edge
2. **Permission Flow**: Test grant/deny/default states
3. **Feature Testing**: Click handling, grouping, sounds
4. **Performance Testing**: Notification delivery timing
5. **Reliability Testing**: Network conditions, service worker

## Success Criteria
-  Notifications work across all major browsers
-  Permission flow handles all user choices gracefully
-  Click actions properly focus/open the app
-  Service worker handles background notifications
-  Comprehensive verification report generated

## Priority: HIGH
Notification functionality is critical for user engagement and must work reliably across all supported browsers.

**Estimated LOC:** ~300-400 LOC across 6 files
**Time Estimate:** 3-4 hours implementation + 2 hours testing