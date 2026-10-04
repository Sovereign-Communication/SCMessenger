# AWS Always-On Node -- CURRENT relay address

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

POLICY (operator directive 2026-08-04): IPs in this repo are ephemeral.
This file is the ONE place the orchestrator updates immediately after every
AWS node rebuild. Read it fresh at use time; never copy an IP from any
other doc, ticket, or config.

## Current (updated 2026-09-14; instance rebuilt ~2026-09-13)

- **2026-09-14: INSTANCE REBUILT ~2026-09-13 with a NEW instance ID and IP.
  The old address below was dead 2026-09-13..14 and nobody recorded the
  replacement -- this doc's update policy was violated. Rediscovered via
  boto3 tag sweep (tag Name=scm-always-on-node, us-east-1), verified live.
- Instance: i-0b41aab756eabd514 (replaced i-006b14491d421bd0d)
- Public IP: 18.234.62.247 (dynamic -- re-verify at use time)
- Bootstrap multiaddr: /ip4/18.234.62.247/tcp/9001
- Health check: http://18.234.62.247:9876/health -- verified 200 healthy
  2026-09-14
- SSH: `ec2-user@18.234.62.247` with key `~/.ssh/scm-node-key.pem`
  (verified 2026-09-14); container `scm-node`, image
  `testbotz/scmessenger:sha-ccce98c`, identity host path
  `/opt/scm-relay-data`.
- Resolution one-liner (no AWS CLI needed; creds are the
  scmessenger-relay-orchestrator IAM user):
  `python -c "import boto3; ec2=boto3.resource('ec2',region_name='us-east-1'); [print(i.id, i.public_ip_address) for i in ec2.instances.filter(Filters=[{'Name':'tag:Name','Values':['scm-always-on-node']}])]"`
- Cost hygiene verified 2026-09-14: exactly 1 billable instance across all
  34 regions, 0 Elastic IPs, 0 orphaned EBS volumes.

## Historical (pre-rebuild lessons, kept for context)

- Identity persistence FIXED + VERIFIED (2026-08-29): root cause was
  image setting `SCM_DATA_DIR` (entrypoint-only) while the app reads
  `SCMESSENGER_DATA_DIR`; identity was written to the container's ephemeral
  layer and rotated on every redeploy (`640c258b` -> `78869300` ->
  `417be00d`). PR #240 (merged, main `b2544d26`) points both env vars at
  `/data` (bound to `/opt/scm-relay-data`). **Verified live:** after two
  consecutive `docker rm -f` + `docker run` redeploys the identity stayed
  `0b332009...` / `12D3KooWKMU...` unchanged -> identity now persists across
  restarts.




## Previous (STALE -- do not use)

- 54.226.67.101 / i-006b14491d421bd0d (instance terminated ~2026-09-13;
  replacement rebuilt with a new ID -- not recorded until 2026-09-14)
- Public IP: 54.226.67.101
- Bootstrap multiaddr: /ip4/54.226.67.101/tcp/9001
- Health check: http://54.226.67.101:9876/health
- Instance: i-006b14491d421bd0d, tag Name=scm-always-on-node
  (account 101533648751, us-east-1, t3.micro, AMI ami-0bdc7d025135d7b49)
- Image: docker.io/testbotz/scmessenger:latest @
  sha256:a58645e886409e057edb7557141e02b64cf0e9fd9f28ecab773b099a6e760583

- Same IP/instance as Current but PRE-REBUILD image (superseded
  2026-08-25): testbotz/scmessenger:latest at commit
  `9f54b1078ad512c895b68029c9e79a1870d7f286`, label
  `gpt-pr139-receipt-filter-20260811` (PR-139 image). Earlier
  revisions of this file had also claimed commit `6b2573fa`
  (PR 136+137+138); corrected 2026-08-24 to the PR-139 label image.
  Superseded by the 2026-08-25 rebuild at main SHA.
- 34.203.213.35 (2026-08-04 rebuild, pre-PR-137 image)
- 54.242.56.150 (prior broken instance)
- 100.56.248.69 (original docs IP; obsolete)
