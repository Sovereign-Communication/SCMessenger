// Inbound delivery-receipt helpers, kept in the lib target so their tests run in CI
// (the `[[bin]]` target sets `test = false`).

use scmessenger_core::message::decode_envelope;

/// Extract the sender's Ed25519 public key (hex) from an inbound envelope.
///
/// `decode_envelope` only understands Drift and legacy bincode V1 bytes. A
/// signed V1/V2 wire envelope, or a bare V2 wire envelope, decrypts fine in
/// `IronCore::receive_message` but used to yield `None` here, which silently
/// skipped the delivery receipt (no key to encrypt it to). Try every decoder
/// the core accepts so a decrypted chat message always has a receipt target.
pub(crate) fn envelope_sender_public_key_hex(envelope_data: &[u8]) -> Option<String> {
    use scmessenger_core::message::{
        decode_wire_envelope, decode_wire_signed_envelope, WireEnvelope, WireSignedEnvelope,
    };
    if let Ok(env) = decode_envelope(envelope_data) {
        return Some(hex::encode(&env.sender_public_key));
    }
    if let Ok(signed) = decode_wire_signed_envelope(envelope_data) {
        let key = match signed {
            WireSignedEnvelope::V1(s) => s.envelope.sender_public_key,
            WireSignedEnvelope::V2(s) => s.envelope.sender_public_key,
        };
        return Some(hex::encode(key));
    }
    match decode_wire_envelope(envelope_data).ok()? {
        WireEnvelope::V1(env) => Some(hex::encode(env.sender_public_key)),
        WireEnvelope::V2(env) => Some(hex::encode(env.sender_public_key)),
    }
}

/// Decide whether a decrypted inbound text message gets a delivery receipt.
///
/// Every genuine chat message (identity-envelope kind "text", or a bare
/// non-enveloped payload) is acknowledged, and a duplicate delivery is
/// acknowledged again: the sender retransmits precisely because it never saw
/// the first receipt. Only sync/config metadata (kind other than "text"),
/// loop-backs from our own key, and messages with no known sender key (nothing
/// to encrypt a receipt to) are skipped.
pub(crate) fn should_ack_inbound(
    envelope_kind: Option<&str>,
    sender_public_key_hex: Option<&str>,
    local_public_key_hex: Option<&str>,
) -> bool {
    let is_chat = envelope_kind.is_none_or(|kind| kind == "text");
    let has_target = sender_public_key_hex.is_some();
    let is_self_loop =
        sender_public_key_hex.is_some() && sender_public_key_hex == local_public_key_hex;
    is_chat && has_target && !is_self_loop
}

/// Inbound delivery-receipt decision regressions (cell test 2026-10-10).
#[cfg(test)]
mod inbound_receipt_tests {
    use super::*;
    use scmessenger_core::message::MessageType;
    use scmessenger_core::IronCore;

    const ALICE: &str = "aa";
    const BOB: &str = "bb";

    #[test]
    fn chat_text_and_bare_payloads_are_acked() {
        assert!(should_ack_inbound(Some("text"), Some(ALICE), Some(BOB)));
        assert!(should_ack_inbound(None, Some(ALICE), Some(BOB)));
    }

    #[test]
    fn duplicate_delivery_is_acked_again() {
        // Drives the real receive path twice with the same wire envelope, the
        // way a sender retransmits after missing the first receipt. The claim
        // under test: `receive_message` accepts the repeat (it must not turn
        // the duplicate into an error, which would skip the ACK), and the ACK
        // decision is made from what that SECOND delivery yields.
        let alice = IronCore::new();
        alice.grant_consent();
        alice.initialize_identity().expect("test setup");
        let bob = IronCore::new();
        bob.grant_consent();
        bob.initialize_identity().expect("test setup");
        let alice_pk = alice
            .get_identity_info()
            .public_key_hex
            .expect("test setup");
        let bob_pk = bob.get_identity_info().public_key_hex.expect("test setup");
        let prepared = alice
            .prepare_message(bob_pk.clone(), "dup".to_string(), MessageType::Text, None)
            .expect("test setup");

        let first = bob
            .receive_message(prepared.envelope_data.clone())
            .expect("first delivery must be accepted");
        let second = bob
            .receive_message(prepared.envelope_data.clone())
            .expect("a duplicate delivery must still return Ok so it can be re-ACKed");
        assert_eq!(first.id, prepared.message_id);
        assert_eq!(second.id, first.id, "duplicate must resolve to the same id");

        let sender = envelope_sender_public_key_hex(&prepared.envelope_data);
        assert_eq!(sender.as_deref(), Some(alice_pk.as_str()));
        assert!(should_ack_inbound(
            Some("text"),
            sender.as_deref(),
            Some(bob_pk.as_str())
        ));
        // A receipt for the duplicate is producible for the original sender.
        let receipt = bob
            .prepare_receipt(alice_pk.clone(), second.id.clone())
            .expect("receipt for a duplicate must be producible");
        assert!(!receipt.is_empty());
    }

    #[test]
    fn sync_metadata_loopback_and_unknown_sender_are_not_acked() {
        assert!(!should_ack_inbound(
            Some("identity_sync"),
            Some(ALICE),
            Some(BOB)
        ));
        assert!(!should_ack_inbound(
            Some("history_sync"),
            Some(ALICE),
            Some(BOB)
        ));
        assert!(!should_ack_inbound(Some("text"), Some(BOB), Some(BOB)));
        assert!(!should_ack_inbound(Some("text"), None, Some(BOB)));
    }

    #[test]
    fn sender_key_is_recovered_from_real_envelope_and_dup_is_reackable() {
        let alice = IronCore::new();
        alice.grant_consent();
        alice.initialize_identity().expect("test setup");
        let bob = IronCore::new();
        bob.grant_consent();
        bob.initialize_identity().expect("test setup");
        let alice_pk = alice
            .get_identity_info()
            .public_key_hex
            .expect("test setup");
        let bob_pk = bob.get_identity_info().public_key_hex.expect("test setup");

        let prepared = alice
            .prepare_message(bob_pk.clone(), "hello".to_string(), MessageType::Text, None)
            .expect("test setup");
        let sender = envelope_sender_public_key_hex(&prepared.envelope_data);
        assert_eq!(sender.as_deref(), Some(alice_pk.as_str()));

        // First delivery and a duplicate both decrypt, and both can be acked.
        for _ in 0..2 {
            let msg = bob
                .receive_message(prepared.envelope_data.clone())
                .expect("test setup");
            assert_eq!(msg.id, prepared.message_id);
            assert!(should_ack_inbound(
                Some("text"),
                sender.as_deref(),
                Some(bob_pk.as_str())
            ));
            assert!(bob
                .prepare_receipt(alice_pk.clone(), msg.id.clone())
                .is_ok());
        }
    }
}
