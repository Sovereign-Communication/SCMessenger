# INVESTIGATE: Android LAN dial fails with generic "NetworkError"

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: read-only investigation (no code changes in this pass).

## Observed symptom

Live Android emulator (emulator-5554), app running ~110 min: Android's
native NsdManager-based mDNS (`_p2p._udp`) successfully advertises and
resolves the app's OWN service (self-discovery), then
`MeshRepository.ensureTransportManager()`'s `onLanAddressResolved` callback
calls `dial(multiaddr)` for the resolved address
(`/ip4/10.0.2.15/tcp/9002/ws` and `/ip4/10.0.2.15/tcp/9001`), and both fail:

```
MeshRepository$ensureTransportManager: Failed to dial discovered LAN peer /ip4/10.0.2.15/tcp/9002/ws: Network error
```

repeated 20x for each address over the session. `dial()` in
`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
(~line 4647) just rethrows whatever `swarmBridge.dial(multiaddr)` (UniFFI
suspend FFI call) throws. On the Rust side,
`core/src/mobile_bridge.rs::dial()` (~line 3148) is:

```rust
pub async fn dial(&self, multiaddr: String) -> Result<(), crate::IronCoreError> {
    let handle = self.handle.lock().clone().ok_or(crate::IronCoreError::NetworkError)?;
    let addr = Multiaddr::from_str(&multiaddr).map_err(|_| crate::IronCoreError::InvalidInput)?;
    handle.dial(addr).await.map_err(|_| crate::IronCoreError::NetworkError)
}
```

`IronCoreError::NetworkError` (`core/src/lib.rs` ~line 53) is a single
generic `#[error("Network error")]` variant with no wrapped detail, and the
`.map_err(|_| ...)` on `handle.dial(addr).await` throws away whatever the
real underlying error was.

`handle.dial()` -> `SwarmCommand::Dial` is handled in
`core/src/transport/swarm.rs` (two call sites around line 4335 and 4815 -
worth checking why there are two).

## Questions to answer (read-only — do not fix yet)

1. Trace `SwarmCommand::Dial`'s handler in `core/src/transport/swarm.rs`
   (both occurrences ~4335 and ~4815 — why two, are they the same event loop
   or two different swarm variants e.g. Android vs desktop cfg?). What real
   error(s) can the underlying `swarm.dial(addr)` (libp2p) return that get
   collapsed into the generic `NetworkError` here? Is there already a
   `tracing::warn!`/`error!` at the swarm-level call site that logs the real
   libp2p `DialError` detail (which might explain why Android's
   `mesh_diagnostics.log` didn't show it in the excerpt already reviewed —
   maybe it's logged at a level/tag not captured, or to a different
   sink)?
2. Is this failure pattern specific to Android self-dialing a loopback-ish
   address it just advertised itself (10.0.2.15 is the emulator's own guest
   interface — dialing yourself is a degenerate case some libp2p transports
   legitimately reject), or would the SAME code path also fail for a dial to
   a genuinely different, reachable peer (e.g. a Windows host reachable via
   an `adb forward`-tunneled TCP port)? Reason from what `handle.dial()` and
   the swarm's transport stack actually check before attempting the TCP
   connect (self-peer-ID filtering, address-family gating, anything Android
   cfg-gates out).
3. Is the blanket `.map_err(|_| NetworkError)` collapsing distinct, actionable
   error cases (DialError::NoAddresses, ConnectionLimit,
   Transport(TransportError::MultiaddrNotSupported), a plain TCP connection
   refused/timeout, etc.) into one indistinguishable string an actual
   observability gap worth its own follow-up ticket? If yes, sketch (do not
   implement) what a better error mapping would look like — this is
   read-only planning only.

## Output format

Plain-text findings (not a diff/patch) answering the three questions above,
each with the specific file:line evidence you traced it from. If you find
you need to read a file not provided in context, say which one and why
rather than guessing at its contents.
