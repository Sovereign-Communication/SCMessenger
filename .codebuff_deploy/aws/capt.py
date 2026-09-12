import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scm_session import session
import json

sess = session()
ec2 = sess.client("ec2")

# Resolve latest AL2023 AMI (read-only) and existing SG/key
images = ec2.describe_images(Owners=["amazon"], Filters=[
    {"Name":"name","Values":["al2023-ami-2023.*x86_64"]},
    {"Name":"state","Values":["available"]},
])
ami = sorted(images["Images"], key=lambda x: x["CreationDate"])[-1]
print("latest AL2023 AMI:", ami["ImageId"], ami["CreationDate"])

sgs = ec2.describe_security_groups(Filters=[{"Name":"group-id","Values":["sg-02288078fa0b39e92"]}])
print("SG exists:", bool(sgs["SecurityGroups"]), "| name:", sgs["SecurityGroups"][0].get("GroupName") if sgs["SecurityGroups"] else None)
print("SG ingress:", json.dumps(sgs["SecurityGroups"][0].get("IpPermissions", [])[:3], default=str)[:600] if sgs["SecurityGroups"] else "-")

keys = ec2.describe_key_pairs()
print("key scm-node-key exists:", any(k["KeyName"]=="scm-node-key" for k in keys.get("KeyPairs",[])) or ec2.describe_key_pairs(KeyNames=["scm-node-key"]).get("KeyPairs"))

def dryrun(params):
    try:
        ec2.run_instances(**params, DryRun=True)
        return "ALLOWED (no exception)"
    except Exception as e:
        resp = getattr(e, "response", {}) or {}
        err = (resp.get("Error") or {})
        return f"code={err.get('Code')} msg={err.get('Message')}"

params = dict(
    ImageId=ami["ImageId"],
    InstanceType="t3.micro",
    MinCount=1, MaxCount=1,
    SecurityGroupIds=["sg-02288078fa0b39e92"],
    KeyName="scm-node-key",
    TagSpecifications=[{"ResourceType":"instance","Tags":[{"Key":"Name","Value":"scm-always-on-node"},{"Key":"role","Value":"relay"}]}],
    UserData="IyEvYmluL2Jhc2gKZWNobyBkcnlydW5",  # harmless placeholder
)
print("RunInstances dry-run (full proven params):", dryrun(params))
# Also test without TagSpecifications / SecurityGroup to isolate what's denied
print("RunInstances dry-run (no tags):", dryrun(dict(ImageId=ami["ImageId"], InstanceType="t3.micro", MinCount=1, MaxCount=1)))