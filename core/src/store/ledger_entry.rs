use crate::transport::addr_filter::{
    is_dialable_multiaddr, is_disclosable_multiaddr, is_recordable_multiaddr, DnsPolicy,
    NetworkMode,
};
use libp2p::Multiaddr;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// CONNECTION LEDGER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub multiaddr: String,
    pub peer_id: Option<String>,
    pub public_key: Option<String>,
    pub nickname: Option<String>,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_seen: Option<u64>,
    pub topics: Vec<String>,
}

/// Maximum number of [`SeedLedgerEntry`] records an invite may carry, and the
/// hard cap [`LedgerManager::import_seed_entries`] enforces on any caller.
///
/// Sized against the QR byte-mode budget: see
/// `crate::relay::invite::QR_BYTE_BUDGET` and the
/// `seed_ledger_full_invite_fits_qr_budget` test in `relay/invite.rs`.
pub const MAX_SEED_LEDGER_ENTRIES: usize = 16;

/// Maximum number of [`LedgerEntry`] records retained in the in-memory ledger.
/// New-insert paths evict the least-useful entry before exceeding this cap.
const MAX_LEDGER_ENTRIES: usize = 1024;

const MAX_LEN_MULTIADDR: usize = 512;
const MAX_LEN_PEER_ID: usize = 128;
const MAX_LEN_PUBLIC_KEY: usize = 512;
const MAX_LEN_NICKNAME: usize = 128;

/// Failure threshold aligned with `core/src/transport/dial_policy.rs` dead-mark.
const LEDGER_DEAD_FAILURE_THRESHOLD: u32 = 3;

/// Monotonic nonce for unique temporary ledger files in `save_with_entries`.
static SAVE_TMP_NONCE: AtomicU64 = AtomicU64::new(0);

/// A routing-only peer record carried inside an invite (item 1 of the v0.4.0
/// ledger seeding work).
///
/// ROUTING ONLY -- NO IDENTITY (operator directive 2026-07-25). This type has
/// exactly one field and must keep exactly one field. `peer_id`, `public_key`,
/// `nickname`, `topics`, `success_count`, `failure_count` and `last_seen` are
/// all deliberately absent: every one of them is identity or behavioural
/// metadata about a third party who never consented to being listed in someone
/// else's invite. An invite says *where to knock*, not *who lives there*.
///
/// The invitee dials the bare address, completes the Noise handshake and learns
/// the peer identity from Identify at connect time, then attaches it locally via
/// [`LedgerManager::annotate_identity`]. Dropping `peer_id` forgoes dial-time
/// identity pinning, which is an availability property; message confidentiality
/// is per-contact X25519 / XChaCha20-Poly1305 established out of band and is
/// unaffected by which node answers at a given address.
///
/// `relay/invite.rs` has a leak-regression test that asserts no peer id, public
/// key or nickname appears in the serialised invite bytes. If you add a field
/// here, that test is what will stop you.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(uniffi::Record))]
pub struct SeedLedgerEntry {
    /// Peer-id-stripped dialable multiaddr, e.g. `/ip4/A.B.C.D/tcp/9001`.
    pub multiaddr: String,
}

/// Strip the endpoint `/p2p/<peer-id>` component(s) from a multiaddr string.
///
/// Matches the CLI ledger's key convention (`cli/src/ledger.rs::strip_peer_id`,
/// which now delegates to the same implementation) so the two ledgers dedupe on
/// identical keys.
///
/// This used to be `multiaddr.find("/p2p/")` + truncate, which collapsed
/// `/ip4/A/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` to the RELAY's bare
/// address while [`ledger_entry_to_shared`] kept `last_peer_id = QmTarget` --
/// a wire record asserting "QmTarget is directly reachable at the relay's IP"
/// that recipients feed into `kademlia.add_address()`. See review F8 and
/// [`crate::transport::addr_filter::strip_peer_id_multiaddr`].
fn strip_peer_id_component(multiaddr: &str) -> String {
    crate::transport::addr_filter::strip_peer_id(multiaddr)
}

fn is_dns_multiaddr(addr_str: &str) -> bool {
    addr_str.contains("/dns/")
        || addr_str.contains("/dns4/")
        || addr_str.contains("/dns6/")
        || addr_str.contains("/dnsaddr/")
}

fn get_multiaddr_port(addr_str: &str) -> Option<u16> {
    if let Ok(addr) = addr_str.parse::<Multiaddr>() {
        for proto in addr.iter() {
            match proto {
                libp2p::multiaddr::Protocol::Tcp(port) => return Some(port),
                libp2p::multiaddr::Protocol::Udp(port) => return Some(port),
                _ => {}
            }
        }
    }
    None
}

fn evict_one_locked(entries: &mut Vec<LedgerEntry>) {
    if entries.len() < MAX_LEDGER_ENTRIES {
        return;
    }

    let victim = if let Some((idx, _)) = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.success_count == 0)
        .min_by_key(|(_, entry)| (entry.last_seen.is_some(), entry.last_seen.unwrap_or(0)))
    {
        idx
    } else if let Some((idx, _)) = entries
        .iter()
        .enumerate()
        .min_by_key(|(_, entry)| (entry.last_seen.is_some(), entry.last_seen.unwrap_or(0)))
    {
        idx
    } else {
        0
    };

    if victim < entries.len() {
        entries.remove(victim);
    }
}

fn annotate_identity_locked(
    entries: &mut Vec<LedgerEntry>,
    multiaddr: String,
    peer_id: String,
    public_key: Option<String>,
    nickname: Option<String>,
) -> bool {
    let normalized_public_key = public_key.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let normalized_nickname = nickname.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    if multiaddr.len() > MAX_LEN_MULTIADDR
        || (!peer_id.is_empty()
            && (peer_id.len() > MAX_LEN_PEER_ID
                || peer_id.parse::<libp2p::PeerId>().is_err()))
        || normalized_public_key
            .as_ref()
            .map_or(false, |value| value.len() > MAX_LEN_PUBLIC_KEY)
        || normalized_nickname
            .as_ref()
            .map_or(false, |value| value.len() > MAX_LEN_NICKNAME)
    {
        return false;
    }

    let target_port = get_multiaddr_port(&multiaddr);
    let mut found_dns_idx = None;
    for (idx, entry) in entries.iter().enumerate() {
        if entry.peer_id.as_deref() == Some(&peer_id)
            && is_dns_multiaddr(&entry.multiaddr)
            && (target_port.is_none() || get_multiaddr_port(&entry.multiaddr) == target_port)
        {
            found_dns_idx = Some(idx);
            break;
        }
    }

    if let Some(idx) = found_dns_idx {
        let entry = &mut entries[idx];
        if normalized_public_key.is_some() {
            entry.public_key = normalized_public_key;
        }
        if normalized_nickname.is_some() {
            entry.nickname = normalized_nickname;
        }
        entry.last_seen = Some(current_timestamp());
        false
    } else if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
        entry.peer_id = Some(peer_id);
        if normalized_public_key.is_some() {
            entry.public_key = normalized_public_key;
        }
        if normalized_nickname.is_some() {
            entry.nickname = normalized_nickname;
        }
        entry.last_seen = Some(current_timestamp());
        false
    } else {
        while entries.len() >= MAX_LEDGER_ENTRIES {
            evict_one_locked(entries);
        }
        entries.push(LedgerEntry {
            multiaddr,
            peer_id: Some(peer_id),
            public_key: normalized_public_key,
            nickname: normalized_nickname,
            success_count: 0,
            failure_count: 0,
            last_seen: Some(current_timestamp()),
            topics: Vec::new(),
        });
        true
    }
}

/// Who an invite's `seed_ledger` is going to.
///
/// Re-review NEW-7: an invite QR is a durable, forwardable artefact, so its
/// default audience has to be "a stranger, eventually". Only a caller that
/// knows the invite is handed over in person may widen it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SeedExportAudience {
    /// Assume the invite will be forwarded, photographed or posted. Only
    /// globally routable, non-DNS addresses survive.
    #[default]
    Untrusted,
    /// The invite is being shown to someone physically present on the same
    /// network, so an RFC1918 address is the entire point. Loopback,
    /// link-local, multicast, broadcast, `0/8`, `192.0.0.0/24` and DNS forms
    /// are STILL dropped -- this widens exactly one rule, not the gate.
    LocalMesh,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(uniffi::Object))]
pub struct LedgerManager {
    /// `None` means "hold the ledger in memory only, never touch the disk".
    ///
    /// Added for review F11: `IronCore::new()` (the storage-less constructor)
    /// used to point its `LedgerManager` at `std::env::temp_dir()`, writing the
    /// node's whole peer topology into a world-readable directory on desktop.
    /// An in-memory core must have an in-memory ledger.
    storage_path: Option<std::path::PathBuf>,
    entries: Arc<Mutex<Vec<LedgerEntry>>>,
    /// Serializes durable snapshots so concurrent mutators cannot write out of
    /// snapshot order. Held from before the entries mutation until after rename.
    save_lock: Mutex<()>,
}

#[cfg_attr(not(target_arch = "wasm32"), uniffi::export)]
impl LedgerManager {
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path: Some(std::path::PathBuf::from(storage_path)),
            entries: Arc::new(Mutex::new(Vec::new())),
            save_lock: Mutex::new(()),
        }
    }

    pub fn load(&self) -> Result<(), crate::IronCoreError> {
        let Some(storage_path) = self.storage_path.as_ref() else {
            return Ok(());
        };
        let ledger_file = storage_path.join("ledger.json");

        // Best-effort startup cleanup: remove unique-tmp siblings leaked by a
        // crashed prior writer. Ignore all errors.
        if let Ok(dir_entries) = std::fs::read_dir(storage_path) {
            let tmp_prefix = format!(
                "{}.tmp.",
                ledger_file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("ledger.json")
            );
            for dir_entry in dir_entries.flatten() {
                let file_name = dir_entry.file_name();
                if file_name.to_str().map_or(false, |name| name.starts_with(&tmp_prefix)) {
                    let _ = std::fs::remove_file(dir_entry.path());
                }
            }
        }

        if ledger_file.exists() {
            if let Ok(metadata) = std::fs::metadata(&ledger_file) {
                if metadata.len() > 16 * 1024 * 1024 {
                    tracing::warn!(
                        "ledger file is {} bytes; parsing may exceed bounded-memory targets",
                        metadata.len()
                    );
                }
            }
            let data = std::fs::read_to_string(&ledger_file)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            let mut entries: Vec<LedgerEntry> =
                serde_json::from_str(&data).map_err(|_| crate::IronCoreError::Internal)?;
            let parsed_len = entries.len();
            entries.retain(|entry| {
                let public_key_ok = entry
                    .public_key
                    .as_ref()
                    .map_or(true, |value| value.trim().len() <= MAX_LEN_PUBLIC_KEY);
                let nickname_ok = entry
                    .nickname
                    .as_ref()
                    .map_or(true, |value| value.trim().len() <= MAX_LEN_NICKNAME);
                let peer_id_ok = entry.peer_id.as_ref().map_or(true, |peer_id| {
                    peer_id.is_empty()
                        || (peer_id.len() <= MAX_LEN_PEER_ID
                            && peer_id.parse::<libp2p::PeerId>().is_ok())
                });
                entry.multiaddr.len() <= MAX_LEN_MULTIADDR && peer_id_ok && public_key_ok && nickname_ok
            });
            let mut changed = entries.len() != parsed_len;
            if entries.len() > MAX_LEDGER_ENTRIES {
                entries.sort_by(|a, b| {
                    (b.success_count > 0)
                        .cmp(&(a.success_count > 0))
                        .then_with(|| b.last_seen.unwrap_or(0).cmp(&a.last_seen.unwrap_or(0)))
                        .then_with(|| a.multiaddr.cmp(&b.multiaddr))
                });
                entries.truncate(MAX_LEDGER_ENTRIES);
                changed = true;
            }
            *self.entries.lock() = entries;
            if changed {
                let _save_guard = self.save_lock.lock();
                let snapshot = { self.entries.lock().clone() };
                if let Err(err) = self.save_with_entries(&snapshot) {
                    tracing::warn!("ledger shrink persist failed: {}", err);
                }
            }
        }
        Ok(())
    }

    fn save_with_entries(&self, entries: &[LedgerEntry]) -> Result<(), crate::IronCoreError> {
        let Some(storage_path) = self.storage_path.as_ref() else {
            return Ok(());
        };
        std::fs::create_dir_all(storage_path).map_err(|_| crate::IronCoreError::StorageError)?;

        let ledger_file = storage_path.join("ledger.json");
        let data =
            serde_json::to_string_pretty(entries).map_err(|_| crate::IronCoreError::Internal)?;

        let tmp_file = ledger_file.with_file_name(format!(
            "ledger.json.tmp.{}.{}",
            std::process::id(),
            SAVE_TMP_NONCE.fetch_add(1, Ordering::Relaxed)
        ));

        let tmp_result = (|| {
            let mut file = std::fs::File::create(&tmp_file)?;
            file.write_all(data.as_bytes())?;
            file.sync_all()
        })();
        if let Err(err) = tmp_result {
            let _ = std::fs::remove_file(&tmp_file);
            tracing::warn!("ledger tmp write failed: {}", err);
            return Err(crate::IronCoreError::StorageError);
        }

        if let Err(err) = std::fs::rename(&tmp_file, &ledger_file) {
            let _ = std::fs::remove_file(&tmp_file);
            tracing::warn!("ledger rename failed: {}", err);
            return Err(crate::IronCoreError::StorageError);
        }

        #[cfg(unix)]
        {
            // Best-effort directory fsync so the rename itself is durable.
            if let Ok(dir) = std::fs::File::open(storage_path) {
                let _ = dir.sync_all();
            }
        }
        #[cfg(not(unix))]
        {
            // Same-directory std::fs::rename maps to MoveFileExW with replace
            // semantics; Windows gives atomic replacement without a separate
            // directory fsync surface here.
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), crate::IronCoreError> {
        let _save_guard = self.save_lock.lock();
        let entries = {
            let guard = self.entries.lock();
            (*guard).clone()
        };
        self.save_with_entries(&entries)
    }

    /// Record that we reached `peer_id` at `multiaddr`.
    ///
    /// INGESTION CHOKE POINT (re-review round 4, F3/NEW-1). The address gate
    /// used to live in the CALLERS, which is how `cli/src/main.rs` ended up with
    /// the DNS gate in `cmd_relay`'s `PeerIdentified` handler and not in
    /// `cmd_start`'s byte-identical one. The gate now lives here, so no caller
    /// can record an unvalidated address at all.
    ///
    /// TWO RULES, both unconditional, with NO parameter a call site can turn the
    /// wrong way -- deliberately stronger than "make the policy a required
    /// argument", because a required argument is still something a future call
    /// site can get wrong:
    ///
    /// 1. **No DNS forms.** A `/dns4/...` entry resolves to whatever its zone
    ///    owner says at dial time and is re-pointable between probes, so a
    ///    stored name is an SSRF primitive with an indefinite lifetime. Every
    ///    caller of this method is recording an address that came off a live
    ///    socket (`swarm.rs` passes the resolved `remote_addr` of an established
    ///    OUTBOUND connection; the UniFFI surface is called from a platform
    ///    client after its own connection succeeded), and a connected socket's
    ///    address is an IP literal by construction. There is therefore no
    ///    legitimate DNS provenance for this entry point and no reason to offer
    ///    one. Operator-configured names reach the swarm through
    ///    `bootstrap_addrs`, not through the ledger.
    /// 2. **A transport component is required.** `"".parse::<Multiaddr>()`
    ///    returns `Ok(<empty>)` (review F9), so "it parsed" proves nothing; an
    ///    empty or peer-id-only record would be stored and later gossiped.
    ///
    /// NOT REJECTED HERE, deliberately: loopback and RFC1918. This method's
    /// meaning is "we actually reached this address", and an address we just
    /// used demonstrably works for us. The routability filter belongs at the
    /// RE-DIAL and DISCLOSURE boundaries -- `build_seed_dial_candidates`,
    /// [`Self::exchange_response_entries`] and [`Self::export_seed_entries`] --
    /// which is where a LAN neighbour stops being useful and starts being
    /// reconnaissance. Rejecting them here would also make
    /// `lan_only_node_discloses_nothing_to_a_stranger` vacuous.
    pub fn record_connection(&self, multiaddr: String, peer_id: String) {
        let _save_guard = self.save_lock.lock();
        if !is_recordable_multiaddr(&multiaddr) {
            tracing::debug!(
                "Refusing to record a connection against a DNS-form or transport-less                  multiaddr: {}",
                multiaddr
            );
            return;
        }
        if multiaddr.len() > MAX_LEN_MULTIADDR
            || (!peer_id.is_empty()
                && (peer_id.len() > MAX_LEN_PEER_ID
                    || peer_id.parse::<libp2p::PeerId>().is_err()))
        {
            return;
        }

        let snapshot = {
        let mut entries = self.entries.lock();
        let target_port = get_multiaddr_port(&multiaddr);
        let mut found_dns_idx = None;
        for (idx, entry) in entries.iter().enumerate() {
            if entry.peer_id.as_deref() == Some(&peer_id)
                && is_dns_multiaddr(&entry.multiaddr)
                && (target_port.is_none() || get_multiaddr_port(&entry.multiaddr) == target_port)
            {
                found_dns_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = found_dns_idx {
            let entry = &mut entries[idx];
            entry.success_count += 1;
            entry.last_seen = Some(current_timestamp());
        } else if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.success_count += 1;
            entry.peer_id = Some(peer_id);
            entry.last_seen = Some(current_timestamp());
        } else {
            while entries.len() >= MAX_LEDGER_ENTRIES {
                evict_one_locked(&mut entries);
            }
            entries.push(LedgerEntry {
                multiaddr,
                peer_id: Some(peer_id),
                public_key: None,
                nickname: None,
                success_count: 1,
                failure_count: 0,
                last_seen: Some(current_timestamp()),
                topics: Vec::new(),
            });
        }
        (*entries).clone()
        };
        let _ = self.save_with_entries(&snapshot);
    }

    pub fn record_failure(&self, multiaddr: String) {
        let _save_guard = self.save_lock.lock();
        let snapshot = {
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.failure_count += 1;
        }
        (*entries).clone()
        };
        let _ = self.save_with_entries(&snapshot);
    }

    /// Attach the identity learned from Identify to a ledger entry.
    ///
    /// SIBLING OF [`Self::record_connection`] (re-review round 4). This is the
    /// OTHER function that can create a `LedgerEntry` from an address, and it is
    /// the wire-driven one (`mobile_bridge.rs` calls it with raw
    /// `/sc/ledger-exchange/1.0.0` data). Gating `record_connection` and leaving
    /// this open would be exactly the partial application the choke-point
    /// refactor exists to stop, so it runs the same ingestion predicate.
    ///
    /// Entries created here keep `success_count = 0`, so they remain in the
    /// unproven seed tier and are never disclosed by
    /// [`Self::exchange_response_entries`]; the ingestion gate is defence in
    /// depth on top of that, not a replacement for it.
    pub fn annotate_identity(
        &self,
        multiaddr: String,
        peer_id: String,
        public_key: Option<String>,
        nickname: Option<String>,
    ) {
        let _save_guard = self.save_lock.lock();
        if !is_recordable_multiaddr(&multiaddr) {
            tracing::debug!(
                "Refusing to annotate a DNS-form or transport-less multiaddr: {}",
                multiaddr
            );
            return;
        }
        let snapshot = {
            let mut entries = self.entries.lock();
            let _ = annotate_identity_locked(
                &mut entries,
                multiaddr,
                peer_id,
                public_key,
                nickname,
            );
            (*entries).clone()
        };
        let _ = self.save_with_entries(&snapshot);
    }

    pub fn dialable_addresses(&self) -> Vec<LedgerEntry> {
        let entries = self.entries.lock();
        entries
            .iter()
            .filter(|e| e.success_count > 0 && e.failure_count < 5)
            .cloned()
            .collect()
    }

    /// Addresses known only from an invite/QR seed: recorded, syntactically
    /// valid, but never yet successfully dialed by us.
    ///
    /// WHY A SEPARATE ACCESSOR rather than relaxing
    /// [`Self::dialable_addresses`]: that filter (`success_count > 0 &&
    /// failure_count < 5`) means "addresses we have actually reached", and the
    /// CLI depends on exactly that meaning -- its startup `DialScheduler` sweep,
    /// its relay ranking and its ledger display all read it. Folding unproven,
    /// attacker-suppliable seed addresses into it would silently change what
    /// every existing caller believes it is getting. Seeds are a strictly
    /// lower-confidence tier, so they get their own accessor and callers opt in
    /// by name: sweep the proven set first, then this one. A first successful
    /// connection promotes a seed into the proven set via
    /// [`Self::record_connection`] with no special casing.
    ///
    /// `limit` bounds the returned Vec (review F4). The seed tier is the
    /// attacker-suppliable tier and this used to clone the ENTIRE unproven set
    /// on every `ConnectToSeedPeers`, synchronously on the swarm event-loop
    /// thread. `0` means "no entries", not "unlimited".
    pub fn seed_addresses(&self, limit: u32) -> Vec<LedgerEntry> {
        let entries = self.entries.lock();
        let mut candidates: Vec<LedgerEntry> = entries
            .iter()
            .filter(|e| e.success_count == 0 && e.failure_count < LEDGER_DEAD_FAILURE_THRESHOLD)
            .cloned()
            .collect();
        candidates.sort_by(|a, b| {
            match (a.last_seen, b.last_seen) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(a_last_seen), Some(b_last_seen)) => b_last_seen.cmp(&a_last_seen),
            }
            .then_with(|| b.success_count.cmp(&a.success_count))
        });
        candidates.into_iter().take(limit as usize).collect()
    }

    /// Export our best-known peers as routing-only seed entries for an invite.
    ///
    /// Ordered by [`Self::get_preferred_relays`] ranking (proven peers, most
    /// recently seen first). The caller is responsible for prepending its own
    /// dialable address -- see `crate::relay::invite::build_seed_ledger`.
    ///
    /// Everything except the multiaddr is dropped here, including the peer id:
    /// see the type-level note on [`SeedLedgerEntry`]. This is the only export
    /// path for invites, so it is also the choke point that keeps third-party
    /// identity out of them.
    ///
    /// ADDRESS FILTERING (re-review NEW-7). This had no address filter at all,
    /// so a node that had dialed loopback or its own LAN baked those addresses
    /// into an invite QR -- a durable, forwardable artefact, which is strictly
    /// worse than the wire disclosure NEW-2 covers. It now runs the SAME
    /// predicate as the ledger-exchange reply
    /// ([`crate::transport::addr_filter::is_disclosable_multiaddr`]), so an
    /// invite can only ever carry globally routable, non-DNS addresses.
    ///
    /// CONSEQUENCE, called out because it is a real functional limit and not an
    /// oversight: an invite can no longer carry an RFC1918 address, so the
    /// "invite the person next to me onto my LAN mesh" cold start is not served
    /// by this function. Wiring that case up needs a caller that knows the
    /// invite is being handed over in person; [`Self::export_seed_entries_for`]
    /// exists for it and is deliberately not the default. Today nothing accepts
    /// an invite at all (review F2), so this closes a latent leak rather than
    /// removing a working feature.
    pub fn export_seed_entries(&self, limit: u32) -> Vec<SeedLedgerEntry> {
        self.export_seed_entries_for(limit, SeedExportAudience::Untrusted)
    }

    /// Merge seed entries learned out-of-band (invite / QR) into the ledger.
    /// Returns the number of entries that were newly added.
    ///
    /// MERGE POLICY (deliberate -- seed data is attacker-suppliable):
    /// - Dedupe key is the `/p2p/`-stripped multiaddr, matching the CLI
    ///   ledger's key convention (`cli/src/ledger.rs::strip_peer_id`).
    /// - A seed carries no identity and no counters, so there is nothing to
    ///   merge into an existing entry: a known address is left completely
    ///   untouched. `success_count`, `failure_count`, `last_seen`, `peer_id`,
    ///   `public_key` and `nickname` all keep their current values. An invite
    ///   is not evidence that a peer was reachable at any particular time, and
    ///   it is certainly not evidence about who is listening there.
    /// - New entries are added with `success_count = 0` and no identity fields.
    ///   That means they are deliberately NOT returned by
    ///   [`Self::dialable_addresses`] (which requires `success_count > 0`) nor
    ///   by [`Self::get_preferred_relays`]: an unproven address handed to us by
    ///   whoever held the invite must not masquerade as an address we have
    ///   actually reached. They surface through [`Self::seed_addresses`]
    ///   instead -- see the reasoning on that method. The first successful
    ///   connection promotes the entry via [`Self::record_connection`], and
    ///   [`Self::annotate_identity`] attaches the identity learned from
    ///   Identify at that point.
    /// - Entries whose multiaddr does not parse, is empty, carries no transport
    ///   component, or is not routable
    ///   ([`crate::transport::addr_filter::is_dialable_multiaddr`]) are
    ///   dropped, and the whole batch is capped at
    ///   [`MAX_SEED_LEDGER_ENTRIES`].
    ///
    /// Uses [`NetworkMode::Local`], i.e. RFC1918 peers stay importable: an
    /// invite is the LAN/mesh cold-start path and a node has no reliable way to
    /// know its own network context from inside the store layer. Callers that
    /// do know (a cellular-only node) should use
    /// [`Self::import_seed_entries_with_mode`].
    pub fn import_seed_entries(&self, entries: Vec<SeedLedgerEntry>) -> u32 {
        let _save_guard = self.save_lock.lock();
        self.import_seed_entries_locked(entries, NetworkMode::Local)
    }

    pub fn get_preferred_relays(&self, limit: u32) -> Vec<LedgerEntry> {
        let entries = self.entries.lock();
        let mut preferred: Vec<LedgerEntry> = entries
            .iter()
            .filter(|e| e.success_count > 0 && e.failure_count < LEDGER_DEAD_FAILURE_THRESHOLD)
            .cloned() // Clone now so we can sort
            .collect();
        // Sort by last_seen descending
        preferred.sort_by_key(|b| std::cmp::Reverse(b.last_seen.unwrap_or(0)));
        preferred.truncate(limit as usize);
        preferred
    }

    pub fn all_known_topics(&self) -> Vec<String> {
        let entries = self.entries.lock();
        let mut topics: Vec<String> = entries.iter().flat_map(|e| e.topics.clone()).collect();
        topics.sort();
        topics.dedup();
        topics
    }

    pub fn summary(&self) -> String {
        let entries = self.entries.lock();
        format!("Ledger contains {} peer entries", entries.len())
    }
}

/// Rust-only surface. Deliberately NOT `uniffi::export`ed: these methods take
/// [`NetworkMode`] or exist purely to keep the swarm event loop bounded, and
/// neither concept belongs in the mobile binding.
impl LedgerManager {
    /// A ledger that lives entirely in memory and never touches the disk.
    ///
    /// Review F11: `IronCore::new()` -- the storage-less constructor -- used to
    /// build its `LedgerManager` over `std::env::temp_dir()`, i.e. it wrote the
    /// node's peer topology (who we talk to, at which addresses, how often)
    /// into a world-readable directory on every desktop platform. An in-memory
    /// core gets an in-memory ledger.
    pub fn ephemeral() -> Self {
        Self {
            storage_path: None,
            entries: Arc::new(Mutex::new(Vec::new())),
            save_lock: Mutex::new(()),
        }
    }

    /// [`Self::export_seed_entries`] with an explicit audience.
    ///
    /// Not `uniffi::export`ed: [`SeedExportAudience`] is a security decision
    /// that must be made by a call site that knows how the invite is delivered,
    /// and "pick an enum variant" is exactly the kind of choice a binding
    /// consumer gets wrong by default.
    pub fn export_seed_entries_for(
        &self,
        limit: u32,
        audience: SeedExportAudience,
    ) -> Vec<SeedLedgerEntry> {
        self.get_preferred_relays(limit)
            .into_iter()
            .map(|entry| strip_peer_id_component(&entry.multiaddr))
            .filter(|addr| match audience {
                SeedExportAudience::Untrusted => is_disclosable_multiaddr(addr),
                SeedExportAudience::LocalMesh => {
                    is_dialable_multiaddr(addr, NetworkMode::Local, DnsPolicy::Reject)
                }
            })
            .map(|multiaddr| SeedLedgerEntry { multiaddr })
            .collect()
    }

    /// [`Self::import_seed_entries`] with an explicit network mode.
    ///
    /// Every rejection reason is deliberate; see review F3 and F9:
    /// - `stripped.is_empty()`: `"".parse::<Multiaddr>()` returns
    ///   `Ok(<empty>)`, so a seed of `"/p2p/QmX"` stripped to `""` and was
    ///   stored, then gossiped onward as an empty record.
    /// - not dialable: loopback / unspecified / link-local (including
    ///   `169.254.169.254`) / multicast / broadcast / RFC1918-in-`Public`.
    ///   Without this an invite holder could load a victim's dial set with
    ///   internal host:port pairs and read open/closed off the dial timing.
    pub fn import_seed_entries_with_mode(
        &self,
        entries: Vec<SeedLedgerEntry>,
        mode: NetworkMode,
    ) -> u32 {
        let _save_guard = self.save_lock.lock();
        self.import_seed_entries_locked(entries, mode)
    }

    fn import_seed_entries_locked(
        &self,
        entries: Vec<SeedLedgerEntry>,
        mode: NetworkMode,
    ) -> u32 {
        let (snapshot, added) = {
        let mut ledger = self.entries.lock();
        let mut added = 0u32;

        if entries.len() > MAX_SEED_LEDGER_ENTRIES {
            tracing::warn!(
                "Seed import capped: {} entries offered, {} accepted",
                entries.len(),
                MAX_SEED_LEDGER_ENTRIES
            );
        }

        for seed in entries.into_iter().take(MAX_SEED_LEDGER_ENTRIES) {
            let stripped = strip_peer_id_component(&seed.multiaddr);
            if stripped.is_empty() {
                tracing::debug!("Dropping seed multiaddr with no transport component");
                continue;
            }
            // `DnsPolicy::Reject`: an invite's `seed_ledger` is supplied by
            // whoever produced the invite, so a `/dns4/...` entry would let them
            // re-point our dial target after the fact (re-review NEW-1).
            if !is_dialable_multiaddr(&stripped, mode, DnsPolicy::Reject) {
                tracing::debug!("Dropping non-routable seed multiaddr: {}", stripped);
                continue;
            }

            let already_known = ledger
                .iter()
                .any(|e| strip_peer_id_component(&e.multiaddr) == stripped);

            // A known address is left exactly as it is. A seed has no field
            // that could improve it, and none that we would trust if it did.
            if !already_known {
                if seed.multiaddr.len() > MAX_LEN_MULTIADDR
                    || stripped.len() > MAX_LEN_MULTIADDR
                {
                    tracing::debug!("Dropping over-length seed multiaddr");
                    continue;
                }
                while ledger.len() >= MAX_LEDGER_ENTRIES {
                    evict_one_locked(&mut ledger);
                }
                ledger.push(LedgerEntry {
                    multiaddr: stripped,
                    peer_id: None,
                    public_key: None,
                    nickname: None,
                    success_count: 0,
                    failure_count: 0,
                    last_seen: None,
                    topics: Vec::new(),
                });
                added += 1;
            }
        }

        ((*ledger).clone(), added)
        };
        let _ = self.save_with_entries(&snapshot);
        added
    }

    pub fn annotate_identities_batch(&self, items: Vec<(String, String, Option<String>, Option<String>)>) {
        let _save_guard = self.save_lock.lock();
        let snapshot = {
            let mut entries = self.entries.lock();
            for (multiaddr, peer_id, public_key, nickname) in items {
                if !is_recordable_multiaddr(&multiaddr) {
                    tracing::debug!(
                        "Refusing to annotate a DNS-form or transport-less multiaddr: {}",
                        multiaddr
                    );
                    continue;
                }
                let _ = annotate_identity_locked(&mut entries, multiaddr, peer_id, public_key, nickname);
            }
            (*entries).clone()
        };
        let _ = self.save_with_entries(&snapshot);
    }

    /// Build the peer list for a `/sc/ledger-exchange/1.0.0` RESPONSE.
    ///
    /// This is the single choke point for review F6: the response goes to any
    /// peer that completed a Noise handshake, with no app-layer opt-in, so
    /// every restriction has to live here rather than at the call site.
    ///
    /// - `limit` is applied BEFORE cloning, so a large ledger cannot make the
    ///   swarm event loop allocate a large vector per request.
    /// - Entries are filtered through the same routability gate as dial
    ///   candidates, so we never disclose our RFC1918-in-`Public` neighbours,
    ///   loopback services, or link-local addresses to an internet peer.
    /// - `known_topics` is dropped unconditionally. Gossipsub topic names are
    ///   group-membership / social-graph data about THIRD PARTIES who never
    ///   consented to appearing in our answer to a stranger, and disclosing
    ///   them directly contradicts the "where to knock, not who lives there"
    ///   principle this feature is documented on (see [`SeedLedgerEntry`]).
    /// - The requester is never echoed back to itself.
    ///
    /// NO `NetworkMode` PARAMETER, deliberately (re-review NEW-2). This used to
    /// take one and the swarm hardcoded `NetworkMode::Local` at the call site,
    /// which is the mode that SKIPS the `is_private()` check. Since
    /// `record_connection` is intentionally unfiltered, every LAN peer we had
    /// ever dialed was a proven, disclosable record -- internal subnet, live
    /// host:port and the neighbour's `last_peer_id` -- shipped to any peer that
    /// completed a Noise handshake. Disclosure is a different question from
    /// dialability and now uses
    /// [`crate::transport::addr_filter::is_disclosable_multiaddr`], which has no
    /// knob a caller can turn the wrong way.
    pub fn exchange_response_entries(
        &self,
        limit: usize,
        requester_peer_id: &str,
    ) -> Vec<SharedPeerEntry> {
        let entries = self.entries.lock();
        entries
            .iter()
            .filter(|e| e.success_count > 0 && e.failure_count < 5)
            .filter(|e| e.peer_id.as_deref() != Some(requester_peer_id))
            .filter(|e| is_disclosable_multiaddr(&strip_peer_id_component(&e.multiaddr)))
            .take(limit)
            .map(ledger_entry_to_shared_routing_only)
            .collect()
    }
}

/// A shared peer entry for ledger exchange.
/// Stripped-down version of ledger data suitable for wire transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SharedPeerEntry {
    /// The multiaddr (transport address only, no /p2p/ suffix)
    pub multiaddr: String,
    /// Last known PeerID at this address (if any)
    pub last_peer_id: Option<String>,
    /// Unix timestamp of last successful connection
    pub last_seen: u64,
    /// Gossipsub topics this peer was subscribed to
    pub known_topics: Vec<String>,
}

/// Convert a stored [`LedgerEntry`] into the wire form used by
/// `/sc/ledger-exchange/1.0.0`.
///
/// UNIT CONVERSION, do not "simplify" this away: [`LedgerEntry::last_seen`] is
/// stored in **milliseconds** (see `current_timestamp` at the top of this
/// file), while [`SharedPeerEntry::last_seen`] is a Unix timestamp in
/// **seconds** -- that is what the CLI ledger emits and what
/// `MultiPathDelivery::record_recipient_seen_via_relay` compares against
/// `unix_now_secs()`. Shipping milliseconds on the wire makes every shared
/// peer look ~1000x more recent than it is and corrupts relay ranking.
pub fn ledger_entry_to_shared(entry: &LedgerEntry) -> SharedPeerEntry {
    SharedPeerEntry {
        multiaddr: strip_peer_id_component(&entry.multiaddr),
        last_peer_id: entry.peer_id.clone(),
        last_seen: entry.last_seen.unwrap_or(0) / 1000,
        known_topics: entry.topics.clone(),
    }
}

/// [`ledger_entry_to_shared`] with `known_topics` forced empty.
///
/// Review F6: the ledger-exchange response is readable by any peer that
/// completes a Noise handshake. Topic names are third-party group membership,
/// not routing information, and have no business in an unauthenticated reply.
/// Kept as a separate function (rather than a parameter) so the wire shape used
/// by the disclosure path is greppable and testable on its own.
pub fn ledger_entry_to_shared_routing_only(entry: &LedgerEntry) -> SharedPeerEntry {
    SharedPeerEntry {
        known_topics: Vec::new(),
        ..ledger_entry_to_shared(entry)
    }
}

fn default_version() -> u8 {
    1
}

/// Ledger exchange request — sent automatically on new connection.
/// "Here are all the peers I know about. Tell me yours."
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeRequest {
    /// Explicit version tag for bincode wire format
    #[serde(default = "default_version")]
    pub version_tag: u8,
    /// Our known peers (shared generously)
    pub peers: Vec<SharedPeerEntry>,
    /// Our own PeerID (so the remote can record us)
    pub sender_peer_id: String,
    /// Protocol version for forward compatibility
    pub version: u32,
}

/// Ledger exchange response — reciprocal sharing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeResponse {
    /// Explicit version tag for bincode wire format
    #[serde(default = "default_version")]
    pub version_tag: u8,
    /// Their known peers (shared back)
    pub peers: Vec<SharedPeerEntry>,
    /// Number of new peers they learned from our request
    pub new_peers_learned: u32,
    /// Protocol version
    pub version: u32,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> (tempfile::TempDir, LedgerManager) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();
        (dir, LedgerManager::new(path))
    }

    fn seed(addr: &str) -> SeedLedgerEntry {
        SeedLedgerEntry {
            multiaddr: addr.to_string(),
        }
    }

    /// A syntactically valid peer id. The old string-truncating
    /// `strip_peer_id_component` accepted any junk after `/p2p/`; the
    /// protocol-iterating replacement (review F8) requires the whole multiaddr
    /// to parse, so the fixtures have to be real.
    fn peer() -> String {
        libp2p::PeerId::random().to_string()
    }

    #[test]
    fn seed_threshold_boundary() {
        let (_dir, mgr) = manager();
        assert_eq!(
            mgr.import_seed_entries(vec![
                seed("/ip4/10.0.0.2/tcp/9001"),
                seed("/ip4/10.0.0.3/tcp/9001"),
                seed("/ip4/10.0.0.4/tcp/9001"),
                seed("/ip4/10.0.0.5/tcp/9001"),
            ]),
            4
        );
        for _ in 0..2 {
            mgr.record_failure("/ip4/10.0.0.2/tcp/9001".to_string());
            mgr.record_failure("/ip4/10.0.0.4/tcp/9001".to_string());
        }
        for _ in 0..3 {
            mgr.record_failure("/ip4/10.0.0.3/tcp/9001".to_string());
            mgr.record_failure("/ip4/10.0.0.5/tcp/9001".to_string());
        }

        let seeds = mgr.seed_addresses(64);
        assert!(seeds.iter().any(|e| e.multiaddr == "/ip4/10.0.0.2/tcp/9001"));
        assert!(!seeds.iter().any(|e| e.multiaddr == "/ip4/10.0.0.3/tcp/9001"));

        mgr.record_connection("/ip4/10.0.0.4/tcp/9001".to_string(), peer());
        mgr.record_connection("/ip4/10.0.0.5/tcp/9001".to_string(), peer());
        let relays = mgr.get_preferred_relays(64);
        assert!(relays.iter().any(|e| e.multiaddr == "/ip4/10.0.0.4/tcp/9001"));
        assert!(!relays.iter().any(|e| e.multiaddr == "/ip4/10.0.0.5/tcp/9001"));
    }

    #[test]
    fn oversize_and_bad_peerid_rejected() {
        let (_dir, mgr) = manager();
        let long_multiaddr = format!(
            "/ip4/198.51.100.9/tcp/9001{}",
            (0..12).map(|_| format!("/p2p/{}", peer())).collect::<String>()
        );
        assert!(long_multiaddr.len() > MAX_LEN_MULTIADDR);
        mgr.annotate_identity(long_multiaddr, peer(), None, None);

        let long_peer = "P".repeat(MAX_LEN_PEER_ID + 1);
        assert!(long_peer.len() > MAX_LEN_PEER_ID);
        mgr.annotate_identity(
            "/ip4/198.51.100.10/tcp/9001".to_string(),
            long_peer,
            None,
            None,
        );

        mgr.annotate_identity(
            "/ip4/198.51.100.11/tcp/9001".to_string(),
            "not-a-peer-id".to_string(),
            None,
            None,
        );

        assert!(mgr.seed_addresses(64).is_empty());
        assert!(mgr.dialable_addresses().is_empty());

        let entries_before = mgr.entries.lock().len();
        mgr.annotate_identity(
            "/ip4/198.51.100.12/tcp/9001".to_string(),
            peer(),
            Some("K".repeat(MAX_LEN_PUBLIC_KEY + 1)),
            None,
        );
        assert_eq!(mgr.entries.lock().len(), entries_before);
        mgr.annotate_identity(
            "/ip4/198.51.100.13/tcp/9001".to_string(),
            peer(),
            None,
            Some("N".repeat(MAX_LEN_NICKNAME + 1)),
        );
        assert_eq!(mgr.entries.lock().len(), entries_before);
    }

    #[test]
    fn ledger_load_caps_oversized_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mk = |addr: String, success, last_seen| LedgerEntry { multiaddr: addr, peer_id: None, public_key: None, nickname: None, success_count: success, failure_count: 0, last_seen: Some(last_seen), topics: Vec::new() };
        let proven = "/ip4/203.0.113.100/tcp/9001".to_string();
        let oldest_zero = "/ip4/10.0.0.0/tcp/9001".to_string();
        let mut entries: Vec<LedgerEntry> = (0..19).map(|i| mk(format!("/ip4/203.0.113.{}/tcp/9001", i + 1), 1, 4000 + i as u64)).collect();
        entries.push(mk(proven.clone(), 3, 5000));
        entries.push(mk(oldest_zero.clone(), 0, 1));
        let extra = MAX_LEDGER_ENTRIES + 100 - entries.len();
        entries.extend((0..extra).map(|i| mk(format!("/ip4/10.0.{}.{}/tcp/9001", i / 250, (i % 250) + 1), 0, 100 + i as u64)));
        std::fs::write(dir.path().join("ledger.json"), serde_json::to_string_pretty(&entries).unwrap()).unwrap();
        let mgr = LedgerManager::new(dir.path().to_string_lossy().to_string());
        mgr.load().expect("load");
        let loaded = mgr.entries.lock();
        assert_eq!(loaded.len(), MAX_LEDGER_ENTRIES);
        assert!(loaded.iter().any(|e| e.multiaddr == proven && e.success_count > 0));
        assert!(!loaded.iter().any(|e| e.multiaddr == oldest_zero));
    }

    #[test]
    fn strip_peer_id_component_matches_cli_convention() {
        let pid = peer();
        assert_eq!(
            strip_peer_id_component(&format!("/ip4/1.2.3.4/tcp/9001/p2p/{pid}")),
            "/ip4/1.2.3.4/tcp/9001"
        );
        assert_eq!(
            strip_peer_id_component("/ip4/1.2.3.4/tcp/9001"),
            "/ip4/1.2.3.4/tcp/9001"
        );
    }

    /// F8 regression. Before the fix this returned `/ip4/1.2.3.4/tcp/443` --
    /// the RELAY's address -- and the caller kept the TARGET's peer id, so the
    /// wire record claimed the target was directly reachable at the relay's
    /// IP:port. Recipients feed that into `kademlia.add_address()`.
    #[test]
    fn strip_peer_id_component_does_not_collapse_circuit_to_relay_address() {
        let relay = peer();
        let target = peer();
        let circuit = format!("/ip4/1.2.3.4/tcp/443/p2p/{relay}/p2p-circuit/p2p/{target}");

        let stripped = strip_peer_id_component(&circuit);

        assert_ne!(
            stripped, "/ip4/1.2.3.4/tcp/443",
            "circuit address collapsed to the bare relay address"
        );
        assert_eq!(
            stripped,
            format!("/ip4/1.2.3.4/tcp/443/p2p/{relay}/p2p-circuit")
        );
        assert!(!stripped.contains(&target), "target peer id must be gone");
        assert!(
            stripped.contains(&relay),
            "relay peer id is part of the address and must survive"
        );
    }

    /// F8 regression at the wire boundary: the record we ship must never say
    /// "<target> is at <relay ip>:<relay port>".
    #[test]
    fn ledger_entry_to_shared_never_binds_target_peer_id_to_relay_address() {
        let relay = peer();
        let target = peer();
        let entry = LedgerEntry {
            multiaddr: format!("/ip4/1.2.3.4/tcp/443/p2p/{relay}/p2p-circuit/p2p/{target}"),
            peer_id: Some(target.clone()),
            public_key: None,
            nickname: None,
            success_count: 1,
            failure_count: 0,
            last_seen: Some(1_700_000_000_000),
            topics: Vec::new(),
        };

        let shared = ledger_entry_to_shared(&entry);

        assert_eq!(shared.last_peer_id.as_deref(), Some(target.as_str()));
        assert_ne!(
            shared.multiaddr, "/ip4/1.2.3.4/tcp/443",
            "shared record binds the target peer id to the relay's direct address"
        );
        assert!(
            shared.multiaddr.contains("/p2p-circuit"),
            "the circuit hop must remain visible so recipients treat it as relayed, got {}",
            shared.multiaddr
        );
    }

    #[test]
    fn import_seed_entries_adds_unproven_entries() {
        let (_dir, mgr) = manager();
        let added = mgr.import_seed_entries(vec![
            seed("/ip4/10.0.0.1/tcp/9001"),
            seed("/ip4/10.0.0.2/tcp/9001"),
        ]);
        assert_eq!(added, 2);

        // Seeds are unproven: they must NOT appear as dialable/preferred.
        assert!(mgr.dialable_addresses().is_empty());
        assert!(mgr.get_preferred_relays(10).is_empty());

        // ...but they must be reachable through the seed accessor, and they
        // must carry no identity whatsoever.
        let seeds = mgr.seed_addresses(64);
        assert_eq!(seeds.len(), 2);
        assert!(seeds.iter().all(|e| e.success_count == 0));
        assert!(
            seeds
                .iter()
                .all(|e| e.peer_id.is_none() && e.public_key.is_none() && e.nickname.is_none()),
            "seed import must not populate identity fields"
        );
    }

    #[test]
    fn import_seed_entries_dedupes_on_stripped_multiaddr() {
        let (_dir, mgr) = manager();
        assert_eq!(
            mgr.import_seed_entries(vec![seed("/ip4/10.0.0.1/tcp/9001")]),
            1
        );
        // Same address, /p2p/ suffix attached -- must dedupe, not duplicate.
        assert_eq!(
            mgr.import_seed_entries(vec![seed(&format!(
                "/ip4/10.0.0.1/tcp/9001/p2p/{}",
                peer()
            ))]),
            0
        );
        let seeds = mgr.seed_addresses(64);
        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0].multiaddr, "/ip4/10.0.0.1/tcp/9001");
        // A peer id smuggled inside the multiaddr string must not survive.
        assert!(seeds[0].peer_id.is_none());
    }

    #[test]
    fn import_seed_entries_never_clobbers_proven_entry() {
        let (_dir, mgr) = manager();
        let proven_peer = peer();
        mgr.record_connection("/ip4/10.0.0.1/tcp/9001".to_string(), proven_peer.clone());
        mgr.record_connection("/ip4/10.0.0.1/tcp/9001".to_string(), proven_peer.clone());
        mgr.record_failure("/ip4/10.0.0.1/tcp/9001".to_string());
        let before = mgr.dialable_addresses();
        assert_eq!(before.len(), 1);
        let (succ, fail, last_seen) = (
            before[0].success_count,
            before[0].failure_count,
            before[0].last_seen,
        );

        // An invite lists an address we already have a history with.
        let added = mgr.import_seed_entries(vec![seed("/ip4/10.0.0.1/tcp/9001")]);
        assert_eq!(added, 0);

        let after = mgr.dialable_addresses();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].success_count, succ, "success_count was clobbered");
        assert_eq!(after[0].failure_count, fail, "failure_count was clobbered");
        assert_eq!(after[0].last_seen, last_seen, "last_seen was clobbered");
        assert_eq!(
            after[0].peer_id.as_deref(),
            Some(proven_peer.as_str()),
            "known peer_id was disturbed by seed data"
        );
    }

    #[test]
    fn import_seed_entries_rejects_garbage_and_caps_batch() {
        let (_dir, mgr) = manager();
        assert_eq!(mgr.import_seed_entries(vec![seed("not-a-multiaddr")]), 0);

        let batch: Vec<SeedLedgerEntry> = (0..MAX_SEED_LEDGER_ENTRIES + 8)
            .map(|i| seed(&format!("/ip4/10.0.1.{}/tcp/9001", i)))
            .collect();
        assert_eq!(
            mgr.import_seed_entries(batch),
            MAX_SEED_LEDGER_ENTRIES as u32
        );
    }

    #[test]
    fn import_seed_entries_survives_reload() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();
        let mgr = LedgerManager::new(path.clone());
        assert_eq!(
            mgr.import_seed_entries(vec![seed("/ip4/10.0.0.7/tcp/9001")]),
            1
        );

        let reloaded = LedgerManager::new(path);
        reloaded.load().expect("load");
        let seeds = reloaded.seed_addresses(64);
        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0].multiaddr, "/ip4/10.0.0.7/tcp/9001");
    }

    #[test]
    fn export_seed_entries_only_exports_proven_peers_without_identity() {
        let (_dir, mgr) = manager();
        let proven = peer();
        let proven_addr = format!("/ip4/198.51.100.1/tcp/9001/p2p/{proven}");
        mgr.import_seed_entries(vec![seed("/ip4/198.51.100.9/tcp/9001")]);
        mgr.record_connection(proven_addr.clone(), proven.clone());
        mgr.annotate_identity(
            proven_addr,
            proven,
            Some("deadbeef".to_string()),
            Some("alice-laptop".to_string()),
        );

        let exported = mgr.export_seed_entries(16);
        assert_eq!(exported.len(), 1);
        // Peer-id-stripped, and the struct has no room for identity at all.
        assert_eq!(exported[0].multiaddr, "/ip4/198.51.100.1/tcp/9001");
    }

    // ------------------------------------------------------------------
    // NEW-7 -- an invite QR must not bake in loopback / LAN addresses
    // ------------------------------------------------------------------

    /// `export_seed_entries` had NO address filter, so any proven entry became
    /// invite content. A node that had dialed `127.0.0.1` (every developer
    /// build, every loopback smoke test) or its own `192.168.x.y` LAN shipped
    /// those in a QR code that outlives the session and can be forwarded.
    #[test]
    fn export_seed_entries_never_bakes_loopback_or_lan_into_an_invite() {
        let (_dir, mgr) = manager();
        for addr in [
            "/ip4/127.0.0.1/tcp/8080",
            "/ip6/::1/tcp/8080",
            "/ip4/169.254.169.254/tcp/80",
            "/ip4/192.168.7.7/tcp/9001",
            "/ip4/10.1.2.3/tcp/9001",
            "/dns4/nas.corp.internal/tcp/443",
            "/ip4/198.51.100.4/tcp/9001",
        ] {
            mgr.record_connection(addr.to_string(), peer());
        }

        let exported = mgr.export_seed_entries(64);
        let addrs: Vec<&str> = exported.iter().map(|e| e.multiaddr.as_str()).collect();
        assert_eq!(
            addrs,
            vec!["/ip4/198.51.100.4/tcp/9001"],
            "invite carried a non-disclosable address: {addrs:?}"
        );
    }

    /// The LAN cold-start case is still reachable, but only by opting in, and
    /// opting in must not also re-enable loopback or an internal hostname.
    #[test]
    fn local_mesh_export_widens_rfc1918_only() {
        let (_dir, mgr) = manager();
        for addr in [
            "/ip4/127.0.0.1/tcp/8080",
            "/ip4/169.254.169.254/tcp/80",
            "/dns4/nas.corp.internal/tcp/443",
            "/ip4/192.168.7.7/tcp/9001",
        ] {
            mgr.record_connection(addr.to_string(), peer());
        }

        let exported = mgr.export_seed_entries_for(64, SeedExportAudience::LocalMesh);
        let addrs: Vec<&str> = exported.iter().map(|e| e.multiaddr.as_str()).collect();
        assert_eq!(addrs, vec!["/ip4/192.168.7.7/tcp/9001"], "got {addrs:?}");
    }

    #[test]
    fn ledger_entry_to_shared_converts_millis_to_seconds() {
        let pid = peer();
        let entry = LedgerEntry {
            multiaddr: format!("/ip4/10.0.0.1/tcp/9001/p2p/{pid}"),
            peer_id: Some(pid),
            public_key: None,
            nickname: None,
            success_count: 3,
            failure_count: 0,
            last_seen: Some(1_700_000_000_123),
            topics: vec!["sc-mesh".to_string()],
        };
        let shared = ledger_entry_to_shared(&entry);
        assert_eq!(shared.last_seen, 1_700_000_000);
        assert_eq!(shared.multiaddr, "/ip4/10.0.0.1/tcp/9001");
        assert_eq!(shared.known_topics, vec!["sc-mesh".to_string()]);
    }

    // ------------------------------------------------------------------
    // F3 -- SSRF / internal-probing addresses must never enter the ledger
    // ------------------------------------------------------------------

    #[test]
    fn import_seed_entries_rejects_ssrf_and_non_routable_addresses() {
        let (_dir, mgr) = manager();
        let hostile = vec![
            // Cloud metadata service.
            seed("/ip4/169.254.169.254/tcp/80"),
            // Loopback -- services bound to the victim's own host.
            seed("/ip4/127.0.0.1/tcp/8080"),
            seed("/ip6/::1/tcp/8080"),
            // IPv4-mapped IPv6 form of the same thing.
            seed("/ip6/::ffff:127.0.0.1/tcp/8080"),
            // Unspecified / multicast / broadcast.
            seed("/ip4/0.0.0.0/tcp/9001"),
            seed("/ip4/224.0.0.1/tcp/9001"),
            seed("/ip4/255.255.255.255/tcp/9001"),
        ];
        let hostile_len = hostile.len();

        assert_eq!(
            mgr.import_seed_entries(hostile),
            0,
            "non-routable seeds were accepted into the ledger"
        );
        assert!(
            mgr.seed_addresses(64).is_empty(),
            "non-routable seeds became dial candidates"
        );
        assert!(mgr.dialable_addresses().is_empty());
        assert_eq!(hostile_len, 7);
    }

    #[test]
    fn import_seed_entries_honours_network_mode_for_rfc1918() {
        let (_dir, local_mgr) = manager();
        // Local mesh: an RFC1918 peer is exactly what invites are for.
        assert_eq!(
            local_mgr.import_seed_entries_with_mode(
                vec![seed("/ip4/192.168.1.1/tcp/443")],
                NetworkMode::Local
            ),
            1
        );

        // Public-only node: it has no route to anyone's LAN, and dialing one is
        // an internal probe.
        let (_dir2, public_mgr) = manager();
        assert_eq!(
            public_mgr.import_seed_entries_with_mode(
                vec![
                    seed("/ip4/192.168.1.1/tcp/443"),
                    seed("/ip4/10.1.2.3/tcp/443"),
                    seed("/ip4/172.20.0.1/tcp/443"),
                ],
                NetworkMode::Public
            ),
            0,
            "RFC1918 seeds accepted on a public-only node"
        );
        assert!(public_mgr.seed_addresses(64).is_empty());
    }

    // ------------------------------------------------------------------
    // F9 -- "" parses as a valid Multiaddr
    // ------------------------------------------------------------------

    #[test]
    fn import_seed_entries_rejects_entries_with_no_transport_component() {
        let (_dir, mgr) = manager();
        // Both of these previously stripped to something that `parse()`
        // accepted ("" and "/p2p-circuit") and were stored and re-gossiped.
        assert_eq!(
            mgr.import_seed_entries(vec![
                seed(&format!("/p2p/{}", peer())),
                seed(&format!("/p2p-circuit/p2p/{}", peer())),
                seed(""),
            ]),
            0
        );
        assert!(mgr.seed_addresses(64).is_empty());
    }

    // ------------------------------------------------------------------
    // F4 -- the seed tier must be bounded before it reaches the event loop
    // ------------------------------------------------------------------

    #[test]
    fn seed_addresses_is_bounded_by_limit() {
        let (_dir, mgr) = manager();
        // Import in MAX_SEED_LEDGER_ENTRIES-sized batches to build a ledger
        // larger than any single caller's cap.
        for batch in 0..8u32 {
            let entries: Vec<SeedLedgerEntry> = (0..MAX_SEED_LEDGER_ENTRIES)
                .map(|i| seed(&format!("/ip4/10.{}.{}.1/tcp/9001", batch, i)))
                .collect();
            mgr.import_seed_entries(entries);
        }
        let total = mgr.seed_addresses(u32::MAX).len();
        assert!(total >= 100, "expected a large ledger, got {total}");

        assert_eq!(mgr.seed_addresses(8).len(), 8);
        assert_eq!(mgr.seed_addresses(1).len(), 1);
        assert_eq!(mgr.seed_addresses(0).len(), 0);
    }

    // ------------------------------------------------------------------
    // F6 -- the ledger-exchange response is an unauthenticated disclosure
    // ------------------------------------------------------------------

    #[test]
    fn exchange_response_entries_caps_filters_and_drops_topics() {
        let (_dir, mgr) = manager();

        // 40 proven, routable peers, each with topic subscriptions.
        for i in 0..40u32 {
            let addr = format!("/ip4/198.51.100.{}/tcp/9001", i + 1);
            let pid = peer();
            mgr.record_connection(addr.clone(), pid.clone());
            mgr.annotate_identity(addr.clone(), pid, None, None);
        }
        {
            let mut entries = mgr.entries.lock();
            for entry in entries.iter_mut() {
                entry.topics = vec!["sc-family-chat".to_string(), "sc-activists".to_string()];
            }
        }
        // A proven but non-routable peer -- we can reach it, a stranger cannot,
        // and telling them about it maps our internal network.
        mgr.record_connection("/ip4/192.168.7.7/tcp/9001".to_string(), peer());
        let requester = peer();
        mgr.record_connection("/ip4/203.0.113.9/tcp/9001".to_string(), requester.clone());

        let response = mgr.exchange_response_entries(16, &requester);

        assert_eq!(response.len(), 16, "response cap not applied");
        assert!(
            response.iter().all(|e| e.known_topics.is_empty()),
            "known_topics leaked group membership into an unauthenticated response"
        );
        assert!(
            !response
                .iter()
                .any(|e| e.multiaddr.starts_with("/ip4/192.168.")),
            "RFC1918 neighbour disclosed to a public peer"
        );
        assert!(
            !response
                .iter()
                .any(|e| e.last_peer_id.as_deref() == Some(requester.as_str())),
            "requester echoed back to itself"
        );
    }

    // ------------------------------------------------------------------
    // NEW-2 -- RFC1918 must never be disclosed, whatever the caller wants
    // ------------------------------------------------------------------

    /// The previous test only proved the filter worked when the CALLER passed
    /// `NetworkMode::Public`, and the swarm passed `Local`. There is no longer a
    /// parameter to get wrong, and this asserts it directly: a ledger made
    /// entirely of LAN neighbours discloses nothing at all.
    #[test]
    fn exchange_response_never_discloses_private_ranges() {
        let (_dir, mgr) = manager();
        for addr in [
            "/ip4/192.168.1.10/tcp/9001",
            "/ip4/192.168.1.11/tcp/9001",
            "/ip4/10.0.2.16/tcp/9001",
            "/ip4/172.20.0.1/tcp/9001",
            "/ip6/fd00::1/tcp/9001",
            "/ip4/127.0.0.1/tcp/8080",
            "/ip4/169.254.169.254/tcp/80",
            "/dns4/nas.corp.internal/tcp/443",
        ] {
            mgr.record_connection(addr.to_string(), peer());
        }
        // One genuinely public neighbour, so an empty result cannot pass by
        // accident.
        mgr.record_connection("/ip4/198.51.100.5/tcp/9001".to_string(), peer());

        let response = mgr.exchange_response_entries(64, "some-other-peer");
        let disclosed: Vec<&str> = response.iter().map(|e| e.multiaddr.as_str()).collect();

        assert_eq!(
            disclosed,
            vec!["/ip4/198.51.100.5/tcp/9001"],
            "internal topology disclosed to an unauthenticated peer: {disclosed:?}"
        );
        assert!(
            response.iter().all(|e| e.known_topics.is_empty()),
            "known_topics leaked"
        );
    }

    // ------------------------------------------------------------------
    // NEW-1 -- a DNS name resolves to whatever its owner says
    // ------------------------------------------------------------------

    #[test]
    fn import_seed_entries_rejects_dns_forms() {
        let (_dir, mgr) = manager();
        assert_eq!(
            mgr.import_seed_entries(vec![
                seed("/dns4/evil.example/tcp/80"),
                seed("/dns6/evil.example/tcp/80"),
                seed("/dns/evil.example/tcp/80"),
                seed("/dnsaddr/evil.example"),
                seed("/dns4/evil.example/tcp/443/p2p-circuit"),
            ]),
            0,
            "a DNS seed was imported; its owner can re-point it at 169.254.169.254"
        );
        assert!(mgr.seed_addresses(64).is_empty());
    }

    // ------------------------------------------------------------------
    // Round 4 -- the INGESTION choke point
    // ------------------------------------------------------------------

    /// F3/NEW-1, core half. The gate used to live in the callers, so a caller
    /// that forgot -- and `cli/src/main.rs:2034` did forget -- put a name into
    /// the ledger that later became a dial target chosen by its zone owner.
    ///
    /// `record_connection` is the only writer that produces `success_count > 0`,
    /// i.e. the only writer whose entries `exchange_response_entries`,
    /// `get_preferred_relays` and the seed-dial proven tier will use, so this is
    /// the door that has to be shut.
    #[test]
    fn record_connection_refuses_dns_forms_from_any_caller() {
        let (_dir, mgr) = manager();
        for addr in [
            "/dns4/evil.example/tcp/80",
            "/dns6/evil.example/tcp/80",
            "/dns/evil.example/tcp/80",
            "/dnsaddr/evil.example",
            "/dns4/evil.example/tcp/443/p2p-circuit",
            "/dns4/nas.corp.internal/tcp/443",
        ] {
            mgr.record_connection(addr.to_string(), peer());
        }
        assert!(
            mgr.dialable_addresses().is_empty(),
            "a DNS name was recorded as a PROVEN address: {:?}",
            mgr.dialable_addresses()
                .iter()
                .map(|e| e.multiaddr.as_str())
                .collect::<Vec<_>>()
        );
        assert!(mgr.get_preferred_relays(64).is_empty());
        assert!(mgr.export_seed_entries(64).is_empty());
        assert!(mgr.exchange_response_entries(64, "someone").is_empty());
    }

    /// F9 at the ingestion boundary: `"".parse::<Multiaddr>()` is `Ok(<empty>)`,
    /// so "it parsed" is not evidence of anything.
    #[test]
    fn record_connection_refuses_addresses_with_no_transport_component() {
        let (_dir, mgr) = manager();
        for addr in ["", "/p2p-circuit", "not-a-multiaddr"] {
            mgr.record_connection(addr.to_string(), peer());
        }
        mgr.record_connection(format!("/p2p/{}", peer()), peer());
        assert!(mgr.dialable_addresses().is_empty());
    }

    /// The gate must not become a routability filter: this method's meaning is
    /// "we actually reached this address", and the loopback/LAN evidence is what
    /// the DISCLOSURE gates are then tested against. If this ever starts
    /// rejecting RFC1918, `lan_only_node_discloses_nothing_to_a_stranger`
    /// becomes vacuous.
    #[test]
    fn record_connection_still_records_loopback_and_lan() {
        let (_dir, mgr) = manager();
        for addr in [
            "/ip4/127.0.0.1/tcp/8080",
            "/ip4/192.168.7.7/tcp/9001",
            "/ip4/10.1.2.3/tcp/9001",
            "/ip6/::1/tcp/8080",
        ] {
            mgr.record_connection(addr.to_string(), peer());
        }
        assert_eq!(mgr.dialable_addresses().len(), 4);
        // ...and none of them is disclosable.
        assert!(mgr.exchange_response_entries(64, "someone").is_empty());
    }

    /// `annotate_identity` is the SIBLING writer -- the wire-driven one. Gating
    /// `record_connection` and leaving this open is exactly the partial
    /// application the choke-point refactor exists to stop.
    #[test]
    fn annotate_identity_refuses_dns_forms_too() {
        let (_dir, mgr) = manager();
        let pid = peer();
        mgr.annotate_identity(
            "/dns4/evil.example/tcp/80".to_string(),
            pid.clone(),
            None,
            None,
        );
        assert!(
            mgr.seed_addresses(64).is_empty(),
            "a DNS name entered the ledger through annotate_identity"
        );
        // The IP form still works, so this is a filter and not an outage.
        mgr.annotate_identity("/ip4/198.51.100.8/tcp/9001".to_string(), pid, None, None);
        assert_eq!(mgr.seed_addresses(64).len(), 1);
    }

    // ------------------------------------------------------------------
    // F11 -- an in-memory core must not write topology to a temp dir
    // ------------------------------------------------------------------

    #[test]
    fn ephemeral_ledger_never_touches_the_filesystem() {
        let before: Vec<_> = std::fs::read_dir(std::env::temp_dir())
            .map(|rd| rd.flatten().map(|e| e.file_name()).collect())
            .unwrap_or_default();

        let mgr = LedgerManager::ephemeral();
        mgr.record_connection("/ip4/198.51.100.5/tcp/9001".to_string(), peer());
        assert_eq!(mgr.dialable_addresses().len(), 1);
        mgr.save().expect("ephemeral save is a no-op, not an error");
        mgr.load().expect("ephemeral load is a no-op, not an error");
        // The entry survives in memory across a load() (which must not clear).
        assert_eq!(mgr.dialable_addresses().len(), 1);

        let after: Vec<_> = std::fs::read_dir(std::env::temp_dir())
            .map(|rd| rd.flatten().map(|e| e.file_name()).collect())
            .unwrap_or_default();
        assert!(
            !after.iter().any(|n| n == "ledger.json") || before.iter().any(|n| n == "ledger.json"),
            "ephemeral ledger wrote ledger.json into the shared temp directory"
        );
    }

    #[test]
    fn ledger_caps_at_max_entries() {
        let (_dir, mut mgr) = manager();
        mgr.storage_path = None;

        let addr = |i: usize| {
            format!("/ip4/10.0.{}.{}/tcp/9001", i / 250, (i % 250) + 1)
        };
        let oldest_zero_addr = addr(0);
        let proven_addr = addr(10);

        let total = MAX_LEDGER_ENTRIES + 5;
        for i in 0..total {
            let multiaddr = addr(i);
            if i < 10 {
                mgr.annotate_identity(multiaddr, peer(), None, None);
            } else {
                mgr.record_connection(multiaddr, peer());
            }
        }

        let entries = mgr.entries.lock();
        assert_eq!(entries.len(), MAX_LEDGER_ENTRIES);
        assert!(
            !entries.iter().any(|e| e.multiaddr == oldest_zero_addr),
            "oldest zero-success entry should have been evicted"
        );
        assert!(
            entries.iter().any(|e| e.multiaddr == proven_addr && e.success_count > 0),
            "proven entry should survive"
        );
    }

    #[test]
    fn concurrent_mutations_persist_last_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();
        let mgr = Arc::new(LedgerManager::new(path.clone()));
        let mut handles = Vec::new();
        for thread_idx in 0..2u32 {
            let mgr = Arc::clone(&mgr);
            handles.push(std::thread::spawn(move || {
                for i in 0..50u32 {
                    let addr = format!("/ip4/198.51.{}.{}/tcp/9001", thread_idx + 100, i + 1);
                    let pid = peer();
                    if i % 2 == 0 {
                        mgr.record_connection(addr, pid);
                    } else {
                        mgr.annotate_identity(
                            addr,
                            pid,
                            Some(format!("pk-{}-{}", thread_idx, i)),
                            Some(format!("nick-{}-{}", thread_idx, i)),
                        );
                    }
                }
            }));
        }
        for handle in handles {
            handle.join().expect("worker thread");
        }

        let final_entries = mgr.entries.lock().clone();
        assert_eq!(final_entries.len(), 100);

        let reloaded = LedgerManager::new(path);
        reloaded.load().expect("reload");
        let reloaded_entries = reloaded.entries.lock().clone();
        assert_eq!(reloaded_entries.len(), final_entries.len());

        let reloaded_by_addr: std::collections::HashMap<String, LedgerEntry> = reloaded_entries
            .into_iter()
            .map(|e| (e.multiaddr.clone(), e))
            .collect();
        for entry in &final_entries {
            let loaded = reloaded_by_addr
                .get(&entry.multiaddr)
                .unwrap_or_else(|| panic!("missing {}", entry.multiaddr));
            assert_eq!(loaded.peer_id, entry.peer_id);
            assert_eq!(loaded.public_key, entry.public_key);
            assert_eq!(loaded.nickname, entry.nickname);
            assert_eq!(loaded.success_count, entry.success_count);
            assert_eq!(loaded.failure_count, entry.failure_count);
            assert_eq!(loaded.last_seen, entry.last_seen);
            assert_eq!(loaded.topics, entry.topics);
        }
    }

    #[test]
    fn save_is_always_parseable() {
        let (dir, mgr) = manager();
        let ledger_file = dir.path().join("ledger.json");
        for i in 0..20u32 {
            let addr = format!("/ip4/198.51.100.{}/tcp/9001", i + 1);
            if i % 2 == 0 {
                mgr.record_connection(addr, peer());
            } else {
                mgr.annotate_identity(addr, peer(), Some(format!("pk-{}", i)), None);
            }
            let data = std::fs::read_to_string(&ledger_file).expect("ledger file readable");
            let parsed: Vec<LedgerEntry> = serde_json::from_str(&data).expect("valid JSON");
            assert_eq!(parsed.len(), (i + 1) as usize);
            let tmp_prefix = "ledger.json.tmp.".to_string();
            let stale: Vec<_> = std::fs::read_dir(dir.path())
                .expect("read dir")
                .flatten()
                .filter(|e| e.file_name().to_str().map_or(false, |n| n.starts_with(&tmp_prefix)))
                .collect();
            assert!(stale.is_empty(), "tmp siblings remain: {:?}", stale);
        }
    }

    #[test]
    fn load_cleans_stale_tmp() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger_file = dir.path().join("ledger.json");
        let entries = vec![LedgerEntry {
            multiaddr: "/ip4/198.51.100.7/tcp/9001".to_string(),
            peer_id: Some(peer()),
            public_key: None,
            nickname: None,
            success_count: 1,
            failure_count: 0,
            last_seen: Some(42),
            topics: Vec::new(),
        }];
        std::fs::write(&ledger_file, serde_json::to_string_pretty(&entries).unwrap()).unwrap();
        let stale = dir.path().join("ledger.json.tmp.123.0");
        std::fs::write(&stale, "stale").unwrap();

        let mgr = LedgerManager::new(dir.path().to_string_lossy().to_string());
        mgr.load().expect("load");

        assert!(!stale.exists());
        assert_eq!(mgr.entries.lock().len(), 1);
    }

    #[test]
    fn load_sanitizes_legacy_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger_file = dir.path().join("ledger.json");
        let valid_one = "/ip4/198.51.100.20/tcp/9001".to_string();
        let valid_two = "/ip4/198.51.100.21/tcp/9001".to_string();
        let entries = vec![
            LedgerEntry {
                multiaddr: format!("/ip4/198.51.100.22/tcp/9001{}", "x".repeat(MAX_LEN_MULTIADDR)),
                peer_id: None,
                public_key: None,
                nickname: None,
                success_count: 1,
                failure_count: 0,
                last_seen: Some(1),
                topics: Vec::new(),
            },
            LedgerEntry {
                multiaddr: "/ip4/198.51.100.23/tcp/9001".to_string(),
                peer_id: Some("not-a-peer-id".to_string()),
                public_key: None,
                nickname: None,
                success_count: 1,
                failure_count: 0,
                last_seen: Some(2),
                topics: Vec::new(),
            },
            LedgerEntry {
                multiaddr: valid_one.clone(),
                peer_id: Some(peer()),
                public_key: None,
                nickname: None,
                success_count: 1,
                failure_count: 0,
                last_seen: Some(3),
                topics: Vec::new(),
            },
            LedgerEntry {
                multiaddr: valid_two.clone(),
                peer_id: None,
                public_key: None,
                nickname: None,
                success_count: 0,
                failure_count: 0,
                last_seen: None,
                topics: Vec::new(),
            },
        ];
        std::fs::write(&ledger_file, serde_json::to_string_pretty(&entries).unwrap()).unwrap();

        let mgr = LedgerManager::new(dir.path().to_string_lossy().to_string());
        mgr.load().expect("load");
        let loaded = mgr.entries.lock();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.iter().any(|e| e.multiaddr == valid_one));
        assert!(loaded.iter().any(|e| e.multiaddr == valid_two));
    }

    #[test]
    fn load_shrink_is_durable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger_file = dir.path().join("ledger.json");
        let mk = |i: usize| LedgerEntry {
            multiaddr: format!("/ip4/10.0.{}.{}/tcp/9001", i / 250, (i % 250) + 1),
            peer_id: None,
            public_key: None,
            nickname: None,
            success_count: 0,
            failure_count: 0,
            last_seen: Some(1000 + i as u64),
            topics: Vec::new(),
        };
        let entries: Vec<LedgerEntry> = (0..MAX_LEDGER_ENTRIES + 50).map(mk).collect();
        std::fs::write(&ledger_file, serde_json::to_string_pretty(&entries).unwrap()).unwrap();

        let path = dir.path().to_string_lossy().to_string();
        let mgr = LedgerManager::new(path.clone());
        mgr.load().expect("first load");
        assert_eq!(mgr.entries.lock().len(), MAX_LEDGER_ENTRIES);

        let on_disk: Vec<LedgerEntry> =
            serde_json::from_str(&std::fs::read_to_string(&ledger_file).unwrap()).unwrap();
        assert_eq!(on_disk.len(), MAX_LEDGER_ENTRIES);

        let reloaded = LedgerManager::new(path);
        reloaded.load().expect("second load");
        assert_eq!(reloaded.entries.lock().len(), MAX_LEDGER_ENTRIES);
    }
}
