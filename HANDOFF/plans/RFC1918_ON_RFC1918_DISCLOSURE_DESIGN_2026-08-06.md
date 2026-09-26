# Design: RFC1918-on-RFC1918 LAN Disclosure + Contact-Chained Ledger Sharing

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** Proposed
**Date:** 2026-08-06
**Component:** `core/src/store/ledger_entry.rs`, `core/src/transport/addr_filter.rs`, `core/src/transport/swarm.rs`
**Protocol:** `/sc/ledger-exchange/1.0.0`

## 1. Core Principle: "RFC1918 Is Routing, Not Identity"

| Address Type | Disclosure Policy | Rationale |
|---|---|---|
| **RFC1918** (10.x, 192.168.x, 172.16-31.x) | **Disclose to ANY peer reachable via RFC1918** | Only routable on same L2/L3 segment; useless to outsiders |
| **CGNAT** (100.64.0.0/10) | **Disclose to peers reachable via CGNAT** | Carrier-internal; same-network-only utility |
| **Public IPv4** | **Never disclose via ledger exchange** | Enables tracking, geolocation, correlation; route via relay |
| **Loopback/Link-local/Multicast** | **Never disclose** | Never routable; SSRF risk |
| **ULA** (fc00::/7) | **Disclose on same-LAN only** | IPv6 private addressing |
| **Global IPv6** | **Never disclose via ledger exchange** | Same tracking risk as public IPv4 |

**The threat model inversion:** An RFC1918 address is *only* useful to someone already on your LAN. Telling a stranger your 192.168.1.50 reveals nothing — they can't route to it. Telling them your 203.0.113.45 reveals your ISP, approximate geography, and enables tracking across sessions.

## 2. Disclosure Rule: Same-Network RFC1918

The predicate: **"Would disclosing this address help the recipient reach this peer via a path they already have?"**

```rust
pub fn is_disclosable_on_rfc1918_network(multiaddr: &str, my_addrs: &[String]) -> bool {
    let Ok(addr) = multiaddr.parse::<Multiaddr>() else { return false; };

    for proto in addr.iter() {
        match proto {
            Protocol::P2pCircuit => return true,  // Relay addresses always OK
            Protocol::Ip4(ip) => {
                // RFC1918 or CGNAT → only disclose if WE also have an address
                // on the SAME private range class
                if ip.is_private() || is_cgnat(&ip) {
                    return has_matching_private_class(&ip, my_addrs);
                }
                // Public IPv4 → NEVER disclose via ledger exchange
                return false;
            }
            Protocol::Ip6(ip) => {
                if let Some(v4) = embedded_ipv4(&ip) {
                    // IPv6 wrapping IPv4 → re-evaluate as IPv4
                    if v4.is_private() || is_cgnat(&v4) {
                        return has_matching_private_class(&v4, my_addrs);
                    }
                    return false;
                }
                // ULA (fc00::/7) → disclose only if we have a ULA too
                let seg0 = ip.segments()[0];
                if seg0 & 0xfe00 == 0xfc00 {
                    return my_addrs.iter().any(|a| {
                        extract_ipv6(a).is_some_and(|my_ip| {
                            let s = my_ip.segments()[0];
                            s & 0xfe00 == 0xfc00
                        })
                    });
                }
                // Global IPv6 → never
                return false;
            }
            Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_) | Protocol::Dnsaddr(_) => {
                return false;  // Never disclose DNS names
            }
            _ => {}
        }
    }
    true
}

fn has_matching_private_class(ip: &Ipv4Addr, my_addrs: &[String]) -> bool {
    my_addrs.iter().any(|a| {
        extract_ipv4(a).is_some_and(|my_ip| {
            same_rfc1918_class(ip, &my_ip)
        })
    })
}
```

**Key insight:** This reuses the existing `is_dialable_for_this_node()` function's RFC1918 class-matching logic (line 803-832 in `addr_filter.rs`) — just reoriented to disclosure instead of dialability.

## 3. Contact Chaining: Facilitating Ledger Exchange via Mutual Contacts

The critical addition: **If A and B are both verified contacts of C, C can facilitate ledger exchange between A and B.**

```
Before:
  A ──verified──→ C ←──verified── B
  (A has no route info for B, B has none for A)

After contact-driven disclosure:
  A ──verified──→ C ←──verified── B
  A gets B's RFC1918 via C's ledger exchange
  B gets A's RFC1918 via C's ledger exchange
```

### Mechanism

```rust
// In exchange_response_entries()
pub fn exchange_response_entries(
    &self,
    limit: usize,
    requester_peer_id: &str,
    my_addrs: &[String],  // NEW: this node's own addresses for same-network check
) -> Vec<SharedPeerEntry> {
    let entries = self.entries.lock();
    let is_verified_contact = self.is_verified_contact(requester_peer_id);

    entries
        .iter()
        .filter(|e| e.success_count > 0 && e.failure_count < LEDGER_DEAD_FAILURE_THRESHOLD)
        .filter(|e| e.peer_id.as_deref() != Some(requester_peer_id))
        .filter(|e| {
            let addr = strip_peer_id_component(&e.multiaddr);
            if is_disclosable_multiaddr(&addr) {
                return true;  // Globally routable → disclose to anyone
            }
            // New: RFC1918/CGNAT → disclose if same network OR if verified contact
            if is_verified_contact {
                return is_disclosable_on_rfc1918_network(&addr, my_addrs) || is_rfc1918_or_cgnat(&addr);
            }
            // Stranger: only if we're on the same RFC1918 network
            return is_disclosable_on_rfc1918_network(&addr, my_addrs);
        })
        .take(limit)
        .map(ledger_entry_to_shared_routing_only)
        .collect()
}
```

### Contact Chaining: The "Mutual Friend" Rule

When C processes a ledger exchange request from A, C includes B's RFC1918 entries even if C doesn't share an RFC1918 network with A, **provided B is also a verified contact of C**.

```
TrustChain State:
  A <───verified─── C ───verified───> B

C's ledger exchange response to A includes B's entries IF:
  - B is a verified contact of C (success_count > 0)
  - AND B's address is RFC1918 or globally routable

C's ledger exchange response to A EXCLUDES B's entries IF:
  - B's address is a public IPv4 (protected)
  - B is not a verified contact of C
```

```rust
fn is_contact_chained(entry: &LedgerEntry, my_verified_contacts: &HashSet<String>) -> bool {
    // If this entry belongs to a peer who is MY verified contact,
    // I can help route through it by sharing its RFC1918 addresses
    entry.peer_id.as_ref().is_some_and(|pid| my_verified_contacts.contains(pid))
}
```

### Directionality-Aware Exchange

Each node independently decides based on ITS view of the requester:

```
A asks C for entries:
  A.view[C] = VerifiedContact  → A is verified to C
  C.view[A] = VerifiedContact  → C considers A verified

B asks C for entries:
  C.view[B] = VerifiedContact  → C considers B verified

C's response to A includes B's RFC1918:
  [OK] B is C's verified contact
  [OK] B's address is RFC1918 (safe to share with A who is also C's contact)
  [OK] A is on same RFC1918 network as C, so A can reach B

C's response to A does NOT include B's public IP:
  [BLOCK] Public IP is NEVER disclosed via ledger exchange
```

## 4. Implementation Changes

### 4.1 `addr_filter.rs` — New Functions

| Function | Lines | Purpose |
|---|---|---|
| `is_disclosable_on_rfc1918_network()` | ~40 | RFC1918/CGNAT/ULA disclosure check |
| `has_matching_private_class()` | ~10 | Same-class RFC1918 check |
| `is_rfc1918_or_cgnat()` | ~10 | Simple classification |
| `extract_ipv6()` | ~10 | IPv6 extraction from multiaddr |

### 4.2 `ledger_entry.rs` — Modified Functions

| Function | Change |
|---|---|
| `exchange_response_entries()` | Add `my_addrs` parameter; use new predicate |
| `is_verified_contact()` | New helper using `success_count > 0` |
| `contact_chained_entries()` | New filtering step for mutual-contact routing |

### 4.3 `swarm.rs` — Call Site

```rust
// Line ~3987 — exchange handler
let my_addrs = core_handle
    .as_ref()
    .and_then(|w| w.upgrade())
    .map(|core| core.listener_addrs())  // My own listen addresses
    .unwrap_or_default();

let response_peers = core_handle
    .as_ref()
    .and_then(|w| w.upgrade())
    .map(|core| {
        core.ledger_manager.exchange_response_entries(
            LEDGER_EXCHANGE_MAX_RESPONSE_PEERS,
            &requester,
            &my_addrs,  // Pass my addresses for same-network check
        )
    })
    .unwrap_or_default();
```

### 4.4 `addr_filter.rs` — `is_disclosable_multiaddr` Unchanged

The existing `is_disclosable_multiaddr` (public-only) stays as the global default. The new `is_disclosable_on_rfc1918_network` is an ADDITIONAL path used specifically in `exchange_response_entries` when the recipient is on the same RFC1918 network.

## 5. Security Analysis

| Attack Vector | Mitigation |
|---|---|
| **Stranger on same WiFi** gets your RFC1918 | This is the INTENDED use case — they need it to reach you. After handshake+exchange, they dial you directly. No harm from knowing 192.168.1.50. |
| **Shared WiFi AP** — attacker on same LAN | RFC1918 doesn't open anything new; they can already ARP-scan. The ledger exchange adds zero attack surface. |
| **Contact chain amplification** — C shares B's RFC1918 widely | C only shares B's entries to C's OTHER verified contacts (not to strangers). Non-transitive at chain boundaries. |
| **Public IP leakage** | BANNED from ledger exchange. The only way to learn a public IP is via direct connection (Identify/observed_addr). |
| **Contact chain over-relay** — C relays B's addresses to 100 peers | Capped at `LEDGER_EXCHANGE_MAX_RESPONSE_PEERS` (64). No amplification beyond what existing exchange permits. |
| **Malicious verified contact** — C deliberately misroutes | Recipient validates all entries through `merge_shared_entries` → can't be forced to dial what they can't reach. |

## 6. Tests

### Unit Tests

1. `test_disclose_rfc1918_to_same_network_peer` — Both on 192.168.1.0/24, RFC1918 disclosed
2. `test_block_rfc1918_to_different_network_peer` — 10.x and 192.168.x, NOT disclosed
3. `test_contact_chained_disclosure` — C includes B's RFC1918 in response to A (A and C verified)
4. `test_contact_chain_does_not_amplify` — C does NOT include B's entries in response to stranger D
5. `test_public_ip_never_in_exchange` — Assert public IPv4 absent from response
6. `test_cgnat_treated_like_rfc1918` — 100.64.x addresses use same rule
7. `test_ula_v6_treated_like_rfc1918` — fc00::/7 disclosure on same-network only
8. `test_global_v6_blocked` — 2001:db8:: disclosed? No.

### Integration Tests

1. `test_three_node_contact_chain_ledger_convergence` — A-B-C chain in swarm
2. `test_rfc1918_disclosure_does_not_break_stranger_exchange` — Existing test still passes

### Existing Tests That Must Still Pass

- `exchange_response_never_discloses_private_ranges` — Stranger behavior unchanged
- `exchange_response_entries_caps_filters_and_drops_topics` — Cap/topics unchanged

## 7. Rollout

1. **Add `is_disclosable_on_rfc1918_network` + helpers** in `addr_filter.rs`
2. **Modify `exchange_response_entries`** signature to accept `my_addrs`
3. **Update swarm call site** to pass listener addresses
4. **Add `is_verified_contact` helper** to `LedgerManager`
5. **Write all 11 tests**
6. **Feature flag:** `ENABLE_RFC1918_ON_RFC1918_DISCLOSURE` (default ON — this is safe)
7. **No protocol version bump** — sender-side filtering only, wire format unchanged

## 8. Total Change Budget

| Component | Files | Lines Added | Lines Modified |
|---|---|---|---|
| `addr_filter.rs` | 1 | ~80 | ~10 |
| `ledger_entry.rs` | 1 | ~50 | ~30 |
| `swarm.rs` | 1 | ~10 | ~5 |
| Tests | 2 | ~300 | ~10 |
| **Total** | **5** | **~440** | **~55** |

## 9. Open Questions

1. **IPv6 link-local (fe80::/10)?** Currently blocked by `is_unconditionally_routable_ipv6`. Should we allow it on same-LAN? iOS uses fe80:: for mDNS.
2. **Contact chain depth limit?** Currently 1 hop (mutual of C). Do we want 2-hop chaining? (C → D → E)
3. **Rate limit on verified contact chaining?** C can share any verified contact's RFC1918. Should there be an explicit "share me" opt-in per contact?

---

**Decision:** RFC1918-on-RFC1918 disclosure + contact chaining — ready for FusionLite planning and implementation.