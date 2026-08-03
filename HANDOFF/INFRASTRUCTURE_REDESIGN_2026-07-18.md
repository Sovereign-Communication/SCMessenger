# Farm-Sim Infrastructure Redesign - Micro-Instance Architecture

**Status:** Planning Phase  
**Date:** 2026-07-18  
**Objective:** Replace single large instance with flexible micro-instance topology for dynamic network testing

---

## Current Problem

Single large instance (m7i-flex.large) at 32.197.246.78:
- SSH/console access lost
- Cannot access or recover
- Requires complete rebuild anyway
- Not ideal for network condition testing

## New Architecture

**Topology:** 7 independent t3.micro instances with VPC networking

```
VPC: scmessenger-farm-sim
├── Subnet A (x.x.x.x/24)
│   ├── relay1 (t3.micro)
│   ├── alice (t3.micro)
│   └── carol (t3.micro)
├── Subnet B (x.x.x.x/24)
│   ├── relay2 (t3.micro)
│   ├── bob (t3.micro)
│   └── david (t3.micro)
└── Subnet C (x.x.x.x/24)
    └── eve (t3.micro)

Network Policy Controller:
  - Ability to modify security groups
  - Ability to add/remove routing rules
  - Ability to inject latency/packet loss via netem
  - Ability to partition subnets

```

## Benefits

1. **Isolated Instances** — Each node independent, can restart/recreate individually
2. **Network Flexibility** — Full VPC control for testing latency, loss, partitions
3. **Cost Efficient** — t3.micro is included in free tier (if applicable)
4. **Scalability** — Easy to add/remove nodes or subnets for different topologies
5. **No Single Point of Failure** — One instance down doesn't affect others

## Infrastructure Components

### 1. VPC Setup (CloudFormation)
- VPC: x.x.x.x/16 (or similar)
- 3 subnets (one per network segment)
- Internet Gateway
- Route tables
- Security groups (allow inter-subnet + outbound)

### 2. EC2 Instances (7x t3.micro)
- **relay1** (Subnet A) — Bootstrap relay node
- **relay2** (Subnet B) — Secondary relay
- **alice** (Subnet A) — User node
- **bob** (Subnet B) — User node
- **carol** (Subnet A) — User node
- **david** (Subnet B) — User node
- **eve** (Subnet C) — Isolated node for multi-hop testing

All instances:
- Ubuntu 22.04 LTS AMI
- Same SSH key (scmessenger-farm-sim-key)
- User data script to:
  - Install Docker
  - Clone SCMessenger repo
  - Build Docker image
  - Start farm-sim topology

### 3. IAM Policy (Enhanced)

Required permissions:
```
EC2:
  - ec2:RunInstances
  - ec2:TerminateInstances
  - ec2:DescribeInstances
  - ec2:DescribeInstanceStatus
  - ec2:CreateSecurityGroup
  - ec2:DeleteSecurityGroup
  - ec2:AuthorizeSecurityGroupIngress
  - ec2:AuthorizeSecurityGroupEgress
  - ec2:RevokeSecurityGroupIngress
  - ec2:RevokeSecurityGroupEgress
  - ec2:CreateNetworkInterface
  - ec2:DeleteNetworkInterface
  - ec2:DescribeNetworkInterfaces

VPC:
  - ec2:CreateVpc
  - ec2:DeleteVpc
  - ec2:CreateSubnet
  - ec2:DeleteSubnet
  - ec2:CreateRouteTable
  - ec2:DeleteRouteTable
  - ec2:CreateRoute
  - ec2:DeleteRoute
  - ec2:AssociateRouteTable
  - ec2:DisassociateRouteTable

Network Management:
  - ec2:ModifyNetworkInterfaceAttribute
  - ec2:DescribeVpcs
  - ec2:DescribeSubnets
  - ec2:DescribeRouteTables
  - ec2:DescribeSecurityGroups

Instance Connect (if using):
  - ec2-instance-connect:SendSSHPublicKey

SSM (if using):
  - ssm:StartSession
  - ssm:TerminateSession
```

### 4. Testing Capabilities

With this architecture:

**Normal Testing:**
- All nodes in same/different subnets as needed
- Direct communication (mDNS, QUIC/TCP)
- Relay routing verification

**Network Condition Testing:**
- **Latency:** Modify security group rules, or use tc (traffic control) on nodes
- **Packet Loss:** Use tc on individual nodes
- **Partitions:** Create/remove routes between subnets
- **Cascading Failures:** Kill instances one by one, observe recovery

**Load Variation:**
- Spin up 2-node topology for quick tests
- Scale to 7-node for comprehensive tests
- Add custom nodes for specific scenarios

---

## Implementation Steps

### Phase 1: Infrastructure Provisioning (Qwen or Terraform)
1. Create VPC with 3 subnets
2. Create security groups
3. Create 7 t3.micro instances
4. Configure user data for auto-setup
5. Test SSH access to all instances

**Estimated time:** 10-15 minutes

### Phase 2: Validation
1. Verify all instances running
2. Verify Docker images built
3. Test inter-node connectivity (ping, mDNS)
4. Bootstrap validation (ledger exchange)

**Estimated time:** 5 minutes

### Phase 3: Test Execution
1. Run Phase 2.1 progressive load tests
2. Run Phase 3 failure injection
3. Test network condition modifications
4. Verify resilience and recovery

**Estimated time:** 45 minutes

---

## CloudFormation Template Structure

```yaml
Resources:
  # VPC
  FarmSimVPC:
    Type: AWS::EC2::VPC
    Properties:
      CidrBlock: x.x.x.x/16

  # Subnets (3)
  SubnetA, SubnetB, SubnetC

  # Security Groups
  FarmSimSecurityGroup:
    - Allow SSH (22) from anywhere
    - Allow all traffic between subnets
    - Allow outbound to internet

  # EC2 Instances (7)
  RelayNode1, RelayNode2, UserNodeAlice, UserNodeBob, UserNodeCarol, UserNodeDavid, UserNodeEve
    - UserData: docker-setup.sh (clone, build, run)
    - KeyName: scmessenger-farm-sim-key
    - InstanceType: t3.micro

  # IAM Role for EC2
  FarmSimEC2Role:
    - Policies: as listed above

Outputs:
  InstanceIPs: [all 7 instances' public IPs]
  VpcId: VPC identifier
  SubnetIds: [subnet IDs for network testing]
```

---

## Cost Estimate

**Free Tier (if eligible):**
- 7x t3.micro instances: ~0 (included in free tier)
- VPC: Free
- Data transfer (internal): Free
- Total: ~$0/month (if free tier eligible)

**If not eligible:**
- 7x t3.micro @ $0.0104/hour = $0.0728/hour (~$52/month running 24/7)
- Recommended: Spin up for testing, tear down after tests

---

## Next Steps

**Option A:** Implement manually via AWS Console
- Create VPC, subnets, security groups
- Launch 7 instances manually
- Configure each with user data script

**Option B:** Implement via CloudFormation/Terraform
- Write IaC template
- Deploy entire stack with one command
- Reproducible and version-controlled
- Easier to iterate on network topology

**Recommendation:** Option B (IaC) for reproducibility and future testing iterations

---

## Success Criteria

After infrastructure deployment:

[OK] All 7 instances running
[OK] SSH access working to all instances
[OK] Docker images built on all instances
[OK] Inter-node connectivity verified (ping, mDNS)
[OK] Bootstrap converges (ledgers exchange)
[OK] Ready for Phase 2&3 testing

---

## Known Limitations & Mitigations

**Limitation:** t3.micro has limited CPU/memory (1 vCPU, 1GB RAM)
- **Mitigation:** Each node runs independently; workloads are light
- **Fallback:** If performance issues, upgrade to t3.small

**Limitation:** Public subnets (NAT/IGW required for outbound)
- **Mitigation:** Each instance gets public IP for testing
- **Alternative:** Private subnets with NAT gateway if needed later

**Limitation:** No persistent storage across instance recreations
- **Mitigation:** Each instance uses ephemeral storage; tests don't require persistence
- **Enhancement:** Add EBS volumes if needed for log archival

---

**Ready to Proceed?**

Confirm if you want to:
1. Deploy via CloudFormation (recommended)
2. Deploy via Terraform
3. Deploy manually via Console
4. Adjust the topology/sizing first

Once approved, I'll create the IaC templates and deploy the new infrastructure.
