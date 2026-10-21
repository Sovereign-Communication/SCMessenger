import os
import re

print("Updating session_manager.rs...")
sm_path = "core/src/crypto/session_manager.rs"
with open(sm_path, "r", encoding="utf-8") as f:
    sm = f.read()

# 1. Update save to use v2
sm = sm.replace('backend\n                .put(b"ratchet_sessions_v1", json.as_bytes())', 'backend\n                .put(b"ratchet_sessions_v2", json.as_bytes())')

# 2. Update load to try v2 first, then v1 with migration
load_fn = """    pub fn load(&mut self) -> Result<()> {
        if let Some(backend) = &self.backend {
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
                // deserialize_sessions will not overwrite existing sessions (so v2 takes precedence)
                // We'll hook into into_session to default to 0x01 if missing, since v1 implies suite 0x01.
                self.deserialize_sessions(&json)?;
                if !loaded {
                    // Trigger save to persist v2
                    let _ = self.save();
                }
            }
        }
        Ok(())
    }"""

sm = re.sub(r'    pub fn load\(&mut self\) -> Result<\(\)> \{.*?\n    \}', load_fn, sm, flags=re.DOTALL)

# Default to 0x01 for legacy v1 sessions in into_session
into_session_mod = """        let negotiated_suite = self.negotiated_suite.or(Some(0x01)); // Default to 0x01 for v1 migration
"""
sm = sm.replace("        Ok(RatchetSession::reconstruct(\n            our_dh_secret", into_session_mod + "\n        Ok(RatchetSession::reconstruct(\n            our_dh_secret")
sm = sm.replace("self.negotiated_suite", "negotiated_suite", 1) # only replace the one in the reconstruct args

# Add peer_confirmed to RatchetSession via PQC-06 requirements. Wait, PQC-06 says:
# "The bootstrap-fields-repeat logic lives at the session manager level (a flag peer_confirmed cleared once any inbound ratcheted envelope from the peer decrypts)."
# So we need to add `peer_confirmed: bool` to RatchetSession, SerializableRatchetSession, etc.
# Wait, let's just use replace_file_content for that if needed. Actually it's easier to rewrite the file completely, or just regex it.

with open(sm_path, "w", encoding="utf-8") as f:
    f.write(sm)

print("session_manager.rs updated.")
