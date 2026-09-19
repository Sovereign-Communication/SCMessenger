// Identity & Crypto - Foundation for KERI support (Phase 4)

pub mod keys;
mod store;

pub use keys::{
    identify_key_type, identity_id_from_public_key_hex, is_valid_identity_id, is_valid_public_key,
    sign_bundle, verify_bundle, IdentityKeys, KeyPair, PeerIdTriad, PublicKeyBundle,
    IDENTITY_ID_PREFIX, PUBLIC_KEY_PREFIX,
};
pub use store::{DeviceMetadata, IdentityStore};

use crate::dspy::signatures::{blake3_hash, get_signature, signature_fingerprint};
use crate::store::backend::StorageBackend;
use anyhow::Result;
use std::sync::Arc;
/// Manages node identity and cryptographic keys
pub struct IdentityManager {
    store: IdentityStore,
    keys: Option<IdentityKeys>,
    nickname: Option<String>,
    device_metadata: Option<DeviceMetadata>,
}

impl IdentityManager {
    /// Create a new identity manager with in-memory storage
    pub fn new() -> Self {
        Self {
            store: IdentityStore::memory(),
            keys: None,
            nickname: None,
            device_metadata: None,
        }
    }

    /// Create a new identity manager with persistent storage
    pub fn with_backend(backend: Arc<dyn StorageBackend>) -> Result<Self> {
        let mut manager = Self {
            store: IdentityStore::persistent(backend),
            keys: None,
            nickname: None,
            device_metadata: None,
        };

        tracing::debug!("IdentityManager::with_backend: Initializing with persistent storage");

        // Load any previously-persisted identity material without generating
        // a new identity. Fresh installs remain uninitialized.
        manager.hydrate_from_store()?;
        Ok(manager)
    }

    fn hydrate_from_store(&mut self) -> Result<()> {
        tracing::debug!("IdentityManager::hydrate_from_store: Loading from persistent store");
        if let Some(nickname) = self.store.load_nickname()? {
            tracing::debug!(
                "IdentityManager::hydrate_from_store: Loaded nickname from store: {:?}",
                nickname
            );
            self.nickname = Some(nickname);
        } else {
            tracing::debug!("IdentityManager::hydrate_from_store: No nickname found in store");
        }
        if let Some(keys) = self.store.load_keys()? {
            tracing::debug!("IdentityManager::hydrate_from_store: Loaded keys from store");
            self.keys = Some(keys);
        } else {
            tracing::debug!("IdentityManager::hydrate_from_store: No keys found in store");
        }
        self.device_metadata = self.store.load_device_metadata()?;
        tracing::debug!(
            "IdentityManager::hydrate_from_store: Loaded device_metadata={:?}",
            self.device_metadata
        );
        self.ensure_device_metadata()?;
        Ok(())
    }

    fn ensure_device_metadata(&mut self) -> Result<()> {
        if self.keys.is_some() && self.device_metadata.is_none() {
            self.device_metadata = Some(self.store.load_or_create_device_metadata()?);
        }
        Ok(())
    }

    /// Generate or load identity keys
    pub fn initialize(&mut self) -> Result<()> {
        self.hydrate_from_store()?;

        if self.keys.is_some() {
            tracing::info!("[OK] Loaded existing identity");
        } else {
            // Generate new keys
            tracing::info!("[OK] Generating new identity");
            let keys = IdentityKeys::generate();
            self.store.save_keys(&keys)?;
            self.keys = Some(keys);
        }

        self.ensure_device_metadata()?;

        Ok(())
    }

    /// Get identity keys (if initialized)
    pub fn keys(&self) -> Option<&IdentityKeys> {
        self.keys.as_ref()
    }

    /// Get identity public key as hex string
    pub fn public_key_hex(&self) -> Option<String> {
        self.keys.as_ref().map(|k| k.public_key_hex())
    }

    /// Get identity ID (hash of public key)
    pub fn identity_id(&self) -> Option<String> {
        self.keys.as_ref().map(|k| k.identity_id())
    }

    /// Sign data with identity key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.keys {
            Some(keys) => keys.sign(data),
            None => Err(anyhow::anyhow!("Identity not initialized")),
        }
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        IdentityKeys::verify(data, signature, public_key)
    }

    /// Set nickname
    pub fn set_nickname(&mut self, nickname: String) -> Result<()> {
        self.store.save_nickname(&nickname)?;
        self.nickname = Some(nickname);
        Ok(())
    }

    /// Get nickname
    pub fn nickname(&self) -> Option<String> {
        self.nickname.clone()
    }

    /// Get installation-local device metadata for tight-pair routing.
    pub fn device_id(&self) -> Option<String> {
        self.device_metadata
            .as_ref()
            .map(|metadata| metadata.device_id.clone())
    }

    /// Get the activation timestamp for this installation instance.
    pub fn seniority_timestamp(&self) -> Option<u64> {
        self.device_metadata
            .as_ref()
            .map(|metadata| metadata.seniority_timestamp)
    }

    /// Export raw identity key bytes for secure platform backup.
    pub fn export_key_bytes(&self) -> Option<Vec<u8>> {
        self.keys.as_ref().map(|keys| keys.to_bytes())
    }

    /// Import raw identity key bytes and persist them in the configured store.
    pub fn import_key_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let keys = IdentityKeys::from_bytes(bytes)?;
        self.store.save_keys(&keys)?;
        self.keys = Some(keys);
        self.ensure_device_metadata()?;
        Ok(())
    }

    /// Get DSPy signature description for identity-related roles.
    /// Used for signature verification schema lookup in authentication flows.
    ///
    /// # Arguments
    /// * `role` - The DSPy role name (e.g., "architect", "coder", "verifier", "auditor")
    ///
    /// # Returns
    /// The signature description if the role is known, `None` otherwise.
    pub fn get_dspy_signature(&self, role: &str) -> Option<String> {
        get_signature(role).map(|s| s.to_string())
    }

    /// Compute a BLAKE3 fingerprint of a signature for content-addressable identification.
    /// Used in identity authentication flows to verify signature integrity.
    ///
    /// # Arguments
    /// * `data` - The data to compute fingerprint for (typically a serialized signature)
    ///
    /// # Returns
    /// A hex-encoded string of the BLAKE3 hash.
    pub fn signature_fingerprint(&self, data: &[u8]) -> String {
        signature_fingerprint(data)
    }

    /// Compute a BLAKE3 hash of data.
    /// Used for signature verification and integrity checks in identity authentication.
    ///
    /// # Arguments
    /// * `data` - The data to hash
    ///
    /// # Returns
    /// A 32-byte array containing the BLAKE3 hash.
    pub fn blake3_hash(&self, data: &[u8]) -> [u8; 32] {
        blake3_hash(data)
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_manager_creation() {
        let manager = IdentityManager::new();
        assert!(manager.keys().is_none());
    }

    #[test]
    fn test_identity_initialization() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        assert!(manager.keys().is_some());
        assert!(manager.public_key_hex().is_some());
        assert!(manager.identity_id().is_some());
        let device_id = manager.device_id().unwrap();
        let parsed_uuid = uuid::Uuid::parse_str(&device_id).unwrap();
        assert_eq!(parsed_uuid.get_version_num(), 4);
        assert!(manager.seniority_timestamp().unwrap() > 0);
    }

    #[test]
    fn test_identity_signing() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        let data = b"test message";
        let signature = manager.sign(data).unwrap();

        assert!(!signature.is_empty());
    }

    #[test]
    fn test_identity_verification() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        let data = b"test message";
        let signature = manager.sign(data).unwrap();

        let keys = manager.keys().unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();

        let valid = manager.verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        // Test invalid signature
        let invalid = manager
            .verify(b"wrong data", &signature, &public_key)
            .unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_identity_persistence() {
        use std::thread;
        use tempfile::tempdir;
        use web_time::Duration;

        const MAX_REOPEN_ATTEMPTS: u64 = 10;
        const REOPEN_BACKOFF_BASE_MS: u64 = 25;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("test_identity")
            .to_str()
            .unwrap()
            .to_string();

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let mut manager1 = IdentityManager::with_backend(backend).unwrap();
        manager1.initialize().unwrap();
        manager1.set_nickname("Alice".to_string()).unwrap();
        let id1 = manager1.identity_id().unwrap();
        let device_id1 = manager1.device_id();
        let seniority1 = manager1.seniority_timestamp();

        drop(manager1);

        let backend2 = Arc::new(
            (0..MAX_REOPEN_ATTEMPTS)
                .find_map(
                    |attempt| match crate::store::backend::SledStorage::new(&path) {
                        Ok(storage) => Some(storage),
                        Err(_) if attempt + 1 < MAX_REOPEN_ATTEMPTS => {
                            thread::sleep(Duration::from_millis(
                                REOPEN_BACKOFF_BASE_MS * (attempt + 1),
                            ));
                            None
                        }
                        Err(err) => panic!("failed to reopen sled identity store: {err}"),
                    },
                )
                .unwrap(),
        );
        let mut manager2 = IdentityManager::with_backend(backend2).unwrap();
        manager2.initialize().unwrap();
        let id2 = manager2.identity_id().unwrap();
        let nick2 = manager2.nickname();
        let device_id2 = manager2.device_id();
        let seniority2 = manager2.seniority_timestamp();

        assert_eq!(id1, id2);
        assert_eq!(nick2, Some("Alice".to_string()));
        assert_eq!(device_id2, device_id1);
        assert_eq!(seniority2, seniority1);
    }

    #[test]
    fn test_identity_import_export_roundtrip() {
        let mut manager1 = IdentityManager::new();
        manager1.initialize().unwrap();
        let exported = manager1.export_key_bytes().unwrap();
        let original_id = manager1.identity_id();
        let original_pub = manager1.public_key_hex();
        let original_device_id = manager1.device_id();

        let mut manager2 = IdentityManager::new();
        manager2.import_key_bytes(&exported).unwrap();

        assert_eq!(manager2.identity_id(), original_id);
        assert_eq!(manager2.public_key_hex(), original_pub);
        assert_ne!(manager2.device_id(), original_device_id);
        assert!(manager2.seniority_timestamp().is_some());
    }

    #[test]
    fn test_with_path_hydrates_existing_identity_without_initialize() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("existing_identity")
            .to_str()
            .unwrap()
            .to_string();

        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let mut manager = IdentityManager::with_backend(backend).unwrap();
            manager.initialize().unwrap();
            manager.set_nickname("PersistedNick".to_string()).unwrap();
        }

        let backend2 = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let manager = IdentityManager::with_backend(backend2).unwrap();
        assert!(manager.keys().is_some());
        assert_eq!(manager.nickname(), Some("PersistedNick".to_string()));
        assert!(manager.device_id().is_some());
        assert!(manager.seniority_timestamp().is_some());
    }

    #[test]
    fn test_get_dspy_signature() {
        let manager = IdentityManager::new();

        // Test all valid DSPy roles
        assert!(manager.get_dspy_signature("architect").is_some());
        assert!(manager.get_dspy_signature("coder").is_some());
        assert!(manager.get_dspy_signature("verifier").is_some());
        assert!(manager.get_dspy_signature("auditor").is_some());

        // Test invalid role
        assert!(manager.get_dspy_signature("invalid_role").is_none());
    }

    #[test]
    fn test_get_dspy_signature_content() {
        let manager = IdentityManager::new();

        // Verify the signature content matches expected descriptions
        let architect = manager.get_dspy_signature("architect").unwrap();
        assert!(architect.contains("System design"));

        let coder = manager.get_dspy_signature("coder").unwrap();
        assert!(coder.contains("Rust code generation"));
    }

    #[test]
    fn test_signature_fingerprint() {
        let manager = IdentityManager::new();

        // Test fingerprint generation
        let data = b"test signature data";
        let fp = manager.signature_fingerprint(data);

        // Fingerprint should be 64 hex characters (32 bytes)
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blake3_hash() {
        let manager = IdentityManager::new();

        // Test hash is deterministic
        let data = b"test data";
        let h1 = manager.blake3_hash(data);
        let h2 = manager.blake3_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);

        // Test different inputs produce different hashes
        let h3 = manager.blake3_hash(b"different data");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_identity_signature_integration() {
        let mut manager = IdentityManager::new();
        manager.initialize().unwrap();

        // Sign some data
        let data = b"identity authentication test";
        let signature = manager.sign(data).unwrap();

        // Verify the signature using identity's verify method
        let keys = manager.keys().unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();
        let valid = manager.verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        // Test fingerprint of signature
        let fp = manager.signature_fingerprint(&signature);
        assert_eq!(fp.len(), 64);
    }
}
