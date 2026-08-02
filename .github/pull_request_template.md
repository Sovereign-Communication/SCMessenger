## Summary
Briefly describe the change.

## Why
What problem does this PR solve? Link the issue, or explain why no issue exists.

## Release Scope
- [ ] `v0.4.0` Android functionality/parity scope
- [ ] `v0.5.0` Android/iOS parity scope
- [ ] Post-0.5.0 follow-up (must not block the unified release)
- [ ] Repo-governance / documentation / tooling work

## Documentation Impact
- [ ] Canonical docs updated
- [ ] Supporting docs updated
- [ ] No docs update needed, because:

## Validation
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`
- [ ] `cargo build --workspace`
- [ ] `cargo test --workspace`
- [ ] GitHub Actions `CI` and `Cross` workflows are green
- [ ] GitHub Actions `Mobile` workflow is green (Android unit tests/APK and iOS build)
- [ ] GitHub Actions `iOS Build & Test` is green when transport/shared-core paths change
- [ ] `./scripts/docs_sync_check.sh` (when present)
- [ ] Targeted platform/manual validation:

## Risk / Security Notes
- [ ] No new security-sensitive behavior introduced
- [ ] Risk notes documented below

## Checklist
- [ ] Changes are focused and minimal
- [ ] Tests were added or updated when needed
- [ ] Existing behavior was revalidated for the changed area
- [ ] Docs/reporting surfaces stay aligned with the change
