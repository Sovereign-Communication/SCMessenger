# RCA + IMPLEMENTATION PLAN â€” DELIVERY ACKS DO NOT CONVERGE (Windowsâ†’Android) (2026-08-25)

Status: Ready for implementation
Supersedes framing in: P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md
Evidence base: main `0064d49a` live rig + full code trace (this document)

---

## 1. Root cause verdict

**H1â€² (variant of H1): the ack IS emitted by Android, but it is malformed on the wire â€”
Android sends a bare, unsigned receipt-JSON payload instead of an encrypted
`MessageType::Receipt` envelope. The sender's ingress rejects it as an undecodable
envelope and silently drops it.**

The original hypotheses resolve as follows:

| Hypothesis | Verdict | Evidence |
|---|---|---|
| H1: Android never emits an application-level DeliveryAck | **Half-true (root cause).** Android *does* call `sendDeliveryReceiptAsync()` on every received message, but encodes the receipt with `uniffi.api.encodeReceipt()` (bare JSON) instead of `ironCore.prepareReceipt()` (signed+encrypted envelope). | `android/.../MeshRepository.kt:2481-2630` vs working paths below |
| H2: Ack blocked by dial-backoff death-marking | **Not blocking this flow.** The Pixel holds an active *inbound* connection to Windows (that's how messages arrive); its outbound ack reuses it. Backoff death-marking is real (see Â§5) but affects Windowsâ†’Pixel dials, not Pixelâ†’Windows acks. | `core/src/transport/dial_policy.rs`, live log |
| H3: message-id mismatch between ack and status store | **Ruled out.** The ack's `messageId` is `msg.id` from core decryption â€” the exact UUID the sender's `prepare_message_with_id` generated. | trace in Â§2 |

### Why the ack is dropped (exact mechanics)

1. Android `sendDeliveryReceiptAsync` (`MeshRepository.kt:2481`) does:
   ```kotlin
   val receiptBytes = uniffi.api.encodeReceipt(receipt)   // â† bare JSON bytes
   attemptDirectSwarmDelivery(..., encryptedData = receiptBytes, ...)
   ```
2. These raw JSON bytes arrive at Windows as `SwarmEvent::MessageReceived { envelope_data }`.
3. `IronCore::receive_message` (`core/src/iron_core.rs:3314`) requires either a
   Drift-signed envelope (`:3328`), a V1/V2 signed wire envelope (`:3339`), or â€” as a
   final fallback â€” an *unsigned* wire envelope which is **rejected at ingress**
   (`:3358-3388`, "Unsigned wire envelope rejected"). Bare JSON matches nothing â†’
   `Err(IronCoreError::CryptoError)`.
4. CLI event loop swallows it: `if let Ok(msg) = core_rx.receive_message(...)`
   (`cli/src/main.rs:2498`). No `MessageType::Receipt` branch runs.
5. Therefore the ONLY pendingâ†’delivered transition trigger â€”
   `history_rx.mark_delivered(receipt.message_id)` (`cli/src/main.rs:2637`,
   `core/src/store/history.rs:292`) â€” never fires. `GET /api/send/{id}`
   (`cli/src/api.rs:837-864`) reads `record.delivered == false` forever.
6. The misleading `[OK] Message delivered successfully to 12D3KooW...` log is
   transport-layer write success only (`core/src/transport/swarm.rs:3700`) â€” it says
   nothing about application-level convergence.

### Proof the protocol supports receipts (Q3)

The wire schema fully supports acks â€” this is NOT a missing-message-kind problem:

- `core/src/message/types.rs:7-14` â€” `MessageType::{Text, Receipt, OnionRelay}` (bincode Message struct, `types.rs:38-51`)
- `core/src/message/types.rs:55-62` â€” `Receipt { message_id, status, timestamp }`
- `core/src/iron_core.rs:1985-2010` â€” `prepare_receipt()` = `encode_receipt` JSON wrapped via `prepare_message_with_id(.., MessageType::Receipt, ..)` â†’ signed+encrypted DriftEnvelope. This is what every healthy platform uses.

### Cross-platform parity (Q5)

| Platform | Emits ack? | How | Works? |
|---|---|---|---|
| CLI â†” CLI | Yes | `cli/src/main.rs:2552-2565` `core_rx.prepare_receipt(pk_hex, msg.id)` â†’ swarm send; consumed at `main.rs:2627-2640` | âœ… |
| iOS/Swift | Yes | `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:2226` `self.ironCore?.prepareReceipt(recipientPublicKeyHex:messageId:)` â†’ envelope bytes â†’ `attemptDirectSwarmDelivery(envelopeData:)` | âœ… |
| WASM | Yes | `wasm/src/lib.rs:1147-1154` delegates to `core.prepare_receipt` | âœ… |
| **Android/Kotlin** | Yes, but **wrong format** | `encodeReceipt` bare JSON (see above) | âŒ **Android-specific bug** |

---

## 2. Full traced code path (sender-side status transition, Q1)

Send: `cli/src/api.rs:755 handle_send_message`
â†’ `core.prepare_message_with_id` (`core/src/iron_core.rs:1055`, generates UUID `message_id`)
â†’ history record inserted with `delivered:false` (`api.rs:777-793`)
â†’ BLE first, else `swarm_handle.send_message` (`api.rs:795-824`).

pendingâ†’delivered transition triggers that exist today:

1. **Inbound `MessageType::Receipt` envelope** handled in two places:
   - `cli/src/main.rs:2627-2640`: decode receipt â†’ `history_rx.mark_delivered(id)` â† **this flips the API status**
   - `core/src/iron_core.rs:3523-3565`: `receive_message` classifies receipts internally â†’ `delegate.on_receipt_received` + `mark_message_sent` (outbox cleanup only)
2. Manual CLI command `history mark-delivered` (`cli/src/main.rs:4249`) â€” test-only.

There is no timeout/convergence job that flips history `delivered`; the ONLY event is
an inbound, correctly-signed `Receipt` envelope. That event existed but was
undecodable when sent by Android.

---

## 3. Fix plan

### FIX-1 (required, Kotlin-only): use `prepareReceipt` on Android

**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**Function:** `private fun sendDeliveryReceiptAsync(...)` (line 2481)

Change the receipt-construction + encode block inside the retry loop (~lines 2510-2560)
from:

```kotlin
val receipt = uniffi.api.Receipt(
    messageId = normalizedMessageId,
    status = uniffi.api.DeliveryStatus.DELIVERED,
    timestamp = (System.currentTimeMillis() / 1000).toULong()
)
val receiptBytes = try {
    ...
    uniffi.api.encodeReceipt(receipt)
    ...
```

to:

```kotlin
val receiptBytes = try {
    Timber.d("[RECEIPT-ENCODE] Preparing envelope via prepareReceipt: msg=$normalizedMessageId")
    val core = ironCore ?: run {
        Timber.e("[ERROR] Receipt send FAILED: ironCore not initialized msg=$normalizedMessageId")
        return@launch
    }
    val keyHex = com.scmessenger.android.utils.PeerKeyUtils.normalizePublicKey(senderPublicKeyHex)
        ?: run {
            Timber.e("[ERROR] Receipt send FAILED: invalid sender public key msg=$normalizedMessageId")
            return@launch
        }
    // Returns SIGNED+ENCRYPTED MessageType::Receipt envelope bytes (Drift format),
    // ready to pass directly to the transport. Mirrors iOS MeshRepository.swift:2226
    // and CLI main.rs:2554.
    core.prepareReceipt(
        recipientPublicKeyHex = keyHex,
        messageId = normalizedMessageId
    )
} catch (e: Exception) { ...existing retry/log handling unchanged... }
```

Notes for implementer:
- Delete the manual `uniffi.api.Receipt(...)` construction entirely â€” `prepare_receipt`
  (`core/src/iron_core.rs:1990-1997`) constructs the Receipt itself with
  `DeliveryStatus::Delivered` and current timestamp.
- Keep all surrounding retry/dedupe/logging logic (`receiptSendMaxAttempts`,
  `pendingReceiptSendJobs`, `logDeliveryAttempt`, `attemptDirectSwarmDelivery`)
  untouched. Only the byte-production step changes.
- `attemptDirectSwarmDelivery`'s parameter name `encryptedData` finally becomes accurate.
- The existing bare-JSON suppression in `onMessageReceived`
  (`messageKind == "receipt" || isBareDeliveryReceiptPayload(...)`, ~line 1825) stays â€”
  it becomes dead-code defense for old peers.
- Update the `[RECEIPT-ENCODE]` success log line to include `bytes=` size so the live
  rig can verify envelope sizes (bare JSON â‰ˆ 70-90 B; envelope â‰ˆ 200+ B).
- Consider threading the already-computed `normalizedSenderKey` from
  `onMessageReceived` into `sendDeliveryReceiptAsync` instead of re-normalizing;
  either is fine mechanically.

**Estimated size:** ~15-20 LoC changed, single file.

### FIX-2 (recommended unit test, Kotlin)

**File:** `android/app/src/test/java/com/scmessenger/android/test/ReceiptUnificationTest.kt`

Add a test asserting the receipt path produces an envelope, not bare JSON:
mock/fake `ironCore.prepareReceipt` to return a fixed ByteArray and assert
`sendDeliveryReceiptAsync` passes those bytes (not `encodeReceipt` output) to the
delivery router. Mirror of existing TEST C structure (line 386).
**Estimated size:** ~40-60 LoC new test.

### FIX-3 (optional hardening, Rust-only, no FFI change): dial_policy active-connection exemption (H2 residue)

Live logs show Windows marking the Pixel dead under one address key while an inbound
connection on another address carries traffic. Backoff is keyed per-address
(`core/src/transport/dial_policy.rs:118-124`), and
`reset_on_connection_established` (`dial_policy.rs:218`; called from
`core/src/transport/swarm.rs:5064` for BOTH inbound and outbound
`ConnectionEstablished`) only clears the remote address of THAT connection â€” stale
address keys for the same PeerId stay dead.

Minimal mechanical fix (pick one):

(a) In `DialPolicyManager::register_dial_attempt`, add an eligibility bypass:
```rust
pub fn register_dial_attempt(&self, addr_key: &str, peer_id: Option<PeerId>) -> bool {
    // ... existing lookups ...
    if let Some(pid) = peer_id {
        if self.has_active_connection(&pid) { /* treat as eligible */ }
    }
}
```
requires a shared registry of connected peer ids (swarm.rs `connection_tracker`
already maintains exactly this â€” pass a handle/clone into `DialPolicyManager`).

(b) Simpler: in the `ConnectionEstablished` handler (`swarm.rs:5054-5066`), after
`reset_on_connection_established(&addr_key, ...)`, also reset every backoff entry whose
`state.peer_id == Some(peer_id)` (add
`DialPolicyManager::reset_all_for_peer(peer_id: PeerId)` iterating `peer_backoff`).

**Estimated size:** ~20-35 LoC + ~30 LoC tests (`dial_policy.rs` test module).
This does NOT gate the primary bug; ship separately if desired.

---

## 4. FFI / binding implications

- **No UniFFI regeneration needed.** `prepare_receipt` is already in
  `core/src/api.udl` (exposed object method) and present in both consumers' generated
  bindings:
  - Kotlin: `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt` (checksum symbol `method_ironcore_prepare_receipt`, wired into `build.gradle:223-224` srcDirs)
  - Swift: `iOS/.../Generated/api.swift:2025` (`func prepareReceipt(...) throws -> Data`)
- No UDL edits, no checksum churn, no Swift/Kotlin consumer API changes required.
- FIX-3 (if taken) touches Rust internals only â€” no UDL change â€” but DOES require
  rebuilding the native libs shipped to both platforms (see build steps).

## 5. Risk notes

1. **FFI rule compliance:** zero FFI surface change â‡’ no mandatory Kotlin AND Swift
   updates. However, if FIX-3 lands, both `xcframework` and Android `.so` artifacts
   must be rebuilt from the same commit to keep version parity across the rig
   (repo rule: FFI surface/binary changes require both platform consumers updated).
2. **Backward compatibility:** old Android builds will keep sending undecodable bare
   JSON receipts; they'll continue failing ingress silently (status quo). Do NOT add
   tolerant ingress parsing of unsigned receipt JSON â€” unsigned data flipping
   `delivered` would be a forgery vector (see `tasks/T4.4/progress.md` receipt-forgery
   note). Let old clients age out.
3. **Duplicate-suppression interplay:** `gateInboundMessage` dedupe runs before the
   receipt send in `onMessageReceived` â€” duplicates correctly do not double-send
   receipts (`pendingReceiptSendJobs` dedupes too). No change needed.
4. **Outbox semantics unchanged:** `mark_message_sent` outbox cleanup
   (`iron_core.rs:3545`) already works once envelopes are decodable; the CLI API path
   (`api.rs`) doesn't enqueue to outbox, so no double-clearing concerns there.
5. **Observability gap** (from original RCA): Rust tracing still doesn't reach
   logcat; `eprintln!` bridges exist in `mobile_bridge.rs`. Not required for this fix,
   but recommended follow-up so future RCAs don't depend on Windows-side logs alone.

## 6. Test plan â€” live Windows + Pixel 6a rig

Build:

```
# 1. Windows CLI daemon
cargo build --release -p scmessenger-cli

# 2. Android APK (build.gradle invokes cargo-ndk for core .so automatically)
cd android
.\gradlew assembleDebug

# 3. Install
adb install -r app\build\outputs\apk\debug\app-debug.apk
```

Verify:

1. Start Windows daemon, ensure Pixel peered (mDNS tcp/lan), `/health` 200 both sides.
2. Send from Windows control API:
   ```
   curl -X POST http://127.0.0.1:9876/api/send -H "Content-Type: application/json" ^
        -d "{\"recipient\":\"<pixel-peer-id-or-nickname>\",\"message\":\"ack-test-1\"}"
   ```
   Capture `message_id`.
3. Confirm visible arrival on Pixel UI.
4. **Pass criterion:** within the receipt retry window (< ~10 s typical),
   `curl http://127.0.0.1:9876/api/send/<message_id>` returns
   `"status":"delivered","delivered":true`. Repeat â‰¥3 sends incl. one with AWS relay
   offline.
5. Phone-side confirmation:
   `adb logcat -s Timber:* | Select-String "RECEIPT-ENCODE"` shows
   `Preparing envelope via prepareReceipt` + SUCCESS with envelope-sized `bytes=`
   (>200 B, was ~80 B before fix).
6. Windows-side confirmation: `%LOCALAPPDATA%\scmessenger\logs\scm.log.*` contains
   `receipt_outbox_cleared` (from `iron_core.rs:3546`) and/or
   `Delivery ACK received` debug line (`main.rs:2634`).
7. Regression checks:
   - Androidâ†’Windows text still arrives and decrypts (no regressions in receive path).
   - CLIâ†”CLI receipts still converge (`scmessenger-cli` two-node smoke).
   - `cargo test -p scmessenger-core --test integration_ironcore_roundtrip` (Test 8
     covers Receipt round-trip firing `on_receipt_received`) passes.
   - `.\gradlew :app:testDebugUnitTest --tests "*ReceiptUnificationTest*"` passes.
   - If FIX-3 landed: `cargo test -p scmessenger-core dial_policy`.

## 7. Implementation checklist

- [ ] FIX-1: `MeshRepository.kt` `sendDeliveryReceiptAsync` â†’ `prepareReceipt` (~15-20 LoC)
- [ ] FIX-2: `ReceiptUnificationTest.kt` envelope assertion (~50 LoC)
- [ ] Rebuild APK, live-rig verification per Â§6 (pass: `delivered:true` â‰¤ 10 s)
- [ ] (optional, separate PR) FIX-3: dial_policy peer-wide reset on ConnectionEstablished (~50 LoC incl. tests)
- [ ] (follow-up ticket) tracingâ†’logcat bridge for Rust core observability
