# P1_ANDROID_UI_001: Chat Empty State and Loading Indicator

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** TODO
**Priority:** P1  User experience
**Estimated LoC Impact:** ~80

## Problem
The Chat screen lacks:
1. Empty state when no messages exist (shows blank white space)
2. Loading indicator while message history loads

## Exact Changes Required
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`
- Add a `when` block around the message list:
  - `messages.isEmpty() && isLoading`  show `CircularProgressIndicator` centered
  - `messages.isEmpty() && !isLoading`  show empty state with icon + text ("No messages yet. Send a message to start the conversation.")
  - `messages.isNotEmpty()`  show current message list
- Add `isLoading` state to `ChatViewModel` that tracks `getConversation()` loading

## Verification
- [ ] Empty state visible when opening chat with new contact
- [ ] Loading spinner visible briefly before messages load
