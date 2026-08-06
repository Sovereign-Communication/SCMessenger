# iOS/macOS post-PR138 rollout status

Date: 2026-08-05 (local)
Owner: GPT backup verifier

## Source

- `origin/main`: `a53dc099607f32871d23e7f6870d5c1d68b9b4ed`
- Functional runtime parent: `6b2573fa` (PR #138)
- The tip commit is documentation-only on top of the merged runtime.

## Rollout evidence

### macOS CLI

- Release build completed successfully from the current main source.
- The CLI was restarted in place using its existing clean-test data directory.
- Startup provenance reported `0.4.0` at the current main source revision.
- Existing identity and peer ledger loaded; auto-reply, control API, and BLE scan started.

### iOS

- Release device build completed successfully from the current main source.
- App was installed over the existing bundle; no uninstall, identity wipe, or data reset was performed.
- Device app inventory reports `SCMessenger` version `0.5.0`, bundle build `9`.
- Normal launch succeeded after the iPhone was unlocked; the app process is running.

## State-preservation rule

This rollout follows the current Windows/Qwen handoff: it is an in-place update, not a fresh identity wipe. Existing identity, contacts, and history must remain available. Do not uninstall, delete app data, or invoke a factory-reset hook for this gate. If the device was intentionally left in first-run onboarding, record that explicitly rather than treating it as a failed preservation check.

## Remaining gate

With the iPhone launch gate complete:

1. Confirm both endpoints show the expected current-main provenance.
2. Run the five-node matrix across iOS, macOS, Android, and the always-on node.
3. Capture paired timestamps for identity, BLE, same-LAN, cloud relay, send, receive, and receipt events.
4. Complete two reproducible bidirectional passes before claiming parity.

No physical parity claim is made by this handoff; the launch and matrix gates remain pending.

Raw logs, peer identifiers, addresses, identity material, and message bodies were not included.
