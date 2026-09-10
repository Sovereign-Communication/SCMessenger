# PR #234 Adversarial Security Review — AGENTS.md Rule 8 Gate

Status: COMPLETE — verdict **APPROVE** (no CRITICAL / HIGH / MEDIUM findings requiring
changes before merge; 1 MEDIUM and 3 LOW/INFO findings recorded for follow-up).

Date: 2026-08-28
Reviewer: Buffy (independent; did NOT author the change set — authored by the
stalled 2026-08-26..28 Claude session on `fix/android-receipt-envelope`)
Gate: `scripts/pr_scope.sh 234`

## Scope (6 merge-blocked files, `origin/main...HEAD`)

| File | Delta |
|---|---|
| `core/src/crypto/encrypt.rs` | +270 / -41 (session-divergence recovery + regression tests) |
| `core/src/transport/swarm.rs` | +408 / -15 (hint dialing, drift store-and-forward, all-nodes-are-relays) |
| `core/src/transport/hint_store.rs` | +202 (new envelope-sourced dial-candidate store) |
| `core/src/transport/dial_policy.rs` | +54 (peer-wide backoff reset) |
| `core/src/transport/behaviour.rs` | +4 (canonical identity_id derivation) |
| `core/src/transport/mod.rs` | +1 (module wiring) |

## Review methodology

Read the full diffs of all six gated files plus supporting context
(`crypto/keys.rs` for `identity_id_from_public_key_hex`, the hint-store SSRF
gate `is_dialable_multiaddr_parsed`, and the custody plumbing). Verified that
recovery/fallback paths cannot add plaintext trust, skip signature checks, or
overwrite another principal's keyed session/slot.

## Findings

### F1 [MEDIUM — ACCEPTED] Relay/custody trust widened to all identified peers
`swarm.rs:5134` removes the `agent_version.contains("relay")` gate; every
identified peer is registered as relay-capable, added to `bootstrap_capability`,
`multi_path_delivery`, and the routing gateways. The drift store-and-forward
fallback (`swarm.rs:3898`) and relay path now route through ANY available
connected peer, and the `RelayRequest` carries `recipient_identity_id` +
`intended_device_id` to that peer (metadata disclosure to formerly-non-infrac
carriers).

Not a content breach: `envelope_data` is the already-encrypted, signed message
payload — a carrier relays ciphertext it cannot read. Abuse of the carrier role
is bounded by the existing `RelayAbuseGuardrails` / per-peer token buckets
(`RELAY_PEER_BUCKET_*`) and carrier candidates are filtered with
`peer_is_blocked`. The semantics change is the explicit intended design
("All nodes are relays", per repo philosophy). **Disposition: ACCEPT for merge;
recommend re-scoping drift metadata (omit `intended_device_id` on the wire) if
carrier metadata becomes a concern.**

### F2 [LOW — INFO] Session-recovery path can be churned
`encrypt.rs` `decrypt_with_ratchet_fallback` now rebuilds the receiver session
from bootstrap fields on EVERY decrypt divergence (retry is bounded to a single
rebuild per envelope; original `first_error` is returned on a second failure, so
no unbounded loop). The `peer_id` slot is derived from the envelope's
`sender_public_key` (blake3), so a forged key can only create/replace ITS OWN
session slot, never the genuine peer's. Rebuild yields an undecryptable session
for forged material, so confidentiality/integrity hold. The V1 rebuild overwrites
the slot with a classical X25519 session — a wire-suite concern (V1 is inherently
classical; negotiation governs suite), not a downgrade of V2 protection.

### F3 [LOW — INFO] Peer-wide backoff reset can be re-armed by a flapping peer
`dial_policy.rs:reset_peer_backoff` clears dead/backoff state for every address
entry of a peer on `ConnectionEstablished`. An adverse peer that repeatedly
connects+drops could keep its own backoff cleared, enabling sustained retries.
Bounded by the concurrent-dial budget; requires a real established connection
(peer-id attestation). Monitor for dial-loop amplification.

### F4 [LOW — INFO] Hint store is process-global and never periodically pruned
`hint_store.rs` is a `OnceLock` global `HashMap`; per-peer candidates are capped
(`MAX_HINTS_PER_PEER=8`) and drop-on-read on TTL expiry, but
`prune_older_than` is `#[allow(dead_code)]` (never called). Memory growth is
bounded by peer count × 8 plus read-time TTL reaping; recommend wiring
`prune_older_than` into the existing periodic backoff-prune tick.

### Verified-good control points
- **Canonical identity (`behaviour.rs`)**: derivation routed through
  `identity_id_from_public_key_hex` = `blake3(pubkey_bytes)` — identical output,
  now single-sourced and gated on a valid Ed25519 curve point. No weakening.
- **Hint ingestion SSRF gate (`hint_store.rs`)**: every hint must pass
  `is_dialable_multiaddr_parsed(NetworkMode::Local, DnsPolicy::Reject)`;
  loopback (127.0.0.1, ::1) and cloud-metadata/link-local (169.254.169.254,
  169.254.x.x) are demonstrably rejected (unit test `restricted_hosts_never_become_hints`).
  Hints dedupe against dial-policy backoff so known-dead addrs cannot burn the
  dial budget.
- **Envelope-hint dialing (`swarm.rs`)**: candidates come only from the gated
  hint store, deduped vs. backoff, dialed with `DialOpts::peer_id` (identity
  pinned). No new attacker-reachable outbound-exfiltration surface beyond the
  existing remote-supplied-address boundary, which uses the same filter.
- **Seed dials (`swarm.rs:infer_seed_network_mode`)**: RFC1918 seeds are only
  dialed when the local node is itself on a private LAN; seeds are
  operator/ledger-configured, not remote-supplied — no SSRF.

## Verdict

**APPROVE.** All changes are safety-relevant but none introduce a new
confidentiality/integrity/authenticity break. F1 is an intentional design change
legitimately covered by existing abuse guards; F2-F4 are recorded for follow-up.
Evidence backing this verdict is the full diff of the 6 gated files cited above,
all 33 PR #234 CI checks green, and the ratchet session-recovery verdict already
recorded in `HANDOFF/plans/UNIFICATION_V2_RESULTS_PLAN.md`.

--- END FILE ---