# V040-T13 F7 (hint widen) + V040-T14 pre-existing feeds -- REVIEW (qwen free lane)

Reviewer: **qwen-max** (DashScope free, non-author; qwen3-32b non-responsive
both modes -> same-tier fallback). Targets: PR #268 `freebuff/v040-t13-f7-hint-widen`,
PR #269 `freebuff/v040-t14-preexisting-fixes`. Dispatched 2026-09-01 via
tmp/qwen_review_dispatch.py (full diffs + function excerpts).

## Lane verification addendum (2026-09-01)
- **#268 HIGH finding is a FALSE POSITIVE.** The reviewer claims iron_core.rs:820/1026
  still use `unwrap_or([0u8; 4])`. Verified on the branch: the actual sites
  (iron_core.rs:825 and :1031) read `unwrap_or([0u8; 8])`; a full grep of the file
  shows zero `[0u8; 4]` -- the widened family is 8-byte-consistent everywhere.
  The version-skew MEDIUM claim is likewise stale: the diff already carries the
  length-checked `try_from` parse (iron_core.rs:2729).
- **#268 disposition after verification: APPROVE** (matches the earlier self-audit);
  the reviewer's remaining items are comment-only (document the sort contract,
  confirm exclusions) -- optional.
- **#269: APPROVE** as filed (4/4 checks clean; comment-only suggestions).
- Both: ANALYSIS only; the Rule-8 APPROVE decisions stay with the CEO seat.

### #268 -- 4-byte -> 8-byte routing-hint widen + stable peer ordering

**1. Width integrity:**
- **Severity:** High
- **Evidence:** `core/src/iron_core.rs:820` and `core/src/iron_core.rs:1026` still use `unwrap_or([0u8; 4])` instead of `unwrap_or([0; 8])`.
- **Fix:** Change `unwrap_or([0u8; 4])` to `unwrap_or([0; 8])` in these lines.

**2. Version skew:**
- **Severity:** Medium
- **Evidence:** The PR documentation claims that the parse must be length-checked and fail cleanly, but there is no explicit check for the length of the hint in `core/src/iron_core.rs:820` and `core/src/iron_core.rs:1026`.
- **Fix:** Add a length check before calling `try_into()` and handle the error gracefully.

**3. Ordering fix:**
- **Severity:** Low
- **Evidence:** The `peers_for_hint` function in `core/src/routing/local.rs:205` now sorts by reliability, but the `active_peers` function in `core/src/routing/local.rs:229` uses the same sorting logic. This could lead to confusion if the contract changes.
- **Fix:** Document the sorting behavior clearly in both functions and ensure consistency.

**4. Excluded surfaces:**
- **Severity:** Low
- **Evidence:** No evidence of drift envelope or wasm mesh frames sharing types/constants/functions with the widened family, but this needs a thorough review.
- **Fix:** Conduct a full codebase scan to confirm the exclusion and add a comment in the relevant files to document the separation.

**Verdict: REQUEST_CHANGES**

### #269 -- two unverified Kademlia add_address feeds removed (mDNS, DCUtR)

**1. mDNS:**
- **Severity:** Low
- **Evidence:** The mDNS handler in `core/src/transport/swarm.rs:5012` no longer calls `add_address`, and there are no downstream dependencies on the removed address-book entry.
- **Fix:** None needed, but add a comment to explain the removal for future reference.

**2. DCUtR:**
- **Severity:** Low
- **Evidence:** The DCUtR handler in `core/src/transport/swarm.rs:4885` no longer calls `add_address`, and there are no routing/score paths reading the removed binding.
- **Fix:** None needed, but add a comment to explain the removal for future reference.

**3. Its nine-site classification:**
- **Severity:** Low
- **Evidence:** The three `SwarmCommand` arms in `core/src/transport/swarm.rs` have zero in-tree issuers, and #267's gate sites are untouched.
- **Fix:** None needed, but add a comment to confirm the classification.

**4. No test silently asserts less:**
- **Severity:** Low
- **Evidence:** The test suite does not contain any mDNS/DCUtR handler tests that assert less.
- **Fix:** Add a comment in the test suite to document the removal and ensure future tests do not inadvertently re-add the functionality.

**Verdict: APPROVE**