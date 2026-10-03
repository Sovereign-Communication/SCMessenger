# Session report: receipt authorization landed

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Owning repository: `Sovereign-Communication/SCMessenger`

Date: 2026-10-02
Branch of record: `freebuff/receipt-auth-contract`
Operator directive: own everything, land merged safely, GitHub Actions only, reclaim disk after commit.

## Landed

| SHA | PR | What |
| --- | --- | --- |
| `1e0453b1` | #427 | `#[allow(clippy::double_must_use)]` on two async_trait bridges (pre-existing repo-wide lint failure) |
| `f235d49d` | #429 | Receipt authorization: bind every message to its authenticated recipient |

Both merged into `main`. Verified in the merged tree, not just in the branch:
all four contract tests present in `main:core/src/store/outbox.rs`, and the
phase-table assertions present in `main:core/tests/integration_ironcore_roundtrip.rs`.

## The blocker, and why it was not mine

PR #429 was BLOCKED on the required `Lint` check with 13 `clippy::double_must_use`
errors in `core/src/transport/wifi_aware.rs` and `wifi_direct.rs` -- files the
receipt change never touches (`git diff origin/main --name-only` listed only the
three receipt files).

Cause: the lint arrived with clippy 1.99.0 and no source commit. The toolchain is
unpinned (`dtolnay/rust-toolchain@stable`, `rust-toolchain.toml` channel = stable).
Main's last green Lint run predated the lint's introduction.

## Reviews (harness-oc MCP panels)

### PR #427 -- transport-gated, CRITICAL_VALIDATOR required
Three independent lanes (`gpt-4o-mini`, `gemini-2.5-flash`, `llama-3.3-70b`):
unanimous `APPROVE_WITH_CONDITIONS`, zero BLOCKs, no security finding.

One lane claimed `Result`'s `must_use` is a clippy lint that the allow would
weaken. That is wrong, and it was the decisive question, so it was checked
directly: `rustc -W help` lists `unused-must-use` as a **rustc** lint, and
`double_must_use` does not exist in rustc at all. A `clippy::` allow cannot reach
a rustc lint, and CI runs clippy with `-D warnings`. The dropped-`Result` net is
fully intact. `anthropic/*` is on the BYOK denylist; frontier tiers exceeded the
harness per-task ceiling.

### PR #429 -- delivery-sensitive (manifest keywords: outbox, receipt, custody, retry)
Lane `gemini-2.5-flash` judged by `gpt-4o-mini`: `APPROVE_WITH_CONDITIONS`,
high agreement, 0.85, no BLOCK. It judged replay, wrong-recipient, and
consume-vs-TTL cases mitigated.

It raised one **CRITICAL**: the legacy fallback in `remove_for_recipient_key`
(used when no `receipt_auth_*` record exists) might let an unauthorized peer
clear retry state, leaving the original bug alive for legacy rows.

**That finding is invalid.** The fallback is authenticated, not unconditional:
`authorized` requires `matching_queue.is_some()`, and a queue row only matches
when its stored `recipient_id` hex-decodes to exactly the caller's key. That key
is `sender_pubkey`, taken from the wire envelope that successfully decrypted the
payload (`core/src/iron_core.rs:3719-3724`); decryption failure returns early, so
the receipt branch is unreachable for an unauthenticated peer. A stranger cannot
present the intended recipient's key.

The prior test only covered the ACK-migration path, which writes an auth record
before the fallback is reachable -- so the fallback itself was untested. Added
`legacy_row_without_authorization_still_rejects_the_wrong_recipient`: a raw legacy
row with no auth sibling, asserting a stranger is refused on both the read and the
clearing path, the row survives the refusal, and the real recipient is still
served. The claim got an executable answer rather than a code-reading argument.

Evidence: `tmp/orchestration/state/PR427-double-must-use-CRITICAL_VALIDATOR.json`,
`tmp/orchestration/state/PR429-receipt-auth-DELIVERY_VALIDATOR.json`.

## CI

Pre-merge, on the exact merged content, `37057117488`: **all 9 jobs green** --
Lint, Rust Linting, Test (ubuntu/macos/windows), Docs, FFI Surface Contract,
Handoff ownership scope, Windows CLI Artifact, Orchestration control plane.

Required checks on main are exactly: Repository Hygiene Checks, Lint, Rust Linting,
Test (ubuntu-latest), Handoff ownership scope. All five passed; zero checks failed
anywhere on the PR. Four non-required Apple jobs (iOS, iOS Build, iOS Build &
Simulator Test, macOS Native Tests) sat `pending` on macOS runner queue capacity,
not on anything in this change. No local build was used at any point.

## Reclaim: nothing was safe to delete

`python scripts/reclaim_safe.py --reclaim` -> `[DONE] Reclaimed 0 target
directory(ies), 0.00 B total freed.`

The four SAFE worktrees have no `target/` at all. The 6.95 GB in the main checkout
is correctly **HOLD**: 93 dirty files from other sessions plus 1 unpushed commit, so
it fails the clean/fully-pushed/merged gates. It was not touched.

Disk: 8.90 GB free of 236.25 GB (96.2% used), verdict TIGHT -- measured by
`python scripts/disk_budget.py`. This is *below* the 10.11 GB at session start, so
the disk got worse, not better; CI, not local builds, produced that. It is close to
the 8 GB BLOCKED floor. Reclaiming the main checkout's `target/` needs a decision
about that tree's 93 uncommitted files, which is not mine to make.

## Shared checkout reconciled

`core/src/store/outbox.rs` and `core/tests/integration_ironcore_roundtrip.rs`
were restored **forward** from `origin/main` (`git checkout origin/main -- <paths>`)
and are now byte-identical to the merged result. Their previous contents were an
older pre-merge form of the same change (raw `recipient_id` instead of the
canonical queue key), and every receipt symbol exists in main at equal or greater
count -- nothing was lost.

`core/src/iron_core.rs` was deliberately left alone. It is `MM`: another session's
WP1 identity work is staged there (12 staged WP1 hunks, `WP1.1 FAIL CLOSED` guard
present, both intact), and only 3 lines of my now-merged receipt work remain
unstaged on a base 186 commits behind main. Removing them would mean editing a file
holding someone else's staged work, so it was left for its owner.

## Left undone, and why

- Main's CI run `37061994387` at `f235d49d` was still `queued` behind the macOS
  runner backlog when this session ended. Confirmatory only: the identical content
  already passed all 9 jobs pre-merge.
- The 6.95 GB `target/` in the main checkout needs an owner decision about that
  tree's uncommitted work.
- Optional follow-up from the review: a TODO to revisit the `#[allow]` when an
  `async_trait` upgrade resolves the double-emission, and pinning the CI toolchain
  so the lint gate stops drifting (that drift is what caused this blocker).