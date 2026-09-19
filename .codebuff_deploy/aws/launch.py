import os, sys, base64, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scm_session import session

USERDATA = r'''#!/bin/bash
set -ex
exec > /var/log/user-data.log 2>&1

dnf install -y docker
systemctl enable --now docker

docker pull testbotz/scmessenger:latest

docker rm -f scm-node 2>/dev/null || true
docker run -d \
  --name scm-node \
  --network host \
  --restart unless-stopped \
  -e RUST_LOG=info,scmessenger=debug \
  testbotz/scmessenger:latest \
  scm --http-bind 0.0.0.0:9876 start

echo "[OK] scm-always-on-node (full parity node) user-data complete"
'''

sess = session()
ec2 = sess.client("ec2")
ami = "ami-0db1c5c6dc64eb019"

params = dict(
    ImageId=ami,
    InstanceType="t3.micro",
    MinCount=1,
    MaxCount=1,
    SecurityGroupIds=["sg-02288078fa0b39e92"],
    KeyName="scm-node-key",
    TagSpecifications=[{
        "ResourceType": "instance",
        "Tags": [
            {"Key": "Name", "Value": "scm-always-on-node"},
            {"Key": "role", "Value": "relay"},
            {"Key": "managed", "Value": "cto-orchestrator"},
        ],
    }],
    UserData=base64.b64encode(USERDATA.encode()).decode(),
    BlockDeviceMappings=[{"DeviceName": "/dev/xvda", "Ebs": {"VolumeSize": 16, "VolumeType": "gp3"}}],
)
resp = ec2.run_instances(**params)
iid = resp["Instances"][0]["InstanceId"]
print("launched", iid)
# wait for it to get a public IP
print("polling public IP...")
ec2.get_waiter("instance_running").wait(InstanceIds=[iid], WaiterConfig={"Delay": 10, "MaxAttempts": 60})
for _ in range(30):
    d = ec2.describe_instances(InstanceIds=[iid])
    ip = d["Reservations"][0]["Instances"][0].get("PublicIpAddress")
    if ip:
        print("PUBLIC_IP", ip)
        break
    time.sleep(8)
else:
    print("no public ip yet")
print("INSTANCE_ID", iid)