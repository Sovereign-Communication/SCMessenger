---
name: bod
description: Convene the SCMessenger Board of Directors - evaluate key architectural, cryptographic, and philosophical proposals using a 5-judge panel with sovereign-harness, requiring 100% unanimous agreement (5/5) plus judge concurrence, bounded by a 10-cent cost ceiling. Use when asked for /bod, board rulings, or architectural alignment.
---

# /bod — convene the SCMessenger Board of Directors

You are convening the Board of Directors of SCMessenger. The Board serves as the
highest governance and alignment body for the repository, evaluating key
strategic, architectural, and philosophical proposals against the project's
foundational doctrine.

## Governance doctrine

Every proposal submitted to the Board is evaluated against the Repo Philosophy:

1. **Nodes, not relays**: There are NO standalone relays in SCMessenger. Only
   NODES exist, and EVERY node relays: store-and-forward custody is a behavior
   all nodes perform, not a role. The cloud instance (scm-always-on-node) is a
   full node. No anonymous packet forwarder exists or may be introduced. Full
   parity across CLI, Android, iOS, and cloud.
2. **Sovereignty**: Sovereign peer-to-peer mesh. Zero centralized dependencies,
   zero tracking services, telemetry, or central coordinators. Discovery is
   ledger sharing between nodes. Eventual delivery via store-and-forward is
   non-negotiable.
3. **Cryptographic integrity**: Ed25519 signing, Blake3 hashes, ephemeral X25519
   ECDH, XChaCha20-Poly1305 AEAD. Storage strictly through IronCore
   (`core/src/store/`). Zero unsafe Rust without formal proof.
4. **Platform parity**: Rust core is the sole cryptographic authority; platform
   adapters are dumb byte pipes.
5. **Hygiene and safety**: No emoji anywhere. No silent truncation. Describe
   only what you have read. Test before claiming completion.

## Load order

Read these tracked files before convening the Board:

1. `AGENTS.md`
2. `HANDOFF/BOD_STATE.md`
3. `HANDOFF/CEO_STATE.md`
4. `HANDOFF/CTO_STATE.md`

## Consensus rules

- **5-Judge Panel**: Proposals are dispatched to 5 independent models via
  `sovereign-harness`.
- **Strict Unanimity (5/5)**: ALL 5 models must vote to approve (or reject).
  Any single model dissenting immediately fails the proposal.
- **Judge Concurrence**: The synthesis judge model must explicitly agree with
  the panel's unanimous vote.
- **Cost Ceiling**: Maximum **$0.10 (10 cents)** per `/bod` evaluation run.
  Default routes through the free model pool ($0.00).

## Execution

To submit a proposal to the Board of Directors:

```bash
# Inline proposal
python scripts/bod_governance.py --proposal "Proposal statement here..." --record

# Proposal from file
python scripts/bod_governance.py --proposal-file path/to/proposal.txt --record

# Dry-run validation
python scripts/bod_governance.py --proposal "..." --dry-run
```

## Resolution outcomes

- `APPROVED`: 5/5 models voted APPROVE, judge concurred. Proposal aligns with
  repo philosophy.
- `REJECTED`: Proposal violated repo doctrine, or failed unanimity / judge
  concurrence.
- `DEFERRED`: Model transport shortfall (< 5 valid votes received) or API
  failure. Fails closed.
