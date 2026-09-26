# BATCH: CLI + WASM bridge wiring (3 tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# AGENT: rust-coder
# MODEL: glm-5.1:cloud
# FALLBACK: qwen3-coder-next:cloud
# TARGET FILES: cli/src/main.rs, cli/src/api.rs, wasm/src/lib.rs

1. **cli_swarm_stats**  Wire swarm stats command into CLI API and daemon status reporting.
2. **get_history_via_api**  Wire history API endpoint into CLI daemon JSON-RPC bridge.
3. **get_identity_from_daemon**  Wire WASM daemon bridge identity retrieval and session management.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.