import os

r_path = 'core/src/crypto/ratchet.rs'
with open(r_path, 'r', encoding='utf-8') as f:
    r = f.read()

r = r.replace('_our_signing_key: &ed25519_dalek::SigningKey,', 'our_signing_key: &ed25519_dalek::SigningKey,')
r = r.replace('bootstrap_hct: Some(hct),', 'bootstrap_hct: Some(hct.clone()),')

with open(r_path, 'w', encoding='utf-8') as f:
    f.write(r)

print("ratchet.rs fixed again.")
