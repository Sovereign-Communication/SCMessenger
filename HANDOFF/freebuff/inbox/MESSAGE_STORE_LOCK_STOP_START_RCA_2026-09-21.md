# RCA: "Message Store Unavailable" on stop -> Start (MESSAGE-STORE-LOCK-001)

Status: RCA complete, fix implemented on this branch
Date: 2026-09-21 (RCA), 2026-09-22 (fix landed here)
Reported by: operator (recurring regression: stop+start on the mesh shows
"Message Store Unavailable")
Fix branch: `freebuff/android-stop-teardown-timeout` (this branch; PR #351)

## Symptom

Stop the mesh, start it again: the app reports "Message Store Unavailable"
(degraded-storage banner). Recurring; every start after the first failure
keeps failing until the app process is killed.

## Root cause (traced end to end, five links)

1. The Rust `IronCore` owns sled's file lock on the message store. The lock
   is released only when the **last** reference to the core drops.
2. `MeshService::stop` drops its own `Arc<IronCore>` -- correct but
   insufficient: on Android the `uniffi.api.IronCore` wrapper is released by
   the GC/cleaner, not by `stop()`.
3. `MeshRepository.startMeshService()` injects the core into the
   process-wide Timber forest (`FileLoggingTree.setIronCore`,
   MeshRepository.kt ~1717). The tree held it in a **strong** field and
   nothing ever cleared it -- so the tree was the one reference a stop
   never released, and the core (with the sled lock) stayed alive for the
   life of the process.
4. The next Start opens the store while the previous instance's lock is
   still held. The old retry budget (10 x 50 ms ~= 0.5 s) is shorter than
   the real release window (the 5 s swarm-shutdown bound plus GC timing),
   so the open fails -> `DegradedStorage` -> `MeshService::start` fails
   loud -> the banner.
5. Because the tree never releases, every subsequent start fails the same
   way until the process dies.

## Fix (both halves, this branch)

- **Kotlin, structural:** `FileLoggingTree` now holds the core via
  `WeakHold` (weak reference) -- logging can never extend the core's life,
  by construction. New `CoreReferenceHolder` interface +
  `releaseCoreReferencesFromLoggingTrees()`, called from both
  `stopMeshService()` and the reset path, is the deterministic half.
- **Rust, budget:** shared `open_with_lock_retry` in `core/src/store/backend.rs`
  (50 x 100 ms = 5 s, matching the teardown tail), now used by the message
  store (`SledStorage`), contacts (`ContactManager::new`), and history
  (`HistoryManager::new`) -- every sled open on the stop -> Start path
  tolerates the same window. Only lock contention is retried; corruption,
  permission, and disk-full still fail immediately, now with the cause
  logged instead of swallowed.
- `core/src/contacts_bridge.rs` and `core/src/mobile_bridge.rs` previously
  had private, differently-sized retry loops; both migrate to the shared
  budget (which also removes a private copy in mobile_bridge that predated
  this RCA).

## Tests

- `FileLoggingTreeCoreRetentionTest.kt` (new, pure JVM): pins both halves --
  stop clears every core reference from the forest; a tree cannot keep a
  core alive.
- `core/src/store/backend.rs` (extended): retry control flow
  (9 retries then success; exhausted budget fails; clean open does not
  retry; non-lock errors are not retried), and
  `production_budget_covers_the_teardown_tail` -- a shrink of the budget
  below the teardown tail fails this assertion, so the regression cannot
  return silently.

## Verification status

- Written to be verified by CI (this branch's push triggers it); no local
  build was run (CI-primary doctrine; local disk TIGHT at the time).
- Field confirmation: install a CI-built APK containing this fix, then
  stop -> Start the mesh on the Pixel; the banner must not appear.

## Rule-8 note

The Rust files touched are `core/src/store/` and bridge files, not the
gated `core/src/{crypto,transport,routing,privacy}` set; still, reviewer
discretion applies since `backend.rs` sits under storage access used by
gated paths.
