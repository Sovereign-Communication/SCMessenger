# P0_BUILD_004_REPO_BLOAT_AUDIT_OPTIMIZATION

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0
**Type:** BUILD
**Status:**  SAFE PHASE COMPLETE  risky items deferred to backlog

## Completed Actions (Safe)
1. **Removed `API_LIMIT_BACKUP_20260415_213830/`** from git tracking (~428 KB of backup snapshots)
2. **Removed `.legacy_ai_config/`** from git tracking (~4.5 MB of old AI tool configs)
3. **Removed root `SCMessengerCore.xcframework/`** from git tracking (duplicate of `iOS/SCMessengerCore.xcframework`)
4. **Removed unused `phase2_apis` feature flag** from `core/Cargo.toml` (zero source references)
5. **Added entries to `.gitignore`** for all removed items to prevent re-tracking

## Deferred to Backlog (Risky / Needs Testing)
- **Feature-gating `quinn` and `ureq`**  each used in only one file, but requires `#[cfg(feature)]` gates
- **Feature-gating `dspy/` module**  has no external callers but may be needed for future orchestration
- **11 `#[allow(dead_code)]` suppressions**  need per-case audit before removal
- **Binary size optimization**  `cargo bloat` analysis requires WASM target and release build

## Verification
- `cargo build -p scmessenger-core` passes
- `cargo test -p scmessenger-core --lib abuse` passes (23/23)
- `cargo test -p scmessenger-core --lib reputation` passes (19/19)

## Phase 1: Bloat Analysis (architect agent)

### Dependency Audit
1. **Rust (`core/`)**: Run `cargo +nightly udeps --all-targets` to find unused dependencies. Analyze `Cargo.toml` (workspace + each crate) for:
   - Dependencies pulled but never imported in `src/`
   - Feature flags enabling unused features (e.g., `default-features = true` pulling in extras)
   - Heavy dependencies that could be replaced with lighter alternatives
   - Duplicate functionality across dependencies (e.g., two HTTP clients, two serde formats)
2. **Android (`android/`)**: Audit `app/build.gradle` and `build.gradle`:
   - Unused Gradle dependencies (check imports vs declarations)
   - Transitive dependency bloat (run `./gradlew :app:dependencies` and flag unused paths)
   - Over-large SDK components (e.g., full Play Services when only one module is needed)
3. **WASM (`wasm/`)**: Audit `wasm/Cargo.toml`:
   - WASM-incompatible dependencies that bloat the binary
   - `wasm-opt` and `wasm-snip` opportunities
   - Feature flags pulling in std::fmt or std::io when `no_std` targets exist
4. **iOS (`iOS/`)**: Check Podfile or SPM dependencies for unused pods/packages

### Binary Size Audit
1. Run `cargo bloat --target wasm32-unknown-unknown --release` for WASM binary breakdown
2. Run `cargo bloat --crates` for Rust crate-level size contribution
3. Check Android APK size breakdown: `./gradlew :app:assembleRelease` then analyze with `bundletool`
4. Flag any binary size regressions from recent commits

### Dead Code Audit
1. Run `cargo +nightly udeps` for unused Rust code
2. Check for dead Kotlin code with Android Lint `UnusedResources`, `UnusedIds`
3. Search for `#[allow(dead_code)]` and `//noinspection` suppressions that mask bloat

### File Bloat Audit
1. Find the top 20 largest files in the repo (excluding `.git/`, `target/`, `build/`)
2. Identify generated files checked into source that should be `.gitignore`d
3. Check for duplicate files (same content, different paths)
4. Audit `docs/` for stale/obsolete documentation

## Phase 2: Execution Plan (architect delivers this)

The architect agent MUST produce a file `HANDOFF/review/P0_BUILD_004_BLOAT_OPTIMIZATION_PLAN.md` with:
- **For each finding:** exact file, exact dependency name, current size, estimated savings, risk level (safe/safe-with-test/risky)
- **Prioritized action list:** safe changes first, then safe-with-test, then risky
- **Rollback strategy** for each change (git command to revert)
- **Verification checklist** to confirm no regressions after each change
- **Explicit NO-GO items**  dependencies or code that looks unused but must be kept (e.g., platform-specific code, conditional compilation targets, security-critical deps)

## Phase 3: Execution (implementer agent)

1. Execute ONLY the "safe" items from the plan first
2. After each safe change: run `cargo check --workspace`, `cargo test --workspace`, and `./gradlew :app:compileDebugKotlin`
3. Then execute "safe-with-test" items, running full test suite after each
4. **DO NOT** execute "risky" items  move those to HANDOFF/backlog/ with detailed notes
5. After all safe changes: run `cargo bloat` again to measure actual savings
6. Write final results to `HANDOFF/done/P0_BUILD_004_REPO_BLOAT_AUDIT_OPTIMIZATION.md`

## Constraints
- **ZERO REGRESSIONS**: If any test fails after a change, revert immediately with `git restore` and log the failure
- **Full functionality**: Do not remove any dependency or code path that is reachable at runtime
- **Platform parity**: Changes must not break any platform (Rust core, Android, WASM, iOS, CLI)
- **Security**: Do not remove any dependency in `core/src/crypto/` or any `proptest`/`kani` verification dependency

[NATIVE_SUB_AGENT: RESEARCH]  Use native sub-agents to scan dependency trees and find unused imports before writing any code
[NATIVE_SUB_AGENT: LINT_FORMAT]  Use native sub-agents to format Cargo.toml and build.gradle after edits