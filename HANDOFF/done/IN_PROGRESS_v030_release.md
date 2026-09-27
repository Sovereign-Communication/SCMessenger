# IN_PROGRESS  v0.3.0 Release Push

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Task:** Merge `fix/p1-cli-025-identify-dedup`  `integration/v0.2.2-pre-android-push-2026-06-05` and bump to v0.3.0
**Worker:** qwen3-coder-next:cloud (subagent, leaf worker)
**Started:** 2026-06-08 ~08:55 PT
**Completed:** 2026-06-08 08:58 PT
**Wall time:** ~3 min

## Result

- [OK] **merge commit SHA:** `0538224ae01c231e0248116eff578d25bb9f48e4`
  - `--no-ff` merge of `fix/p1-cli-025-identify-dedup` (3 unique commits) into `integration/v0.2.2-pre-android-push-2026-06-05`
  - 0 conflicts. `ort` strategy clean merge.
- [OK] **release commit SHA:** `665a5199d9daf514978bf854d4b5a7af7737e25b`
- [OK] **tag v0.3.0 created:** yes, pointing at merge commit `0538224a`
  - Tag annotation: `v0.3.0: P0/P1 Android bundle + identity race + mDNS peer-loss + UI A/B (Agy overnight ship 411940d1)`
- [OK] **final HEAD SHA:** `665a5199d9daf514978bf854d4b5a7af7737e25b`
- [OK] **working tree clean** at end
- [OK] **no push** performed (Lucas gate)

## Version bump files changed (5 files, 10 lines)

| File | Change |
|---|---|
| `Cargo.toml` | `[workspace.package].version` `0.2.1`  `0.3.0` (propagates to core/cli/mobile/desktop_bridge/wasm  all 5 inherit) |
| `android/build.gradle` | `versionCode` `7`  `8`; `versionName` `'0.2.1'`  `'0.3.0'` |
| `android/shared/build.gradle.kts` | `packageVersion` `"0.2.1"`  `"0.3.0"` |
| `iOS/SCMessenger/SCMessenger/Info.plist` | `CFBundleShortVersionString` `0.2.1`  `0.3.0`; `CFBundleVersion` `5`  `6` |
| `docs/CURRENT_STATE.md` | "Version Details" block (lines 609-613): Current Version, GitHub Release URL, Android & iOS version stamps updated |

## Conflicts encountered

**None.** The integration branch had no commits ahead of `main` that the fix branch didn't already know about (the 5 SHAs in the prompt brief  6e1c8494, 070f048a, etc.  were either on the fix branch tip or were intermediate integration commits, but they are all reachable from the fix branch tip; `git log fix..integration` returned empty). The `ort` merge was clean and 3-way automatic.

## Out-of-scope items NOT changed (intentionally)

- `REMAINING_WORK_TRACKING.md`  has no top-level "Current Version" line; the many `v0.2.1` mentions are historical record ("v0.2.1 Critical Bug Fixes (2026-03-12)", "Regressed from v0.2.1 to v0.1.9", etc.) and must not be falsified.
- `docs/historical/*`  per hard rules, untouched.
- `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`  historical doc, untouched.
- `docs/V0.2.1_*.md` (4 plan files)  named after the v0.2.1 version, treated as historical. Untouched.
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_*`  active handoff state, untouched (this file is the only new IN_PROGRESS).
- `core/tests/integration_contact_block.rs`, `docs/historical/iOS/iOS_BUILD_DEPLOY_GUIDE.md`, `docs/historical/MASTER_BUG_TRACKER.md`, `iOS/SCMessenger/SCMessenger/Info.plist` source-build configs, `scripts/burndown_issues.sh`  all contain `0.2.1` references but are documentation/build context, not canonical version sources. Untouched.
- `Cargo.lock`  not present (no Rust lockfile committed in this repo; workspace lock is local).

## What was NOT run (per scope)

- `cargo check --workspace`  version bump only touched workspace.package.version + native packaging files; no Rust code changed. 411940d1 + 5a4132f0 already verified the Rust tree on the fix branch.
- `gradle` / `gradlew`  slot 3 is doing the APK verify.
- No test fixes attempted (24 pre-existing failures  separate task).

## Notes for Lucas

- Tag is at the MERGE commit (`0538224a`), not at the final release commit (`665a5199`). This is intentional: the merge commit is the meaningful "v0.3.0 point in history" with the P0/P1 bundle; the release commit is a paperwork version-bump commit on top. If you want the tag moved to `665a5199`, run `git tag -d v0.3.0 && git tag v0.3.9 -m "..." 665a5199` after review.
- `android/` uses Groovy `.gradle` (not `.kts` as the orchestrator brief mentioned). Root `android/build.gradle` is the canonical version source; `app/build.gradle` inherits via `rootProject.ext`.
- iOS version was bumped in lockstep (56) to keep the Android/iOS versionCode parity, since CURRENT_STATE.md documents them together.
