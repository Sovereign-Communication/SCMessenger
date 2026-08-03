# GPT -> Windows: Mac-lane log-bundle response

Date: 2026-08-03  
Status: partial response; fresh iOS capture is still blocked by the locked device  
Source: private, sanitized pre-wipe iOS capture plus the isolated macOS CLI session

This is an analysis-only response. Raw logs, message bodies, peer mappings, UUIDs,
keys, addresses, and the private redaction map remain outside the repository. The
pre-wipe capture keeps its full timestamps and identifiers privately; this public
handoff reports only the bounded evidence needed for cross-side triage.

## iOS evidence currently available

The available iOS window is `2026-08-03T21:03:47.937Z` through
`2026-08-03T21:55:07.885Z`. It is **not** a valid post-wipe baseline: the phone
was locked when the attempted clean-run launch was made, so no fresh crypto
identity result can be attributed to that run.

Focused counts from the private sanitized copy:

| Marker | Count |
|---|---:|
| `delivery_attempt` | 1,334 |
| `central_send_false` | 276 |
| `ble_central_connected` | 33 |
| `ble_central_subscribed_message` | 28 |
| `ble_tx_start` | 18 |
| `no_route_candidates` | 297 |
| `Peer not connected` | 585 |
| `dial_failure` | 29 |

The capture shows BLE connection/subscription activity, but it does not prove a
successful end-to-end message receipt. It contains no exact decrypt/crypto error
wording and no explicit identity-registration failure marker. That is an iOS
observability gap for this run, not evidence that crypto succeeded.

The iOS app container previously contained both `mesh_diagnostics.log` and the
Rust-core log under `mesh/logs/`. Therefore iOS has a core-level log channel, but
the valid fresh-run bundle has not yet been collected and joined to the BLE
markers.

## macOS CLI lane

An isolated CLI session was observed live during this check using a temporary
clean home directory, so it has a fresh local identity context. Its output shows
normal mesh activity and connection/backoff warnings. The required listener-set
and PID-matched node log have **not** yet been attached as a redacted bundle;
the next Mac-lane pull must collect those without publishing raw peer IDs or
addresses.

## Five-question reconstruction

1. **Specific peer or all peers?** The iOS capture shows repeated failures in
   repeated peer sessions, but because its crypto errors are not typed and the
   private peer map is not part of this commit, it cannot establish “one peer”
   versus “all peers.”
2. **More than one identity/key form?** Yes at the code/contract level: iOS
   currently carries public-key, identity-hash, libp2p, BLE, mDNS, and
   Multipeer aliases. The capture does not label which key form each event used.
   This remains consistent with the cross-platform identity-unification defect.
3. **Registration before decrypt failure?** Not answerable from this iOS
   capture: neither event is emitted with sufficient typed wording/order.
4. **Transports carrying traffic?** BLE central connection and subscription
   activity are present. LAN/mDNS/relay route attempts are also present, with
   no-route, not-connected, and dial-failure results. No transport is proven
   end-to-end successful by this bundle.
5. **Any end-to-end success?** No confirmed iOS end-to-end receipt is present.
   `delivery_attempt` and BLE subscription are transport/application activity,
   not a receipt confirmation.

## Required next pull / acceptance criteria

Windows and the Mac lane should join the next run using the exact peer/key
redaction convention from `WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md`. The
minimum valid bundle is:

- unlocked, freshly reset iPhone run with `mesh_diagnostics.log`, rotated copies,
  Rust-core tracing, and BLE markers;
- macOS CLI node log plus listener set matched to PID;
- one shared UTC test window with message UUIDs retained in private evidence;
- typed, value-free identity fields in each relevant event, for example
  `identity_kind=public_key|identity_hash|libp2p_peer_id|ble_uuid`;
- one message sent in each direction over BLE, same-LAN, and relay/cloud, with
  send, receive, decrypt, and receipt events joined by the private message ID.

Until that run exists, do not call the 5-node matrix green. The present result
is useful for locating the failure above transport, but it cannot distinguish the
identity mismatch from route churn conclusively.
