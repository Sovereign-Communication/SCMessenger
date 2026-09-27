# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1200
# token_budget: 12000

# P0_ANDROID_022_Relay_Peer_Contacts_Filter

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P1 Android partial (relay peers appearing as contacts) + planfromclaudeforhermes 2 Phase D.3
**Depends on:** P0_BUILD_001
**Note:** Last commit (704338c0) added `isBootstrapRelayPeer()` dynamic detection. This task wires the UI filter that uses it.

---

## Verified Gap

Bootstrap relay peers (used for NAT traversal) appear in the user's contacts list. They're infrastructure, not user contacts. Per `docs/historical/plans/PRODUCTION_ROADMAP.md` 1.2: "Fix relay peers appearing as user contacts (add infrastructure flag/filter)".

HEAD `704338c0` added `isBootstrapRelayPeer()` that dynamically builds the known relay set from circuit breaker addresses + discovered relay peers. This task wires the UI filter.

## Scope (~80 LoC across 2 files)

### Part A: Add infrastructure flag to Contact (LOC: ~30)

In `android/app/src/main/java/com/scmessenger/android/data/Contact.kt` (or wherever the data class lives):

```kotlin
data class Contact(
    // ... existing fields ...
    val isInfrastructurePeer: Boolean = false,  // true for relay/relay-adjacent
)
```

In `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`:
- When creating a Contact from a discovered peer, call `if (peerManager.isBootstrapRelayPeer(peerId)) contact.copy(isInfrastructurePeer = true)`
- When receiving a `PeerDiscovered` event, set the flag based on the same check

### Part B: Filter contacts list (LOC: ~50)

In `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt` (or `ContactsViewModel.kt`):

```kotlin
val visibleContacts = contacts.filter { !it.isInfrastructurePeer }
    .sortedBy { it.nickname }
```

Apply in:
- `ContactsScreen` LazyColumn items
- `ContactsViewModel.contacts` StateFlow (filter at source, not at render)
- Search/filter queries
- Contact count badges in Dashboard

Add a separate "Mesh Relays" section/tab if user wants visibility (optional; ~30 LoC additional).

## File Targets

- `android/app/src/main/java/com/scmessenger/android/data/Contact.kt` [EDIT  add isInfrastructurePeer]
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT  set flag on discovery]
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsViewModel.kt` [EDIT  filter at source]
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt` [EDIT  use filtered list]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
./gradlew :app:assembleDebug -x lint --quiet
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` succeeds
2. New unit test: `ContactsViewModelTest` covers filter excludes infrastructure peers, search query respects filter
3. Manual: with 2 user contacts and 3 known relay peers, contacts list shows only 2 user contacts
4. Manual: relay peers still appear in mesh/peer network screen (don't hide from diagnostic view)
5. Commit: `android: v0.2.1 filter infrastructure peers from contacts list`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001] [BUILDS_ON: 704338c0]
