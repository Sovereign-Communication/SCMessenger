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
The function 'set_privacy_config' is defined in 'core\src\lib.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'core\src\lib.rs' and read the implementation of 'set_privacy_config'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'set_privacy_config' MUST be invoked to work fully. 

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

1. **Rust Core Integration**: Verified that `set_privacy_config` correctly parses JSON and updates the atomic `PrivacyConfig` with immediate effect on message processing.
2. **CLI Integration**: 
   - Enhanced `scm config privacy` with flags `--padding`, `--onion`, `--cover`, and `--timing`.
   - Allows users to enable/disable features individually (e.g., `scm config privacy --onion true`).
3. **WASM Exposure**:
   - Exposed `setPrivacyConfig` in `wasm/src/lib.rs`.
   - Accepts a JS object and bridges it to the core's JSON-based update path.
4. **Verification**: Confirmed that updating settings via the CLI results in immediate state changes in the core, verified by the `get_privacy_config` path.


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at cli/src/main.rs:1011; wasm/src/lib.rs:910. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
