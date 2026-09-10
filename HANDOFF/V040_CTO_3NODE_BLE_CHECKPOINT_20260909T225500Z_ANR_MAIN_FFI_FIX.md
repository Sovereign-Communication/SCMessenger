# V040 CTO checkpoint - ANR fix: PlatformBridge main-thread FFI dispatch

## Metadata

- Stage: `ANR_MAIN_FFI_FIX` (Android UI-hang defect class, code-complete + gated)
- UTC timestamp: 2026-09-09T22:55:00Z
- Branch: `cto/t2-disk-ruling-2026-08-31` (PR #279)
- HEAD at implementation: `3794c909553caaf3673fbbf02cf253348d769336`
- Operator directive: "Android UI issue resurfaced. ensure we resolve this and
  all issues fully so they don't resurface. unify the app!" + "track work in
  PR's as appropriate"

## Defect (root-caused from system ANR traces, not guessed)

Recurring UI hang / "app not responding" on the Pixel. Evidence pulled live
from the device this session via `dumpsys dropbox --print data_app_anr`
(archive: `tmp/cto/dropbox_data_app_anr.txt`, 3.78 MB, 6 records 2026-09-08
23:26Z through 2026-09-09 12:02Z; 19 ANR records naming this app today):

- Main thread blocked INSIDE `libscmessenger_core.so` through uniffi FFI:
  - `uniffi_scmessenger_core_fn_method_meshservice_pause` (47 stacks)
  - `uniffi_scmessenger_core_fn_method_meshservice_update_device_state` (64)
  - `uniffi_scmessenger_core_fn_method_meshservice_resume` (10)
- Kotlin entry points: `AndroidPlatformBridge.onEnteringBackground/Foreground`
  (from both `MainActivity.onPause/onResume` via `notifyBackground/Foreground`
  AND from the Rust-driven uniffi `PlatformBridge` callback),
  `updateBatteryState` (battery broadcasts + 30s periodic),
  `onNetworkChanged`, `onMotionChanged` (SCREEN_ON/OFF broadcast receiver),
  plus `MeshForegroundService` start ("executing service ... waited 20001ms"
  ANR subjects) and input-dispatch ANRs (FocusEvent waited 5001ms).
- Corroborating live logcat: `Choreographer: Skipped 79 frames` at
  12:03:25 (main thread blocked >=1.3 s at startup).
- Mechanism: uniffi `PlatformBridge` callbacks are invoked by Rust on the
  main thread; the Kotlin overrides then called back into Rust synchronously
  (pause/resume/updateDeviceState FFI). Main -> Rust -> (callback) -> main ->
  blocking Rust call = deadlock-class UI freeze.

## Fix (one file: android/.../service/AndroidPlatformBridge.kt)

All PlatformBridge override paths that reach blocking FFI now dispatch onto
the existing IO `scope` (`CoroutineScope(SupervisorJob() + Dispatchers.IO)`):

1. `onEnteringBackground` / `onEnteringForeground`: `scope.launch` around
   `pauseMeshService()` / `resumeMeshService()` with try/catch. Covers BOTH
   entry paths (lifecycle + Rust callback).
2. Motion broadcast receiver (`onReceive`, main thread): only sets the
   volatile `currentMotionState` inline; `onMotionChanged` dispatched via
   `scope.launch`.
3. `checkBatteryState()` / `checkNetworkState()` (30s periodic from FGS, and
   any external caller): now `scope.launch { deviceStateMutex.withLock { ... } }`
   - off main AND preserving the existing serialization contract.
4. `onBatteryChanged` / `onNetworkChanged` / `onMotionChanged` bodies
   (updateDeviceState FFI) therefore only ever execute on the IO scope.

Explicitly audited and found safe (no change): `TopicManager.refreshKnownTopics`
runs on its own IO scope; `MeshRepository.getTopics()` runBlocking is only
called from that scope; CoreDelegate overrides delegate into `repoScope`;
`awaitPeerConnection` (getPeers FFI) is a suspend fun on IO; shutdown
runBlocking paths run from service-stop coroutines. AndroidPlatformBridge is
the ONLY component with main-thread-reachable blocking FFI.

## Root cause NOT fixed (Rust side - rule-8 gated, ticketed)

`meshService.pause/resume` (and `update_device_state`) block for >10 s in the
Rust core - they should be async/non-blocking. That is `core/src/` work and
requires an adversarial security review per AGENTS.md rule 8. Filed here as
the required ticket; Android must never call it on main regardless, which
this fix guarantees.

## Gates (all green, all under scripts/build_lock.py, holder `cto-anr-fix`)

- `:app:compileDebugKotlin` EXITCODE=0
- `:app:testDebugUnitTest` EXITCODE=0
- `:app:assembleDebug` EXITCODE=0
- Logs: `tmp/cto/ANR_FIX_20260909/{compile,unittest,apk_build}.log`
- APK: `android/app/build/outputs/apk/debug/app-debug.apk`
  SHA256 `3334b46147613c42399b6a4f225773ded3cb3ec885e1f7fc008c9ac48a7ae4cf`
  (71,593,546 bytes, built 12:41 local / gate log confirms)
- Gate tree: HEAD `3794c909` + this AndroidPlatformBridge.kt diff.

## Verdicts

- Main-thread FFI dispatch fix: PASS (gated compile + unit + APK build)
- Live on-device hang-free verification: UNVERIFIED (install + observe next;
  operator lane is driving the device, adb at 192.168.0.111:34895)
- Rust-side pause/resume blocking: FAIL (pre-existing, ticketed above, rule-8)
- Full-suite re-run for PR: unchanged from prior checkpoint (1849/0/25 at
  2b84879f); this diff is Android-only, no core changes.

## Follow-ups queued

1. Install fixed APK, observe passive logs (Choreographer/ANR watchdog) for
   the same signature; E8-grade evidence before any "hang fixed" claim.
2. Rule-8 review packet for Rust pause/resume/update_device_state blocking
   behavior (follows T14 packet precedent in HANDOFF/review/).
3. Config-rewrite regression (external_addr nulled) still open from prior
   checkpoint.

## POST-GATE LIVE VERIFICATION (2026-09-09T22:58Z)

Installed `app-debug.apk` (sha256 `3334b461...`) on the Pixel via the TLS adb
serial `adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp` (replace-install,
data preserved), launched via monkey, observed passively:

- Window 1 (startup, 4683 lines): ONE `Skipped 31 frames` at 12:49:35.307
  (one-time Compose/Application init, ~0.5 s, BEFORE service init) — vs the
  defect signature of repeated 79+ frame skips and 10-20 s ANR blocks.
- Window 2 (~3.5 min steady state): **0** `ANR detected`, **0** input-dispatch
  timeouts, **0** skipped-frames, watchdog silent, PID stable (10431).
- Service healthy: `MeshRepository service state: RUNNING`; BLE advertising
  started (mode=1, txPower=2), GATT identity beacon 430 bytes refreshed,
  BLE scanner duty-cycling on worker threads.
- Logs: `tmp/cto/ANR_FIX_20260909/logcat_postfix_window{1,2}.log`
- VERDICT: UI-hang defect FIXED (live, passive-log evidence, E8-grade for
  the ANR axis: absence of the failure signature across a steady-state
  window plus the root-cause fix, not just an absence claim).

## SECOND ROOT CAUSE FOUND DURING VERIFY (not fixed this session - rule-8 queued)

Post-fix passive logs show the Pixel NOT rejoining the mesh:
`peersDiscovered=0`, `Bootstrap: no proven ledger relay candidates`, and the
Pixel's `ledger.json` is empty (`[]`). LAN re-seed also failed: mDNS started,
self-resolve correctly filtered (`mDNS: ignoring self-resolved service`),
but the phone NEVER discovers the Windows node's service.

Root-cause chain (evidence: `tmp/cto/D2_GOLIVE_20260909/node-out.log`
18:31-18:33Z + vendored libp2p-mdns 0.48.0 source):

1. Windows node mDNS response contains a nested self-circuit route listen
   address: `/ip4/192.168.0.222/tcp/9001/p2p/<self>/p2p-circuit/p2p/<AWS>/
   p2p-circuit/p2p/<self>` -> `TxtRecordTooLong` exclusion warnings, plus
   `os error 10040` datagram overflow on the mdns read path.
2. Vendored source proves mdns advertises the swarm's ListenAddresses
   verbatim (`libp2p-swarm-0.47.1/src/behaviour/listen_addresses.rs`: only
   NewListenAddr/ExpiredListenAddr mutate it) — so a nested-circuit listen
   address can only come from `listen_on()` being called with a circuit base
   whose host:port was the node's OWN address (relay-reservation path,
   swarm.rs:5225 via relay_reservation_multiaddr/build_routable_relay_addrs
   — the is_self_address guard missed because bound_addresses was incomplete
   in an early-startup window).
3. Effect: Windows' LAN advertisement is degraded/oversized; the phone's
   NsdManager never receives a usable response; with the ledger empty the
   phone has no relay candidates either -> isolated from the mesh on every
   transport except BLE (which is up and healthy).

REQUIRED FIX (queued, rule-8 gated, core/src/transport/swarm.rs):
- At the reservation call site, validate the normalized reservation base
  against the CURRENT swarm listen/external set before `listen_on`, and skip
  self-matching bases (defense in depth beyond the identify-time snapshot).
- Add regression test: a nested self-circuit base must never produce a
  listen_on, and listen addresses containing /p2p-circuit/ must never reach
  the mDNS advertisement set.
- This follows the T14 packet precedent: implement on the PR branch with
  tests, file HANDOFF/review packet, independent verdict before merge.

## FINAL VERDICTS (this checkpoint)

- ANR main-thread FFI fix: PASS (gated + live-verified above)
- APK `3334b461...` installed and running: PASS
- Mesh rejoin (LAN/cell) from the Pixel: FAIL — second root cause documented
  above, fix queued as rule-8 gated work (next session's first task)
- BLE transport availability on Pixel: PASS (advertising + scanning live)

---

## ADDENDUM 2026-09-10 — D10 LAN-discovery fix LANDED (queued rule-8 item executed)

The queued core fix is implemented, gated, committed, and pushed.

- Commit: `7ff317f0` on `cto/t2-disk-ruling-2026-08-31` (PR #279 head, confirmed
  via `gh pr view 279` after push). Files: `core/src/transport/swarm.rs` (+360/-4),
  `HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md` (new).
- Fix shape: structural wildcard-aware `is_self_endpoint` (private/loopback-only
  claims — same-port foreign relays stay eligible, test-enforced),
  `is_valid_reservation_base` gate at the live reservation call site
  (circuit bases, undiscoverable bases, self bases rejected; no valid base ->
  reservation skipped and retried on next identify), `is_canonical_reservation_addr`
  tripwire (exactly one terminal `/p2p-circuit`).
- Regression tests: 8/8 passing (`cargo test -p scmessenger-core --lib d10_`,
  `tmp/cto/d10_tests_final.log`).
- Gates (authoritative Windows, under build lock): fmt 0
  (`tmp/cto/d10_fmt_final.log`), clippy CI-exact 0
  (`tmp/cto/D10_CLIPPY_20260909T142328Z/clippy.log`; the battery's earlier
  clippy 101 was the bare `-A` shorthand artifact, not a code defect),
  full workspace suite 0 (`tmp/cto/D10_GATE_20260909T234503Z/workspace_tests.log`).
- Review packet: `HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md`
  — verdict PENDING; merge to main BLOCKED per rule 8 until an independent
  adversarial APPROVE is recorded.
- Branch deploy: docker-publish dispatched at the SHA (D1 pattern):
  run https://github.com/Sovereign-Communication/SCMessenger/actions/runs/34421758994
- Stale-lock note: the battery left a stale lock (`cto-d10-clippy`, dead pid 3112,
  age 1816 s > 1800 s stale threshold); released with the matching holder name.
- Not done in this pass (per directive): node restarts, APK rebuilds, the
  config-rewrite (T14 pin) and empty-ledger persistence questions — scheduled
  after this lands and the rejoin is verified.

Verdict: D10 fix PASS at code level + all local gates; LIVE rejoin verification
still UNVERIFIED (requires node redeploy + Pixel-side rejoin, next pass).

---

## ADDENDUM 2026-09-10T01:20Z — D10 GOLIVE: both desktop nodes on 7ff317f0

**Deploy states (all evidence paths under `tmp/cto/`):**

- **docker-publish** run 34421758994: SUCCESS (image `testbotz/scmessenger:sha-7ff317f`).
- **AWS deploy**: PASS — `tmp/cto/D10_GOLIVE_20260909T145402Z/aws_deploy.log`:
  identity-preserving `/data` mount verified (`[OK] persistent volume mounted`),
  identity preserved (`12D3KooWGvCW…`, seniority 1788524000), node reports
  `0.4.0 (7ff317f0…)` healthy; post-deploy `peers=[Windows]`, `external_addrs=[18.234.62.247:9001]`,
  `connection_path_state=DirectPreferred` (verified live via /api/diagnostics).
- **Windows reroll**: PASS — `tmp/cto/D10_REROLL_20260909T145417Z/`:
  stop → release rebuild 10m49s (EXITCODE=0, exe sha256 `660BE35D…`) →
  config pin restored (PIN_RESTORED=YES) → relaunch with T14 env
  (`SC_BOOTSTRAP_NODES=/ip4/127.0.0.1/tcp/19001,/ip4/18.234.62.247/tcp/9001`).
  Live: `/version` git_hash `5c7caa1d` (docs commit atop D10; swarm.rs identical
  to 7ff317f0 — exe built 00:54Z from this tree), identity `12D3KooWD6vZ…` preserved,
  `external_addrs=['147.81.41.188:9001']` (T14 pin live), AWS reconnected
  (`Identified peer 12D3KooWGvCW…` every 60s).

**D10 proof (Windows axis — the decisive evidence):**
- Live listener set: exactly ONE circuit listener —
  `/ip4/18.234.62.247/tcp/9001/p2p/<AWS>/p2p-circuit/p2p/<self>` — canonical
  single-circuit; NO nested/double-circuit addresses (pre-fix live node showed the poison shape).
- File log (`scm.log.2026-09-10-01`, since relaunch): **0 `TxtRecordTooLong`,
  0 `os error 10040`** vs the pre-relaunch hour file's **206** `TxtRecordTooLong`.
  mDNS enabled on iface 192.168.0.222; reservation ACCEPTED with canonical
  address; reservation base logged (link-local IPv6 picked from identify set —
  ordering nit for a later pass, validity unaffected).
- BOM incident (fixed in-pass): the pin-restore via `Set-Content -Encoding UTF8`
  (PS 5.1) wrote a UTF-8 BOM; serde rejected the config (`expected value at line
  1 column 1`) and the first relaunched process exited. Fixed with
  `tmp/cto/d10_strip_bom.ps1` (BOM stripped, python-validated JSON), relaunched
  cleanly. Lesson recorded: config writes on this host must be BOM-free.

**Config rewrite evidence (rewrite-source hunt, not deep-dived per directive):**
config.json `external_addr` was null again, file mtime **2026-09-10T00:11:40Z** —
inside the D10 workspace-test battery window (23:45:03Z→00:12:06Z). Current
prime suspect: a workspace test (or default-config writer) touching the real
`%APPDATA%` path. Recorded for the pending hunt; pin restored this pass.

**Rejoin status:**
- Windows ledger is actively dialing the Pixel's LAN address
  (`/ip4/192.168.0.111/…` attempts with backoff in `scm.log.2026-09-10-01`):
  the desktop re-seed path is alive.
- Pixel adb unreachable this pass (empty `adb devices`; `adb connect
  192.168.0.111:34895` fails silently) — phone-side rejoin proof therefore
  **UNVERIFIED this pass** (evidence access, not a regression signal: Windows's
  mDNS advertisement is now provably intact, which was the broken leg).
- Deliveries are expected to resume as the Pixel's own dial loop hits the now-
  healthy Windows advertisement; scoring the full rejoin (peersDiscovered>0,
  message flow) is the next pass once adb is back.

**Verdicts:** AWS deploy PASS; Windows deploy PASS; D10 live proof PASS
(no nested circuits, no TXT overflow); mutual desktop reconnection PASS;
Pixel-side rejoin UNVERIFIED (adb unreachable — next pass).

---

## ADDENDUM 2026-09-10T02:10Z — REJOIN PASS: Pixel back on the mesh (D10 proven end-to-end)

**Deploy:** staged D10 APK (sha256 d0143c65…, HEAD 8c74a6a2) replace-installed
(lastUpdateTime 15:56:34 local = 01:56Z, data preserved, versionCode=14),
launched via monkey (authorized deploy-verify flow), new PID 7140.
Evidence: tmp/cto/D10_REJOIN_20260910T015627Z/ (logcat_w1_90s / w2_210s / w3_final).

**Rejoin scoring (from actual lines):**
- (a) mDNS/LAN discovery of WINDOWS: **PASS** — `16:01:09.637 MeshRepository:
  TCP/mDNS: LAN peer detected 12D3KooWD6vZQrUq…(Windows) with 8 local addresses`;
  self-resolve correctly ignored (`mDNS: ignoring self-resolved service`).
- Dial: **PASS** — `Successfully dialed discovered LAN peer
  /ip4/192.168.0.222/tcp/9001 via SwarmBridge` (correct port-fallback from a
  failed 9002/ws attempt); full identify: `agent=scmessenger/0.4.0/full/relay/
  12D3KooWD6vZ…, 32 addresses`.
- Desktop mutual: **PASS** — Windows /api/diagnostics peers now include the
  Pixel `12D3KooWR9io…` AND AWS; `Connected to 12D3KooWR9io… via
  /ip4/192.168.0.134/tcp/59956`; `Learned new contact 'Lucas'`;
  `DIAL-BACKOFF Reset backoff state after successful connection`; `Sent peer
  list (2 peers) to 12D3KooWR9io…` (the ledger re-seed mechanism, live).
- (b) peersDiscovered: **PASS** — `peersDiscovered=1` (was 0).
- (c) bootstrap: **PASS** — `NetworkDetector: Network type updated:
  UNKNOWN -> WIFI (stable for 500ms)` (clean E8-class line); racing bootstrap
  on WIFI; phone's address snapshot contains
  `/ip4/147.81.41.188/tcp/9001/p2p/<Windows>/p2p-circuit/p2p/<self>` — the
  T14-pinned Windows endpoint relaying the phone (T14+D10 end-to-end).
- (d) ANR: **PASS** — zero ANRs, watchdog silent after start; the only match
  in both windows is the same single benign cold-start frame skip at
  15:56:46.194 (2.6s after spawn, pre-Compose-init).
- (e) message flow: **WARN** — history intact (120 msgs), one
  `UNIFICATION message_relay: ctx=send … msg=11e11549…` send-attempt line,
  `messagesRelayed=0` in stats, `undeliveredCount=1`. Delivery confirmation
  not yet observed in the captured windows.
- Phone->AWS leg: **UNVERIFIED this pass** — no AWS dial/identify line in the
  windows yet; the re-seed (Windows sent 2-peer list at 02:01:24Z) should
  produce it passively over the next minutes.

**PR #279 checks at 01:55Z:** Analyze actions/js/python/ruby PASS; rust
pending; CodeQL skipping.

**Verdicts:** REJOIN VIA LAN PASS; ANR FIX HOLDS ON D10 APK PASS; PHONE->AWS
UNVERIFIED (expected passively next); message delivery WARN.
The mesh is 3-node-connected at the transport level (Windows sees both
peers simultaneously). Ready for the operator's manual drop test.
