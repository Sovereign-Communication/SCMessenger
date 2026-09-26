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
The function 'drift_deactivate' is defined in 'core\src\lib.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'core\src\lib.rs' and read the implementation of 'drift_deactivate'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'drift_deactivate' MUST be invoked to work fully. 

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

1. **Rust Core Integration**: 
   - Added `self.drift_deactivate()` to `IronCore::stop()` to ensure graceful protocol suspension on shutdown.
   - Verified functionality via `test_drift_activation` unit test in `core/src/lib.rs`.
2. **Mobile Bridge Wiring**:
   - Integrated into `MeshService::set_relay_budget` in `core/src/mobile_bridge.rs` (called when budget is set to 0).
3. **WASM Integration**:
   - Integrated into `IronCore::update_settings` in `wasm/src/lib.rs` (called when `relay_enabled` is toggled off).
4. **Verification**: Confirmed that all call paths for protocol suspension are correctly wired and that audit events are emitted during these transitions.


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at wasm/src/lib.rs:1011; core/src/mobile_bridge.rs:927. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
