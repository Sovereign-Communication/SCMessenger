<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder-next:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_PLATFORM_001_Outbox_Flush_PeerDiscovered

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P1 wire dormant modules
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P1.10 (Outbox flush on PeerDiscovered incomplete) + planfromclaudeforhermes 2 Phase C.4
**Depends on:** P1_CORE_001 (Drift wire provides the new envelope format)

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P1.10: "The CLI swarm loop has partial outbox drain logic, but mobile/WASM paths have NO outbox flush. Offlineonline delivery is unreliable without this."

`outbox` is in `core/src/store/outbox.rs`. CLI has it wired; Android (`MeshRepository.kt`) and WASM (`daemon_bridge.rs`) do not.

## Scope (~80 LoC across 2 files)

### Part A: Android outbox flush (LOC: ~50)

In `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`:

Find `onPeerDiscovered()` (or equivalent callback when a peer becomes reachable). Currently it may log or update UI; add:
```kotlin
fun onPeerDiscovered(peerId: String) {
    // ... existing logic (UI update, peer table update) ...
    
    // Flush outbox to this peer
    scope.launch {
        val pendingMessages = outbox.peekForPeer(peerId)
        for (msg in pendingMessages) {
            try {
                transport.send(msg)
                outbox.markSent(msg.id)
            } catch (e: Exception) {
                Timber.w(e, "Outbox flush failed for peer $peerId")
            }
        }
    }
}
```

Verify outbox access via UniFFI: `outbox.peekForPeer(peerId: String): List<OutboxEntry>` and `outbox.markSent(id: String)` must be exposed in `core/src/mobile_bridge.rs`. If not, add to mobile_bridge (this counts toward the 80 LoC).

### Part B: WASM outbox flush (LOC: ~30)

In `wasm/src/daemon_bridge.rs`:

Find the `on_peer_discovered` handler (or message handler that processes `PeerDiscovered` events from daemon WebSocket). Add:
```rust
async fn on_peer_discovered(&mut self, peer_id: PeerId) {
    // ... existing logic ...
    
    // Flush outbox
    let pending = self.outbox.peek_for_peer(&peer_id);
    for msg in pending {
        if let Err(e) = self.send_via_daemon(&msg).await {
            warn!("Outbox flush failed: {}", e);
        } else {
            self.outbox.mark_sent(&msg.id);
        }
    }
}
```

## File Targets

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT  add outbox flush in onPeerDiscovered]
- `core/src/mobile_bridge.rs` [EDIT  verify outbox peekForPeer/markSent are exposed via UniFFI; add if missing]
- `wasm/src/daemon_bridge.rs` [EDIT  add outbox flush in on_peer_discovered]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib store::outbox

# Android build
cd android
./gradlew :app:compileDebugKotlin -q

# WASM check
cd ..
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown

# Manual: simulate offline  online
# 1. Start CLI daemon with no network
# 2. Send message to peer (goes to outbox)
# 3. Bring network up
# 4. Verify message delivers
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: outbox peekForPeer returns expected entries, markSent removes from outbox, Android onPeerDiscovered triggers flush, WASM on_peer_discovered triggers flush
3. `grep "outbox.peekForPeer\|outbox.peek_for_peer" android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt wasm/src/daemon_bridge.rs` returns  1 hit each
4. Manual: send message while peer offline  goes to outbox; bring peer online  message delivers within 5 seconds
5. Android `./gradlew :app:assembleDebug -x lint` succeeds
6. WASM `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes
7. Commit: `feat(wire): v0.2.1 outbox flush on PeerDiscovered  Android + WASM`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: CROSS_PLATFORM] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P1_CORE_001] [PARALLEL_WITH: P1_CORE_002, P1_CORE_003, P1_CORE_004]
