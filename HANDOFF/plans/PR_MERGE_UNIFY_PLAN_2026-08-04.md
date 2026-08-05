# PR Merge / Unify Plan -- 2026-08-04

Owner: main orchestrator (Windows host, AGENTS.md rule 5(b) push authority).
Goal: every open PR either merged green or explicitly closed; a working Android
app on the operator's Pixel 6a from post-merge main.

## Order of operations

### Step 1 -- PR #136 (critical path, blocks everything)

`fix(identity): complete canonicalization on public key (steps 2-5)`.
Current CI state 2026-08-04: 4 red jobs (Test ubuntu/windows/macos, macOS
Native Tests); all build/lint/binding/Android/iOS/WASM/CodeQL jobs green.

Red-job root cause is test expectations, not compile: block gate fixed by
db4401d7 + e04b23f9 (both on the PR head), inbox/history keying fixed by
de091e46 (currently only on fix/identity-block-gate-readdeep-2026-08-04), plus
test updates sitting uncommitted in the working tree (integration_e2e.rs,
integration_ironcore_roundtrip.rs, api.udl comment).

Actions:
1. Verify the four suites locally (scoped, one build at a time):
   integration_contact_block, integration_e2e, integration_ironcore_roundtrip,
   cli integration_message_requests.
2. Cherry-pick de091e46 onto fix/identity-canonicalization-steps2-5; commit the
   working-tree test updates on top.
3. Push (triggers CI). Iterate until the 4 red jobs go green.
4. Security gate: adversarial review evidence is on file --
   HANDOFF/review/CORE_BLOCK_GATE_ADVERSARIAL_REVIEW_2026-08-04.md and
   HANDOFF/done/CORE_BLOCK_GATE_HARDENING.md (committed 3e8defda on
   orch/qwen-takeover-setup-2026-08-04; both land on main with the Step-2
   merge). Merge only after CI is fully green AND that evidence is confirmed
   to cover the landed diff.
5. Merge (--no-ff), push main.

### Step 2 -- Orchestration tooling branches

- Local main tooling commits (lake_route qwenpaid ladder fix, dispatch_dial /
  footer parser / batch_handoff / build_lock, ORCHESTRATION.md wiring) were
  pushed to origin/main 2026-08-04 (a5f00be2..a5ab7f39).
- Branch orch/qwen-takeover-setup-2026-08-04 carries the openrouter_direct lane
  wiring + block-gate tickets + audit docs. Rebase/merge to main after Step 1
  (no conflict expected with core/ code).
- Commit of AGENTS.md rule-5(b) + .qwen/commands/orchestrate.md + this plan +
  the Sonnet lockout doc: merge with Step 2 batch.

### Step 3 -- Dependabot batch (12 open PRs)

PRs: #108 #107 #106 #104 #103 #102 #100 #99 #69 #67 #65 #64.
GitHub reports 7 open vulnerabilities on the default branch (3 high) -- these
bumps are the remediation path; do not let them rot.

Plan (one CI cycle instead of twelve):
1. From post-Step-2 main, cut `integration/unify-dependabot-2026-08-04`.
2. Merge each dependabot branch into it; resolve conflicts (mostly
   gradle/libs.versions.toml and Cargo.toml/Cargo.lock).
3. Run the full gate via CI on the integration branch (push) and locally scoped
   where faster.
4. Merge the integration branch to main; GitHub auto-closes the PRs as merged.
   Close any PR whose bump is superseded by a newer version instead.

### Step 4 -- Loose-branch triage

- `fix/parity-critical-core`: local-only, ahead 1 / behind 23 of main, contains
  mutex/tracing/GATT/identity-guard fixes. Rebase onto post-Step-3 main; if the
  identity work overlaps with PR #136's canonicalization, reconcile by diff
  review before merging; otherwise land it as its own PR.
- Stale local branches (android-gate-0.4.0-verify, audit_system, verify/*,
  wip/*): no action until v0.4.0 close-out; do not delete without operator OK.

### Step 5 -- Android deliverable (operator waiting)

After Step 1 merges, main CI rebuilds the Android Debug APK artifact from the
merged tree. Download the artifact (`gh run download`) and install on the
Pixel 6a via adb (device presence: adb devices; install:
`adb install -r <apk>`). Smoke test: launch, identity creation, add contact,
send a message. Report result + APK provenance (commit SHA) to the operator.

## Rules that bind this plan

- CI on push is the full gate; scoped local runs only to avoid burning 43-min
  CI cycles on known-broken pushes. One build tool at a time on this host.
- No merge without green CI + applicable security-gate evidence (AGENTS.md
  rule 8, .claude/rules/security.md).
- No force-push to main. Dependabot PRs are never edited by hand.
