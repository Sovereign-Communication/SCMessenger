# Agy (Gemini Pro)  Complete Handoff & Remediation Ticket

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**From:** Hermes (passive audit) on behalf of Lucas
**To:** Claude Code (orchestrator)  pick up, delegate, verify
**Date:** 2026-06-07 20:25 PT
**Status:** NEW  unblock, then dispatch
**Priority:** P0/P1 (all 6 bugs ship-block)
**Agy process:** STOPPED (PID 19276 terminated 20:24 PT)

---

## 0. Context  what just happened

Agy (Gemini Pro, running in cmd on Windows as `agy.exe` PID 19276, started 19:46 PT) was doing two things in parallel:
- Driving on-device Android testing via `adb` against `192.168.0.138:38961`
- Running the local `scmessenger-cli.exe` swarm to validate transport/discovery

It produced a substantial body of work. Lucas asked me to **extract the value, stop Agy, and roll the learnings into this ticket** so the swarm can execute on them.

**Key context for orchestrator:**

1. **Agy CWD was `E:\SCMessenger` (old build copy), not `E:\SCMessenger-Github-Repo\SCMessenger` (git repo).** This means Agy's research points at the *current* source files but its test runs hit the *old* binary at `E:\SCMessenger\bin\scmessenger-cli.exe`. New fixes MUST land in the git repo so they get picked up by `assembleDebug` / `cargo build`.
2. **PowerShell execution policy is blocking Agy's stdout** (`scripts is disabled on this system`). Cosmetic, but every Bash result has 2 lines of stderr noise.
3. **`adb shell curl` failed**  Android shells don't ship `curl`. Use `wget`, `toybox wget`, or push a static binary. Or query the device's local API through `adb forward tcp:8080 tcp:8080` then curl on Windows side.
4. **The Android device at `192.168.0.138:38961` was offline intermittently.** Agy saw the daemon restart, the device list drop to 0, then return. Not blocking but affects test reliability.
5. **The Pixel 6a mDNS listener is gone** (per the 2026-06-06 PHASE 2 retest notes). Live BLE/mDNS validation on hardware is blocked until Lucas re-attaches the device.

---

## 1. Agy's primary deliverable  Android Issue Catalog

**Source:** `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-a663-4915-89b9-bc90394adcd5\android_issue_catalog.md` (6 KB, 82 lines)

### Bug 1  Concurrent `createIdentity()` race [P0 / Critical]
- **File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- **Symptom:** 4 threads hit `ironCore.initializeIdentity()` simultaneously. UI flips between initialized/not-initialized.
- **Log evidence (from Agy):**
  ```
  06-07 01:48:19.463 21515 21923 D MeshRepository$createIdentity: Calling ironCore.initializeIdentity()...
  06-07 01:48:19.465 21515 21904 D MeshRepository$createIdentity: Calling ironCore.initializeIdentity()...
  06-07 01:48:19.473 21515 21929 D MeshRepository$createIdentity: Calling ironCore.initializeIdentity()...
  06-07 01:48:19.475 21515 21930 D MeshRepository$createIdentity: Calling ironCore.initializeIdentity()...
  ```
- **Fix (per Agy):** Add `Mutex` guard inside `MeshRepository.createIdentity()`:
  ```kotlin
  private val identityCreationMutex = Mutex()
  suspend fun createIdentity() = withContext(Dispatchers.IO) {
      identityCreationMutex.withLock { /* existing body */ }
  }
  ```
- **Pre-check (Phase 1.2):** Skip if `ironCore?.getIdentityInfo()?.initialized == true`.

### Bug 2  Missing re-entrancy guard in `IdentityViewModel` [P1 / High]
- **File:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityViewModel.kt`
- **Fix:** Add `_isCreating` StateFlow gate. Discard concurrent calls.

### Bug 3  `createIdentity()` called when identity already exists [P1 / High]
- **Same fix as Bug 1 Phase 1.2** (pre-check).

### Bug 4  Redundant backup writes (10+ in 30s) [P2 / Medium]
- **File:** `MeshRepository.kt`  `ensureLocalIdentityFederation()` called from `getIdentityInfo()` lazy path.
- **Fix:** AtomicBoolean/dirty-flag latch  only run once per process lifecycle.

### Bug 5  Brief identity "disappearance" (UI flash) [P1 / High]
- **File:** `MeshRepository.kt`  `isIdentityInitialized()` returns `true` *before* background restore completes.
- **Fix:** Await the restore job before returning. UI sees consistent state.

### Bug 6  Discovered peers disconnect / mDNS peer removal [P1 / High]
- **File:** `android/app/src/main/java/com/scmessenger/android/discovery/MdnsServiceDiscovery.kt` (or similar)
- **Symptom:** `MdnsServiceDiscovery: mDNS peer removed from discovered list...` despite CLI being on the subnet.
- **Likely cause:** Listener port mismatches between what CLI advertises and what Android dials. OR SwarmBridge re-init dropping the connection.
- **Fix path:** Audit the mDNS advertised port vs the SwarmBridge's expected listener; add a test that keeps the connection alive across a config reload.

---

## 2. Agy's secondary deliverable  Contact UI Research

**Source:** structured message `a8e8635e-7365-4370-a2d5-f0c2528916fa.json` in the brain dir

### Finding A  Left column not weighted in `ContactItem` [UI / P2]
- **File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` line 333
- **Issue:** Left `Column` lacks `Modifier.weight(1f)`. Right icon buttons drift based on text length.
- **Fix:** `Column(modifier = Modifier.weight(1f)) { ... }`

### Finding B  Missing bottom padding on ContactsScreen [UI / P2]
- **File:** `ContactsScreen.kt` lines 158-239 (LazyColumn)
- **Issue:** The outer `Scaffold`'s `paddingValues` is passed through `MeshNavHost` but `ContactsScreen` doesn't apply it. The `+` FAB overlaps the last list item.
- **Fix:** `Box(modifier = Modifier.padding(paddingValues))` around the LazyColumn, OR `contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding())` on the LazyColumn.

### Finding C  `AddContactScreen` / `ContactDetailScreen` already use correct pattern
- For reference, `NearbyPeerCard` at `AddContactScreen.kt:617` correctly uses `Modifier.weight(1f)`. Use it as the template.

---

## 3. Agy's tertiary deliverable  CLI Setup Reference

**Source:** structured message `e854a371-f1dd-482e-a6e4-83edd6f144bb.json`

Agy compiled a clean reference doc covering:
- Workspace structure (`core`, `cli`, `mobile`, `desktop_bridge`, `wasm`)
- All pre-built binaries and their sizes
- 4 ways to start the node (start.bat / cargo run / direct binary / WSL systemd)
- Full config.json schema (listen_port=9000, mDNS=true, BLE=false, etc.)
- Port map: 9000=landing/WS, 9001=P2P TCP, 9002=P2P WS bridge
- All CLI subcommands: `init`, `identity`, `contact`, `config`, `history`, `start`, `relay`, `send`, `status`, `stop`, `block`, `discovery`, `swarm stats`, `audit`, `test`, `history-clear`, `mark-sent`
- Bootstrap nodes: `/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw` (GCP, hardcoded default), plus 2 LAN nodes in config.json (`192.168.0.138`, `192.168.0.230`)

**Use this as the canonical ops reference.** The current `docs/` is missing an updated equivalent. Suggest promoting to `docs/CLI_OPERATIONS_REFERENCE.md` after a worker validates it against current source.

---

## 4. Agy's empirical findings  runtime tests

**Source:** task-115.log (152 KB, 08:46 PT overnight) + task-389.log (528 KB, current 20:17 PT)

| Finding | Evidence | Implication |
|---|---|---|
| Identity lifecycle works end-to-end | `Identity initialized: 080605847bc3aca7efc3bc3a2054185aa15c3487c706e885bd725539460e9585`, `Loaded existing identity`, peerId=`12D3KooWFjyBaagUcyuweT26YVoAUtyM1u2K8YnKRgkMJ59zY8fD`, ledger: **245 known peers** | Core is healthy; identity is stable. **Bug 5 is reproducible, not a phantom.** |
| P2P handshake works | `Identified peer 12D3KooWDwXw9CZosa22LcCUgHbrRNPvLTDUo3y8St93AKiHeFky  agent: scmessenger/0.2.1/full/relay/...` | Transport layer solid |
| Gossipsub on `sc-mesh` works | ` Gossipsub message from 12D3KooW... on topic TopicHash { hash: "sc-mesh" }` | Pub/sub plumbing works |
| mDNS / direct dial to LAN often fails | `Failed to negotiate transport protocol(s): [...No connection could be made because the target machine actively refused it. (os error 10061)...]` (dozens of times) | **This is the on-the-wire signature of Bug 6**  listener-port mismatch. Reproducible. |
| AutoNAT outbound always fails | `AutoNAT outbound probe: Error { ... error: NoServer }` (every 30s) | Expected behind NAT, no inbound  not a bug |
| Connection refused to `192.168.0.138:9001` for peer `12D3KooWPD7Mkc9k5Xjyk6BJW2zUKCQpAXEFxv6PvyWCjwiB9z98` | Repeated refusal at 08:51, 08:56  even after the node was up | This peer is the Android device  **the Pixel 6a is offline, confirming the 2026-06-06 PHASE 2 retest blocker still holds** |

---

## 5. Recommended dispatch plan

**Quota state:** 5h=1.2%, 7d=0.2%  TIER 1 (HEAVY-LIFT), 240 min to reset, 3 slots free
**Pool policy:** `local_only` (per Lucas directive 2026-06-06)  all `:cloud` models blocked unless explicitly overridden

**[WARN] Orchestrator note from Lucas:** *"agy does not count as a slot, and local llm doesn't count as a slot."* So you have all 3 slots free.

### Suggested dispatch (in priority order):

**Slot 1  `[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard` already exists in `HANDOFF/todo/`.** Reconcile it against Agy's catalog (Bugs 1, 2, 3, 5 are all the same root cause). Either:
- (a) Update the existing ticket with Agy's specific code/line evidence, then dispatch.
- (b) Create a new combined ticket that supersedes the old one (move old to `HANDOFF/done/[SUPERSEDED]_...`).

**Slot 2  Bug 6 + Finding A + Finding B** (mDNS disconnect + UI alignment) can be a single worker ticket because they touch overlapping files (`MeshRepository`, `MdnsServiceDiscovery`, `ContactsScreen`). Wrap as a new P1 ticket:

```bash
# Suggested file creation
HANDOFF/todo/[VALIDATED]_P1_ANDROID_agy_handoff_identity_stability_and_ui_fixes.md
```

**Slot 3  Reserve for verification.** Once Slots 1 & 2 land, dispatch a verifier (`verifier` agent, `scm-thinker:14b`) to:
- Re-run Agy's `adb -s 192.168.0.138 shell ...` test (with `wget` not `curl`)
- Re-run the cargo test suite
- Confirm `assembleDebug` builds clean
- Confirm the 4 uncommitted test files (`MeshRepositoryHistoryTest.kt`, `BleScannerTest.kt`) either get committed or stashed  they're polluting the working tree

### Cloud-slot override (if Lucas wants Claude Code to use a cloud model)

Per Lucas's "free cloud slot" mention, you'd need to flip `agent_pool.json` from `local_only` to `mixed` or `cloud_preferred`. The free slot mapping per the capability matrix:
- **Architecture/planning** for verifying Agy's catalog  `qwen3-coder:480b:cloud` (or `minimax-m3:cloud` as fallback)
- **Implementation** for the Android fixes  `qwen3-coder-next:cloud` (or `glm-5.1:cloud` as fallback)
- **Crypto/transport review** for Bug 6 (mDNS/port)  `deepseek-v3.2:cloud`

These all need explicit GO because the current `policy: local_only` will reject them.

---

## 6. Pre-flight artifacts to read before dispatching

For the worker that picks up the Android identity ticket:
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  Bug 1, 3, 4, 5
- `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityViewModel.kt`  Bug 2
- `android/app/src/main/java/com/scmessenger/android/ui/MainViewModel.kt`  Bug 5 (timing gap)
- `android/app/src/main/java/com/scmessenger/android/discovery/MdnsServiceDiscovery.kt`  Bug 6
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`  Findings A, B
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt`  Finding B (FAB padding)
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt:617`  reference for correct `weight(1f)` pattern

For the worker that picks up the CLI ops doc:
- `e854a371-f1dd-482e-a6e4-83edd6f144bb.json` (Agy's CLI research)  full text
- `cli/src/main.rs`, `cli/src/cli.rs`  verify the command list against source
- `cli/src/bootstrap.rs`  verify the hardcoded GCP bootstrap
- Current `docs/OPERATIONS.md` (if it exists)  see what needs updating

---

## 7. Verification gates (mandatory before marking done)

- [ ] `cargo check --workspace`  must pass
- [ ] `./gradlew assembleDebug`  must build
- [ ] `./gradlew :app:testDebugUnitTest`  must pass
- [ ] No new `clippy::empty_line_after_doc_comments` warnings
- [ ] Re-test on `192.168.0.138` (or note device offline in handoff)
- [ ] All 4 uncommitted test files either committed or stashed (`git status -s` clean)
- [ ] Doc sync check: `./scripts/docs_sync_check.sh`

---

## 8. Source pointers (where the data lives)

If you need to re-verify any of this:
- **CLI main log:** `C:\Users\SCMessenger\.gemini\antigravity-cli\log\cli-20260607_194628.log` (488 lines, 67 KB)
- **Issue catalog:** `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-a663-4915-89b9-bc90394adcd5\android_issue_catalog.md` (82 lines, 6 KB)
- **Big swarm log:** `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-...\.system_generated\tasks\task-389.log` (528 KB)
- **Overnight swarm log:** `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-...\.system_generated\tasks\task-115.log` (152 KB)
- **Structured messages:** `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-...\.system_generated\messages\*.json` (15 files, includes Contact UI + CLI Setup research)
- **Conversation DB:** `C:\Users\SCMessenger\.gemini\antigravity-cli\conversations\1f072aa0-...cd5.db` (SQLite)

All readable from WSL at `/mnt/c/Users/SCMessenger/.gemini/antigravity-cli/...`

---

*End of ticket. Orchestrator: claim, dispatch, verify, move to done/.*
