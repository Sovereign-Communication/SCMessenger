# WINDOWS -> GPT: please push an iOS build to Christy's iPhone

Status: OPEN REQUEST -- blocks the 0.4.0/0.5.0 paired matrix
Raised: 2026-08-02 by Windows Claude (orchestrator)
Owner: Mac lane (GPT) -- Windows has no xcodebuild and cannot install to iOS

## The ask

Build and install the current `gpt/takeover-integration` iOS app onto Christy's
iPhone. Her installed build predates `c4052f7e` and is the ONLY remaining
blocker to the paired Android<->iOS test. Everything on the Android side is
verified green (evidence below).

## Why it is required -- verified, not assumed

Operator expectation: the iPhone should appear under "Nearby" when adding a
contact on Android. It does not, and the cause is a publish-side mDNS mismatch
that `c4052f7e` already fixes -- her device just does not have that build.

1. Android browses `_p2p._udp` (libp2p-mdns default),
   `android/.../transport/MdnsServiceDiscovery.kt:77`.
2. Christy's CURRENT iOS build publishes ONLY `_scmessenger._tcp` -- the
   pre-fix constant at `mDNSServiceDiscovery.swift:36`.
3. On `gpt/takeover-integration` this is already fixed:
   `mDNSServiceDiscovery.swift:33` `serviceTypes = ["_p2p._udp", "_scmessenger._tcp"]`,
   `:81` maps NetService over BOTH types, `:113` `services.forEach { $0.publish() }`,
   and `Info.plist` gained `_p2p._udp` in NSBonjourServices.
   So the fixed build BROWSES and PUBLISHES on both -- dual-stack, which also
   avoids regressing iOS<->iOS discovery.
4. Live confirmation on the operator's Pixel 6a right now: mesh service running,
   `peersDiscovered=0` sustained across 192s of uptime. Android is browsing
   correctly and seeing nothing, exactly as a publish-side mismatch predicts.

## Android side is READY -- do not re-verify, this is done

- v0.4.0 (versionCode 14) installed on the physical Pixel 6a, launches clean,
  zero FATAL/UnsatisfiedLinkError, pid stable.
- Built from a REAL arm64 Rust cross-compile. Proof: `core/target/android-libs`
  deleted before the build then recreated; `llvm-nm -D` shows
  `uniffi_scmessenger_core_fn_func_auto_block_exempt_peer` at identical
  addresses in the raw .so and the APK-extracted .so.
- `RoleNavigationPolicyTest` 3/3 PASS.
- LISTENER TRUTH VERIFIED on-device: every advertised port is genuinely bound.
  `/proc/net/tcp{,6}` listening set includes 80, 443, 8080, 9001, 9002, 9090,
  36229, 41207, 41773, 43951 -- and the app's exported
  `refreshAddressesSnapshots` listener list matches. Your `f9ea745a` fix works;
  the old "advertise 9001 while bound elsewhere" bug is gone.
- Mesh participation is ON. Reachable LAN address is `192.168.0.140`.

## What to do

1. Build the iOS app from `gpt/takeover-integration` (current tip) and install
   it on Christy's physical iPhone. Confirm the installed build actually
   contains `c4052f7e` -- please report the CFBundleVersion and the commit SHA
   it was built from.
2. PROVENANCE GATE: the Android APK on the Pixel was built from `09cf82c0`.
   Your newer commits (`4e92578e`..`5218554e`) are CI/iOS-only and touch no
   `core/`, `cli/`, or `android/` code, so the Rust core is identical. If you
   land anything that DOES touch `core/`, tell me and I will rebuild Android so
   both phones run the same core SHA before we test.
3. Report back when installed; Windows will then drive the paired matrix.

## Matrix Windows will run once she is updated

- Android QR -> iOS scan -> Android-to-iOS message + receipt
- iOS QR -> Android scan -> iOS-to-Android message + receipt
- iPhone visible under "Nearby" on Android's Add Contact screen
- Restart both apps; repeat (persistence)
- Confirm unknown-sender/contact-approval prompt appears
- Record per row: message ID, both public keys, route peer ID, selected
  transport, ConnectionEstablished event, receipt event, failure reason

## Note on what is testable BEFORE she updates

Only `Android QR -> iOS scan -> Android-to-iOS` could work today, since that
direction depends only on Android's QR being correct (it now is). "Nearby"
discovery and the iOS->Android direction both require her update.

## Constraint

The operator has restricted Windows adb to READ-ONLY except installs, so I will
not toggle settings or launch activities on the Pixel. Any iOS-side device
interaction is yours.
