// Configuration management for SCMessenger CLI
//
// Cross-platform config stored in:
// - macOS: ~/.config/scmessenger/config.toml
// - Linux: ~/.config/scmessenger/config.toml
// - Windows: %APPDATA%\scmessenger\config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default port for listening
    #[serde(default)]
    pub listen_port: u16,

    /// Enable mDNS for local network discovery
    #[serde(alias = "mdns", default)]
    pub enable_mdns: bool,

    /// Enable BLE for discovery
    #[serde(default)]
    pub enable_ble: bool,

    /// Enable WiFi-Aware for discovery
    #[serde(default)]
    pub enable_wifi_aware: bool,

    /// Enable DHT for wide area network discovery
    #[serde(default)]
    pub enable_dht: bool,

    /// Storage path for messages and identity
    #[serde(default)]
    pub storage_path: Option<String>,

    /// Network settings
    #[serde(default)]
    pub network: NetworkConfig,

    /// User-configured bootstrap nodes (community ledger)
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Maximum number of peers to maintain
    #[serde(default)]
    pub max_peers: usize,

    /// Connection timeout in seconds
    #[serde(default)]
    pub connection_timeout: u64,

    /// Enable NAT traversal
    #[serde(default)]
    pub enable_nat_traversal: bool,

    /// Enable relay fallback
    #[serde(default)]
    pub enable_relay: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_port: 9000, // Default to 9000 instead of random
            enable_mdns: true,
            enable_ble: true,
            enable_wifi_aware: true,
            enable_dht: true,
            storage_path: None,
            network: NetworkConfig::default(),
            bootstrap_nodes: Vec::new(), // No hardcoded bootstrap nodes (community ledger)
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            connection_timeout: 30,
            enable_nat_traversal: true,
            enable_relay: true,
        }
    }
}

impl Config {
    /// Get the config directory path (cross-platform)
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("scmessenger");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        Ok(config_dir)
    }

    /// Get the data directory path (cross-platform).
    /// Honors SCMESSENGER_DATA_DIR env var (absolute path) for running isolated
    /// instances side by side, e.g. local multi-node discovery testing.
    pub fn data_dir() -> Result<PathBuf> {
        if let Ok(env_path) = std::env::var("SCMESSENGER_DATA_DIR") {
            let path = PathBuf::from(env_path);
            std::fs::create_dir_all(&path).context("Failed to create data directory")?;
            return Ok(path);
        }

        let data_dir = dirs::data_local_dir()
            .context("Failed to determine data directory")?
            .join("scmessenger");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        Ok(data_dir)
    }

    /// Get the config file path
    /// Honors SCMESSENGER_CONFIG env var (absolute path to config file).
    /// Falls back to config_dir/config.json if not set.
    pub fn config_file() -> Result<PathBuf> {
        if let Ok(env_path) = std::env::var("SCMESSENGER_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Ok(path);
            }
            // If env var set but file missing, still use it (will create default there)
            return Ok(path);
        }
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if config_file.exists() {
            let contents =
                std::fs::read_to_string(&config_file).context("Failed to read config file")?;
            let config: Config =
                serde_json::from_str(&contents).context("Failed to parse config file")?;

            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let contents = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_file, contents).context("Failed to write config file")?;
        Ok(())
    }

    /// Set a config value
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "listen_port" => {
                self.listen_port = value.parse().context("Invalid port number")?;
            }
            "enable_mdns" => {
                self.enable_mdns = value.parse().context("Invalid boolean value")?;
            }
            "enable_ble" => {
                self.enable_ble = value.parse().context("Invalid boolean value")?;
            }
            "enable_wifi_aware" => {
                self.enable_wifi_aware = value.parse().context("Invalid boolean value")?;
            }
            "enable_dht" => {
                self.enable_dht = value.parse().context("Invalid boolean value")?;
            }
            "storage_path" => {
                self.storage_path = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "max_peers" => {
                self.network.max_peers = value.parse().context("Invalid number")?;
            }
            "connection_timeout" => {
                self.network.connection_timeout = value.parse().context("Invalid number")?;
            }
            "enable_nat_traversal" => {
                self.network.enable_nat_traversal =
                    value.parse().context("Invalid boolean value")?;
            }
            "enable_relay" => {
                self.network.enable_relay = value.parse().context("Invalid boolean value")?;
            }
            "bootstrap_node_add" => {
                if !value.is_empty() {
                    self.bootstrap_nodes.push(value.to_string());
                }
            }
            "bootstrap_node_remove" => {
                if !value.is_empty() {
                    self.bootstrap_nodes.retain(|n| n != value);
                }
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
        self.save()?;
        Ok(())
    }

    /// Get a config value
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "listen_port" => Some(self.listen_port.to_string()),
            "enable_mdns" => Some(self.enable_mdns.to_string()),
            "enable_ble" => Some(self.enable_ble.to_string()),
            "enable_wifi_aware" => Some(self.enable_wifi_aware.to_string()),
            "enable_dht" => Some(self.enable_dht.to_string()),
            "storage_path" => self.storage_path.clone(),
            "max_peers" => Some(self.network.max_peers.to_string()),
            "connection_timeout" => Some(self.network.connection_timeout.to_string()),
            "enable_nat_traversal" => Some(self.network.enable_nat_traversal.to_string()),
            "enable_relay" => Some(self.network.enable_relay.to_string()),
            "bootstrap_nodes" => Some(self.bootstrap_nodes.join(",")),
            _ => None,
        }
    }

    /// List all config values
    pub fn list(&self) -> Vec<(String, String)> {
        vec![
            ("listen_port".to_string(), self.listen_port.to_string()),
            ("enable_mdns".to_string(), self.enable_mdns.to_string()),
            ("enable_ble".to_string(), self.enable_ble.to_string()),
            (
                "enable_wifi_aware".to_string(),
                self.enable_wifi_aware.to_string(),
            ),
            ("enable_dht".to_string(), self.enable_dht.to_string()),
            (
                "storage_path".to_string(),
                self.storage_path
                    .clone()
                    .unwrap_or_else(|| "(auto)".to_string()),
            ),
            ("max_peers".to_string(), self.network.max_peers.to_string()),
            (
                "connection_timeout".to_string(),
                format!("{}s", self.network.connection_timeout),
            ),
            (
                "enable_nat_traversal".to_string(),
                self.network.enable_nat_traversal.to_string(),
            ),
            (
                "enable_relay".to_string(),
                self.network.enable_relay.to_string(),
            ),
            (
                "bootstrap_nodes".to_string(),
                self.bootstrap_nodes.join(","),
            ),
        ]
    }

    /// Helper to strip /p2p/PeerID suffix from a multiaddr string
    fn strip_peer_id(multiaddr: &str) -> String {
        if let Some(idx) = multiaddr.find("/p2p/") {
            multiaddr[..idx].to_string()
        } else {
            multiaddr.to_string()
        }
    }

    /// Add a bootstrap node to the config
    pub fn add_bootstrap_node(&mut self, multiaddr: String) -> Result<()> {
        // Check for duplicates by IP:Port only (strip PeerID)
        let stripped = Self::strip_peer_id(&multiaddr);
        if self
            .bootstrap_nodes
            .iter()
            .any(|n| Self::strip_peer_id(n) == stripped)
        {
            anyhow::bail!("Bootstrap node already exists");
        }
        self.bootstrap_nodes.push(multiaddr);
        self.save()?;
        Ok(())
    }

    /// Remove a bootstrap node from the config
    pub fn remove_bootstrap_node(&mut self, multiaddr: &str) -> Result<()> {
        let stripped = Self::strip_peer_id(multiaddr);
        let removed_count = self
            .bootstrap_nodes
            .iter()
            .filter(|n| Self::strip_peer_id(n) == stripped)
            .count();
        if removed_count == 0 {
            anyhow::bail!("Bootstrap node not found");
        }
        self.bootstrap_nodes
            .retain(|n| Self::strip_peer_id(n) != stripped);
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.listen_port, 9000);
        assert!(config.enable_mdns);
        assert!(config.enable_dht);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.listen_port, deserialized.listen_port);
    }
}
