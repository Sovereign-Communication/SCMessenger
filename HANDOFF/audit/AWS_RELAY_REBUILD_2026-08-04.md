# AWS Relay Teardown + Rebuild - 2026-08-04

**Status:** Complete (with one explicit permission gap - see below)
**Executed by:** Claude Code agent, AWS account 101533648751, region us-east-1,
IAM user `scmessenger-relay-orchestrator`.

Every command and output in this document was run and captured live in this
session. Nothing below is inferred or assumed except where explicitly marked
"not verified" or "inferred."

---

## NEW INSTANCE - READ THIS FIRST

```
Instance ID:  i-06b37ed4b6976ac56
Public IP:    34.203.213.35
Elastic IP:   NONE (allocation denied by IAM policy - see Permission Gaps)
Instance type: t3.micro
Key pair:     scm-node-key (reused from prior instance; no local .pem available)
Security group: sg-02288078fa0b39e92 / scm-node-sg (reused, unchanged rules)
Launched:     2026-08-04T06:58:15Z
```

**The public IP is `34.203.213.35`.** It is NOT `100.56.248.69` (the address
still referenced in `HANDOFF/todo/AWS_CLOUD_NODE_PRELAUNCH_CHECKLIST.md` and
other docs) and it is also NOT the prior broken instance's IP
`54.242.56.150`. No Elastic IP could be attached (see below), so **this IP
will change again on the next stop/start** - the root-cause fix for IP drift
was attempted but blocked by IAM policy, not resolved.

Any doc or bootstrap config that hardcodes an old IP needs to be updated to
`34.203.213.35`, and that update will go stale again if the instance is ever
stopped/restarted without an Elastic IP.

---

## 1. What the two scripts actually do (read in full before running anything)

### `infra/aws/farm-sim-manage.sh`
- Pure `aws` CLI wrapper (no boto3). Requires `aws` binary on PATH.
- Resolves the target instance by **tag `Name=scmessenger-farm-relay`**, hardcoded key file `scmessenger-farm-sim-key.pem`.
- Commands: `start`, `stop`, `status`, `ssh`, `logs`, `keepawake on|off`, `iterate`, `teardown`.
- `teardown` terminates by tag lookup, then deletes SG `scmessenger-farm-sim-sg` and key pair `scmessenger-farm-sim-key`.
- **Finding: this script would NOT have found the broken instance.** The
  broken instance was tagged `Name=scm-always-on-node`, used key
  `scm-node-key`, and SG `scm-node-sg`/`sg-02288078fa0b39e92` - none of which
  match this script's hardcoded `scmessenger-farm-relay` /
  `scmessenger-farm-sim-key` / `scmessenger-farm-sim-sg` names. Running
  `farm-sim-manage.sh teardown` as-is would have printed "No instance found"
  and done nothing. This is a real drift between the repo's scripts and what
  was actually deployed by hand at some point - not just stale docs.

### `infra/aws/provision-relay.sh`
- Also a pure `aws` CLI wrapper, explicitly checks `command -v aws` and exits
  with an error if missing (it does NOT fall back to anything).
- Creates SG `scmessenger-relay-sg` opening **tcp/443, tcp/80, udp/443**, and
  tcp/22 restricted to the caller's IP. Runs `docker run ... -p 443:443 -p
  80:80 -p 443:443/udp -p 9876:9876 testbotz/scmessenger:latest scm relay
  --http-bind 0.0.0.0:9876`.
- **Finding: this script's port scheme (443/80/udp443) does not match the
  actual relay protocol port used in production (9001 tcp+udp) or the
  existing/broken instance's security group** (which opens 9001 tcp+udp to
  0.0.0.0/0 and 9876 tcp restricted). Running `provision-relay.sh --apply`
  verbatim would have produced an instance that answers on the wrong ports
  and would not have been reachable via the bootstrap multiaddr format
  (`/ip4/.../tcp/9001`) used elsewhere in the repo (`docs/RELAY_OPERATOR_GUIDE.md`,
  `infra/ec2/alpha-relay-userdata.sh`). It also does not open 9876 to any
  caller IP in its own SG rules, so even its own health-check port would have
  been unreachable.
- Neither script matches the actual deployed resource names (`scm-node-key`,
  `scm-node-sg`, `scm-always-on-node` tag), confirming the live instance was
  provisioned by some other, undocumented process.

### Tooling reality check
- `aws` CLI binary is not on PATH in this Git Bash shell (`where aws` found
  nothing), but **AWS CLI v1.45.51 IS installed via pip**
  (`C:\Users\SCM\AppData\Roaming\Python\Python314\Scripts\aws.cmd`). Adding
  that directory to `PATH` for the session made `aws` work normally. No
  install was needed; this was purely a PATH issue on this machine.
- Given the naming mismatches above, I did not run either script verbatim.
  I ran the individual `aws` CLI calls directly, matching
  `provision-relay.sh`'s "no black-box plan/apply, review each call" spirit,
  but using the port scheme and resource names that actually match the
  current running convention (`infra/ec2/alpha-relay-userdata.sh` +
  `docs/RELAY_OPERATOR_GUIDE.md`, both of which document `--listen
  /ip4/0.0.0.0/tcp/9001` + `--http-bind 0.0.0.0:9876`), and reused the
  existing security group and key pair rather than creating new ones,
  since their rules already matched the correct port layout.

---

## 2. Teardown - exact commands and output

```
$ aws sts get-caller-identity
{
    "UserId": "AIDARPI7AQ5X2VNISBXNG",
    "Account": "101533648751",
    "Arn": "arn:aws:iam::101533648751:user/scmessenger-relay-orchestrator"
}

$ aws ec2 terminate-instances --instance-ids i-078cb870316683e79 --region us-east-1
{
    "TerminatingInstances": [
        {
            "InstanceId": "i-078cb870316683e79",
            "CurrentState": { "Code": 32, "Name": "shutting-down" },
            "PreviousState": { "Code": 16, "Name": "running" }
        }
    ]
}

$ aws ec2 wait instance-terminated --instance-ids i-078cb870316683e79 --region us-east-1
(exit code 0)

$ aws ec2 describe-instances --instance-ids i-078cb870316683e79 --region us-east-1 \
    --query 'Reservations[].Instances[0].[InstanceId,State.Name]' --output text
i-078cb870316683e79	terminated
```

Old instance confirmed `terminated`. The security group (`sg-02288078fa0b39e92`)
and key pair (`scm-node-key`) were NOT deleted - they were reused for the
new instance (see below), since their rules/name already matched the correct
port scheme and there was no reason to churn them.

---

## 3. Provisioning - exact commands and output

AMI resolution (via `describe-images`, same technique `provision-farm-sim.sh`
uses to avoid needing `ssm:GetParameters`, which the IAM policy does not
grant):

```
$ aws ec2 describe-images --owners amazon --region us-east-1 \
    --filters "Name=name,Values=al2023-ami-2*-x86_64" "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text
ami-08bc385c9fc5afc94
```

User-data used (written to a temp file in the repo root as
`userdata_scm_relay.txt`, deleted immediately after the launch call
completed - matches the existing `provision-farm-sim.sh` pattern of using a
relative `file://` path to dodge Git-Bash/MSYS path-mangling; confirmed via
`git status --short` afterward that nothing was left behind):

```bash
#!/bin/bash
set -ex
exec > /var/log/user-data.log 2>&1

dnf install -y docker
systemctl enable --now docker

docker pull testbotz/scmessenger:latest

docker run -d \
  --name scm-relay \
  --network host \
  --restart unless-stopped \
  -e RUST_LOG=info,scmessenger=debug \
  testbotz/scmessenger:latest \
  scm --http-bind 0.0.0.0:9876 relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000 --name scm-always-on-node

echo "[OK] scm-always-on-node user-data complete"
```

This mirrors `infra/ec2/alpha-relay-userdata.sh`'s proven `--network host` +
explicit `scm relay` invocation (adapted from Ubuntu/apt-get to Amazon
Linux 2023/dnf, matching `provision-relay.sh`'s AMI choice), and it is the
current image tag `testbotz/scmessenger:latest` - **not** anything from the
`fix/identity-canonicalization-steps2-5` branch or PR #136, per the explicit
instruction not to deploy that yet.

Launch call (reusing existing SG and key pair by ID/name):

```
$ aws ec2 run-instances \
    --image-id ami-08bc385c9fc5afc94 \
    --instance-type t3.micro \
    --key-name scm-node-key \
    --security-group-ids sg-02288078fa0b39e92 \
    --region us-east-1 \
    --block-device-mappings file://bdm_scm_relay.json \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=scm-always-on-node}]" \
    --user-data file://userdata_scm_relay.txt \
    --instance-initiated-shutdown-behavior stop
```

Real output (trimmed to the fields that matter; full JSON was captured live):

```
"ReservationId": "r-0a1373f033cf70dc1"
"InstanceId": "i-06b37ed4b6976ac56"
"ImageId": "ami-08bc385c9fc5afc94"
"InstanceType": "t3.micro"
"KeyName": "scm-node-key"
"SecurityGroups": [{"GroupId": "sg-02288078fa0b39e92", "GroupName": "scm-node-sg"}]
"State": {"Name": "pending"}
"LaunchTime": "2026-08-04T06:58:15.000Z"
```

```
$ aws ec2 wait instance-running --instance-ids i-06b37ed4b6976ac56 --region us-east-1
(exit code 0)

$ aws ec2 describe-instances --instance-ids i-06b37ed4b6976ac56 --region us-east-1 \
    --query 'Reservations[].Instances[0].[InstanceId,State.Name,PublicIpAddress,InstanceType]' --output text
i-06b37ed4b6976ac56	running	34.203.213.35	t3.micro
```

`bdm_scm_relay.json` used for the launch call:
```json
[{"DeviceName":"/dev/xvda","Ebs":{"VolumeSize":20,"VolumeType":"gp3"}}]
```

---

## 4. Elastic IP (IP-drift root cause fix) - DENIED

```
$ aws ec2 allocate-address --domain vpc --region us-east-1

An error occurred (UnauthorizedOperation) when calling the AllocateAddress
operation: You are not authorized to perform this operation. User:
arn:aws:iam::101533648751:user/scmessenger-relay-orchestrator is not
authorized to perform: ec2:AllocateAddress on resource:
arn:aws:ec2:us-east-1:101533648751:elastic-ip/* with an explicit deny in an
identity-based policy: arn:aws:iam::101533648751:policy/SCMessengerRelayFreeTierOnly.
```

This is real, observed output, not a guess. Cross-checked against
`infra/aws/iam-policy-scmessenger-relay.json`, which contains an explicit,
unconditional deny:

```json
{
  "Sid": "DenyElasticIpAllocationBeyondFreeAllowance",
  "Effect": "Deny",
  "Action": ["ec2:AllocateAddress"],
  "Resource": "*"
}
```

This deny is intentional (see `infra/aws/README.md`: "No Elastic IP
allocation - avoids the ... charge for an EIP not attached to a running
instance"). It is a hard IAM-level block, not a transient error - no retry or
different call shape will get around it from this IAM user.

**Root-cause fix NOT applied. IP drift is not resolved and will recur on the
next stop/start or teardown/rebuild cycle**, exactly as it did between the
prior instance's `100.56.248.69` (per old docs) and its actual last IP
`54.242.56.150`, and now this new instance's `34.203.213.35`.

**Action needed from the operator:** either (a) grant `ec2:AllocateAddress`
+ `ec2:AssociateAddress` to `scmessenger-relay-orchestrator` (removing or
scoping down the `DenyElasticIpAllocationBeyondFreeAllowance` statement -
note a single unattached EIP has a small hourly charge, contrary to the
policy comment which assumes zero EIP usage), or (b) accept IP drift as a
standing condition and invest in a DDNS-based approach instead (referenced
but apparently never implemented, per `provision-relay.sh`'s closing comment
about pointing "your DDNS hostname" at the instance).

---

## 5. Security group changes

**None were needed.** My current egress IP (confirmed via
`curl -s https://checkip.amazonaws.com` -> `147.81.41.188`) already matches
the SG's existing restricted-ingress rule for both port 22 and port 9876
(`147.81.41.188/32`, present in `sg-02288078fa0b39e92` before I touched
anything). No `AuthorizeSecurityGroupIngress` call was made or needed.

Full current SG state (`sg-02288078fa0b39e92` / `scm-node-sg`, unchanged by
this session):

| Port | Proto | Source | Purpose |
|---|---|---|---|
| 22 | tcp | 147.81.41.188/32 | SSH |
| 9001 | tcp | 0.0.0.0/0 | libp2p peers |
| 9001 | udp | 0.0.0.0/0 | libp2p QUIC |
| 9876 | tcp | 147.81.41.188/32 | HTTP health API |

Note: port 9000 (the `--http-port` status/landing page) is not opened in
this SG at all and was not tested - it was not in scope per the task
(only 9001 and 9876 were required).

---

## 6. Health verification - real command output

Waited in a polling loop (15s interval) for cloud-init (dnf install docker,
`systemctl enable --now docker`, `docker pull testbotz/scmessenger:latest`,
`docker run`) to finish. First successful health response came at
**t+45 seconds** after `run-instances`:

```
--- attempt 0 (t=0s) ---
health curl http_code=000
--- attempt 1 (t=15s) ---
health curl http_code=000
--- attempt 2 (t=30s) ---
health curl http_code=000
--- attempt 3 (t=45s) ---
health curl http_code=200
HEALTH OK
```

Full verbose curl at that point:
```
$ curl -sv -m 5 http://34.203.213.35:9876/health
*   Trying 34.203.213.35:9876...
* Established connection to 34.203.213.35 (34.203.213.35 port 9876)
> GET /health HTTP/1.1
> Host: 34.203.213.35:9876
< HTTP/1.1 200 OK
< content-type: application/json
< date: Tue, 04 Aug 2026 06:59:48 GMT
{"status":"healthy"}
```

Re-confirmed again ~90 seconds later for stability, from a fresh command
(not a repeat of the same curl call):
```
$ date -u
Tue Aug  4 07:00:39 UTC 2026
$ curl -s -m 5 -w "\nHTTP_CODE:%{http_code}\n" http://34.203.213.35:9876/health
{"status":"healthy"}
HTTP_CODE:200
```

Relay port 9001 TCP connectivity, verified via a real Python socket connect
(not assumed from the SG rule alone):
```
$ python3 -c "socket.connect(('34.203.213.35', 9001))"
TCP 9001 CONNECT: SUCCESS
```
Re-confirmed a second time at the same time as the second health check:
`TCP 9001 CONNECT: SUCCESS`.

Port 22 TCP handshake also succeeds (`TCP 22 CONNECT: SUCCESS`) - consistent
with the original instance's symptom description, expected since the SG rule
is unchanged and AL2023 sshd starts early in boot.

**Port 9001 UDP: NOT conclusively verified.** A UDP probe packet was sent and
no response and no ICMP port-unreachable came back within 3 seconds. That is
consistent with the QUIC listener being open (QUIC does not respond to
arbitrary non-handshake bytes) but it is **not proof** the UDP path works -
UDP has no handshake to observe from a plain socket probe. This is called
out explicitly rather than claimed as verified.

**What was NOT verified (no SSH access, honestly disclosed):**
- The exact Docker image digest/hash actually pulled - there is still no
  local `.pem` file for `scm-node-key` on this machine (same constraint as
  the original broken instance), so no shell access to run `docker image ls`
  or `docker inspect`. Freshness is inferred, not confirmed: this is a
  brand-new instance with no pre-existing Docker layer cache, so the
  `docker pull testbotz/scmessenger:latest` in the user-data must have
  fetched from Docker Hub rather than reusing a stale local image - but the
  actual digest was never inspected.
- `docker logs` / container process state were not inspected for the same
  reason.
- `aws ec2 get-console-output` was called and returned only a timestamp
  header with no console body at the time of the request (AL2023 console
  logging had not yet populated) - it added no additional evidence beyond
  the network-level checks above.
- A real two-node relay/dial test (Windows CLI actually routing traffic
  through this relay) was **not** performed - out of scope for this task,
  which was explicitly "get a healthy relay container running," not a full
  P2P integration test.

---

## 7. Permission gaps encountered

1. **`ec2:AllocateAddress` - explicit DENY** (see section 4). Confirmed via
   live API call, not inferred from reading the policy alone. Blocks the
   IP-drift root-cause fix entirely for this IAM user. Needs an operator
   decision (widen the policy, or accept drift + use DDNS).
2. No other permission errors were hit. `ec2:RunInstances` for `t3.micro`,
   `ec2:TerminateInstances`, SG describe, key-pair describe, and
   `ec2:GetConsoleOutput` all worked as expected under
   `SCMessengerRelayFreeTierOnly`.
3. `ec2:AuthorizeSecurityGroupIngress` was never attempted since it was not
   needed (my IP already matched the existing rule) - so its permission
   status remains unconfirmed either way, though the policy's
   `NetworkingReadAndSecurityGroups` statement should allow it if it becomes
   necessary later.

---

## 8. Verdict for 5-node run 2

**READY**, with one caveat.

Reasoning:
- The relay container is up, healthy, and responding with real HTTP 200 /
  `{"status":"healthy"}` on port 9876 from this machine, confirmed twice
  ~90 seconds apart.
- Port 9001 (the actual relay/bootstrap port referenced in
  `docs/RELAY_OPERATOR_GUIDE.md` and the multiaddr format nodes will dial)
  accepts TCP connections, confirmed twice via direct socket test.
- The image running is the current `testbotz/scmessenger:latest` (freshly
  pulled at instance boot, per the user-data script that executed), not
  anything from the unmerged identity-canonicalization branch - correct per
  the task's explicit instruction.
- **Caveat: the public IP is `34.203.213.35` and will drift again on any
  future stop/start** because Elastic IP allocation is denied. Whoever
  configures node bootstrap addresses for run 2 needs to use
  `34.203.213.35`, not any IP from older docs, and should re-check the IP
  immediately before the run starts if there is any chance the instance
  was stopped/restarted in between.
- UDP reachability on 9001 is unverified (see above) - if run 2 depends on
  the QUIC/UDP path specifically rather than TCP, treat that as an open
  question, not a confirmed pass.
