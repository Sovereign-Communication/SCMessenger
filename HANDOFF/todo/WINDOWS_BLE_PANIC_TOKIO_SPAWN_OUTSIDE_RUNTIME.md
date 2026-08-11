# Windows BLE crashes the node: tokio::spawn from a WinRT event handler

Status: Active
Severity: P1 -- enabling BLE on Windows kills the process; BLE transport is
unusable on this platform until fixed
Found: 2026-08-10 by the always-on soak, first run with `enable_ble: true`
Build: 0.4.0 (e5284b7b), sha256 ba888428..

## Symptom

With `enable_ble: true` in `%APPDATA%\scmessenger\config.json`, the node dies
17.6 s after start with exit code `3221226505` (0xC0000409, the Windows
fast-fail code a Rust abort produces):

    thread '<unnamed>' (22880) panicked at cli\src\ble_windows.rs:117:13:
    there is no reactor running, must be called from the context of
    a Tokio 1.x runtime

Immediately preceding it in the same run, BLE had come up and found a peer:

    16:08:34.603  btleplug: acquired Bluetooth manager; 1 adapter(s) visible
    16:08:34.717  Windows BLE: initializing GATT Service Provider...
    16:08:34.744  Windows BLE: starting peripheral LE advertisement...
    16:08:34.754  BLE scan active (filtering to SCM service 0000df01-..)
    16:08:35.079  BLE found matching peripheral: PeripheralId(67:D8:03:64:6B:A3)

So the crash arrives once BLE is actually exercised, not at initialisation.

## Root cause

`cli/src/ble_windows.rs:117` calls `tokio::spawn` inside a
`TypedEventHandler` registered on `identity_char.ReadRequested`:

    identity_char.ReadRequested(&TypedEventHandler::new(
        move |_sender, args| {
            let args_ref = args.ok()?;
            let deferral = args_ref.GetDeferral()?;
            ...
            tokio::spawn(async move {          // <-- line 117
                ...
            });
        },
    ))

WinRT invokes that callback on a Windows thread-pool thread. That thread has
no Tokio runtime in thread-local context, and bare `tokio::spawn` requires
one, so it panics. The panic crosses an FFI callback boundary and aborts the
process rather than unwinding.

This fires the moment a BLE central (the Android phone) issues a Read on the
identity characteristic -- i.e. exactly when BLE starts being useful.

## Fix

Capture a runtime `Handle` while still on a Tokio thread (at GATT setup time)
and use it inside the callback:

    let handle = tokio::runtime::Handle::current();   // setup, on a Tokio thread
    // ...
    let h = handle.clone();
    identity_char.ReadRequested(&TypedEventHandler::new(move |_sender, args| {
        // ...
        h.spawn(async move { ... });                  // not tokio::spawn
        Ok(())
    }))?;

Audit the whole file for the same pattern before fixing just this one --
`WriteRequested` on the message characteristic and any other handler
registered from `GattServiceProvider` will have the identical defect. A grep
for `tokio::spawn` inside `TypedEventHandler` closures in
`cli/src/ble_windows.rs` should come back empty when this is done.

Also worth checking: `deferral` is taken but, on the panic path, never
completed. Even after the spawn is fixed, confirm every early-return path
completes the deferral or the GATT operation hangs client-side.

## Verification

    cargo build -p scmessenger-cli
    # set enable_ble true in %APPDATA%\scmessenger\config.json
    # run the node, then have Android connect and read the identity char

Non-vacuous pass condition: the node survives an Android BLE identity read AND
`Windows BLE: ...` log lines continue after it. Process staying alive with BLE
never exercised is NOT a pass -- that is the state the bug hides in.

## THERE IS NO CONFIG MITIGATION -- `enable_ble: false` does not disable BLE

Setting `enable_ble: false` was tried first and **does not work**. Verified
empirically, not inferred:

    %APPDATA%\scmessenger\config.json  ->  "enable_ble": false   (mtime 16:10 UTC)
    node started by the scheduled task at 16:17:12 UTC
    grep -c 'ble_windows|ble_mesh|GATT Service Provider' <that run log>  ->  18

    16:17:48 ble_mesh:    BLE: CLI GATT central for service df01..
    16:17:48 ble_windows: Windows BLE: initializing GATT Service Provider...
    16:17:48 ble_windows: Windows BLE: starting peripheral LE advertisement...

So BLE comes up fully -- GATT provider, advertising and scanning -- with the
flag off, and the node remains exposed to the panic above.

This is a **second, separate defect** and it needs its own trace. What has been
ruled out so far:

- Not an env override: `SCMESSENGER_CONFIG` is unset in process, User and
  Machine scopes; no `config.json` in the repo root or the process cwd.
- Not a silent default fallback at the call sites: all three BLE start blocks
  (`cli/src/main.rs:1949`, `:1978`, `:3142`) are inside `if config.enable_ble`.
- Not a parse-failure fallback: every `Config` field carries
  `#[serde(default)]`, and the `Config::load()` call sites in the start path
  use `?`, not `unwrap_or_default()`.
- Not a stale file: the value reads back as `false` immediately before and
  after the affected node starts.

Which means `config.enable_ble` is somehow true at those call sites while the
on-disk value is false. Whoever picks this up should start by logging the
loaded `config.enable_ble` at the top of the start path -- note
`cli/src/config.rs:73` has `enable_ble: true` in the `Default` impl, so any
path that reaches `Config::default()` rather than the file turns BLE on.

Practical consequence: **while the operator's phone is nearby with BLE on, the
Windows node cannot be kept up by configuration alone.** The interim options
are to turn BLE off on the phone, or land the `handle.spawn` fix above. Left
alone, the repeated panics trip the soak's 5-restarts-per-hour cap and halt it
with the evidence preserved.

## Note on how this was found

The soak supervisor caught it working as intended: classified `immediate_exit`
(17.6 s, under the 30 s crash-loop threshold), captured a bundle with the run
log and node logs before relaunching, and restarted into generation 2. Bundle:

    %LOCALAPPDATA%\scmessenger\soak\artifacts\20260810T160816Z_immediate_exit\

Left running with BLE enabled, the repeated panics would have tripped the
5-restarts-per-hour cap and halted the soak with the evidence preserved.

## IMPLEMENTATION CONTRACT (handoff to the Windows/Android transport lane)

Assigned to the session working `windows-lane/android-parity-dial-dedup`
(worktree `C:\Users\SCM\Documents\GitHub\scm-winlane`) by operator instruction,
2026-08-10. This is scoped work with a definition of done, not a research task.

**Two defects, both required. Fixing only defect 1 leaves BLE unusable; fixing
only defect 2 leaves the node crashing.**

### Defect 1 -- `tokio::spawn` from a WinRT callback (the panic)

Files: `cli/src/ble_windows.rs` (confirmed at `:117`), and any sibling handler
in the same file registered on a `GattLocalCharacteristic`.

- [ ] Capture `tokio::runtime::Handle::current()` at GATT setup time, on a
      Tokio thread, before registering handlers.
- [ ] Replace every `tokio::spawn` inside a `TypedEventHandler` closure with
      `handle.spawn`.
- [ ] Grep gate: `tokio::spawn` must not appear inside any
      `TypedEventHandler` closure in `cli/src/ble_windows.rs`.
- [ ] Confirm every early-return path completes its `GetDeferral()` deferral.
      The panic path currently takes a deferral and never completes it, which
      hangs the GATT client even once the panic is gone.

### Defect 2 -- `enable_ble: false` does not disable BLE

Files: `cli/src/main.rs` start path (`:1949`, `:1978`, `:3142`),
`cli/src/config.rs` (`:73` has `enable_ble: true` in the `Default` impl).

- [ ] Log the loaded `config.enable_ble` at the top of the start path and
      identify why it is true at the call sites while the file says false.
- [ ] Fix so the on-disk value is authoritative.
- [ ] Regression gate: with `"enable_ble": false`, a started node produces
      **zero** `ble_windows` / `ble_mesh` / `GATT Service Provider` log lines.
      Command: `grep -c 'ble_windows\|ble_mesh\|GATT Service Provider' <run log>`
      must print `0`. It printed `18` when this was filed.

### Definition of done -- do NOT mark this complete without

1. `cargo test --workspace --no-run` passes (repo compile gate). Note the
   workspace `target/debug/deps` was reclaimed on 2026-08-10, so budget for a
   cold rebuild; ~17 GB free at time of writing.
2. The defect-2 regression gate above prints `0`.
3. With `enable_ble: true`, the node **survives an Android BLE identity
   characteristic read**, and `Windows BLE:` log lines continue after it.
   A node that stays alive with BLE never exercised is a VACUOUS pass -- that
   is precisely the state the bug hides in. Drive a real read from the Pixel.
4. The always-on soak stays up with BLE enabled and the phone nearby:
   `python scripts/soak_supervisor.py status` shows no new
   `*_immediate_exit` bundle in
   `%LOCALAPPDATA%\scmessenger\soak\artifacts\` across at least 15 minutes.

### Coordination notes

- The Windows node runs continuously under the `SCMessengerSoak` scheduled
  task (logon-triggered, user `SCM`). Stop it before rebuilding the binary:
  `python scripts/soak_supervisor.py stop`. The soak pins a binary by sha256
  and will **halt rather than silently soak a different build**, so after
  rebuilding, re-pin deliberately:
  `python scripts/soak_supervisor.py pin <path-to-new-exe>`.
- The pinned soak binary is a COPY at
  `%LOCALAPPDATA%\scmessenger\soak\bin\scmessenger-cli-e5284b7b.exe`, not the
  one in `target/`. Rebuilding `target/` does not affect the running soak.
- The operator's phone -> orchestrator bridge depends on the Android identity
  `a43772fe..`. If this work reinstalls the Android app, see
  `HANDOFF/todo/ANDROID_REINSTALL_UPDATE_INBOX_BRIDGE_ALLOWLIST.md` -- the
  bridge fails silently, with no error on any side.

## Related

- `HANDOFF/IN_PROGRESS/ANDROID_RELAY_INBOUND_EVIDENCE_2026-08-10_CELLULAR.md`
  -- the relay/inbound investigation this was found alongside. BLE was `false`
  for the whole of that test (though per defect 2, BLE was in fact running),
  so BLE was not the transport under test there.
