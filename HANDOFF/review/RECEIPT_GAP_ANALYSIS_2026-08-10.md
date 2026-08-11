# Receipt Gap Analysis — why no outbound message ever shows `delivered=true`

Status: Complete (read-only diagnosis, no source files modified)
Author: Claude (Cowork sandbox), diagnosis-only per task constraints
Scope: `tracking/pre-v040-tag-work` (PR 139), root of the five-node merge gate

## One-sentence root cause

`IronCore::prepare_receipt()` (`core/src/iron_core.rs:1920-1934`) returns the
bare `serde_json::to_vec(&Receipt)` payload instead of a properly encrypted,
signed `Message`/`Envelope` — and both production call sites that use it,
`cli/src/main.rs:2433-2436` (Windows CLI) and
`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2465-2525`
(`sendDeliveryReceiptAsync`), hand that bare JSON straight to the transport's
`send_message()` as if it were already wire-ready, so the receiving peer's
`receive_message()` → `decode_wire_envelope()`/`decode_envelope()`
(`core/src/message/codec.rs:103-291`) rejects it as an undecodable envelope
before it can ever be classified as `MessageType::Receipt` — meaning
`mark_delivered()`/`mark_message_sent()` are never reached on either
platform, for any message, ever.

## Which of the four possibilities

Closest to **(c) transmitted but never received** — but with an important
precision the four-way framing doesn't quite capture, stated here rather
than force-fit into a label:

- (a) receiver never generates receipts — **ruled out**. Both the Windows
  CLI (`cli/src/main.rs:2431-2444`, unconditional on any inbound
  `MessageType::Text`) and Android (`MeshRepository.kt:2140`,
  `sendDeliveryReceiptAsync`, unconditional on any inbound genuine text)
  do call the receipt-construction code.
- (b) receipts generated but never transmitted — **ruled out**. The bytes
  are handed to `swarm_handle.send_message()` / `attemptDirectSwarmDelivery()`
  and dispatched onto the wire; the sender-side transport layer reports
  success/acceptance for the send (see `Libp2pMessageResponse{accepted:true}`
  sent unconditionally by the receiving swarm loop at
  `core/src/transport/swarm.rs:3606-3609`, before any envelope decode is
  even attempted).
- (c) transmitted but never received — **closest fit, with a caveat**. The
  bytes physically arrive at the peer's libp2p request-response handler
  (transport-level "received"), but the payload is not a valid `Envelope`/
  `EnvelopeV2`/`DriftEnvelope` structure, so `IronCore::receive_message()`
  (`core/src/iron_core.rs:3256`) fails at the decode step
  (`decode_wire_envelope`/`decode_envelope`, `core/src/message/codec.rs:217-291`)
  and returns `Err(IronCoreError::CryptoError)` before the message ever
  reaches the `MessageType::Receipt` classification logic at
  `core/src/iron_core.rs:3423-3462` or the CLI's own match arm at
  `cli/src/main.rs:2506-2521`. It is "received" at the socket but never
  "received" at the application layer — dropped as malformed, not lost.
- (d) received but ID never matches — **ruled out as the primary cause**.
  If the payload did decode, the IDs would match correctly: the sender's
  history record is keyed by `prepared.message_id`
  (`cli/src/api.rs:759`, from `PreparedMessage.message_id`), which is the
  same UUID embedded as `Message.id` in the plaintext
  (`core/src/iron_core.rs:815-823`); the receiver echoes exactly that value
  back as `Receipt.message_id` (`cli/src/main.rs:2433`,
  `msg.id.clone()`; Android's `MeshRepository.kt:2453-2454`,
  `messageId = normalizedMessageId`). Same UUID namespace throughout — no
  mismatch, this branch of the investigation is moot because decode never
  succeeds in the first place.

## The evidence, in order of the assigned investigation steps

### 1. Does the sender ever expect a receipt? Is the outbound record registered anywhere a receipt could match it?

- `cli/src/api.rs:735-773` `handle_send_message` calls
  `core.prepare_message_with_id(...)` (line 742-748) then unconditionally
  persists a `MessageRecord{ id: prepared.message_id, delivered: false, ... }`
  into `history_store_manager()` (line 757-767) — **this is the record a
  later receipt matches against**, keyed by `message_id`. Registration
  happens regardless of whether the outbox/drift store is touched.
- `prepare_message_with_id` and `prepare_message` are the *same* function
  (`prepare_message_internal`, both at `core/src/iron_core.rs:984-1003`
  simply forward to it). Inside it (`core/src/iron_core.rs:944-966`), an
  outbox/drift entry is only enqueued **if the peer is not already
  connected** (`if !connected { self.outbox.write().enqueue(...) }`). Per
  the live probe, both of this node's peers (AWS relay, Android) were
  connected throughout, so `prepare_message_internal` never took the
  enqueue branch for any of these sends — **this is why
  `/api/drift-status` reports `{"state":"Dormant","store_size":0}`: it is
  the correct, expected behavior for an already-connected peer, not a
  contract violation.** The drift/outbox store and the history
  `delivered` flag are two independent mechanisms; drift-store emptiness
  is not evidence about receipt delivery.
- `queued_transport_send_remains_in_outbox_until_receipt`
  (`core/src/iron_core.rs:4877-4920`) documents the *outbox* contract
  specifically: once a message is queued (peer not yet connected at
  prepare time), establishing a connection alone must not clear it — only
  a receipt (`mark_message_sent`) may. This test never exercises the
  already-connected-at-prepare-time path and does not test the
  history-`delivered` flag at all; it is not in tension with the live
  `Dormant`/`store_size:0` state. **Conclusion: the sender-side contract
  for registering a receipt-matchable record is intact and is not the
  defect.**

### 2. Does the receiver ever generate a receipt?

**Yes, on both platforms — the generation and even the auto-fire trigger
are correct.** The defect is in what gets produced and sent, not whether
receipt-sending code runs.

- Windows CLI: `cli/src/main.rs:2399` (`MessageType::Text` arm, inside the
  `SwarmEvent::MessageReceived` handling block that is confirmed live for
  the `start` subcommand) unconditionally calls
  `core_rx.prepare_receipt(pk_hex.clone(), msg.id.clone())` at line 2433,
  then `swarm_handle.send_message(peer_id, ack_bytes, None, None)` at line
  2436.
- Android: `MeshRepository.kt:2140` calls `sendDeliveryReceiptAsync(...)`
  unconditionally after storing a genuine inbound text
  (also fires for `identity_sync`/`history_sync`/duplicate housekeeping
  messages at lines 2029, 2043, 2088, 2099 — receipts are sent even more
  liberally than on Windows). `sendDeliveryReceiptAsync`
  (`MeshRepository.kt:2422-2551`) builds a `uniffi.api.Receipt`, calls
  `uniffi.api.encodeReceipt(receipt)` (line 2465) to get `receiptBytes`,
  then passes `encryptedData = receiptBytes` directly into
  `attemptDirectSwarmDelivery(...)` (line 2517-2525) — the Kotlin-side
  FFI call into the same shared core (`SwarmBridge::send_message`,
  `core/src/mobile_bridge.rs:3061-3097`, doc comment: "Send an **encrypted
  message envelope**"), which itself just forwards the bytes verbatim to
  the same `swarm.send_message()` used by the CLI.
- **The defect**: `prepare_receipt()` (`core/src/iron_core.rs:1920-1934`)
  and the underlying `encode_receipt()`
  (`core/src/message/types.rs:227-233`, explicitly documented "the ONLY way
  receipts should be serialized anywhere in the codebase") produce **only**
  `serde_json::to_vec(&receipt)` — a bare JSON blob of the `Receipt` struct.
  Contrast with the normal Text-message path,
  `prepare_message_internal` (`core/src/iron_core.rs:735-979`), which for
  every other message type builds a `Message` struct, calls
  `encode_message`, encrypts via `encrypt_message`/
  `encrypt_with_ratchet_fallback`, and wraps the result in a `DriftEnvelope`
  (lines 815-896) before returning `envelope_data`. `prepare_receipt` skips
  every one of these steps. Both call sites (CLI and Android) then hand
  this bare JSON to `send_message()` as if it were already a prepared
  envelope — neither re-wraps it.
- **The codebase's own test proves this is the intended contract being
  skipped in production**: `core/tests/integration_ironcore_roundtrip.rs`
  `test_receipt_roundtrip_flips_state` (lines 316-375) calls
  `bob.prepare_receipt(...)` to get `receipt_bytes` (line 344-346),
  converts them to a UTF-8 string (line 348-349), and then — critically —
  passes that string as the **text body of a second call**,
  `bob.prepare_message_with_id(pubkey(&alice), receipt_str,
  MessageType::Receipt, None)` (lines 351-353), before ever calling
  `receive_message`. That second call is exactly the envelope-construction
  step (`Message` → encrypt → `DriftEnvelope`) that production code omits.
  The test passes because the test author (correctly) performs a step that
  neither `cli/src/main.rs` nor `MeshRepository.kt` performs. This is why
  unit/integration coverage exists and is green, while the live behavior
  fails: the test exercises the correct two-step contract; production code
  only does step one.

### 3. Would the ID even match, and does `mark_delivered` fail loudly or silently?

Moot for the live failure (decode never succeeds, so ID comparison is never
reached), but answered for completeness:

- IDs are consistent: `prepared.message_id` (history key) ==
  `Message.id` embedded in the encrypted payload
  (`core/src/iron_core.rs:815,823`) == `msg.id` the receiver echoes back
  as `Receipt.message_id` on both CLI (`cli/src/main.rs:2433`) and Android
  (`MeshRepository.kt:2454`). No namespace mismatch exists in this
  codebase today.
- `mark_delivered()` (`core/src/store/history.rs:292-305`) does not fail
  loudly on an unknown id: it logs `tracing::warn!("Message {} not found in
  history, could not mark as delivered", id)` and returns `Ok(())` either
  way — a caller cannot distinguish "matched and flipped" from "no such
  id" without reading logs. This is a real (separate, smaller) hardening
  gap, but it is not reachable in the current failure — the receipt never
  gets far enough to call `mark_delivered` at all.

### 4. Live check — has any receipt ever been observed inbound on this node?

Checked the live node (PID 15520, generation 1, `run_20260811T042328Z_gen001.log`,
`%LOCALAPPDATA%\scmessenger\soak\runlogs\`) and the same-hour tracing log
(`%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-08-11-04`) without
restarting or querying anything beyond `GET`s against `http://127.0.0.1:9876`.

- `grep -inE "receipt|mark_delivered|on_receipt|Delivery ACK|Delivered:"` on
  the ~3049-line runlog matches only `sc-receipt-convergence` gossipsub
  topic-subscription log lines (already independently confirmed, in the
  earlier `CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`
  investigation, to be an unrelated relay-custody bookkeeping topic with no
  path to the application-level receipt/`mark_delivered` flow) — **zero**
  matches for `Delivered:`, `Delivery ACK`, `mark_delivered`, or
  `on_receipt`.
- The `println!("\n{} Delivered: {}", ...)` at `cli/src/main.rs:2510` is a
  raw stdout print, not gated by log level, and confirmed captured in this
  same log file (other `println!` output like `"[OK] Listening on 24
  address(es)"` appears verbatim at line 51). Its total absence across the
  full runlog is direct evidence that **no inbound `MessageType::Receipt`
  was ever successfully decoded and processed by this node, in this
  generation's entire run to date.**
  Caveat: the process log filter only emits `INFO`/`WARN`/`ERROR` (no
  `DEBUG`/`TRACE` lines appear anywhere in the file), so `tracing::debug!`
  calls like `"Sending delivery ACK for..."` (line 2435) and `"Delivery ACK
  received from..."` (line 2513) cannot be confirmed or ruled out from
  this log alone — only the unconditional `println!` is dispositive here.
- Strong corroborating (not conclusive — raw bytes were not captured) live
  evidence of the decode failure itself: `grep -c "unexpected end of
  file"` returns **15** occurrences of
  `WARN scmessenger_core::iron_core: Failed to decode wire envelope: io
  error: unexpected end of file` spread across the ~22-minute window
  (04:23:28Z–04:45:02Z), recurring roughly every time
  identity_sync/history_sync traffic (which, per code, triggers a receipt
  reply) was exchanged with a peer. `"unexpected end of file"` is exactly
  bincode's signature failure mode when it interprets a JSON payload's
  leading bytes as a `Vec<u8>` length prefix and then finds far fewer
  bytes than that (garbage) length demands — consistent with, though not
  proven byte-for-byte to be, the raw Receipt-JSON payloads described
  above. `DRIFT_VERSION = 0x01` (`core/src/drift/mod.rs:86`) and
  `WIRE_TAG_V2 = 0x02` (`core/src/message/types.rs:108`); JSON receipts
  start with `{` (`0x7B`), matching neither tag, so
  `decode_wire_envelope`/`decode_envelope`
  (`core/src/message/codec.rs:217-291`) always falls through to the raw
  bincode-`Envelope` attempt and fails.
- Current `/api/history` (queried live, `POST /api/history {"limit":10}`)
  matches the task's stated pattern exactly: every `direction:"sent"` row
  (all inbox-bridge auto-generated `[SEEN]`/`[ACK]` acknowledgement texts)
  has `delivered:false`; every `direction:"received"` row (all
  `identity_sync`/`history_sync` housekeeping from Android, identity_id
  `a43772fe...4a279a`) has `delivered:true` (trivially, per
  `core/src/iron_core.rs:3491`). `/api/drift-status` still reports
  `{"state":"Dormant","store_size":0}` (see item 1 — expected, not
  evidence of the bug).

## Where the fix would land, and the merge-gate question

The defective function, `prepare_receipt()`, lives at `core/src/iron_core.rs:1920-1934`
— **not** inside `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`,
or `core/src/privacy/`. The two production call sites are in `cli/src/main.rs`
(unblocked) and Android's Kotlin (`android/`, a different build entirely, not
a Rust-core path at all).

The narrowest correct fix mirrors what the integration test already proves
correct: have `prepare_receipt()` (or its call sites) route the receipt JSON
through the *existing* `prepare_message_internal` envelope-construction
pipeline (`Message` struct → `encode_message` → `encrypt_message`/
`encrypt_with_ratchet_fallback` → `DriftEnvelope`) with
`MessageType::Receipt`, exactly as
`core/tests/integration_ironcore_roundtrip.rs:351-353` already does by
hand. That pipeline already exists and is already exercised for every Text
send; a fix along these lines calls it rather than modifying it.

**On paper this does not require editing files under `core/src/{crypto,transport,routing,privacy}/`** — it would call already-existing, presumably-already-reviewed functions from those modules (`encrypt_message`, `encrypt_with_ratchet_fallback`) from `core/src/iron_core.rs`, or reroute the call sites in `cli/src/main.rs` and `MeshRepository.kt` to call `prepare_message_with_id(..., MessageType::Receipt, ...)` instead of `prepare_receipt(...)` + raw send. Whether that remains true depends on the actual implementation chosen (if it turns out to require new code inside `core/src/transport/swarm.rs` — e.g. if the receiving-side classification needs a distinct wire-level fast path for receipts — that would trip the merge gate). Given this is fundamentally a wire-encryption/envelope-correctness bug, I'd flag it as security-adjacent regardless of the literal file path and let whoever implements it consult `crypto-security-auditor`/the adversarial review protocol rather than assume the four-directory rule is the only trigger — but as diagnosed, the fix is not mechanically forced into the merge-blocked directories.

## What could not be verified without the Android device

- adb was unreachable (per task constraints), so none of Android's own
  logcat/`mesh_diagnostics.log` output for the current live session could
  be read. Everything stated above about Android's behavior is derived
  from **static analysis of `MeshRepository.kt`** (confirmed: it calls the
  identical `uniffi.api.encodeReceipt()` → raw-bytes-to-`send_message()`
  pattern as the Windows CLI) plus a prior, already-landed investigation
  (`HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`)
  and the same-day `HANDOFF/review/WINDOWS_ANDROID_PROBE_2026-08-10.md`
  probe — not from a live Android log captured in this session.
- Could not confirm, from Android-side evidence, whether Android's
  `attemptDirectSwarmDelivery` for a receipt ever actually leaves the
  device (i.e., whether Android independently also fails at an earlier
  step, e.g. network reachability) versus failing at the same
  envelope-decode step on the Windows side that this report documents.
  The Windows-side log evidence (`decode_wire_envelope` failures,
  `Delivered:` never printed) only proves the receiver-side (Windows)
  half of the round trip is broken this way; it does not independently
  prove Android's receiver-side decode fails identically, though the
  shared-core code path makes it extremely likely (same `receive_message`,
  same `decode_wire_envelope`, same bug class) it does.
- Could not obtain raw wire bytes of an actual receipt payload
  in-flight (no packet capture was taken) to confirm byte-for-byte that
  the specific "unexpected end of file" bincode errors observed in the
  live log are receipts and not some other malformed/unrelated traffic on
  the same peer connection. This is circumstantial, not a captured
  smoking-gun byte sequence — flagged explicitly as such above.
- Could not verify whether `tracing::debug!("Sending delivery ACK for {}
  to {}", ...)` (`cli/src/main.rs:2435`) or `"Delivery ACK received from
  {}: msg_id={}"` (line 2513) ever fired, since the live process's log
  filter only emits INFO and above. Only the unconditional `println!`
  ("Delivered: ...") could be used as dispositive negative evidence.

## Summary table

| Question | Answer | Confidence |
|---|---|---|
| Sender registers a receipt-matchable record? | Yes, in history store, unconditionally | High (code-confirmed) |
| Receiver generates a receipt? | Yes, on both CLI and Android, unconditionally on inbound text | High (code-confirmed) |
| Receipt payload is transmitted? | Yes, handed to `send_message`/`attemptDirectSwarmDelivery` | High (code-confirmed) |
| Receipt payload is a valid wire envelope? | **No** — bare JSON, not `Message`→encrypt→`DriftEnvelope` | High (code + test contradiction) |
| Receiving side can decode it? | No — fails `decode_wire_envelope`/`decode_envelope` | High (code-confirmed; live-log-corroborated) |
| IDs would match if decode succeeded? | Yes | High (code-confirmed) |
| `mark_delivered` ever called on this node? | Never, this generation | High (live: zero `Delivered:` prints) |
| Root cause is crypto/transport/routing/privacy-blocked? | Not mechanically — fix lands in `iron_core.rs`/`cli/`/Kotlin calling existing crypto fns | Medium — depends on implementation choice; flagged as security-adjacent regardless |
