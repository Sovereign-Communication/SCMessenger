# V040 CTO checkpoint - D1 custody fix live + Android bootstrap deadlock (D3/D4) characterized

## Metadata

- Stage: `D1_LIVE_CUTOVER` (code-level milestone; NOT a three-node completion stage)
- UTC timestamp: 2026-09-09T08:00:00Z (local HST 21:00-21:50, Sep 8)
- Session: Freebuff `/cto` continuation; operator ruling in force: no dispatch,
  no worker worktrees; CTO owns Windows/AWS, operator drives Android.
- Branch: `cto/t2-disk-ruling-2026-08-31`
- Run commit (this stage): `8b1fdc22` (D1 fix + checkpoints; parent `d10ffda8`)

## D1 defect and fix

**Defect (live, reproduced 05:39-05:44Z):** relay-custody dispatch attempts burn
at the periodic-pull cadence (~15s), exhaust the 12-attempt guard in ~5 minutes,
and the cap is permanent. A custody entry whose destination merely restarts the
app (or drops cell briefly) is then refused FOREVER - `Max delivery attempts
(12) exceeded` warned every 5s while the destination sat connected 20+ minutes.
EXP1 (`12D3KooWR9io...-1788932363410-...-19`): 12 dispatches 05:39:50-05:44:30,
last refusal 05:44:35Z, never retried again. Store-and-forward that gives up in
5 minutes defeats its own purpose.

**Fix (minimal, in the obvious place):**

- `core/src/store/relay_custody.rs`: `reset_delivery_attempts_for_destination()`
  re-arms (`delivery_attempts -> 0`) every non-delivered pending custody record
  for a destination; `[CUSTODY-REARM]` info log on fire.
- `core/src/transport/swarm.rs`: called from the `ConnectionEstablished` handler
  gated on the 0->1 transition (one re-arm per connection episode; per-episode
  cap still prevents infinite retry churn against a connected-but-not-accepting
  peer).
- Regression tests: `fresh_connection_re_arms_exhausted_custody_dispatch_attempts`
  (burn 12 -> refused -> re-arm -> deliverable -> cap re-trips in the same
  episode), `custody_rearm_skips_delivered_records` (no resurrection of
  delivered records).
- Gate: `cargo test -p scmessenger-core --lib store::relay_custody` =
  **31 passed, 0 failed** (log `tmp/cto/D1_GATE_20260909T061500Z/d1_gate.log`,
  EXITCODE=0).

## Provenance matrix (all verified this session, fresh commands)

| Artifact | Commit | Hash | Verdict |
|---|---|---|---|
| Windows CLI exe | 8b1fdc22 | sha256 `71eeb626e0709e6ce32a0e0f337d605e2c0b84daad7248f473bfdcdb0b882d63` | LIVE, PID 9316, owns 9876/9001/9002 |
| Rollback (pre-D1) | d10ffda8 | `26c4570938d8abdd...` staged at `tmp/radio-26c45709/rollback/` | available |
| AWS image | 8b1fdc22 | `testbotz/scmessenger:sha-8b1fdc2`, deployed via `scripts/aws_deploy.sh` | LIVE, identity PRESERVED (`12D3KooWGvCW...`) |
| Android APK | 8b1fdc22 | sha256 `0b0ffe31f2fc88bcd429d029ec3bede8dc98d19f2758faaa9f0bbc3bd5454634` | INSTALLED via `adb install -r` (replace; device identity preserved), PID verified |

- Windows `/version`: `git_hash: 8b1fdc22`, identity `12D3KooWD6vZQr...` stable,
  T14 pin live: `external_addrs = ["147.81.41.188:9001", "192.168.0.222:9001"]`.
- AWS redeploy: health OK, `/opt/scm-relay-data:/data` mount verified by the
  script's regression guard, `CLI Version: 0.4.0 (8b1fdc22...)` in container log.
- Docker publish: run `34318608778` SUCCESS building from `8b1fdc22` (13 min).
- Windows<->AWS reconnected post-cutover (`[DIAL-BACKOFF] Reset ... successful
  connection` 06:53:03Z; Identify ticks every 60s).

## Live D1 scenario (reproduced on the new builds)

1. CTO sent Windows->Pixel message `69a3248c` at 07:27:48Z via `/api/send`
   (recipient = Pixel contact `e3d4aaec...`).
2. Swarm ROUTE_DECISION: attempt 1 direct -> failed; attempt 2 via AWS relay
   (score 50.0) -> `[OK] Message relayed successfully ... (278ms)`.
3. AWS: `Relay request from 12D3KooWD6vZ...` -> `relay custody accepted` ->
   **`Accepted custody ... for offline destination 12D3KooWR9io...`** (the
   Pixel's circuit had flapped down seconds earlier - the exact D1 scenario).
4. The message is now correctly HELD by AWS pending the Pixel's next stable
   connection episode; with D1 live, that connection will re-arm and deliver.
   (Pre-fix, if attempts burned during instability, the entry would wedge
   forever. The 05:39 EXP1 wedge is un-replayable on AWS because its custody
   store was container-ephemeral - see A4 - so delivery of `69a3248c` is the
   live proof vehicle instead.)

## Verdicts

| Item | Verdict |
|---|---|
| D1 code fix + unit gate | **PASS** (31/31, both regressions) |
| Windows D1 cutover | **PASS** (71eeb626 live, config unchanged, no env) |
| AWS D1 deploy | **PASS** (sha-8b1fdc2, identity preserved, mount guard OK) |
| APK D1 build/install | **PASS** (0b0ffe31, replace-install preserved ledger) |
| D1 live re-arm delivery of `69a3248c` | **UNVERIFIED** - requires the Pixel to hold one stable connection episode to AWS; blocked by D3 below |
| BLE transport | **PASS** (carried from 053500Z checkpoint; unchanged by this stage) |
| 3-node test readiness | **BLOCKED on D3/D4 (Android lane)** |

## NEW blocking findings: Android bootstrap deadlock (operator lane)

All evidence from this session's logcat (Pixel app PIDs 27520/29799/30244/31671/978,
21:02-21:49 local). No android/ code was touched, per the operator's lane rule.

**D3a - NetworkDetector reports offline while WiFi is healthy.** App logs
`Bootstrap dial failed ... reason=Device offline - no active network` and
`Bootstrap: network=UNKNOWN` while `dumpsys connectivity` shows WIFI
CONNECTED/VALIDATED and the phone demonstrably reaches Windows:9001 (`nc -z`
rc=0) and AWS:9001. Ground-truth probes (ICMP filtered by the AP - ping is
NOT a valid probe on this network) and detector logs are all captured.

**D3b - detector/bootstrap state divergence.** After a forced network
transition (`svc data disable/enable`) NetworkDetector updated to
`UNKNOWN -> WIFI` (21:43:27) yet the next bootstrap cycle (21:43:58) still read
`network=UNKNOWN` - bootstrap reads a stale/cached network type, not the
detector's current state.

**D3c - candidate revocation deadlock.** Two dial failures under the false
offline reading revoke the only proven candidate; bootstrap then idles forever
on `no proven ledger relay candidates` with no re-proof path except process
restart, and cold starts re-enter the race whenever the last network transition
is recent. This is the direct blocker for the 3-node re-test.

**D4 - single-candidate fragility.** The proven set contains ONLY the Windows
LAN address (`192.168.0.222:9001`); AWS (`18.234.62.247:9001`), which this very
device reached over cellular earlier today (05:28Z evidence), is never a
bootstrap candidate. One revoked candidate = full deadlock.

**Context (from 053500Z checkpoint, still standing):** app backgrounded without
the FGS freezes (OS cached-process freeze, CPU ticks frozen; observed 21:15-21:22)
and a swipe-away kills the mesh; start via Dashboard/Settings mesh toggle.

**Trigger amplifier (environment):** the "KG" AP band-steers; BSSID roams at
~21:16:47/21:26:23/21:27:35/21:32:27 (dumpsys wifi ASSOCIATED_BSSID_EVENT)
cause real transition churn that multiplies the cold-start race windows.

## Addendum (08:15Z): dispatch semantics + current topology

Code-verified dispatch semantics (`dispatch_pending_custody_for_peer`,
swarm.rs:1862-1933): dispatch requires (a) `swarm.is_connected(destination)` -
ANY live connection counts, circuit-via-Windows included - and (b) the 15s
`custody_pull_interval` tick. So the held message `69a3248c` delivers the
moment the Pixel holds ONE stable ~30s connection to AWS (direct or circuit).
No further fix required for the last-mile; it is purely a connectivity-hold
question on the Pixel.

Current topology (08:12Z): Windows<->AWS connected; Pixel disconnected from
both (app's mesh shows "running" but its connections dropped in the BSSID-roam
churn; adb tunnel also down intermittently, same root cause). Windows learned
contact 'Lucas' from the Pixel's identity envelope at 08:00:51Z (identity-sync
path proven on D1 builds) before the 08:01:21Z disconnect.

Operator action to close the D1 live proof: open/toggle the app so it holds a
stable connection ~30s+; CTO verifies the `[CUSTODY-REARM]`/dispatch/delivery
line on AWS and the arrival on the Pixel.

## Addendum 2 (08:48Z): LAN delivery proven live on D1 builds

After the operator's mesh toggle, the Pixel reconnected DIRECT to Windows
(08:42:57Z, `/ip4/192.168.0.111/tcp/47836`) and Windows shared its peer list.
Windows->Pixel message `09171d52` (08:47:15Z) DELIVERED with application
receipt: `[OK][OK] Delivered: 09171d52`, history marked delivered, receipt
outbox cleared. App-level LAN delivery on D1 builds: PASS.

The Pixel received the AWS peer address in the peer-list exchange but has not
dialed AWS (D4). Held custody message `69a3248c` therefore still waits; the
operator's cell-only phase (WiFi off) forces the direct Pixel->AWS dial, which
is precisely the D1 delivery scenario. PASS criteria recorded above.

## Re-test plan (operator)

1. (Operator, Pixel) Start the mesh via the app's Dashboard/Settings MESH
   TOGGLE (FGS holds it; no freeze), keep the app task open, WiFi on.
2. (CTO) Verify Pixel<->AWS stable connection; with D1 live, custody message
   `69a3248c` should deliver on the fresh episode - close the D1 live proof.
3. (Operator) Manual drop test: WiFi off -> cell-only via AWS; WiFi back.
4. (Both) Full 3-node matrix per the V040 package (BLE leg needs a second
   Android in range).
5. Queued post-run: A4 (point AWS custody-audit store at /data), D2 (Android
   advertises loopback addrs; AWS dials ::1 and hits itself), D3/D4 (bootstrap
   detector + multi-candidate), AWS executor still `run_instances`-DENIED.

## Evidence index

- `tmp/cto/D1_GATE_20260909T061500Z/d1_gate.log` - 31/31 custody tests
- `tmp/cto/D1_CUTOVER_20260909T062500Z/` - rebuild log, exe hash, node logs
- `tmp/cto/D1_APK_20260909T071500Z/` - APK build log + hash (0b0ffe31)
- `tmp/radio-26c45709/rollback/SHA256SUMS.txt` - rollback staging
- `tmp/cto/aws_d1_proof.sh` / `aws_exp1_status.sh` / `aws_exp1_deep.sh` - AWS evidence pullers
- `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T053500Z_TRANSPORT_VERIFY.md` - prior stage
- Docker publish run `34318608778` (SUCCESS, sha-8b1fdc2); commit `8b1fdc22` pushed
