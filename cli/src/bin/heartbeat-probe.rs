// Heartbeat watchdog probe binary.
//
// A minimal harness that runs the EXACT detection loop the production node
// runs (cli/src/main.rs `log_silence_watchdog` task): poll
// `scmessenger_cli::config::latest_log_age_secs` on a log dir and exit(1)
// when the newest log file is older than the timeout. Used only by
// cli/tests/heartbeat_watchdog_integration.rs to prove the watchdog behavior
// end-to-end without binding ports or touching real node state.
//
// Ticket: HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md

use std::time::Duration;

fn main() {
    let log_dir = std::env::var("SCM_PROBE_LOG_DIR").unwrap_or_else(|_| ".".to_string());
    let timeout_secs: u64 = std::env::var("SCM_LOG_SILENCE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600);

    // Same polling cadence as production: 1/10th of the timeout (min 1s in
    // the probe, so short test thresholds still poll sensibly).
    let poll = Duration::from_secs((timeout_secs / 10).max(1));
    loop {
        if let Some(age) =
            scmessenger_cli::config::latest_log_age_secs(std::path::Path::new(&log_dir))
        {
            if age > timeout_secs {
                eprintln!(
                    "log_silence_probe: no log output for {}s (threshold {}s) -- exiting 1",
                    age, timeout_secs
                );
                std::process::exit(1);
            }
        }
        std::thread::sleep(poll);
    }
}
