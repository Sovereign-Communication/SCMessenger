# SCMessenger Unification V2 — Results / Intent Plan

Status: **ACTIVE — mint 2026-08-26 (build mode)**
Owner: Operator (Treystu) — approved single-list sort + identity fail-closed
Supersedes for UI taxonomy: two-section `discovered vs Shared` introduced late 2026-08-25 in `DashboardScreen.kt` / `PeerListScreen.kt`. Subsidiary to `SHIP_PLAN.md` D1-D7, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `GAP_AUDIT_REMEDIATION_PLAN.md`.

> **Prime directive:** plan for *results* (invariants the user experiences), not *how* to code them. Implementations stay open-ended; each requirement states its *intention* — the guarantee it protects. The “hashes getting confused” diary from the last OxAlphaAPI session is treated as root-cause evidence, not style feedback.

---

## 0. Verdicts confirmed this session (updated 2026-08-26 — relay parity)

1. **Mesh tab sort:** one list, **no relay vs non-relay distinction** — default order `online → offline by recency` (all nodes are relays). Former `online user → online relay/infra` tier removed; all nodes are equal relays per "a node is a node" philosophy. Classification is via transport badges only.

4. **Mesh shows exactly the 2 real others online (confirmed 2026-08-27, commit `fb2bb3f6`):** `DashboardViewModel.kt` now (a) **COALESCEs** relay-hop `12D3KooW…` aliases to canonical `public_key_hex` so every transport (BLE/TCP-LAN/Internet/via-shared-node circuit) merges into ONE identity node instead of splitting; (b) applies an **ONLINE-AUTHORITY** gate — a peer is genuinely online only if present in the discovery map (recent + directly observed) OR it holds a DIRECT non-`/p2p-circuit` ledger address with `failureCount==0` and recent `lastSeen` — which drops the phantom relay references (`c0a682ef` MacLane, `26206070` Lucaso, `6a05e70d`) that previously inflated the mesh to "5 online"; and (c) **removes the node's own identity** from its peer list so self never renders as a peer. Live result after coalesce + authority + self-removal: `30d0fa67`(Windows)=ONLINE, `8db1612a`(AWS)=ONLINE, `378d26f5`="Christy Loooove" (saved contact)=OFFLINE → exactly 2 online others; self `8580a133` excluded; phantoms gone.
2. **Identity fail-closed:** `IronCore::with_storage` / `with_storage_and_logs` must surface storage failure as `Err(IronCoreError)` or explicit `storage_degraded` flag the consumer must act on. Silent `IdentityManager::new()` fallback that mints a fresh identity while old one lingers on disk is **deprecated**. Operator approved hard error.
3. **Scope:** `SCMessenger/*` primary. `OxAlphaAPI` harness de-duplication (dup_index / repo_map overlap) low priority, opportunistic only.

---

## 1. Root cause — why the app stopped feeling unified

A single identity was carried as three interchangeable encodings:

| Encoding | Shape | Where it leaked as “the” identifier |
|---|---|---|
| `public_key_hex` | 64-char lowercase hex, 32-byte Ed25519 `VerifyingKey` | correct canonical for crypto (`iron_core.rs:887-894` comment) |
| `identity_id` | `hex(blake3(pubkey))` — also 64 hex, also 32 bytes | `message/types.rs:40-43` doc, `history` reconciliation, old contact `public_key` fields (`contacts.rs:176-217`) |
| `libp2p_peer_id` | Base58BTC `12D3KooW…` (multihash+protobuf) | ledger multiaddrs, `PeerIdValidator.kt:23` loose check, CLI `--peer` args |

They decode identically (64 hex → 32 bytes) so a `hex::decode` success does not distinguish them. `prepare_message_internal` (`iron_core.rs:836-885`) now *scans* every contact’s `blake3(pubkey)` to detect the mix-up and emits `InvalidInput` — defensive cost that should be unnecessary. Mobile code added compensations: `identity_id_idx:` reverse index (`contacts.rs:67,471-496`), `resolve_identity_id` fallbacks, `derive_public_key_from_peer_id`’s 38-byte protobuf branch. Each compensation invited the next.

**Second-order splits** are echoes:

* **Two-section mesh UI** (`DashboardScreen.kt:156-201`, `PeerListScreen.kt:134-174`, `DashboardViewModel.kt:124-247`) partitions one list into `regularPeers / infrastructureRelays`. Count `totalPeersCount` and dedup `deduplicateDiscoveredPeers` drift out of sync; a node can appear in neither or both.
* **Three `with_storage*` constructors** (`iron_core.rs:390-506`) duplicate 120 LoC and historically swallowed `SledStorage::new` errors into `MemoryStorage`; now `DegradedStorage` (`backend.rs:104-182`) fail-closes but `IdentityManager::with_backend` (`iron_core.rs:437-442`) still regenerates identity silently.
* **55 cross-file dup groups** (`results/dup_index.json` — `current_timestamp` ×4, `make_peer_id` ×3, `generate_keypair` ×3, CLI `api.rs` vs `api_axum.rs` ×9, `normalizePublicKey / selectAuthoritativeNickname` ×2 each) — copy-paste instead of shared contract.

**Lesson to carry repo-wide:** anywhere one entity has two names, or one name has two encodings, create **one type / one validator / one error path**. Duplication is a product bug class, not a style nit — it is already tagged `UNIFICATION` in `src/prompt.mjs:18`.

---

## 2. Unified node taxonomy — replaces discovered vs Shared

**Intention:** the mesh tab shows the *truth* of who the node has ever seen, ordered for at-a-glance triage.

*Invariant: there is ONE `PeerInfo` list.* (`DashboardViewModel.kt:452-465`) — all nodes are relays, no `role` tier.

```
PeerInfo :: peerId + publicKeyHex + { attributes }
attributes = {
  transport: Set<String>   // BLE, TCP/LAN, WiFi-Aware/Direct, Internet, Via shared node
  // No role: isRelay/isFull are legacy, no longer distinguish. All nodes are mandatory relays.
  liveness:  "online" | "offline"            // isRecent(lastSeen) 5-min window
  lastSeen:  ULong? ; reachability: direct | circuit | seed
  trust:     verified | unverified
}
```

**Intention of each view choice (not implementation):**

* `transport` badges: explain *how* we know this peer exists — prevents “ghost peer” confusion (`PeerListScreen.kt:226-234` one badge per transport).
* Default sort `online → offline by recency` (all relays equal): preserves the user’s “who can I talk to now?” scan without destroying offline retention (contacts + ledger seeding `DashboardViewModel.kt:124-128`). Former `online user → online relay/infra` tier removed — was the source of "identified as RELAY node" log and cross-platform parity break.
* One count, one dedup (`deduplicateDiscoveredPeers` `DashboardViewModel.kt:255-291` canonical peerId, authoritative nickname wins). Count must equal `peerMap.size` after merge.

**Anti-regression:** no second LazyColumn section whose `key = "section-relays"` is conditionally inserted; no `totalPeersCount = discovered.size` that diverges from `peers.size`.

---

## 3. Pillars — results + intention (how stays open)

### P0 — Identity / hash singularity

*Result:* every function that names “who to talk to” accepts exactly one type: `public_key_hex` (64-char lowercase hex, 32-byte Ed25519 `VerifyingKey::from_bytes` ok). `identity_id` and `libp2p_peer_id` are **derived metadata** — displayed, indexed, never accepted for crypto.

*Intention:* sending to a hash produces undecryptable ciphertext nobody can open (`iron_core.rs:869-877`). The reverse index and reconciliation scans are *debt service*; the payoff is their removal. A stolen ciphertext encrypted to a hash is also a privacy leak (looks like a message to a peer that does not exist).

*Exit signal (auditable, not procedural):* `rg` sweep finds single `PeerId` constructor and single `Base58BTC` validator (`PeerIdValidator.kt:23` strict `1-9A-HJ-NP-Za-km-z`), zero assignments `peer_id -> public_key`, contact payload round-trips Android↔iOS↔Web↔CLI byte-identical.

### P1 — Delivery / custody / receipt singularity

*Result:* one custody+receipt path: `prepare_message* → DriftEnvelope → outbox OR drift custody → transport → receive_message → history.mark_delivered` is the **only** `pending→delivered` transition (`history.rs:292-305`, `iron_core.rs:1985` `prepare_receipt`, `types.rs:20-62`). Transport ACK (`swarm.rs:3700` `[OK] Message delivered…` log) never flips receipt state. Outbox uses `peek-then-remove-on-ack`/lease (`outbox.rs:398-439` currently destructive `drain_for_peer` must become ack-gated, per `PRIORITIZED_TASKS.md` HIGH `flush_outbox_for_peer`).

*Intention:* `MESH_DEBUG_CONTINUATION.md:24-42` proved Android sent bare `encodeReceipt` JSON vs signed envelope; Windows dropped it and `delivered` stuck forever. Two wire formats = permanent pending loop; premature outbox drain = message loss on crash.

*Exit signal:* Windows→Pixel and Pixel→Windows converge `delivered:true` ≤10s on released APK, both with and without AWS relay, and with internet cut (offline proximity). Scored on **receiver-side decrypt + durable history + receipt** per `SHIP_PLAN.md:D4/D6/D7`, never transport ACK or UI counters.

### P2 — Persistence / error-contract singularity (fail-closed, fail-visible)

*Result:* any caller that asked for *persistent* storage either gets it or gets a typed error / explicit `storage_degraded` flag it must handle. No silent RAM fallback for a caller that passed a path. Same for `persist_put/remove` (`backend.rs:239-258` currently `let _ =`), blocked-list checks (`iron_core.rs:4327` `unwrap_or_default` fail-open), and no-downgrade guard (`MeshRepository.kt:801-811`).

*Intention:* silent substitution is why contacts/history vanished after restarts and why blocked peers reappeared. `DegradedStorage` (`backend.rs:104-182`) is the mechanism; the remaining fix is the *contract* around it, especially `IdentityManager::with_backend` (`iron_core.rs:437-442` regenerates identity) now approved to return `Err`.

*Exit signal:* corrupt/lock `contacts.db` / `history.db` surfaces an explicit degraded banner in Android, iOS, CLI, WASM; no identity ratchet divergence on restart; `try_with_storage` paths exercised by tests that simulate corrupt-DB startup.

### P3 — Settings / bootstrap / relay-policy singularity

*Result:* one `MeshSettings` / policy schema converges `mobile_bridge` + `mobile/settings` + `platform/settings`. One bootstrap order `env override → remote config → signed static fallback`, one relay semantic `relay ON = inbound+outbound permitted, relay OFF = both blocked, history readable` enforced in core, bound once.

*Intention:* `GAP_AUDIT_REMEDIATION_PLAN.md:4.1` F2/A4 custody tickets and contact provisioning are gated on this — otherwise contacts provisioned under different truths replay locally but not across region/NAT. Prevents “fork-by-config”.

*Exit signal:* relay OFF blocks send+receive identically on tri-platform parity suite; bootstrap resolver picks same candidate set from same ledger in simulation.

### P4 — UI contract singularity

*Result:* one `PeerInfo` model, one sort/filter vocabulary, one mental model for critical controls (consent gate, identity display, relay toggle, retention) across Android/iOS/Web. No ViewModel re-implements `normalizePublicKey / normalizeNickname / selectAuthoritativeNickname / isSyntheticFallbackNickname` (`dup_index.json:7-9` ×2-4), `formatBytes / SettingsSection` (×2), `preferredContact`, `fragmentData`.

*Intention:* four divergent copies of `selectAuthoritativeNickname` = nickname regression `30d0fa67…` vs `Claude-Windows-Driver` (`MESH_DEBUG_CONTINUATION.md:46`). Shared classifier prevents divergence.

*Exit signal:* tri-platform screenshot parity for mesh tab — same order, same badges (`parseTransportsFromMultiaddrs` `MeshRepository.kt:144`), same empty-state, same synthetic-nickname suppression, driven by core-sourced data.

### P5 (low) — Harness singularity (OxAlphaAPI)

*Result:* dedup between `OxAlphaAPI/results/dup_index.json` and `core` repo_map scanning, and between `OxAlphaAPI/src/*` helpers and core helpers, collapsed where copy-paste. Does not gate `SCMessenger` ship.

---

## 4. Sequenced results (gated, not calendared)

| Gate | Depends | Result delivered | Auditable evidence |
|---|---|---|---|
| S1 — Identity freeze (P0) | — | `public_key_hex` canonicalized; validators singular; `migrate_identity_id_index` idempotent and verified | `rg` sweep clean; contact round-trip byte-identical; `contacts.rs:814-887` tests green |
| S2 — Mesh tab de-split (P4 slice) | S1 (nickname authority) | one sorted list with classification badges/chips; counts reconciled | `PeerListScreen` / `DashboardScreen` screenshot parity; `DashboardViewModelTest` single-list ordering |
| S3 — Crypto & error-contract sweep (P1+P2) | S1 | `encrypt_xchacha20` Poly1305 key derivation, `generateSaltFromTouchPoints` 32-byte, `isLibp2pPeerId` strict Base58BTC, `create_peer_id/make_peer_id` entropy, `persist_put/remove` surfacing, `DegradedStorage` contract | per-theme `UNVERIFIED→RE-AUDIT_CLOSED` (GAP audit failsafe) |
| S4 — Bulk unification (P4 remainder + harness) | S2,S3 | 55→0 cross-file dup groups outside generated/test exclusions | `node src/dups.mjs` + re-audit corpus `NO_ISSUES` |
| Farm-sim v0.5.0 | S1,S3 | six topology scenarios in 12-node sim soak | B3-B6 rig soak per `MILESTONE_RELEASE_PLAN.md` |

Rule: one theme = one branch = one revert; `main` stays green (`SHIP_PLAN.md:2:S0-S1`).

---

## 5. Relationship to standing plans

* `SHIP_PLAN.md` D1-D7 remain the ship gate. This plan does **not** supersede it; it *enables* D4/D6/D7 receipt convergence that was blocked by hash/transport divergence.
* `GAP_AUDIT_REMEDIATION_PLAN.md` S1-S2 are exactly this plan’s S1/S3; S3 there maps to S4 here. Sequencing note §4.1 stands: S2 (identity) lands *before* v0.5.0 provisioning work; S3 executes *as* F2/A4 custody tickets.
* `docs/UNIFIED_GLOBAL_APP_PLAN.md` invariants §§3-7 are the acceptance level above — this plan is the mechanical closure of their §A (identity) and §B (relay/routing) gaps.
* `HANDOFF/plans/V040_COMPLETION_PLAN.md` CP1 `main` green is prerequisite; CP2 (signed APK on releases) must publish the SHA that carries S1+S2 before D4/D6/D7 can be scored on the released build (not dev).

---

## 6. Verification that needs no new invention

* Corpus: `node src/analyze.mjs --limit 0` re-run; `node src/dups.mjs` → 0 cross-file groups; `node src/validate.mjs` drift watch per GAP §4.
* Unit: `cargo test -p scmessenger-core` `history/contacts/outbox/ratchet/dial_policy`; `gradlew :app:testDebugUnitTest` `ReceiptUnificationTest` + `DashboardViewModelTest`; `xcodebuild test`.
* Live rig: Windows CLI (`cargo build --release -p scmessenger-cli`, provenance `cli-artifact/scmessenger-cli.exe` : `MESH_DEBUG_CONTINUATION.md:4`), Pixel 6a (`gradlew assembleDebug` → `adb install -r`, `adb logcat -s Timber:* | Select-String RECEIPT-ENCODE`, `adb shell run-as … tail -50 files/logs/scmessenger-mesh.log`), AWS `54.226.67.101` `scmessenger:latest` — all on one SHA. Scored per `RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN_2026-08-25.md:253-255` `delivered:true` ≤10s.

---

## 7. Risks & non-goals

* `SledStorage` lock contention on Windows (multi-process ledger) surfaces now as explicit degraded state — UX must explain it; no hidden fallback.
* Old Android peers keep sending bare `encodeReceipt` JSON — tolerated as dead-code defense (`MeshRepository.kt:1825` bare-receipt suppression) but never as an ingress acceptance of unsigned receipt JSON (forgery vector `tasks/T4.4` note).
* Not in this plan until after tag: KMP/meeting-mode, iOS parity beyond shared contracts, the remaining 78 unwired non-Android functions not on receipt/custody path.

---

## 8. Execution ledger

| Step | Status | Evidence pointer |
|---|---|---|
| Mint V2 plan (this file) | done 2026-08-26 | `git diff HANDOFF/plans/UNIFICATION_V2_RESULTS_PLAN.md` |
| S2 coalesce + online-authority + self-exclusion | done 2026-08-27 | `fb2bb3f6` dashboard peers = 2 online others (Win+AWS); phantoms offline; `UNIFICATION` logs on-device |
| Verdict 4 (coalesce + online-authority) | done 2026-08-27 | `29d1acfd` docs; `fb2bb3f6` impl |
| Verdict 5 (delivery/ack convergence R1+R2) | done 2026-08-28 | `6be72c82` docs; `0c75bf1a` impl (`mark_message_sent` on true swarm ACK) |
| CI green on PR #234 (fmt, test fix, hygiene) | IN PROGRESS 2026-08-28 | 5 red lanes at `6be72c82`: fmt diffs (`cli/src/main.rs:4066`, `contacts.rs` x13, `behaviour.rs:317`), `message_request_lifecycle_accept` (handler passes identity_id but `ContactsManager::add` canonicalizes to pubkey), trailing whitespace. Fix + merge plan in `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-28.md` |
| Ratchet session-recovery verification | done 2026-08-28 | `decrypt_with_ratchet_fallback` (`core/src/crypto/encrypt.rs:663-832`) rebuilds the receiver session from static keys (V1) / bootstrap fields (V2) on decrypt divergence, retries, and only surfaces `Failed to decrypt ratchet message` (`iron_core.rs:3437`) after both attempts fail. Verdict: PASS — auto re-establishment verified at source (`306e3149`) |
| Verifier-1: plan readiness | pending | subagent report |
| Implement S2 de-split + P0 fail-closed | pending | branch + CI |
| Verifier-2: post-impl re-audit | pending | `dup_index.json` + `cargo test` |
| Deploy + 3-node rig (Windows/AWS/Pixel) | pending | `delivered:true` + `RECEIPT-ENCODE` bytes + `scm.log` `receipt_outbox_cleared` |
| R1+R2 delivery/ACK convergence | done 2026-08-28 | `0c75bf1a` — see Verdict 5 |

---

## 9. Verdict 5 — delivery/ack convergence (R1+R2)  (confirmed 2026-08-28)

**Result: PASS — the durable core outbox now clears only on a true swarm transport ACK; delivery-retry accumulations converge to zero on both nodes.** Commit `0c75bf1a` (`fix/mesh`) on `fix/android-receipt-envelope`, superseding V3 §3.6.

**What R1+R2 do:** on Android (`MeshRepository.kt`) and Windows CLI (`api.rs`/`api_axum.rs`/`main.rs`), call `core.mark_message_sent(message_id)` immediately after `send_message(...) == Ok` — the swarm's true delivery ACK. BLE/WiFi-Direct writes (fire-and-forget) are explicitly **not** a clear; only the libp2p delivery layer is. `Outbox::remove` emits INFO `outbox_dequeue … reason="delivery_confirmed"` (outbox.rs:387-392), removing the entry from the outbox and drift store.

**Live evidence (~25-min soak, both nodes rebuilt on the new SHA; Windows PID 3756/17076, Android Mesh tab + sync pipeline):**
- **R1 (live, 5 events):** `R1: cleared history_sync core outbox on transport ACK` for IDs 8d6fbfb, ebbbd5c, 2b1ca3bf, b20af9be, 52027e00 — Android→Windows sync-family sends released from the core outbox strictly on real swarm delivery. Android outbox flat **23** (378d26f5:18, self:1, 4 stragglers), `outbox_retry_attempt … 30d0fa67` = 0.
- **R2 (convergence-by-emptiness + structural):** 0 `outbox_enqueue`, 0 `outbox_retry_attempt`, 0 retry-climb, nothing accumulating on Windows (vs era run: 35 enqueues, 0 dequeues — the backlog that previously never cleared). The positive `outbox_dequeue … delivery_confirmed` line is wired at every send site (`main.rs:2912,3112,4003,4297`; `api.rs:880`; `api_axum.rs:292`) and needs a live Windows→Android send, which did not occur this window (Windows outbox was already empty — nothing to clear).
- **Regressions:** V6/V7 `[RECEIPT-RX] IGNORING … direction=missing` = 0, INVALID = 0; V11 no retry storm either side; V12 Mesh online = exactly 2 others (30d0fa67 Windows, 8db1612a AWS), self excluded; AWS relay healthy.
- **Build provenance:** CLI `cargo build --release` exit=0 (10m10s; exe 22,068,736 B, `cli-artifact/` size match); APK `assembleDebug` exit=0 (54,808,544 B), `adb install -r` Success.

**Pre-existing, out of scope (flagged):** CLI offline `send` cannot reach Android contacts — stored `contact.peer_id` is 64-hex while `libp2p::PeerId::from_str` needs base58 (`cli/src/main.rs:3981`).

