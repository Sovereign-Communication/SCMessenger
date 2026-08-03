# iOS log-bundle analysis (redacted)

This companion is intentionally compact and contains no raw log lines. The
private capture retains the full identifiers and redaction map for a controlled
join when the fresh run is available.

## Findings

- BLE central sessions reached connected/subscribed states, but the capture did
  not show a confirmed receipt.
- Route selection is noisy: `no_route_candidates`, `Peer not connected`, and
  dial failures dominate the non-BLE path.
- No exact iOS decrypt error or identity-registration error was emitted in the
  available window. The next run must preserve the exact Rust error text.
- Source review still shows multiple route aliases and an untyped 64-hex
  ambiguity. Encryption must use the canonical Ed25519 public key; route IDs
  must remain aliases only.

## Recommendations to Windows/Qwen

1. Add a deterministic, value-free `identity_kind` field to the Android and iOS
   core/transport events before the next paired run.
2. Join registration, decrypt, route, send, receive, and receipt events by a
   private message UUID and a stable redacted peer label.
3. Run the same test script on the unlocked fresh iPhone, Android, macOS CLI,
   Windows CLI, and cloud node; record the UTC start/end times in both bundles.
4. Treat `central_send_false` as a failed send until a matching receiver and
   receipt event exists.
5. Keep the libp2p connection-close panic fix out of the parity conclusion, but
   land it before broad soak testing because node churn can mask messaging
   failures.
