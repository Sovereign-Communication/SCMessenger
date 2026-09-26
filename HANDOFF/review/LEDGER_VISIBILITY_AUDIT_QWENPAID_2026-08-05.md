<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

### READ-ONLY AUDIT REPORT

**RESULT:** DONE
**VERIFICATION:** NONE (read-only audit)
**FILES:** `core/src/store/ledger_entry.rs`
**NOTES:**
1.  **ROOT CAUSE:** `exchange_response_entries` (line 1170) filters by `success_count > 0`. Fresh Android node has zero proven peers; reciprocal exchange returns empty set because Android cannot offer "proven" peers to trigger a rich response, and iOS only shares what *it* considers proven/disclosable.
2.  **CONTRIBUTING FACTOR:** `is_disclosable_multiaddr` (line 1173) strips RFC1918. If "home network" implies LAN-only connectivity without global IPv4/IPv6, iOS correctly redacts all neighbors from the wire response per NEW-2 security doctrine.
3.  **FIX DIRECTION:** Verify if `is_disclosable_multiaddr` supports a `LocalMesh` context for authenticated/paired peers, or if ledger exchange requires a separate "trusted peer" bypass for LAN convergence.
4.  **ACTION:** Orchestrator must inspect `core/src/transport/addr_filter.rs::is_disclosable_multiaddr` and `swarm.rs` ledger-exchange handler to confirm if network mode is hardcoded to Public/Untrusted for reciprocal sharing.

---

### Ranked Hypothesis List

#### 1. Disclosure Filter Redacts LAN Topology (Most Likely)
*   **Evidence For:**
    *   `ledger_entry.rs:1173`: `exchange_response_entries` unconditionally applies `is_disclosable_multiaddr`.
    *   Test `exchange_response_never_discloses_private_ranges` (line 2051) explicitly asserts that RFC1918 (`192.168.x.x`, `10.x.x.x`) addresses are **never** returned in a ledger exchange response.
    *   Problem statement specifies "home network". If the fleet is communicating over LAN IPs, iOS is architecturally forbidden from sharing those IPs via this protocol.
    *   iOS sees 5 nodes likely because it discovered them via mDNS/LocalDiscovery (which bypasses ledger exchange disclosure rules) or has them stored locally, but cannot *share* them via the ledger exchange wire protocol.
*   **Evidence Against:**
    *   If the nodes have global IPv6 or port-forwarded IPv4, they should be disclosable.
    *   The filter is working as designed for security (NEW-2); the bug may be that "paired" nodes don't get a trust escalation.
*   **Sites to Inspect:**
    *   `core/src/transport/addr_filter.rs`: Definition of `is_disclosable_multiaddr`. Does it accept a `NetworkMode` or `TrustLevel`?
    *   `core/src/transport/swarm.rs:5668` / `6224`: How is `exchange_response_entries` called? Is it always called with the assumption of an untrusted peer?
*   **Fix Sketch:** Introduce a `TrustedPeer` or `LocalMesh` context to `exchange_response_entries` (or a sibling method) that allows RFC1918 disclosure when the remote peer is cryptographically verified as a paired/fleet member. Alternatively, ensure local discovery runs independently of ledger exchange.
*   **Adversarial Review Flag:** **YES.** Relaxing disclosure controls re-enables the SSRF/topology leak vector described in NEW-2. Any fix must strictly gate RFC1918 disclosure on cryptographic pairing proof, not just connection establishment.

#### 2. Provenance Gate Blocks Fresh Node Convergence
*   **Evidence For:**
    *   `ledger_entry.rs:1171`: `exchange_response_entries` requires `success_count > 0`.
    *   A fresh Android node has `success_count == 0` for all entries (including Christy, until the first connection completes and `record_connection` is called).
    *   Ledger exchange is reciprocal. If Android sends an empty/unproven list, iOS might still respond (per swarm.rs:3936 "reciprocates automatically"), but the *content* of that response is gated by iOS's own ledger state.
    *   However, if Android hasn't successfully dialed Christy *before* the exchange triggers, or if the exchange triggers before `record_connection` persists the success, Android's view remains empty.
*   **Evidence Against:**
    *   Swarm docs say reciprocity is automatic regardless of payload. iOS should send its full (filtered) view regardless of what Android sent.
    *   This hypothesis explains why Android sees *nothing*, but not why iOS sees *5*. It's a necessary condition for Android's emptiness but insufficient to explain the asymmetry alone.
*   **Sites to Inspect:**
    *   `core/src/transport/swarm.rs:2223`: Timing of `share_ledger` vs `record_connection`.
    *   `ledger_entry.rs:830`: `record_connection` logic.
*   **Fix Sketch:** Ensure `record_connection` is called atomically with connection establishment *before* ledger exchange triggers, or allow `annotate_identity` entries (success_count=0) to be included in exchange responses for bootstrapping.
*   **Adversarial Review Flag:** No.

#### 3. Identity Canonicalization Drops Entries During Ingest
*   **Evidence For:**
    *   PR #136 mentions `identity_id_from_public_key_hex`. If Android's ingest path uses a different canonicalization than iOS, entries might be keyed differently or dropped.
    *   `ledger_entry.rs:1227`: `ledger_entry_to_shared_routing_only` blanks routing-irrelevant fields. If Android relies on a field that gets blanked (e.g., `nickname` or `public_key`) for rendering, it might filter them out at the UI layer.
*   **Evidence Against:**
    *   `SharedPeerEntry` (wire format) retains `multiaddr` and `last_peer_id`. These are the primary keys.
    *   Ingest (`import_seed_entries` / `annotate_identity`) is robust to missing optional fields.
    *   Unlikely to cause *total* invisibility unless the UI strictly requires a field that the wire protocol strips.
*   **Sites to Inspect:**
    *   `core/src/identity/keys.rs`: `identity_id_from_public_key_hex`.
    *   Android/Kotlin UI code (not provided): What fields does the node list render?
*   **Fix Sketch:** Align Android UI rendering to use `multiaddr` + `peer_id` as primary identifiers, tolerant of missing metadata.
*   **Adversarial Review Flag:** No.

#### 4. Rendering Layer Mismatch (UI vs Ledger)
*   **Evidence For:**
    *   Android might be reading from a "Contacts" store rather than the "Ledger" store.
    *   iOS might be aggregating mDNS + Ledger + Contacts, while Android only shows Contacts.
*   **Evidence Against:**
    *   Doctrine states "discovery is ledger sharing". Assuming architecture adherence, both should read ledger.
*   **Sites to Inspect:**
    *   Kotlin/UniFFI bindings for `LedgerManager.dialable_addresses` vs `seed_addresses`.
    *   Android ViewModel/Repository feeding the node list.
*   **Fix Sketch:** Audit Android data source. Ensure it subscribes to ledger updates.
*   **Adversarial Review Flag:** No.

### Missing Files Needed for Definitive Diagnosis
1.  `core/src/transport/addr_filter.rs` - To confirm exact RFC1918 redaction logic and available bypasses.
2.  `core/src/transport/swarm.rs` (lines 3936-4010, 5668, 6224) - To verify reciprocal exchange trigger conditions and network mode context.
3.  `core/src/identity/keys.rs` - To verify identity canonicalization parity.
4.  Android/Kotlin source files - To verify rendering data source.