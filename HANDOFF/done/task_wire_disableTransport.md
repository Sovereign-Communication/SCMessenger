<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

TARGET: android\app\src\main\java\com\scmessenger\android\transport\TransportManager.kt

SYSTEM DIRECTIVE: COMPREHENSIVE DEAD-END RESOLUTION
The function 'disableTransport' is defined in 'android\app\src\main\java\com\scmessenger\android\transport\TransportManager.kt' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open 'android\app\src\main\java\com\scmessenger\android\transport\TransportManager.kt' and read the implementation of 'disableTransport'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where 'disableTransport' MUST be invoked to work fully.

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

================================================================================
INTEGRATION SUMMARY
================================================================================

ANALYSIS:
---------
The 'disableTransport' function in TransportManager.kt is a method that disables
a specific transport (BLE, WiFi Aware, WiFi Direct) at runtime. The function
exists but is never called because:
1. TransportManager was defined but never instantiated in MeshRepository
2. Settings changes via SettingsViewModel had no mechanism to call disableTransport

INTEGRATION PLAN:
-----------------
Files to modify:
1. android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt
   - Add imports: TransportManager, TransportType
   - Add transportManager field
   - Initialize TransportManager in initializeManagers()
   - Add applyTransportSettings() method to call enableTransport/disableTransport

2. android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt
   - Update updateBleEnabled() to call meshRepository.applyTransportSettings()
   - Update updateWifiAwareEnabled() to call meshRepository.applyTransportSettings()
   - Update updateWifiDirectEnabled() to call meshRepository.applyTransportSettings()
   - Update updateInternetEnabled() to call meshRepository.applyTransportSettings()
   - Update updateRelayEnabled() to call meshRepository.applyTransportSettings()

3. android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt
   - Add applyTransportSettings() wrapper method for settings changes

EXECUTION:
----------
1. Added imports for TransportManager and TransportType to MeshRepository.kt
2. Added transportManager field to MeshRepository
3. Initialized TransportManager in initializeManagers() with proper callbacks
4. Added applyTransportSettings() method that:
   - Compares current settings to determine which transport changed
   - Calls enableTransport() or disableTransport() on TransportManager
5. Updated SettingsViewModel settings update methods to call applyTransportSettings()
6. Added applyTransportSettings() wrapper in MeshServiceViewModel

BUILD STATUS:
-------------
Android: BUILD SUCCESSFUL
Rust: cargo check --workspace completed successfully

STATUS: SUCCESS_STOP


--- CLOSEOUT EVIDENCE ---
VERIFIED WIRED: Called at MeshRepository.kt:4777,4788,4799. Production call path confirmed.
Verified by: orchestrator-TRP-2026-05-03
