# V040-T7 -- Stage the Android work so device time is verification, never authoring

Status: OPEN (filed 2026-08-31, operator directive)
Priority: P1 -- runs whenever the handset is away, which is most of the time
Lane: Freebuff / DeepSeek V4 Flash
Scope: `android/`, plus new verification scripts under `scripts/device/`. No
changes to `core/` or `cli/`.

## Why

Operator directive 2026-08-31: non-connected nodes are still coded to parity, so
that when the handset appears the session is quick verification and log capture
rather than writing code. Policy: `docs/rules/CONTINUOUS_EXECUTION.md` section 5.

The handset is Tier B -- intermittent, operator-carried. Device time is the
scarcest resource in this project. Every minute spent authoring a test while the
phone is in hand is a minute not spent gathering evidence, and a failed device
session that produces no logs wastes the window completely.

## The parity contract

An Android item is "parity ready" only when **all four** exist before the
handset appears. Three of four is not ready.

1. **Merged code**, green under CI: unit tests, the Android wiring gate, lint.
2. **A verification script** that runs with no authoring on the day -- exact
   `adb` commands, expected output, pass/fail stated in advance.
3. **A log-capture recipe**: the exact `RUST_LOG` value, the logcat filter, the
   capture window in minutes, and the artifact path. This must exist *before*
   the test, because if it fails, the evidence has to already be in hand.
4. **A pre-registered failure disposition**: which ticket it becomes, and
   whether it blocks the tag.

## What to do in this ticket

**Step 1 -- inventory.** List every v0.4.0 and v1.0.0 item that needs the
handset. Sources: `SHIP_PLAN.md` G3 (D4/D6/D7 and the churn gate),
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` Phase 1 (its whole Stage B is Android/LAN),
and any `HANDOFF/todo/` ticket whose acceptance needs a device. Produce one
table: item, what it needs the device for, and which of the four contract
elements already exist.

**Step 2 -- close the gaps that do not need the device.** For each item, write
the missing verification script and log-capture recipe. This is the bulk of the
work and none of it needs the handset.

Put scripts in `scripts/device/`, one per item, named for the item. Each must:

- Check preconditions first and fail fast with a clear message: device visible
  to `adb`, the installed APK's SHA matching the intended build, the CLI node
  running.
- Print `[OK]` / `[FAIL]` per assertion, never just dump logs for a human to
  read. The point is that the operator can run it and read one verdict.
- Write its captured artifacts to a single named directory per run.
- Be idempotent and safe to re-run.

**Step 3 -- respect the Android agent authorization scope.** Per
`docs/rules/ANDROID.md`, Android agents are authorized for **app updates and
passive log collection only**; active device and mesh driving belongs to the
Windows, aidws, and Ubuntu agents. Write the scripts so the active driving side
runs from the Windows node and the handset side is passive. Do not write a
script that requires an Android agent to drive the mesh.

**Step 4 -- record the signing consequence.** The fleet currently runs a
*debug*-signed build, and D4/D6/D7 must be scored on the *released* APK, so
every test device needs an uninstall-and-reinstall before the first scored run
(see `docs/ANDROID_RELEASE_SIGNING.md`). Note this in the inventory against
every item that requires a scored run, because it changes the order of
operations on device day: reinstall first, then test, and identity/history on
those devices does not survive it.

## Acceptance

1. The inventory table exists and covers every device-gated item, with the four
   contract elements marked present or absent for each.
2. Every item marked "parity ready" has all four, and the scripts run to their
   precondition check without a device attached -- that is, they fail with
   `[FAIL] no device` rather than a stack trace.
3. No item is marked ready on the strength of code alone.
4. `cargo test --workspace --no-run` and the Android build still pass.
   Never read `$?` after a pipe.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- No hardcoded UI strings in Android code; they go in `strings.xml`.
- Shared checkout: touch only what this task requires.
- If an item cannot be made parity-ready without the device, say so explicitly
  in the inventory with the reason. `UNVERIFIED` and "needs device to author"
  are both acceptable answers; a silent omission is not.
