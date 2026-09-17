//! AND-06 behavioural parity: the UniFFI-exposed key-validity check against the
//! Kotlin contract it is meant to replace.
//!
//! The vectors below are copied verbatim from
//! `android/app/src/test/java/com/scmessenger/android/utils/PeerIdValidatorCurveVectorTest.kt`.
//! That file's own doc comment asserts the contract this test executes:
//!
//!   "These run on the JVM without the native library, which is why the
//!    pure-Kotlin fallback authority exists; when the UniFFI path is wired
//!    on-device the same vectors must pass against the core implementation."
//!
//! So this drives the *exported* symbol Kotlin will call
//! (`scmessenger_core::is_valid_public_key`, the function declared in api.udl),
//! not the internal helper it delegates to. If the two implementations disagree
//! on any vector, replacing Kotlin with the core call is a behaviour change and
//! this test is where that shows up.

use scmessenger_core::is_valid_public_key;

// RFC 8032 section 7.1 public keys (valid Ed25519 points).
const RFC8032_PUB1: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const RFC8032_PUB2: &str = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
const RFC8032_PUB3: &str = "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";

fn valid(hex: &str) -> bool {
    is_valid_public_key(hex.to_string())
}

#[test]
fn rfc8032_public_keys_are_valid_curve_points() {
    for pk in [RFC8032_PUB1, RFC8032_PUB2, RFC8032_PUB3] {
        assert!(valid(pk), "RFC 8032 public key must be a valid point: {pk}");
    }
}

#[test]
fn y_equals_one_is_valid_only_with_sign_bit_zero() {
    // y = 1 => x^2 = 0 => canonical encoding requires sign bit 0.
    let y_one_sign0 = format!("01{}", "00".repeat(31));
    let y_one_sign1 = format!("01{}{}", "00".repeat(30), "80");

    assert!(
        valid(&y_one_sign0),
        "y=1 with sign bit 0 (x=0, canonical) must be accepted"
    );
    assert!(
        !valid(&y_one_sign1),
        "y=1 with sign bit 1 (x=0, NON-canonical) must be rejected -- the Kotlin \
         authority rejects it, so the core must too or the migration changes behaviour"
    );
}

#[test]
fn y_equals_p_minus_one_is_valid_only_with_sign_bit_zero() {
    // y = p-1 => y^2 = 1 => x^2 = 0; the sign bit decides canonical validity.
    let p_minus_1_sign0 = format!("ec{}{}", "ff".repeat(30), "7f");
    let p_minus_1_sign1 = format!("ec{}{}", "ff".repeat(30), "ff");
    assert_eq!(p_minus_1_sign0.len(), 64);
    assert_eq!(p_minus_1_sign1.len(), 64);

    assert!(valid(&p_minus_1_sign0), "y=p-1 with sign bit 0 must be accepted");

    // Top byte ff = 0x7f | 0x80: same y, non-canonical sign bit.
    let non_canonical = format!("{}ff", &p_minus_1_sign0[..62]);
    assert!(
        !valid(&non_canonical),
        "y=p-1 with a non-canonical sign bit must be rejected"
    );
    assert!(!valid(&p_minus_1_sign1));
}

#[test]
fn y_values_at_or_above_p_are_rejected() {
    let y_equals_p = format!("ed{}{}", "ff".repeat(30), "ff");
    assert!(!valid(&y_equals_p), "y = p is out of field range");
    assert!(!valid(&"ff".repeat(32)), "all-ff encodes y >> p");
}

#[test]
fn malformed_inputs_are_rejected() {
    for bad in [
        "",
        "zz",
        "30d0fa67",
        &RFC8032_PUB1[..62],                  // truncated
        &format!("{RFC8032_PUB1}aa"),         // extended
        &format!("{}g", &RFC8032_PUB1[..63]), // non-hex byte
    ] {
        assert!(!valid(bad), "malformed input must be rejected: {bad:?}");
    }
}

#[test]
fn uppercase_hex_is_accepted() {
    // Kotlin lowercases in normalizePublicKeyHex and its byte parser accepts
    // uppercase; the core's is_ascii_hexdigit + hex::decode must agree, or a
    // contact pasted as uppercase hex would resolve on one platform only.
    assert!(valid(&RFC8032_PUB1.to_uppercase()));
}

#[test]
fn identity_id_derivation_refuses_non_canonical_key_encodings() {
    // The boundary that matters: `identity_id` and `public_key_hex` are both 64
    // hex characters, so a lax point test silently promotes identity_ids to
    // public keys. `identity_id_from_public_key_hex` is the core guard that
    // prevents double-hashing an identity_id, and it must refuse the same
    // shapes this test's canonicality cases refuse.
    use scmessenger_core::identity::identity_id_from_public_key_hex;

    let y_equals_p = format!("ed{}{}", "ff".repeat(30), "ff");
    assert!(
        identity_id_from_public_key_hex(&y_equals_p).is_none(),
        "y = p is not a key, so no identity_id may be derived from it"
    );
    assert!(
        identity_id_from_public_key_hex(&"ff".repeat(32)).is_none(),
        "all-ff is not a key, so no identity_id may be derived from it"
    );

    // The accept path still works, and the derivation is a hash (not a
    // pass-through), so a real key cannot round-trip as its own identity_id.
    let derived = identity_id_from_public_key_hex(RFC8032_PUB1)
        .expect("a canonical RFC 8032 key must derive an identity_id");
    assert_eq!(derived.len(), 64);
    assert_ne!(derived, RFC8032_PUB1);
}

#[test]
fn the_contract_is_a_64_hex_curve_point_and_nothing_else() {
    // Boundary: 63 and 65 characters are not 64, regardless of content.
    assert!(!valid(&RFC8032_PUB1[..63]));
    assert!(!valid(&format!("{RFC8032_PUB1}0")));
    // And the accepted form is the exact exact-length, in-range point.
    assert!(valid(RFC8032_PUB1));
}
