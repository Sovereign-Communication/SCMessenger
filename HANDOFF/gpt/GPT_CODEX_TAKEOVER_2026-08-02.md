# GPT takeover: 0.4.0 Android / 0.5.0 parity

Date: 2026-08-02

## Current integration set

Branch: `gpt/takeover-integration`

Commits to merge into `main` after the Windows Android gate:

- `6ffe6898` — restore the full functional CLI command layer.
- `a86c109c` — Claude wrap-up handoff and Android accessibility fix.
- `b4721e38` — remove both fake hole-punch success APIs; retain real libp2p DCUtR and relay fallback.
- `f9ea745a` — unify Android/iOS identity routing, use live listener exports, remove the dead APK relay hint, and add Android parser regression tests.
- `e07a9c4e` — format the restored CLI layer.
- `2678422c` — bring the latest `main` handoff into the integration branch.
- `e247a640`, `c4c6a048`, `dbd706a7`, `d255ddc1`, `3f2bc016` — consolidate workflow least-privilege, security remediation, and the iOS lane into this PR branch; the iOS lane's stale static-listener startup code was resolved in favor of the live-listener path above.

The branch is pushed as `origin/gpt/takeover-integration`. The Mac lane must not merge Rust/core changes directly into `main`; Windows owns the final merge and release gate.

## Northstar

0.4.0 is the Android release built from the same functional source as 0.5.0.
0.5.0 is the same behavior with iOS parity. Both platforms must:

1. export `peer_id` as the libp2p routing identity and `public_key` as the canonical contact identity;
2. export only live listener addresses, never a guessed `/tcp/9001` endpoint;
3. preserve routing hints on import and dial only a validated libp2p peer ID;
4. establish a real `ConnectionEstablished` path, then prove encrypted message and receipt delivery in both directions;
5. fall back to a real relay when DCUtR cannot establish a direct path;
6. report truthful logs: queued dial, connected, direct/DCUtR, relay, message receipt, and failure are distinct outcomes.

Freenet is not integrated in this repository. The repo contains only Freenet lessons learned, not a callable implementation or dependency. Do not port the separate approximately 800-line protocol into the release freeze. Use libp2p DCUtR plus relay fallback now; evaluate a separately versioned Freenet adapter after 0.5.0 with its own threat model and interoperability tests.

## Required Windows gates

1. Run a fresh Android Rust build (`cargo ndk` / `buildRustAndroid`) after cleaning stale native outputs, then run Android unit tests and `:app:assembleDebug`.
2. Install the fresh APK on both available Android/iOS test paths as applicable. Do not accept a reused `.so` as evidence.
3. Build/install the iOS target from the same branch. The generic iOS device build already passes on the Mac; physical-device install and runtime remain required.
4. Run the paired matrix with both phones and the always-on cloud node:
   - Android QR -> iOS scan -> Android-to-iOS message and receipt.
   - iOS QR -> Android scan -> iOS-to-Android message and receipt.
   - Restart both apps; repeat with a listener port conflict and a stale address.
   - Confirm direct/DCUtR success when possible and relay fallback when direct is unavailable.
   - Confirm unknown sender/contact approval and no relay node appears as a user contact.
5. Record the message ID, sender/recipient public keys, route peer ID, selected transport, connection event, receipt event, and failure reason for every matrix row.

## Josh log handoff

Josh should not need to reproduce a bug or manually interpret a large log stream. For each failed row, ask him for one exported log bundle from each phone containing:

- app version/build and device model;
- local public key, libp2p peer ID, and live listener list;
- the UTC test start time and message ID;
- `ConnectionEstablished`/disconnect events and selected transport;
- send, decrypt, receipt, and retry lines for that message;
- the exported diagnostics/log file from the app's diagnostics/share action.

Have Josh start the run with the message ID visible on screen, reproduce once, stop, and share the two phone bundles plus the always-on node window for the same UTC interval. This is enough to correlate all three points without asking him to trim or redact logs by hand. Redact private keys, backup passphrases, and message contents before external sharing.

## Remaining blockers

- Android Gradle/device verification is still outstanding on the Mac because no Java runtime is installed; the PR's `Mobile` workflow now runs Android unit tests and `assembleDebug` with the same native-library and binding tasks used by the project, and Windows remains the authoritative device gate.
- Physical two-phone runtime verification is outstanding even though the Rust workspace and generic iOS build pass.
- Verify the always-on cloud node has a current identity, reachable listener, synchronized clock, and logs retained for the test interval.
- Do not merge historical PRs #120, #121, #123, or #124 without re-auditing them against this branch.
