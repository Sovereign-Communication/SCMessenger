# Canonical Outlier Audit -- iteration 0 (baseline skeleton)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210
Model: Buffy (Freebuff) -- task header names "GLM 5.3 Flash"; agent identity discrepancy recorded, rules followed as written.
Mode: REPORT-ONLY

## Snapshot commands

Command: `git rev-parse HEAD`
```
1acb63531aaa1e7c22071bb7430c72b5a9753210
```

Command: `git branch --show-current` (at M0, before branch creation)
```
feat/v040-multi-transport-store-forward
```

Command: `git status --short` (at M0, before branch creation)
```
 M HANDOFF/CTO_STATE.md
 M HANDOFF/freebuff/README.md
 M android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
 M android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt
 M android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt
 M android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt
 M android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt
 M android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt
 M android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt
 M android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt
 M android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt
 M android/app/src/test/java/com/scmessenger/android/test/SettingsViewModelTest.kt
 M core/src/iron_core.rs
?? HANDOFF/freebuff/inbox/V040_3NODE_BASELINE_AND_UPDATE_PLAN_2026-09-19.md
?? HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
?? HANDOFF/review/RULE8_PR289_VERDICT_2026-09-17.md
?? android/app/src/test/java/com/scmessenger/android/test/OrderingAndNotificationRoutingTest.kt
```
All modified/untracked files above belong to other sessions' in-progress work and are
left untouched per task rule 1 and AGENTS.md rule 11.

Command: `git tag --list` (ALL tags, no truncation)
```
freebuff-snapshot/245d21ce-450f-4f3b-90d5-eb6b7c119d89
freebuff-snapshot/d4ad9f34-e5c0-4bdd-8a18-1911adefd62a
v0.1.0
v0.1.1
v0.1.9
v0.2.1
v0.3.5
v0.4.0-rc.1
```
Newest version tag: `v0.4.0-rc.1`. A final `v0.4.0` tag DOES NOT exist.

Command: `grep -n "^version" Cargo.toml`
```
9:version = "0.4.0"
```

Command: `grep -n -E "versionCode|versionName" android/build.gradle`
```
24:        versionCode = 15
25:        versionName = '0.4.0'
```
(app/build.gradle.kts:136-137 consumes these via rootProject.ext)

Command: `git rev-list --left-right --count main...HEAD`
```
0	85
```
main is a direct ancestor of this HEAD (main + 85 commits).

## Branch-base deviation (recorded, not improvised silently)

Task section 6 says to base `glm/canonical-outlier-audit` from current `main`.
That was not possible without violating AGENTS.md rules 11/12:

Command: `git diff --name-only HEAD main | wc -l`
```
71
```
Four files that are DIRTY in the shared checkout also differ between HEAD and main:
`HANDOFF/CTO_STATE.md`, `android/.../MeshRepository.kt`,
`android/.../NotificationHelper.kt`, `core/src/iron_core.rs`
(command: `git diff --name-only HEAD main | grep -E "iron_core|MeshRepository|..."`).
Checking out a main-based branch would refuse or overwrite those other sessions'
uncommitted changes. Sanctioned alternative used (AGENTS.md rule 11 "if you need an
isolated tree" spirit + task rule 1 "shared checkout, touch only your files"):
branch created from current HEAD instead:

Command: `git checkout -b glm/canonical-outlier-audit`
```
Switched to a new branch 'glm/canonical-outlier-audit'
```
Consequence for the PR: the branch is main + 85 feat/v040 commits + audit files.
Only the audit files listed in section 5 of the task are committed by this session;
the PR body must state that explicitly.

## Working evidence

Raw outputs saved under `tmp/canonical_audit_2026-09-19/`:
- `baseline.txt` (rev, status, tags, version)
- `tags.txt` (full tag list)

## Counts

- M0 baseline: complete. No findings yet (skeleton).

## Not done / UNVERIFIED

- Dimensions A-G: not started at iteration 0.

## Next iteration aim

- Read authority docs (task section 2), then DIM-A doctrine noun search.
