# V040 CTO BLE architecture and ownership — tracked

Status: Active canonical architecture note

## Owners

- `.claude/commands/CTO.md` is the tracked command entry point and owns only
  seat bootstrap, load order, safety boundaries, delegation, and closeout.
- `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md` is the sole
  owner of the three-node BLE procedure, provenance matrix, checkpoint schema,
  evidence gates, stop conditions, and verdict rules.
- `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_<UTC>_<STAGE>.md` owns one immutable
  execution snapshot and is the only durable stage-state record.
- Node APIs and logs are read-only evidence inputs. Ignored `tmp/` files are
  historical evidence references only.

## Data flow

1. `/cto` loads repository rules and the tracked package.
2. The package derives fresh Git, AWS, Windows, and Pixel state.
3. The package writes a tracked stage checkpoint with complete commands,
   evidence paths, identities, hashes, and a closed verdict.
4. Each next stage consumes the checkpoint plus fresh state; it never treats an
   old checkpoint as current reachability.
5. Correlation consumes Android, Windows, and AWS captures and produces the BLE
   verdict.
6. Cleanup consumes the correlation result and writes the tracked final handoff.

There is one policy owner, one stage-record owner, and read-only evidence input.
The ignored `tmp/cto` package and architecture note remain only as historical
references and must not be edited as authorities.

## Fresh-checkout resume contract

A fresh checkout discovers `/cto` at `.claude/commands/CTO.md`, discovers the
workflow at this file and the tracked package, and discovers new state through
the `HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_*.md` naming contract. In
Codebuff/Freebuff clients, the same seat is registered as the project skill
`.agents/skills/cto/SKILL.md` (invocable as `/skill:cto`); it contains no
procedure of its own and points to the same tracked package.
