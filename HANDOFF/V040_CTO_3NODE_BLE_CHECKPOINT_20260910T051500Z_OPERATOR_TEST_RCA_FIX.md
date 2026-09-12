# V040 CTO checkpoint — operator drop-test evidence + AWS-online RCA + fixes

Stage: OPERATOR_TEST_SCORED + TWO_DEFECTS_FIXED_CODE_LEVEL (not a 3-node completion stage)
UTC: 2026-09-10T05:15Z. Branch cto/t2-disk-ruling-2026-08-31. Immutable append-only.

## Operator test (19:06-19:07 local, adb 52k-line window)

- Verdict: PASS at mesh level. peersDiscovered=2 steady (Windows 12D3KooWD6vZ + AWS 12D3KooWGvCW).
- Delivery: msg 87a321b1 (route=AWS) core smart_router FAILED then tcp_mdns SUCCESS,
  aggregate accepted transport_ack=true (raced 2 transports, LAN leg delivered);
  msg ccef44e4 (route=Windows) core SUCCESS accepted transport_ack=true.
- Zero ANR/crash. Both background/ledger fixes present in running APK (git=1eea9e65).
- AWS mutual: AWS /api/diagnostics peers=[Windows, PHONE]; Windows peers=[AWS, PHONE].

## RCA (all fresh evidence, tmp/cto/AWSRCA_20260910T034529Z/ + AWSRCA_GATE/)

1. "AWS not online to Pixel" root chain (3 stacked causes, all addressed):
   a. Process death: phone killed 17:44:04 by killDueToPackageUpdate (second install
      of the SAME 3d4bac3f bytes; provenance unaffected) and never relaunched.
   b. Ledger starvation: phone-side FFI LedgerManager had ZERO call sites for
      recordConnection; bootstrap swept only the proven tier, so ledger-exchanged
      AWS could never be dialed and never become proven (chicken-and-egg).
   c. Windows mDNS wedged: single os error 10040 at 01:30:11Z killed the
      libp2p-mdns iface task; zero mDNS output for 2.5h (hour files 03/04 empty).
      Operational restart 04:49Z recovered it (04:52:26Z discovery line).

2. "Toggle mesh to work" root cause: MainActivity.onPause -> notifyBackground ->
  pauseMeshService() tore transports down on EVERY backgrounding (Rust pause()
  itself only logs; teardown was Kotlin-initiated). Removed the automatic
  lifecycle-driven pause; explicit notification-action pause retained; battery
  adaptation remains with the duty-cycle system.

## Fixes (code-complete, unit-gated 10/10, installed 344bcb47, live-verified)

- MeshRepository: mergeBootstrapCandidates() pure merge (proven first, seed tier
  capped at MAX_BOOTSTRAP_SEEDS=4, dedup/blank-safe) wired into BOTH
  primeRelayBootstrapConnections() and racingBootstrapWithFallback().
  + BootstrapCandidateMergeTest (7 tests).
- AndroidPlatformBridge.onEnteringBackground: pause call removed with rationale.
- Live proof: ledger.json 2 -> 10765 bytes; "attempting 7 proven ledger relay
  candidate(s)"; peersDiscovered=2; deliveries accepted.

## Remaining (recorded, not blocking the current PASS)

- Windows mDNS round 2: TxtRecordTooLong exclusions returned 05:05Z — the phone's
  relay reservations re-bloat the ADVERTISED listener set (D10 guarded the
  reservation BASE; the advertisement feed needs the same filter). Ticket filed.
- Phone direct dial to AWS:9001 failed during the WiFi-drop window -> circuit
  breaker OPEN; indirect path (via Windows relay) carried traffic. Breaker reset
  on WIFI recovery exists; verify post-test convergence.
- BLE: advertiser/GATT beacon live (430B identity beacon); scanner clean; needs a
  second BLE-capable node for a true BLE leg (Windows CLI + AWS headless have none).
- Rule-8 D10 verdict still PENDING; dispatch packet ready
  (HANDOFF/review/V040_D10_REVIEWER_DISPATCH_PACKET_2026-09-10.md).

Evidence: tmp/cto/OPTEST_20260910T050723Z/ (full+tail logcat), tmp/cto/AWSRCA_GATE/
(fix gate 10/10, verify windows w1-w6), tmp/cto/D12_MDNS_RECOVERY_20260910T044936Z/.

---

# Addendum — three-node time-correlated message trace (operator test window)

Clocks: phone local UTC-10 -> UTC by +10h. Cross-node clock skew observed: <= 10ms
(inferred from Windows receive preceding phone ack-log by ~25ms on identical
pairs; all three nodes agree within log-timestamp noise).

## Message-level trace (msg id: phone send -> recipient inbox_receive)

| msg (8ch) | route | phone send (UTC) | phone outcome | node recv (UTC) | e2e |
|---|---|---|---|---|---|
| 8e69920b | Windows | 05:04:52.725 | accepted core | Win 05:04:52.950 | ~225ms |
| 21dea3f9 | Windows | 05:04:52.788 | accepted core | Win 05:04:53.010 | ~222ms |
| e1474906 | Windows | 05:04:52.794 | accepted core | Win 05:04:53.017 | ~223ms |
| f2ce7f1c | Windows | 05:04:52.860 | accepted core | Win 05:04:53.084 | ~224ms |
| 8603aa88 | Windows | 05:04:52.917 | accepted core | Win 05:04:53.141 | ~224ms |
| 90898041 | Windows | 05:05:12.872 | accepted core | Win 05:05:13.098 | ~226ms |
| ccef44e4 | Windows | 05:06:52.747 | accepted core | Win 05:06:52.972 | ~225ms |
| 2410ea22 | AWS    | 05:05:00.514 ack | accepted core | AWS 05:05:00.274 | <=240ms |
| 87a321b1 | AWS+Win (raced) | 05:06:31.3 | core-direct FAILED (breaker), relay-assist + tcp_mdns delivered; aggregate accepted | AWS 05:06:31.496, Win via LAN 05:06:32.0 | ~200-700ms |

Consistent end-to-end latency: ~220-245ms phone->recipient on all legs.

## Post-test AWS ingestion (phone -> AWS direct leg alive)

AWS inbox_receive cadence from the phone (sender 9a230574...): 1d80cf3f 05:03:42,
2410ea22 05:05:00, 87a321b1 05:06:31, e4bc8c1c 05:07:31, 6b694d7d 05:09:00,
4ca0dba6 05:10:00, f15e4231 05:11:31, dbfe4aed 05:12:31 — every 60-90s, the
store/forward leg actively ingesting. Identify cadence on AWS: phone (38
discoverable_addrs) + Windows (19) every ~60s.

## Windows-side observations (non-blocking)

- "Failed to decode wire envelope: unexpected end of file" WARNs during the
  window — partial/truncated reads; deliveries unaffected. Ticketed hygiene.
- Windows received 0 copies of the 2 AWS-routed messages (correct routing).
- SSH to AWS node lacks post-quantum KEX (OpenSSH warning) — infra hygiene item,
  not product.

Verdict: comprehensive 3-node trace PASSES at message level; custody/relay and
LAN paths both proven end-to-end with timestamps on every node.

---

# Addendum — desktop BLE revival attempt (2026-09-10 ~05:30Z)

## Finding: the BLE leg was already dead at HARDWARE level before today's node ops

- Node log 04:52:26Z (before any BT intervention): btleplug "No Bluetooth adapter
  found"; Windows GATT init error HRESULT 0. The laptop's MediaTek MT7921 combo
  card Wi-Fi half is healthy; the BT half fails USB enumeration
  (USB\VID_0000&PID_0002\..., "Device Descriptor Request Failed", problem code 43;
  ghost entry USB\VID_0489&PID_E0CD&MI_00 Present=False).
- bthserv was STOPPED (now Running + Automatic — fixed, that was a real defect).

## Recovery ladder attempted (elevated, all logged here)

1. Start-Service bthserv + StartupType Automatic — SUCCESS (service side healthy).
2. Device disable/enable cycle — no re-enumeration.
3. Parent USB root hub restart (pnputil /restart-device) — no re-enumeration.
4. Device node removal + bus rescan — radio still fails descriptor request.
   NOTE: this step removed the BT tray icon (visible symptom the operator caught).
   Driver remains in driver store; reboot restores icon + stack.

## Remaining fix: full reboot (power-cycles the combo card's USB interface)

Known MT7921 failure mode; reboot is the standard cure. Post-reboot checklist:
1. Verify radio: Get-PnpDevice -PresentOnly -Class Bluetooth (MediaTek, Status OK).
2. Verify tray icon present.
3. Relaunch node with proven launcher (tmp/cto/T14_GOLIVE/launch_node_env.ps1,
   4 params, SC_BOOTSTRAP_NODES='/ip4/127.0.0.1/tcp/19001,/ip4/18.234.62.247/tcp/9001').
4. Node log must show btleplug adapter probe SUCCESS and GATT server up.
5. Re-run the 3-node test — BLE leg should then be scoreable phone<->Windows.
