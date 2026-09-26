# P0  Android: Identity Generation Regression in Onboarding Flow

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** OPEN
**Severity:** P0 (blocker for new users)
**Reported by:** User, 2026-06-05 ~14:20 PT, via Telegram
**Detected in:** v0.2.3 (versionCode 7, installed 14:05:52 on adb-26261JEGR01896-6pHTac)
**Reporter's claim:** "Recent Android app update just pushed to my phone broke identity generation for onboarding flow."

## Symptoms (user-reported)

- Onboarding flow's identity-creation step fails / misbehaves after the v0.2.3 update.
- No FATAL exception observed in logcat for the identity path itself  user-visible failure is the gating signal.

## What I could observe without modifying code

1. `MeshRepository.createIdentity()` calls `ironCore.initializeIdentity()` and then `setNickname()`.
2. Logcat shows `initializeIdentity` being invoked **810 times in a single second** during cold start. This is multi-component contention, not necessarily a user-visible failure, but it is suspicious.
3. Identity *does* persist (cached fields visible: `peerId=12D3KooW`, `id=df73bbfa`, salt set, `identity_keys` blob in db)  but the pattern of repeated calls could mask a race where the *final* state is wrong.
4. `OnboardingScreen.kt` `IdentityStep` does not appear to await `createIdentity` to completion before navigating forward  needs verification.

## Repro steps (for the subagent that picks this up)

1. `adb shell pm clear com.scmessenger.android` (or fresh install on a clean device).
2. Launch, step through onboarding.
3. Enter nickname, submit identity step.
4. **Expected:** identity is created once, nickname stored, advance to next step.
5. **Actual (per user report):** identity step fails or app reaches a broken state.

## Files to investigate (precise)

| File | Why |
|---|---|
| `android/app/src/main/java/com/scmessenger/android/onboarding/OnboardingScreen.kt` (lines 110138) | The `IdentityStep` `createIdentity` call site. Check whether it `await`s the suspend function or fires-and-forgets; check for duplicate invocations on recomposition. |
| `android/app/src/main/java/com/scmessenger/android/mesh/MeshRepository.kt` | `createIdentity()`  does it guard against re-entry? Does it surface errors to the UI or swallow them? |
| `core/src/identity/` (IronCore FFI) | Confirm `initializeIdentity` is idempotent. If two callers race, do they clobber each other? |
| `core/src/identity/identity_manager.rs` or equivalent | Salt/keys generation  could the regression be a deterministic-vs-random change? |

## Hypothesis (entry point for the subagent)

The most likely regression is **one of:**

1. **Re-entrancy in `createIdentity()`**  recomposition or a LaunchedEffect re-fires the call, and the second call fails or overwrites the first.
2. **Missing `await` in `IdentityStep`**  the function returns before identity is actually persisted; navigation proceeds, next step reads stale state.
3. **Error swallow in `MeshRepository.createIdentity`**  returns `Result::Err` that the UI does not display, leaving the spinner forever.

## Cross-OS impact

If the bug is in `scmessenger-core` (FFI / identity module), this affects:
- iOS (Swift UniFFI bindings)
- WASM thin-client (browser)
- Linux/Windows CLI daemons
- All three client platforms have onboarding-style identity creation flows.

If the bug is in `OnboardingScreen.kt` only, Android-only.

**Action:** when fixing, audit all identity-creation call sites across platforms.

## Acceptance criteria

- [ ] Fresh-install onboarding creates identity exactly once.
- [ ] Onboarding proceeds to the next step only after identity is verified persisted.
- [ ] Error state is visible to user (not silent).
- [ ] `logcat` shows exactly one `initializeIdentity` call per onboarding session.
- [ ] Build verification: `cargo check --workspace` + `./gradlew :app:assembleDebug` both pass.
- [ ] Install on device, repro the original flow, confirm fix.
- [ ] Hand off a post-mortem to `HANDOFF/STATE/`.

## Out of scope (do NOT do in this ticket)

- Refactoring OnboardingScreen UI.
- Adding new identity features.
- Cross-OS triangulation work (separate ticket).

## Build environment (for the subagent)

Source on Windows: `/mnt/e/SCMessenger-Github-Repo/SCMessenger` (CRLF).
Build copy on Linux ext4: `/home/scmessenger/scmessenger-build/` (LF, faster).
Canonical build command lives in `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md`  Build Environment.

## Reference

- Adjacent context: `HANDOFF/STATE/2026-06-05_UNIFFI_BINDING_RACE_FIX.md` (build-pipeline fix from this session, unrelated to runtime identity bug).
