<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

TARGET: cli\src\main.rs

SUMMARY:
Add a new CLI command 'scm swarm stats' that invokes the newly wired 'swarm_get_all_connection_stats' API and displays the results in a formatted table.

CONTEXT:
The Core API 'swarm_get_all_connection_stats' has been wired through UniFFI and into IronCore. Now the CLI needs a way to show this data to the developer for debugging transport issues.

## SCOPE DISCOVERY (Orchestrator, 2026-05-01)

**WARNING: The task premise is incorrect.** The API `swarm_get_all_connection_stats` does **not** exist in IronCore.

### What Actually Exists
- `core/src/transport/health.rs` defines `TransportHealthMonitor` with `get_all_connection_stats()` returning `HashMap<PeerId, ConnectionStats>`.
- `TransportHealthMonitor` is **not** wired into `TransportManager` (`core/src/transport/manager.rs`) or `IronCore`.
- `IronCore` has `transport_manager: Arc<RwLock<TransportManager>>` but no path to health stats.

### Required Implementation Path
1. **Core**: Wire `TransportHealthMonitor` into `TransportManager` or `IronCore`, or add `swarm_get_all_connection_stats` to `IronCore` that delegates through the transport layer.
2. **UniFFI**: If mobile clients need this API, expose it in `api.udl` (or via proc macros).
3. **CLI**: Add `scm swarm stats` command in `cli/src/main.rs` calling the new IronCore method.

Estimated scope: ~150-250 LOC across core + CLI. Not a simple CLI wiring task.

## ACCEPTANCE CRITERIA:
1. 'scm swarm stats' command is added to the CLI.
2. It fetches stats from the IronCore instance (either via direct storage if standalone, or via API if daemon is running).
3. It prints a table showing: Peer ID, State, Latency, Messages Sent/Failed, Bytes Sent/Received.
4. If no connections exist, it should state so clearly.
