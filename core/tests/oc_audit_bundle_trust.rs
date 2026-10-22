//! OC audit probe (LOCAL, NOT SHIPPED) — does a forged `supported_suites`
//! survive into, and steer, the real send-path cipher-suite decision?
//!
//! Traced (see audit/repo-state-2026-09-27/07-UNIFICATION-PROPOSALS.md §P2):
//!   - `should_use_ratcheted_encryption` (core/src/crypto/encrypt.rs:453) picks
//!     ratchet vs legacy from `recipient_bundle.supported_suites`.
//!   - `verify_bundle` (core/src/identity/keys.rs:609) signs over
//!     `... || created_at || supported_suites`, so tampering is DETECTABLE.
//!   - `verify_bundle` has ZERO production callers.
//!   - `get_contact_bundle` (store/contacts.rs:683) — bare `serde_json::from_slice`.
//!   - `save_contact_bundle` (store/contacts.rs:666) — bare `to_vec` + put.
//!   - `save_contact_bundle` itself has ZERO production callers (all 5 are tests).
//!
//! Run: CARGO_PROFILE_TEST_DEBUG=0 cargo test -p scmessenger-core --test oc_audit_bundle_trust

use scmessenger_core::crypto::{
    encrypt_with_ratchet_fallback, is_ratcheted_envelope, RatchetSessionManager,
};
use scmessenger_core::identity::{sign_bundle, verify_bundle, IdentityKeys, PublicKeyBundle};
use scmessenger_core::message::WireEnvelope;

/// Was the ratcheted / forward-secret path actually taken?
fn wire_is_ratcheted(wire: &WireEnvelope) -> bool {
    match wire {
        WireEnvelope::V1(env) => is_ratcheted_envelope(env),
        // V2 is the hybrid/DriftEnvelope path, reached only when a bundle
        // advertised a hybrid suite -- so reaching it also proves ratcheting.
        WireEnvelope::V2(_) => true,
    }
}

fn bundle_for(keys: &IdentityKeys) -> PublicKeyBundle {
    sign_bundle(keys).expect("sign_bundle must succeed")
}

/// Downgrade a genuine bundle's advertised suites WITHOUT re-signing it --
/// exactly what an attacker, or a buggy importer, would produce.
fn forge_supported_suites(genuine: &PublicKeyBundle, suites: Vec<u8>) -> PublicKeyBundle {
    let mut forged = genuine.clone();
    forged.supported_suites = suites;
    forged
}

fn recipient_pk_of(keys: &IdentityKeys) -> [u8; 32] {
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&keys.signing_key.verifying_key().to_bytes());
    pk
}

#[test]
fn oc_T1_tampered_supported_suites_is_detectable_by_verify_bundle() {
    let keys = IdentityKeys::generate();
    let genuine = bundle_for(&keys);

    assert!(
        verify_bundle(&genuine).is_ok(),
        "a freshly signed bundle must verify"
    );
    assert!(
        genuine
            .supported_suites
            .iter()
            .any(|s| s == &0x02 || s == &0x03),
        "genuine bundle must advertise a hybrid suite, got {:?}",
        genuine.supported_suites
    );

    let forged = forge_supported_suites(&genuine, vec![]);
    assert_eq!(
        forged.ed25519_public, genuine.ed25519_public,
        "the forgery keeps the real identity -- only the suite field is touched"
    );
    assert!(
        verify_bundle(&forged).is_err(),
        "a bundle with a forged supported_suites MUST fail verify_bundle, because \
         supported_suites is inside the signed bytes. If this fails, the field is \
         NOT covered and P2 is far worse than stated."
    );
}

#[test]
fn oc_T2_forged_field_survives_the_contact_store_round_trip() {
    let node = scmessenger_core::IronCore::new();
    node.grant_consent();
    node.initialize_identity().expect("identity init");

    let keys = node.get_identity_keys().expect("keys");
    let forged = forge_supported_suites(&bundle_for(&keys), vec![]);

    let mgr = node.contacts_store_manager();
    mgr.save_contact_bundle(&keys.public_key_hex(), &forged)
        .expect("store must accept the bundle");

    let read_back = mgr
        .get_contact_bundle(&keys.public_key_hex())
        .expect("read must succeed")
        .expect("bundle must be present");

    assert!(
        read_back.supported_suites.is_empty(),
        "the forged (downgraded) supported_suites must survive storage, got {:?}",
        read_back.supported_suites
    );
    assert!(
        verify_bundle(&read_back).is_err(),
        "and the bundle read back out of the store is still never verified: \
         nothing in the store path called verify_bundle"
    );
}

#[test]
fn oc_T3_the_forged_field_decides_ratchet_vs_legacy_in_the_real_send_path() {
    // THE DECISIVE A/B. Identical sender keys, identical recipient key bytes,
    // identical plaintext, no pre-existing session. The ONLY difference is the
    // recipient bundle handed to the send-path decision.
    let sender = IdentityKeys::generate();
    let recipient = IdentityKeys::generate();
    let recipient_pk = recipient_pk_of(&recipient);

    let genuine = bundle_for(&recipient);
    let forged = forge_supported_suites(&genuine, vec![]);

    let encrypt = |bundle: &PublicKeyBundle, require_pq: bool| {
        let mut sessions = RatchetSessionManager::new();
        encrypt_with_ratchet_fallback(
            &sender.signing_key,
            Some(bundle),
            &recipient_pk,
            b"identical plaintext on both arms",
            Some(&mut sessions),
            "peer-under-test",
            None,
            None,
            require_pq,
            None,
        )
    };

    // Arm 1: the bundle the recipient actually signed.
    let genuine_wire = encrypt(&genuine, false).expect("genuine bundle must encrypt");
    assert!(
        wire_is_ratcheted(&genuine_wire),
        "CONTROL: a genuinely signed bundle advertising a hybrid suite must take \
         the ratcheted path"
    );

    // Arm 2: the same bundle with one field forged after signing.
    let forged_wire = encrypt(&forged, false).expect("forged bundle must ALSO encrypt");
    assert!(
        !wire_is_ratcheted(&forged_wire),
        "a bundle whose supported_suites was forged after signing still steers the \
         send-path decision -- the sender SILENTLY falls back to legacy static-ECDH \
         instead of erroring, because nothing on the read/write path verified it."
    );
}

#[test]
fn oc_T4_the_hard_error_branch_exists_but_production_never_sets_its_flag() {
    // Corrects an earlier claim in the proposals document: the hard-error branch
    // ("V1 peer cannot be sent to when require_pq=true") is real, but the only
    // production call site -- iron_core.rs:1040 -- passes require_pq = false. So a
    // forged bundle cannot interop-deny via the CLI send path; it can only
    // silently downgrade.
    let sender = IdentityKeys::generate();
    let recipient = IdentityKeys::generate();
    let recipient_pk = recipient_pk_of(&recipient);
    let forged = forge_supported_suites(&bundle_for(&recipient), vec![]);

    let mut sessions = RatchetSessionManager::new();
    let res = encrypt_with_ratchet_fallback(
        &sender.signing_key,
        Some(&forged),
        &recipient_pk,
        b"payload",
        Some(&mut sessions),
        "peer-under-test",
        None,
        None,
        true, // require_pq -- the flag production never sets
        None,
    );
    assert!(
        res.is_err(),
        "with require_pq=true a downgraded bundle must hard-error; reachable only \
         by callers other than iron_core.rs:1040"
    );
}
