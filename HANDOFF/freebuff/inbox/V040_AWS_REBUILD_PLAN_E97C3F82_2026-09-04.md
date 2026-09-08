Task: V040_AWS_REDEPLOY_BLOCKED_E97C3F82_2026-09-04.md (superseding: in-place swap blocked; root path now = rebuild)
Type: QUESTION

# AWS node i-0b735c4f26aea42ed -- rebuild plan at sha-e97c3f8, CEO ruling requested

Date: 2026-09-04 (~12:00Z)
From: CTO lane (freebuff)
Status: AWAITING RULING -- nothing on the node has been touched

## Finding: the in-place root path does not exist; the IAM policy now permits only rebuild

Operator direction (2026-09-04): "you setup docker -- rebuild if needed; AWS EC2 is
completely at your own creation." Probed every root path with the local boto3 identity
`arn:aws:iam::101533648751:user/scmessenger-relay-orchestrator` (live dry-runs, 11:53Z):

| Action | Result | Action | Result |
|---|---|---|---|
| run_instances (full proven params: key/sg/tags/userdata/16GB gp3) | **ALLOWED** | create_snapshot | DENIED |
| stop_instances | **ALLOWED** | create_image | DENIED |
| reboot_instances | **ALLOWED** | modify-instance-attribute (userData) | DENIED |
| terminate_instances | **ALLOWED** | attach_volume / detach_volume | DENIED |
| describe/read (instance, volume, addresses, logs via ssh ec2-user) | **ALLOWED** | EIP associate / disassociate | DENIED |

The Aug-30 "RunInstances DENIED" record is stale -- the policy was widened to full
instance lifecycle while keeping data-plane ops (snapshot/image/userdata/volume move)
closed. On-instance root is unreachable (ec2-user not in docker group 993-empty, no
NOPASSWD sudo, no IAM instance profile, root ssh refused) and the tool layer blocks
embedded privilege wrappers. **Teardown + rebuild is the only sanctioned path and is
fully IAM-permitted.**

## Consequences of rebuild (accepted, evidence-backed)

1. **New public IP.** 54.235.20.24 is an auto-assigned address (describe_addresses:
   no EIPs in account; NI owner=amazon; associate DENIED). Rebuild = real IP churn --
   this exercises the previously-deferred AWS IP-churn gate for real.
2. **New peer identity.** The 12D3KooW9uRMQTswPUjUn2YfTLx5sjH26v2AtjRfgiE73WLprBfD key
   lives under root-only /root/.config (ls DENIED as ec2-user). Fresh boot regenerates.
   Runbook V040_3NODE_VALIDATION_RUNBOOK gate B already contemplates this ("if it
   changed, the mesh re-learns it; note the new ID").
3. **Fresh relay ledger.** Old volume cannot move (attach DENIED). All readable data
   archived first (below). Relay ledger is re-seeded by Windows + Pixel post-boot.

## Pre-teardown evidence archive (read-only, COMPLETE)

`tmp/run-evidence/aws-rebuild-e97c3f82-20260904/pre-teardown/`
- `scm-node-data-archive-20260904-1200Z.tar.gz` (1.4 MB: logs/ Aug-31..Sep-4 hourly,
  storage/ incl. ledger.json 88 KB + sled db, outbox/, peers.json.migrated-1788287892,
  peers.json.poisoned-backup-1788196)
- `peers.json.migrated-1788287892`, `peers.json.poisoned-backup-1788196` (standalone copies)

## Proposed plan (each step reversible until the last)

1. **Launch replacement first** (old node keeps running during build/verify -- no outage
   until step 3). New instance per launch.py shape with HARDENED userdata:
   install docker + enable; `usermod -aG docker ec2-user` (permanently closes the
   recurring non-root blocker: all future redeploys become ssh+docker as ec2-user);
   `docker pull testbotz/scmessenger:sha-e97c3f8` (run 33860486330, SUCCESS);
   run `scm-node` container: --network host --restart unless-stopped
   -v /opt/scm-relay-data:/data, env RUST_LOG=info,scmessenger=debug, SCM_DATA_DIR/
   SCMESSENGER_DATA_DIR=/data, SCM_CONFIG_DIR=/root/.config/scmessenger,
   LISTEN_PORT=9000, cmd `scm --http-bind 0.0.0.0:9876 start`.
2. **Verify new node** via Control API (no root needed): http://<new-ip>:9876/api/identity
   (expect `CLI Version: 0.4.0 (e97c3f82...)`), /api/peers, /api/external-address.
   Capture new instance id + public IP + container id.
3. **Cut over**: stop old instance i-0b735c4f26aea42ed (volume retained as recovery
   artifact; do NOT terminate yet).
4. **Fleet re-mesh**: operator restarts the Windows CLI (currently hard-seeding
   /ip4/54.235.20.24/tcp/9001) with the new AWS seed; Pixel stays attached to Windows.
   I provide the exact restart block with the new IP.
5. **Terminate old instance only after post-validation is green** (gate 9 evidence:
   3-node mesh on same-SHA fleet, dead-marks/:50-closes gone, forwarding leg).

## Single decision needed from the CEO seat

Approve the rebuild sequence with **stop-then-terminate-later** (recommended, recovery
artifact until validation) OR **immediate full teardown** (terminate old up front, no
rollback)? Also confirm: (a) accept new IP + new identity + fresh relay ledger as
documented above; (b) the operator will run the Windows re-seed restart when I hand
over the block; (c) post-validation green is the trigger for old-instance termination.
