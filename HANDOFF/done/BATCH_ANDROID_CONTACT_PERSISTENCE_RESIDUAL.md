# BATCH_ANDROID_CONTACT_PERSISTENCE_RESIDUAL

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** VERIFIED REMAINING WORK  Partially fixed, one gap remains
**Agent:** Android/Kotlin implementer
**Budget:** 1800s (MIXED tier)
**Source:** docs/CURRENT_STATE.md "Contact Persistence & Data Integrity Issues" (2026-03-14 audit)

---

## Verified Current State

The 2026-03-14 audit identified 4 contact persistence bugs. **3 of 4 are already fixed** in current code:

| Bug | Status | Evidence |
|-----|--------|----------|
| Permission request loop on startup | **FIXED** | `MainActivity.kt:168-172` has `permissionRequestInProgress.compareAndSet(false, true)` atomic guard |
| Relay peers in user contact list | **FIXED** | `MeshRepository.kt:1308-1309` has `if (!isRelay)` guard before `upsertFederatedContact` |
| Contact auto-creation duplication | **PARTIALLY FIXED** | `upsertFederatedContact` uses `contactUpsertMutex` (line 6884), but dedup relies on list scan  not a unique constraint |
| Discovered peers persist after discovery stops | **LIKELY FIXED** | `nearbyDisconnectGraceMs` reduced to 5s (CURRENT_STATE.md 2026-03-14) + disconnect callbacks prune aliases |

**One confirmed remaining gap:**
- `upsertFederatedContact` does NOT enforce uniqueness at the database level. The 2026-03-14 audit captured a duplicate contact created 4 seconds apart for relay peer `93a35a87...`. The mutex prevents concurrent duplicates but not rapid sequential duplicates from separate `onPeerIdentified` callbacks.

## Scope

### Part A: Database-Level Deduplication

1. In `core/src/store/contacts.rs`:
   - Add unique constraint or upsert logic to prevent duplicate `public_key_hex` entries
   - `ContactManager::add()` should check for existing contact with same `public_key_hex` before insert
   - If contact exists, merge/update instead of creating duplicate
2. Add unit test: `test_add_contact_idempotent`  calling `add()` twice with same public key must result in exactly one contact

### Part B: Android Contact Upsert Hardening

1. In `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`:
   - In `upsertFederatedContact` (line 6873), add explicit duplicate check BEFORE acquiring mutex:
     - Query contact manager by `canonicalPeerId` or `publicKey`
     - If exists, call `updateLastSeen` / `setNickname` instead of creating new contact
   - Only create new contact if no existing match found
2. Verify the existing `contactUpsertMutex` remains in place for thread safety

### Part C: Verification

1. `./gradlew :app:compileDebugKotlin`  pass
2. `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"`  pass (or note if test env unavailable)
3. `cargo test -p scmessenger-core -- contacts::tests::test_add_contact_idempotent`  pass (new test)
4. `cargo test --workspace`  pass

## File Targets

- `core/src/store/contacts.rs` [EDIT  add idempotent add/upsert logic + test]
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT  lines 6873-6930, add pre-mutex dedup check]
- `core/src/store/contacts.rs` [ADD test]

## Constraints

- Do NOT change `Contact` struct schema (no serde break)
- Do NOT remove `contactUpsertMutex`  keep it for thread safety
- The dedup check must be case-insensitive on public key (hex keys may vary in case)
- If core-level dedup is sufficient, Android-level change may be minimal or unnecessary  verify after core change

## Acceptance Gates

1. `cargo build --workspace` passes
2. `cargo test --workspace` passes
3. New core unit test `test_add_contact_idempotent` passes
4. Android `upsertFederatedContact` has explicit duplicate check or delegates to core-level dedup
5. `./gradlew :app:compileDebugKotlin` passes
6. `REMAINING_WORK_TRACKING.md` updated to mark contact persistence audit complete

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
