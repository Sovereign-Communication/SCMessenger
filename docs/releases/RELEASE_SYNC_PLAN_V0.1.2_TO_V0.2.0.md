# SCMessenger Release Synchronization Plan (v0.1.2 -> v0.2.0)

Status: Active (workspace/version metadata aligned to v0.2.0; GitHub release/tag cleanup still pending maintainer execution)
Last updated: 2026-03-07

This plan outlines the steps to synchronize the SCMessenger versions between the codebase and GitHub releases.

Current repository reality:

1. `v0.2.0` is the active alpha baseline in the codebase and contributor-facing docs.
2. Public GitHub releases/tags still only expose `v0.1.0` and `v0.1.1`.
3. Planned follow-on workstreams `WS13` and `WS14` are currently tracked as `v0.2.1`, not as part of the active `v0.2.0` alpha closeout.

## Execution Snapshot (2026-03-03)

Completed in repository:

1. Workspace/package version metadata bumped to `0.2.0`.
2. Android/iOS displayed app version metadata aligned to `0.2.0`.
3. Repo-local release notes artifacts created under `docs/releases/`.

Pending maintainer-run release ops:

1. Create/push git tags (`v0.1.2`, `v0.2.0`) per release policy.
2. Draft/publish GitHub releases using `docs/releases/RELEASE_NOTES_V0.1.2_GH.md` and `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`.

## Current GitHub-facing alignment tasks

1. Keep repository/docs/support/issue surfaces explicitly aligned to `v0.2.0` as the active alpha line.
2. Avoid presenting `WS13` / `WS14` as unfinished `v0.2.0` work; they are currently `v0.2.1` planning scope.
3. When maintainers cut the GitHub release, ensure the tag/release naming matches the already-shipped `0.2.0` workspace metadata.

---

### GitHub Release Workflow

#### v0.1.2 (Immediate)
1. **Tag the Repository**: Create a git tag `v0.1.2` at the current HEAD.
2. **Draft Release**: Use `docs/releases/RELEASE_NOTES_V0.1.2_GH.md` to create the release on GitHub.
3. **Publish**: Finalize as a "Prerelease" or "Release" based on stability confidence.

#### v0.2.0 (Active alpha line; GitHub release still pending)
1. **Complete Release Sweep**: Execute the Post-WS12 residual-risk and repo-alignment closeout work.
2. **Confirm GitHub surfaces**: Ensure README, support routing, issue intake, and release notes all treat `v0.2.0` as the active alpha line.
3. **Commit and Tag**: Tag the already-versioned codebase as `v0.2.0`.
4. **Draft Release**: Use `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`.
5. **CI Assets**: Ensure GitHub Actions successfully builds and attaches binaries/artifacts for all platforms.

---

### Documentation and Release Notes

#### [NEW] `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`
- Consolidated release notes for v0.1.2 based on `docs/RELEASE_NOTES_V0.1.2_ALPHA.md`, ready for GitHub pasting.

#### [NEW] `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`
- Draft release notes for v0.2.0 based on the major WS0-WS12 implementation work.

---

### Verification and Gates

#### [MODIFY] `REMAINING_WORK_TRACKING.md`
- Update status to reflect that v0.2.0 work is transitioning from execution closure to release verification.

## Verification Plan

### Automated Tests
- Run full workspace tests:
  ```bash
  cargo test --workspace
  ```
- Verify platform builds:
  - Android: `cd android && ./gradlew assembleDebug`
  - iOS: `bash ./iOS/verify-test.sh`
  - WASM: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`

### Manual Verification
- Review the drafted release notes for accuracy against `REMAINING_WORK_TRACKING.md` outcomes.
- Execute the **Post-WS12 Residual Risk Closure Sweep** (item 14 in `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`) to ensure no open critical risks remain.
