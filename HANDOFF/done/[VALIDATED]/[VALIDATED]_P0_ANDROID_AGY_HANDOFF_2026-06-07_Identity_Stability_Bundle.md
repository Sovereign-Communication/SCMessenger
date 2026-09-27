<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision  2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Priority:** P0 (1 race) + P1 (4) + P2 (1) + UI A/B + CLI ops ref
**Recommended scope:** Bugs 1-5 + UI A/B single worker pass; defer Bug 6 (mDNS) to verification pass (Pixel 6a offline)
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Full handoff is in `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`
(13.7 KB, source-of-truth for the 6 bugs + 2 UI findings + CLI ops ref). This
ticket is the dispatch wrapper. Agy (Gemini Pro worker) ran 19:46  20:24 PT
on 2026-06-07 and is now dead. Per Lucas directive, this track is explicitly
authorized for dispatch.

**Blocker reminder:** Dispatch still blocked from inside the current Claude
session by the `Claude=` env var (env contamination, halt memo at
`HANDOFF/TELEGRAM_OUT_2026-06-07_22-05_PT_ORCHESTRATOR_HALT.md`). Restart in a
fresh shell to unblock.

---

# [VALIDATED] P0_ANDROID  Agy Handoff: Identity Stability & UI Bundle (6 bugs, 1 ops ref)

**Status:** READY-FOR-DISPATCH
**Priority:** P0 (1 race) + P1 (4) + P2 (1)  all ship-block
**Source:** Agy (Gemini Pro) ran 2026-06-07 19:46  20:24 PT. Full handoff in `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`
**Agy process:** STOPPED (PID 19276, 20:24 PT)
**Quota state:** 5h=1.2%, 7d=0.2%  TIER 1, 240 min to reset
**Slots:** 3/3 free (Agy didn't count; local LLMs don't count)

---

## What's in this bundle (from Agy's 35-min investigation)

Agy did real on-device Android testing + local swarm testing. Findings are concrete, evidence-backed, and ready to land.

### Bug 1 (P0)  Concurrent `createIdentity()` race
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**Symptom:** 4 threads call `ironCore.initializeIdentity()` simultaneously. Log evidence (01:48:19) shows 4 entries in 12ms.
**Fix:** Add `private val identityCreationMutex = Mutex()` and wrap `createIdentity()` body in `mutex.withLock { ... }`.
**Pre-check:** Skip if `ironCore?.getIdentityInfo()?.initialized == true`.

### Bug 2 (P1)  `IdentityViewModel` re-entrancy guard missing
**File:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityViewModel.kt`
**Fix:** Add `_isCreating` StateFlow gate.

### Bug 3 (P1)  `createIdentity()` fires when identity already exists
**Same fix as Bug 1 (pre-check).**

### Bug 4 (P2)  Redundant backup writes (10+ in 30s)
**File:** `MeshRepository.kt` `ensureLocalIdentityFederation()` lazy path.
**Fix:** AtomicBoolean latch, run once per process lifecycle.

### Bug 5 (P1)  Brief identity "disappearance" (UI flash)
**File:** `MeshRepository.kt` `isIdentityInitialized()` returns `true` before restore completes.
**Fix:** Await the restore job before returning.

### Bug 6 (P1)  mDNS / discovered peers disconnect
**File:** `android/app/src/main/java/com/scmessenger/android/discovery/MdnsServiceDiscovery.kt`
**Symptom:** `MdnsServiceDiscovery: mDNS peer removed from discovered list...`
**Live signature from Agy's task-115.log (dozens of `os error 10061  No connection could be made because the target machine actively refused it`):**
```
2026-06-07T08:51:18Z  Outgoing connection error to 12D3KooWPD7Mkc9k5Xjyk6BJW2zUKCQpAXEFxv6PvyWCjwiB9z98: ...tcp/9001: : No connection could be made because the target machine actively refused it. (os error 10061)
```
**Likely cause:** Listener-port mismatch between mDNS-advertised port and SwarmBridge's expected port. Audit both sides.

### UI Bug A (P2)  Left column not weighted in `ContactItem`
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt` line 333
**Fix:** `Column(modifier = Modifier.weight(1f)) { ... }` (matches `NearbyPeerCard` at `AddContactScreen.kt:617`).

### UI Bug B (P2)  Missing bottom padding for `+` FAB
**File:** `ContactsScreen.kt` (LazyColumn) + `MeshApp.kt` (outer Scaffold)
**Fix:** Apply `paddingValues` to LazyColumn contentPadding so the FAB doesn't overlap the last list item.

### Bonus  CLI Operations Reference
**Source:** Agy's structured message `e854a371-f1dd-482e-a6e4-83edd6f144bb.json`
Full CLI command list, port map (9000/9001/9002), config.json schema, bootstrap nodes, 4 ways to start the node. **Promote to `docs/CLI_OPERATIONS_REFERENCE.md` after a worker validates against current source.**

---

## Recommended dispatch

**Option A  Single worker (1 slot, ~1800s budget):** Land Bugs 15 + UI A/B in one go. All touch overlapping files (MeshRepository, IdentityViewModel, ContactsScreen). Use `rust-coder:7b` or `scm-coder:7b`. Bug 6 is the riskiest (mDNS)  keep it for a separate verification pass.

**Option B  Two workers (2 slots):**
- **Slot 1 (scm-coder:7b):** Bugs 15 + UI A/B (single Kotlin/Android workstream)
- **Slot 2 (scm-thinker:14b):** Audit Bug 6 (mDNS listener port vs SwarmBridge)  read-only review, write a fix plan to `HANDOFF/done/`

**Option C  Cloud override (free cloud slot, your "fire on all cylinders" mode):**
- Flip `agent_pool.json` policy from `local_only` to `mixed`
- **Slot 1:** `qwen3-coder-next:cloud` (or `glm-5.1:cloud`)  implement Bugs 15 + UI A/B
- **Slot 2:** `qwen3-coder:480b:cloud` (or `minimax-m3:cloud`)  audit Bug 6 with deep architecture review
- **Slot 3:** `deepseek-v3.2:cloud`  review the mDNS fix for security/correctness

---

## Pre-flight artifacts

| File | Purpose |
|---|---|
| `HANDOFF/IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` | **Full ticket**  read this first |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-\android_issue_catalog.md` | Original 6-bug catalog with log evidence |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-\.system_generated\messages\a8e8635e-` | Contact UI research |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-\.system_generated\messages\e854a371-` | CLI Setup research |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-\.system_generated\tasks\task-115.log` | Overnight swarm test (152 KB) |
| `C:\Users\SCMessenger\.gemini\antigravity-cli\brain\1f072aa0-\.system_generated\tasks\task-389.log` | Current session swarm test (528 KB) |

All WSL-readable at `/mnt/c/Users/SCMessenger/.gemini/antigravity-cli/...`

---

## Verification gates (must pass before moving to `done/`)

- [ ] `cargo check --workspace`  clean
- [ ] `./gradlew assembleDebug`  builds
- [ ] `./gradlew :app:testDebugUnitTest`  passes
- [ ] `clippy --workspace -D warnings`  no new warnings
- [ ] No new uncommitted files (the 2 dirty test files from 2026-06-05 must be either committed or stashed)
- [ ] `./scripts/docs_sync_check.sh`  passes
- [ ] If Bug 6 fix lands, manual verification: 2 CLI nodes start, mDNS peer stays connected for 60s (requires Pixel 6a back online, currently blocked)

---

## Do NOT do

-  Don't reopen the uncommitted test files (`MeshRepositoryHistoryTest.kt`, `BleScannerTest.kt`)  they're leftover from the 2026-06-05 P0_024 fix and should be in their own commit
-  Don't delete the `IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md` file  it's the source of truth for the work
-  Don't use `local_only` override without Lucas's explicit GO

---

## Orchestrator decision tree

```
If quota is TIER 1 and Lucas hasn't said "fire on all cylinders":
    Use Option A (single local worker, scm-coder:7b)
If Lucas says "fire on all cylinders" or "use the cloud slot":
    Use Option C (cloud override, 2-3 slots)
If Pixel 6a is offline AND Bug 6 needs live retest:
    Land Bugs 15 + UI A/B now, defer Bug 6 to a verification slot
```

---

*Dispatched by Hermes (passive audit) on behalf of Lucas  2026-06-07 20:27 PT*


# REPO_MAP Context for Task: [VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle

**Target function: `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle`**

## android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt (2 chunks, 771 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt: 5 functions; 42 imports android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt: 5 functions; 42 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ContactsScreen` | 51 | Text, TopAppBar, Icon, padding, FloatingActionButton, Contacts, fillMaxSize, hiltViewModel, mutableStateOf, collectAsState |
| `ContactItem` | 285 | Icon, background, padding, fillMaxSize, mutableStateOf, rememberSwipeToDismissBoxState, SwipeToDismissBox, Box |
| `NearbyPeerItem` | 522 | Text, Icon, width, Card, padding, copy, weight, Row, size, cardColors |
| `AddContactDialog` | 586 | parseContactImportPayload, Text, AlertDialog, remember, mutableStateOf, emptyList, OutlinedButton, toString, orEmpty, getText |
| `formatTimestamp` | 753 | formatTimestamp, Date, currentTimeMillis, SimpleDateFormat, format, getDefault, toEpochMillis |
| `ContactsScreen` | 51 | padding, Text, Column, Contacts, fillMaxSize, mutableStateOf, collectAsState, FloatingActionButton, hiltViewModel, Scaffold |
| `ContactItem` | 285 | padding, fillMaxSize, mutableStateOf, background, rememberSwipeToDismissBoxState, SwipeToDismissBox, Icon, Box |
| `NearbyPeerItem` | 522 | Card, cardColors, padding, weight, Text, size, fillMaxWidth, copy, Row, width |
| `AddContactDialog` | 586 | getText, Text, toString, emptyList, mutableStateOf, OutlinedButton, parseContactImportPayload, orEmpty, remember, AlertDialog |
| `formatTimestamp` | 753 | Date, currentTimeMillis, SimpleDateFormat, getDefault, toEpochMillis, format, formatTimestamp |

### Imports
- `import androidx.compose.foundation.background`
- `import androidx.compose.foundation.clickable`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.lazy.LazyColumn`
- `import androidx.compose.foundation.lazy.items`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.filled.Add`
- `import androidx.compose.material.icons.filled.CameraAlt`
- `import androidx.compose.material.icons.filled.Close`
- `import androidx.compose.material.icons.filled.ContentPaste`
- `import androidx.compose.material.icons.filled.Delete`
- `import androidx.compose.material.icons.filled.Edit`
- `import androidx.compose.material.icons.filled.Info`
- `import androidx.compose.material.icons.filled.Person`
- `import androidx.compose.material.icons.filled.PersonAdd`
- `import androidx.compose.material.icons.filled.Sensors`
- `import androidx.compose.material3.*`
- `import androidx.compose.material3.SwipeToDismissBox`
- `import androidx.compose.material3.SwipeToDismissBoxValue`
- `import androidx.compose.material3.rememberSwipeToDismissBoxState`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.focus.FocusRequester`
- `import androidx.compose.ui.focus.focusRequester`
- `import androidx.compose.ui.graphics.Color`
- `import androidx.compose.ui.platform.LocalContext`
- `import androidx.compose.ui.platform.LocalSoftwareKeyboardController`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt (2 chunks, 400 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt: 4 functions; 26 imports android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt: 4 functions; 26 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ConversationsScreen` | 35 | Text, TopAppBar, Icon, IconButton, Suppress, hiltViewModel, mutableStateOf, collectAsState, Scaffold |
| `StatItem` | 245 | Text, ConversationItem, DeliveryStatePresentation, Column, rememberSwipeToDismissBoxState, firstOrNull |
| `ConversationItem` | 263 | DeliveryStatePresentation, onRequestDelete, Card, fillMaxSize, rememberSwipeToDismissBoxState, SwipeToDismissBox, firstOrNull, Box |
| `formatTimestamp` | 385 | formatTimestamp, Date, currentTimeMillis, SimpleDateFormat, format, getDefault, toEpochMillis |
| `ConversationsScreen` | 35 | Text, IconButton, mutableStateOf, collectAsState, hiltViewModel, Suppress, Icon, TopAppBar, Scaffold |
| `StatItem` | 245 | Text, Column, firstOrNull, rememberSwipeToDismissBoxState, DeliveryStatePresentation, ConversationItem |
| `ConversationItem` | 263 | Card, fillMaxSize, rememberSwipeToDismissBoxState, Box, DeliveryStatePresentation, SwipeToDismissBox, firstOrNull, onRequestDelete |
| `formatTimestamp` | 385 | Date, currentTimeMillis, SimpleDateFormat, getDefault, toEpochMillis, format, formatTimestamp |

### Imports
- `import androidx.compose.foundation.clickable`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.lazy.LazyColumn`
- `import androidx.compose.foundation.lazy.items`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.Chat`
- `import androidx.compose.material.icons.filled.Delete`
- `import androidx.compose.material3.*`
- `import androidx.compose.material3.SwipeToDismissBox`
- `import androidx.compose.material3.SwipeToDismissBoxValue`
- `import androidx.compose.material3.rememberSwipeToDismissBoxState`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.graphics.Color`
- `import androidx.compose.ui.text.style.TextOverflow`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.service.MeshEventBus`
- `import com.scmessenger.android.ui.chat.DeliveryStateMapper`
- `import com.scmessenger.android.ui.chat.DeliveryStatePresentation`
- `import com.scmessenger.android.ui.chat.DeliveryStateSurface`
- `import com.scmessenger.android.ui.viewmodels.ConversationsViewModel`
- `import com.scmessenger.android.utils.toEpochMillis`
- `import java.text.SimpleDateFormat`
- `import java.util.*`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/DashboardScreen.kt (2 chunks, 468 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/DashboardScreen.kt: 11 functions; 35 imports android/app/src/main/java/com/scmessenger/android/ui/screens/DashboardScreen.kt: 11 functions; 35 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `DashboardScreen` | 41 | Text, TopAppBar, LazyColumn, padding, fillMaxSize, hiltViewModel, PaddingValues, collectAsState, Scaffold |
| `PeerItem` | 191 | Text, Icon, width, background, padding, weight, Row, size, Spacer, Column |
| `StatusCard` | 267 | Text, Card, padding, copy, Row, cardColors, fillMaxWidth |
| `StatCard` | 302 | Text, Icon, background, Card, padding, copy, height, size, Column, Spacer |
| `ConnectionStatusCard` | 340 | Text, Icon, Card, padding, Row, TransportItem, Column, fillMaxWidth |
| `TransportItem` | 365 | Text, Icon, formatBytes, height, padding, Row, Column, Spacer, TextDetailRow, fillMaxWidth |
| `TextDetailRow` | 378 | Text, formatBytes, padding, Row, formatDuration, fillMaxWidth, toLong |
| `formatBytes` | 389 | Text, formatBytes, Card, padding, formatDuration, Column, DashboardToPeerListNavigation, fillMaxWidth, toLong |
| `formatDuration` | 398 | Text, Button, Card, padding, formatDuration, Column, DashboardToPeerListNavigation, fillMaxWidth, toLong |
| `DashboardToPeerListNavigation` | 410 | Text, Button, Card, padding, Column, fillMaxWidth |
| `DashboardToTopologyNavigation` | 441 | Text, Button, Card, padding, Column, fillMaxWidth |
| `DashboardScreen` | 41 | padding, Text, fillMaxSize, LazyColumn, collectAsState, hiltViewModel, Scaffold, PaddingValues, TopAppBar |
| `PeerItem` | 191 | padding, Text, Column, weight, size, background, fillMaxWidth, Row, width, Icon |
| `StatusCard` | 267 | Card, cardColors, padding, Text, fillMaxWidth, copy, Row |
| `StatCard` | 302 | Card, padding, Column, Text, size, background, copy, height, Icon, Spacer |
| `ConnectionStatusCard` | 340 | Card, padding, Column, Text, fillMaxWidth, TransportItem, Row, Icon |
| `TransportItem` | 365 | padding, Text, Column, fillMaxWidth, formatBytes, TextDetailRow, height, Row, Icon, Spacer |
| `TextDetailRow` | 378 | padding, Text, formatBytes, fillMaxWidth, toLong, formatDuration, Row |
| `formatBytes` | 389 | Card, padding, Column, Text, formatBytes, fillMaxWidth, toLong, DashboardToPeerListNavigation, formatDuration |
| `formatDuration` | 398 | Card, padding, Column, Text, fillMaxWidth, toLong, Button, DashboardToPeerListNavigation, formatDuration |
| `DashboardToPeerListNavigation` | 410 | Card, padding, Column, Text, fillMaxWidth, Button |
| `DashboardToTopologyNavigation` | 441 | Card, padding, Column, Text, fillMaxWidth, Button |

### Imports
- `import androidx.compose.foundation.background`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.lazy.LazyColumn`
- `import androidx.compose.foundation.lazy.items`
- `import androidx.compose.foundation.shape.CircleShape`
- `import androidx.compose.foundation.shape.RoundedCornerShape`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.filled.Bluetooth`
- `import androidx.compose.material.icons.filled.Bolt`
- `import androidx.compose.material.icons.filled.NetworkWifi`
- `import androidx.compose.material.icons.filled.People`
- `import androidx.compose.material.icons.filled.Person`
- `import androidx.compose.material.icons.filled.Router`
- `import androidx.compose.material.icons.filled.Settings`
- `import androidx.compose.material.icons.filled.Wifi`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.Composable`
- `import androidx.compose.runtime.collectAsState`
- `import androidx.compose.runtime.getValue`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.graphics.Color`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.compose.ui.unit.sp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import androidx.navigation.NavHostController`
- `import com.scmessenger.android.ui.dashboard.PeerListScreen`
- `import com.scmessenger.android.ui.dashboard.TopologyScreen`
---

## android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt (2 chunks, 513 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt: Defines 1 types: MdnsServiceDiscovery; 30 functions; 8 imports android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt: Defines 1 types: MdnsServiceDiscovery; 30 functions; 8 imports

### Structs/Classes
- MdnsServiceDiscovery

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onDiscoveryStarted` | 59 | clear, resolveService, d, i, regType, onServiceFound, onDiscoveryStopped, equals |
| `onDiscoveryStopped` | 69 | clear, resolveService, d, remove, lost, i, onServiceLost, onServiceFound, equals |
| `onServiceFound` | 80 | resolveService, d, remove, onServiceRegistered, lost, i, onServiceLost, equals |
| `onServiceLost` | 94 | d, Suppress, resolved, onServiceRegistered, remove, i, onServiceResolved |
| `onServiceRegistered` | 109 | d, Suppress, resolved, String, i, onServiceResolved, attributes |
| `onServiceResolved` | 120 | d, Suppress, String, onPeerDiscovered, isNullOrBlank, attributes |
| `onServiceUnregistered` | 178 | d, postDelayed, startDiscovery, failure, errorCode, i, onStartDiscoveryFailed, shl, e |
| `onStartDiscoveryFailed` | 188 | d, postDelayed, startDiscovery, failure, errorCode, onStopDiscoveryFailed, shl, e |
| `onStopDiscoveryFailed` | 211 | registration, d, postDelayed, startDiscovery, onRegistrationFailed, errorCode, registerService, shl, e |
| `onRegistrationFailed` | 229 | registration, d, postDelayed, errorCode, registerService, shl, e, onResolveFailed |
| `onResolveFailed` | 252 | start, w, onUnregistrationFailed, e |
| `onUnregistrationFailed` | 264 | start, w, getSystemService, registerService, e |
| `start` | 276 | catch, w, startDiscovery, getSystemService, registerService, i, e |
| `stop` | 308 | stopServiceDiscovery, clear, catch, unregisterService, i, e |
| `registerService` | 340 | onServiceUnregistered, setAttribute, onServiceRegistered, onRegistrationFailed, onUnregistrationFailed, NsdServiceInfo |
| `onRegistrationFailed` | 355 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onRegistrationFailed, onUnregistrationFailed, onDiscoveryStopped, registerService |
| `onUnregistrationFailed` | 358 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onServiceFound, onUnregistrationFailed, onDiscoveryStopped, registerService |
| `onServiceRegistered` | 362 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onServiceLost, onServiceFound, onDiscoveryStopped, registerService |
| `onServiceUnregistered` | 366 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped, registerService |
| `startDiscovery` | 378 | discoverServices, onDiscoveryStarted, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onDiscoveryStarted` | 380 | discoverServices, onDiscoveryStarted, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onDiscoveryStopped` | 383 | discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onServiceFound` | 387 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onResolveFailed |
| `onServiceLost` | 391 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceResolved, onResolveFailed |
| `onStartDiscoveryFailed` | 395 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceResolved, onResolveFailed |
| `onStopDiscoveryFailed` | 399 | resolveService, discoverServices, Suppress, onStopDiscoveryFailed, onServiceResolved, onResolveFailed |
| `resolveService` | 415 | cleanup, resolveService, Suppress, stop, onServiceResolved, onResolveFailed |
| `onResolveFailed` | 417 | cleanup, resolveService, Suppress, stop, onServiceResolved, onResolveFailed |
| `onServiceResolved` | 420 | cleanup, resolveService, Suppress, stop, onServiceResolved |
| `cleanup` | 433 | stop |
| `onDiscoveryStarted` | 59 | regType, d, onDiscoveryStopped, resolveService, i, equals, onServiceFound, clear |
| `onDiscoveryStopped` | 69 | d, remove, resolveService, lost, i, equals, onServiceFound, clear, onServiceLost |
| `onServiceFound` | 80 | d, remove, resolveService, lost, i, equals, onServiceLost, onServiceRegistered |
| `onServiceLost` | 94 | d, remove, resolved, onServiceResolved, i, Suppress, onServiceRegistered |
| `onServiceRegistered` | 109 | d, onServiceResolved, resolved, String, i, Suppress, attributes |
| `onServiceResolved` | 120 | d, String, Suppress, isNullOrBlank, attributes, onPeerDiscovered |
| `onServiceUnregistered` | 178 | d, failure, errorCode, postDelayed, i, startDiscovery, onStartDiscoveryFailed, e, shl |
| `onStartDiscoveryFailed` | 188 | onStopDiscoveryFailed, d, failure, errorCode, postDelayed, startDiscovery, e, shl |
| `onStopDiscoveryFailed` | 211 | d, registration, onRegistrationFailed, errorCode, postDelayed, startDiscovery, registerService, e, shl |
| `onRegistrationFailed` | 229 | d, onResolveFailed, registration, errorCode, postDelayed, registerService, e, shl |
| `onResolveFailed` | 252 | onUnregistrationFailed, start, w, e |
| `onUnregistrationFailed` | 264 | registerService, getSystemService, start, w, e |
| `start` | 276 | w, catch, getSystemService, i, startDiscovery, registerService, e |
| `stop` | 308 | unregisterService, catch, i, stopServiceDiscovery, clear, e |
| `registerService` | 340 | setAttribute, onServiceUnregistered, onRegistrationFailed, onUnregistrationFailed, NsdServiceInfo, onServiceRegistered |
| `onRegistrationFailed` | 355 | onServiceUnregistered, onRegistrationFailed, onDiscoveryStopped, startDiscovery, onUnregistrationFailed, registerService, onDiscoveryStarted, onServiceRegistered |
| `onUnregistrationFailed` | 358 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onUnregistrationFailed, onServiceFound, registerService, onDiscoveryStarted, onServiceRegistered |
| `onServiceRegistered` | 362 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onServiceFound, registerService, onServiceLost, onDiscoveryStarted, onServiceRegistered |
| `onServiceUnregistered` | 366 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onServiceFound, onStartDiscoveryFailed, registerService, onServiceLost, onDiscoveryStarted |
| `startDiscovery` | 378 | onStopDiscoveryFailed, discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onDiscoveryStarted |
| `onDiscoveryStarted` | 380 | onStopDiscoveryFailed, discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onDiscoveryStarted |
| `onDiscoveryStopped` | 383 | discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onServiceFound` | 387 | onResolveFailed, discoverServices, resolveService, onServiceFound, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onServiceLost` | 391 | onResolveFailed, discoverServices, onServiceResolved, resolveService, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onStartDiscoveryFailed` | 395 | onResolveFailed, discoverServices, onServiceResolved, resolveService, onStartDiscoveryFailed, onStopDiscoveryFailed |
| `onStopDiscoveryFailed` | 399 | onResolveFailed, discoverServices, onServiceResolved, resolveService, Suppress, onStopDiscoveryFailed |
| `resolveService` | 415 | onResolveFailed, cleanup, onServiceResolved, resolveService, Suppress, stop |
| `onResolveFailed` | 417 | onResolveFailed, cleanup, onServiceResolved, resolveService, Suppress, stop |
| `onServiceResolved` | 420 | cleanup, onServiceResolved, resolveService, Suppress, stop |
| `cleanup` | 433 | stop |

### Imports
- `import android.content.Context`
- `import android.net.nsd.NsdManager`
- `import android.net.nsd.NsdServiceInfo`
- `import android.os.Build`
- `import android.os.Handler`
- `import android.os.Looper`
- `import java.util.concurrent.ConcurrentHashMap`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt (2 chunks, 8931 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt: Defines 23 types: MeshRepository, BootstrapSource, EnvironmentBootstrapSource, LocalTransportFallbackResult, RoutingHints; 295 functions; 31 imports android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt: Defines 23 types: MeshRepository, BootstrapSource, EnvironmentBootstrapSource, LocalTransportFallbackResult, RoutingHints; 295 functions; 31 imports

### Structs/Classes
- AllRelaysFailed
- BleRouteObservation
- BootstrapAttempt
- BootstrapResult
- BootstrapSource
- Connected
- DecodedMessagePayload
- DeliveryAttemptResult
- DeliveryStatus
- EnvironmentBootstrapSource
- Failure
- IdentityEmissionSignature
- LocalTransportFallbackResult
- MdnsFallback
- MeshRepository
- MessageIdentityHints
- MessageTracking
- PeerDiscoveryInfo
- PendingOutboundEnvelope
- ReplayDiscoveredIdentity
- RoutingHints
- Success
- TransportIdentityResolution

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `getBootstrapNodesForSettings` | 81 | unavailable, mapToSmartTransportType, getBootstrapNodes, split, emptyList, isMeshParticipationEnabled, getenv, trim, isNotEmpty |
| `getBootstrapNodes` | 85 | unavailable, mapToSmartTransportType, getBootstrapNodes, split, emptyList, isMeshParticipationEnabled, getenv, trim, isNotEmpty |
| `getBootstrapNodes` | 90 | unavailable, mapToSmartTransportType, split, emptyList, isMeshParticipationEnabled, getenv, trim, mapFromSmartTransportType, isNotEmpty |
| `isMeshParticipationEnabled` | 95 | unavailable, mapToSmartTransportType, isMeshParticipationEnabled, mapFromSmartTransportType |
| `mapToSmartTransportType` | 104 | isEnabledFlag, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, mapFromSmartTransportType |
| `mapFromSmartTransportType` | 117 | lowercase, isEnabledFlag, LocalTransportFallbackResult, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, trim |
| `requireMeshParticipationEnabled` | 125 | get, lowercase, isEnabledFlag, LocalTransportFallbackResult, requireMeshParticipationEnabled, isMeshParticipationEnabled, IllegalStateException, trim, attemptWifiThenBleFallback |
| `isEnabledFlag` | 133 | get, lowercase, LocalTransportFallbackResult, isEnabledFlag, tryWifi, isNotEmpty, trim, attemptWifiThenBleFallback |
| `attemptWifiThenBleFallback` | 150 | tryBle, LocalTransportFallbackResult, tryWifi, isNotEmpty, trim, attemptWifiThenBleFallback |
| `getAvailableStorageMB` | 190 | triggering, get, w, trackNetworkFailure, mapToSmartTransportType, classifyBootstrapError, Triple, getAvailableStorageMB, checkAndRecordMessage, recordFailure |
| `checkAndRecordMessage` | 198 | triggering, get, w, isNodeUnreachable, trackNetworkFailure, mapToSmartTransportType, triggerFallbackProtocol, classifyBootstrapError, Triple, checkAndRecordMessage |
| `enhanceNetworkErrorLogging` | 204 | triggering, get, compareAndSet, w, isNodeUnreachable, trackNetworkFailure, triggerFallbackProtocol, classifyBootstrapError, recordFailure, enhanceNetworkErrorLogging |
| `trackNetworkFailure` | 210 | get, compareAndSet, w, isNodeUnreachable, trackNetworkFailure, triggerFallbackProtocol, recordFailure, isNotEmpty, triggering |
| `triggerFallbackProtocol` | 231 | compareAndSet, allowRequest, w, catch, set, i, dial, isNotEmpty, recursion |
| `isCorrupted` | 527 | recordSuccess, currentTimeMillis, markCorrupted, recordFailure |
| `markCorrupted` | 534 | recordSuccess, currentTimeMillis, recordFailure |
| `recordSuccess` | 541 | currentTimeMillis, forMessage, MessageTracking, recordFailure |
| `recordFailure` | 553 | recoverFromCorruption, currentTimeMillis, MessageTracking, forMessage |
| `forMessage` | 567 | recoverFromCorruption, currentTimeMillis, MessageTracking |
| `recoverFromCorruption` | 574 | currentTimeMillis, getenv, MessageTracking, isEnabledFlag |
| `isTerminalIdentityFailure` | 606 | isTerminalIdentityFailure, markCorrupted, w, markMessageCorrupted, terminalIdentityFailureMessage, trim |
| `terminalIdentityFailureMessage` | 614 | markCorrupted, w, markMessageCorrupted, getMessageIdTracking, terminalIdentityFailureMessage, trim |
| `markMessageCorrupted` | 633 | markCorrupted, w, forMessage, isCorrupted, detectAndRecoverMessageTracking, add, messageId, getMessageIdTracking |
| `getMessageIdTracking` | 643 | w, forMessage, isCorrupted, detectAndRecoverMessageTracking, add, recoverFromCorruption, messageId, i |
| `detectAndRecoverMessageTracking` | 657 | w, isCorrupted, add, incrementAttemptCount, recoverFromCorruption, i, messageId, getMessageIdTracking, isNotEmpty |
| `incrementAttemptCount` | 685 | getRetryDelay, shouldRetryMessage, recordFailure, getMessageIdTracking |
| `getRetryDelay` | 695 | logRetryStormDetection, d, storms, logMessageDeliveryAttempt, shouldRetryMessage, getMessageIdTracking |
| `shouldRetryMessage` | 709 | logRetryStormDetection, d, storms, w, logMessageDeliveryAttempt, getMessageIdTracking, count, checkReinstallState, enabled |
| `logMessageDeliveryAttempt` | 717 | logRetryStormDetection, d, w, initializeManagers, thread, storms, count, onCreate, checkReinstallState, enabled |
| `logRetryStormDetection` | 724 | d, catch, w, startStorageMaintenance, initializeManagers, initializeRepository, thread, i, count, onCreate |
| `initializeRepository` | 748 | catch, d, w, contains, i, File, exists, checkReinstallState, startStorageMaintenance |
| `checkReinstallState` | 756 | d, w, FIX, initializeManagers, contains, i, File, exists, checkReinstallState |
| `initializeManagers` | 782 | enforceRetention, catch, FIX, migrateContactsFromOldLocation, HistoryManager, loop, w, initializeManagers, MeshSettingsManager, toULong |
| `verifyContactDataIntegrity` | 872 | exist, d, catch, w, diagnostics, list, e, take, isNullOrEmpty, minOf |
| `migrateContactsFromOldLocation` | 916 | d, apply, getSharedPreferences, length, i, File, exists, edit, putBoolean, getBoolean |
| `migrateStaleRoutingHints` | 1002 | orEmpty, list, trim, add, getSharedPreferences, contains, migrateStaleRoutingHints, split, startsWith, Contact |
| `migrateTruncatedPublicKeys` | 1057 | w, orEmpty, key, list, trim, take, getSharedPreferences, startsWith, getBoolean |
| `testLedgerRelayConnectivity` | 1122 | d, catch, w, toIntOrNull, InetSocketAddress, Socket, close, getPreferredRelays, indexOf, reachable |
| `startMeshService` | 1166 | start, d, getCore, i, getState, currentTimeMillis, withStorageAndLogs, e |
| `onPeerDiscovered` | 1229 | d, catch, isBootstrapRelayPeer, resolveTransportIdentity, getIdentityInfo, extractPublicKeyFromPeerId, isNullOrBlank, PeerDiscoveryInfo, prepopulateDiscoveryNickname |
| `onPeerIdentified` | 1318 | d, recordTransportEvent, listOf, onPeerIdentified, contains, peerId, currentTimeMillis, sorted, isNotEmpty, trim |
| `onPeerDisconnected` | 1546 | aliases, d, onPeerDisconnected, emitDisconnectedIfChanged, pruneDisconnectedPeer, recordTransportEvent, remove, currentTimeMillis, trim |
| `onMessageReceived` | 1581 | load, logDeliveryAttempt, onMessageReceived, i, checkAndRecordMessage, detected, disabled, enabled |
| `onReceiptReceived` | 1979 | get, d, catch, logDeliveryState, loadPendingOutbox, lowercase, removePendingOutbound, onReceiptReceived, trim |
| `sendDeliveryReceiptAsync` | 2113 | get, d, prepareReceipt, catch, sendDeliveryReceiptAsync, launch, blocked, isBlocked, emptyList, i |
| `sendIdentitySyncIfNeeded` | 2221 | d, catch, isBootstrapRelayPeer, launch, add, remove, prepareMessageWithId, sendIdentitySyncIfNeeded, extractPublicKeyFromPeerId, encodeIdentitySyncPayload |
| `sendHistorySyncIfNeeded` | 2282 | catch, w, isBootstrapRelayPeer, sendHistorySyncIfNeeded, getIdentityInfo, currentTimeMillis, trim, isEmpty |
| `sendHistorySyncDataIfNeeded` | 2349 | parseRoutingHints, emptyList, conversation, d, sendHistorySyncDataIfNeeded, buildRoutePeerCandidates, putIfAbsent, w, chunked, JSONObject |
| `initializeAndStartBle` | 2411 | d, onDataReceived, w, noteBleRouteObservation, BleGattClient, BleScanner, hasAllPermissions, onPeerDiscovered, onPeerIdentityRead, loadSettings |
| `updateBleIdentityBeacon` | 2491 | delay, launch, isNullOrEmpty, identity, getIdentityInfo, getListeningAddresses, currentTimeMillis, emptyList, setIdentityBeaconInternal, isEmpty |
| `setIdentityBeaconInternal` | 2522 | normalizeOutboundListenerHints, take, distinct, buildBeacon, put, normalizeExternalAddressHints, getExternalAddresses, JSONObject, toByteArray, libp2p_peer_id |
| `buildBeacon` | 2530 | distinct, take, buildBeacon, put, JSONObject, toByteArray, emptyList, libp2p_peer_id, toString, JSONArray |
| `onPeerIdentityRead` | 2600 | isNotBlank, w, noteBleRouteObservation, getString, optJSONArray, JSONObject, isNullOrBlank, optString, toString, trim |
| `updateDiscoveredPeer` | 2782 | selectCanonicalPeerId, copy, updateDiscoveredPeer, maxOf, normalize, selectAuthoritativeNickname, normalizeNickname |
| `noteBleRouteObservation` | 2821 | resolveFreshBlePeerId, orEmpty, noteBleRouteObservation, asSequence, BleRouteObservation, fallback, currentTimeMillis, isNotEmpty, trim, isEmpty |
| `resolveFreshBlePeerId` | 2835 | d, candidate, asSequence, remove, fallback, currentTimeMillis, resolveFreshBlePeerId, isNotEmpty, trim, isEmpty |
| `pruneDisconnectedPeer` | 2868 | d, loadSettings, initializeAndStartWifi, pruneDisconnectedPeer, trim, isEmpty, normalizePublicKey |
| `initializeAndStartWifi` | 2891 | WifiTransportManager, d, onDataReceived, w, initialize, startDiscovery, hasAllPermissions, initializeAndStartWifi, onPeerDiscovered, loadSettings |
| `initializeAndStartSwarm` | 2922 | d, catch, startSwarm, ensureLocalIdentityFederation, initializeAndStartSwarm, getSwarmBridge, transport, getIdentityInfo, i, e |
| `ensureLocalIdentityFederation` | 2947 | orEmpty, ensureLocalIdentityFederation, restoreIdentityFromBackup, persistIdentityBackup, cacheIdentityFields, getIdentityInfo, i, grantConsent, isNotEmpty, trim |
| `restoreIdentityFromBackup` | 2983 | catch, apply, w, exportIdentityBackup, commit, edit, restoreIdentityFromBackup, persistIdentityBackup, getString, putString |
| `restoreIdentityFromBackup` | 3000 | apply, d, catch, exportIdentityBackup, w, createNewFile, commit, persistIdentityBackup, lost, putString |
| `persistIdentityBackup` | 3005 | apply, d, catch, exportIdentityBackup, w, createNewFile, commit, persistIdentityBackup, cacheIdentityFields, lost |
| `cacheIdentityFields` | 3035 | apply, d, readCachedIdentityFields, take, IdentityInfo, getString, putLong, remove, contains, exists |
| `readCachedIdentityFields` | 3056 | IdentityInfo, setBleComponents, getString, contains, toULong, getLong, setPlatformBridge, setTransportManager, getBoolean |
| `setPlatformBridge` | 3075 | clear, catch, cancel, w, setBleComponents, setPlatformBridge, stopMeshService, stopNetworkChangeWatch, setTransportManager, stopScanning |
| `stopMeshService` | 3090 | clear, cleanup, catch, cancel, w, stopAdvertising, stop, stopNetworkChangeWatch, stopScanning, stopMonitoring |
| `pauseMeshService` | 3161 | d, service, pause, resumeMeshService, resetStats, i, getStats, resetServiceStats, notifyNetworkRecovered, resume |
| `resumeMeshService` | 3169 | d, flushPendingOutbox, resetStats, i, getStats, resetServiceStats, primeRelayBootstrapConnections, notifyNetworkRecovered, resume |
| `resetServiceStats` | 3177 | d, getServiceState, updateStats, flushPendingOutbox, resetStats, i, getState, getStats, primeRelayBootstrapConnections, notifyNetworkRecovered |
| `notifyNetworkRecovered` | 3188 | getServiceState, updateStats, flushPendingOutbox, coerceAtLeast, toULong, i, getState, getStats, currentTimeMillis, primeRelayBootstrapConnections |
| `getServiceState` | 3200 | ServiceStats, updateStats, coerceAtLeast, toULong, currentTimeMillis, getState, getStats |
| `updateStats` | 3207 | d, ServiceStats, coerceAtLeast, toULong, headless, currentTimeMillis, getStats, peers |
| `startPeriodicStatsUpdate` | 3248 | 256, Hash, delay, updateStats, variants, startPeriodicStatsUpdate, identity_id, format |
| `validateAndStandardizeId` | 3279 | catch, IllegalArgumentException, orEmpty, w, list, take, isSame, canonicalContactId, isBlank, contacts |
| `canonicalContactId` | 3308 | d, catch, w, resolveIdentity, take, formats, canonicalContactId, normalize, public_key_hex, trim |
| `canonicalId` | 3351 | e, canonicalId, isNullOrEmpty, canonicalContactId, Contact, addContact, trim |
| `addContact` | 3354 | trim, canonicalId, isNullOrEmpty, Contact, addContact, e |
| `getContact` | 3397 | get, catch, removeContact, removeConversation, classification, w, canonicalId, getContact, remove, hasConversationWith |
| `hasConversationWith` | 3406 | catch, removeContact, removeConversation, w, canonicalId, isSame, remove, conversation, showing, isNotEmpty |
| `removeContact` | 3415 | removeConversation, removeContact, catch, w, d, canonicalId, isSame, remove, showing, isEmpty |
| `listContacts` | 3446 | d, catch, list, setContactNickname, setNickname, search, peerId, searchContacts, emptyList, blockPeer |
| `searchContacts` | 3450 | d, catch, setContactNickname, setNickname, search, peerId, searchContacts, emptyList, blockPeer, ensureServiceInitializedFireAndForget |
| `setContactNickname` | 3454 | d, catch, setContactNickname, setNickname, peerId, blockPeer, ensureServiceInitializedFireAndForget, i, getContactCount, count |
| `getContactCount` | 3459 | catch, messages, peerId, ensureServiceInitializedFireAndForget, blockPeer, i, getContactCount, count, e, unblockPeer |
| `blockPeer` | 3467 | catch, messages, peerId, ensureServiceInitializedFireAndForget, blockPeer, i, blockAndDeletePeer, e, unblockPeer |
| `unblockPeer` | 3477 | catch, w, messages, isPeerBlocked, isBlocked, peerId, ensureServiceInitializedFireAndForget, i, blockAndDeletePeer, e |
| `blockAndDeletePeer` | 3492 | catch, listBlockedPeers, w, isPeerBlocked, isBlocked, peerId, ensureServiceInitializedFireAndForget, i, emptyList, getBlockedCount |
| `isBlocked` | 3501 | catch, w, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, emptyList, getBlockedCount, blockedCount, listBlockedPeers |
| `listBlockedPeers` | 3511 | catch, w, e, emptyList, ensureServiceInitializedFireAndForget, getBlockedCount, signData, blockedCount, listBlockedPeers |
| `getBlockedCount` | 3521 | catch, w, ensureServiceInitializedFireAndForget, getBlockedCount, signData, verifySignature, blockedCount, e |
| `signData` | 3535 | catch, getSeniorityTimestamp, ensureServiceInitializedFireAndForget, signData, getDeviceId, verifySignature, e |
| `verifySignature` | 3545 | catch, getRegistrationState, getSeniorityTimestamp, ensureServiceInitializedFireAndForget, getDeviceId, verifySignature, e |
| `getDeviceId` | 3559 | getInboxCount, catch, w, exportLogs, getRegistrationState, getDeviceId, getSeniorityTimestamp |
| `getSeniorityTimestamp` | 3563 | getInboxCount, catch, w, exportLogs, getRegistrationState, inboxCount, getSeniorityTimestamp |
| `getRegistrationState` | 3567 | getInboxCount, catch, ID, w, exportLogs, getRegistrationState, updateContactDeviceId, inboxCount |
| `exportLogs` | 3575 | getInboxCount, catch, ID, w, exportLogs, updateContactDeviceId, inboxCount, i, updateDeviceId |
| `getInboxCount` | 3588 | getInboxCount, catch, ID, w, updateContactDeviceId, getIdentityInfoNonBlocking, inboxCount, i, updateDeviceId |
| `updateContactDeviceId` | 3596 | catch, w, updateContactDeviceId, getIdentityInfoNonBlocking, getIdentityInfo, cacheIdentityFields, i, updateDeviceId |
| `getIdentityInfoNonBlocking` | 3615 | d, w, readCachedIdentityFields, ensureLocalIdentityFederation, cacheIdentityFields, identity, getIdentityInfo, ensureServiceInitializedFireAndForget, getState |
| `getIdentityInfo` | 3641 | d, w, ensureLocalIdentityFederation, cacheIdentityFields, setNickname, getIdentityInfo, ensureServiceInitializedFireAndForget, IllegalStateException, trim, isEmpty |
| `setNickname` | 3660 | d, catch, w, e, persistIdentityBackup, setNickname, cacheIdentityFields, getIdentityInfo, i, IllegalStateException |
| `setLocalNickname` | 3705 | withContext, setLocalNickname, catch, resolveIdentity, copy, toULong, i, currentTimeMillis, normalize, toString |
| `sendMessage` | 3723 | withContext, get, catch, trim, e, isNullOrEmpty, isSame, toULong, currentTimeMillis, normalize |
| `dial` | 4018 | withContext, database, catch, backup, contains, isIdentityInitialized, dialPeer, i, file, dial |
| `dialPeer` | 4031 | database, catch, w, backup, restoreIdentityFromBackup, contains, isIdentityInitialized, getIdentityInfo, dialPeer, file |
| `isIdentityInitialized` | 4042 | database, catch, w, restoreIdentityFromBackup, contains, getIdentityInfo, lost, i, getState, File |
| `grantConsent` | 4091 | catch, d, w, initializeAndStartBle, hasRequiredRuntimePermissions, ensureServiceInitializedFireAndForget, i, hasAllPermissions, getState, initializeAndStartWifi |
| `hasRequiredRuntimePermissions` | 4100 | d, catch, w, initializeAndStartSwarm, initializeAndStartBle, hasRequiredRuntimePermissions, hasAllPermissions, getState, initializeAndStartWifi, onRuntimePermissionsGranted |
| `onRuntimePermissionsGranted` | 4104 | withContext, d, catch, w, createIdentity, initializeAndStartSwarm, initializeAndStartBle, getState, initializeAndStartWifi |
| `createIdentity` | 4131 | withContext, d, createIdentity, catch, grantConsent, ensureLocalIdentityFederation, initializeAndStartSwarm, persistIdentityBackup, initializeIdentity, i |
| `ensureServiceInitializedDeferred` | 4172 | d, MeshService, MeshSettings, getState, starting |
| `ensureServiceInitializedFireAndForget` | 4240 | start, w, delay, paths, ensureServiceInitializedFireAndForget, ensureServiceInitializedDeferred, getState, currentTimeMillis, ensureServiceInitialized |
| `ensureServiceInitialized` | 4250 | start, w, flush, delay, checkSelfPermission, add, currentTimeMillis, getState, ensureServiceInitializedDeferred, hasAllPermissions |
| `hasAllPermissions` | 4271 | get, markMessageDelivered, flush, checkSelfPermission, canonicalId, add, getRecentMessages, getMessage, search, addMessage |
| `addMessage` | 4278 | get, markMessageDelivered, clear, flush, canonicalId, add, getRecentMessages, getMessage, search, emptyList |
| `getMessage` | 4282 | get, markMessageDelivered, clear, canonicalId, getRecentMessages, getMessage, search, clearConversation, emptyList, getConversation |
| `getRecentMessages` | 4286 | markMessageDelivered, clear, catch, validateAndStandardizeId, canonicalId, getRecentMessages, search, clearConversation, emptyList, getConversation |
| `getConversation` | 4291 | markMessageDelivered, clear, catch, validateAndStandardizeId, e, canonicalId, search, clearConversation, getConversation, emptyList |
| `searchMessages` | 4295 | markMessageDelivered, clear, catch, validateAndStandardizeId, getHistoryStats, e, search, clearConversation, stats, emptyList |
| `markMessageDelivered` | 4299 | markMessageDelivered, clear, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, removePendingOutbound, clearHistory |
| `clearHistory` | 4304 | clear, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, clearHistory, count, getMessageCount |
| `clearConversation` | 4309 | enforceRetention, catch, validateAndStandardizeId, getHistoryStats, clearConversation, stats, i, count, getMessageCount, e |
| `getHistoryStats` | 4321 | enforceRetention, catch, getHistoryStats, stats, pruneBefore, count, getMessageCount, e, timestamp |
| `getMessageCount` | 4325 | enforceRetention, catch, pruneBefore, count, getMessageCount, e, timestamp |
| `enforceRetention` | 4336 | enforceRetention, resetAllData, catch, cancel, w, clear, pruneBefore, e, timestamp |
| `pruneBefore` | 4349 | resetAllData, clear, catch, cancel, w, flush, shutdown, pruneBefore, stop, e |
| `resetAllData` | 4362 | clear, catch, cancel, w, flush, apply, shutdown, stop, edit |
| `recordConnection` | 4412 | recordConnectionFailure, dialableAddresses, recordConnection, emptyList, recordFailure, replayDiscoveredPeerEvents, getDialableAddresses, trim, isEmpty, isLibp2pPeerId |
| `recordConnectionFailure` | 4416 | normalizePublicKey, dialableAddresses, emptyList, prepopulateDiscoveryNickname, recordFailure, replayDiscoveredPeerEvents, recordConnectionFailure, trim, isEmpty, isLibp2pPeerId |
| `getDialableAddresses` | 4420 | dialableAddresses, emptyList, prepopulateDiscoveryNickname, replayDiscoveredPeerEvents, getDialableAddresses, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `replayDiscoveredPeerEvents` | 4424 | ReplayDiscoveredIdentity, prepopulateDiscoveryNickname, replayDiscoveredPeerEvents, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `getAllKnownTopics` | 4502 | allKnownTopics, getLedgerSummary, catch, w, summary, getConnectionPathState, getServiceStateName, emptyList, getState, getAllKnownTopics |
| `getLedgerSummary` | 4506 | getLedgerSummary, catch, w, summary, getConnectionPathState, getServiceStateName, getDiscoveredPeerCount, getState, getNatStatus |
| `getConnectionPathState` | 4510 | catch, w, getDiscoveredPeerCount, getConnectionPathState, getServiceStateName, getPendingOutboxCount, loadPendingOutbox, getState, getNatStatus |
| `getNatStatus` | 4519 | catch, w, getDiscoveredPeerCount, getServiceStateName, getPendingOutboxCount, loadPendingOutbox, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getState |
| `getServiceStateName` | 4528 | checkSelfPermission, getPendingOutboxCount, loadPendingOutbox, getDiscoveredPeerCount, getServiceStateName, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getState, getMissingRuntimePermissions |
| `getDiscoveredPeerCount` | 4532 | FIX, checkSelfPermission, getPendingOutboxCount, getDiscoveredPeerCount, loadPendingOutbox, getPendingTerminalFailureCode, isBlank, getPendingDeliverySnapshot, getMissingRuntimePermissions |
| `getPendingOutboxCount` | 4536 | FIX, checkSelfPermission, getPendingOutboxCount, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, isBlank, getPendingDeliverySnapshot, getMissingRuntimePermissions |
| `getPendingDeliverySnapshot` | 4540 | withContext, FIX, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, isBlank, getPendingDeliverySnapshot, exportDiagnosticsAsync |
| `getPendingTerminalFailureCode` | 4546 | withContext, catch, FIX, w, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, getPendingTerminalFailureCode, exportDiagnostics, put |
| `getMissingRuntimePermissions` | 4553 | withContext, catch, FIX, w, exportDiagnosticsInternal, checkSelfPermission, loadPendingOutbox, exportDiagnostics, put, JSONObject |
| `exportDiagnosticsAsync` | 4568 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, currentTimeMillis, getState, toString |
| `exportDiagnostics` | 4585 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, getDiscoveryStats, getClientStats |
| `exportDiagnosticsInternal` | 4586 | catch, w, exportDiagnosticsInternal, exportDiagnostics, put, JSONObject, isNullOrBlank, getDiscoveryStats, getClientStats |
| `saveLedger` | 4662 | saveLedger, isNotEmpty, save, asSequence, normalize, isNullOrBlank, normalizeNickname, emitIdentityDiscoveredIfChanged, trim, isEmpty |
| `emitIdentityDiscoveredIfChanged` | 4672 | isNotEmpty, IdentityEmissionSignature, distinct, asSequence, toList, normalize, isNullOrBlank, sorted, normalizeNickname, emitIdentityDiscoveredIfChanged |
| `emitConnectedIfChanged` | 4723 | Connected, emitDisconnectedIfChanged, currentTimeMillis, normalize, emitConnectedIfChanged, isEmpty, emitPeerEvent |
| `emitDisconnectedIfChanged` | 4744 | failed, catch, load, loadSettings, w, currentTimeMillis, normalize, Disconnected, isEmpty, emitPeerEvent |
| `loadSettings` | 4766 | failed, catch, load, w, defaultSettings, MeshSettings, getDefaultSettings, loadSettings |
| `getDefaultSettings` | 4781 | save, defaultSettings, MeshSettings, i, saveSettings |
| `saveSettings` | 4805 | d, save, applyTransportSettings, disableTransport, i, enableTransport, loadSettings, saveSettings |
| `applyTransportSettings` | 4815 | d, disableTransport, enableTransport, loadSettings |
| `validateSettings` | 4863 | catch, w, computeRelayAdjustment, computeAdjustmentProfile, validate, validateSettings, computeProfile, BleAdjustment, computeBleAdjustment |
| `computeAdjustmentProfile` | 4877 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, computeAdjustmentProfile, setRelayBudget, computeProfile, overrideBleInterval, BleAdjustment, computeBleAdjustment |
| `computeBleAdjustment` | 4882 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, setRelayBudget, updateDeviceState, overrideBleInterval, BleAdjustment, computeBleAdjustment |
| `computeRelayAdjustment` | 4891 | RelayAdjustment, overrideBleScanInterval, computeRelayAdjustment, overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, overrideBleInterval, clearAdjustmentOverrides |
| `overrideBleInterval` | 4900 | overrideBleScanInterval, overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, overrideBleInterval, clearAdjustmentOverrides |
| `setRelayBudget` | 4904 | overrideRelayMaxPerHour, clearOverrides, setRelayBudget, overrideRelayMax, updateDeviceState, getTopics, emptyList, clearAdjustmentOverrides |
| `updateDeviceState` | 4908 | overrideRelayMaxPerHour, subscribeTopic, clearOverrides, overrideRelayMax, updateDeviceState, getTopics, emptyList, clearAdjustmentOverrides |
| `overrideRelayMax` | 4912 | catch, w, overrideRelayMaxPerHour, subscribeTopic, clearOverrides, overrideRelayMax, getTopics, emptyList, clearAdjustmentOverrides |
| `clearAdjustmentOverrides` | 4916 | catch, w, unsubscribeTopic, subscribeTopic, clearOverrides, getTopics, emptyList, clearAdjustmentOverrides |
| `getTopics` | 4931 | catch, w, unsubscribeTopic, subscribeTopic, publishTopic |
| `subscribeTopic` | 4936 | catch, w, unsubscribeTopic, subscribeTopic, publishTopic, sendToAllPeers |
| `unsubscribeTopic` | 4943 | catch, w, unsubscribeTopic, buildDialCandidatesForPeer, connectToPeer, publishTopic, sendToAllPeers |
| `publishTopic` | 4951 | catch, w, buildDialCandidatesForPeer, identity_ids, connectToPeer, publishTopic, sendToAllPeers |
| `sendToAllPeers` | 4963 | catch, d, w, buildDialCandidatesForPeer, identity_ids, contains, shouldAttemptDial, connectToPeer, dial, sendToAllPeers |
| `connectToPeer` | 4970 | d, catch, ensurePendingOutboxRetryLoop, buildDialCandidatesForPeer, identity_ids, contains, shouldAttemptDial, connectToPeer, dial, e |
| `ensurePendingOutboxRetryLoop` | 4996 | catch, ensurePendingOutboxRetryLoop, w, load, delay, ensureCoverTrafficLoop, flushPendingOutbox, primeRelayBootstrapConnections |
| `ensureCoverTrafficLoop` | 5019 | catch, d, load, w, delay, attemptDirectSwarmDelivery, prepareCoverTraffic, sendToAllPeers |
| `attemptDirectSwarmDelivery` | 5048 | isNotBlank, logDeliveryAttempt, currentTimeMillis, attemptDirectSwarmDelivery, isNullOrBlank, firstOrNull |
| `awaitPeerConnection` | 5695 | isEmpty, catch, d, delay, flushPendingOutbox, loadPendingOutbox, lock, awaitPeerConnection, getPeers, outbox |
| `flushPendingOutbox` | 5710 | logDeliveryState, outbox, yield, hasNext, primeRelayBootstrapConnections, d, flushPendingOutbox, loadPendingOutbox, lock, currentTimeMillis |
| `enqueuePendingOutbound` | 5906 | isMessageDeliveredLocally, logDeliveryState, enqueuePendingOutbound, add, loadPendingOutbox, PendingOutboundEnvelope, currentTimeMillis, toMutableList, toString, randomUUID |
| `loadPendingOutboxAsync` | 5970 | PendingOutboundEnvelope, isBlank, emptyList, optJSONObject, orEmpty, randomUUID, readText, add, currentTimeMillis, optString |
| `loadPendingOutboxSync` | 6029 | PendingOutboundEnvelope, isBlank, emptyList, optJSONObject, has, orEmpty, randomUUID, readText, optLong, add |
| `loadPendingOutbox` | 6074 | catch, savePendingOutbox, w, writeText, put, JSONObject, toString, JSONArray |
| `savePendingOutbox` | 6077 | catch, w, writeText, pendingOutboxExpiryReason, Suppress, put, JSONObject, toString, JSONArray |
| `pendingOutboxExpiryReason` | 6107 | catch, d, orEmpty, list, emptyList, resolveCanonicalPeerId, resolveIdentity, normalizePublicKey |
| `resolveCanonicalPeerId` | 6116 | catch, d, orEmpty, list, isSame, emptyList, resolveCanonicalPeerId, resolveIdentity, normalizePublicKey |
| `resolveCanonicalPeerIdFromMessageHints` | 6200 | catch, isBootstrapRelayPeer, orEmpty, resolveCanonicalPeerIdFromMessageHints, list, isSame, first, emptyList, normalize, isNotEmpty |
| `encodeMessageWithIdentityHints` | 6237 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `encodeIdentitySyncPayload` | 6241 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `encodeMeshMessagePayload` | 6245 | normalizeOutboundListenerHints, orEmpty, encodeMeshMessagePayload, take, distinct, put, getIdentityInfo, normalizeExternalAddressHints, getListeningAddresses, getExternalAddresses |
| `decodeMessageWithIdentityHints` | 6280 | isNotBlank, DecodedMessagePayload, jsonArrayToStringList, decodeMessageWithIdentityHints, JSONObject, normalizeNickname, startsWith, optJSONObject, optJSONArray, optString |
| `jsonArrayToStringList` | 6320 | isSyntheticFallbackNickname, distinct, add, jsonArrayToStringList, lowercase, length, startsWith, emptyList, optString, selectAuthoritativeNickname |
| `normalizePublicKey` | 6330 | isSyntheticFallbackNickname, lowercase, startsWith, selectAuthoritativeNickname, normalizeNickname, isNotEmpty, trim, normalizePublicKey |
| `normalizeNickname` | 6337 | isSyntheticFallbackNickname, lowercase, isBlePeerId, startsWith, selectAuthoritativeNickname, normalizeNickname, isNotEmpty, trim |
| `isSyntheticFallbackNickname` | 6341 | fromString, isSyntheticFallbackNickname, trim, isBlePeerId, startsWith, isWifiPeerId, selectAuthoritativeNickname, normalizeNickname, lowercase |
| `selectAuthoritativeNickname` | 6348 | fromString, Regex, isSyntheticFallbackNickname, isBlePeerId, isWifiPeerId, matches, selectAuthoritativeNickname, normalizeNickname, trim, isEmpty |
| `isBlePeerId` | 6366 | isIdentityId, fromString, Regex, selectCanonicalPeerId, isBlePeerId, isWifiPeerId, matches, trim, isEmpty, isLibp2pPeerId |
| `isWifiPeerId` | 6370 | isIdentityId, Regex, selectCanonicalPeerId, isBlePeerId, isWifiPeerId, matches, trim, isEmpty, isLibp2pPeerId |
| `selectCanonicalPeerId` | 6378 | isIdentityId, selectCanonicalPeerId, isBlePeerId, prepopulateDiscoveryNickname, normalizeNickname, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `prepopulateDiscoveryNickname` | 6400 | catch, list, startsWith, emptyList, takeLast, isNullOrBlank, selectAuthoritativeNickname, normalizeNickname, orEmpty, prepopulateDiscoveryNickname |
| `resolveKnownPeerNickname` | 6438 | isNotBlank, asSequence, dialableAddresses, isNullOrBlank, normalizeNickname, firstOrNull, trim, resolveKnownPeerNickname, normalizePublicKey |
| `annotateIdentityInLedger` | 6493 | d, orEmpty, annotateIdentityInLedger, annotateIdentity, buildDialCandidatesForPeer, isNotEmpty, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `appendRoutingHint` | 6525 | d, orEmpty, add, appendRoutingHint, startsWith, split, toMutableList, isNotEmpty, trim, isEmpty |
| `storeLastKnownRoutePeerId` | 6556 | get, d, catch, w, indexOf, add, appendRoutingHint, copy, mergeNotes, split |
| `mergeNotes` | 6575 | resolveTransportIdentity, indexOf, substring, split, isNullOrBlank, isNotEmpty, trim, isLibp2pPeerId, joinToString |
| `resolveTransportIdentity` | 6603 | d, catch, isBootstrapRelayPeer, resolveTransportIdentity, list, getIdentityInfo, extractPublicKeyFromPeerId, orEmpty, isLibp2pPeerId, normalizePublicKey |
| `persistRouteHintsForTransportPeer` | 6691 | catch, normalizeOutboundListenerHints, d, parseRoutingHints, list, isBlank, extractPublicKeyFromPeerId, persistRouteHintsForTransportPeer, orEmpty, normalizePublicKey |
| `upsertFederatedContact` | 6762 | catch, d, isBootstrapRelayPeer, orEmpty, list, isNullOrBlank, isNotEmpty, trim, isEmpty, normalizePublicKey |
| `upsertRoutingListeners` | 6859 | savePendingOutbox, isBlank, isNullOrBlank, removePendingOutbound, orEmpty, loadPendingOutbox, currentTimeMillis, promotePendingOutboundForPeer, upsertRoutingListeners, joinToString |
| `removePendingOutbound` | 6871 | promotePendingOutboundForPeer, savePendingOutbox, logDeliveryState, copy, loadPendingOutbox, isBlank, toMutableList, currentTimeMillis, isNullOrBlank, trim |
| `promotePendingOutboundForPeer` | 6879 | isMessageDeliveredLocally, savePendingOutbox, containsKey, logDeliveryState, copy, loadPendingOutbox, currentTimeMillis, toMutableList, pruneDeliveredReceiptCache, isNullOrBlank |
| `isMessageDeliveredLocally` | 6903 | get, isMessageDeliveredLocally, catch, containsKey, remove, markDeliveredReceiptSeen, pruneDeliveredReceiptCache, currentTimeMillis, putIfAbsent |
| `markDeliveredReceiptSeen` | 6917 | clear, parseRoutingHints, take, putAll, isNullOrEmpty, remove, pruneDeliveredReceiptCache, currentTimeMillis, RoutingHints, emptyList |
| `pruneDeliveredReceiptCache` | 6922 | clear, parseRoutingHints, take, putAll, isNullOrEmpty, remove, pruneDeliveredReceiptCache, currentTimeMillis, RoutingHints, emptyList |
| `parseRoutingHints` | 6939 | parseRoutingHints, removePrefix, isNullOrEmpty, startsWith, emptyList, RoutingHints, split, isNotEmpty, trim |
| `parseAllRoutingPeerIds` | 6984 | distinct, removePrefix, add, buildRoutePeerCandidates, startsWith, emptyList, parseAllRoutingPeerIds, split, isNullOrBlank, parseLastKnownRoute |
| `parseLastKnownRoute` | 7003 | discoverRoutePeersForPublicKey, addAll, removePrefix, add, buildRoutePeerCandidates, startsWith, split, parseAllRoutingPeerIds, isNullOrBlank, isNotEmpty |
| `buildRoutePeerCandidates` | 7013 | discoverRoutePeersForPublicKey, addAll, add, lastOrNull, buildRoutePeerCandidates, parseAllRoutingPeerIds, isNullOrBlank, asReversed, trim, isEmpty |
| `discoverRoutePeersForPublicKey` | 7083 | discoverRoutePeersForPublicKey, orEmpty, asSequence, toList, dialableAddresses, emptyList, isNotEmpty, trim, isEmpty, isLibp2pPeerId |
| `routeCandidateMatchesRecipient` | 7117 | routeCandidateMatchesRecipient, catch, isKnownRelay, dialableAddresses, extractPublicKeyFromPeerId, emptyList, trim, isEmpty, isLibp2pPeerId, normalizePublicKey |
| `buildDialCandidatesForPeer` | 7151 | getDialHintsForRoutePeer, normalizeAddressHint, distinct, take, buildDialCandidatesForPeer, dialableAddresses, emptyList, relayCircuitAddressesForPeer, isNullOrBlank, prioritizeAddressesForCurrentNetwork |
| `getDialHintsForRoutePeer` | 7171 | getDialHintsForRoutePeer, normalizeOutboundListenerHints, normalizeAddressHint, distinct, buildDialCandidatesForPeer, dialableAddresses, normalizeExternalAddressHints, contains, emptyList, getLocalIpAddress |
| `normalizeOutboundListenerHints` | 7183 | normalizeOutboundListenerHints, normalizeAddressHint, distinct, contains, normalizeExternalAddressHints, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, trim |
| `normalizeExternalAddressHints` | 7189 | normalizeAddressHint, distinct, contains, normalizeExternalAddressHints, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, trim, isEmpty |
| `normalizeAddressHint` | 7195 | normalizeAddressHint, removePrefix, substring, contains, startsWith, isDialableAddress, toMultiaddrFromSocketAddress, getLocalIpAddress, removeSuffix, lastIndexOf |
| `toMultiaddrFromSocketAddress` | 7216 | extractIpv4FromMultiaddr, toIntOrNull, Regex, isSpecialUseIpv4, isSameLanAddress, isPrivateIpv4, removePrefix, substring, contains, startsWith |
| `isDialableAddress` | 7236 | extractIpv4FromMultiaddr, toIntOrNull, isSpecialUseIpv4, isSameLanAddress, isPrivateIpv4, parseIpv4Octets, contains, isDialableAddress, split |
| `parseIpv4Octets` | 7249 | toIntOrNull, isSpecialUseIpv4, isPrivateIpv4, parseIpv4Octets, split |
| `isPrivateIpv4` | 7256 | isBootstrapRelayPeer, isSpecialUseIpv4, isKnownRelay, isPrivateIpv4, parseIpv4Octets, trim, equals |
| `isSpecialUseIpv4` | 7263 | isBootstrapRelayPeer, isSpecialUseIpv4, isKnownRelay, parseIpv4Octets, emptyList, relayCircuitAddressesForPeer, trim, isLibp2pPeerId, equals |
| `isKnownRelay` | 7280 | closed, d, Relays, isBootstrapRelayPeer, isKnownRelay, parseBootstrapRelay, isCircuitOpen, emptyList, relayCircuitAddressesForPeer, getHealthyRelays |
| `relayCircuitAddressesForPeer` | 7289 | closed, d, Relays, add, parseBootstrapRelay, isCircuitOpen, emptyList, relayCircuitAddressesForPeer, getHealthyRelays, toSet |
| `parseBootstrapRelay` | 7331 | isBootstrapRelayPeer, trimEnd, parseBootstrapRelay, substring, isBlank, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, lastIndexOf, trim, isEmpty |
| `isBootstrapRelayPeer` | 7341 | isBootstrapRelayPeer, createEmergencyContact, isBlank, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey |
| `isBootstrapRelayPeerFromKey` | 7352 | isBootstrapRelayPeer, createEmergencyContact, toULong, isBlank, extractPeerIdFromPublicKey, Contact, currentTimeMillis, normalize |
| `createEmergencyContact` | 7369 | catch, take, add, toULong, extractPeerIdFromPublicKey, Contact, currentTimeMillis, i, normalize, e |
| `validatePeerBeforeContactCreation` | 7408 | d, w, isBootstrapRelayPeer, take, isBlank, isValidPublicKey, isValidPeerId |
| `logIdentityResolutionDetails` | 7440 | d, take, WebSocket, TCP, currentTimeMillis, primeRelayBootstrapConnections |
| `primeRelayBootstrapConnections` | 7460 | d, allowRequest, shouldAttemptDial, currentTimeMillis, i, dial, getTransportPriority |
| `primeRelayBootstrapConnectionsLegacy` | 7522 | extractIpv4FromMultiaddr, catch, d, isSameLanAddress, distinct, shouldAttemptDial, currentTimeMillis, dial, getLocalIpAddress, sameSubnet24 |
| `prioritizeAddressesForCurrentNetwork` | 7537 | extractIpv4FromMultiaddr, isSameLanAddress, distinct, indexOf, substring, split, getLocalIpAddress, sameSubnet24, prioritizeAddressesForCurrentNetwork, isEmpty |
| `isSameLanAddress` | 7544 | extractIpv4FromMultiaddr, isSameLanAddress, indexOf, substring, split, getLocalIpAddress, sameSubnet24, ports |
| `extractIpv4FromMultiaddr` | 7550 | extractIpv4FromMultiaddr, racingBootstrapWithFallback, indexOf, substring, split, i, getTransportPriority, sameSubnet24, ports, joinToString |
| `sameSubnet24` | 7559 | racingBootstrapWithFallback, getOpenCircuits, network, isNotEmpty, split, i, one, getTransportPriority, sameSubnet24, ports |
| `racingBootstrapWithFallback` | 7576 | getOpenCircuits, network, resetAll, listOf, probePorts, i, getTransportPriority, isNotEmpty, one, getNetworkDiagnostics |
| `attemptMdnsFallback` | 7676 | withTimeoutOrNull, delay, peer, i, MdnsFallback, getStats, toInt, primeRelayBootstrapConnectionsLegacy, e |
| `bootstrapWithFallbackStrategy` | 7724 | failed, racingBootstrapWithFallback, cancel, i, startNetworkChangeWatch, currentTimeMillis, s, e |
| `startNetworkChangeWatch` | 7747 | cancel, w, ago, coerceAtMost, minOf, currentTimeMillis, i, s, startNetworkChangeWatch |
| `stopNetworkChangeWatch` | 7791 | cancel, extractPortFromMultiaddr, classifyBootstrapError, stopNetworkChangeWatch, setOf |
| `classifyBootstrapError` | 7801 | extractPortFromMultiaddr, host, unreachable, setOf |
| `extractPortFromMultiaddr` | 7839 | get, toIntOrNull, Regex, coerceAtMost, shouldAttemptDial, currentTimeMillis, trim, isEmpty, find |
| `shouldAttemptDial` | 7846 | coerceAtMost, shouldAttemptDial, currentTimeMillis, to, trim, isEmpty |
| `getPreferredRelay` | 7878 | performMaintenance, StatFs, getNetworkFailureSummary, startStorageMaintenance, getPreferredRelays, getNetworkDiagnosticsSnapshot, detectAndRecoverMessageTracking, updateDiskStats, toULong, getPreferredRelay |
| `getNetworkFailureSummary` | 7885 | logRetryStormDetection, performMaintenance, d, catch, StatFs, detectAndRecoverMessageTracking, getNetworkDiagnosticsSnapshot, updateDiskStats, getSummary, toULong |
| `getNetworkDiagnosticsSnapshot` | 7890 | logRetryStormDetection, performMaintenance, d, catch, StatFs, w, delay, detectAndRecoverMessageTracking, updateDiskStats, toULong |
| `startStorageMaintenance` | 7893 | logRetryStormDetection, performMaintenance, d, catch, StatFs, w, delay, detectAndRecoverMessageTracking, updateDiskStats, toULong |
| `getExternalAddresses` | 7927 | getListeners, getNetworkInterfaces, orEmpty, hasMoreElements, getExternalAddresses, getListeningAddresses, emptyList, getLocalIpAddress, nextElement, addresses |
| `getListeningAddresses` | 7937 | getListeners, getNetworkInterfaces, orEmpty, trim, isSpecialUseIpv4, isPrivateIpv4, hasMoreElements, startsWith, emptyList, getLocalIpAddress |
| `getLocalIpAddress` | 7940 | orEmpty, getNetworkInterfaces, isSpecialUseIpv4, trim, isPrivateIpv4, hasMoreElements, startsWith, getLocalIpAddress, nextElement, lowercase |
| `getIdentityExportString` | 7983 | getListeningAddresses, getExternalAddresses, libp2p_peer_id, getLocalIpAddress, replace, getIdentityInfo, JSONObject, getIdentityExportString, JSONArray, normalizeOutboundListenerHints |
| `observeIncomingMessages` | 8021 | withContext, observePeers, emit, observeNetworkStats, distinct, dialableAddresses, emptyList, getStats |
| `observePeers` | 8028 | withContext, emit, observeNetworkStats, delay, distinct, dialableAddresses, emptyList, getStats |
| `observeNetworkStats` | 8043 | withContext, cleanup, emit, cancel, saveLedger, clear, delay, getStats, stopMeshService |
| `cleanup` | 8060 | cleanup, clear, catch, cancel, saveLedger, i, stopMeshService, e |
| `getDiagnosticsLogPath` | 8093 | catch, readLines, writeText, getDiagnosticsLogs, i, getDiagnosticsLogPath, File, exists, takeLast, clearDiagnosticsLogs |
| `getDiagnosticsLogs` | 8100 | catch, readLines, writeText, logDeliveryState, isBlank, i, getDiagnosticsLogPath, File, exists, takeLast |
| `clearDiagnosticsLogs` | 8117 | catch, isNotBlank, w, unknown, writeText, logDeliveryState, logDeliveryAttempt, isBlank, i, getDiagnosticsLogPath |
| `logDeliveryState` | 8128 | isNotBlank, w, unknown, logDeliveryState, logDeliveryAttempt, isBlank, i |
| `logDeliveryAttempt` | 8133 | isNotBlank, w, unknown, logDeliveryAttempt, migrateToCanonicalIds, i |
| `migrateToCanonicalIds` | 8160 | get, catch, w, list, add, getSharedPreferences, copy, i, getBoolean, resolveIdentity |
| `emergencyContactRecovery` | 8250 | get, catch, w, distinct, isNullOrEmpty, extractPublicKeyFromPeerId, recent, e |
| `detectAndRepairCorruption` | 8318 | d, list, emergencyContactRecovery, backupCorruptedDatabase, stats, i, toInt, e |
| `backupCorruptedDatabase` | 8364 | mkdirs, catch, listFiles, copyTo, currentTimeMillis, File, i, exists, e |
| `handleBleTransportDegradation` | 8406 | recordSuccess, w, setBackgroundMode, recordTransportEvent, clearPeerCache, attemptBleRecovery, handleBleTransportDegradation, isDegraded, recordFailure, getHealth |
| `recordTransportEvent` | 8422 | recordSuccess, attemptBleRecovery, handleBleTransportDegradation, getActiveTransports, getTransportHealthSummary, isDegraded, emptyList, recordFailure, getSummary |
| `getTransportHealthSummary` | 8437 | d, handleBleFailure, attemptBleRecovery, getActiveTransports, getTransportHealthSummary, emptyList, getSummary |
| `getActiveTransports` | 8445 | d, handleBleFailure, attemptBleRecovery, getActiveTransports, emptyList, forceRestartScanning |
| `handleBleFailure` | 8454 | d, handleBleFailure, attemptBleRecovery, clearPeerCache, forceRestartScanning |
| `attemptBleRecovery` | 8464 | transportTypeFromValue, d, attemptBleRecovery, clearPeerCache, forceRestartScanning |
| `forceRestartScanning` | 8473 | transportTypeFromValue, d, getAvailableTransports, getSmartAvailableTransports, emptyList, clearPeerCache, forceRestartScanning, fromValue |
| `clearPeerCache` | 8482 | transportTypeFromValue, d, getAvailableTransports, getAvailableTransportsSorted, getSmartAvailableTransports, emptyList, clearPeerCache, fromValue |
| `transportTypeFromValue` | 8494 | getAvailableTransports, getAvailableTransportsSorted, getSmartAvailableTransports, emptyList, fromValue, getDedupStats |
| `getSmartAvailableTransports` | 8502 | getAvailableTransports, getSubscribedTopicsList, getAvailableTransportsSorted, emptyList, getDedupStats |
| `getAvailableTransportsSorted` | 8510 | getSubscribedTopicsList, getAvailableTransportsSorted, getKnownTopicsList, emptyList, getDedupStats |
| `getDedupStats` | 8518 | filterMessagesByTopic, getSubscribedTopicsList, getKnownTopicsList, emptyList, getDedupStats |
| `getSubscribedTopicsList` | 8526 | filterMessagesByTopic, getSubscribedTopicsList, getKnownTopicsList, emptyList, enableTransport |
| `getKnownTopicsList` | 8534 | filterMessagesByTopic, startAllTransports, getKnownTopicsList, emptyList, enableTransport, startAll |
| `filterMessagesByTopic` | 8542 | filterMessagesByTopic, startAllTransports, enableTransport, shouldUseTransport, startAll |
| `enableTransport` | 8550 | getBleQuotaCount, startAllTransports, enableTransport, shouldUseTransport, startAll |
| `startAllTransports` | 8558 | isPortLikelyBlocked, getBleQuotaCount, shouldUseTransport, startAll |
| `shouldUseTransport` | 8566 | getBleQuotaCount, getNetworkStateLogString, isPortLikelyBlocked, toLogString, shouldUseTransport, getNetworkDiagnostics |
| `getBleQuotaCount` | 8574 | getBleQuotaCount, getNetworkStateLogString, isPortLikelyBlocked, toLogString, getHealthyRelays, getNetworkDiagnostics |
| `isPortLikelyBlocked` | 8582 | getNetworkStateLogString, isPortLikelyBlocked, toLogString, getHealthyRelays, getLastFailure, getNetworkDiagnostics |
| `getNetworkStateLogString` | 8590 | toLogString, getHealthyRelays, getLastFailure, getNetworkDiagnostics, getLastFailureReason |
| `getHealthyRelays` | 8598 | getOpenCircuits, getOpenCircuitCount, getHealthyRelays, getLastFailure, getLastFailureReason |
| `getLastFailure` | 8606 | getOpenCircuits, catch, getOpenCircuitCount, formatReportForUser, generateReport, formatDiagnosticsReportForUser, getLastFailure, e, getLastFailureReason |
| `getLastFailureReason` | 8614 | getOpenCircuits, catch, getOpenCircuitCount, formatReportForUser, generateReport, formatDiagnosticsReportForUser, hasDnsFailures, e, getLastFailureReason |
| `getOpenCircuitCount` | 8622 | hasPortBlocking, getOpenCircuits, catch, formatReportForUser, generateReport, formatDiagnosticsReportForUser, hasDnsFailures, e |
| `formatDiagnosticsReportForUser` | 8630 | hasPortBlocking, catch, formatReportForUser, generateReport, hasDnsFailures, e |
| `hasDnsFailures` | 8644 | hasDnsFailures, hasPortBlocking |
| `hasPortBlocking` | 8652 | hasPortBlocking |
| `getBootstrapNodesForSettings` | 81 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapToSmartTransportType, trim, split, getenv, getBootstrapNodes, unavailable |
| `getBootstrapNodes` | 85 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapToSmartTransportType, trim, split, getenv, getBootstrapNodes, unavailable |
| `getBootstrapNodes` | 90 | isMeshParticipationEnabled, emptyList, isNotEmpty, mapFromSmartTransportType, mapToSmartTransportType, trim, split, getenv, unavailable |
| `isMeshParticipationEnabled` | 95 | isMeshParticipationEnabled, mapToSmartTransportType, unavailable, mapFromSmartTransportType |
| `mapToSmartTransportType` | 104 | isMeshParticipationEnabled, mapFromSmartTransportType, isEnabledFlag, IllegalStateException, requireMeshParticipationEnabled |
| `mapFromSmartTransportType` | 117 | isMeshParticipationEnabled, lowercase, trim, isEnabledFlag, LocalTransportFallbackResult, IllegalStateException, requireMeshParticipationEnabled |
| `requireMeshParticipationEnabled` | 125 | isMeshParticipationEnabled, lowercase, trim, isEnabledFlag, LocalTransportFallbackResult, attemptWifiThenBleFallback, IllegalStateException, get, requireMeshParticipationEnabled |
| `isEnabledFlag` | 133 | lowercase, isNotEmpty, trim, LocalTransportFallbackResult, isEnabledFlag, attemptWifiThenBleFallback, tryWifi, get |
| `attemptWifiThenBleFallback` | 150 | isNotEmpty, trim, LocalTransportFallbackResult, tryBle, attemptWifiThenBleFallback, tryWifi |
| `getAvailableStorageMB` | 190 | triggering, recordFailure, mapToSmartTransportType, enhanceNetworkErrorLogging, getAvailableStorageMB, checkAndRecordMessage, Triple, classifyBootstrapError, trackNetworkFailure, w |
| `checkAndRecordMessage` | 198 | triggering, mapFromSmartTransportType, mapToSmartTransportType, enhanceNetworkErrorLogging, recordFailure, isNodeUnreachable, Triple, classifyBootstrapError, trackNetworkFailure, w |
| `enhanceNetworkErrorLogging` | 204 | triggering, recordFailure, enhanceNetworkErrorLogging, isNodeUnreachable, compareAndSet, classifyBootstrapError, trackNetworkFailure, w, triggerFallbackProtocol, get |
| `trackNetworkFailure` | 210 | triggering, isNotEmpty, recordFailure, isNodeUnreachable, compareAndSet, trackNetworkFailure, w, triggerFallbackProtocol, get |
| `triggerFallbackProtocol` | 231 | recursion, isNotEmpty, allowRequest, set, dial, catch, compareAndSet, i, w |
| `isCorrupted` | 527 | currentTimeMillis, recordFailure, markCorrupted, recordSuccess |
| `markCorrupted` | 534 | currentTimeMillis, recordFailure, recordSuccess |
| `recordSuccess` | 541 | currentTimeMillis, MessageTracking, forMessage, recordFailure |
| `recordFailure` | 553 | currentTimeMillis, MessageTracking, recoverFromCorruption, forMessage |
| `forMessage` | 567 | currentTimeMillis, MessageTracking, recoverFromCorruption |
| `recoverFromCorruption` | 574 | currentTimeMillis, MessageTracking, getenv, isEnabledFlag |
| `isTerminalIdentityFailure` | 606 | isTerminalIdentityFailure, markCorrupted, trim, markMessageCorrupted, w, terminalIdentityFailureMessage |
| `terminalIdentityFailureMessage` | 614 | terminalIdentityFailureMessage, markCorrupted, getMessageIdTracking, trim, markMessageCorrupted, w |
| `markMessageCorrupted` | 633 | add, markCorrupted, detectAndRecoverMessageTracking, forMessage, messageId, w, isCorrupted, getMessageIdTracking |
| `getMessageIdTracking` | 643 | add, detectAndRecoverMessageTracking, i, forMessage, messageId, w, recoverFromCorruption, isCorrupted |
| `detectAndRecoverMessageTracking` | 657 | isNotEmpty, incrementAttemptCount, add, i, messageId, w, recoverFromCorruption, isCorrupted, getMessageIdTracking |
| `incrementAttemptCount` | 685 | recordFailure, getMessageIdTracking, shouldRetryMessage, getRetryDelay |
| `getRetryDelay` | 695 | d, logMessageDeliveryAttempt, shouldRetryMessage, logRetryStormDetection, storms, getMessageIdTracking |
| `shouldRetryMessage` | 709 | d, logMessageDeliveryAttempt, checkReinstallState, count, storms, logRetryStormDetection, w, enabled, getMessageIdTracking |
| `logMessageDeliveryAttempt` | 717 | d, onCreate, thread, checkReinstallState, initializeManagers, count, storms, logRetryStormDetection, w, enabled |
| `logRetryStormDetection` | 724 | d, onCreate, thread, initializeRepository, checkReinstallState, catch, count, i, initializeManagers, startStorageMaintenance |
| `initializeRepository` | 748 | d, catch, checkReinstallState, File, i, exists, startStorageMaintenance, w, contains |
| `checkReinstallState` | 756 | d, checkReinstallState, FIX, File, i, exists, initializeManagers, w, contains |
| `initializeManagers` | 782 | migrateContactsFromOldLocation, enforceRetention, w, currentTimeMillis, catch, loop, FIX, count, i, toULong |
| `verifyContactDataIntegrity` | 872 | d, exist, isNullOrEmpty, catch, diagnostics, minOf, contacts, orEmpty, i, list |
| `migrateContactsFromOldLocation` | 916 | getBoolean, d, edit, putBoolean, length, apply, File, i, exists, getSharedPreferences |
| `migrateStaleRoutingHints` | 1002 | getBoolean, Contact, add, trim, split, joinToString, orEmpty, startsWith, list, getSharedPreferences |
| `migrateTruncatedPublicKeys` | 1057 | getBoolean, trim, orEmpty, startsWith, list, take, key, getSharedPreferences, w |
| `testLedgerRelayConnectivity` | 1122 | d, emptyList, connect, indexOf, close, catch, split, InetSocketAddress, getPreferredRelays, isEmpty |
| `startMeshService` | 1166 | d, withStorageAndLogs, currentTimeMillis, i, getCore, start, e, getState |
| `onPeerDiscovered` | 1229 | d, getIdentityInfo, PeerDiscoveryInfo, extractPublicKeyFromPeerId, prepopulateDiscoveryNickname, catch, isBootstrapRelayPeer, isNullOrBlank, resolveTransportIdentity |
| `onPeerIdentified` | 1318 | d, isNotEmpty, currentTimeMillis, listOf, trim, peerId, joinToString, sorted, onPeerIdentified, recordTransportEvent |
| `onPeerDisconnected` | 1546 | d, currentTimeMillis, remove, aliases, pruneDisconnectedPeer, trim, emitDisconnectedIfChanged, onPeerDisconnected, recordTransportEvent |
| `onMessageReceived` | 1581 | load, disabled, onMessageReceived, i, detected, enabled, logDeliveryAttempt, checkAndRecordMessage |
| `onReceiptReceived` | 1979 | lowercase, d, loadPendingOutbox, catch, trim, logDeliveryState, onReceiptReceived, get, removePendingOutbound |
| `sendDeliveryReceiptAsync` | 2113 | d, sendDeliveryReceiptAsync, emptyList, catch, blocked, trim, launch, i, prepareReceipt, senderId |
| `sendIdentitySyncIfNeeded` | 2221 | d, encodeIdentitySyncPayload, prepareMessageWithId, sendIdentitySyncIfNeeded, normalizePublicKey, add, remove, catch, trim, isBootstrapRelayPeer |
| `sendHistorySyncIfNeeded` | 2282 | getIdentityInfo, currentTimeMillis, catch, sendHistorySyncIfNeeded, trim, isBootstrapRelayPeer, isEmpty, w |
| `sendHistorySyncDataIfNeeded` | 2349 | d, withIndex, buildRoutePeerCandidates, distinct, launch, parseRoutingHints, get, put, w, putIfAbsent |
| `initializeAndStartBle` | 2411 | d, onDataReceived, hasAllPermissions, BleGattClient, onPeerDiscovered, loadSettings, onPeerIdentityRead, BleScanner, w, noteBleRouteObservation |
| `updateBleIdentityBeacon` | 2491 | emptyList, isNullOrEmpty, getIdentityInfo, setIdentityBeaconInternal, identity, currentTimeMillis, launch, delay, updateBleIdentityBeacon, isEmpty |
| `setIdentityBeaconInternal` | 2522 | toString, setIdentityBeaconInternal, JSONArray, toByteArray, normalizeExternalAddressHints, distinct, normalizeOutboundListenerHints, put, take, libp2p_peer_id |
| `buildBeacon` | 2530 | toString, emptyList, JSONArray, toByteArray, distinct, put, take, libp2p_peer_id, buildBeacon, JSONObject |
| `onPeerIdentityRead` | 2600 | normalizePublicKey, isNotBlank, trim, getString, optString, isNullOrBlank, noteBleRouteObservation, toString, w, optJSONArray |
| `updateDiscoveredPeer` | 2782 | normalizeNickname, selectCanonicalPeerId, maxOf, updateDiscoveredPeer, selectAuthoritativeNickname, copy, normalize |
| `noteBleRouteObservation` | 2821 | asSequence, isNotEmpty, currentTimeMillis, BleRouteObservation, trim, resolveFreshBlePeerId, orEmpty, fallback, isEmpty, noteBleRouteObservation |
| `resolveFreshBlePeerId` | 2835 | asSequence, d, isNotEmpty, currentTimeMillis, remove, candidate, trim, resolveFreshBlePeerId, fallback, isEmpty |
| `pruneDisconnectedPeer` | 2868 | d, normalizePublicKey, pruneDisconnectedPeer, trim, loadSettings, isEmpty, initializeAndStartWifi |
| `initializeAndStartWifi` | 2891 | d, onDataReceived, initialize, hasAllPermissions, WifiTransportManager, onPeerDiscovered, startDiscovery, loadSettings, initializeAndStartWifi, w |
| `initializeAndStartSwarm` | 2922 | d, transport, getIdentityInfo, catch, i, initializeAndStartSwarm, updateBleIdentityBeacon, loadSettings, ensureLocalIdentityFederation, getSwarmBridge |
| `ensureLocalIdentityFederation` | 2947 | grantConsent, cacheIdentityFields, getIdentityInfo, isNotEmpty, restoreIdentityFromBackup, trim, i, orEmpty, persistIdentityBackup, ensureLocalIdentityFederation |
| `restoreIdentityFromBackup` | 2983 | completes, exportIdentityBackup, edit, importIdentityBackup, putString, catch, restoreIdentityFromBackup, getString, commit, apply |
| `restoreIdentityFromBackup` | 3000 | completes, exportIdentityBackup, d, edit, importIdentityBackup, putString, createNewFile, catch, commit, apply |
| `persistIdentityBackup` | 3005 | completes, exportIdentityBackup, d, edit, cacheIdentityFields, putString, createNewFile, catch, apply, commit |
| `cacheIdentityFields` | 3035 | d, getBoolean, edit, putString, remove, putBoolean, putLong, apply, getString, toLong |
| `readCachedIdentityFields` | 3056 | getBoolean, setPlatformBridge, getString, setTransportManager, toULong, getLong, setBleComponents, IdentityInfo, contains |
| `setPlatformBridge` | 3075 | stopScanning, w, stopNetworkChangeWatch, stopMonitoring, setPlatformBridge, catch, stopMeshService, setTransportManager, setBleComponents, clear |
| `stopMeshService` | 3090 | stopScanning, stop, cleanup, w, stopNetworkChangeWatch, stopMonitoring, catch, stopAdvertising, clear, cancel |
| `pauseMeshService` | 3161 | d, notifyNetworkRecovered, resume, resumeMeshService, getStats, resetStats, i, service, resetServiceStats, pause |
| `resumeMeshService` | 3169 | d, notifyNetworkRecovered, resume, getStats, resetStats, flushPendingOutbox, i, resetServiceStats, primeRelayBootstrapConnections |
| `resetServiceStats` | 3177 | d, notifyNetworkRecovered, getStats, resetStats, flushPendingOutbox, i, updateStats, getServiceState, getState, primeRelayBootstrapConnections |
| `notifyNetworkRecovered` | 3188 | coerceAtLeast, currentTimeMillis, getStats, flushPendingOutbox, i, toULong, updateStats, getServiceState, getState, primeRelayBootstrapConnections |
| `getServiceState` | 3200 | coerceAtLeast, currentTimeMillis, getStats, ServiceStats, toULong, updateStats, getState |
| `updateStats` | 3207 | coerceAtLeast, d, peers, currentTimeMillis, getStats, headless, toULong, ServiceStats |
| `startPeriodicStatsUpdate` | 3248 | format, delay, Hash, 256, startPeriodicStatsUpdate, updateStats, identity_id, variants |
| `validateAndStandardizeId` | 3279 | isSame, canonicalContactId, catch, trim, IllegalArgumentException, isBlank, contacts, orEmpty, list, take |
| `canonicalContactId` | 3308 | d, public_key_hex, canonicalContactId, catch, resolveIdentity, trim, formats, take, isEmpty, w |
| `canonicalId` | 3351 | isNullOrEmpty, Contact, canonicalContactId, trim, canonicalId, e, addContact |
| `addContact` | 3354 | isNullOrEmpty, Contact, trim, canonicalId, e, addContact |
| `getContact` | 3397 | showing, getContact, isNotEmpty, remove, catch, canonicalId, removeConversation, hasConversationWith, removeContact, w |
| `hasConversationWith` | 3406 | showing, isSame, isNotEmpty, remove, catch, canonicalId, removeConversation, isEmpty, w, removeContact |
| `removeContact` | 3415 | showing, isSame, d, remove, catch, canonicalId, removeConversation, isEmpty, w, removeContact |
| `listContacts` | 3446 | d, searchContacts, emptyList, blockPeer, search, catch, peerId, count, listContacts, i |
| `searchContacts` | 3450 | unblockPeer, d, searchContacts, emptyList, blockPeer, search, catch, peerId, count, i |
| `setContactNickname` | 3454 | unblockPeer, d, blockPeer, catch, peerId, count, i, setNickname, ensureServiceInitializedFireAndForget, setContactNickname |
| `getContactCount` | 3459 | unblockPeer, blockPeer, catch, peerId, count, i, messages, ensureServiceInitializedFireAndForget, e, getContactCount |
| `blockPeer` | 3467 | unblockPeer, blockPeer, catch, peerId, i, messages, ensureServiceInitializedFireAndForget, e, blockAndDeletePeer |
| `unblockPeer` | 3477 | unblockPeer, catch, peerId, i, isPeerBlocked, messages, isBlocked, ensureServiceInitializedFireAndForget, w, e |
| `blockAndDeletePeer` | 3492 | emptyList, getBlockedCount, catch, peerId, i, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, w, e |
| `isBlocked` | 3501 | emptyList, getBlockedCount, blockedCount, catch, isPeerBlocked, isBlocked, ensureServiceInitializedFireAndForget, w, listBlockedPeers |
| `listBlockedPeers` | 3511 | emptyList, getBlockedCount, blockedCount, catch, signData, ensureServiceInitializedFireAndForget, w, e, listBlockedPeers |
| `getBlockedCount` | 3521 | verifySignature, getBlockedCount, blockedCount, catch, ensureServiceInitializedFireAndForget, w, e, signData |
| `signData` | 3535 | verifySignature, catch, getSeniorityTimestamp, getDeviceId, ensureServiceInitializedFireAndForget, e, signData |
| `verifySignature` | 3545 | verifySignature, catch, getRegistrationState, getSeniorityTimestamp, getDeviceId, ensureServiceInitializedFireAndForget, e |
| `getDeviceId` | 3559 | w, catch, getInboxCount, getRegistrationState, getDeviceId, exportLogs, getSeniorityTimestamp |
| `getSeniorityTimestamp` | 3563 | w, catch, getInboxCount, getRegistrationState, inboxCount, exportLogs, getSeniorityTimestamp |
| `getRegistrationState` | 3567 | updateContactDeviceId, w, getInboxCount, catch, getRegistrationState, inboxCount, ID, exportLogs |
| `exportLogs` | 3575 | updateContactDeviceId, updateDeviceId, w, getInboxCount, catch, inboxCount, i, ID, exportLogs |
| `getInboxCount` | 3588 | updateContactDeviceId, updateDeviceId, getInboxCount, catch, inboxCount, i, ID, getIdentityInfoNonBlocking, w |
| `updateContactDeviceId` | 3596 | cacheIdentityFields, updateContactDeviceId, updateDeviceId, getIdentityInfo, catch, i, getIdentityInfoNonBlocking, w |
| `getIdentityInfoNonBlocking` | 3615 | d, cacheIdentityFields, getIdentityInfo, identity, readCachedIdentityFields, ensureServiceInitializedFireAndForget, ensureLocalIdentityFederation, w, getState |
| `getIdentityInfo` | 3641 | d, cacheIdentityFields, getIdentityInfo, trim, setNickname, isEmpty, ensureServiceInitializedFireAndForget, ensureLocalIdentityFederation, w, IllegalStateException |
| `setNickname` | 3660 | d, cacheIdentityFields, getIdentityInfo, catch, trim, i, setNickname, persistIdentityBackup, isEmpty, IllegalStateException |
| `setLocalNickname` | 3705 | currentTimeMillis, catch, resolveIdentity, i, copy, randomUUID, withContext, toULong, toString, setLocalNickname |
| `sendMessage` | 3723 | isSame, isNullOrEmpty, currentTimeMillis, catch, resolveIdentity, trim, toULong, randomUUID, withContext, toString |
| `dial` | 4018 | check, isIdentityInitialized, dial, catch, i, file, withContext, database, dialPeer, backup |
| `dialPeer` | 4031 | grantConsent, check, isIdentityInitialized, getIdentityInfo, w, dial, catch, restoreIdentityFromBackup, file, i |
| `isIdentityInitialized` | 4042 | grantConsent, getIdentityInfo, catch, restoreIdentityFromBackup, lost, File, i, exists, database, w |
| `grantConsent` | 4091 | grantConsent, d, getState, initializeAndStartBle, hasAllPermissions, catch, i, hasRequiredRuntimePermissions, ensureServiceInitializedFireAndForget, initializeAndStartWifi |
| `hasRequiredRuntimePermissions` | 4100 | d, initializeAndStartBle, hasAllPermissions, catch, initializeAndStartSwarm, hasRequiredRuntimePermissions, initializeAndStartWifi, w, getState, onRuntimePermissionsGranted |
| `onRuntimePermissionsGranted` | 4104 | d, initializeAndStartBle, createIdentity, catch, initializeAndStartSwarm, withContext, initializeAndStartWifi, w, getState |
| `createIdentity` | 4131 | grantConsent, d, createIdentity, catch, i, initializeAndStartSwarm, persistIdentityBackup, withContext, initialize_identity, IllegalStateException |
| `ensureServiceInitializedDeferred` | 4172 | d, MeshSettings, starting, MeshService, getState |
| `ensureServiceInitializedFireAndForget` | 4240 | currentTimeMillis, delay, paths, ensureServiceInitializedFireAndForget, start, w, ensureServiceInitializedDeferred, getState, ensureServiceInitialized |
| `ensureServiceInitialized` | 4250 | hasAllPermissions, currentTimeMillis, add, delay, checkSelfPermission, start, w, ensureServiceInitializedDeferred, addMessage, getState |
| `hasAllPermissions` | 4271 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, hasAllPermissions, add, canonicalId, getMessage |
| `addMessage` | 4278 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, add, removePendingOutbound, clear, canonicalId |
| `getMessage` | 4282 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, removePendingOutbound, clear, canonicalId, i |
| `getRecentMessages` | 4286 | markMessageDelivered, getRecentMessages, emptyList, searchMessages, search, markDelivered, validateAndStandardizeId, removePendingOutbound, clear, catch |
| `getConversation` | 4291 | markMessageDelivered, emptyList, searchMessages, search, markDelivered, validateAndStandardizeId, clear, catch, canonicalId, i |
| `searchMessages` | 4295 | markMessageDelivered, searchMessages, emptyList, search, markDelivered, validateAndStandardizeId, getHistoryStats, catch, i, clearConversation |
| `markMessageDelivered` | 4299 | markMessageDelivered, markDelivered, validateAndStandardizeId, getHistoryStats, catch, count, i, clearConversation, stats, getMessageCount |
| `clearHistory` | 4304 | validateAndStandardizeId, getHistoryStats, catch, count, i, clearConversation, stats, getMessageCount, clear, e |
| `clearConversation` | 4309 | validateAndStandardizeId, getHistoryStats, enforceRetention, catch, count, i, clearConversation, stats, getMessageCount, e |
| `getHistoryStats` | 4321 | enforceRetention, getHistoryStats, catch, count, timestamp, stats, getMessageCount, pruneBefore, e |
| `getMessageCount` | 4325 | enforceRetention, catch, count, timestamp, getMessageCount, pruneBefore, e |
| `enforceRetention` | 4336 | enforceRetention, w, catch, cancel, clear, timestamp, resetAllData, pruneBefore, e |
| `pruneBefore` | 4349 | shutdown, stop, w, catch, cancel, clear, resetAllData, pruneBefore, e, flush |
| `resetAllData` | 4362 | shutdown, edit, clear, cancel, catch, apply, flush, w, stop |
| `recordConnection` | 4412 | emptyList, normalizePublicKey, recordFailure, trim, recordConnection, isEmpty, recordConnectionFailure, getDialableAddresses, isLibp2pPeerId, replayDiscoveredPeerEvents |
| `recordConnectionFailure` | 4416 | emptyList, normalizePublicKey, recordFailure, prepopulateDiscoveryNickname, trim, isEmpty, recordConnectionFailure, getDialableAddresses, isLibp2pPeerId, replayDiscoveredPeerEvents |
| `getDialableAddresses` | 4420 | emptyList, normalizePublicKey, prepopulateDiscoveryNickname, trim, isEmpty, isLibp2pPeerId, getDialableAddresses, replayDiscoveredPeerEvents, dialableAddresses |
| `replayDiscoveredPeerEvents` | 4424 | normalizePublicKey, prepopulateDiscoveryNickname, trim, isEmpty, isLibp2pPeerId, ReplayDiscoveredIdentity, replayDiscoveredPeerEvents |
| `getAllKnownTopics` | 4502 | getLedgerSummary, emptyList, allKnownTopics, getNatStatus, w, getAllKnownTopics, catch, getServiceStateName, getConnectionPathState, summary |
| `getLedgerSummary` | 4506 | getLedgerSummary, getNatStatus, w, catch, getServiceStateName, getConnectionPathState, summary, getState, getDiscoveredPeerCount |
| `getConnectionPathState` | 4510 | getNatStatus, w, loadPendingOutbox, catch, getServiceStateName, getPendingOutboxCount, getConnectionPathState, getState, getDiscoveredPeerCount |
| `getNatStatus` | 4519 | getNatStatus, loadPendingOutbox, catch, getServiceStateName, getPendingDeliverySnapshot, isBlank, getPendingOutboxCount, getPendingTerminalFailureCode, w, getState |
| `getServiceStateName` | 4528 | loadPendingOutbox, getServiceStateName, getPendingDeliverySnapshot, isBlank, checkSelfPermission, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getState, getDiscoveredPeerCount |
| `getDiscoveredPeerCount` | 4532 | loadPendingOutbox, isBlank, FIX, checkSelfPermission, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getPendingDeliverySnapshot, getDiscoveredPeerCount |
| `getPendingOutboxCount` | 4536 | loadPendingOutbox, isBlank, FIX, checkSelfPermission, exportDiagnostics, getPendingOutboxCount, getMissingRuntimePermissions, getPendingTerminalFailureCode, getPendingDeliverySnapshot |
| `getPendingDeliverySnapshot` | 4540 | loadPendingOutbox, exportDiagnosticsAsync, exportDiagnosticsInternal, isBlank, FIX, checkSelfPermission, exportDiagnostics, withContext, getMissingRuntimePermissions, getPendingTerminalFailureCode |
| `getPendingTerminalFailureCode` | 4546 | loadPendingOutbox, exportDiagnosticsAsync, catch, exportDiagnosticsInternal, isBlank, FIX, checkSelfPermission, exportDiagnostics, put, withContext |
| `getMissingRuntimePermissions` | 4553 | loadPendingOutbox, exportDiagnosticsAsync, currentTimeMillis, catch, exportDiagnosticsInternal, FIX, exportDiagnostics, put, withContext, checkSelfPermission |
| `exportDiagnosticsAsync` | 4568 | currentTimeMillis, catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, toString, w |
| `exportDiagnostics` | 4585 | catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, w, JSONObject |
| `exportDiagnosticsInternal` | 4586 | catch, exportDiagnosticsInternal, getDiscoveryStats, exportDiagnostics, put, getClientStats, isNullOrBlank, w, JSONObject |
| `saveLedger` | 4662 | asSequence, normalizePublicKey, saveLedger, isNotEmpty, normalizeNickname, trim, save, emitIdentityDiscoveredIfChanged, isEmpty, isNullOrBlank |
| `emitIdentityDiscoveredIfChanged` | 4672 | asSequence, normalizePublicKey, isNotEmpty, normalizeNickname, trim, distinct, sorted, IdentityEmissionSignature, emitIdentityDiscoveredIfChanged, isEmpty |
| `emitConnectedIfChanged` | 4723 | currentTimeMillis, emitDisconnectedIfChanged, emitPeerEvent, Connected, isEmpty, emitConnectedIfChanged, normalize |
| `emitDisconnectedIfChanged` | 4744 | failed, load, currentTimeMillis, catch, emitPeerEvent, Disconnected, loadSettings, isEmpty, w, normalize |
| `loadSettings` | 4766 | failed, defaultSettings, load, catch, MeshSettings, getDefaultSettings, loadSettings, w |
| `getDefaultSettings` | 4781 | defaultSettings, MeshSettings, saveSettings, i, save |
| `saveSettings` | 4805 | d, saveSettings, i, enableTransport, save, loadSettings, applyTransportSettings, disableTransport |
| `applyTransportSettings` | 4815 | d, loadSettings, enableTransport, disableTransport |
| `validateSettings` | 4863 | w, computeProfile, computeBleAdjustment, validate, catch, computeAdjustmentProfile, BleAdjustment, validateSettings, computeRelayAdjustment |
| `computeAdjustmentProfile` | 4877 | setRelayBudget, computeProfile, computeBleAdjustment, overrideBleInterval, overrideBleScanInterval, computeAdjustmentProfile, BleAdjustment, computeRelayAdjustment, RelayAdjustment |
| `computeBleAdjustment` | 4882 | setRelayBudget, computeBleAdjustment, overrideBleInterval, overrideBleScanInterval, BleAdjustment, updateDeviceState, computeRelayAdjustment, RelayAdjustment |
| `computeRelayAdjustment` | 4891 | setRelayBudget, clearOverrides, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState, computeRelayAdjustment, RelayAdjustment |
| `overrideBleInterval` | 4900 | setRelayBudget, clearOverrides, overrideRelayMax, overrideBleInterval, overrideBleScanInterval, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState |
| `setRelayBudget` | 4904 | setRelayBudget, clearOverrides, getTopics, updateDeviceState, emptyList, overrideRelayMax, overrideRelayMaxPerHour, clearAdjustmentOverrides |
| `updateDeviceState` | 4908 | clearOverrides, getTopics, emptyList, overrideRelayMax, subscribeTopic, clearAdjustmentOverrides, overrideRelayMaxPerHour, updateDeviceState |
| `overrideRelayMax` | 4912 | getTopics, clearOverrides, emptyList, w, subscribeTopic, catch, clearAdjustmentOverrides, overrideRelayMaxPerHour, overrideRelayMax |
| `clearAdjustmentOverrides` | 4916 | getTopics, clearOverrides, emptyList, w, subscribeTopic, catch, unsubscribeTopic, clearAdjustmentOverrides |
| `getTopics` | 4931 | publishTopic, subscribeTopic, catch, unsubscribeTopic, w |
| `subscribeTopic` | 4936 | publishTopic, sendToAllPeers, subscribeTopic, catch, unsubscribeTopic, w |
| `unsubscribeTopic` | 4943 | publishTopic, sendToAllPeers, unsubscribeTopic, catch, connectToPeer, buildDialCandidatesForPeer, w |
| `publishTopic` | 4951 | publishTopic, sendToAllPeers, catch, identity_ids, connectToPeer, buildDialCandidatesForPeer, w |
| `sendToAllPeers` | 4963 | d, sendToAllPeers, shouldAttemptDial, dial, catch, identity_ids, connectToPeer, isLibp2pPeerId, buildDialCandidatesForPeer, w |
| `connectToPeer` | 4970 | d, shouldAttemptDial, dial, catch, identity_ids, connectToPeer, isLibp2pPeerId, buildDialCandidatesForPeer, e, contains |
| `ensurePendingOutboxRetryLoop` | 4996 | load, catch, flushPendingOutbox, delay, ensureCoverTrafficLoop, w, ensurePendingOutboxRetryLoop, primeRelayBootstrapConnections |
| `ensureCoverTrafficLoop` | 5019 | d, load, sendToAllPeers, catch, delay, w, prepareCoverTraffic, attemptDirectSwarmDelivery |
| `attemptDirectSwarmDelivery` | 5048 | currentTimeMillis, isNotBlank, isNullOrBlank, logDeliveryAttempt, attemptDirectSwarmDelivery, firstOrNull |
| `awaitPeerConnection` | 5695 | toMutableList, d, item, loadPendingOutbox, currentTimeMillis, catch, hasNext, flushPendingOutbox, listIterator, delay |
| `flushPendingOutbox` | 5710 | toMutableList, d, currentTimeMillis, listIterator, lock, logDeliveryState, pendingOutboxExpiryReason, item, next, flushPendingOutbox |
| `enqueuePendingOutbound` | 5906 | toMutableList, isMessageDeliveredLocally, loadPendingOutbox, PendingOutboundEnvelope, currentTimeMillis, add, randomUUID, logDeliveryState, enqueuePendingOutbound, toString |
| `loadPendingOutboxAsync` | 5970 | currentTimeMillis, readText, exists, randomUUID, add, optString, PendingOutboundEnvelope, until, emptyList, isNotEmpty |
| `loadPendingOutboxSync` | 6029 | currentTimeMillis, readText, exists, randomUUID, add, optString, has, PendingOutboundEnvelope, optInt, until |
| `loadPendingOutbox` | 6074 | JSONArray, savePendingOutbox, catch, put, toString, w, writeText, JSONObject |
| `savePendingOutbox` | 6077 | pendingOutboxExpiryReason, JSONArray, catch, put, Suppress, toString, w, writeText, JSONObject |
| `pendingOutboxExpiryReason` | 6107 | d, emptyList, normalizePublicKey, catch, resolveIdentity, orEmpty, list, resolveCanonicalPeerId |
| `resolveCanonicalPeerId` | 6116 | d, isSame, emptyList, normalizePublicKey, catch, resolveIdentity, orEmpty, list, resolveCanonicalPeerId |
| `resolveCanonicalPeerIdFromMessageHints` | 6200 | isSame, emptyList, normalizePublicKey, isNotEmpty, catch, trim, isBootstrapRelayPeer, orEmpty, resolveCanonicalPeerIdFromMessageHints, list |
| `encodeMessageWithIdentityHints` | 6237 | normalizePublicKey, getIdentityInfo, normalizeNickname, encodeMessageWithIdentityHints, JSONObject, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `encodeIdentitySyncPayload` | 6241 | normalizePublicKey, getIdentityInfo, normalizeNickname, JSONObject, JSONArray, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `encodeMeshMessagePayload` | 6245 | normalizePublicKey, getIdentityInfo, normalizeNickname, JSONObject, JSONArray, trim, encodeMeshMessagePayload, normalizeOutboundListenerHints, normalizeExternalAddressHints, orEmpty |
| `decodeMessageWithIdentityHints` | 6280 | decodeMessageWithIdentityHints, normalizePublicKey, normalizeNickname, isNotBlank, optJSONObject, trim, MessageIdentityHints, startsWith, jsonArrayToStringList, optString |
| `jsonArrayToStringList` | 6320 | lowercase, emptyList, isNotEmpty, normalizePublicKey, normalizeNickname, length, add, trim, distinct, selectAuthoritativeNickname |
| `normalizePublicKey` | 6330 | lowercase, normalizePublicKey, isNotEmpty, normalizeNickname, trim, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `normalizeNickname` | 6337 | lowercase, isBlePeerId, isNotEmpty, normalizeNickname, trim, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `isSyntheticFallbackNickname` | 6341 | lowercase, isBlePeerId, fromString, normalizeNickname, trim, isWifiPeerId, selectAuthoritativeNickname, startsWith, isSyntheticFallbackNickname |
| `selectAuthoritativeNickname` | 6348 | matches, isBlePeerId, fromString, normalizeNickname, Regex, trim, isWifiPeerId, selectAuthoritativeNickname, isEmpty, isSyntheticFallbackNickname |
| `isBlePeerId` | 6366 | matches, isBlePeerId, fromString, Regex, selectCanonicalPeerId, trim, isWifiPeerId, isEmpty, isLibp2pPeerId, isIdentityId |
| `isWifiPeerId` | 6370 | matches, isBlePeerId, Regex, selectCanonicalPeerId, trim, isWifiPeerId, isEmpty, isLibp2pPeerId, isIdentityId |
| `selectCanonicalPeerId` | 6378 | isBlePeerId, normalizePublicKey, normalizeNickname, prepopulateDiscoveryNickname, selectCanonicalPeerId, trim, isEmpty, isLibp2pPeerId, isIdentityId |
| `prepopulateDiscoveryNickname` | 6400 | emptyList, normalizePublicKey, normalizeNickname, prepopulateDiscoveryNickname, catch, takeLast, orEmpty, selectAuthoritativeNickname, list, isNullOrBlank |
| `resolveKnownPeerNickname` | 6438 | asSequence, normalizePublicKey, normalizeNickname, isNotBlank, trim, isNullOrBlank, dialableAddresses, resolveKnownPeerNickname, firstOrNull |
| `annotateIdentityInLedger` | 6493 | d, buildDialCandidatesForPeer, normalizePublicKey, isNotEmpty, trim, orEmpty, annotateIdentity, isEmpty, annotateIdentityInLedger, isLibp2pPeerId |
| `appendRoutingHint` | 6525 | appendRoutingHint, toMutableList, d, isNotEmpty, add, trim, split, joinToString, orEmpty, startsWith |
| `storeLastKnownRoutePeerId` | 6556 | appendRoutingHint, d, isNotEmpty, indexOf, add, catch, trim, split, copy, mergeNotes |
| `mergeNotes` | 6575 | isNotEmpty, indexOf, trim, split, joinToString, isNullOrBlank, isLibp2pPeerId, substring, resolveTransportIdentity |
| `resolveTransportIdentity` | 6603 | d, normalizePublicKey, getIdentityInfo, extractPublicKeyFromPeerId, catch, isBootstrapRelayPeer, orEmpty, list, isLibp2pPeerId, resolveTransportIdentity |
| `persistRouteHintsForTransportPeer` | 6691 | d, normalizePublicKey, persistRouteHintsForTransportPeer, catch, normalizeOutboundListenerHints, isBlank, orEmpty, list, parseRoutingHints, extractPublicKeyFromPeerId |
| `upsertFederatedContact` | 6762 | d, isNotEmpty, normalizePublicKey, catch, trim, isBootstrapRelayPeer, orEmpty, list, isNullOrBlank, isEmpty |
| `upsertRoutingListeners` | 6859 | toMutableList, savePendingOutbox, currentTimeMillis, upsertRoutingListeners, isNullOrBlank, removePendingOutbound, joinToString, split, isNotEmpty, loadPendingOutbox |
| `removePendingOutbound` | 6871 | toMutableList, loadPendingOutbox, savePendingOutbox, currentTimeMillis, trim, isBlank, promotePendingOutboundForPeer, copy, isNullOrBlank, isEmpty |
| `promotePendingOutboundForPeer` | 6879 | toMutableList, isMessageDeliveredLocally, loadPendingOutbox, currentTimeMillis, savePendingOutbox, trim, pruneDeliveredReceiptCache, copy, containsKey, isNullOrBlank |
| `isMessageDeliveredLocally` | 6903 | isMessageDeliveredLocally, currentTimeMillis, putIfAbsent, catch, remove, markDeliveredReceiptSeen, pruneDeliveredReceiptCache, containsKey, get |
| `markDeliveredReceiptSeen` | 6917 | RoutingHints, isNullOrEmpty, emptyList, currentTimeMillis, putIfAbsent, remove, putAll, pruneDeliveredReceiptCache, take, parseRoutingHints |
| `pruneDeliveredReceiptCache` | 6922 | RoutingHints, isNullOrEmpty, emptyList, currentTimeMillis, remove, putAll, pruneDeliveredReceiptCache, take, parseRoutingHints, clear |
| `parseRoutingHints` | 6939 | removePrefix, emptyList, RoutingHints, isNullOrEmpty, isNotEmpty, trim, split, startsWith, parseRoutingHints |
| `parseAllRoutingPeerIds` | 6984 | removePrefix, emptyList, parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, trim, split, distinct, startsWith |
| `parseLastKnownRoute` | 7003 | removePrefix, parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, trim, split, startsWith, isNullOrBlank, isEmpty |
| `buildRoutePeerCandidates` | 7013 | parseAllRoutingPeerIds, isNotEmpty, buildRoutePeerCandidates, add, asReversed, trim, lastOrNull, isNullOrBlank, isEmpty, discoverRoutePeersForPublicKey |
| `discoverRoutePeersForPublicKey` | 7083 | asSequence, emptyList, normalizePublicKey, isNotEmpty, trim, orEmpty, isLibp2pPeerId, isEmpty, discoverRoutePeersForPublicKey, dialableAddresses |
| `routeCandidateMatchesRecipient` | 7117 | emptyList, normalizePublicKey, catch, trim, isEmpty, isLibp2pPeerId, routeCandidateMatchesRecipient, extractPublicKeyFromPeerId, dialableAddresses, isKnownRelay |
| `buildDialCandidatesForPeer` | 7151 | getDialHintsForRoutePeer, emptyList, dialableAddresses, relayCircuitAddressesForPeer, prioritizeAddressesForCurrentNetwork, normalizeAddressHint, distinct, isNullOrBlank, isLibp2pPeerId, buildDialCandidatesForPeer |
| `getDialHintsForRoutePeer` | 7171 | getDialHintsForRoutePeer, emptyList, normalizeAddressHint, normalizeExternalAddressHints, distinct, normalizeOutboundListenerHints, trim, getLocalIpAddress, isLibp2pPeerId, isEmpty |
| `normalizeOutboundListenerHints` | 7183 | replace, isDialableAddress, normalizeAddressHint, trim, normalizeExternalAddressHints, normalizeOutboundListenerHints, distinct, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress |
| `normalizeExternalAddressHints` | 7189 | replace, isDialableAddress, normalizeAddressHint, trim, normalizeExternalAddressHints, distinct, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress, isEmpty |
| `normalizeAddressHint` | 7195 | removePrefix, replace, isDialableAddress, removeSuffix, normalizeAddressHint, trim, toMultiaddrFromSocketAddress, startsWith, getLocalIpAddress, isEmpty |
| `toMultiaddrFromSocketAddress` | 7216 | removePrefix, contains, isDialableAddress, removeSuffix, matches, isSameLanAddress, Regex, toMultiaddrFromSocketAddress, trim, startsWith |
| `isDialableAddress` | 7236 | isDialableAddress, isSameLanAddress, toIntOrNull, parseIpv4Octets, split, isSpecialUseIpv4, extractIpv4FromMultiaddr, isPrivateIpv4, contains |
| `parseIpv4Octets` | 7249 | parseIpv4Octets, split, isSpecialUseIpv4, isPrivateIpv4, toIntOrNull |
| `isPrivateIpv4` | 7256 | parseIpv4Octets, trim, isBootstrapRelayPeer, isSpecialUseIpv4, equals, isPrivateIpv4, isKnownRelay |
| `isSpecialUseIpv4` | 7263 | emptyList, relayCircuitAddressesForPeer, parseIpv4Octets, trim, isBootstrapRelayPeer, isSpecialUseIpv4, equals, isLibp2pPeerId, isKnownRelay |
| `isKnownRelay` | 7280 | d, emptyList, relayCircuitAddressesForPeer, Relays, closed, parseBootstrapRelay, getFailureCount, trim, isBootstrapRelayPeer, isCircuitOpen |
| `relayCircuitAddressesForPeer` | 7289 | d, emptyList, relayCircuitAddressesForPeer, Relays, closed, add, parseBootstrapRelay, getFailureCount, isCircuitOpen, getHealthyRelays |
| `parseBootstrapRelay` | 7331 | trimEnd, extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, parseBootstrapRelay, trim, isBootstrapRelayPeer, isBlank, isEmpty, substring, lastIndexOf |
| `isBootstrapRelayPeer` | 7341 | extractPeerIdFromPublicKey, isBootstrapRelayPeerFromKey, isBootstrapRelayPeer, isBlank, createEmergencyContact |
| `isBootstrapRelayPeerFromKey` | 7352 | extractPeerIdFromPublicKey, Contact, currentTimeMillis, isBootstrapRelayPeer, isBlank, toULong, normalize, createEmergencyContact |
| `createEmergencyContact` | 7369 | extractPeerIdFromPublicKey, Contact, currentTimeMillis, add, catch, i, toULong, take, e, normalize |
| `validatePeerBeforeContactCreation` | 7408 | d, isBootstrapRelayPeer, isBlank, take, w, isValidPeerId, isValidPublicKey |
| `logIdentityResolutionDetails` | 7440 | d, TCP, currentTimeMillis, WebSocket, take, primeRelayBootstrapConnections |
| `primeRelayBootstrapConnections` | 7460 | d, allowRequest, currentTimeMillis, shouldAttemptDial, dial, getTransportPriority, i |
| `primeRelayBootstrapConnectionsLegacy` | 7522 | d, isSameLanAddress, currentTimeMillis, shouldAttemptDial, catch, dial, prioritizeAddressesForCurrentNetwork, distinct, getLocalIpAddress, isEmpty |
| `prioritizeAddressesForCurrentNetwork` | 7537 | isSameLanAddress, indexOf, prioritizeAddressesForCurrentNetwork, distinct, split, getLocalIpAddress, isEmpty, substring, extractIpv4FromMultiaddr, sameSubnet24 |
| `isSameLanAddress` | 7544 | isSameLanAddress, indexOf, split, getLocalIpAddress, sameSubnet24, substring, ports, extractIpv4FromMultiaddr |
| `extractIpv4FromMultiaddr` | 7550 | racingBootstrapWithFallback, indexOf, getNetworkDiagnostics, split, joinToString, getTransportPriority, i, sameSubnet24, substring, ports |
| `sameSubnet24` | 7559 | racingBootstrapWithFallback, isNotEmpty, getNetworkDiagnostics, split, joinToString, getOpenCircuits, getTransportPriority, i, network, sameSubnet24 |
| `racingBootstrapWithFallback` | 7576 | isNotEmpty, resetAll, getNetworkDiagnostics, listOf, joinToString, getOpenCircuits, network, getTransportPriority, i, probePorts |
| `attemptMdnsFallback` | 7676 | toInt, MdnsFallback, primeRelayBootstrapConnectionsLegacy, peer, getStats, withTimeoutOrNull, i, delay, e |
| `bootstrapWithFallbackStrategy` | 7724 | racingBootstrapWithFallback, failed, currentTimeMillis, startNetworkChangeWatch, cancel, i, e, s |
| `startNetworkChangeWatch` | 7747 | s, coerceAtMost, currentTimeMillis, minOf, i, w, cancel, startNetworkChangeWatch, ago |
| `stopNetworkChangeWatch` | 7791 | setOf, stopNetworkChangeWatch, classifyBootstrapError, extractPortFromMultiaddr, cancel |
| `classifyBootstrapError` | 7801 | host, setOf, unreachable, extractPortFromMultiaddr |
| `extractPortFromMultiaddr` | 7839 | toIntOrNull, coerceAtMost, Regex, shouldAttemptDial, currentTimeMillis, find, trim, isEmpty, get |
| `shouldAttemptDial` | 7846 | coerceAtMost, currentTimeMillis, shouldAttemptDial, trim, to, isEmpty |
| `getPreferredRelay` | 7878 | StatFs, getNetworkDiagnosticsSnapshot, performMaintenance, getSummary, getPreferredRelay, getNetworkDiagnostics, detectAndRecoverMessageTracking, updateDiskStats, getNetworkFailureSummary, getPreferredRelays |
| `getNetworkFailureSummary` | 7885 | d, StatFs, getNetworkDiagnosticsSnapshot, performMaintenance, getSummary, getNetworkDiagnostics, catch, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats |
| `getNetworkDiagnosticsSnapshot` | 7890 | d, StatFs, performMaintenance, catch, getNetworkDiagnostics, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats, toULong, delay |
| `startStorageMaintenance` | 7893 | d, StatFs, performMaintenance, catch, detectAndRecoverMessageTracking, handleBleTransportDegradation, updateDiskStats, toULong, delay, logRetryStormDetection |
| `getExternalAddresses` | 7927 | lowercase, emptyList, nextElement, addresses, getNetworkInterfaces, hasMoreElements, orEmpty, getLocalIpAddress, getListeners, getExternalAddresses |
| `getListeningAddresses` | 7937 | lowercase, emptyList, nextElement, getNetworkInterfaces, hasMoreElements, trim, orEmpty, startsWith, getLocalIpAddress, isEmpty |
| `getLocalIpAddress` | 7940 | lowercase, nextElement, getNetworkInterfaces, hasMoreElements, trim, orEmpty, startsWith, getLocalIpAddress, isEmpty, isSpecialUseIpv4 |
| `getIdentityExportString` | 7983 | toMutableList, replace, normalizeExternalAddressHints, distinct, libp2p_peer_id, secondary, getIdentityExportString, normalizeOutboundListenerHints, put, getLocalIpAddress |
| `observeIncomingMessages` | 8021 | emptyList, getStats, distinct, observePeers, emit, withContext, observeNetworkStats, dialableAddresses |
| `observePeers` | 8028 | emptyList, getStats, distinct, emit, delay, withContext, observeNetworkStats, dialableAddresses |
| `observeNetworkStats` | 8043 | cleanup, saveLedger, getStats, stopMeshService, emit, delay, withContext, clear, cancel |
| `cleanup` | 8060 | cleanup, saveLedger, catch, stopMeshService, i, clear, e, cancel |
| `getDiagnosticsLogPath` | 8093 | getDiagnosticsLogPath, readLines, catch, takeLast, getDiagnosticsLogs, joinToString, File, i, exists, isEmpty |
| `getDiagnosticsLogs` | 8100 | getDiagnosticsLogPath, readLines, catch, takeLast, joinToString, File, i, exists, isBlank, isEmpty |
| `clearDiagnosticsLogs` | 8117 | logDeliveryAttempt, getDiagnosticsLogPath, isNotBlank, catch, isBlank, File, i, exists, logDeliveryState, unknown |
| `logDeliveryState` | 8128 | isNotBlank, isBlank, i, logDeliveryState, unknown, w, logDeliveryAttempt |
| `logDeliveryAttempt` | 8133 | isNotBlank, i, unknown, w, migrateToCanonicalIds, logDeliveryAttempt |
| `migrateToCanonicalIds` | 8160 | getBoolean, add, catch, resolveIdentity, i, copy, list, getSharedPreferences, w, get |
| `emergencyContactRecovery` | 8250 | isNullOrEmpty, extractPublicKeyFromPeerId, catch, distinct, w, e, get, recent |
| `detectAndRepairCorruption` | 8318 | toInt, d, backupCorruptedDatabase, i, list, stats, emergencyContactRecovery, e |
| `backupCorruptedDatabase` | 8364 | listFiles, mkdirs, copyTo, currentTimeMillis, catch, File, i, exists, e |
| `handleBleTransportDegradation` | 8406 | recordFailure, clearPeerCache, recordTransportEvent, isDegraded, recordSuccess, handleBleTransportDegradation, attemptBleRecovery, w, setBackgroundMode, getHealth |
| `recordTransportEvent` | 8422 | emptyList, recordFailure, getSummary, recordSuccess, isDegraded, handleBleTransportDegradation, getTransportHealthSummary, attemptBleRecovery, getActiveTransports |
| `getTransportHealthSummary` | 8437 | d, emptyList, getSummary, getTransportHealthSummary, handleBleFailure, attemptBleRecovery, getActiveTransports |
| `getActiveTransports` | 8445 | d, emptyList, handleBleFailure, attemptBleRecovery, forceRestartScanning, getActiveTransports |
| `handleBleFailure` | 8454 | d, clearPeerCache, handleBleFailure, forceRestartScanning, attemptBleRecovery |
| `attemptBleRecovery` | 8464 | d, transportTypeFromValue, clearPeerCache, forceRestartScanning, attemptBleRecovery |
| `forceRestartScanning` | 8473 | d, emptyList, getSmartAvailableTransports, transportTypeFromValue, clearPeerCache, forceRestartScanning, fromValue, getAvailableTransports |
| `clearPeerCache` | 8482 | d, emptyList, getSmartAvailableTransports, transportTypeFromValue, clearPeerCache, getAvailableTransportsSorted, fromValue, getAvailableTransports |
| `transportTypeFromValue` | 8494 | emptyList, getSmartAvailableTransports, getAvailableTransportsSorted, getDedupStats, fromValue, getAvailableTransports |
| `getSmartAvailableTransports` | 8502 | emptyList, getSubscribedTopicsList, getAvailableTransportsSorted, getDedupStats, getAvailableTransports |
| `getAvailableTransportsSorted` | 8510 | emptyList, getSubscribedTopicsList, getKnownTopicsList, getAvailableTransportsSorted, getDedupStats |
| `getDedupStats` | 8518 | emptyList, getSubscribedTopicsList, getKnownTopicsList, filterMessagesByTopic, getDedupStats |
| `getSubscribedTopicsList` | 8526 | emptyList, getSubscribedTopicsList, getKnownTopicsList, filterMessagesByTopic, enableTransport |
| `getKnownTopicsList` | 8534 | emptyList, startAll, getKnownTopicsList, filterMessagesByTopic, startAllTransports, enableTransport |
| `filterMessagesByTopic` | 8542 | startAll, filterMessagesByTopic, startAllTransports, shouldUseTransport, enableTransport |
| `enableTransport` | 8550 | startAll, getBleQuotaCount, startAllTransports, shouldUseTransport, enableTransport |
| `startAllTransports` | 8558 | shouldUseTransport, isPortLikelyBlocked, getBleQuotaCount, startAll |
| `shouldUseTransport` | 8566 | getNetworkStateLogString, getBleQuotaCount, getNetworkDiagnostics, shouldUseTransport, toLogString, isPortLikelyBlocked |
| `getBleQuotaCount` | 8574 | getNetworkStateLogString, getBleQuotaCount, getNetworkDiagnostics, getHealthyRelays, toLogString, isPortLikelyBlocked |
| `isPortLikelyBlocked` | 8582 | getNetworkStateLogString, getNetworkDiagnostics, getLastFailure, getHealthyRelays, toLogString, isPortLikelyBlocked |
| `getNetworkStateLogString` | 8590 | getNetworkDiagnostics, getLastFailure, getHealthyRelays, toLogString, getLastFailureReason |
| `getHealthyRelays` | 8598 | getLastFailure, getOpenCircuits, getHealthyRelays, getOpenCircuitCount, getLastFailureReason |
| `getLastFailure` | 8606 | formatDiagnosticsReportForUser, catch, getLastFailure, getOpenCircuits, generateReport, getOpenCircuitCount, getLastFailureReason, e, formatReportForUser |
| `getLastFailureReason` | 8614 | formatDiagnosticsReportForUser, catch, generateReport, getOpenCircuits, hasDnsFailures, getOpenCircuitCount, getLastFailureReason, e, formatReportForUser |
| `getOpenCircuitCount` | 8622 | formatDiagnosticsReportForUser, catch, getOpenCircuits, generateReport, hasDnsFailures, hasPortBlocking, e, formatReportForUser |
| `formatDiagnosticsReportForUser` | 8630 | catch, generateReport, hasDnsFailures, hasPortBlocking, e, formatReportForUser |
| `hasDnsFailures` | 8644 | hasPortBlocking, hasDnsFailures |
| `hasPortBlocking` | 8652 | hasPortBlocking |

### Imports
- `import android.content.Context`
- `import android.content.SharedPreferences`
- `import android.content.pm.PackageManager`
- `import androidx.core.content.ContextCompat`
- `import com.scmessenger.android.service.TransportType`
- `import com.scmessenger.android.transport.NetworkDetector`
- `import com.scmessenger.android.transport.SmartTransportRouter`
- `import com.scmessenger.android.transport.TransportManager`
- `import com.scmessenger.android.utils.CircuitBreaker`
- `import com.scmessenger.android.utils.NetworkFailureMetrics`
- `import com.scmessenger.android.utils.PeerIdValidator`
- `import com.scmessenger.android.utils.PeerKeyUtils`
- `import com.scmessenger.android.utils.Permissions`
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.async`
- `import kotlinx.coroutines.cancel`
- `import kotlinx.coroutines.flow.MutableSharedFlow`
- `import kotlinx.coroutines.flow.MutableStateFlow`
- `import kotlinx.coroutines.flow.StateFlow`
- `import kotlinx.coroutines.flow.asSharedFlow`
- `import kotlinx.coroutines.flow.asStateFlow`
- `import kotlinx.coroutines.flow.filter`
- `import kotlinx.coroutines.flow.update`
- `import kotlinx.coroutines.isActive`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.sync.Mutex`
- `import kotlinx.coroutines.sync.withLock`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt (2 chunks, 217 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt: Defines 2 types: PreferencesRepository, ThemeMode; 13 functions; 7 imports android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt: Defines 2 types: PreferencesRepository, ThemeMode; 13 functions; 7 imports

### Structs/Classes
- PreferencesRepository
- ThemeMode

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `setServiceAutoStart` | 47 | setOnboardingCompleted, d, setServiceAutoStart, setVpnMode |
| `setVpnMode` | 58 | d, setInstallChoiceCompleted, setVpnMode, setOnboardingCompleted, i |
| `setOnboardingCompleted` | 73 | setInstallChoiceCompleted, setOnboardingCompleted, i |
| `setInstallChoiceCompleted` | 84 | d, setThemeMode, setInstallChoiceCompleted, i, lowercase |
| `setThemeMode` | 107 | d, setShowPeerCount, setThemeMode, setNotificationsEnabled, lowercase |
| `setNotificationsEnabled` | 118 | setNotificationsEnabled, setAutoAdjustEnabled, d, setShowPeerCount |
| `setShowPeerCount` | 129 | d, setAutoAdjustEnabled, setShowPeerCount, setManualAdjustmentProfile |
| `setAutoAdjustEnabled` | 144 | setManualAdjustmentProfile, clear, d, w, clearAll, setAutoAdjustEnabled |
| `setManualAdjustmentProfile` | 155 | setManualAdjustmentProfile, clear, d, w, setBleRotationEnabled, clearAll |
| `clearAll` | 166 | clear, d, w, setBleRotationEnabled, seconds, clearAll, setBleRotationIntervalSec |
| `setBleRotationEnabled` | 179 | d, setIdentityNickname, setBleRotationEnabled, NICKNAME, seconds, isNullOrBlank, setBleRotationIntervalSec |
| `setBleRotationIntervalSec` | 191 | d, setIdentityNickname, remove, NICKNAME, i, isNullOrBlank, trim, setBleRotationIntervalSec |
| `setIdentityNickname` | 206 | setIdentityNickname, remove, i, isNullOrBlank, trim |
| `setServiceAutoStart` | 47 | d, setOnboardingCompleted, setServiceAutoStart, setVpnMode |
| `setVpnMode` | 58 | d, setOnboardingCompleted, setInstallChoiceCompleted, setVpnMode, i |
| `setOnboardingCompleted` | 73 | setOnboardingCompleted, i, setInstallChoiceCompleted |
| `setInstallChoiceCompleted` | 84 | lowercase, d, setThemeMode, setInstallChoiceCompleted, i |
| `setThemeMode` | 107 | lowercase, d, setThemeMode, setShowPeerCount, setNotificationsEnabled |
| `setNotificationsEnabled` | 118 | setAutoAdjustEnabled, setNotificationsEnabled, d, setShowPeerCount |
| `setShowPeerCount` | 129 | setAutoAdjustEnabled, d, setManualAdjustmentProfile, setShowPeerCount |
| `setAutoAdjustEnabled` | 144 | setAutoAdjustEnabled, d, clearAll, w, setManualAdjustmentProfile, clear |
| `setManualAdjustmentProfile` | 155 | d, clearAll, clear, setBleRotationEnabled, setManualAdjustmentProfile, w |
| `clearAll` | 166 | d, clearAll, w, setBleRotationIntervalSec, seconds, setBleRotationEnabled, clear |
| `setBleRotationEnabled` | 179 | d, setBleRotationIntervalSec, seconds, setBleRotationEnabled, NICKNAME, isNullOrBlank, setIdentityNickname |
| `setBleRotationIntervalSec` | 191 | d, setBleRotationIntervalSec, remove, trim, i, NICKNAME, isNullOrBlank, setIdentityNickname |
| `setIdentityNickname` | 206 | remove, trim, i, isNullOrBlank, setIdentityNickname |

### Imports
- `import android.content.Context`
- `import androidx.datastore.core.DataStore`
- `import androidx.datastore.preferences.core.*`
- `import androidx.datastore.preferences.preferencesDataStore`
- `import kotlinx.coroutines.flow.Flow`
- `import kotlinx.coroutines.flow.map`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt (2 chunks, 1095 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt: 16 functions; 27 imports android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt: 16 functions; 27 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `SettingsScreen` | 35 | LaunchedEffect, rememberCoroutineScope, hiltViewModel, mutableStateOf, collectAsState, SnackbarHostState |
| `DataManagementSection` | 270 | Text, AlertDialog, Card, onResetAll, textButtonColors, mutableStateOf, TextButton, fillMaxWidth |
| `ServiceControlSection` | 333 | Text, Card, padding, Row, Column, fillMaxWidth |
| `MeshSettingsSection` | 387 | Text, Card, padding, Messaging, cardColors, Column, fillMaxWidth |
| `AppPreferencesSection` | 476 | Text, Card, padding, SwitchPreference, Column, fillMaxWidth |
| `ThemeSection` | 512 | Text, Card, padding, onThemeModeChange, Column, ThemeRadioOption, fillMaxWidth |
| `ThemeRadioOption` | 548 | Text, width, padding, SwitchPreference, Row, RadioButton, Spacer, fillMaxWidth |
| `SwitchPreference` | 572 | Text, padding, weight, Row, Switch, Column, fillMaxWidth |
| `InfoSection` | 609 | Text, Card, padding, InfoRow, Row, Column, toString, fillMaxWidth |
| `InfoRow` | 633 | Text, padding, Card, Row, IdentitySection, fillMaxWidth |
| `IdentitySection` | 653 | Text, remember, OutlinedTextField, Card, padding, mutableStateOf, Column, fillMaxWidth |
| `IdentityUnavailableSection` | 826 | Text, Button, Card, padding, PrivacySection, Column, spacedBy, fillMaxWidth |
| `PrivacySection` | 855 | Text, Icon, width, Button, Card, padding, height, Intent, size, Column |
| `AdvancedSettingsSection` | 899 | Text, Button, Card, padding, Column, fillMaxWidth |
| `SettingsToMeshSettingsNavigation` | 942 | Text, Button, Card, padding, Column, fillMaxWidth |
| `SettingsToPowerSettingsNavigation` | 973 | Text, Button, Card, padding, Column, fillMaxWidth |
| `SettingsScreen` | 35 | LaunchedEffect, mutableStateOf, collectAsState, SnackbarHostState, rememberCoroutineScope, hiltViewModel |
| `DataManagementSection` | 270 | Card, Text, mutableStateOf, fillMaxWidth, textButtonColors, onResetAll, TextButton, AlertDialog |
| `ServiceControlSection` | 333 | Card, padding, Column, Text, fillMaxWidth, Row |
| `MeshSettingsSection` | 387 | Card, padding, Column, Text, cardColors, Messaging, fillMaxWidth |
| `AppPreferencesSection` | 476 | Card, padding, Column, Text, fillMaxWidth, SwitchPreference |
| `ThemeSection` | 512 | Card, padding, Column, Text, ThemeRadioOption, fillMaxWidth, onThemeModeChange |
| `ThemeRadioOption` | 548 | padding, Text, fillMaxWidth, Row, width, RadioButton, Spacer, SwitchPreference |
| `SwitchPreference` | 572 | padding, Text, Column, weight, Switch, fillMaxWidth, Row |
| `InfoSection` | 609 | Card, padding, Column, Text, fillMaxWidth, InfoRow, Row, toString |
| `InfoRow` | 633 | padding, Text, Card, fillMaxWidth, Row, IdentitySection |
| `IdentitySection` | 653 | Card, padding, Column, Text, mutableStateOf, OutlinedTextField, fillMaxWidth, remember |
| `IdentityUnavailableSection` | 826 | Card, padding, Column, Text, PrivacySection, fillMaxWidth, Button, spacedBy |
| `PrivacySection` | 855 | Card, padding, Column, Text, size, fillMaxWidth, OutlinedButton, Intent, Button, height |
| `AdvancedSettingsSection` | 899 | Card, padding, Column, Text, fillMaxWidth, Button |
| `SettingsToMeshSettingsNavigation` | 942 | Card, padding, Column, Text, fillMaxWidth, Button |
| `SettingsToPowerSettingsNavigation` | 973 | Card, padding, Column, Text, fillMaxWidth, Button |

### Imports
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.rememberScrollState`
- `import androidx.compose.foundation.verticalScroll`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.filled.Block`
- `import androidx.compose.material.icons.filled.ContentCopy`
- `import androidx.compose.material.icons.filled.Info`
- `import androidx.compose.material.icons.filled.Share`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.platform.LocalContext`
- `import androidx.compose.ui.res.stringResource`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.BuildConfig`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.data.PreferencesRepository`
- `import com.scmessenger.android.ui.components.ErrorState`
- `import com.scmessenger.android.ui.components.WarningBanner`
- `import com.scmessenger.android.ui.settings.MeshSettingsScreen`
- `import com.scmessenger.android.ui.settings.PowerSettingsScreen`
- `import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel`
- `import com.scmessenger.android.ui.viewmodels.SettingsViewModel`
- `import kotlinx.coroutines.launch`
---

## android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt (2 chunks, 166 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt: Defines 1 types: TopicManager; 10 functions; 4 imports android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt: Defines 1 types: TopicManager; 10 functions; 4 imports

### Structs/Classes
- TopicManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `initialize` | 50 | d, catch, toMutableSet, subscribeTopic, add, subscribe, i, refreshKnownTopics, e |
| `subscribe` | 65 | catch, toMutableSet, unsubscribeTopic, subscribeTopic, add, remove, i, e, unsubscribe |
| `unsubscribe` | 81 | LedgerManager, catch, d, SwarmBridge, topics, toMutableSet, unsubscribeTopic, remove, getTopics, i |
| `refreshKnownTopics` | 97 | LedgerManager, d, catch, SwarmBridge, topics, autoSubscribeToPeerTopics, contains, getTopics, subscribe, getAllKnownTopics |
| `autoSubscribeToPeerTopics` | 120 | d, filterMessagesByTopic, Suppress, messages, contains, subscribe, publish, publishTopic, isSubscribed |
| `filterMessagesByTopic` | 133 | getSubscribedTopicsList, messages, toList, contains, getKnownTopicsList, publishTopic, publish, isSubscribed |
| `publish` | 142 | getSubscribedTopicsList, toList, contains, getKnownTopicsList, publishTopic, isSubscribed |
| `isSubscribed` | 149 | getSubscribedTopicsList, toList, getKnownTopicsList, contains |
| `getSubscribedTopicsList` | 156 | getKnownTopicsList, toList |
| `getKnownTopicsList` | 163 | toList |
| `initialize` | 50 | d, toMutableSet, subscribe, add, subscribeTopic, catch, refreshKnownTopics, i, e |
| `subscribe` | 65 | toMutableSet, add, remove, subscribeTopic, catch, unsubscribeTopic, i, e, unsubscribe |
| `unsubscribe` | 81 | getTopics, d, SwarmBridge, toMutableSet, remove, getAllKnownTopics, unsubscribeTopic, catch, topics, i |
| `refreshKnownTopics` | 97 | getTopics, d, SwarmBridge, getAllKnownTopics, subscribe, catch, topics, autoSubscribeToPeerTopics, LedgerManager, e |
| `autoSubscribeToPeerTopics` | 120 | publishTopic, d, subscribe, filterMessagesByTopic, isSubscribed, Suppress, publish, messages, contains |
| `filterMessagesByTopic` | 133 | publishTopic, getSubscribedTopicsList, getKnownTopicsList, isSubscribed, publish, messages, contains, toList |
| `publish` | 142 | publishTopic, getSubscribedTopicsList, getKnownTopicsList, isSubscribed, contains, toList |
| `isSubscribed` | 149 | getKnownTopicsList, contains, toList, getSubscribedTopicsList |
| `getSubscribedTopicsList` | 156 | getKnownTopicsList, toList |
| `getKnownTopicsList` | 163 | toList |

### Imports
- `import kotlinx.coroutines.flow.MutableStateFlow`
- `import kotlinx.coroutines.flow.StateFlow`
- `import kotlinx.coroutines.flow.asStateFlow`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt (1 chunks, 689 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt: 4 functions; 30 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `AddContactScreen` | 44 | Text, emptyList, IconButton, mutableStateOf, collectAsState, Icon, hiltViewModel, remember, TopAppBar, Scaffold |
| `ManualEntryTab` | 176 | padding, Column, fillMaxSize, stringResource, isNotBlank, spacedBy, Row, IdenticonFromPeerId, verticalScroll, rememberScrollState |
| `QRScanTab` | 285 | padding, Text, Column, fillMaxSize, size, height, Icon, Spacer |
| `NearbyDiscoveryTab` | 371 | padding, Text, Column, fillMaxSize, size, height, Icon, Spacer |

### Imports
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.rememberScrollState`
- `import androidx.compose.foundation.verticalScroll`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.filled.*`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.platform.LocalContext`
- `import androidx.compose.ui.res.stringResource`
- `import androidx.compose.ui.text.font.FontFamily`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.google.android.gms.common.ConnectionResult`
- `import com.google.android.gms.common.GoogleApiAvailability`
- `import com.google.android.gms.common.api.CommonStatusCodes`
- `import com.google.mlkit.common.MlKitException`
- `import com.google.mlkit.vision.barcode.common.Barcode`
- `import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions`
- `import com.google.mlkit.vision.codescanner.GmsBarcodeScanning`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.ui.components.ErrorBanner`
- `import com.scmessenger.android.ui.components.IdenticonFromPeerId`
- `import com.scmessenger.android.ui.viewmodels.ContactsViewModel`
- `import com.scmessenger.android.utils.ContactImportParseResult`
- `import com.scmessenger.android.utils.parseContactImportPayload`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt (1 chunks, 311 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt: 5 functions; 27 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `IdentityScreen` | 39 | Text, LaunchedEffect, IconButton, getQrCodeData, collectAsState, hiltViewModel, Icon, TopAppBar, loadIdentity, Scaffold |
| `IdentityNotInitializedView` | 113 | padding, Text, Column, mutableStateOf, OutlinedTextField, onCreateIdentity, fillMaxWidth, Button, spacedBy |
| `IdentityContent` | 150 | padding, Card, Column, fillMaxSize, cardColors, Text, ErrorBanner, spacedBy, Suppress, verticalScroll |
| `QRCodeDisplay` | 263 | Card, padding, QRCodeWriter, size, generateQRCode, catch, Image, asImageBitmap, remember, e |
| `generateQRCode` | 292 | QRCodeWriter, createBitmap, setPixel, encode |

### Imports
- `import android.graphics.Bitmap`
- `import androidx.compose.foundation.Image`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.rememberScrollState`
- `import androidx.compose.foundation.verticalScroll`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.filled.Refresh`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.graphics.asImageBitmap`
- `import androidx.compose.ui.platform.LocalContext`
- `import androidx.compose.ui.res.stringResource`
- `import androidx.compose.ui.text.font.FontFamily`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.google.zxing.BarcodeFormat`
- `import com.google.zxing.qrcode.QRCodeWriter`
- `import com.scmessenger.android.R`
- `import com.scmessenger.android.ui.components.CopyableText`
- `import com.scmessenger.android.ui.components.ErrorBanner`
- `import com.scmessenger.android.ui.components.IdenticonFromPeerId`
- `import com.scmessenger.android.ui.viewmodels.IdentityViewModel`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt (1 chunks, 379 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt: Defines 9 types: Screen, Conversations, Contacts, AddContact, Dashboard; 5 functions; 32 imports

### Structs/Classes
- AddContact
- BlockedPeers
- Contacts
- Conversations
- Dashboard
- Diagnostics
- Identity
- Screen
- Settings

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `MeshApp` | 42 | LaunchedEffect, mutableStateOf, collectAsState, hiltViewModel, navigate, delay, rememberNavController, refreshIdentityState |
| `MeshNavHost` | 123 | ContactsScreen, navigate, composable, NavHost, ConversationsScreen, PaddingValues, startDestinationForRole |
| `MeshBottomBar` | 288 | Text, roleBasedBottomNavItems, currentBackStackEntryAsState, navigate, startsWith, NavigationBarItem, Icon, Screen, popUpTo |
| `roleBasedBottomNavItems` | 333 | roleBasedBottomNavItems, startDestinationForRole |
| `startDestinationForRole` | 336 | startDestinationForRole |

### Imports
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.material.icons.automirrored.filled.Chat`
- `import androidx.compose.material.icons.filled.Add`
- `import androidx.compose.material.icons.filled.Block`
- `import androidx.compose.material.icons.filled.People`
- `import androidx.compose.material.icons.filled.Router`
- `import androidx.compose.material.icons.filled.Settings`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.Composable`
- `import androidx.compose.runtime.LaunchedEffect`
- `import androidx.compose.runtime.collectAsState`
- `import androidx.compose.runtime.getValue`
- `import androidx.compose.runtime.mutableStateOf`
- `import androidx.compose.runtime.remember`
- `import androidx.compose.runtime.setValue`
- `import androidx.compose.ui.Modifier`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import androidx.navigation.NavHostController`
- `import androidx.navigation.compose.NavHost`
- `import androidx.navigation.compose.composable`
- `import androidx.navigation.compose.currentBackStackEntryAsState`
- `import androidx.navigation.compose.rememberNavController`
- `import com.scmessenger.android.ui.contacts.AddContactScreen`
- `import com.scmessenger.android.ui.contacts.ContactDetailScreen`
- `import com.scmessenger.android.ui.dashboard.PeerListScreen`
- `import com.scmessenger.android.ui.dashboard.TopologyScreen`
- `import com.scmessenger.android.ui.identity.IdentityScreen`
- `import com.scmessenger.android.ui.screens.*`
- `import com.scmessenger.android.ui.viewmodels.DeepLinkData`
- `import com.scmessenger.android.ui.viewmodels.MainViewModel`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/BlockedPeersScreen.kt (1 chunks, 158 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/BlockedPeersScreen.kt: 3 functions; 17 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `BlockedPeersScreen` | 23 | padding, Text, fillMaxSize, Column, IconButton, collectAsState, size, hiltViewModel, Scaffold, isEmpty |
| `BlockedPeerItem` | 108 | Card, padding, Column, weight, Text, fillMaxWidth, take, Row, formatDate, isNullOrBlank |
| `formatDate` | 151 | Date, getDefault, SimpleDateFormat, format, toLong, formatDate |

### Imports
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.lazy.LazyColumn`
- `import androidx.compose.foundation.lazy.items`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.filled.Block`
- `import androidx.compose.material.icons.filled.Delete`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.ui.viewmodels.ConversationsViewModel`
- `import java.text.SimpleDateFormat`
- `import java.util.*`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt (1 chunks, 489 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt: 3 functions; 32 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ChatScreen` | 38 | LaunchedEffect, rememberLazyListState, setPeer, collectAsState, rememberCoroutineScope, hiltViewModel, loadConversation, remember |
| `MessageBubble` | 430 | padding, Text, Column, fillMaxWidth, clip, background, RoundedCornerShape, formatTimestamp, Box |
| `formatTimestamp` | 464 | Date, getDefault, SimpleDateFormat, toEpochMillis, format, formatTimestamp |

### Imports
- `import androidx.compose.foundation.background`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.lazy.LazyColumn`
- `import androidx.compose.foundation.lazy.items`
- `import androidx.compose.foundation.lazy.rememberLazyListState`
- `import androidx.compose.foundation.shape.CircleShape`
- `import androidx.compose.foundation.shape.RoundedCornerShape`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.automirrored.filled.Send`
- `import androidx.compose.material.icons.filled.Block`
- `import androidx.compose.material.icons.filled.CheckCircle`
- `import androidx.compose.material.icons.outlined.ChatBubbleOutline`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.draw.clip`
- `import androidx.compose.ui.graphics.Color`
- `import androidx.compose.ui.text.font.FontWeight`
- `import androidx.compose.ui.unit.dp`
- `import androidx.compose.ui.unit.sp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.ui.chat.MessageInput`
- `import com.scmessenger.android.ui.viewmodels.ChatViewModel`
- `import com.scmessenger.android.ui.viewmodels.ContactsViewModel`
- `import com.scmessenger.android.ui.viewmodels.ConversationsViewModel`
- `import com.scmessenger.android.utils.toEpochMillis`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt (1 chunks, 451 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt: 3 functions; 33 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `DiagnosticsScreen` | 39 | refreshLogs, emptyList, LaunchedEffect, getNetworkDiagnosticsReport, mutableStateOf, rememberCoroutineScope, getDiagnosticsLogs, hiltViewModel, PerformanceMonitor, getNotificationStats |
| `refreshLogs` | 57 | refreshLogs, isServiceHealthy, LaunchedEffect, getNetworkDiagnosticsReport, getAllAnrEvents, WarningBanner, getDiagnosticsLogs, getHealthSummary, getHealthStatus, getAnrStats |
| `shareDiagnosticsBundle` | 387 | startActivity, addFlags, shareDiagnosticsBundle, File, Intent, putExtra, createChooser, getUriForFile, writeText |

### Imports
- `import android.content.Context`
- `import android.content.Intent`
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.rememberScrollState`
- `import androidx.compose.foundation.verticalScroll`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.automirrored.filled.ArrowBack`
- `import androidx.compose.material.icons.filled.Delete`
- `import androidx.compose.material.icons.filled.Refresh`
- `import androidx.compose.material.icons.filled.Share`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.platform.LocalContext`
- `import androidx.compose.ui.text.font.FontFamily`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.network.DiagnosticsReporter`
- `import com.scmessenger.android.service.AnrEvent`
- `import com.scmessenger.android.service.PerformanceMonitor`
- `import com.scmessenger.android.service.ServiceHealthMonitor`
- `import com.scmessenger.android.ui.components.ErrorState`
- `import com.scmessenger.android.ui.components.InfoBanner`
- `import com.scmessenger.android.ui.components.WarningBanner`
- `import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleFormatter`
- `import com.scmessenger.android.ui.diagnostics.DiagnosticsBundleInput`
- `import com.scmessenger.android.ui.dialogs.NetworkStatusDialog`
- `import com.scmessenger.android.ui.viewmodels.SettingsViewModel`
- `import com.scmessenger.android.utils.NotificationHelper`
- `import kotlinx.coroutines.launch`
---

## android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt (1 chunks, 440 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt: 3 functions; 21 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `OnboardingScreen` | 27 | mutableListOf, collectAsState, add, hiltViewModel, rememberMultiplePermissionsState, toList |
| `ConsentInfoItem` | 299 | Text, size, fillMaxWidth, spacedBy, ImportContactDialog, Row, Icon |
| `ImportContactDialog` | 329 | Text, Column, OutlinedTextField, fillMaxWidth, spacedBy, copy, heightIn, AlertDialog |

### Imports
- `import androidx.compose.foundation.layout.*`
- `import androidx.compose.foundation.rememberScrollState`
- `import androidx.compose.foundation.text.KeyboardActions`
- `import androidx.compose.foundation.text.KeyboardOptions`
- `import androidx.compose.foundation.verticalScroll`
- `import androidx.compose.material.icons.Icons`
- `import androidx.compose.material.icons.filled.CheckCircle`
- `import androidx.compose.material.icons.filled.Lock`
- `import androidx.compose.material.icons.filled.Shield`
- `import androidx.compose.material.icons.filled.Warning`
- `import androidx.compose.material3.*`
- `import androidx.compose.runtime.*`
- `import androidx.compose.ui.Alignment`
- `import androidx.compose.ui.Modifier`
- `import androidx.compose.ui.platform.LocalFocusManager`
- `import androidx.compose.ui.text.font.FontFamily`
- `import androidx.compose.ui.text.input.ImeAction`
- `import androidx.compose.ui.text.style.TextAlign`
- `import androidx.compose.ui.unit.dp`
- `import androidx.hilt.navigation.compose.hiltViewModel`
- `import com.scmessenger.android.ui.viewmodels.MainViewModel`
---

## android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt (1 chunks, 152 lines)
Function `[VALIDATED]_P0_ANDROID_AGY_HANDOFF_2026-06-07_Identity_Stability_Bundle` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt: Defines 1 types: IdentityViewModel; 5 functions; 10 imports

### Structs/Classes
- IdentityViewModel

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `loadIdentity` | 62 | d, createIdentity, identity, catch, launch, getIdentityInfoNonBlocking, w, e |
| `createIdentity` | 91 | createIdentity, isNotBlank, catch, launch, i, setNickname, getIdentityInfoNonBlocking, e |
| `getQrCodeData` | 126 | getIdentityExportString, catch, withContext, e, clearSuccessMessage, clearError |
| `clearSuccessMessage` | 142 | clearError |
| `clearError` | 149 |  |

### Imports
- `import androidx.lifecycle.ViewModel`
- `import androidx.lifecycle.viewModelScope`
- `import com.scmessenger.android.data.MeshRepository`
- `import dagger.hilt.android.lifecycle.HiltViewModel`
- `import javax.inject.Inject`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.flow.*`
- `import kotlinx.coroutines.launch`
- `import kotlinx.coroutines.withContext`
- `import timber.log.Timber`
---
