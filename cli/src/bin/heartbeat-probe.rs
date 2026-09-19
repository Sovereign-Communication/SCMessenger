// Heartbeat watchdog probe binary.
//
// A minimal harness that runs the EXACT detection loop the production node
// runs (cli/src/main.rs `log_silence_watchdog` task): poll
// `scmessenger_cli::config::latest_log_age_secs` on a log dir, require TWO
// consecutive silence readings, and exit(1) when they happen. It also mirrors
// production's first-reading diagnostic, which is written to the watchdog's
// own log so it can never feed back into the measurement it reports on --
// the 2026-09-16 fail-open this probe exists to catch.
//
// Modes (SCM_PROBE_MODE):
//   silence (default) -- never write to the monitored dir; the caller supplies
//                        a stale log file. Proves the exit path end to end.
//   healthy           -- append a node log line to the monitored dir on every
//                        poll, i.e. a node that is making progress. Proves the
//                        watchdog never kills a node that keeps logging.
//
// Bounds: SCM_PROBE_MAX_POLLS (0 = unbounded). In healthy mode the probe exits
// 0 once it has completed that many polls without ever seeing silence.
// SCM_PROBE_DEADLINE_SECS (0 = unbounded) is the harness's safety net: if the
// exit condition has not been reached in that many seconds the probe exits 3,
// with a distinct code, so a test asserts a real failure instead of hanging.
//
// Exit codes: 0 = bounded run completed without silence (healthy), 1 = two
// consecutive silence readings (the watchdog verdict), 3 = deadline expired
// without a verdict (detection path broken -- what the 2026-09-16 self-reset
// produced in production, where the process never exited).
//
// Ticket: HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md

use std::io::Write;
use std::time::Duration;

fn main() {
    let log_dir = std::path::PathBuf::from(
        std::env::var("SCM_PROBE_LOG_DIR").unwrap_or_else(|_| ".".to_string()),
    );
    let timeout_secs: u64 = std::env::var("SCM_LOG_SILENCE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600);
    let mode = std::env::var("SCM_PROBE_MODE").unwrap_or_else(|_| "silence".to_string());
    let max_polls: u64 = std::env::var("SCM_PROBE_MAX_POLLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let deadline_secs: u64 = std::env::var("SCM_PROBE_DEADLINE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let started = std::time::Instant::now();

    // Same polling cadence as production: 1/10th of the timeout (min 1s in
    // the probe, so short test thresholds still poll sensibly).
    let poll = Duration::from_secs((timeout_secs / 10).max(1));
    let mut consecutive_silence: u32 = 0;
    let mut polls: u64 = 0;

    loop {
        std::thread::sleep(poll);
        polls += 1;

        // Checked before any branch, including the `continue` that follows a
        // first silence reading: a broken detection path must end the probe
        // with a distinguishable code rather than looping forever.
        if deadline_secs > 0 && started.elapsed() >= Duration::from_secs(deadline_secs) {
            eprintln!(
                "log_silence_probe: no verdict after {}s (deadline {}s) -- never reached two consecutive silence readings; exiting 3",
                started.elapsed().as_secs(),
                deadline_secs
            );
            std::process::exit(3);
        }

        if mode == "healthy" {
            // A node making progress: append to the file the watchdog measures.
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_dir.join("scm.log.0"))
            {
                let _ = writeln!(f, "probe-activity poll={}", polls);
            }
        }

        if let Some(age) = scmessenger_cli::config::latest_log_age_secs(&log_dir) {
            if age > timeout_secs {
                consecutive_silence = consecutive_silence.saturating_add(1);
                if consecutive_silence >= 2 {
                    eprintln!(
                        "log_silence_probe: no log output for {}s across {} consecutive readings (threshold {}s) -- exiting 1",
                        age, consecutive_silence, timeout_secs
                    );
                    std::process::exit(1);
                }
                // Mirrors production: the first-reading diagnostic is recorded
                // in the watchdog's own log, not in the log being measured.
                let _ = scmessenger_cli::config::append_watchdog_diagnostic(
                    &log_dir,
                    "WARNING",
                    &format!(
                        "log_silence_probe: measured {}s without log output (threshold {}s); requiring a second consecutive reading before exit",
                        age, timeout_secs
                    ),
                );
                eprintln!(
                    "log_silence_probe: measured {}s without log output (threshold {}s) -- first reading only, not exiting yet",
                    age, timeout_secs
                );
                if max_polls > 0 && polls >= max_polls {
                    eprintln!(
                        "log_silence_probe: reached SCM_PROBE_MAX_POLLS={} while silent -- exiting 0",
                        max_polls
                    );
                    std::process::exit(0);
                }
                continue;
            }
        }
        // Fresh (or unreadable/missing) -- reset the streak.
        consecutive_silence = 0;
        if max_polls > 0 && polls >= max_polls {
            eprintln!(
                "log_silence_probe: {} polls with fresh log output -- node healthy, exiting 0",
                polls
            );
            std::process::exit(0);
        }
    }
}
