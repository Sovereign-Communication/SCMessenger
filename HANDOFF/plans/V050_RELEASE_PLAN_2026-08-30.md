# v0.5.0 Release Plan (DRAFT — pre-tag 2026-08-30)

Status: OPEN draft. Feast on v0.4.0 first; this is the next-tag head-start backlog.
Grounding: this thread's verified parity + queued tickets + deferred epics still
open from `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md`.

## 0. Where we are at the moment of writing

- Merged on `main`: #239 routing-peer-seen, #240 AWS identity persistence, #241/#242
  2-node parity evidence, #243 space-clearing SOP, #246 desktop packageVersion.
- In the merge lane (still open at write time): #244 (D4 history flavor coalescing),
  #245 (Android never-drop-at-cap), #247 (P0 send-crypto ticket doc).
- Live 3-node topology intended: Windows node, AWS Ubuntu node, Pixel 6a (adb).
  Windows<->AWS relay proven; Pixel ingests only as a live mesh peer (it idles at
  0 peers when not online/foregrounded — a real linkage gap).

## 1. Verified foundation carried into v0.5.0

- Delivery convergence: sent items are never silently dropped at the retry cap;
  exhausted undelivered sends stay `queued/delivering` with patient backoff (#245).
- Identity coalescing: conversations match history across pubkey/identity_id
  flavors in the Rust core (#244) — the D4 split in the contact book ("multiple
  Lucas identities") is being collapsed on the core side.
- End-to-end encryption + relay custody chain from v0.4.0 remain the base.

## 2. North-star themes for the tag

1. **Close the crypto/identity gap.** The live P0 — sends to a newly-generated node
   identity failing Drift signature verification while the old id reads delivered —
   is the standout correctness item. See
   `HANDOFF/plans/P0_SEND_CRYPTO_ROOTCAUSE_LANE_2026-08-30.md`.
2. **Make "3 nodes actually behave as one mesh" the demo.** Parity was proven 2-node;
   the Pixel's join path must be made deterministic (auto identity creation, sustained
   peer membership) rather than requiring a foregrounded unlocked app.
3. **Pay down the dependency/toolchain debt** before it bites a release (see 4).

## 3. Queued core/android work (already ticket-tracked)

| ID | Item | Area |
|----|------|------|
| P0_SEND_CRYPTO_FAILS_VS_DELIVERED | Drift envelope signature failure vs delivered-provenance gap | transport/store |
| D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING | mobile_bridge has a SEPARATE, non-coalesced history impl | android bridge |
| DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE | toolchain/dependency uplift | build |
| DEPENDABOT_TRIAGE_PRE_TAG | yamux 0.12.1 HIGH requires libp2p-yamux upgrade (hard-pinned `^0.12.1`; 0.13.10 lock entry is derelict) — see ticket; hickory mitigated at 0.25.2 | deps/security |

## 4. Deferred epics carried from the 2026-08-01 unified plan (now v0.5.0 scope)

| Epic | Why deferred, why now | Risk if skipped |
|------|----------------------|-----------------|
| iOS mDNS/Info.plist **dual-stack** (`_p2p._udp` + `_scmessenger._tcp`) | Cross-platform LAN discovery is broken today; unification prescribes dual-stack so iOS-iOS does not regress | iOS<->Android pairing impossible over LAN |
| Freenet hole-punch port (measured baseline 64% success) | Deliberately not in the 0.4.0 freeze | hard NAT traversal dead-ends |
| Dependency & toolchain uplift | release-freeze discipline; DEPENDENCY_DEBT ticket | security + future-proofing lag |
| Repo split / HANDOFF hygiene | cross-cutting, reversible-only moves preferred | repo weight grows |
| iOS 0.5.0 train branches merge | six-branch train still open | stale divergence |

## 5. Acceptance criteria for the v0.5.0 tag

- P0 send-crypto root-caused with an on-device reproduction and a fix that closes
  the delivered-but-never-verified gap; adversarial (Opus/free-lane) APPROVE on file.
- A deterministic Pixel join: fresh install auto-initializes identity, signs in,
  and holds peer membership with the Windows + AWS nodes (2-peer sustained), and a
  Windows->Pixel->Windows round-trip converges to delivered with on-device receipt.
- iOS multipeer-coexistence: an iOS and an Android device exchange a message over
  a shared path (BLE first, LAN/mDNS after dual-stack).
- Dependency triage closed: yamux >= 0.13.10 in `Cargo.lock`, hickory re-confirmed
  in patched range, with the tightened lock adversarially reviewed.
- CI green on every required lane at the tag SHA; `v0.5.0` tag created only after
  the signed-APK gate (same D2-style operator step as v0.4.0).

## 6. Non-goals

- No new crypto primitives outside the Freenet port. No repo split until v0.4.x
  is stable and v0.5.0 foundation merges. No server/accounts architecture change.

--- END DRAFT ---