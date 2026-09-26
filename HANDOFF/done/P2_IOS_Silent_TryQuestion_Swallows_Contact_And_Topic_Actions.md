# P2_IOS_Silent_TryQuestion_Swallows_Contact_And_Topic_Actions

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2
**Platform:** iOS
**Status:** TODO
**Source:** native sweep 2026-07-04 (independent re-verification pass; distinct
from T14/T15 in `docs/release-readiness-2026-07-02.md`, which cover
`IdentityBackupSheets.swift` and `VerifySafetyNumberSheet.swift` specifically 
both confirmed already fixed in this sweep. This task covers two *different*
files with the same bug class that T15's fix did not touch.)

## Problem

Two user-facing action handlers use Swift's `try?` to call a throwing repository/
manager method and then proceed as if the action succeeded, with no error
surfaced to the user and no distinguishing of failure from success:

### 1. `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift:132-134`

```swift
private func leaveTopic(_ topic: String) {
    try? topicManager?.unsubscribe(from: topic)
}
```

This is directly beneath `joinMesh()` (lines 122-130) in the same file, which
DOES handle errors correctly:

```swift
private func joinMesh() {
    do {
        try topicManager?.subscribe(to: topicName)
        error = nil
        topicName = ""
    } catch {
        self.error = error.localizedDescription
    }
}
```

`leaveTopic` has no equivalent `do`/`catch`  if `unsubscribe(from:)` throws
(e.g. storage error, topic not found), the UI gives no indication and the topic
likely still shows as "joined" to the user (or worse, disappears from a list
that assumed success), with no way to retry or understand why leaving failed.

### 2. `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift:323-329`

```swift
.swipeActions(edge: .trailing, allowsFullSwipe: true) {
    Button("Accept") {
        try? repository.acceptMessageRequest(peerId: request.peerId)
        loadRequests()
    }
    .tint(.green)
}
```

`loadRequests()` is called unconditionally right after the `try?`, regardless
of whether `acceptMessageRequest` succeeded or failed. If it fails, the swipe
action visually completes (the row likely disappears/refreshes as if accepted,
since `loadRequests()` re-fetches the list), giving false positive feedback:
the user believes they accepted a contact request, but the sender's message
request may still be pending/unaccepted with no working key wired up (this is
the same underlying `accept_message_request` codepath flagged as historically
buggy in T2 of `docs/release-readiness-2026-07-02.md`  the CLI-side fix for
that is tracked separately; this task is specifically about the iOS UI not
noticing or reporting when the call fails, regardless of the underlying cause).

## Why this matters

Both are the same anti-pattern already fixed for T15
(`VerifySafetyNumberSheet.swift`) elsewhere in the app, showing the project has
an established convention (catch and surface `error.localizedDescription`) that
just wasn't applied consistently to these two call sites.

## Fix Plan

### JoinMeshView.swift

Mirror the `joinMesh()` pattern:

```swift
private func leaveTopic(_ topic: String) {
    do {
        try topicManager?.unsubscribe(from: topic)
        error = nil
    } catch {
        self.error = error.localizedDescription
    }
}
```

(Confirm `error` is displayed somewhere in this view's body already, since
`joinMesh()` already relies on it  if so this is a drop-in fix with no new UI
needed.)

### MainTabView.swift

Only call `loadRequests()` on success, and surface an error otherwise. Exact UI
treatment (toast, inline banner, alert) should match whatever pattern
`MainTabView`/its parent already uses for other action failures  check
sibling views first. Minimal viable fix:

```swift
Button("Accept") {
    do {
        try repository.acceptMessageRequest(peerId: request.peerId)
        loadRequests()
    } catch {
        // surface `error.localizedDescription` via whichever error-display
        // mechanism this view already has (check for an `@State var error`
        // or similar in the enclosing view before adding a new one)
    }
}
```

If this view currently has no error-state property, add a minimal one
consistent with `JoinMeshView`'s `error: String?` pattern rather than
inventing a new mechanism.

## Files to Touch

- `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift` [EDIT]  lines 132-134
- `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` [EDIT]  lines 323-329
  (and possibly the enclosing view's state declarations, to add an error property
  if none exists)

## Verification

No Xcode available in this sweep. A build-capable session should:
1. Build the iOS target.
2. Manually verify (or add a unit/UI test if the project has a harness for it)
   that a failing `unsubscribe`/`acceptMessageRequest` call surfaces a visible
   error and does NOT silently refresh the list as if it succeeded.

## Acceptance Criteria

- `leaveTopic` in JoinMeshView.swift surfaces failures the same way `joinMesh`
  does (consistent error-handling convention within the same file).
- The message-request "Accept" swipe action in MainTabView.swift does not
  call `loadRequests()` (or otherwise imply success) when
  `acceptMessageRequest` throws; the failure is visible to the user.
