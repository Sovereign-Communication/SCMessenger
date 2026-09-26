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
# MODEL: glm-5.1:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_ANDROID_MESSAGE_SEARCH_UI_001

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 Android UX parity
**Source:** AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md

---

## Verified Gap

Android `ChatScreen.kt` and `ConversationsScreen.kt` have NO message search functionality. The CLI has `history --search` and `core` exposes `MeshRepository.searchMessages()` (verified in `MeshRepository.kt:4469`), but there is NO UI for it.

**Verified Code State:**
- `MeshRepository.kt:4469`  `searchMessages(query, limit)` exists and returns `List<MessageRecord>`
- `ChatScreen.kt:38`  No search bar or search UI component
- `ConversationsScreen.kt:35`  No search functionality
- Strings.xml  No search-related strings

## Scope

### Part A: Search Bar in ConversationsScreen

1. Add a search bar (TextField with leading search icon) to `ConversationsScreen.kt` top bar
2. Filter conversation list based on search query matching peer nickname or message content preview
3. Clear search button (X) when query is non-empty
4. Empty state: "No conversations matching '$query'"

### Part B: Search in ChatScreen

1. Add search icon to `ChatScreen.kt` top bar actions
2. When tapped, show a search bottom sheet or inline search field
3. Search within current conversation history (use `MeshRepository.searchMessages(query, peerFilter=currentPeerId)`)
4. Highlight matching messages in the lazy list (scroll to first match)
5. Show "X of Y matches" with up/down navigation arrows

### Part C: Strings

Add all new strings to `strings.xml` (search hint, empty state, match count, etc.)

## Constraints

- Use Material3 SearchBar or OutlinedTextField
- Follow existing architecture: ViewModel -> Compose UI
- All new strings in `strings.xml`
- Respect dark/light theme

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` [EDIT]
- `android/app/src/main/res/values/strings.xml` [EDIT]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. Search bar renders in ConversationsScreen preview
3. Search UI toggles in ChatScreen
4. No hardcoded strings

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
