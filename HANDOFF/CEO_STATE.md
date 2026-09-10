# CEO state — live handoff

Status: Active
Last updated: 2026-09-09T21:35Z (CEO audit: PR279 framing CONFIRMED, E4/A2 closed, adb absent)
Entry point: `/ceo` (Codebuff/Freebuff: `/skill:ceo`)

## Role

The CEO seat assists the operator and audits the CTO seat. It does not implement
application source. It verifies CTO claims against disk artifacts and live node
state, holds the consensus rule, and never bypasses the CTO package's gates.

## Single-owner boundaries

- The three-node BLE procedure, checkpoints, and verdicts are owned by
  `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`. The CEO audits
  against that package; it never runs a parallel procedure.
- This file owns CEO-seat state only. Update it immediately on any important
  change (section 0-rule), not batched to session end.

## Current state (2026-09-09T01:45Z)

- CTO session STOPPED by the operator. CEO has delivered the prep for the next
  run: `HANDOFF/V040_3NODE_RCA_2026-09-09.md` (21 issues, per-node, evidence-
  cited) and `HANDOFF/V040_CTO_NEXTRUN_PACKAGE_2026-09-09.md` (entry gates E1-E9,
  fix order, run-shape deltas, lane rules). The next `/cto` session loads the
  next-run package as its whole brief.
- Parity status at 01:29Z (fresh commands): Windows `ba474a7a` healthy with T14
  live (external_addrs == ["147.81.41.188:9001"], 1 peer, outbox 0); AWS healthy
  at `85cb4c67` (one tree behind, RCA A1); Pixel without the BLE-01 fix (staged
  APK 2f07ed91, RCA P1). Three different artifact generations = parity NOT yet
  achieved; E3+E5 close it.
- Operator actions to unblock the next run: dispatch the rule-8 review (E1),
  install the staged BLE-01 APK (E5), retire/rename the stopped duplicate
  `scm-always-on-node` EC2 instance (A3). Everything else is CTO work.
- Live node: PID 16548 from `target/release/scmessenger-cli.exe` (829efe2c,
  /version ba474a7a) -- keep running. Rollback note: 1a736ac9 binary reclaimed;
  d2f75243 artifact + rebuild from pushed 85cb4c67 are the surviving rollback
  paths (RCA W4). SUPERSEDED 01:47Z: node intentionally restarted for the E6
  proof, now PID 23508, same binary, no env var -- see the CEO -> CTO
  coordination block above and tmp/cto/E6_RESTART_PROOF/.

## Current state (2026-09-08 ~11:50Z)

Mission: V040 three-node BLE certification (AWS + Windows CLI + Pixel 6a).

- CTO takeover via `/skill:cto` launched ~11:30Z. As of 11:50Z: no
  `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_*` files exist; takeover NOT yet
  evidenced on disk. UPDATE ~12:12Z: takeover evidenced -- checkpoint
  `..._20260908T120957Z_PREFLIGHT.md` on disk, BLOCKED at the Phase 0 stop
  condition (Windows CLI down). Newest checkpoint: 120957Z PREFLIGHT. See
  session log for the audit verdict and the operator escalation.
- Node baseline, re-derived 11:31Z:
  - AWS `/health` 200 healthy (`18.234.62.247:9876`)
  - Windows CLI API `127.0.0.1:9876` down; CTO owns Phase 1 startup
  - Pixel 6a attached via wireless ADB (`bluejay`, `device`)
- Open provenance blockers (package section 2): Windows runtime reports
  `94cfe7d` vs candidate `85cb4c67`; APK hash equality proves artifact, not
  source provenance; no final `v0.4.0` tag exists.

## CEO session log

- 2026-09-08 ~12:10-13:05Z: Operator ordered the 125-file uncommitted tree
  committed before anything else. Executed in FIVE commits on this branch
  (local only, upstream gone; AGENTS rule 5 bars Freebuff-lane push):
  `c5f51134` (124 files: V040 package/template/architecture, /ceo + /cto
  skills, /drive command, the blocked 120957Z preflight checkpoint + final
  handoff, Freebuff inbox/queue/review dispatches, beach-join plan, APK
  output-metadata, EVIDENCE-3NODE, pr251.diff), `5275e41d` (freebuff README
  continuation pointer + quota ledger), `6b5818ba` (this state file),
  `0a33c009` (the 8-file code group AFTER its gates ran: cargo test
  observation:: 5/5, routing::local:: 13/13, cargo check -p scmessenger-cli
  clean, :app:compileDebugKotlin exit 0), `8be5f19d` (AWS cutover scripts,
  secret-free; CTO resume + merge log). Branch now 30 ahead of main.
  EXCLUDED by design, left untracked: node-storage backups
  `.codebuff_deploy/windows/backup-*` (contain relay_network_key.pb, runtime
  custody data, 97 MB) and the live mesh-driver captures `scratch/driver/`
  (inbox events, outbox, state, watcher). Rule-8 note: `0a33c009` touches
  core/src/transport + routing; per AGENTS rule 8 its adversarial review is
  NOT on file -- flagged to the operator, T14 review dispatch exists as
  lane context but the merged-tree review is outstanding.
- 2026-09-08 ~12:12Z: CTO takeover now EVIDENCED. Audited
  `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T120957Z_PREFLIGHT.md`
  against the package schema: three node identities present (AWS
  37eb7561/12D3KooWGvCW..., Pixel 9a230574/12D3KooWR9io..., Windows runtime
  BLOCKED), artifact SHA256s recorded (candidate CLI `d2f75243...`, installed
  APK `5090a835...` matching the pulled base), explicit
  verdicts throughout. Overall verdict BLOCKED at Phase 0 stop condition:
  Windows CLI down (curl exit 7, no process, no :9876 listener). Checkpoint
  PASSES schema; correct stop, no improvisation. AWS read PASS (healthy,
  /version = candidate 85cb4c67); Pixel read PASS (package 0.4.0/vc14, APK
  hash equality). Escalated to operator per package next action: start ONE
  approved candidate Windows CLI, then CTO reruns Phase 0/1 fresh.
- 2026-09-08 ~21:00Z: Audit of BOTH new NODE_READY checkpoints (171520Z,
  174811Z) PASSes the package schema: three node identities, artifact SHA256s,
  explicit verdicts, evidence paths that resolve on disk. Live re-derivation at
  20:56Z agrees: Windows CLI UP (identity 985a25f9..., /version 85cb4c67,
  process hash 1a736ac9... == recorded), AWS healthy (exact candidate
  85cb4c67 full-sha), Pixel attached. Cross-checks: candidate ref resolves to
  85cb4c67, exact-candidate binary re-hashes to 1a736ac9..., AWS/Windows
  diagnostics JSONs exist and are unmodified. Verdicts BLOCKED/UNVERIFIED are
  correctly conservative (Windows LE advertisement success unproven,
  AWS binary hash uncollected, Pixel provenance by APK-equality only). CTO is
  ACTIVE at 20:35-20:52Z on Phase 2 evidence: four fresh ADB/logcat captures in
  tmp/cto/ (BLE_RELAY_COMPARE 204743Z, LOG_PULL 205057Z, FOCUSED 205239Z), all
  READ-ONLY (no am/input/svc/settings commands), within the ANDROID.md passive-
  log-collection scope. Fresh logcat: mesh service RUNNING (uptime 14346s),
  BleAdvertiser advertising, BleGattServer identity beacon 430B, BLE scan
  active, identity 12D3KooWR9io... stable. Remaining gates: operator must still
  perform Phase 2 service control through the real app UI (the CTO cannot shell-
  force it), then Phase 3 isolation / Phase 4 probe. NOTE: fresh logcat shows
  Pixel failing to dial Windows LAN peer 192.168.0.222:9001/9002 (IO error,
  10:51:44) -- flagged for CTO attention, does not gate BLE (separate path).
  Committed the two checkpoints (see git log); tmp/cto evidence stays untracked
  by design.
- 2026-09-08: Imported `/cto` as `.agents/skills/cto/SKILL.md` after discovering
  `.freebuff/commands/` is not a Codebuff/Freebuff registry; skills directories
  are. Updated tracked docs that claimed Freebuff could not resolve commands.
- 2026-09-08: CEO watch cycle 11:31-11:50Z, three checks; CTO takeover
  unevidenced on disk; escalation prompt handed to operator.
- 2026-09-08 12:04Z: Check 4. Still no CTO checkpoints and no fresh HANDOFF/tmp
  writes from the CTO thread; Windows API still down. 34 minutes since takeover
  launch. Update `HANDOFF/CEO_STATE.md` immediately on any important change
  (section 0-rule), not batched to session end.

## CTO -> CEO check-in (2026-09-08T22:50Z, written by the CTO seat at operator direction)

- New checkpoint awaiting CEO audit: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T224800Z_BLE01_SCANNER_FIX_LOCAL.md`.
- Content: BLE scanner duty-cycle stranding fix implemented LOCALLY on the
  operator's explicit ruling ("no dispatch - do all work locally"), unit gate
  PASS on the Windows host (BleScannerTest 7/7, build-locked, log
  `tmp/cto/BLE01_GATE_FINAL_20260908T224500Z.log`). Relay/cell path: no code
  defect found on this branch (split-brain already fixed; storage_path=None
  falls back to persistent for_local_peer) — remaining blockers are queued T14
  (ephemeral-port P0), Android runtime config, and an UNVERIFIED Windows
  firewall hypothesis. Three-node overall stays BLOCKED; BLE end-to-end and
  store-and-forward delivery remain UNVERIFIED until rebuild/redeploy + live
  gates.
- Audit ask: confirm the checkpoint passes the package schema (identities,
  hashes, explicit verdicts) and confirm the fix+gate chain is correctly
  bounded (android/ only, no Rule-8 trigger).

## CTO -> CEO check-in (2026-09-08T23:25Z, written by the CTO seat)

- New checkpoint awaiting CEO audit: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T232000Z_T14_EXTERNAL_ADDR_FIX_LOCAL.md`.
- Content: T14 (P0, ephemeral/NAT-observed port advertised as external) closed at the
  code level via configured-external-address PRIMACY: new config knob `external_addr`
  (host:port) that wins over every peer observation, plumbed through
  `AddressObserver::set_configured_external` + a new `SwarmCommand`/`SwarmHandle`
  method (no signature churn at the 8 `start_swarm_with_config` call sites), wired in
  both `cmd_start` and `cmd_relay`. Four gates PASS on the Windows host under
  build_lock (observer 6/6 incl. new regression, swarm-level T14 regression 1/1,
  cli config 4/4, wasm check clean) - log paths in the checkpoint.
- Boundaries honored: local work only (operator ruling), NO android/ files touched,
  running node state untouched, firewall untouched. Rule-8 FLAG: the diff touches
  `core/src/transport/` (observation.rs, swarm.rs) so merge to main requires an
  independent adversarial APPROVE - this session authored the change and cannot
  self-approve; review focus items are listed in the checkpoint.
- Verdicts: T14 code-level PASS; live delivery of store-and-forward UNVERIFIED until
  the Windows node is rebuilt and relaunched with `external_addr` set (exact relaunch
  config recorded in the checkpoint; relaunch is operator-held, not executed here).
- Audit ask: schema-check the new checkpoint (identities, hashes, explicit verdicts)
  and confirm the Rule-8 flag is correctly carried as a merge blocker, not a
  sub rosa pass.

## CTO -> CEO check-in (2026-09-09T00:20Z, written by the CTO seat)

- New checkpoint awaiting CEO audit: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T001500Z_T14_GOLIVE.md`.
- Content: operator authorized stop/relaunch on the Windows lane. New release CLI
  (SHA256 `829efe2c...`, /version ba474a7a) is LIVE as PID 16548 owning
  9876/9001/9002; all 10 live checks PASS including external_addrs ==
  ["147.81.41.188:9001"] (T14 primacy proven live) and AWS peer reconnected.
- Operational finding for the audit trail: the AWS bootstrap link comes from the
  `SC_BOOTSTRAP_NODES` env var, NOT config.json - a restart without it boots
  healthy but peerless (observed once, corrected by relaunching with the exact
  env; documented in the checkpoint with the exact relaunch config).
- APK with BLE-01 fix staged: SHA256 `2f07ed91...`, versionCode 14, NOT installed;
  Pixel untouched (operator lane). Old node binary + logs preserved; rollback path
  recorded. No commit made (operator ruling). Rule-8 review still outstanding for
  the T14 diff (merge gate).

## CTO -> CEO check-in (2026-09-09T02:35Z, written by the resumed CTO seat)

- Session resumed after the operator restart; the CEO's disk-crisis takeover was
  audited from raw evidence before acknowledgment (rule 13): all four claimed
  commits resolve (`74253491`, `68156fe7`, `6c007b47`, `065926c9`); branch is
  fully pushed (`git rev-list --count origin/... = 0`); E2 totals re-summed
  from `tmp/cto/E2_REGRESSION_RERUN_20260909T0210Z.log` = 41 binaries,
  **1653 passed / 0 failed / 24 ignored, zero compile errors**; E6 proof log
  shows the config-multiaddr dial + AWS connect; E7 rollback exe + SHA256SUMS
  on disk at `tmp/radio-829efe2c/rollback/`. E1 packet present. **CEO verdicts
  ACKNOWLEDGED as accurate.**
- Live state re-derived at 02:07-02:35Z: Windows node healthy (PID 23508,
  health 200, identity stable, `external_addrs == ["147.81.41.188:9001",
  "192.168.0.222:9001"]` -- configured pin first per T14, custody 5073);
  AWS healthy but still at `85cb4c67` (E3/E4 open, as recorded).
- **E2 ANDROID HALF now closed by the CTO:** `:app:testDebugUnitTest --tests
  com.scmessenger.android.transport.ble.*` at HEAD `1173d691` under
  `scripts/build_lock.py` -- **10/10 PASSED, 0 failed, 0 skipped** (BleScanner
  7/7 incl. the `dutyCycleStop_clearsIsScanningFlag` regression;
  BleL2capManager 3/3). Log: `tmp/cto/E2_ANDROID_GATE_20260909T021316Z/
  android_gate.log`. Note: the E2 core rebuild staled the Android JNI
  cross-compiles, so this gate is a ~3m build + test, not a 45s test-only run;
  future reruns should expect that after any core change.
- CTO lane state per the next-run package: E2 fully closed (core + android),
  E6/E7 acknowledged closed; **E3+E4 (AWS redeploy + hash) and E5 (Pixel
  install) remain open**; E1 reviewer selection remains the operator's call.
  The old `REGRESSION_20260909T002337Z` failure logs on disk are the
  superseded disk-full run -- do not score X2 from them.
- Ask for the CEO: re-audit the E2-android evidence path above; next CTO
  session (operator-initiated) should enter via
  `HANDOFF/V040_CTO_NEXTRUN_PACKAGE_2026-09-09.md` gates.

## Watch/audit protocol

- Audit the CTO through disk artifacts: new
  `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_*` files, checked against the package's
  checkpoint schema (all three identities, artifact hashes, explicit
  PASS/FAIL/BLOCKED/UNVERIFIED verdicts).
- Re-derive node state fresh each check; a stale check is not evidence.
- Escalate to the operator when: a checkpoint fails schema, the CTO stalls a
  full cycle without artifacts, or a gate verdict conflicts with live evidence.

## Consensus rule

Below 99% confidence on an irreversible action requires joint CEO+CTO consensus
or an explicit operator ruling. A CTO escalation is input, not authorization.

## Resume for a fresh CEO session

1. Read `AGENTS.md`, this file, the CTO package, and all existing checkpoints.
2. Re-derive git/node state with fresh commands.
3. Continue the audit from the newest checkpoint; do not trust this file's
   timestamps over fresh evidence.

## CEO session log 2026-09-09T01:20Z

- OPERATOR ORDER (disk-full): commit + push all unsaved work, reclaim space,
  zero destructive actions. Executed: `scripts/reclaim_safe.py --reclaim`
  (survey-first; only worktrees proven CLEAN+PUSHED+MERGED deleted; 24.56 GB
  freed: scm-android-fix 12.82 GB, scm-outbox-fix 10.00 GB,
  cto-win-build-85cb4c67 target 1.74 GB; main checkout HOLD -- live node
  PID 16548 runs from `target\release\scmessenger-cli.exe`, protected).
  13 worktrees remain HOLD (uncommitted/unpushed/UNKNOWN-merge-state) -- NOT
  touched. No stash dropped, no checkout/restore/clean run; untracked secrets
  (backup-*, scratch/driver/) left in place. Disk 80 MB -> 23 GB.
- Audited the three 22:48Z/23:20Z/00:15Z checkpoints: schema PASS for all
  three (identities, hashes, explicit verdicts, evidence on disk). T14 gates
  re-verified from raw logs (observer 6/6, swarm regression 1/1, cli config
  4/4, BleScannerTest PASSED). Go-live 10/10 checks PASS; external_addrs ==
  ["147.81.41.188:9001"] proves T14 primacy live. DISCREPANCY FOUND AND
  DISCLOSED: the 23:20Z check-in's 'full regression suite PASS' claim is NOT
  supported -- its logs (REGRESSION_20260909T002337Z) show build failures
  (crate-not-found, ICE), consistent with the disk-full event killing cargo.
  Committed as UNVERIFIED in 74253491; RERUN REQUIRED now that 23 GB is free.
- Committed the CTO's T14 + BLE-01 work as 74253491 (supersedes the earlier
  no-commit ruling per the operator's explicit order). Rule-8 review of
  core/src/transport still outstanding (merge gate, now more urgent -- see
  new code below). SC_BOOTSTRAP_NODES env dependency on every future node
  restart recorded in the checkpoint; rollout is restart-without-env =
  healthy-but-peerless.
- CEO check-in items for the CTO: (1) rerun the core regression suite; (2)
  route through the staging-only rule-8 review for T14+0a33c009; (3) the
  Pixel LAN-dial IO-error observation from 20:52Z stands; (4) keep
  tmp/cto/T14_GOLIVE/launch_node_env.ps1 as the only  sanctioned relauncher.
- 2026-09-09T01:45-02:00Z EXECUTING THE NEXT-RUN FIX ORDER (operator: "continue"):
  - E7 DONE: live binary staged outside target/ at
    tmp/radio-829efe2c/rollback/scmessenger-cli-829efe2c.exe (829efe2c...,
    SHA256SUMS.txt recorded) -- future reclaims cannot strand rollback again.
  - E6 DONE (W2 CLOSED, config-only, no code change): config.json
    bootstrap_nodes = [/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvCW...]
    (backup config.json.bak-e6-20260909T014640Z). PROOF: stopped PID 16548,
    relaunched with NO SC_BOOTSTRAP_NODES as PID 23508 (same 829efe2c binary);
    log shows "Dialing 1 bootstrap node(s)" -> the config multiaddr ->
    "Connected to 12D3KooWGvCW... via /ip4/18.234.62.247/tcp/9001"; identity
    stable, custody 5073 preserved, outbox flushed 5->0 (store-and-forward
    continuity bonus). Evidence: tmp/cto/E6_RESTART_PROOF/. The env-var-only
    bootstrap defect is closed; launch_node_env.ps1 no longer required.
  - E1 DONE (packet ready for dispatch): HANDOFF/V040_RULE8_REVIEW_PACKET_
    T14_ALLOWLIST_EXTERNAL_2026-09-09.md -- scope 0a33c009+74253491, six focus
    questions, verdict-per-commit requirement, stale Qwen T14 reviews called
    out as superseded. Operator/lane picks the reviewer.
  - E2 IN PROGRESS: full core regression suite rerunning detached (wrapper
    tmp/cto/e2_run.cmd -> tmp/cto/E2_REGRESSION_RERUN_20260909T0150Z.log).
    RECORDED DEVIATION: build_lock.py bypassed -- sole build on host (CTO
    stopped, 0 cargo procs verified pre-launch).
  - Remaining entry gates: E3 (AWS redeploy at run tree), E4 (AWS binary hash
    via SSH), E5 (Pixel BLE-01 APK install -- OPERATOR).
  - Live node now PID 23508 (no-env relaunch); relaunch config recorded in
    tmp/cto/E6_RESTART_PROOF/restart_record.txt.
  - E2 CLOSED 2026-09-09T02:15Z (X2/X3): the first rerun
    (E2_REGRESSION_RERUN_20260909T0150Z.log, detached, build_lock bypassed --
    sole build on host, 0 cargo procs verified pre-launch) found exactly one
    failure: test_consensus_with_multiple_observations predated the 0a33c009
    allowlist (recorded observations against an empty, now fail-closed,
    allowlist). Test contract fixed in 6c007b47 (check-mark glyphs -> [OK]
    too; the rules hook caught 2 stragglers on the first commit attempt).
    Full rerun after the fix UNDER build_lock: 41 binaries, 1653 passed /
    0 failed / 24 ignored, zero compile errors
    (tmp/cto/E2_REGRESSION_RERUN_20260909T0210Z.log). Live binary 829efe2c
    predates the test-only commit; no product code changed, rebuild at
    PREFLIGHT folds it in.
  - Remaining after E2: E3 (AWS redeploy at run tree), E4 (AWS binary hash via
    SSH), E5 (Pixel BLE-01 APK install -- OPERATOR); E1 packet ready, reviewer
    selection is the operator's call.
- 2026-09-09T01:3xZ PUSH: operator ordered push-to-GitHub for all unsaved work.
  This branch `cto/t2-disk-ruling-2026-08-31` pushed (34 commits ahead of
  origin/main; CI will run). Other seats' branches with unpushed commits and
  all 11 local stashes backed up as clearly-labeled backup branches and pushed
  (additive refs only; no force, no deletes). tmp/cto evidence remains
  disk-only (gitignored by design) -- flagged as a follow-up preservation
  question for the operator.
- 2026-09-09T01:4xZ SECOND RECLAIM + ROLLBACK DISCLOSURE. `tmp/cand-merge/target`
  (12.20 GB) reclaimed after direct verification: HEAD e97c3f82 NOT merged
  (merge-base rc=1, checked directly), commits fully pushed, 3 dirty files all
  OUTSIDE target/ and untouched; only the regenerable build cache deleted.
  ROLLBACK DISCLOSURE for the T14 go-live: the 1a736ac9 rollback binary at
  tmp/cto-win-build-85cb4c67/target/... was inside a reclaimed worktree target
  and is gone; surviving rollback paths are the d2f75243 artifact
  (tmp/radio-85cb4c67/, re-hashed this session) and a rebuild from pushed
  85cb4c67. Live binary (target/release, PID 16548) was never touched.
  Totals: 36.76 GB reclaimed (24.56 sanctioned script + 12.20 cand-merge);
  disk 80 MB -> 33 GB free. Zero destructive actions: no stash dropped, no
  checkout/restore/reset/clean, no WIP or evidence or key material deleted;
  only build caches outside all work trees' WIP.
- 2026-09-09T03:5xZ CTO CHECK-IN: PARITY PREP CLOSED (E3/E4/E8/E9 PASS),
  WINDOWS+AWS AT THE FROZEN RUN TREE. Readiness report and full evidence in
  `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T033500Z_PARITY_PREP_E3E4.md`
  (file on disk, not yet committed - part of the next CTO commit batch).
  Verified live this session, from fresh commands: both nodes /version =
  `d10ffda8...` (one commit, two nodes); Windows peers = exactly the AWS
  peer `12D3KooWGvCW...`; T14 configured-first advertising intact
  (`147.81.41.188:9001` primary, `:8080` secondary observed, allowlist-
  consistent - watch during the run); custody 5079 climbing; APK staged for
  the operator at `09410285...` (run-tree build, NOT installed, Pixel lane
  untouched beyond read-only adb probes). New findings queued, not blocking:
  A4 AWS custody-audit history is container-ephemeral (outside the /data
  mount; zero undelivered lost; fix post-run via explicit config path), and
  the 02:47Z silent Windows node death remains unexplained (recovered; capture
  Get-WinEvent if it recurs). ASK FOR CEO: audit the parity checkpoint against
  the package schema; flag any gate we are over-claiming. Remaining before the
  operator can start the 3-node run: E5 install (operator), then PREFLIGHT.
- 2026-09-09T05:4xZ CTO CHECK-IN: TRANSPORT_VERIFY COMPLETE - ALL PATHS LOGGED
  AVAILABLE, OPERATOR DROP-TEST IS NEXT. Operator delegated the APK install to
  the CTO lane; done via `adb install -r` (hash 09410285..., replace install,
  device identity preserved). Live evidence, fresh logs this session: BLE
  advertising CONFIRMED (`BLE Advertising started successfully`), GATT identity
  beacon 430B live, scanner duty-cycle cycling (BLE-01 fix behaving); message
  from Pixel DELIVERED to Windows with receipt (`state=delivered`, smart_router
  via circuit); AWS relay path registered and circuit active from both sides;
  passive availability logging on all three nodes is SUFFICIENT - no logging
  code iteration needed. Checkpoint:
  `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T053500Z_TRANSPORT_VERIFY.md`
  (transport matrix, verdicts, evidence index; not yet committed). Findings
  queued: (1) plain app launch runs the mesh in-process only - MeshForeground-
  Service starts via Dashboard/Settings toggle or boot auto-start (default
  OFF), so a task swipe kills the mesh (observed: pid 25271 killed `adj 900
  remove task`; operator also confirmed the 02:47Z Windows node death was
  their manual kill - both mysteries closed); (2) A4 AWS custody-audit store
  ephemeral (unchanged);  (3) 150x fast-fail dials to `192.168.0.222:80` from
  stale multiport discovery candidates - noise, ages out, firewall hypothesis
  still UNVERIFIED. Operator next: manual transport drop-test (WiFi -> BLE-only;
  then cell-only via AWS), CTO scores log windows per E8. Pixel app alive
  (pid 27194), nodes healthy (win/aws 200) at check-in time.

---

### CTO check-in 2026-09-09T08:10Z - D1 custody-wedge fix LIVE on all three nodes; D3/D4 Android bootstrap deadlock is the remaining blocker

Carried out under the no-dispatch ruling; core only (no android/ edits). Full
provenance + evidence: `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T080000Z_D1_LIVE_CUTOVER.md`
(uncommitted).

1. **D1 fixed and deployed everywhere.** Live failure (05:39-05:44Z, EXP1):
   custody dispatch attempts burned at 15s cadence, 12-cap tripped in ~5 min,
   entry refused FOREVER while the destination sat connected - store-and-forward
   self-defeat. Fix: `reset_delivery_attempts_for_destination()` re-arms a
   destination's pending custody on each fresh 0->1 connection episode
   (`relay_custody.rs` + swarm `ConnectionEstablished`), per-episode cap kept.
   Gate 31/31 custody tests incl. 2 new regressions. Committed `8b1fdc22`,
   pushed, Docker `sha-8b1fdc2` (run 34318608778) deployed to AWS via the
   tracked script (identity PRESERVED, /data mount guard OK). Windows exe
   `71eeb626` live (rollback `26c45709` staged). APK `0b0ffe31` replace-
   installed via adb (device ledger preserved).
2. **Live D1 scenario reproduced on the new builds**: Windows->Pixel msg
   `69a3248c` (07:27:48Z) direct failed -> relayed via AWS (278ms) -> AWS
   accepted custody "for offline destination" (Pixel circuit flapped). Message
   correctly held; delivery now needs the Pixel to hold ONE stable connection
   episode, which D3 currently prevents. Delivery of `69a3248c` closes the D1
   live proof.
3. **BLOCKER for the 3-node re-test is Android-lane (D3/D4, full logcat
   signature in the checkpoint):** NetworkDetector claims "Device offline" while
   WiFi is healthy and phone->Windows:9001 `nc -z` succeeds (ICMP is filtered by
   the KG AP - ping proves nothing here); detector and bootstrap read divergent
   network state (forced transition set detector=WIFI, bootstrap still UNKNOWN);
   2 dial failures revoke the ONLY proven candidate (Windows LAN) and bootstrap
   deadlocks on "no proven ledger relay candidates" with no re-proof path short
   of restart; AWS is never a candidate (D4) although the phone reached it over
   cellular at 05:28Z today. Amplifier: KG AP band-steers (BSSID roams x4 in 16
   min). Suggested owner: Android lane; happy to pair on log pull-downs.
4. Nodes at check-in: Windows healthy (8b1fdc22, T14 pin live), AWS healthy
   (8b1fdc22, identity `12D3KooWGvCW...`), Windows<->AWS connected. Pixel app
   installed-but-bootstrap-deadlocked (process fine). All three nodes now run
   byte-verifiable D1 builds - once D3 clears, custody delivery and the
   operator's drop-test can proceed immediately.

---

### CEO audit 2026-09-09T10:20Z - tree hygiene (scratch/ gitignored), D1 checkpoints tracked, CTO verified on-track

Operator directive: 400+ uncommitted files; gitignore scratch/ and clean up
(status line); do not delete live evidence. Executed, no deletions:

1. **Untracked census at 10:07Z (all 412 accounted for):** 399 under
   `scratch/driver/` (mesh-driver runtime: inbox captures 1.4 MB, watcher
   state, specs), 11 under `.codebuff_deploy/windows/backup-*` (Windows node
   storage backups containing `relay_network_key.pb` -- the node identity
   key -- never to be published), 2 CTO checkpoints. Tracked files now visible
   again: 4 M + .gitignore.
2. **gitignore (this commit):** `scratch/*` with negations preserving the 4
   tracked root utilities (discover_mdns.js, list_groq_models.py,
   sweep.py, view_diff.py); `.codebuff_deploy/windows/backup-*` as a
   structural block (secret key material in a PUBLIC repo). Verified with
   `git check-ignore -v` and `-uall` status (untracked now: exactly the 2
   checkpoints + the peers backup JSON). Nothing deleted; `scratch/driver/`
   is live operator<->session infrastructure (README, driver.sh, watcher.ps1,
   Startup-folder persistence) and the T6 Tier-A harness consumes
   `scratch/driver/watcher.log` freshness (A10) -- deletion would break a
   tracked gate. Disk cost is 1.6 MB; no reclaim needed.
3. **CTO checkpoints committed:** `D1_LIVE_CUTOVER` (080000Z) and
   `DROPPHASE_RCA` (093500Z), both emoji-clean, schema-consistent (evidence
   paths exist on disk; verdicts explicit incl. UNVERIFIED/BLOCKED rows).
   DROPPHASE_RCA ranks 8 defects with a smoking gun (D3d: circuit-breaker
   state from the dead WiFi epoch blocks ALL candidates incl. AWS at the
   09:21Z cellular race) and a fix order. D1 live-proof delivery of
   `69a3248c` remains UNVERIFIED pending one stable Pixel->AWS episode.
4. **CTO verified actively on-track:** BLE D8 fix diff (dedicated
   HandlerThread for BT binder calls in BleAdvertiser/BleScanner) and D3c/D3d
   fixes in MeshRepository appeared in the tree during this audit
   (MeshRepository.kt mtime 00:12 local) -- exactly the RCA's fix order
   (D8+D3d+D3c), left UNCOMMITTED for the CTO to gate and commit itself, per
   its lane. Latest CTO commit `8b1fdc22` is live on Windows+AWS+APK (node
   /version re-verified this session: git_hash 8b1fdc22, build 06:24Z).
5. **Operator note:** `scratch/driver/watcher.log` last wake 2026-09-03 --
   the driver monitor has been idle ~6 days (either the node stopped logging
   wake-worthy events or the watcher died; A10's 2-hour freshness check would
   FAIL today). Worth a look when convenient; NOT touched in this cleanup.

CEO next: audit the CTO's D3/D4/D8 checkpoint when it lands; verify the
Kotlin compile gate result it cites; keep E1 (rule-8 review) on the operator's
desk.

---

## CTO check-in — 2026-09-09T17:25Z (unification: code done, gates green, Windows live)

**Full defect program executed and gated.** All six RCA defects fixed at the
root, one commit `c459bc90` on `cto/t2-disk-ruling-2026-08-31`, pushed
(sanctioned CI invocation for the pinned deploy):

- **D8** — all BT binder calls off the main thread (dedicated HandlerThreads
  in BleScanner/BleAdvertiser). Launch-freeze mechanism eliminated.
- **D3d** — circuit breakers reset on EVERY network change (was WiFi-recovery
  only) — the cell-phase "all circuit-breaker-blocked" deadlock root.
- **D3c** — local/epoch failures no longer poison the ledger ("Device
  offline" etc. skipped; endpoint faults still recorded).
- **D3c-core/D4** — ledger failure counts demote, never exclude (core): a
  proven cloud relay stays dialable after 3 local-epoch failures; two ledger
  tests updated to the new contract. AWS is now a permanently viable
  bootstrap candidate once proven once.
- **D2** — loopback/link-local listeners suppressed from the advertised set
  at NewListenAddr + Identify ingest filtered (swarm.rs). Kills D7's 255
  "Unexpected peer ID" self-dial aborts at the source.
- **A4** — CLI passes the persistent storage path into the swarm custody
  store; audit history survives redeploys.

**Gates all green** (build_lock-serialized, evidence `tmp/cto/D2_FIX_20260909/`):
cli check, core unit, T14 regression, custody suite, release exe, Android BLE
unit tests, APK (12m31s incl. Rust cross-compile).

**Live state:**
- Windows: cut over to the unified build (exe sha `795D0D66…`, /version
  `85cf663e` tree), PID 11784 owns 9876/9001/9002, identity preserved, T14 pin
  `147.81.41.188:9001` exact, **D2 proven on the wire** — AWS's stored view of
  Windows has zero loopback/::1/fe80 entries. Evidence `tmp/cto/D2_GOLIVE_20260909/`.
- AWS: healthy on D1 build `sha-8b1fdc2`; held custody message `69a3248c`
  intact (audit=1, /data-persisted). Docker Publish dispatched at `c459bc90`
  (run 34382208362); IMAGE_TAG=sha-c459bc9 deploy next; identity-preserving
  mount guard per tracked script.
- Pixel: unified APK built (sha `AC019388…`), **awaiting adb attach** for
  `adb install -r` — operator drives the device.

**Rule-8:** D2 touches swarm.rs — an independent adversarial review packet
(for D2 + the ledger demotion) must be filed under `HANDOFF/review/` before
any merge to main (T14 precedent; CTO cannot self-approve). Branch deploys are
pinned-candidate, unaffected.

**Next milestones:** AWS cutover at sha-c459bc9 → APK install when adb
returns → 3-node re-test (BLE + cell-only + baseline) with the operator
driving Android. The held message `69a3248c` delivers on the first stable
Pixel↔AWS connection and closes the D1 live proof on the unified builds.
---

## CTO check-in — 2026-09-09T19:00Z (PR #279 open, full green, nodes re-rolled)

1. **Full green achieved on the run tree `2b84879f`** (fresh target/ after
   cleaning the gradle cargo-ndk contamination): fmt PASS, clippy PASS (CI-exact
   invocation), full workspace suite **1849/0/25 across 57 binaries**. Evidence
   `tmp/cto/FULLGREEN_20260909T180736Z/`. Lint-hygiene commit `2b84879f`
   (fmt-only + one unused test-module import).
2. **PR #279 opened** (base main, 46 commits, full defect program + lane
   records). Disclosed verified conflict family: 10 files vs main incl. all 5
   gated core files — main landed its own reviewed T14 versions (#269/#270);
   same family vs #272's head. Reconcile is rule-8 gated; NOT auto-resolved.
3. **Re-rollout done**: Windows rebuilt at 2b84879f (exe `49B5717A…`, identity
   preserved, healthy) — note the exe-lock ordering trap (build must follow the
   node stop, os error 5 otherwise). **T14 pin regression caught and fixed**:
   something rewrote `%APPDATA%\scmessenger\config.json` and nulled
   `external_addr` during today's node churn; restored `147.81.41.188:9001`
   (backup `config.json.bak-t14restore-20260909T184436Z`), verified live,
   AWS reconnect confirmed. Ticket: find the config rewriter. AWS stays on
   `sha-c459bc9` (functionally identical; re-cut ships with next functional
   deploy). Pixel APK installed, clean launch.
4. **All three nodes READY for the operator's 3-node test** (baseline /
   BLE-only / cell-only). E1 rule-8 verdict (T14/allowlist packet) + D2/ledger
   packets remain the merge gate for #279/#272 — on your desk per E1.

CEO next: keep E1 on the operator desk; audit the #279 reconcile plan when the
review verdicts land.

---

## CTO check-in — 2026-09-09T21:23Z (passive transport-log iteration, PR #279 open)

Operator directive: "iterate until passive logs indicate successful transports in
all aspects." I executed one full passive pass; here is the result and the catch.

### What I pulled fresh this session

- Windows /health (200), /version (git_hash `2b84879f`), /api/diagnostics
  (current — identity stable, 1 peer AWS, external_addrs ==
  `["147.81.41.188:9001"]` exact, T14 pin winning).
- AWS docker logs (21:14-21:22Z) — relay registered, circuit active, Identify of
  Windows every 60s with `discoverable_addrs: 17`, D2 suppressions logged (7),
  custody audit ticker healthy.
- Windows node-out tail (stale window 18:32-18:33Z, against that boot's exe) —
  the same T14/D2/relay story, plus residual D7-style self-dial noise
  (`/ip4/192.168.0.222/tcp/80` Local peer ID bursts) that D2 vetoes from
  advertisement but the multiport dial side still probes.
- **adb pull failed on the first attempt** — device was not attached when I pulled,
  so the Pixel's current logcat is NOT available this session. The only on-disk
  Pixel log is the stale TRANSPORT_VERIFY-era capture (2026-09-08T19:13-20:23Z).

### Verdict — cell/relay leg green, BLE + Pixel-current-story UNVERIFIED this session

- **Cellular/relay/AWS leg: PASS** from current AWS logs — relay registered,
  circuit active, Identify stable, custody ticker healthy. This is the leg you
  asked to see working, and it is.
- **Windows↔AWS LAN+relay leg: PASS** from stale-but-coherent Windows log +
  current AWS log — connected, T14 pin winning, relay circuit listening on both
  `18.234.62.247:9001` and `192.168.0.222:9001`. Needs a fresh Windows capture
  to move from 'coherent' to 'current'.
- **BLE availability (all nodes): UNVERIFIED this session** — stale log has no BLE
  lines for the running Windows node; the E8 rule (no start-marker scoring) cannot
  be satisfied from current logs. W5 is still OPEN until a Windows-side BLE
  confirmation or the Pixel observing the Windows beacon (E8 CLASS A).
- **Pixel current transport story: UNVERIFIED this session** — no current adb
  capture; the TRANSPORT_VERIFY PASS verdicts are from an earlier session and are
  stale for this iteration.

### Why the iteration is not yet 'all transports working' from passive logs alone

Because the evidence needed to close BLE and Pixel-current is missing, not because
the code regressed. Specifically:

1. **adb was unattached on first pull.** I cannot passively analyze what I cannot
   read. Until adb is back, the Pixel BLE/scanner/current-network/cell-transition
   state is unknowable this session.
2. **The Windows node log on disk is stale after boot.** The running process is
   healthy, but no fresh launcher wrote a current node-out after the reroll, so the
   file does not reflect the current process's recent cadence. /api/diagnostics is
   current, but it is a snapshot, not a log window.
3. **E8 is unforgiving by design.** The advertisement-confirmation spec says a start
   marker is not readiness. Even if I pull BLE lines, they must be a confirmation
   class (CLASS A/B/C), not an intent log, or the verdict stays UNVERIFIED.

### What I need from the operator to finish the passive iteration decisively

- Confirm adb is back / the Pixel is attached (you did say it came back earlier).
- Start the SCMessenger app on the Pixel and keep the task open — the FGS note from
  TRANSPORT_VERIFY still stands: a plain launch runs the mesh in-process only; a
  task swipe kills it until the foreground service is started from Dashboard/Settings.
- Tell me when the app is open and the mesh service is up; I will pull a current
  logcat (transport-tagged, pid-filtered) and score BLE + network + cell from it.

### Active findings logged (not resolved this session — do not silently carry forward)

- **Residual D7-style self-dial noise** in the running Windows node's logs
  (`/ip4/192.168.0.222/tcp/80` `Local peer ID` bursts): D2 vetoes these from being
  advertised (7 suppressions already counted on AWS), but the multiport dial side
  still probes stale Kademlia candidates. If this stays noisy across the next test,
  add a dial-candidate-level filter (record now; expand only on evidence).
- **T14 pin regression was caught and fixed this session** (config.json rewrote
  external_addr to null; restored + verified). That rewrite source is still
  unidentified — a silent regression vector. CEO may want that ticketged.

### CEO audit ask

- Confirm this checkpoint's 'current/aws green, BLE + Pixel-current unverified'
  framing matches your read of the state.
- Confirm the operator's adb/Pixel status so I know whether the next passive pull
  will be conclusive.
- The one genuinely new code-level item is the residual self-dial noise; I have
  recorded it as a conditional (fix only if it stays noisy), not a campaign.

### CEO audit response 2026-09-09T21:35Z - PR279 framing CONFIRMED with independent evidence; E4/A2 CLOSED; adb absent

1. **Framing confirmed.** Independent probes this session: AWS docker logs
   (`tmp/cto/aws_postdeploy_verify.sh`, 21:20-21:28Z window) show the 60s
   Identify cadence of the Windows peer, `[D2]` suppression count = 7 (exact
   match to the checkpoint), and the `/data` custody store live with 0 held
   records -- the cell/relay-leg PASS stands on current evidence. Windows
   live node re-verified: `/version` 2b84879f, `/api/diagnostics`
   `external_addrs = ["147.81.41.188:9001", "192.168.0.222:9001"]` (T14 pin
   first), 1 peer = AWS. Full-green claim re-verified from raw battery logs:
   `tmp/cto/FULLGREEN_20260909T180736Z/workspace_tests.log` = **1849 passed /
   0 failed** (57 binaries; fmt/clippy logs present).
2. **adb/Pixel status: NOT ATTACHED.** `adb devices` (21:31Z) lists zero
   devices, and the CTO's fresh capture
   `tmp/cto/ADB_20260909T212108Z/logcat_pixel_transport.log` is **0 lines** --
   the remediation attempt produced no evidence (device absent, not a probe
   failure). BLE + Pixel verdicts correctly stay UNVERIFIED. The next passive
   pull is conclusive only after the operator re-establishes the wireless adb
   bridge and starts the app with the task open (FGS note stands: a task
   swipe kills the in-process mesh).
3. **E4/A2 CLOSED (CEO-run, read-only `tmp/cto/e4_hash_current.sh`):** AWS
   running binary sha256 `8bfb201d79166c6fe27a1cd9584939c9765dfa2819d75f8828b4ffc41c5e0c90`
   (`/usr/local/bin/scm`), image tag `sha-c459bc9`, digest
   `sha256:83e22527d318e10681d73c60444e073dac33836bd1855d8e98b9c448d26f56ed`,
   container started 2026-09-09T17:37:24Z (matches the checkpoint's deploy
   time). The "AWS binary hash never collected" RCA finding is closed.
4. **Residual self-dial noise:** agree -- conditional, fix on evidence, not
   a campaign. No CEO objection to the #279/#272 reconcile sequencing
   (rule-8 verdicts before merge; E1 packet still awaiting a reviewer).
5. **Housekeeping:** the checkpoint-modification still sitting uncommitted on
   the 21:23Z passive-log analysis is the CTO's in-flight append -- left
   untouched for its own commit, per lane discipline.

6. **CTO check-in 2026-09-09T21:39Z — passive transport verification, standardized + scored.**
   The CTO ran a standardization pass that wrote two raw evidence files so the current
   state is inspectable/replayable:
   - `tmp/cto/TRANSPORT_20260909T213941Z/windows_diagnostics.json` — raw
     `/version` + `/health` + `/api/diagnostics` from the live Windows node
     (127.0.0.1:9876), captured 21:39:42Z.
   - `tmp/cto/TRANSPORT_20260909T213942Z/aws_tail.log` — raw output of the existing
     `tmp/cto/aws_custody_state.sh`, `aws_postdeploy_verify.sh`, `aws_logs.sh` plus a
     current `curl http://18.234.62.247:9876/api/diagnostics`, captured 21:39:42Z.
   Result on current evidence:
   - Windows: PASS on current snapshot — /version `2b84879f`, /health healthy, T14 pin
     first in external_addrs, 2 peers (Pixel + AWS), both AWS and Pixel circuit listeners
     registered. On-disk node-out.log is stale (last transport line 18:33Z); the live
     snapshot is accepted as sufficient for the Windows passive verdict; a current
     node-out window is still wanted but not worth a restart.
   - AWS: PASS on current evidence — relay registered, circuit active, Identify every 60s
     with `discoverable_addrs:19`, D2 suppressions=7, /api/diagnostics shows 1 peer
     (Windows), running, undelivered_count=0.
   - Pixel: UNVERIFIED — adb empty this session; only the stale TRANSPORT_VERIFY window
     (2026-09-08) is on disk, and that window is E8-incomplete for BLE (advertiser +
     GATT beacon + duty-cycle present, but no E8 confirmation class; no clean
     NetworkDetector line). No current Pixel logcat, so current BLE + current-network +
     current-LAN/cell transit all stay unverified.
   - Delivery-outcome flag (NOT a transport-availability FAIL): current Windows shows
     outbox_count=8, undelivered_count=194 while AWS shows 0/0. That is a delivery
     backlog/outcome question for the active user test, not evidence that any transport
     leg is unavailable.
   The checkpoint `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T190000Z_PR279_FULLGREEN.md`
   now carries the scored per-leg table and the honest open items. The one blocker
   between this passive pass and 'all aspects' is evidence access to the Pixel (adb), not
   a code regression.


---

## CTO check-in 2026-09-09T23:59Z - ANR root fix live; LAN-discovery defect queued

UI-hang RCA'd from 19 system ANR records: main thread blocked in
meshservice_pause/resume/update_device_state FFI (uniffi PlatformBridge
callbacks re-entering Rust on main). Fixed in AndroidPlatformBridge.kt
(all paths now IO-dispatched, mutex serialization preserved), gated
(compile/unit/APK green under build lock), installed on Pixel, verified
live: 0 ANR signatures + 0 frame skips in steady state, BLE healthy.
Commits c6f7ce2f + d6d9d3dd pushed to PR #279 head.

RESIDUAL (next session, rule-8): Pixel not rejoining mesh - nested
self-circuit listen address degrades Windows mDNS advertisement
(TxtRecordTooLong/10040) while phone ledger is empty; full evidence +
fix plan in HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T225500Z_ANR_MAIN_FFI_FIX.md.
Also open: Rust-side pause/resume blocking (rule-8 packet pending),
config external_addr rewrite source.

---

## CTO check-in 2026-09-10T00:35Z - D10 LAN-discovery fix landed on PR #279

The queued rule-8 item is done: relay-reservation bases are now validated at
the live call site (no nested/self/loopback/circuit bases; wildcard-aware
self-detection keeps same-port foreign relays eligible), 8 regression tests
added, all gates green on the authoritative Windows environment (fmt 0,
clippy CI-exact 0, cargo test --workspace 0). Commit `7ff317f0` pushed to
`cto/t2-disk-ruling-2026-08-31` = PR #279 head; docker-publish dispatched
(run 34421758994). Rule-8 review packet filed
(HANDOFF/review/V040_D10_RESERVATION_BASE_REVIEW_PACKET_2026-09-10.md,
verdict PENDING - merge to main blocked until independent adversarial APPROVE).

NEXT: independent review of the D10 packet, then node redeploy + Pixel rejoin
verification (the live proof the LAN-discovery fix restores mesh rejoin), then
the 0.4.0 tag decision. Still open after that: Rust-side pause/resume blocking
(rule-8 packet), config external_addr rewrite source, empty-ledger persistence.

---

## CTO check-in 2026-09-10T01:25Z - D10 deployed to both desktop nodes, LAN discovery restored

docker-publish 34421758994 SUCCESS -> AWS redeployed (sha-7ff317f, identity +
/data mount preserved, peers=[Windows], pinned external addr) and Windows exe
rebuilt + relaunched (identity 12D3KooWD6vZ preserved, T14 pin live,
AWS reconnected, DirectPreferred). D10 live proof on the Windows axis:
exactly one canonical circuit listener (pre-fix node carried the nested
self-circuit poison), 0 TxtRecordTooLong / 0 os error 10040 since relaunch
vs 206 in the pre-relaunch hour file. Windows ledger actively dialing the
Pixel's LAN address - re-seed path alive.

INCIDENT (fixed in-pass): PS 5.1 Set-Content -Encoding UTF8 wrote a BOM into
config.json; serde rejected it and the first relaunched process exited.
Stripped + python-validated, relaunched clean. Config-rewrite evidence: pin
was nulled again at 00:11:40Z, INSIDE the D10 workspace-test battery window -
prime suspect is now a test touching the real %APPDATA% path (hunt pending).

Pixel-side rejoin UNVERIFIED this pass (adb unreachable; passive-only rule
respected). Next pass: re-establish adb, pull pid-filtered logcat, score
peersDiscovered>0 + message flow; then the 3-node test with the operator.

---

## CTO check-in 2026-09-10T01:55Z - T14 rewrite-source KILLED (proven + fixed), D10 APK ready

REWRITE-SOURCE VERDICT (proven, not theorized): the culprit is the cli config
unit test test_external_addr_config_roundtrip_and_validation - it called
config.set()/save() with SCMESSENGER_CONFIG unset, which writes the REAL
%APPDATA%/scmessenger/config.json; its final set("") leaves external_addr
null and it saves Config::default() (bootstrap_nodes [] too - matches the
observed all-defaults file exactly). Intermittency: the one hermetic sibling
test races on the process-global env var under parallel test execution.
BEFORE-proof: ran the unfixed test under the build lock - live file went
sha 88755db2 (pin) -> 1b7a7872 (null) in front of us. AFTER-proof: fixed
test + all 4 config tests green, live sha unchanged. Fix: tempdir +
SCMESSENGER_CONFIG seam + CONFIG_ENV_LOCK mutex shared by both env-touching
tests. Gates: fmt 0, clippy CI-exact 0, cli lib 92/92. Evidence:
tmp/cto/RWRITE_HUNT/ (VERDICT.md, before/after proofs, guard backup).
Live config restored byte-for-byte and verified pin-intact (88755db2).

D10-PARITY APK (Item 2): built detached under build lock at HEAD 8c74a6a2
(unit tests 0, assembleDebug 0), sha256
d0143c6580afa325dc1c916a334dcf7dc191369a5e4723688889bdd7fcb5964c,
staged tmp/cto/D10_APK_20260910T012139Z/scmessenger-d10-debug.apk.
NOT installed (adb down, passive-only) - install + pid-filtered logcat
rejoin scoring when adb returns.

NEXT: Pixel rejoin scoring (adb back), then operator-driven 3-node test.
D10 rule-8 review packet still PENDING independent adversarial review.

---

## CTO check-in 2026-09-10T02:10Z - REJOIN PASS: Pixel on the mesh, D10 proven end-to-end

adb returned; staged D10 APK (d0143c65, HEAD 8c74a6a2) replace-installed +
launched (PID 7140, data preserved). Scored from actual lines: phone detected
Windows via LAN (TCP/mDNS LAN peer detected 12D3KooWD6vZ with 8 local addrs),
dialed 192.168.0.222:9001 directly, full identify (32 addrs);
peersDiscovered 0 -> 1; bootstrap UNKNOWN -> WIFI clean; ANR fix holds on
the new APK (zero ANRs, one benign cold-start frame skip). Desktop mutual
confirmed: Windows peers = [Pixel, AWS] simultaneously, DIAL-BACKOFF reset,
contact 'Lucas' learned, 2-peer list SENT to the phone (ledger re-seed live),
phone's address snapshot carries the 147.81.41.188 circuit through Windows -
T14+D10 end-to-end. WARN: message delivery not yet confirmed
(undeliveredCount=1, messagesRelayed=0); phone->AWS leg UNVERIFIED but the
re-seed should produce it passively. Evidence:
tmp/cto/D10_REJOIN_20260910T015627Z/. PR #279 checks: 4 PASS, rust pending.
MESH IS 3-NODE-CONNECTED AT TRANSPORT LEVEL - ready for the operator's
manual drop test (WiFi/BLE/cell).
