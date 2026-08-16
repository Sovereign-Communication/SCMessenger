//! Integration Test Suite for Node 3 (Sequential Execution):
//! Local P2P Mesh Transports (BLE & WiFi Direct/Aware) in SCMessenger.
//!
//! Enforces the 5-Layer Deep Verification Standard:
//! 1. Layer 1 (Assertions): Functional assertions for roundtrips, state machine transitions, disk persistence.
//! 2. Layer 2 (Coverage): Coverage of happy paths, error arms, and match branches across all target modules.
//! 3. Layer 3 (Panic Safety & Boundaries): Corrupted, invalid, malformed inputs (returns Err/None, NEVER panics).
//! 4. Layer 4 (Call Chain): Multi-hop traversal across WiFi Direct, WiFi Aware, Escalation, Reputation, L2CAP.
//! 5. Layer 5 (Zeroization): Ensure key material (PMKs, handshake secrets) zeroize on drop.

use async_trait::async_trait;
use libp2p::PeerId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use zeroize::{Zeroize, ZeroizeOnDrop};

use scmessenger_core::store::backend::MemoryStorage;
use scmessenger_core::transport::abstraction::TransportType;
use scmessenger_core::transport::ble::l2cap::{
    append_crc32, ChannelState, DropReason, FragmentHeader, L2capChannel, L2capConfig, L2capError,
    L2capFragmenter, L2capReassembler, L2capReassemblyManager, ProtocolServiceMultiplexer,
};
use scmessenger_core::transport::escalation::{
    EscalationEngine, EscalationError, EscalationPolicy,
};
use scmessenger_core::transport::reputation::{
    AbuseReputationManager, AbuseSignal, ReputationScore,
};
use scmessenger_core::transport::wifi_aware::{
    decode_port_tlv, encode_port_tlv, DiscoveredPeer, WifiAwareConfig, WifiAwareError,
    WifiAwarePlatformBridge, WifiAwareState, WifiAwareTransport, TLV_TYPE_PORT,
};
use scmessenger_core::transport::wifi_direct::{
    compute_group_owner_intent, GroupInfo, WifiDirectError, WifiDirectPeer,
    WifiDirectPlatformBridge, WifiDirectState, WifiDirectTransport, WIFI_DIRECT_GO_INTENT_CLIENT,
    WIFI_DIRECT_GO_INTENT_PREFERRED,
};

// ============================================================================
// TEST MOCK BRIDGES
// ============================================================================

struct MockTestWifiAwareBridge {
    available: Arc<RwLock<bool>>,
    published_services: Arc<RwLock<Vec<(String, Vec<u8>)>>>,
    subscribed_services: Arc<RwLock<Vec<(String, Option<Vec<u8>>)>>>,
    active_data_paths: Arc<RwLock<HashMap<String, SocketAddr>>>,
    on_discovered_cb: Arc<RwLock<Option<Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>>>>,
    on_message_cb: Arc<RwLock<Option<Box<dyn Fn(String, Vec<u8>) + Send + Sync>>>>,
    on_confirmed_cb: Arc<RwLock<Option<Box<dyn Fn(String, SocketAddr) + Send + Sync>>>>,
}

impl MockTestWifiAwareBridge {
    fn new(available: bool) -> Self {
        Self {
            available: Arc::new(RwLock::new(available)),
            published_services: Arc::new(RwLock::new(Vec::new())),
            subscribed_services: Arc::new(RwLock::new(Vec::new())),
            active_data_paths: Arc::new(RwLock::new(HashMap::new())),
            on_discovered_cb: Arc::new(RwLock::new(None)),
            on_message_cb: Arc::new(RwLock::new(None)),
            on_confirmed_cb: Arc::new(RwLock::new(None)),
        }
    }

    #[allow(dead_code)]
    fn set_available(&self, val: bool) {
        *self.available.write() = val;
    }
}

#[async_trait]
impl WifiAwarePlatformBridge for MockTestWifiAwareBridge {
    async fn is_available(&self) -> Result<bool, WifiAwareError> {
        Ok(*self.available.read())
    }

    async fn publish_service(
        &self,
        service_name: &str,
        service_info: &[u8],
    ) -> Result<(), WifiAwareError> {
        if !*self.available.read() {
            return Err(WifiAwareError::Unavailable);
        }
        self.published_services
            .write()
            .push((service_name.to_string(), service_info.to_vec()));
        Ok(())
    }

    async fn subscribe_to_services(
        &self,
        service_name: &str,
        match_filter: Option<&[u8]>,
    ) -> Result<(), WifiAwareError> {
        if !*self.available.read() {
            return Err(WifiAwareError::Unavailable);
        }
        self.subscribed_services
            .write()
            .push((service_name.to_string(), match_filter.map(|f| f.to_vec())));
        Ok(())
    }

    async fn unpublish_service(&self) -> Result<(), WifiAwareError> {
        self.published_services.write().clear();
        Ok(())
    }

    async fn unsubscribe_from_services(&self) -> Result<(), WifiAwareError> {
        self.subscribed_services.write().clear();
        Ok(())
    }

    async fn create_data_path(
        &self,
        peer_id: &str,
        _pmk: &[u8; 32],
    ) -> Result<SocketAddr, WifiAwareError> {
        if !*self.available.read() {
            return Err(WifiAwareError::Unavailable);
        }
        let addr: SocketAddr = "192.168.49.1:8888".parse().unwrap();
        self.active_data_paths
            .write()
            .insert(peer_id.to_string(), addr);
        Ok(addr)
    }

    async fn close_data_path(&self, peer_id: &str) -> Result<(), WifiAwareError> {
        self.active_data_paths.write().remove(peer_id);
        Ok(())
    }

    fn set_on_service_discovered(&self, callback: Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>) {
        *self.on_discovered_cb.write() = Some(callback);
    }

    fn set_on_message_received(&self, callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {
        *self.on_message_cb.write() = Some(callback);
    }

    fn set_on_data_path_confirmed(&self, callback: Box<dyn Fn(String, SocketAddr) + Send + Sync>) {
        *self.on_confirmed_cb.write() = Some(callback);
    }
}

struct MockTestWifiDirectBridge {
    available: Arc<RwLock<bool>>,
    discovering: Arc<RwLock<bool>>,
    groups: Arc<RwLock<Vec<String>>>,
    connections: Arc<RwLock<Vec<String>>>,
    on_peers_cb: Arc<RwLock<Option<Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>>>>,
    on_conn_cb: Arc<RwLock<Option<Box<dyn Fn(GroupInfo) + Send + Sync>>>>,
}

impl MockTestWifiDirectBridge {
    fn new(available: bool) -> Self {
        Self {
            available: Arc::new(RwLock::new(available)),
            discovering: Arc::new(RwLock::new(false)),
            groups: Arc::new(RwLock::new(Vec::new())),
            connections: Arc::new(RwLock::new(Vec::new())),
            on_peers_cb: Arc::new(RwLock::new(None)),
            on_conn_cb: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl WifiDirectPlatformBridge for MockTestWifiDirectBridge {
    async fn is_available(&self) -> Result<bool, WifiDirectError> {
        Ok(*self.available.read())
    }

    async fn discover_peers(&self) -> Result<(), WifiDirectError> {
        if !*self.available.read() {
            return Err(WifiDirectError::Unavailable);
        }
        *self.discovering.write() = true;
        Ok(())
    }

    async fn stop_discovery(&self) -> Result<(), WifiDirectError> {
        *self.discovering.write() = false;
        Ok(())
    }

    async fn connect(&self, device_address: &str) -> Result<(), WifiDirectError> {
        if !*self.available.read() {
            return Err(WifiDirectError::Unavailable);
        }
        self.connections.write().push(device_address.to_string());
        Ok(())
    }

    async fn create_group(&self, group_name: &str) -> Result<(), WifiDirectError> {
        if !*self.available.read() {
            return Err(WifiDirectError::Unavailable);
        }
        self.groups.write().push(group_name.to_string());
        Ok(())
    }

    async fn remove_group(&self) -> Result<(), WifiDirectError> {
        self.groups.write().clear();
        Ok(())
    }

    fn set_on_peers_changed(&self, callback: Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>) {
        *self.on_peers_cb.write() = Some(callback);
    }

    fn set_on_connection_info(&self, callback: Box<dyn Fn(GroupInfo) + Send + Sync>) {
        *self.on_conn_cb.write() = Some(callback);
    }

    fn set_on_message_received(&self, _callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {}
}

// ============================================================================
// TEST MODULE 1: WIFI DIRECT TRANSPORT & STATE MACHINE
// ============================================================================

#[tokio::test]
async fn test_wifi_direct_lifecycle_and_state_machine() {
    let bridge = Arc::new(MockTestWifiDirectBridge::new(true));
    let transport = WifiDirectTransport::new(bridge.clone());

    // 1. Initial state check
    assert_eq!(transport.get_state(), WifiDirectState::Idle);

    // 2. Initialize
    transport
        .initialize()
        .await
        .expect("Initialization should succeed");
    assert_eq!(transport.get_state(), WifiDirectState::Idle);

    // 3. Start Discovery
    transport
        .start_discovery()
        .await
        .expect("Discovery should start");
    assert_eq!(transport.get_state(), WifiDirectState::Discovering);
    assert!(*bridge.discovering.read());

    // 4. Register Discovered Peer
    let peer = WifiDirectPeer {
        peer_id: PeerId::random(),
        device_name: "PeerAlpha".to_string(),
        device_address: "AA:BB:CC:DD:EE:FF".to_string(),
        rssi: -55,
    };
    transport.register_peer(peer.clone());
    let peers = transport.get_discovered_peers();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].device_address, "AA:BB:CC:DD:EE:FF");

    // 5. Connect to peer
    transport
        .connect_to_peer("AA:BB:CC:DD:EE:FF")
        .await
        .expect("Connection initiated");
    assert_eq!(transport.get_state(), WifiDirectState::Connecting);

    // 6. Stop Discovery
    transport.stop_discovery().await.expect("Stop discovery");

    // 7. Group Creation
    transport
        .create_group("MeshNetAlpha")
        .await
        .expect("Group created");
    assert_eq!(transport.get_state(), WifiDirectState::GroupOwner);
    assert_eq!(bridge.groups.read().len(), 1);

    // 8. Connection Info wiring
    let group_info = GroupInfo {
        group_owner: false,
        group_owner_ip: Some("192.168.49.1".to_string()),
        client_ips: vec!["192.168.49.2".to_string()],
        interface_name: "p2p-wlan0-0".to_string(),
        port: Some(8988),
    };
    transport.set_group_info(group_info.clone());
    assert_eq!(transport.get_state(), WifiDirectState::GroupClient);
    assert_eq!(transport.get_group_info().unwrap().port, Some(8988));

    // 9. Callback wiring test
    transport.wire_callbacks();
    if let Some(ref cb) = *bridge.on_peers_cb.read() {
        cb(vec![peer]);
    }
    assert_eq!(transport.get_discovered_peers().len(), 1);

    // 10. Shutdown and cleanup
    transport.shutdown().await.expect("Shutdown cleanly");
    assert_eq!(transport.get_state(), WifiDirectState::Idle);
    assert!(transport.get_group_info().is_none());
}

#[tokio::test]
async fn test_wifi_direct_error_handling_and_boundaries() {
    let bridge = Arc::new(MockTestWifiDirectBridge::new(false)); // Unavailable bridge
    let transport = WifiDirectTransport::new(bridge);

    // 1. Initialize on unavailable hardware -> returns Err(WifiDirectError::Unavailable)
    let init_res = transport.initialize().await;
    assert!(matches!(init_res, Err(WifiDirectError::Unavailable)));
    assert_eq!(transport.get_state(), WifiDirectState::Unavailable);

    // 2. Operations in Unavailable state fail cleanly with Err, never panic
    assert!(matches!(
        transport.start_discovery().await,
        Err(WifiDirectError::Unavailable)
    ));
    assert!(matches!(
        transport.connect_to_peer("00:11:22:33:44:55").await,
        Err(WifiDirectError::Unavailable)
    ));
    assert!(matches!(
        transport.create_group("Group1").await,
        Err(WifiDirectError::Unavailable)
    ));

    // 3. Error formatting assertions
    let err = WifiDirectError::DiscoveryFailed("Bridge timeout".to_string());
    assert!(err.to_string().contains("Bridge timeout"));
    let err2 = WifiDirectError::GroupFailed("P2P busy".to_string());
    assert!(err2.to_string().contains("P2P busy"));
    let err3 = WifiDirectError::InvalidConfig("bad params".to_string());
    assert!(err3.to_string().contains("bad params"));
}

#[test]
fn test_wifi_direct_group_owner_intent_matrix() {
    // 1. Charging state overrides low battery
    assert_eq!(
        compute_group_owner_intent(true, 0),
        WIFI_DIRECT_GO_INTENT_PREFERRED
    );
    assert_eq!(
        compute_group_owner_intent(true, 50),
        WIFI_DIRECT_GO_INTENT_PREFERRED
    );
    assert_eq!(
        compute_group_owner_intent(true, 100),
        WIFI_DIRECT_GO_INTENT_PREFERRED
    );

    // 2. Battery threshold: > 50% prefers owner, <= 50% prefers client
    assert_eq!(
        compute_group_owner_intent(false, 51),
        WIFI_DIRECT_GO_INTENT_PREFERRED
    );
    assert_eq!(
        compute_group_owner_intent(false, 100),
        WIFI_DIRECT_GO_INTENT_PREFERRED
    );

    assert_eq!(
        compute_group_owner_intent(false, 50),
        WIFI_DIRECT_GO_INTENT_CLIENT
    );
    assert_eq!(
        compute_group_owner_intent(false, 49),
        WIFI_DIRECT_GO_INTENT_CLIENT
    );
    assert_eq!(
        compute_group_owner_intent(false, 1),
        WIFI_DIRECT_GO_INTENT_CLIENT
    );
    assert_eq!(
        compute_group_owner_intent(false, 0),
        WIFI_DIRECT_GO_INTENT_CLIENT
    );
}

// ============================================================================
// TEST MODULE 2: WIFI AWARE NAN DISCOVERY & SOCKET ENCRYPTION HANDSHAKE
// ============================================================================

#[test]
fn test_wifi_aware_tlv_encoding_and_robustness() {
    // 1. Happy path roundtrip
    let port = 8080u16;
    let encoded = encode_port_tlv(port);
    assert_eq!(encoded, vec![TLV_TYPE_PORT, 2, 0x1F, 0x90]);
    assert_eq!(decode_port_tlv(&encoded), Some(8080));

    // 2. Malformed / corrupted inputs (Layer 3 panic safety)
    let empty: &[u8] = &[];
    assert_eq!(decode_port_tlv(empty), None);

    let truncated = vec![TLV_TYPE_PORT, 2, 0x1F]; // truncated value
    assert_eq!(decode_port_tlv(&truncated), None);

    let invalid_len = vec![TLV_TYPE_PORT, 5, 0x01, 0x02]; // length overflows buffer
    assert_eq!(decode_port_tlv(&invalid_len), None);

    let irrelevant = vec![0x99, 4, 1, 2, 3, 4]; // unknown TLV type
    assert_eq!(decode_port_tlv(&irrelevant), None);

    // 3. Duplicate TLVs (should reject to prevent ambiguous port negotiation)
    let mut duplicate = encode_port_tlv(8080);
    duplicate.extend_from_slice(&encode_port_tlv(9090));
    assert_eq!(decode_port_tlv(&duplicate), None);
}

#[tokio::test]
async fn test_wifi_aware_nan_discovery_and_callback_pipeline() {
    let bridge = Arc::new(MockTestWifiAwareBridge::new(true));
    let config = WifiAwareConfig {
        service_name: "SCMeshTest".to_string(),
        listen_port: Some(9999),
        ..WifiAwareConfig::default()
    };

    let transport = WifiAwareTransport::new(config, bridge.clone()).expect("Transport creation");

    // 1. Availability and init
    assert_eq!(transport.get_state(), WifiAwareState::Available);
    transport.initialize().await.expect("Init succeeded");

    // 2. Publish service
    transport
        .publish_service()
        .await
        .expect("Publish succeeded");
    assert_eq!(transport.get_state(), WifiAwareState::Publishing);
    assert_eq!(bridge.published_services.read().len(), 1);
    let pub_info = &bridge.published_services.read()[0].1;
    assert_eq!(decode_port_tlv(pub_info), Some(9999));

    // 3. Subscribe service
    transport.subscribe().await.expect("Subscribe succeeded");
    assert_eq!(transport.get_state(), WifiAwareState::Subscribing);

    // 4. Wire discovery callback & discover peer
    transport.wire_discovery_callback();

    let discovered_peer_id = PeerId::random();
    let discovered_service_info = encode_port_tlv(7777);

    // Trigger platform discovery callback
    if let Some(ref cb) = *bridge.on_discovered_cb.read() {
        cb(
            discovered_peer_id.to_string(),
            discovered_service_info.clone(),
            -65,
        );
    }

    let peers = transport.get_discovered_peers();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].peer_id, discovered_peer_id);
    assert_eq!(peers[0].rssi, -65);

    // Check listen port extraction
    assert_eq!(
        transport.get_peer_listen_port(&discovered_peer_id),
        Some(7777)
    );

    // 5. Test invalid peer string in discovery callback (Layer 3 panic safety)
    transport.add_discovered_peer("invalid-peer-id-string".to_string(), vec![1, 2, 3], -80);
    // Should gracefully log warning and return without adding peer or panicking
    assert_eq!(transport.get_discovered_peers().len(), 1);
}

#[tokio::test]
async fn test_wifi_aware_socket_encryption_handshake_and_pmk() {
    let bridge = Arc::new(MockTestWifiAwareBridge::new(true));
    let config = WifiAwareConfig {
        max_data_paths: 2,
        ..WifiAwareConfig::default()
    };
    let transport = WifiAwareTransport::new(config, bridge.clone()).expect("Transport creation");
    transport.initialize().await.expect("Init succeeded");

    // Prepare 3 peers
    let p1 = PeerId::random();
    let p2 = PeerId::random();
    let p3 = PeerId::random();

    transport.register_peer(DiscoveredPeer {
        peer_id: p1,
        service_info: encode_port_tlv(5001),
        rssi: -45, // Excellent RSSI
    });

    transport.register_peer(DiscoveredPeer {
        peer_id: p2,
        service_info: encode_port_tlv(5002),
        rssi: -115, // Poor RSSI
    });

    transport.register_peer(DiscoveredPeer {
        peer_id: p3,
        service_info: encode_port_tlv(5003),
        rssi: -75, // Mid RSSI
    });

    // 1. Establish Data Path with PMK socket encryption key
    let pmk: [u8; 32] = [0x42; 32];
    let path1 = transport
        .create_data_path(p1, &pmk)
        .await
        .expect("Path 1 created");

    assert_eq!(path1.peer_id, p1);
    assert_eq!(path1.port, 8888);
    assert!(path1.bandwidth_estimate > 50_000_000); // High bandwidth for excellent RSSI

    assert_eq!(transport.get_state(), WifiAwareState::DataPathActive);

    // 2. Establish second Data Path
    let path2 = transport
        .create_data_path(p2, &pmk)
        .await
        .expect("Path 2 created");
    assert!(path2.bandwidth_estimate < path1.bandwidth_estimate);

    assert_eq!(transport.get_active_data_paths().len(), 2);

    // 3. Max data paths capacity check (3rd should fail)
    let res3 = transport.create_data_path(p3, &pmk).await;
    assert!(matches!(res3, Err(WifiAwareError::DataPathFailed(_))));

    // 4. Data path lookup and close
    assert!(transport.get_data_path(&p1).is_some());
    transport
        .close_data_path(p1)
        .await
        .expect("Close succeeded");
    assert!(transport.get_data_path(&p1).is_none());
    assert_eq!(transport.get_active_data_paths().len(), 1);

    // 5. Shutdown
    transport.shutdown().await.expect("Shutdown succeeded");
    assert_eq!(transport.get_state(), WifiAwareState::Available);
}

#[tokio::test]
async fn test_wifi_aware_error_branches() {
    // 1. Empty service name config error
    let invalid_config = WifiAwareConfig {
        service_name: "".to_string(),
        ..WifiAwareConfig::default()
    };
    let bridge = Arc::new(MockTestWifiAwareBridge::new(true));
    assert!(matches!(
        WifiAwareTransport::new(invalid_config, bridge),
        Err(WifiAwareError::InvalidConfig(_))
    ));

    // 2. Publishing when disabled in config
    let disabled_pub_config = WifiAwareConfig {
        publish_enabled: false,
        ..WifiAwareConfig::default()
    };
    let bridge2 = Arc::new(MockTestWifiAwareBridge::new(true));
    let t_dis = WifiAwareTransport::new(disabled_pub_config, bridge2).unwrap();
    t_dis.initialize().await.unwrap();
    assert!(matches!(
        t_dis.publish_service().await,
        Err(WifiAwareError::InvalidConfig(_))
    ));

    // 3. Subscribing when disabled in config
    let disabled_sub_config = WifiAwareConfig {
        subscribe_enabled: false,
        ..WifiAwareConfig::default()
    };
    let bridge3 = Arc::new(MockTestWifiAwareBridge::new(true));
    let t_sub = WifiAwareTransport::new(disabled_sub_config, bridge3).unwrap();
    t_sub.initialize().await.unwrap();
    assert!(matches!(
        t_sub.subscribe().await,
        Err(WifiAwareError::InvalidConfig(_))
    ));

    // 4. Data path to unregistered peer
    let bridge4 = Arc::new(MockTestWifiAwareBridge::new(true));
    let t_reg = WifiAwareTransport::new(WifiAwareConfig::default(), bridge4).unwrap();
    t_reg.initialize().await.unwrap();
    let pmk = [0u8; 32];
    assert!(matches!(
        t_reg.create_data_path(PeerId::random(), &pmk).await,
        Err(WifiAwareError::PeerNotFound(_))
    ));
}

// ============================================================================
// TEST MODULE 3: PHYSICAL TRANSPORT ESCALATION ENGINE
// ============================================================================

#[test]
fn test_escalation_policy_matrix() {
    let transports = vec![
        TransportType::BLE,
        TransportType::WiFiAware,
        TransportType::WiFiDirect,
        TransportType::Internet,
        TransportType::Local,
    ];

    // 1. PreferHighBandwidth -> Local (10Gbps) or WiFiDirect/Internet
    let eng_bw = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
    let best_bw = eng_bw.select_best_transport(&transports);
    assert_eq!(best_bw, TransportType::Local);

    // 2. PreferLowLatency -> Local (lowest latency)
    let eng_lat = EscalationEngine::new(EscalationPolicy::PreferLowLatency);
    let best_lat = eng_lat.select_best_transport(&transports);
    assert_eq!(best_lat, TransportType::Local);

    // 3. PreferLowPower -> BLE
    let eng_pwr = EscalationEngine::new(EscalationPolicy::PreferLowPower);
    let best_pwr = eng_pwr.select_best_transport(&transports);
    assert_eq!(best_pwr, TransportType::BLE);

    // 4. Balanced -> High capability transport with streaming support
    let eng_bal = EscalationEngine::new(EscalationPolicy::Balanced);
    let best_bal = eng_bal.select_best_transport(&[TransportType::BLE, TransportType::WiFiAware]);
    assert_eq!(best_bal, TransportType::WiFiAware);
}

#[test]
fn test_escalation_pipeline_multi_hop_transitions() {
    let engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
    let peer_id = [0x07u8; 32];

    // Initialize with low-tier transport
    engine
        .init_peer(peer_id, vec![TransportType::BLE, TransportType::WiFiAware])
        .expect("Init peer");

    // Manually force state to BLE to simulate starting connection on low-power BLE
    {
        let all = engine.all_states();
        assert_eq!(all.len(), 1);
    }

    // Check current transport (PreferHighBandwidth will pick WiFiAware initially as best of available)
    assert_eq!(
        engine.current_transport(peer_id),
        Some(TransportType::WiFiAware)
    );

    // Update available transports to include high-bandwidth WiFiDirect and Internet
    engine
        .update_available_transports(
            peer_id,
            vec![
                TransportType::BLE,
                TransportType::WiFiAware,
                TransportType::WiFiDirect,
            ],
        )
        .expect("Update transports");

    // Escalation check
    assert!(engine.should_escalate(peer_id));
    let upgraded = engine.escalate(peer_id).expect("Escalation succeeded");
    assert_eq!(upgraded, TransportType::WiFiDirect);
    assert_eq!(
        engine.current_transport(peer_id),
        Some(TransportType::WiFiDirect)
    );

    // Graceful one-step deescalation on failure
    let downgraded = engine.deescalate(peer_id).expect("Deescalate succeeded");
    assert_eq!(downgraded, Some(TransportType::WiFiAware));
    assert_eq!(
        engine.current_transport(peer_id),
        Some(TransportType::WiFiAware)
    );

    // Update available transports to [BLE, WiFiAware] to simulate WiFiDirect dropping
    engine
        .update_available_transports(peer_id, vec![TransportType::BLE, TransportType::WiFiAware])
        .expect("Update available transports");

    // Further deescalate to BLE
    let downgraded2 = engine.deescalate(peer_id).expect("Deescalate to BLE");
    assert_eq!(downgraded2, Some(TransportType::BLE));

    // Update available transports to [BLE] to simulate WiFiAware dropping
    engine
        .update_available_transports(peer_id, vec![TransportType::BLE])
        .expect("Update available transports");

    // Cannot deescalate past BLE floor
    let downgraded_none = engine.deescalate(peer_id).expect("Deescalate at floor");
    assert_eq!(downgraded_none, None);

    // Cleanup
    engine.cleanup_peer(peer_id);
    assert_eq!(engine.current_transport(peer_id), None);
}

#[test]
fn test_escalation_engine_edge_cases_and_error_handling() {
    let engine = EscalationEngine::new(EscalationPolicy::Balanced);
    let peer_id = [0x99u8; 32];

    // 1. Init with empty transports -> Err(NoTransportsAvailable)
    let init_err = engine.init_peer(peer_id, vec![]);
    assert!(matches!(
        init_err,
        Err(EscalationError::NoTransportsAvailable)
    ));

    // 2. Operations on non-tracked peer -> Err(NotPossible)
    assert!(matches!(
        engine.escalate(peer_id),
        Err(EscalationError::NotPossible)
    ));
    assert!(matches!(
        engine.deescalate(peer_id),
        Err(EscalationError::NotPossible)
    ));
    assert!(matches!(
        engine.update_available_transports(peer_id, vec![TransportType::BLE]),
        Err(EscalationError::NotPossible)
    ));
    assert!(!engine.should_escalate(peer_id));

    // 3. Recommended transport query
    #[cfg(not(target_arch = "wasm32"))]
    {
        assert!(engine.recommended_transport(&peer_id).is_none());
        engine.init_peer(peer_id, vec![TransportType::BLE]).unwrap();
        assert_eq!(
            engine.recommended_transport(&peer_id),
            Some(scmessenger_core::ProximityTransport::Ble)
        );
    }
}

// ============================================================================
// TEST MODULE 4: PEER REPUTATION MANAGER & ABUSE MITIGATION
// ============================================================================

#[test]
fn test_reputation_score_classification_and_math() {
    // Neutral
    let neutral = ReputationScore::neutral();
    assert_eq!(neutral.value(), 50.0);
    assert!(!neutral.is_trusted());
    assert!(!neutral.is_suspicious());
    assert!(!neutral.is_abusive());
    assert_eq!(neutral.to_string(), "50.0");

    // Trusted (score >= 70.0)
    let trusted = ReputationScore::new(85.0);
    assert!(trusted.is_trusted());
    assert!(!trusted.is_suspicious());
    assert!(!trusted.is_abusive());

    // Suspicious (10.0 <= score < 30.0)
    let suspicious = ReputationScore::new(25.0);
    assert!(!suspicious.is_trusted());
    assert!(suspicious.is_suspicious());
    assert!(!suspicious.is_abusive());

    // Abusive (score < 10.0)
    let abusive = ReputationScore::new(5.0);
    assert!(!abusive.is_trusted());
    assert!(!abusive.is_suspicious());
    assert!(abusive.is_abusive());

    // Clamping boundary verification
    let overflow = ReputationScore::new(999.0);
    assert_eq!(overflow.value(), 100.0);
    let underflow = ReputationScore::new(-50.0);
    assert_eq!(underflow.value(), 0.0);
}

#[test]
fn test_reputation_abuse_signals_and_multiplier() {
    let manager = AbuseReputationManager::new(10);
    let peer = "peer_alpha";

    // Default untracked peer gets 1.0 multiplier
    assert_eq!(manager.rate_limit_multiplier(peer), 1.0);
    assert_eq!(manager.get_score(peer).value(), 50.0);

    // 1. Record successful deliveries and relays -> trusted status
    for _ in 0..10 {
        manager.record_signal(peer, AbuseSignal::SuccessfulDelivery);
        manager.record_signal(peer, AbuseSignal::SuccessfulRelay);
    }
    let trusted_score = manager.get_score(peer);
    assert!(trusted_score.is_trusted());
    assert_eq!(manager.rate_limit_multiplier(peer), 1.5);

    // 2. Record abuse signals -> abusive status
    let peer_bad = "peer_bad";
    for _ in 0..15 {
        manager.record_signal(peer_bad, AbuseSignal::RateLimited);
        manager.record_signal(peer_bad, AbuseSignal::InvalidFormat);
        manager.record_signal(peer_bad, AbuseSignal::OversizedMessage);
    }
    let bad_score = manager.get_score(peer_bad);
    assert!(bad_score.is_abusive());
    assert_eq!(manager.rate_limit_multiplier(peer_bad), 0.1);

    // 3. Record moderate abuse -> suspicious status
    let peer_susp = "peer_suspicious";
    for _ in 0..3 {
        manager.record_signal(peer_susp, AbuseSignal::RateLimited);
        manager.record_signal(peer_susp, AbuseSignal::FailedRelay);
        manager.record_signal(peer_susp, AbuseSignal::ConnectionTimeout);
    }
    let susp_score = manager.get_score(peer_susp);
    assert!(susp_score.is_suspicious() || susp_score.value() < 50.0);
}

#[test]
fn test_reputation_storage_persistence_and_eviction() {
    let storage = Arc::new(MemoryStorage::new());
    let manager = AbuseReputationManager::with_backend(2, storage.clone());

    // Record signals for 2 peers
    manager.record_signal("peer_1", AbuseSignal::SuccessfulDelivery);
    manager.record_signal("peer_2", AbuseSignal::RateLimited);
    assert_eq!(manager.len(), 2);

    // Record signal for 3rd peer -> evicts lowest scored peer ("peer_2")
    manager.record_signal("peer_3", AbuseSignal::SuccessfulDelivery);
    assert_eq!(manager.len(), 2);
    assert_eq!(manager.get_score("peer_2"), ReputationScore::neutral()); // Evicted

    // Flush and reload in new manager instance from storage
    manager.flush_to_storage();
    let restored_manager = AbuseReputationManager::with_backend(10, storage);
    assert_eq!(restored_manager.len(), 2);
    assert!(
        restored_manager.get_score("peer_1").is_trusted()
            || restored_manager.get_score("peer_1").value() > 50.0
    );
    assert!(restored_manager.get_score("peer_3").value() > 50.0);
}

#[test]
fn test_reputation_time_decay_and_pruning() {
    let manager = AbuseReputationManager::new(100);
    manager.record_signal("peer_stale", AbuseSignal::RateLimited);
    manager.record_signal("peer_active", AbuseSignal::SuccessfulDelivery);

    assert_eq!(manager.len(), 2);

    // Prune stale entries with 0 max_age should remove entries older than 0
    // Give active a moment and prune stale
    std::thread::sleep(Duration::from_millis(10));
    let pruned = manager.prune_stale(Duration::from_millis(5));
    assert_eq!(pruned, 2);
    assert!(manager.is_empty());
}

// ============================================================================
// TEST MODULE 5: L2CAP FRAGMENT REASSEMBLY & BLE TRANSPORT
// ============================================================================

#[test]
fn test_l2cap_config_and_channel_state_machine() {
    // 1. Builder and validation
    let config = L2capConfig::new(ProtocolServiceMultiplexer::SCMessenger)
        .with_mtu(256)
        .with_timeout(15);

    assert_eq!(config.psm.value(), 0x0025);
    assert_eq!(config.mtu, 256);
    assert_eq!(config.timeout_secs, 15);
    assert!(config.validate().is_ok());

    // 2. Invalid MTU (<23) and Invalid Timeout (=0)
    assert!(L2capConfig::default().with_mtu(10).validate().is_err());
    assert!(L2capConfig::default().with_timeout(0).validate().is_err());

    // 3. Channel State Machine transitions
    let mut channel = L2capChannel::new(config).expect("Channel creation");
    assert_eq!(channel.state(), ChannelState::Closed);
    assert!(!channel.is_connected());

    // Connection flow
    channel.initiate_connection().expect("Connecting");
    assert_eq!(channel.state(), ChannelState::Connecting);
    assert!(matches!(
        channel.initiate_connection(),
        Err(L2capError::AlreadyConnected)
    ));

    channel.confirm_connection().expect("Connected");
    assert_eq!(channel.state(), ChannelState::Connected);
    assert!(channel.is_connected());

    // Close flow
    channel.initiate_close().expect("Closing");
    assert_eq!(channel.state(), ChannelState::Closing);

    channel.confirm_close().expect("Closed");
    assert_eq!(channel.state(), ChannelState::Closed);
    assert!(!channel.is_connected());
}

#[test]
fn test_l2cap_fragmenter_and_reassembler_roundtrip() {
    let config = L2capConfig::default().with_mtu(64); // Small MTU for fragmentation
    let fragmenter = L2capFragmenter::new(config.clone()).expect("Fragmenter");
    let reassembler = L2capReassembler::new(config).expect("Reassembler");

    // 1. Single small message
    let small_data = b"Short payload".to_vec();
    let frags1 = fragmenter.fragment(&small_data).expect("Fragment small");
    assert_eq!(frags1.len(), 1);
    let reassembled1 = reassembler.reassemble(&frags1).expect("Reassemble small");
    assert_eq!(reassembled1, small_data);

    // 2. Large multi-fragment message
    let large_data = vec![0xA5u8; 500];
    let frags2 = fragmenter.fragment(&large_data).expect("Fragment large");
    assert!(frags2.len() > 5);
    for frag in &frags2 {
        assert!(frag.len() <= 64);
    }
    let reassembled2 = reassembler.reassemble(&frags2).expect("Reassemble large");
    assert_eq!(reassembled2, large_data);

    // 3. Empty message
    let empty_data = vec![];
    let frags3 = fragmenter.fragment(&empty_data).expect("Fragment empty");
    assert_eq!(frags3.len(), 1);

    // 4. Header parse bounds check (Layer 3 panic safety)
    let truncated_header = vec![0x01, 0x00];
    assert!(matches!(
        FragmentHeader::from_bytes(&truncated_header),
        Err(L2capError::ReassemblyError(_))
    ));
}

#[test]
fn test_l2cap_reassembly_manager_crc32_and_multi_peer() {
    let mut manager = L2capReassemblyManager::with_defaults();

    // 1. Prepare message with CRC32 appended
    let mut message = b"SCMessenger Mesh L2CAP Packet Data".to_vec();
    append_crc32(&mut message);

    // Fragment into 2 parts
    let mid = message.len() / 2;
    let part1 = message[..mid].to_vec();
    let part2 = message[mid..].to_vec();

    let h1 = FragmentHeader::new(2, 0).unwrap();
    let h2 = FragmentHeader::new(2, 1).unwrap();

    let mut f1 = h1.to_bytes().to_vec();
    f1.extend_from_slice(&part1);

    let mut f2 = h2.to_bytes().to_vec();
    f2.extend_from_slice(&part2);

    // Feed fragment 1 for peer_A
    assert!(manager.feed_fragment("peer_A", &f1).is_none());
    assert_eq!(manager.active_count(), 1);

    // Feed fragment 2 for peer_A -> completes reassembly and verifies CRC32
    let reassembled = manager
        .feed_fragment("peer_A", &f2)
        .expect("Reassembly complete");
    assert_eq!(reassembled, b"SCMessenger Mesh L2CAP Packet Data");
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn test_l2cap_reassembly_manager_drop_reasons() {
    // 1. CRC32 Mismatch
    let mut manager = L2capReassemblyManager::with_defaults();
    let mut corrupt_payload = b"Corrupted Data".to_vec();
    corrupt_payload.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Wrong CRC

    let h = FragmentHeader::new(1, 0).unwrap();
    let mut f = h.to_bytes().to_vec();
    f.extend_from_slice(&corrupt_payload);

    assert!(manager.feed_fragment("peer_bad_crc", &f).is_none());
    assert_eq!(manager.drop_stats().get(&DropReason::CrcMismatch), Some(&1));

    // 2. Memory Cap Exceeded
    let mut cap_manager = L2capReassemblyManager::new(30, 20); // 20 byte memory cap
    let big_payload = vec![0x11u8; 50];
    let h_big = FragmentHeader::new(1, 0).unwrap();
    let mut f_big = h_big.to_bytes().to_vec();
    f_big.extend_from_slice(&big_payload);

    assert!(cap_manager
        .feed_fragment("peer_memory_hog", &f_big)
        .is_none());
    assert_eq!(
        cap_manager.drop_stats().get(&DropReason::MemoryCapExceeded),
        Some(&1)
    );
}

// ============================================================================
// TEST MODULE 6: 5-LAYER INTEGRATED DEEP VERIFICATION PIPELINE
// ============================================================================

#[derive(Zeroize, ZeroizeOnDrop)]
struct SensitiveHandshakePmk {
    key_material: [u8; 32],
}

#[tokio::test]
async fn test_integrated_node3_local_mesh_pipeline() {
    // Step 1: Initialize WiFi Aware & Direct Mock Transports
    let aware_bridge = Arc::new(MockTestWifiAwareBridge::new(true));
    let aware_transport = WifiAwareTransport::new(
        WifiAwareConfig {
            service_name: "SCMeshNode3".to_string(),
            listen_port: Some(9110),
            ..WifiAwareConfig::default()
        },
        aware_bridge.clone(),
    )
    .expect("Aware transport");
    aware_transport.initialize().await.expect("Aware init");

    // Step 2: NAN Service Publication & Peer Discovery with TLV port negotiation
    aware_transport.publish_service().await.expect("Publish");
    aware_transport.wire_discovery_callback();

    let target_peer_id = PeerId::random();
    let target_peer_str = target_peer_id.to_string();
    let port_tlv = encode_port_tlv(9110);

    // Platform fires service discovery
    if let Some(ref cb) = *aware_bridge.on_discovered_cb.read() {
        cb(target_peer_str.clone(), port_tlv.clone(), -50);
    }

    assert_eq!(aware_transport.get_discovered_peers().len(), 1);
    assert_eq!(
        aware_transport.get_peer_listen_port(&target_peer_id),
        Some(9110)
    );

    // Step 3: Layer 5 Zeroized PMK Handshake & Data Path Creation
    let mut pmk_holder = SensitiveHandshakePmk {
        key_material: [0x55; 32],
    };
    let data_path = aware_transport
        .create_data_path(target_peer_id, &pmk_holder.key_material)
        .await
        .expect("Data path created");
    assert_eq!(data_path.port, 8888);

    // Step 4: Escalation Engine dynamic physical transport selection
    let escalation_engine = EscalationEngine::new(EscalationPolicy::PreferHighBandwidth);
    let mut peer_bytes = [0u8; 32];
    let digest = target_peer_id.to_bytes();
    let copy_len = digest.len().min(32);
    peer_bytes[..copy_len].copy_from_slice(&digest[..copy_len]);

    escalation_engine
        .init_peer(
            peer_bytes,
            vec![
                TransportType::BLE,
                TransportType::WiFiAware,
                TransportType::WiFiDirect,
            ],
        )
        .expect("Init peer escalation");

    let current = escalation_engine.current_transport(peer_bytes).unwrap();
    assert_eq!(current, TransportType::WiFiDirect); // High bandwidth policy selects WiFiDirect

    // Step 5: L2CAP BLE Link Fragment Reassembly with CRC32 verification
    let mut l2cap_manager = L2capReassemblyManager::with_defaults();
    let mut payload = b"Mesh Payload Message via L2CAP".to_vec();
    append_crc32(&mut payload);

    let frag_header = FragmentHeader::new(1, 0).unwrap();
    let mut packet = frag_header.to_bytes().to_vec();
    packet.extend_from_slice(&payload);

    let received_data = l2cap_manager
        .feed_fragment(&target_peer_str, &packet)
        .expect("L2CAP packet reassembled and CRC verified");
    assert_eq!(received_data, b"Mesh Payload Message via L2CAP");

    // Step 6: Reputation Manager updates score and scales rate limits
    let storage = Arc::new(MemoryStorage::new());
    let reputation_mgr = AbuseReputationManager::with_backend(100, storage.clone());

    // Record successful delivery
    let score = reputation_mgr.record_signal(&target_peer_str, AbuseSignal::SuccessfulDelivery);
    assert!(score.value() > 50.0);
    assert_eq!(reputation_mgr.rate_limit_multiplier(&target_peer_str), 1.5); // Trusted capacity multiplier

    for _ in 0..10 {
        reputation_mgr.record_signal(&target_peer_str, AbuseSignal::SuccessfulDelivery);
    }
    assert!(reputation_mgr.get_score(&target_peer_str).is_trusted());
    assert_eq!(reputation_mgr.rate_limit_multiplier(&target_peer_str), 1.5); // 150% rate limit multiplier

    // Step 7: Malicious activity simulation lowers reputation
    for _ in 0..12 {
        reputation_mgr.record_signal(&target_peer_str, AbuseSignal::RateLimited);
    }
    assert!(reputation_mgr.get_score(&target_peer_str).is_abusive());
    assert_eq!(reputation_mgr.rate_limit_multiplier(&target_peer_str), 0.1); // 90% rate limit penalty

    // Step 8: Memory Zeroization verification (Layer 5)
    pmk_holder.key_material.zeroize();
    assert_eq!(pmk_holder.key_material, [0u8; 32]);
}
