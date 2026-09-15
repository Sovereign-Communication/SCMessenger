# P1 - Windows node silent wedge: logs/API freeze ~2h45m with zero fingerprint

Status: Open
Priority: P1 (blocks "perfect 0.4.0 readiness" log-analysis concurrence; tag
decision must explicitly accept or defer this)
Filed: 2026-09-15 by the Freebuff lane, three-node cellular triangulation audit
Host: Windows desktop node, PID 20536 at time of wedge (build `f985b10`,
0.4.0 unified stack), launched `start -p 9001` on 2026-09-14 ~13:52 HST.

## Evidence (all commands run 2026-09-14 ~18:15-18:25 HST / 04:15-04:25Z)

1. Last log line written by the node:
   `2026-09-15T01:32:14.085760Z` (`Identified peer 12D3KooWKT1e1...` /
   `Identify observed address ... 18.234.62.247:9001`) in
   `%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-09-15-01` (file mtime
   `2026-09-14 15:32:14 -1000`). No further lines across two additional log
   files (scm.log.2026-09-15-00/-01 naming rolls hourly; no new files after).
2. Process still alive: `tasklist | grep -i scmessenger` -> PID 20536 present.
3. HTTP API dead: `curl -m 5 http://127.0.0.1:9001/api/identity` returned
   empty (connection accepted then no response - 9001 was still LISTENING).
4. Socket pile-up: `netstat -ano | grep 20536` -> 6 x CLOSE_WAIT on
   192.168.0.121:9002 from 192.168.0.134, no ESTABLISHED peers.
5. No fingerprint in the frozen log: `grep -E "panic|FATAL|ERROR"` in the
   wedge window returns nothing (only two benign earlier lines: a peer
   disconnect at 01:02:16Z and the BLE HRESULT-0 "success" error at 01:02:58Z,
   both ~30 min before the freeze). Watchdog did not fire
   (`swarm_event_loop_died` absent): the event loop thread stayed alive while
   the node stopped making progress.
6. Cross-node corroboration (AWS `docker logs scm-node`): last successful
   delivery to Windows `12D3KooWD6vZ...` at `01:32:50Z` (72 total in 12h),
   then zero for ~2h50m while the Pixel kept sending; 12 custody entries
   queued for Windows-destined messages 01:57Z-04:13Z.
7. Restart effect: `taskkill /F` PID 20536, relaunch same command line at
   ~04:21:30Z -> at `04:22:14Z` AWS burst-delivered ALL 12 held custody
   entries within one second (grep count jumped to 100 in 20 min window);
   Windows logged 3 inbox_receive + 102 delivery ACKs in the new hour log;
   phone `pending_outbox.json` drained to `[]` and both stuck message IDs
   (`59e89dfb`, `8b2011ac`) processed application delivery receipts at
   04:22:24Z.

## Interpretation

The node process survived but stopped making progress: log writer silent, API
event loop unresponsive (curl hangs), relay custody dispatch stalled. The
existing watchdog (`is_event_loop_alive()` poll, cli/src/main.rs:2160) cannot
catch this class because the event loop never dies. Zero-downtime failure
mode: a field user sees "pending" for hours with no restart, no error, and no
crash report.

Window anchor: freeze began between 01:32:14Z (last Windows log) and
01:32:50Z (last AWS->Windows delivery OK). The last observed activity was
peer identify traffic on both transports (AWS peer and Pixel peer within 15s
of each other), plus 6 CLOSE_WAIT sockets that had accumulated on 9002.

## Required for close (definition of done)

1. Reproduce or narrow: run the Windows node under a hang-detection probe
   (heartbeat thread + minidump on 10 min log silence) across a soak that
   includes Pixel connect/disconnect churn; capture what the threads are
   blocked on at freeze.
2. Fix forward with the diagnosis in hand. Candidate suspects to check first:
   request-response/dcututr event interleaving after the libp2p 0.57 bump,
   the Ping-failure close_connection path added 2026-09-14 (CTO_STATE
   item 3), and relay reservation churn (the frozen log ends on identify +
   observed-address lines from both peers).
3. Watchdog upgrade: add a progress heartbeat (last-log-timestamp monitor, not
   just event-loop-alive) that force-restarts the node process on N minutes of
   silence while peers are connected. This converts the silent wedge into a
   visible, bounded outage regardless of root cause.
   **[DONE 2026-09-15]** Implemented in cli/ (not rule-8 gated):
   - `cli/src/config.rs`: `pub fn latest_log_age_secs()` (single source of
     truth, lib-visible),
   - `cli/src/main.rs`: `log_silence_watchdog` tokio task in `cmd_start` --
     polls the newest mtime in the node's log dir every timeout/10, exits(1)
     when silence exceeds `SCM_LOG_SILENCE_TIMEOUT_SECS` (default 600s).
     Same fail-fast policy as the existing `swarm_event_loop_died` watchdog:
     a node that exits gets restarted; a zombie silently drops traffic.
   - Black-box proof: `cli/src/bin/heartbeat-probe.rs` (runs the same
     detection loop via the lib path) +
     `cli/tests/heartbeat_watchdog_integration.rs` (2 tests: exits-nonzero-on
     -silence past threshold; stays-alive while log actively written).
     Verified locally: `cargo test -p scmessenger-cli --test
     heartbeat_watchdog_integration` -> `2 passed; 0 failed` in 6.32s
     (2026-09-14 session, build `5m36s` cold). CI Test lane re-runs it.
4. Re-run the 3-node cellular triangulation for 6h+ post-fix with zero wedges.

Scoring impact on the v0.4.0 gate: the store-and-forward doctrine itself
performed perfectly (12/12 custody entries drained, zero loss, receipts
converged). The wedge is a node-liveness defect, not a custody-correctness
defect. Tag decision may proceed with this ticket explicitly accepted as a
known P1 for 0.5.0 IF the heartbeat watchdog (item 3) lands pre-tag so the
failure mode is bounded and observable in the field.

## Update 2026-09-15T19:15Z: LIVE FALSE-POSITIVE INCIDENT + FIX LANDED

The v1 watchdog killed a HEALTHY Windows node at 16:57:17Z: it measured
659s of "silence" while the custody audit had written one second earlier
(16:57:16.97). Cause: `read_dir().entry.metadata()` enumeration served a
stale newest-mtime (~16:46:18) for the actively appended log on Windows.
With no supervisor present the node stayed down ~75 minutes and AWS held
custody undelivered - the operator's "pending/stored" symptom.

Fix commit `8ec95702` (pushed; CI gating):
- `latest_log_age_secs` direct-stats each file (`fs::metadata`) and ORs a
  size-growth signal - a growing log is activity even if every mtime lies.
- Watchdog exits only after TWO consecutive silence readings (poll =
  timeout/10, so <=1 extra poll interval of latency); first reading logs a
  warn.
- Fails open: unreadable entries skipped, poisoned baseline lock reports
  no data instead of inventing silence.
- Black-box integration tests still pass (2/2, 6.41s); the
  stale-enumeration failure mode is unreachable by construction (no
  enumeration metadata trusted).
- Fixed binary verified live on the Windows node since 18:46:47Z; zero
  `log_silence` warnings in 17+ min of production custody-audit traffic
  (the buggy build fired at ~11 min). Watchdog re-armed at the 600s default.

Consequence disclosure: the "max 10 min bounded outage" property still
assumes SOMEONE (or a supervisor task) restarts the node after a legitimate
watchdog exit. Zero-loss custody behavior was re-confirmed in this incident
(AWS custody burst-delivered at 18:12:01Z the instant the node returned).

End-to-end receipt verification after recovery (19:00-19:02Z): stuck message
1833a343 converged state=delivered on the phone within seconds of the app's
outbox flush; Windows published the delivery ACKs (19:00:36Z) and received
fresh inbound messages from the phone (inbox_receive 6b4a708f, 2691efb4);
first DIRECT LAN connection established Windows<->Pixel
(/ip4/192.168.0.134), previously relay-only.
