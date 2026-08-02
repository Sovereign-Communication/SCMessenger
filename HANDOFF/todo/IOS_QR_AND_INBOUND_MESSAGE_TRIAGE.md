# TRIAGE: iOS QR identity failure + inbound message not surfacing

Status: TODO - analysis only, do NOT write code in this pass
Reported by operator 2026-08-01 from live two-device testing
(iPhone running iOS 0.5.0 build 9, Android running v0.4.0)

## Operator-reported symptoms (ground truth)

1. iOS QR GENERATION/SCAN FAILS. When scanning for iOS, the app reports
   `Missing Identity ID in Payload`. The operator cannot connect using the
   iOS-produced QR code.
2. iOS CAN scan the ANDROID QR code and a connection IS established.
3. BUT messages do not come through after that connection.
4. EXPECTED behaviour that is missing: an inbound message should raise a
   notification, and an unknown sender should prompt a new-connection /
   contact-request approval flow.

## Evidence available

- `scmessenger_diagnostics_bundle_both.txt` (repo root, 444 lines) contains
  diagnostics from BOTH phones. Read it in full.

## Known-good context (verified, do not re-derive)

- The Android QR payload is built in
  `android/.../data/MeshRepository.kt:9311-9354` (getIdentityExportString).
  It emits keys: version, peer_id, public_key, device_id, identity_id,
  nickname, libp2p_peer_id, relay, connection_hints. In minimal-QR mode
  connection_hints holds a single local TCP multiaddr.
- The Android parser is `utils/ContactImportParser.kt:19-91`; it requires
  peerId + publicKey and gathers listeners from "listeners",
  "external_addresses" and "connection_hints".
- `identity_id` is a Blake3 hash of the public key. Android treats it as the
  canonical id (MeshRepository.kt:3803-3854) and explicitly warns that a
  64-hex identity_id is NOT a valid libp2p PeerId (MeshRepository.kt:6170).
- iOS 0.5.0 lives on unmerged branches `origin/gpt/v050-ios-release-ready`
  and `origin/gpt/v050-ios-device-install`. Those branches change NOTHING
  under `core/`, and relay PROTOCOL_VERSION is 1 on both sides, so there is
  no wire-protocol version skew.
- SEPARATE KNOWN BUG, already filed, do not re-report: iOS mDNS advertises
  `_scmessenger._tcp` while Android uses `_p2p._udp`, so LAN mDNS discovery
  cannot work between them.
- The literal string "Missing Identity ID in Payload" does NOT appear
  anywhere in the repo on main or on the two iOS 0.5.0 branches. Determine
  where it actually comes from.

## Questions to answer

A. What exactly produces `Missing Identity ID in Payload`? If it is not in
   the tree, say so and identify the closest validation that would emit it,
   plus which side (iOS generator vs iOS parser) is at fault.
B. Does the iOS QR payload include an `identity_id` field at all? Compare the
   iOS payload key set against the Android key set above and list every
   mismatch in BOTH directions (missing, renamed, wrong case, wrong type).
C. Once iOS connects via the Android QR, why does no message arrive? Use the
   bundle. Distinguish: no transport connection, connection but no delivery,
   delivery but no decrypt, or decrypt but no UI/notification surfacing.
D. Is there an inbound-message notification path on iOS, and a contact-request
   / unknown-sender approval prompt? If they are absent or unwired, name the
   file and the missing call.
E. Rank the root causes by confidence and give the minimal fix for each, with
   file:line and an estimated LoC. Do not write the code yet.

## Rules

- Analysis only. Produce findings, not patches.
- Cite file:line for every claim. Quote the log lines you rely on.
- Clearly separate VERIFIED-IN-EVIDENCE from INFERRED.
- If the bundle is insufficient to answer a question, say exactly which
  additional log or command output is required.
