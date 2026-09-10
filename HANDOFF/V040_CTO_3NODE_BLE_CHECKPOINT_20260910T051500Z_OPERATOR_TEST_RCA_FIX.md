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
