# PR #134: Remaining Tasks to Resolve All 5-Node Run 1 Findings

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status**: OPEN
**Date**: 2026-08-04
**Baseline**: `origin/main` at `ba362cc5` (post PR #133 merge)

---

## What PR #133 FIXED (from 5-node run 1 analysis)

From `HANDOFF/audit/FIVE_NODE_RUN_1_ANALYSIS.md` "What is still BROKEN" section:

| # | Issue | Status | PR #133 Fix |
|---|-------|--------|-------------|
| 3 | Node-fatal panic: `ConnectionClosed` handled per-PEER instead of per-CONNECTION | **FIXED** | `core/src/transport/swarm.rs`: guarded match arms added |
| 4 | Crash cannot self-recover: stale lock blocks restart | **FIXED** | `scripts/clean_target.sh` + CLI liveness watchdog |
| 5 | CLI nodes cannot reply: no responder | **FIXED** | `cli/src/main.rs`: `--auto-reply` / `SCM_AUTO_REPLY=1` added |

---

## What is STILL BROKEN (NOT fixed by PR #133)

### 1. Identity Keying — BLOCKING (Priority: CRITICAL)
**File**: `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`

**Root Cause**: One peer addressed under two indistinguishable forms:
- `public_key_hex` (64 hex chars) — REQUIRED for encryption
- `identity_id = blake3(public_key)` (64 hex chars) — one-way hash, CANNOT encrypt

The message envelope carries `identity_id` (the hash), but encryption needs the public key. This causes "wrong key" failures.

**Affected Code**:
- `core/src/identity/keys.rs` — both functions return 64 hex chars, indistinguishable
- `core/src/iron_core.rs:706` — `prepare_message_internal` uses `recipient_id` directly as public key
- `core/src/iron_core.rs:712` — sender uses `identity.identity_id()` (hash)
- `core/src/iron_core.rs:3036` — contact lookup uses public key
- `core/src/iron_core.rs:3066, 3090` — blocked checks use `message.sender_id` (hash)
- `android/app/src/main/java/com/scmessenger/android/MeshRepository.kt` — keys contacts by `identity_id` from BLE beacon
- iOS equivalent: `MeshRepository.swift` — same issue

**Required Fix**:
1. Canonicalise contacts on the **public key** everywhere
2. Add `identity_id` → `public_key` index for routing hints
3. Validate on send path: reject recipient_id that matches a known `identity_id`
4. Make the two visually distinct in logs/payloads (prefix/tag)
5. Migration: detect and repair contacts keyed by hash

**Cross-Platform**: iOS MUST agree on same convention. BLE beacon carries BOTH fields.

---

### 2. Directional Decrypt Failure — BLOCKING (Priority: CRITICAL)
**File**: `HANDOFF/audit/DIRECTIONAL_PARITY_DIAGNOSTIC.md`

**Evidence**: Same node pair, same transport:
- Android → Windows CLI: **WORKS** (received, decrypted, 0 failures)
- Windows CLI → Android: **FAILS** (`Failed to decrypt ... wrong key`)

**Per DIRECTIONAL_PARITY_DIAGNOSTIC.md**: This WORKS/FAIL asymmetry rules out connection, discovery, transport, and stale-contact explanations. Points squarely at **identity keying** (item 1 above).

**Resolution**: Fix identity keying first. This may resolve automatically once canonicalization is in place.

---

### iOS Parity Tasks (from GPT_IOS_LANE_KICKOFF.md)

### Task 1: U6 iOS Receipt Unification
**File**: `iOS/SCMessenger/SCMessenger/` — find `CoreDelegateImpl.swift`, `SmartTransportRouter.swift`
- Replace local Swift receipt encode/decode with UniFFI bindings `encodeReceipt`/`decodeReceipt`
- Add `ReceiptUnificationTest.swift`: round-trip through core encode -> decode
- Regenerate XCFramework if UDL surface requires it
- Gate: `xcodebuild build + test PASS`

### Task 2: Swift Relay De-Hardcode + Discarded Bootstrap Bug
**File**: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- Line 129: DELETE `private static let defaultBootstrapRelay = "/ip4/100.56.248.69/tcp/9001"`
- Source bootstrap from ledger (`getPreferredRelays` / `dialableAddresses`)
- **KNOWN BUG**: Line ~848 `bootstrapAddrs` computed but DISCARDED; line ~1062 `startSwarm` receives empty array
- Wire computed addresses into `startSwarm`
- Gate: `xcodebuild build PASS` + boot smoke log showing non-empty bootstrap

### Task 3: D-03 XCTest Target Registration
**File**: `HANDOFF/todo/D-03_iOS_XCTest_target_register_SC.md`
- Register `SCMessengerTests` in `.xcodeproj`
- `xcodebuild test -project iOS/SCMessenger.xcodeproj -scheme SCMessengerTests` must run
- CI workflow `ios-build-test.yml` needs alignment after this lands

---

## Remaining Release Readiness Tasks (from GPT_PLANNING_040_050_VERDICT.md)

### v0.4.0-alpha.1 Critical Path (Section 1.2)

| ID | Task | Owner | Depends On | Size | Evidence |
|----|------|-------|------------|------|----------|
| 040-G0 | Baseline freeze | Windows orchestrator | none | S | Record immutable main, staging parent/tip |
| 040-S1a | Seeding remediation (v2a, v2c, v2b, 1c, packet 2) | qwen via Windows | 040-G0 | L | Race, cap, persistence, determinism tests pass |
| 040-S1b | Complete original finding closure (F2, F3, F6, F7, F10, F12, F13, F16, NEW-6) | qwen via Windows, GPT review | 040-S1a | M | Every operator-mandated finding FIXED or operator-signed release decision |
| 040-S2 | Independent adversarial verdict | GPT, Windows gatekeeper | S1a, S1b | M | Verdict names every finding, SHIP/NO-SHIP line |
| 040-S3 | Windows compile, test, FFI gates | Windows orchestrator | final S1 | M | fmt, clippy, build/test, Android gates, P6 FFI snapshot |
| 040-S4 | Current-head local delivery truth | Windows + Android emulator | final S1 | M | ConnectionEstablished + delivered receipts both directions |
| 040-S5 | Literal Josh WAN proof | Operator + Josh/Lucas | final S1 | M | Hawaii ↔ PA via AWS relay, queued reconnect |
| 040-S6 | Release truth and tag | Windows + operator | S2, S3, S4, S5 | S | Version, CHANGELOG, artifacts, CI, tag at same SHA |

### v0.5.0 iOS-Android Parity (Section 2.3)

| ID | Task | Owner | Depends On | Size | Gate |
|----|------|-------|------------|------|------|
| V050-I0 | Restore committed test truth | Mac Swift | v0.4 branch | S | Fix stale tests, app build + XCTest pass at one SHA |
| V050-I1 | Bindings ratchet | Mac + Windows FFI | I0 | S/M | copy-bindings.sh, assert-generated-path.sh, verify_ios_bindings.sh, P6 FFI, Xcode build/test |
| V050-I2 | Retry-timing parity | Mac Swift | I0 | M | XCTest: initial delay, adaptive schedule, age ceiling, no downgrade |
| V050-I3 | Truthful transport settings | Mac Swift | I0 | S/M | Every iOS control changes real behavior; unsupported Android toggles absent |
| V050-I4 | Receipt state-machine contract | GPT think, qwen core, Mac Swift | I0, S1 free | M | Exact Sent/Delivered/Read/Failed semantics; core/FFI gates |
| V050-I5 | Physical parity matrix | Mac + operator devices | I1-I4 | L | iOS↔Android, iOS↔iOS: BLE, TCP/mDNS/Multipeer, relay, fallback, restart, background |

---

## Security Remediation (from WINDOWS_REQUEST_RELEASE_READINESS_AND_UNIFICATION_2026-07-29.md)

### Agy Packet W1: Seeding Terminal Blockers
- Enforce load size limit before parsing in `ledger_entry.rs`
- Serialize `load()` state replacement with saves
- Multi-manager/process contract: coordination or fail closed

### Agy Packet W2: Security Remediation
- **W2a**: CodeQL #28-#30 (`backup.rs` hard-coded salt) — adjudicate REAL/FALSE_POSITIVE/TEST_ONLY
- **W2b**: Log-visualizer DOM safety (#31-#33) — fix without ad-hoc escaping
- **W2c**: Dependabot alerts — npm `ws`, `path-to-regexp`, `qs`; Rust `rustls-webpki`, `yamux`, `hickory-proto`
- **W2d**: Workflow least privilege — explicit permissions, immutable SHA pinning

### Agy Packet W3: Transport/Settings Authority
- Android `loadSettings()` forcibly turns WiFi flags on
- Android Internet toggle doesn't affect Swarm
- Two independent WiFi Direct manager stacks
- 500ms debounce drops persistence writes
- Core checks PlatformBridge during MeshService.start() but bridge installed after

### Agy Packet W4: Receipt/Notification Single-Source Policy
- Fix core receipt-state wildcard in `iron_core.rs` mapping Read/Failed through Delivered
- Android notification classification → call core `classifyNotification`
- Verify Swift/Kotlin bindings preserve same state names/transitions

### Agy Packet W5: Version/Release Truth
- Fix `scripts/sync_version.sh` (targets obsolete paths)
- Add read-only verifier for tag/Cargo/Android/iOS/WASM/desktop version agreement
- Disable automatic stable tagging (next tag: operator `v0.4.0-alpha.1`)
- Release artifacts fail closed (no debug APK when signing secrets absent)
- iOS release automation truthful (Mac signing, archive/export, Apple account evidence)

### Agy Packet W6: Josh Easy-Install/Debug Plan
- Wait for GPT request, then produce: install path, checksum verification, first-launch steps, diagnostics export, reinstall/identity-backup recovery, debugging decision tree, evidence fields for Hawaii↔Josh test

---

## 5-Node Run 2 Sequencing (from FIVE_NODE_RUN_1_ANALYSIS.md Section "Sequencing for run 2")

1. [OK] Land panic fix and lock-file recovery fix (DONE in PR #133)
2. [OK] Add CLI reply capability (DONE in PR #133)
3. [ ] **Resolve identity keying** — BLOCKED, nothing else matters until settled
4. [ ] Re-pair both phones after any identity change
5. [ ] Confirm GATT + advertising from live stack state, not log absence
6. [ ] Post shared UTC window; every node captures full window
7. [ ] Record every pair directionally, receiver-side evidence only

---

## Cross-Platform Log Bundle Protocol (from WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md)

**Need from Mac lane (iOS + macOS CLI)**:
- iOS: `mesh_diagnostics.log`, core-level tracing (or explicit confirmation it doesn't exist), BLE central markers, decrypt/crypto failures with EXACT wording
- macOS CLI: node log + listener set matched to PID
- Both: shared UTC test window, message UUIDs retained, typed identity fields (`identity_kind=public_key|identity_hash|libp2p_peer_id|ble_uuid`)

**Five Questions Both Sides Must Answer**:
1. Do decrypt failures correlate with specific peer or all peers?
2. Evidence of MORE THAN ONE identity/key form for same logical peer?
3. Does identity-registration failure PRECEDE decrypt failures?
4. Which transports actually carried traffic?
5. Any end-to-end success evidence?

---

## Verification Standards (from GPT_PLANNING_040_050_VERDICT.md Section 3)

### What Counts as Connection/Delivery/Receipt

| Claim | Minimum Evidence | Non-Evidence |
|-------|------------------|--------------|
| Connected | `ConnectionEstablished` for intended peer/address + role + provenance | dial queued, discovery count, socket-open, generic "connected" UI |
| Delivered | Receiver decrypts unique envelope + durably stores exactly one history row | transport ACK, relay custody, HTTP success, sender-side log only |
| Receipt round trip | Receiver creates receipt for message ID; sender core classifies it; platform callback updates history; pending retry removed; Delivered appears only then | locally calling `markDelivered`, synthetic callback, receipt parse test only |
| Recovered | After forced disconnect/restart, queued/custody state drains unattended and converges without duplicates | manual redial, manual DB edit, restarting until it happens |

### Per-PR Gate Matrix

| Change Class | Required Gate |
|--------------|---------------|
| Documentation/tooling | rules check, `git diff --check`, link/path validation, factual review |
| Rust outside protected paths | Windows fmt, clippy, workspace build/test/compile, focused tests |
| `core/src/crypto, transport, routing, privacy` | All Rust gates + independent adversarial verdict + release-gatekeeper review |
| UDL/FFI/public API | Rust gates, P6 snapshot, regenerated Kotlin/Swift, Android build/tests, Mac binding drift/build/XCTest |
| Android/Kotlin behavior | Windows Gradle unit/lint/assemble + emulator/device behavior |
| iOS Swift/project | Mac binding check, app build, full XCTest, physical evidence for radios/background |
| Docker/farm/relay | Config/build validation, immutable image digest, health/readiness, authentic contact/send/receipt test |
| Cross-platform delivery | Both platform build gates, matching provenance, complete delivery/receipt protocol |

---

## Top 5 "Looks Done But Isn't" Risks (from GPT_PLANNING_040_050_VERDICT.md Section 4)

1. **Queued dial reported as connection** — Killer: target-scoped `ConnectionEstablished` + unique payload + receipt
2. **Transport success or malformed receipt becomes Delivered** — Killer: trace one ID through receiver-decrypt → receipt-create → sender-classify → history/outbox/UI; inject malformed receipts
3. **Green compile hides incomplete security fix** — Killer: race/crash tests, sabotage-and-restore, full Windows gates, independent final verdict
4. **Tested source differs from installed artifacts** — Killer: compare build provenance/image digest, install only checksummed artifacts, core-init smoke
5. **Version merge creates wrong release before gates** — Killer: keep version bump terminal, resolve auto-tag in release packet, operator verify prerelease tag/artifact SHA

---

## Delegation Plan

### Qwen Free Tier (Windows Execution Lane)
1. **Identity canonicalization** (core/src/identity/keys.rs, iron_core.rs, MeshRepository.kt) — PRIMARY BLOCKER
2. **Seeding remediation** (ledger_entry.rs v2a/v2c/v2b/1c/packet2)
3. **Security packets W1-W5** (via agy/Gemini delegations)
4. **Transport/settings authority** (W3)
5. **Receipt/notification policy** (W4)
6. **Version/release truth** (W5)
6. **Windows gates execution** (fmt, clippy, build, test, FFI snapshot)

### GPT / Mac Lane (iOS + Adversarial Review)
1. **iOS Tasks 1-3** (U6 receipt unification, relay de-hardcode, XCTest registration)
2. **v0.5.0 iOS parity lane** (V050-I0 through I5)
3. **Adversarial review** of seeding remediation (040-S2)
4. **Adversarial review** of any core/crypto/transport/routing/privacy changes
5. **Log bundle collection** from iOS + macOS CLI (per WINDOWS_LOG_BUNDLE_PROTOCOL)

### Operator (Human Gates)
1. Final merge/tag decisions
2. Apple Developer account / TestFlight for farm pilot
3. AWS/Alibaba cloud rig reopening (P1-14/P1-18)
4. Josh WAN test coordination (Hawaii ↔ Pennsylvania)
5. Auto-tag workflow resolution

---

## Immediate Next Actions (Priority Order)

1. **[CRITICAL]** Identity canonicalization on public key — unblocks everything else
2. **[HIGH]** Seeding remediation completion + adversarial verdict
3. **[HIGH]** iOS U6 receipt unification + relay de-hardcode + XCTest registration
4. **[HIGH]** Security packets W1-W5 via agy delegation
5. **[MED]** Transport/settings authority (W3) + Receipt policy (W4)
6. **[MED]** Version/release truth (W5) + Josh plan (W6)
7. **[MED]** v0.4.0 local delivery proof (040-S4) + Josh WAN proof (040-S5)
8. **[LOW]** v0.5.0 farm sim re-cut + iOS parity implementation

---

## Notes for Delegation

- **Qwen**: Use `scripts/delegate_task.py --provider qwenpaid --model qwen3.8-max-preview` for implementation packets
- **agy/Gemini**: Use for bounded audit/security packets (W1, W2a-d, parts of W3-W5)
- **GPT**: Reserve for adversarial review, cross-cutting strategy, iOS Swift work
- **Windows orchestrator**: Single writer for all build verification; Mac is authoritative for iOS/xcodebuild
- **Never** merge to main without operator approval; never tag without all gates passing
- **All** core/crypto/transport/routing/privacy changes require adversarial review before done