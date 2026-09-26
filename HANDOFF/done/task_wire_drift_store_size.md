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
The function 'drift_store_size' is defined in 'core\src\lib.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'core\src\lib.rs' and read the implementation of 'drift_store_size'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'drift_store_size' MUST be invoked to work fully. 

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

1. **Rust Core Integration**: Verified `drift_store_size()` reporting in `IronCore` via unit tests.
2. **Mobile Bridge Wiring**: Added `drift_store_size` to the `export_diagnostics` output in `core/src/mobile_bridge.rs` for platform-level telemetry.
3. **WASM Integration**: Exposed `getDriftStoreSize` via `wasm_bindgen` in `wasm/src/lib.rs` to allow browser-side storage monitoring.
4. **CLI Integration**:
   - Included `store_size` in the `DriftStatusResponse` within `cli/src/api.rs`.
   - Updated `scm status` in `cli/src/main.rs` to show the number of messages currently held in the mesh store.
5. **Verification**: Confirmed that mesh store occupancy is now correctly reported across all high-level application interfaces.


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at wasm/src/lib.rs:871; cli/src/api.rs:551,652; core/src/mobile_bridge.rs:409. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
