use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc, Mutex};

// =====================================================================

// Stub types for BLE mesh UI integration (Phase 1B wiring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    MessageReceived {
        from: String,
        message_id: String,
        content: String,
        timestamp: u64,
    },
    NetworkStatus {
        status: String,
        peer_count: usize,
    },
    PeerDiscovered {
        peer_id: String,
        transport: String,
        public_key: String,
        identity: String,
    },
    IdentityInfo {
        peer_id: String,
        public_key: String,
        nickname: Option<String>,
        libp2p_peer_id: Option<String>,
    },
    IdentityExportData {
        identity_id: String,
        public_key: String,
        private_key: String,
        storage_path: String,
    },
    ContactList {
        contacts: Vec<serde_json::Value>,
    },
    HistoryList {
        peer_id: String,
        messages: Vec<serde_json::Value>,
    },
    MessageStatus {
        message_id: String,
        status: String,
    },
    Error {
        message: String,
    },
    ConfigValue {
        key: String,
        value: String,
    },
    ConfigData {
        config: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiOutbound {
    Legacy(UiEvent),
    JsonRpc(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiCommand {
    IdentityShow,
    IdentityExport,
    ContactList,
    HistoryList {
        peer_id: String,
        limit: Option<u32>,
    },
    Status,
    Send {
        recipient: String,
        message: String,
        id: Option<String>,
    },
    ContactAdd {
        peer_id: String,
        name: Option<String>,
        public_key: Option<String>,
    },
    ContactRemove {
        contact: String,
    },
    ConfigGet {
        key: String,
    },
    ConfigList,
    ConfigSet {
        key: String,
        value: String,
    },
    ConfigBootstrapAdd {
        multiaddr: String,
    },
    ConfigBootstrapRemove {
        multiaddr: String,
    },
    FactoryReset,
    Restart,
    DaemonRpc {
        id: String,
        intent: String,
    },
}

pub struct WebContext {
    pub node_peer_id: String,
    pub node_public_key: String,
    pub bootstrap_nodes: Vec<String>,
    pub ledger: Arc<Mutex<crate::ledger::ConnectionLedger>>,
    pub peers: Arc<Mutex<HashMap<PeerId, Option<String>>>>,
    pub start_time: Instant,
    pub transport_bridge: Arc<Mutex<crate::transport_bridge::TransportBridge>>,
    pub ui_port: u16,
    /// Optional IronCore reference for WASM JSON-RPC bridge handlers
    /// (contacts, settings, history, blocking). None when core is not
    /// available (e.g. bootstrap-only CLI modes).
    pub core: Option<Arc<scmessenger_core::IronCore>>,
}

impl Clone for WebContext {
    fn clone(&self) -> Self {
        Self {
            node_peer_id: self.node_peer_id.clone(),
            node_public_key: self.node_public_key.clone(),
            bootstrap_nodes: self.bootstrap_nodes.clone(),
            ledger: Arc::clone(&self.ledger),
            peers: Arc::clone(&self.peers),
            start_time: self.start_time,
            transport_bridge: Arc::clone(&self.transport_bridge),
            ui_port: self.ui_port,
            core: self.core.clone(),
        }
    }
}

impl std::fmt::Debug for WebContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebContext").finish_non_exhaustive()
    }
}

// JSON-RPC types imported from core (shared wire contract with WASM client).
use scmessenger_core::wasm_support::rpc::{
    parse_intent, rpc_error, rpc_result, ClientIntent, JsonRpcErrorBody, JsonRpcNotification,
    JsonRpcRequest, JsonRpcResponse, ERR_PARAMS,
};

/// WebSocket message sender handle connected clients.
type WsSender = futures::channel::mpsc::UnboundedSender<warp::ws::Message>;
type WsSenderList = Arc<Mutex<Vec<WsSender>>>;

use warp::Filter;

/// Resolve a message's sender to a canonical Ed25519 public key, or None if
/// the sender identifier is UNRESOLVABLE.
///
/// T3/P4 (identifier-gate follow-up): the pending-requests listing must FAIL
/// CLOSED on resolution failure. A sender whose identifier cannot be resolved
/// to a real Ed25519 public key -- via the authenticated envelope key
/// (`sender_public_key_hex`, the canonical source) or a valid key-valued
/// `sender_id` (legacy pre-field messages) -- cannot be proven known-or-blocked,
/// so it must be suppressed from the listing rather than shown as a clean
/// request. This pure helper centralizes the resolvability decision so it is
/// unit-testable without a live core.
fn resolve_sender_public_key(
    sender_public_key_hex: &Option<String>,
    sender_id: &str,
) -> Option<String> {
    sender_public_key_hex
        .clone()
        .filter(|k| scmessenger_core::identity::keys::is_valid_public_key(k))
        .or_else(|| {
            scmessenger_core::identity::keys::is_valid_public_key(sender_id)
                .then(|| sender_id.to_string())
        })
}

/// True iff the sender identifier resolves to a valid Ed25519 public key (see
/// [`resolve_sender_public_key`]).
fn sender_is_resolvable(sender_public_key_hex: &Option<String>, sender_id: &str) -> bool {
    resolve_sender_public_key(sender_public_key_hex, sender_id).is_some()
}

/// Stable request key shared by pending-list and accept lookups.
/// Authenticated public keys take precedence; legacy identity IDs are a safe
/// fallback only when they have the expected identity-ID format.
fn message_request_key(sender_public_key_hex: &Option<String>, sender_id: &str) -> Option<String> {
    if let Some(key) = sender_public_key_hex
        .as_deref()
        .filter(|key| scmessenger_core::identity::keys::is_valid_public_key(key))
    {
        return scmessenger_core::identity::keys::identity_id_from_public_key_hex(key);
    }

    // Older messages used sender_id for the public key. Preserve the same
    // canonical request key when that legacy field is explicitly key-shaped.
    if scmessenger_core::identity::keys::is_valid_public_key(sender_id) {
        return scmessenger_core::identity::keys::identity_id_from_public_key_hex(sender_id);
    }

    // Without an authenticated envelope key, a legacy public key and a
    // legacy identity_id are intentionally kept in their wire form. Both are
    // 64-hex identifiers and cannot be distinguished safely from plaintext.
    scmessenger_core::identity::keys::is_valid_identity_id(sender_id).then(|| sender_id.to_string())
}

/// Start the warp HTTP + WebSocket server on `127.0.0.1:<port>`.
///
/// Returns a broadcast sender for pushing server events to connected clients
/// and a command receiver for inbound UI commands from the WebSocket bridge.
pub async fn start(
    port: u16,
    ctx: Arc<WebContext>,
) -> anyhow::Result<(
    broadcast::Sender<UiOutbound>,
    mpsc::UnboundedReceiver<UiCommand>,
)> {
    let (ui_tx, _) = broadcast::channel::<UiOutbound>(256);
    let (ui_cmd_tx, ui_cmd_rx) = mpsc::unbounded_channel::<UiCommand>();

    // Shared list of WebSocket senders for broadcast push notifications.
    let ws_senders: WsSenderList = Arc::new(Mutex::new(Vec::new()));

    // Serve a minimal landing page at GET /
    let ctx_landing = ctx.clone();
    let landing = warp::path::end().map(move || {
        let body = format!(
            "<!DOCTYPE html><html><head><title>SCMessenger</title></head>\
             <body><h1>SCMessenger</h1><p>Node: {}</p><p>Public Key: {}</p>\
             <p>Uptime: {:?}</p></body></html>",
            ctx_landing.node_peer_id,
            ctx_landing.node_public_key,
            ctx_landing.start_time.elapsed(),
        );
        warp::reply::html(body)
    });

    // WebSocket route at GET /ws — upgrades to JSON-RPC bridge
    let ws_senders_filter = ws_senders.clone();
    let ctx_ws = ctx.clone();
    let ui_tx_ws = ui_tx.clone();
    let ui_cmd_tx_ws = ui_cmd_tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let ctx = ctx_ws.clone();
            let ui_tx = ui_tx_ws.clone();
            let ui_cmd_tx = ui_cmd_tx_ws.clone();
            let senders = ws_senders_filter.clone();
            ws.on_upgrade(move |websocket| {
                handle_ws_connection(websocket, ctx, ui_tx, ui_cmd_tx, senders)
            })
        });

    // Static route for /ui — serves the web app
    let ui_route = warp::path("ui").and(warp::fs::dir("ui"));

    // Static route for /wasm — serves built WASM assets used by the UI
    let wasm_route = warp::path("wasm").and(warp::fs::dir("wasm"));

    let routes = landing.or(ws_route).or(ui_route).or(wasm_route);

    // Bind and serve on 127.0.0.1 only (local bridge, never exposed to network).
    let bound_addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();

    // Bind the TCP listener first so failures are reported instead of silently
    // panicking inside a spawned task.
    let tcp_listener = match tokio::net::TcpListener::bind(bound_addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                "[ERROR] Failed to bind Warp server on {}: {}",
                bound_addr,
                e
            );
            return Err(anyhow::anyhow!("Warp bind failed on {}: {}", bound_addr, e));
        }
    };
    let local_addr = tcp_listener.local_addr()?;
    // warp::serve(...).incoming(listener).run() is the correct API chain
    tokio::spawn(warp::serve(routes).incoming(tcp_listener).run());
    tracing::info!("Warp HTTP+WS server listening on ws://{}", local_addr);

    Ok((ui_tx, ui_cmd_rx))
}

/// Handle a single upgraded WebSocket connection.
///
/// Reads JSON-RPC text frames, dispatches them to `handle_jsonrpc_request`,
/// and sends the serialised response back to the client. Also subscribes to
/// the UI broadcast and forwards server-push notifications.
async fn handle_ws_connection(
    websocket: warp::ws::WebSocket,
    ctx: Arc<WebContext>,
    ui_tx: broadcast::Sender<UiOutbound>,
    ui_cmd_tx: mpsc::UnboundedSender<UiCommand>,
    senders: WsSenderList,
) {
    use futures::StreamExt;
    use futures_util::SinkExt;

    let (mut ws_tx, mut ws_rx) = websocket.split();

    // Channel to forward broadcast notifications to this client.
    let (client_tx, mut client_rx) = futures::channel::mpsc::unbounded::<warp::ws::Message>();

    // Register this client's sender for broadcast push.
    senders.lock().await.push(client_tx.clone());

    // Subscribe to UI broadcast for push notifications.
    let mut ui_rx = ui_tx.subscribe();

    // Spawn task that reads from the broadcast and forwards to this client.
    let senders_task = senders.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = ui_rx.recv() => {
                    match res {
                        Ok(outbound) => {
                            let text = match &outbound {
                                UiOutbound::JsonRpc(val) => serde_json::to_string(val).unwrap_or_default(),
                                UiOutbound::Legacy(evt) => serde_json::to_string(evt).unwrap_or_default(),
                            };
                            if client_tx.unbounded_send(warp::ws::Message::text(text)).is_err() {
                                break; // client disconnected
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("WS client lagged {} broadcast messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = client_rx.next() => {
                    // Placeholder for client->broadcast relay if needed.
                }
            }
        }
        // Unregister on disconnect.
        senders_task.lock().await.retain(|s| !s.is_closed());
    });

    // Read loop: process incoming JSON-RPC text frames from the client.
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WebSocket receive error: {}", e);
                break;
            }
        };

        let text = if msg.is_text() {
            msg.to_str().unwrap_or("").to_string()
        } else if msg.is_close() {
            break;
        } else {
            continue;
        };

        let response = handle_jsonrpc_request(&text, &ctx, &ui_cmd_tx).await;
        if let Err(e) = ws_tx
            .send(warp::ws::Message::text(
                serde_json::to_string(&response).unwrap_or_default(),
            ))
            .await
        {
            tracing::warn!("WebSocket send error: {}", e);
            break;
        }
    }

    // Cleanup: remove this client's sender.
    senders.lock().await.retain(|s| !s.is_closed());
    tracing::info!("WebSocket client disconnected");
}

/// Dispatch a single JSON-RPC request through the core intent parser and
/// return a JSON-RPC response.
pub async fn handle_jsonrpc_request(
    raw: &str,
    ctx: &WebContext,
    _ui_cmd_tx: &mpsc::UnboundedSender<UiCommand>,
) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => {
            return rpc_error(
                None,
                JsonRpcErrorBody {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                    data: None,
                },
            )
        }
    };

    let id = req.id.clone();
    let intent = match parse_intent(&req) {
        Ok(i) => i,
        Err(e) => return rpc_error(id, e),
    };

    match intent {
        ClientIntent::GetIdentity {} => {
            let peer_count = ctx.peers.lock().await.len();
            let mut m = Map::new();
            m.insert("identityId".to_string(), ctx.node_peer_id.clone().into());
            m.insert(
                "publicKeyHex".to_string(),
                ctx.node_public_key.clone().into(),
            );
            m.insert("peerCount".to_string(), peer_count.into());
            m.insert(
                "bootstrapNodes".to_string(),
                ctx.bootstrap_nodes.clone().into(),
            );
            m.insert(
                "uptimeSecs".to_string(),
                ctx.start_time.elapsed().as_secs().into(),
            );
            let mut payload = Value::Object(m);
            // Enrich with core identity fields when available
            if let Some(ref core) = ctx.core {
                let info = core.get_identity_info();
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("nickname".to_string(), info.nickname.into());
                    obj.insert(
                        "libp2pPeerId".to_string(),
                        info.libp2p_peer_id.clone().into(),
                    );
                    obj.insert("initialized".to_string(), info.initialized.into());
                    if let Some(local_peer_id) = info.libp2p_peer_id {
                        obj.insert("localPeerId".to_string(), local_peer_id.into());
                    }
                }
            }
            rpc_result(id, payload)
        }
        ClientIntent::InitializeIdentity {} => {
            if let Some(ref core) = ctx.core {
                match core.initialize_identity() {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("initialized".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to initialize identity: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::SendMessage {
            recipient,
            message,
            id: msg_id,
        } => {
            // Forward as UiCommand so the main loop picks it up.
            let cmd = UiCommand::Send {
                recipient,
                message,
                id: msg_id,
            };
            if let Err(e) = _ui_cmd_tx.send(cmd) {
                return rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: format!("Command channel closed: {}", e),
                        data: None,
                    },
                );
            }
            let mut m = Map::new();
            m.insert("status".to_string(), "queued".into());
            rpc_result(id, Value::Object(m))
        }
        ClientIntent::ScanPeers {} => {
            let peers_lock = ctx.peers.lock().await;
            let peer_list: Vec<Value> = peers_lock
                .iter()
                .map(|(pid, key)| {
                    let mut m = Map::new();
                    m.insert("peerId".to_string(), pid.to_string().into());
                    m.insert("publicKey".to_string(), key.clone().into());
                    Value::Object(m)
                })
                .collect();
            let mut m = Map::new();
            m.insert("peers".to_string(), Value::Array(peer_list));
            rpc_result(id, Value::Object(m))
        }
        ClientIntent::GetTopology {} => {
            let peers_lock = ctx.peers.lock().await;
            let peer_count = peers_lock.len();
            let ledger = ctx.ledger.lock().await;
            let known_peers = ledger.all_known_topics();
            let mut m = Map::new();
            m.insert("peerCount".to_string(), peer_count.into());
            m.insert("knownPeers".to_string(), known_peers.len().into());
            m.insert(
                "bootstrapNodes".to_string(),
                ctx.bootstrap_nodes.clone().into(),
            );
            rpc_result(id, Value::Object(m))
        }
        // ── Contacts ──
        ClientIntent::GetContacts {} => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_store_manager();
                match mgr.list() {
                    Ok(contacts) => {
                        let list: Vec<Value> = contacts
                            .into_iter()
                            .map(|c| {
                                let mut m = Map::new();
                                m.insert("peerId".to_string(), c.peer_id.into());
                                m.insert("nickname".to_string(), c.nickname.into());
                                m.insert("localNickname".to_string(), c.local_nickname.into());
                                Value::Object(m)
                            })
                            .collect();
                        let mut m = Map::new();
                        m.insert("contacts".to_string(), Value::Array(list));
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch contacts: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::AddContact { peer_id, nickname } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_store_manager();
                let mut contact =
                    scmessenger_core::store::Contact::new(peer_id.clone(), String::new());
                if let Some(n) = nickname {
                    contact = contact.with_nickname(n);
                }
                match mgr.add(contact) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("added".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to add contact: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RemoveContact { peer_id } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_store_manager();
                match mgr.remove(peer_id) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("removed".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to remove contact: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Settings ──
        ClientIntent::GetSettings {} => {
            if let Some(ref core) = ctx.core {
                let info = core.get_identity_info();
                let mut m = Map::new();
                m.insert("nickname".to_string(), info.nickname.into());
                m.insert("identityId".to_string(), info.identity_id.into());
                m.insert("publicKeyHex".to_string(), info.public_key_hex.into());
                m.insert("libp2pPeerId".to_string(), info.libp2p_peer_id.into());
                m.insert("initialized".to_string(), info.initialized.into());
                rpc_result(id, Value::Object(m))
            } else {
                let mut m = Map::new();
                m.insert("identityId".to_string(), ctx.node_peer_id.clone().into());
                m.insert(
                    "publicKeyHex".to_string(),
                    ctx.node_public_key.clone().into(),
                );
                rpc_result(id, Value::Object(m))
            }
        }
        ClientIntent::UpdateSettings { key, value } => {
            if let Some(ref core) = ctx.core {
                match key.as_str() {
                    "nickname" => {
                        if let Some(nick) = value.as_str() {
                            match core.set_nickname(nick.to_string()) {
                                Ok(()) => {
                                    let mut m = Map::new();
                                    m.insert("updated".to_string(), true.into());
                                    rpc_result(id, Value::Object(m))
                                }
                                Err(e) => rpc_error(
                                    id,
                                    JsonRpcErrorBody {
                                        code: -32000,
                                        message: format!("Failed to set nickname: {:?}", e),
                                        data: None,
                                    },
                                ),
                            }
                        } else {
                            rpc_error(
                                id,
                                JsonRpcErrorBody {
                                    code: ERR_PARAMS,
                                    message: "value must be a string for nickname".to_string(),
                                    data: None,
                                },
                            )
                        }
                    }
                    _ => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: ERR_PARAMS,
                            message: format!("Unknown setting key: {}", key),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── History ──
        ClientIntent::GetHistory { limit } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.recent(None, limit.unwrap_or(50) as u32) {
                    Ok(messages) => {
                        let list: Vec<Value> = messages
                            .into_iter()
                            .map(|m| {
                                let mut map = Map::new();
                                map.insert("id".to_string(), m.id.into());
                                map.insert("senderId".to_string(), m.peer_id.into());
                                map.insert("content".to_string(), m.content.into());
                                map.insert("timestamp".to_string(), m.timestamp.into());
                                map.insert(
                                    "direction".to_string(),
                                    format!("{:?}", m.direction).into(),
                                );
                                Value::Object(map)
                            })
                            .collect();
                        let mut m = Map::new();
                        m.insert("messages".to_string(), Value::Array(list));
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch history: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::GetConversation { peer_id, limit } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.conversation(peer_id.clone(), limit.unwrap_or(50) as u32) {
                    Ok(messages) => {
                        let list: Vec<Value> = messages
                            .into_iter()
                            .map(|m| {
                                let mut map = Map::new();
                                map.insert("id".to_string(), m.id.into());
                                map.insert("senderId".to_string(), m.peer_id.into());
                                map.insert("content".to_string(), m.content.into());
                                map.insert("timestamp".to_string(), m.timestamp.into());
                                Value::Object(map)
                            })
                            .collect();
                        let mut m = Map::new();
                        m.insert("messages".to_string(), Value::Array(list));
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch conversation: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Blocking ──
        ClientIntent::BlockPeer { peer_id, reason } => {
            if let Some(ref core) = ctx.core {
                match core.block_peer(peer_id, None, reason) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("blocked".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to block peer: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::UnblockPeer { peer_id } => {
            if let Some(ref core) = ctx.core {
                match core.unblock_peer(peer_id, None) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("unblocked".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to unblock peer: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Onion routing ──
        ClientIntent::PrepareOnionMessage {
            envelope_data,
            relay_public_keys_json,
        } => {
            if let Some(ref core) = ctx.core {
                match hex::decode(&envelope_data) {
                    Ok(data) => match core.prepare_onion_message(data, relay_public_keys_json) {
                        Ok(onion_bytes) => {
                            let mut m = Map::new();
                            m.insert("onionData".to_string(), hex::encode(onion_bytes).into());
                            rpc_result(id, Value::Object(m))
                        }
                        Err(e) => rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: -32000,
                                message: format!("Onion prepare failed: {:?}", e),
                                data: None,
                            },
                        ),
                    },
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: ERR_PARAMS,
                            message: format!("Invalid hex for envelope_data: {}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::PeelOnionLayer {
            onion_data,
            relay_secret_key,
        } => {
            if let Some(ref core) = ctx.core {
                let onion_bytes = match hex::decode(&onion_data) {
                    Ok(b) => b,
                    Err(e) => {
                        return rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: ERR_PARAMS,
                                message: format!("Invalid hex for onion_data: {}", e),
                                data: None,
                            },
                        )
                    }
                };
                let secret_bytes = match hex::decode(&relay_secret_key) {
                    Ok(b) => b,
                    Err(e) => {
                        return rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: ERR_PARAMS,
                                message: format!("Invalid hex for relay_secret_key: {}", e),
                                data: None,
                            },
                        )
                    }
                };
                match core.peel_onion_layer(onion_bytes, secret_bytes) {
                    Ok(result) => {
                        let mut m = Map::new();
                        m.insert(
                            "nextHop".to_string(),
                            result.next_hop.map(hex::encode).into(),
                        );
                        m.insert(
                            "remainingData".to_string(),
                            hex::encode(result.remaining_data).into(),
                        );
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Onion peel failed: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Ratchet session diagnostics ──
        ClientIntent::RatchetSessionCount {} => {
            if let Some(ref core) = ctx.core {
                let count = core.ratchet_session_count();
                let mut m = Map::new();
                m.insert("sessionCount".to_string(), count.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RatchetHasSession { peer_id } => {
            if let Some(ref core) = ctx.core {
                let has = core.ratchet_has_session(peer_id);
                let mut m = Map::new();
                m.insert("hasSession".to_string(), has.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── DSPy integrity ──
        ClientIntent::Blake3Hash { data_hex } => {
            if let Some(ref core) = ctx.core {
                match hex::decode(&data_hex) {
                    Ok(data) => {
                        let hash = core.dspy_blake3_hash(&data);
                        let mut m = Map::new();
                        m.insert("hash".to_string(), hex::encode(hash).into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: ERR_PARAMS,
                            message: format!("Invalid hex for data_hex: {}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Routing prefetch ──
        ClientIntent::RoutingIsPrefetchComplete {} => {
            if let Some(ref core) = ctx.core {
                let complete = core.routing_is_prefetch_complete();
                let mut m = Map::new();
                m.insert("isPrefetchComplete".to_string(), complete.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RoutingIsPrefetchInProgress {} => {
            if let Some(ref core) = ctx.core {
                let in_progress = core.routing_is_prefetch_in_progress();
                let mut m = Map::new();
                m.insert("isPrefetchInProgress".to_string(), in_progress.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RoutingMarkRefreshFailed { hint } => {
            if let Some(ref core) = ctx.core {
                let hint_bytes: [u8; 8] = hint
                    .and_then(|h| hex::decode(h).ok())
                    .and_then(|v| {
                        if v.len() == 8 {
                            let mut arr = [0u8; 8];
                            arr.copy_from_slice(&v);
                            Some(arr)
                        } else {
                            None
                        }
                    })
                    .unwrap_or([0; 8]);
                core.routing_mark_refresh_failed(hint_bytes);
                let mut m = Map::new();
                m.insert("marked".to_string(), true.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RoutingNextRefreshHint {} => {
            if let Some(ref core) = ctx.core {
                let hint = core.routing_next_refresh_hint();
                let hint_hex = hint.map(hex::encode).unwrap_or_default();
                let mut m = Map::new();
                m.insert("hint".to_string(), hint_hex.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RoutingStartRefresh { hint } => {
            if let Some(ref core) = ctx.core {
                let hint_bytes: [u8; 8] = hex::decode(&hint)
                    .ok()
                    .and_then(|v| {
                        if v.len() == 8 {
                            let mut arr = [0u8; 8];
                            arr.copy_from_slice(&v);
                            Some(arr)
                        } else {
                            None
                        }
                    })
                    .unwrap_or([0; 8]);
                core.routing_start_refresh(hint_bytes);
                let mut m = Map::new();
                m.insert("started".to_string(), true.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── DSPy modules ──
        ClientIntent::ClearHistory {} => {
            if let Some(ref core) = ctx.core {
                match core.clear_history() {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("cleared".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to clear history: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::ListBlocked {} => {
            if let Some(ref core) = ctx.core {
                match core.list_blocked() {
                    Ok(blocked) => {
                        let list: Vec<Value> = blocked
                            .into_iter()
                            .map(|b| {
                                let mut map = Map::new();
                                map.insert("peerId".to_string(), b.peer_id.into());
                                map.insert("reason".to_string(), b.reason.into());
                                map.insert("blockedAt".to_string(), b.blocked_at.into());
                                Value::Object(map)
                            })
                            .collect();
                        let mut m = Map::new();
                        m.insert("blocked".to_string(), Value::Array(list));
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to list blocked: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Identity backup (P0) ──
        ClientIntent::ExportIdentityBackup { passphrase } => {
            if let Some(ref core) = ctx.core {
                match core.export_identity_backup(passphrase) {
                    Ok(backup) => {
                        let mut m = Map::new();
                        m.insert("backup".to_string(), backup.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to export identity backup: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::ImportIdentityBackup { backup, passphrase } => {
            if let Some(ref core) = ctx.core {
                match core.import_identity_backup(backup, passphrase) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("imported".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to import identity backup: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Nickname (P0) ──
        ClientIntent::SetNickname { nickname } => {
            if let Some(ref core) = ctx.core {
                match core.set_nickname(nickname) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("updated".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to set nickname: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Audit log (P0) ──
        ClientIntent::GetAuditLog {} => {
            if let Some(ref core) = ctx.core {
                let events = core.get_audit_log();
                let list: Vec<Value> = events
                    .into_iter()
                    .map(|e| serde_json::to_value(e).unwrap_or_else(|_| Value::Object(Map::new())))
                    .collect();
                let mut m = Map::new();
                m.insert("events".to_string(), Value::Array(list));
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::GetAuditEventsSince { since_timestamp } => {
            if let Some(ref core) = ctx.core {
                let all_events = core.get_audit_log();
                let since = since_timestamp.unwrap_or(0);
                let filtered: Vec<serde_json::Value> = all_events
                    .into_iter()
                    .filter(|e| {
                        serde_json::to_value(e)
                            .ok()
                            .and_then(|v| v.get("timestamp").and_then(|t| t.as_u64()))
                            .unwrap_or(0)
                            >= since
                    })
                    .map(|e| serde_json::to_value(e).unwrap_or_else(|_| Value::Object(Map::new())))
                    .collect();
                let mut m = Map::new();
                m.insert("events".to_string(), Value::Array(filtered));
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Privacy config (P0) ──
        ClientIntent::SetPrivacyConfig { config_json } => {
            if let Some(ref core) = ctx.core {
                match core.set_privacy_config(config_json) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("updated".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to set privacy config: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::GetPrivacyConfig {} => {
            if let Some(ref core) = ctx.core {
                let config = core.get_privacy_config();
                let mut m = Map::new();
                m.insert("config".to_string(), config.into());
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Message requests (P0) ──
        // A message request is an inbox message from a peer who isn't a
        // contact (and isn't blocked) yet - mirrors Android's
        // MeshRepository.getPendingMessageRequests()/RequestsInboxViewModel,
        // but peeks the inbox instead of draining it so polling twice
        // doesn't make requests vanish before the user acts on them.
        ClientIntent::GetPendingMessageRequests {} => {
            if let Some(ref core) = ctx.core {
                let contacts = core.contacts_store_manager().list().unwrap_or_default();
                // Hold BOTH identifier flavors for every contact: a contact may
                // be stored keyed by identity_id while an inbound message's
                // sender_id now carries the canonical public key.
                let mut contact_peer_ids: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for c in contacts {
                    contact_peer_ids.insert(c.peer_id);
                    contact_peer_ids.insert(c.public_key);
                }
                let inbox_messages = core.peek_received_messages();
                let mut by_sender: HashMap<String, Vec<&scmessenger_core::store::ReceivedMessage>> =
                    HashMap::new();
                for msg in &inbox_messages {
                    // T3/P4 (identifier-gate follow-up): FAIL CLOSED on an
                    // unresolvable sender identifier. A message whose sender
                    // cannot be resolved to a real Ed25519 public key -- whether
                    // via the authenticated envelope key (sender_public_key_hex,
                    // the canonical source) or a valid key-valued sender_id --
                    // cannot be proven known-or-blocked, so showing it as a clean
                    // request would be a fail-open listing. Suppress it instead;
                    // the authoritative drop/hide already happens at core ingress.
                    let resolvable_public_key =
                        resolve_sender_public_key(&msg.sender_public_key_hex, &msg.sender_id);
                    let Some(request_key) =
                        message_request_key(&msg.sender_public_key_hex, &msg.sender_id)
                    else {
                        continue;
                    };
                    let is_known = contact_peer_ids.contains(&request_key)
                        || resolvable_public_key
                            .as_ref()
                            .is_some_and(|key| contact_peer_ids.contains(key));
                    // T4: resolve block status through the core manager's
                    // single dual-flavor policy rather than maintaining a
                    // second CLI-side expansion of public key and identity_id.
                    // A lookup error fails closed so a blocked sender is not
                    // exposed as a pending request.
                    let is_blocked = core
                        .is_peer_blocked(request_key.clone(), None)
                        .unwrap_or(true);
                    // Fail closed: if we cannot resolve the sender to a valid
                    // Ed25519 public key, we cannot say it is unknown-but-valid
                    // (a legitimate new-requester case); suppress it.
                    if !is_known && !is_blocked {
                        by_sender.entry(request_key).or_default().push(msg);
                    }
                }

                let mut requests: Vec<(u64, Value)> = by_sender
                    .into_iter()
                    .map(|(peer_id, msgs)| {
                        let latest = msgs
                            .iter()
                            .max_by_key(|m| m.received_at)
                            .expect("group has at least one message");
                        let preview = String::from_utf8(latest.payload.clone())
                            .unwrap_or_else(|_| "[binary]".to_string());
                        let mut m = Map::new();
                        m.insert("peerId".to_string(), peer_id.into());
                        m.insert("nickname".to_string(), Value::Null);
                        m.insert("messagePreview".to_string(), preview.into());
                        m.insert("messageTimestamp".to_string(), latest.received_at.into());
                        m.insert("messageCount".to_string(), msgs.len().into());
                        (latest.received_at, Value::Object(m))
                    })
                    .collect();
                requests.sort_by_key(|b| std::cmp::Reverse(b.0));

                let mut m = Map::new();
                m.insert(
                    "requests".to_string(),
                    Value::Array(requests.into_iter().map(|(_, v)| v).collect()),
                );
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        /// Accept = add the sender as a contact, using the Ed25519 public key
        // captured from their message envelope at receive time (cryptographically
        // verified there) rather than an unauthenticated discovery broadcast.
        ClientIntent::AcceptMessageRequest { request_id } => {
            if let Some(ref core) = ctx.core {
                // The request_id from the pending list IS the canonical request key
                // (output of message_request_key). Use it directly to match stored
                // messages without re-canonicalizing.
                let canonical_req_id = request_id.clone();
                let public_key_hex = core
                    .peek_received_messages()
                    .into_iter()
                    .filter(|m| {
                        message_request_key(&m.sender_public_key_hex, &m.sender_id).as_deref()
                            == Some(&canonical_req_id)
                    })
                    .filter_map(|m| {
                        resolve_sender_public_key(&m.sender_public_key_hex, &m.sender_id)
                    })
                    .next_back();

                match public_key_hex {
                    Some(public_key) => {
                        // UNIFICATION V2 P0: the canonical identifier is
                        // public_key_hex. ContactsManager::add() canonicalizes
                        // peer_id to the pubkey anyway; store it directly so
                        // the contact is consistent with the canonical form.
                        let contact =
                            scmessenger_core::store::Contact::new(public_key.clone(), public_key);
                        match core.contacts_store_manager().add(contact) {
                            Ok(()) => {
                                let mut m = Map::new();
                                m.insert("accepted".to_string(), true.into());
                                rpc_result(id, Value::Object(m))
                            }
                            Err(e) => rpc_error(
                                id,
                                JsonRpcErrorBody {
                                    code: -32000,
                                    message: format!("Failed to accept message request: {:?}", e),
                                    data: None,
                                },
                            ),
                        }
                    }
                    None => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32002,
                            message: format!(
                                "No pending message request found from {}",
                                request_id
                            ),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // Reject = block the sender (matches Android's
        // RequestsInboxViewModel.rejectRequest); list_blocked_peers filtering
        // in GetPendingMessageRequests keeps them from reappearing.
        ClientIntent::RejectMessageRequest { request_id } => {
            if let Some(ref core) = ctx.core {
                // The request_id is the canonical key (identity_id). Find the
                // corresponding message to get its authenticated public key, so
                // we block by public key and the dual-flavor write also covers
                // the identity_id alias.
                let canonical_req_id = request_id.clone();
                let public_key_hex = core
                    .peek_received_messages()
                    .into_iter()
                    .filter(|m| {
                        message_request_key(&m.sender_public_key_hex, &m.sender_id).as_deref()
                            == Some(&canonical_req_id)
                    })
                    .filter_map(|m| m.sender_public_key_hex)
                    .next_back();

                let block_identifier = public_key_hex
                    .map(|public_key| format!("pk:{public_key}"))
                    .unwrap_or_else(|| format!("id:{canonical_req_id}"));

                match core.block_peer(
                    block_identifier,
                    None,
                    Some("message_request_rejected".to_string()),
                ) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("rejected".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to reject message request: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── History search/stats (P1) ──
        ClientIntent::SearchMessages { query, limit } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.search(query, limit.unwrap_or(50) as u32) {
                    Ok(messages) => {
                        let list: Vec<Value> = messages
                            .into_iter()
                            .map(|m| {
                                let mut map = Map::new();
                                map.insert("id".to_string(), m.id.into());
                                map.insert("senderId".to_string(), m.peer_id.into());
                                map.insert("content".to_string(), m.content.into());
                                map.insert("timestamp".to_string(), m.timestamp.into());
                                map.insert(
                                    "direction".to_string(),
                                    format!("{:?}", m.direction).into(),
                                );
                                Value::Object(map)
                            })
                            .collect();
                        let mut m = Map::new();
                        m.insert("messages".to_string(), Value::Array(list));
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to search messages: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::GetHistoryStats {} => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.stats() {
                    Ok(stats) => {
                        let mut m = Map::new();
                        m.insert("totalMessages".to_string(), stats.total_messages.into());
                        m.insert("sentCount".to_string(), stats.sent_count.into());
                        m.insert("receivedCount".to_string(), stats.received_count.into());
                        m.insert(
                            "undeliveredCount".to_string(),
                            stats.undelivered_count.into(),
                        );
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to get history stats: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::MarkMessageDelivered { message_id } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.mark_delivered(message_id) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("delivered".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to mark message delivered: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::DeleteMessage { message_id } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.delete(message_id) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("deleted".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to delete message: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::ClearConversation { peer_id } => {
            if let Some(ref core) = ctx.core {
                let mgr = core
                    .history_manager()
                    .expect("Failed to initialize history manager");
                match mgr.clear_conversation(peer_id) {
                    Ok(()) => {
                        let mut m = Map::new();
                        m.insert("cleared".to_string(), true.into());
                        rpc_result(id, Value::Object(m))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to clear conversation: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Diagnostics (P1) ──
        // Note: get_diagnostics is not yet implemented in the core crate.
        // It returns aggregated diagnostic info from core + transport state.
        ClientIntent::GetDiagnostics {} => {
            if let Some(ref core) = ctx.core {
                let info = core.get_identity_info();
                let peer_count = ctx.peers.lock().await.len();
                let ledger = ctx.ledger.lock().await;
                let mut m = Map::new();
                m.insert("identityId".to_string(), info.identity_id.into());
                m.insert("publicKeyHex".to_string(), info.public_key_hex.into());
                m.insert("initialized".to_string(), info.initialized.into());
                m.insert("peerCount".to_string(), peer_count.into());
                m.insert(
                    "uptimeSecs".to_string(),
                    ctx.start_time.elapsed().as_secs().into(),
                );
                m.insert(
                    "knownPeers".to_string(),
                    ledger.all_known_topics().len().into(),
                );
                m.insert(
                    "bootstrapNodes".to_string(),
                    ctx.bootstrap_nodes.clone().into(),
                );
                rpc_result(id, Value::Object(m))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Self-test (P2) ──
        // Note: run_self_test is not yet implemented in the core crate.
        ClientIntent::RunSelfTest {} => rpc_error(
            id,
            JsonRpcErrorBody {
                code: -32001,
                message: "run_self_test not yet implemented".to_string(),
                data: None,
            },
        ),
    }
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::{message_request_key, resolve_sender_public_key, sender_is_resolvable};

    /// A valid Ed25519 public key (generated; 64 hex chars).
    fn valid_key() -> String {
        // Deterministic: a real generated key is always a valid curve point.
        let keys = scmessenger_core::identity::IdentityKeys::generate();
        keys.public_key_hex()
    }

    #[test]
    fn resolvable_from_authenticated_envelope_key() {
        let key = valid_key();
        assert!(sender_is_resolvable(
            &Some(key.clone()),
            "garbage-sender-id"
        ));
        assert_eq!(
            resolve_sender_public_key(&Some(key.clone()), "garbage-sender-id"),
            Some(key)
        );
    }

    #[test]
    fn resolvable_from_valid_key_valued_sender_id() {
        let key = valid_key();
        // No authenticated envelope key (legacy pre-field message) but the
        // sender_id itself is a valid public key -> resolvable.
        assert!(sender_is_resolvable(&None, &key));
        assert_eq!(resolve_sender_public_key(&None, &key), Some(key));
    }

    #[test]
    fn unresolvable_when_neither_flavor_is_a_valid_curve_point() {
        // 0x7f*32 is 64 hex chars but NOT a valid Ed25519 curve point (probed
        // against ed25519-dalek), and the authenticated envelope key is absent.
        let bogus = "7f".repeat(32);
        assert!(!sender_is_resolvable(&None, &bogus));
        assert_eq!(resolve_sender_public_key(&None, &bogus), None);
        // A short/garbage sender_id is likewise unresolvable.
        assert!(!sender_is_resolvable(&None, "not-a-key"));
        assert_eq!(resolve_sender_public_key(&None, "not-a-key"), None);
    }

    #[test]
    fn invalid_envelope_key_does_not_win_over_valid_sender_id() {
        // An envelope key that is not a valid curve point must not shadow a
        // valid key-valued sender_id: resolvability falls through to the
        // sender_id flavor.
        let key = valid_key();
        assert!(sender_is_resolvable(&Some("7f".repeat(32)), &key));
        assert_eq!(
            resolve_sender_public_key(&Some("7f".repeat(32)), &key),
            Some(key)
        );
    }

    #[test]
    fn request_key_prefers_authenticated_key_and_falls_back_to_legacy_identity_id() {
        let key = valid_key();
        // A legacy identity ID has no authenticated envelope key. Use an
        // off-curve value here so the test exercises the identity-ID fallback
        // rather than the separate legacy-public-key compatibility path.
        let identity_id = "7f".repeat(32);

        assert_eq!(
            message_request_key(&Some(key.clone()), "legacy-sender-id"),
            scmessenger_core::identity::keys::identity_id_from_public_key_hex(&key)
        );
        assert_eq!(
            message_request_key(&None, &key),
            scmessenger_core::identity::keys::identity_id_from_public_key_hex(&key)
        );
        assert_eq!(message_request_key(&None, &identity_id), Some(identity_id));
        assert_eq!(message_request_key(&None, "not-a-key"), None);
    }
}
