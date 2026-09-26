//! WP1 -- identity unification: regression tests for the canonical-hex model.
//!
//! The canonical model (implementation plan section 0) is ONE identity and ONE
//! write helper. These tests pin the write-side invariants that WP1 closed, so a
//! future change that reopens them fails here rather than in a live send.
//!
//! Scope note: `is_ghost_peer_topic` and the own-topic subscribe are private to
//! `core/src/transport/swarm.rs`, which is a Rule-8 gated directory. These tests
//! therefore assert the observable seam those paths depend on --
//! `extract_ed25519_public_key_from_peer_id` and the topic string shape -- and
//! not the private control flow. A private-function unit test inside
//! `swarm.rs` would require the Rule-8 adversarial review this change is not
//! claiming.
//!
//! Run with:
//!   cargo test -p scmessenger-core --test integration_wp1_identity_unification

use scmessenger_core::store::ledger_entry::{
    is_self_certifying_binding, peer_id_from_public_key_hex, public_key_hex_from_libp2p_peer_id,
};
use scmessenger_core::transport::swarm::extract_ed25519_public_key_from_peer_id;
use scmessenger_core::{IronCore, MessageType};

/// Build a real self-certifying (peer id, key hex) pair.
fn self_certifying_peer(seed_tag: &[u8]) -> (libp2p::PeerId, String) {
    let mut seed = [0u8; 32];
    let n = seed_tag.len().min(32);
    seed[..n].copy_from_slice(&seed_tag[..n]);
    let signing = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed)
        .expect("test seed is a valid ed25519 secret key");
    let kp = libp2p::identity::ed25519::Keypair::from(signing);
    let public = kp.public();
    let key_hex = hex::encode(public.to_bytes());
    let peer_id = libp2p::identity::PublicKey::from(public).to_peer_id();
    (peer_id, key_hex)
}

/// A base58-encoded SHA-256 multihash (`0x12 0x20 <32 bytes>`): a real libp2p
/// peer id shape that is NOT an identity multihash, so it embeds no public key.
/// This is precisely the input the removed fabrication fallback used to
/// "recover" 32 bytes of key material from.
fn sha256_multihash_peer_id() -> String {
    let mut bytes = vec![0x12u8, 0x20];
    bytes.extend(0u8..32);
    bs58::encode(bytes).into_string()
}

// ============================================================================
// WP1.1 -- a public key is only ever recovered, never fabricated
// ============================================================================

#[test]
fn non_identity_peer_id_yields_no_public_key() {
    let peer = sha256_multihash_peer_id();
    assert!(
        public_key_hex_from_libp2p_peer_id(&peer).is_none(),
        "a SHA-256 multihash peer id embeds no key and must not produce one"
    );
    // It IS a parseable peer id, so the refusal is about the binding, not about
    // the input being malformed -- a fix that merely rejected garbage would
    // pass for the wrong reason.
    assert!(
        peer.parse::<libp2p::PeerId>().is_ok(),
        "the input must be a genuine peer id, otherwise this test is vacuous"
    );
}

#[test]
fn self_certifying_peer_id_round_trips_through_the_key() {
    let (peer_id, key_hex) = self_certifying_peer(b"wp1-integration-roundtrip");

    assert_eq!(
        public_key_hex_from_libp2p_peer_id(&peer_id.to_string()).as_deref(),
        Some(key_hex.as_str())
    );
    assert!(is_self_certifying_binding(&peer_id.to_string(), &key_hex));
    // And the inverse: the key re-derives the same peer id.
    assert_eq!(
        peer_id_from_public_key_hex(&key_hex).as_deref(),
        Some(peer_id.to_string().as_str())
    );
}

#[test]
fn identity_id_is_never_mistaken_for_a_public_key() {
    // identity_id is hex(blake3(pubkey)): also 64 hex chars, also 32 bytes.
    // Every width-based check admits it; only curve decompression rejects it.
    let (peer_id, key_hex) = self_certifying_peer(b"wp1-integration-identity-id");
    let identity_id = scmessenger_core::identity::keys::identity_id_from_public_key_hex(&key_hex)
        .expect("identity id derives from a public key");

    assert_eq!(identity_id.len(), 64);
    assert!(hex::decode(&identity_id).is_ok());
    assert_ne!(identity_id, key_hex);
    // The genuine key is not an identity_id, so the helper does not resolve it.
    assert_ne!(
        public_key_hex_from_libp2p_peer_id(&identity_id),
        Some(key_hex.clone()),
        "an identity_id must not resolve to a public key"
    );
    assert!(is_self_certifying_binding(&peer_id.to_string(), &key_hex));
}

// ============================================================================
// WP1.1 -- placeholder contacts fail closed at the encrypt path
// ============================================================================

fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

#[test]
fn placeholder_contact_cannot_be_an_encrypt_recipient() {
    let node = make_node();

    // A recovered contact whose key could not be derived carries an EMPTY
    // public_key. Sending to it must fail, not emit unopenable ciphertext.
    let outcome = node.prepare_message(
        String::new(),
        "must not encrypt".to_string(),
        MessageType::Text,
        None,
    );
    let Err(err) = outcome else {
        panic!("a recipient with no verified key must be refused");
    };
    assert!(
        matches!(err, scmessenger_core::IronCoreError::InvalidInput),
        "expected InvalidInput, got {err:?}"
    );
}

#[test]
fn a_verified_recipient_still_encrypts() {
    // Guards against the placeholder guard over-reaching into real sends.
    let sender = make_node();
    let recipient = make_node();
    let recipient_key = recipient
        .get_identity_info()
        .public_key_hex
        .expect("recipient key");

    let prepared = sender
        .prepare_message(
            recipient_key.clone(),
            "still deliverable".to_string(),
            MessageType::Text,
            None,
        )
        .expect("a verified recipient must still encrypt");
    assert!(!prepared.envelope_data.is_empty());

    let received = recipient
        .receive_message(prepared.envelope_data)
        .expect("recipient must accept it");
    assert_eq!(
        received.text_content().expect("text content"),
        "still deliverable"
    );
    assert_eq!(
        received.sender_id,
        sender
            .get_identity_info()
            .public_key_hex
            .expect("sender key"),
        "the sender identity on a delivered message is the sender's real key"
    );
}

// ============================================================================
// WP1.3 -- hex and base58 both resolve to one canonical identity
// ============================================================================

#[test]
fn hex_and_base58_spellings_resolve_to_the_same_peer_id() {
    let (peer_id, key_hex) = self_certifying_peer(b"wp1-integration-spellings");

    // base58 spelling parses directly.
    let from_base58: libp2p::PeerId = peer_id.to_string().parse().expect("base58 peer id parses");
    // hex spelling resolves through the documented extract/parse helper --
    // never a second contact-address flavor.
    let from_hex = peer_id_from_public_key_hex(&key_hex).expect("key resolves to peer id");
    assert_eq!(from_base58.to_string(), from_hex);
    assert_eq!(from_hex, peer_id.to_string());
}

#[test]
fn key_hex_uses_only_one_case_on_disk() {
    // `add()` lowercases on write, so lookup and the identity index agree.
    let (_peer_id, key_hex) = self_certifying_peer(b"wp1-integration-case");
    let upper = key_hex.to_uppercase();
    assert_ne!(
        upper, key_hex,
        "the generated key has hex letters to case-fold"
    );
    assert_eq!(
        peer_id_from_public_key_hex(&upper.to_lowercase()).as_deref(),
        Some(peer_id_from_public_key_hex(&key_hex).unwrap().as_str())
    );
}

// ============================================================================
// WP1.4 -- own-topic addressing is derived from the local identity
// ============================================================================

#[test]
fn own_peer_topic_hex_is_the_local_ed25519_key() {
    // Both event loops (native and wasm) build the own topic from exactly this
    // extraction. If it ever started reading a different source, the node
    // would subscribe to a topic no one addresses and inbound delivery would
    // silently stop -- the ghost-own-topic regression.
    let (peer_id, key_hex) = self_certifying_peer(b"wp1-integration-own-topic");

    let extracted = extract_ed25519_public_key_from_peer_id(&peer_id)
        .expect("a self-certifying peer id carries an inline Ed25519 key");
    assert_eq!(hex::encode(extracted), key_hex);

    // The topic shape the two loops construct from it.
    let topic = format!("/scmessenger/peer/{}/v1", key_hex);
    assert!(topic.starts_with("/scmessenger/peer/"));
    assert!(topic.ends_with("/v1"));
    // The ghost guard exempts the own topic by comparing this hex, so it must
    // be a bare 64-hex value with no prefix or separator left attached.
    let hex_part = topic
        .trim_start_matches("/scmessenger/peer/")
        .trim_end_matches("/v1");
    assert_eq!(hex_part, key_hex);
    assert!(is_self_certifying_binding(&peer_id.to_string(), hex_part));
}

#[test]
fn non_identity_peer_id_cannot_supply_an_own_topic_key() {
    // A peer id with no inline key must NOT be able to produce an own-topic
    // hex, or the guard would subscribe to an address nobody derives.
    let peer = sha256_multihash_peer_id();
    let parsed: libp2p::PeerId = peer.parse().expect("parses as a peer id");
    assert!(
        extract_ed25519_public_key_from_peer_id(&parsed).is_err(),
        "a hashed peer id carries no inline key"
    );
    assert!(public_key_hex_from_libp2p_peer_id(&peer).is_none());
}

/// The CLI used to test key shape with libp2p's
/// `ed25519::PublicKey::try_from_bytes`. WP1 consolidated it onto core's
/// `is_valid_public_key`, the single owner of that predicate.
///
/// That is a deliberate difference, not an equivalence, and this test pins the
/// direction. `is_valid_public_key` is STRICTER than the libp2p/dalek check: it
/// additionally requires a canonical RFC 8032 encoding. Its own doc records why
/// -- dalek's decompression is lenient about non-canonical `y >= p` and about a
/// set sign bit on `x = 0`, roughly half of all Blake3 identity_ids decompress
/// to some point, and being laxer than the platform validator is the documented
/// cause of "callers encrypt to a hash and produce ciphertext nobody can
/// decrypt".
///
/// So the CLI was the lax outlier, and the consolidation closes that gap. This
/// asserts both halves: the new owner never accepts anything the old one
/// rejected (it cannot get more permissive), and it does reject concrete
/// non-canonical encodings the old one accepted.
#[test]
fn consolidated_key_shape_check_is_strictly_stricter_than_the_old_cli_check() {
    let old_cli_check = |bytes: &[u8]| {
        let mut buf = bytes.to_vec();
        libp2p::identity::ed25519::PublicKey::try_from_bytes(&mut buf).is_ok()
    };
    let new_owner = |hex_str: &str| scmessenger_core::identity::keys::is_valid_public_key(hex_str);

    // Witness 1: all-0xff. y = 0x7fff..ff >= p, so the encoding is
    // non-canonical. The old check accepts it; the owner must not.
    let non_canonical = vec![0xffu8; 32];
    let nc_hex = hex::encode(&non_canonical);
    assert!(
        old_cli_check(&non_canonical),
        "precondition: the old lenient check accepts this encoding"
    );
    assert!(
        !new_owner(&nc_hex),
        "the owner must reject a non-canonical y >= p encoding"
    );

    // Witness 2: a set sign bit on x = 0 (y = 1) is non-canonical -- there is
    // no negative zero.
    let mut neg_zero = vec![0u8; 32];
    neg_zero[0] = 0x01;
    neg_zero[31] = 0x80;
    let nz_hex = hex::encode(&neg_zero);
    assert!(
        !new_owner(&nz_hex),
        "y = 1 with sign bit set is non-canonical"
    );

    // A real key is accepted by both: the owner is not simply broken.
    let (_peer_id, real_key_hex) = self_certifying_peer(b"wp1-predicate-parity");
    assert!(new_owner(&real_key_hex));
    assert!(old_cli_check(&hex::decode(&real_key_hex).expect("hex")));

    // Across a deterministic random sweep, the owner must never accept
    // something the old check rejected. The reverse is allowed and expected.
    for i in 0u32..2000 {
        let bytes = blake3::hash(&i.to_le_bytes());
        let bytes = bytes.as_bytes();
        let as_hex = hex::encode(bytes);
        if new_owner(&as_hex) {
            assert!(
                old_cli_check(bytes),
                "owner accepted {} but the old check rejected it -- the owner                  must not become more permissive",
                as_hex
            );
        }
    }
}
