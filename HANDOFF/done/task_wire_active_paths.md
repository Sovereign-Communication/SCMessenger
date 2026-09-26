<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

TARGET: core/src/routing/multipath.rs

Wiring task: 

The function 'active_paths' is defined in 'core/src/routing/multipath.rs' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

PHASE 1: CONTEXT GATHERING
1. Open 'core/src/routing/multipath.rs' and read the implementation of 'active_paths'. Understand its parameters, return type, and exact purpose.
2. Use search tools to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'active_paths' MUST be invoked to work fully.

PHASE 2: THE INTEGRATION PLAN
Write a concise list of exactly which files you are going to modify and where the function will be injected.

PHASE 3: EXECUTION
Wire the function into ALL identified locations. Ensure you add the proper imports to the top of those files.

PHASE 4: TEST & ITERATE
1. Run a localized compiler check (cargo check for Rust, or ./gradlew lint for Kotlin).
2. IF COMPILE FAILS: Enter ITERATION. Read the exact error, fix the syntax or imports, and run the test again.
3. IF SUCCESSFUL: Verify you successfully wired all targets from Phase 2. If the integration is 100% complete and compiles cleanly, output exactly:
STATUS: SUCCESS_STOP
