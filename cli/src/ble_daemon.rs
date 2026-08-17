//! Bluetooth central capability and scan lifecycle for the desktop CLI.

use btleplug::api::{
    Central, CentralEvent, CentralState, Manager as _, Peripheral as _, ScanFilter,
};
use std::collections::HashSet;
use uuid::Uuid;

/// SCMessenger's primary BLE GATT service (DF01).
const GATT_SERVICE_UUID: u128 = 0x0000_DF01_0000_1000_8000_0080_5F9B_34FB;

/// Log whether the local Bluetooth stack exposes at least one adapter.
/// On Windows, handles adapter not present and permission denied cases gracefully.
pub async fn probe_and_log() {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        match btleplug::platform::Manager::new().await {
            Ok(manager) => {
                tracing::info!("btleplug: Bluetooth manager created successfully");

                match manager.adapters().await {
                    Ok(adapters) => {
                        if adapters.is_empty() {
                            tracing::warn!(
                                "btleplug: no Bluetooth adapters found. BLE functionality will be unavailable."
                            );
                        } else {
                            tracing::info!(
                                "btleplug: acquired Bluetooth manager; {} adapter(s) visible",
                                adapters.len()
                            );
                            for a in adapters.iter().take(3) {
                                tracing::debug!("btleplug adapter: {:?}", a);
                            }
                        }
                    }
                    Err(e) => {
                        // Handle Windows-specific permission denied errors
                        let err_str = e.to_string().to_lowercase();
                        if err_str.contains("access denied") || err_str.contains("permission") {
                            tracing::warn!(
                                "btleplug: permission denied accessing Bluetooth adapters.
                                 Check Windows Bluetooth permissions in Settings > Privacy > Bluetooth.
                                 BLE functionality will be unavailable."
                            );
                        } else if err_str.contains("not found") || err_str.contains("no device") {
                            tracing::warn!(
                                "btleplug: no Bluetooth adapter found. BLE daemon will operate in fallback mode."
                            );
                        } else {
                            tracing::warn!("btleplug: failed to list adapters: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("btleplug: failed to create manager: {}", e);
            }
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        tracing::debug!("btleplug: BLE probe skipped on this target OS");
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
    /// The requested BLE role or operation is not exposed by this backend.
    Unsupported(String),
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
            BleError::Unsupported(e) => write!(f, "BLE capability unsupported: {}", e),
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
    /// BLE is fully operational
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

/// BLE daemon for the desktop CLI with graceful error handling.
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
            let manager = match btleplug::platform::Manager::new().await {
                Ok(m) => m,
                Err(e) => {
                    self.status =
                        BleStatus::Unavailable(BleError::ManagerInitFailed(e.to_string()));
                    return Err(BleError::ManagerInitFailed(e.to_string()));
                }
            };

            let adapters = match manager.adapters().await {
                Ok(adapters) => adapters,
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    let ble_error =
                        if err_str.contains("access denied") || err_str.contains("permission") {
                            BleError::PermissionDenied
                        } else if err_str.contains("not found") || err_str.contains("no device") {
                            BleError::NoAdapter
                        } else {
                            BleError::Other(e.to_string())
                        };
                    self.status = BleStatus::Unavailable(ble_error.clone());
                    return Err(ble_error);
                }
            };

            if adapters.is_empty() {
                self.status = BleStatus::Unavailable(BleError::NoAdapter);
                return Err(BleError::NoAdapter);
            }

            self.adapters = adapters;
            let adapter_info = self.get_adapter_info().await;
            if !adapter_info.iter().any(|adapter| adapter.is_powered) {
                self.status = BleStatus::Unavailable(BleError::AdapterNotPowered);
                return Err(BleError::AdapterNotPowered);
            }
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

    /// Get information about all detected adapters.
    async fn get_adapter_info(&self) -> Vec<BleAdapterInfo> {
        let mut info = Vec::with_capacity(self.adapters.len());
        for adapter in &self.adapters {
            let state = adapter
                .adapter_state()
                .await
                .unwrap_or(CentralState::Unknown);
            let name = adapter.adapter_info().await.ok();
            info.push(BleAdapterInfo {
                name,
                address: None,
                is_powered: state == CentralState::PoweredOn,
                supports_le: state == CentralState::PoweredOn,
            });
        }
        info
    }

    /// Check if BLE is available and operational.
    pub fn is_available(&self) -> bool {
        matches!(self.status, BleStatus::Available(_))
    }

    /// Get the current BLE status.
    pub fn status(&self) -> &BleStatus {
        &self.status
    }

    /// Scan for real SCMessenger BLE advertisements for the requested duration.
    /// Returned values are the platform's peripheral addresses, not fabricated entries.
    pub async fn scan_for_advertisements(&mut self, duration_ms: u64) -> BleResult<Vec<String>> {
        if !self.is_available() {
            return Err(BleError::Other(format!(
                "BLE not available: {:?}",
                self.status()
            )));
        }

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let service = Uuid::from_u128(GATT_SERVICE_UUID);
            let adapter = self.adapters.first().ok_or(BleError::NoAdapter)?;
            let mut events = adapter
                .events()
                .await
                .map_err(|e| BleError::Other(format!("BLE events unavailable: {}", e)))?;

            adapter
                .start_scan(ScanFilter {
                    services: vec![service],
                })
                .await
                .map_err(|e| BleError::Other(format!("BLE scan start failed: {}", e)))?;

            let mut found = HashSet::new();
            let scan = async {
                while let Some(event) = futures_util::StreamExt::next(&mut events).await {
                    let (id, event_matches) = match event {
                        CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                            (id, false)
                        }
                        CentralEvent::ServicesAdvertisement { id, services } => {
                            (id, services.contains(&service))
                        }
                        CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                            (id, service_data.contains_key(&service))
                        }
                        CentralEvent::ManufacturerDataAdvertisement { .. }
                        | CentralEvent::DeviceConnected(_)
                        | CentralEvent::DeviceDisconnected(_)
                        | CentralEvent::StateUpdate(_) => continue,
                    };

                    let peripheral = match adapter.peripheral(&id).await {
                        Ok(peripheral) => peripheral,
                        Err(e) => {
                            tracing::debug!("BLE peripheral lookup failed during scan: {}", e);
                            continue;
                        }
                    };

                    let is_scm = event_matches
                        || peripheral
                            .properties()
                            .await
                            .ok()
                            .flatten()
                            .map(|properties| {
                                properties.services.contains(&service)
                                    || properties.service_data.contains_key(&service)
                            })
                            .unwrap_or(false);
                    if is_scm {
                        found.insert(peripheral.address().to_string());
                    }
                }
            };

            let _ = tokio::time::timeout(std::time::Duration::from_millis(duration_ms), scan).await;

            adapter
                .stop_scan()
                .await
                .map_err(|e| BleError::Other(format!("BLE scan stop failed: {}", e)))?;

            let mut results: Vec<_> = found.into_iter().collect();
            results.sort_unstable();
            Ok(results)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(BleError::Other(
                "BLE scan not supported on this platform".to_string(),
            ))
        }
    }

    /// Report the peripheral-role capability honestly.
    ///
    /// btleplug exposes the desktop central API used above, not a portable local
    /// GATT server. Windows uses the separate native WinRT path in `ble_windows`;
    /// macOS must not claim that advertising has started.
    pub async fn advertise_service(&mut self, _service_uuid: &str, _data: &[u8]) -> BleResult<()> {
        Err(BleError::Unsupported(
            "btleplug does not provide desktop peripheral GATT advertising; use the native Windows path where available".to_string(),
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
        match btleplug::platform::Manager::new().await {
            Ok(manager) => match manager.adapters().await {
                Ok(adapters) => {
                    let mut powered_on = false;
                    for adapter in adapters {
                        if matches!(adapter.adapter_state().await, Ok(CentralState::PoweredOn)) {
                            powered_on = true;
                            break;
                        }
                    }
                    powered_on
                }
                Err(e) => {
                    tracing::debug!("btleplug adapter availability check failed: {}", e);
                    false
                }
            },
            Err(_) => false,
        }
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
