//! AND-06: the strict public-key decoder against the values the LIVE mesh
//! actually stores, plus generated keys at scale.
//!
//! Provenance of the corpus below (captured 2026-09-17, evidence under
//! `tmp/v040-live-verify-20260917/`): every value is read verbatim from a
//! running node's control API, not invented:
//!
//!   * `LIVE_IDENTITIES` -- `/api/identity` on the Windows node and the AWS
//!     cloud node, plus the third mesh peer as recorded in the Windows node's
//!     `/api/peers`. Each row is (libp2p_peer_id, public_key_hex, identity_id)
//!     exactly as the node reports them, with `self_certifying: true`.
//!   * `LIVE_NON_KEY_64HEX` -- 64-hex identifiers the same nodes store in their
//!     peer/ledger records (`allowed_peer_id`, `last_peer_id`,
//!     `observed_peer_ids`). These are hashes, NOT curve points. They exist in
//!     this test as the adversarial half of the corpus: the decoder exists to
//!     tell a public key from an identity_id, so its behaviour on real hashes
//!     is the thing being measured.
//!
//! Why this test is worth its lines: `is_valid_public_key` is the gate that
//! `identity_id_from_public_key_hex` sits behind, and that derivation is how a
//! peer's identity_id is resolved from its public key. If strictness rejected a
//! key a node legitimately produced, identity resolution would break in
//! production while every vector-only test still passed. So the assertion is
//! two-sided: real keys MUST be accepted, and the decoder must never be more
//! permissive than the lenient decode it replaced.

use ed25519_dalek::SigningKey;
use scmessenger_core::identity::keys::identity_id_from_public_key_hex;
use scmessenger_core::is_valid_public_key;

/// (libp2p_peer_id, public_key_hex, identity_id) exactly as reported by the
/// live nodes: Windows node, AWS cloud node, third mesh peer.
const LIVE_IDENTITIES: &[(&str, &str, &str)] = &[
    (
        "12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw",
        "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e",
        "985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826",
    ),
    (
        "12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31",
        "69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c",
        "37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006",
    ),
    (
        "12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn",
        "30dce2bb779b4f1419f6d7d9e91b3ae201aed9e3b181aef674a9496f340a0645",
        "f83ab16319ca5b801f1c088935f2215c6aae9aa246b992f01c0f27f06b03cbe5",
    ),
];

/// Real 64-hex identifiers the nodes store that are NOT public keys (peer
/// hashes from ledger/address records).
const LIVE_NON_KEY_64HEX: &[&str] = &[
    "27872c8a5e8221c25de95363913a5027be83f9aa96cd489ad03aed137f94e992",
    "2d94a6e78905995144587d0c220ea7c5f4451ed9d413f6cfa86c0efcffc9d3c3",
    "3854e44295c1384854b89312e5c3925f8431b6f4c41ed66979b82b94bc93b5d7",
    "72996ca8d3adca1dc3fde3c58722774586eaef41ed5593d015e260364feba176",
    "a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a",
    "ce540ced016300402cdb310349998f482c0ebe7df999c34eb3ffda6516604821",
];

fn valid(hex: &str) -> bool {
    is_valid_public_key(hex.to_string())
}

/// The lenient decode this change replaced: dalek's own point check, with no
/// canonicity rules. Every strict acceptance must also satisfy this, so the
/// change can only ever remove acceptances, never add them.
fn lenient_dalek_decode(hex: &str) -> bool {
    match hex::decode(hex) {
        Ok(bytes) => match <[u8; 32]>::try_from(bytes.as_slice()) {
            Ok(arr) => ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Deterministic 32-byte seeds so the generated-key sample is reproducible.
fn deterministic_seed(i: usize) -> [u8; 32] {
    let hash = blake3::hash(format!("and06-live-key-corpus-{i}").as_bytes());
    let mut seed = [0u8; 32];
    seed.copy_from_slice(hash.as_bytes());
    seed
}

#[test]
fn every_public_key_the_live_mesh_stores_is_accepted() {
    for (peer, public_key_hex, identity_id) in LIVE_IDENTITIES {
        assert!(
            valid(public_key_hex),
            "the live node {peer} stores public_key_hex {public_key_hex} \
             (identity_id {identity_id}) and the strict decoder REJECTED it -- \
             strictness that rejects a key a node produced is a defect in this \
             fix, not strictness"
        );
    }
}

#[test]
fn live_identity_ids_still_derive_from_their_live_public_keys() {
    // identity_id = blake3(public_key bytes), and the derivation refuses to run
    // unless is_valid_public_key passes. So this asserts the live mesh's own
    // identity resolution path still works end to end on real values.
    for (peer, public_key_hex, identity_id) in LIVE_IDENTITIES {
        let derived = identity_id_from_public_key_hex(public_key_hex);
        assert_eq!(
            derived.as_deref(),
            Some(*identity_id),
            "identity resolution for live peer {peer} broke: deriving the \
             identity_id from its public key no longer yields the identity_id \
             the node actually holds"
        );
    }
}

#[test]
fn strictness_is_monotone_relative_to_the_lenient_decode() {
    let mut checked = 0usize;
    let mut strict_only: Vec<String> = Vec::new();

    let mut check = |value: &str| {
        checked += 1;
        if valid(value) && !lenient_dalek_decode(value) {
            strict_only.push(value.to_string());
        }
    };

    for (_, public_key_hex, identity_id) in LIVE_IDENTITIES {
        check(public_key_hex);
        check(identity_id);
    }
    for value in LIVE_NON_KEY_64HEX {
        check(value);
    }
    // Generated curve points and hashes, at a scale no hand-written vector list
    // reaches.
    for i in 0..4096 {
        let signing_key = SigningKey::from_bytes(&deterministic_seed(i));
        check(&hex::encode(signing_key.verifying_key().to_bytes()));
        check(&hex::encode(blake3::hash(&deterministic_seed(i)).as_bytes()));
    }
    // Boundary encodings that the canonicity rules exist for.
    for boundary in [
        format!("01{}", "00".repeat(31)),
        format!("01{}{}", "00".repeat(30), "80"),
        format!("ec{}{}", "ff".repeat(30), "7f"),
        format!("ec{}{}", "ff".repeat(30), "ff"),
        "ff".repeat(32),
        "00".repeat(32),
    ] {
        check(&boundary);
    }

    assert!(
        checked >= 8000,
        "expected a large corpus, only checked {checked}"
    );
    assert!(
        strict_only.is_empty(),
        "the strict decoder accepted {} value(s) the lenient decode rejects, so \
         it is not strictly stricter: {strict_only:?}",
        strict_only.len()
    );
}

#[test]
fn generated_keys_at_scale_are_all_accepted_and_derive_their_identity() {
    const SAMPLE: usize = 4096;
    for i in 0..SAMPLE {
        let signing_key = SigningKey::from_bytes(&deterministic_seed(i));
        let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
        assert!(
            valid(&public_key_hex),
            "generated Ed25519 public key #{i} was rejected"
        );
        let expected = hex::encode(blake3::hash(
            &signing_key.verifying_key().to_bytes(),
        )
        .as_bytes());
        assert_eq!(
            identity_id_from_public_key_hex(&public_key_hex).as_deref(),
            Some(expected.as_str()),
            "generated key #{i} did not derive its documented identity_id"
        );
    }
}

#[test]
fn live_non_key_identifiers_are_measured_not_assumed() {
    // This is deliberately a REPORT with a weak invariant, not a hard threshold.
    // Roughly half of all 32-byte strings decode to some curve point; that is
    // inherent to Ed25519 decompression and predates this change. What this
    // change is responsible for is that the rate did not get WORSE, which the
    // monotonicity test enforces over a much larger corpus. Printing the real
    // count keeps the number visible instead of asserted-away.
    let mut strict_accepts = 0usize;
    let mut lenient_accepts = 0usize;
    for value in LIVE_NON_KEY_64HEX {
        if valid(value) {
            strict_accepts += 1;
        }
        if lenient_dalek_decode(value) {
            lenient_accepts += 1;
        }
    }
    println!(
        "[OK] live non-key 64-hex identifiers: {} sampled, strict decoder \
         accepts {strict_accepts}, lenient decode accepts {lenient_accepts}",
        LIVE_NON_KEY_64HEX.len()
    );

    // Named, not just counted: which real identities the classifier still
    // reads as curve points. Live, `GET /api/peer-resolve` on both candidate
    // nodes reported `input_kind: public_key, self_certifying: true` for two of
    // these three identity_ids, so the ambiguity is real and reachable - and
    // identical under the lenient decode, i.e. pre-existing rather than
    // introduced here. See HANDOFF/audit/LIVE_VERIFICATION_305_2026-09-18.md.
    for (label, value) in [
        ("live identity_id (windows 985a25f9)", LIVE_IDENTITIES[0].2),
        ("live identity_id (aws 37eb7561)", LIVE_IDENTITIES[1].2),
        ("live identity_id (peer f83ab163)", LIVE_IDENTITIES[2].2),
    ] {
        println!(
            "[REPORT] {label}: strict={} lenient={}",
            valid(value),
            lenient_dalek_decode(value)
        );
    }
    assert!(
        strict_accepts <= lenient_accepts,
        "strict decoder accepted more real non-key identifiers ({strict_accepts}) \
         than the lenient decode ({lenient_accepts})"
    );
}
