#!/usr/bin/env python3
"""
SCMessenger Core CLI Driver — Cross-platform Python harness for the Rust CLI daemon.

Usage:
    python scripts/core_cli_driver.py start       # Launch daemon in background
    python scripts/core_cli_driver.py stop        # Kill daemon gracefully
    python scripts/core_cli_driver.py status      # Check if daemon is running
    python scripts/core_cli_driver.py identity    # Show local identity (JSON)
    python scripts/core_cli_driver.py discovery status
    python scripts/core_cli_driver.py discovery peers
    python scripts/core_cli_driver.py contact list
    python scripts/core_cli_driver.py send <peer_id> <message>
    python scripts/core_cli_driver.py daemon-log [N]  # Tail N lines from daemon log

All output is JSON-wrapped for machine consumption by Claude Code.
"""

import os
import sys
import json
import subprocess
import signal
import time
from pathlib import Path

# ── project-relative paths ──────────────────────────────────────────────
REPO_ROOT = Path(__file__).resolve().parent.parent
TMP_DIR = REPO_ROOT / "tmp"
TMP_DIR.mkdir(parents=True, exist_ok=True)

PID_FILE = TMP_DIR / "daemon.pid"
LOG_FILE = TMP_DIR / "daemon.log"
DATA_DIR = TMP_DIR / "daemon_data"
DATA_DIR.mkdir(parents=True, exist_ok=True)

# ── OS detection ─────────────────────────────────────────────────────────
IS_WINDOWS = sys.platform == "win32"

# ── binary resolution ────────────────────────────────────────────────────
def find_binary():
    """Return the path to the compiled CLI binary, or None."""
    exe_name = "scmessenger-cli.exe" if IS_WINDOWS else "scmessenger-cli"
    target_root = REPO_ROOT / "target"
    
    if not target_root.exists():
        return None

    # 1. Check direct profile dirs (target/debug, target/release)
    for profile in ["release", "debug"]:
        cand = target_root / profile / exe_name
        if cand.exists():
            return str(cand)

    # 2. Check triple-specific dirs (target/x86_64-pc-windows-msvc/debug, etc.)
    for target_dir in target_root.iterdir():
        if not target_dir.is_dir():
            continue
        for profile in ["release", "debug"]:
            cand = target_dir / profile / exe_name
            if cand.exists():
                return str(cand)

    return None

def get_run_cmd(args: list[str]) -> list[str]:
    """Build the command line — prefer pre-built binary, fall back to cargo run."""
    bin_path = find_binary()
    if bin_path:
        return [bin_path] + args
    return ["cargo", "run", "-p", "scmessenger-cli", "--"] + args

# ── helpers ──────────────────────────────────────────────────────────────
def emit(status: str, **kwargs):
    """Print JSON result to stdout."""
    out = {"status": status, "os": "windows" if IS_WINDOWS else "unix", "ts": int(time.time())}
    out.update(kwargs)
    print(json.dumps(out, indent=2))

def fail(reason: str, detail: str = ""):
    emit("error", reason=reason, detail=detail)
    sys.exit(1)

# ── daemon lifecycle ─────────────────────────────────────────────────────
def cmd_start():
    if PID_FILE.exists():
        pid = PID_FILE.read_text().strip()
        if _pid_alive(pid):
            emit("daemon_already_running", pid=int(pid))
            return
        PID_FILE.unlink(missing_ok=True)

    cmd = get_run_cmd(["start"])
    log_fh = open(str(LOG_FILE), "wb")

    if IS_WINDOWS:
        # CREATE_NEW_PROCESS_GROUP = 0x200, CREATE_NO_WINDOW = 0x08000000
        proc = subprocess.Popen(
            cmd,
            stdout=log_fh,
            stderr=subprocess.STDOUT,
            cwd=str(REPO_ROOT),
            creationflags=0x08000200,
        )
    else:
        proc = subprocess.Popen(
            cmd,
            stdout=log_fh,
            stderr=subprocess.STDOUT,
            cwd=str(REPO_ROOT),
            start_new_session=True,
        )

    PID_FILE.write_text(str(proc.pid))
    # Give the daemon a moment to start / fail
    time.sleep(1.5)

    if proc.poll() is not None:
        PID_FILE.unlink(missing_ok=True)
        tail = _tail_log(20)
        emit("daemon_crashed", pid=proc.pid, exit_code=proc.returncode, log_tail=tail)
        return

    emit("daemon_started", pid=proc.pid, log=str(LOG_FILE))


def cmd_stop():
    if not PID_FILE.exists():
        emit("daemon_not_running", reason="No PID file found")
        return

    pid_str = PID_FILE.read_text().strip()
    if not pid_str:
        PID_FILE.unlink(missing_ok=True)
        emit("daemon_not_running", reason="Empty PID file")
        return

    try:
        pid = int(pid_str)
    except ValueError:
        PID_FILE.unlink(missing_ok=True)
        emit("daemon_not_running", reason="Corrupt PID file")
        return

    killed = _kill_process(pid)
    PID_FILE.unlink(missing_ok=True)

    if killed:
        emit("daemon_stopped", pid=pid)
    else:
        emit("daemon_not_running", reason=f"PID {pid} was not alive")


def cmd_status():
    if not PID_FILE.exists():
        emit("daemon_not_running", reason="No PID file")
        return

    pid_str = PID_FILE.read_text().strip()
    try:
        pid = int(pid_str)
    except ValueError:
        PID_FILE.unlink(missing_ok=True)
        emit("daemon_not_running", reason="Corrupt PID file")
        return

    if _pid_alive(pid):
        emit("daemon_running", pid=pid)
    else:
        emit("daemon_not_running", reason=f"PID {pid} not alive")


def cmd_daemon_log(lines: int = 100):
    tail = _tail_log(lines)
    print(tail)


# ── mesh commands ─────────────────────────────────────────────────────────
def cmd_identity():
    """Run 'scm identity show' and return its output."""
    run_cli(["identity", "show"])


def cmd_contact_list():
    run_cli(["contact", "list"])


def cmd_send(recipient: str, message: str):
    run_cli(["send", recipient, message])


def cmd_discovery(args: list[str]):
    run_cli(["discovery"] + args)


def run_cli(args: list[str]):
    """Execute a subcommand via subprocess and emit its output."""
    cmd = get_run_cmd(args)
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
            timeout=30,
        )
        emit(
            "ok" if result.returncode == 0 else "error",
            exit_code=result.returncode,
            stdout=result.stdout.strip(),
            stderr=result.stderr.strip(),
            cmd=cmd,
        )
    except subprocess.TimeoutExpired:
        fail("Command timed out after 30s", detail=" ".join(cmd))
    except FileNotFoundError:
        fail(
            "Binary not found and cargo not on PATH — build the CLI first",
            detail="Run: cargo build -p scmessenger-cli",
        )


# ── OS-specific process management ───────────────────────────────────────
def _pid_alive(pid: int) -> bool:
    """Cross-platform check whether a PID is running."""
    if IS_WINDOWS:
        try:
            out = subprocess.run(
                ["tasklist", "/FI", f"PID eq {pid}", "/NH", "/FO", "CSV"],
                capture_output=True,
                text=True,
                timeout=5,
            )
            return str(pid) in out.stdout
        except Exception:
            return False
    else:
        try:
            os.kill(pid, 0)
            return True
        except (ProcessLookupError, PermissionError, OSError):
            return False


def _kill_process(pid: int) -> bool:
    """Cross-platform kill. Returns True if process was terminated."""
    if IS_WINDOWS:
        try:
            subprocess.run(
                ["taskkill", "/PID", str(pid), "/F"],
                capture_output=True,
                timeout=10,
            )
            return True
        except Exception:
            return False
    else:
        try:
            os.kill(pid, signal.SIGKILL)
            return True
        except ProcessLookupError:
            return False
        except OSError:
            # Fallback: try SIGTERM first
            try:
                os.kill(pid, signal.SIGTERM)
                time.sleep(0.5)
                os.kill(pid, signal.SIGKILL)
                return True
            except Exception:
                return False


def _tail_log(lines: int) -> str:
    """Return the last `lines` lines from the daemon log."""
    if not LOG_FILE.exists():
        return "[no log file]"

    if IS_WINDOWS:
        try:
            out = subprocess.run(
                ["powershell", "-NoProfile", "-Command",
                 f"Get-Content '{LOG_FILE}' -Tail {lines}"],
                capture_output=True, text=True, timeout=5,
            )
            return out.stdout.strip() or "[empty]"
        except Exception:
            pass

    # Unix / fallback: read line by line from the end
    try:
        with open(LOG_FILE, "rb") as fh:
            fh.seek(0, 2)
            size = fh.tell()
            buf = b""
            chunk_size = 4096
            collected = 0
            ptr = size
            while ptr > 0 and collected < lines:
                read_size = min(chunk_size, ptr)
                ptr -= read_size
                fh.seek(ptr)
                buf = fh.read(read_size) + buf
                collected = buf.count(b"\n")
            lines_out = buf.split(b"\n")
            # Return last N lines
            return b"\n".join(lines_out[-lines:]).decode("utf-8", errors="replace")
    except Exception as e:
        return f"[error reading log: {e}]"


# ── dispatch ──────────────────────────────────────────────────────────────
def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(0)

    command = sys.argv[1]
    args = sys.argv[2:]

    if command == "start":
        cmd_start()
    elif command == "stop":
        cmd_stop()
    elif command == "status":
        cmd_status()
    elif command == "daemon-log":
        lines = int(args[0]) if args else 100
        cmd_daemon_log(lines)
    elif command == "identity":
        cmd_identity()
    elif command == "contact":
        if args and args[0] == "list":
            cmd_contact_list()
        else:
            fail("Unknown contact subcommand", detail=" ".join(args) if args else "(none)")
    elif command == "send":
        if len(args) < 2:
            fail("Usage: send <recipient> <message>")
        cmd_send(args[0], " ".join(args[1:]))
    elif command == "discovery":
        if not args:
            fail("Usage: discovery <status|scan|peers>")
        cmd_discovery(args)
    elif command == "raw":
        if not args:
            fail("Usage: raw <subcommand> [args...]")
        run_cli(args)
    else:
        fail("Unknown command", detail=command)


if __name__ == "__main__":
    main()
