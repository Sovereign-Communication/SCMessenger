# V040-T13 F-DHT Gate Rework -- ADVERSARIAL REVIEW (qwen free lane)

Reviewer: **qwen3.8-max-0902** (DashScope free, non-author -- independent pass)
Target: PR #267 `freebuff/v040-t13-fdht-gate` (ledger_entry.rs, swarm.rs, cli/ledger.rs)
Dispatched: 2026-09-01 by the Freebuff lane via scripts + tmp/qwen_review_dispatch.py
Substrate: full changed files on the branch + full PR diff (~79k input tokens)
Status: **APPROVE with findings** -- 5 findings (1 HIGH-severity robustness, 1 MEDIUM,
        2 LOW, 1 INFO). NOTE: findings are ANALYSIS; the Rule-8 APPROVE decision
        stays with the CEO seat (never-delegated class per docs/QWEN_QUOTA_LEDGER.md).

# V040-T13 F-DHT Gate Rework — Adversarial Review

## Finding 1: `is_locally_verified_pair` comparison asymmetry enables canonical bypass
**Severity: HIGH**
**Evidence:** `ledger_entry.rs:2376-2378`
```rust
let canonical = canonical_ledger_peer_id(peer_id, None);
bound == peer_id || canonical.as_deref() == Some(bound)
```
The stored `bound` is always written as lowercased canonical hex (via `canonical_ledger_peer_id` in `record_connection`). The first arm (`bound == peer_id`) compares the stored lowercase hex against the raw caller-supplied string. If a caller passes an uppercase hex or base58 PeerId that doesn't match the second arm's canonical derivation (e.g., due to a non-Ed25519 key where `canonical` returns `None`), the first arm fails closed correctly. However, if `peer_id` is passed as uppercase hex, `bound == peer_id` fails but `canonical` normalizes it—this is fine. The real issue: `bound` is compared raw against `peer_id` without normalizing `bound` first. Since `record_connection` stores lowercase, any caller passing mixed-case hex hits `bound == peer_id` = false, falls through to `canonical`, which works. But this dual-path comparison is fragile and undocumented.
**Fix:** Normalize both sides: `bound.to_lowercase() == peer_id.trim().to_lowercase() || canonical.as_deref() == Some(bound)`.

## Finding 2: Bootstrap entries with `peer_id: None` are permanently unverifiable by pair
**Severity: MEDIUM**
**Evidence:** `ledger_entry.rs:2108-2112` (`add_bootstrap` new-entry path sets `peer_id: None`), `ledger_entry.rs:2373` (`is_locally_verified_pair` requires `entry.peer_id.as_deref()` to be `Some`).
A bootstrap entry created via `add_bootstrap` without a known peer_id has `locally_verified: true` and `is_bootstrap: true`, but `is_locally_verified_pair` will always return `false` for it because `bound` is `None`. This means bootstrap addresses can never satisfy the DHT gate until a dial completes and `record_connection` fills the slot. This is documented as an "accepted cost" but contradicts the invariant that bootstraps are immediately usable relay roots.
**Fix:** Either require `add_bootstrap` callers to supply a peer_id, or have `is_locally_verified_pair` return `true` for `is_bootstrap` entries regardless of binding (with a separate address-only check).

## Finding 3: `merge_shared_entries` skips `observed_peer_ids` recording on verified entries
**Severity: LOW**
**Evidence:** `ledger_entry.rs:2037-2041`
```rust
if !entry.locally_verified {
    entry.peer_id.get_or_insert_with(|| pid.to_string());
    record_observed_peer_id_locked(entry, pid);
}
```
When an entry is `locally_verified`, neither the binding nor `observed_peer_ids` is updated from wire data. The PR body claims `observed_peer_ids` should still be recorded on verified entries for stale-identity signaling (matching `record_identified_peer`'s behavior at line 1868). The wire-merge path omits this, creating an inconsistency between Identify-heard and ledger-exchange-heard observations.
**Fix:** Move `record_observed_peer_id_locked(entry, pid)` outside the `!entry.locally_verified` guard in `merge_shared_entries`, matching `record_identified_peer`.

## Finding 4: CLI `is_known_good_key` uses stricter failure gate than `is_locally_verified_pair`
**Severity: LOW**
**Evidence:** `cli/src/ledger.rs:374-378` requires `e.failure_count == 0`; `ledger_entry.rs:2367` requires `e.failure_count < LEDGER_DEAD_FAILURE_THRESHOLD` (i.e., `< 3`).
The CLI dial scheduler's `is_known_good_key` rejects entries with any failures, while the DHT gate accepts entries with up to 2 failures. A peer dialed successfully once then failing twice is DHT-publishable but not CLI-dialable-as-known-good. This isn't a security hole but creates a semantic split where the DHT advertises peers the local dialer won't prioritize.
**Fix:** Align `is_known_good_key` to use `< LEDGER_DEAD_FAILURE_THRESHOLD` or document the intentional divergence.

## Finding 5: `clamp_wire_last_seen_ms` TOCTOU on `current_timestamp()`
**Severity: INFO**
**Evidence:** `ledger_entry.rs:30-32`
Two calls to `current_timestamp()` within one merge operation (one for the clamp, one implicitly via other operations) can yield different values. An attacker sending `last_seen = now + 1` could occasionally land 1ms above local time if the clock ticks between the two reads. Impact is negligible (ranking tie-break only, never verification).
**Fix:** Accept as-is; the threat model doesn't warrant sub-millisecond precision.

## Attacks tried
1. Address-keyed lookup via `entry_for_multiaddr` — confirmed, no wire-supplied peer-id keying.
2. All six add_address sites audited — wasm inserts nothing, native gates all three Kademlia feeds.
3. Bypass B (`!locally_verified` guard) — confirmed on `record_identified_peer`, `merge_shared_entries`, `import_legacy_cli_entries`. Found Finding 3 inconsistency.
4. Mobile/CLI fail-closed — `ledger_verified_pair` returns `false` on `None` core handle.
5. Ledger-exchange stripping — `strip_peer_id_component` applied consistently before comparison.

APPROVE

## Triage disposition (2026-09-01, commit `79b4958c`, PR body updated)

All five findings verified against the branch tree by the lane:

- **F1 (HIGH)** -- TRUE as robustness, not a bypass. The raw arm was dead for
  the live caller (libp2p PeerId.to_string() is base58; stored bound is
  canonical lowercase hex); the canonical arm handled every wire form and both
  arms fail closed. FIXED: raw arm hardened to
  `eq_ignore_ascii_case(peer_id.trim())` + rationale comment. Test:
  `pair_gate_matches_mixed_case_hex_and_rejects_others`.
- **F2 (MEDIUM)** -- TRUE as a stated limitation; behavior is the doctrine
  (operator bootstrap = hearsay, pair-publishable only after a dial proves it).
  Exemption rejected (would reopen the F-DHT gate for hearsay). DOCUMENTED in
  `add_bootstrap`.
- **F3 (LOW)** -- TRUE; wire-merge omitted observed_peer_ids on verified
  entries, inconsistent with `record_identified_peer` and weakening the P0
  stale-identity dial-guard signal. FIXED: recording moved outside the
  `!locally_verified` guard; binding slot stays dial-proven only. Test:
  `wire_merge_records_observed_pid_on_verified_entry`.
- **F4 (LOW)** -- TRUE as a divergence; INTENTIONAL (dial scheduler avoids
  flaky peers vs the gate's publishability-until-dead). DOCUMENTED, no change.
- **F5 (INFO)** -- accepted as-is (reviewer concurred).

Gates on the triaged tree: fmt clean, clippy (documented gate) 0, core+cli
--all-targets clean, wasm32 proof OK, core 1402/0 (+2), cli 82/0.
