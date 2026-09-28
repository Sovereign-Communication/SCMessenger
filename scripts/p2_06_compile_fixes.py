import os
import re

# 1. Fix ratchet.rs
r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

r = r.replace('crate::crypto::PublicKeyBundle', 'crate::identity::PublicKeyBundle')
r = r.replace('crate::crypto::hybrid_encapsulate', 'crate::crypto::pq::hybrid::hybrid_encapsulate')
r = r.replace('crate::crypto::HybridCiphertext', 'crate::crypto::pq::hybrid::HybridCiphertext')
r = r.replace('crate::crypto::hybrid_decapsulate', 'crate::crypto::pq::hybrid::hybrid_decapsulate')

r = r.replace('let (new_root_key, sending_chain_key) = root_key_ratchet(&root_key_0, dh_output.as_bytes());', 'let (new_root_key, sending_chain_key) = root_key_ratchet(&RatchetKey::from_bytes(root_key_0), dh_output.as_bytes());')
r = r.replace('            root_key: root_key_0,', '            root_key: RatchetKey::from_bytes(root_key_0),')

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

# 2. Fix session_manager.rs
sm_path = 'core/src/crypto/session_manager.rs'
with open(sm_path, 'r', encoding='utf-8') as f:
    sm = f.read()

# Fix self borrow
load_fn = """    pub fn load(&mut self) -> Result<()> {
        let backend = if let Some(b) = &self.backend {
            b.clone()
        } else {
            return Ok(());
        };

        let mut loaded = false;
        // Try v2
        if let Some(bytes) = backend
            .get(b"ratchet_sessions_v2")
            .map_err(|e| anyhow::anyhow!("Failed to load v2 sessions: {}", e))?
        {
            let json = String::from_utf8(bytes)
                .map_err(|e| anyhow::anyhow!("Invalid ratchet session encoding: {}", e))?;
            self.deserialize_sessions(&json)?;
            loaded = true;
        }
        
        // Try v1 for migration
        if let Some(bytes) = backend
            .get(b"ratchet_sessions_v1")
            .map_err(|e| anyhow::anyhow!("Failed to load v1 sessions: {}", e))?
        {
            let json = String::from_utf8(bytes)
                .map_err(|e| anyhow::anyhow!("Invalid ratchet session encoding: {}", e))?;
            self.deserialize_sessions(&json)?;
            if !loaded {
                // Trigger save to persist v2
                let _ = self.save();
            }
        }
        
        Ok(())
    }"""

sm = re.sub(r'    pub fn load\(&mut self\) -> Result<\(\)> \{.*?        Ok\(\(\)\)\n    \}', load_fn, sm, flags=re.DOTALL)

# Fix negotiated_suite error
sm = sm.replace('let negotiated_suite = negotiated_suite.or(Some(0x01)); // Default to 0x01 for v1 migration', 'let negotiated_suite = self.negotiated_suite.or(Some(0x01)); // Default to 0x01 for v1 migration')

with open(sm_path, 'w', encoding='utf-8') as f:
    f.write(sm)

print("fixes applied.")
