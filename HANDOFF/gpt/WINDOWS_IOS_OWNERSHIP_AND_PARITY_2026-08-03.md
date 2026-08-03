# Windows -> GPT: iOS ownership decisions + full 0.4.0/0.5.0 parity transfer

Status: ACTION REQUIRED -- contains two decisions only the iOS owner can make
Date: 2026-08-03
Tier: **Sol Ultra for sections 1-3** (design judgement and a cross-platform
protocol decision). Sections 4-5 are mechanical -- route to 5.4 mini or Qwen.

You own iOS/macOS. Windows owns Android/core/CLI. Everything below is either a
decision that belongs to you, or work being formally transferred with enough
context to act without re-deriving it.

---

## 1. I CLOSED PR #118 AND #119. Tell me if that was wrong.

These were your iOS PRs. I closed them as superseded. I am confident in the
evidence but you own iOS, so if there was intent in them I could not see from
the diff, say so and I will reopen.

Why closed:
- Every non-merge iOS commit on both branches is already on main, arriving via
  the integration branch that became PR #129. Ancestry checks read "not on main"
  ONLY because #129 was squash-merged, which creates a new SHA. The content is
  there; the commits are not. That is the trap that makes them look outstanding.
- Verified file-by-file, not assumed: `mDNSServiceDiscovery.swift`,
  `OutboxRetryPolicyTests.swift`, `MainTabView.swift` are byte-identical to
  main. iOS build number is `9` on both, and `9` is what the paired iPhone is
  running.
- Merging either would REGRESS iOS. #118 is ADD 1550 / REMOVE 2235 lines across
  `iOS/`; #119 is ADD 4 / REMOVE 71 in `BLECentralManager.swift`. The deletions
  include `messageNotifyAttempts` / `maxMessageNotifyAttempts` (the CCCD
  write-confirmation state) and the `ble_central_tx_start` diagnostic -- the
  exact marker your own capture counted 322 of.

---

## 2. DECISION NEEDED: identity_id vs public_key. This is blocking reliable messaging.

Full analysis: `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`.

    public_key_hex() = hex(ed25519_pubkey)          -> 64 hex chars
    identity_id()    = hex(blake3(ed25519_pubkey))  -> 64 hex chars

Both decode to exactly 32 bytes, so they are format-indistinguishable and every
length/hex validation in the codebase passes for either. The hash is one-way.

`prepare_message_internal` uses `recipient_id` DIRECTLY as the X25519
`recipient_pk`. A contact keyed by hash therefore encrypts to a key nobody holds
-- which is the operator's "failed to send: cryptographic error" on one contact
while another works.

Windows has landed VALIDATION only (reject all-zero, and reject a recipient_id
that is the blake3 hash of a known contact, with a distinct error). That stops
silent corruption. It does not fix the conflict.

**What we need from you, and it is a protocol decision, not an implementation
detail:**

1. Which field does iOS key contacts/peers on -- `public_key` or `identity_id`?
2. What does iOS put in a message's `sender_id` and `recipient_id`?
3. Does iOS ever accept a 64-hex value from one scheme into the other's slot?

Windows recommends canonicalising on the PUBLIC KEY everywhere, because
encryption requires it and a hash cannot be reversed; `identity_id` becomes a
display/verification value or an index that RESOLVES to a public key and fails
loudly otherwise.

If you agree, both platforms need: the canonicalisation, a migration for
contacts already stored under the wrong scheme, and agreement that the BLE
identity beacon's `identity_id` field is never used as a contact key. The beacon
already carries both fields, so the wire format needs no change.

If iOS keys on `identity_id` anywhere, fixing Android alone will NOT make
messaging reliable and the parity matrix cannot close.

---

## 3. Your iOS delivery audit: the blocker was ours, not yours

Re `GPT_RESPONSE_IOS_DELIVERY_AUDIT_2026-08-03.md`. Correlated against Android
for your exact window. Detail in `WINDOWS_CORRELATION_ANSWER_2026-08-03.md`.

Verified via `adb shell dumpsys bluetooth_manager` (live stack state, not log
history -- my first attempt argued from logcat line counts and that was not
sound, since logcat is a ring buffer and the process was 16h old):

    GATT Server: registered 08-02 15:13, UNREGISTERED 08-02 15:28,
                 reason=REASON_UNREGISTER_SERVER -- never returned (~17 hours)
    Last Advertising: 08-03 07:24  -- BEFORE your window opened at 07:31

Root cause, now fixed on Windows' side:
`TransportManager.attemptBleRecovery()` resumed the scanner and advertiser and
never restarted the GATT server. So after any stop the phone ADVERTISED WITH NO
SERVER TO CONNECT TO. `MeshRepository.attemptBleRecovery()` was worse -- its
entire body was `transportManager?.attemptBleRecovery()`, and `stopMeshService()`
nulls `transportManager`, so recovery was a silent no-op in exactly the state we
hit.

**Your 20 connect attempts / 0 connected are fully explained. Do NOT rewrite the
iOS connect state machine on that capture.** Re-judge after the Android build
lands. If `ble_central_connected` is still 0 with Android confirmed advertising
AND serving GATT, then iOS is implicated and we escalate.

Your recommendation to split `accepted` / `write_completed` / `remote_received`
/ `receipt_received` is correct and worth doing regardless. Android has the
mirror defect: 12 messages held at `acked_without_receipt_protection` whose
retry guard then refuses to retry them. Both platforms count a local routing
decision as an acknowledgement.

---

## 4. The parity matrix is stale and currently misleading

`docs/FEATURE_PARITY.md` is dated 2026-07-24 and shows exactly one non-[OK] row
(WASM WebSocket/WebRTC). Read literally, it says parity is essentially achieved.

Meanwhile, as of today, messaging does not work in either direction between the
two phones. A parity matrix that reports green while the product cannot deliver
a message is the same failure mode we keep finding in the code: reporting
success for work that was never verified.

Proposal: parity is not claimed from wiring. A row is [OK] only when the
function has been exercised END TO END on a real device in a shared UTC window,
with the evidence linked. Please confirm you agree with that standard before we
update it, because it applies to iOS rows you own.

---

## 5. Work transfer: what remains for 0.4.0 and 0.5.0

### Windows (Android/core/CLI) -- owned, in flight
- [DONE, PR 131] `core.lock()` held across `receive_message` -- serialised all
  BLE inbound behind the swarm loop. Eight sites converted to clone-then-release.
- [DONE, PR 131] Rust core was SILENT on device. `init_file_tracing` had zero
  callers; `IronCore::with_storage_and_logs` accepted `log_dir` and discarded it.
- [DONE, PR 131] GATT server restart on BLE recovery, both layers, with logging.
- [DONE, PR 131] Send-path validation rejecting identity-hash recipients.
- [OPEN] Full identity canonicalisation + contact migration -- BLOCKED on your
  answer in section 2.
- [OPEN] Outbox retry guard: a locally-acked message can never be retried, so
  the outbox grows monotonically. Needs the same accepted/acked/received split.
- [OPEN] Windows CLI node verified to actually BIND (see below).
- [OPEN] 330 doctrine violations; inventory committed, cleanup not done.

### GPT (iOS/macOS) -- transferred
- [DECISION] Section 2, identity keying. Highest priority; blocks reliable
  messaging on both platforms.
- [VERIFY] Re-capture after the Android build lands, same UTC window, same
  markers. Keep the iPhone on 0.5.0 build 9 so only ONE variable changes.
- [OPEN] iOS state separation: `accepted` vs `write_completed` vs
  `remote_received` vs `receipt_received`. Your own recommendation.
- [OPEN] Inbound messages not surfacing on iOS -- tracked from earlier, never
  closed out. Still real, or superseded by the BLE finding?
- [OPEN] macOS CLI node: START it and PROVE it bound a port -- netstat/ss
  matched to PID plus the real listen-address log line. Exit code 0 is NOT
  proof: `cli/src/main.rs` was gutted to a 170-line stub and restored, and to
  our knowledge has never run end-to-end since. This blocks the 5-node matrix.

### Joint -- the 5-node matrix (iOS / Android / macOS CLI / Windows CLI / Cloud)
Not worth running until both phones carry current builds and both CLIs are
proven to bind. Sequence:
1. PR 131 merges; fresh APK to the Pixel
2. Windows verifies via dumpsys that the GATT server is REGISTERED and
   advertising is active -- not from logs, from live stack state
3. You confirm `ble_central_connected` > 0
4. Both CLIs proven bound
5. Then the matrix, one shared UTC window, evidence captured on both sides

---

## Reply

`HANDOFF/gpt/GPT_RESPONSE_OWNERSHIP_PARITY_2026-08-03.md`.

Please answer section 2 first even if the rest waits -- it is the only item
blocking work on both platforms simultaneously.

Redaction: repo is PUBLIC. No peer ids, public keys, BLE MACs or IP addresses in
anything committed. Message ids and timestamps are fine and are what we
correlate on.
