# Orchestrator Takeover — 2026-08-28 (Buffy as orchestrator, deepseek model)

Status: ACTIVE
Author: Buffy (Freebuff orchestrator), taking over from the OxAlpha-folder
OpenCode/Claude session that stalled on provider rate limits.
Branches from: `fix/android-receipt-envelope` @ `6be72c82` (PR #234, 52 commits
ahead of main `589479c3`).
Supersedes: nothing; records the handoff state so any fresh session can resume.

> Prime directive (from UNIFICATION_V2_RESULTS_PLAN.md): plan for results
> (invariants the user experiences), not how to code them.

---

## 1. What the stalled session did (verified from git log + session logs)

The 48-hour burst (2026-08-26 to 2026-08-28) landed 52 commits on
`fix/android-receipt-envelope`, all authored by `Claude (Cowork sandbox)`:

- **Unification V2 (identity canonicalization):** self-certifying key gate on
  Rust ledger writes + load-time poison repair (`2360a3b6`, `9b5b0f2d`);
  canonical public_key_hex dedup merging 12D3+30d0fa duplicates (`afcce5f0`);
  contact libp2p->hex migration (`749779b8`); P0 live canonicalize + Kotlin
  Ed25519 point validation (`580d69fb`); online-authority gate + self-node
  exclusion, phantom coalescing (`fb2bb3f6`).
- **Android mesh UI:** LazyColumn crash fixes (Column+verticalScroll, stable
  keys, debounce, mutex), accurate nearby count (2 not 9), nickname authority
  restoration, mesh click auto-add contact.
- **Delivery convergence (V3, R1+R2):** `0c75bf1a` converges the core outbox on
  true swarm delivery ACK (`mark_message_sent` on transport ACK, not BLE
  fire-and-forget); `6be72c82` records verdict 5.
- **Docs:** verdicts 4 and 5 recorded in `UNIFICATION_V2_RESULTS_PLAN.md`.

The last session activity (log: run `48f5faa1`, session `ses_fc0712b4`) was a
post-commit verification pass: checking Android pending-outbox JSON (the 55
stuck entries), `UNIFICATION saving contact nickname` events, and spawning a
subagent to verify the ratchet session-recovery path (`Failed to decrypt
ratchet message` / `invalid ciphertext, wrong key`, commit `306e3149` /
`838f9ecd`). It hit `Error from provider (Console): Rate limit exceeded` at
11:57-11:58 UTC and the run was disposed at 12:01 UTC; resumed under a new run
with a different model, then stalled again. **No new commits after `6be72c82`.**

## 2. CI state at takeover (verified via `gh pr view 234 --json statusCheckRollup`)

5 red lanes, 3 root causes:

| Lane | Status | Root cause |
|---|---|---|
| CI / Lint (cargo fmt) | FAIL | fmt diffs in `cli/src/main.rs:4066`, `core/src/store/contacts.rs` x13, `core/src/transport/behaviour.rs:317` |
| Lint / Rust Linting | FAIL | same fmt diffs |
| Repository Hygiene | FAIL | trailing whitespace (CRLF) |
| CI / Test (ubuntu-latest) | FAIL | `message_request_lifecycle_accept`, exit 101 |
| CI / Test (macos-latest) | FAIL | same test |
| CI / Test (windows-latest) | was running | same test expected |

**Test failure root cause (git-blame verified, `580d69fb`):**
`ContactsManager::add()` (`core/src/store/contacts.rs:517`) canonicalizes
`contact.peer_id` to `public_key_hex` whenever the public_key is a valid
64-hex. `AcceptMessageRequest` (`cli/src/server.rs:1505-1506`) still creates
`Contact::new(request_id.clone(), public_key)` where `request_id` is the
identity_id (blake3 hash), so the stored contact's peer_id is rewritten to the
pubkey. The test (`cli/tests/integration_message_requests.rs:146`) asserts
`contacts[0].peer_id == alice_identity_id` — mismatch.

**Approved fix (operator decision 2026-08-28):** handler uses
`Contact::new(public_key.clone(), public_key)`; test asserts
`contacts[0].peer_id == pubkey(&alice)` (canonical pubkey identifier per
Unification V2 P0).

## 3. Environment verified at takeover

- Pixel 6a connected: `adb devices` -> `adb-26261JEGR01896-6pHTac` (device).
- AWS node: `ec2-user@54.226.67.101` with `~/.ssh/scm-node-key.pem`,
  passwordless sudo, docker installed (needs sudo), container `scm-node`,
  identity persists at `/opt/scm-relay-data`. Image
  `testbotz/scmessenger:latest` builds from main via
  `.github/workflows/docker-publish.yml` (push: main + workflow_dispatch).
- gh authenticated as Treystu (https, keyring).
- Toolchain: `cargo` on PATH; `python3` available; gradlew in `android/`.

## 4. Orchestration policies (operator directives 2026-08-28)

1. **Dispatch:** all worker work via the subagent feature, SAME model as this
   session (deepseek). No delegate_task.py / qwenpaid / openrouter rotation;
   no OpenCode-native kimi/glm subagents. Workers run in isolated
   orchestration worktrees with canonical packets + worker-result footers.
2. **Reviews:** any required review (VALIDATOR / CRITICAL_VALIDATOR /
   adversarial) is a review subagent. >99% confidence -> stands. Below 99% ->
   reviewer writes a COMPLETE handoff (verified findings, unresolved items,
   file/line refs, open questions) and a FRESH session continues from it.
   Repeat until >99%. No review skipped, no finding waived by the controller.
3. **CI:** red lanes get a HOLISTIC fix subagent addressing every failure at
   once; gate + push; re-poll from zero. Merge ONLY when ALL checks green —
   no exceptions, no thresholds.
4. **Tag:** the operator owns the v0.4.0 tag and release publish. The
   orchestrator drives everything up to that boundary and hands off an
   evidence pack.

## 5. Plan (from approved takeover strategy)

- **Phase 0 (this doc):** canonicalize session lessons; commit + push
  doc-only on the PR branch.
- **Phase 1:** holistic CI-fix subagent (fmt + test/handler fix + whitespace +
  ratchet session-recovery verification). CRITICAL_VALIDATOR before
  integrating anything touching `core/src/transport/`.
- **Phase 2:** merge PR #234 to main ONLY when all checks green; run
  `scripts/pr_scope.sh 234` first; record evidence.
- **Phase 3:** 3-node live test (AWS scm-node rebuild, Windows CLI, Pixel 6a
  APK) per SHIP_PLAN D4/D6/D7 scoring; pull all logs.
- **Phase 4:** ticket + fix rig findings via the orchestration loop.
- **Phase 5:** prepare tag-boundary evidence pack; operator cuts the tag.

## 6. Open items carried forward

- Ratchet session-recovery verification (code-level) — must complete and be
  recorded in UNIFICATION_V2_RESULTS_PLAN.md.
- D5 branch hygiene sweep (stale branches) — for the tag boundary.
- README refresh, CHANGELOG truthing, release-notes draft — tag boundary.

--- END FILE ---

## 7. Prep completed while CI runs (2026-08-28, unblocked work)

- **D5 branch hygiene inventory (read-only):** 212 remote branches exist.
  17 are merged into main (safe delete candidates: `codex/pr139-five-node-gate-fixes`,
  `gpt/pr139-libp2p-admission-fix-20260811`, `gpt/pr139-receipt-filter-20260811`,
  `gpt/pr139-receipt-fix-20260812`, `orch/qwen-takeover-setup-2026-08-04`,
  `pr-138`, `windows/pr139-ble-parity`, `fork/burn1`, plus pixiegirlchristy
  mirror refs). ~195 remain stale (many `cto/*`, `checkpoint-*`, `dependabot/*`,
  `copilot/sub-pr-*`, `codex/*`, `claude/*`). Deletion requires operator
  approval — inventory only, no deletion performed.
- **AWS deploy readiness:** health 200 `{"status":"healthy"}` at
  54.226.67.101:9876. Docker needs sudo on the box (passwordless). Deploy
  script drafted: `scripts/aws_deploy.sh` (pull testbotz/scmessenger:latest,
  restart scm-node, identity persists at /opt/scm-relay-data, health poll).
  Image rebuilds from main via `.github/workflows/docker-publish.yml` on merge.
- **Release docs state:** README already claims v0.4.0-rc.1 correctly
  (lines 20-21); CHANGELOG has an Unreleased section; release-notes draft
  referenced in V040_COMPLETION_PLAN (Phase A) — no separate file found yet,
  needs drafting at the tag boundary.
- **Signing config:** `android/keystore.properties.template` exists
  (storeFile=release.keystore); actual keystore + secrets are operator-owned.
- **Backup review lane configured:** `scripts/fireworks_opinion.sh`
  (operator-provided Fireworks key, stored in ~/.config/scmorc/fireworks.env,
  never committed). agy (Gemini CLI) available for reviewer roles; opus and
  gemini-3.1-pro named as reviewer options by operator.

--- END FILE ---
