# Implementation Plan: CLI Discovery Parity

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Achieve feature parity between the SCMessenger CLI and the Android application by exposing discovery controls (BLE, mDNS, WiFi-Aware) and manual scan triggers.

## User Review Required

> [!IMPORTANT]
> **Discovery Defaults**: BLE and mDNS will be enabled by default in the CLI configuration to match the "Aggressive Discovery" philosophy of the mesh.
> **Platform Limitations**: WiFi-Aware and BLE behavior may vary across Windows, Linux, and macOS due to driver support. We will use the existing core abstractions to handle cross-platform compatibility.

## Proposed Changes

### [cli](file:///c:/Users/kanal/Documents/Github/SCMessenger/cli)

#### [MODIFY] [cli.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/cli/src/cli.rs)
- Add `Discovery` subcommand to the `Commands` enum.
- Define `DiscoveryAction` enum:
  - `Status`: Lists active discovery transports and their health.
  - `Scan`: Triggers immediate probes (beacons/multicast).
  - `Peers`: Shows a list of peers discovered via local transports (mDNS/BLE) separate from the global DHT.

#### [MODIFY] [config.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/cli/src/config.rs)
- Add `enable_ble: bool` (Default: `true`).
- Add `enable_wifi_aware: bool` (Default: `true`).
- Update `set` and `get` methods to support these new keys.

#### [MODIFY] [main.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/cli/src/main.rs)
- Implement `cmd_discovery` dispatch logic.
- Update node startup logic to respect `enable_ble` and `enable_wifi_aware`.
- Ensure discovery background tasks report status back to the `SwarmHandle` or a shared state.

#### [MODIFY] [api.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/cli/src/api.rs)
- Add JSON-RPC handlers for:
  - `discovery_status`
  - `discovery_scan`
  - `discovery_peers`

### [scripts](file:///c:/Users/kanal/Documents/Github/SCMessenger/scripts)

#### [MODIFY] [core_cli_driver.py](file:///c:/Users/kanal/Documents/Github/SCMessenger/scripts/core_cli_driver.py)
- Add `discovery` wrapper command.
- Update `status` to include discovery health in the JSON output.

## Verification Plan

### Automated Tests
- `cargo test -p scmessenger-cli` to verify CLI argument parsing.
- Unit tests for new config keys.

### Manual Verification
1. **Status Check**: Run `scm discovery status` after starting the node.
2. **Scan Trigger**: Run `scm discovery scan` and tail logs via `scm daemon-log` to verify transport activity.
3. **Peer List**: Verify that `scm discovery peers` shows local LAN/BLE peers even if they aren't in the global contact list.
