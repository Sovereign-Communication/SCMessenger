# Post-Green Execution Plan (PR 129 -> main)

## Dependency Order

```
0. eprintln->tracing (PREREQUISITE for 1)
1. BLE inbound wedge (BLOCKS everything downstream)
2. CLI bind verification (unblocks 3, 4)
3. AWS node: real build (depends on 2, unblocks 4)
4. 5-node matrix (depends on 1, 2, 3)
5. Outbox drain verification (depends on 1)
6. iOS inbound (parallel with 1, but lower priority)
7. HANDOFF backlog triage (independent, parallel)
8. Doctrine cleanup (independent, parallel)
9. Redaction scrub (independent, parallel)
10. GitHub release v0.4.0 (depends on 1, 4, 6, 7)
```

---

## 0. Convert eprintln! to tracing in mobile_bridge.rs [PREREQUISITE]

**Why**: 15 `eprintln!` calls in `core/src/mobile_bridge.rs` (confirmed count) are
invisible on Android -- stderr does not reach logcat. Every diagnostic inside
`on_data_received` (lines 1390, 1403, 1421, 1429, 1436) is an `eprintln!`.
Without visible logging, the wedge in item 1 cannot be localised.

**First action**: Replace all 15 `eprintln!` in `mobile_bridge.rs` with
`tracing::info!` / `tracing::warn!` / `tracing::error!` as appropriate.
Preserve the `[IronCore]` prefix in the message text for grep-ability.

**Verification**: Build the Android app, trigger any BLE interaction, confirm
the former `eprintln!` messages now appear in `adb logcat` filtered by the
`[IronCore]` tag. One line of logcat output suffices.

**Effort**: 30 min.

---

## 1. BLE Inbound Wedge -- Critical Bug

### Observed behaviour

- `BleGattServer.kt:376-384` logs "mesh_ble_forward" BEFORE `onDataReceived`,
  "mesh_ble_forward_return" AFTER. Device shows 264/0 and later 46/0 -- the
  function NEVER returns.
- Chain: `BleGattServer.kt:380` -> `MeshRepository.kt:2837`
  `meshService?.onDataReceived(peerId, data)` (synchronous UniFFI) ->
  `mobile_bridge.rs:1385 MeshService::on_data_received` ->
  `iron_core.rs:2994 IronCore::receive_message`
- The delegate at `iron_core.rs:3162` is never reached: the Kotlin delegate
  logs "Message from" at `MeshRepository.kt:1752` and that string appears
  ZERO times in device logs.

### Lock order in receive_message (iron_core.rs:2994-3172)

Phase 1 (inside the ratchet `else` block, guards dropped at line 3055):
  1. `identity.read()` @3026
  2. `contact_manager.read()` @3035
  3. `ratchet_sessions.write()` @3041

Phase 2 (after phase-1 guards released):
  4. `blocked_manager.read()` @3065
  5. `blocked_manager.read()` @3089
  6. `contact_manager.read()` @3075
  7. `delegate.read()` @3105 (receipt path only)
  8. `inbox.write()` @3129
  9. `audit_log.write()` @3154
  10. `delegate.read()` @3162

### Lock order in prepare_message_internal (iron_core.rs:696-848)

Phase 1 (guards held SIMULTANEOUSLY until line 790):
  1. `identity.read()` @703
  2. `contact_manager.read()` @748
  3. `ratchet_sessions.write()` @754
  4. `audit_log.write()` @755

Phase 2 (after phase-1 guards released):
  5. `drift_store.write()` @840 OR `outbox.write()` @848

### Key structural difference

In the send path, `ratchet_sessions.write()` and `audit_log.write()` are held
SIMULTANEOUSLY (lines 754-755). In the receive path, `ratchet_sessions.write()`
is released at line 3055 BEFORE `audit_log.write()` is acquired at line 3154.
This means the receive path does NOT hold `ratchet_sessions` when it takes
`audit_log`, so a simple ABBA between these two is ruled out. But the send
path holds both at once, and any other thread blocked on `ratchet_sessions`
while the send path holds `audit_log` creates a different contention pattern.

### Top 3 candidate root causes

**Candidate A: `identity.read()` starvation by a pending `identity.write()`**
`parking_lot::RwLock` is writer-preferring. If any thread is waiting on
`identity.write()` (e.g., `initialize_identity` at line 621, `set_nickname`
at line 1227, or `import_identity_from_payload` at line 1661), the RwLock
blocks new readers. The receive path's `identity.read()` @3026 would block
indefinitely until the write lock is granted and released. The write holder
itself might be blocked on something else (e.g., `audit_log.write()` at line
627, which the send path also holds).
- **Confirming evidence**: `tracing::info!` added at line 3026 (before
  `identity.read()`) fires but the next tracing at line 3027 (after) does not.
- **Eliminating evidence**: Both fire, meaning `identity.read()` is not the
  bottleneck.

**Candidate B: `ratchet_sessions.write()` contention from the send path**
The send path holds `ratchet_sessions.write()` and `audit_log.write()`
simultaneously during `encrypt_with_ratchet_fallback` (lines 754-756). If the
send path is invoked on the tokio runtime while `receive_message` is blocked
on `ratchet_sessions.write()` @3041, the send path would be holding the lock
and the receive path would be waiting. The send path should release the lock
after the crypto operation completes (milliseconds), so this alone would cause
a delay, not a permanent block. But if the send path is ALSO blocked -- e.g.,
the tokio runtime's thread pool is exhausted, or the send path is blocked on
`inbox.write()` or `outbox.write()` I/O -- then `ratchet_sessions.write()`
would be held indefinitely.
- **Confirming evidence**: `tracing::info!` before `ratchet_sessions.write()`
  @3041 fires but the next tracing at line 3042 does not. AND device log shows
  concurrent send activity (e.g., `encrypt_with_ratchet_fallback` tracing).
- **Eliminating evidence**: Line 3042 tracing fires, meaning the lock was
  acquired successfully.

**Candidate C: Sled database I/O blocking inside `inbox.write()` or `audit_log.write()`**
The `inbox.write()` @3129 calls `Inbox::receive()` which does `backend.put()`
via Sled. The `audit_log.write()` @3154 also does I/O. If the Sled database
is in a bad state (compaction storm, disk full, lock contention on the sled
tree), these calls could block for seconds or indefinitely. The phase-1 guards
(`identity`, `ratchet_sessions`) are already released by this point, so the
function would appear to be stuck "after crypto" but before the delegate.
- **Confirming evidence**: `tracing::info!` after line 3055 (phase-1 guards
  released) fires, but tracing before `inbox.write()` @3129 does not, OR
  tracing before `inbox.write()` fires but tracing after `inbox.write()` @3140
  does not.
- **Eliminating evidence**: Tracing around `inbox.write()` fires normally,
  meaning the block is further downstream.

### Diagnostic sequence (ordered)

Step 1: **Instrument `receive_message` with tracing at every lock boundary**.
Add `tracing::info!` at these exact points:

| Point | Before/After | Line | What it proves |
|-------|-------------|------|----------------|
| A | before `identity.read()` | ~3026 | Function entry confirmed |
| B | after `identity.read()` | ~3027 | identity lock not contended |
| C | before `contact_manager.read()` | ~3035 | identity guard still held |
| D | after `contact_manager.read()` | ~3036 | contact_manager not contended |
| E | before `ratchet_sessions.write()` | ~3041 | pre-lock state |
| F | after `ratchet_sessions.write()` | ~3042 | ratchet_sessions not contended |
| G | after `decrypt_with_ratchet_fallback` returns | ~3055 | crypto not blocking |
| H | before `inbox.write()` | ~3129 | phase-1 guards released |
| I | after `inbox.write()` | ~3140 | inbox I/O not blocking |
| J | before `audit_log.write()` | ~3154 | inbox guard released |
| K | after `audit_log.write()` | ~3159 | audit I/O not blocking |
| L | before `delegate.read()` | ~3162 | all locks released |

Step 2: **Add lock-hold-duration tracing to the send path**. In
`prepare_message_internal`, add `tracing::info!` at lines 754 (before
`ratchet_sessions.write()`), 755 (after), and 790 (after both guards are
released). Log the elapsed time. This tells us whether the send path is
holding `ratchet_sessions` for seconds.

Step 3: **Add `tracing::info!` to `identity.write()` holders**. At lines
621, 1227, 1661 -- before and after the write lock acquisition. This tells
us whether a write lock is pending and blocking readers.

Step 4: **Deploy and reproduce**. Run the instrumented build on a device with
BLE peer, capture logcat filtered by the tracing target. The highest letter
reached (A-L) before the log goes silent identifies the exact blocking point.

Step 5: **Interpret**. If the last log is at E (before ratchet_sessions.write),
Candidate B is confirmed. If at A (before identity.read), Candidate A is
confirmed. If at H or I, Candidate C is confirmed.

### Outbox drain verification (depends on this fix)

After the wedge is fixed, the stuck outbox should drain automatically because
receipts arrive over the same inbound path. Verification:

1. Before fix: observe outbox count N (e.g., 9 or 10).
2. After fix: send a message to a peer, observe the receipt arrive via
   `on_receipt_received` (line 3105-3112), confirm the message is removed
   from the outbox.
3. Monitor outbox count over 5 minutes. It should decrease to 0.
4. If outbox does not drain, the retry guard at `mobile_bridge.rs` (the
   "transport-acked message cannot be downgraded" rule) may need separate
   investigation. But do NOT plan a separate outbox fix until the wedge is
   resolved and this verification step is run.

**Effort**: 2-4 hours for instrumentation + 1-2 hours for diagnosis + fix
depends on root cause.

---

## 2. CLI Node Bind Verification

**Why**: `cli/src/main.rs` is 4195 lines (restored from a gutted 170-line
version). It has never been run end-to-end since the restore. The `cmd_relay`
subcommand at line 2660 starts a libp2p swarm and HTTP server. The
`cmd_run` subcommand at line ~1500 starts a WebSocket UI + swarm. Both
must bind real ports.

**First action**: Build and run `scmessenger-cli run` on macOS or Windows.
Capture the PID. Then:

```
netstat -an | grep <PID>    # macOS/Linux
netstat -ano | findstr <PID>  # Windows
```

Or use `ss -tlnp | grep <PID>` on Linux. Match the port to the log line
`"P2P swarm started on /ip4/0.0.0.0/tcp/<PORT>"` (line 2826).

**Verification**: A `LISTEN` socket bound to the expected port appears in
netstat/ss output, AND the log line with the exact port number is present.
Exit code 0 alone is NOT sufficient.

**Effort**: 1 hour.

---

## 3. AWS Node: Real Build

**Why**: The AWS node currently runs a stub loop. `cloud/mesh/Dockerfile.cli`
builds the real `scmessenger-cli` binary. After PR 129 merges, the Docker
image is republished, letting the node run a real build.

**First action**: Build the Docker image locally:
```
docker build -f cloud/mesh/Dockerfile.cli -t scm-cli-node .
```
Then push to the AWS ECR registry and deploy to the running ECS task.

**Verification**: The ECS task logs show the "P2P swarm started on" line
from the CLI, AND the node's peer ID appears in the connection ledger of
another node (e.g., the macOS CLI).

**Effort**: 2 hours (build + deploy + verify).

---

## 4. 5-Node Matrix

**Why**: End-to-end validation across all target platforms.

**Depends on**: Items 1 (BLE fix), 2 (CLI bind), 3 (AWS real build).

**Matrix**:
| Node | Platform | Transport |
|------|----------|-----------|
| A | iOS | BLE + internet |
| B | Android | BLE + internet |
| C | macOS CLI | Internet |
| D | Windows CLI | Internet |
| E | AWS (Docker) | Internet (relay) |

**First action**: After items 1-3 are done, run all 5 nodes simultaneously.
Send a message from each node to every other node (5x4 = 20 messages).
Verify each message is received and the receipt is delivered.

**Verification**: For each of the 20 message pairs, the sender sees a
"Delivered" receipt and the receiver sees the message content. No message
is lost or stuck in the outbox for more than 30 seconds.

**Effort**: 4 hours (setup + test + debug).

---

## 5. Outbox Drain Verification

**Depends on**: Item 1 (BLE fix).

Covered in item 1's "Outbox drain verification" section. Do NOT plan a
separate outbox fix until the wedge is resolved and this verification step
is run.

**Effort**: 1 hour (verification only, assuming the wedge fix resolves it).

---

## 6. iOS Inbound Messages Not Surfacing

**Why**: Separate from the Android BLE wedge. iOS receives messages but they
do not surface in the UI.

**First action**: Check the iOS delegate callback chain. The Rust side calls
`delegate.on_message_received` at `iron_core.rs:3163`. The Swift delegate
must receive this callback and update the UI. Check if the delegate is
properly wired (similar to the Kotlin delegate at `MeshRepository.kt:1752`).

**Verification**: Send a message from another node to the iOS device. The
message appears in the iOS app's conversation view within 5 seconds.

**Effort**: 2-4 hours (depends on root cause).

---

## 7. HANDOFF Backlog Triage

**Why**: 41 files in `HANDOFF/todo/`, 7 in `HANDOFF/in_progress/` (confirmed
counts). Some may be stale or already done.

**First action**: Read each file's header/status line. Categorise as:
- DONE (work already completed in a later PR)
- STALE (superseded by architecture changes)
- ACTIVE (still needs doing)

**Verification**: A markdown table in HANDOFF/ with each file's status and
a one-line summary. The table has exactly 48 rows (41 + 7).

**Effort**: 2 hours.

---

## 8. Doctrine Cleanup (330 violations, 120 files)

**Why**: `hygiene.yml` enforces doctrine in `--changed` mode. The 330
violations across 120 files are not blocking (they're not in changed files),
but they'll surface as soon as any of those files are touched.

**First action**: Run the hygiene check in full-repo mode (not `--changed`)
to get the complete list. Then batch-fix the violations by category
(naming, import order, etc.).

**Verification**: `hygiene.yml` passes in full-repo mode with zero violations.

**Effort**: 4-8 hours (depends on violation types).

---

## 9. Redaction Scrub

**Why**: The repo is PUBLIC. Existing HANDOFF docs contain peer IDs, public
keys, BLE MACs, and IPs. Forward scrub only (no git history rewrite).

**First action**: Scan all files in `HANDOFF/` for hex strings matching
peer IDs (64-char hex), public keys (64-char hex), BLE MACs (6-byte
colon-separated hex), and IP addresses. Replace with `[REDACTED]` or
equivalent.

**Verification**: `grep -rE '[0-9a-f]{64}' HANDOFF/` returns no matches
(except known test/fixtures). `grep -rE '([0-9a-f]{2}:){5}[0-9a-f]{2}'`
returns no matches in HANDOFF/.

**Effort**: 2 hours.

---

## 10. GitHub Release v0.4.0

**Depends on**: Items 1, 4, 6, 7.

**First action**: Draft release notes summarising the changes since v0.3.0.
Tag the commit after all dependent items are verified.

**Verification**: The release appears on the GitHub releases page with
correct tag, binary assets, and release notes.

**Effort**: 1 hour.

---

## Parallelism

| Parallel track | Items |
|----------------|-------|
| Track A (critical) | 0 -> 1 -> 5 |
| Track B (CLI+AWS) | 2 -> 3 |
| Track C (iOS) | 6 |
| Track D (cleanup) | 7, 8, 9 |
| Track E (release) | 10 (after A, B, C, D) |

Tracks B, C, D can run in parallel with track A. Track E is gated on all
others.

---

## Risks / What Could Invalidate the Plan

1. **The wedge is not a lock issue**. If the root cause is a Sled database
   corruption or a file descriptor leak, the diagnostic sequence in item 1
   will still localise it, but the fix will be different (DB repair, FD
   limit increase, etc.).

2. **The outbox does not drain after the wedge fix**. The retry guard
   ("transport-acked message cannot be downgraded") may be a separate bug.
   Item 5's verification step will catch this.

3. **iOS inbound is the same root cause as Android**. If the iOS issue is
   also a blocking `receive_message` call (not BLE-specific), fixing item 1
   may fix item 6. Verify after item 1 is resolved.

4. **CLI build fails on the restored main.rs**. The 4195-line file was
   restored from a gutted version. If it has compilation errors, item 2
   will surface them. Effort estimate assumes a clean build.

5. **AWS Docker image build fails**. The Dockerfile uses `rust:1.95-slim`.
   If the workspace has dependency issues, the build may fail. Unlikely
   after PR 129 is green.

6. **Hygiene violations are deeper than naming**. If some of the 330
   violations are architectural (e.g., wrong module boundaries), the fix
   effort could be much larger than estimated.

7. **Redaction scrub misses something**. The regex patterns may not catch
   all sensitive data (e.g., base64-encoded keys, non-standard formats).
   A manual review of the diff is recommended.
