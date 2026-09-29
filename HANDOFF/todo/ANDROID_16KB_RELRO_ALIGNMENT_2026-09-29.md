# TODO: Android native libraries fail the 16 KB RELRO alignment check

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN, diagnosed, fix not started
Opened: 2026-09-29, from the operator's report on the fresh install
Priority: MEDIUM (a warning dialog for testers now; a Play Store requirement later)

## Symptom

On the operator's Pixel 6a (Android 17, 4 KB pages) the first launch of the
newest CI APK shows "This app isn't 16 KB compatible: RELRO alignment check
failed". The platform logged it as
`AppWarnings: Showing PageSizeMismatchDialog for package com.scmessenger.android`
at 21:02:30Z, one second after the launch. It is a warning only: the app started
its foreground service 12 seconds later and no native fault appears in logcat.

## Diagnosis (measured on the installed APK, CI Mobile run 36311537543)

Program headers of every arm64-v8a library, parsed from the APK:

| Library | LOAD `p_align` | `PT_GNU_RELRO` end | end mod 16 KB |
|---|---|---|---|
| `libscmessenger_core.so` (ours) | 0x4000 (ok) | 0x1206000 | 0x2000 (fails) |
| `libjnidispatch.so` (JNA) | 0x4000 (ok) | 0x2a000 | 0x2000 (fails) |
| `libandroidx.graphics.path.so` | 0x4000 (ok) | 0x6000 | 0x2000 (fails) |

So `-z max-page-size=16384` is already in effect (the LOAD segments are aligned),
but the RELRO segment ends on a 4 KB or 8 KB boundary. All twelve native
libraries in the APK are stored DEFLATED, so zip alignment is not the issue.

## Suspected fix (UNVERIFIED: prove it with the same ELF parse)

- Our library: also pass `-z common-page-size=16384` to the linker for the
  Android targets (cargo-ndk / `.cargo/config.toml` or the Gradle cargo step),
  so lld pads the RELRO end to 16 KB.
- Third-party libraries: check for newer JNA and `androidx.graphics:graphics-path`
  releases that are 16 KB clean; if none exist, decide whether the warning is
  acceptable for the 0.4.0 test builds.

## Acceptance

- The ELF parse shows every arm64-v8a library with `PT_GNU_RELRO` start and end
  both a multiple of 0x4000 and `p_align >= 0x4000`.
- The dialog no longer appears on an Android 17 device; no functional change.
- Add the ELF check to `scripts/verify_apk_native_libs.py` (CI already runs it on
  the debug APK) so the alignment cannot regress silently.

Tracking: the GitHub issue linked from `HANDOFF/ASTRA_HANDOFF_2026-09-29.md`.
