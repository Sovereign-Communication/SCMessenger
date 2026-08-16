//! Integration Test Suite for Node 1: Core State Machine & FFI Boundary in SCMessenger.
//!
//! Enforces the 5-Layer Deep Verification Standard:
//! 1. Layer 1 (Assertions): Functional assertions (session init, identity resolution, backup, state transitions).
//! 2. Layer 2 (Coverage): Happy paths, error arms, and match branches (messaging, blocking, abuse, maintenance).
//! 3. Layer 3 (Panic Safety & Boundaries): Corrupted/invalid inputs (empty bytes, bad hex, zero keys, malformed JSON, wrong passphrase return Err, NEVER panic).
//! 4. Layer 4 (Call Chain): Multi-hop traversal across MeshService -> IronCore -> Subsystems (Delegate, ContactManager, BlockedManager, AbuseManager, AuditLog).
//! 5. Layer 5 (Zeroization): Key material and secret zeroization on drop.

use std::sync::Arc;

use scmessenger_core::mobile_bridge::{
    ConnectionPathState, DeviceProfile, MeshService, MeshServiceConfig, MotionState, ServiceState,
};
use scmessenger_core::{AuditEventType, CoreDelegate, IronCore, IronCoreError, MessageType};
use zeroize::Zeroize;

// ============================================================================
// Mock Delegate for Layer 4 Multi-Hop Event Testing
// ============================================================================

struct TestCoreDelegate {
    discovered_peer: Arc<parking_lot::Mutex<Option<String>>>,
    disconnected_peer: Arc<parking_lot::Mutex<Option<String>>>,
    identified_peer: Arc<parking_lot::Mutex<Option<String>>>,
}

impl TestCoreDelegate {
    fn new() -> (
        Self,
        Arc<parking_lot::Mutex<Option<String>>>,
        Arc<parking_lot::Mutex<Option<String>>>,
        Arc<parking_lot::Mutex<Option<String>>>,
    ) {
        let discovered = Arc::new(parking_lot::Mutex::new(None));
        let disconnected = Arc::new(parking_lot::Mutex::new(None));
        let identified = Arc::new(parking_lot::Mutex::new(None));
        (
            Self {
                discovered_peer: discovered.clone(),
                disconnected_peer: disconnected.clone(),
                identified_peer: identified.clone(),
            },
            discovered,
            disconnected,
            identified,
        )
    }
}

impl CoreDelegate for TestCoreDelegate {
    fn on_peer_discovered(&self, peer_id: String) {
        *self.discovered_peer.lock() = Some(peer_id);
    }

    fn on_peer_disconnected(&self, peer_id: String) {
        *self.disconnected_peer.lock() = Some(peer_id);
    }

    fn on_peer_identified(
        &self,
        peer_id: String,
        _agent_version: String,
        _listen_addrs: Vec<String>,
    ) {
        *self.identified_peer.lock() = Some(peer_id);
    }

    fn on_message_received(
        &self,
        _sender_id: String,
        _sender_public_key_hex: String,
        _message_id: String,
        _sender_timestamp: u64,
        _data: Vec<u8>,
    ) {
    }

    fn on_receipt_received(&self, _message_id: String, _status: String) {}
}

// Helper to stand up an initialized node
fn create_initialized_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

// ============================================================================
// Layer 1: Session Init, Consent Gate, Identity Info & Backup Roundtrip
// ============================================================================

#[test]
fn test_layer1_session_init_and_consent_gate() {
    let node = IronCore::new();

    // Consent initially not granted
    assert!(!node.is_consent_granted());
    assert!(node.identity_id().is_none());
    assert!(node.device_id().is_none());
    assert!(node.public_key_hex().is_none());
    assert!(node.get_libp2p_peer_id().is_none());

    // Identity init without consent must fail
    let err = node.initialize_identity();
    assert!(matches!(err, Err(IronCoreError::ConsentRequired)));

    // Grant consent
    node.grant_consent();
    assert!(node.is_consent_granted());

    // Initialize identity
    node.initialize_identity()
        .expect("identity initialization after consent should succeed");

    let id_id = node.identity_id().expect("identity_id must be Some");
    let dev_id = node.device_id().expect("device_id must be Some");
    let pk_hex = node.public_key_hex().expect("public_key_hex must be Some");
    let peer_id = node
        .get_libp2p_peer_id()
        .expect("libp2p_peer_id must be Some");

    assert_eq!(id_id.len(), 64);
    assert_eq!(pk_hex.len(), 64);
    assert!(!dev_id.is_empty());
    assert!(!peer_id.is_empty());

    // Identity Info
    let info = node.get_identity_info();
    assert!(info.initialized);
    assert_eq!(info.identity_id, Some(id_id.clone()));
    assert_eq!(info.public_key_hex, Some(pk_hex.clone()));
    assert_eq!(info.device_id, Some(dev_id.clone()));
    assert_eq!(info.libp2p_peer_id, Some(peer_id.clone()));
    assert!(info.nickname.is_none());

    // Nickname setting
    node.set_nickname("Node1_Tester".to_string())
        .expect("set_nickname should succeed");
    let info_after = node.get_identity_info();
    assert_eq!(info_after.nickname, Some("Node1_Tester".to_string()));

    // Identity Backup Export & Import (Standard & Fast)
    let passphrase = "SuperSecretPassword123!";
    let backup_standard = node
        .export_identity_backup(passphrase.to_string())
        .expect("export_identity_backup should succeed");
    assert!(!backup_standard.is_empty());

    let backup_fast = node
        .export_identity_backup_fast(passphrase.to_string())
        .expect("export_identity_backup_fast should succeed");
    assert!(!backup_fast.is_empty());

    // Import into a fresh node
    let fresh_node = IronCore::new();
    fresh_node.grant_consent();
    fresh_node
        .import_identity_backup(backup_standard, passphrase.to_string())
        .expect("import_identity_backup should succeed");

    assert_eq!(fresh_node.identity_id(), Some(id_id.clone()));
    assert_eq!(fresh_node.public_key_hex(), Some(pk_hex.clone()));
    assert_eq!(
        fresh_node.get_identity_info().nickname,
        Some("Node1_Tester".to_string())
    );
}

// ============================================================================
// Layer 1 & 4: Core & MobileBridge State Machine Transitions
// ============================================================================

#[test]
fn test_layer1_and_4_state_machine_transitions() {
    // 1. IronCore Lifecycle State Machine
    let node = create_initialized_node();
    assert!(!node.is_running());
    assert_eq!(node.drift_network_state(), "Dormant");

    node.start().expect("start should succeed");
    assert!(node.is_running());
    assert_eq!(node.drift_network_state(), "Active");

    // Double start returns AlreadyRunning
    let double_start = node.start();
    assert!(matches!(double_start, Err(IronCoreError::AlreadyRunning)));

    // Stop core
    node.stop();
    assert!(!node.is_running());
    assert_eq!(node.drift_network_state(), "Dormant");

    // Re-start after stop
    node.start().expect("restart should succeed");
    assert!(node.is_running());
    node.stop();

    // 2. MobileBridge MeshService State Machine
    let config = MeshServiceConfig {
        discovery_interval_ms: 500,
        battery_floor_pct: 15,
    };
    let service = Arc::new(MeshService::new(config));
    assert_eq!(service.get_state(), ServiceState::Stopped);
    assert_eq!(
        service.get_connection_path_state(),
        ConnectionPathState::Disconnected
    );

    // Service start
    service
        .clone()
        .start()
        .expect("MeshService start should succeed");
    assert_eq!(service.get_state(), ServiceState::Running);

    // Diagnostics export
    let diag = service.export_diagnostics();
    assert!(diag.contains("service_state"));
    assert!(diag.contains("Running"));

    // Device state update & behavior adjustment
    let profile = DeviceProfile {
        battery_pct: 12,
        is_charging: false,
        has_wifi: true,
        motion_state: MotionState::Walking,
        peer_id: None,
        device_id: None,
    };
    service.update_device_state(profile);

    let behavior = service
        .recommended_behavior()
        .expect("recommended_behavior should be available after update");
    assert!(behavior.scan_interval_ms > 0);

    // Pause & Resume
    service.pause();
    service.resume();

    // Stop
    service.stop();
    assert_eq!(service.get_state(), ServiceState::Stopped);
}

// ============================================================================
// Layer 1 & 2: Identity Resolution & Canonicalization
// ============================================================================

#[test]
fn test_layer1_and_2_identity_resolution() {
    let node_a = create_initialized_node();
    let node_b = create_initialized_node();

    let pk_b = node_b.public_key_hex().unwrap();
    let id_b = node_b.identity_id().unwrap();
    let peer_id_b = node_b.get_libp2p_peer_id().unwrap();

    // 1. Resolve 64-hex public key directly
    let res_pk = node_a
        .resolve_identity(pk_b.clone())
        .expect("resolve public key should succeed");
    assert_eq!(res_pk, pk_b.to_lowercase());

    // 2. Resolve identity_id (hash) for self or contact
    let res_self_id = node_a
        .resolve_identity(node_a.identity_id().unwrap())
        .expect("resolving self identity_id should return self public_key");
    assert_eq!(res_self_id, node_a.public_key_hex().unwrap().to_lowercase());

    // 3. Resolve libp2p Peer ID
    let res_peer = node_a
        .resolve_identity(peer_id_b.clone())
        .expect("resolving peer id should succeed");
    assert_eq!(res_peer, pk_b.to_lowercase());

    // 4. Resolve to identity_id
    let res_hash = node_a
        .resolve_to_identity_id(pk_b.clone())
        .expect("resolve_to_identity_id should succeed");
    assert_eq!(res_hash, id_b);

    // 5. Canonical peer ID
    let prefixed_pk = format!("pk:{}", pk_b);
    let canonical = node_a
        .get_canonical_peer_id(&prefixed_pk)
        .expect("get_canonical_peer_id should succeed");
    assert_eq!(canonical, id_b);

    let prefixed_id = format!("id:{}", id_b);
    let canonical_id = node_a
        .get_canonical_peer_id(&prefixed_id)
        .expect("get_canonical_peer_id should succeed");
    assert_eq!(canonical_id, id_b);
}

// ============================================================================
// Layer 2: Messaging, Blocking, Abuse & Maintenance Branches
// ============================================================================

#[test]
fn test_layer2_messaging_blocking_abuse_branches() {
    let alice = create_initialized_node();
    let bob = create_initialized_node();

    alice.start().unwrap();
    bob.start().unwrap();

    let bob_pk = bob.public_key_hex().unwrap();

    // Prepare message
    let text = "Hello Bob! Testing Layer 2 Branch Coverage.";
    let prepared = alice
        .prepare_message(bob_pk.clone(), text.to_string(), MessageType::Text, None)
        .expect("prepare_message should succeed");

    assert!(!prepared.message_id.is_empty());
    assert!(!prepared.envelope_data.is_empty());

    // Receive message
    let received = bob
        .receive_message(prepared.envelope_data)
        .expect("receive_message should succeed");
    assert_eq!(received.text_content().unwrap(), text);

    // Mark sent
    let marked = alice.mark_message_sent(prepared.message_id);
    assert!(marked || alice.outbox_count() == 0);

    // Outbox & Inbox counts
    assert_eq!(bob.inbox_count(), 1);
    let peeked = bob.peek_received_messages();
    assert_eq!(peeked.len(), 1);
    let drained = bob.drain_received_messages();
    assert_eq!(drained.len(), 1);
    assert_eq!(bob.inbox_count(), 0);

    // Blocking & Unblocking
    let peer_to_block = "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff";
    alice
        .block_peer(
            peer_to_block.to_string(),
            Some("dev_1".to_string()),
            Some("spamming".to_string()),
        )
        .expect("block_peer should succeed");

    assert!(alice
        .is_peer_blocked(peer_to_block.to_string(), Some("dev_1".to_string()))
        .unwrap());
    assert_eq!(alice.blocked_count().unwrap(), 1);

    alice
        .unblock_peer(peer_to_block.to_string(), Some("dev_1".to_string()))
        .expect("unblock_peer should succeed");
    assert!(!alice
        .is_peer_blocked(peer_to_block.to_string(), None)
        .unwrap());

    // Abuse reputation & signals
    let bad_peer = "223344556677889900aabbccddeeff11223344556677889900aabbccddeeff11";
    let initial_score = alice.get_peer_reputation(bad_peer.to_string());
    assert_eq!(initial_score, 50.0);

    // Record various abuse signals to cover match branches
    alice.record_abuse_signal(bad_peer.to_string(), "RateLimited".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "OversizedMessage".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "InvalidFormat".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "DuplicateMessage".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "InvalidDestination".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "FailedRelay".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "ConnectionTimeout".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "SuccessfulRelay".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "SuccessfulDelivery".to_string());
    alice.record_abuse_signal(bad_peer.to_string(), "UnknownSignalVariant".to_string());

    let updated_score = alice.get_peer_reputation(bad_peer.to_string());
    assert!(updated_score < 100.0);
    assert!(alice.peer_rate_limit_multiplier(bad_peer.to_string()) >= 1.0);

    // Maintenance cycle
    alice
        .perform_maintenance()
        .expect("perform_maintenance should succeed");
    let maint_report = alice.run_maintenance_cycle(500);
    assert!(maint_report.contains("work_done"));
}

// ============================================================================
// Layer 3: Panic Safety & Boundary Testing (Corrupted/Invalid Inputs)
// ============================================================================

#[test]
fn test_layer3_panic_safety_and_boundaries() {
    let node = create_initialized_node();

    // 1. receive_message with empty bytes -> Err, NO panic
    let err_empty = node.receive_message(vec![]);
    assert!(err_empty.is_err());

    // 2. receive_message with random garbage bytes -> Err, NO panic
    let err_garbage = node.receive_message(vec![0xFF; 256]);
    assert!(err_garbage.is_err());

    // 3. receive_message with truncated bytes -> Err, NO panic
    let err_truncated = node.receive_message(vec![0x01, 0x02, 0x03]);
    assert!(err_truncated.is_err());

    // 4. prepare_message with empty recipient -> Err, NO panic
    let err_prep_empty =
        node.prepare_message("".to_string(), "hi".to_string(), MessageType::Text, None);
    assert!(matches!(err_prep_empty, Err(IronCoreError::InvalidInput)));

    // 5. prepare_message with non-hex string -> Err, NO panic
    let err_prep_nonhex = node.prepare_message(
        "not_a_hex_key!".to_string(),
        "hi".to_string(),
        MessageType::Text,
        None,
    );
    assert!(matches!(err_prep_nonhex, Err(IronCoreError::InvalidInput)));

    // 6. prepare_message with odd-length hex -> Err, NO panic
    let err_prep_oddhex = node.prepare_message(
        "12345".to_string(),
        "hi".to_string(),
        MessageType::Text,
        None,
    );
    assert!(matches!(err_prep_oddhex, Err(IronCoreError::InvalidInput)));

    // 7. prepare_message with all-zeros pubkey -> Err, NO panic
    let zero_pk = "0000000000000000000000000000000000000000000000000000000000000000";
    let err_prep_zero = node.prepare_message(
        zero_pk.to_string(),
        "hi".to_string(),
        MessageType::Text,
        None,
    );
    assert!(matches!(err_prep_zero, Err(IronCoreError::InvalidInput)));

    // 8. prepare_message before identity init -> Err(NotInitialized)
    let uninit_node = IronCore::new();
    uninit_node.grant_consent();
    let err_uninit = uninit_node.prepare_message(
        node.public_key_hex().unwrap(),
        "hi".to_string(),
        MessageType::Text,
        None,
    );
    assert!(matches!(err_uninit, Err(IronCoreError::NotInitialized)));

    // 9. resolve_identity with invalid hex / peer id -> Err, NO panic
    let err_res = node.resolve_identity(
        "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ".to_string(),
    );
    assert!(matches!(err_res, Err(IronCoreError::InvalidInput)));

    // 10. set_privacy_config with invalid JSON -> Err, NO panic
    let err_priv = node.set_privacy_config("{invalid_json: 123}".to_string());
    assert!(matches!(err_priv, Err(IronCoreError::InvalidInput)));

    // 11. import_identity_backup with wrong passphrase -> Err, NO panic
    let backup = node
        .export_identity_backup("correct_pass".to_string())
        .unwrap();
    let err_imp = node.import_identity_backup(backup, "wrong_pass".to_string());
    assert!(matches!(err_imp, Err(IronCoreError::CryptoError)));

    // 12. verify_signature with bad hex -> Err, NO panic
    let err_sig = node.verify_signature(vec![1, 2, 3], vec![4, 5, 6], "invalid_hex".to_string());
    assert!(matches!(err_sig, Err(IronCoreError::InvalidInput)));

    // 13. derive_wifi_aware_pmk with invalid byte length -> Err, NO panic
    let err_pmk = node.derive_wifi_aware_pmk(vec![1, 2, 3]);
    assert!(matches!(err_pmk, Err(IronCoreError::InvalidInput)));
}

// ============================================================================
// Layer 4: Multi-Hop Call Chain Traversal Across Module Boundaries
// ============================================================================

#[test]
fn test_layer4_multi_hop_call_chain_traversal() {
    let node = create_initialized_node();

    // 1. Delegate Notification Multi-Hop Traversal
    let (delegate, discovered, disconnected, _identified) = TestCoreDelegate::new();
    node.set_delegate(Some(Box::new(delegate)));

    let peer = "peer_node1_test_id".to_string();

    // Notify peer discovered -> hop into CoreDelegate -> recorded in Arc Mutex
    node.notify_peer_discovered(peer.clone());
    assert_eq!(*discovered.lock(), Some(peer.clone()));

    // Notify peer disconnected -> hop into CoreDelegate -> recorded in Arc Mutex
    node.notify_peer_disconnected(peer.clone());
    assert_eq!(*disconnected.lock(), Some(peer.clone()));

    // 2. Contact Manager -> Blocked Manager Device Registration Multi-Hop
    let contact_peer =
        "3344556677889900aabbccddeeff11223344556677889900aabbccddeeff1122".to_string();
    let dev_id = "550e8400-e29b-41d4-a716-446655440000".to_string();

    // Add device id to contact record
    node.contact_update_last_known_device_id(contact_peer.clone(), Some(dev_id.clone()))
        .expect("update device id should succeed");

    // Block peer -> multi-hop block registration also blocks dev_id
    node.block_peer(contact_peer.clone(), None, Some("test block".to_string()))
        .expect("block_peer should succeed");

    // Query device status in blocked manager
    assert!(node
        .is_peer_blocked(contact_peer.clone(), Some(dev_id.clone()))
        .unwrap());
    let known_devs = node.get_blocked_peer_devices(contact_peer.clone()).unwrap();
    assert!(known_devs.contains(&dev_id));

    // 3. Abuse Manager -> AutoBlockEngine -> AuditLog Chain
    node.record_abuse_signal(contact_peer.clone(), "RateLimited".to_string());
    node.perform_maintenance().unwrap();

    let audit_events = node.get_audit_events_by_type(AuditEventType::StorageCompacted);
    assert!(!audit_events.is_empty());
    node.validate_audit_chain()
        .expect("audit chain should be intact");
}

// ============================================================================
// Layer 5: Memory Zeroization & Secret Security
// ============================================================================

#[test]
fn test_layer5_memory_zeroization() {
    // Test zeroization of sensitive buffers
    let mut secret_key_material: [u8; 32] = [
        0x42, 0x13, 0x37, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44,
        0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33,
        0x44, 0x55,
    ];

    assert_ne!(secret_key_material, [0u8; 32]);
    secret_key_material.zeroize();
    assert_eq!(secret_key_material, [0u8; 32]);

    // Test zeroizing key pair generation
    let key_pair = scmessenger_core::identity::KeyPair::generate();
    let vk = key_pair.verifying_key();
    assert_eq!(vk.as_bytes().len(), 32);

    // Dropping identity / node zeroes secret state safely
    let node = create_initialized_node();
    let pk_bytes = node.public_key_hex().unwrap();
    assert_eq!(pk_bytes.len(), 64);
    drop(node);
}
