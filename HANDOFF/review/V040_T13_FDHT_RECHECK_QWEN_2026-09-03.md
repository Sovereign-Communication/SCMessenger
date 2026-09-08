# PR #267 re-review (round 2) — qwen verdict + disposition record

- Model: qwen3.8-2.4t-a95b (ledger-confirmed 100%, 1M context)
- Reviewed head: 80197ef5 (triage 79b4958c + hostile-re-review fix 80197ef5)
- Verdict: REQUEST_CHANGES — 4 findings; 1 Medium, 2 Low, 1 Informational
- Disposition: 0 code changes required. All three actionable-looking findings verified
  against the committed tree and dispositioned NOT-APPLICABLE; Informational confirmed.

## Per-finding disposition (each verified against origin/freebuff/v040-t13-fdht-gate)
| # | Finding | Verdict | Evidence |
|---|---|---|---|
| 1 MED | Case-fold arm not restricted to hex bindings | NOT-APPLICABLE | hex_casefold fires only when `trimmed.len()==64 && eq_ignore_ascii_case(bound)`. Stored bounds are exclusively 64-char hex (Ed25519, canonicalized at record_connection) or real base58 PeerIds (46-52 chars; 64 base58 chars decode to ~47 bytes, not a valid multihash). A 64-char non-hex bound cannot exist on a verified entry (peer_id writes on verified entries come only from completed dials; all wire/legacy writes are `!locally_verified`-gated). Test `pair_gate_rejects_base58_case_variants` already pins base58 case-variant fail-closed. The "raw arm unreachable" framing is a misread: the comment documents the raw arm as LOAD-BEARING for un-canonicalizable base58 bounds. |
| 2 LOW | Migration skips observed_peer_ids on verified entries | NOT-APPLICABLE | ledger_entry.rs:2253-2261 — legacy pid/observed import gated on `!e.locally_verified`. Fail-closed and uniform with Bypass-B doctrine (legacy pid never written onto verified entries): pre-doctrine legacy observations are unvetted noise; importing them would poison the signal. Verified entries repopulate observed_peer_ids from live wire observations (record_observed_peer_id_locked at 1874/2051) post-migration, so the dial-guard stale-identity diagnostic is intact. |
| 3 LOW | observed_peer_ids wire-writable on verified entries | NOT-APPLICABLE | Already hardened exactly as the reviewer suggests: dedupe (any() check) + cap (MAX_OBSERVED_PEER_IDS_PER_ENTRY, oldest-drop) in record_observed_peer_id_locked (ledger_entry.rs:528-542). Never touches the current-binding slot on verified entries (peer_id writes are !locally_verified-gated). F-DHT predicate never consults the field (is_locally_verified_pair reads only peer_id/success/failure). It is the documented P0 stale-identity signal. Reviewer concedes "not a disclosure bypass". |
| 4 INFO | Main bypasses closed; bootstrap None fails closed | CONFIRMED | Kademlia inserts pair-gated (swarm.rs 4549-4552/4698-4705/5188-5191); ledger_verified_pair fails closed (2707-2712); wasm Identify inserts nothing (7910-7920); AddKadAddress/RegisterEndpoint arms removed; bootstrap entries with peer_id None cannot pass (predicate requires current binding + success_count>0). |

## Verdict
No defects found in 80197ef5. Findings 1-3 dispositioned with evidence; finding 4 confirms the
triage + fix commits achieved their stated contracts. PR #267 stands at 80197ef5; no re-run of
gates was required (zero-diff disposition).
