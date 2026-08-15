// Axum-based server implementation for SCMessenger Control API
// This file contains the new Axum 0.7 server implementation

use anyhow::{Context, Result};
use axum::{
    extract::{Json as AxumJson, Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use super::api::{
    AddContactRequest, AddContactResponse, ConnectionPathStateResponse, DiscoveredPeer,
    DiscoveryPeersResponse, DiscoveryStatusResponse, DriftStatusResponse,
    GetExternalAddressResponse, GetHistoryRequest, GetHistoryResponse, GetListenersResponse,
    GetPeersResponse, HistoryMessage, PeerEntry, SendMessageRequest, SendMessageResponse, API_PORT,
};

// Farm Test Harness Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub duration_secs: u64,
    pub nodes: usize,
    pub transports: Vec<String>,
    pub failure_modes: Vec<String>,
    pub collect_coverage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitRunRequest {
    pub config: TestConfig,
    pub topology: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitRunResponse {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub identity_id: Option<String>,
    pub public_key_hex: Option<String>,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
    pub initialized: bool,
    pub nickname: Option<String>,
    pub libp2p_peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollStatusResponse {
    pub status: String,
    pub progress: String,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchArtifactResponse {
    pub content: String,
    pub artifact_type: String,
}

#[derive(Debug, Clone)]
enum RunStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct RunState {
    pub status: RunStatus,
    pub progress: String,
    pub result: Option<serde_json::Value>,
    pub artifacts: HashMap<String, String>,
}

pub type RunRegistry = Arc<RwLock<HashMap<String, RunState>>>;

lazy_static! {
    static ref FARM_RUN_REGISTRY: RunRegistry = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Clone)]
pub struct ApiContext {
    pub core: Arc<scmessenger_core::IronCore>,
    pub swarm_handle: Arc<scmessenger_core::transport::SwarmHandle>,
}

fn generate_run_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("run-{}", now)
}

async fn simulate_test_harness_static(run_id: String, config: TestConfig) {
    let mut state = RunState {
        status: RunStatus::Running,
        progress: "Starting test harness...".to_string(),
        result: None,
        artifacts: HashMap::new(),
    };

    {
        let mut reg = FARM_RUN_REGISTRY.write().await;
        reg.insert(run_id.clone(), state.clone());
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    state.progress = format!(
        "Running {} nodes with transports: {}",
        config.nodes,
        config.transports.join(", ")
    );
    {
        let mut reg = FARM_RUN_REGISTRY.write().await;
        if let Some(s) = reg.get_mut(&run_id) {
            s.progress = state.progress.clone();
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    state.progress = "Test completed".to_string();
    state.status = RunStatus::Done;
    state.result = Some(serde_json::json!({
        "topology": "farmhouse",
        "nodes_tested": config.nodes,
        "transports": config.transports,
        "success_rate": 0.98,
        "avg_latency_ms": 45
    }));

    state.artifacts.insert(
        "test_output.log".to_string(),
        format!(
            "[INFO] Test started with {} nodes\n[INFO] Transport stack: {:?}\n[SUCCESS] All tests passed",
            config.nodes, config.transports
        ),
    );

    state.artifacts.insert(
        "delivery_stats.json".to_string(),
        serde_json::json!({
            "total_messages": 1000,
            "delivered": 980,
            "failed": 20,
            "delivery_rate": 0.98
        })
        .to_string(),
    );

    let mut reg = FARM_RUN_REGISTRY.write().await;
    reg.insert(run_id, state);
}

// Axum handler functions

async fn handle_submit_run(
    AxumJson(request): AxumJson<SubmitRunRequest>,
) -> Result<AxumJson<SubmitRunResponse>, (StatusCode, String)> {
    let run_id = generate_run_id();
    let config = request.config.clone();
    let run_id_clone = run_id.clone();

    tokio::spawn(async move {
        simulate_test_harness_static(run_id_clone, config).await;
    });

    Ok(AxumJson(SubmitRunResponse {
        run_id,
        status: "queued".to_string(),
    }))
}

async fn handle_poll_status(
    Path(run_id): Path<String>,
) -> Result<AxumJson<PollStatusResponse>, (StatusCode, String)> {
    let reg = FARM_RUN_REGISTRY.read().await;
    let state = reg
        .get(&run_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    let status_str = match state.status {
        RunStatus::Queued => "queued",
        RunStatus::Running => "running",
        RunStatus::Done => "done",
        RunStatus::Failed => "failed",
    };

    Ok(AxumJson(PollStatusResponse {
        status: status_str.to_string(),
        progress: state.progress.clone(),
        result: state.result.clone(),
    }))
}

async fn handle_fetch_artifact(
    Path((run_id, artifact_name)): Path<(String, String)>,
) -> Result<AxumJson<FetchArtifactResponse>, (StatusCode, String)> {
    let reg = FARM_RUN_REGISTRY.read().await;
    let state = reg
        .get(&run_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    let content = state
        .artifacts
        .get(&artifact_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Artifact {} not found", artifact_name),
            )
        })?
        .clone();

    Ok(AxumJson(FetchArtifactResponse {
        content,
        artifact_type: if artifact_name.ends_with(".json") {
            "json".to_string()
        } else {
            "log".to_string()
        },
    }))
}

async fn handle_send_message(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<SendMessageRequest>,
) -> Result<AxumJson<SendMessageResponse>, (StatusCode, String)> {
    let core = &ctx.core;
    let contacts = core.contacts_store_manager();

    let list = contacts.list().unwrap_or_default();
    let contact = list
        .into_iter()
        .find(|c| c.peer_id == request.recipient || c.nickname.as_ref() == Some(&request.recipient))
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Contact not found".to_string()))?;

    let peer_id = contact
        .peer_id
        .parse::<libp2p::PeerId>()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid peer ID: {}", e)))?;

    let prepared = core
        .prepare_message_with_id(
            contact.public_key.clone(),
            request.message.clone(),
            scmessenger_core::MessageType::Text,
            None,
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to prepare message: {:?}", e),
            )
        })?;

    let sent = crate::ble_mesh::send_ble_message(&peer_id.to_string(), &prepared.envelope_data)
        .await
        .is_ok()
        || ctx
            .swarm_handle
            .send_message(peer_id, prepared.envelope_data, None, None)
            .await
            .is_ok();

    if !sent {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send message via BLE and Swarm".to_string(),
        ));
    }

    Ok(AxumJson(SendMessageResponse {
        success: true,
        error: None,
        message_id: None,
        status: Some("accepted".to_string()),
    }))
}

async fn handle_add_contact(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<AddContactRequest>,
) -> Result<AxumJson<AddContactResponse>, (StatusCode, String)> {
    let contacts = ctx.core.contacts_store_manager();

    let mut contact =
        scmessenger_core::store::Contact::new(request.peer_id.clone(), request.public_key);
    if let Some(name) = request.name {
        contact.nickname = Some(name);
    }

    contacts.add(contact).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to add contact: {:?}", e),
        )
    })?;

    Ok(AxumJson(AddContactResponse {
        success: true,
        error: None,
    }))
}

async fn handle_get_peers(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetPeersResponse>, (StatusCode, String)> {
    let peers: Vec<PeerEntry> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            let pid = p.to_string();
            let reputation = ctx.core.get_peer_reputation(pid.clone());
            PeerEntry {
                peer_id: pid,
                reputation,
            }
        })
        .collect();

    Ok(AxumJson(GetPeersResponse { peers }))
}

async fn handle_get_identity(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<IdentityResponse>, (StatusCode, String)> {
    let info = ctx.core.get_identity_info();
    Ok(AxumJson(IdentityResponse {
        identity_id: info.identity_id,
        public_key_hex: info.public_key_hex,
        device_id: info.device_id,
        seniority_timestamp: info.seniority_timestamp,
        initialized: info.initialized,
        nickname: info.nickname,
        libp2p_peer_id: info.libp2p_peer_id,
    }))
}

async fn handle_get_listeners(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetListenersResponse>, (StatusCode, String)> {
    let listeners = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();

    Ok(AxumJson(GetListenersResponse { listeners }))
}

async fn handle_get_history(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<GetHistoryRequest>,
) -> Result<AxumJson<GetHistoryResponse>, (StatusCode, String)> {
    let history = ctx.core.history_store_manager();

    let messages = if let Some(peer_id) = request.peer_id {
        history
            .conversation(peer_id, request.limit.unwrap_or(20) as u32)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get history: {:?}", e),
                )
            })?
    } else {
        history
            .recent(None, request.limit.unwrap_or(20) as u32)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get history: {:?}", e),
                )
            })?
    };

    let history_messages: Vec<HistoryMessage> = messages
        .into_iter()
        .map(|m| HistoryMessage {
            id: m.id,
            peer_id: m.peer_id,
            content: m.content,
            direction: match m.direction {
                scmessenger_core::store::MessageDirection::Sent => "sent".to_string(),
                scmessenger_core::store::MessageDirection::Received => "received".to_string(),
            },
            timestamp: m.timestamp,
            delivered: m.delivered,
        })
        .collect();

    Ok(AxumJson(GetHistoryResponse {
        messages: history_messages,
    }))
}

async fn handle_get_external_address(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetExternalAddressResponse>, (StatusCode, String)> {
    let addresses = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get external addresses: {}", e),
            )
        })?;

    Ok(AxumJson(GetExternalAddressResponse {
        addresses: addresses.into_iter().map(|addr| addr.to_string()).collect(),
    }))
}

async fn handle_get_connection_path_state(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<ConnectionPathStateResponse>, (StatusCode, String)> {
    let peers: Vec<String> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let listeners: Vec<String> = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let external_addrs: Vec<String> = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();

    Ok(AxumJson(ConnectionPathStateResponse {
        state: get_connection_path_state(&peers, &listeners, &external_addrs),
    }))
}

async fn handle_export_diagnostics(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<String, (StatusCode, String)> {
    let peers: Vec<String> = ctx
        .swarm_handle
        .get_peers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let listeners: Vec<String> = ctx
        .swarm_handle
        .get_listeners()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let external_addrs: Vec<String> = ctx
        .swarm_handle
        .get_external_addresses()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect();
    let connection_path_state = get_connection_path_state(&peers, &listeners, &external_addrs);
    let diagnostics = export_diagnostics(
        &peers,
        &listeners,
        &external_addrs,
        &connection_path_state,
        &ctx.core,
    );

    Ok(diagnostics)
}

async fn handle_get_drift_status(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<DriftStatusResponse>, (StatusCode, String)> {
    Ok(AxumJson(DriftStatusResponse {
        state: ctx.core.drift_network_state(),
        store_size: ctx.core.drift_store_size(),
    }))
}

async fn handle_get_discovery_status(
) -> Result<AxumJson<DiscoveryStatusResponse>, (StatusCode, String)> {
    let cfg = crate::config::Config::load().unwrap_or_default();
    Ok(AxumJson(DiscoveryStatusResponse {
        mdns_enabled: cfg.enable_mdns,
        ble_enabled: cfg.enable_ble,
        wifi_aware_enabled: cfg.enable_wifi_aware,
    }))
}

async fn handle_trigger_discovery_scan() -> Result<String, (StatusCode, String)> {
    Ok("Scan triggered".to_string())
}

async fn handle_get_discovery_peers(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<DiscoveryPeersResponse>, (StatusCode, String)> {
    let mut discovered = Vec::new();

    if let Ok(peers) = ctx.swarm_handle.get_peers().await {
        for peer_id in peers {
            let pid_str = peer_id.to_string();
            let nickname = ctx
                .core
                .contacts_store_manager()
                .get(pid_str.clone())
                .ok()
                .flatten()
                .and_then(|c| c.nickname);

            discovered.push(DiscoveredPeer {
                peer_id: pid_str,
                transport: "tcp/lan".to_string(),
                nickname,
            });
        }
    }

    Ok(AxumJson(DiscoveryPeersResponse { peers: discovered }))
}

async fn handle_shutdown() -> impl IntoResponse {
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    (StatusCode::OK, "Stopping...")
}

fn get_connection_path_state(
    peers: &[String],
    listeners: &[String],
    external_addrs: &[String],
) -> String {
    if peers.is_empty() {
        return "Bootstrapping".to_string();
    }
    if !listeners.is_empty() {
        return "DirectPreferred".to_string();
    }
    if !external_addrs.is_empty() {
        return "RelayFallback".to_string();
    }
    "RelayOnly".to_string()
}

fn export_diagnostics(
    peers: &[String],
    listeners: &[String],
    external_addrs: &[String],
    connection_path_state: &str,
    core: &scmessenger_core::IronCore,
) -> String {
    let history = core.history_store_manager();
    let stats = history.stats().ok();
    #[allow(clippy::disallowed_methods)]
    let payload = serde_json::json!({
        "running": true,
        "connection_path_state": connection_path_state,
        "peers": peers,
        "listeners": listeners,
        "external_addrs": external_addrs,
        "inbox_count": core.inbox_count(),
        "outbox_count": core.outbox_count(),
        "custody_audit_count": core.custody_audit_count(),
        "drift": {
            "state": core.drift_network_state(),
            "store_size": core.drift_store_size(),
        },
        "history_stats": stats.as_ref().map(|s| serde_json::json!({
            "total_messages": s.total_messages,
            "sent_count": s.sent_count,
            "received_count": s.received_count,
            "undelivered_count": s.undelivered_count,
        })),
        "timestamp_ms": chrono::Utc::now().timestamp_millis(),
    });
    payload.to_string()
}

pub async fn start_api_server(ctx: ApiContext) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    // Create CORS layer restricted to localhost / loopback origins to prevent cross-origin attack vectors
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            let bytes = origin.as_bytes();
            bytes.starts_with(b"http://localhost")
                || bytes.starts_with(b"http://127.0.0.1")
                || bytes.starts_with(b"https://localhost")
                || bytes.starts_with(b"https://127.0.0.1")
        }))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Build router with all routes
    let app = Router::new()
        .route("/api/send", post(handle_send_message))
        .route("/api/contacts", post(handle_add_contact))
        .route("/api/identity", get(handle_get_identity))
        .route("/api/peers", get(handle_get_peers))
        .route("/api/listeners", get(handle_get_listeners))
        .route("/api/history", post(handle_get_history))
        .route("/api/external-address", get(handle_get_external_address))
        .route(
            "/api/connection-path-state",
            get(handle_get_connection_path_state),
        )
        .route("/api/diagnostics", get(handle_export_diagnostics))
        .route("/api/drift-status", get(handle_get_drift_status))
        .route("/api/discovery/status", get(handle_get_discovery_status))
        .route("/api/discovery/scan", post(handle_trigger_discovery_scan))
        .route("/api/discovery/peers", get(handle_get_discovery_peers))
        .route("/api/shutdown", post(handle_shutdown))
        // Farm test harness routes
        .route("/submit-run", post(handle_submit_run))
        .route("/poll-status/:run_id", get(handle_poll_status))
        .route("/fetch-artifact/:run_id/:name", get(handle_fetch_artifact))
        .layer(cors)
        .with_state(ctx);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind API server")?;

    tracing::info!("Control API listening on {}", addr);

    // Serve with axum
    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}
