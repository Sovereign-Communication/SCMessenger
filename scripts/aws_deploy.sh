#!/usr/bin/env bash
# AWS scm-node redeploy at the current main SHA.
#
# Usage: scripts/aws_deploy.sh [host-ip]
#   With no argument the host is discovered from the EC2 API by instance tag,
#   because the node's public IP changes on every instance replacement and a
#   hardcoded address goes stale silently. Requires ~/.ssh/scm-node-key.pem and,
#   for discovery, ~/.config/scmorc/aws.env.
#
# Precondition: the Docker Publish workflow has built testbotz/scmessenger:latest
# from the SHA you intend to deploy.
#
# THIS IS THE ONLY SUPPORTED DEPLOY PATH. The `-v /opt/scm-relay-data:/data`
# mount below is what makes the node's identity and ledger survive a redeploy.
# A container started without it writes identity to the ephemeral container
# layer, so the node comes back as a stranger and the mesh cannot find it. That
# regression has happened once already (2026-08-31): the instance was replaced
# using a userdata script that omitted the mount, silently discarding PR #240.
set -euo pipefail

KEY="$HOME/.ssh/scm-node-key.pem"
DATA_DIR="/opt/scm-relay-data"
IMAGE="testbotz/scmessenger:latest"

HOST_IP="${1:-}"
if [ -z "$HOST_IP" ]; then
  echo "[INFO] Discovering scm-always-on-node public IP from the EC2 API"
  HOST_IP=$(python3 - <<'PY'
import os, sys
sys.path.insert(0, os.path.expanduser("~/Documents/GitHub/SCMessenger/.codebuff_deploy/aws"))
try:
    from scm_session import session
except Exception:
    print("", end="")
    raise SystemExit(0)
ec2 = session().client("ec2")
r = ec2.describe_instances(Filters=[
    {"Name": "tag:Name", "Values": ["scm-always-on-node", "scm-node", "scmessenger"]},
    {"Name": "instance-state-name", "Values": ["running"]},
])
ips = [i.get("PublicIpAddress") for res in r.get("Reservations", [])
       for i in res.get("Instances", []) if i.get("PublicIpAddress")]
print(ips[0] if ips else "", end="")
PY
)
fi

if [ -z "$HOST_IP" ]; then
  echo "[FAIL] Could not determine the node's public IP. Pass it explicitly:"
  echo "       scripts/aws_deploy.sh <host-ip>"
  exit 1
fi

HOST="ec2-user@${HOST_IP}"
SSH="ssh -i $KEY -o ConnectTimeout=15 -o BatchMode=yes -o StrictHostKeyChecking=no $HOST"
echo "[INFO] Target: $HOST"

echo "[INFO] Pulling $IMAGE"
$SSH "sudo docker pull $IMAGE" >/dev/null

echo "[INFO] Ensuring the persistent data directory exists at $DATA_DIR"
$SSH "sudo mkdir -p $DATA_DIR"

# If a previous container ran WITHOUT the mount, its identity lives in the
# container layer and would be lost here. Rescue it before removing the
# container, but never overwrite an identity that is already persisted.
echo "[INFO] Rescuing any container-local identity into $DATA_DIR"
$SSH "if sudo docker inspect scm-node >/dev/null 2>&1; then \
  if [ ! -f $DATA_DIR/storage/db ]; then \
    sudo docker cp scm-node:/data/. $DATA_DIR/ 2>/dev/null && echo '[INFO] rescued container-local identity'; \
  else echo '[INFO] persisted identity already present, not overwriting'; fi; \
fi"

echo "[INFO] Restarting scm-node with persistent identity at $DATA_DIR"
$SSH "sudo docker rm -f scm-node >/dev/null 2>&1 || true; \
sudo docker run -d --name scm-node --network host --restart unless-stopped \
  -e RUST_LOG=info,scmessenger=debug \
  -e SCM_DATA_DIR=/data -e SCMESSENGER_DATA_DIR=/data \
  -v $DATA_DIR:/data \
  $IMAGE \
  scm --http-bind 0.0.0.0:9876 start" >/dev/null

echo "[INFO] Waiting for health..."
HEALTHY=0
for _ in $(seq 1 15); do
  H=$(curl -s -m 5 "http://${HOST_IP}:9876/health" 2>/dev/null || true)
  case "$H" in
    *'"status":"healthy"'*) echo "[OK] health: $H"; HEALTHY=1; break ;;
  esac
  sleep 4
done
[ "$HEALTHY" -eq 1 ] || { echo "[FAIL] node did not become healthy"; exit 1; }

echo "[INFO] Verifying the mount is actually present (the regression guard):"
MOUNTS=$($SSH "sudo docker inspect scm-node --format '{{json .Mounts}}'")
echo "  $MOUNTS"
case "$MOUNTS" in
  *"$DATA_DIR"*) echo "[OK] persistent volume mounted" ;;
  *) echo "[FAIL] scm-node is running WITHOUT the $DATA_DIR mount -- identity will be lost"; exit 1 ;;
esac

echo "[INFO] Deployed identity and provenance:"
curl -s -m 10 "http://${HOST_IP}:9876/api/identity" || true
echo
$SSH "sudo docker logs scm-node 2>&1 | grep -m2 -E 'CLI Version|Core Provenance'" || true

echo "[DONE] AWS node redeployed at $HOST_IP. Confirm the git hash matches the intended main SHA."
