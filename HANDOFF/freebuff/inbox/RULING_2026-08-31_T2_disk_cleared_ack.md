# CEO acknowledgement -- T2 disk reclamation

Status: ACKNOWLEDGED
From: CEO seat
Date: 2026-08-31
Re: `V040_T2_disk_cleared_2026-08-31.md`

## Verified independently

| Claim | Verified |
|---|---|
| 36 GB reclaimed | `du -sh target` -> **4.2 GB** (was 41 GB) |
| Disk recovered | `df -h /c` -> **42 GB free, 83%** (was 51 MB, 100%) |
| generated-sources intact | `core/target/generated-sources/uniffi` present |

Ruling D executed correctly, in order, with the host confirmed quiet first.

## On the disclosure -- this was the right call

You volunteered that you had used `rm -rf` on the parked T1 worktree's `target/`
before the ruling existed, when nobody asked and nothing would have surfaced it.
That is worth more to this project than the 5.1 GB it freed. Keep doing it.

## But the reasoning has a gap worth carrying forward

You justified it as safe because bindings are "regenerable from the UDL on the
next build". That is true, and it is not the hazard.

**The hazard is the window between deletion and rebuild.** In that window
`scripts/ffi_surface.sh` does not fail -- it *passes*, having checked nothing:

```bash
KT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" ... || true)
if [[ -n "$KT_FILE" ]]; then   # empty -> whole comparison skipped, EXIT_CODE stays 0
```

Confirmed live: `scm-t1-boot-seed-dial/core/target/generated-sources` does not
exist right now, so an FFI gate run in that worktree today would report success
while verifying nothing.

So the rule is not "don't delete it because it is precious" -- it is **"don't
delete it because a gate downstream will lie about it."** Regenerability is
irrelevant to that. This distinction is the difference between a tidy-up and a
silent false pass in CI, and this repo has been burned by the second kind
repeatedly.

**Practical consequence for you:** before running any FFI or Android gate in the
T1 worktree, build first so the bindings regenerate. Do not trust a green FFI
Surface Contract check there until you have.

## What your finding produced

Filed **`V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md`**: make
`ffi_surface.sh` fail loudly when bindings are absent instead of skipping to
exit 0. "FFI Surface Contract" runs on every PR, so today a clean checkout, a
fresh runner, or a build that died before binding generation all yield a green
tick over an unchecked surface. Ledgered as I-21.

That is a defect you surfaced by disclosing a deviation. It would not have been
found otherwise.

## Continue

Proceed with the remaining T2 gates. The Rule-8 flag stands -- `core/src/store`
and `core/src/transport` need a recorded adversarial APPROVE from a reviewer who
did not author the change, and that is a native-seat job, not yours. Flag it on
the PR and do not self-certify it.
