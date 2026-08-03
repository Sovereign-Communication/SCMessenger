# SCMessenger P2P Connection Plan

**Status:** Draft  
**Last updated:** 2026-07-22  
**Source:** Fusion Lite multi-model planning (2 runs, $0.011 total)

---

## Verified: What Already Exists

Before making changes, confirm the following infrastructure is already in place:

| Component | File(s) | Status |
|-----------|---------|--------|
| **Ledger sync on connect** | `cli/src/main.rs:1868`, `core/src/transport/swarm.rs:1864` | Auto-shares ledger entries when a new libp2p connection is established |
| **Ledger receive + merge** | `cli/src/main.rs:1957`, `cli/src/main.rs:2932` | `SwarmEvent::LedgerReceived` merges entries via `merge_shared_entries()` |
| **Ledger convergence test** | `core/tests/integration_ledger_convergence.rs` | Proves two nodes converge ledgers via exchange protocol |
| **Multi-port adaptive listening** | `core/src/transport/swarm.rs:2073` | Tries binding to multiple ports, reports success/failure |
| **Transport escalation** | `core/src/transport/escalation.rs` | Automatic negotiation of best transport (bandwidth/latency/power) |
| **mDNS discovery** | `core/src/transport/behaviour.rs`, `android/.../TransportManager.kt` | Enabled by default, resolves LAN peers |
| **SubnetProbe (LAN probing)** | `android/.../TransportManager.kt:191` | Probes LAN subnet for open ports |
| **Bootstrap nodes** | `cli/src/config.rs`, `cli/src/bootstrap.rs` | Configurable via `config set bootstrap_node_add` |
| **DHT discovery** | `core` (libp2p kad) | Enabled by default via `enable_dht: true` |

---

## Gap 1: IPv6 Loopback Rejection

**File:** `cli/src/ledger.rs:652`  
**Problem:** `is_dialable_multiaddr()` still rejects IPv6 loopback (`::1`). IPv4 loopback was already fixed (removed `ip.is_loopback()` check for IPv4), but the IPv6 branch at line 652 still has `ip.is_loopback()`.

**Current code:**
```rust
"ip6" => {
    if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv6Addr>() {
        if ip.is_loopback() || ip.is_unspecified() {
            return false;
        }
```

**Fix:** Remove `ip.is_loopback()` from the IPv6 branch, mirroring the IPv4 fix:
```rust
"ip6" => {
    if let Ok(ip) = parts[i + 1].parse::<std::net::Ipv6Addr>() {
        if ip.is_unspecified() {
            return false;
        }
```

**Verification:** CLI should accept `/ip6/::1/tcp/...` addresses as dialable.

---

## Gap 2: Android Listener Address Injection (Critical)

**Problem:** The Android app has the CLI as a contact but no listener addresses. `connectToPeer()` is never called because `listeners` is empty. The Android libp2p node receives the CLI's connection but has no address to dial back.

### Approach A: Auto-extract from Incoming Connection (Preferred)

When the Android app receives an incoming libp2p connection, extract the remote peer's observed address and store it for dial-back.

**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  
**Location:** In the connection handler, around where `swarmBridge` processes incoming connections

**Logic:**
1. When a new connection arrives from a peer, extract the remote address
2. Check if the peer is a known contact
3. If yes, store the address as a listener and trigger `connectToPeer()`

**Key insight:** The CLI is reachable from the emulator at `10.0.2.2:9001`. The Android app needs to learn this address and dial it.

### Approach B: Re-add Contact with Listener via Import JSON

Use the existing `ContactImportParser.kt` to import the CLI contact with the listener address.

**JSON format:**
```json
{
  "peer_id": "12D3KooW<redacted>",
  "public_key": "<redacted-pubkey>",
  "nickname": "Windows-Host",
  "listeners": ["/ip4/10.0.2.2/tcp/9001"]
}
```

**Flow:**
1. Copy JSON to Android clipboard
2. Open AddContactDialog (paste button on ContactsScreen)
3. Paste — parser extracts `listeners` array
4. `addContact()` is called with non-empty listeners
5. `connectToPeer()` is triggered with the listener address
6. Android dials `10.0.2.2:9001`

---

## Gap 3: Connection Persistence During Ledger Exchange

**Problem:** The CLI briefly connected to the Android app but the connection dropped before the ledger exchange completed. The Android libp2p node doesn't know the CLI's address, so it can't maintain the connection.

**File:** `cli/src/main.rs` (around line 1957, `SwarmEvent::LedgerReceived` handler)  
**Fix:** Add keepalive during the ledger exchange to prevent premature connection close

```rust
SwarmEvent::LedgerReceived { from_peer, entries } => {
    // Extend connection keepalive during exchange
    if let Some(conn) = swarm_handle.get_connection(&from_peer) {
        conn.keep_alive(std::time::Duration::from_secs(30));
    }
    
    let mut l = ledger_rx.lock().await;
    let new_count = l.merge_shared_entries(&entries);
    // ... rest of existing handler
}
```

**Verification:** Connection should stay alive for at least 30 seconds during ledger exchange. Log timestamps before and after merge.

---

## Gap 4: Backoff Reset on Bootstrap Config Change

**Problem:** When a bootstrap node address is added or changed, the exponential backoff from previous failures prevents immediate re-dial attempts.

**File:** `cli/src/ledger.rs`  
**Fix:** Add a method to reset backoff for a specific peer or address

```rust
pub fn reset_backoff_for_peer(&mut self, peer_id: &str) {
    for entry in self.entries.values_mut() {
        if entry.last_peer_id.as_deref() == Some(peer_id) {
            entry.consecutive_failures = 0;
            entry.backoff_seconds = INITIAL_BACKOFF;
            entry.next_attempt_after = 0; // Allow immediate re-attempt
        }
    }
}
```

**Integration:** Call this when `add_bootstrap()` is called or when the bootstrap config changes.

---

## Implementation Order

1. **Gap 1 (IPv6 loopback fix)** — 5 minutes, one-line change
2. **Gap 2 (Android listener injection)** — 1-2 hours, critical path
   - Prefer Approach B (import JSON) for immediate testing
   - Implement Approach A (auto-extract) for long-term robustness
3. **Gap 3 (Connection persistence)** — 30 minutes, depends on Gap 2
4. **Gap 4 (Backoff reset)** — 15 minutes, quick fix

---

## Verification Steps

### Immediate Test (after Gap 1)
```bash
# CLI is already rebuilt with loopback fix
# Check CLI diagnostics
curl http://127.0.0.1:9876/api/diagnostics | jq '.peers'
# Should show Android peer ID in connected peers
```

### After Gap 2 (Android listener injection)
```bash
# From Android emulator, check if connection to CLI is established
# Check CLI peers
curl http://127.0.0.1:9876/api/peers
# Should show Android peer ID
```

### After Gap 3 (Connection persistence)
```bash
# Send a message from CLI to Android
curl -X POST http://127.0.0.1:9876/api/send \
  -H "Content-Type: application/json" \
  -d '{"recipient":"12D3KooW<redacted>","message":"test from CLI"}'
# Check Android app for received message
# Check CLI diagnostics for outbox delivery
```

### Full Integration Test
```bash
# Run ledger convergence test
cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored
```

---

## Edge Cases

| Case | Handling |
|------|----------|
| IPv6 loopback | Gap 1 fix |
| Emulator restarts (new IP) | ADB port forward stays stable; re-dial on reconnect |
| CLI restarts | Bootstrap re-dial every 120s picks up connection |
| Android app restarts | Contacts persist in database; `loadContacts()` on init |
| Port already in use | Multi-port adaptive listening tries next port |
| Firewall blocks ports | Use relay circuit via bootstrap node (p2p-circuit) |