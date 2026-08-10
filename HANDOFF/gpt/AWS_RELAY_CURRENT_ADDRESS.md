# AWS Always-On Node -- CURRENT relay address

POLICY (operator directive 2026-08-04): IPs in this repo are ephemeral.
This file is the ONE place the orchestrator updates immediately after every
AWS node rebuild. Read it fresh at use time; never copy an IP from any
other doc, ticket, or config.

## Current (updated 2026-08-05, post PR-138 rollout rebuild)

- Public IP: 54.226.67.101 (verified healthy: 200 {"status":"healthy"})
- Bootstrap multiaddr: /ip4/54.226.67.101/tcp/9001
- Verified network PeerId: 12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
- Full iOS bootstrap multiaddr: /ip4/54.226.67.101/tcp/9001/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
- Health check: http://54.226.67.101:9876/health
- Instance: i-006b14491d421bd0d, tag Name=scm-always-on-node
  (account 101533648751, us-east-1, t3.micro, AMI ami-0bdc7d025135d7b49)
- Image: testbotz/scmessenger:latest at commit 6b2573fa (PR 136+137+138)

## iOS join seed

Use this payload as the QR/invite `bootstrap_peers` value. The iOS join flow
persists it into the local ledger, uses it for cold-start bootstrap, and
promotes it after a live Identify event confirms the cloud node.

```json
{"bootstrap_peers":["/ip4/54.226.67.101/tcp/9001/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2"],"topics":[]}
```

## Previous (STALE -- do not use)

- 34.203.213.35 (2026-08-04 rebuild, pre-PR-137 image)
- 54.242.56.150 (prior broken instance)
- 100.56.248.69 (original docs IP; obsolete)
