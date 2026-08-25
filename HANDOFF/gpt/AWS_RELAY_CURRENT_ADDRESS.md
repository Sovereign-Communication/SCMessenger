# AWS Always-On Node -- CURRENT relay address

POLICY (operator directive 2026-08-04): IPs in this repo are ephemeral.
This file is the ONE place the orchestrator updates immediately after every
AWS node rebuild. Read it fresh at use time; never copy an IP from any
other doc, ticket, or config.

## Current (updated 2026-08-24; image identity corrected 2026-08-24)

- Public IP: 54.226.67.101 (verified healthy: 200 {"status":"healthy"})
- Bootstrap multiaddr: /ip4/54.226.67.101/tcp/9001
- Health check: http://54.226.67.101:9876/health
- Instance: i-006b14491d421bd0d, tag Name=scm-always-on-node
  (account 101533648751, us-east-1, t3.micro, AMI ami-0bdc7d025135d7b49)
- Image: testbotz/scmessenger:latest at commit
  `9f54b1078ad512c895b68029c9e79a1870d7f286`, label
  `gpt-pr139-receipt-filter-20260811`. CORRECTION 2026-08-24: earlier
  revisions of this file claimed commit `6b2573fa` (PR 136+137+138);
  the running container is actually the PR-139 label image, verified to
  exist and to be an ancestor of main -- it ALREADY INCLUDES PR #139.
  It is still OLDER than main and must be rebuilt at the tag SHA for
  the four-node gate.
- SSH: `ec2-user@54.226.67.101` with key `~/.ssh/scm-node-key.pem`
  (exists since 2026-08-01; earlier "no SSH key" notes are stale).
  Identity persists at host path `/opt/scm-relay-data`; container name
  `scm-node`; ~16 GB free at last check.

## Previous (STALE -- do not use)

- 34.203.213.35 (2026-08-04 rebuild, pre-PR-137 image)
- 54.242.56.150 (prior broken instance)
- 100.56.248.69 (original docs IP; obsolete)
