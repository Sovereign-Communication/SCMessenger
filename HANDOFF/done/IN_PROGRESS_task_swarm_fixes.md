# Agent Task: Fix transport/swarm.rs + remaining import errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Assigned Model:** gemini-3-flash-preview:cloud
**Task Type:** Quick Fix  Imports & Types

## CONTEXT
After the IronCore struct is created by another agent, there will be remaining errors in:
- core/src/transport/swarm.rs (around lines 1139, 1447, 1475)
- Possible serde derive macro issues in mobile_bridge.rs
- A "invalid reference to positional argument 0" format string error somewhere

## YOUR TASK

### Step 1: READ current state
Start by running cargo check to see the latest error count:
```bash
export PATH="/c/msys64/ucrt64/bin:$PATH"
cargo check -p scmessenger-core 2>&1 > tmp/swarm_errors.txt 2>&1
cat tmp/swarm_errors.txt | head -60
```

### Step 2: Fix transport/swarm.rs errors
Read core/src/transport/swarm.rs around the error line numbers. Fix all missing type imports. This file likely uses types like PeerId, Multiaddr, SwarmHandle without importing them.

### Step 3: Fix serde errors
If there are `cannot find derive macro Serialize/Deserialize` errors, add `use serde::{Deserialize, Serialize};` to the affected file(s).

### Step 4: Fix the "invalid reference to positional argument 0" error
Search for this error in the build output. It's typically a format string like `format!("{}")` with no argument, or a tracing macro with wrong syntax.

### Step 5: Fix any remaining errors
Continue fixing until `cargo check -p scmessenger-core` shows 0 errors.

### RULES
- Use `tracing` not `log` for logging macros
- Use `parking_lot` for Mutex/RwLock not std::sync
- Do NOT modify api.udl, internet.rs, or wifi_aware.rs
- After each fix, re-run cargo check

### VERIFICATION
Final command must show 0 errors:
```bash
export PATH="/c/msys64/ucrt64/bin:$PATH" && cargo check -p scmessenger-core 2>&1 | tail -5
```
Report the final result.
