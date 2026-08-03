# GPT -> Windows/Claude: five-node install and reset gate

Status: Mac lane ready; Android reset decision requested
Date: 2026-08-03

## Mac evidence

- `origin/main` is `0e4b6cdc` (PR #132 merged).
- The iOS app was built from that exact commit for a physical device.
- Build metadata is `0.5.0`, bundle build `9`.
- The app was installed and launched successfully on Christy's iPhone; the
  process is live.
- The merged iOS source contains the dual-stack LAN discovery change: it
  browses and publishes `_p2p._udp` plus the legacy `_scmessenger._tcp` type.
- The macOS CLI was built from the same `0e4b6cdc` source and is live. Its
  control API is bound on loopback port `9876`; its listener set was verified
  from the owning process and the CLI emitted real listener lines for the P2P
  and WebSocket surfaces.

The CLI reports the known platform limitation that desktop BLE peripheral
advertising is not implemented. It can still participate through LAN, relay,
and its normal desktop transports; do not treat that warning as a failed
binary or claim that the desktop node is a BLE advertiser.

## Fresh install versus identity reset

The current five-node handoffs require fresh binaries and fresh processes, not
an identity wipe. In particular, the paired capture request says not to change
identities or contacts during the window. A data wipe would change the public
keys and invalidate contact/identity attribution, so it is not authorized by
this handoff.

Windows/Claude should therefore:

1. Build or install the current Android artifact from the merged mainline.
2. Stop/force-stop the old process, install the fresh APK, and launch it so
   BLE initialization and GATT registration run again.
3. Clear or rotate only the diagnostic capture/log buffer immediately before
   the shared UTC window.
4. Preserve the existing identity and contacts unless the operator explicitly
   authorizes a separate clean-identity test with new contact provisioning.
5. Prove live GATT server registration and active advertising from stack state,
   then return the exact UTC window and Android build/source provenance.

If Windows believes a true clean-data install is required, pause before doing
it and reply with the reason, the expected acceptance change, and whether the
identity/contact migration test will be run separately. Do not silently wipe
either phone.

## Start condition for the shared window

Start only after Windows confirms the post-#132 Android process is fresh, GATT
is registered, advertising is active, and the Android log capture is clean.
Then provide the UTC start/stop window to the Mac lane. GPT will correlate the
iOS capture against that window without exposing device identifiers, keys,
addresses, IPs, or message bodies.

Acceptance remains directional and receiver-based: both iOS -> Android and
Android -> iOS must show recipient processing and a sender-observed receipt,
then both directions must pass once more after restarting the two phone apps.
