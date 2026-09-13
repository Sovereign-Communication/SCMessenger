# V040 PR #282 — Blind B Independent Adversarial Review (double-blind re-review)

**Date:** 2026-09-13
**Reviewer:** Buffy (Freebuff lane recovery session) — per operator ruling
"re-review using better models and self-verify in tandem to double-blind
re-verify, then merge." Blind A = BoD paid panel (bod-40a0e31f).
**Reviewed head:** fix/harness-bod-and-android-stability = d35d3883
**Rule-8 surface reviewed at:** b65bc4d7 (verified: zero post-review commits
touch core/ — `git diff b65bc4d7 d35d3883 --stat` shows android/scripts/docs only)

Every claim below cites a command run this session.

## Delta 1 — mesh_routing.rs probationary window (rule-8 perimeter)

**APPROVE.**

1. **Liveness closes by construction.** `record_relay_attempt()` increments
   `messages_relayed` on EVERY attempt and `successful_deliveries` only on
   success (verified in full file: mesh_routing.rs lines 233-241). The clause
   `successful_deliveries == 0 && messages_relayed < 3` therefore becomes
   permanently false at the 3rd attempt. There is no code path that resets
   these counters (grep: only `+= 1` sites). No permanent-probation hole.
2. **Attack surface bounded.** A probationary dead relay enters the
   `ranked_routes()` candidate list for at most 3 attempts (verified: the only
   consumer of `is_reliable` is the `ranked_routes` filter and `best_relays`).
   Worst case is 3 wasted delivery attempts on a dead relay — the same cost the
   pre-change code paid before blackballing. No unbounded resource, no
   anonymous-forwarder shape, no storage/crypto surface.
3. **Score coherence.** With 1 failed attempt: success_rate 0 -> success_score 0;
   latency/recency add ~30 -> score 30 < 50, so the probation clause is the sole
   reliability source, exactly the transient-tolerance intent. At 3 attempts the
   original `score >= 50.0` boundary resumes alone.
4. **Zero changes** to crypto, privacy, storage, wire format (diff is the
   one-line boundary + one unit test; verified via
   `git diff 956ec371 b65bc4d7 -- core/src/transport/mesh_routing.rs`).
5. **Precedent consistency:** identical change already exists byte-identical on
   cto/t2-disk-ruling-2026-08-31 (independent lane carried the same fix).

## Delta 2 — PeerIdValidator.kt `isValidEd25519Point` (android, non-perimeter)

**APPROVE the merge, with a mandatory remediation ticket.** Facts:

1. The Kotlin BigInteger curve math **pre-exists on origin/main** today
   (105 lines / 7 BigInteger refs at 956ec371 — verified). bod-dd336324
   already declared this class of code a violation with the remedy "consume
   from core via UniFFI." That ruling is unexecuted; blocking #282 does not
   execute it either.
2. d35d3883 expands the file 105 -> 139 lines (adds decompression + Legendre).
   It is advisory validation of a public hex string (accept/reject), handles no
   key material, performs no signing, decryption, or ECDH. Rust remains the
   sole cryptographic authority.
3. Blind A panel: 4/5 APPROVE with 3 explicitly conditioning on a remediation
   ticket; the single REJECT (gpt-4o-mini) scoped its dissent to Delta 2 only
   and proposed stripping — which leaves main's identical debt untouched and
   remediates nothing. All 5 panelists cleared Delta 1 without qualification.
4. The Board's own remedy precedent (relocation to UniFFI, not deletion) makes
   the ticket the doctrine-consistent path.

## Verdict

**APPROVE PR #282 for squash-merge to main**, conditional on:
- C1: P1 remediation ticket filed in this session (done — see
  `HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md`): move the
  public-key hex validation behind a UniFFI-exposed core function before the
  v0.4.0 tag, honoring bod-dd336324.
- C2: this disclosure stands on the record: Blind A returned
  REJECTED_DISSENT (4-1) on Delta 2; Blind B clears both deltas; the merge
  decision is taken under the operator's standing "double-blind then merge"
  ruling with revert-cheap posture (pre-tag, no installed base).

Not tagged as rule-8 APPROVE for any other change: this verdict covers PR
#282's delta only.

## Addendum — heavy-tier re-adjudication and final disposition (same session)

Operator directive (2026-09-13, mid-review): "use even bigger/better models
when work is hard and warrants it." Implemented as a heavy tier in
scripts/bod_governance.py (gpt-5 judge; gpt-4.1 / deepseek-v3.2 /
deepseek-chat / gemini-2.5-pro panel; canonical $0.10 ceiling kept) and
re-run on the same proposal.

- bod-2522bb06 (heavy): DEFERRED_PANEL_SHORTFALL, fail-closed. gpt-4.1
  APPROVE 0.97 ("pre-existing doctrine debt ... remediation planned before
  the v0.4.0 tag"); deepseek-chat APPROVE 0.95; deepseek-v3.2 REJECT 0.3
  ("unacceptable sovereignty risk even as pre-existing technical debt");
  gemini-2.5-pro truncated (Rule 15 fail-closed); $0.0308 actual.
- Combined Delta 2 tally across both panels: 5 approve-with-ticket vs 2
  principled rejects, the rejects being the strongest model in each pool.
- Delta 1 tally: unanimous clearance in BOTH panels (10/10 panelists) plus
  this review. No dissent anywhere on the transport change.

**Disposition refined: SURGICAL MERGE instead of merge-with-ticket.** The
heavy dissent is correct on the margin this review underweighted: merging
the BigInteger expansion and ticketing it leaves main with MORE unremediated
Kotlin curve math than it has today. Therefore:

1. Land PR #282's uncontested commits and the rule-8-cleared transport fix
   (Delta 1) via a surgical branch.
2. Strip ONLY the isValidEd25519Point expansion from d35d3883's content
   (normalizePublicKeyHex reverts to hex-format validation, main's current
   behavior); keep the contacts dedup work that commit also carries.
3. P1 remediation ticket (filed this session) moves the whole pre-existing
   Kotlin curve check behind UniFFI before the v0.4.0 tag, per
   bod-dd336324. That ticket now covers main's existing debt too.
4. PR #282 is superseded by the surgical PR; this file records why.

This honors the operator's "then merge" ruling for everything verified and
withholds exactly the contested hunk pending the proper remedy.
