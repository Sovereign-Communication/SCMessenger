#!/usr/bin/env python3
"""One-command idempotent (re)deploy of the OpenClaw node.

Usage:
  python redeploy_openclaw.py            # check / repair existing node, launch only if missing
  python redeploy_openclaw.py --launch   # force-launch if no healthy node found
  OPENROUTER_API_KEY=sk-or-... python redeploy_openclaw.py   # also configure harness key file

Doctrine (POSTMORTEM.md 2026-09-21):
  - NEVER terminate; this script only launches when no instance with tag Name=openclaw-node exists.
  - Resolve the node LIVE by tag; never trust a recorded IP.
  - Root volume gets 4G swap + Ollama OOM guards via user-data (fixes the OOM cascade).
  - Secrets are never embedded in user-data or printed.
"""
import json
import os
import subprocess
import sys
import time

import boto3

REGION = "us-east-1"
TAG_NAME = "openclaw-node"
INSTANCE_TYPE = "m7i-flex.large"  # the only 8GB type the IAM allowlist permits
KEY_NAME = "openclaw-key"
SG_NAME = "openclaw-sg"
SSH_KEY = os.path.expanduser("~/.ssh/openclaw-key.pem")
USER = "ec2-user"
ssm = boto3.client("ssm", region_name=REGION)
ec2 = boto3.resource("ec2", region_name=REGION)
c2 = boto3.client("ec2", region_name=REGION)

USER_DATA = """#!/usr/bin/env bash
set -eu
# 4G swap (fixes OOM kills that destroyed the previous node)
fallocate -l 4G /swapfile && chmod 600 /swapfile && mkswap /swapfile && swapon /swapfile
grep -q '/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
# Ollama OOM guards (dormant until ollama is installed; honored from first boot)
mkdir -p /etc/systemd/system/ollama.service.d
cat > /etc/systemd/system/ollama.service.d/override.conf <<'EOF'
[Service]
Environment=OLLAMA_KEEP_ALIVE=0
Environment=OLLAMA_MAX_LOADED_MODELS=1
Environment=OLLAMA_NUM_PARALLEL=1
EOF
"""


def sh(cmd, timeout=60):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True,
                          timeout=timeout, errors="replace")


def ssh(cmd, timeout=90):
    return sh(f'ssh -i "{SSH_KEY}" -o ConnectTimeout=15 -o StrictHostKeyChecking=accept-new '
              f'{USER}@{PUBLIC_IP} "{cmd}"', timeout=timeout)


def find_node():
    for i in ec2.instances.filter(Filters=[{"Name": "tag:Name", "Values": [TAG_NAME]},
                                           {"Name": "instance-state-name",
                                            "Values": ["pending", "running", "stopped"]}]):
        return i
    return None


def wait_ssh(deadline_s=600):
    print("[i] waiting for SSH ...")
    end = time.time() + deadline_s
    while time.time() < end:
        r = ssh("echo ok", timeout=30)
        if r.returncode == 0 and "ok" in r.stdout:
            return True
        time.sleep(15)
    return False


def latest_al2023_ami():
    try:
        resp = ssm.get_parameter(
            Name="/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-6.1-x86_64",
            WithDecryption=False)
        return resp["Parameter"]["Value"]
    except Exception:
        return "ami-0b2c9d1f3edcfd709"  # known-good fallback (2026-09-21 build)


def sg_id():
    for sg in c2.describe_security_groups(Filters=[{"Name": "group-name", "Values": [SG_NAME]}])[
            "SecurityGroups"]:
        return sg["GroupId"]
    raise SystemExit(f"missing SG {SG_NAME} — create it (tcp/22 from your IP only) before launch")


def launch():
    ami = latest_al2023_ami()
    print(f"[i] launching {INSTANCE_TYPE} with AMI {ami}")
    inst = ec2.create_instances(
        ImageId=ami, InstanceType=INSTANCE_TYPE, MinCount=1, MaxCount=1,
        KeyName=KEY_NAME, SecurityGroupIds=[sg_id()],
        UserData=USER_DATA,
        TagSpecifications=[{"ResourceType": "instance", "Tags": [
            {"Key": "Name", "Value": TAG_NAME},
            {"Key": "Purpose", "Value": "openclaw-gateway"},
            {"Key": "ManagedBy", "Value": "scmessenger-relay-orchestrator"}]}],
    )[0]
    print(f"[i] launched {inst.id}; waiting for running + status ok ...")
    inst.wait_until_running()
    c2.get_waiter("instance_status_ok").wait(InstanceIds=[inst.id])
    inst.reload()
    return inst


def ensure(cond, label, fix, timeout=120):
    """Require an observed successful probe, including after any repair."""
    def passed(result):
        return result.returncode == 0 if hasattr(result, "returncode") else result is True
    if passed(cond()):
        print(f"[ok] {label}")
        return True
    print(f"[..] {label}: missing — fixing")
    r = fix()
    if r is False or (hasattr(r, "returncode") and r.returncode != 0):
        raise RuntimeError(f"repair failed: {label}")
    if not passed(cond()):
        raise RuntimeError(f"repair did not pass verification: {label}")
    print(f"[ok] {label} (fixed)")
    return True


def main():
    force_launch = "--launch" in sys.argv
    global PUBLIC_IP

    inst = find_node()
    if inst is None:
        if not force_launch:
            print("[!] no openclaw-node instance found; re-run with --launch to create one")
            return 1
        inst = launch()
    inst.reload()
    PUBLIC_IP = inst.public_ip_address
    state = inst.state["Name"]
    print(f"[i] node {inst.id} state={state} ip={PUBLIC_IP}")

    if state == "stopped":
        print("[i] starting stopped node")
        inst.start()
        inst.wait_until_running()
        inst.reload()
        PUBLIC_IP = inst.public_ip_address
    if not wait_ssh():
        print("[!!] node unreachable over SSH (OOM boot loop? check console output)")
        return 2

    # --- idempotent install/repair ---
    def has(cmd):
        return lambda: ssh(f"command -v {cmd} >/dev/null 2>&1 && echo y").returncode == 0

    ensure(lambda: ssh("test -d /home/ec2-user/.openclaw && echo y").returncode == 0,
           "OpenClaw rootless install",
           lambda: ssh("curl -fsSL https://openclaw.ai/install-cli.sh | bash -s -- --no-onboard",
                       timeout=600))

    ensure(lambda: ssh("systemctl --user is-active openclaw-gateway | grep -q active"),
           "gateway service active",
           lambda: ssh("systemctl --user daemon-reload && systemctl --user enable --now "
                       "openclaw-gateway && sudo loginctl enable-linger ec2-user", timeout=60))

    ensure(lambda: ssh("command -v ollama >/dev/null 2>&1 && echo y").returncode == 0,
           "ollama installed",
           lambda: ssh("curl -fsSL https://ollama.com/install.sh | sh", timeout=600))

    ensure(lambda: ssh("ollama list | grep -q sparkx25:4b"),
           "SparkX-4B model present",
           lambda: ssh(
               "curl -sL --max-time 420 -o /tmp/Spark-Q4_K_M.gguf "
               "https://huggingface.co/XHToken/Spark-X2.5-4B-GGUF/resolve/main/Spark-X2.5-4B-Q4_K_M.gguf "
               "&& printf 'FROM /tmp/Spark-Q4_K_M.gguf\\n' > /tmp/M.sparkx "
               "&& ollama create sparkx25:4b -f /tmp/M.sparkx && rm -f /tmp/Spark-Q4_K_M.gguf /tmp/M.sparkx",
               timeout=560))

    ensure(lambda: ssh("python3 -m pip --version >/dev/null 2>&1 && echo y").returncode == 0,
           "pip (user)",
           lambda: ssh("curl -sS https://bootstrap.pypa.io/pip/3.9/get-pip.py -o /tmp/gp.py "
                       "&& python3 /tmp/gp.py --user", timeout=180))

    ensure(lambda: ssh("test -x ~/.local/bin/harness-mcp && echo y").returncode == 0,
           "sovereign-harness installed",
           lambda: ssh("python3 -m pip install --user --quiet "
                       "git+https://github.com/Sovereign-Communication/harness.git", timeout=300))

    # tolerant protocol negotiation (gateway speaks newer MCP than harness)
    ensure(lambda: ssh("grep -q 'isinstance(requested, str)' "
                       "~/.local/lib/python3.9/site-packages/harness/mcp.py && echo y").returncode == 0,
           "harness MCP version-negotiation patch",
           lambda: ssh("python3 - <<'EOF'\n"
                       "p='/home/ec2-user/.local/lib/python3.9/site-packages/harness/mcp.py'\n"
                       "s=open(p).read()\n"
                       "o='    def _negotiate_version(self, requested):\\n'\n"
                       "n=o+'        if isinstance(requested, str) and requested:\\n'\n"
                       "n+='            return requested\\n'\n"
                       "s=s.replace(o,n,1); open(p,'w').write(s)\n"
                       "EOF", timeout=60))

    # env.vars provider markers (OLLAMA marker required for provider registration)
    key = os.environ.get("OPENROUTER_API_KEY", "")
    ensure(lambda: ssh("grep -q '^OLLAMA_API_KEY=' ~/.openclaw/env.vars && echo y").returncode == 0,
           "env.vars provider markers",
           lambda: ssh("printf 'OLLAMA_API_KEY=local\\n' > ~/.openclaw/env.vars"
                       + (f" && printf 'OPENROUTER_API_KEY={key}\\n' >> ~/.openclaw/env.vars"
                          if key else ""), timeout=30))

    ensure(lambda: ssh("~/.openclaw/bin/openclaw mcp list | grep -q harness"),
           "harness MCP registered",
           lambda: ssh("python3 - <<'EOF'\n"
                       "import json, subprocess\n"
                       "entry={'command':'/home/ec2-user/.local/bin/harness-mcp','args':[],'env':{\n"
                       "'mcp_allow_write':'true',\n"
                       "'mcp_allowed_roots':'/home/ec2-user/.openclaw/workspace'}}\n"
                       "key=''\n"
                       "for l in open('/home/ec2-user/.openclaw/env.vars'):\n"
                       "    if l.startswith('OPENROUTER_API_KEY='): key=l.strip().split('=',1)[1]\n"
                       "if key: entry['env']['OPENROUTER_API_KEY']=key\n"
                       "subprocess.run(['/home/ec2-user/.openclaw/bin/openclaw','mcp','set','harness',\n"
                       "json.dumps(entry)], check=True)\n"
                       "EOF", timeout=60))

    ensure(lambda: ssh("test -f ~/.config/harness/openrouter.env && echo y").returncode == 0,
           "harness openrouter.env key file",
           lambda: ssh("mkdir -p ~/.config/harness && grep '^OPENROUTER_API_KEY=' ~/.openclaw/env.vars "
                       "> ~/.config/harness/openrouter.env && chmod 600 ~/.config/harness/openrouter.env",
                       timeout=30) if key else None)

    # config chain + timeouts + $0-idle
    ensure(lambda: ssh("~/.openclaw/bin/openclaw config get agents.defaults.model.primary | "
                       "grep -q sparkx25:4b"), "model chain: sparkx primary",
           lambda: ssh("set -a; . ~/.openclaw/env.vars; set +a; "
                       "~/.openclaw/bin/openclaw config set agents.defaults.model.primary "
                       "ollama/sparkx25:4b && "
                       "~/.openclaw/bin/openclaw config set agents.defaults.model.fallbacks "
                       "'[\"openrouter/z-ai/glm-5.3-flash\",\"openrouter/deepseek/deepseek-v4.1-flash\"]'",
                       timeout=60))
    ensure(lambda: ssh("~/.openclaw/bin/openclaw config get agents.defaults.timeoutSeconds | "
                       "grep -q 3600"), "gateway turn timeout 3600",
           lambda: ssh("~/.openclaw/bin/openclaw config set agents.defaults.timeoutSeconds 3600",
                       timeout=30))
    ensure(lambda: ssh("~/.openclaw/bin/openclaw config get agents.defaults.heartbeat.every | "
                       "grep -q 0m"), "$0-idle: heartbeat off",
           lambda: ssh("~/.openclaw/bin/openclaw config set agents.defaults.heartbeat.every 0m",
                       timeout=30))
    ensure(lambda: ssh("~/.openclaw/bin/openclaw config get "
                       "plugins.entries.memory-core.config.dreaming.enabled | grep -q False"),
           "$0-idle: memory dreaming off",
           lambda: ssh("~/.openclaw/bin/openclaw config set "
                       "plugins.entries.memory-core.config.dreaming.enabled false", timeout=30))

    # final health
    print()
    print("[i] final health:")
    for probe in ("systemctl --user is-active openclaw-gateway scm-node scm-bridge 2>/dev/null",
                  "~/.openclaw/bin/openclaw mcp probe harness 2>&1 | tail -1",
                  "ollama list | grep -i spark",
                  "free -h | grep -i swap"):
        r = ssh("set -o pipefail; " + probe, timeout=60)
        print("   ", (r.stdout or r.stderr).strip().replace("\n", " | ")[:160])
        if r.returncode != 0:
            raise RuntimeError("final health probe failed")
    print()
    print("[i] DONE — update NODE.md with the new IP if it changed, and re-stage the audit "
          "target (~/.openclaw/workspace/scm/) if this was a fresh launch.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
