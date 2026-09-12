# V040 CTO three-node BLE checkpoint template

Use this tracked schema for each stage. Copy the file to:
`HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_<UTC-BASIC>_<STAGE>.md`
then fill every field. Never overwrite an existing checkpoint.

## Metadata

- Stage: `PREFLIGHT | NODE_READY | RADIO_ISOLATED | PROBE_SENT | CORRELATION | CLEANUP`
- UTC timestamp:
- Operator/session:
- Overall verdict: `PASS | FAIL | BLOCKED | UNVERIFIED`

## Exact commands and complete outputs

List every command run, in order, with exit status. Store complete output in
repo-local evidence files and link each file here. Do not replace complete
outputs with excerpts.

1. Command:
   - Exit:
   - Complete output:
2. Command:
   - Exit:
   - Complete output:

## Repository provenance

- Branch:
- HEAD:
- HEAD tree:
- Selected candidate ref:
- Candidate commit/tree:
- Relevant tags:

## Three-node matrix

| Node | Reachability | Version/commit/artifact | Identity | Verdict |
|---|---|---|---|---|
| AWS cloud node | | | | |
| Windows CLI | | | | |
| Pixel 6a | | | | |

## Stage-specific evidence

- Android capture path:
- Windows capture path:
- AWS capture/API output path:
- Message label/content:
- Message ID:
- BLE-specific ingress/decrypt evidence:
- Competing TCP/mDNS/Wi-Fi/cellular/relay evidence:
- Radio state:
- Cleanup state:

## Blockers and next action

- Blockers:
- Evidence still UNVERIFIED:
- Exact next action:
- Final handoff path:
