# V040 CTO checkpoint - NICKNAME-OWNERSHIP-001 / NICKNAME-AUTHORITY-001

## Device evidence (not guessed)

Pixel ledger.json pulled 2026-09-11 (pid 3986, hang-fixed APK):

- **54 entries**, nickname `androidulaator` on **22 rows / 2 distinct peer_ids / 2 public keys**
- Owner A (correct): `1c158e869c2aa62e…` — emulator, 14 multiaddrs
- Owner B (POISON): `30d0fa678c218b22…` — **Windows**, multiaddrs `/ip4/192.168.0.222/tcp/{9001,80,9090,9002/ws,443,8080,57009}`
- Pixel own identity remains `Lucas` (`7d03252b…`)

Log: `UNIFICATION upsertFederatedContact: peer 1c158e86… incomingNick=androidulaator existingNick=androidulaator` and IdentityDiscovered events carrying that nick.

## Root cause (code, confirmed)

1. **D1 last-writer-wins** — `selectAuthoritativeNickname` `else -> incomingNormalized` when both nicks are real. Self-reported nick overwrote a real existing name.
2. **D2 ledger annotate steals multiaddr rows** — `ledger_entry.rs` `find(|e| e.multiaddr == multiaddr)` then **overwrote peer_id AND nickname** even when the row belonged to another peer.
3. **D3 fan-out write** — `annotateIdentityInLedger` wrote a real nick onto **every** dial-candidate multiaddr for `routePeerId` with `shouldWriteNick = true` unconditionally for non-synthetic nicks.
4. Agent string does **not** embed nickname (`scmessenger/{ver}/…/relay/{peerId}` only) — excluded.

## Fix (this change set)

| ID | File | Change |
|---|---|---|
| NICKNAME-AUTHORITY-001 | MeshRepository + DashboardViewModel + ContactsViewModel | When both nicks real and differ → **keep existing** (never last-writer-wins). Synthetic still loses to real. |
| NICKNAME-OWNERSHIP-001 | `core/src/store/ledger_entry.rs` | Multiaddr match only claims peer_id/nick if row unowned or already this peer. Other peer → observe-only, refuse identity claim. |
| NICKNAME-OWNERSHIP-001 | MeshRepository.annotateIdentityInLedger | Write real nick only if ledger row’s peer_id is blank or equals routePeerId. |
| NICKNAME-OWNERSHIP-001 | MeshRepository.reclaimExclusiveFederatedNickname | On setLocalNickname/addContact with a real name, strip that federated nick from every other peer (localNickname untouched). |

## Gates

- `cargo check -p scmessenger-core --lib` EXIT 0
- `cargo test -p scmessenger-core --lib annotate_identity` **3 passed / 0 failed**
- `:app:compileDebugKotlin` BUILD SUCCESSFUL
- NicknameAuthorityTest: pending run (compile green)

## Verify on device (iterate until perfect)

1. Install new APK on emulator + Pixel.
2. On emulator Settings set nickname `androidulaator` again.
3. On Pixel pull `files/ledger.json` via `run-as` — count distinct peer_ids with that nick. **Must be 1** (emulator only). Windows `30d0fa67` must have null nickname.
4. Pixel Dashboard must show Windows as `peer-30d0fa67` or user-assigned, **never** `androidulaator`.
5. Grep logcat: `NICKNAME-OWNERSHIP-001` reclaim lines and `annotate_identity refused`.

## Residual

- Existing poisoned Windows ledger rows on Pixel need reclaim/clear after APK install (reclaim runs on next setLocalNickname; may need one Settings edit or load-time sanitize).
- load-time sanitize of duplicate non-synthetic nicks across peer_ids (optional follow-up).

## LIVE VERIFY 2026-09-12T0045Z (APK 28A0E407 / commit d2a33098)

- Pixel installed, pid 28685, mesh FGS active, BLE running, 0 ANR / 0 Slow main in window.
- ledger.json after install: **24 entries, every nickname is None**.
  - Windows `30d0fa67` multiaddrs present, **nick clean** (was 22 poisoned rows / 2 peers).
  - Emulator `1c158e86` multiaddrs present, nick None (will re-federate; ownership rules keep it exclusive).
- **PASS for poison clear on Windows.** Operator: confirm Dashboard does not show Windows as androidulaator.
