# Logging Levels Audit — 2026-08-08

**Generated:** 2026-08-08
**Scope:** Full-repo logging verbosity audit across Rust core/CLI, Android, iOS,
WASM, triggered by a live Android<->iOS field test that produced 151 app-owned
lines out of a 43,608-line logcat capture, with zero lines from
`MdnsServiceDiscovery` and `SubnetProbe`.
**Method:** Static analysis only. No build was run (disk at 98% full, 6.5 GB
free). Two source edits were made (see Section E); both are logging
configuration only, no logic/control-flow changes.

---

## Executive Summary

- Android's Kotlin logging (Timber) is **not** the bottleneck: in a debug
  build (`assembleDebug`, `debuggable=true`), `Timber.DebugTree` logs every
  call site unconditionally, and Android does not apply its "non-debuggable
  app" log-level throttle to a debuggable app. If `MdnsServiceDiscovery` and
  `SubnetProbe` code actually ran, their `Timber.i/d/w/e` calls would appear
  in `adb logcat` with no setprop needed.
- The most likely reason those two classes produced zero lines is that their
  code **never ran**, not that their output was filtered. `TransportManager
  .startAll()` only constructs and starts them when
  `enableMdns = settings.internetEnabled` is `true`
  (`MeshRepository.kt:2380`, `:10124`); the default is `true`, but this is a
  per-device, per-install runtime setting worth confirming on the test
  devices before the next run. See Section D for the full reasoning and the
  verification step to run.
- The Rust core's structured JSON tracing (`outbox_enqueue`, `outbox_dequeue`,
  `inbox_receive`, etc., all at INFO) writes to a **file**
  (`<log_dir>/scmessenger-mesh.log`), never to logcat/os_log. Nothing bridges
  it to the platform log stream on Android or iOS. This file must be pulled
  from the device after the test — it was very likely the actual explanation
  for "the CLI has good logs but the phones don't": the phones *do* have
  comparable logs, just not in logcat.
  `RUST_LOG` cannot be set on mobile (no process environment to export it
  from), so the mobile file log is locked to whatever fallback level is
  compiled in. This is the one genuine build-time gap with no runtime
  workaround (Section C, item 1) — it has been raised for debug/dev builds
  (Section E) but requires a rebuild to take effect, and does not affect iOS
  at all (iOS always compiles the Rust core with `--release`).
- `Bootstrap all-failed (consecutive=28)` (`MeshRepository.kt:8962`) is
  misleading, not under-logged: `primeRelayBootstrapConnections()`
  (`MeshRepository.kt:8927`) has `val addresses = emptyList<String>()`
  hardcoded. Zero bootstrap addresses are ever attempted, so the log is
  reporting "0 of 0 succeeded" as if it were "N of N failed." No amount of
  verbosity will produce per-endpoint detail because no endpoints are dialed.
  This is a logic bug, not a logging gap — flagged, not fixed (out of scope
  per task constraints).
- The Windows CLI's single `os error 10040` (WSAEMSGSIZE) line for mDNS
  comes from the `libp2p-mdns` crate's own internal tracing, at a target
  (`libp2p_mdns`) the app's default filter does not raise. `RUST_LOG` target
  scoping fixes this today, no rebuild needed (Section B).

---

## A. Current state table

| Lane | Default level | Set at (file:line) | Compile-time stripped? | Runtime filtered? |
|---|---|---|---|---|
| **Rust core (file tracing)**, used by Android+iOS via `IronCore::with_storage_and_logs` | `info` (fallback); `RUST_LOG` overrides if set | `core/src/store/tracing_init.rs:51` (now `:51-63` after this audit's edit, see Section E) | No — no `max_level_*`/`release_max_level_*` feature is set on `log`/`tracing` anywhere in the workspace (`Cargo.toml`, `core/Cargo.toml`, `cli/Cargo.toml`) | Yes, via `tracing_subscriber::EnvFilter`. `RUST_LOG` cannot reach mobile processes, so mobile is always on the fallback. |
| **Rust CLI (`scmessenger-cli`)** | `info` (fallback); `RUST_LOG` overrides if set | `cli/src/main.rs:657-658` | No | Yes, `EnvFilter`, dual output: stdout (`:662`) + hourly-rotating file at `<data_dir>/logs/scm.log` (`:652-653,:664-667`) |
| **Android Kotlin (Timber)**, debug build | Everything (V/D/I/W/E) unconditionally, to logcat + file | `MeshApplication.kt:43-45` (`BuildConfig.DEBUG` branch → `Timber.DebugTree()` + `FileLoggingTree`) | No | No level filter in debug. `Timber.DebugTree` calls `Log.println()` directly; because `debuggable=true` (`android/app/build.gradle:141`), Android's per-app log-level throttle for non-debuggable apps does not apply either. |
| **Android Kotlin (Timber)**, release build | WARN/ERROR only to logcat; **all levels** to file (see note) | `MeshApplication.kt:47-50`, `ReleaseTree.log()` at `:139-145` | No (ProGuard keeps `timber.log.**` and does not `-assumenosideeffects` any Log call — verified in `android/app/proguard-rules.pro`) | Yes for logcat (`ReleaseTree` drops `priority < Log.WARN`, `MeshApplication.kt:141`). **No** for the file: `FileLoggingTree.log()` (`android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt:28-74`) has no priority check at all, despite the comment at `MeshApplication.kt:48-49` claiming it "writes only WARN+ (it is gated by priority)". See Section D privacy note. |
| **iOS (os.Logger)** | Unified logging default (`.default`/`.info` not persisted by default; `.error`/`.fault` always persisted) | Per-file `Logger(subsystem: "com.scmessenger", category: "...")`, e.g. `iOS/SCMessenger/SCMessenger/Transport/mDNSServiceDiscovery.swift:20` | No compile-time stripping found (no `#if DEBUG` gates around logger calls in the files sampled) | Yes, by the OS unified-logging persistence policy, independent of app code — see Section B for the Console.app/`log` runtime knob. |
| **Rust core (file tracing) on iOS** | Same `info` fallback as Android, via the same `tracing_init.rs` | Same file | No | Same as Android, but the debug/dev-profile bump added in this audit (Section E) **does not apply to iOS**: `scripts/rebuild_ios_core.sh:6-7` always builds with `cargo build --release --target aarch64-apple-ios[-sim]`, so `debug_assertions` is always `false` for the shipped xcframework regardless of Xcode scheme. |
| **WASM (`tracing-wasm`)** | Full `TRACE`, no app-side filter | `wasm/src/lib.rs:176-186`, `init_logging()` called from 4 constructors (`:221,252,283,314`) | No | No app-side filter; verbosity as seen in the browser is controlled entirely by the DevTools console's own log-level toggles (Verbose/Info/Warnings/Errors), not by this app. |

---

## B. Runtime knobs — HIGHEST PRIORITY (no rebuild required)

### Android (debug APK, already installed)

Kotlin/Timber logging in a debug build is **already maximally verbose** and
unfiltered by Android's log level system (the app is `debuggable=true`, so
the OS does not throttle DEBUG/VERBOSE the way it does for a production app).
The problem observed (151 of 43,608 lines were app-owned) is a **capture/
filtering** problem, not a verbosity problem. Use tag-scoped `adb logcat`
so the framework noise never enters the capture in the first place, pull the
Rust-core file log (which never reaches logcat at all), and confirm the
mDNS/LAN-discovery gate is actually on:

```bash
# 1. Confirm the app + its Rust core are actually debuggable/at the level we expect
adb shell dumpsys package com.scmessenger.android | grep -i "debuggable\|versionName"

# 2. Bump the OS-level per-tag throttle anyway (no-op on a debuggable app, but
#    free insurance in case a release/staged build is what's actually installed)
for TAG in MdnsServiceDiscovery SubnetProbe TransportManager MeshRepository \
           SwarmBridge WifiAwareTransport WifiDirectTransport NetworkDetector \
           TransportHealthMonitor MeshForegroundService MeshService; do
  adb shell setprop log.tag.$TAG VERBOSE
done

# 3. Grow the on-device ring buffer so high framework noise doesn't rotate
#    app lines out before you can pull them (default is often 1-4M per buffer)
adb logcat -G 16M

# 4. Capture ONLY the app's own tags for the duration of the test (this is
#    the fix for "151 of 43608 lines" -- everything else was framework noise
#    crowding the same buffer)
adb logcat -v threadtime \
  MdnsServiceDiscovery:V SubnetProbe:V TransportManager:V MeshRepository:V \
  SwarmBridge:V WifiAwareTransport:V WifiDirectTransport:V NetworkDetector:V \
  TransportHealthMonitor:V MeshForegroundService:V MeshService:V \
  AndroidRuntime:E *:S > android_app_only.log

# 5. Separately, ALSO capture unfiltered in parallel (belt-and-suspenders,
#    in case a tag was missed above) -- run in a second terminal
adb logcat -v threadtime > android_full.log

# 6. After the test: pull the Rust core's own structured JSON log. This is
#    the file that has outbox_enqueue/outbox_dequeue/inbox_receive events at
#    INFO -- it NEVER appears in logcat, it must be pulled separately.
adb shell run-as com.scmessenger.android \
  cat files/logs/scmessenger-mesh.log > core_mesh_log.jsonl
# Also worth pulling -- the Timber/FileLoggingTree mirror (same events, plus
# WARN/ERROR that may not have a Rust-side equivalent):
adb shell run-as com.scmessenger.android \
  cat files/mesh_diagnostics.log > android_file_tree.log

# 7. Verify the mDNS/LAN-discovery runtime gate is ON before the test starts
#    (Settings screen: "Internet"/LAN toggle -> internetEnabled). If this is
#    off, TransportManager.startAll() takes the else-branch and NEVER
#    constructs MdnsServiceDiscovery or SubnetProbe -- zero lines is then
#    expected behavior, not a bug. In-app: Settings > Network > confirm the
#    "Internet" toggle is on. Via adb, look for either log line right after
#    mesh start:
#      "All transports started (including mDNS LAN discovery + TCP subnet probe)"
#      "All transports started (mDNS/LAN disabled as requested)"
adb logcat -d | grep "All transports started"
```

Tags that matter for the three things this field test needs (message
delivery, transport selection, peer discovery): `MdnsServiceDiscovery`,
`SubnetProbe`, `TransportManager`, `MeshRepository`, `SwarmBridge`,
`WifiAwareTransport`, `WifiDirectTransport`, `NetworkDetector`,
`TransportHealthMonitor`, `MeshForegroundService`, `MeshService`.

### Windows CLI node (PowerShell)

```powershell
# Broad transport + delivery visibility for a bootstrap/relay run
$env:RUST_LOG = "info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug,scmessenger_core::routing=debug"
.\target\debug\scmessenger-cli.exe start

# Specifically to get context on the single "os error 10040" (WSAEMSGSIZE)
# mDNS line -- this comes from the libp2p-mdns crate's OWN tracing calls,
# at a target our default filter never raises:
$env:RUST_LOG = "info,libp2p_mdns=trace,if_watch=debug,scmessenger_core::transport=debug"
.\target\debug\scmessenger-cli.exe start
```

Both stdout (`cli/src/main.rs:662`) and the rotating file at
`<data_dir>/logs/scm.log` (`:652-667`) honor `RUST_LOG` identically, so no
extra flag is needed to also get the file capture.

### macOS relay node (`scripts/run5.sh`)

The script already had a working precedent for target-scoped `RUST_LOG`
(`OSX_RUST_LOG` at line ~56). It was fixed as part of this audit — see
Section E — because one of its targets was a dead module path. After the
fix it now includes `libp2p_mdns=trace`, `if_watch=debug`, and the correct
`scmessenger_core::store::outbox`/`inbox` targets. No further action needed
to use it; just run the script as normal.

### iOS (os_log / Console.app), no rebuild

Unified logging on iOS does not use an env var; `.debug`/`.info` messages
from `Logger` calls are captured in memory but not shown/persisted by
default. To raise the effective capture level for an already-installed
build:

```
# With the device connected to a Mac and the app running:
# Xcode > Window > Devices and Simulators > select device > Open Console
# Console.app "Action" menu (or the device console's toolbar) ->
#   check "Include Debug Messages" and "Include Info Messages"
# Then filter by subsystem in the search bar:
subsystem:com.scmessenger

# Command-line equivalent (Mac, device attached, from Terminal):
log stream --level debug --predicate 'subsystem == "com.scmessenger"'
```

Categories that matter: `mDNS`, `Repository`, `Platform`, `TransportRouter`,
`Multipeer`, `BLE-Central`, `BLE-Peripheral`, `BLE-L2CAP`, `CoreDelegate`,
`Background`.

For the Rust-core file log on iOS, the equivalent of the Android `run-as`
pull is retrieving `<Documents>/scmessenger-mesh.log` via Xcode's device
file browser (Window > Devices and Simulators > select device > select the
app under Installed Apps > gear icon > Download Container), then inspecting
`AppData/Documents/scmessenger-mesh.log` in the downloaded `.xcappdata`
bundle.

### WASM / browser

No app-side knob exists or is needed — `tracing-wasm` already emits at full
`TRACE`. Open DevTools console and ensure "Verbose"/"Info" log levels are
enabled in the console's own level filter (they are hidden by default in
Chrome DevTools even though the app emits them).

---

## C. Build-time gaps

1. **Rust core mobile file-tracing default cannot be raised without a
   rebuild — and cannot be raised on iOS at all via this mechanism.**
   `core/src/store/tracing_init.rs:51` (pre-audit) hardcoded
   `EnvFilter::new("info")` as the fallback used whenever `RUST_LOG` is
   unset — which is always, on mobile, since there is no process
   environment to set it from. This audit changed the fallback to be
   richer under `cfg(debug_assertions)` (Section E), but:
   - It only takes effect on the **next** debug build/install — it does not
     help a field test against the currently-installed APK/app.
   - It does **not** help iOS under any circumstance: `scripts/
     rebuild_ios_core.sh:6-7` always runs `cargo build --release`, so
     `debug_assertions` is `false` for the iOS xcframework regardless of
     the Xcode scheme used to build the app around it. Raising iOS's
     mobile-file-log verbosity would require either building a second,
     non-release xcframework variant for field-test use, or plumbing an
     actual runtime-configurable filter through the FFI (a `set_log_level`
     UniFFI function backed by `tracing_subscriber::reload::Handle`) —
     this is a real code change, not a config tweak, and was left
     unimplemented per the "do not change program logic" constraint.
   Minimal follow-up (not done here, flagged instead): add a
   `uniffi::export` function like `IronCore::set_log_filter(directive:
   String)` wrapping a `tracing_subscriber::reload::Handle`, then call it
   from a debug-only settings toggle on both platforms.

2. **No bridge from Rust `tracing` to Android `logcat` or iOS `os_log`.**
   No `android_logger`, `tracing-android`, `paranoid_android`, or
   equivalent crate is a dependency anywhere in the workspace (checked
   `Cargo.toml`, `core/Cargo.toml`, `cli/Cargo.toml`, `wasm/Cargo.toml`).
   The Rust core's tracing output is file-only on mobile. This is why the
   151-line logcat capture could never have contained the
   `outbox_enqueue`/`inbox_receive`/etc. events even if every setting were
   correct — they were never going to be in logcat, they are in
   `scmessenger-mesh.log`. Not fixed here (adding a new logging-bridge
   dependency and wiring a second `tracing_subscriber` layer is more than
   a "levels/filters" config change and needs its own review); the runtime
   workaround is Section B item 6 (pull the file).

3. **Android release build: `FileLoggingTree` has no priority gate despite
   documentation claiming one.** See Section D privacy note — not a
   verbosity gap (it under-filters, not over-filters), listed here because
   fixing the comment-vs-code mismatch is a code change and was left
   flagged rather than fixed, per the "confined to debug/dev builds"
   constraint (this file is shared by both build types).

4. **No debug/trace instrumentation in the outbox/inbox hot path.**
   `core/src/store/outbox.rs` (1,145 lines) has exactly 5 `tracing::` call
   sites total (all `info`/`warn`, none `debug`/`trace`);
   `core/src/store/inbox.rs` (593 lines) has exactly 1. The existing calls
   cover the high-level lifecycle events (`outbox_enqueue`,
   `outbox_dequeue`, `inbox_receive`) but nothing in between — e.g. no
   per-retry, per-recipient-resolution, or backoff-decision tracing. Adding
   this is new logging *statements*, not configuration, so it was not done
   here; flagged as a follow-up (see Section F).

None of the above involve a `max_level_*`/`release_max_level_*` `log`/
`tracing` feature — a full search of every `Cargo.toml` in the workspace
found none set, so nothing is statically compiled out in the sense the task
asked about; the gaps are all "no lever exists yet" rather than "a lever was
disabled."

---

## D. Can `MdnsServiceDiscovery` and `SubnetProbe` log at all in the shipped debug APK, and at what level?

**Yes, unconditionally, at all levels (`Timber.i/d/w/e`), with no internal
gate of their own** — neither class checks `BuildConfig.DEBUG`,
`Log.isLoggable`, or any level threshold before logging (verified by reading
both files in full:
`android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`,
`android/app/src/main/java/com/scmessenger/android/transport/SubnetProbe.kt`).
Every method — `start()`, `onServiceFound`, `onServiceResolved`,
`onStartDiscoveryFailed`, `onRegistrationFailed`, `onResolveFailed`, etc. —
logs at INFO, DEBUG, WARN, or ERROR unconditionally on every call.

**Their logging does not sit below what a default `adb logcat` capture
shows.** In a debug build:
- `Timber.DebugTree` (`MeshApplication.kt:44`) forwards every call straight
  to `android.util.Log.println()` with no filtering.
- The app is `debuggable=true` (`android/app/build.gradle:141`), which
  exempts it from Android's OS-level throttle that would otherwise silently
  drop DEBUG/VERBOSE from a non-debuggable app's UID at the logging-daemon
  level. That throttle — the actual mechanism `adb shell setprop
  log.tag.<TAG> VERBOSE` exists to override — does not apply here in the
  first place.
- `adb logcat` with no priority filter shows all levels by default.

**Conclusion: their silence in the 43,608-line capture means they never
ran** — this is not a logging-level problem to fix, it is a "did the code
path execute" problem. Static analysis found the specific gate that could
cause exactly this:

`TransportManager.startAll(enableMdns: Boolean = true)` at
`android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt:103-138`
only calls `getOrCreateMdns()`/`discovery.start()` and
`getOrCreateSubnetProbe()`/`probe.start()` inside `if (enableMdns)`
(`:124-134`). Both call sites that invoke `startAll` pass
`enableMdns = settings.internetEnabled` (`MeshRepository.kt:2380` and
`:10124`). If that setting is `false` on the test device, the `else` branch
runs instead (`:135-137`, logs only `"All transports started (mDNS/LAN
disabled as requested)"`) and **neither class is ever constructed** — zero
lines is then the fully expected, correct result, not a bug.
`internetEnabled` defaults to `true` in every constructor found
(`MeshRepository.kt:5298`, `:6010`; `SettingsViewModel.kt:199`), so a fresh
install should have it on — but it is a persisted, user-toggleable setting,
so it is worth explicitly confirming on the actual test device rather than
assuming the default held. Section B step 7 gives the exact check
(`grep "All transports started"` in the capture) to run immediately after
the next field test to settle this either way.

A second, independent possibility (also consistent with zero lines and
worth ruling out in the next capture) is an exception thrown earlier in the
same coroutine, in `initializeAndStartBle()`/`initializeAndStartWifi()`/
`initializeAndStartSwarm()` (`MeshRepository.kt:2354-2369`), before
`ensureTransportManager()`/`startAll()` is even reached at `:2372-2383` —
each of those three calls is independently try/caught and logged with
`Timber.w`, so checking whether those three warning lines are present in
the next capture will confirm or rule this out.

**Privacy note (Android release only, unrelated to the field-test question
above but found during this audit and in scope per the task's privacy
constraint):** `MeshApplication.kt:48-49` comments that the release-build
`FileLoggingTree` "writes only WARN+ (it is gated by priority)", but
`FileLoggingTree.log()` (`android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt:28-74`)
has no priority check anywhere in its body — it writes every priority level
it receives, in both debug and release builds, to
`<filesDir>/mesh_diagnostics.log` and to the `IronCore.recordLog()`
summarizer. This audit's targeted search for plaintext/key material passed
to `Timber.*` calls found none, but the comment-vs-code mismatch means the
intended release-build privacy boundary for this on-device file is not
actually enforced by code, only by convention of what call sites happen to
log. Not fixed here (a real level-check inside a shared file is more than
a per-build-type config toggle and touches release behavior); flagged as a
follow-up in Section F.

---

## E. Edits made (logging configuration only)

1. **`core/src/store/tracing_init.rs`** — the mobile file-tracing fallback
   filter (used only when `RUST_LOG` is unset, which is always true on
   mobile) is now `#[cfg(debug_assertions)]`-gated: dev/debug cargo profiles
   get
   `"info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug"`;
   release profiles keep `"info"` unchanged. Confirmed this maps correctly
   to Android's Gradle build types: `android/app/build.gradle:403`
   (`def rustProfile = isRelease ? "--release" : ""`) means `assembleDebug`
   compiles the Rust core in the `dev` cargo profile
   (`debug_assertions=true`), `assembleRelease` in `release`
   (`debug_assertions=false`, per the workspace `[profile.release]` in the
   root `Cargo.toml`, which does not override `debug-assertions` so it
   keeps Cargo's own default of `false`). **Requires a rebuild + reinstall
   to take effect; does not help the currently-installed APK or iOS (see
   Section C item 1).**
2. **`scripts/run5.sh`** — `OSX_RUST_LOG` referenced
   `scmessenger_core::mesh::delivery=debug`, a module that does not exist
   (`core/src/lib.rs` has no `mesh` module); `EnvFilter` silently ignores
   unmatched targets, so that clause was inert. Replaced with the correct
   `scmessenger_core::store::outbox`/`store::inbox` targets, and added
   `libp2p_mdns=trace`/`if_watch=debug` to give the mDNS `os error 10040`
   line real context. This is a shell-script constant, takes effect
   immediately on next run, no rebuild involved.

No ProGuard rules needed changing (no `-assumenosideeffects` strips any log
call today). No Android/iOS/WASM logging *configuration* files needed
changing beyond the above — the verbosity problem on mobile is overwhelmingly
a capture-methodology and file-pull problem (Section B), not a
level-is-too-low problem.

---

## F. Flagged follow-ups (out of scope for this audit; not fixed)

- `MeshRepository.kt:8927`: `primeRelayBootstrapConnections()` has
  `val addresses = emptyList<String>()` hardcoded, so `Bootstrap all-failed`
  always fires with zero real dial attempts. Logic bug, not a logging gap.
- `core/src/store/outbox.rs`/`inbox.rs`: add `tracing::debug!`
  instrumentation for per-retry / per-recipient-resolution / backoff
  decisions inside the outbox/inbox hot path (currently only lifecycle
  entry/exit events exist, at `info`).
- `FileLoggingTree.kt` (Android): either implement the priority gate the
  `MeshApplication.kt:48-49` comment claims exists, or correct the comment
  to describe actual behavior.
- A real runtime-configurable Rust log filter (UniFFI `set_log_filter`
  backed by `tracing_subscriber::reload::Handle`) would let both Android and
  iOS raise mobile file-tracing verbosity from an in-app debug toggle
  without a rebuild — the current fallback-string approach (Section E) is a
  static, compile-time-only lever.
