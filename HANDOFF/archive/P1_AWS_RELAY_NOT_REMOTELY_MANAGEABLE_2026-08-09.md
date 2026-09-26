# P1 -- the AWS relay cannot be updated: no working SSH key on this machine

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Open -- BLOCKED on operator
Filed: 2026-08-09 ~17:00Z (Windows lane, during the five-node prep)

## What is blocked

Node 3 of the five-node fleet (AWS Ubuntu headless relay) is running a **stale
image** built from `6b2573fa` (PR 136+137+138). The current anchor is
`d48558a8` -- roughly 60 commits of mesh, relay and CLI work newer, including
every dial-policy and relay-classification fix landed today.

The instance is alive and healthy (`{"status":"healthy"}` at
`http://54.226.67.101:9876/health`), so it participates in the mesh -- but as an
**old-code peer**. Any three- or five-node result that includes it is testing a
mixed-version fleet unless this is resolved.

## The blocker

A candidate image now EXISTS and is published, so the expensive part is done:

```
docker.io/testbotz/scmessenger:sha-d48558a
docker.io/testbotz/scmessenger:tracking-pre-v040-tag-work
digest sha256:a69830281160bbacec4fddb7233a31b89ce51da1ef01e95d6bd2b34944af5bd6
```

(Built by CI via `workflow_dispatch` on `docker-publish.yml`. Note the workflow
only applies `latest` on the default branch, so this did NOT touch the
production `latest` tag.)

What is missing is a way to tell the instance to pull it.

- **SSH fails with all three local keys.** `scm-node-key.pem`,
  `scmessenger-farm-sim-key-v2.pem` and `scmessenger-farm-sim-key.pem` all
  return `Permission denied (publickey,...)` for `ubuntu@54.226.67.101`.
- `HANDOFF/audit/AWS_RELAY_REBUILD_2026-08-04.md` records why:
  *"Key pair: scm-node-key (reused from prior instance; no local .pem
  available)."* The local `scm-node-key.pem` is from an older instance and does
  not match the deployed key pair.
- **No AWS CLI is installed on this host**, so SSM Session Manager, EC2 Instance
  Connect, and `describe-instances` are all unavailable too.

## What an operator needs to choose

1. **Supply the correct `.pem`** for key pair `scm-node-key`, or
2. **Install and configure the AWS CLI** here (credentials already exist at
   `~/.config/scmorc/aws.env`), then use SSM Session Manager -- this requires
   the instance to have the SSM agent and an IAM instance profile; neither is
   confirmed, or
3. **Use EC2 Instance Connect / the AWS console** to run the update by hand, or
4. **Rebuild the instance** with a key pair we hold locally.

Option 3 is the fastest one-off; option 1 or 2 is what makes this repeatable.

## The update command, once access exists

Do NOT build on the instance. A previous attempt OOM'd after 16 hours on the
t3.micro; that is why a prebuilt image was published.

```bash
docker pull testbotz/scmessenger@sha256:a69830281160bbacec4fddb7233a31b89ce51da1ef01e95d6bd2b34944af5bd6
# stop and remove the running container, then re-run it with the same
# port mapping and volume mounts as the current one -- capture
# `docker inspect` on the existing container FIRST so the identity
# volume and -p 443:443 / -p 9001 / -p 9876 mappings are preserved.
```

**Preserve the identity volume.** The relay's PeerId is referenced in
bootstrap multiaddrs and in other nodes' ledgers; a fresh identity would add
one more ghost entry to every peer on the fleet, which is a defect we are
actively trying to reduce.

## Interim position

Until this is resolved the fleet is: Windows (anchor), Android (anchor once its
APK lands), AWS (stale). A two-node anchored test plus a stale relay is worth
running for transport evidence, but it is NOT a clean three-node result and
must not be recorded as one.
