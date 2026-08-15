# Phase 1 Task 1.4 Complete: Hyper 0.14 → Axum 0.7 Migration

**Date:** 2026-05-07  
**Task:** Migrate HTTP API from Hyper 0.14 to Axum 0.7  
**Status:** [OK] COMPLETE  
**Lines Changed:** ~400 LoC

## Changes Made

### 1. Updated Dependencies (cli/Cargo.toml)

**Removed:**
- `hyper = { version = "0.14", features = ["full"] }`

**Added:**
- `axum = "0.7"`
- `tower = "0.4"`
- `tower-http = { version = "0.5", features = ["cors", "trace"] }`
- `hyper = { version = "1", features = ["client", "http1"] }` (for client functions)
- `hyper-util = { version = "0.1", features = ["client-legacy", "http1", "tokio"] }`

### 2. Updated Client Functions (cli/src/api.rs)

All API client functions migrated from Hyper 0.14 to Hyper 1.x:
- `send_message_via_api()`
- `add_contact_via_api()`
- `get_peers_via_api()`
- `get_history_via_api()`
- `get_external_address_via_api()`
- `get_listeners_via_api()`
- `get_connection_path_state_via_api()`
- `get_drift_state_via_api()`
- `get_discovery_status()`
- `trigger_discovery_scan()`
- `get_discovery_peers()`
- `export_diagnostics_via_api()`
- `stop_node_via_api()`

**Pattern Change:**
```rust
// OLD (Hyper 0.14)
let client = hyper::Client::new();
let req = Request::builder()
    .body(Body::from(json))?;
let resp = client.request(req).await?;
let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;

// NEW (Hyper 1.x)
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

let client = Client::builder(TokioExecutor::new()).build_http();
let req = hyper::Request::builder()
    .body(Full::new(Bytes::from(json)))?;
let resp = client.request(req).await?;
let body_bytes = resp.into_body().collect().await?.to_bytes();
```

### 3. Migrated Server Implementation (cli/src/api.rs)

**ApiContext:**
- Added `#[derive(Clone)]` to support Axum's state management

**Handler Functions - Converted to Axum Pattern:**

All handlers migrated from Hyper service_fn to Axum extractors:

```rust
// OLD (Hyper 0.14)
async fn handle_send_message(req: Request<Body>, ctx: Arc<ApiContext>) -> Result<Response<Body>> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: SendMessageRequest = serde_json::from_slice(&body_bytes)?;
    // ... logic ...
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&response)?))?)
}

// NEW (Axum 0.7)
async fn handle_send_message(
    State(ctx): State<Arc<ApiContext>>,
    AxumJson(request): AxumJson<SendMessageRequest>,
) -> Result<AxumJson<SendMessageResponse>, (StatusCode, String)> {
    // ... logic ...
    Ok(AxumJson(SendMessageResponse {
        success: true,
        error: None,
    }))
}
```

**Migrated Handlers:**
- `handle_send_message` - POST /api/send
- `handle_add_contact` - POST /api/contacts
- `handle_get_peers` - GET /api/peers
- `handle_get_listeners` - GET /api/listeners
- `handle_get_history` - POST /api/history
- `handle_get_external_address` - GET /api/external-address
- `handle_get_connection_path_state` - GET /api/connection-path-state
- `handle_export_diagnostics` - GET /api/diagnostics
- `handle_get_drift_status` - GET /api/drift-status
- `handle_get_discovery_status` - GET /api/discovery/status
- `handle_trigger_discovery_scan` - POST /api/discovery/scan
- `handle_get_discovery_peers` - GET /api/discovery/peers
- `handle_shutdown` - POST /api/shutdown

**Server Function:**

```rust
// OLD (Hyper 0.14)
pub async fn start_api_server(ctx: ApiContext) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    let make_svc = make_service_fn(move |_conn| {
        let ctx = ctx.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, ctx.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    server.await.context("API server error")?;
    Ok(())
}

// NEW (Axum 0.7)
pub async fn start_api_server(ctx: ApiContext) -> Result<()> {
    let ctx = Arc::new(ctx);
    let addr = SocketAddr::from(([127, 0, 0, 1], API_PORT));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/send", post(handle_send_message))
        .route("/api/contacts", post(handle_add_contact))
        .route("/api/peers", get(handle_get_peers))
        .route("/api/listeners", get(handle_get_listeners))
        .route("/api/history", post(handle_get_history))
        .route("/api/external-address", get(handle_get_external_address))
        .route("/api/connection-path-state", get(handle_get_connection_path_state))
        .route("/api/diagnostics", get(handle_export_diagnostics))
        .route("/api/drift-status", get(handle_get_drift_status))
        .route("/api/discovery/status", get(handle_get_discovery_status))
        .route("/api/discovery/scan", post(handle_trigger_discovery_scan))
        .route("/api/discovery/peers", get(handle_get_discovery_peers))
        .route("/api/shutdown", post(handle_shutdown))
        .layer(cors)
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind API server")?;

    tracing::info!("Control API listening on {}", addr);

    axum::serve(listener, app)
        .await
        .context("API server error")?;

    Ok(())
}
```

## Verification

### Compilation Check
```bash
cargo check -p scmessenger-cli
```
**Result:** [OK] PASSED

### Dependency Tree Check
```bash
cargo tree -p scmessenger-cli | grep hyper
```
**Result:**
- [OK] Hyper 1.9.0 present (used by Axum and our client code)
- [OK] Hyper 0.14.32 only present as transitive dependency through `igd-next` → `libp2p-upnp`
- [OK] No direct dependency on Hyper 0.14 from CLI crate

## Benefits of Axum 0.7

1. **Type-Safe Routing:** Compile-time route validation
2. **Extractors:** Clean separation of concerns (State, Json, Path, etc.)
3. **Better Error Handling:** Structured error responses with (StatusCode, String) tuples
4. **Modern Async:** Built on Hyper 1.x and Tower ecosystem
5. **CORS Support:** Integrated via tower-http middleware
6. **Cleaner Code:** Less boilerplate than Hyper 0.14 service_fn pattern

## API Endpoints Preserved

All 13 original API endpoints maintained:
- [OK] POST /api/send
- [OK] POST /api/contacts
- [OK] GET /api/peers
- [OK] GET /api/listeners
- [OK] POST /api/history
- [OK] GET /api/external-address
- [OK] GET /api/connection-path-state
- [OK] GET /api/diagnostics
- [OK] GET /api/drift-status
- [OK] GET /api/discovery/status
- [OK] POST /api/discovery/scan
- [OK] GET /api/discovery/peers
- [OK] POST /api/shutdown

## Next Steps

**Task 1.5:** Verify Hyper 0.14 Removal (verify transitive dependencies acceptable)  
**Task 1.6:** Phase 1 Verification Gate (full test suite)

## Notes

- Hyper 0.14 remains in transitive dependencies through libp2p-upnp → igd-next
- This is acceptable as it doesn't affect our API server code
- All direct CLI dependencies now use Hyper 1.x or Axum 0.7
- CORS is now properly configured with tower-http
- Error handling improved with structured responses

**Task 1.4 Status: [OK] COMPLETE**
