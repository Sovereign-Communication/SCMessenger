import os
import re

# 1. Update ratchet.rs
r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

r = r.replace('    pub transcript_hash: Option<[u8; 32]>,', '    pub transcript_hash: Option<[u8; 32]>,\n    /// Flag indicating if the peer has proven they established the session.\n    pub peer_confirmed: bool,\n    /// Hybrid ciphertext for session bootstrap (stored until peer confirmed).\n    pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,')

# reconstruct
r = r.replace('        transcript_hash: Option<[u8; 32]>,', '        transcript_hash: Option<[u8; 32]>,\n        peer_confirmed: bool,\n        bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,')
r = r.replace('            transcript_hash,\n        }', '            transcript_hash,\n            peer_confirmed,\n            bootstrap_hct,\n        }')

# init_as_sender
r = r.replace('            transcript_hash: None,\n        })', '            transcript_hash: None,\n            peer_confirmed: false,\n            bootstrap_hct: None,\n        })')

# init_as_receiver
r = r.replace('            transcript_hash: None,\n        })', '            transcript_hash: None,\n            peer_confirmed: true,\n            bootstrap_hct: None,\n        })')

# init_as_sender_hybrid
r = r.replace('            transcript_hash: Some(transcript_hash),\n        })', '            transcript_hash: Some(transcript_hash),\n            peer_confirmed: false,\n            bootstrap_hct: Some(hct),\n        })')
r = r.replace('our_signing_key: &ed25519_dalek::SigningKey,', '_our_signing_key: &ed25519_dalek::SigningKey,', 1)

# init_as_receiver_hybrid
r = r.replace('            transcript_hash: Some(transcript_hash),\n        })', '            transcript_hash: Some(transcript_hash),\n            peer_confirmed: true,\n            bootstrap_hct: None,\n        })')

# handle_dh_ratchet (peer successfully decrypted our message and responded)
r = r.replace('        let message_key = self.get_message_key(&their_dh, message_number)?;\n\n        let nonce_obj = XNonce::from_slice(nonce);', '        let message_key = self.get_message_key(&their_dh, message_number)?;\n\n        // The peer sent us a valid ratcheted message, they have established the session.\n        self.peer_confirmed = true;\n\n        let nonce_obj = XNonce::from_slice(nonce);')

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

# 2. Update session_manager.rs
sm_path = 'core/src/crypto/session_manager.rs'
with open(sm_path, 'r', encoding='utf-8') as f:
    sm = f.read()

sm = sm.replace('    pub transcript_hash_hex: Option<String>,', '    pub transcript_hash_hex: Option<String>,\n    #[serde(default)]\n    pub peer_confirmed: bool,\n    pub bootstrap_hct_ephemeral_hex: Option<String>,\n    pub bootstrap_hct_mlkem_hex: Option<String>,')

sm = sm.replace('            transcript_hash_hex: session.transcript_hash.map(hex::encode),\n        }', '            transcript_hash_hex: session.transcript_hash.map(hex::encode),\n            peer_confirmed: session.peer_confirmed,\n            bootstrap_hct_ephemeral_hex: session.bootstrap_hct.as_ref().map(|h| hex::encode(h.x25519_ephemeral_public)),\n            bootstrap_hct_mlkem_hex: session.bootstrap_hct.as_ref().map(|h| hex::encode(&h.mlkem_ciphertext)),\n        }')

into_session = """
        let bootstrap_hct = if let (Some(e_hex), Some(m_hex)) = (self.bootstrap_hct_ephemeral_hex, self.bootstrap_hct_mlkem_hex) {
            let e_bytes = hex::decode(e_hex).map_err(|e| anyhow::anyhow!("bad eph: {}", e))?;
            let m_bytes = hex::decode(m_hex).map_err(|e| anyhow::anyhow!("bad mlkem: {}", e))?;
            if e_bytes.len() != 32 || m_bytes.len() != 1088 { anyhow::bail!("bad hct lengths"); }
            let mut e_arr = [0u8; 32];
            e_arr.copy_from_slice(&e_bytes);
            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                x25519_ephemeral_public: e_arr,
                mlkem_ciphertext: m_bytes,
            })
        } else {
            None
        };
"""

sm = sm.replace('        Ok(RatchetSession::reconstruct(', into_session + '\n        Ok(RatchetSession::reconstruct(')
sm = sm.replace('self.negotiated_suite,\n            transcript_hash,\n        ))', 'self.negotiated_suite,\n            transcript_hash,\n            self.peer_confirmed,\n            bootstrap_hct,\n        ))')

with open(sm_path, 'w', encoding='utf-8') as f:
    f.write(sm)

print("ratchet.rs and session_manager.rs updated with bootstrap_hct.")
