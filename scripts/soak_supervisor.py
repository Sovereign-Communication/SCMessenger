#!/usr/bin/env python3
"""Always-on soak supervisor for the local SCMessenger node.

Keeps a PINNED node binary running continuously, watches it, captures an
artifact bundle whenever it fails, and relaunches it only when doing so is
safe. Optionally supervises scripts/inbox_bridge.py alongside it so the
message -> orchestrator handoff is always-on too.

Why the constraints exist (each one maps to a way an unattended soak goes
wrong on this machine):

*   Binary pinning. The soak holds one build so results stay comparable across
    a 5-node run. If the binary on disk changes mid-soak, the supervisor halts
    rather than silently soaking a different build and attributing old
    failures to new code.
*   Disk floor. C: runs ~98% full and the node writes an hourly log file
    forever. The supervisor refuses to launch below a free-space floor, and
    prunes its own artifacts and the node's logs to a retention cap.
*   Crash-loop halt. Restarts are capped in a rolling window and backed off
    exponentially. Past the cap the supervisor stops trying and leaves a HALT
    marker, because a tight relaunch loop destroys the evidence of the first
    failure and burns the disk writing logs about it.
*   Fatal-reason list. Some failures must never be retried automatically
    (identity/config errors, storage corruption). Those halt for a human.
*   Single-instance lock. Two supervisors sharing one data directory would
    fight over sled and produce failures that are artifacts of the harness.

Usage:
    python scripts/soak_supervisor.py pin <path-to-scm-binary>
    python scripts/soak_supervisor.py run [--with-bridge]
    python scripts/soak_supervisor.py status
    python scripts/soak_supervisor.py resume     # clear a HALT and continue
    python scripts/soak_supervisor.py stop
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request
from collections import deque
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
API = "http://127.0.0.1:9876"


def _localappdata() -> Path:
    base = os.environ.get("LOCALAPPDATA")
    if base:
        return Path(base) / "scmessenger"
    return Path.home() / ".local" / "share" / "scmessenger"


NODE_HOME = _localappdata()
NODE_LOGS = NODE_HOME / "logs"
SOAK_HOME = NODE_HOME / "soak"
ARTIFACTS = SOAK_HOME / "artifacts"
RUN_LOGS = SOAK_HOME / "runlogs"
PIN_PATH = SOAK_HOME / "pin.json"
STATUS_PATH = SOAK_HOME / "status.json"
HALT_PATH = SOAK_HOME / "HALT.json"
LOCK_PATH = SOAK_HOME / "supervisor.lock"
STOP_PATH = SOAK_HOME / "STOP"

DEFAULTS = {
    # Probing
    "probe_interval_secs": 15,
    "probe_timeout_secs": 5,
    "unreachable_probes_before_fail": 4,
    # A cold node binds its HTTP listener some seconds after the process
    # starts. Without a grace window the supervisor would call a perfectly
    # healthy start-up "wedged" and kill it, producing a restart loop caused
    # entirely by its own impatience.
    "startup_grace_secs": 90,
    # A node that is reachable but meshed with nobody is degraded, not dead.
    # Captured as an artifact but NOT restarted by default: 0 peers is a
    # legitimate state when no peer happens to be online.
    "restart_on_zero_peers": False,
    "zero_peer_degraded_after_secs": 900,
    # Restart policy
    "max_restarts_per_hour": 5,
    "backoff_secs": [5, 15, 45, 120, 300],
    "min_healthy_uptime_secs": 600,
    "min_run_secs_to_not_be_crashloop": 30,
    # Disk safety
    "min_free_disk_mb": 2048,
    "artifact_retention_bundles": 40,
    "node_log_retention_files": 48,
    "runlog_retention_files": 40,
}

# Substrings that mean "do not relaunch, a human must look". Matched against
# the tail of the run log after a failure.
FATAL_LOG_SIGNATURES = [
    "failed to open sled",
    "database is corrupted",
    "corrupted storage",
    "identity not initialized",
    "failed to load identity",
    "address already in use",
]


# --------------------------------------------------------------------------
# helpers
# --------------------------------------------------------------------------


def now_ms() -> int:
    return int(time.time() * 1000)


def iso(ts_ms=None) -> str:
    ts = now_ms() if ts_ms is None else ts_ms
    return datetime.fromtimestamp(ts / 1000, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def stamp() -> str:
    return datetime.now(tz=timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def write_json_atomic(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    with open(tmp, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, default=str)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def read_json(path: Path, default=None):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError):
        return default


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def free_disk_mb(path: Path) -> int:
    usage = shutil.disk_usage(str(path))
    return usage.free // (1024 * 1024)


def load_config() -> dict:
    config = dict(DEFAULTS)
    override = read_json(NODE_HOME / "soak.json")
    if isinstance(override, dict):
        config.update(override)
    return config


def api_get(path: str, timeout=5):
    try:
        with urllib.request.urlopen(API + path, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8") or "{}")
    except (urllib.error.URLError, OSError, ValueError, TimeoutError):
        return None


def api_get_text(path: str, timeout=8):
    try:
        with urllib.request.urlopen(API + path, timeout=timeout) as response:
            return response.read().decode("utf-8", errors="replace")
    except (urllib.error.URLError, OSError, TimeoutError):
        return None


def api_post(path: str, payload: dict, timeout=5):
    try:
        request = urllib.request.Request(
            API + path,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8") or "{}")
    except (urllib.error.URLError, OSError, ValueError, TimeoutError):
        return None


# --------------------------------------------------------------------------
# pinning
# --------------------------------------------------------------------------


def cmd_pin(args) -> int:
    binary = Path(args.binary).resolve()
    if not binary.is_file():
        print("[FAIL] no such binary: %s" % binary)
        return 1
    digest = sha256_file(binary)
    try:
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, capture_output=True, text=True, timeout=15
        ).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        commit = "unknown"
    pin = {
        "binary": str(binary),
        "sha256": digest,
        "size_bytes": binary.stat().st_size,
        "pinned_at": iso(),
        "git_commit": commit,
    }
    write_json_atomic(PIN_PATH, pin)
    print("[OK] pinned %s" % binary)
    print("     sha256 %s" % digest)
    print("     commit %s" % commit)
    print()
    print("[INFO] The soak will refuse to run if this binary changes on disk.")
    print("[INFO] After rebuilding, re-pin deliberately to start a new soak generation.")
    return 0


def verify_pin() -> tuple:
    """Returns (pin_dict, error_or_None)."""
    pin = read_json(PIN_PATH)
    if not pin:
        return None, "no pinned binary -- run `soak_supervisor.py pin <binary>` first"
    binary = Path(pin["binary"])
    if not binary.is_file():
        return pin, "pinned binary is gone: %s" % binary
    actual = sha256_file(binary)
    if actual != pin["sha256"]:
        return pin, (
            "pinned binary CHANGED on disk (expected %s, found %s). "
            "Re-pin deliberately to soak the new build." % (pin["sha256"][:16], actual[:16])
        )
    return pin, None


# --------------------------------------------------------------------------
# artifact capture
# --------------------------------------------------------------------------


def capture_bundle(reason: str, detail: str, context: dict, run_log: Path) -> Path:
    """Snapshot everything needed to diagnose a failure, before relaunching.

    Called while the node may still be alive (wedge case), so the live probes
    run first and the file copies second.
    """
    bundle = ARTIFACTS / ("%s_%s" % (stamp(), reason))
    bundle.mkdir(parents=True, exist_ok=True)

    # Live state first -- this is the part that disappears on restart.
    diagnostics = api_get_text("/api/diagnostics")
    if diagnostics:
        (bundle / "diagnostics.txt").write_text(diagnostics, encoding="utf-8")
    for name, path in (("peers", "/api/peers"), ("listeners", "/api/listeners"),
                       ("discovery_status", "/api/discovery/status"),
                       ("connection_path_state", "/api/connection-path-state")):
        payload = api_get(path, timeout=4)
        if payload is not None:
            write_json_atomic(bundle / ("%s.json" % name), payload)

    # Supervisor's own view.
    write_json_atomic(bundle / "context.json", {
        "reason": reason,
        "detail": detail,
        "captured_at": iso(),
        **context,
    })

    # The node's stdout/stderr for this run.
    if run_log.is_file():
        try:
            data = run_log.read_bytes()
            (bundle / "run.log").write_bytes(data[-2 * 1024 * 1024:])
        except OSError:
            pass

    # Hourly node logs touched during this run.
    started_at = context.get("run_started_at_ms", 0) / 1000
    if NODE_LOGS.is_dir():
        copied = 0
        for log in sorted(NODE_LOGS.glob("scm.log.*"), key=lambda p: p.stat().st_mtime, reverse=True):
            if log.stat().st_mtime + 3600 < started_at or copied >= 3:
                break
            try:
                shutil.copy2(log, bundle / log.name)
                copied += 1
            except OSError:
                pass

    # Bridge state, if the bridge is part of this soak.
    bridge_status = NODE_HOME / "inbox_bridge.status.json"
    if bridge_status.is_file():
        try:
            shutil.copy2(bridge_status, bundle / "inbox_bridge.status.json")
        except OSError:
            pass

    write_json_atomic(bundle / "system.json", {
        "free_disk_mb": free_disk_mb(NODE_HOME),
        "captured_at": iso(),
    })

    print("[INFO] artifacts captured: %s" % bundle)
    return bundle


def prune(config: dict) -> None:
    """Keep the soak from consuming the little disk that is left."""
    def keep_newest(directory: Path, pattern: str, keep: int, is_dir=False):
        if not directory.is_dir():
            return
        entries = [p for p in directory.glob(pattern) if p.is_dir() == is_dir]
        entries.sort(key=lambda p: p.stat().st_mtime, reverse=True)
        for stale in entries[keep:]:
            try:
                shutil.rmtree(stale) if is_dir else stale.unlink()
            except OSError:
                pass

    keep_newest(ARTIFACTS, "*", config["artifact_retention_bundles"], is_dir=True)
    keep_newest(RUN_LOGS, "run_*.log", config["runlog_retention_files"])
    keep_newest(NODE_LOGS, "scm.log.*", config["node_log_retention_files"])


# --------------------------------------------------------------------------
# halt / lock
# --------------------------------------------------------------------------


def halt(reason: str, detail: str, context: dict) -> None:
    payload = {"halted_at": iso(), "reason": reason, "detail": detail, **context}
    write_json_atomic(HALT_PATH, payload)
    print()
    print("[FAIL] SOAK HALTED: %s" % reason)
    print("       %s" % detail)
    print("       Nothing will relaunch until you run:")
    print("       python scripts/soak_supervisor.py resume")


def acquire_lock() -> bool:
    SOAK_HOME.mkdir(parents=True, exist_ok=True)
    existing = read_json(LOCK_PATH)
    if existing:
        pid = existing.get("pid")
        if pid and pid_alive(pid):
            print("[FAIL] another supervisor is already running (pid %s)" % pid)
            print("       Two supervisors on one data dir corrupt sled. Refusing to start.")
            return False
    write_json_atomic(LOCK_PATH, {"pid": os.getpid(), "started_at": iso()})
    return True


def pid_alive(pid: int) -> bool:
    try:
        result = subprocess.run(
            ["tasklist", "/FI", "PID eq %d" % pid, "/NH"],
            capture_output=True, text=True, timeout=10,
        )
        return str(pid) in result.stdout
    except (OSError, subprocess.SubprocessError):
        return False


# --------------------------------------------------------------------------
# process control
# --------------------------------------------------------------------------


def launch_node(pin: dict, run_log: Path):
    run_log.parent.mkdir(parents=True, exist_ok=True)
    handle = open(run_log, "ab", buffering=0)
    handle.write(("\n==== launch %s ====\n" % iso()).encode())
    process = subprocess.Popen(
        [pin["binary"], "start"],
        stdout=handle,
        stderr=subprocess.STDOUT,
        cwd=str(REPO_ROOT),
    )
    return process, handle


def stop_node(process, graceful_timeout=20) -> str:
    """Graceful shutdown via the node's own endpoint, then escalate."""
    if process.poll() is not None:
        return "already_exited"
    api_post("/api/shutdown", {}, timeout=5)
    deadline = time.time() + graceful_timeout
    while time.time() < deadline:
        if process.poll() is not None:
            return "graceful"
        time.sleep(0.5)
    process.terminate()
    try:
        process.wait(timeout=10)
        return "terminated"
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)
        return "killed"


def launch_bridge():
    bridge = REPO_ROOT / "scripts" / "inbox_bridge.py"
    if not bridge.is_file():
        return None
    log = RUN_LOGS / "bridge.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    handle = open(log, "ab", buffering=0)
    return subprocess.Popen(
        [sys.executable, str(bridge), "run"],
        stdout=handle, stderr=subprocess.STDOUT, cwd=str(REPO_ROOT),
    )


# --------------------------------------------------------------------------
# the soak loop
# --------------------------------------------------------------------------


def cmd_run(args) -> int:
    config = load_config()

    if HALT_PATH.is_file():
        existing = read_json(HALT_PATH) or {}
        print("[FAIL] soak is halted since %s: %s" % (
            existing.get("halted_at"), existing.get("reason")))
        print("       %s" % existing.get("detail", ""))
        print("       Review the newest bundle in %s, then `resume`." % ARTIFACTS)
        return 1

    pin, pin_error = verify_pin()
    if pin_error:
        print("[FAIL] %s" % pin_error)
        return 1

    if not acquire_lock():
        return 1

    # A node may already be running that this supervisor does not own -- an
    # orphan left behind when a previous supervisor was killed. Launching a
    # second node against the same sled data directory corrupts it, so never
    # do that silently.
    if api_get("/health", timeout=3) is not None:
        if not args.takeover:
            print("[FAIL] a node is already answering on %s but this supervisor did not" % API)
            print("       start it (likely an orphan from a killed supervisor).")
            print("       Launching a second node on the same data dir would corrupt sled.")
            print("       Re-run with --takeover to shut that node down and assume control.")
            try:
                LOCK_PATH.unlink()
            except OSError:
                pass
            return 1
        print("[WARNING] taking over an already-running node: shutting it down first")
        api_post("/api/shutdown", {}, timeout=5)
        deadline = time.time() + 30
        while time.time() < deadline and api_get("/health", timeout=2) is not None:
            time.sleep(1)
        if api_get("/health", timeout=2) is not None:
            halt("takeover_failed",
                 "an existing node would not shut down; refusing to run a second one",
                 {})
            try:
                LOCK_PATH.unlink()
            except OSError:
                pass
            return 1
        print("[OK] previous node stopped, assuming control")

    if STOP_PATH.is_file():
        STOP_PATH.unlink()

    print("[INFO] soak starting")
    print("[INFO] binary  : %s" % pin["binary"])
    print("[INFO] sha256  : %s" % pin["sha256"][:16])
    print("[INFO] commit  : %s" % pin.get("git_commit", "unknown"))
    print("[INFO] artifacts: %s" % ARTIFACTS)
    print("[INFO] stop with: python scripts/soak_supervisor.py stop")
    print()

    bridge_process = launch_bridge() if args.with_bridge else None
    if args.with_bridge:
        if bridge_process:
            print("[OK] inbox bridge supervised (pid %d)" % bridge_process.pid)
        else:
            print("[WARNING] inbox bridge not found, continuing without it")

    restart_times = deque()
    restart_index = 0
    generation = 0
    exit_code = 0

    try:
        while True:
            prune(config)

            free_mb = free_disk_mb(NODE_HOME)
            if free_mb < config["min_free_disk_mb"]:
                halt("disk_floor",
                     "only %d MB free, floor is %d MB. Launching would risk a disk-full "
                     "failure that looks like a node bug." % (free_mb, config["min_free_disk_mb"]),
                     {"free_disk_mb": free_mb})
                exit_code = 1
                break

            # Re-verify the pin every generation: a concurrent rebuild in this
            # shared checkout would otherwise swap the binary underneath us.
            pin, pin_error = verify_pin()
            if pin_error:
                halt("pin_violation", pin_error, {"generation": generation})
                exit_code = 1
                break

            generation += 1
            run_started = now_ms()
            run_log = RUN_LOGS / ("run_%s_gen%03d.log" % (stamp(), generation))
            process, log_handle = launch_node(pin, run_log)
            print("[OK] node launched (pid %d, generation %d)" % (process.pid, generation))

            reason, detail = watch(process, config, run_started, generation, bridge_process)
            uptime = (now_ms() - run_started) / 1000

            if reason == "operator_stop":
                print("[INFO] stop requested, shutting the node down")
                stop_node(process)
                log_handle.close()
                break

            context = {
                "generation": generation,
                "run_started_at_ms": run_started,
                "run_started_at": iso(run_started),
                "uptime_secs": round(uptime, 1),
                "pid": process.pid,
                "exit_code": process.poll(),
                "pin_sha256": pin["sha256"],
                "git_commit": pin.get("git_commit"),
                "restarts_in_last_hour": len(restart_times),
            }

            bundle = capture_bundle(reason, detail, context, run_log)

            if process.poll() is None:
                stop_node(process)
            log_handle.close()

            fatal = fatal_signature(run_log)
            if fatal:
                halt("fatal_signature",
                     "run log matched a do-not-retry signature: %s" % fatal,
                     {**context, "bundle": str(bundle)})
                exit_code = 1
                break

            now = time.time()
            restart_times.append(now)
            while restart_times and now - restart_times[0] > 3600:
                restart_times.popleft()

            if len(restart_times) > config["max_restarts_per_hour"]:
                halt("crash_loop",
                     "%d restarts in the last hour exceeds the cap of %d. Relaunching "
                     "further would overwrite evidence and burn disk."
                     % (len(restart_times), config["max_restarts_per_hour"]),
                     {**context, "bundle": str(bundle)})
                exit_code = 1
                break

            if uptime >= config["min_healthy_uptime_secs"]:
                restart_index = 0  # it was stable; treat this as a fresh incident
            delay = config["backoff_secs"][min(restart_index, len(config["backoff_secs"]) - 1)]
            restart_index += 1

            print("[WARNING] node down after %.0fs (%s: %s)" % (uptime, reason, detail))
            print("[INFO] relaunching in %ds (restart %d/%d this hour)"
                  % (delay, len(restart_times), config["max_restarts_per_hour"]))
            if not sleep_interruptible(delay):
                break
    finally:
        if bridge_process and bridge_process.poll() is None:
            bridge_process.terminate()
        try:
            LOCK_PATH.unlink()
        except OSError:
            pass

    print("[INFO] supervisor exited")
    return exit_code


def sleep_interruptible(seconds: float) -> bool:
    """Sleep, but wake early on a stop request. False means stop."""
    deadline = time.time() + seconds
    while time.time() < deadline:
        if STOP_PATH.is_file():
            return False
        time.sleep(min(1.0, max(0.0, deadline - time.time())))
    return True


def fatal_signature(run_log: Path):
    try:
        tail = run_log.read_bytes()[-256 * 1024:].decode("utf-8", errors="replace").lower()
    except OSError:
        return None
    for signature in FATAL_LOG_SIGNATURES:
        if signature in tail:
            return signature
    return None


def watch(process, config: dict, run_started: int, generation: int, bridge_process):
    """Probe until something is wrong. Returns (reason, detail)."""
    consecutive_unreachable = 0
    zero_peers_since = None
    bridge_failures = 0
    bridge_given_up = False
    probes = deque(maxlen=40)

    while True:
        if STOP_PATH.is_file():
            return "operator_stop", "STOP file present"

        if process.poll() is not None:
            code = process.poll()
            uptime = (now_ms() - run_started) / 1000
            if uptime < config["min_run_secs_to_not_be_crashloop"]:
                return "immediate_exit", "process exited after %.1fs with code %s" % (uptime, code)
            return "process_exit", "process exited with code %s" % code

        health = api_get("/health", timeout=config["probe_timeout_secs"])
        peers_payload = api_get("/api/peers", timeout=config["probe_timeout_secs"])
        peer_count = None
        if isinstance(peers_payload, dict):
            for key in ("peers", "connected_peers"):
                if isinstance(peers_payload.get(key), list):
                    peer_count = len(peers_payload[key])
                    break

        healthy = isinstance(health, dict) and str(health.get("status", "")).lower() == "healthy"
        in_startup = (now_ms() - run_started) / 1000 < config["startup_grace_secs"]
        probes.append({
            "at": iso(), "healthy": healthy, "peer_count": peer_count,
            "startup_grace": in_startup,
        })

        if healthy:
            consecutive_unreachable = 0
        elif in_startup:
            # Still binding. Not evidence of a wedge yet.
            pass
        else:
            consecutive_unreachable += 1
            if consecutive_unreachable >= config["unreachable_probes_before_fail"]:
                return "unreachable", (
                    "process alive but /health failed %d consecutive probes -- wedged"
                    % consecutive_unreachable)

        # Reachable but meshed with nobody. Degraded, not necessarily dead.
        if healthy and peer_count == 0:
            zero_peers_since = zero_peers_since or time.time()
            stalled = time.time() - zero_peers_since
            if stalled > config["zero_peer_degraded_after_secs"]:
                if config["restart_on_zero_peers"]:
                    return "mesh_dead", "0 connected peers for %.0fs" % stalled
        else:
            zero_peers_since = None

        if bridge_process is not None and bridge_process.poll() is not None:
            code = bridge_process.poll()
            bridge_failures += 1
            # A bridge that dies instantly (bad or missing config, exit 2) would
            # otherwise be respawned every probe forever, burning process
            # creations and filling bridge.log with the same error. Give up
            # after a few and surface it, rather than spinning silently.
            if bridge_failures > 3:
                if not bridge_given_up:
                    print("[WARNING] inbox bridge exited %d times (last code %s); "
                          "not restarting it again" % (bridge_failures, code))
                    print("[WARNING] check %s -- the node soak continues without it"
                          % (RUN_LOGS / "bridge.log"))
                    bridge_given_up = True
                bridge_process = None
            else:
                print("[WARNING] inbox bridge exited (code %s), restarting it" % code)
                replacement = launch_bridge()
                if replacement:
                    bridge_process = replacement

        write_status({
            "supervisor_pid": os.getpid(),
            "node_pid": process.pid,
            "generation": generation,
            "run_started_at": iso(run_started),
            "uptime_secs": round((now_ms() - run_started) / 1000, 1),
            "healthy": healthy,
            "peer_count": peer_count,
            "zero_peers_for_secs": round(time.time() - zero_peers_since, 1) if zero_peers_since else 0,
            "consecutive_unreachable": consecutive_unreachable,
            "free_disk_mb": free_disk_mb(NODE_HOME),
            "bridge_pid": bridge_process.pid if bridge_process else None,
            "recent_probes": list(probes)[-10:],
        })

        time.sleep(config["probe_interval_secs"])


def write_status(payload: dict) -> None:
    payload["updated_at"] = iso()
    payload["updated_at_ms"] = now_ms()
    write_json_atomic(STATUS_PATH, payload)


# --------------------------------------------------------------------------
# other commands
# --------------------------------------------------------------------------


def cmd_status(args) -> int:
    if HALT_PATH.is_file():
        payload = read_json(HALT_PATH) or {}
        print(json.dumps(payload, indent=2))
        print()
        print("[FAIL] soak is HALTED (%s)" % payload.get("reason"))
        print("       newest artifacts: %s" % newest_bundle())
        return 1

    status = read_json(STATUS_PATH)
    if not status:
        print("[FAIL] no soak status -- the supervisor has never run")
        return 1
    print(json.dumps(status, indent=2))
    print()
    age = (now_ms() - int(status.get("updated_at_ms", 0))) / 1000
    if age > 120:
        print("[FAIL] status is %.0fs stale -- the supervisor is not running" % age)
        return 1
    if not status.get("healthy"):
        print("[WARNING] node is not answering /health")
    peers = status.get("peer_count")
    if peers == 0:
        print("[WARNING] node is up but meshed with 0 peers")
    print("[OK] soak alive, generation %s, uptime %.0fs, %d MB free"
          % (status.get("generation"), status.get("uptime_secs", 0), status.get("free_disk_mb", 0)))
    return 0


def newest_bundle():
    if not ARTIFACTS.is_dir():
        return None
    bundles = sorted([p for p in ARTIFACTS.glob("*") if p.is_dir()],
                     key=lambda p: p.stat().st_mtime, reverse=True)
    return bundles[0] if bundles else None


def cmd_resume(args) -> int:
    if not HALT_PATH.is_file():
        print("[INFO] not halted, nothing to resume")
        return 0
    payload = read_json(HALT_PATH) or {}
    HALT_PATH.unlink()
    print("[OK] cleared halt (%s)" % payload.get("reason"))
    print("[INFO] start the soak again with: python scripts/soak_supervisor.py run")
    return 0


def cmd_stop(args) -> int:
    SOAK_HOME.mkdir(parents=True, exist_ok=True)
    STOP_PATH.write_text(iso(), encoding="utf-8")
    print("[OK] stop requested; the supervisor will shut the node down gracefully")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)

    pin_parser = sub.add_parser("pin")
    pin_parser.add_argument("binary")
    pin_parser.set_defaults(handler=cmd_pin)

    run_parser = sub.add_parser("run")
    run_parser.add_argument("--with-bridge", action="store_true",
                            help="also supervise scripts/inbox_bridge.py")
    run_parser.add_argument("--takeover", action="store_true",
                            help="shut down an already-running node and assume control")
    run_parser.set_defaults(handler=cmd_run)

    for name, handler in (("status", cmd_status), ("resume", cmd_resume), ("stop", cmd_stop)):
        sub.add_parser(name).set_defaults(handler=handler)

    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n[INFO] interrupted")
        sys.exit(0)
