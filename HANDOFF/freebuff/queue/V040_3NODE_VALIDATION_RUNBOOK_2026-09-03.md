# V040 Three-node validation runbook -- candidate 177bd840

Task: V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md
Type: RUNBOOK (execution record; gates marked PASS/FAIL/UNVERIFIED as evidence lands)
Date: 2026-09-03
Supersedes: V040_3NODE_ROLLOUT_PLAN_2026-09-03.md SHA pin (3891d11c -> 177bd840).

## Candidate
- SHA: **177bd840** (= origin/cto/v040-candidate-2026-09-02 = refs/pull/272/head)
- Delta over approved 3891d11c: test-only (core/tests/test_address_observation.rs
  +13/-8); Rule-8 FINAL APPROVE on file with delta addendum; CI 33/33 green.

## Node identities (all verified on 177bd840 content)
| Node | Identity | Address | Provenance |
|---|---|---|---|
| Windows CLI | 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw | 147.81.41.188 | exe 35c49318 (tree == 177bd840) |
| AWS cloud | 12D3KooW9uRMQTswPUjUn2YfTLx5sjH26v2AtjRfgiE73WLprBfD | 54.235.20.24:9000/9001 | container sha-177bd84, digest sha256:4518385a |
| Pixel 6a | 12D3KooWKdek3ZZH9pjQgQriKnGcmN98PYmG2rChBwd1ABMFHQrx | LAN 192.168.0.129 | APK CI run 33719858693, sha256 862c12da |

## Evidence gates -- FINAL MARKS (2026-09-03 ~08:00Z)

### Gate 1 -- Seed dialing: **PASS**
- AWS boot 07:25:50 "[SEED-DIAL] sweep 1: 56 candidate(s), peers=0" + "Dialed seed
  peer for NAT hole punch: /ip4/10.9.86.76/tcp/80/p2p/12D3KooWKdek3".
  Evidence: tmp/run-evidence/aws-177bd840-startup.log; node log
  /opt/scm-relay-data/logs/scm.log.2026-09-03-07.
- Windows boot 06:43:33 "[SEED-DIAL] sweep 1: 1017 candidate(s), peers=0".
  Evidence: .codebuff_deploy/windows/wincli-177bd840-20260902T204329.log;
  .codebuff_deploy/EVIDENCE-3NODE-177bd840-2026-09-03.md.

### Gate 2 -- External-address filtering: **PASS** (log evidence, all 3 nodes)
- Consensus/advertised external addrs are configured listen ports: Windows
  192.168.0.121:80; Pixel 192.168.0.129:9001 ("listen-port consistent" per
  tmp/run-evidence/pixel-177bd840-restart-evidence.md); AWS LISTEN_PORT 9000.
- Reflected ephemeral NAT source ports (147.81.41.188:1546 etc.) did NOT enter
  consensus; node correctly declared "behind NAT -- relay required for inbound".
  T14/#270 allowlist gate held.
- Caveat: formal per-node diagnostics external_addrs capture not run as a
  standalone check; an explicit diagnostics sweep remains available pre-tag.

### Gate 3 -- Ledger/peer propagation: **PASS**
- Windows<->AWS: relay circuit ACCEPTED + ledger exchange 07:26:31-32Z post-redeploy.
- Android: peersDiscovered 1 then 2 (Windows LAN + AWS relay 014b8105).
- AWS boot: 45 ledger migration events; device_id c11dd372 retained.
- Evidence: tmp/run-evidence/3NODE-TEST-2026-09-03-WIFI-BLE-CELL.md;
  tmp/run-evidence/aws-redeploy-177bd840-evidence.md.

### Gate 4 -- Inbound reachability + delivery: **PASS**
- WiFi deliveries 53ms/39ms both directions; BLE GATT received_and_decrypted x2.
- Relay circuit ACCEPTED via AWS; 25 custody items delivered to Windows at
  07:36:59-07:37:06Z (accept_immediate_pull).
- tcp_mdns burst delivered 15-msg backlog at 07:44:58Z after WiFi reconnect.
- Evidence: 3NODE-TEST file; live-test-0738/FINDINGS-2026-09-03-0738Z.md;
  AWS-WIN-LINK-ANALYSIS-2026-09-03-0800Z.md.

### Gate 5 -- Coordinated restart re-mesh: **PASS**
- AWS redeploy 07:25:50Z re-meshed with Windows 07:26:31Z unaided.
- Pixel restart 07:32Z re-formed mesh via p2p-circuit; BLE delivery OK.
- Evidence: aws-redeploy-177bd840-evidence.md; pixel-177bd840-restart-evidence.md.

### Gate 6 -- AWS IP churn: **UNVERIFIED** (genuinely unexercised)
- Dynamic public IP (no EIP: describe_addresses -> 0; Association.AllocationId
  absent -- read-only check 08:00Z). No churn test run: EIP release/reassign or
  instance stop/start on the live always-on node is invasive; IAM incomplete
  (run_instances dry-run DENIED per tmp/run-evidence/06-iam-evidence.txt).
- Available pre-tag action WITH CEO approval: stop/start i-0b735c4f26aea42ed
  (dynamic IP changes), confirm re-mesh + ledger continuity as Gate 5; or full
  churn test on a throwaway instance once run_instances IAM granted.

## Artifacts (2026-09-03)
1. Docker image: CI run 33724848436 SUCCESS; tags :sha-177bd84 +
   :cto-v040-candidate-2026-09-02; digest sha256:4518385a...
2. Android APK: CI run 33719858693 SUCCESS; sha256 862c12da (installed).
3. Windows exe: CI-built merge ref 35c49318 (tree == 177bd840), sha256 bd1db414...

## Redeploy record (07:25Z) -- DONE
- Image sha-177bd84 (id 3020c5eae75d), container scm-node running (docker ps
  08:03Z). Provenance log line: CLI Version 0.4.0 (177bd8406e...). Peer identity
  unchanged. Data volume preserved (bind /opt/scm-relay-data -> /data; 45 ledger
  migrations; device_id c11dd372). Full-parity env: SCM_DATA_DIR=/data,
  SCMESSENGER_DATA_DIR=/data, SCM_CONFIG_DIR=/root/.config/scmessenger,
  LISTEN_PORT=9000 -- recreation used inspect-captured config.

PRE-REDEPLOY WIFI/BLE/CELL EVIDENCE (pulled 07:00-07:15Z, old image a79047b5) is
NOT attributable to 177bd840 -- labeled pre-redeploy / UNVERIFIED for certification.

## Evidence output locations (tmp/run-evidence/)
- 3NODE-TEST-2026-09-03-WIFI-BLE-CELL.md (test-window refs)
- live-test-0738/FINDINGS-2026-09-03-0738Z.md (07:38-07:46Z live window)
- AWS-WIN-LINK-ANALYSIS-2026-09-03-0800Z.md (dead-mark/custody root cause)
- aws-redeploy-177bd840-evidence.md, aws-177bd840-startup.log
- pixel-177bd840-restart-evidence.md
- 06-iam-evidence.txt (Gate 6 reason)
- AWS node log: /opt/scm-relay-data/logs/scm.log.2026-09-03-{07,08}
- Windows log: .codebuff_deploy/windows/wincli-177bd840-20260902T204329.log

## Do NOT (governance)
- No tag, no merge, no version bump, no posting of the verdict chain.
- Never terminate i-0b735c4f26aea42ed; never run launch.py; no IP churn test
  without CEO approval. No candidate or .codebuff_deploy/ changes.
