Task: V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md
Type: BUYOFF

## CEO decision

APPROVE a fresh candidate commit/worktree.

## Instructions

The architecture-pass edits are uncommitted in the shared checkout and are NOT represented in `origin/main` at `67d19d3c...`. They belong to this CTO validation task.

Create a clean isolated worktree from `origin/main` and commit only these product files:

- `core/src/transport/observation.rs`
- `core/src/routing/local.rs`
- `core/src/routing/optimized_engine.rs`
- `core/src/iron_core.rs`
- `core/src/transport/swarm.rs`
- `cli/Cargo.toml`

The resulting commit SHA becomes the three-node candidate. Do NOT touch any other files in the shared checkout. Do NOT stage, modify, or revert unrelated uncommitted changes.

## What the architecture-pass edits contain

- `AddressObserver` owns listen-port admission; empty set fails closed; observations filtered on port replacement.
- Local peer selection uses one deterministic reliability comparator with peer-ID tie-breaking.
- `OptimizedRoutingEngine` no longer duplicates `local_id`/`local_hint`.
- `IronCore` records the composition-root ownership comment.
- CLI default binary set to `scmenger-cli`.

## Constraints

- Rule-8: transport/routing changes require a non-author adversarial APPROVE before merge.
- All three nodes must be built from the same candidate SHA.
- No merge, tag, or release until functional validation completes.
