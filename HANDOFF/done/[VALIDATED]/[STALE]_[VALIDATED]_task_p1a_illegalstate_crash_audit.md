# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 2880

# Task: Audit and Harden IllegalStateException Crash Sites in MeshRepository.kt

**Priority:** P1
**Model:** qwen3-coder-next:cloud
**Budget:** 2880
**Assigned to:** implementer
**Created:** 2026-05-13
**Status:** FAILED  prematurely moved to done/ without code changes. MeshRepository.kt still has 13 IllegalStateException throw sites.
**Source:** MASTER AUDIT 2026-05-13 (P1 Android Hardening, sub-task 2)

## Summary

`MeshRepository.kt` contains 13 `IllegalStateException` throw sites that crash the app on invariant violations. Most represent recoverable states where a logged warning + graceful fallback is better than a crash. Audit each site and convert to safe fallbacks where possible.

## Current State

13 IllegalStateException sites remain in `MeshRepository.kt` (was 14, 1 already removed):

| Line | Context | Recoverable? |
|------|---------|-------------|
| 128 | Core init guard | Likely  fallback to uninitialized state |
| 2100 | Service not RUNNING | Likely  retry/queue |
| 2124 | Service startup failed | Likely  wrap exception, retry path |
| 3015 | Core not initialized | Likely  return error result |
| 3711 | Null IronCore for nickname | Yes  return early |
| 3729 | Nickname persist failed | Yes  log + return error |
| 3803 | Invalid contact public key | Yes  return error to caller |
| 3904 | Invalid public key format | Yes  return error to caller |
| 3922 | Headless relay as recipient | Keep  this is a logic bug |
| 3930 | IronCore not initialized | Yes  return error result |
| 3934 | Empty message ID | Keep  true invariant |
| 4031 | (needs reading) | Audit required |
| 4179 | Service init failed | Likely  retry path |

## What To Do

1. Read `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` lines around each of the 13 sites listed above
2. For each site:
   - If the error is recoverable (caller can handle null/error): replace `throw IllegalStateException(...)` with `Timber.w(...)` + a fallback return (null, error result, early return)
   - If the invariant truly makes continued execution impossible: keep the throw but add a comment explaining why
3. Document how many were converted vs kept

## Target

Reduce from 13 to <= 3 remaining `IllegalStateException` throw sites (only the truly unrecoverable ones).

## Verification

1. `cd android && ./gradlew assembleDebug -x lint --quiet` must pass
2. `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` must pass
3. Count of `IllegalStateException` in MeshRepository.kt before and after documented in commit message

## Files Expected to Change

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
