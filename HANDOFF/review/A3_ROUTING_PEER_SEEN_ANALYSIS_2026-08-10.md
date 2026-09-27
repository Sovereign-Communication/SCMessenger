<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

[OK] 1. `routing_peer_seen` has no production caller. Search method: identifier search for `routing_peer_seen` across core, transport, mobile, and bridge trees. The only non-generated hits are the definition at `core/src/iron_core.rs:2571` and a comment at `core/src/routing/optimized_engine.rs:310`. No call exists in `core/src/transport/swarm.rs` or `core/src/mobile_bridge.rs`.

[FAIL] 2. The intended seam is the swarm connection-established path, not the mobile bridge. `routing_peer_seen` requires both a peer and a transport at `core/src/iron_core.rs:2571`. The swarm already observes transport-scoped peer activity and calls `record_message_activity` at `core/src/transport/swarm.rs:3698`, but it does not call `routing_peer_seen`. A mobile-bridge discovery callback is not the correct source because it is not authoritative for transport connectivity.

[WARNING] 3. Feeding raw connection events directly into `LocalCell` can violate the stated invariant: LocalCell “intentionally updates only peers already known to the local topology; an announcement cannot create a peer” (`core/src/iron_core.rs:2588`). The correct seam is: first prove the peer is already admitted to local topology, then allow `peer_seen` / hint refresh. Connection events alone must not create routable state.

[FAIL] 4. If any peer that can open a connection can enter the routing table, it can become a direct candidate at `core/src/routing/engine.rs:162` and later raise its reliability through successful local delivery signals. That can attract traffic. Concrete trust gate: before a peer is routable, prove key possession and local admission — e.g. an existing contact/registered custody relationship — and fail closed if blocked. The proposed diff enforces contact-or-known-public-key admission and blocked-peer fail-closed behavior.

[WARNING] 5. Yes. The zero-confidence fallbacks are `core/src/routing/engine.rs:211` and `core/src/routing/engine.rs:220`. If confidence rises and the decision becomes direct at `core/src/routing/engine.rs:162`, message flow can move out of the `StoreAndCarry` branch and therefore skip any custody accounting that is only triggered for `StoreAndCarry`. Custody accounting should be tied to actual relay custody, not merely the routing fallback.

```diff
--- a/core/src/iron_core.rs
+++ b/core/src/iron_core.rs
@@ -1878,9 +1878,10 @@
                 true
             }
         };
         if blocked {
             return;
         }
+        self.routing_peer_seen(peer_id.clone(), String::new());
         if let Some(delegate) = self.delegate.read().as_ref() {
             delegate.on_peer_discovered(peer_id.clone());
         }
@@ -2568,12 +2569,69 @@
     /// Record that a peer was seen on a given transport.
     pub fn routing_peer_seen(&self, peer_id_hex: String, _transport: String) {
-        if let Some(engine) = self.routing_engine.write().as_mut() {
-            engine.record_message_activity(&peer_id_hex);
+        // [OK] Refresh routing only for peers already admitted to the local
+        // topology. Raw connectivity must not create a routable peer.
+        let blocked = match self
+            .blocked_manager
+            .read()
+            .is_blocked_resolved(&peer_id_hex, None)
+        {
+            Ok(blocked) => blocked,
+            Err(_) => true,
+        };
+        if blocked {
+            return;
+        }
+
+        let admitted = {
+            let contacts = self.contact_manager.read();
+            contacts
+                .get(peer_id_hex.clone())
+                .ok()
+                .flatten()
+                .or_else(|| contacts.get_by_public_key(&peer_id_hex).ok().flatten())
+                .is_some()
+        };
+        if !admitted {
+            return;
+        }
+
+        let Ok(peer_bytes) = hex::decode(&peer_id_hex) else {
+            return;
+        };
+        let Ok(peer_id) = <[u8; 32]>::try_from(peer_bytes.as_slice()) else {
+            return;
+        };
+
+        let transport = match _transport.to_ascii_lowercase().as_str() {
+            "ble" => crate::routing::TransportType::BLE,
+            "wifi_direct" => crate::routing::TransportType::WiFiDirect,
+            "wifi_aware" => crate::routing::TransportType::WiFiAware,
+            "tcp" => crate::routing::TransportType::TCP,
+            "quic" => crate::routing::TransportType::QUIC,
+            _ => crate::routing::TransportType::BLE,
+        };
+
+        let hint: [u8; 4] = blake3::hash(&peer_bytes).as_bytes()[0..4]
+            .try_into()
+            .unwrap_or([0u8; 4]);
+
+        if let Some(engine) = self.routing_engine.write().as_mut() {
+            engine.record_message_activity(&peer_id_hex);
+            engine
+                .base_engine_mut()
+                .local_cell_mut()
+                .peer_seen(peer_id, transport);
+            engine
+                .base_engine_mut()
+                .local_cell_mut()
+                .update_peer_hints(&peer_id, vec![hint]);
         }
     }

     /// Update peer hint vectors for routing table.
     pub fn routing_update_peer_hints(&self, peer_id_hex: String, hints: Vec<Vec<u8>>) {
```

---ORCHESTRATION_METADATA---
RESULT: DONE
VERIFICATION: NONE
FILES: ["core/src/iron_core.rs"]
NOTES: ["The correct production caller should ultimately be the swarm connection-established handler with the real transport; this diff adds a safe core-side seam and blocks untrusted table insertion.", "No dial ordering, relay candidate construction, or address filtering was changed."]
---END---