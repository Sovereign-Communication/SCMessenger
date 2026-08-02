# Orchestrator Dispatch Plan -- 2026-08-01

**Status:** Active
**Integration branch:** `orchestrator/integration-pass-2026-08-01`
**Objective:** Converge open in-progress and high-priority todo tasks into a single integration branch that passes all build/test/CI gates, supporting v0.4.0 completion and v0.5.0 parity.

## Current State

- Fresh integration branch created from `main` at commit `73c78d1d`.
- `launch_claude.ps1` updated to support non-interactive prompts and background-agent mode for direct Qwen-model delegation.
- Prompt files prepared under `tmp/dispatch_prompts/`.

## First-Wave Dispatches (independent, safe)

| Task | Model (exact) | Quota remaining | Mode | Prompt file |
|------|---------------|-----------------|------|-------------|
| U2 Topic Constants Centralization | qwen3.5-flash-2026-02-23 | 1,000,000 | Non-interactive | tmp/dispatch_prompts/U2_topic_constants.md |
| A-05 iOS Receipt Unification | qwen3-coder-480b-a35b-instruct | 666,608 | Background agent | tmp/dispatch_prompts/A05_iOS_receipt.md |
| PQC-09 Hybrid Onion Investigation | qwen3.7-max-preview | 995,911 | Background agent | tmp/dispatch_prompts/PQC09_hybrid_onion.md |
| Branch Cleanup/Convergence Inquiry | qwen3.7-max-preview | 995,911 | Background agent | tmp/dispatch_prompts/branch_cleanup_inquiry.md |

## Next-Wave Candidates (after first-wave results)

- D-05 unwrap/panic hardening (Rust, file-sharded across two CODER models)
- A-04 Android receipt unification (mirror of A-05)
- D1 desktop bridge UniFFI verification
- T-02/T-03/T-04 transport tasks
- PQC-07 ratchet wiring (requires adversarial review gate)
- Bootstrap topology wiring
- Ledger seeding/gossip hardening

## Constraints

- Do NOT dispatch to `kimi-k2.7-code` (native quota preservation).
- Use exact Qwen model names from the current quota list.
- No emojis in any committed file.
- Every code change must pass `cargo build --workspace`, `cargo clippy --workspace`, `cargo fmt --all -- --check`, and `cargo test --workspace --no-run` before the task file is moved to `HANDOFF/done/`.
- Crypto/transport/privacy changes require `crypto-security-auditor` adversarial review before merge.

## Request for GPT

Please review this plan for:
1. Gaps in the dispatch order relative to v0.4.0/v0.5.0 milestones.
2. Tasks that should be prioritized, deprioritized, or blocked.
3. Any tasks that need an audit gate before dispatch.
4. Suggested next-wave additions or removals.

Report findings in this folder or via the standard GPT handoff channel.
