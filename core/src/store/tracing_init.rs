//! Structured tracing initialization for mesh observability
//!
//! Configures a non-blocking JSON file appender for the core mesh router.
//! The log directory is provided by the host OS via FFI to respect mobile sandboxing.

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize file-based structured tracing with JSON output.
///
/// CRITICAL: Uses `.try_init()` to survive mobile app warm boots where the
/// MeshService is recreated without killing the process. Using `.init()` would
/// panic on the second initialization attempt.
///
/// # Arguments
/// * `log_directory` - Absolute path to log directory from host OS (e.g., iOS Documents, Android filesDir)
///
/// # Output
/// Writes to `<log_directory>/scmessenger-mesh.log` with structured JSON lines:
/// ```json
/// {"timestamp":"2026-03-24T23:00:00Z","level":"INFO","message_id":"abc-123","event":"outbox_enqueue",...}
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn init_file_tracing(log_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = Path::new(log_directory);

    // Ensure the directory exists (critical for mobile sandboxing)
    std::fs::create_dir_all(log_path)?;

    // Non-blocking file appender (prevents I/O from blocking async mesh router)
    let file_appender = tracing_appender::rolling::never(log_path, "scmessenger-mesh.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // CRITICAL: Must leak the guard to prevent appender from flushing/dropping prematurely.
    // The guard's lifetime must outlive the tracing subscriber, or logs will be lost.
    // This is safe because the tracing subscriber is process-global and never uninstalled.
    std::mem::forget(_guard);

    // Build JSON formatter layer
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(false) // Omit module paths for cleaner mobile logs
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    // ENV-based filter. RUST_LOG is unavailable on mobile (there is no shell
    // to export it from before the app process starts), so this fallback is
    // the only lever mobile builds have. Debug/dev cargo profiles (Android
    // `assembleDebug`, built via cargo-ndk WITHOUT `--release` per
    // android/app/build.gradle's `rustProfile`; and local `cargo build`) get
    // a richer default so a field test captures enough to debug message
    // delivery end to end. Optimized/release profiles (Android
    // `assembleRelease`, and the iOS xcframework -- which scripts/
    // rebuild_ios_core.sh ALWAYS builds with `--release`, independent of the
    // Xcode scheme -- so this richer default never applies to iOS) keep the
    // quieter "info" default to bound on-device log file growth and avoid
    // ever writing chat content to disk. RUST_LOG, when settable (desktop
    // CLI), always takes precedence over this fallback either way.
    #[cfg(debug_assertions)]
    let default_filter = "info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug";
    #[cfg(not(debug_assertions))]
    let default_filter = "info";

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    // Install global subscriber (try_init = mobile-safe for warm boots)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .try_init()
        .map_err(|e| format!("Tracing init failed: {}", e))?;

    tracing::info!(
        event = "tracing_initialized",
        log_directory = %log_directory,
        format = "json"
    );

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn init_file_tracing(_log_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    // File logging is not supported on WASM target; rely on console_error_panic_hook or js tracing instead.
    Ok(())
}
