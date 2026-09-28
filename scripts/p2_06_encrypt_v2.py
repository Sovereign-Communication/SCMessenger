import os
import re

e_path = 'core/src/crypto/encrypt.rs'
with open(e_path, 'r', encoding='utf-8') as f:
    e = f.read()

# 1. Update decrypt_message_ratcheted to set peer_confirmed
dec_v1 = """    let aad = envelope.sender_public_key.as_slice();

    let plaintext = session.decrypt(
        dh_public,
        message_number,
        &envelope.nonce,
        &envelope.ciphertext,
        aad,
    )?;
    
    session.peer_confirmed = true;
    Ok(plaintext)
}
"""

e = re.sub(r'    let aad = envelope\.sender_public_key\.as_slice\(\);\n\n    session\.decrypt\(\n        dh_public,\n        message_number,\n        &envelope\.nonce,\n        &envelope\.ciphertext,\n        aad,\n    \)\n\}', dec_v1, e, flags=re.DOTALL)

# 2. Add decrypt_message_ratcheted_v2
dec_v2 = """

pub fn decrypt_message_ratcheted_v2(
    session: &mut crate::crypto::RatchetSession,
    envelope: &crate::message::EnvelopeV2,
) -> Result<Vec<u8>> {
    let dh_public = envelope
        .ratchet_dh_public
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Ratcheted V2 envelope missing ratchet_dh_public field"))?;
    let message_number = envelope.ratchet_message_number.ok_or_else(|| {
        anyhow::anyhow!("Ratcheted V2 envelope missing ratchet_message_number field")
    })?;

    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length in ratcheted V2 envelope");
    }

    let aad = envelope.sender_public_key.as_slice();

    let plaintext = session.decrypt(
        dh_public,
        message_number,
        &envelope.nonce,
        &envelope.ciphertext,
        aad,
    )?;
    
    session.peer_confirmed = true;
    Ok(plaintext)
}
"""

e = e.replace("pub fn encrypt_message_ratcheted(", dec_v2 + "pub fn encrypt_message_ratcheted(")

with open(e_path, 'w', encoding='utf-8') as f:
    f.write(e)
print("encrypt.rs updated with decrypt_message_ratcheted_v2.")
