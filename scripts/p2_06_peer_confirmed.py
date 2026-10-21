import os
import re

# 1. Update ratchet.rs
r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

r = r.replace('    pub transcript_hash: Option<[u8; 32]>,', '    pub transcript_hash: Option<[u8; 32]>,\n    /// Flag indicating if the peer has proven they established the session.\n    pub peer_confirmed: bool,')

# reconstruct
r = r.replace('        transcript_hash: Option<[u8; 32]>,', '        transcript_hash: Option<[u8; 32]>,\n        peer_confirmed: bool,')
r = r.replace('            transcript_hash,\n        }', '            transcript_hash,\n            peer_confirmed,\n        }')

# init_as_sender
r = r.replace('            transcript_hash: None,\n        })', '            transcript_hash: None,\n            peer_confirmed: false,\n        })')

# init_as_receiver
r = r.replace('            transcript_hash: None,\n        })', '            transcript_hash: None,\n            peer_confirmed: true,\n        })') # receivers already confirmed their own derivation

# init_as_sender_hybrid
r = r.replace('            transcript_hash: Some(transcript_hash),\n        })', '            transcript_hash: Some(transcript_hash),\n            peer_confirmed: false,\n        })')

# init_as_receiver_hybrid
r = r.replace('            transcript_hash: Some(transcript_hash),\n        })', '            transcript_hash: Some(transcript_hash),\n            peer_confirmed: true,\n        })')

# handle_dh_ratchet (which means peer successfully decrypted our message and responded, or vice versa, either way we received a ratcheted message!)
# wait, actually peer_confirmed should be set to true on successful decrypt!
r = r.replace('        let message_key = self.get_message_key(&their_dh, message_number)?;\n\n        let nonce_obj = XNonce::from_slice(nonce);', '        let message_key = self.get_message_key(&their_dh, message_number)?;\n\n        // The peer sent us a valid ratcheted message, they have established the session.\n        self.peer_confirmed = true;\n\n        let nonce_obj = XNonce::from_slice(nonce);')

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

# 2. Update session_manager.rs
sm_path = 'core/src/crypto/session_manager.rs'
with open(sm_path, 'r', encoding='utf-8') as f:
    sm = f.read()

sm = sm.replace('    pub transcript_hash_hex: Option<String>,', '    pub transcript_hash_hex: Option<String>,\n    #[serde(default)]\n    pub peer_confirmed: bool,')

sm = sm.replace('            transcript_hash_hex: session.transcript_hash.map(hex::encode),\n        }', '            transcript_hash_hex: session.transcript_hash.map(hex::encode),\n            peer_confirmed: session.peer_confirmed,\n        }')

sm = sm.replace('            self.negotiated_suite,\n            transcript_hash,\n        ))', '            negotiated_suite,\n            transcript_hash,\n            self.peer_confirmed,\n        ))')

with open(sm_path, 'w', encoding='utf-8') as f:
    f.write(sm)

print("ratchet.rs and session_manager.rs updated with peer_confirmed")
