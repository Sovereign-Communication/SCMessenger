//! Best-effort Bluetooth adapter discovery via btleplug (desktop CLI only).
//! Full GATT advertising/scanning and Drift→RPC proxy are follow-on work.

use btleplug::api::{Central, CentralState, Manager as _};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;

fn classify_adapter_error(operation: &str, error: impl std::fmt::Display) -> BleError {
    let reason = error.to_string().to_lowercase();
    if reason.contains("access denied") || reason.contains("permission") {
        BleError::PermissionDenied
    } else if reason.contains("not found") || reason.contains("no device") {
        BleError::NoAdapter
    } else {
        BleError::Other(format!("{} failed", operation))
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
async fn new_manager() -> BleResult<btleplug::platform::Manager> {
    match AssertUnwindSafe(btleplug::platform::Manager::new())
        .catch_unwind()
        .await
    {
        Ok(Ok(manager)) => Ok(manager),
        Ok(Err(_)) => Err(BleError::ManagerInitFailed(
            "btleplug manager initialization failed".to_string(),
        )),
        Err(_) => Err(BleError::ManagerInitFailed(
            "btleplug manager initialization panicked".to_string(),
        )),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
async fn list_adapters(
    manager: &btleplug::platform::Manager,
) -> BleResult<Vec<btleplug::platform::Adapter>> {
    match AssertUnwindSafe(manager.adapters()).catch_unwind().await {
        Ok(Ok(adapters)) => Ok(adapters),
        Ok(Err(error)) => Err(classify_adapter_error(
            "btleplug adapter enumeration",
            error,
        )),
        Err(_) => Err(BleError::Other(
            "btleplug adapter enumeration panicked".to_string(),
        )),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
async fn inspect_adapters(
    adapters: &[btleplug::platform::Adapter],
) -> BleResult<Vec<BleAdapterInfo>> {
    let mut infos = Vec::with_capacity(adapters.len());
    for adapter in adapters {
        let mut state = match AssertUnwindSafe(adapter.adapter_state())
            .catch_unwind()
            .await
        {
            Ok(Ok(state)) => state,
            Ok(Err(error)) => {
                return Err(classify_adapter_error(
                    "btleplug adapter state probe",
                    error,
                ));
            }
            Err(_) => {
                return Err(BleError::Other(
                    "btleplug adapter state probe panicked".to_string(),
                ));
            }
        };

        // CoreBluetooth on macOS initializes asynchronously; give it up to 500ms to report PoweredOn
        if state != CentralState::PoweredOn {
            for _ in 0..5 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                if let Ok(Ok(s)) = AssertUnwindSafe(adapter.adapter_state()).catch_unwind().await {
                    state = s;
                    if state == CentralState::PoweredOn {
                        break;
                    }
                }
            }
        }

        infos.push(BleAdapterInfo {
            // btleplug does not expose a stable cross-platform address/name here.
            name: None,
            address: None,
            is_powered: state == CentralState::PoweredOn,
            // An adapter returned by btleplug implements the BLE Central API. This
            // does not imply that the adapter can advertise as a peripheral.
            supports_le: true,
        });
    }

    if infos.iter().any(|info| info.is_powered) {
        Ok(infos)
    } else {
        Err(BleError::AdapterNotPowered)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
async fn probe_status() -> BleStatus {
    let manager = match new_manager().await {
        Ok(manager) => manager,
        Err(error) => return BleStatus::Unavailable(error),
    };
    let adapters = match list_adapters(&manager).await {
        Ok(adapters) => adapters,
        Err(error) => return BleStatus::Unavailable(error),
    };
    if adapters.is_empty() {
        return BleStatus::Unavailable(BleError::NoAdapter);
    }

    match inspect_adapters(&adapters).await {
        Ok(infos) => BleStatus::Available(infos),
        Err(error) => BleStatus::Unavailable(error),
    }
}

/// Probe and log the local Bluetooth stack without allowing a btleplug panic to
/// escape into the CLI task. A visible adapter is not treated as peripheral
/// advertising support.
pub async fn probe_and_log() -> BleStatus {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        let status = probe_status().await;
        match &status {
            BleStatus::Available(adapters) => {
                tracing::info!(
                    route = "ble_probe",
                    adapter_count = adapters.len(),
                    terminal_result = "available",
                    "btleplug BLE adapter probe"
                );
            }
            BleStatus::Unavailable(error) => {
                tracing::warn!(
                    route = "ble_probe",
                    terminal_result = "unavailable",
                    error = %error,
                    "btleplug BLE adapter probe"
                );
            }
            BleStatus::Disabled => {
                tracing::warn!(
                    route = "ble_probe",
                    terminal_result = "disabled",
                    "btleplug BLE adapter probe"
                );
            }
        }
        status
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let status = BleStatus::Unavailable(BleError::Other(
            "BLE not supported on this platform".to_string(),
        ));
        tracing::debug!(
            route = "ble_probe",
            terminal_result = "unavailable",
            error = ?status,
            "btleplug BLE adapter probe"
        );
        status
    }
}

/// BLE daemon error types for graceful handling.
#[derive(Debug, Clone, PartialEq)]
pub enum BleError {
    /// No Bluetooth adapter present on the system
    NoAdapter,
    /// Permission denied accessing Bluetooth (common on Windows)
    PermissionDenied,
    /// Bluetooth adapter not powered on
    AdapterNotPowered,
    /// Failed to initialize BLE manager
    ManagerInitFailed(String),
    /// Operation timed out
    Timeout,
    /// Generic BLE error
    Other(String),
}

impl std::fmt::Display for BleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BleError::NoAdapter => write!(f, "No Bluetooth adapter found"),
            BleError::PermissionDenied => write!(f, "Bluetooth permission denied"),
            BleError::AdapterNotPowered => write!(f, "Bluetooth adapter not powered on"),
            BleError::ManagerInitFailed(e) => write!(f, "Failed to initialize BLE manager: {}", e),
            BleError::Timeout => write!(f, "BLE operation timed out"),
            BleError::Other(e) => write!(f, "BLE error: {}", e),
        }
    }
}

impl std::error::Error for BleError {}

/// Result type for BLE operations
pub type BleResult<T> = Result<T, BleError>;

/// BLE daemon status
#[derive(Debug, Clone, PartialEq)]
pub enum BleStatus {
    /// At least one powered-on btleplug BLE Central adapter was verified.
    /// This does not imply peripheral advertising support.
    Available(Vec<BleAdapterInfo>),
    /// BLE is unavailable but can be retried
    Unavailable(BleError),
    /// BLE is disabled by user/system settings
    Disabled,
}

/// Information about a detected BLE adapter
#[derive(Debug, Clone, PartialEq)]
pub struct BleAdapterInfo {
    pub name: Option<String>,
    pub address: Option<String>,
    pub is_powered: bool,
    pub supports_le: bool,
}

/// BLE daemon configuration
#[derive(Debug, Clone)]
pub struct BleConfig {
    pub scan_interval_ms: u64,
    pub advertisement_timeout_ms: u64,
    pub max_retry_attempts: u32,
    pub fallback_mode: bool,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            scan_interval_ms: 1000,
            advertisement_timeout_ms: 5000,
            max_retry_attempts: 3,
            fallback_mode: false,
        }
    }
}

/// BLE daemon for Windows CLI with graceful error handling.
/// Not yet constructed/wired outside this module - Windows BLE integration
/// is still pending compared to Android's BLE transport.
#[allow(dead_code)]
pub struct BleDaemon {
    config: BleConfig,
    adapters: Vec<btleplug::platform::Adapter>,
    status: BleStatus,
}

impl BleDaemon {
    /// Create a new BLE daemon with the given configuration.
    pub fn new(config: BleConfig) -> Self {
        Self {
            config,
            adapters: Vec::new(),
            status: BleStatus::Unavailable(BleError::ManagerInitFailed(
                "Not initialized".to_string(),
            )),
        }
    }

    /// Initialize the BLE daemon, probing for adapters.
    /// On Windows, this handles:
    /// - Missing Bluetooth adapter
    /// - Permission denied errors
    /// - Bluetooth service not running
    pub async fn initialize(&mut self) -> BleResult<()> {
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            self.adapters.clear();
            let manager = match new_manager().await {
                Ok(manager) => manager,
                Err(error) => {
                    self.status = BleStatus::Unavailable(error.clone());
                    return Err(error);
                }
            };

            let adapters = match list_adapters(&manager).await {
                Ok(adapters) => adapters,
                Err(error) => {
                    self.status = BleStatus::Unavailable(error.clone());
                    return Err(error);
                }
            };

            if adapters.is_empty() {
                self.status = BleStatus::Unavailable(BleError::NoAdapter);
                return Err(BleError::NoAdapter);
            }

            let adapter_info = match inspect_adapters(&adapters).await {
                Ok(info) => info,
                Err(error) => {
                    self.status = BleStatus::Unavailable(error.clone());
                    return Err(error);
                }
            };
            self.adapters = adapters;
            self.status = BleStatus::Available(adapter_info);
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            self.status = BleStatus::Unavailable(BleError::Other(
                "BLE not supported on this platform".to_string(),
            ));
            Err(BleError::Other(
                "BLE not supported on this platform".to_string(),
            ))
        }
    }

    /// Check if BLE is available and operational.
    pub fn is_available(&self) -> bool {
        matches!(self.status, BleStatus::Available(_))
    }

    /// Get the current BLE status.
    pub fn status(&self) -> &BleStatus {
        &self.status
    }

    /// Scan for BLE advertisements.
    /// Handles the case where the BLE adapter is not present or permission is denied.
    pub async fn scan_for_advertisements(&mut self, _duration_ms: u64) -> BleResult<Vec<String>> {
        if !self.is_available() {
            return Err(BleError::Other(format!(
                "BLE not available: {:?}",
                self.status()
            )));
        }

        Err(BleError::Other(
            "BLE scan is not implemented by BleDaemon; use the GATT central ingress".to_string(),
        ))
    }

    /// Advertise a service via BLE.
    /// On Windows, this handles:
    /// - Adapter not present (returns error)
    /// - Permission denied (returns graceful error)
    /// - Bluetooth disabled (returns graceful error)
    pub async fn advertise_service(&mut self, _service_uuid: &str, _data: &[u8]) -> BleResult<()> {
        if !self.is_available() {
            return Err(BleError::Other(format!(
                "BLE not available: {:?}",
                self.status()
            )));
        }

        Err(BleError::Other(
            "BLE peripheral advertising requires a native platform GATT API; btleplug does not provide it here"
                .to_string(),
        ))
    }

    /// Gracefully shutdown the BLE daemon.
    pub fn shutdown(&mut self) {
        self.status = BleStatus::Disabled;
    }
}

impl Default for BleDaemon {
    fn default() -> Self {
        Self::new(BleConfig::default())
    }
}

/// Check if BLE is likely available on this system.
/// This is a best-effort check that doesn't require full initialization.
pub async fn is_ble_available() -> bool {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        matches!(probe_status().await, BleStatus::Available(_))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format unix timestamp to human readable string
pub fn format_timestamp(timestamp: u64) -> String {
    chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Attempt to enable BLE (Windows only).
/// This tries to prompt for Bluetooth permissions if available.
#[cfg(target_os = "windows")]
pub async fn try_enable_bluetooth() -> BleResult<()> {
    use tokio::process::Command;

    let output = Command::new("sc").args(["query", "bthserv"]).output().await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("RUNNING") {
                Ok(())
            } else {
                Err(BleError::Other("Bluetooth service not running".to_string()))
            }
        }
        Err(e) => Err(BleError::Other(format!(
            "Failed to check Bluetooth service: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_error_display() {
        let err = BleError::NoAdapter;
        assert_eq!(format!("{}", err), "No Bluetooth adapter found");

        let err = BleError::PermissionDenied;
        assert_eq!(format!("{}", err), "Bluetooth permission denied");

        let err = BleError::Other("test".to_string());
        assert_eq!(format!("{}", err), "BLE error: test");
    }

    #[test]
    fn test_ble_config_default() {
        let config = BleConfig::default();
        assert_eq!(config.scan_interval_ms, 1000);
        assert_eq!(config.max_retry_attempts, 3);
    }

    #[test]
    fn test_ble_status_initialization() {
        let daemon = BleDaemon::new(BleConfig::default());
        assert!(!daemon.is_available());
        assert!(matches!(
            daemon.status(),
            BleStatus::Unavailable(BleError::ManagerInitFailed(_))
        ));
    }

    #[test]
    fn test_ble_status_disabled() {
        let mut daemon = BleDaemon::new(BleConfig::default());
        daemon.status = BleStatus::Disabled;
        assert!(!daemon.is_available());
        assert_eq!(daemon.status(), &BleStatus::Disabled);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_ble_error_variants() {
        assert_eq!(
            BleError::NoAdapter.to_string(),
            "No Bluetooth adapter found"
        );
        assert_eq!(
            BleError::PermissionDenied.to_string(),
            "Bluetooth permission denied"
        );
        assert_eq!(
            BleError::AdapterNotPowered.to_string(),
            "Bluetooth adapter not powered on"
        );
        assert!(BleError::Timeout.to_string().contains("timed out"));
    }

    #[test]
    fn test_ble_daemon_fallback_logic() {
        let mut daemon = BleDaemon::new(BleConfig {
            fallback_mode: true,
            ..BleConfig::default()
        });

        // Initial state
        assert!(!daemon.is_available());

        // Manual status injection for testing
        daemon.status = BleStatus::Unavailable(BleError::NoAdapter);
        assert!(!daemon.is_available());
        assert_eq!(
            daemon.status(),
            &BleStatus::Unavailable(BleError::NoAdapter)
        );
    }
}
