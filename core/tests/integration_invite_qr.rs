// Issue #469 T1: `IronCore::create_invite_qr` / `redeem_invite_qr`.
//
// Round trip between two ephemeral cores, plus the fail-closed cases: tampered
// signature, malformed input, expired token, self-redeem.

use scmessenger_core::relay::invite::InviteToken;
use scmessenger_core::IronCore;

const ADDR_A: &str = "/ip4/203.0.113.9/tcp/9001";
const ADDR_B: &str = "/ip4/198.51.100.11/tcp/9002";

fn core_with_identity() -> IronCore {
    let core = IronCore::new();
    core.grant_consent();
    core.initialize_identity().expect("identity");
    core
}

fn addrs() -> Vec<String> {
    vec![ADDR_A.to_string(), ADDR_B.to_string()]
}

#[test]
fn create_then_redeem_imports_unproven_seeds() {
    let inviter = core_with_identity();
    let invitee = core_with_identity();

    let payload = inviter.create_invite_qr(addrs(), 3600).expect("create");
    assert!(payload.starts_with("SCI1:"));

    let report = invitee.redeem_invite_qr(payload).expect("redeem");
    assert_eq!(report.addresses_offered, 2);
    assert_eq!(report.addresses_imported, 2);
    assert_eq!(report.dial_addrs.len(), 2);

    let inviter_peer = inviter.get_libp2p_peer_id().expect("peer id");
    assert_eq!(
        report.inviter_peer_id.as_deref(),
        Some(inviter_peer.as_str())
    );
    assert_eq!(
        report.dial_addrs[0],
        format!("{}/p2p/{}", ADDR_A, inviter_peer)
    );

    let seeds = invitee.ledger_manager.seed_addresses(16);
    assert_eq!(seeds.len(), 2);
    assert!(seeds.iter().all(|e| e.success_count == 0));
    // Unproven entries must not look like dialable (proven) candidates.
    assert!(invitee.ledger_manager.dialable_addresses().is_empty());
}

#[test]
fn tampered_seed_ledger_is_rejected() {
    let inviter = core_with_identity();
    let invitee = core_with_identity();
    let payload = inviter.create_invite_qr(addrs(), 3600).expect("create");

    let mut token = InviteToken::from_qr_payload(&payload).expect("decode");
    token.seed_ledger[0].multiaddr = "/ip4/198.51.100.99/tcp/1".to_string();
    let forged = token.to_qr_payload().expect("encode");

    assert!(invitee.redeem_invite_qr(forged).is_err());
    assert_eq!(invitee.ledger_manager.entry_count(), 0);
}

#[test]
fn malformed_input_is_rejected() {
    let invitee = core_with_identity();
    for bad in [
        "",
        "not-an-invite",
        "SCI1:",
        "SCI1:!!!not-base64!!!",
        "SCI1:AAAA",
        "SCI2:AAAA",
    ] {
        assert!(
            invitee.redeem_invite_qr(bad.to_string()).is_err(),
            "must reject {bad:?}"
        );
    }
    assert_eq!(invitee.ledger_manager.entry_count(), 0);
}

#[test]
fn expired_token_is_rejected() {
    let inviter = core_with_identity();
    let invitee = core_with_identity();

    let public_key =
        hex::decode(inviter.get_identity_info().public_key_hex.expect("pk")).expect("hex pk");
    let unsigned = InviteToken::new(
        inviter.identity_id().expect("id"),
        public_key,
        "open".to_string(),
    )
    .with_expiry(0)
    .with_seed_ledger(vec![scmessenger_core::relay::invite::SeedLedgerEntry {
        multiaddr: ADDR_A.to_string(),
    }]);
    let sig = inviter
        .sign_data(unsigned.get_signable_data().expect("signable"))
        .expect("sign")
        .signature;
    let payload = unsigned
        .with_signature(sig)
        .to_qr_payload()
        .expect("encode");

    assert!(invitee.redeem_invite_qr(payload).is_err());
    assert_eq!(invitee.ledger_manager.entry_count(), 0);
}

#[test]
fn self_redeem_and_bad_ttl_and_no_addresses_are_rejected() {
    let node = core_with_identity();
    let payload = node.create_invite_qr(addrs(), 3600).expect("create");
    assert!(node.redeem_invite_qr(payload).is_err());

    assert!(node.create_invite_qr(addrs(), 0).is_err());
    assert!(node
        .create_invite_qr(vec!["/ip4/127.0.0.1/tcp/9".to_string()], 60)
        .is_err());
}
