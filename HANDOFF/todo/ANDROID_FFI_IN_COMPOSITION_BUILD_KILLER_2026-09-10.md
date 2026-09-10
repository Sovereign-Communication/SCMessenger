# ANDROID: make FFI-in-Compose-composition a build failure (ANR class killer)

Status: OPEN — filed 2026-09-10T03:22Z by the CTO seat, jointly proposed with
the CEO seat (recurrence-control thread, seq 1139 direction (c)).
Priority: HIGH — this is the durable class-killer behind three consecutive
recurrence events (PlatformBridge ANR 2026-09-09; diagnostics-share crash +
Settings FFI-in-composition 2026-09-10).
Owner lane: android agent (implementation) + CTO seat (gate wiring).

## The defect class (proven three times)

Synchronous `MeshRepository` -> uniffi FFI calls reachable from `@Composable`
functions block the main thread on every recomposition, and UI-entry crash
paths (FileProvider, implicit exceptions) kill the process outright. Each fix
was hand-found; nothing prevents the next one.

## The ask

A build-failing check that rejects the SHAPE, not individual call sites:

1. **Preferred: detekt custom rule or Kotlin compiler/IR check** that flags
   direct `meshRepository.<suspend-blocking|blocking>` invocations inside
   functions annotated `@Composable` (allowlist file for the narrow set of
   state-reading calls that are provably cheap and non-FFI).
2. **Acceptable alternative: ArchUnit-style JVM unit test** in
   `android/app/src/test/` that walks compiled class metadata (or source via
   a lightweight parser) and fails when a `@Composable` function's body
   references the `MeshRepository` facade directly.
3. **Minimum viable (if 1-2 stall):** a CI grep gate (script +
   workflow step) with an explicit allowlist, loud on any new match —
   weakest (misses aliases/wrappers) but ship it if better options are slow.

## Acceptance criteria

- The check fails the build on a reintroduced `getContactCount()`-style call
  in composition (prove with a deliberate regression fixture, then remove it).
- Existing code passes without an allowlist entry larger than ~10 items.
- Documented in the android agent's gate runbook so every future PR runs it.

## Out of scope here

- The one known remaining dead wrapper (`ConversationsViewModel.getMessageCount()`)
  — separate android-lane cleanup, flagged in CTO_STATE.md residues.
- Rust-side `meshService.pause/resume` blocking behavior — separate rule-8
  packet (core), tracked in CTO_STATE.md.
