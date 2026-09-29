import os

r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

# Fix concat
r = r.replace('&ss_hybrid.concat(&transcript_hash)', '[&ss_hybrid.as_bytes()[..], &transcript_hash[..]].concat()')

# Fix diffie_hellman
r = r.replace('our_dh_secret.diffie_hellman(&their_bundle.x25519_public)', 'our_dh_secret.diffie_hellman(&X25519PublicKey::from(their_bundle.x25519_public))')

# Fix their_dh_public conversions
r = r.replace('their_dh_public: Some(their_bundle.x25519_public)', 'their_dh_public: Some(X25519PublicKey::from(their_bundle.x25519_public))')
r = r.replace('their_dh_public: Some(hct.x25519_ephemeral_public)', 'their_dh_public: Some(X25519PublicKey::from(hct.x25519_ephemeral_public))')

# Fix hybrid_decapsulate call
# Current:
#         let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
#             &our_signing_key,
#             &sender_bundle.mlkem_encaps_key,
#             hct,
#         )?;
# New:
#         let our_x25519_secret = super::encrypt::ed25519_to_x25519_secret(our_signing_key);
#         let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
#             &our_x25519_secret,
#             our_mlkem_keypair,
#             hct,
#         )?;
new_decap = """        let our_x25519_secret = super::encrypt::ed25519_to_x25519_secret(our_signing_key);
        let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
            &our_x25519_secret,
            our_mlkem_keypair,
            hct,
        )?;"""
r = r.replace("""        let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
            &our_signing_key,
            &sender_bundle.mlkem_encaps_key,
            hct,
        )?;""", new_decap)

# Add our_mlkem_keypair to init_as_receiver_hybrid
r = r.replace("""    pub fn init_as_receiver_hybrid(
        our_signing_key: &ed25519_dalek::SigningKey,
        sender_bundle: &crate::identity::PublicKeyBundle,""", """    pub fn init_as_receiver_hybrid(
        our_signing_key: &ed25519_dalek::SigningKey,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        sender_bundle: &crate::identity::PublicKeyBundle,""")

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

print("ratchet.rs compile fixes applied.")
