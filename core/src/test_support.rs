//! Test-only helpers shared by more than one module's `#[cfg(test)]` code.
//!
//! Compiled only under `cfg(test)`, so nothing here reaches a release build.
//! It lives in its own module rather than inside a functional one (it used to
//! sit in `identity::keys`, which is production code and should carry no test
//! scaffolding).
//!
//! Note this is NOT reachable from `core/tests/` or from the CLI: those compile
//! the crate without `cfg(test)`. They carry their own copies, which cannot be
//! de-duplicated without publishing a test API.

/// Build a real self-certifying `(libp2p peer id, public key hex)` pair for
/// tests, deterministically from `seed_tag`.
pub(crate) fn self_certifying_keypair(seed_tag: &[u8]) -> (String, String) {
    let mut seed = [0u8; 32];
    let n = seed_tag.len().min(32);
    seed[..n].copy_from_slice(&seed_tag[..n]);
    let signing =
        libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed).expect("valid test seed");
    let kp = libp2p::identity::ed25519::Keypair::from(signing);
    let peer_id = libp2p::identity::PublicKey::from(kp.public())
        .to_peer_id()
        .to_string();
    let key_hex = hex::encode(kp.public().to_bytes());
    (peer_id, key_hex)
}
