// Black-box integration tests for the log-silence heartbeat watchdog.
//
// These spawn the `heartbeat-probe` binary, which runs the EXACT detection
// loop the production node runs (cli/src/main.rs `log_silence_watchdog`):
// poll `scmessenger_cli::config::latest_log_age_secs` on a log dir, require TWO
// consecutive silence readings, and mirror the first-reading diagnostic by
// writing it to the watchdog's own log.
//
// They close the CI-gated proof item of
// HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md -- "no log output for
// T seconds -> process exits 1" is asserted by execution, not by inspection --
// and additionally pin the 2026-09-16 fail-open: the watchdog's own diagnostic
// must never be readable as node liveness, so a healthy node is never killed
// and a wedged one always is.
//
// Temp dirs live in the repo-local tmp/ (rule 2), never the system temp dir.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

fn probe_path() -> PathBuf {
    // Cargo guarantees the binary target of the same package is built before
    // integration tests run and exposes its path at compile time.
    PathBuf::from(env!("CARGO_BIN_EXE_heartbeat-probe"))
}

fn repo_tmp() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tmp")
}

fn fresh_log_dir(tag: &str) -> PathBuf {
    let dir = repo_tmp().join(format!(
        "scm_hb_it_{}_{}_{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    std::fs::create_dir_all(&dir).expect("create log dir");
    dir
}

fn diagnostics_path(dir: &std::path::Path) -> PathBuf {
    scmessenger_cli::config::watchdog_diagnostics_path(dir)
}

fn probe(
    dir: &std::path::Path,
    timeout_secs: &str,
    extra: &[(&str, &str)],
) -> std::process::Output {
    let mut cmd = Command::new(probe_path());
    cmd.env("SCM_PROBE_LOG_DIR", dir)
        .env("SCM_LOG_SILENCE_TIMEOUT_SECS", timeout_secs);
    for (k, v) in extra {
        cmd.env(k, v);
    }
    cmd.output().expect("spawn heartbeat-probe")
}

#[test]
fn watchdog_exits_1_on_log_silence_past_threshold() {
    let dir = fresh_log_dir("silence");
    // The "last log write" is the dir's only file; we simply never touch it
    // again, simulating a wedged node whose writers are all stalled.
    let stale = dir.join("scm.log.0");
    std::fs::write(&stale, "stale\n").expect("write stale log");
    let mtime_before = std::fs::metadata(&stale)
        .expect("stat stale log")
        .modified()
        .expect("mtime");

    // Fresh file, so give the mtime a head start before the probe runs.
    std::thread::sleep(Duration::from_secs(4));

    let start = Instant::now();
    let out = probe(&dir, "2", &[("SCM_PROBE_DEADLINE_SECS", "20")]);
    let elapsed = start.elapsed();

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    // Exit 1 specifically: two consecutive silence readings. Exit 3 would mean
    // the harness deadline expired without a verdict (the self-reset defect).
    assert_eq!(
        out.status.code(),
        Some(1),
        "probe must exit 1 (two consecutive silence readings) on silence; stderr: {}",
        stderr
    );
    // Threshold 2s -> detection within a few poll cycles; guards against the
    // probe silently looping forever on a broken detection path.
    assert!(
        elapsed < Duration::from_secs(60),
        "probe took {elapsed:?} to detect silence -- detection path likely broken"
    );
    assert!(
        stderr.contains("log_silence_probe"),
        "expected the watchdog diagnostic on stderr; got: {stderr}"
    );
    // The exit must be reached through the two-consecutive-readings guard, not
    // the first reading: that guard is what stops a single bad measurement
    // from killing a healthy node.
    assert!(
        stderr.contains("consecutive readings"),
        "expected the two-consecutive-readings exit path; got: {stderr}"
    );
    assert!(
        stderr.contains("first reading only"),
        "expected the probe to log its first silence reading before exiting; got: {stderr}"
    );

    // The diagnostic is recorded where the operator can read it...
    let diag = diagnostics_path(&dir);
    assert!(
        diag.is_file(),
        "watchdog diagnostics must be recorded at {}",
        diag.display()
    );
    let diag_body = std::fs::read_to_string(&diag).expect("read diagnostics");
    assert!(
        diag_body.contains("measured"),
        "diagnostics should record the first silence reading; got: {diag_body}"
    );

    // ...and it did not touch the log the watchdog measures.
    let mtime_after = std::fs::metadata(&stale)
        .expect("stat stale log")
        .modified()
        .expect("mtime");
    assert_eq!(
        mtime_before, mtime_after,
        "the watchdog's own diagnostic refreshed the monitored log -- the self-reset defect is back"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn watchdog_probe_diagnostic_does_not_reset_the_silence_streak() {
    // Regression pin for the 2026-09-16 fail-open: with the diagnostic written
    // through the monitored log, the streak could never reach 2. Under the old
    // behaviour the probe below does not exit at all (and the age assertion
    // fails); under the fixed behaviour it exits 1 within ~2 polls.
    let dir = fresh_log_dir("selfreset");
    let stale = dir.join("scm.log.0");
    std::fs::write(&stale, "stale\n").expect("write stale log");

    std::thread::sleep(Duration::from_secs(4));

    let start = Instant::now();
    let out = probe(&dir, "2", &[("SCM_PROBE_DEADLINE_SECS", "20")]);
    let elapsed = start.elapsed();

    assert_eq!(
        out.status.code(),
        Some(1),
        "a silent node must still exit 1 even though the watchdog writes its own diagnostic; \
         stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        elapsed < Duration::from_secs(30),
        "probe took {elapsed:?}; the self-reset path is back"
    );

    // After the probe has exited, the monitored log must STILL read as silent:
    // the probe's warning went to the watchdog's own log, not into this stream.
    let age = scmessenger_cli::config::latest_log_age_secs(&dir).expect("age of monitored dir");
    assert!(
        age > 2,
        "the probe's own diagnostic reset the measurement to {age}s"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn watchdog_stays_alive_while_log_is_fresh() {
    let dir = fresh_log_dir("fresh");
    let log = dir.join("scm.log.0");
    std::fs::write(&log, "live").expect("write fresh log");

    // Touch the log every 300ms (well under the 2s threshold) for 3s, then
    // stop and give the probe a short grace window: it must NOT have exited
    // while the log kept moving. We observe this by racing the probe against
    // our own touch loop and asserting it is still running when we stop.
    let mut child = Command::new(probe_path())
        .env("SCM_PROBE_LOG_DIR", &dir)
        .env("SCM_LOG_SILENCE_TIMEOUT_SECS", "2")
        .env("SCM_PROBE_DEADLINE_SECS", "30")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn heartbeat-probe");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        std::fs::write(&log, "live").expect("touch log");
        std::thread::sleep(Duration::from_millis(300));
    }

    let still_running = child.try_wait().expect("poll probe").is_none();
    // While fresh, the probe must not have exited on its own.
    assert!(
        still_running,
        "probe exited while the log was being actively written -- false positive"
    );
    assert!(
        !diagnostics_path(&dir).exists(),
        "a node that keeps logging must not accrue silence diagnostics"
    );

    // Now stop touching: the probe must exit within ~threshold.
    let start = Instant::now();
    let status = child.wait().expect("wait probe");
    assert_eq!(
        status.code(),
        Some(1),
        "probe should reach the two-reading verdict shortly after writes stop (took {:?})",
        start.elapsed()
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn watchdog_never_kills_a_node_that_keeps_logging() {
    // The healthy side of the guard: the probe writes its own node log lines
    // into the monitored dir every poll (like the 60s custody audit does), with
    // the same tiny threshold that kills a silent node. It must run to its poll
    // budget and exit 0, never 1.
    let dir = fresh_log_dir("healthy");
    let start = Instant::now();
    let out = probe(
        &dir,
        "2",
        &[
            ("SCM_PROBE_MODE", "healthy"),
            ("SCM_PROBE_MAX_POLLS", "8"),
            ("SCM_PROBE_DEADLINE_SECS", "30"),
        ],
    );
    let elapsed = start.elapsed();

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "a node that keeps logging must not be killed; status {:?}, stderr: {stderr}",
        out.status.code()
    );
    assert!(
        elapsed < Duration::from_secs(60),
        "healthy probe took {elapsed:?}"
    );
    assert!(
        !diagnostics_path(&dir).exists(),
        "no silence diagnostic should ever be recorded for a node writing logs; stderr: {stderr}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
