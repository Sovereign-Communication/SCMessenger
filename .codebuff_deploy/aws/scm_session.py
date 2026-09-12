"""SCMessenger AWS session helper. Loads ~/.config/scmorc/aws.env creds via boto3
without printing secret values. Intended for the cutover lane only."""
import os, sys


def load_env(path):
    env = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip().strip('"').strip("'")
    return env


def session():
    path = os.path.expanduser("~/.config/scmorc/aws.env")
    if not os.path.isfile(path):
        raise SystemExit("MISSING: ~/.config/scmorc/aws.env")
    e = load_env(path)
    for req in ("AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"):
        if not e.get(req):
            raise SystemExit(f"MISSING key: {req}")
    import boto3
    sess = boto3.Session(
        aws_access_key_id=e["AWS_ACCESS_KEY_ID"],
        aws_secret_access_key=e["AWS_SECRET_ACCESS_KEY"],
        region_name=e.get("AWS_DEFAULT_REGION") or e.get("AWS_REGION") or "us-east-1",
    )
    return sess