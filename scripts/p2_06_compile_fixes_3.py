import os

# 1. Fix require_pq in wasm/src/lib.rs
wasm_path = "wasm/src/lib.rs"
if os.path.exists(wasm_path):
    with open(wasm_path, "r", encoding="utf-8") as f:
        w = f.read()
    w = w.replace('scmessenger_core::MeshSettings {', 'scmessenger_core::MeshSettings {\n            require_pq: false,')
    with open(wasm_path, "w", encoding="utf-8") as f:
        f.write(w)

# 2. Add bootstrap_hct to RatchetSession and SerializableRatchetSession
ratchet_path = "core/src/crypto/ratchet.rs"
with open(ratchet_path, "r", encoding="utf-8") as f:
    r = f.read()

r = r.replace('pub peer_confirmed: bool,', 'pub peer_confirmed: bool,\n    /// Hybrid ciphertext for session bootstrap (stored until peer confirmed).\n    pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,')

r = r.replace('peer_confirmed: bool,', 'peer_confirmed: bool,\n        bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,')

r = r.replace('peer_confirmed,\n        }', 'peer_confirmed,\n            bootstrap_hct,\n        }')

r = r.replace('peer_confirmed: false,\n        })', 'peer_confirmed: false,\n            bootstrap_hct: None,\n        })')

r = r.replace('peer_confirmed: true,\n        })', 'peer_confirmed: true,\n            bootstrap_hct: None,\n        })')

r = r.replace('            bootstrap_hct: None,\n        })', '            bootstrap_hct: Some(hct),\n        })', 1) # Only for init_as_sender_hybrid! Wait, I should make sure it replaces the right one.
# Re-read: I did a blanket replace of `peer_confirmed: false, \n })` -> `bootstrap_hct: None, })`.
# I should fix init_as_sender_hybrid to set Some(hct).
with open(ratchet_path, "w", encoding="utf-8") as f:
    f.write(r)

import re
with open(ratchet_path, "r", encoding="utf-8") as f:
    r = f.read()
r = re.sub(r'(pub fn init_as_sender_hybrid.*?peer_confirmed: false,\n            )bootstrap_hct: None', r'\1bootstrap_hct: Some(hct)', r, flags=re.DOTALL)
r = r.replace('our_signing_key: &ed25519_dalek::SigningKey,', '_our_signing_key: &ed25519_dalek::SigningKey,', 1)
r = r.replace('sender_bundle: &crate::identity::PublicKeyBundle,', '_sender_bundle: &crate::identity::PublicKeyBundle,', 1)
with open(ratchet_path, "w", encoding="utf-8") as f:
    f.write(r)

# 3. Add to session_manager.rs
sm_path = "core/src/crypto/session_manager.rs"
with open(sm_path, "r", encoding="utf-8") as f:
    sm = f.read()

sm = sm.replace('pub peer_confirmed: bool,', 'pub peer_confirmed: bool,\n    pub bootstrap_hct_ephemeral_hex: Option<String>,\n    pub bootstrap_hct_mlkem_hex: Option<String>,')

sm = sm.replace('peer_confirmed: session.peer_confirmed,', 'peer_confirmed: session.peer_confirmed,\n            bootstrap_hct_ephemeral_hex: session.bootstrap_hct.as_ref().map(|h| hex::encode(h.x25519_ephemeral_public)),\n            bootstrap_hct_mlkem_hex: session.bootstrap_hct.as_ref().map(|h| hex::encode(h.mlkem_ciphertext)),')

into_session = """
        let bootstrap_hct = if let (Some(e_hex), Some(m_hex)) = (self.bootstrap_hct_ephemeral_hex, self.bootstrap_hct_mlkem_hex) {
            let e_bytes = hex::decode(e_hex).map_err(|e| anyhow::anyhow!("bad eph: {}", e))?;
            let m_bytes = hex::decode(m_hex).map_err(|e| anyhow::anyhow!("bad mlkem: {}", e))?;
            if e_bytes.len() != 32 || m_bytes.len() != 1088 { anyhow::bail!("bad hct lengths"); }
            let mut e_arr = [0u8; 32];
            e_arr.copy_from_slice(&e_bytes);
            let mut m_arr = [0u8; 1088];
            m_arr.copy_from_slice(&m_bytes);
            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                x25519_ephemeral_public: e_arr,
                mlkem_ciphertext: m_arr,
            })
        } else {
            None
        };
"""

sm = sm.replace('        Ok(RatchetSession::reconstruct(', into_session + '\n        Ok(RatchetSession::reconstruct(')
sm = sm.replace('self.peer_confirmed,\n        ))', 'self.peer_confirmed,\n            bootstrap_hct,\n        ))')

with open(sm_path, "w", encoding="utf-8") as f:
    f.write(sm)

print("ratchet.rs, session_manager.rs, wasm/src/lib.rs updated")
