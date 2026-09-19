# V040 REVIEW DISPATCH -- #267 re-review at triage head 80197ef5 (qwen free lane)

Status: TRIAGED 2026-09-03 (re-review after triage 79b4958c + hostile re-review fix 80197ef5) -- verdict REQUEST_CHANGES, 4 findings, all dispositioned NOT-APPLICABLE/CONFIRMED, zero code changes; disposition at HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md
Target: PR #267 freebuff/v040-t13-fdht-gate @ 80197ef5 (F-DHT gate doctrine PR; 3 files, +507/-108 vs main)
Model: qwen3.8-2.4t-a95b (ledger-confirmed 100%, 1M context)
Context: tmp/rev267b.diff (full PR diff incl. both fix commits) + docs/ARCHITECTURE_SCOPE_V040.md
Reviewer constraint: MUST NOT be the author. Draft the verdict BEFORE reading the PR body's
triage table or HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md. Read-only.

## What the fix commits changed (verify, do not trust)
Branch commits vs main: d1e61e68 (original F-DHT gate) -> 6286a262 (rework per Rule-8 verdict:
per-pair proof, close bypasses) -> 7e119375 (guard Identify path from writing wire pid onto
locally_verified bindings) -> 79b4958c (triage of the qwen findings: normalized pair compare,
bootstrap peer_id None handling, observed_peer_ids on verified entries, CLI is_known_good_key
alignment, timestamp TOCTOU disposition) -> 80197ef5 (close base58 case-fold corner in the pair
predicate: eq_ignore_ascii_case after trim; raw arm made unreachable; canonical arm still rejects
base58/garbage).

## Attack checklist (re-review)
1. Pair predicate at 80197ef5: can trim + case-insensitive matching admit an unproven
   (address, peer-id) pair that canonicalization would reject? Is the raw arm truly
   unreachable, and does the canonical arm still reject base58/garbage?
2. The six add_address sites: each still enforces its claimed behavior (pair-gated,
   per-address, wasm inserts nothing)? No path re-adds hearsay; no wire or legacy peer id
   can land on a locally_verified entry.
3. record_observed_peer_id_locked placement after the triage: can an attacker overwrite the
   current-binding slot or poison observed_peer_ids; does the dial-guard/stale-identity
   signal still work?
4. merge_shared_entries and migration writes: observed_peer_ids preserved on verified
   entries; only hygienic survivors imported.
5. mobile_bridge + CLI fail closed on None (never fail open); ledger-exchange request/response
   pass exactly the canonical address/pid forms the predicate compares.
6. Bootstrap entries with peer_id: None: can they pass the pair gate before a dial fills the
   slot, and is that the documented intent?

## Output contract
1. Verdict to HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md (attack -> verdict ->
   evidence; then plain APPROVE / REQUEST_CHANGES, max 60 lines).
2. Inbox reply 3 lines in HANDOFF/freebuff/inbox/.
Do NOT pad. Do NOT post to the PR.
