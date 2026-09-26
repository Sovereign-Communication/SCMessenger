<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

TARGET: core\src\lib.rs

SYSTEM DIRECTIVE: COMPREHENSIVE DEAD-END RESOLUTION
The function 'drift_activate' is defined in 'core\src\lib.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'core\src\lib.rs' and read the implementation of 'drift_activate'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'drift_activate' MUST be invoked to work fully. 

PHASE 2: THE INTEGRATION PLAN
Write a concise list of exactly which files you are going to modify and where the function will be injected. 

PHASE 3: EXECUTION
Wire the function into ALL identified locations. Ensure you add the proper imports to the top of those files.

PHASE 4: TEST & ITERATE
1. Run a localized compiler check (cargo check for Rust, or .\gradlew lint for Kotlin).
2. Read the terminal output. 
3. IF COMPILE FAILS: Enter ITERATION. Read the exact error, fix the syntax or imports, and run the test again. 
STATUS: SUCCESS_STOP

## Implementation Evidence

1. **Rust Core Integration**: Added a unit test `test_drift_activation` to `core/src/lib.rs` to verify explicit toggle logic and audit logging.
2. **Mobile Bridge Wiring**:
   - In `MeshService::set_relay_budget`, added logic to call `core.drift_activate()` if budget > 0, and `deactivate` otherwise.
   - In `MeshService::start`, added logic to activate drift if a positive budget is already set.
3. **WASM Integration**:
   - In `IronCore::new` and `IronCore::with_storage`, added logic to sync drift state based on `relay_enabled` setting.
   - In `IronCore::update_settings`, added logic to toggle drift protocol when the relay setting changes.
4. **Verification**: Checked cross-platform call paths to ensure no dead-ends remain for these functions.


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at wasm/src/lib.rs:238,269,300,1009; core/src/mobile_bridge.rs:295,925. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
