import os
import re

e_path = 'core/src/crypto/encrypt.rs'
with open(e_path, 'r', encoding='utf-8') as f:
    e = f.read()

# 1. Update encrypt_message_ratcheted to generate EnvelopeV2 if suite == 0x02
# Current encrypt_message_ratcheted:
# pub fn encrypt_message_ratcheted(
#     sender_signing_key: &SigningKey,
#     session: &mut crate::crypto::RatchetSession,
#     plaintext: &[u8],
# ) -> Result<crate::message::Envelope> {

e = e.replace("""pub fn encrypt_message_ratcheted(
    sender_signing_key: &SigningKey,
    session: &mut crate::crypto::RatchetSession,
    plaintext: &[u8],
) -> Result<crate::message::Envelope> {""", """pub fn encrypt_message_ratcheted(
    sender_signing_key: &SigningKey,
    session: &mut crate::crypto::RatchetSession,
    plaintext: &[u8],
) -> Result<crate::message::WireEnvelope> {""")

v2_body = """    let sender_public_bytes = sender_signing_key.verifying_key().to_bytes();
    let result = session.encrypt(plaintext, &sender_public_bytes)?;

    if session.negotiated_suite == Some(0x02) {
        let (pq_kem_ciphertext, pq_encaps_key) = if !session.peer_confirmed {
            if let Some(hct) = &session.bootstrap_hct {
                (Some(hct.mlkem_ciphertext.clone()), None) // pq_encaps_key comes in later tasks (PQC-07)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let ephemeral_public_key = if !session.peer_confirmed {
            if let Some(hct) = &session.bootstrap_hct {
                hct.x25519_ephemeral_public.to_vec()
            } else {
                result.our_dh_public.to_vec()
            }
        } else {
            result.our_dh_public.to_vec()
        };

        let transcript_hash = if !session.peer_confirmed {
            session.transcript_hash.map(|h| h.to_vec())
        } else {
            None
        };

        Ok(crate::message::WireEnvelope::V2(crate::message::EnvelopeV2 {
            suite: 0x02,
            sender_public_key: sender_public_bytes.to_vec(),
            ephemeral_public_key,
            nonce: result.nonce,
            ciphertext: result.ciphertext,
            ratchet_dh_public: Some(result.our_dh_public.to_vec()),
            ratchet_message_number: Some(result.message_number),
            pq_kem_ciphertext,
            pq_encaps_key,
            transcript_hash,
        }))
    } else {
        Ok(crate::message::WireEnvelope::V1(crate::message::Envelope {
            sender_public_key: sender_public_bytes.to_vec(),
            ephemeral_public_key: result.our_dh_public.to_vec(),
            nonce: result.nonce,
            ciphertext: result.ciphertext,
            ratchet_dh_public: Some(result.our_dh_public.to_vec()),
            ratchet_message_number: Some(result.message_number),
        }))
    }
}
"""

e = re.sub(r'    let sender_public_bytes = sender_signing_key.*?ratchet_message_number: Some\(result\.message_number\),\n    \}\)\n\}', v2_body, e, flags=re.DOTALL)

# 2. Update encrypt_with_ratchet_fallback
e = e.replace("""pub fn encrypt_with_ratchet_fallback(
    sender_signing_key: &SigningKey,
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    peer_id: &str,
) -> Result<crate::message::Envelope> {""", """pub fn encrypt_with_ratchet_fallback(
    sender_signing_key: &SigningKey,
    recipient_bundle: Option<&crate::identity::PublicKeyBundle>,
    recipient_public_key_fallback: &[u8; 32], // Legacy path for V1
    plaintext: &[u8],
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    peer_id: &str,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
) -> Result<crate::message::WireEnvelope> {""")

fb_body = """    if let Some(manager) = session_manager {
        // If we have bundles for both sides, try hybrid session
        if let (Some(our_b), Some(their_b)) = (our_bundle, recipient_bundle) {
            let session = manager.get_or_create_session_hybrid(
                peer_id,
                sender_signing_key,
                our_b,
                their_b,
            )?;
            return encrypt_message_ratcheted(sender_signing_key, session, plaintext);
        } else {
            // Fallback to classical V1
            let their_x25519 = crate::crypto::encrypt::ed25519_public_to_x25519(recipient_public_key_fallback)?;
            let session = manager.get_or_create_session(
                peer_id,
                sender_signing_key,
                &their_x25519,
            )?;
            return encrypt_message_ratcheted(sender_signing_key, session, plaintext);
        }
    }

    Ok(crate::message::WireEnvelope::V1(encrypt_message(sender_signing_key, recipient_public_key_fallback, plaintext)?))
}
"""

e = re.sub(r'    if let Some\(manager\) = session_manager \{.*?    encrypt_message\(sender_signing_key, recipient_public_key, plaintext\)\n\}', fb_body, e, flags=re.DOTALL)

# 3. Update decrypt_with_ratchet_fallback
e = e.replace("""pub fn decrypt_with_ratchet_fallback(
    recipient_signing_key: &SigningKey,
    envelope: &crate::message::Envelope,
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
) -> Result<Vec<u8>> {""", """pub fn decrypt_with_ratchet_fallback(
    recipient_signing_key: &SigningKey,
    wire_envelope: &crate::message::WireEnvelope,
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    our_mlkem_keypair: Option<&crate::crypto::pq::MlKem768KeyPair>,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
    sender_bundle: Option<&crate::identity::PublicKeyBundle>,
) -> Result<Vec<u8>> {""")

dec_body = """    match wire_envelope {
        crate::message::WireEnvelope::V1(envelope) => {
            if is_ratcheted_envelope(envelope) {
                if let Some(manager) = session_manager {
                    let peer_id = hex::encode(&envelope.sender_public_key);
                    
                    let session = if !manager.has_session(&peer_id) {
                        // Create a classical receiver session if it doesn't exist
                        let mut sender_ed = [0u8; 32];
                        sender_ed.copy_from_slice(&envelope.sender_public_key);
                        let sender_x25519 = crate::crypto::encrypt::ed25519_public_to_x25519(&sender_ed)?;
                        manager.create_receiver_session(&peer_id, recipient_signing_key, &sender_x25519)?
                    } else {
                        manager.get_session_mut(&peer_id).unwrap()
                    };
                    
                    return decrypt_message_ratcheted(session, envelope);
                }
                bail!("Ratcheted V1 envelope received but no active ratchet session");
            }
            decrypt_message(recipient_signing_key, envelope)
        }
        crate::message::WireEnvelope::V2(envelope_v2) => {
            if let Some(manager) = session_manager {
                let peer_id = hex::encode(&envelope_v2.sender_public_key);
                
                let session = if !manager.has_session(&peer_id) {
                    if let (Some(our_k), Some(our_b), Some(their_b)) = (our_mlkem_keypair, our_bundle, sender_bundle) {
                        let hct = if let (Some(eph), Some(mlk)) = (&envelope_v2.ephemeral_public_key, &envelope_v2.pq_kem_ciphertext) {
                            let mut e = [0u8; 32]; e.copy_from_slice(eph);
                            let mut m = [0u8; 1088]; m.copy_from_slice(mlk);
                            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                                x25519_ephemeral_public: e,
                                mlkem_ciphertext: m.to_vec(),
                            })
                        } else {
                            None
                        };
                        
                        manager.create_receiver_session_hybrid(
                            &peer_id,
                            recipient_signing_key,
                            our_k,
                            our_b,
                            their_b,
                            hct.as_ref(),
                        )?
                    } else {
                        bail!("V2 ratcheted envelope received, but missing keys/bundles for hybrid init");
                    }
                } else {
                    manager.get_session_mut(&peer_id).unwrap()
                };
                
                return decrypt_message_ratcheted_v2(session, envelope_v2);
            }
            bail!("V2 envelope received but no session manager available");
        }
    }
}
"""

e = re.sub(r'    if is_ratcheted_envelope\(envelope\) \{.*?    decrypt_message\(recipient_signing_key, envelope\)\n\}', dec_body, e, flags=re.DOTALL)

with open(e_path, 'w', encoding='utf-8') as f:
    f.write(e)

print("encrypt.rs updated.")
