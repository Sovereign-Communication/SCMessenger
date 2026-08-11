# iOS/macOS launch and runtime evidence -- 2026-08-10

Status: ACTIVE -- current iOS app launch succeeds through CoreDevice, but a fresh signed reinstall is blocked by an Xcode team/account mismatch; macOS CLI is installed and supervised with debug logging.
Lane: MAC LANE
Coordination: PR #139, `tracking/pre-v040-tag-work`

## Repository anchors

- Shared checkout: `main` at `fa835584` (clean before this handoff).
- Live PR #139 head: `e5284b7b7af194a53d4207f37d845cc16d2d7c56`.
- Isolated PR-head worktree: `tmp/worktrees/pr139-head-e528`.
- Five-node gate: HELD/CLOSED per latest PR #139 coordination comment.

## iPhone launch evidence

Known paired device: iPhone 15 Pro Max, UDID `<redacted>8D3A`, CoreDevice identifier `<redacted>598F` (short suffixes retained for correlation).

The first current inventory query failed while the phone tunnel was disconnected:

```text
CoreDeviceError 4000
Connection was invalidated
Network error 54: Connection reset by peer
```

After the operator restored the Xcode/device connection, `xcrun devicectl list devices` reported the phone as `available (paired)`, and CoreDevice acquired a tunnel and developer disk image services. No uninstall or re-pair was attempted.

The live installed app was bundle `SovereignCommunications.SCMessenger`, version `0.4.0`, build `9`. A direct CoreDevice launch initially returned success; no uninstall was requested. On the final replay, the exact bundle inventory returned empty and a subsequent launch returned CoreDevice error 10002, `The requested application ... is not installed`. This is consistent with a stale/invalid developer installation being invalidated or removed after launch, while background notification activity came from the prior runtime state. The app-owned background mesh logs still do not prove signed-artifact parity.

## Existing iOS dump

Source: `tmp/ios-device-check/` (not for commit).

- `scmessenger-mesh.log`: 174,480 JSON lines, 45,884,999 bytes.
- Time range: `2026-08-04T06:15:32Z` through `2026-08-10T03:07:16Z`.
- Levels: INFO 87,896; WARN 67,467; ERROR 12,595; DEBUG 6,522.
- 49,144 failed-dial messages, predominantly `Failed to dial the requested peer`.
- Repeated connection-refused errors and dial backoff/dead-peer decisions.
- 21 logged Android relay reservations accepted; 21 logged relay-server acceptance events for that Android peer.
- 35 swarm starts and 34 logged starts with the ChristyLove iOS PeerId.
- The log ends on 2026-08-10 03:07Z; it is not a fresh dump from the current failed launch attempt.

Interpretation: background mesh activity and notifications are independently alive, while stale/self-address candidates and repeated transport failures remain prominent. Receiver-side `inbox_receive` plus exact ACK is still required before claiming delivery.

## Build evidence

- Mac-authoritative unsigned device-target compile: `[OK]` at `tmp/ios-debug-build-escalated.log` using `xcodebuild ... -destination generic/platform=iOS CODE_SIGNING_ALLOWED=NO build`.
- Rust iOS library compiled successfully as part of that gate.
- Signed device build attempts failed before compilation with:

```text
No Accounts: Add a new account in Accounts settings.
No profiles for 'SovereignCommunications.SCMessenger' were found.
```

The local project is configured for Personal Team `JSZ3...6WH4C`, while the installed Apple Development certificate is for team `7FW4...2N396`; Xcode preferences currently expose only the former and no provisioning profile is present. A signed artifact cannot be produced until the account/team is corrected in Xcode. Existing unsigned artifacts must not be installed on the physical phone.

The full-device sysdiagnose and combined CoreDevice diagnose commands were attempted after reconnect. Both returned Apple `CoreDeviceCLISupport.DiagnoseError` code `0`; the combined command produced only a small partial archive containing its diagnostic error log. The successful artifacts for this round are repository-local files under `tmp/`.

## macOS CLI state

- Installed `<user-home>/.local/bin/scmessenger-cli` from PR head `e5284b7b7af194a53d4207f37d845cc16d2d7c56`; artifact fingerprint prefix `6e9d50ba4479`.
- `<user-home>/Library/LaunchAgents/io.scmessenger.cli.plist` is loaded and verified as `gui/<uid-redacted>/io.scmessenger.cli`, with the process running and listeners active.
- `scripts/install.sh` now sets the application-support working directory, `RunAtLoad`, `KeepAlive`, `ThrottleInterval=30`, durable stdout/stderr paths, plist validation, automatic load/verification, and `RUST_LOG=debug`.
- Replacing the binary initially triggered launchd `OS_REASON_CODESIGNING: embedded signature doesn't match attached signature`; an explicit ad-hoc re-sign of the installed binary followed by a reload restored a running service.
- The daemon uses the existing user data directory under `<user-home>/Library/Application Support/scmessenger`; preserve it and do not reset it.
- Fresh macOS debug logs show repeated dial backoffs, self/local peer negotiation failures, and endpoint PeerId changes; these remain Windows-lane/transport blockers, not a supervisor failure.
- Sanitized network correlation retained for cross-lane comparison: `x.x.x.101`, `x.x.x.121`, `x.x.x.126`, `x.x.x.135`, `x.x.x.136`, `x.x.x.139`, and `x.x.x.188`; only IPv4 final octets are exposed.

## Next gates

1. Add the correct Apple ID/team and provisioning profile in Xcode, then produce a signed iOS build and install in place.
2. Capture fresh signed-app version/build/PeerId/identity-history continuity and app-owned diagnostics; do not wipe or re-pair.
3. Run matched iOS/macOS/Windows/Android probes with receiver-side evidence and exact route classification.
4. Have Windows and Mac independently validate the same frozen SHA, transport/PeerId behavior, and receiver-backed receipts.
5. Mirror the evidence in PR #139 before any five-node gate advancement; the current gate remains HELD/CLOSED.
