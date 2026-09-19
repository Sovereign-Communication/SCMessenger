# V040 REVIEW DISPATCH -- #270 ephemeral-port P0 (adversarial, qwen free max)

Status: **DISPATCHED 2026-09-01 -- REVIEW FILED, RESOLVED.** Model: qwen3-30b-a3b-thinking-2507 (qwq-plus non-responsive -> same-tier fallback). Verdict: `HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md`. **Resolution 2026-09-01: finding REJECTED after full investigation** -- the promotion sites sit inside the `#[cfg(not(target_arch="wasm32"))]` event loop; the wasm loop records observations for diagnostics parity only (zero add_external_address in the wasm build). Guard kept; commit `6fd0230b` (comments only) documents the empty-set semantics. Effective disposition: APPROVE. PR #270 body updated with the resolution.
Priority: P0 -- the whole mesh was advertising a non-dialable address at the source; this PR fixes it
Lane: **Qwen free -- qwq-plus** (reasoning tier, 906,398 remaining per docs/QWEN_QUOTA_LEDGER.md -- adversarial review is reasoning-tier work; spend-first bucket, expires 10-06)
Target: PR **#270** `freebuff/v040-t14-ephemeral-port` (2 files: `core/src/transport/observation.rs`,
`core/src/transport/swarm.rs`)
Reviewer constraint: MUST NOT be the author. Draft your verdict BEFORE reading the author's audit
section in the PR body; then check your findings against it.

## What it does
`AddressObserver` gains a listen-port allowlist (fed by NewListenAddr). Observations whose port is not
a port we listen on are dropped at record time and excluded from the consensus; both
`add_external_address` promotion sites refuse non-listen ports (defense-in-depth). Consensus ties
break deterministically (count desc, address asc).

## Attack checklist
1. **The deepest question first**: does dropping all non-listen-port observations break legitimate
   NAT-mapped advertisement? Trace every reflection source (Identify observed_addr, the on-demand
   reflection protocol). An inbound reflection through a port-forwarded NAT carries the mapped
   LISTEN port; an outbound flow carries the ephemeral source port. Which survives the allowlist?
2. Whitelist evasion: can an attacker still win promotion via a permitted port on a chosen public IP
   (single-observer topologies -- mobile-hub), via observation timing (observations recorded before
   the first NewListenAddr), or via the empty-listen-set fail-open question -- confirm it fails CLOSED.
3. Tie-break determinism: no HashMap-iteration-order dependence left in `recalculate_consensus`
   or the promotion picks.
4. Reader consistency: diagnostics + mobile hints + relay/circuit construction all consume the
   filtered consensus or guarded sites -- no unguarded reader.
5. Regression surface: DCUtR / relay / hole-punch paths untouched and unbroken (the PR claims zero
   change there -- verify the diff shows it).
6. The documented residuals are INTENTIONAL (single-observer injection; wasm accept-all diagnostics
   parity) -- your job is to confirm they are real, correctly documented, and not bigger than claimed.

## Method
- `cd /c/Users/SCM/Documents/GitHub/SCMessenger && gh pr diff 270 > tmp/rev270.diff` (worktree
  `scm-t14-ephemeral-port` on disk for the full tree).
- Review-only: no code edits.

## Output contract
1. Verdict to `HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md` (attack -> verdict -> evidence).
2. One comment on PR #270 with the same verdict.
3. 3-line inbox note (verdict, file path, fixes needed if any).