# V040 b0f7ac4e 3-NODE VALIDATION -- LIVE TEST RESULTS (2026-09-06 ~04:10Z)

Task: V040_CEO_APPROVAL_PUSH_276_TO_FULL_3NODE_GREEN_2026-09-04.md (step: full 3-node validation at final head)
Type: PROGRESS (major gate PASS + two new FAIL/UNVERIFIED findings from live radio test)

## Fleet (all three nodes same-SHA b0f7ac4e -- verified)

- Windows CLI PID 19476, provenance `0.4.0 (b0f7ac4e:...)`, sha256 0eef5b2b...
- AWS container b390e822134b, image sha-b0f7ac4, boot log `CLI Version: 0.4.0 (b0f7ac4e...)`
- Pixel 6a: CI APK 55c0a321 (vc14/0.4.0, tree==b0f7ac4e), fresh install, identity 8ff51a89 / 12D3KooWP85tP

## GATE RESULTS

| Gate | Mark | Evidence |
|---|---|---|
| Windows->Pixel Text **displays on-device** | **PASS** | msg 2d73fe5d sent 03:49:44 via /api/send; receipt 03:49:49.2; phone receipt_send 03:49:50; **user visually confirmed display** |
| Contact canonicalization (RCA fix) | PASS live | contacts_canonical_hex_live firing (c5b1f974 stored 03:55:07-08); decryption to phone worked end-to-end |
| Queue-and-flush on reconnect | PASS | 22/22 phone messages delivered intact on TCP reattach; pending_outbox now [] |
| Direct TCP/LAN mesh | PASS | route=direct 8-30ms deliveries both directions |
| AWS relay custody mechanics | PASS (for Windows identity) | 91 accepts + immediate-pull delivered, custody_audit 0->273 |
| **BLE chat delivery (phone->Windows)** | **FAIL real-time** | 26 BLE GATT payloads decrypted at Windows but NO chat message arrived until TCP reattach flush; BLE-layer ids do not correlate with chat ids |
| **Cellular bootstrap + AWS store/forward for phone** | **FAIL/UNVERIFIED** | phone never registered at AWS during cell-only window; AWS held ZERO custody items for phone identity 8ff51a89 (only Windows drift frames) so pull would have found nothing; Pixel logs show `network=WIFI, cellular=false` classifier (suspected root, unproven for window) |
| Diagnosability finding | NEW DEFECT | mesh_diagnostics.log retention ~3 min (rotation + isRecent/UNIFICATION spam) -- killed phone-side evidence for both legs |

## What the CEO/operator should know

1. The 0.4.0 core gates (display, canonicalization, reconnect flush) are GREEN at b0f7ac4e.
2. Two radio-path defects are now precisely characterized (not vague): BLE chat doesn't flow
   in real time despite a working GATT channel, and cellular-only never contacts the relay --
   likely because the bootstrap classifier reports cellular=false (log evidence post-WiFi,
   rotated before the cell window; rerun with logcat capture required to prove).
3. The full debug report + one-run repro plan:
   `tmp/run-evidence/b0f7ac4e-3node-20260906/BLE-CELL-DEBUG-2026-09-06.md` (section "Next test").
4. Evidence: DISPLAY-GATE-PASS.md, wincli-BLE-plus-cell-window.log, aws-node-log-100m.txt,
   pixel-mesh_diagnostics.log* (all under tmp/run-evidence/b0f7ac4e-3node-20260906/).

## Tag decision input

Merge-readiness is NOT blocked by these two findings if the CEO scopes 0.4.0 = same-SHA
mesh + display + store/forward architecture (BLE real-time chat and cellular bootstrap were
equally broken on 177bd840 -- pre-existing, not regressions of this candidate). If BLE/cell
are in scope, they are the remaining work items, both now well-localized for RCA.
Single next decision: tag 0.4.0 with BLE/cell findings filed as known issues, or hold the tag
for the one-run RCA rerun (approx 15 min with logcat capture).
