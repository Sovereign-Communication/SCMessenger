# TASK: Preserve dial error detail instead of collapsing to generic NetworkError

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO, low priority, not blocking. Found via read-only investigation
(`INVESTIGATE_ANDROID_DIAL_NETWORKERROR.md`, moved to done/ alongside this).

## Finding

`core/src/mobile_bridge.rs::dial()` (~line 3148) does
`handle.dial(addr).await.map_err(|_| crate::IronCoreError::NetworkError)`,
discarding the real libp2p `DialError` (self-dial, no-addresses, connection
limit, transport/multiaddr-not-supported, actual IO/timeout) into one
indistinguishable string. `core/src/transport/swarm.rs`'s two
`SwarmCommand::Dial` handlers (~4335 non-WASM, ~4815 WASM — intentionally
duplicated per-platform, not a bug) already have the real `Err(e)` in scope
via `format!("{}", e)` before it's sent back through the reply channel, but
that detail never survives to the FFI boundary or the Android/CLI-visible
error.

Observed impact: an Android emulator session logged 20x
`Failed to dial discovered LAN peer /ip4/10.0.2.15/tcp/9002/ws: Network error`
for its OWN mDNS-advertised address — almost certainly libp2p's routine
`DialError::DialSelf` rejection (self-discovery dialing itself), not a real
network problem, but indistinguishable from an actual failure without this
fix.

## Suggested direction (verify against the ACTUAL installed libp2p version's
`DialError`/`TransportError` enum shape before implementing — do not assume
the variant names below are exact, they were proposed without checking the
crate source)

Add specific `IronCoreError` variants (e.g. `DialSelf`, `NoAddresses`,
`ConnectionLimit`, `MultiaddrNotSupported`, `IoError(String)`) and map real
`DialError` cases to them in `mobile_bridge.rs::dial()`, keeping
`NetworkError` as the fallback for anything else. Callers (Android
`MeshRepository.dial()`, CLI) can then silently ignore `DialSelf` instead of
logging it as a failure, and surface other variants more usefully.

## Do NOT

- Do not change `handle.dial()`/swarm dial logic itself, only the error
  mapping at the FFI boundary.
- Verify the real `DialError` enum shape (`rg -n "enum DialError" ~/.cargo`
  or the workspace's locked libp2p version) before writing match arms —
  don't guess variant names.

## Gate

Standard compile gate + existing dial-related tests green. This is
`core/src/transport/` adjacent (error mapping only, not the dial logic
itself) — confirm with the operator whether the adversarial-review gate
applies before merging, given it's error-handling not crypto/protocol logic.
