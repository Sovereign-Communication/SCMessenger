// Black-box integration tests for the log-silence heartbeat watchdog.
//
// These spawn the `heartbeat-probe` binary, which runs the EXACT detection
// loop the production node runs (poll `latest_log_age_secs`, exit(1) on
// silence past the timeout). This closes the CI-gated proof item of
// HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md: "no log output
// for T seconds -> process exits 1" is asserted by execution, not by
// inspection.

use std::process::Command;
use std::time::{Duration, Instant};

fn probe_path() -> std::path::PathBuf {
    // Cargo guarantees the binary target of the same package is built before
    // integration tests run and exposes its path at compile time.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_heartbeat-probe"))
}

fn fresh_log_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "scm_hb_it_{}_{}_{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    std::fs::create_dir_all(&dir).expect("create temp log dir");
    dir
}

#[test]
fn watchdog_exits_1_on_log_silence_past_threshold() {
    let dir = fresh_log_dir("silence");
    // The "last log write" is the dir's only file; we simply never touch it
    // again, simulating a wedged node whose writers are all stalled.
    std::fs::write(dir.join("scm.log.0"), "stale").expect("write stale log");

    // Fresh file, so give the mtime a head start before the probe runs.
    std::thread::sleep(Duration::from_secs(4));

    let start = Instant::now();
    let out = Command::new(probe_path())
        .env("SCM_PROBE_LOG_DIR", &dir)
        .env("SCM_LOG_SILENCE_TIMEOUT_SECS", "2")
        .output()
        .expect("spawn heartbeat-probe");
    let elapsed = start.elapsed();

    assert!(
        out.status.code().is_some_and(|c| c != 0),
        "probe must exit nonzero on silence; got {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    // Threshold 2s -> detection within a few poll cycles; guards against the
    // probe silently looping forever on a broken detection path.
    assert!(
        elapsed < Duration::from_secs(60),
        "probe took {:?} to detect silence -- detection path likely broken",
        elapsed
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("log_silence_probe"),
        "expected the watchdog diagnostic on stderr"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn watchdog_stays_alive_while_log_is_fresh() {
    let dir = fresh_log_dir("fresh");
    std::fs::write(dir.join("scm.log.0"), "live").expect("write fresh log");

    // Touch the log every 300ms (well under the 2s threshold) for 3s, then
    // stop and give the probe a short grace window: it must NOT have exited
    // while the log kept moving. We observe this by racing the probe against
    // our own touch loop and asserting it is still running when we stop.
    let mut child = Command::new(probe_path())
        .env("SCM_PROBE_LOG_DIR", &dir)
        .env("SCM_LOG_SILENCE_TIMEOUT_SECS", "2")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn heartbeat-probe");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        std::fs::write(dir.join("scm.log.0"), "live").expect("touch log");
        std::thread::sleep(Duration::from_millis(300));
    }

    let still_running = child.try_wait().expect("poll probe").is_none();
    // While fresh, the probe must not have exited on its own.
    assert!(
        still_running,
        "probe exited while the log was being actively written -- false positive"
    );

    // Now stop touching: the probe must exit within ~threshold.
    let start = Instant::now();
    let status = child.wait().expect("wait probe");
    let exit_after_silence = start.elapsed() < Duration::from_secs(30) && !status.success();
    assert!(
        exit_after_silence,
        "probe should exit nonzero shortly after writes stop (status: {:?})",
        status.code()
    );

    std::fs::remove_dir_all(&dir).ok();
}
