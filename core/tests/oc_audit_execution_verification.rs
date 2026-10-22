//! OC audit execution verification — LOCALLY WRITTEN, NOT SHIPPED.
//!
//! Written inside the throwaway clone at `runtime/worktrees/scmessenger-main`
//! to settle audit verdicts that had only been established by reading source.
//! NOT part of the SCMessenger repository; ships with nothing. Nothing in
//! `../SCMessenger` was touched.
//!
//! Covers:
//!   S-3 / issue #393 — what `verify_bundle` actually guarantees.
//!   S-6 / issue #378 — lock contention surfaces as "not initialized".

use scmessenger_core::identity::keys::{sign_bundle, verify_bundle, IdentityKeys};
use scmessenger_core::IronCore;

// ---------------------------------------------------------------------------
// S-3 / #393 — what verify_bundle actually guarantees
// ---------------------------------------------------------------------------

#[test]
fn oc_s3_verify_bundle_detects_in_flight_tampering() {
    let keys = IdentityKeys::generate();
    let bundle = sign_bundle(&keys).unwrap();
    assert!(verify_bundle(&bundle).is_ok(), "a genuine bundle must verify");

    // A bundle modified after signing is rejected.
    let mut tampered = bundle.clone();
    tampered.ed25519_public[0] ^= 1;
    assert!(
        verify_bundle(&tampered).is_err(),
        "in-place tampering must be rejected"
    );
}

#[test]
fn oc_s3_verify_bundle_does_NOT_bind_the_bundle_to_an_expected_identity() {
    // FINDING: verify_bundle is an INTEGRITY check, not an AUTHENTICITY check.
    //
    // It proves the signature matches the keys carried inside the bundle. It
    // does NOT prove the bundle belongs to the peer the caller expected. A
    // bundle freshly signed by a completely different identity verifies
    // cleanly, because it is internally consistent.
    let attacker = IdentityKeys::generate();
    let attacker_bundle = sign_bundle(&attacker).unwrap();

    assert!(
        verify_bundle(&attacker_bundle).is_ok(),
        "MEASURED: a bundle signed by an arbitrary other identity PASSES \
         verification, because it is internally self-consistent"
    );

    // And its key is genuinely the attacker's, not any expected peer's.
    let attacker_public = attacker.signing_key.verifying_key().to_bytes();
    assert_eq!(
        attacker_bundle.ed25519_public, attacker_public,
        "the bundle carries the signer's own key"
    );
}

#[test]
fn oc_s3_ingestion_without_verification_would_accept_a_forged_bundle() {
    // Models the future contact-bundle import that issue #393 is guarding.
    // Whether or not verify_bundle is called, a bundle from an unexpected
    // signer is not what the gate catches; only in-flight tampering is.
    let expected_peer = IdentityKeys::generate();
    let attacker = IdentityKeys::generate();

    let forged = sign_bundle(&attacker).unwrap();

    // The naive path: persist whatever arrived. No verification, no binding.
    let persisted = forged.clone();
    let expected_public = expected_peer.signing_key.verifying_key().to_bytes();

    assert_ne!(
        persisted.ed25519_public, expected_public,
        "a naive ingestion path stores the attacker's key while attributing \
         the record to the expected peer"
    );

    // Even the gate the issue asks for would not stop this one, because the
    // bundle is self-consistent.
    assert!(
        verify_bundle(&persisted).is_ok(),
        "verify_bundle does not reject a self-consistent bundle from the \
         wrong signer, so wiring it alone would not satisfy the issue's \
         'forged bundle rejected at ingest' criterion"
    );
}

// ---------------------------------------------------------------------------
// S-6 / #378 — lock contention surfaces as "not initialized", not as degraded
// ---------------------------------------------------------------------------

#[test]
fn oc_s6_lock_contention_reports_not_initialized_not_degraded() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let storage_str = storage.to_str().unwrap().to_string();

    // Node A configures and initializes a real identity on this store.
    let node_a = IronCore::with_storage(storage_str.clone());
    node_a.grant_consent();
    node_a
        .initialize_identity()
        .expect("node A must be able to initialize");
    assert!(
        node_a.get_identity_info().initialized,
        "node A must report itself initialized"
    );

    // Node B opens the SAME store. sled takes an exclusive lock, so B cannot
    // read the identity back. This is the contention the issue describes.
    let node_b = IronCore::with_storage(storage_str.clone());
    let info_b = node_b.get_identity_info();

    // THE DEFECT, executed: B reports "no identity", which is what makes the
    // CLI print "No identity found ... Run 'scm init' first" even though the
    // identity exists and the real problem is a lock.
    assert!(
        !info_b.initialized,
        "a lock-contended store reports initialized == false — this is the \
         misleading-diagnostic mechanism"
    );

    // ...and the information needed to say something accurate IS available.
    assert!(
        node_b.is_storage_degraded(),
        "a lock-contended store reports itself degraded, so an accurate \
         diagnostic was available to the caller and unused"
    );

    // Nothing was lost or misconfigured; the second reader simply cannot get in.
    assert!(
        node_a.get_identity_info().initialized,
        "node A still holds a working identity: nothing was misconfigured"
    );
}
