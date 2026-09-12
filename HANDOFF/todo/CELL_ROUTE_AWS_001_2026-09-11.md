# Ticket: CELL-ROUTE-AWS-001

Status: OPEN
Filed: 2026-09-11T06:40Z CTO seat
Related: `HANDOFF/audit/RCA_STOP_RACE_AND_CELL_STORED_2026-09-11.md`

## Problem

On CELLULAR the pending-outbox item for phone→Windows kept route=Windows
(LAN) with `dialCandidates=0`. UI showed `state=stored` (local retry), not
delivered. Bootstrap logged `no proven ledger relay candidates; network=CELLULAR`
after AWS disconnected. Message only delivered after WIFI return.

## Expected

When network is cellular and the selected route has zero dialable
candidates, fall back to a **proven AWS relay** public multiaddr
(`/ip4/18.234.62.247/tcp/9001`, pk `69805e17`) for custody/relay.

## Acceptance

1. Unit/integration test: cellular + LAN-only route → AWS candidate selected.
2. Live cell window: `inbox_receive` on AWS (or delivery ACK) without WIFI return.
3. No regression on LAN direct delivery.
4. Harness verify + Windows gates.

## Do not

- `pm clear` to reset state.
- Claim custody failure when the UI string is local outbox `stored`.
