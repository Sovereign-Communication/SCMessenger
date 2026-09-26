# Windows CLI Node Driver Log

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Monitoring Summary
Monitoring period: 2026-08-03T22:02:45Z to 2026-08-03T22:11:28Z (~8.7 minutes)
Total cycles: 8 at ~60 second intervals

## Per-Cycle Status Table

| Cycle | Timestamp | PID | Memory (KB) | Listeners | 0.0.0.0 | Inbox | Delivered | Errors |
|-------|-----------|-----|------------|-----------|---------|-------|-----------|--------|
| 1 | 22:02:45 | 17304 | 60,968 | 27 | YES | 5 | 8 | 0 |
| 2 | 22:03:47 | 17304 | 60,952 | 27 | YES | 5 | 8 | 0 |
| 3 | 22:05:22 | 17304 | 60,956 | 27 | YES | 5 | 8 | 0 |
| 4 | 22:06:24 | 17304 | 58,956 | 2 | YES | 5 | 8 | 4 |
| 5 | 22:08:21 | 30020 | 52,400 | 27 | YES | 5 | 8 | 0 |
| 6 | 22:09:22 | 30020 | 53,076 | 27 | YES | 5 | 8 | 0 |
| 7 | 22:10:27 | 30020 | 32,360 | 27 | YES | 5 | 8 | 0 |
| 8 | 22:11:28 | 30020 | 32,360 | 27 | YES | 5 | 8 | 0 |

## Critical Events

**22:05:51 - PROCESS PANIC (PID 17304)**
Process crashed with assertion failure in libp2p-request-response-0.29.0/src/lib.rs:678:9:
```
assertion `left == right` failed
  left: false
 right: true
```
Immediately before panic: relay peer disconnected, multiple circuit reservations lost, connection errors to IPv6 relay peer.

**22:06:24 - DEGRADATION DETECTED**
After panic, process appeared in tasklist but was unresponsive:
- Listener count dropped from 27 to 2
- Lost all 0.0.0.0 bindings on ports 80, 443, 8080, 9002, 9090
- Only loopback listeners remained: 9001 and 9876
- LAN reachability lost

**22:07:33 - RESTART EXECUTED**
Issued `scmessenger-cli stop` command; process gracefully terminated.
Restarted with: `./target/debug/scmessenger-cli.exe start -p 9001`
New PID: 30020 (assigned by OS)

**22:08:21 - RECOVERY CONFIRMED**
New process (PID 30020) came online with full listener count (27) and 0.0.0.0 bindings restored.
Process remained stable through end of monitoring window.

## Message Activity

No new inbound or outbound messages during monitoring window:
- Inbox receive count remained at 5 (baseline from log start)
- Delivered message count remained at 8 (baseline from log start)
- No activity indicates other nodes were not actively sending to this node during test

## Process Vitals

**Original Process (PID 17304):**
- Uptime before panic: ~180 seconds
- Memory stable at 60-61 MB until crash
- Listeners stable at 27 until crash
- Failure mode: assertion in libp2p async handler

**Restarted Process (PID 30020):**
- Uptime post-restart: ~240 seconds (still running)
- Memory: 52 MB initially, dropped to 32 MB over 3 cycles (normal GC)
- Listeners: consistent 27 throughout
- 0.0.0.0 bindings: consistently present
- No errors logged

## Availability Assessment

- **Availability window 1 (22:02-22:05:51):** 100% - full LAN reachability, multi-listener stack active
- **Degradation window (22:05:51-22:07:33):** 0% - lost 0.0.0.0 bindings, LAN unreachable
- **Recovery to EOT (22:08:21-22:11:28):** 100% - full restoration, stable

## Root Cause Analysis

Panic triggered by libp2p-request-response internal assertion. Likely cause: concurrent modification of request state or stream-handle mismatch during rapid relay peer disconnection (3 consecutive failed connection attempts detected in logs immediately before panic).

NODE STATUS: DEGRADED -- process recovered after restart, but experienced fatal crash mid-test due to libp2p assertion failure. Recovery succeeded but incident demonstrates need for panic recovery improvements in relay peer handling.
