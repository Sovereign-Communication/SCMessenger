# D4 Exit Criterion: AWS Always-On Node Rebuild Runbook

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** Ready for execution upon SHA freeze and CI image publication
**Target:** D4 Milestone (Pixel 6a <-> AWS Node Verified E2E Delivery Receipt)
**Author:** Orchestrator Lane C
**Date:** 2026-08-15

---

## 1. Architecture Doctrine & Objectives

- **Nodes, Not Relays:** In SCMessenger, all participants are full NODES. Store-and-forward custody is a capability executed by every node, not a separate standalone relay server. The AWS instance is an always-on **Cloud Node**.
- **D4 Milestone Goal:** D4 requires an end-to-end verified message and delivery receipt between two independent endpoints (Pixel 6a <-> AWS Cloud Node), scored on receiver-side decryption, durable storage, and receipt round-trip.
- **Runbook Objective:** Provide exact, copy-pasteable, verified steps to transition the AWS Cloud Node from its current state (running closed PR branch `9f54b107`) to the official tagged `v0.4.0-alpha.1` SHA while guaranteeing **zero identity loss** (preserving ledger state, Peer ID, and cryptographic keys).

---

## 2. Hard Constraints & Safety Rules

- **[WARNING] NEVER COMPILE ON THE T3.MICRO INSTANCE:** A previous attempt to run `cargo build` on this instance ran for 16 hours before failing with Out Of Memory (OOM). Any rebuild MUST pull a prebuilt Docker image from Docker Hub (`testbotz/scmessenger:sha-<short_sha>` or `testbotz/scmessenger:latest`).
- **[WARNING] PRESERVE HOST DATA MOUNT:** Node identity and ledger data reside on host directory `/opt/scm-relay-data`. Never delete, wipe, or format `/opt/scm-relay-data`.
- **SSH Authentication:** The instance runs **Amazon Linux 2023**. Connect as user `ec2-user` with key `~/.ssh/scm-node-key.pem`. (Connecting as `ubuntu` will fail with Permission Denied).
- **Docker Privilege:** Running docker commands requires `sudo`.

---

## 3. Verified Live Baseline (State as of 2026-08-15)

- **AWS Instance ID:** `i-006b14491d421bd0d` (us-east-1, `t3.micro`, tag `scm-always-on-node`, state: `running`)
- **Current Dynamic Public IP:** `54.226.67.101` (Note: Dynamic IP -- verify before connecting)
- **Running Image Digest:** `testbotz/scmessenger@sha256:a58645e886409e057edb7557141e02b64cf0e9fd9f28ecab773b099a6e760583`
- **Running Git SHA:** `9f54b1078ad512c895b68029c9e79a1870d7f286` (`gpt/pr139-receipt-filter-20260811`)
- **Node Identity ID:** `0b33200936f41deb55e674e1d798b5c2aac7494a8a95ea34cd59c3b013c226ad`
- **Node libp2p Peer ID:** `12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`
- **Node Nickname:** `scm-always-on-node`
- **Host Data Directory:** `/opt/scm-relay-data`

---

## 4. Pre-Rebuild CI Verification (Blocking Dependency)

Before running the deployment steps on AWS, verify that GitHub Actions CI (`docker-publish.yml`) has built and pushed the target Docker image to Docker Hub.

Run locally on the host machine:
```bash
python -c "import urllib.request, json; data = json.loads(urllib.request.urlopen('https://hub.docker.com/v2/repositories/testbotz/scmessenger/tags?page_size=10').read().decode()); print([(t['name'], t['last_updated'], t.get('digest')) for t in data['results']])"
```
**Required Result:** Ensure tag `sha-<short_sha>` (or `latest` for main) exists with the expected commit hash before touching the node.

---

## 5. Execution Runbook (Step-by-Step)

### Step 1: Resolve Dynamic Public IP

Run from the orchestrator workstation:
```bash
python -c "import boto3; ec2 = boto3.client('ec2', region_name='us-east-1'); res = ec2.describe_instances(Filters=[{'Name':'tag:Name','Values':['scm-always-on-node']},{'Name':'instance-state-name','Values':['running']}]); ip = res['Reservations'][0]['Instances'][0]['PublicIpAddress']; print('NODE_IP=' + ip)"
```
Export the resolved IP:
```bash
export NODE_IP="<resolved-ip>"
```

### Step 2: Verify SSH Access and Baseline Health

Check SSH connectivity:
```bash
ssh -o ConnectTimeout=5 -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP hostname
```
*Expected Output:* `ip-172-31-19-216.ec2.internal`

Check current baseline HTTP endpoints:
```bash
curl -s http://$NODE_IP:9876/health
curl -s http://$NODE_IP:9876/version
curl -s http://$NODE_IP:9876/api/identity
```

### Step 3: Create Persistent Data Backup on Host

SSH into the node and create a timestamped backup of the persistent database:
```bash
ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo tar -czf /opt/scm-node-data-backup-\$(date +%Y%m%dT%H%M%SZ).tar.gz -C /opt scm-relay-data && ls -lh /opt/scm-node-data-backup-*.tar.gz"
```

### Step 4: Pull Prebuilt Docker Image

Pull the new target image on the remote node:
```bash
# Example for specific short SHA (replace <TARGET_SHA_TAG> e.g. sha-965ee91 or latest):
export TARGET_TAG="latest"

ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker pull testbotz/scmessenger:${TARGET_TAG}"
```

### Step 5: Gracefully Stop and Rename Existing Container

Do not immediately delete the previous container; rename it for instant rollback capability:
```bash
ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker stop scm-node && sudo docker rename scm-node scm-node-backup-\$(date +%Y%m%d)"
```

### Step 6: Start New Container with Persistent Storage

Launch the updated container using the exact verified configuration:
```bash
ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker run -d \
  --name scm-node \
  --restart unless-stopped \
  --network host \
  -v /opt/scm-relay-data:/root/.local/share/scmessenger \
  -e RUST_LOG=info,scmessenger=debug \
  -e LISTEN_PORT=9000 \
  -e SCM_CONFIG_DIR=/root/.config/scmessenger \
  -e SCM_DATA_DIR=/root/.local/share/scmessenger \
  testbotz/scmessenger:${TARGET_TAG} \
  scm --http-bind 0.0.0.0:9876 start --port 9000"
```

---

## 6. Post-Rebuild Verification Gates

Execute these verification checks immediately following container start. All 4 checks must pass.

### Gate 1: HTTP Health Check
```bash
curl -s http://$NODE_IP:9876/health
```
*Expected Output:*
```json
{"status":"healthy"}
```

### Gate 2: Version & Provenance Verification
```bash
curl -s http://$NODE_IP:9876/version
```
*Expected Output:*
- `version`: `"0.4.0"`
- `git_hash`: Matches the target commit SHA.
- `core_provenance`: Contains `0.4.0 (<target_sha>:...)`

### Gate 3: Cryptographic Identity Preservation
```bash
curl -s http://$NODE_IP:9876/api/identity
```
*Expected Output:*
```json
{
  "device_id": "e7a76bf1-2742-43d1-9a97-bf12f90a4b61",
  "identity_id": "0b33200936f41deb55e674e1d798b5c2aac7494a8a95ea34cd59c3b013c226ad",
  "initialized": true,
  "libp2p_peer_id": "12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy",
  "nickname": "scm-always-on-node",
  "public_key_hex": "8db1612aa6330be410f7f181a43ee4743b23045bb1d3c69594d864c37b28f92c",
  "seniority_timestamp": 1786476044
}
```
*Criteria:* `identity_id` and `libp2p_peer_id` must match the pre-rebuild baseline exactly.

### Gate 4: Container Startup Logs Check
```bash
ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker logs --tail 30 scm-node"
```
*Criteria:*
- Logs show `[OK] Loaded existing identity`.
- Logs show `[OK] Peer ID: 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`.
- Logs show `HTTP health server listening on 0.0.0.0:9876`.
- No sled database locking errors or panics.

---

## 7. Emergency Rollback Procedure

If any verification gate fails:

1. Stop and remove the failed container:
   ```bash
   ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker stop scm-node && sudo docker rm scm-node"
   ```
2. Restore the previous container:
   ```bash
   ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo docker rename scm-node-backup-\$(date +%Y%m%d) scm-node && sudo docker start scm-node"
   ```
3. If data corruption occurred, restore `/opt/scm-relay-data` from the backup tarball created in Step 3:
   ```bash
   ssh -i "$HOME/.ssh/scm-node-key.pem" ec2-user@$NODE_IP "sudo rm -rf /opt/scm-relay-data && sudo tar -xzf \$(ls -t /opt/scm-node-data-backup-*.tar.gz | head -n 1) -C / && sudo docker restart scm-node"
   ```
4. Verify health and identity using Gate 1-3 commands.
