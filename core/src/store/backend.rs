// Storage abstraction for cross-platform persistence

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ScanResult = Vec<(Vec<u8>, Vec<u8>)>;

/// Unified storage trait for cross-platform data persistence
pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String>;
    fn remove(&self, key: &[u8]) -> Result<(), String>;
    fn scan_prefix(&self, prefix: &[u8]) -> Result<ScanResult, String>;
    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String>;
    fn flush(&self) -> Result<(), String>;
    fn approximate_size(&self) -> Result<u64, String>;
}

/// In-memory storage useful for testing and temporary WASM execution
#[derive(Clone)]
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let guard = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?;
        Ok(guard.get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .remove(key);
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<ScanResult, String> {
        let mut results = Vec::new();
        for (key, value) in self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .iter()
        {
            if key.starts_with(prefix) {
                results.push((key.clone(), value.clone()));
            }
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        let count = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .keys()
            .filter(|k| k.starts_with(prefix))
            .count();
        Ok(count)
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }

    fn approximate_size(&self) -> Result<u64, String> {
        let data = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?;
        let size: u64 = data.iter().map(|(k, v)| (k.len() + v.len()) as u64).sum();
        Ok(size)
    }
}

/// Degraded storage backend representing a failed persistent storage open.
///
/// Every operation on this backend returns an explicit storage error containing
/// the reason the underlying store failed to open. This ensures that security
/// controls (such as blocked-list checks) and data pipelines fail closed rather
/// than silently operating against empty in-memory state.
#[derive(Clone, Debug)]
pub struct DegradedStorage {
    path: String,
    error: String,
}

impl DegradedStorage {
    pub fn new(path: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            error: error.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

impl StorageBackend for DegradedStorage {
    fn put(&self, _key: &[u8], _value: &[u8]) -> Result<(), String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn get(&self, _key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn remove(&self, _key: &[u8]) -> Result<(), String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn scan_prefix(&self, _prefix: &[u8]) -> Result<ScanResult, String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn count_prefix(&self, _prefix: &[u8]) -> Result<usize, String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn flush(&self) -> Result<(), String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }

    fn approximate_size(&self) -> Result<u64, String> {
        Err(format!(
            "storage degraded for path '{}': {}",
            self.path, self.error
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_lock_contention(err: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    if err.kind() == ErrorKind::WouldBlock {
        return true;
    }
    #[cfg(windows)]
    {
        if let Some(raw_os_error) = err.raw_os_error() {
            // 32 = ERROR_SHARING_VIOLATION, 33 = ERROR_LOCK_VIOLATION
            if raw_os_error == 32 || raw_os_error == 33 {
                return true;
            }
        }
    }
    let msg = err.to_string().to_lowercase();
    msg.contains("lock")
        || msg.contains("sharing violation")
        || msg.contains("resource temporarily unavailable")
        || msg.contains("would block")
        || msg.contains("being used by another process")
}

#[cfg(not(target_arch = "wasm32"))]
fn open_with_lock_retry<F>(mut open: F) -> std::result::Result<sled::Db, (u32, sled::Error)>
where
    F: FnMut() -> sled::Result<sled::Db>,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match open() {
            Ok(db) => return Ok(db),
            Err(error)
                if matches!(&error, sled::Error::Io(io_err) if is_lock_contention(io_err))
                    && attempt < SledStorage::LOCK_MAX_OPEN_ATTEMPTS =>
            {
                std::thread::sleep(SledStorage::LOCK_RETRY_DELAY);
            }
            Err(error) => return Err((attempt, error)),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::is_lock_contention;
    use std::io::{Error, ErrorKind};

    #[test]
    fn retry_control_flow_allows_nine_retries_before_tenth_open() {
        let mut failures = 9;
        let (opens, retries) = super::open_with_lock_retry(|| {
            if failures == 0 {
                Ok(sled::Config::default().temporary(true).open().unwrap())
            } else {
                failures -= 1;
                Err(sled::Error::Io(Error::new(ErrorKind::WouldBlock, "busy")))
            }
        })
        .map(|_| (10, 9))
        .unwrap();
        assert_eq!((opens, retries), (10, 9));

        let result = super::open_with_lock_retry(|| {
            Err(sled::Error::Io(Error::new(ErrorKind::WouldBlock, "held")))
        });
        assert_eq!(result.unwrap_err().0, 10);
    }

    #[test]
    fn classifier_separates_lock_and_non_lock_errors() {
        assert!(is_lock_contention(&Error::new(
            ErrorKind::WouldBlock,
            "busy"
        )));
        assert!(is_lock_contention(&Error::new(
            ErrorKind::Other,
            "sharing violation"
        )));
        assert!(!is_lock_contention(&Error::new(
            ErrorKind::PermissionDenied,
            "permission denied"
        )));
        assert!(!is_lock_contention(&Error::new(
            ErrorKind::Other,
            "permission denied"
        )));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct SledStorage {
    db: sled::Db,
}

#[cfg(not(target_arch = "wasm32"))]
impl SledStorage {
    /// Maximum number of open attempts, including the first attempt.
    const LOCK_MAX_OPEN_ATTEMPTS: u32 = 10;
    const LOCK_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

    pub fn new(path: &str) -> std::result::Result<Self, String> {
        match open_with_lock_retry(|| {
            sled::Config::default()
                .path(path)
                .mode(sled::Mode::LowSpace)
                .use_compression(false)
                .open()
        }) {
            Ok(db) => Ok(Self { db }),
            Err((attempt, e)) => Err(match e {
                sled::Error::Corruption { at, .. } => {
                    format!("corruption detected at {:?}: {}", at, e)
                }
                sled::Error::Io(ref io_err) if is_lock_contention(io_err) => {
                    format!(
                        "database locked by another process (lock contention) after {} open attempts: {}",
                        attempt, io_err
                    )
                }
                sled::Error::Io(ref io_err) => format!("io error: {}", io_err),
                _ => e.to_string(),
            }),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageBackend for SledStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.db.insert(key, value).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let value = self.db.get(key).map_err(|e| e.to_string())?;
        Ok(value.map(|ivec| ivec.to_vec()))
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.db.remove(key).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<ScanResult, String> {
        let mut results = Vec::new();
        for item in self.db.scan_prefix(prefix) {
            let (k, v) = item.map_err(|e| e.to_string())?;
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        Ok(self.db.scan_prefix(prefix).count())
    }

    fn flush(&self) -> Result<(), String> {
        self.db.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn approximate_size(&self) -> Result<u64, String> {
        self.db.size_on_disk().map_err(|e| e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

#[cfg(target_arch = "wasm32")]
impl IndexedDbStorage {
    pub async fn new(db_name: &str) -> std::result::Result<Self, String> {
        use js_sys::wasm_bindgen::JsCast;
        use rexie::*;
        let store_name = "scmessenger_store";

        let rexie = Rexie::builder(db_name)
            .version(1)
            .add_object_store(ObjectStore::new(store_name))
            .build()
            .await
            .map_err(|e| e.to_string())?;

        let data = Arc::new(RwLock::new(HashMap::new()));

        let transaction = rexie
            .transaction(&[store_name], TransactionMode::ReadOnly)
            .map_err(|e| e.to_string())?;
        let store = transaction.store(store_name).map_err(|e| e.to_string())?;

        let all_keys_js = store
            .get_all_keys(None, None)
            .await
            .map_err(|e| format!("{:?}", e))?;

        let mut entries = Vec::new();
        for key_js in all_keys_js {
            if let Ok(Some(value_js)) = store.get(key_js.clone()).await {
                if value_js.is_instance_of::<js_sys::Uint8Array>() {
                    let key_arr = js_sys::Uint8Array::new(&key_js);
                    let val_arr = js_sys::Uint8Array::new(&value_js);
                    entries.push((key_arr.to_vec(), val_arr.to_vec()));
                }
            }
        }
        let mut map = data
            .write()
            .map_err(|e| format!("storage lock poisoned: {}", e))?;
        for (key, value) in entries {
            map.insert(key, value);
        }

        Ok(Self {
            db_name: db_name.to_string(),
            store_name: store_name.to_string(),
            data: data.clone(),
        })
    }

    pub fn new_sync(db_name: &str) -> std::result::Result<Self, String> {
        use futures::channel::oneshot;
        use futures::executor::block_on;
        use wasm_bindgen_futures::spawn_local;

        let (sender, receiver) = oneshot::channel();
        let db_name = db_name.to_string();
        spawn_local(async move {
            match Self::new(&db_name).await {
                Ok(storage) => {
                    let _ = sender.send(Ok(storage));
                }
                Err(e) => {
                    let _ = sender.send(Err(e));
                }
            }
        });
        block_on(receiver).map_err(|e| format!("new_sync: oneshot channel dropped: {}", e))?
    }

    fn persist_put(&self, key: Vec<u8>, value: Vec<u8>) {
        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(rexie) = rexie::Rexie::builder(&db_name).version(1).build().await {
                if let Ok(tx) = rexie.transaction(&[&store_name], rexie::TransactionMode::ReadWrite)
                {
                    if let Ok(store) = tx.store(&store_name) {
                        let key_js = js_sys::Uint8Array::from(key.as_slice());
                        let value_js = js_sys::Uint8Array::from(value.as_slice());
                        // idb / rexie put
                        let _ = store.put(&value_js, Some(&key_js)).await;
                    }
                    let _ = tx.done().await;
                }
            }
        });
    }

    fn persist_remove(&self, key: Vec<u8>) {
        let db_name = self.db_name.clone();
        let store_name = self.store_name.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(rexie) = rexie::Rexie::builder(&db_name).version(1).build().await {
                if let Ok(tx) = rexie.transaction(&[&store_name], rexie::TransactionMode::ReadWrite)
                {
                    if let Ok(store) = tx.store(&store_name) {
                        let key_js = js_sys::Uint8Array::from(key.as_slice());
                        let _ = store.delete((&key_js).into()).await;
                    }
                    let _ = tx.done().await;
                }
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
impl StorageBackend for IndexedDbStorage {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .insert(key.to_vec(), value.to_vec());
        self.persist_put(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let guard = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?;
        Ok(guard.get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<(), String> {
        self.data
            .write()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .remove(key);
        self.persist_remove(key.to_vec());
        Ok(())
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Result<ScanResult, String> {
        let mut results = Vec::new();
        for (k, v) in self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .iter()
        {
            if k.starts_with(prefix) {
                results.push((k.clone(), v.clone()));
            }
        }
        Ok(results)
    }

    fn count_prefix(&self, prefix: &[u8]) -> Result<usize, String> {
        let count = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?
            .keys()
            .filter(|k| k.starts_with(prefix))
            .count();
        Ok(count)
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }

    fn approximate_size(&self) -> Result<u64, String> {
        let data = self
            .data
            .read()
            .map_err(|e| format!("storage lock poisoned: {}", e))?;
        let size: u64 = data.iter().map(|(k, v)| (k.len() + v.len()) as u64).sum();
        Ok(size)
    }
}
