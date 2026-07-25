# AWS free-tier IAM setup for the SCMessenger cloud node/farm-sim rig

Status: Active. Supports FARM_FINAL_PLAN.md WS-FARM-B (B4: cloud nodes,
B5/B6: rig-based farm-topology simulation). Free tier only, $0 budget target.

## Why this file exists

`iam-policy-scmessenger-relay.json` is a scoped IAM policy so the orchestrator
(or a delegated agent) can provision/manage the node EC2 instance(s) without
ever holding root credentials or broad account access. It is deliberately
restrictive:

- **Instance type locked to `t2.micro`/`t3.micro` only** (free-tier eligible) -
  `ec2:RunInstances` is denied for any other type via a `Condition`.
- **EBS volume capped at 30 GB, gp2/gp3 only** - matches the free-tier storage
  allowance.
- **Region locked to `us-east-1`** - keeps the free-tier hour pool
  unambiguous; change the `aws:RequestedRegion` values in the policy if you
  want a different home region, but pick ONE region and stick to it.
- **No Elastic IP allocation** - avoids the (small but real) charge for an EIP
  not attached to a running instance; the DDNS approach in the farm plan
  doesn't need one anyway.
- **Explicit deny list** for anything that costs money outside EC2/EBS basics
  or that expands blast radius: `iam:*`, `s3:*`, `rds:*`, spot/reserved
  instance purchasing, `ModifyInstanceAttribute` (blocks instance-type
  resizing past the free tier), etc. Deny statements win over allow
  statements in IAM evaluation, so this holds even if a later edit
  accidentally widens an Allow.
- **No self-service IAM** - this identity cannot create/modify IAM users,
  roles, or policies (including its own) - a compromised or buggy delegated
  agent can't escalate its own privileges.
- **Budgets/CloudWatch alarm permissions included** - so the same identity can
  also set up its own spend alarm (see below) without needing a separate
  higher-privilege pass.

This policy does NOT by itself guarantee zero spend - IAM can't prevent
free-tier hour/storage exhaustion if you leave instances running past the
750 hrs/month or 30 GB allowance. Pair it with an AWS Budget (step 4 below).

## Setup steps (operator-only, ~10 minutes)

1. **Do not use your AWS root account for this.** Log into the AWS Console as
   root only to create the IAM user in step 2, then use the IAM user for
   everything else, always.

2. **Create a dedicated IAM user** (IAM console -> Users -> Create user):
   - Name: `scmessenger-relay-orchestrator` (or similar).
   - Access type: Programmatic access (generates an access key + secret).
   - Do NOT enable console/password access for this user - API-only.

3. **Attach the policy:**
   - IAM console -> Policies -> Create policy -> JSON tab -> paste the
     contents of `iam-policy-scmessenger-relay.json` -> name it
     `SCMessengerRelayFreeTierOnly` -> Create.
   - Back on the user (step 2) -> Add permissions -> Attach policy directly ->
     select `SCMessengerRelayFreeTierOnly`.

4. **Set an AWS Budget (defense in depth, do this even though the policy
   restricts spend-causing actions):**
   - Billing console -> Budgets -> Create budget -> Cost budget.
   - Amount: $1.00 (a $0 budget can't alert - $1 catches literally any real
     spend before it compounds).
   - Alert threshold: 1% actual spend (fires almost immediately on any
     charge).
   - Email the alert to yourself. Optionally add a second budget at $5 as a
     "something is seriously wrong" tripwire.

5. **Generate the access key** (IAM console -> the user -> Security
   credentials -> Create access key -> "Application running outside AWS").
   Copy both the Access Key ID and Secret Access Key immediately - AWS only
   shows the secret once.

6. **Store credentials the same way every other lane's key is stored in this
   repo - never in chat, never committed:**
   ```
   # ~/.config/scmorc/aws.env  (outside the repo tree, gitignored by location)
   AWS_ACCESS_KEY_ID=AKIA...
   AWS_SECRET_ACCESS_KEY=...
   AWS_DEFAULT_REGION=us-east-1
   ```
   If you ever paste a key into a chat/session transcript by accident, treat
   it as burned and rotate it (IAM console -> the user -> Security
   credentials -> deactivate old key -> create a new one) - same policy as
   this repo's other credential-leak incidents on record.

7. **Tell the orchestrator it's ready.** Once `~/.config/scmorc/aws.env`
   exists, provisioning work (Terraform or plain `aws` CLI calls scoped to
   this policy) can proceed. Every `terraform plan` / `aws ec2 run-instances`
   equivalent should be reviewed before it executes against the real account,
   even though the IAM policy itself is the hard backstop.

## Optional but recommended: automatic kill-switch (Budget Actions)

Step 4's $1 budget only emails you - it does not stop anything by itself. AWS
Budgets can also take an automatic ACTION when a threshold is crossed:
attaching a hard-deny IAM policy to the node user, and/or stopping the node
EC2 instance. This closes the gap between "you get an email" and "spend
actually stops."

**Note on scope: this does NOT require adding any permission to the
orchestrator's own `SCMessengerRelayFreeTierOnly` policy.** That policy
deliberately denies `iam:*` so a delegated agent can never touch IAM,
including its own permissions - Budget Actions need to CREATE an IAM role and
policy, which is exactly the kind of action that policy is designed to
prevent. This is a one-time, operator-only, few-minutes console step using
the three files below - not something to hand to the orchestrator.

1. **Create the quarantine policy** (the thing that gets attached when the
   kill-switch fires): IAM console -> Policies -> Create policy -> JSON ->
   paste `quarantine-policy.json` -> name it `SCMessengerBudgetQuarantine` ->
   Create. It's a single explicit `Deny: * on *` - explicit Deny always wins
   over any Allow in IAM evaluation, so attaching this to the node user
   neutralizes it regardless of what `SCMessengerNodeFreeTierOnly` still
   allows.

2. **Create the Budget Actions execution role** (the identity AWS Budgets
   itself assumes to perform the action - separate from your node user):
   IAM console -> Roles -> Create role -> Custom trust policy -> paste
   `budget-action-trust-policy.json` -> name it
   `SCMessengerBudgetActionExecutionRole` -> attach a new policy pasted from
   `budget-action-execution-policy.json` (name it
   `SCMessengerBudgetActionExecution`) -> Create role.

3. **Wire the action into your budget**: Billing -> Budgets -> open the $1
   budget from step 4 -> Add action:
   - Action type: **IAM policy**.
   - Policy: `SCMessengerBudgetQuarantine` (from step 1 above).
   - Target: the `scmessenger-relay-orchestrator` user.
   - Execution role: `SCMessengerBudgetActionExecutionRole` (from step 2).
   - Trigger threshold: actual spend >= $2 (above the $1 email alert, so you
     get the warning first and the hard stop second if you don't act).
   - Approval: choose "Automatic" for a true kill-switch, or "Manual (via
     email/SNS)" if you'd rather approve the action yourself first - both are
     reasonable; automatic is the closer match to "kill-switch."

4. **Optional second action, same budget**: repeat with Action type
   **EC2 instance stop** targeting instances tagged `Name=scmessenger-farm-relay`
   (the tag `provision-relay.sh` already applies) - belt-and-suspenders so
   even if the IAM quarantine is somehow bypassed, the actual running
   instance stops too. Uses the same execution role (its policy already
   includes `ec2:StopInstances` scoped to that tag).

To undo the quarantine after you've fixed whatever caused the spend: IAM
console -> the `scmessenger-relay-orchestrator` user -> detach
`SCMessengerBudgetQuarantine`. Nothing else needs to change - the original
`SCMessengerRelayFreeTierOnly` policy is untouched throughout.

## If you'd rather not hand over any AWS key at all

The orchestrator can still write and validate the Terraform/compose
configuration, and dry-run everything against the LOCAL docker rig
(`cloud/mesh/docker-compose.mesh-test.yml`) with zero cloud dependency - the
farm-topology simulation (B6) works entirely locally. Cloud deployment (B4)
is the only piece that needs real AWS access.
