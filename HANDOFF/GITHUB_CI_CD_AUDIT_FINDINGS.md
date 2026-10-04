# GitHub CI/CD Configuration Audit — Findings Only

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-07-13  
**Scope:** GitHub Enterprise trial integration, Actions/runners readiness, iOS CI/CD, workflow consolidation  
**Status:** Audit complete; findings listed below; no recommendations yet (handoff to orchestrator for strategy)

---

## SECTION 1: GitHub Account & Enterprise Trial Status

### 1.1 Current Billing Blocker (CORRECTED 2026-07-14: RESOLVED)

**CORRECTION (2026-07-14, verified via `git remote -v`):** the repo has ALREADY
been transferred to the `Sovereign-Communication` org
(`github.com/Sovereign-Communication/SCMessenger`). The personal-account
billing lock below is STALE - it no longer gates Actions. Runners execute on
the org; the remaining iOS blocker is purely the workflow defects in Section 3
(now fixed), NOT account access. Original stale finding preserved below.

**Finding (STALE):** Personal GitHub account (`Treystu`) is locked due to billing issue; blocks ALL Actions execution.
- **Evidence:** _QUEUE.md "Open decision points" item 3: "account is locked due to a billing issue (personal account governs the public repo; the Enterprise trial does not cover it)"
- **Impact:** Free macOS/Linux/Windows runners are available on public repo but cannot execute (account lock prevents it)
- **Status:** Two paths forward documented: (a) fix personal account billing, OR (b) transfer repo to GitHub Enterprise trial org
- **Gating:** iOS CI lane (WS-FARM-C) cannot proceed until resolved

### 1.2 GitHub Enterprise Trial Setup

**Finding:** Enterprise trial org exists but repository not transferred into it.
- **Evidence:** FARM plan `WS-FARM-C` references "transfer repo into the trial org" as alternative to billing fix
- **Current state:** Repo is PUBLIC on personal account
- **What trial grants:** Unlimited Actions minutes, macOS/Linux/Windows runners, branch protection, required reviewers (if org-level rules needed)
- **Unknown:** Trial term, org name, current status (active/expired/pending setup)

### 1.3 Runner Availability (if account unlocked)

**Finding:** All required runners are free for public repos; no paid-tier infrastructure assumption.
- **Runners in use:**
  - `ubuntu-latest`: 6 workflows
  - `macos-14` (Apple Silicon M1): 3 workflows
  - `macos-latest`: 3 workflows
  - `windows-latest`: 1 workflow
  - `${{ matrix.os }}` (matrix): 2 workflows
- **Status:** Ready to execute once account is unlocked
- **macOS caveat:** `macos-14` (M1/ARM) is free on public repos; no x86 macOS runner currently declared

---

## SECTION 2: Workflow Portfolio (14 files, 3,244 lines)

### 2.1 Workflow Inventory

| Workflow | Lines | Trigger | Runner | Status | Notes |
|---|---|---|---|---|---|
| `ci.yml` | 76 | push/PR on main | ubuntu, macos, windows | Active | Lint, test matrix, docs, FFI surface |
| `mobile.yml` | 69 | push/PR on main | ubuntu, macos | Active | Android APK + iOS build (no simulator test) |
| `ios-build-test.yml` | 217 | workflow_dispatch OR push/PR on iOS/** | macos-14 | **BROKEN** | 7 defects listed below; orchestrator callback ready |
| `cross-platform-test.yml` | 276 | workflow_dispatch | ubuntu, macos, windows | **Dispatchable** | Orchestrator integration; sprint_id tracking |
| `cross.yml` | 148 | push/PR on main | ubuntu | Active | Cross-compilation test matrix |
| `release.yml` | 295 | push on tags or manual | ubuntu, macos, windows | Active | Multi-platform CLI binary builds |
| `security.yml` | 191 | weekly schedule or manual | ubuntu | Active | cargo-audit weekly; GitHub issue creation on findings |
| `hygiene.yml` | 235 | push/PR on main | ubuntu | Active | Keystore/secret detection; artifact tracking |
| `docker-test-suite.yml` | 208 | push/PR on main | ubuntu | Active | Docker Compose multi-node test |
| `docker-publish.yml` | 52 | manual or tag push | ubuntu | Active | Docker image build/push to registry |
| `lint.yml` | 138 | push/PR on main | ubuntu | Active | Format + clippy + deny checks |
| `auto-label.yml` | 71 | Issues/PRs | ubuntu | Active | Issue labeling automation |
| `stale.yml` | 43 | scheduled | ubuntu | Active | Auto-close stale issues |
| `scm-test-diagnose-fix.lock.yml` | 1,225 | (lock file; dormant) | — | **Dormant** | Historical; not active; 1225 lines is a code smell |

### 2.2 Orchestrator Integration (Ready)

**Finding:** Cross-platform-test.yml and ios-build-test.yml already have orchestrator callback plumbing in place.
- **Implemented:** `ORCHESTRATOR_URL` and `ORCHESTRATOR_TOKEN` secrets used for result posting
- **Payload structure:** Sprint ID, workflow/job name, status (passed/failed), platform, branch, run ID, run URL, timestamp
- **Non-fatal failures:** Callback failures don't block job (curl with `-f` but error piped to echo with "WARNING")
- **Status:** Ready to wire; just needs secrets configured and account unlock

### 2.3 Secrets Configuration (Partial)

**Finding:** Secrets referenced but only some are configured/safe.

**Secrets in use (status unknown):**
- `ORCHESTRATOR_URL` — needed for callback
- `ORCHESTRATOR_TOKEN` — needed for callback
- `COPILOT_GITHUB_TOKEN` — Copilot integration (may not be in use)
- `GH_AW_GITHUB_TOKEN` — GitHub App token (unclear purpose)
- `GH_AW_GITHUB_MCP_SERVER_TOKEN` — MCP server token (unclear purpose)
- `DOCKERHUB_USERNAME` — Docker push credentials
- `DOCKERHUB_TOKEN` — Docker push credentials

**Secrets commented out (good practice):**
- `IOS_CERTIFICATE_BASE64`, `IOS_CERTIFICATE_PASSWORD`
- `KEYSTORE_*`, `PROVISIONING_PROFILE_BASE64`
- `ANDROID_KEYSTORE_BASE64`

**Status:** Audit cannot verify which secrets actually exist in repo settings without GitHub UI access.

---

## SECTION 3: iOS CI/CD Defects (ios-build-test.yml)

### 3.1 Seven Documented Defects (TASK_CI_IOS_MACOS_RUNNER_FIX.md)

All defects are in `ios-build-test.yml` and prevent trustworthy CI signal:

1. **Failure masking** (~lines 101, 112)
   - Xcodebuild piped to `xcpretty || true`
   - Job can NEVER fail
   - Fix: Remove `|| true`; add `set -o pipefail` before xcodebuild

2. **Lowercase paths** (~lines 84-94)
   - Uses `ios/` and `ios/**`
   - Rule: uppercase `iOS/` everywhere (case-insensitive APFS masks this locally)
   - Fix: Replace all `ios/` with `iOS/`

3. **xcodebuild missing -project flag** (~line 120)
   - Invokes xcodebuild with `-scheme` but no `-project`
   - Project lives at `iOS/SCMessenger/SCMessenger.xcodeproj`
   - Fix: Add `-project iOS/SCMessenger/SCMessenger.xcodeproj`

4. **No PR/push triggers** (line 8–end)
   - Currently workflow_dispatch-only
   - FFI drift not caught on code changes
   - Fix: Add `pull_request` and `push` triggers on paths: `iOS/**`, `core/src/api.udl`, `mobile/**`

5. **Missing bindings-drift gate**
   - Not regenerating Swift bindings in CI
   - Known drift exists: PQC-05 added `require_pq` to `core/src/api.udl:263` (2026-07-06) but Swift regen last ran 2026-07-02
   - Fix: Regenerate bindings (`cargo run --bin gen_swift --features gen-bindings`), diff against `iOS/.../Generated/api.swift`, fail on mismatch
   - **Caveat:** Do NOT commit regen until PQC-10 lands (it changes identity signatures; one cycle avoids double-regen)

6. **Emoji in verify script** (`scripts/verify_ios_bindings.sh`)
   - Lines 17, 24, 35, 42 contain emoji
   - Violates `.claude/rules/no-emojis.md` repo-wide rule
   - Fix: Either replace verify script with bindings-drift gate (item 5) or strip emoji and extend to full-surface diff

7. **No simulator XCTest runs**
   - Workflow builds for simulator but does not run tests
   - NotificationVerificationTests, BackupPassphraseValidatorTests, MeshBackgroundServiceTests exist but are not invoked
   - Limitation: Simulator cannot cover CoreBluetooth, Multipeer/AWDL, APNs, true background scheduling (hardware-waived cells)
   - Fix: Add simulator destination XCTest run; document hardware waiver cells in acceptance

### 3.2 Acceptance Criteria for iOS Fix

From task file:
- Push touching `iOS/` or `api.udl` triggers workflow on `macos-14`
- Deliberately broken Swift file makes job FAIL (prove failure works)
- Bindings drift (current state) makes job FAIL until regen lands
- No emoji, uppercase `iOS/` paths throughout

### 3.3 iOS Path Forward (v1.0.0 gating)

**Dependency:** GitHub billing unlock OR repo transfer to trial org (blocks runner access)  
**Task file:** TASK_CI_IOS_MACOS_RUNNER_FIX.md (currently in HANDOFF/done/ but header still says TODO — premature move)  
**Post-fix:** Single XCFramework + Swift bindings regen AFTER PQC-10 lands (per _QUEUE.md note)  
**Interop testing:** Joins PQC-13 matrix (desktop<->iOS, Android<->iOS, CLI<->iOS via relay)

---

## SECTION 4: Workflow Consistency & Configuration Issues

### 4.1 Path Filter Fragmentation

**Finding:** Each workflow declares its own path triggers; no shared convention.

**Patterns observed:**
- `ci.yml` and `mobile.yml`: trigger on push/PR to main (no path filter)
- `cross.yml`: path-filtered (`core/**`, `mobile/**`, `Cargo.*`, `rust-toolchain.toml`, etc.) but uses dorny/paths-filter action
- `ios-build-test.yml`: was workflow_dispatch only (broken)
- `release.yml`: triggers on tags or manual (no path filter)
- `security.yml`: weekly schedule + manual (no path filter)
- `hygiene.yml`, `docker-test-suite.yml`, `lint.yml`: all trigger on push/PR to main (no path filter)

**Consequence:** Broad coverage but potentially over-triggering (all workflows run on unrelated changes like docs).

### 4.2 Artifact Retention Policies

**Finding:** Artifact upload patterns are inconsistent.

**Observed:**
- `ios-build-test.yml`: `retention-days: 30` (explicit)
- `cross-platform-test.yml`: `retention-days: 7` (explicit)
- `mobile.yml`, `release.yml`, others: no retention policy (defaults to GitHub's account-level setting, typically 90 days)

**Storage implication:** If all artifacts default to 90 days, storage costs could accumulate (especially mobile APKs, XCFrameworks, release binaries).

### 4.3 Cache Strategy

**Status:** Two cache actions in use:
- `actions/cache@v3` (1 use) — slightly stale
- `actions/cache@v4` (multiple) — current
- `Swatinem/rust-cache@v2` (multiple) — Rust-specific, handles registry + git + target/ automatically

**Finding:** Cache keys vary across workflows.
- Some use `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- Others use `linux-cargo-`, `macos-14`, etc. (custom keys)
- No unified cache key strategy documented

**Consequence:** Cache hit rate likely suboptimal; potential for stale deps on force-refresh.

### 4.4 Dormant Workflows

**Finding:** `scm-test-diagnose-fix.lock.yml` is 1,225 lines but appears to be a lock/historical file, not active.
- **Status:** File exists, should be either deleted or moved to `HANDOFF/done/`
- **Naming:** `.lock.yml` extension is non-standard (Actions doesn't understand it; treated as active workflow with "lock" in name)

### 4.5 Auto-Label Configuration

**Finding:** `auto-label.yml` is active but implementation details unknown (no visibility into label rules without GitHub UI/API).

---

## SECTION 5: Branch Protection & Required Checks

### 5.1 Current Status (from CI_FAILURES.md)

**Finding:** Branch protection rules exist but are documented as incomplete/unchecked.
- Docs reference: "Edit rule for `main` branch" but don't list what's currently configured
- Required status checks might be stale ("Required status check 'ci / rust-core' is expected but not present" is listed as common error)
- Docs say "Update required status checks" but don't confirm they're set up for current workflows

### 5.2 What Should Be Checked

**Suggested required status checks for main branch:**
- `CI / lint` (from ci.yml)
- `CI / test` (from ci.yml)
- `Lint` (from lint.yml, if separate)
- `Repository Hygiene Checks` (from hygiene.yml)
- Possibly: `Security Scan` (weekly, but may not run on every PR)

**Status:** Requires GitHub UI verification; audit cannot confirm from code alone.

---

## SECTION 6: Execution Plan & Farm Alignment

### 6.1 Plan Constraints (from V1_0_0_EXECUTION_PLAN.md)

**Finding:** Execution plan explicitly **excludes CI from v1.0.0 scope.**
- Section 1.2: "**No CI.** All verification is local: this Windows machine + one physical Pixel 6a. AWS box explicitly out of scope for this plan."
- Section 2.4: "If GitHub Actions ever comes back (H1, [HUMAN]), it supplements this regime; nothing below depends on it."
- Section 3 (Phase 2): "CI workflow files written but dormant until H1" (WS-D3 KMP packaging)

**Consequence:** GitHub Actions is **not blocking v1.0.0 ship.** Current work assumes local validation only.

### 6.2 Farm Plan Alignment (FARM_FINAL_PLAN.md)

**Finding:** Farm plan upgrades iOS CI to **v1.0.0-blocking** because half+ farm users carry iPhones.
- **Decision:** iOS parity is IN scope (resolved 2026-07-13)
- **New gate:** GitHub billing unlock + TASK_CI_IOS_MACOS_RUNNER_FIX (WS-FARM-C)
- **Cascade:** iOS XCFramework regen after PQC-10, interop testing in PQC-13 matrix

**Consequence:** GitHub/CI is now gating farm rollout (not core v1.0.0, but farm seed operations).

---

## SECTION 7: What GitHub Enterprise Trial Enables (if obtained)

**Assumption:** Enterprise trial org is active and repo is transferred into it.

### 7.1 CI/CD Improvements

- Unlimited Actions minutes (no usage limits)
- All runners (ubuntu, macos, windows) included
- Organization-level branch protection (if needed for multiple repos later)
- Required reviewers from code owners (currently Free tier limitation)
- Repository secrets scoped to org (not just personal account)

### 7.2 Security & Compliance Features

- Dependabot on organization level (currently in repo)
- Secret scanning (built-in on private repos; public repos limited)
- Advanced security features (code scanning, secret alerts)

### 7.3 Non-blocking for v1.0.0, but needed for farm

- iOS CI (free macOS runners) [OK]
- Android CI (free Linux runners) [OK]
- Windows CLI CI (free Windows runners) [OK]
- Orchestrator callback integration [OK] (already plumbed)

---

## SECTION 8: Consolidation Checklist

### 8.1 Before Account Unlock/Transfer

- [ ] Verify Enterprise trial org exists, has capacity, billing is active
- [ ] Confirm transfer procedure (org admin steps)
- [ ] Document org name, trial end date, included features
- [ ] Check if any existing secrets (ORCHESTRATOR_TOKEN, Docker creds) need re-creation at org level

### 8.2 Immediately After Unlock

- [ ] Confirm `ci.yml` and `mobile.yml` trigger and pass
- [ ] Verify cache hits are working (check Actions run logs for cache statistics)
- [ ] Run `cross-platform-test.yml` manually to test orchestrator callback
- [ ] Spot-check that free runners are being allocated (ubuntu-latest, macos-14, windows-latest)

### 8.3 iOS Workflow Fixes (parallel, can start anytime)

- [ ] Fix 7 defects in `ios-build-test.yml` per TASK_CI_IOS_MACOS_RUNNER_FIX.md
- [ ] Add bindings-drift gate (defer commit until PQC-10)
- [ ] Strip emoji from `scripts/verify_ios_bindings.sh`
- [ ] Add simulator XCTest job
- [ ] Verify lowercase -> uppercase iOS/ path fixes

### 8.4 Workflow Cleanup

- [ ] Archive or delete `scm-test-diagnose-fix.lock.yml` (or rename to `.archived`)
- [ ] Document path filter strategy (centralize or standardize)
- [ ] Audit retention policies; set reasonable defaults (30 days for test artifacts, 90+ for release binaries)
- [ ] Consolidate cache keys across workflows (e.g., all Rust workflows use same key format)

### 8.5 Branch Protection Audit (requires GitHub UI)

- [ ] List current required status checks on main branch
- [ ] Remove stale checks (if "ci / rust-core" no longer exists)
- [ ] Add current workflow check names
- [ ] Verify dismiss-stale-reviews is enabled
- [ ] Confirm require-branches-up-to-date is enabled

### 8.6 Secrets Verification (requires GitHub UI)

- [ ] Verify ORCHESTRATOR_URL exists and is reachable (test callback on next cross-platform-test.yml run)
- [ ] Verify ORCHESTRATOR_TOKEN is valid
- [ ] Verify DOCKERHUB_USERNAME and DOCKERHUB_TOKEN (if docker-publish is used)
- [ ] Document which GitHub tokens are needed (COPILOT_GITHUB_TOKEN, GH_AW_* tokens may be unused)
- [ ] Remove unused secrets to reduce attack surface

### 8.7 Monitoring & Maintenance

- [ ] Set up Actions usage dashboard (Actions -> Usage)
- [ ] Create alert thresholds for minute usage
- [ ] Schedule monthly cache cleanup (Actions -> Caches)
- [ ] Document failure patterns in CI_FAILURES.md (update dated examples)
- [ ] Assign workflow maintenance owner (currently orphaned?)

---

## SECTION 9: Known Unknowns

### 9.1 Cannot Verify Without GitHub UI/API Access

- Actual secrets that exist in repo settings
- Current branch protection rules and required status checks
- Workflow enable/disable state (visible in Actions tab)
- Cache statistics (hit/miss rates)
- Actions usage (minutes consumed this billing period)
- Any runner group configurations
- Organization settings (if trial transfer has occurred)

### 9.2 Cannot Verify Without Execution

- Whether cross-platform-test.yml orchestrator callback actually works (needs ORCHESTRATOR_URL reachable)
- Whether cache keys are effective (need to see % hit rate over 10+ runs)
- Whether timeout-minutes settings are realistic for current hardware
- Whether workflow trigger filters work as intended (paths-filter action behavior)

### 9.3 Unclear from Docs

- Purpose of `GH_AW_*` tokens (AI Workbench integration?)
- Whether `docker-publish.yml` is actively used or abandoned
- Current Copilot integration status
- Why `scm-test-diagnose-fix.lock.yml` is in the repo (historical? in development?)

---

## SECTION 10: Enterprise Trial Impact on v1.0.0 vs. Farm

### 10.1 v1.0.0 Scope (Execution Plan: "No CI")

| Component | Depends on GitHub | Status |
|---|---|---|
| Phase 1 transport | Local Windows + Android emulator | [OK] Independent |
| Phase 2 PQC | Local compile + cargo test | [OK] Independent |
| KMP desktop (WS-D) | Local build verification only | [OK] Independent |
| Release build (F1) | Local cargo + scripts | [OK] Independent |

**Conclusion:** Core v1.0.0 ship does NOT depend on GitHub Actions being unlocked.

### 10.2 Farm Rollout (FARM_FINAL_PLAN.md: iOS is a gate)

| Component | Depends on GitHub | Status | Blocker |
|---|---|---|---|
| iOS CI (WS-FARM-C) | macos-14 runner + billing unlock | [OK] If account unlocked | **Yes** |
| iOS XCFramework regen | macos-14 runner | [OK] If account unlocked | **Yes** (post-PQC-10) |
| iOS interop drills | Physical iPhone + local farm rig | [OK] Independent | No |
| 12-node farm sim | AWS Docker rig (approved separately) | [OK] AWS ready | No (infra ready) |

**Conclusion:** GitHub account unlock IS required for iOS CI lane, which gates farm seed rollout (not core v1.0.0).

---

## SECTION 11: Summary for Orchestrator Decision

### Enterprise Trial: Help or Overhead?

**Does it help?**
- [OK] Eliminates account billing blocker
- [OK] Unlocks free macOS/Linux/Windows runners (already available, just need account unlock)
- [OK] Provides org-level structure if multi-repo later
- [FAIL] NOT needed for core v1.0.0 (all local validation)
- [OK] NEEDED for farm seed iOS CI lane (v1.0.0-blocking per latest decision)

**Can CI/CD be fixed?**
- [OK] Yes: Account unlock (simple one-time action) + iOS workflow fixes (listed, ready to implement)
- [WARNING] Timing: iOS fixes are ready now, but can't run until account is unlocked
- [WARNING] Coverage: Some workflows are plumbed but have never been executed (unknown if they actually work)

### Decision Point Summary

**For orchestrator:** Determine if v1.0.0 ships before or after iOS CI becomes operational.
- **Before (keep CI dormant):** GitHub account status stays as-is; skip iOS CI fixes; document as post-v1.0.0 work
- **After (enable GitHub):** Unlock account / transfer to trial org -> fix iOS workflow -> run PQC-13 interop matrix (which includes iOS)

Either path is viable per execution plan ("H1 CI restoration remains open [HUMAN] but non-blocking by construction").

