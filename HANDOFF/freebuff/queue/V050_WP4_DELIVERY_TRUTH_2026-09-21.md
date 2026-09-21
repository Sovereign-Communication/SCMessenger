# V050-WP4 — Delivery truth: receipts, honest delivered, wedge

Status: OPEN (filed 2026-09-21 CTO)
Priority: P0 — umbrella WP4
Lane: Freebuff
Authority: `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` §2 WP4
Refs (implement these; do not invent new RCs):
- `todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md`
- `todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md`
- `todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md`
- Freebuff `V040_T_WATCHDOG_POSITIVE_TEST.md`

## Implement

1. Receipt convergence: receiver app verification is what sender "delivered"
   means; transport ACK alone must not claim user-visible delivered.
2. Crypto-fail path: signature/verify failures never report delivered=true.
3. Windows wedge: watchdog positive test (healthy-quiet must not exit); live
   wedge remains observable.

## Acceptance

- [ ] Unit/integration tests for receipt vs transport-ACK distinction
- [ ] Watchdog positive test green on Windows
- [ ] JEV pack: `instruction_matches` + canonical rows pass
- [ ] WP5 live 3-node receipts after WP1–3 land

## Review gate

Orchestrator + tests. Rule-8 if core dispatch/status changes in gated dirs.

## Rules

No emojis. Evidence contract. Worktree. No self-merge.
