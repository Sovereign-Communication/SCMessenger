# 2026-06-05 — UniFFI binding gen race fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Incident
Background gradle build (`proc_dd98199660eb`) failed with:

```
Task :app:generateUniFFIBindings FAILED
thread 'main' panicked at core/src/bin/gen_kotlin.rs:36:9:
scmessenger_mobile cdylib not found. Please run: cargo build -p scmessenger-mobile
Process 'command 'cargo'' finished with non-zero exit value 101
```

## Root cause
The `doLast` block in `android/app/build.gradle` did two cargo invocations in sequence:

1. `cargo build -p scmessenger-mobile` → produces cdylib at `<CARGO_TARGET_DIR>/debug/libscmessenger_mobile.so`
2. `cargo run --bin gen_kotlin --features gen-bindings` → **rebuilds scmessenger-core with the gen-bindings feature**, which changes scmessenger-core's fingerprint. The mobile crate depends on scmessenger-core, so cargo invalidates the previously-built mobile cdylib to rebuild it with the new core fingerprint.

The gen_kotlin binary runs as part of step 2's `cargo run`, but at that moment the cdylib is in a transient state — either deleted, moved, or being rebuilt — so `Utf8Path::exists()` returns false on the hardcoded relative path `../target/debug/libscmessenger_mobile.so` and the script panics with "cdylib not found."

The race was **intermittent**: the same binary invoked directly AFTER the build finished would find the cdylib. The build itself is non-deterministic by design (cargo's incremental rebuild only deletes files it needs to rebuild, and the gen_kotlin binary runs as a build step, not after the build).

## Fix (commit 86d60276)
Three changes:

### 1. `android/app/build.gradle` — split `doLast` into 4 explicit steps
```
1. cargo build -p scmessenger-mobile (host target)
2. STAGE the cdylib to core/target/staged-cdylib/   ← new, decouples from cargo target dir
3. cargo build --bin gen_kotlin --features gen-bindings  ← was cargo run; build only
4. exec the gen_kotlin binary directly with SCMESSENGER_CDYLIB_PATH env var
```

### 2. `core/src/bin/gen_kotlin.rs` and `gen_swift.rs` — search-order priority
```
1. SCMESSENGER_CDYLIB_PATH (explicit override)         ← new
2. CARGO_TARGET_DIR (debug/release + Android ABIs)     ← new, fixes prior bug
3. Hardcoded relative paths (host triple, MSVC)         ← kept as fallback
4. Panic with actionable message
```

The `CARGO_TARGET_DIR` lookup covers:
- `<target>/debug/libscmessenger_mobile.{so,dll,dylib}`
- `<target>/release/libscmessenger_mobile.{so,dll,dylib}`
- `<target>/<triple>/debug/libscmessenger_mobile.so` for all 4 Android ABIs and 2 iOS simulator targets (the cdylib compiled for mobile targets lives under the triple subdir)

### 3. CRLF preservation
Files on `/mnt/e/...` (Windows side) use CRLF; the build copy on `/home/...` (Linux side) uses LF. Initial sync via `cp` introduced a spurious line-ending change across the whole workspace. Fixed by running `sed -i 's/$/\\r/'` on the source repo after syncing the build copy's LF files. The spurious `mobile_bridge.rs`, `behaviour.rs`, `swarm.rs` "changes" were reverted with `git checkout -- ...`.

## Verification
- `cargo check --bin gen_kotlin --features gen-bindings` → clean (2m 29s)
- `./gradlew :app:assembleDebug -x lint --no-daemon --offline` → **BUILD SUCCESSFUL in 1m 42s**, 46 actionable tasks, 4 executed, 42 up-to-date
- `adb install -r` → Success, app launches (PID 23656), no FATAL EXCEPTION

## Build env reminders (don't break these)
- `JAVA_HOME=/home/scmessenger/.local/jdk/jdk-17.0.12+7` (the `+7` variant — the `-7` one doesn't exist)
- `ANDROID_NDK_HOME=/home/scemessenger/android-sdk/ndk/26.1.10909125`
- `CARGO_TARGET_DIR=/home/scemessenger/.cargo-target` (NOT the build copy's target/)
- Build from `/home/scemessenger/scmessenger-build/android` (NOT `/mnt/e/...` — 9P bridge I/O errors)
- `CARGO_INCREMENTAL=0` is required to prevent rlib metadata corruption
- `GRADLE_USER_HOME=/home/scemessenger/.gradle` is required to keep C: from filling

## Commits
- `86d60276` fix(android): robust UniFFI binding gen — stage cdylib + honor CARGO_TARGET_DIR

## Artifacts
- `tmp/APP-v0.2.3-debug.apk` (E: drive, 278 MB) — MD5 `65c5a0b4eed51df1d978c441cdb6412c`
- `/home/scmessenger/APP-v0.2.3-debug.apk` (linux native, 278 MB) — same MD5
- Installed on Pixel 6a `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`, versionName 0.2.1, versionCode 7, lastUpdateTime 2026-06-05 14:05:52
