# V040 REVIEW DISPATCH -- #268 + #269 (mechanical pair, qwen free plus)

Status: **DISPATCHED 2026-09-01 -- REVIEWS FILED.** Model: qwen-max (qwen3-32b non-responsive both modes -> same-tier fallback). Verdicts: `HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md` (#268 APPROVE after false-positive verification), `HANDOFF/review/V040_T14_PREEXISTING_REVIEW_QWEN_2026-09-01.md` (#269 APPROVE).
Priority: P1 -- no known defect; both already self-audited clean; this is the independent double-check
Lane: **Qwen free -- qwen3-32b** (large general tier, 766,099 remaining per docs/QWEN_QUOTA_LEDGER.md -- code review of someone else's work is large-general work)
Targets:
- PR **#268** `freebuff/v040-t13-f7-hint-widen` (14 files: routing/engine/local, swarm.rs, iron_core.rs,
  cli/server.rs, 2 retry/mycorrhizal tests, diagnostics.rs, ironcore_roundtrip.rs)
- PR **#269** `freebuff/v040-t14-preexisting-fixes` (1 file: `core/src/transport/swarm.rs`, +25/-18)
Reviewer constraint: MUST NOT be the author of either PR. Draft verdicts BEFORE reading the audit
sections already on the PRs.

## #268 -- 4-byte -> 8-byte routing-hint widen + stable peer ordering
Checklist:
1. Width integrity: full-tree sweep for leftover 4-byte constants, truncated slices, or byte-parity
   assumptions IN the hint family (core, cli, wasm, ALL tests incl. integration_*) -- no read/parse
   may expect 4 bytes of an 8-byte field, no writer may emit 4.
2. Version skew: pre-upgrade peer pushes a 4-byte hint / consumes an 8-byte one -- the parse must be
   length-checked and fail cleanly (no corruption, no panic). The PR documents this boundary -- verify
   the code matches the claim.
3. Ordering fix: `peers_for_hint` + `route_message` fully deterministic; any remaining
   HashMap-iteration-order dependence in hint selection/announcement.
4. Excluded surfaces: drift envelope and wasm mesh frames must share NO type/constant/function with
   the widened family -- confirm the exclusion is real (compiler-enforced, not just unexercised).

## #269 -- two unverified Kademlia add_address feeds removed (mDNS, DCUtR)
Checklist:
1. mDNS: nothing downstream depends on the removed address-book entry (dial must use explicit
   DialOpts from the discovery event).
2. DCUtR: no routing/score path read the removed binding; hole-punch connections still establish
   (libp2p dcutr is connection-based -- verify no kad dependency).
3. Its nine-site classification: the three SwarmCommand arms truly have zero in-tree issuers; #267's
   gate sites are untouched by THIS diff (that belongs to #267, not here).
4. No test silently asserts less (suite scan for mDNS/DCUtR handler tests).

## Method
- `cd /c/Users/SCM/Documents/GitHub/SCMessenger && gh pr diff 268 > tmp/rev268.diff && gh pr diff 269 > tmp/rev269.diff`
  (worktrees `scm-t13-f7`, `scm-t14-preexisting-fixes` on disk).
- Review-only: no code edits.

## Output contract
1. Verdicts to `HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md` and
   `HANDOFF/review/V040_T14_PREEXISTING_REVIEW_QWEN_2026-09-01.md` (attack -> verdict -> evidence).
2. One comment on each PR with the verdict.
3. One 4-line inbox note covering both.