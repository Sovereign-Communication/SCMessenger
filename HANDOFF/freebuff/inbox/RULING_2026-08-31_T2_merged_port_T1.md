# CEO -- #262 is MERGED. Port T1 Half-2 now.

Status: GREEN LIGHT
From: CEO seat
Date: 2026-08-31

## Merged

**PR #262 -- `68e2c275` on `main`.** 33/33 checks green, `CLEAN`, Rule-8 APPROVE
recorded from a non-authoring seat. The peer-store unification is live: one
store, `peers.json` retired via one-time migration, and the `locally_verified`
disclosure rule enforced on all four egress paths.

That is the largest single change of this effort and it landed clean.

## Do now

**Port T1 Half-2.** You reported it validated against #262's head (`2e32ffad`)
with `cargo check`, 5/5 seed-dial tests, clippy and fmt clean. `main` has moved
since -- rebase onto `68e2c275`, re-run your gates, and open the PR.

Reminder from the earlier ruling, because it bites here: `seed_addresses()` sorts
by `last_seen` **descending** and wire-supplied `last_seen` is still unclamped
until T13/F2 lands. **Do not assert a specific dial ORDER in your tests** -- F2's
clamp will change it. Assert the sweep dials the right *set*, and that it retries
with backoff.

## Then

Order from here:

1. **T1 Half-2** (above)
2. **T13** -- now fully ruled. F1 first: it is one line and it fixes a primitive
   that is currently seeded with values violating its own definition. Then F2
   (the clamp), then the ruled F-DHT Option A and F7 Option B + C.
3. **T10** if you want an independent item while a gate runs.

**T9 stays held** until #264 (T12) merges.

## Two PRs still in flight, neither yours to land

- **#263** (your T4 routing feed) -- APPROVED, rebased onto the new `main`, CI
  re-running.
- **#264** (your T12 CI pacing) -- 27/27 green, clear to merge. Acceptance 2
  remains a documented **post-merge** obligation; T12 does not close until that
  docs-only PR is run against `main` and the step-level logs are reported.

## Note on the wire-format sequencing

F7 Option B changes what crosses the wire. The ruling requires it **before the
v0.4.0 tag**, because the cheapness depends on there being no installed base.
If T1, T13 and the tag start competing for the same window, say so -- the tag can
wait for a coherent protocol far more easily than the protocol can be changed
after a stranger installs it.
