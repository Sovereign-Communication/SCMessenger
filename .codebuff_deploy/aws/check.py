import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__))) if False else None
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scm_session import session

sess = session()
sts = sess.client("sts")
idn = sts.get_caller_identity()
print("caller:", idn["Arn"])

ec2 = sess.client("ec2")
# ReadOnly describe
inst = ec2.describe_instances(Filters=[{"Name":"tag:Name","Values":["scm-always-on-node","scm-node","scmessenger"]}])
rows = []
for r in inst.get("Reservations", []):
    for i in r.get("Instances", []):
        name = next((t["Value"] for t in i.get("Tags",[]) if t["Key"]=="Name"), "-")
        ip = i.get("PublicIpAddress") or "-"
        rows.append((i["InstanceId"], name, i["State"]["Name"], ip, i.get("InstanceType"), i.get("ImageId")))
print("--- instances (by tag) ---")
for row in rows:
    print(" ".join(str(x) for x in row))

# RunInstances dry-run to confirm perms
try:
    ec2.run_instances(ImageId="ami-0db1c5c6dc64eb019", MinCount=1, MaxCount=1, DryRun=True)
    print("run_instances dry-run: ALLOWED (DryRun true honored -> would proceed)")
except Exception as e:
    code = getattr(e, "response", {}).get("Error", {}).get("Code", "")
    if "DryRunOperation" in str(e) or code == "DryRunOperation":
        print("run_instances dry-run: ALLOWED")
    elif "UnauthorizedOperation" in str(e) or code == "UnauthorizedOperation":
        print("run_instances dry-run: DENIED")
    else:
        print("run_instances dry-run: ERR", code, e)
# TerminateInstances dry-run
try:
    ec2.terminate_instances(InstanceIds=[rows[0][0]] if rows else ["i-00000000000000000"], DryRun=True)
    print("terminate_instances dry-run: ALLOWED")
except Exception as e:
    code = getattr(e, "response", {}).get("Error", {}).get("Code", "")
    print("terminate_instances dry-run:", "ALLOWED" if code=="DryRunOperation" else ("DENIED" if code=="UnauthorizedOperation" else f"ERR {code}"))