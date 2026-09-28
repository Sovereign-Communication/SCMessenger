# Task Verification Template

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

##  Verification Checklist

### For ALL Completed Tasks:
- [ ] **Code Exists**: All implementation files present and compile successfully
- [ ] **Integrated**: Actually called from production code paths (not dormant)
- [ ] **Replaces Old**: Legacy implementations removed/disabled where applicable
- [ ] **Cross-Platform**: All platforms (Android, iOS, WASM, CLI) use the new implementation
- [ ] **Test Coverage**: Integration tests verify actual usage in real scenarios
- [ ] **Performance**: Expected benefits achieved (bandwidth, latency, reliability)
- [ ] **Documentation**: Usage documented in relevant guides and API references

##  Verification Levels

### Level 1: Code Existence Verification
```bash
# Check files exist and compile
find [implementation_path] -name "*.rs" | wc -l
cargo check --features [relevant_features]
./gradlew :app:compileDebugJavaWithJavac
```

### Level 2: Integration Verification  
```bash
# Verify actual usage in production code
grep -r "[KeyClass]\|[KeyFunction]" core/src/transport/
grep -r "use.*[module]" core/src/lib.rs
# Check Android integration
grep -r "[AndroidComponent]" android/app/src/main/java/
```

### Level 3: Functional Verification
```bash
# Runtime behavior verification
cargo test --test "*[feature]*" -- --nocapture
./gradlew :app:connectedDebugAndroidTest
# Verify legacy replacement
grep -r "[legacy_implementation]" core/src/lib.rs | wc -l
```

### Level 4: Cross-Platform Verification
```bash
# Verify all platforms use the new implementation
# Check Android, iOS, WASM, CLI consistency
grep -r "[feature]" android/ ios/ wasm/ cli/
```

##  Task-Specific Verification Requirements

### For Android Tasks:
- [ ] Compiles without errors: `./gradlew :app:compileDebugJavaWithJavac`
- [ ] Integration verified: Critical components wired in `MeshRepository`
- [ ] Tests pass: `./gradlew :app:connectedDebugAndroidTest`
- [ ] No ANR issues: Runtime performance acceptable
- [ ] Cross-transport compatibility: BLE, WiFi, Internet all work

### For Core Rust Tasks:
- [ ] Compiles: `cargo check --workspace`
- [ ] Integrated: Used in `core/src/lib.rs` production paths
- [ ] Tests pass: `cargo test --workspace`
- [ ] Performance: Benchmarks show expected improvements
- [ ] Cross-platform: UniFFI bindings work correctly

### For Security Tasks:
- [ ] Security features compile: `cargo check --features security`
- [ ] Integration: Cryptographic functions used in message processing
- [ ] Tests: Security-specific test suite passes
- [ ] No regressions: Existing security properties maintained
- [ ] Documentation: Security properties documented

### For Network Tasks:
- [ ] Network features compile: `cargo check --features network`
- [ ] Integration: Used in transport layer and bootstrap
- [ ] Tests: Network simulation tests pass
- [ ] Fallback: Multiple transport options work
- [ ] Reliability: Connection stability verified

##  Verification Script Integration

All tasks should include a verification script that can be run with:

```bash
./scripts/verify_task_completion.sh [task_type] [mode]
```

**Modes:**
- `strict` - Exit on first failure (default)
- `report` - Continue on failure, generate comprehensive report
- `validate` - Validation only, no automatic fixes

##  Verification Reporting

After verification, generate a report including:
- Verification timestamp
- Task identifier
- Verification level results (1-4)
- Any failures or warnings
- Cross-platform consistency status
- Performance metrics (if applicable)

##  Immediate Actions for New Tasks

1. **Add this checklist** to every new task definition
2. **Create verification script** for the specific task type
3. **Run verification** before marking task as completed
4. **Update CI pipeline** to include automated verification
5. **Document wiring points** in the task description

---
*This template ensures ZERO dormant implementations and 100% comprehensive task completion*