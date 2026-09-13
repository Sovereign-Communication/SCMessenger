# Antigravity session 91fbb52e — orphan-hunk census vs PR #282

Date: 2026-09-13
Written by: Buffy (Freebuff lane), recovery session
Method: `git diff 66a09cbc backup/cto-t2-disk-dirty-20260913 -- <file>` (the session's
own edits, captured in backup commit `b1c8957e`) checked against both
`fix/harness-bod-and-android-stability` (PR #282 head) and `origin/main`.

The Antigravity session (6,024 steps) edited this shared checkout directly while
its PR #282 branched from main@956ec371 in worktree `tmp/wt-pr-stability`. The two
lines diverged: PR #282 carries the newer HANG-ANR-001 / HANG-MAIN-001 refinements
(already merged via #281), while this checkout's dirty files are the pre-#281 state
plus the session's edits. An edit "contained" below means its intent exists in PR
#282; "ORPHAN" means it exists nowhere else and must be re-applied after merge.

## File census (10 modified files)

| File | Local delta | vs PR #282 | Disposition |
|---|---|---|---|
| core/src/transport/mesh_routing.rs | (session made none locally) | byte-identical | none |
| android/.../AndroidPlatformBridge.kt | (session made none locally) | byte-identical | none |
| android/.../SubnetProbe.kt | (session made none locally) | byte-identical | none |
| android/.../FileLoggingTree.kt | (session made none locally) | byte-identical | none |
| scripts/fusion_lite.py | DEFAULT_MAX_TOKENS 300 -> 4096 | contained in PR #282 (line 70) | none |
| android/.../MeshApplication.kt | remove compose-crash swallow; explicit kill chain | **ORPHAN #1** | re-apply |
| android/.../MainActivity.kt | onDestroy: stop watchdog before super | **ORPHAN #2** | re-apply |
| android/.../AnrWatchdog.kt | no startService during hang; dead-main zombie kill; main-stack capture | **ORPHAN #3** | re-apply |
| android/.../MeshRepository.kt | cellular: prioritize public IPv4 addrs first (`isPublicIpv4Address`) | **ORPHAN #4** (0 hits in #282) | re-apply |
| cli/src/main.rs (cmd_relay) | relay-mode Text/Receipt handling: identity learning, UI broadcast, delivery-ACK emit, mark_delivered | **ORPHAN #5** (0 hits in origin/main cmd_relay) | re-apply (D4-critical) |

## Detail

### ORPHAN #1 — MeshApplication.kt crash-handler hardening
Removes the `isComposeCrash` swallow branch (swallowing an uncaught exception on
the main thread kills Looper.loop() while background threads keep the process
alive = frozen UI + 20s framework Service ANR) and always chains to the previous
handler, else `Process.killProcess(myPid())` + `exitProcess(10)`.
PR #282 still contains the swallow branch (line 156-164).

### ORPHAN #2 — MainActivity.kt onDestroy ordering
`anrWatchdog.stop()` moved BEFORE `super.onDestroy()`.

### ORPHAN #3 — AnrWatchdog.kt
- `reduceSystemLoad()`: no `context.startService()` during a hang (service
  lifecycle callbacks run on main; AMS waits 20s -> fatal framework ANR). Log only.
- `showBusyIndicator()`: no main-thread toast post during a hang; log only.
- Recovery path: if main thread is dead, kill the zombie process cleanly so the
  OS can restart it.
- Diagnostics: capture main-thread alive/state/stackTrace into the ANR report.
Note: PR #282's AnrWatchdog has the HANG-ANR-001 refinements (logging-only
variant of the same intent); the zombie-kill + stack-capture hunks are the
increment beyond it.

### ORPHAN #4 — MeshRepository.kt cellular address prioritization
`prioritizeAddressesForCurrentNetwork()`: on `networkDetector.isCellularNetwork`,
float public IPv4 addresses ahead of everything else (new helper
`isPublicIpv4Address` rejects 10/8, 172.16/12, 192.168/16, 127/8, 169.254/16).
Absent from PR #282 (grep count 0).

### ORPHAN #5 — cli/src/main.rs cmd_relay receipt convergence (D4-critical)
The headless relay event loop previously handled ONLY `MessageType::OnionRelay`.
The session's edit adds, for `MessageType::Text`: envelope decode ->
`learn_sender_identity_from_envelope`, identity-envelope unwrap, UI broadcast
(Legacy + JsonRpc notif), self-loop suppression, and delivery-ACK emission via
`core_arc.prepare_receipt(...)` + `swarm_handle.send_message(...)`; for
`MessageType::Receipt`: `decode_receipt` + `history_rx.mark_delivered(...)`.
Verified: origin/main's cmd_relay region contains ZERO occurrences of
prepare_receipt / MessageType::Receipt (grep count 0). Without this, a Windows
node run in relay mode never returns receipts and never marks delivery — the
exact D4 receipt-convergence gap.
Rule-8 note: cli/ is NOT inside the rule-8 perimeter (core/src/{crypto,transport,
routing,privacy}/ only). This change is cli-only; no rule-8 gate required.
Touches `core` API surface only through existing public calls.

## Recovery plan

1. Re-apply orphans #1-#5 on top of merged main, one logical commit per orphan.
2. Orphans #1-#4 (Android): must be reconciled with PR #282's newer AnrWatchdog /
   MeshApplication state — apply as deltas on the #282 versions, not as cherry-picks
   of the pre-#281 blobs. Gates: `:app:compileDebugKotlin` + `:app:testDebugUnitTest`
   under build lock.
3. Orphan #5 (cli): applies cleanly on main's cmd_relay (same surrounding code as
   66a09cbc). Gates: `cargo check -p scmessenger-cli`, `cargo test -p scmessenger-cli`,
   fmt + clippy CI-exact, under build lock.
4. After gates: PR to main referencing this census; the receipt-convergence commit
   should be flagged in the D4 evidence path.

Backup lineage: dirty tree preserved verbatim in `backup/cto-t2-disk-dirty-20260913`
(commit `b1c8957e`, 20 files). Nothing was discarded.
