import os

test_code = """use scmessenger_core::crypto::{
    RatchetSessionManager,
    encrypt_with_ratchet_fallback,
    decrypt_with_ratchet_fallback,
};
use scmessenger_core::identity::{IdentityKeys, sign_bundle};
use scmessenger_core::message::WireEnvelope;

fn generate_identities() -> (IdentityKeys, scmessenger_core::identity::PublicKeyBundle, IdentityKeys, scmessenger_core::identity::PublicKeyBundle) {
    let alice = IdentityKeys::generate();
    let alice_bundle = sign_bundle(&alice).unwrap();
    let bob = IdentityKeys::generate();
    let bob_bundle = sign_bundle(&bob).unwrap();
    (alice, alice_bundle, bob, bob_bundle)
}

#[test]
fn test_pq_session_full_send_receive() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();
    
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    
    let plaintext1 = b"Hello Bob!";
    let bob_id = bob.identity_id();
    
    // Alice sends to Bob
    let envelope1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        plaintext1,
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle)
    ).unwrap();
    
    // Should be V2
    match &envelope1 {
        WireEnvelope::V2(v2) => {
            assert_eq!(v2.suite, 0x02);
            assert!(v2.pq_kem_ciphertext.is_some());
            assert!(v2.transcript_hash.is_some());
        }
        _ => panic!("Expected V2 envelope"),
    }
    
    // Bob decrypts
    let decrypted1 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        &envelope1,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle)
    ).unwrap();
    
    assert_eq!(decrypted1, plaintext1);
    
    // Bob sends to Alice
    let plaintext2 = b"Hi Alice!";
    let alice_id = alice.identity_id();
    
    let envelope2 = encrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&alice_bundle),
        &alice_bundle.ed25519_public,
        plaintext2,
        Some(&mut bob_manager),
        &alice_id,
        Some(&bob_bundle)
    ).unwrap();
    
    // Should be V2, but without PQ init fields because peer is confirmed
    match &envelope2 {
        WireEnvelope::V2(v2) => {
            assert_eq!(v2.suite, 0x02);
            assert!(v2.pq_kem_ciphertext.is_none());
            assert!(v2.transcript_hash.is_none());
        }
        _ => panic!("Expected V2 envelope"),
    }
    
    // Alice decrypts
    let decrypted2 = decrypt_with_ratchet_fallback(
        &alice.signing_key,
        &envelope2,
        Some(&mut alice_manager),
        Some(&alice.mlkem_keypair),
        Some(&alice_bundle),
        Some(&bob_bundle)
    ).unwrap();
    
    assert_eq!(decrypted2, plaintext2);
}

#[test]
fn test_pq_session_lost_first_envelope() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();
    
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();
    
    // Alice sends msg 1
    let env1 = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_bundle), &bob_bundle.ed25519_public,
        b"Msg 1", Some(&mut alice_manager), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    // Alice sends msg 2
    let env2 = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_bundle), &bob_bundle.ed25519_public,
        b"Msg 2", Some(&mut alice_manager), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    // Bob only receives msg 2 (msg 1 lost)
    // Since peer_confirmed is false, env2 still has the PQ fields!
    let dec2 = decrypt_with_ratchet_fallback(
        &bob.signing_key, &env2, Some(&mut bob_manager),
        Some(&bob.mlkem_keypair), Some(&bob_bundle), Some(&alice_bundle)
    ).unwrap();
    
    assert_eq!(dec2, b"Msg 2");
    
    // Bob receives msg 1 out of order
    let dec1 = decrypt_with_ratchet_fallback(
        &bob.signing_key, &env1, Some(&mut bob_manager),
        Some(&bob.mlkem_keypair), Some(&bob_bundle), Some(&alice_bundle)
    ).unwrap();
    
    assert_eq!(dec1, b"Msg 1");
}

#[test]
fn test_pq_session_transcript_mismatch() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();
    
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();
    
    // Alice sends
    let mut env = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_bundle), &bob_bundle.ed25519_public,
        b"Secret", Some(&mut alice_manager), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    // Tamper with transcript hash
    if let WireEnvelope::V2(v2) = &mut env {
        if let Some(hash) = &mut v2.transcript_hash {
            hash[0] ^= 0xFF;
        }
    }
    
    // Bob should reject
    let res = decrypt_with_ratchet_fallback(
        &bob.signing_key, &env, Some(&mut bob_manager),
        Some(&bob.mlkem_keypair), Some(&bob_bundle), Some(&alice_bundle)
    );
    assert!(res.is_err());
}

#[test]
fn test_v2_initiator_to_v1_peer() {
    let (alice, alice_bundle, bob, _bob_bundle_ignored) = generate_identities();
    
    // Create a V1 bundle for bob
    let mut bob_v1_bundle = _bob_bundle_ignored.clone();
    bob_v1_bundle.supported_suites = vec![0x01]; // only V1
    
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();
    
    // Alice initiates, but bob only supports V1
    let env = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_v1_bundle), &bob_v1_bundle.ed25519_public,
        b"Fallback", Some(&mut alice_manager), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    // Should fallback to V1
    match &env {
        WireEnvelope::V1(_) => {},
        _ => panic!("Expected V1 envelope fallback"),
    }
    
    // Bob decrypts (treating it as V1)
    let dec = decrypt_with_ratchet_fallback(
        &bob.signing_key, &env, Some(&mut bob_manager),
        None, None, None // Bob doesn't even have PQ bundles configured in this call
    ).unwrap();
    
    assert_eq!(dec, b"Fallback");
}

#[test]
fn test_pq_session_persistence() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();
    
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();
    
    // Alice sends
    let env = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_bundle), &bob_bundle.ed25519_public,
        b"Msg", Some(&mut alice_manager), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    decrypt_with_ratchet_fallback(
        &bob.signing_key, &env, Some(&mut bob_manager),
        Some(&bob.mlkem_keypair), Some(&bob_bundle), Some(&alice_bundle)
    ).unwrap();
    
    // Serialize and deserialize alice manager
    let json = alice_manager.serialize_sessions().unwrap();
    let mut new_alice = RatchetSessionManager::new();
    new_alice.deserialize_sessions(&json).unwrap();
    
    // Alice sends again
    let env2 = encrypt_with_ratchet_fallback(
        &alice.signing_key, Some(&bob_bundle), &bob_bundle.ed25519_public,
        b"Msg 2", Some(&mut new_alice), &bob_id, Some(&alice_bundle)
    ).unwrap();
    
    let dec2 = decrypt_with_ratchet_fallback(
        &bob.signing_key, &env2, Some(&mut bob_manager),
        Some(&bob.mlkem_keypair), Some(&bob_bundle), Some(&alice_bundle)
    ).unwrap();
    
    assert_eq!(dec2, b"Msg 2");
}
"""

with open('core/tests/integration_pq_session.rs', 'w', encoding='utf-8') as f:
    f.write(test_code)

print("Tests written to core/tests/integration_pq_session.rs")
