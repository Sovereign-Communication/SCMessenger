# V040 checkpoint - drop-test RCA: five defects ranked, one smoking gun

Stage: `DROPPHASE_RCA` (analysis; no new code)
Timestamp: 2026-09-09T09:35:00Z
Phase window analyzed: 09:05:47Z WiFi drop -> 09:25:29Z baseline recovery
Evidence: `tmp/cto/DROPPHASE_20260909T0930Z/` (383,771-line logcat dump,
16MB ring buffer held the entire test; RCA_KEY_LINES.txt has the extract)

## Timeline (all verified from raw logs)

- 09:02-09:05Z: operator messages arrive at Windows over LAN - app-level
  delivery healthy on D1 builds.
- 09:05:47Z: WiFi drops (Windows: `Lost relay peer` on 192.168.0.111 paths).
- 09:05-09:21Z: phone in D3 deadlock: bootstrap cycles `no proven ledger relay
  candidates` on WIFI/UNKNOWN/CELLULAR in turn; the ONLY proven candidate
  (Windows LAN) was revoked during the flap; NetworkDetector and bootstrap read
  divergent network state throughout.
- 09:21:49Z (SMOKING GUN, dump line 322700ff): operator toggles mesh; the app
  correctly detects CELLULAR and RACES bootstrap across 5 transports
  (`Racing bootstrap: network=CELLULAR, transports=wss->tcp->quic->tcp->ws`)
  then REFUSES TO DIAL ANYTHING: `No candidate addresses available (all
  circuit-breaker-blocked or throttled)`. The circuit breaker state accumulated
  during the WiFi flap blocks every candidate - including AWS - in the new
  network epoch. Zero dials left the phone toward AWS over the whole phase.
- 09:24:11Z: two circuit attempts to AWS (via Windows relay, tcp/8080 circuit
  path) DID reach AWS - `Inbound relay circuit established` x2 - but both died
  in protocol negotiation: `Incoming connection negotiation aborted ... ->
  /ip4/147.81.41.188/tcp/8080/p2p-circuit: Listen error: Failed to negotiate
  transport protocol(s)`.
- 09:25:29Z: WiFi restored, SubnetProbe re-proves Windows, `UNIFICATION
  peer_connection` re-established, baseline recovered.

## Defects, ranked

| ID | Defect | Layer | Evidence |
|---|---|---|---|
| D3d | Circuit-breaker state from the old network epoch blocks ALL candidates (incl. AWS) in the new epoch; racing bootstrap refuses to dial | Android | dump line 322721 `No candidate addresses available (all circuit-breaker-blocked or throttled)` at the cellular race |
| D3a/b/c | Detector divergence + candidate revocation deadlock (previously logged, re-confirmed) | Android | bootstrap `network=UNKNOWN` while detector=WIFI; 2-failure revocation; no re-proof path |
| D4 | AWS never in the proven candidate set; nothing dials it proactively | Android | no `18.234.62.247` dial line in the entire dump; AWS peers listed only Windows all phase |
| D7 | Inbound circuit negotiation aborts on AWS (circuit established, then negotiation dies at the tcp/8080 p2p-circuit path) | core (needs triage) | AWS log 09:24:11.681/.711 (two aborts seconds after two `Inbound relay circuit established`) |
| D2 | Loopback addresses in advertised sets; AWS dials ::1 to reach the Pixel and hits itself | core + Android | AWS `Unexpected peer ID ... at /ip6/::1/tcp/9090` recurring |
| A4 | AWS custody-audit ticker resets on redeploy (store outside /data); held custody CONTENT survived (audit count 1, message held) | ops | audit count 37 -> 0 -> 1 across redeploys |
| D8 | LAUNCH HANG (23:26-23:37Z): BLE duty-cycle calls BluetoothAdapter binder (isLeEnabled) on the MAIN THREAD at every scan-window boundary; with a half-wedged BT stack (died 23:21:56 right after the cellular mesh-toggle re-initialized BLE; rebound 23:23:42) each call blocks ~5s; app freezes at launch; AnrWatchdog fires every 5s; main thread 0 CPU (blocked, not busy). Recovered: force-stop + svc bluetooth disable/enable + clean relaunch = 0 stalls, 1 peer (pid 31741). FIX (Android): move all BluetoothAdapter binder calls off the main thread (dedicated BLE dispatcher thread) + timeout/fallback | Android | dump hang_28841.log; main-tid gap 23:26:20-23:36; BT process death/rebind lines 326549/331751 |

## Verdicts for the phase

- LAN leg (Windows<->Pixel): **PASS** (messages + receipts, pre-drop)
- Cell leg (Pixel->AWS): **FAIL - blocked by D3d (+D4, D7)** - no direct dial
  ever attempted; the two circuit attempts died in negotiation
- Held custody message `69a3248c`: still held, intact - delivery still pending
  one stable Pixel-AWS connection episode
- BLE: advertising/GATT previously confirmed; delivery leg needs 2nd Android

## Fix order (owner lane)

1. D3d+D3 (Android): clear circuit-breaker + re-prove candidates on network
   change; never let 2 failures revoke the last candidate; bootstrap must read
   the detector's live state.
2. D4 (Android): seed AWS as a permanent candidate (ledger relay entry) and
   dial it on every network change.
3. D7 (core): triage the tcp/8080 p2p-circuit negotiation abort on AWS inbound.
4. D2 (core+Android): strip loopback/non-routable addrs from advertisements.
5. A4 (ops): point the AWS custody-audit store at /data.

## Re-test gate

Cell phase passes when: WiFi off -> AWS log shows direct Pixel dial (CGNAT
addr) -> `Dispatching custody ... via periodic_pull` for `69a3248c` -> Windows
sees the identity envelope -> Pixel app shows the message. Then the full
3-node matrix (BLE leg with 2nd Android) per the V040 package.
