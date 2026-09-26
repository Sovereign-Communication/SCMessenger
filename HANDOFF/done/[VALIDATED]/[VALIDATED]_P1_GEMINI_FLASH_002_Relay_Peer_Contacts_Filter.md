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
**Cross-ref:** `STATE/PLAN_VERIFICATION_2026-06-11.md` 1 (BLE/Wi-Fi Aware  gap includes contact list)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (UI/Compose filter  well-scoped, mechanical)
**Rationale:** D3 from the Android stability plan. Single flag addition to `Contact` data class + small Composable filter. The plumbing (`R-AND-RELAY-001`, `isBootstrapRelayPeer()`) is already exposed per the last commit. Gemini 3.5 Flash is a good fit: simple data class extension + UI predicate. ~40 LoC.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_002  Filter Relay Peers From Contacts List

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android stability (D3)
**Source:** `ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` (relay peers polluting contacts list)
**Depends on:** none
**Blocks:** none

---

## Verified Gap

`ContactsScreen.kt` shows every peer the device has ever seen, including bootstrap relay infrastructure peers. Users see 5-15 "ghost contacts" (relay nodes) mixed in with real human contacts. There's no UI distinction. The plumbing already exists: `R-AND-RELAY-001` added `isBootstrapRelayPeer()` in the last commit, but the contacts list filter is not using it.

## Scope (~40 LoC across 2 files)

### Part A: Add `infrastructure_peer: Boolean = false` to `Contact` data class (LOC: ~5)

In `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`:
- Extend `Contact` data class with `infrastructure_peer: Boolean` field (default false for backward compat)
- Populate from `R-AND-RELAY-001.isBootstrapRelayPeer(peerId)` when adding contacts
- No DB migration needed (sled store, new field is optional in serialized form)

### Part B: Filter at Composable (LOC: ~35)

In `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt`:
- In the `LazyColumn` items lambda: `items(contacts.filter { !it.infrastructure_peer })`
- Add a debug-only toggle in settings: "Show infrastructure peers" (off by default)
- Show count badge: "12 contacts (3 hidden infrastructure peers)" when filter active

## File Targets

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT  add field, ~5 LoC]
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactsScreen.kt` [EDIT  filter, badge, ~35 LoC]

## Build Verification

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:assembleDebug -x lint --quiet
# Unit test:
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.ui.contacts.ContactsFilterTest"
```

## Acceptance Gates

1. APK builds
2. Unit test `ContactsFilterTest` passes: given 12 contacts (3 marked `infrastructure_peer=true`), `filter` returns 9
3. Manual: open Contacts screen, count badges match expected
4. No regression: existing `contact list` test (`RoleNavigationPolicyTest`) still passes

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 2]
