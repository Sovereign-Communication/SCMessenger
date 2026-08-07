// Per-message encryption: X25519 ECDH + XChaCha20-Poly1305
//
// Flow:
// 1. Convert sender's Ed25519 signing key → X25519 static secret
// 2. Generate ephemeral X25519 keypair
// 3. ECDH: ephemeral_secret × recipient_x25519_public → shared_secret
// 4. KDF: Blake3::derive_key(shared_secret) → symmetric_key
// 5. Encrypt: XChaCha20-Poly1305(symmetric_key, random_nonce, plaintext)
// 6. Output: Envelope { sender_pub, ephemeral_pub, nonce, ciphertext }
//
// Recipient reverses:
// 1. Convert recipient's Ed25519 key → X25519 static secret
// 2. ECDH: recipient_secret × ephemeral_public → shared_secret
// 3. KDF: same derivation → symmetric_key
// 4. Decrypt: XChaCha20-Poly1305(symmetric_key, nonce, ciphertext)

use anyhow::{bail, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

/// KDF context string for deriving encryption keys from ECDH shared secrets.
/// Changing this breaks compatibility with all existing messages.
pub const KDF_CONTEXT: &str = "iron-core v2 message encryption 2026-02-05";

/// Convert an Ed25519 signing key to an X25519 static secret for ECDH.
///
/// Ed25519 and X25519 share the same underlying curve (Curve25519),
/// so we can derive X25519 keys from Ed25519 keys deterministically.
/// The conversion uses the clamped SHA-512 hash of the Ed25519 secret key,
/// which is how Ed25519 internally derives its scalar.
pub fn ed25519_to_x25519_secret(signing_key: &SigningKey) -> StaticSecret {
    // Ed25519 secret scalar is SHA-512(secret_key_bytes)[0..32], clamped.
    // x25519-dalek StaticSecret expects the raw 32-byte secret and does its own clamping.
    let mut hash = <sha2::Sha512 as sha2::Digest>::digest(signing_key.to_bytes());
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&hash[..32]);

    let secret = StaticSecret::from(secret_bytes);

    // Zeroize intermediates
    secret_bytes.zeroize();
    hash.as_mut_slice().zeroize();

    secret
}

/// Validate that a hex-encoded Ed25519 public key is well-formed.
///
/// Checks:
/// 1. Hex decoding succeeds
/// 2. Length is exactly 32 bytes
/// 3. Bytes represent a valid compressed Ed25519 point
///
/// Returns an error with a specific message if validation fails.
pub fn validate_ed25519_public_key(public_key_hex: &str) -> Result<()> {
    use curve25519_dalek::edwards::CompressedEdwardsY;

    // Decode hex
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|_| anyhow::anyhow!("Invalid hex encoding in public key"))?;

    // Check length
    if public_key_bytes.len() != 32 {
        bail!(
            "Public key must be exactly 32 bytes (64 hex characters), got {} bytes ({} hex characters)",
            public_key_bytes.len(),
            public_key_hex.len()
        );
    }

    // Validate Ed25519 format by attempting decompression
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&public_key_bytes);

    let compressed = CompressedEdwardsY::from_slice(&key_array)
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key format"))?;

    compressed.decompress().ok_or_else(|| {
        anyhow::anyhow!("Public key is not a valid Ed25519 point (decompression failed)")
    })?;

    Ok(())
}

/// Convert an Ed25519 verifying (public) key to an X25519 public key.
///
/// Uses the birational map from Ed25519 (twisted Edwards) to X25519 (Montgomery).
/// This is the standard conversion: u = (1 + y) / (1 - y) mod p.
pub fn ed25519_public_to_x25519(public_key_bytes: &[u8; 32]) -> Result<X25519PublicKey> {
    use curve25519_dalek::edwards::CompressedEdwardsY;

    let compressed = CompressedEdwardsY::from_slice(public_key_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key"))?;

    let edwards_point = compressed
        .decompress()
        .ok_or_else(|| anyhow::anyhow!("Failed to decompress Ed25519 public key"))?;

    let montgomery = edwards_point.to_montgomery();
    Ok(X25519PublicKey::from(montgomery.to_bytes()))
}

/// Derive a symmetric encryption key from an ECDH shared secret using Blake3.
fn derive_key(shared_secret: &[u8]) -> [u8; 32] {
    blake3::derive_key(KDF_CONTEXT, shared_secret)
}

/// Encrypt a plaintext message for a recipient.
///
/// # Arguments
/// * `sender_signing_key` - Sender's Ed25519 signing key (for sender identification)
/// * `recipient_public_key` - Recipient's Ed25519 public key bytes (32 bytes)
/// * `plaintext` - The message bytes to encrypt
///
/// # Returns
/// An `Envelope` containing everything needed for decryption.
pub fn encrypt_message(
    sender_signing_key: &SigningKey,
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
) -> Result<crate::message::Envelope> {
    // Convert recipient's Ed25519 public key to X25519
    let recipient_x25519 = ed25519_public_to_x25519(recipient_public_key)?;

    // Generate ephemeral X25519 keypair for this message
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // ECDH: ephemeral_secret × recipient_public → shared_secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_x25519);

    // KDF: derive symmetric key
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Generate random nonce (24 bytes for XChaCha20)
    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    // Encrypt with AAD (Additional Authenticated Data)
    // Bind sender public key as AAD to prevent sender spoofing
    let sender_public_bytes = sender_signing_key.verifying_key().to_bytes();
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: &sender_public_bytes,
            },
        )
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(crate::message::Envelope {
        sender_public_key: sender_signing_key.verifying_key().to_bytes().to_vec(),
        ephemeral_public_key: ephemeral_public.to_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        ratchet_dh_public: None,
        ratchet_message_number: None,
    })
}

/// Decrypt an envelope using the recipient's signing key.
///
/// # Arguments
/// * `recipient_signing_key` - Recipient's Ed25519 signing key
/// * `envelope` - The encrypted envelope
///
/// # Returns
/// The decrypted plaintext bytes.
pub fn decrypt_message(
    recipient_signing_key: &SigningKey,
    envelope: &crate::message::Envelope,
) -> Result<Vec<u8>> {
    // Validate envelope fields
    if envelope.ephemeral_public_key.len() != 32 {
        bail!("Invalid ephemeral public key length");
    }
    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length");
    }

    // Convert recipient's Ed25519 signing key to X25519 static secret
    let recipient_x25519_secret = ed25519_to_x25519_secret(recipient_signing_key);

    // Reconstruct ephemeral public key
    let mut ephemeral_bytes = [0u8; 32];
    ephemeral_bytes.copy_from_slice(&envelope.ephemeral_public_key);
    let ephemeral_public = X25519PublicKey::from(ephemeral_bytes);

    // ECDH: recipient_secret × ephemeral_public → shared_secret
    let shared_secret = recipient_x25519_secret.diffie_hellman(&ephemeral_public);

    // KDF: derive symmetric key
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Decrypt
    let nonce = XNonce::from_slice(&envelope.nonce);
    let sender_public_bytes = envelope.sender_public_key.as_slice();
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: envelope.ciphertext.as_slice(),
                aad: sender_public_bytes,
            },
        )
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(plaintext)
}

/// Sign an envelope with the sender's signing key.
pub fn sign_envelope(
    sender_signing_key: &SigningKey,
    envelope: &mut crate::message::Envelope,
) -> Result<()> {
    let signature = sender_signing_key.sign(&envelope.to_bytes_for_signing()?);
    envelope.signature = Some(signature.to_bytes().to_vec());
    Ok(())
}

/// Verify an envelope's signature.
pub fn verify_envelope(envelope: &crate::message::Envelope) -> Result<bool> {
    let Some(signature) = &envelope.signature else {
        return Ok(false);
    };
    let public_key = &envelope.sender_public_key;
    IdentityKeys::verify(&envelope.to_bytes_for_signing()?, signature, public_key)
}

/// Check if an envelope has ratchet fields (V2 format).
pub fn is_ratcheted_envelope(envelope: &crate::message::Envelope) -> bool {
    envelope.ratchet_dh_public.is_some() && envelope.ratchet_message_number.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::IdentityKeys;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let sender_keys = IdentityKeys::generate();
        let recipient_keys = IdentityKeys::generate();

        let plaintext = b"Hello, world!";
        let envelope = encrypt_message(
            &sender_keys.signing_key,
            &recipient_keys.signing_key.verifying_key().to_bytes(),
            plaintext,
        )
        .unwrap();

        let decrypted = decrypt_message(&recipient_keys.signing_key, &envelope).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_with_empty_message() {
        let sender_keys = IdentityKeys::generate();
        let recipient_keys = IdentityKeys::generate();

        let plaintext = b"";
        let envelope = encrypt_message(
            &sender_keys.signing_key,
            &recipient_keys.signing_key.verifying_key().to_bytes(),
            plaintext,
        )
        .unwrap();

        let decrypted = decrypt_message(&recipient_keys.signing_key, &envelope).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_large_message() {
        let sender_keys = IdentityKeys::generate();
        let recipient_keys = IdentityKeys::generate();

        let plaintext = vec![0x42u8; 10000];
        let envelope = encrypt_message(
            &sender_keys.signing_key,
            &recipient_keys.signing_key.verifying_key().to_bytes(),
            &plaintext,
        )
        .unwrap();

        let decrypted = decrypt_message(&recipient_keys.signing_key, &envelope).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails_decrypt() {
        let sender_keys = IdentityKeys::generate();
        let recipient_keys = IdentityKeys::generate();
        let wrong_keys = IdentityKeys::generate();

        let plaintext = b"Hello";
        let envelope = encrypt_message(
            &sender_keys.signing_key,
            &recipient_keys.signing_key.verifying_key().to_bytes(),
            plaintext,
        )
        .unwrap();

        let result = decrypt_message(&wrong_keys.signing_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_verify_envelope() {
        let sender_keys = IdentityKeys::generate();
        let recipient_keys = IdentityKeys::generate();

        let plaintext = b"Test message";
        let mut envelope = encrypt_message(
            &sender_keys.signing_key,
            &recipient_keys.signing_key.verifying_key().to_bytes(),
            plaintext,
        )
        .unwrap();

        sign_envelope(&sender_keys.signing_key, &mut envelope).unwrap();
        assert!(verify_envelope(&envelope).unwrap());

        // Tamper with ciphertext
        envelope.ciphertext[0] ^= 1;
        assert!(!verify_envelope(&envelope).unwrap());
    }
}