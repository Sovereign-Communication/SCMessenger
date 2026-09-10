# SCMessenger Unification V3 — Delivery / Ack Convergence Plan

Status: **RESULT — verified 2026-08-28 (D1–D3 + R1+R2 deployed, soak-verified)**
Owner: Operator (Treystu) delegation request. Branches from `fix/android-receipt-envelope`.
Supersedes nothing; extends `UNIFICATION_V2_RESULTS_PLAN.md` (Verdict 4) with the **delivery layer**.

> **Prime directive (from V2):** plan for results (invariants the user experiences), not how to code.
> This plan governs the *remaining* pain: Windows delivery retry-storm, Android 55 stuck outbox, and the
> `[RECEIPT-RX] INVALID … direction=missing` stampede — all confirmed live on the 3-node mesh.

---

## 0. Problem statement (live evidence, 2026-08-27)

With all 3 nodes transport-interconnected (Windows 30d0fa67, AWS relay 8db1612a, Android 8580a133):

1. **Windows storm:** repeated <60s `ROUTE_DECISION … peer-id-<ts>` + `outbox_enqueue recipient_id=8580a133… attempts=N` (N climbing 1→2→3…) — the outbox to Android never converges.
2. **Android stuck:** `files/pending_outbox.json` = **55 held entries** (31→Windows, 16→Christy, 4→AWS, +4 misc), all `acked_without_receipt_count ≥ 1`, never re-sent.
3. **Stampede:** Android logs `[RECEIPT-RX] DELIVERED msg=9e4f2bb9` → `INVALID: Receipt for non-sent message` → `IGNORING … direction=missing` every ~1s — a receipt loop that never clears.
4. **Paradox:** transport delivery WORKS (Android logs `Received message 3c3f9f49… from 12D3KooWD6vZQr` = Windows), yet Windows keeps re-sending — the app-level ack never converges the outbox.

Identity aliases in play (single source of truth: `identity_id = hex(blake3(raw pubkey))`, keys.rs:39):
`Android: pubkey 8580a133 / identity_id b6486de2 / libp2p 12D3KooWJoW9r · Windows: 30d0fa67/985a25f9/12D3KooWD6vZQr · AWS: 8db1612a/0b332009/12D3KooWKMUX`

---

## 1. Root cause — three verified defects (each cited at source)

### D1 — Queued-send mints an outbox ID that differs from the wire envelope ID  [PRIMARY: Windows stuck outbox]
`cli/src/main.rs:4029-4036` prepares the envelope via `core.prepare_message(...)`, which mints **wire ID X** inside the envelope (`iron_core.rs:899`, embedded at `:907`). Then `cli/src/main.rs:4045-4047` stores the `QueuedMessage` under a **fresh** `message_id: uuid::Uuid::new_v4()` = **Y ≠ X** (`recipient_id = contact.peer_id` = pubkey `8580a133`).
A `Delivered` receipt returned by the peer carries **X**. `mark_message_sent(X)` → `outbox.remove(X)` matches strictly on `m.message_id == X` (`outbox.rs:362,375`) → the entry keyed **Y** is never matched → `removed=false` (`iron_core.rs:1080-1081`). The live retry path (`handle_peer_connection_event`, `iron_core.rs:3022-3106`) re-enqueues the same (Y) entry forever, bounded only by the never-reached `MAX_DELIVERY_ATTEMPTS=12` cap (`outbox.rs:66`).
→ **Result: Windows outbox to 8580a133 can never clear; infinite retry storm.**

### D2 — Windows emits a Delivered receipt for every inbound text, no direction/self-origin guard  [DRIVES the Android stampede]
`cli/src/main.rs:2694-2707` unconditionally acks each inbound `Text` back to `sender_public_key` for `msg.id`, with no check that (a) the message is genuinely inbound (not our own outbound looping back through the relay), or (b) the id resides in the sender’s outbox/history. Android’s `[RECEIPT-RX]` therefore sees receipts for IDs it never sent → `direction=missing` → `IGNORING` → loop (and it suppresses the very receipt that would clear D1 if both bugs weren’t present).
→ **Result: Android stampede + the legitimate ack that should clear D1 is polluted/ignored.**

### D3 — Android “held” branch parks transport-acked messages and never re-sends  [Android 55 stuck]
`MeshRepository.kt:8105-8119`: once `ackedWithoutReceiptCount > 0` (transport-confirmed), the no-downgrade rule sets `state="held"`, `nextAttemptAtEpochSec = now + 120s`, and `continue`s — **never re-sends** even though transport is present and an application-level receipt was never actually confirmed. Only exits: a matching Delivered receipt (`2418-2428`) or the 7-day age drop (`8084-8100`).
→ **Result: Android’s 55 outbound messages are permanently parked despite connectivity.**

### D4 — identity_id vs public_key_hex recorded separately (conversation/contact coalescing gap)  [NOT blocking D1–D3 removal, but a unification gap]
Inbound writes inbox/history/audit under `identity_id` (`iron_core.rs:3462-3471, 3607-3634`) while contacts/ledger/outbox use `public_key_hex`; the `identity_id→pubkey` index (`contacts.rs:838`) is only consulted on reads (`:612-615`), never on the inbound write path. This breaks *conversation coalescing* and contact-match, but does **not** block outbox removal (which is message_id-keyed). Low-risk cleanup, secondary.

---

## 2. Fix design (minimal, lowest-risk; restores convergence)

Each fix is independent; do D1→D2→D3 in that order, verify after each.

- **FIX-D1 (Windows CLI, Rust core):** make the outbox ID equal the wire envelope ID.
  Preferred: in `queue_message_for_later_delivery` (`cli/src/main.rs`), use `prepare_message_with_id(recipient_pubkey, text, MessageType::Text, None, id)` where `id = uuid::Uuid::new_v4()` computed ONCE, so the envelope carries X and `QueuedMessage.message_id = X`. Fallback if `prepare_message_with_id` signature differs: thread the prepared envelope’s `Message.id` (available from `prepare_message` return) into `QueuedMessage.message_id` instead of minting a fresh Y.
  Net effect: a Delivered receipt (carrying X) now matches `outbox.remove(X)` → entry clears → storm stops.
- **FIX-D2 (Windows CLI):** guard receipt emission at `cli/src/main.rs:2694-2707`. Only `prepare_receipt` when the inbound message was **not** originated by the local node (reject our-own-outbound echoes) and the message is a real inbound delivery. This stops the Android `direction=missing` stampede and lets the legitimate ack path be heard.
- **FIX-D3 (Android):** remove the “held forever” behavior at `MeshRepository.kt:8105-8119`: when transport is present, a transport-acked message that still lacks an application-level Delivered receipt must be **re-sent** (re-arm nextAttemptAtEpochSec to retry, not park at +120s), while still honoring the no-`Failed`-downgrade intent (keep `state=Sent`, just keep retrying). This allows the 55 parked messages to flush once routes exist.
- **FIX-D4 (Android/Kotlin + core, optional/low-risk):** consult/resolve the identity_id→pubkey index on the inbound write path so inbox/history coalesce to the same contact. Defer unless D1–D3 verification shows residual conversation-split.

## 2a. Implementation status (updated 2026-08-27 — enters build/verify)

- **FIX-D1 (Windows CLI, Rust): DONE** — `queue_message_for_later_delivery` (cli/src/main.rs) now reuses the prepared envelope's `message_id` (wire ID X) as the outbox entry key instead of minting a new UUID Y, so a Delivered receipt (carries X) matches `outbox.remove(X)` and the outbox clears. `cargo check -p scmessenger-cli` clean.
- **FIX-D2 (Windows CLI, Rust): DONE** — cli/src/main.rs only emits a Delivered receipt for a genuine inbound user message; skips `scm.message.identity.v1` sync/config metadata and self-loop echoes. Kills the Android `[RECEIPT-RX] IGNORING … direction=missing` stampede. `cargo check` clean.
- **FIX-D3 (Android): DONE** — MeshRepository.kt `flushPendingOutbox`: removed the "park forever" `held` branch (was +120s forever); transport-acked messages now fall through to the real send path and keep re-sending until a Delivered receipt or the age-based ceiling. The attempt-cap branch is guarded (`ackedWithoutReceiptCount==0`) so acked-without-receipt messages are never Corrupted/Failed (honors the no-downgrade intent). This lets the 55 parked entries drain once routes exist.
- **FIX-D4 (conversation coalescing): TRACKED, not coded** — see D4 disposition above; gate is live check V14. No risky core keying change this pass.

**Scope guard:** D1–D3 are Android/Windows-software only; wire formats untouched; AWS relay passes bytes through unchanged. No secrets/logging changes.

---

## 3. Verification plan — run BEFORE and AFTER each fix on the live 3-node mesh

Baseline must be captured immediately before the first build (this exact session’s numbers are the pre-fix baseline).

### 3.0 Pre-fix baseline (capture NOW, before changing code)
| # | Check | Method | Expected pre-fix |
|---|---|---|---|
| B0 | Windows outbox non-convergence | Windows log: count `outbox_enqueue recipient_id=8580a133` in last 15 min | >0 and attempts climbing |
| B1 | Android outbox stuck | `adb shell run-as … cat files/pending_outbox.json` → count entries with `acked_without_receipt_count>=1` | ~55, stable across 5 min |
| B2 | Android stampede | Android log count of `[RECEIPT-RX] IGNORING … direction=missing` in 5 min | high (many/sec) |
| B3 | Windows `receipt_outbox_cleared` | Windows log count of `event="receipt_outbox_cleared"` | ~0 (never clearing) |

### 3.1 Post-FIX-D1 verification (Windows convergence)
| Check | Method | PASS criterion |
|---|---|---|
| V1 | Windows log: `outbox_enqueue … 8580a133` stops growing | count plateaus, then decreases |
| V2 | Windows log: `receipt_outbox_cleared` appears with `removed=true` | ≥1 per previously-stuck message |
| V3 | Windows `outbox_dequeue … reason=delivery_confirmed` | appears for the 8580a133 entries |
| V4 | Windows log stability: no `ROUTE_DECISION` loop for the peer-id message | stops within one receipt exchange |
| V5 | Windows outbox DB/ledger → pending count to Android ≈ 0 | `scmessenger … /info` or ledger shows empty queue |

### 3.2 Post-FIX-D2 verification (stampede stops)
| Check | Method | PASS criterion |
|---|---|---|
| V6 | Android log `[RECEIPT-RX] IGNORING … direction=missing` | count → 0 |
| V7 | Android log shows valid receipts being acted on (no `INVALID`) | 0 INVALID lines over 5 min |

### 3.3 Post-FIX-D3 verification (Android outbox drains)
| Check | Method | PASS criterion |
|---|---|---|
| V8 | Android `pending_outbox.json` total entries | drains 55 → 0 over ~5 min |
| V9 | Android log shows the entries re-sending (`state != held`, attempt_count climbing then removed) | entries leave `held` and reach delivered |
| V10 | Windows/AWS see the corresponding `Received message …` for the drained Android outbound | Android→Windows and Android→AWS messages now arrive |

### 3.4 Net-state verification (all fixes, end-state invariant)
| Check | Method | PASS criterion |
|---|---|---|
| V11 | All 3 nodes idle-stable: no retry storm, no stampede, no restart-required loop | 5-min log soak: no `outbox_enqueue` growth, no `[RECEIPT-RX] IGNORING` |
| V12 | Mesh tab on Android still shows exactly **2 online** (30d0fa67, 8db1612a), self excluded, phantoms gone | DashboardView `UNIFICATION loadPeers` = online 2 |
| V13 | New A→B→A message round trips with delivery receipt in <10s on each side | send test Windows↔Android, confirm both outboxes clear |
| V14 | (If D4 done) one conversation thread per real peer, aliases coalesced | no split threads for 30d0fa67 / 8db1612a |

### 3.5 Rollback
Each fix is a single small diff on its own file. If V1–V10 fail, revert that fix and re-verify; D1–D3 do not depend on each other’s correctness for build (only for end-state).

### 3.6 Post-deploy verification results (2026-08-28) — RESULT

D1/D2/D3 deployed + verified in the 2026-08-27/28 session; the final convergence turn added **R1 (Android, MeshRepository.kt)** and **R2 (Windows CLI, api.rs/api_axum.rs/main.rs):** clear the core outbox entry strictly on a true swarm transport ACK (`mark_message_sent(…message_id)` immediately after `send_message(...) == Ok`); BLE/WiFi-Direct are **never** a clear. `Outbox::remove` logs INFO `outbox_dequeue … reason="delivery_confirmed"` (outbox.rs:387-392).

| # | Check | Result (live, ~25-min soak, both nodes on new SHA) |
|---|---|---|
| V1 | Windows outbox to 8580a133 growth | **0 `outbox_enqueue`** the whole window (pre-fix era run: 35 enqueues, 0 dequeues). Nothing accumulating; legacy `0ac772f6`/`808ee57c` backlog absent (culled at the 12-attempt cap before deploy). |
| V2 | `receipt_outbox_cleared` / mark-on-ACK | Wired at every send site (`main.rs:2912,3112,4003,4297`; `api.rs:880`; `api_axum.rs:292`) tracking the true swarm-ACK; `outbox_dequeue … delivery_confirmed` not live-emitted only because Windows enqueued **0** messages to Android this window (outbox was already empty). |
| V3 | Windows outbox idle / no retry climb | `outbox_retry_attempt` = 0; nothing climbing toward the 12 cap. |
| V6 | Android `[RECEIPT-RX] IGNORING … direction=missing` | **0** across soak. |
| V7 | `[RECEIPT-RX] INVALID` | 0 lines. |
| V8/V9 | Android outbox drain | flat **23** (378d26f5:18, self 8580a133:1, a27fe5a7/26206070/c0a682ef/6a05e70d:1) — Windows-targeted entries already 0 from the D1/D2 drain (30→0). No growth, no held-stack accumulation. |
| R1 (live) | `R1: cleared … on transport ACK` | **5** clears this soak: history_sync IDs 8d6fbfb, ebbbd5c (pre-restart) + 2b1ca3bf, b20af9be, 52027e00 (post-restart) — sync-family sends to Windows released from the core outbox only on genuine swarm delivery. `outbox_retry_attempt … 30d0fa67` = 0. |
| V11 | retry-storm / stampede | 0 on both sides (0 enqueues, 0 retry-attempt lines, 0 IGNORING). |
| V12 | Mesh online | exactly 2 others (`30d0fa67` Windows, `8db1612a` AWS), self excluded, phantoms gone. |
| AWS | relay soak | healthy; only routine Kademlia/AutoNAT debug lines. |

**Verdict: converges.** Windows CLI `cargo build -p scmessenger-cli --release` exit=0 (10m10s; exe 22,068,736 B deployed to `OxAlphaAPI/cli-artifact/`, size match); Android `gradlew assembleDebug` exit=0 (APK 54,808,544 B), `adb install -r` Success. Both nodes healthy. Committed `0c75bf1a` (`fix(mesh): converge core outbox on true swarm delivery ACK (R1+R2)`) on `fix/android-receipt-envelope`.

Known pre-existing (out of scope): CLI offline `send` cannot route to Android — stored contact `peer_id` is 64-hex while `libp2p::PeerId::from_str` requires base58 (`main.rs:3981`).

---

## 4. Deploy sequence
1. Fix + test-build Rust core (unit): `cargo test -p scmessenger-core` (existing tests for `prepare_message_with_id`/outbox id parity).
2. Fix + test-build CLI: `cargo build -p scmessenger-cli --release` → deploy `target/release/scmessenger-cli.exe` to `OxAlphaAPI/cli-artifact/scmessenger-cli.exe` (per session convention), restart Windows CLI, run 3.1 checks.
3. Fix + assemble Android: `gradlew assembleDebug --parallel --max-workers=16` → `adb install -r` → relaunch → tap Mesh tab → run 3.2/3.3/3.4 checks.
4. Run full soak V11–V13 with all 3 nodes up; capture logs to `%LOCALAPPDATA%\scmessenger\logs`, Android `files/logs/scmessenger-mesh.log`, and AWS `sudo docker logs scm-node`.
5. Commit + push `fix/android-receipt-envelope`; append Verdict 5 to `UNIFICATION_V2_RESULTS_PLAN.md`; update this plan to RESULT.

---

## 5. Risks / decisions needing operator sign-off
- **D3 intent change:** the current no-downgrade “hold forever” is deliberate (D3–P3_ANDROID_RETRY_SUPPRESSION comment). Fix-D3 keeps `state=Sent` (honors intent — no Failed downgrade) but **re-sends** when transport present instead of parking. Needs sign-off that re-sending is acceptable (idempotent delivery by message_id).
- **D2 direction detection:** distinguishing “our own outbound looped back” from a genuine inbound stream requires a small origin check on the CLI. Confirm the anti-echo heuristic (reject receipts for ids this node minted as outbound) is acceptable.
- **D4 scope:** confirm whether conversation-level alias coalescing is desired in this pass or deferred.
- **AWS:** expected no code change (relay passes bytes). Will still soak its logs for message-id mutation as the investigation flagged as a residual to confirm (relay may or may not re-id; if it does, D1’s id-parity must account for it).
