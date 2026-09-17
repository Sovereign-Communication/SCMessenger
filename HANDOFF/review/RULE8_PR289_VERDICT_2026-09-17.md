# Rule-8 adversarial review — PR #289 (Kotlin crypto-consolidation)

Status: CLOSED — APPROVE-WITH-NOTES
Date: 2026-09-17
Reviewer: harness structured-claims panel (3 independent non-author models) x2 runs,
plus controller code-evidence disposition. Author excluded from the panel by construction.

Scope note: PR #289's diff consolidates three duplicated Ed25519 peer-id validators in
`android/` Kotlin into `PeerIdValidator` + tests, and fixes workflow SDK-setup pins.
Kotlin-side crypto-adjacent consolidation; the Rust `core/src/crypto` perimeter itself is
not touched by this diff. Reviewed at the stricter Rule-8 standard anyway because it moves
validation logic that guards peer identity material.

Method: `harness.cli verify` structured-claims mode (candidate-defect claims manifest +
verbatim diff source window, per-claim panel votes, deterministic convergence tally).
Paid pool: deepseek-v4-flash / gpt-4o-mini / gemini-3.8-flash, 2048-token votes,
reasoning-effort off. Two runs (task-ids rule8-289a, rule8-289b) to test vote stability.

## Claims and tallies

- C1: lenient `isLikelyPeerId` copy retained that bypasses the strict validator —
  NOT REAL (0R/2NR run a; 0R/2NR run b). No lenient copy survives the diff.
- C2: cosmetic key validation dropped from the #295-relayed helper — NOT REAL.
  The removed clause `!("all".len() == 64)` was trivially true dead code; the retained
  `len == 64 && hex` check is strictly equivalent for the inputs it guarded.
- C3: fallback path reintroduces the lenient form — NOT REAL (0R/2NR both runs).
- C4: cosmetic test-name inaccuracy — 1R/1NR split in BOTH runs (never converged);
  controller disposition: NOT REAL — the test method names accurately describe the
  behavior each test exercises (verified against the diff's test bodies).
- C5: unhandled curve-point acceptance divergence — NOT REAL (0R/2NR both runs).
  All three consolidated copies share the same strict algorithm.

## Disposition

No claim reached the 2/3 converged-real bar in either run. The two split claims were
dispositioned against the actual diff text (evidence contract: panel votes are raw
material, code evidence decides). Verdict: APPROVE-WITH-NOTES.

Notes for the record:
- Third-voter instability is a known harness constraint (see PR #297 verdict); two-run
  repetition was used as the stability check here instead of a fourth model.
- Cumulative authorized paid spend for this session's reviews: ~$0.020 of $0.05.

Artifacts: `C:\Users\SCM\Documents\GitHub\Harness\audits\scmessenger\_runs\` (rule8-289a,
rule8-289b), claims/source windows under repo-local `tmp/rule8_pr289_*` (untracked).
