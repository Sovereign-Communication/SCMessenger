<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# D2 / D3 diagnosis -- seed re-dial and self-addressed sends on `origin/main`

Owner of this diagnosis: SCMessenger. Read-only, 2026-09-25, against
`origin/main` `d1c4a173` using `git grep` and `git show`. The D2 and D3
tickets themselves are part of PR #372 and land with it. Implementation is the
main lane's.

## D2 -- the seed sweep never re-dials a missing seed-tier peer

### Confirmed mechanism

- `cli/src/seed_dial.rs` line 45: `sweep_decision(peer_count, candidates)`
  returns `SweepAction::WatchOnly` whenever `peer_count > 0`; `Dial` only when
  `peer_count == 0 && candidates > 0`. The unit tests at lines 114-131 pin
  exactly this shape.
- `cli/src/seed_dial.rs` line 63: `sweep_once` calls `swarm.get_peers()`; on
  `WatchOnly` it logs "connected; re-check in 120s" and returns 120.
- `cli/src/main.rs` line 2469-2476: the boot task loops `sweep_once` and sleeps
  the returned delay forever. There is no other caller
  (`git grep 'sweep_once'` returns only this loop and the definition).
- Consequence: once ANY peer is attached, a proven seed-tier peer that is down
  is never re-dialed by the sweep. The only other dialer is the
  `DialScheduler` (line 2480), which dials ledger peers "that pass backoff" --
  a peer marked dead by the backoff table is excluded until its backoff
  expires, and `config list` exposes no re-dial key (per the ticket's own
  evidence).

### Fix shape (ticket, unchanged)

Re-dial a proven seed-tier peer that is missing or marked dead regardless of
`peer_count`, with a bounded cadence and a cap; keep the exponential backoff
for empty candidate lists. No new config key unless the fix needs one (then
it lands in `scm config list` with a test).

## D3 -- a self-addressed send is accepted and lands in the outbox

### Confirmed mechanism

- `cli/src/api.rs` line 816: `authorize_api_recipient` checks ONLY
  `core.is_peer_blocked(recipient.identity_id, None)` (403 if blocked). There
  is no own-identity check; `git grep` for `own` / `self` in `api.rs` finds
  nothing on the send path.
- `cli/src/api.rs` line 849 `handle_send_message`: `resolve_api_recipient`
  resolves the recipient (by id, name, or peer id), then
  `prepare_message_with_id`, then a history row with `delivered: false`, then
  `swarm_handle.send_message(recipient.peer_id, ...)`. A self-addressed send
  targets the node's own peer id, which is not a connected peer, so the send
  fails; the BLE fallback is attempted; if the event loop is alive the handler
  returns `202 retrying` and the outbox entry is left for a retry that will
  also fail. The node then dials itself, fails, and the dial-backoff table
  marks its own peer id dead (the ticket's evidence).
- `core/src/store/outbox.rs`: no own-identity or self-recipient check (the
  `self` matches there are `self` receivers and a doc comment). `next_retry_at`
  exists on entries (line 21) and is set on failure, but there is no periodic
  sweep on main that retries due entries (see the outbox retry ticket and
  #364's core half).
- `cli/src/api.rs`: `git grep 'outbox/cancel'` finds no route; `DELETE
  /api/contacts` is not present on main either. There is no queue-cancel
  path.
- Note: `core/src/iron_core.rs` DOES have a hydrated-self guard for the
  LEDGER (`reopened_core_arms_ledger_self_filter_without_initialize_identity`,
  line ~5090: the node refuses to record its own identity as a peer). The
  equivalent guard is missing on the SEND path, which is the asymmetry D3
  names.

### Fix shape (ticket, three independent parts)

1. Enqueue guard: reject `recipient == own identity` in
   `handle_send_message` with a 4xx and a clear error string (compare
   `recipient.public_key` / `identity_id` with `core.public_key_hex()` /
   `identity_id()`).
2. Drain guard: at outbox drain, drop entries whose recipient resolves to the
   local identity (log + counter, no retry).
3. Queue-cancel route: `POST /api/outbox/cancel` keyed by message_id.

### Dependency

Part 2's "outbox drain" is the periodic sweep landing in #364's core half;
until that lands, a self-addressed entry that predates the guard is never
retried and never dropped. Land the enqueue guard first (it is a `cli/`
change, no Rule-8 by path), then the drain guard with the sweep, then the
cancel route.
