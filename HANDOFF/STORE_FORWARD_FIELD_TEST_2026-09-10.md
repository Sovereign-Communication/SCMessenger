# AWS store/forward field-test runbook (operator, out of house)

Candidate: `b5048dd4` (Android APK SHA256 `10C43A79…`)
Fleet: Windows CLI + AWS relay + Pixel 6a

## Before you leave (on home Wi‑Fi)

1. Open SCMessenger on the Pixel. Complete onboarding / create identity if prompted.
2. Confirm mesh starts (Dashboard shows peers). `peersDiscovered` should reach 2
   (Windows + AWS) while you are home.
3. Leave mesh ON. Do not force-stop the app.

## While away (cellular / no home LAN)

1. Windows and AWS stay up on home/AWS (they do not move).
2. From Windows, send a message to the phone contact:
   ```
   curl -s -X POST http://127.0.0.1:9876/api/send \
     -H 'Content-Type: application/json' \
     -d '{"recipient":"<PHONE_PUBLIC_KEY_HEX>","message":"store-forward-away-1"}'
   ```
   Or use the CLI/API you already use for sends.
3. AWS should accept custody (`inbox_receive` or custody logs on AWS).
4. When you return / phone reconnects over cellular or Wi‑Fi, the message should
   arrive on the phone and a delivery ACK should return.

## Success criteria (from logs, not UI alone)

| Node | Log evidence |
|---|---|
| Windows | `[TRANSPORT-LANE]` outbound to AWS; `Message delivered` or outbox flush after phone returns |
| AWS | `inbox_receive` of the message while phone offline; later `Sending delivery ACK` to phone |
| Phone | inbox shows the message after reconnect; no "Message Store Unavailable" |

## Identity triad (fill after install)

After onboarding, grab from Windows once mesh learns the phone:

```
curl -s "http://127.0.0.1:9876/api/peer-resolve?input=<id-from-win-peers>"
```

Or from Windows `/api/peers` → `triad` object.

Record:

| Field | Value |
|---|---|
| Phone libp2p PeerID | (from Windows peers) |
| Phone public_key | (triad.public_key_hex) |
| Phone identity_id | (triad.identity_id) |

## If mesh fails to start

- "Message Store Unavailable" after toggle: force-stop app, reopen once.
  Do not reinstall unless store stays degraded.
- Ensure only ONE instance (Settings → Apps → SCMessenger → Force stop, then open).

## Windows ↔ AWS (already verified)

Both at `e8c8f52b` / `b5048dd4` tree. Bidirectional message+receipt proven
(`5dc172ad`, `51711686`). They carry store/forward while the phone is away.
