# V040-BJ-P01 — Beach-join Phase 0-1 (operator pull-forward)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN (filed 2026-09-20)
Priority: P1 Wave 1 -- operator multi-select pulled Phase 0-1 forward
Lane: Freebuff
Scope: Phase 0 spec + Phase 1 Android hotspot share only.
Do **not** implement Phase 2-3 (core ingest / trust wiring) in this wave
without a new operator go. Full design:
`HANDOFF/plans/BEACH_JOIN_AUDIT_AND_PLAN_2026-09-05.md` (read whole for
context). Original continuation ticket:
`HANDOFF/freebuff/queue/V040_BEACH_JOIN_CONTINUATION_2026-09-05.md` -- its
"finish current mission first" clause is **partially overridden** for
Phase 0-1 only by operator interview 2026-09-20.

## Operator rulings (carried forward)

- Auto-hotspot flow.
- Stranger-grade trust is required for beach **use** (Phase 3) -- not this
  wave's exit.
- Working mesh first; this is Wave 1 supporting work for join/share.

## Phase 0 -- formats (spec, no product code required)

Join-bundle v1 + single-QR payload spec must exist as a short doc or header
next to the Phase 1 code:

- `WIFI:S:<ssid>;P:<pass>;;`
- `http://<hotspot-ip>:<port>/scmessenger.apk`
- `#bundle=<base64-v1>`
- `#sha256=<hex>`
- mandatory format version
- Bundle v1 fields: `cloud_seed_addrs[]`, inviter contact bundle, bundle
  signature, APK SHA-256, version

Acceptance: spec on the page; Phase 1 cites it.

## Phase 1 -- hotspot share + verified install (Android)

Starting points (verify on current main -- do not trust line numbers blindly):

- `ApkShareManager.kt` local APK host / `getLocalIpAddress`
- QR render + ApkShare dialog + Settings entry
- JoinMesh scan screen / routes

Work:

1. Sender opens `LocalOnlyHotspotReservation` on Share; bind host to hotspot
   interface IP. **Replace first-IPv4-pickup** when it can return CGNAT /
   wrong iface (comment historically records mis-seeding).
2. QR encodes WIFI creds + URL + SHA-256 + bundle pointer (Phase 0 format).
3. Receiver: fetch + **SHA-256 verify-before-install, fail closed**.
4. If no second handset: mark multi-device legs UNVERIFIED; single-device
   compile/unit/CI still required.
5. Do not leak hotspot-local addrs as globally dialable (check addr_filter;
   regression test if needed -- read-only in core unless test fails).

## Scope correction

- `is_dialer()` ledger guard stays.
- BLE ingress stays receive_message-only -- do not synthesize
  `ConnectionEstablished` from BLE.
- MeshVpnService orphan status out of scope.
- Phase 2 seed import / Phase 3 trust: **not this ticket**.

## Acceptance

1. Spec present.
2. Phase 1 code + unit tests CI green.
3. Device proof optional this wave: UNVERIFIED allowed if no second handset;
   say which leg.
4. `check_wiring.py` green if UI routes change.

## Review gate

None for android-only Phase 0-1 unless core store filter changes.

## Rules

No emojis. Freebuff may not drive Pixel UI; operator validates install flow.
Evidence contract.
