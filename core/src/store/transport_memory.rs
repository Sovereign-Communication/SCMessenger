use crate::store::backend::StorageBackend;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMemoryEntry {
    pub transport: String,
    pub port: u16,
    pub last_success_unix: u64,
    pub ladder_rank: u32,
}

pub struct TransportMemoryStore {
    backend: Arc<dyn StorageBackend>,
}

impl TransportMemoryStore {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    fn key(peer_id: &PeerId, network_fingerprint: &str) -> String {
        format!("tmem:{}:{}", peer_id, network_fingerprint)
    }

    pub fn record_success(
        &self,
        peer_id: &PeerId,
        network_fingerprint: &str,
        transport: String,
        port: u16,
        ladder_rank: u32,
    ) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = TransportMemoryEntry {
            transport,
            port,
            last_success_unix: now,
            ladder_rank,
        };
        let bytes = serde_json::to_vec(&entry).map_err(|e| e.to_string())?;
        self.backend
            .put(Self::key(peer_id, network_fingerprint).as_bytes(), &bytes)?;
        Ok(())
    }

    pub fn get_last_good(
        &self,
        peer_id: &PeerId,
        network_fingerprint: &str,
    ) -> Result<Option<TransportMemoryEntry>, String> {
        if let Some(bytes) = self
            .backend
            .get(Self::key(peer_id, network_fingerprint).as_bytes())?
        {
            let entry: TransportMemoryEntry =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}

/// Network fingerprint used to scope `TransportMemoryStore` keys per physical network.
///
/// TRANSPORT_UNIFICATION: Ideally this is `hash(WiFi BSSID + subnet /24)` (or for
/// wired: `hash(MAC + subnet /24)`) so a port that worked on a coffee-shop WiFi
/// does not bias dialing on a home LAN where the same port may be firewalled.
/// On Android this requires `Context` + `WifiManager`/`ConnectivityManager` and
/// runtime permissions (`ACCESS_FINE_LOCATION` / `NEARBY_WIFI_DEVICES`), which
/// are not available inside the Rust core without a platform bridge. On native
/// desktop the BSSID is similarly not available without OS-specific APIs.
///
/// Until a platform bridge calls `IronCore::set_network_fingerprint()` (or this
/// function is wired to a callback that queries the Android `MeshRepository`),
/// this intentionally returns a global placeholder. **Callers must handle the
/// placeholder correctly**: treat a missing `get_last_good` as "no prior
/// preference" and fall back to the full port-probing ladder (443, 80, 8080,
/// relay circuit). The placeholder is still a valid `StorageBackend` key, so
/// `record_success`/`get_last_good` remain correct — they are simply scoped
/// per-device rather than per-network, which is safe (may be slightly less
/// optimal) and never causes a dial failure.
///
/// When the bridge is available, replace the body with:
/// ```ignore
/// // hash(BSSID + "/" + subnet) truncated to 16 hex chars
/// format!("{:x}", hash(bssid + subnet))
/// ```
/// and callers require no change — they already key by whatever string is
/// returned here.
pub fn get_network_fingerprint() -> String {
    // PLACEHOLDER — see doc comment above for why this is not yet wired and
    // how callers must handle it. Do NOT change callers to assume per-network
    // scoping until this returns a real fingerprint; the fallback ladder must
    // remain the primary dial strategy when the key is global.
    "placeholder_network_fingerprint".to_string()
}

/// Returns `true` if `fp` is the placeholder value (i.e. not yet per-network scoped).
/// Callers may use this to decide whether to log at `debug!` that port memory is global.
pub fn is_placeholder_fingerprint(fp: &str) -> bool {
    fp == "placeholder_network_fingerprint"
}
