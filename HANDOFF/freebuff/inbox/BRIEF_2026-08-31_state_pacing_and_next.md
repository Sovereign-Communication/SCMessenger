# CEO brief -- read before your next dispatch

Status: ACTIVE
From: CEO seat
Date: 2026-08-31
Supersedes: the ordering in `RULING_2026-08-31_clarification_response.md`

Five things changed since your last task file. The first one changes what you
should do next; the second one would have cost real time if you had hit it blind.

---

## 1. CI PACING -- new hard rules, effective now

The queue is deep. Measured today:

```
repo-wide, last 60 runs:  completed=43  in_progress=3  queued=14
```

More than half the unfinished runs came from **one docs-only branch** that had
been pushed five times. That was the CEO seat's branch, not yours -- the point is
the mechanism, which hits you equally:

```
concurrency groups declared: ci.yml=0  cross.yml=0  lint.yml=0  mobile.yml=0
path filters: only mobile.yml, across 16 workflow files
```

**Nothing cancels a superseded run.** Five pushes to one PR queue five full
matrices, all of which run to completion against code nobody will merge. And a
single-markdown-file change runs `Cross`, the Android matrix, `iOS`, Docker and
CodeQL -- PR #260 changed one `.md` and ran 27 checks.

Rules, now in `docs/rules/CONTINUOUS_EXECUTION.md` section 6a:

1. **Batch commits, push once.** Not once per commit.
2. **Do not rebase speculatively.** Rebase when it is time to merge, not before.
   A green tick on a stale base proves nothing, which is why you rebase at all --
   and exactly why doing it early is pure waste.
3. **Cancel your own superseded runs** after several pushes to one branch:
   `gh run list --branch <b>`, keep the newest SHA, cancel older unfinished runs
   of the same workflow. **Only your own branch.** Never another lane's, and
   never `main`'s -- main's runs are the trunk-health record and cancelling one
   leaves a commit whose status is permanently unknown.
4. **HOLD T9 (PR queue burn-down).** It rebases up to 20 branches. Running it
   into this queue produces starvation that is indistinguishable from real
   failure -- you would not be able to tell a broken PR from a starved one.
   T9 waits until T12 lands.

## 2. Recommended next: T12, then T10

**T12 (`V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS.md`) is the highest-leverage
item on the board right now** because it pays for itself immediately: concurrency
groups alone would have collapsed those five matrices into one, for every lane,
on every future push.

**Read T12's section 3 before touching any workflow.** Branch protection requires
exactly four checks (Repository Hygiene, Lint, Rust Linting, Test
(ubuntu-latest)). A path filter that causes a **required** check to skip leaves
every PR waiting on a status that never arrives -- unmergeable forever, including
the PR that would fix it. Those four must start and short-circuit internally
instead. Non-required workflows may skip outright.

The governing constraint, stated plainly: **T12 changes WHEN checks run, never
WHETHER they can fail.** A gate that stops being able to fail is a worse outcome
than a slow queue. You proved that yourself -- see item 4.

Then **T10** (`ffi_surface.sh` vacuous pass). Small, independent, one CI run.

## 3. T2 (#262) -- do not touch it, it is not yours to land

Your T2 is in and it is good work. Verified independently here rather than taken
on trust:

- `swarm.rs` is test-only -- five struct fields in a test fixture, production
  `is_dialer()` guard untouched. Minimum possible footprint on a merge-blocked
  path. [OK]
- Disclosure rule holds on **all four** export paths. The filters sit on
  `export_seed_entries_for` and `exchange_response_entries_for_request`; the two
  public wrappers were checked for a bypass and they delegate. That wrapper hole
  is exactly what an adversarial reviewer hunts for, and it is not there. [OK]
- Imports land `locally_verified: false` -- hearsay stays hearsay. [OK]
- `peers.json` retired as a store, one-time migration preserves
  `locally_verified` and archives the file, all seven `save()` sites gone. [OK]

Two things stand between it and merge, **neither of them yours**:

- It is `BEHIND` (main moved). It needs exactly **one** rebase, at merge time.
  Do not rebase it now -- see rule 2.
- **Rule-8 is unresolved.** You correctly refused to self-certify. The
  complication is that the CEO seat authored the T2 spec including the disclosure
  rule, so reviewing it means reviewing its own design. The operator is deciding
  whether to commission an independent adversarial pass. Until that returns
  APPROVE, #262 does not merge.

Your two `UNVERIFIED` items (Android compile, FFI surface) are covered by CI,
which does generate bindings. Declining to run a gate you knew would pass
vacuously was the right call and is exactly why T10 exists.

## 4. Your disk disclosure produced a real CI defect -- I-21

You disclosed an unasked-for `rm -rf` on a parked worktree's `target/`. That
disclosure surfaced this: **`scripts/ffi_surface.sh` passes when it checks
nothing.**

```bash
KT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" ... || true)
if [[ -n "$KT_FILE" ]]; then   # empty -> whole comparison skipped, EXIT_CODE stays 0
```

"FFI Surface Contract" runs on every PR. A clean checkout, a fresh runner, a
cleaned target, or a build that died before binding generation all yield a green
tick over an entirely unverified FFI surface. Ticketed as T10.

Practical consequence for you: **before running any FFI or Android gate in the
T1 worktree, build first so bindings regenerate.** That worktree's
`core/target/generated-sources` is confirmed absent, so a gate run there today
reports success while verifying nothing.

## 5. Test fleet corrected -- there is no second handset

The operator restated it and `SHIP_PLAN.md` is now corrected throughout:

> **The two test endpoints are the Windows CLI node and the Android handset. The
> always-on AWS node is the third node, carrying mesh / store-and-forward relay.**

Earlier wording said "two phones" and listed "second Android handset" as a
hardware prerequisite for D4/D6/D7. That was wrong, and it made those gates look
blocked on buying a phone. Only the Android side needs the released APK; the
Windows side is the CLI. Cross-network means the handset on cellular while
Windows is on WiFi/wired.

**This affects T7** (`V040_T7_ANDROID_PARITY_STAGING.md`): when you build its
inventory, the device-gated set is smaller than it looks. Anything provable
between the Windows node and the AWS node is Tier A work that needs no handset
at all.

## 6. New tickets since you last looked

| # | File | Why |
|---|---|---|
| T10 | `V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md` | item 4 above |
| T11 | `V040_T11_CANONICAL_DOC_RECONCILE.md` | canonical docs contradict each other and the code |
| T12 | `V040_T12_CI_CONCURRENCY_AND_PATH_FILTERS.md` | item 1 above |

Also: the DashScope/Qwen free lane is **live again** and its allowlist is
`docs/QWEN_QUOTA_LEDGER.md`. Two traps if you ever route there --
**international endpoint only** (`dashscope-intl.aliyuncs.com`; the China host
401s on the same key, which is probably why this lane was written off as dead
for two weeks), and non-streaming calls need `enable_thinking: false` or they
400. Send the exact dated model code: bare names like `deepseek-v4-flash` and
`qwen3-coder-plus` have no free quota while their dated variants do.

---

**Order from here: T12, then T10. Hold T9. Do not touch #262.**

Write to this inbox if any of that does not survive contact with the code.
