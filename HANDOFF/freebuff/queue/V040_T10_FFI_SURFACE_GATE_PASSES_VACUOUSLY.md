# V040-T10 -- The FFI surface gate passes when it checks nothing

Status: OPEN (filed 2026-08-31)
Priority: P1 -- a CI gate that cannot fail when it matters most
Lane: Freebuff / DeepSeek V4 Flash
Scope: `scripts/ffi_surface.sh`. Small change, high leverage.

## The defect

`scripts/ffi_surface.sh` locates the generated UniFFI bindings, then compares
their symbol surface against a committed snapshot:

```bash
KT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" -name "api.kt" -o -name "scmessenger_core.kt" 2>/dev/null | head -1 || true)
SWIFT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" -name "SCMessengerCore.swift" 2>/dev/null | head -1 || true)
...
if [[ -n "$KT_FILE" ]]; then
    # ...the entire comparison...
fi
```

If `core/target/generated-sources/` is absent, `find` prints nothing, `KT_FILE`
and `SWIFT_FILE` are empty, **both comparison blocks are skipped**, `EXIT_CODE`
stays `0`, and the script exits successfully having verified nothing.

`scripts/clean_target.sh`'s own header documents this hazard --
"a VACUOUS 'Updated Swift snapshot' with exit 0 and no bindings -- a silent,
passing lie" -- and `clean_target.sh` guards against it by backing the tree up.
**`ffi_surface.sh` itself does not.**

This matters because "FFI Surface Contract" is a CI check on every PR. A clean
checkout, a fresh runner, a cleaned target, or a build that failed before
binding generation all produce the same result: the gate reports success while
the FFI surface is entirely unchecked. The one circumstance where you most need
this gate -- bindings missing or not regenerated -- is the exact circumstance
where it cannot fail.

## How it was found

The Freebuff lane cleared disk space by removing a parked worktree's `target/`
before the `clean_target.sh` ruling existed. Its reasoning was that bindings are
regenerable from the UDL, which is true. But regenerability is not the hazard:
the hazard is the window between deletion and rebuild, in which this gate
reports a pass. Confirmed live -- `scm-t1-boot-seed-dial/core/target/generated-sources`
does not currently exist, so an FFI gate run in that worktree today would pass
vacuously.

## Required change

Make absent bindings a **failure**, not a skip:

- If `core/target/generated-sources/` does not exist, or neither binding file is
  found, print a clear `[FAIL]` naming what is missing and how to regenerate it,
  and exit non-zero.
- Apply this to both the Kotlin and Swift paths independently -- one present and
  one absent must also fail, not half-pass.
- `--update` mode must refuse to write a snapshot from missing bindings. Writing
  an empty or partial snapshot would silently *lower* the contract for every
  future run, which is worse than the current bug.
- Keep the existing "no snapshot found" warning distinct from "no bindings
  found". They are different conditions with different fixes.

Do not paper over it by making CI generate bindings first -- the point is that
the gate must be honest about what it did or did not check.

## Acceptance

1. With `core/target/generated-sources/` present and matching: passes, as today.
2. With the directory renamed away: **exits non-zero** with a message naming the
   missing bindings. Verify by moving it aside and restoring it -- do not delete
   it.
3. With only the Kotlin binding present: fails on the Swift side rather than
   reporting overall success.
4. `--update` with bindings absent refuses and exits non-zero without writing.
5. Existing snapshots are unchanged by this work -- `git diff` shows no
   modification under the snapshot directory.
6. Never read `$?` after a pipe.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Do not delete `core/target/generated-sources/` while testing -- move it aside
  and restore it. It is expensive to regenerate and other lanes share this host.
- CI-only work, no handset required -- valid never-idle work under
  `docs/rules/CONTINUOUS_EXECUTION.md`.
- Shared checkout: touch only what this task requires.
