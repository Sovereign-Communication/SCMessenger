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

/// In-memory storage useful for testing and temporary execution
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

/// Sled-backed persistent storage (desktop/mobile)
#[cfg(not(target_arch = "wasm32"))]
pub struct SledStorage {
    db: sled::Db,
}

#[cfg(not(target_arch = "wasm32"))]
impl SledStorage {
    pub fn new(path: &str) -> std::result::Result<Self, String> {
        let db = sled::Config::default()
            .path(path)
            .mode(sled::Mode::LowSpace)
            .use_compression(false)
            .open()
            .map_err(|e| e.to_string())?;
        Ok(Self { db })
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