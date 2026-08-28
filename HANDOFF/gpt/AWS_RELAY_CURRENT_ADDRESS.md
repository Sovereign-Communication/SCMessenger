# AWS Always-On Node -- CURRENT relay address

POLICY (operator directive 2026-08-04): IPs in this repo are ephemeral.
This file is the ONE place the orchestrator updates immediately after every
AWS node rebuild. Read it fresh at use time; never copy an IP from any
other doc, ticket, or config.

## Current (updated 2026-08-25; container rebuild, same IP)

- Public IP: 54.226.67.101
- Bootstrap multiaddr: /ip4/54.226.67.101/tcp/9001
- Health check: http://54.226.67.101:9876/health
- Instance: i-006b14491d421bd0d, tag Name=scm-always-on-node
  (account 101533648751, us-east-1, t3.micro, AMI ami-0bdc7d025135d7b49)
- Image: docker.io/testbotz/scmessenger:latest @
  sha256:a58645e886409e057edb7557141e02b64cf0e9fd9f28ecab773b099a6e760583,
  rebuilt 2026-08-25. git_hash now
  `0064d49a0a0a8464dd22dcac2da70e5e455c7743` (= current main) --
  version parity with the Windows + Android rig nodes achieved.
- SSH: `ec2-user@54.226.67.101` with key `~/.ssh/scm-node-key.pem`.
  Identity persists at host path `/opt/scm-relay-data`; container name
  `scm-node`.

## Previous (STALE -- do not use)

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
