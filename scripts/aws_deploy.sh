#!/usr/bin/env bash
# AWS scm-node redeploy at the merged main SHA (Phase 3 of ORCHESTRATOR_TAKEOVER_2026-08-28).
# Usage: scripts/aws_deploy.sh   (requires ~/.ssh/scm-node-key.pem)
# Precondition: Docker Publish workflow has built testbotz/scmessenger:latest from main.
set -euo pipefail
HOST="ec2-user@54.226.67.101"
KEY="$HOME/.ssh/scm-node-key.pem"
SSH="ssh -i $KEY -o ConnectTimeout=10 -o BatchMode=yes $HOST"

echo "[INFO] Pulling testbotz/scmessenger:latest on $HOST"
$SSH "sudo docker pull testbotz/scmessenger:latest" >/dev/null

echo "[INFO] Restarting scm-node (identity persists at /opt/scm-relay-data)"
$SSH "sudo docker rm -f scm-node >/dev/null 2>&1 || true; \
sudo docker run -d --name scm-node --restart unless-stopped \
  -p 9001:9001 -p 9876:9876 \
  -v /opt/scm-relay-data:/data \
  testbotz/scmessenger:latest" | tail -1

echo "[INFO] Waiting for health..."
for i in $(seq 1 15); do
  H=$(curl -s -m 5 http://54.226.67.101:9876/health 2>/dev/null || true)
  if echo "$H" | grep -q '"status":"healthy"'; then
    echo "[OK] health: $H"
    break
  fi
  sleep 4
done

echo "[INFO] Image identity:"
$SSH "sudo docker inspect scm-node --format 'image={{.Config.Image}} started={{.State.StartedAt}}'"
echo "[DONE] AWS node redeployed. Verify git_hash == merged main SHA."
