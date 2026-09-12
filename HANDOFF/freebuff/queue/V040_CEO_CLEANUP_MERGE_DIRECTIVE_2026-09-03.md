# V040 CEO directive: candidate commit, safe cleanup, PR closure, merge order

Status: AUTHORITATIVE. Replaces the blocked state from
`HANDOFF/freebuff/inbox/V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02_blocked.md`.
Supersedes nothing else; keep `V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md`
as the three-node validation contract.

Read first, in order: `AGENTS.md`, `CLAUDE.md`, `HANDOFF/freebuff/README.md`,
`HANDOFF/freebuff/inbox/README.md` (return format), this directive.

## 1. CEO decisions answering the blocked note

1. **Candidate is authorized.** The uncommitted architecture-pass files in the
   shared checkout belong to this task and MAY be isolated/copied into the
   candidate worktree. The candidate worktree is `scm-v040-candidate`
   (`C:/Users/SCM/Documents/GitHub/scm-v040-candidate`), branch
   `cto/v040-candidate-2026-09-02`, checked out at `origin/main`
   (`67d19d3c40fb346c4286ae47ce2e0be8cb7be5ab`). Do NOT create a new worktree;
   use this one.
2. **Commit the reconciled set there**, push the branch, open the PR. The new
   commit SHA is the three-node candidate. Do NOT commit in the main shared
   checkout (`C:/Users/SCM/Documents/GitHub/SCMessenger`) under any
   circumstances.
3. **Everything else in the shared checkout is OFF LIMITS.** Never stage,
   commit, revert, stash, or clean any file you were not explicitly assigned in
   this directive. The main checkout's uncommitted work is another session's
   live state.
4. **Squash-merge gotcha is in force.** This project merges PRs by squash. A
   branch whose tip is NOT an ancestor of `origin/main` may still be fully
   merged (its content lives in main under a new SHA). Never conclude "merged"
   or "unmerged" from ancestry alone. The only accepted proof is
   `git diff --quiet origin/main...<branch>` (empty output = content merged).
   Report the command output for every branch you classify.

## 2. Candidate reconciliation and commit

The six product files exist in TWO working copies: the main checkout and
`scm-v040-candidate`. Verified facts (2026-09-02 audit):

- Byte-identical in both: `cli/Cargo.toml`, `core/src/iron_core.rs`,
  `core/src/routing/local.rs`, `core/src/routing/optimized_engine.rs`.
- Differ between the two: `core/src/transport/observation.rs`,
  `core/src/transport/swarm.rs`. The main-checkout `swarm.rs` contains the
  `NewListenAddr -> address_observer.set_listen_ports(...)` wiring
  (`core/src/transport/swarm.rs:5271-5274`). Verify the candidate's copies
  contain it too; if not, take the main-checkout version.
- For each differing file: `git diff --no-index` the two working copies, take
  the superset that COMPILES and passes the focused suites (section 4). Record
  the per-file decision and the byte hashes in the completion note.
- Include `docs/ARCHITECTURE_SCOPE_V040.md` (untracked in the main checkout;
  copy it into the candidate) as part of the commit.
- Commit scope: EXACTLY these 7 paths, nothing else.

Suggested commit subject: `feat(core,cli): V040 architecture pass -- single-owner address admission, unified local ordering, CLI default-run`.

## 3. Cleanup protocol (order is mandatory)

Export FIRST, then delete. Every export goes to repo-local `tmp/recovery/`
(gitignored). Record `sha256sum` of every export in the completion note.

### Step 3.1 -- export Group C (dirty worktrees). NEVER remove before export.

| Worktree | Local state to export | Risk |
|---|---|---|
| `scm-mailbox` | `git diff --binary` (193 files; mostly CRLF churn, includes 1 binary `docs/NARC_MASTER_TRACKER.md`) | Binary delta may be real |
| `scm-secutils` | `git diff --binary` (16 files, 4757+/4757-, looks CRLF-only) | Verify with `git diff --ignore-space-at-eol --quiet`; if clean, discard after export |
| `scm-t1-boot-seed-dial` | `cli/src/lib.rs`, `cli/src/main.rs` diffs + untracked `cli/src/seed_dial.rs` (hash `1b91722a...` -- UNIQUE, not committed anywhere) | HIGHEST VALUE: unique local variant |
| `scm-t1-half2-validation` | `cli/src/lib.rs`, `cli/src/main.rs` diffs + untracked `cli/src/seed_dial.rs` (hash `aa67df45...` == committed version) | Keep for completeness |
| `scm-t10-ffi-gate` | `scripts/ffi_surface.sh` diff (+27/-6) | May be follow-up to merged PR 261 |
| `scm-t13-fdht` | `cli/src/ledger.rs`, `core/src/store/ledger_entry.rs`, `core/src/transport/swarm.rs` diffs (197+/81-) | REAL uncommitted production code |

Export form: `git -C <worktree> diff --binary > tmp/recovery/<name>-<tip12>.diff`
plus `git -C <worktree> ls-files --others --exclude-standard | xargs -I{} cp <worktree>/{} tmp/recovery/<name>-{}/`
for untracked files. Verify each export is non-empty and hashed BEFORE any removal.

### Step 3.2 -- reconcile and commit the candidate (section 2) BEFORE any worktree removal.

### Step 3.3 -- free disk with the ONLY sanctioned target tool.

- `scripts/clean_target.sh --all` in the main checkout (standing ruling,
  `SHIP_PLAN.md` I-20, 2026-08-31). NEVER `rm -rf target/`, NEVER bare
  `cargo clean`. This preserves built binaries and backs up/verifies
  `core/target/generated-sources/`.
- Run this BEFORE the workspace gate and 3-node builds; `cargo check
  -p scmessenger-cli --lib` is currently failing with `os error 112` (disk
  full) and `cargo test -p scmessenger-cli --lib` with `LNK1318` (PDB, linked
  to the same space pressure). Disk headroom is a hard prerequisite.

### Step 3.4 -- remove Group A worktrees (clean, tip verified live on origin).

Use `git worktree remove <path>` (NOT `rm -rf`). Branch refs survive removal;
only the working dirs are deleted. Verified live refs (2026-09-02):

| Worktree | Tip | Live origin ref | Est. size |
|---|---|---|---|
| `scm-t1-half2` | `5187b7abf8f9` | `origin/freebuff/v040-t1-half2` | 506 MB |
| `scm-t12-ci-pacing` | `ec177fd98b8f` | `origin/freebuff/v040-t12-ci-pacing` | 45 MB |
| `scm-t13-f7` | `7bafe83dda24` | `origin/freebuff/v040-t13-f7-hint-widen` | 7.8 GB |
| `scm-t13-fdht-main` | `80197ef5ed4c` | `origin/freebuff/v040-t13-fdht-gate` | 8.2 GB |
| `scm-t14-ephemeral-port` | `6fd0230b31bc` | `origin/freebuff/v040-t14-ephemeral-port` | 8.1 GB |
| `scm-t14-preexisting-fixes` | `b2a7b345d860` | `origin/freebuff/v040-t14-preexisting-fixes` | 8.1 GB |
| `scm-t4-routing-feed` | `bc5bff0fdeca` | `origin/freebuff/v040-t4-routing-feed` | 509 MB |
| `scm-t5-docs-sync` | `464ab0642639` | `origin/freebuff/v040-t5-docs-sync` | 45 MB |
| `scm-t8-restore-test` | `8fc5881730b2` | `origin/freebuff/v040-t8-restore-test` | 162 MB |

Expected reclaim: ~33 GB. Before each removal, confirm the branch content is
merged or PR-tracked per section 5 (content-diff proof, not ancestry).

### Step 3.5 -- Group B and prunable records.

- `scm-t2-unify-ledgers`: clean; local tip `2e32ffad8f60`, origin-tracking
  `81cca9a8f506` (exposed via PR 262 head), NO live branch ref. `git worktree
  remove` is safe (the local branch ref stays in the shared repo), but keep the
  branch and do not close PR 262 until the T2 merge decision (section 5).
- Prunable entries (dirs already gone): first create backup refs for the two
  ORPHAN commits with no local or live remote ref:
  - `git update-ref refs/backup/cto-l7-audit-status 2fc9cf6640d7a026d4efcb6b696b865838ec94b5`
  - `git update-ref refs/backup/cto-l8-kernel-lane-policy d9403708a2e3ae2cab359b88b916575b5c4d85c2`
  - `e01c-pq-mixing`: stale `locked` marker for dead PID 19192, directory
    missing. Remove the marker file
    (`.git/worktrees/e01c-pq-mixing/locked`) then let `git worktree prune`
    clear the entry.
  - All other prunable tips already have local or live refs; safe to prune.
- NEVER `git worktree prune` before the two backup refs and the lock removal
  are done.

### Step 3.6 -- keep `scm-v040-candidate` until the candidate PR is merged.

## 4. Gate commands on the candidate SHA

Run all in the candidate worktree, in order, and paste raw output:

1. `cargo fmt --check`
2. `cargo test -p scmessenger-core --lib observation`
3. `cargo test -p scmessenger-core --lib local`
4. `cargo test -p scmessenger-core --lib optimized_engine`
5. `cargo test -p scmessenger-cli --lib`
6. `cargo check -p scmessenger-core --all-targets`
7. `cargo check -p scmessenger-cli --lib`
8. `git diff --check`

Mark each PASS, FAIL, or UNVERIFIED with the command and output. Do not claim a
gate passed without its output.

## 5. PR closure and merge order

Produce a disposition table for every open PR touching these branches with the
proof for each row: `git diff --quiet origin/main...<branch>` output and the
PR head SHA. Expected dispositions (verify, do not assume):

- **Content already merged (close the PR):** PR 266 (T1 half2), PR 258 (T1
  boot seed dial), PR 261 (T2 disk ruling), T10 FFI gate, T5 docs sync, T8
  restore test -- confirm each with the content-diff proof.
- **Close without merge:** `t1-half2-validation` (superseded by PR 266) and
  any branch whose content-diff against main is empty.
- **Merge after their review evidence is on file:** T13 FDHT gate (QWEN
  verdicts in `HANDOFF/freebuff/inbox/`), T13 F7 hint widen, T14
  ephemeral-port, T14 preexisting-fixes, T12 CI pacing, T4 routing-feed, T2
  unify peer ledgers. Each touches `core/src/{transport,routing}` -- Rule-8
  requires a NON-AUTHOR adversarial APPROVE before merge; attach the verdict
  file name to the row.
- **New PR this session:** the architecture candidate (section 2). Its
  transport/routing surface also requires a non-author adversarial APPROVE
  before merge.

Merge authority: the Windows orchestrator (CEO) seat only. You prepare each
PR with evidence and a one-line merge recommendation; the CEO approves each
merge individually. Propose the exact merge order in your completion note
(dependencies first: T2 ledgers before branches that build on ledger state;
T14/T13/T4 before or after each other per your content-diff conflicts). Do not
merge, tag, or release anything yourself.

## 6. Handoff contract

- Return path: `HANDOFF/freebuff/inbox/` per `inbox/README.md` format
  (`Task:` / `Type:` header, exact commands and output, SHAs, gate marks).
- One file per workstream: candidate commit, cleanup record, PR disposition
  table, three-node validation (per the 09-02 handoff).
- Every claim carries the command that produced it, a run URL, or the word
  `UNVERIFIED`.
- End each note with the single next decision needed from the CEO seat.
- Hourly tracking continues per `V040_CTO_TRACKING_PROTOCOL.md`; this
  directive clears the blocked state and is the current work order.