# Experience Library

Lessons from real failures. Read before planning CI, merge, or security work.

## CI

- **Check conclusions `neutral` and `skipped` are non-blocking.** An audit script
  that treated them as failures briefly showed 4 green PRs as red (2026-10-18).
  Always use `GET /repos/{org}/{repo}/commits/{sha}/check-runs`, never the
  combined status API (returns `pending` for everything on Actions).
- **cargo-deny failures are contagious.** A red `cargo deny check` on main
  infects every PR rebased onto it (#503 flipped green→red, 2026-10-18).
  Fix main first; never rebase the queue onto a red main.
- **Verdicts are exact-head-scoped.** A "cleared to merge" on one SHA does not
  carry to a rebased head (#604, 2026-10-18).
- **`sudo apt-get` in CI assumes GitHub-hosted.** The `linux-build-deps`
  action's apt path failed on Ubicloud runners (2026-10-19, PR #608).
  Environment-sensitive steps need runner-aware branching.

## Merge discipline

- **Never push local state over a PR head that doesn't match.** Local copies
  for #489/#521/#550/#554/#556/#562 did not match GitHub heads (2026-10-18);
  pushing would have destroyed lane work.
- **No force-push on shared branches.** #506 was rebased via GitHub's
  head-guarded branch-update API, preserving history (2026-10-19).
- **Batch by relatedness, not count.** Related PRs share failure modes and
  conflict less than arbitrary batches (2026-10-19).

## Security process

- **Rule-8 verdicts are commit-scoped.** A verdict certifying "crypto module
  untouched" was invalidated by two later commits (PR #139). Re-review or
  explicitly record which deltas were reviewed post-hoc.
- **Independence must be auditable.** Lane agent sessions are ephemeral;
  prior involvement by the same lane is invisible unless declared
  (SECURITY_PROTOCOL.md, 2026-10-19).

## Tooling

- **`gh-api` is positional** (`gh-api METHOD /path [json-body]`), not curl-style.
- **`gh-api` needs `PYTHONPATH=/opt/muse/skills/custom-connector-setup/bin`**
  or it fails with ModuleNotFoundError.
- **GitHub log downloads need different auth** than the API token; the
  `/logs` endpoint 401s via the CLI. Use the claims API for step-level detail.
