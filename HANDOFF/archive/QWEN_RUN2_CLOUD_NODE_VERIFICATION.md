# Qwen Task: Cloud Node Verification for 5-Node Run 2

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date**: 2026-08-04
**Status**: EXECUTE NOW
**Priority**: HIGH - required for run 2
**Owner**: Qwen free tier / Windows execution lane

---

## Objective

Verify the AWS cloud node at 100.56.248.69:9001 is healthy, running current HEAD, and ready for the 5-node test.

---

## Required Actions

### 1. Container Image Verification
- [ ] Check running container image digest matches latest CI build
- [ ] Verify it's the `testbotz/scmessenger` CI image with PR #133 fixes
- [ ] Confirm image was built from `origin/main` at `ba362cc5` or later

### 2. Node Health Verification
- [ ] Verify node identity (peer ID) is current and stable
- [ ] Verify reachable listener on public endpoint (100.56.248.69:9001)
- [ ] Verify synchronized clock (NTP)
- [ ] Verify logs retained for test interval (configure retention if needed)

### 3. Functional Verification
- [ ] Test dial from local Windows CLI to cloud node
- [ ] Verify `ConnectionEstablished` event
- [ ] Verify relay custody store operational (can accept/store/forward messages)
- [ ] Verify no zombie processes (swarm liveness guard working)

### 4. Configuration
- [ ] Confirm bootstrap addresses configured correctly
- [ ] Verify no hardcoded fallback addresses that could conflict
- [ ] Check relay custody TTL/config appropriate for test window

---

## Deliverable

Create `HANDOFF/audit/CLOUD_NODE_RUN2_VERIFICATION_2026-08-04.md` with:

| Check | Status | Evidence |
|-------|--------|----------|
| Image digest | | |
| Peer ID | | |
| Listener reachable | | |
| Clock sync | | |
| Log retention | | |
| Dial test | | |
| Relay custody | | |
| No zombie | | |

Include commands run and raw output.

---

## Commands to Run (suggested)

```bash
# SSH to AWS instance
ssh -i ~/.ssh/aws_key ubuntu@100.56.248.69

# Check container
docker ps --format "table {{.Image}}\t{{.Status}}\t{{.Ports}}"
docker inspect <container_id> --format '{{.Image}}' | sha256sum

# Check node logs
docker logs <container_id> --tail 100 | grep -E "(identity|listener|peer_id|startup)"

# Test dial from Windows (run on Windows host)
scm dial --peer <cloud_peer_id> --timeout 30

# Check relay custody
docker exec <container_id> scm relay status
```

---

## Gates

- All checks PASS
- Image is current HEAD (post PR #133)
- Node responds to dial within 10 seconds
- Relay custody accepts and forwards test message

---

## Notes

- This node is the AWS relay used by all 5 nodes for internet connectivity
- Must be healthy before UTC window starts
- If issues found, escalate immediately - may need container restart/redeploy
- Coordinate with Windows orchestrator on UTC window timing