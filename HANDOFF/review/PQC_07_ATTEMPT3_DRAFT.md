<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

```diff
--- a/core/src/crypto/ratchet.rs
+++ b/core/src/crypto/ratchet.rs
@@ -151,6 +151,9 @@ pub struct RatchetSession {
     pub pq_their_encaps_key: Option<Vec<u8>>,
     /// Pending ciphertext we must keep sending until peer acks.
     pub pq_pending_ct: Option<Vec<u8>>,
+    /// Pending PQ shared secret to mix into root key at next DH ratchet step.
+    /// Cleared after use in handle_dh_ratchet.
+    pub pending_pq_ss: Option<Vec<u8>>,
 }
 
 impl RatchetSession {
@@ -181,6 +184,7 @@ impl RatchetSession {
         pq_their_encaps_key: Option<Vec<u8>>,
         pq_pending_ct: Option<Vec<u8>>,
         skipped_keys: HashMap<([u8; 32], u32), RatchetKey>,
+        pending_pq_ss: Option<Vec<u8>>,
     ) -> Self {
         Self {
             our_dh_secret,
@@ -201,6 +205,7 @@ impl RatchetSession {
             pq_their_encaps_key,
             pq_pending_ct,
             skipped_keys,
+            pending_pq_ss,
         }
     }
 
@@ -605,15 +610,15 @@ impl RatchetSession {
         let their_encaps_key = self
             .pq_their_encaps_key
             .as_ref()
-            .ok_or_else(|| anyhow::anyhow!("No PQ encapsulation key from peer"))?;
+            .ok_or_else(|| anyhow::anyhow!("No PQ encapsulation key from peer (suite 0x02)"))?;
 
         // Encapsulate with their current key
         let (ct, ss_pq) = crate::crypto::pq::encapsulate(their_encaps_key)?;
 
         // Rotate our keypair: move current to previous, generate new
         self.pq_prev_keypair = self.pq_our_keypair.take();
-        self.pq_our_keypair = Some(crate::crypto::pq::generate());
+        self.pq_our_keypair.replace(crate::crypto::pq::generate());
         let new_encaps_key = self.pq_our_keypair.as_ref().unwrap().public_key().to_vec();
 
         // Store pending ciphertext until acked
@@ -621,6 +626,8 @@ impl RatchetSession {
 
         // Store the PQ shared secret for mixing at next DH ratchet step
         self.pending_pq_ss = Some(ss_pq.to_vec());
+
+        Ok((ct, new_encaps_key))
     }
 
     /// Handle incoming PQ fields during DH ratchet step (suite 0x02 only).
@@ -628,25 +635,25 @@ impl RatchetSession {
         pq_kem_ciphertext: &[u8],
         pq_encaps_key: &[u8],
     ) -> Result<Vec<u8>> {
-        if self.negotiated_suite != Some(0x02) {
-            bail!("PQ fields only expected for suite 0x02");
-        }
-
         // Try to decapsulate with current keypair first
         if let Some(ref our_keypair) = self.pq_our_keypair {
             match crate::crypto::pq::decapsulate(our_keypair, pq_kem_ciphertext) {
                 Ok(ss_pq) => {
                     // Success - update their encaps key and clear pending ct if any
                     self.pq_their_encaps_key = Some(pq_encaps_key.to_vec());
-                    self.pq_pending_ct = None;
-                    return Ok(ss_pq.to_vec());
+                    self.pq_pending_ct = None;
+                    // Store the PQ shared secret for mixing at next DH ratchet step
+                    self.pending_pq_ss = Some(ss_pq.to_vec());
+                    return Ok(());
                 }
                 Err(_) => {
                     // Fall through to try previous keypair
                 }
             }
         }
+        if self.negotiated_suite != Some(0x02) {
+            bail!("PQ fields only expected for suite 0x02 (no keypair match)");
+        }
 
         // Try with previous keypair (one-step-behind tolerance)
         if let Some(ref prev_keypair) = self.pq_prev_keypair {
@@ -655,10 +662,12 @@ impl RatchetSession {
                 Ok(ss_pq) => {
                     // Success with previous keypair
                     self.pq_their_encaps_key = Some(pq_encaps_key.to_vec());
-                    self.pq_pending_ct = None;
-                    return Ok(ss_pq.to_vec());
+                    self.pq_pending_ct = None;
+                    // Store the PQ shared secret for mixing at next DH ratchet step
+                    self.pending_pq_ss = Some(ss_pq.to_vec());
+                    return Ok(());
                 }
                 Err(_) => {
-                    // Both failed
+                    // Both keypairs failed to decapsulate
                 }
             }
         }
@@ -666,7 +675,7 @@ impl RatchetSession {
         bail!("Failed to decapsulate PQ ciphertext with either current or previous keypair")
     }
 
-    /// Validate that PQ fields are present when expected (suite 0x02 only).
+    /// Validate that PQ fields are present when expected (suite 0x02 only, post-bootstrap).
     pub fn validate_pq_fields_present(&self, has_pq_fields: bool) -> Result<()> {
         if self.negotiated_suite == Some(0x02)
             && self.pq_their_encaps_key.is_some()
@@ -705,11 +714,15 @@ impl RatchetSession {
         };
 
         let pq_ss = if self.negotiated_suite == Some(0x02) {
-            // Check if we have PQ fields available (should be provided externally)
-            // For now, we'll assume the PQ handling is done in the calling code
-            // This function just handles the DH part, PQ is handled separately
-            None
+            // Use any pending PQ shared secret from the last PQ ratchet step
+            self.pending_pq_ss.clone()
         } else {
+            // Non-PQ sessions never have PQ material
+            None
+        };
+
+        // Clear pending PQ secret after using it (idempotent if already None)
+        self.pending_pq_ss = None;
+
             None
         };
 
--- a/core/src/crypto/encrypt.rs
+++ b/core/src/crypto/encrypt.rs
@@ -313,10 +313,10 @@ pub fn decrypt_message_ratcheted_v2(
         }
     }
 
-    // Handle ongoing PQ ratchet fields (suite 0x02 only). The bootstrap
-    // ciphertext (first message, !peer_confirmed) is already consumed by
-    // init_as_receiver_hybrid at session setup -- only process pq fields
-    // here for post-confirmation messages, which represent a genuine PQ
+    // Handle ongoing PQ ratchet fields (suite 0x02 only, post-bootstrap).
+    // The bootstrap ciphertext (first message, !peer_confirmed) is already
+    // consumed by init_as_receiver_hybrid at session setup -- only process
+    // pq fields here for post-confirmation messages, which represent a genuine PQ
     // ratchet step from perform_pq_ratchet_step, not the initial bootstrap.
     if session.negotiated_suite == Some(0x02) && session.peer_confirmed {
         // The anti-stripping check only applies at genuine cadence boundaries
@@ -327,10 +327,9 @@ pub fn decrypt_message_ratcheted_v2(
         if expects_pq_fields {
             session.validate_pq_fields_present(envelope.pq_kem_ciphertext.is_some())?;
         }
-        if let (Some(pq_kem_ciphertext), Some(pq_encaps_key)) =
-            (&envelope.pq_kem_ciphertext, &envelope.pq_encaps_key)
-        {
-            session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)?;
+        if let (Some(pq_kem_ciphertext), Some(pq_encaps_key)) = (&envelope.pq_kem_ciphertext, &envelope.pq_encaps_key) {
+            // This stores the PQ shared secret for mixing at next DH ratchet step
+            session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)?;
         }
     }
 
```