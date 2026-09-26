# P1 - Windows node silent wedge: logs/API freeze ~2h45m with zero fingerprint

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

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

## Update 2026-09-16T04:27Z: WEDGE RECURRED, AND THE v2 WATCHDOG FAILED OPEN

Passive audit of the live three-node mesh (no UI driving; operator active).
The Windows node wedged again and the v2 watchdog did not exit. Raw evidence,
all commands run this session:

- Windows process alive: `tasklist` -> `scmessenger-cli.exe` PID 1032;
  `netstat -ano` shows it LISTENING on 0.0.0.0:{80,443,8080,9002,9090} and
  127.0.0.1:9001; control API `curl http://127.0.0.1:9876/health` ->
  `{"status":"healthy"}` (HTTP 200, 5 ms).
- Last operational log line: `2026-09-16T03:19:17.606293Z  [OK] Custody ...
  delivered to 12D3KooWGvCWJNoWnReNCT...`. The relay-custody audit task,
  which logs unconditionally every 60s even when idle (`Relay custody audit
  log count: N` every :47), last fired 03:18:47Z and has NOT fired since.
  Silence at capture: 68 minutes (04:27Z - 03:19Z).
- The entire current-hour log `scm.log.2026-09-16-04` is 3 lines, all of them
  the watchdog warning and nothing else:
    04:02:47.341278Z measured 659s without log output (threshold 600s);
                     requiring a second consecutive reading before exit
    04:13:47.326254Z measured 659s ...
    04:24:47.328801Z measured 660s ...
  Constant ~660s at 11-minute spacing, never exiting.
- Consequence: the Pixel's only pending message is stuck. `files/pending_outbox.json`
  holds exactly one entry (`cf84b8ab-60ba-4614-903f-6de0627637e8`, peer
  `30d0fa67...`) whose listeners are the Windows LAN endpoints
  (`/ip4/192.168.0.121/tcp/{80,443,9090,8080,60863}`, `/tcp/9002/ws`), and every
  dial to them fails: `Failed to dial /ip4/192.168.0.121/tcp/80`,
  `.../tcp/8080`, `.../tcp/9002/ws` -> `IronCoreException$NetworkException`,
  ending `delivery_attempt ... outcome=failed ...
  reason=all_transports_failed` then `delivery_state ... state=stored
  attempt=23`. The phone also reports `0 peers (Core)`.

### Root cause of the fail-open: the watchdog's own warning resets its own
### silence measurement

`cli/src/main.rs` `log_silence_watchdog` emits its first-reading diagnostic
through `tracing::warn!`, which the log appender writes into the SAME
directory the watchdog measures via `config::latest_log_age_secs`. The file's
newest mtime therefore becomes "now" on every warning, so the next poll reads
age ~0, hits the `consecutive_silence = 0` reset branch, and the streak can
never reach the `>= 2` exit condition. The measured ~660s is the age of the
watchdog's OWN previous warning - not node activity. Under a real wedge the
watchdog now warns forever and never exits: strictly worse than v1, which at
least terminated.

### Why the CI test did not catch this (test-fidelity gap)

`cli/tests/heartbeat_watchdog_integration.rs` drives
`cli/src/bin/heartbeat-probe.rs`, which (a) exits on the FIRST silent reading
and (b) never writes anything into the monitored directory. It therefore
exercises neither the two-consecutive-readings guard nor the production
warning-write that defeats it. The test passes while production fails.

### Decision required before code changes (operator ruling, rule 9)

The minimal correctness fix is "the watchdog's own writes must not count as
node activity" (snapshot the directory's total log bytes immediately AFTER
the warning and require them unchanged on the next poll before incrementing
the streak; or route the warning off the monitored stream). But that converts
a silent wedge into a deliberate `exit(1)`, and per this ticket's own
"Consequence disclosure" there is no supervisor on this host to restart it -
so detection without a restart policy means a down node, which is what cost
the operator 75 minutes on 2026-09-15. Options:
  (a) exit(1) + install a restart policy (Windows Task Scheduler / supervisor);
  (b) keep the process alive and recover in-process (re-spawn the stalled
      swarm/event-loop task) so no supervisor is needed;
  (c) both.
The underlying stall is in the swarm/event loop (`core/src/transport/swarm`,
rule-8 gated: needs an adversarial security review before it can land).

### Update 2026-09-16: measurement half fixed (detection works again)

The self-reset is gone; the recovery-policy ruling above is still open.
Landed on `feat/v040-multi-transport-store-forward` (cli only, no core/ touch,
so no rule-8 review gate):

- `config::append_watchdog_diagnostic` writes the watchdog's own diagnostics to
  `logs/watchdog/watchdog.log` - one level below the monitored directory, so the
  scan (which only considers regular files directly inside `log_dir`) can never
  read them back as liveness - and `latest_log_age_secs` additionally skips that
  file BY NAME, so the invariant holds even if it is ever written one level up.
- `main.rs` no longer emits the first-reading diagnostic through `tracing::*`;
  the console still shows it (the operator's live channel) and the exit path is
  unchanged. The two-consecutive-readings guard is untouched, so one bad
  measurement still cannot kill a healthy node.
- Probe and tests now reproduce production instead of a friendlier variant:
  the probe requires two consecutive silence readings, writes its first-reading
  diagnostic exactly where production does, and has a `SCM_PROBE_DEADLINE_SECS`
  bound so a broken detection path exits 3 (fail fast) rather than hanging CI.
  `SCM_PROBE_MODE=healthy` writes real node log lines to prove a logging node is
  never killed. Temp dirs moved to the repo-local `tmp/` (rule 2).
- Verified failing-then-passing: with the diagnostic temporarily routed into the
  monitored file (the pre-fix behaviour), both silence tests fail with
  `left: Some(3), right: Some(1)` and stderr `no verdict after 20s ... never
  reached two consecutive silence readings` - the production symptom exactly.
  Reverted, they pass. `cargo test -p scmessenger-cli`: 124 tests, 0 failed.
  `cargo clippy -p scmessenger-cli --all-targets`: no findings in the touched
  files. `cargo fmt --all -- --check`: clean.

What this does NOT do: it does not make a wedged node recover by itself. The
watchdog now genuinely exits 1 on a real wedge, which on this host means the
node goes down until something restarts it - options (a)/(b)/(c) above are
still the operator's call, and the underlying stall in the swarm/event loop is
still unfixed and rule-8 gated.

## Update 2026-09-16T05:45Z: node recovered, stuck message DELIVERED, supervised restart landed

Recovery executed on the Windows host (operator directed). Evidence below is
from commands run this session.

**Before (05:25:44Z).** Node PID 1032, up since 09-15 08:46:45 local, listening
on 80/443/8080/9002/9090 with `/health` = healthy, but the last operational log
line was 03:19:17.606293Z - 2h06m of silence. The 05 hour file held exactly two
lines, both v2 watchdog warnings (05:08:47.293488Z, 05:19:47.293010Z).

**Deploy.** Stopped the node (ports released), then
`cargo build --release -p scmessenger-cli` (6m06s). Note the ordering: a running
node holds the .exe lock, so cargo fails with `Access is denied` until it is
stopped. Relaunched through the new supervisor at 05:34:03Z (node PID 17768).

**The stuck message `cf84b8ab-60ba-4614-903f-6de0627637e8` - delivered.**
- Node side within ~30s of the node returning: five
  `Sending delivery ACK for cf84b8ab-... to 12D3KooWKT1e1PU7p3...` lines from
  05:32:53.706895Z to 05:32:57.692755Z, each paired with
  `[OK] Message delivered successfully to 12D3KooWKT1e1PU7p3... (14-50ms)`.
- Phone side at 19:33:03.634 local (05:33:03Z), 38s after the ACKs:
  `[RECEIPT-RX] Received from core: msg=cf84b8ab-... status=Delivered`,
  `History lookup: found=true direction=SENT`, then
  `delivery_state msg=cf84b8ab-... state=delivered
  detail=delivery_receipt_duplicate_status=delivered`.
  The five ACKs were deduped correctly (`[RECEIPT-RX] DUPLICATE: Already
  delivered and already processed receipt`), so the duplicates are noise, not a
  second delivery.
- Outbox drained: `files/pending_outbox.json` is now `[]`; it held exactly this
  one entry, at attempt=23 with `all_transports_failed`.
- Mesh reformed in both directions: the node connected to the phone
  (/ip4/192.168.0.111/tcp/443) and to AWS (18.234.62.247:9001) at 05:34:06Z;
  AWS logged `Relay circuit reservation ACCEPTED via 12D3KooWD6vZQ...` at
  05:35:06Z and received fresh phone messages (`inbox_receive` 4ab6b3fa at
  05:36:06Z, dbd6a2f3 at 05:37:18Z).

**Fixed watchdog on the live node, 05:34:03Z -> 05:43:42Z.** 320 log lines;
custody-audit heartbeat every 60s and growing (12283 -> 12292);
`log_silence` lines since restart: 0; `logs/watchdog/` diagnostics directory:
does not exist (it is created only when a silence reading occurs); `/health`
healthy. The fixed watchdog neither exits nor fabricates silence on a node that
is working.

### Availability gap closed (option (a), minimal form)

`scripts/run_node_supervised.ps1` - bounded restart wrapper, no service
framework, no install, no elevation. Restarts the node on NON-ZERO exit with
exponential backoff (5s, 10s, ... capped at 60s); gives up with a `[FAIL]` line
after 5 restarts inside 60 minutes; resets the budget when a child ran >= 10
minutes, so a wedge hours into a healthy run is not a crash loop; treats exit 0
as a clean stop and does not restart; refuses to double-start while a
scmessenger-cli process is alive. `scripts/stop_node.ps1` is the matching
one-command stop. Supervisor log: `tmp/node-supervisor.log`.

Trade-off, plainly: it supervises only while its own wrapper process lives, so a
logoff or reboot ends supervision, and it is NOT release-grade supervision for
the always-on/cloud role - a Windows service or a Task Scheduler entry with a
restart policy is still the right answer there. What it deliberately does not
do: recover an in-process stall any more cheaply than a full node restart, or
supervise the supervisor itself.

Two defects in my own first wrapper revision, both found live rather than
theorised, are worth recording: `Start-Process -PassThru` returned an EMPTY
`ExitCode` on this host, which the wrapper read as failure and turned into bogus
restarts (fixed with the call operator + `$LASTEXITCODE`); and a stale node
holding the ports makes every launch fail instantly, which is
indistinguishable from a crash loop without a guard (hence the double-start
check).

### Status (updated)

- Detection: FIXED, CI-gated, and verified on the live node.
- Recovery: option (a) implemented in minimal form and now in use for this node.
  Whether to promote it to Task Scheduler/a service for the always-on role is
  still the operator's call.
- The original pending/stored defect: RESOLVED for this message, and root-caused
  to the node wedge - not to custody correctness and not to the cellular path.
- Windows node is up and supervised as of 05:34:03Z; it needs operator action
  only if the wrapper process itself dies.
- Separate observation, not scored as a defect: on startup the node logs
  `GHOST-IDENTITY-001 skip auto-subscribe ghost peer topic:
  /scmessenger/peer/30d0fa678c218b.../v1` - the recipient identity of this
  message is treated as a ghost for auto-subscribe purposes. Delivery completed
  anyway; recorded for follow-up.
- Still open, and different from detection: the underlying stall in the swarm
  event loop (`core/src/transport/swarm`, rule-8 gated) is unfixed, so a wedge
  can still occur. It is now detected and restarted instead of silent.
