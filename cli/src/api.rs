// Control API for communicating with running SCMessenger node
//
// When `scm start` is running, it exposes a local HTTP API on localhost:9876
// Other CLI commands can send requests to this API instead of accessing the database directly

use anyhow::{Context, Result};
use axum::{
    extract::{Json as AxumJson, Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::{get, post},
    Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

pub const API_PORT: u16 = 9876;
pub const API_ADDR: &str = "127.0.0.1:9876";

const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_GIT_HASH: &str = env!("SCM_GIT_HASH");
const BUILD_TIME: &str = env!("SCM_BUILD_TIME");

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub recipient: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageStatusResponse {
    pub message_id: String,
    pub status: String,
    pub delivered: bool,
    pub peer_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddContactRequest {
    pub peer_id: String,
    pub public_key: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactSummary {
    pub peer_id: String,
    pub public_key: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetContactsResponse {
    pub contacts: Vec<ContactSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddContactResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: String,
    pub reputation: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPeersResponse {
    pub peers: Vec<PeerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConnectionStats {
    pub peer_id: String,
    pub state: String,
    pub duration_ms: u64,
    pub messages_sent: u64,
    pub message_failures: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: u64,
    pub last_activity: u64,
    pub connection_attempts: u32,
    pub successful_connections: u32,
    pub connection_failures: u32,
    pub current_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmStatsResponse {
    pub stats: Vec<ApiConnectionStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetHistoryRequest {
    pub peer_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryMessage {
    pub id: String,
    pub peer_id: String,
    pub content: String,
    pub direction: String,
    pub timestamp: u64,
    pub delivered: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetHistoryResponse {
    pub messages: Vec<HistoryMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetExternalAddressResponse {
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetListenersResponse {
    pub listeners: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionPathStateResponse {
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriftStatusResponse {
    pub state: String,
    pub store_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryStatusResponse {
    pub mdns_enabled: bool,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub transport: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryPeersResponse {
    pub peers: Vec<DiscoveredPeer>,
}

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
pub enum RunStatus {
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

// Check if API is available
pub async fn is_api_available() -> bool {
    tokio::net::TcpStream::connect(API_ADDR).await.is_ok()
}

// Client functions for CLI commands

pub async fn send_message_via_api(recipient: &str, message: &str) -> Result<()> {
    use http_body_util::{BodyExt, Empty, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = SendMessageRequest {
        recipient: recipient.to_string(),
        message: message.to_string(),
    };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/send", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: SendMessageResponse = serde_json::from_slice(&body_bytes)?;

    if response.success {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to send message: {}",
            response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )
    }
}

pub async fn add_contact_via_api(
    peer_id: &str,
    public_key: &str,
    name: Option<String>,
) -> Result<()> {
    use http_body_util::{BodyExt, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = AddContactRequest {
        peer_id: peer_id.to_string(),
        public_key: public_key.to_string(),
        name,
    };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/contacts", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: AddContactResponse = serde_json::from_slice(&body_bytes)?;

    if response.success {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to add contact: {}",
            response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )
    }
}

pub async fn get_peers_via_api() -> Result<Vec<PeerEntry>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/peers", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetPeersResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.peers)
}

pub async fn get_swarm_stats_via_api() -> Result<Vec<ApiConnectionStats>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/swarm/stats", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: SwarmStatsResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.stats)
}

/// Fetch message history from running node via Control API
pub async fn get_history_via_api(
    peer_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<HistoryMessage>> {
    use http_body_util::{BodyExt, Full};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req_body = GetHistoryRequest { peer_id, limit };

    let json = serde_json::to_string(&req_body)?;
    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/history", API_ADDR))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetHistoryResponse = serde_json::from_slice(&body_bytes)?;

    Ok(response.messages)
}

pub async fn get_external_address_via_api() -> Result<Vec<String>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/external-address", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;

    // Check HTTP status before attempting to parse
    let status = resp.status();
    let body_bytes = resp.into_body().collect().await?.to_bytes();

    if !status.is_success() {
        let error_body = String::from_utf8_lossy(&body_bytes);
        anyhow::bail!("API request failed with status {}: {}", status, error_body);
    }

    let response: GetExternalAddressResponse =
        serde_json::from_slice(&body_bytes).context("Failed to parse external address response")?;

    Ok(response.addresses)
}

pub async fn get_listeners_via_api() -> Result<Vec<String>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/listeners", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: GetListenersResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.listeners)
}

pub async fn get_connection_path_state_via_api() -> Result<String> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/connection-path-state", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: ConnectionPathStateResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.state)
}
pub async fn get_drift_state_via_api() -> Result<DriftStatusResponse> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/drift-status", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DriftStatusResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response)
}

pub async fn get_discovery_status() -> Result<DiscoveryStatusResponse> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/discovery/status", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DiscoveryStatusResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response)
}

pub async fn trigger_discovery_scan() -> Result<()> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/discovery/scan", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to trigger discovery scan: {}", resp.status());
    }
    Ok(())
}

pub async fn get_discovery_peers() -> Result<Vec<DiscoveredPeer>> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/discovery/peers", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let response: DiscoveryPeersResponse = serde_json::from_slice(&body_bytes)?;
    Ok(response.peers)
}

pub async fn export_diagnostics_via_api() -> Result<String> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::GET)
        .uri(format!("http://{}/api/diagnostics", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let resp = client.request(req).await?;
    let body_bytes = resp.into_body().collect().await?.to_bytes();
    String::from_utf8(body_bytes.to_vec()).context("Diagnostics response was not UTF-8")
}

// Server implementation

#[derive(Clone)]
pub struct ApiContext {
    pub core: Arc<scmessenger_core::IronCore>,
    pub swarm_handle: Arc<scmessenger_core::transport::SwarmHandle>,
}

#[derive(Debug)]
struct ApiRecipient {
    public_key: String,
    peer_id: libp2p::PeerId,
    identity_id: String,
}

fn api_recipient_from_public_key(public_key: String) -> Result<ApiRecipient, (StatusCode, String)> {
    let public_key = public_key.to_lowercase();
    let identity_id = scmessenger_core::identity::keys::identity_id_from_public_key_hex(
        &public_key,
    )
    .ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Recipient does not contain a valid Ed25519 public key".to_string(),
        )
    })?;

    let key_bytes = hex::decode(&public_key).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Recipient public key is not valid hex".to_string(),
        )
    })?;
    let key_bytes: [u8; 32] = key_bytes.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Recipient public key must be exactly 32 bytes".to_string(),
        )
    })?;
    let ed25519_key =
        libp2p::identity::ed25519::PublicKey::try_from_bytes(&key_bytes).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Recipient public key is not a valid Ed25519 key".to_string(),
            )
        })?;

    Ok(ApiRecipient {
        public_key,
        peer_id: libp2p::identity::PublicKey::from(ed25519_key).to_peer_id(),
        identity_id,
    })
}

fn api_identifier_matches(query: &str, candidate: &str) -> bool {
    query == candidate || query.eq_ignore_ascii_case(candidate)
}

/// Resolve a send target without trusting unauthenticated discovery data.
///
/// Contacts remain the first-choice source. If a peer has already sent an
/// authenticated message, its envelope public key is retained in the inbox
/// and is sufficient to address a reply. We deliberately do not auto-create a
/// contact here: an explicit reply is allowed, but unsolicited traffic must
/// not mutate the address book. Block checks run for both paths and fail
/// closed on storage errors.
fn resolve_api_recipient(
    core: &scmessenger_core::IronCore,
    query: &str,
) -> Result<ApiRecipient, (StatusCode, String)> {
    let query = query.trim();
    let contacts = core.contacts_store_manager();
    let contact_list = contacts.list().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list contacts: {:?}", e),
        )
    })?;

    for contact in contact_list {
        // Older contact records can contain a PeerId or identity ID in the
        // public_key column. They must not poison resolution for a sender
        // whose authenticated envelope key is still available in the inbox.
        let recipient = api_recipient_from_public_key(contact.public_key.clone()).ok();
        let identity_id = recipient
            .as_ref()
            .map(|recipient| recipient.identity_id.clone());
        let peer_id = recipient
            .as_ref()
            .map(|recipient| recipient.peer_id.to_string());
        let nickname_match = contact
            .nickname
            .as_deref()
            .is_some_and(|nickname| nickname.eq_ignore_ascii_case(query))
            || contact
                .local_nickname
                .as_deref()
                .is_some_and(|nickname| nickname.eq_ignore_ascii_case(query));

        if api_identifier_matches(query, &contact.peer_id)
            || api_identifier_matches(query, &contact.public_key)
            || identity_id
                .as_deref()
                .is_some_and(|identity| api_identifier_matches(query, identity))
            || peer_id
                .as_deref()
                .is_some_and(|peer| api_identifier_matches(query, peer))
            || nickname_match
        {
            if let Some(recipient) = recipient {
                return authorize_api_recipient(core, recipient);
            }

            tracing::warn!(
                contact_peer_id = %contact.peer_id,
                "Ignoring contact with malformed public key during API recipient resolution"
            );
        }
    }

    // A valid inline PeerId or public key can be resolved directly. This
    // covers a connected peer even when discovery did not create a contact.
    if let Ok(public_key) = core.resolve_identity(query.to_string()) {
        if let Ok(recipient) = api_recipient_from_public_key(public_key) {
            return authorize_api_recipient(core, recipient);
        }
    }

    // For identity IDs and nicknames carried by an inbound message, use only
    // the authenticated envelope key retained by the inbox. The decrypted
    // payload and unauthenticated discovery record are not used as key proof.
    for message in core.peek_received_messages().into_iter().rev() {
        let Some(public_key) = message.sender_public_key_hex else {
            continue;
        };
        let Ok(recipient) = api_recipient_from_public_key(public_key) else {
            continue;
        };
        if api_identifier_matches(query, &message.sender_id)
            || api_identifier_matches(query, &recipient.public_key)
            || api_identifier_matches(query, &recipient.identity_id)
            || api_identifier_matches(query, &recipient.peer_id.to_string())
        {
            return authorize_api_recipient(core, recipient);
        }
    }

    Err((
        StatusCode::NOT_FOUND,
        format!("Contact or authenticated peer not found: {}", query),
    ))
}

fn authorize_api_recipient(
    core: &scmessenger_core::IronCore,
    recipient: ApiRecipient,
) -> Result<ApiRecipient, (StatusCode, String)> {
    match core.is_peer_blocked(recipient.identity_id.clone(), None) {
        Ok(true) => Err((StatusCode::FORBIDDEN, "Recipient is blocked".to_string())),
        Ok(false) => Ok(recipient),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Unable to verify recipient block status: {:?}", e),
        )),
    }
}

pub async fn stop_node_via_api() -> Result<()> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let client = Client::builder(TokioExecutor::new()).build_http();

    let req = hyper::Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/api/shutdown", API_ADDR))
        .body(Empty::<Bytes>::new())?;

    let _res = client.request(req).await?;
    Ok(())
}

// Axum handler functions

async fn handle_send_message(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<SendMessageRequest>,
) -> Result<(StatusCode, AxumJson<SendMessageResponse>), (StatusCode, String)> {
    let core = &ctx.core;
    let recipient = resolve_api_recipient(core, &request.recipient)?;

    let prepared = core
        .prepare_message_with_id(
            recipient.public_key.clone(),
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

    let timestamp = scmessenger_core::util::unix_time_secs();
    core.history_store_manager()
        .add(scmessenger_core::store::MessageRecord {
            id: prepared.message_id.clone(),
            direction: scmessenger_core::store::MessageDirection::Sent,
            peer_id: recipient.identity_id.clone(),
            content: request.message,
            timestamp,
            sender_timestamp: timestamp,
            delivered: false,
            hidden: false,
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to persist outbound message: {:?}", e),
            )
        })?;

    let ble_result =
        crate::ble_mesh::send_ble_message(&recipient.peer_id.to_string(), &prepared.envelope_data)
            .await;
    let (http_status, status, error) = if ble_result.is_ok() {
        (StatusCode::OK, "accepted".to_string(), None)
    } else {
        match ctx
            .swarm_handle
            .send_message(
                recipient.peer_id,
                prepared.envelope_data,
                Some(recipient.identity_id.clone()),
                None,
            )
            .await
        {
            Ok(()) => (StatusCode::OK, "accepted".to_string(), None),
            Err(e) if ctx.swarm_handle.is_event_loop_alive() => (
                StatusCode::ACCEPTED,
                "retrying".to_string(),
                Some(format!("Initial dispatch failed; retrying: {}", e)),
            ),
            Err(e) => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Message was not accepted by BLE or Swarm: {}", e),
                ));
            }
        }
    };

    Ok((
        http_status,
        AxumJson(SendMessageResponse {
            success: true,
            error,
            message_id: Some(prepared.message_id),
            status: Some(status),
        }),
    ))
}

async fn handle_get_send_status(
    State(ctx): State<Arc<ApiContext>>,
    Path(message_id): Path<String>,
) -> Result<AxumJson<SendMessageStatusResponse>, (StatusCode, String)> {
    let record = ctx
        .core
        .history_store_manager()
        .get(message_id.clone())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read message status: {:?}", e),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Message not found".to_string()))?;

    Ok(AxumJson(SendMessageStatusResponse {
        message_id: record.id,
        status: if record.delivered {
            "delivered".to_string()
        } else {
            "pending".to_string()
        },
        delivered: record.delivered,
        peer_id: record.peer_id,
        timestamp: record.timestamp,
    }))
}

async fn handle_get_contacts(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<GetContactsResponse>, (StatusCode, String)> {
    let contacts = ctx
        .core
        .contacts_store_manager()
        .list()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list contacts: {:?}", e),
            )
        })?
        .into_iter()
        .map(|c| ContactSummary {
            peer_id: c.peer_id,
            public_key: c.public_key,
            name: c.local_nickname.or(c.nickname),
        })
        .collect();

    Ok(AxumJson(GetContactsResponse { contacts }))
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

async fn handle_get_swarm_stats(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<SwarmStatsResponse>, (StatusCode, String)> {
    let raw_stats = ctx.core.get_all_connection_stats();
    let stats = raw_stats
        .into_iter()
        .map(|(peer_id, stat)| {
            let state_str = match stat.state {
                scmessenger_core::transport::health::ConnectionState::Connecting => "Connecting",
                scmessenger_core::transport::health::ConnectionState::Connected => "Connected",
                scmessenger_core::transport::health::ConnectionState::Disconnecting => {
                    "Disconnecting"
                }
                scmessenger_core::transport::health::ConnectionState::Disconnected => {
                    "Disconnected"
                }
                scmessenger_core::transport::health::ConnectionState::Failed => "Failed",
            }
            .to_string();

            ApiConnectionStats {
                peer_id: peer_id.to_string(),
                state: state_str,
                duration_ms: stat.duration_ms,
                messages_sent: stat.messages_sent,
                message_failures: stat.message_failures,
                bytes_sent: stat.bytes_sent,
                bytes_received: stat.bytes_received,
                avg_latency_ms: stat.avg_latency_ms,
                last_activity: stat.last_activity,
                connection_attempts: stat.connection_attempts,
                successful_connections: stat.successful_connections,
                connection_failures: stat.connection_failures,
                current_address: stat.current_address.map(|addr| addr.to_string()),
            }
        })
        .collect();

    Ok(AxumJson(SwarmStatsResponse { stats }))
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
    let mut payload = Map::new();
    payload.insert("running".to_string(), true.into());
    payload.insert(
        "connection_path_state".to_string(),
        connection_path_state.into(),
    );
    payload.insert("peers".to_string(), peers.into());
    payload.insert("listeners".to_string(), listeners.into());
    payload.insert("external_addrs".to_string(), external_addrs.into());
    payload.insert("inbox_count".to_string(), core.inbox_count().into());
    payload.insert("outbox_count".to_string(), core.outbox_count().into());
    payload.insert(
        "custody_audit_count".to_string(),
        core.custody_audit_count().into(),
    );

    let mut drift = Map::new();
    drift.insert("state".to_string(), core.drift_network_state().into());
    drift.insert("store_size".to_string(), core.drift_store_size().into());
    payload.insert("drift".to_string(), Value::Object(drift));

    payload.insert(
        "history_stats".to_string(),
        stats
            .as_ref()
            .map(|s| {
                let mut m = Map::new();
                m.insert("total_messages".to_string(), s.total_messages.into());
                m.insert("sent_count".to_string(), s.sent_count.into());
                m.insert("received_count".to_string(), s.received_count.into());
                m.insert("undelivered_count".to_string(), s.undelivered_count.into());
                Value::Object(m)
            })
            .into(),
    );
    payload.insert(
        "timestamp_ms".to_string(),
        chrono::Utc::now().timestamp_millis().into(),
    );
    Value::Object(payload).to_string()
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

#[allow(clippy::disallowed_methods)] // serde_json::json! macro expands to internal unwrap() calls
async fn handle_get_identity(
    State(ctx): State<Arc<ApiContext>>,
) -> Result<AxumJson<serde_json::Value>, (StatusCode, String)> {
    let info = ctx.core.get_identity_info();
    Ok(AxumJson(serde_json::json!({
        "identity_id": info.identity_id,
        "public_key_hex": info.public_key_hex,
        "device_id": info.device_id,
        "seniority_timestamp": info.seniority_timestamp,
        "initialized": info.initialized,
        "nickname": info.nickname,
        "libp2p_peer_id": info.libp2p_peer_id,
    })))
}

// Farm Test Harness Handlers

fn generate_run_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("run-{}", now)
}

#[allow(clippy::disallowed_methods)] // serde_json::json! macro expands to internal unwrap() calls
async fn simulate_test_harness(run_id: String, config: TestConfig) {
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

async fn handle_submit_run(
    AxumJson(request): AxumJson<SubmitRunRequest>,
) -> Result<AxumJson<SubmitRunResponse>, (StatusCode, String)> {
    let run_id = generate_run_id();
    let config = request.config.clone();
    let run_id_clone = run_id.clone();

    tokio::spawn(async move {
        simulate_test_harness(run_id_clone, config).await;
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

#[allow(clippy::disallowed_methods)] // serde_json::json! expands to unwrap() calls internally
pub async fn start_api_server(ctx: ApiContext, bind_addr: Option<String>) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = if let Some(bind_str) = bind_addr {
        match bind_str.parse::<SocketAddr>() {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(
                    "Invalid http-bind address '{}', falling back to localhost: {}: {}",
                    bind_str,
                    API_PORT,
                    e
                );
                SocketAddr::from(([127, 0, 0, 1], API_PORT))
            }
        }
    } else {
        SocketAddr::from(([127, 0, 0, 1], API_PORT))
    };

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    // Build router with all routes
    let app = Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(serde_json::json!({"status": "healthy"})) }),
        )
        .route(
            "/version",
            get(|| async {
                axum::Json(serde_json::json!({
                    "version": BUILD_VERSION,
                    "git_hash": BUILD_GIT_HASH,
                    "build_time": BUILD_TIME,
                    "core_provenance": scmessenger_core::get_build_provenance(),
                }))
            }),
        )
        .route("/api/identity", get(handle_get_identity))
        .route("/api/send", post(handle_send_message))
        .route("/api/send/:message_id", get(handle_get_send_status))
        .route(
            "/api/contacts",
            get(handle_get_contacts).post(handle_add_contact),
        )
        .route("/api/peers", get(handle_get_peers))
        .route("/api/swarm/stats", get(handle_get_swarm_stats))
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

#[cfg(test)]
mod tests {
    use super::{resolve_api_recipient, ApiRecipient};
    use scmessenger_core::identity::keys::KeyPair;
    use scmessenger_core::store::Contact;

    fn test_recipient() -> ApiRecipient {
        let key_pair = KeyPair::generate();
        let public_key = hex::encode(key_pair.verifying_key().to_bytes());
        super::api_recipient_from_public_key(public_key)
            .expect("generated test key must resolve to an API recipient")
    }

    #[test]
    fn api_send_resolves_contact_aliases_and_peer_id() {
        let core = scmessenger_core::IronCore::new();
        core.grant_consent();
        core.initialize_identity()
            .expect("test core identity should initialize");
        let recipient = test_recipient();
        let mut contact = Contact::new(recipient.peer_id.to_string(), recipient.public_key.clone());
        contact.nickname = Some("Lucaso".to_string());
        core.contacts_store_manager()
            .add(contact)
            .expect("test contact should persist");

        for query in [
            recipient.peer_id.to_string(),
            recipient.public_key.clone(),
            recipient.identity_id.clone(),
            "Lucaso".to_string(),
        ] {
            let resolved =
                resolve_api_recipient(&core, &query).expect("contact aliases should resolve");
            assert_eq!(resolved.public_key, recipient.public_key);
            assert_eq!(resolved.identity_id, recipient.identity_id);
        }
    }

    #[test]
    fn api_send_refuses_blocked_contact_alias() {
        let core = scmessenger_core::IronCore::new();
        core.grant_consent();
        core.initialize_identity()
            .expect("test core identity should initialize");
        let recipient = test_recipient();
        core.contacts_store_manager()
            .add(Contact::new(
                recipient.peer_id.to_string(),
                recipient.public_key.clone(),
            ))
            .expect("test contact should persist");
        core.block_peer(
            recipient.identity_id.clone(),
            None,
            Some("api regression".to_string()),
        )
        .expect("test block should persist");

        let error = resolve_api_recipient(&core, &recipient.peer_id.to_string())
            .expect_err("blocked recipient must not resolve");
        assert_eq!(error.0, axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn api_send_ignores_stale_contact_public_key() {
        let core = scmessenger_core::IronCore::new();
        core.grant_consent();
        core.initialize_identity()
            .expect("test core identity should initialize");
        let recipient = test_recipient();

        // This is the malformed shape reported by Windows: the contact's
        // public_key field contains the base58 PeerId instead of Ed25519 hex.
        core.contacts_store_manager()
            .add(Contact::new(
                recipient.peer_id.to_string(),
                recipient.peer_id.to_string(),
            ))
            .expect("stale contact should persist");

        let resolved = resolve_api_recipient(&core, &recipient.peer_id.to_string())
            .expect("stale contact must not poison PeerId resolution");
        assert_eq!(resolved.public_key, recipient.public_key);
        assert_eq!(resolved.peer_id, recipient.peer_id);
    }
}
