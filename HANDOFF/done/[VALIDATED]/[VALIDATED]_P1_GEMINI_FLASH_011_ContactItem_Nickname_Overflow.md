<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `IN_PROGRESS_claude_slot2_status.md` 2026-06-08 03:30 PT (UI A/B shipped but D2 still pending)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (Compose UI annotation, mechanical)
**Rationale:** Per slot2 log, UI A (`weight(1f)` on ContactItem left column) and UI B (LazyColumn contentPadding bottom 88dp) shipped. There's a related UI bug: long nicknames overflow the right column. Add a `TextOverflow.Ellipsis` and maxLines=2 cap. ~15 LoC of Compose annotation. Trivial for Flash.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 5000

# P1_GEMINI_FLASH_011  ContactItem Long-Nickname Overflow Fix

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android UI polish
**Source:** `HANDOFF/IN_PROGRESS/IN_PROGRESS_claude_slot2_status.md` 2026-06-08 03:30 PT (UI A/B completed but not this)
**Depends on:** none

---

## Verified Gap

`ContactsScreen.kt` `ContactItem` Composable: nickname `Text` on the right column has no overflow handling. Long nicknames (>30 chars) push the timestamp/role badge off-screen. Pixel 6a screenshots show this on contacts with public key-derived default nicknames.

## Scope (~15 LoC, 1 file)

In `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt`:

Find the nickname `Text(...)` in `ContactItem`. Add:
- `maxLines = 2`
- `overflow = TextOverflow.Ellipsis`
- `modifier = Modifier.weight(1f, fill = false).padding(end = 8.dp)` to the surrounding Row

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt` [EDIT  1 Text() call, ~15 LoC]

## Build Verification

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:assembleDebug -x lint --quiet
# Run UI test if available:
./gradlew :app:connectedDebugAndroidTest --tests "com.scmessenger.android.ui.contacts.ContactItemOverflowTest" || echo "Test not yet authored  that's P1_GEMINI_FLASH_006 scope"
```

## Acceptance Gates

1. APK builds
2. Manual: open Contacts with a contact that has a 50-char nickname  ellipsis appears, timestamp still visible
3. Manual: 2-line nickname  wraps to 2 lines, doesn't push out timestamp
4. `RoleNavigationPolicyTest` still passes (no regression)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: COMPOSE] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 11]
