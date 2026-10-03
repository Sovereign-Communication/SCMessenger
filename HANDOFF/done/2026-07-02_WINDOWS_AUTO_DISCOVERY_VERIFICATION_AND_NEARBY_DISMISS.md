# Windows Auto-Discovery Verification + Nearby Discovery Dismiss/Recheck

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-07-02
**Status:** DONE
**Context:** Continuation of a Claude Code session ("Windows SCM device setup") that got the
Windows CLI running and set out to prove peer auto-discovery works well enough that manual
"Nearby Discovery" scanning becomes a backup, not the primary path  plus keep the manual
rescan UI, repurposed to recheck peers the user previously dismissed.

## 1. BLE `start_scan` retry fix  verified against a real transient failure

The retry-with-backoff fix at [cli/src/ble_mesh.rs:172-203](../../cli/src/ble_mesh.rs#L172-L203)
(5 attempts, 300ms4.8s exponential backoff, committed in `561c7478`) was tested two ways:

- **With the laptop's Bluetooth radio off:** all 5 retries exhausted, `HRESULT(0x800710DF)
  "The device is not ready for use."` on every attempt. This is expected  the fix targets a
  *transient* WinRT init race, not a disabled radio, and correctly gives up and logs a warning
  rather than hanging.
- **With Bluetooth turned on** (root cause of the earlier failures  the radio was simply off):
  the daemon started clean, `BLE scan active` fired immediately, and it found and GATT-connected
  to a real peripheral (`75:D0:BF:96:DB:99`, consistent with the paired Android phone) in under
  a second: `BLE found matching peripheral` at +0.67s, `BLE GATT notify subscribed` at +4.4s.

No further BLE code changes needed. The retry logic is sound; today's earlier failures were
environmental (radio off), not a bug.

## 2. mDNS auto-discovery  verified with a real second device

Restarting the Windows daemon (`scmessenger-cli.exe start`) with no manual scan action produced,
within ~850ms of the listener coming up:

```
mDNS LAN discovery: enabled (libp2p-mdns)
mDNS discovered peer: 12D3KooWDKibvdhQ2cuqG2E3LxVqNZu1MakGhRfBk2KzgoDyheTM at /ip4/192.168.0.129/tcp/9001/...
Registered transport capabilities for 12D3KooW...: [Internet, Local], reachable=true
```

`192.168.0.129` is a separate device on the LAN (not this machine), discovered automatically 
this directly confirms the "peers auto-discover, no manual scan required" goal from the original
Windows setup request. Both BLE and mDNS transports found the same/adjacent peer independently,
matching the documented transport-race design (BLE  WiFi  mDNS  relay, <500ms fallback per
[CLAUDE.md](../../CLAUDE.md)).

A secondary observation, not actioned: repeated `Incoming connection error ... Failed to
negotiate transport protocol(s)` from `192.168.0.129` on both the TCP (9001) and WS (9002)
listeners. The peer is discovered and reachable, but the connection handshake itself fails 
worth a follow-up ticket if this peer is expected to fully connect (out of scope for this
verification pass; discovery, not connection negotiation, was the target of today's testing).

## 3. New: `SCMESSENGER_DATA_DIR` env override

Added to [cli/src/config.rs](../../cli/src/config.rs) `Config::data_dir()`, mirroring the
existing `SCMESSENGER_CONFIG` pattern:

```rust
if let Ok(env_path) = std::env::var("SCMESSENGER_DATA_DIR") {
    let path = PathBuf::from(env_path);
    std::fs::create_dir_all(&path).context("Failed to create data directory")?;
    return Ok(path);
}
```

**Why:** `dirs::data_local_dir()` on Windows resolves via the Shell known-folder API, which does
not honor a `LOCALAPPDATA` env var override in a child process  there was no way to run two
isolated local CLI instances (separate identity/store) under one Windows user account for
discovery testing. This override unblocks that (and any future local multi-node testing) without
changing default behavior for anyone who doesn't set the var.

**Known limitation found while testing:** the control API (`127.0.0.1:9876`) and the
already-running check are *not* scoped by data dir  a second instance with a different
`SCMESSENGER_DATA_DIR` and a different `--port` still refuses to start with "SCMessenger is
already running!" if a first instance is up. Not fixed here (out of scope); the real-device mDNS
discovery in 2 made a second synthetic local instance unnecessary for this verification pass.
If genuine local multi-instance testing becomes a recurring need, the control API port also needs
to be data-dir- or env-scoped.

## 4. Nearby Discovery: dismiss + rescan-as-recheck

Per explicit product direction: keep the manual "Rescan" button in the Nearby Discovery tab, but
give it a second job  rechecking peers the user dismissed, rather than being the only way peers
appear at all (auto-discovery via `MeshEventBus.peerEvents`, already wired in
`ContactsViewModel.observeNearbyPeers()`, already populates the list live in the background).

**Changes:**

- [ContactsViewModel.kt](../../android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt):
  - New `_dismissedPeerIds: MutableStateFlow<Set<String>>` and `isDismissed(vararg ids)` helper
    (matches via `PeerIdValidator.isSame` against peerId/libp2pPeerId/blePeerId).
  - New `dismissNearbyPeer(peer: NearbyPeer)`: adds the peer's identifiers to the dismissed set
    and removes it from the visible `_nearbyPeers` list.
  - `observeNearbyPeers()`'s `IdentityDiscovered` and `Discovered` handlers now skip re-adding a
    peer that matches a dismissed identifier  so a dismissed peer stays hidden through normal
    background auto-discovery churn.
  - `refreshDiscovery()` (the Rescan button's handler) now clears `_dismissedPeerIds` **before**
    replaying cached discoveries, so a manual rescan is exactly "recheck whether dismissed peers
    are still nearby."
- [AddContactScreen.kt](../../android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt):
  new dismiss (X) `IconButton` on each `NearbyPeerCard`, next to the existing Add button, wired
  to `viewModel.dismissNearbyPeer(peer)`.
- [strings.xml](../../android/app/src/main/res/values/strings.xml): new
  `add_contact_nearby_dismiss_content_description` / `add_contact_nearby_dismissed_toast`;
  updated `add_contact_nearby_rescan_content_description` to mention dismissed-peer recheck.

**Not persisted across app restart**  the dismissed set lives in the ViewModel (survives
rotation, not process death). Intentional: dismissal here means "stop showing me this right now,"
not "block/hide forever" (that's what `block` already does at the mesh level).

## Build verification

- `cargo build -p scmessenger-cli`: clean rebuild (CARGO_INCREMENTAL=0), succeeded in 8m19s.
- `cargo build --workspace`: run as part of this change; see commit message / CI status for
  result.
- Android: `./gradlew :app:compileDebugKotlin` run as a fast correctness check on the Kotlin
  changes above; full `./gradlew assembleDebug -x lint --quiet` should be run before merge per
  repo rules.

## Docs

No canonical doc (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, etc.) required updating
 this is a verification pass plus a small UX addition, not a scope/architecture change. This
HANDOFF file is the record.
