# AWS Cloud Node (100.56.248.69) Pre-Launch Checklist

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** EXECUTE NOW (independent of identity canonicalization)
**Priority:** CRITICAL — must be ready to restart immediately when new image lands
**Date:** 2026-08-04

---

## Pre-Flight Checks (Do Now)

### 1. AWS Credentials + Instance Access
- [ ] AWS CLI installed on this machine? `aws --version`
- [ ] Credentials at `~/.config/scmorc/aws.env` present and readable?
- [ ] Can resolve instance: `bash infra/aws/farm-sim-manage.sh status`
- [ ] Instance state = running?
- [ ] Public IP = 100.56.248.69?

### 2. SSH Access to Instance
- [ ] SSH key file exists? (scmessenger-relay-key.pem or scmessenger-farm-sim-key.pem)
- [ ] Can SSH in: `ssh -i <key> ec2-user@100.56.248.69 "hostname"`
- [ ] Docker is running: `docker ps`
- [ ] Relay container is running: `docker ps | grep scmessenger`

### 3. Current Container State
- [ ] Check current image tag: `docker image ls | grep scmessenger`
- [ ] Check ports listening: `netstat -tlnp | grep 9001` or `docker port <container>`
- [ ] Check logs for errors: `docker logs <container> 2>&1 | tail -20 | grep -i "error\|panic"`
- [ ] Verify relay is accepting connections: `curl -s http://100.56.248.69:9876/health`

### 4. Prepare for Fresh Deploy
- [ ] Docker pull latest image ready? `docker pull testbotz/scmessenger:latest` (test command, don't pull yet)
- [ ] Stop/restart strategy documented? (docker stop → docker run with new image)
- [ ] Logs retained or cleared? Decision: [CLEAR on restart] / [KEEP for analysis]

---

## When Identity Canonicalization Lands

**Timeline: T+0 (identity merged to main)**

1. **Pull latest code** (on your Windows machine):
   ```bash
   git pull origin main
   cargo build -p scmessenger-cli --release
   cd android && ./gradlew assembleDebug -x lint --quiet
   ```

2. **CI publishes new Docker image** (automatically after merge):
   - GitHub Actions rebuilds: `testbotz/scmessenger:latest` gets new tag
   - Wait for CI to complete (5-10 min)

3. **SSH to AWS instance** (T+10):
   ```bash
   ssh -i scmessenger-relay-key.pem ec2-user@100.56.248.69
   docker pull testbotz/scmessenger:latest
   docker stop <old-container-id>
   docker run -d -p 9001:9001 -p 9876:9876 \
     -v /data:/data testbotz/scmessenger:latest \
     /path/to/relay/binary --http-bind 0.0.0.0:9876
   docker ps  # verify new container running
   curl http://localhost:9876/health  # verify health
   ```

4. **Verify connectivity** (T+15):
   ```bash
   # From Windows:
   scmessenger-cli config bootstrap add /ip4/100.56.248.69/tcp/9001
   scmessenger-cli start
   # Check logs for: "Connected to relay" + "ConnectionEstablished"
   ```

---

## Acceptance Criteria

[OK] AWS instance is running and SSH-accessible
[OK] Docker is running on instance
[OK] Current relay container is healthy (no errors in logs)
[OK] Health endpoint responds (curl 200)
[OK] Know the exact restart command for new image
[OK] Can pull latest image without errors
[OK] Windows CLI can dial 100.56.248.69:9001 after restart

---

## If Instance is Down

Run: `bash infra/aws/farm-sim-manage.sh start`
Then verify all checks above.

If AWS CLI not installed: `choco install awscli` (Windows) or per AWS docs.

---

## Deliverable

When all checks pass: `HANDOFF/audit/CLOUD_NODE_PRELAUNCH_2026-08-04.md`

Contents:
- Instance ID + state + IP
- Docker version + image tag
- Health endpoint response
- Exact restart command ready to run
- Windows CLI can dial cloud node (test result)

---

## Critical Note

Do NOT restart the container until NEW image lands (identity canonicalization merged). Once it lands:
1. Git pull main locally
2. Wait for CI to complete (new image pushed)
3. SSH to instance
4. `docker pull` + `docker stop` + `docker run` new image
5. Verify health

This ensures no version mismatch between Windows CLI + cloud relay.
