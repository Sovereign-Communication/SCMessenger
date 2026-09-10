> **RESOLVED 2026-09-10 -- moved to done/ with evidence (see below).**
> Evidence (this session, main@45ab59f9): CTO_STATE 2026-08-30 records the fix
> verified in source at `MainViewModel.kt:374` -> `connectToPeer`, with the route
> registered at `MeshApp.kt:412`. The earlier CTO_STATE entry that flagged this
> ticket as resolved-but-unmoved is repeated here so the move is traceable.


# P0 -- Android deep link parses connection addresses but deliberately never dials them

Status: Open -- root cause identified in code, needs operator decision
Filed: 2026-08-18 (CTO lane)
Severity: P0 (blocks out-of-band mobile rendezvous and D4 Android <-> AWS node off-LAN communication)
Gate mapping: **D4** (two devices, no shared network, receipt proven; Android <-> AWS node across networks)
Authority: `POST_TAG_QUEUE.md` Section 2 (Row 1 correction)

## Summary

PR #177 dispositioned `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS` to S4 (post-tag), asserting that:
> "Verified a working in-app path exists independent of the empty bootstrap list: `ui/join/JoinMeshScreen.kt:359-370` parses a join bundle's `bootstrap_peers` and dials via `SwarmHandle.dial()`"

That justification is **VOID**. Investigation shows:
1. `JoinMeshScreen` is unrouted dead code (`[C1_ZERO_CALLERS]`). No user or code path can reach it.
2. The actual reachable in-app path (`MainActivity` -> `MainViewModel.handleDeepLink`) parses addresses and stores `DeepLinkData`, but **deliberately does not dial**, blocked by an unwired TODO pending operator approval (`MainViewModel.kt:361-363`).

Consequently, neither claimed in-app rendezvous path can connect a user. This directly blocks D4 out-of-band seeding on Android.

## Verified Code Evidence

### 1. `JoinMeshScreen` is unreachable dead code

- Static analysis and wiring verification on `origin/main` report:
  `[C1_ZERO_CALLERS] ui/join/JoinMeshScreen.kt:49 - JoinMeshScreen`
  -- zero callers in Kotlin codebase.
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt` has no `composable()` route for `JoinMeshScreen`.
- Its five sub-views (`QrScannerView`, `ParsingView`, `ConnectingView`, `SuccessView`, `ErrorView`) in `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt` are all `C4_TRANSITIVE_DEAD` behind it.
- While the parsing and dial logic at `JoinMeshScreen.kt:359-370` exists in text, it cannot be reached or executed by any user interaction. Uninvokable code is not a working in-app path.

### 2. The reachable deep-link path parses but deliberately does not dial

- **Manifest Declaration:** `android/app/src/main/AndroidManifest.xml:83-89` declares `scmessenger://invite` and `scmessenger://add` deep links for `MainActivity` (restored in PR #176). Verified live on device:
  `pm query-activities -a VIEW -d scmessenger://invite` -> `MainActivity`.
- **Activity Routing:** `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt:148-153` (cold start) and `:348-355` (`onNewIntent`) route `Intent.ACTION_VIEW` to `mainViewModel.handleDeepLink(uri)`.
- **ViewModel Parsing & Sanitization:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:307-359` `handleDeepLink(uri)` parses `public_key`, `libp2p_peer_id`, and multiaddrs from `listeners`, `connection_hints`, `listener`, and `bootstrap`, sanitizes them via `DeepLinkValidator.sanitizeDeepLinkMultiaddrs`, and sets `_pendingDeepLink.value = data`.
- **UI Consumption:** `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt:237` consumes `mainVm.consumeDeepLink()` to populate `AddContactScreen` fields.
- **Unwired Auto-Dial:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:361-363` reads verbatim:
  ```kotlin
  // TODO: Wire auto-dial via MeshRepository.connectToPeer(peerId, addresses)
  // once validation is reviewed and approved by the operator.
  // For now, only parse and expose via DeepLinkData.listeners.
  ```

The reachable path parses and sanitizes the address, but deliberately refrains from dialling `MeshRepository.connectToPeer(peerId, addresses)`.

## Impact on D4 (Android <-> AWS node)

- D4 is defined as a Pixel 6a exchanging a message with the AWS EC2 node across networks (phone dials outbound to the node's public IP).
- D4 execution assumed that the AWS node address can be seeded out-of-band via invite link or QR code.
- Because `JoinMeshScreen` is unreachable and `MainViewModel.handleDeepLink` deliberately does not dial, Android cannot initiate a connection to the seeded out-of-band address.
- S4 deferral of `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS` on the belief that an in-app rendezvous path was functioning is unsupportable.

## Open Decision for Operator (AGENTS.md Rule 9)

Per AGENTS.md rule 9, architecture and security decisions escalate to the operator:

- **Option (i):** Return `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS` to D4 work.
  - *Trade-off:* Eliminates out-of-band dependency by shipping default/configured bootstrap nodes, but expands pre-tag scope.
- **Option (ii):** Approve and wire the auto-dial TODO at `MainViewModel.kt:361-363` (`meshRepository.connectToPeer(peerId, addresses)`).
  - *Trade-off:* Minimal code change (~5 LOC) making deep-link/QR invites functional immediately, but requires operator approval of the connection trigger security surface.
- **Option (iii):** Restore `JoinMeshScreen` route and entry point in `MeshApp.kt` NavHost.
  - *Trade-off:* Restores the dedicated multi-step QR scanning and connection progress UI, but adds navigation complexity and UI testing surface.

## Acceptance Criteria

1. Operator selects direction among Options (i), (ii), or (iii).
2. The chosen architectural path is implemented and verified.
3. Android node receiving an out-of-band rendezvous seed (invite link / QR bundle / bootstrap config) successfully dials the target address.
