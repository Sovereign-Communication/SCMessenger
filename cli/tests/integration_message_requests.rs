//! Integration test: the CLI's JSON-RPC message-request handlers
//! (get_pending_message_requests / accept_message_request / reject_message_request),
//! driven through the real handler (not just parse_intent) with a real IronCore.

use scmessenger_cli::server::{handle_jsonrpc_request, UiCommand, WebContext};
use scmessenger_core::{IronCore, MessageType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// `contacts_manager()` opens a sled DB at `IronCore`'s `storage_path` fresh
/// on every call; an in-memory `IronCore::new()` (empty storage_path) makes
/// every node in every test share the same relative "contacts.db" path,
/// causing sled's exclusive-open lock to collide across tests running in
/// parallel. Use a unique temp dir per node instead.
fn make_node() -> IronCore {
    let dir = tempfile::tempdir().unwrap();
    // Leak the tempdir so it outlives the IronCore instance for the life of
    // the test process; these are small, short-lived test-only directories.
    let path = Box::leak(Box::new(dir))
        .path()
        .to_str()
        .unwrap()
        .to_string();
    let node = IronCore::with_storage(path);
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized")
}

fn make_ctx(core: Arc<IronCore>) -> WebContext {
    let ledger = scmessenger_cli::ledger::ConnectionLedger::new(
        scmessenger_core::store::LedgerManager::ephemeral(),
    );

    WebContext {
        node_peer_id: "test-node".to_string(),
        node_public_key: pubkey(&core),
        bootstrap_nodes: vec![],
        ledger: Arc::new(Mutex::new(ledger)),
        peers: Arc::new(Mutex::new(HashMap::new())),
        start_time: Instant::now(),
        transport_bridge: Arc::new(Mutex::new(
            scmessenger_cli::transport_bridge::TransportBridge::new(),
        )),
        ui_port: 0,
        core: Some(core),
    }
}

async fn rpc(
    ctx: &WebContext,
    tx: &tokio::sync::mpsc::UnboundedSender<UiCommand>,
    method: &str,
    params: serde_json::Value,
) -> serde_json::Value {
    let raw = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();
    let resp = handle_jsonrpc_request(&raw, ctx, tx).await;
    if let Some(err) = resp.error {
        panic!("RPC {} failed: {:?}", method, err);
    }
    resp.result
        .expect("response must have a result when there's no error")
}

/// A message from a non-contact peer must show up as a pending request;
/// accepting it must add the sender as a contact and remove it from the
/// pending list; a second poll must still show it (peek, not drain).
#[test]
fn message_request_lifecycle_accept() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let alice = make_node();
        let bob = Arc::new(make_node());
        let bob_pubkey = pubkey(&bob);

        let prepared = alice
            .prepare_message(
                bob_pubkey,
                "hi, we've never talked before".to_string(),
                MessageType::Text,
                None,
            )
            .expect("prepare_message succeeds");
        bob.receive_message(prepared.envelope_data)
            .expect("receive_message succeeds");

        let ctx = make_ctx(bob.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        std::mem::forget(rx);

        // First poll: the request must be visible.
        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        let requests = result["requests"].as_array().expect("requests array");
        assert_eq!(
            requests.len(),
            1,
            "expected exactly one pending request: {:?}",
            requests
        );
        let alice_identity_id = requests[0]["peerId"].as_str().unwrap().to_string();
        assert_eq!(requests[0]["messageCount"], 1);
        assert_eq!(
            requests[0]["messagePreview"],
            "hi, we've never talked before"
        );

        // Second poll (peek, not drain): the request must still be there.
        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        assert_eq!(
            result["requests"].as_array().unwrap().len(),
            1,
            "peek must not consume the request"
        );

        // Accept: adds Alice as a contact.
        let result = rpc(
            &ctx,
            &tx,
            "accept_message_request",
            json!({ "request_id": alice_identity_id }),
        )
        .await;
        assert_eq!(result["accepted"], true);

        // Accepted contacts land in the CLI's contacts_store_manager(), the
        // same store the send path (UiCommand::Send) reads from - not the
        // UniFFI-bridge contacts_manager(), which the send path never
        // looks at (T2).
        let contacts = bob.contacts_store_manager().list().expect("list contacts");
        assert_eq!(contacts.len(), 1);
        // UNIFICATION V2 P0: the canonical contact identifier is the Ed25519
        // public_key_hex, not the derived identity_id. AcceptMessageRequest
        // stores the pubkey as peer_id (matching ContactsManager::add()'s
        // live canonicalization), so assert against Alice's public key.
        let alice_pubkey = pubkey(&alice);
        assert_eq!(contacts[0].peer_id, alice_pubkey);
        assert!(
            !contacts[0].public_key.is_empty(),
            "accepted contact must have a real public key"
        );

        // Now that Alice is a contact, she must no longer show up as a pending request.
        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        assert_eq!(
            result["requests"].as_array().unwrap().len(),
            0,
            "accepted sender must not still appear as pending"
        );
    });
}

/// Rejecting a message request must block the sender and remove them from
/// the pending list.
#[test]
fn message_request_lifecycle_reject() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let eve = make_node();
        let bob = Arc::new(make_node());
        let bob_pubkey = pubkey(&bob);

        let prepared = eve
            .prepare_message(
                bob_pubkey,
                "unsolicited message".to_string(),
                MessageType::Text,
                None,
            )
            .expect("prepare_message succeeds");
        bob.receive_message(prepared.envelope_data)
            .expect("receive_message succeeds");

        let ctx = make_ctx(bob.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        std::mem::forget(rx);

        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        let requests = result["requests"].as_array().unwrap();
        assert_eq!(requests.len(), 1);
        let eve_identity_id = requests[0]["peerId"].as_str().unwrap().to_string();

        let result = rpc(
            &ctx,
            &tx,
            "reject_message_request",
            json!({ "request_id": eve_identity_id.clone() }),
        )
        .await;
        assert_eq!(result["rejected"], true);

        let blocked = bob.list_blocked_peers().expect("list blocked peers");
        assert!(blocked.iter().any(|b| b.peer_id == eve_identity_id));

        // The rejected/blocked sender must not still appear as pending.
        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        assert_eq!(
            result["requests"].as_array().unwrap().len(),
            0,
            "rejected sender must not still appear as pending"
        );

        // And must not have been silently added as a contact.
        let contacts = bob.contacts_store_manager().list().expect("list contacts");
        assert!(contacts.is_empty());
    });
}

/// T2: a message from a sender who is *already* a CLI contact
/// (contacts_store_manager(), the store `UiCommand::Send` and every `scm
/// contacts` path use) must never show up as a pending request, even
/// though get_pending_message_requests used to check the separate
/// UniFFI-bridge contacts_manager() and so treated every CLI-only contact
/// as a stranger.
#[test]
fn existing_cli_contact_message_is_not_a_pending_request() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let alice = make_node();
        let bob = Arc::new(make_node());
        let bob_pubkey = pubkey(&bob);
        let alice_pubkey = pubkey(&alice);
        let alice_identity_id = alice
            .get_identity_info()
            .identity_id
            .expect("alice identity id");

        // Bob already knows Alice as a CLI contact before she messages him.
        bob.contacts_store_manager()
            .add(scmessenger_core::store::Contact::new(
                alice_identity_id.clone(),
                alice_pubkey,
            ))
            .expect("bob adds alice as a contact");

        let prepared = alice
            .prepare_message(
                bob_pubkey,
                "hey, it's me".to_string(),
                MessageType::Text,
                None,
            )
            .expect("prepare_message succeeds");
        bob.receive_message(prepared.envelope_data)
            .expect("receive_message succeeds");

        let ctx = make_ctx(bob.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        std::mem::forget(rx);

        let result = rpc(&ctx, &tx, "get_pending_message_requests", json!({})).await;
        assert_eq!(
            result["requests"].as_array().unwrap().len(),
            0,
            "a message from an existing CLI contact must not be a pending request"
        );
    });
}

/// Accepting an unknown request_id (no matching inbox message) must fail
/// with a clear error rather than silently adding a contact with a garbage
/// public key.
#[test]
fn accept_unknown_request_id_fails() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bob = Arc::new(make_node());
        let ctx = make_ctx(bob.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        std::mem::forget(rx);

        let raw = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accept_message_request",
            "params": { "request_id": "nonexistent-sender" },
        })
        .to_string();
        let resp = handle_jsonrpc_request(&raw, &ctx, &tx).await;
        assert!(
            resp.error.is_some(),
            "accepting an unknown request_id must fail"
        );
        assert!(bob.contacts_store_manager().list().unwrap().is_empty());
    });
}
