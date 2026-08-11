# PR #139 GPT-MAC runtime status

RESULT: BLOCKED

HEAD: `1cdbbae6ae0934bd5acab95bb866108a2d05c54e` (live GitHub PR #139 head,
verified from `refs/pull/139/head` on 2026-08-07; the PR packet's embedded
`5b8b8e7b` prose is stale). Local checkout was fast-forwarded to this exact
commit before build.

MACOS_CLI: Built with `cargo build -p scmessenger-cli` after clearing only the
cached CLI/core build artifacts. `target/debug/scmessenger-cli --version`
reported `scm 0.4.0 (1cdbbae6 2026-08-08T01:44:50.759576+00:00)`. At
2026-08-08T01:47:25Z the node launched against the existing data directory,
loaded the existing identity, loaded 64 ledger entries, enabled mDNS, created
the Bluetooth manager, and started a BLE scan. The runtime status later showed
0 connected peers, 38 listeners, no external address, and `Bootstrapping`;
shutdown at 2026-08-08T01:53:53Z was clean. The core log line reported only
`0.4.0` rather than a hash; the CLI stamp is the authoritative macOS binary
provenance captured above.

IOS: Physical device was present and paired over USB. Xcode and `xcdevice`
identified the installed bundle `SovereignCommunications.SCMessenger` at
version `0.5.0`, build `9`. The repository's in-place installer was invoked
with `UNINSTALL_FIRST=0` and reached the real-device `xcodebuild` step after
regenerating current-head Swift/FFI bindings. Build exited 65 before producing
an installable app because Xcode reported exactly: `No Accounts: Add a new
account in Accounts settings.` and `No profiles for
'SovereignCommunications.SCMessenger' were found`. No uninstall or install was
performed; the existing app container was not intentionally modified.

IDENTITY: macOS pre/post checks loaded the same existing identity and showed a
64-hex-character public key; contacts remained 0 and history remained 0 before
and after the clean CLI run. No outbound message was sent, so canonical
outbound sender-id form was not independently verified. iOS identity,
contacts, history, and outbound sender-id preservation remain unverified
because the signed in-place update could not start.

FLEET: Not exercised. The current cloud-node address source was read only from
`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`; no older address was used and no
bootstrap/config edit was made. macOS transport initialization evidence is
limited to BLE manager/scan startup and mDNS initialization. The runtime had
no connected peers, no custody receipts, no message IDs, no message bodies,
and no bidirectional message, receipt, ledger-convergence, LAN/BLE delivery,
or restart/reconnect proof. These are intentionally not claimed.

BLOCKERS: Physical iOS install is blocked by missing Xcode account and
provisioning profile for `SovereignCommunications.SCMessenger` (xcodebuild exit
65). Required next action: sign into the Apple developer account in Xcode and
make a valid development profile available for team `JSZ36WH4C`, then rerun
the same in-place installer with `UNINSTALL_FIRST=0`. After successful launch,
verify iOS identity/contacts/history and canonical outbound sender-id, then
run the two-direction fleet/receipt/transport/restart matrix with sanitized
evidence.

WORKTREE_NOTE: The install preparation regenerated the current-head UniFFI
Swift/FFI files at `iOS/SCMessenger/SCMessenger/Generated/`; those related
platform binding updates are included with this handoff commit. The three
pre-existing untracked audit files were preserved unchanged.
