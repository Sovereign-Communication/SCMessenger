# CORE: prune stale pre-D10 double-circuit addresses from dial candidate sets

Status: OPEN — filed 2026-09-10T03:30Z by the CTO seat.
Priority: MEDIUM (harmless today — fallback works — but noisy and wasteful).
Perimeter: `core/src/transport/` (rule-8 gated) — do NOT implement without a
review packet; this ticket is the evidence + scope definition only.
Owner: CTO seat or dispatched worker, after the D10 verdict lands (one gated
change at a time on this perimeter).

## Live evidence (2026-09-10 passive windows, tmp/cto/PASSIVE_FINAL_20260910T020553Z/)

The phone's dial book still carries pre-D10 poison shapes and periodically
burns dial attempts on them:

- `Failed to dial /ip4/192.168.0.222/tcp/443/p2p/<Windows>/p2p-circuit/p2p/<AWS>/p2p-circuit/p2p/<Windows>`
  (double-circuit through a base that is itself a circuit route) — repeated
  in windows W2/W3.
- The same family appears in address snapshots shared via identify, so the
  stale shapes propagate peer-to-peer even after the source node is fixed.

## Root (already fixed at the source by D10, but not reaped)

D10 (`7ff317f0`) stopped NEW poison listeners/reservations; it did not purge
historically accumulated candidates from ledgers/peers.json/address books on
peers that learned them before the fix.

## Scope for the fix (when scheduled)

1. On identify/address-learning: reject (or flag-and-expire) candidate
   multiaddrs containing more than one `/p2p-circuit` segment — reuse
   `is_canonical_reservation_addr` semantics; consider a shared helper
   `is_plausible_dial_target(addr)`.
2. On ledger load / dial-candidate construction: same filter before enqueue.
3. Expiry: a candidate that has failed N consecutive dials with a
   structural-reject reason should be dropped, not merely backed off.
4. Regression tests: double-circuit candidate rejected at learning time;
   already-persisted double-circuit candidate pruned at load; valid
   single-circuit candidates unaffected.

## Non-goals

- No behavior change for direct or canonical single-circuit candidates.
- No ledger format change required (filtering is read-time + learn-time).
