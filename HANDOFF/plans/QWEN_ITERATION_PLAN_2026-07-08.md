# Qwen Iteration Plan -- Next 2 Iterations

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Created:** 2026-07-08
**Author:** Qwen session (`/scmqwen` orchestrator)
**Purpose:** Concrete plan for the next two iterations of Phase 1 work,
using Qwen models via DashScope (zero Anthropic cost) with round-robin
model selection.
**Sequencing authority:** `HANDOFF/V1_0_0_EXECUTION_PLAN.md`
**Current state:** See updated `HANDOFF/todo/_QUEUE.md`

---

## State Summary (as of 2026-07-08)

### Phase 1 -- Completed (15+ items since 07-06)
- P1-01 through P1-08: All Stage A/B items done (compile gates, ANR fix,
  transport negotiation investigation, mDNS self-loopback, LAN discovery
  wiring, battery receiver fix)
- P1-10: Port-strategy design note done
- P1-11: Listen-side adaptive port selection done (swarm)
- P1-12: Advertise/dial/remember adaptive ports done (swarm)
- P1-13: Hardcode sweep done (swarm)
- P1-15: Transport-matrix ground-truth audit done
- P1-16 BLE TX: CLI outbound write path done + adversarial audit PASS
- P1-16 BLE MAC rotation: done (swarm)
- P1_CORE Rate-limited negotiation failure signal: done
- Clippy debt across CLI/desktop-bridge/WASM: resolved
- Adaptive TTL test race condition: fixed
- NEXT_ITER_01/02/03: All compile gates, adversarial review, docs sync done

### Phase 1 -- Remaining (6 actionable items)
1. **Emulator validation** [INFRA] -- Android emulator install started
   (emulator.exe installed, system image downloading)
2. **NEXT_ITER_04: Live Device Retest / Pairing** [DEVICE] -- Run test
   matrix against emulator (or operator's Pixel if available)
3. **P1-17: Windows WiFi Direct** [HUMAN-GATED] -- Operator must decide
   build-vs-waive
4. **P1-14: Hostile-network validation** [DEVICE] -- After item 2
5. **P1-18: Relay cells** [DEVICE] -- After item 2
6. **P1-19: Phase 1 exit review** -- GATE TO PHASE 2

### Infrastructure
- `/scmqwen` command created -- Qwen orchestrator with round-robin model
  selection across 4 tiers ([FLASH], [CODER], [THINK], [MAX])
- Dispatch helper at `tmp/scmqwen/qwen_dispatch.sh` with state tracking
- 130+ Qwen models available on DashScope, ~1M free tokens/model/90 days
- Android emulator: emulator.exe installed, system image downloading

---

## Iteration 1: Emulator + Baseline Validation

**Goal:** Get Android emulator running, install fresh APK, run the
NEXT_ITER_04 test matrix against it, fix any issues found.

**Timeline:** Same session as emulator install (~15-30 min for download + setup)

### Step 1: Complete Emulator Setup
```bash
# Set ANDROID_HOME (needed by cargo-ndk and gradle)
setx ANDROID_HOME "C:\Users\SCM\AppData\Local\Android\Sdk"
# Add to PATH for this session
export PATH="$PATH:C:/Users/SCM/AppData/Local/Android/Sdk/platform-tools"
export PATH="$PATH:C:/Users/SCM/AppData/Local/Android/Sdk/emulator"

# Wait for system image download (background task b8ybfglbc)
# Then create AVD:
"$SDK/cmdline-tools/latest/bin/sdkmanager.bat" "platforms;android-35"  # ensure installed
"$SDK/cmdline-tools/latest/bin/avdmanager.bat" create avd \
  -n scm_pixel_35 \
  -k "system-images;android-35;google_apis;x86_64" \
  -d pixel_6a \
  --force

# Boot emulator (headless if no display):
"$SDK/emulator/emulator.exe" -avd scm_pixel_35 -no-snapshot -gpu swiftshader_indirect -no-audio -no-boot-anim &
# Wait for boot:
adb wait-for-device
adb shell getprop sys.boot_completed  # should return "1"
```

**AVD config:**
- Name: `scm_pixel_35`
- Device: Pixel 6a profile
- System image: API 35, Google APIs, x86_64
- GPU: swiftshader_indirect (software, works without hardware)
- Audio: disabled (no need, saves resources)
- Snapshot: disabled (clean boot each time)

### Step 2: Build & Install Clean APK
```bash
# Build the native libraries (all Android targets)
cd C:/Users/SCM/Documents/GitHub/SCMessenger
export CARGO_INCREMENTAL=0
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 build --release -p scmessenger-core

# Build and install the APK
cd android
./gradlew assembleDebug -x lint --quiet
adb install -r app/build/outputs/apk/debug/app-debug.apk

# Grant permissions
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_SCAN
adb shell pm grant com.scmessenger.android android.permission.BLUETOOTH_CONNECT
adb shell pm grant com.scmessenger.android android.permission.ACCESS_FINE_LOCATION
adb shell pm grant com.scmessenger.android android.permission.ACCESS_WIFI_STATE
adb shell pm grant com.scmessenger.android android.permission.CHANGE_WIFI_STATE
adb shell pm grant com.scmessenger.android android.permission.POST_NOTIFICATIONS
```

### Step 3: Run NEXT_ITER_04 Test Matrix (Emulator Edition)

**Test 1: Listener Bind (Issue 1 fix)**
```bash
# After app start with internet enabled
adb shell cat /proc/net/tcp | grep "2329"  # port 9001 in hex
adb logcat -d | grep "Swarm listening on"
```

**Test 2: Rescan ANR (Issue 2 fix)**
```bash
# In Nearby Contacts, hammer "Rescan" repeatedly
adb shell dumpsys dropbox --print data_app_anr | grep -c "SubnetProbe"
# Should show 0 ANR entries
```

**Test 3: Outbound Dial (Issue 4 fix)**
```bash
# With Windows daemon at LISTEN on 9001, check connection
adb logcat -d | grep "ConnectionEstablished"
# Should see real libp2p connection, NOT "Failed to negotiate"
```

**Test 4: BLE Backoff (Issue 3)**
- NOTE: Emulator has no BLE hardware. Skip this test -- it needs the
  physical Pixel or a real BLE adapter.

**Test 5: End-to-End**
```bash
# Start CLI daemon on Windows (separate terminal)
cargo run --release -p scmessenger-cli -- daemon --no-discovery

# On emulator: send a test message to the CLI's peer ID
# Observe via logcat:
adb logcat -d | grep "message_received\|delivery_status"
# Verify receipt on CLI side (check daemon log)
```

### Step 4: Fix Any Issues Found
- For each test failure: capture exact logcat/daemon output
- Dispatch a [CODER] Qwen worker with the failure evidence
- Apply the returned patch, rebuild, retest
- Gate: all applicable tests must pass before moving to Iteration 2

**Success criteria:**
- [ ] Emulator boots and app installs
- [ ] Swarm listener binds successfully (Test 1)
- [ ] No ANR on rescan (Test 2)
- [ ] Outbound dial establishes connection (Test 3)
- [ ] End-to-end message exchange works (Test 5)
- [ ] APK built from current main (fresh .so files)

---

## Iteration 2: Phase 1 Completion + WiFi Direct Decision

**Goal:** Resolve remaining Phase 1 items, conduct exit review,
gate into Phase 2.

**Timeline:** After Iteration 1 succeeds (1-2 hours)

### Step 1: WiFi Direct Decision [HUMAN]
- Present the P1-17 design note (HANDOFF/plans/P1-17_windows_wifi_direct_design.md)
  to the operator with a summary of Section 2 options
- **Option A (build):** Implement legacy-client path (Windows joins
  Android GO SoftAP as standard Wi-Fi client via netsh wlan, then
  TCP-dials group owner). Scope: CLI credential ingress + join + dial.
  Estimated: 2-3 Qwen [CODER] dispatches.
- **Option B (waive):** Narrow the Phase 1 matrix WiFi Direct cell to
  Android<->Android [BLOCKED-HW] and record the waiver.
- **Default recommendation:** Waive for v1.0.0. The WiFi Direct cell
  is Android-to-Android by physics (NAN), and the Android<->Windows
  equivalent is already covered by mDNS/LAN + TCP. Building legacy-client
  support adds CLI complexity for a cell that the operator's single-Pixel
  test scenario doesn't exercise. Defer to v1.1.

### Step 2: Hostile-Network Validation (P1-14)
- Run firewall profile tests on Windows:
  1. Block 9001/9002 inbound; verify communication shifts to 443/80
  2. Allow only 443; verify delivery via 443
  3. Allow only 80; verify delivery via 80
- Test emulator on different network profiles (LAN vs hotspot simulation)
- Verify adaptive port ladder behavior
- **Note:** This test may be limited on the emulator since the emulator
  uses the host's network stack. A physical device is needed for true
  network switching. If emulator tests are inconclusive, document and
  defer to physical-device validation.

### Step 3: Relay Cells Validation (P1-18)
```bash
# Terminal 1: CLI in relay mode
cargo run --release -p scmessenger-cli -- relay --name test-relay

# Terminal 2: CLI as regular node connecting to relay
cargo run --release -p scmessenger-cli -- daemon --bootstrap <relay-peer-id>

# On emulator: install, start mesh, go offline, then back online
# Verify custody: message sent while emulator is offline arrives when it returns
adb shell am force-stop com.scmessenger.android  # "go offline"
# Wait, then restart
adb shell am start -n com.scmessenger.android/.ui.MainActivity
adb logcat -d | grep "receive_message\|delivery_status"
```

### Step 4: Phase 1 Exit Review (P1-19)
1. Run `finalize-checklist` skill over all Phase 1 changes
2. Run `cargo test --workspace --no-run` (compile gate)
3. Run `cargo clippy --workspace -- -D warnings` (lint gate)
4. Run `./gradlew assembleDebug` (Android gate)
5. Run `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` (WASM gate)
6. Run `cargo fmt --all -- --check` (format gate)
7. Run `scripts/docs_sync_check.sh` (docs gate)
8. Dispatch [THINK] Qwen worker for release-gatekeeper review
9. Compile the Phase 1 exit matrix with all cells green/waived
10. Present final status to operator for Phase 2 gate approval

**Success criteria:**
- [ ] All build gates pass
- [ ] Phase 1 matrix cells green or operator-waived
- [ ] All adversarial review verdicts on file
- [ ] Documentation synced and headers updated
- [ ] HANDOFF state machine consistent (todo/ only has active items)
- [ ] Operator signs waivers and approves Phase 2 entry

---

## /scmqwen Dispatch Guide (for this plan)

When executing either iteration, use `/scmqwen` as follows:

### Pre-dispatch Validation (cheap, orchestrator-local)
```
/scmqwen triage <task-file>
```
Runs ALREADY_WIRED / FALSE_POSITIVE / NEEDS_REVIEW check without
dispatching to any Qwen model.

### Implementation Tasks
```
/scmqwen <task-file>
```
Dispatches a [CODER] Qwen worker (round-robin from coder tier) with
the task specification. Returns PATCH format output.

### Analysis/Review Tasks
```
/scmqwen analyze <file-or-diff>
```
Dispatches a [THINK] Qwen worker for root-cause analysis or
adversarial security review. Returns structured findings.

### Mechanical Tasks
```
/scmqwen fix <mechanical-item>
```
Dispatches a [FLASH] Qwen worker for doc headers, strings.xml moves,
HANDOFF hygiene, renames.

---

## Slam-Dunk Fixes Identified (do immediately, no dispatch needed)

1. **ANDROID_HOME not set** -- Add to system environment:
   `setx ANDROID_HOME "C:\Users\SCM\AppData\Local\Android\Sdk"`
   (cargo-ndk and gradle both read this)

2. **adb not in PATH** -- Add to PATH:
   `setx PATH "%PATH%;C:\Users\SCM\AppData\Local\Android\Sdk\platform-tools"`

3. **~55 VALIDATED items in todo/** -- These are historical records
   from prior sessions. Batch-move to `HANDOFF/done/[VALIDATED]/`
   directory to clean the queue signal.

4. **No Android Studio installed** -- Not a blocker. All SDK tooling
   (sdkmanager, avdmanager, adb, emulator) works without Android Studio.
   Document this in REMAINING_WORK_TRACKING.md.

---

## Risk Register for These Iterations

| Risk | Impact | Mitigation |
|------|--------|------------|
| System image download fails/interrupted | Blocks Iteration 1 | Re-run sdkmanager; try non-Google APIs image as fallback |
| Emulator boot fails (GPU driver issue) | Blocks Iteration 1 | Use `-gpu off` (software rendering, slower but works) |
| APK install fails on emulator | Blocks testing | Check minSdk compatibility; verify .so ABI match |
| BLE tests impossible on emulator | Gaps test coverage | Skip BLE tests; document as physical-device-only |
| WiFi Direct needs physical hardware | Can't test cell | Recommend waiver for v1.0.0 |
| Hostile-network tests limited on emulator | Gaps test coverage | Document limitation; defer to physical device |
| Relay 3-node test resource-heavy on Windows | Performance | Test sequentially; use lightweight CLI instances |
