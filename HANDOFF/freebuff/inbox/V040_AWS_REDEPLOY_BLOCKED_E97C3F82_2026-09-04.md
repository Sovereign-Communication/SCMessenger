Task: V040_CTO_FINAL_HEAD_e97c3f82_AWS_LEG_2026-09-04.md (AWS leg of the 3-node final-tree rollout)
Type: BLOCKED

# BLOCKED -- AWS container swap at e97c3f82 needs node root (operator action required)

## Blocker (verified, not inferred)
The scm-node container on i-0b735c4f26aea42ed (54.235.20.24) cannot be
replaced from the Freebuff harness because docker on the node requires root
and every root path is closed to the harness:

- ec2-user is NOT in group docker: `getent group docker` -> `docker:x:993:`
  (no members); socket is `srw-rw---- root docker /var/run/docker.sock`
- No NOPASSWD sudo: `/etc/sudoers.d/90-cloud-init-users` absent (probe via
  ssh as ec2-user)
- No IAM instance profile on the instance (`iam profile: NONE`) -> SSM Run
  Command unavailable; local creds also lack ssm:* perms
- Root ssh refused by sshd: "Please login as the user ec2-user"
- Harness blocks the embedded sudo literal at the tool level in both
  run_terminal_command and request_elevation
- IAM RunInstances DENIED (rollout-plan B1) -> no instance recycle as an
  alternative; user-data does not re-run on stop/start

## Image is ready (the only thing NOT blocked)
docker-publish run 33860486330 at e97c3f82 = SUCCESS (workflow_dispatch at
ref cto/v040-candidate-2026-09-02). Tags published:
`testbotz/scmessenger:sha-e97c3f8` + `:cto-v040-candidate-2026-09-02`.
SHA: e97c3f8247b29dd344467e05137b24f0f110a10a (final tree, #272 head).

## Operator commands + staged files
Everything is staged under tmp/run-evidence/aws-redeploy-e97c3f82-20260904/:

1. tmp/run-evidence/aws-redeploy-e97c3f82-20260904/OPERATOR-DEPLOY-BLOCK.md
   -- probe + full deploy block (snapshots old container config to the
   preserved volume first: /opt/scm-relay-data/scm-node.pre-e97c3f82.inspect.json,
   then pull sha-e97c3f8, stop/rm, run with same volume/env/network/restart).
   Probe (fail-fast, no hang):
   ssh -i ~/.ssh/scm-node-key.pem ec2-user@54.235.20.24 'echo probe; sudo -n docker ps --format "{{.ID}} {{.Image}}" 2>&1 | head -2'
2. tmp/run-evidence/aws-redeploy-e97c3f82-20260904/capture-post.sh -- one-shot
   after-state capture from all three nodes (run from repo root after the swap).

## Node identities / addresses
- Windows CLI: 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw, PID
  20688, log .codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log
  (e97c3f82 exe, live)
- Pixel 6a: 12D3KooWBPdNE1NB4uwMq41jL2YQxBZcMQBSwfnnEFBxpRVLkEGW (Lucas),
  PID 2452, e97c3f82 APK; adb serial 26261JEGR01896 (wireless); on-device
  mesh logs files/mesh_diagnostics.log(.1-.3) + files/logs/scmessenger-mesh.log
  via `adb shell run-as com.scmessenger.android cat ...`
- AWS: 12D3KooW9uRMQTswPUjUn2YfTLx5sjH26v2AtjRfgiE73WLprBfD at 54.235.20.24
  (container scm-node, OLD 177bd840-era image, started 2026-09-03T07:25:50Z);
  logs on preserved volume /opt/scm-relay-data/logs/scm.log.*

## Pre-state evidence + BEFORE-ANALYSIS.md headline
Evidence: tmp/run-evidence/aws-redeploy-e97c3f82-20260904/pre/ (wincli-pre*.log,
scm.log.2026-09-04-08/-09, pixel mesh logs + logcat) and
BEFORE-ANALYSIS.md there. Headline findings (user's wifi/BLE/cell test):
- Forwarding PROVEN 09:50:38Z: Pixel (cellular-only) reserved a relay circuit
  through Windows via 10.0.41.4:9002/ws; messages crossed both ways
  [OK] delivered 11-48ms.
- Pixel reconnect after the 09:50:38.571Z radio flip did NOT happen:
  ActivityManager froze PID 2452 at 09:51:14Z (app backgrounded) -- Android
  app-freezer blocker, foreground the app to re-mesh (not an e97c3f82 defect).
- OLD AWS image dead-marked Windows ~20x in the 09:00Z hour ("Peer marked as
  dead after 3 failed attempts" for 12D3KooWD6vZQ) while the link was up --
  the exact #273-fixed behavior; post-e97c3f82 expected delta: zero dead-marks,
  no :50 recycle closes on the AWS<->Windows link.

## Gate marks (this phase)
- SEED-DIAL / AWS boot at final tree: UNVERIFIED (blocked on the swap)
- LEDGER/PEER propagation on e97c3f82 fleet: UNVERIFIED for the AWS leg
  (Windows+Pixel legs live and meshed pre-state)
- EXTERNAL-ADDRESS filtering: UNVERIFIED on AWS old image (pre-redeploy)
- INBOUND REACHABILITY / relay forwarding: PASS in pre-state via Windows
  relay (09:50:38Z evidence); NOT attributable to e97c3f82 on AWS until swap
- COORDINATED RESTART: UNVERIFIED (this is the restart)
- AWS IP CHURN: UNVERIFIED (RunInstances/AllocateAddress IAM-denied; only
  in-place reuse possible -- documented rollout-plan B1)
- Container swap action itself: BLOCKED (this note)

## PR state (verified via gh, 2026-09-04 ~10:30Z)
- #272 OPEN head=e97c3f82 cto/v040-candidate-2026-09-02 -> main (NOT merged)
- #273 MERGED (onto cto/v040-candidate-2026-09-02)
- #274 MERGED (onto cto/v040-candidate-2026-09-02, as 48672b18; contained in
  the e97c3f82 tree)

## Single next CEO decision
Run the probe, execute the deploy block from OPERATOR-DEPLOY-BLOCK.md (paste
back the docker ps line + first ~15 lines of `docker logs scm-node` boot log
so provenance is verified: CLI 0.4.0 (e97c3f82...), peer id 12D3KooW9uRMQ...
unchanged, ledger migrations over the preserved volume), then I run
capture-post.sh and deliver the before/after verification on the same-SHA
fleet. If the probe says "a password is required", advise how root is
administered on the node (console / password / different key).
