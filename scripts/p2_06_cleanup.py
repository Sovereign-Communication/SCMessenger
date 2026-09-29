import os

# Let's cleanly patch ratchet.rs
r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

# Fix double declaration of bootstrap_hct
# Wait, let's just restore ratchet.rs to its state before my previous hacky scripts, or just fix it in place.
# Let's fix in place.
import re

# 1. Remove the first duplicate bootstrap_hct field
r = re.sub(r'    peer_confirmed: bool,\n        bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,\n', '    peer_confirmed: bool,\n', r, count=1)
# Actually, the struct is `pub peer_confirmed: bool`. The arguments in `reconstruct` are `peer_confirmed: bool,`
# Let's see:
# error[E0124]: field `bootstrap_hct` is already declared
#   --> core\src\crypto\ratchet.rs:144:5
#    |
# 142 |         bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,
#    |         ------------------------------------------------------------------ `bootstrap_hct` first declared here
# 143 |     /// Hybrid ciphertext for session bootstrap (stored until peer confirmed).
# 144 |     pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,

# Oh! So line 142 was `bootstrap_hct` and 144 was `pub bootstrap_hct`. Let's just fix it by replacing the whole struct definition.
struct_def = r"""pub struct RatchetSession {
    /// Our current DH ratchet keypair (sending side).
    our_dh_secret: X25519StaticSecret,
    /// Our current DH ratchet public key.
    our_dh_public: X25519PublicKey,
    /// Their current DH ratchet public key (receiving side).
    their_dh_public: Option<X25519PublicKey>,
    /// Root key — updated on every DH ratchet step.
    root_key: RatchetKey,
    /// Our sending chain (None until DH ratchet is initialized).
    sending_chain: Option<Chain>,
    /// Our receiving chain (None until we receive their DH ratchet key).
    receiving_chain: Option<Chain>,
    /// Number of DH ratchet steps performed.
    dh_step_count: u32,
    /// Skipped message keys: (their_dh_public_bytes, message_number) → message_key.
    skipped_keys: HashMap<([u8; 32], u32), RatchetKey>,
    /// Whether this session has been initialized (we've received their DH key).
    initialized: bool,
    /// Our X25519 identity secret (for first DH ratchet step on receiver side).
    /// Kept only until the first DH ratchet is performed, then zeroized.
    our_identity_secret: Option<X25519StaticSecret>,
    /// The negotiated cryptographic suite (from PQC-04).
    pub negotiated_suite: Option<u8>,
    /// The transcript hash binding the session to the negotiation (from PQC-04).
    pub transcript_hash: Option<[u8; 32]>,
    /// Flag indicating if the peer has proven they established the session.
    pub peer_confirmed: bool,
    /// Hybrid ciphertext for session bootstrap (stored until peer confirmed).
    pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,
}"""

r = re.sub(r'pub struct RatchetSession \{.*?\}', struct_def, r, flags=re.DOTALL)

# Reconstruct args
reconstruct_args = r"""    pub(crate) fn reconstruct(
        our_dh_secret: X25519StaticSecret,
        our_dh_public: X25519PublicKey,
        their_dh_public: Option<X25519PublicKey>,
        root_key: RatchetKey,
        sending_chain: Option<Chain>,
        receiving_chain: Option<Chain>,
        dh_step_count: u32,
        initialized: bool,
        our_identity_secret: Option<X25519StaticSecret>,
        negotiated_suite: Option<u8>,
        transcript_hash: Option<[u8; 32]>,
        peer_confirmed: bool,
        bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,
    )"""

r = re.sub(r'    pub\(crate\) fn reconstruct\(.*?\)', reconstruct_args, r, flags=re.DOTALL)

# Reconstruct body
reconstruct_body = r"""        Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public,
            root_key,
            sending_chain,
            receiving_chain,
            dh_step_count,
            skipped_keys: HashMap::new(),
            initialized,
            our_identity_secret,
            negotiated_suite,
            transcript_hash,
            peer_confirmed,
            bootstrap_hct,
        }"""
r = re.sub(r'        Self \{.*?        \}', reconstruct_body, r, flags=re.DOTALL)

# Fix init_as_sender (has `Some(hct)` and `_our_signing_key` mistakes)
def fix_init_as_sender(m):
    return m.group(0).replace('bootstrap_hct: Some(hct)', 'bootstrap_hct: None').replace('_our_signing_key', 'our_signing_key')
r = re.sub(r'    pub fn init_as_sender\(.*?\n    \}', fix_init_as_sender, r, flags=re.DOTALL)

def fix_init_as_receiver(m):
    return m.group(0).replace('bootstrap_hct: Some(hct)', 'bootstrap_hct: None').replace('_our_signing_key', 'our_signing_key')
r = re.sub(r'    pub fn init_as_receiver\(.*?\n    \}', fix_init_as_receiver, r, flags=re.DOTALL)

def fix_init_as_sender_hybrid(m):
    text = m.group(0)
    text = text.replace('our_signing_key: &ed25519_dalek::SigningKey,', '_our_signing_key: &ed25519_dalek::SigningKey,')
    if 'bootstrap_hct: None' in text:
        text = text.replace('bootstrap_hct: None', 'bootstrap_hct: Some(hct)')
    return text
r = re.sub(r'    pub fn init_as_sender_hybrid\(.*?\n    \}', fix_init_as_sender_hybrid, r, flags=re.DOTALL)

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

# Fix session_manager.rs: m_arr.to_vec()
sm_path = 'core/src/crypto/session_manager.rs'
with open(sm_path, 'r', encoding='utf-8') as f:
    sm = f.read()

sm = sm.replace('mlkem_ciphertext: m_arr,', 'mlkem_ciphertext: m_arr.to_vec(),')

with open(sm_path, 'w', encoding='utf-8') as f:
    f.write(sm)

print("ratchet.rs and session_manager.rs cleaned up.")
