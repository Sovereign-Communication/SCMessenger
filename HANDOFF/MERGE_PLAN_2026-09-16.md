# 7-Day Merge Plan — 2026-09-16

Date: 2026-09-16
Author lane: workahead rollout audit
Base: origin/main 1e2fb747

## Window

PRs opened 2026-09-09..2026-09-16 (16 total).

Bases:

- `#283`, `#288`, `#289`, `#298`-`#302` -> `main`
- `#290`-`#297` -> `feat/v040-multi-transport-store-forward` (which is `#288`'s head)

## Verdict

No existing PR covers the window.

`#288` (`feat/v040`... -> `main`, 54 files +5390/-585,
MERGEABLE/BLOCKED, 22 checks pass / 0 fail / 11 pending, 0 reviews)
predates the other 15, claims none, has 6 direct file overlaps.
It becomes the carrier only after its children land.

## Phase 0 (anytime)

- `#283` docs V1 readiness audit -> `main`.
  All green, no conflicts (just BEHIND, trivial).

## Phase 1 (into `feat/v040`, in order)

1. `#290` SubnetProbe null-safety (CLEAN 7/7, review only, first).
2. `#291` notif cold-start gate (1 JVM fail — fix
   `NotificationGateTest` + `NotificationHelperGateTest`
   `Boolean?` vs `Boolean`, then review).
3. `#294` release-signing gate (code well-formed incl.
   `HAS_KEYSTORE` guard; needs out-of-band
   `SCMESSENGER_KEY_ALIAS` verification + secrets signoff;
   confirm no debug-APK consumers).
4. `#293` docker localhost/non-root (partial 2/4 — land with
   scoped follow-up for bearer/IPC auth + CORS; review
   `SCM_HTTP_BIND` override + `/data` ownership).
5. `#296` identity-spoof + WASM topic (needs crypto review
   confirming IDs derive from verified envelope + Rule-8 +
   spoof-negative test + Rust CI).
6. `#297` outbox canonical drain + Sled (needs store review +
   Rust CI; disjoint from `#296`, either order).

HOLD:

- `#292` (docs-only, no implementation — do not merge as fix).
- `#295` (docs-only + semantic contradiction:
  buffer-and-replay vs `#291` fail-closed gate on
  `NotificationHelper.kt` — product decision required, then
  implement or close as duplicate of `#288`/`#290`/`#291`).

## Phase 2: refresh `#288`

Merge Phase 1, re-run 11 pendings, approval, then merge to `main`.

Overlap notes:

- `#293` Dockerfile vs `#288` Dockerfile disjoint hunks.
- `#294` release.yml adjacent-hunk near-miss (land `#294` early).
- `#296` swarm.rs hunks (8121/9043) disjoint from `#288`
  (<=5797) but depend on its helpers.
- `#297` cli/main.rs + iron_core.rs disjoint from `#288`.

## Phase 3 (main stack)

- `#289` curve unification after `#288` (needs 2 vector-test
  fixes for y=p-1 and y=1 sign-bit cases; APK/Kotlin-lint
  failures are infra setup-android flake — retry; rustls
  RUSTSEC-2026-0285 likely clears via `#288`'s Cargo.lock; no
  file overlap with rollout stack, semantic only).
- Then rollout drafts `#298` (JoinMesh GMS gate) ->
  `#299` (importSeedAddresses seam) ->
  `#300` (APK host hardening) ->
  `#302` (install-QR emit) ->
  `#301` (iOS link share); each needs its 5 targeted gates
  green + ready + review.
- Follow-ups after `#298` merges: `#299` JoinMesh call-site,
  `#302` accept-side.
- `#301` needs exact v0.4.0 asset URL post-D2.

## Out of window

Dependabot `#211`-`#214`, old `cto/*` drafts: separate batch,
untouched.

## Coordination

Coordination notes posted 2026-09-16 on `#288`, `#294`,
`#296`, `#297`.

## Re-verify 2026-09-16 (later same day) — deltas

Live re-verify later same day found material changes since the plan
was written. This section records deltas only; the Phase 0-3 order
above still stands unless noted.

- `#292` and `#295` are NO LONGER docs-only. `#292` now +79/-33
  incl. `cli/src/main.rs` (implementation landed ~22:47-22:54Z
  window; re-review its code, HOLD verdict needs revisiting).
  `#295` now +163/-55 with code but FAILING Android JVM Unit Tests
  + 4 jobs pending (was CLEAN docs-only). `#295`'s semantic
  contradiction with `#291` (buffer-and-replay vs fail-closed gate)
  still stands and now has failing tests attached.
- `#289`: the 4 earlier failures (APK, JVM vector tests,
  Kotlin-lint infra, cargo-deny rustls) are CLEARED/superseded —
  fresh 30-pending run in flight; still BLOCKED, still needs approval.
- Rollout stack regressions under fix: `#299` FAIL Android Debug APK
  + Lint; `#300` FAIL Android Debug APK + Kotlin Linting; `#302`
  FAIL Android Wiring Gate; `#301` FAIL Kotlin Linting (suspect
  infra flake on Swift-only diff — diagnosis in flight). `#298`
  clean so far (2 pass / 28 pending, 0 fail).
- Zero approvals on ALL 16 PRs (`reviewDecision` `""` everywhere) —
  approvals are now the universal blocker alongside CI; the merge
  train needs reviewers assigned before anything can land.
- Feat head moved `b8f069a6` -> `93408dbb` but docs-only
  (`HANDOFF/CTO_STATE.md` + shadow audit doc) — no code drift, no
  rebase pressure.
- `#303` itself: 12 pass / 6 pending, docs-cheap confirmed, still
  draft + UNMERGED — CTO state on main does NOT yet contain this plan.
