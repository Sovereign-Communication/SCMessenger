# V040 CTO next-run package — three-node test after parity fixes

Written by the CEO seat, 2026-09-09T01:40Z, at operator direction ("CTO is
stopped; prep the CTO for the next 3 node test after RCA + parity fixes").
This is the whole brief for the next `/cto` session. Read it with
`HANDOFF/V040_3NODE_RCA_2026-09-09.md` (issue IDs W*/A*/P*/X* below refer to
it) and `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md` — the
checkpoint schema, stage names, and evidence rules of that package carry
forward UNCHANGED. This file only adds the entry gates, the fix-execution
order, and the updated lane rules.

## Mission

Certify the same candidate tree on all three nodes (Windows CLI, AWS cloud
node, Pixel 6a) with the package's BLE end-to-end gate: Phase 3 isolation ->
Phase 4 single probe -> Phase 5 correlation. Everything before that in this
package exists only to make those phases meaningful.

## Entry gates — ALL must be green before PREFLIGHT

The CTO verifies each with fresh commands and records the evidence in the
PREFLIGHT checkpoint. Do not skip; a failed gate means STOP and report.

- **E1 (X1) Rule-8 review on file.** Independent adversarial APPROVE covering
  `0a33c009` + `74253491` (both touch `core/src/transport`). Merge gate; the
  CTO must not self-approve and must not merge without it.
- **E2 (X2/X3) Full core regression suite green.** CLOSED 2026-09-09T02:15Z:
  41 binaries, 1653 passed / 0 failed (`tmp/cto/
  E2_REGRESSION_RERUN_20260909T0210Z.log`), including the stale-allowlist-test
  fix `6c007b47`. Future runs still require a fresh full-suite pass on the
  exact run tree — targeted gates alone are not sufficient evidence.
- **E3 (A1) AWS on the run tree.** Redeploy the cloud node at the frozen
  run commit using `.codebuff_deploy/aws/` scripts; `/version` git_hash must
  equal the run commit exactly.
- **E4 (A2) AWS binary hash collected.** SSH `sha256sum` of the container
  binary (or image digest) recorded in PREFLIGHT — this was UNVERIFIED every
  previous run.
- **E5 (P1) BLE-01 APK installed on the Pixel.** Operator installs the staged
  APK `2f07ed91` (or its successor from the run tree); record `pm path` APK
  SHA256 == expected, and the app service identity afterwards.
- **E6 (W2) Bootstrap link no longer env-only.** CLOSED 2026-09-09T01:47Z:
  AWS multiaddr persisted in `%APPDATA%\scmessenger\config.json`
  `bootstrap_nodes` (backup `config.json.bak-e6-20260909T014640Z`); no-env
  restart PROVEN — PID 23508 dialed exactly the config node and connected to
  AWS (`tmp/cto/E6_RESTART_PROOF/`, identity stable, custody preserved,
  outbox flushed 5->0).
- **E7 (W4) Rollback staged outside target/.** CLOSED 2026-09-09T01:45Z for
  the live binary: `tmp/radio-829efe2c/rollback/scmessenger-cli-829efe2c.exe`
  + `SHA256SUMS.txt`. Any future cutover must repeat this for the
  then-current binary BEFORE stopping the running node.
- **E8 (W5) Advertisement-confirmation evidence defined.** The run's BLE
  readiness = Pixel-side observation of the Windows SCM beacon and/or a
  confirmed-advertising log line — start markers are explicitly insufficient.
- **E9 Disk headroom >= 20 GB.** Run `scripts/reclaim_safe.py` first; builds
  under `scripts/build_lock.py` as before.

## Fix-execution order (if E1-E9 are not all green yet)

1. X1 review dispatch (blocks merge, not the device work) — operator/lane.
2. X2 suite rerun — CTO, ~1 build cycle, E2 evidence.
3. E6 bootstrap persistence + E7 rollback staging — CTO, local config +
   file copy; no code change expected.
4. E3+E4 AWS redeploy + hash capture — CTO (AWS scripts) or operator.
5. E5 Pixel APK install — OPERATOR ONLY (Android lane, no shell-forced
   service actions).
6. W6 firewall probe: after E5, the CTO tests LAN inbound from the Pixel's
   logs only (read-only). If inbound 9001/9002 is blocked, STOP and request
   the operator's firewall ruling — the previous no-firewall-changes ruling
   does NOT auto-carry; each change needs its own.
7. A3 (duplicate tagged EC2 instance) — operator housekeeping, any time
   before E3.

## Run shape (after entry gates)

Reuse the 2026-09-08 package phases verbatim, with these deltas:

- PREFLIGHT additionally records: the one run commit + per-node artifact
  hashes (E3/E4/E5 close W3), the rollback SHA256 (E7), and the no-env
  restart proof (E6).
- Node READY stages use E8 evidence, not start markers.
- Phase 3 isolation and Phase 4 single-probe rules are unchanged: exactly one
  probe message, no retries, synchronized UTC markers on Windows + Android.
- Phase 5 correlation requires BLE-specific ingress + decrypt on Windows and
  NO TCP/mDNS/Wi-Fi/relay/AWS route — unchanged, plus: correlate against the
  W6 firewall outcome so a blocked LAN port is distinguishable from a BLE
  failure.
- CLEANUP and the final tracked handoff are unchanged.

## Lane rules for the next run

- Windows/AWS: CTO owns, including cutover WITH the recorded relaunch config
  (`tmp/cto/T14_GOLIVE/launch_node_env.ps1` pattern) once E6 lands.
- Pixel: operator owns all service/radio actions; CTO read-only log collection
  only (ANDROID.md scope) — no `am`/`input`/`svc`/`settings put`.
- No commit during device phases without an explicit operator order (the
  2026-09-08 "no commit yet" ruling applied to that session only; ask, do not
  assume either way).
- Preserve every prior checkpoint; never overwrite; state files updated
  immediately on change (section 0-rule).
- Evidence lives in `tmp/cto/<STAGE>_<UTCZ>/` with complete outputs; decision
  on X4 (archiving to `HANDOFF/`) belongs to the operator.

## Stop conditions

Any of: entry gate fails after one retry; identity change on any node;
artifact hash mismatch vs the run tree; probe correlation ambiguity; disk
below 10 GB; any checkpoint-schema failure the CEO audits. STOP means write
the checkpoint with explicit verdicts and hand back — exactly as the CTO did
correctly at the 120957Z preflight.

## Success definition

One commit, one artifact generation per node, BLE end-to-end probe correlated
without any fallback route, cleanup verified — then, and only then, the
v0.4.0 tag conversation (X6) reopens per the ship plan.
