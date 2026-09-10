# P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE — LIVE REPRODUCTION + RCA (2026-08-25)

Status: Open (supersedes prior framing; now has live two-device evidence)
Discovered-live: 2026-08-25, Windows(Pixel 6a pair) @ main `0064d49a`
Severity: HIGH — blocks SHIP_PLAN D4 ("both see a delivery receipt")

## Environment

- Windows CLI daemon `0064d49a:main`, Pixel 6a debug APK from same tree,
  AWS Ubuntu relay `0064d49a` (all three nodes version-identical).
- Peering verified both directions: Pixel via tcp/lan mDNS; AWS via bootstrap.
- Control-plane healthy: `/health` 200 on all three nodes.

## Reproduction

1. Android -> Windows text message: arrives, decrypts, durable history. OK.
2. Windows -> Android POST /api/send: `{"success":true,"status":"accepted"}`,
   message_id b9b53d3e... AND 8f87625d...
3. Message VISIBLY ARRIVES on Pixel (UI dump shows full text in conversation).
4. GET /api/send/{id} remains `"status":"pending","delivered":false`
   indefinitely (>2 min, multiple attempts, with and without AWS relay present).

## Evidence

- Windows log (`%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-08-25-10`):
  - `[OK] Message delivered successfully to 12D3KooWJoW9...` (transport write OK)
  - Recurring `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts`
    targeting the Pixel peer — outbound dials fail while inbound connection
    carries traffic.
- Relay presence makes no difference -> not a routing problem.
- Android logcat contains ZERO app-level ack/receipt lines (Rust core logs do
  not surface to logcat) — observability gap blocks deeper RCA from phone side.

## RCA hypotheses (ordered)

1. H1: Android core never emits an application-level DeliveryAck on receive
   (feature missing or gated behind flag). Most likely.
2. H2: Ack is emitted but dropped because sender peer is dial-backoff
   blacklisted at ack time (Windows marked Pixel dead; ack dials fail).
3. H3: Windows receives ack but send-status store is keyed by a different
   message id than the ack references (id mismatch).

## Fix directions

- Add tracing->logcat bridge for Android Rust core (unblocks all future RCAs).
- Instrument ack emission on Android receive path; then re-test.
- If H2: dial_policy should exempt peers with active inbound connections from
  backoff death-marking.

## Related

- GAP_AUDIT_REMEDIATION_PLAN.md S3 (storage/error contracts) shares the
  observability work.
- Blocks SHIP_PLAN CP3 (D4 scoring).
