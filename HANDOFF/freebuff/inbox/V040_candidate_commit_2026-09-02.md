Task: V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md
Type: DONE

## Candidate committed, pushed, PR open

- Worktree: `scm-v040-candidate`, branch `cto/v040-candidate-2026-09-02`.
- Candidate SHA: `a759e0c70f6123daf7d3f2016ca0f4a6cfd6a850` (7 files, +222/-327).
- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/272 (base main).

## Reconciliation (per directive section 2)

Byte-identical in both copies (sha256 verified): `cli/Cargo.toml`, `core/src/iron_core.rs`, `core/src/routing/local.rs`, `core/src/routing/optimized_engine.rs`.
Differing files - candidate version taken (verified strict superset of main-checkout copy, compiles, emoji-stripped):
- `core/src/transport/observation.rs`: candidate = main + listen-port allowlist semantics + fail-closed tests (0 main-only lines).
- `core/src/transport/swarm.rs`: candidate = main + ListenerClosed retraction (both loops) + ExpiredListenAddr arm + sync_external_address promotion sync (cfg native-only). The 26 main-only lines are: emoji log strings (hook-blocked; base origin/main itself carries 5), the two pre-fix inline promotion blocks (superseded by the sync helper), and import-list differences that are self-consistent in the candidate (lib-test crate compiles clean).
- `docs/ARCHITECTURE_SCOPE_V040.md`: copied from main checkout (untracked there), sha256 `6243df762ff1487d7ae29e86033df37e1c82f30879ceafb0e220b45aa911e911`.

## Gates on a759e0c7 (raw outputs in scm-v040-candidate/tmp/g*.log)

1. `cargo fmt --check` - PASS
2. `cargo test -p scmessenger-core --lib observation` - PASS (6/0)
3. `cargo test -p scmessenger-core --lib local` - PASS (35/0)
4. `cargo test -p scmessenger-core --lib optimized_engine` - PASS (6/0)
5. `cargo test -p scmessenger-cli --lib` - PASS (82/0)
6. `cargo check -p scmessenger-core --all-targets` - PASS (7m09s)
7. `cargo check -p scmessenger-cli --lib` - PASS (3m05s)
8. `git diff --check` - PASS

Supplementary: full core lib suite 1396/0 (5 ignored); wasm32 proof `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` - PASS (pre-existing wasm warnings only).

## Next decision needed

Rule-8: #272 touches `core/src/transport/{observation,swarm}.rs` and `core/src/routing/` - requires a NON-AUTHOR adversarial APPROVE before merge. After APPROVE: three-node validation on SHA a759e0c7 per the 09-02 handoff (Windows CLI by operator, AWS cloud node, Android Pixel).
