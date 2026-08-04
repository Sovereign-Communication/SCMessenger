# GPT -> Nemotron/Windows: clean-install status

Date: 2026-08-04  
Related handoff: `HANDOFF/audit/PR134_REMAINING_TASKS.md`  
Status: macOS clean; iOS install present but launch trust is blocked

## macOS lane: verified clean

- Built from merged `origin/main` at `ba362cc5`.
- Version reported by the binary: `0.4.0`.
- Started with a brand-new isolated HOME and `SCM_AUTO_REPLY=1`.
- `history-count`: **0**.
- `contact list`: **No contacts yet**.
- The node is running and responding as the active macOS driver.
- The previous CLI process was left undisturbed; the clean node uses separate
  local state.

This is a clean state, not a parity pass. It still needs the shared run-2
nickname and directional matrix after the identity fix is in the installed
artifacts.

## iOS lane: install done, clean reset not yet proven

- Existing SCMessenger was uninstalled successfully, removing its app data.
- A new signed `0.5.0` build 9 was installed.
- The app launch was rejected by iOS before application code ran, so the
  explicit reset marker was not emitted and the Keychain identity cannot yet be
  declared cleared.
- The local Apple Development signing identity is not valid/trusted on this
  Mac (`security find-identity` reports zero valid identities). Xcode regenerated
  a device profile, but that alone does not make the app launchable.
- No contacts/history/identity claim is made for iOS until the app launches and
  the reset path is verified.

## Required next action

Use one of these two authorized paths:

1. On the iPhone, trust the Apple Development identity for this app/developer
   in Settings, then launch the installed build with the explicit factory-reset
   test argument and verify the reset marker, empty contacts, empty history, and
   a newly generated identity; or
2. Provide a valid signed/TestFlight/GitHub artifact that the device already
   trusts, install it, and run the same in-app full reset verification.

After the reset is verified, pull the fresh iOS diagnostics/core logs before
any pairing. Then continue PR #134's run-2 sequence: identity-fix artifact,
fresh Android/iOS pairing, release macOS auto-reply driver, and the complete
N-by-N BLE/LAN/relay matrix.

Temporary reset hooks and raw logs remain outside the repository. No physical
parity claim is made.
