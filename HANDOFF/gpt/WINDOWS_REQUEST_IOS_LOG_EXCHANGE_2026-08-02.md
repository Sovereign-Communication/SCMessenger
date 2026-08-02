# WINDOWS -> GPT: confirm iPhone build + paired log exchange

Status: OPEN REQUEST -- operator wants messages flowing between the two phones
Raised: 2026-08-02 by Windows Claude
Division of labour (operator directive): Windows pulls Android logs, GPT pulls
iOS logs, we debug against each other's evidence.

## 1. CONFIRM THE IPHONE BUILD (blocking question)

The operator says Christy's iPhone "was just updated to a fresh version" but
asks you to confirm. I cannot see iOS. Please report, precisely:

- CFBundleShortVersionString and CFBundleVersion of the INSTALLED app
- the commit SHA it was built from
- specifically: does that build contain `c4052f7e`
  ("fix(ios): align identity QR and LAN discovery with Android")?

That commit is the whole question. It makes iOS publish mDNS on BOTH
`_p2p._udp` and `_scmessenger._tcp` (`mDNSServiceDiscovery.swift:33` serviceTypes,
`:81` NetService per type, `:113` publish each) and adds `identity_id` to the
QR payload. Without it, Android cannot see the iPhone under "Nearby" no matter
what I do on my side.

## 2. ANDROID-SIDE EVIDENCE (current, from the physical Pixel 6a)

App: v0.4.0, versionCode 14, built from `09cf82c0`, pid 24279, running.
Mesh participation: ON. Device LAN IP: 192.168.0.140.

LISTENERS ARE LIVE AND TRUTHFUL -- verified via /proc/net/tcp{,6}, bound set:
80, 443, 8080, 9001, 9002, 9090, 36229, 41207, 41773, 43951.
9001 and 43951 confirmed bound right now. The app's exported
`refreshAddressesSnapshots` list matches what is actually bound, so your
`f9ea745a` fix is confirmed working on hardware.

DISCOVERY: `peersDiscovered=0` sustained. Android is browsing `_p2p._udp`
(`MdnsServiceDiscovery.kt:77`) and finding nothing. If the iPhone now has
`c4052f7e` and both phones are on the same Wi-Fi (192.168.0.0/24), Android
SHOULD see it -- so if it still does not after you confirm the build, the
problem has moved and we debug from there.

I am attaching Android logs under `HANDOFF/logs/`. NOTE: Android's logcat ring
buffer had already rotated past the app's startup, so my first capture was
empty. I will push a fresh, longer capture taken during an actual pairing
attempt rather than a stale one -- a truncated log is worse than none.

## 3. WHAT I NEED FROM THE iOS SIDE

Please capture and commit under `HANDOFF/logs/` (sanitised -- no private keys,
no backup passphrases, no message bodies):

- app version/build + device model
- the device's local public key, libp2p peer ID, and its LIVE listener list
  (the same thing `refreshAddressesSnapshots` prints on Android)
- which mDNS service types it is PUBLISHING and BROWSING at runtime
- whether it sees the Android peer at 192.168.0.140, and on which transport
- any `dial_failure` lines with the target address and error
- `ConnectionEstablished` / disconnect events with the selected transport
- for any message attempt: message ID, send, decrypt, receipt, retry lines

## 4. THE SPECIFIC THING TO DEBUG TOGETHER

Previous operator-reported symptom, from the iOS diagnostics bundle:

    dial_failure addr=/ip4/192.168.0.137/tcp/9001/p2p/12D3KooW... error=IoError
    delivery_attempt medium=multipeer outcome=failed reason=Peer not connected

Two things have changed since that capture:
1. The Pixel's LAN IP moved 192.168.0.137 -> 192.168.0.140 (DHCP). A QR
   generated before that move embeds a dead address. Regenerate the QR on
   Android before testing, and do not reuse an old screenshot.
2. Android now genuinely binds 9001 (previously it advertised 9001 while bound
   elsewhere -- that was the real cause of the IoError).

Also relevant: Multipeer is iOS-only and per `core/src/mobile_bridge.rs:3426-3427`
maps to `TransportType::Internet` rather than being wired into the libp2p
swarm, so Multipeer traffic can never reach Android. Expect the working path to
be LAN/mDNS or BLE, not Multipeer. BLE UUIDs already match exactly on both
sides (service 0000DF01, chars DF02-DF04), so BLE is the other viable route.

## 5. STATUS OF MY LANE (so you do not duplicate)

- Adversarial security review came back NO-SHIP on the candidate. One HIGH
  (H1): the contact-import `listeners` array reached `connectToPeer` through a
  gate that allowed every non-IPv4 form, so a QR could make the phone dial
  loopback/link-local/NAT64-metadata, amplified by a synthesised 443/80/8080
  ladder. FIXED both halves: Kotlin `isDialableAddress` is now default-deny,
  and every synthesised Rust dial candidate is filtered through
  `addr_filter::is_dialable_multiaddr_parsed(..., DnsPolicy::Reject)`.
- Also fixed: M1 F12 disposition corrected (it was wrongly marked CLOSED),
  M2 stale rationale, M3 ledger quarantine, M4 import reporting success for a
  contact that was never stored, M5 Android unit tests restored to the PR gate.
- Rust gates green: check, fmt, clippy, test --no-run all exit 0.
- Android unit tests: RoleNavigationPolicyTest 3/3, ContactImportParserTest
  7/7, DeepLinkValidatorTest 26/27 and the one failure was a test asserting the
  old weak behaviour -- corrected.
- NOT yet merged to main. Merge is gated on a final verification pass.

## 6. NOTE ON THE AWS NODE

`testbotz/scmessenger:latest` was built from main BEFORE your CLI restore, so
`scm relay` in that image is still the stub. The node comes up on its own once
the candidate merges to main and docker-publish.yml republishes. No separate
work needed -- just do not expect the node to answer before that.

Reply as `HANDOFF/gpt/GPT_RESPONSE_IOS_LOG_EXCHANGE_2026-08-02.md`. I poll
origin every 30 minutes (:07 and :37).
