# Node Model — Canonical

Status: Active
Set by: operator, 2026-09-22
Supersedes: any doc, ticket, log, or agent phrasing that distinguishes node
"roles" or "classes" of node.

## The one distinction

**All nodes are fully functional and identical. The only distinction between
nodes is whether an identity is loaded or not.** That is the whole model.

- Every node relays. Store-and-forward custody is a behavior all nodes
  perform, not a role (AGENTS.md "nodes, not relays" doctrine).
- A node with no identity loaded still runs the full transport/relay surface;
  it just cannot do identity-dependent work (message processing, custody
  admission) until one is loaded. That gap is real and is sized as a bigger
  design item in the completed investigation
  (`INVESTIGATE_IDENTITY_OPTIONAL_RELAY_MODE.md`, summarized in
  `HANDOFF/todo/_QUEUE.md`) -- it is a known limitation of the current code,
  not a node type.
- There are no "relay nodes," "bridge nodes," "light nodes," or "client
  nodes." Use "node," always. "Relay" is only ever a verb or a code-level
  identifier (`RelayCustodyStore`, `cmd_relay`, `relay_custody_msg_`).

## Fleet purposes (deployment, not node type)

Exactly two always-on cloud nodes exist today, both running the same full
node software:

| Box | Instance | Purpose | Notes |
|---|---|---|---|
| Always-on node | `i-0b41aab756eabd514` (tag `scm-always-on-node`) | Stability and reliability — an always-attached peer so the mesh has a constant custody/dial target | Docker `scm-node`, us-east-1 |
| OpenClaw node | `i-01df07a1b747b99a8` | OpenClaw gateway dogfooding SCMessenger + Sovereign-Harness in tandem; also runs a full SCM node as part of that dogfood | m7i-flex.large, dynamic IP — canonical state in `~/Documents/GitHub/OC/NODE.md` |

Both boxes are the same software doing the same node work; they differ only
in who runs them, why they exist, and which identity they carry. `scm-bridge`
(scm_bridge.py on the OpenClaw box) is an application-layer service that
happens to run on a node — it is not a node kind.

## Writing rules for every doc, ticket, and log read

1. Never write "the relay node" or "the bridge node." Write "the always-on
   node," "the OpenClaw node," or "the node at <instance/ip>."
2. A node's address and identity are properties, not its definition.
3. When inherited prose says "relay + bridge," read it as "two nodes, one of
   which runs the bridge service."

## Authority

Operator statement 2026-09-22: "all nodes are fully functional with no
distinction other than Identity or no identity loaded. That's the only
distinction. We have 1 always on node for stability and reliability, and
another node running OpenClaw and dogfooding SCMessenger/Harness in tandem."
