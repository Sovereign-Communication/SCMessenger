# Windows lane -- PR 139 resume state (authoritative)

Written 2026-08-11 by the Windows `/orchestrate` session, wrapping at operator
request (API budget). Read this FIRST on resume, then the latest PR 139 comments.

Pushed head: `d9674458` on `tracking/pre-v040-tag-work` (= PR 139 head branch).
PR comment mirroring this state: issuecomment-5249136798.

## The one thing that matters

**The five-node gate is blocked by a code defect, not by test flakiness.**

`prepare_receipt()` (`core/src/iron_core.rs:1920-1934`) returns bare
`serde_json::to_vec(&Receipt)` instead of an encrypted `Message`/`Envelope`.
Both call sites -- `cli/src/main.rs:2433-2436` and Android
`MeshRepository.kt:2465-2525` -- pass that raw JSON to `send_message()` as if it
were a wire envelope. The peer fails `decode_wire_envelope`
(`core/src/message/codec.rs:217-291`) before reaching the `MessageType::Receipt`
branch that calls `mark_delivered`.

Result: 0 of 40 outbound messages on this node have EVER been marked delivered.
The gate demands "receiver-side inbox_receive plus a delivery ACK with the exact
message ID". That evidence cannot be produced until this is fixed. Re-running
the five-node test more times cannot make it pass.

The repo's own `test_receipt_roundtrip_flips_state`
(`core/tests/integration_ironcore_roundtrip.rs:316-375`) passes only because at
lines 351-353 it performs the missing `prepare_message_with_id(...,
MessageType::Receipt, ...)` step by hand. Production omits it.

Full analysis: `HANDOFF/review/RECEIPT_GAP_ANALYSIS_2026-08-10.md`.

NOT YET FIXED. Receipt logic is WS-A delivery logic -> the delivery gate applies
(3 independent verifiers or one Fusion Lite panel) before it can merge. It is
also wire-encryption correctness, so treat it as security-adjacent even though
`iron_core.rs` is not literally under the four merge-blocked directories.

## Landed this session (pushed)

- `cfaf1b7a` -- BLE process-abort fix. `tokio::spawn` from WinRT
  `TypedEventHandler` callbacks (no Tokio runtime on the COM dispatch thread)
  aborted the node whenever a BLE central read the identity characteristic; 3
  aborts in one boot, each one resetting the soak clock. Both handlers now use a
  `Handle` captured before entering WinRT. `cargo check -p scmessenger-cli` and
  `--tests` exit 0, fmt clean.
  **UNVERIFIED:** the two new regression tests have NOT been observed passing --
  the test build was killed under disk pressure. Re-run
  `cargo test -p scmessenger-cli --lib ble_windows:: -j6` before trusting it.
- `cfaf1b7a` -- PR comment watcher repair. It had NEVER worked: zero successful
  cycles in 22h, cursor frozen, 10+ comments missed. Root cause was not `gh` but
  that `jq` was absent on this host (`gh --jq` uses gh's embedded engine and
  worked; external `jq` pipes silently yielded `count=0`). `jq` 1.7.1 is now
  installed at `~/.local/bin/jq.exe`. **Seven other repo scripts still pipe
  through external jq** and were silently degrading the same way:
  `advanced_monitor.sh`, `check_ollama_models.sh`, `ensure_models.sh`,
  `error_handler.sh`, `get-node-info.sh`, `launch_agent.sh`,
  `test_all_bootstrap_nodes.sh`. They should now work, but none were re-verified.
- `cfaf1b7a` -- PRIVACY. `HANDOFF/todo/INBOX_*.md` and `HANDOFF/logs/` are now
  gitignored. The bridge writes operator `identity_id`, `public_key`,
  `device_id`, LAN addresses and a ROUTABLE PUBLIC IP into those files, this
  repository is PUBLIC, and neither path was ignored. Nothing had been committed.
  Any evidence file must be scanned and redacted before staging.
- `d9674458` -- the receipt gap analysis above.
- `04ab4f7f` -- branch/channel contract (see below). Its subject carries a stray
  leading `@` from a shell quoting error; cosmetic, not worth rewriting history.

## Branch strategy -- settled

One PR (#139), one integration branch (`tracking/pre-v040-tag-work`).
`codex/pr139-five-node-gate-fixes` is FULLY MERGED and stale as an anchor; the
Mac lane's state doc still names it and should stop.

`android/pr139-transport-durability` is NOT merged, 4 commits ahead, including
`fd7655fa fix(android): guard mDNS service-lost against the local peer id` --
plausibly load-bearing for the address-churn/`[DIAL-BACKOFF]` symptoms. Merging
it is an OPEN CONSENSUS ITEM with GPT-MAC. Contract:
`HANDOFF/gpt/WINDOWS_PR139_UNIFIED_BRANCH_AND_CHANNELS_2026-08-10.md`.

## Infrastructure state

- Windows node: LIVE, `http://127.0.0.1:9876`, `/version` git_hash `e5284b7b`,
  binary `%LOCALAPPDATA%\scmessenger\soak\bin\scmessenger-cli-e5284b7b.exe`.
  NOTE: `tasklist` for `scmessenger-cli.exe` is a FALSE NEGATIVE -- the soak runs
  a SHA-pinned binary name. Check the pinned name or curl `/version`.
  PROVENANCE AMBIGUITY: `/version` reports `git_hash=e5284b7b` while
  `core_provenance` embeds `1023d7ae`. Two SHAs from one binary; the gate needs
  all nodes on one head, so resolve this before adjudicating parity.
- Scheduled task `\SCMessengerSoak`: operator decision 2026-08-10 -- boot at
  LOGIN ONLY, user SCM scoped. Already correct as registered (single
  LogonTrigger `Adam\SCM`, `InteractiveToken`, RestartOnFailure 3x/PT5M,
  ExecutionTimeLimit PT0S). Do NOT propose S4U or a boot trigger; that was
  explicitly rejected. Consequence: the node dies at logoff by design, so a
  one-hour gate run must sit inside one uninterrupted session.
- Inbox bridge: single-instance lock added
  (`%LOCALAPPDATA%\scmessenger\soak\inbox_bridge.lock`). `/handoff` vs
  ordinary-chat routing implemented (`/handoff` -> ticket + `[ACK]`; else ->
  `HANDOFF/logs/chat/<date>.jsonl` + `[SEEN]`). Allow-list is the authenticated
  identity_id and is CORRECT (receiving replies proves it matches).
- Android: adb connected wirelessly (Pixel 6a, `bluejay`). Live as an
  authenticated mesh peer.

## UNCOMMITTED work in the tree at wrap time

`scripts/inbox_bridge.py` and `scripts/test_inbox_bridge_routing.py` were being
edited by a background agent when the session wrapped. Two corrections were
issued to it and MUST be confirmed before that file is trusted or committed:

1. Housekeeping detection must NOT key on `schema` starting with `scm.message.`
   -- human and housekeeping messages BOTH carry
   `"schema":"scm.message.identity.v1"`. They differ by `kind` (`text` vs
   `history_sync`/`identity_sync`) and whether `text` is populated. Keying on
   schema would silently swallow every human message.
2. The `/handoff` prefix test was running against the RAW JSON envelope, which
   begins `{"schema":...`, so it could NEVER match -- `/handoff` was
   non-functional. The inner `.text` must be extracted first, then both the
   housekeeping test and the prefix test applied to that extracted text. The
   chat log must store extracted text, not the raw envelope.

Verify both before committing. Do NOT commit that file unread.

Also uncommitted and NOT mine -- leave alone: `HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md`,
`HANDOFF/gpt/GPT_MAC_PR139_TAKEOVER_2026-08-07.md`, `scripts/fusion_lite.py`,
`docs/LOGGING_LEVELS_AUDIT_2026-08-08.md`, `docs/security/AGY_EVIDENCE_SWEEP_c3dae2de.md`,
`scratch/sweep.py`, `screen.png`, `window_dump.xml`. The last two are untracked
artifacts at the repo root and may contain device screen content -- do not commit
them; consider gitignoring.

## Open questions to GPT-MAC (unanswered since 2026-08-10T05:33Z)

The `pixiegirlchristy` identity has not posted since then; all later comments are
under `Treystu`. Five Windows questions remain open: canonical macOS PeerId and
whether its data dir persists across rebuilds; on-device confirmation of the AWS
node in the iOS ledger; iOS DCUtR-attempt logging; macOS/iOS field-test
artifacts; and the panic SITE from the 13m23s capture. Plus: is the 19:00:35Z
re-listing of `8a16beb5-4f2a-4844-b913-70c4cd35a726` a fresh request or a stale
checklist? Windows answered NOT RECEIVED twice on 2026-08-10, verified three ways.

## Resume order

1. Read latest PR 139 comments for a GPT-MAC reply.
2. Confirm concurrence on the receipt diagnosis and the `fd7655fa` merge.
3. Fix `prepare_receipt` behind the WS-A delivery gate. DELEGATE it.
4. Confirm the BLE regression tests actually pass.
5. Re-anchor five nodes on one head, preserve identities.
6. Only then start the one-hour clock.

Constraints that bit this session: disk C: 9.9 GB free (96%) at wrap time and FALLING -- a killed Android gradle build burned ~4 GB -- serialize builds,
never `cargo clean --target`; CI/Actions unavailable, all gates local; the repo
is PUBLIC.

## Wrap-time addendum (read before doing anything)

- **DISK IS THE FIRST PROBLEM.** C: fell from 14 GB to 9.9 GB free (96%) during
  this session; an Android `assembleDebug` was killed mid-flight by a session
  limit and left ~4 GB behind. Reclaim space BEFORE any build. Use
  `scripts/clean_target.sh --dry-run` first. NEVER `cargo clean --target <triple>`
  (wipes the whole tree). Android build artifacts under `android/*/build/` and
  `~/.gradle/caches` are the likely recoverable wins.
- **The node RESTARTED** during wrap-up: PID went 15520 -> 30036 while the
  supervisor (10224) stayed up. Cause not established. It may be the supervisor
  relaunching after a bridge restart, or another BLE abort. Check
  `%LOCALAPPDATA%\scmessenger\soak\runlogs\` for the generation boundary and a
  panic signature. If it was a BLE abort, note the pushed fix `cfaf1b7a` is in
  the SOURCE but the running binary is still the OLD pinned
  `scmessenger-cli-e5284b7b.exe` -- the fix is NOT live until the node is
  rebuilt and re-pinned.
- Two background agents were killed by the session limit mid-task: the Android
  log-pull/APK-install job (never completed; no logs pulled, no APK installed)
  and the inbox-bridge housekeeping filter (see the two corrections above --
  neither was confirmed applied). Treat both as NOT DONE.
- The operator's request "pull my logs and push a new version to this Android
  device" is still OUTSTANDING. adb was connected at wrap time.
