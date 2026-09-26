# SCMessenger Completion Plan - LoC Based Execution

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Current Codebase Metrics
- **Total LoC**: 170,948
- **Rust Core**: 55,858 LoC (core/)
- **CLI**: 6,879 LoC (cli/)  
- **WASM**: 5,362 LoC (wasm/)
- **Android**: 35,185 LoC (android/)
- **Other Kotlin**: 8,892 LoC

## Immediate Execution Tasks (P0)

### 1. Android JAVA_HOME Configuration (~50 LoC)
**Location**: Environment setup
**Action**: Configure JAVA_HOME for Android builds
**Handoff**: System configuration task

### 2. Core Unused Code Cleanup (~1,200 LoC)
**Files**: core/src/lib.rs, core/src/transport/swarm.rs, core/src/transport/behaviour.rs
**Actions**:
- Remove unused imports (CoverTrafficGenerator, AuditEvent, Toggle)
- Remove unused constants (STORAGE_SCHEMA_VERSION, LEGACY_* keys)
- Remove unused functions (read_schema_version, build_registration_request)
- Remove unused variables (multiport_config, storage_path, headless, path)
- Fix redundant clone calls

### 3. WASM Unused Code Cleanup (~150 LoC) 
**Files**: wasm/src/lib.rs
**Actions**: Remove unused storage_path field from MeshSettingsManager

## Platform-Specific Completion Tasks

### Windows CLI (95% Complete - 6,879/7,240 LoC)
**Remaining**: ~361 LoC
- Integration test stabilization (200 LoC)
- BLE daemon edge case handling (161 LoC)

### WASM Client (85% Complete - 5,362/6,308 LoC) 
**Remaining**: ~946 LoC
- IndexedDB persistence implementation (600 LoC)
- Browser test coverage (346 LoC)

### Android Native (85% Complete - 35,185/41,394 LoC)
**Remaining**: ~6,209 LoC  
- Power management optimization (2,500 LoC)
- JAVA_HOME configuration + build fixes (50 LoC)
- Beta hardening and testing (3,659 LoC)

## Beta Readiness Requirements (P1)

### 1. Anti-Abuse Hardening (~2,500 LoC)
**Location**: Core relay modules
**Scope**: Mandatory relay spam/flood controls across all platforms

### 2. CI/CD Pipeline Setup (~1,200 LoC)
**Location**: GitHub workflows, test infrastructure
**Scope**: Automated testing and deployment pipelines

### 3. Documentation Completion (~800 LoC)
**Location**: docs/ directory
**Scope**: User guides, operator instructions, API documentation

## Execution Strategy

1. **Immediate**: Execute P0 cleanup tasks (1,400 LoC total)
2. **Parallel**: Android JAVA_HOME config + WASM IndexedDB (650 LoC)  
3. **Sequential**: Platform-specific completion tasks (7,516 LoC)
4. **Final**: Beta hardening and documentation (4,500 LoC)

**Total Remaining LoC**: ~13,966 LoC (8.2% of total codebase)

## Handoff Ready
All tasks are documented with clear LoC estimates and file locations. Subagents can execute immediately using existing HANDOFF system.