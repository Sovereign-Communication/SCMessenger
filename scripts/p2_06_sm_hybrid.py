import os
import re

sm_path = 'core/src/crypto/session_manager.rs'
with open(sm_path, 'r', encoding='utf-8') as f:
    sm = f.read()

# Add get_or_create_session_hybrid
hybrid_methods = """
    /// Get or create a hybrid ratchet session for a peer (as sender).
    pub fn get_or_create_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
    ) -> Result<&mut RatchetSession> {
        if !self.sessions.contains_key(peer_id) {
            let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
                &our_bundle.supported_suites,
                &their_bundle.supported_suites,
                &our_bundle.ed25519_public,
                &their_bundle.ed25519_public,
            )?;

            let their_x25519 = x25519_dalek::PublicKey::from(their_bundle.x25519_public);
            let session = if suite == 0x02 {
                RatchetSession::init_as_sender_hybrid(
                    our_signing_key,
                    their_bundle,
                    hash,
                )?
            } else {
                let mut s = RatchetSession::init_as_sender(our_signing_key, &their_x25519)?;
                s.negotiated_suite = Some(suite);
                s.transcript_hash = Some(hash);
                s
            };

            self.sessions.insert(peer_id.to_string(), session);
        }
        Ok(self.sessions.get_mut(peer_id).expect("session just inserted"))
    }

    /// Create a receiver hybrid session.
    pub fn create_receiver_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
        hct_opt: Option<&crate::crypto::pq::hybrid::HybridCiphertext>,
    ) -> Result<&mut RatchetSession> {
        let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
            &their_bundle.supported_suites, // Initiator's suites
            &our_bundle.supported_suites,   // Responder's suites
            &their_bundle.ed25519_public,   // Initiator's ed25519
            &our_bundle.ed25519_public,     // Responder's ed25519
        )?;

        let sender_x25519 = x25519_dalek::PublicKey::from(their_bundle.x25519_public);
        let session = if suite == 0x02 {
            if let Some(hct) = hct_opt {
                RatchetSession::init_as_receiver_hybrid(
                    our_signing_key,
                    our_mlkem_keypair,
                    their_bundle,
                    hct,
                    hash,
                )?
            } else {
                anyhow::bail!("Suite 0x02 requires hybrid ciphertext for receiver initialization");
            }
        } else {
            let mut s = RatchetSession::init_as_receiver(our_signing_key, &sender_x25519)?;
            s.negotiated_suite = Some(suite);
            s.transcript_hash = Some(hash);
            s
        };

        self.sessions.insert(peer_id.to_string(), session);
        Ok(self.sessions.get_mut(peer_id).expect("session just inserted"))
    }
"""

sm = sm.replace('    /// Get an existing session for a peer (returns None if no session exists).', hybrid_methods + '\n    /// Get an existing session for a peer (returns None if no session exists).')

with open(sm_path, 'w', encoding='utf-8') as f:
    f.write(sm)

print("session_manager.rs updated with hybrid methods.")
