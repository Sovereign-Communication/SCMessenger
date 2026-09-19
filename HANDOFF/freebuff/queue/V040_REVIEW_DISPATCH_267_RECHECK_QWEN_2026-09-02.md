# V040 REVIEW DISPATCH -- #267 F-DHT gate re-review at head 80197ef5 (qwen free lane)

Status: DISPATCHED 2026-09-02 (confirmatory re-review after triage commit 79b4958c)
Target: PR #267 freebuff/v040-t13-fdht-gate @ 80197ef5 (branch head incl. triage commit 79b4958c)
Model: qwen3.8-2.4t-a95b (ledger-confirmed 100%, 1M context)
Context: tmp/rev267b.diff (full PR diff at head)
Reviewer constraint: MUST NOT be the author. Draft the verdict BEFORE reading HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md or the PR body. Read-only.

## Background
First qwen review (2026-09-01) returned APPROVE with 5 findings (1 HIGH: is_locally_verified_pair comparison asymmetry; 1 MEDIUM: add_bootstrap peer_id None; 2 LOW; 1 INFO). Triage commit 79b4958c: F1 fixed (normalized eq_ignore_ascii_case compare after trim + test), F3 fixed (record_observed_peer_id_locked moved outside !locally_verified guard + test), F2/F4 documented as intentional, F5 accepted. A later hostile self-audit verified the triage. Your job: confirm the CURRENT head state holds the gate invariant.

## Attack checklist (re-review)
1. is_locally_verified_pair at head: normalized compare - can whitespace/case tricks admit an unproven pair? Is the raw arm truly unreachable and does the canonical arm still reject base58/garbage? Export-equivalent health tier? No observed_peer_ids path?
2. record_observed_peer_id_locked outside the !locally_verified guard: can an attacker now overwrite the current-binding slot or poison observed_peer_ids? Does the dial-guard/stale-identity signal still work?
3. The HIGH finding's fix + the new tests: do they assert real fail-closed state (not vacuous)?
4. No path writes a wire or legacy peer id onto a locally_verified entry; wasm inserts nothing.
5. Any NEW weakness introduced by the triage commit itself.

## Output contract
1. Verdict to HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-02.md (attack -> verdict -> evidence; plain APPROVE / REQUEST_CHANGES, max 60 lines).
2. Inbox reply 3 lines.
Do NOT pad. Do NOT post to the PR.
