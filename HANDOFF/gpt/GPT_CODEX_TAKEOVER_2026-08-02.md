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
- `5218554e` — move the iOS lane to the current macOS runner image and make the prebuilt Rust artifact reusable by Xcode.
- `fc166cce` — isolate `MeshBackgroundServiceTests` on the main actor for Swift 6 XCTest compatibility. PR #129 is fully green after this fix.

The branch is pushed as `origin/gpt/takeover-integration`. The Mac lane must not merge Rust/core changes directly into `main`; Windows owns the final merge and release gate.

PR #129 Actions evidence: runs `30762332383` (workspace, FFI, and platform
tests), `30762332398` (Android/iOS bindings and ABI lanes), `30762332390`
(mobile artifacts), `30762332411` (lint), `30762331029` (CodeQL), and
`30762332388` (iOS simulator plus macOS native). All required checks passed,
including iOS build-for-testing and XCTest. Keep the PR open until Windows has
accepted the physical-device evidence below; then merge this branch into
`main` and tag only from the accepted main commit.

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

## Current device evidence and immediate next action

Windows Claude's status sync (`9070c9df`, based on the install request
`217301b7`) verified the Android side on a physical Pixel 6a:

- v0.4.0 / versionCode 14 launches cleanly from a real arm64 Rust cross-build;
  the missing UniFFI symbol is present in both the raw and APK-extracted
  library, and `RoleNavigationPolicyTest` is 3/3.
- Listener truth is confirmed on-device: every advertised port matches a real
  bound port. The f9ea745a live-listener fix is therefore device-verified.
- This is arm64-only evidence; a release build still needs all four Android
  ABIs and a fresh APK install.

The requested iOS update has also been executed from `fc166cce`:

- the signed app reports version `0.5.0`, build `9`, and was installed
  in-place on Christy's paired iPhone, preserving the existing app container;
- the first install attempt hit a transient CoreDevice disconnect, and the
  immediate retry succeeded;
- launch was denied by iOS because the developer profile/app signature has not
  yet been explicitly trusted on the phone. The operator must trust the Apple
  Development profile in iPhone Settings > General > VPN & Device Management
  (and confirm Developer Mode if prompted), then launch
  `SovereignCommunications.SCMessenger`. Do not uninstall: that would erase
  identity, contacts, and history.

After trust, Windows should record the installed CFBundleVersion and commit
SHA, confirm the iPhone publishes both `_p2p._udp` and
`_scmessenger._tcp`, and start the paired matrix. The physical message/receipt
matrix is not complete until both directions succeed; CI green is not a
substitute for this runtime gate.

## Josh log handoff

Josh should not need to reproduce a bug or manually interpret a large log stream. For each failed row, ask him for one exported log bundle from each phone containing:

- app version/build and device model;
- local public key, libp2p peer ID, and live listener list;
- the UTC test start time and message ID;
- `ConnectionEstablished`/disconnect events and selected transport;
- send, decrypt, receipt, and retry lines for that message;
- the exported diagnostics/log file from the app's diagnostics/share action.

Have Josh start the run with the message ID visible on screen, reproduce once, stop, and share the two phone bundles plus the always-on node window for the same UTC interval. This is enough to correlate all three points without asking him to trim or redact logs by hand. Redact private keys, backup passphrases, and message contents before external sharing.

The low-friction contract is one reproduction and one diagnostics export per
phone. Josh should not be asked to grep or interpret logs. The app/operator
should capture the same UTC interval on both phones and the always-on node;
the correlator uses the message ID, public keys, route peer ID, and transport
to join the three bundles. If a failure occurs, report only:

`build/device | UTC start | message ID | sender/recipient public keys | route peer ID | transport | last connection event | last receipt/retry event | exported diagnostics path`

This gives GPT/Claude enough evidence to distinguish discovery, identity,
dial, encryption, receipt, and relay failures without exposing private keys or
message content.

## Claude execution contract: bounded Qwen-first delegation

Claude owns the Windows/device lane and the final merge recommendation. Keep
all code changes on a fresh integration branch or a bounded PR; do not merge
directly to `main`. Use the Windows Qwen free tier as the primary execution
lane wherever the task is deterministic, and have Claude review each result.
Use the existing orchestration materials:

- `HANDOFF/GEMINI_ORCHESTRATOR_QUICKSTART.md`
- `docs/ORCHESTRATION_PLAYBOOK.md`
- `docs/QWEN_QUOTA_LEDGER.md`
- `.claude/commands/scmqwen.md`

Dispatch disjoint tasks with an explicit artifact and acceptance test:

1. Android QR/identity/route audit and the fresh all-ABI build/install.
2. iOS trust/launch, mDNS discovery, QR import, and inbound unknown-sender
   approval verification.
3. Two-phone plus cloud-node matrix, including direct/DCUtR and relay
   fallback, restart, stale address, and port-conflict cases.
4. Josh diagnostics export/correlation and a short failure report for every
   failed row.
5. Security-review follow-up on DNS policy, deep-link multiaddr validation,
   recency clamping, bounded ledger reads, and identity routing.
6. Final convergence report: exact commit, APK/app build numbers, all test
   rows, logs, and merge/tag recommendation.

Each Qwen dispatch must return: files changed, commands run, pass/fail output,
remaining risk, and quota/result entry. Do not spend the 0.4.0 freeze on the
separate Freenet protocol or on speculative auto-dial. The release transport
is real libp2p DCUtR with relay fallback.

## Remaining blockers

- Android Gradle/device verification is still outstanding on the Mac because no Java runtime is installed; the PR's `Mobile` workflow now runs Android unit tests and `assembleDebug` with the same native-library and binding tasks used by the project, and Windows remains the authoritative device gate.
- Physical two-phone runtime verification is outstanding even though the Rust workspace, generic iOS build, iOS simulator XCTest, and Android arm64 device gate pass.
- iOS is installed but not yet launched because Christy's developer profile is awaiting explicit trust on the phone.
- The paired matrix must prove both QR directions, both message/receipt directions, restart persistence, unknown-sender approval, direct/DCUtR, and relay fallback using the same UTC window on both phones and the cloud node.
- Verify the always-on cloud node has a current identity, reachable listener, synchronized clock, and logs retained for the test interval.
- The adversarial security review and deep-link validation review must return a verdict before the Windows merge decision. Parsing/validation may land before auto-dial, but untrusted QR addresses must never trigger unchecked dialing.
- Do not merge historical PRs #120, #121, #123, or #124 without re-auditing them against this branch.

## Release acceptance order

1. PR #129 required checks green (completed).
2. Windows fresh all-ABI Android build and physical install (arm64 evidence is
   complete; all-ABI release evidence remains).
3. iOS profile trusted, current app launched, and build/commit recorded.
4. Execute and archive the two QR directions plus bidirectional message and
   receipt rows.
5. Repeat after restart and with stale/blocked direct paths; prove relay
   fallback and unknown-sender approval.
6. Correlate Josh's two phone bundles with the always-on node, resolve every
   unexplained failure, then merge to `main` and tag 0.4.0/0.5.0.

GPT checks this file and recent commits every 15 minutes while the parity gate
is active. Claude/Windows remains the device owner; any explicit GPT install
request is acted on by the Mac lane and recorded here.
