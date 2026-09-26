<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: core\src\mobile_bridge.rs

SYSTEM DIRECTIVE: COMPREHENSIVE DEAD-END RESOLUTION
The function 'compute_relay_adjustment' is defined in 'core\src\mobile_bridge.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'core\src\mobile_bridge.rs' and read the implementation of 'compute_relay_adjustment'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'compute_relay_adjustment' MUST be invoked to work fully. 

PHASE 2: THE INTEGRATION PLAN
Write a concise list of exactly which files you are going to modify and where the function will be injected. 

PHASE 3: EXECUTION
Wire the function into ALL identified locations. Ensure you add the proper imports to the top of those files.

PHASE 4: TEST & ITERATE
1. Run a localized compiler check (cargo check for Rust, or .\gradlew lint for Kotlin).
2. Read the terminal output. 
3. IF COMPILE FAILS: Enter ITERATION. Read the exact error, fix the syntax or imports, and run the test again. 
4. IF SUCCESSFUL: Verify you successfully wired all targets from Phase 2. If the integration is 100% complete and compiles cleanly, output exactly:
STATUS: SUCCESS_STOP

---
### PHASE 2: IMPLEMENTATION LOG
- **Target File**: `core/src/mobile_bridge.rs`
  - Added `AutoAdjustEngine` to `MeshService`.
  - Wired `compute_profile` and `compute_relay_adjustment` into `update_device_state` (the production loop).
  - Computed adjustments are now logged and applied to the relay budget.
- **Target File**: `core/src/lib.rs`
  - Added `AutoAdjustEngine` to `IronCore` for CLI parity.
- **Target File**: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - Consolidated platform and core to use the same `AutoAdjustEngine` instance.

**VERIFICATION**:
- Integration complete across Core, Android, and CLI.
- Overrides are now shared between Platform (over UniFFI) and Core.
- Production loop (device state updates) now triggers adaptive Relay budget tuning.



--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at core/src/mobile_bridge.rs:871. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
