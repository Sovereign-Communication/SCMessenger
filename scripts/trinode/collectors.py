"""Pluggable raw-log collectors.

Each collector writes raw files into `<run_dir>/raw/<node>/` and returns a
CollectResult (file list, collection timestamps, free-form meta). Parsing is a
separate step (parse.py), so a collector can be swapped or replayed with
--from-dir without touching the correlator.

Hard rules (operator constraints):
  * the AWS host/user/key come ONLY from arguments or environment
    (SCM_AWS_HOST / SCM_AWS_USER / SCM_AWS_KEY); nothing is stored and no IP
    is hardcoded; the manifest records only a salted host fingerprint;
  * collection is passive and read-only (logcat -d, cat, docker logs, GET);
  * every external command goes through an injectable `Runner` so tests (and
    this offline build) never touch a device or host.
"""
from __future__ import annotations

import glob
import hashlib
import json
import os
import shutil
import subprocess
import time
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Callable, Dict, List, Optional, Sequence


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


@dataclass
class CmdResult:
    rc: int
    out: bytes
    err: bytes = b""


Runner = Callable[[Sequence[str], float], CmdResult]


def default_runner(argv: Sequence[str], timeout: float) -> CmdResult:
    p = subprocess.run(list(argv), capture_output=True, timeout=timeout)
    return CmdResult(p.returncode, p.stdout, p.stderr)


@dataclass
class CollectResult:
    node: str
    files: List[str] = field(default_factory=list)     # paths relative to run_dir
    collected_at: str = ""
    meta: Dict[str, object] = field(default_factory=dict)
    errors: List[str] = field(default_factory=list)


def _write(run_dir: str, node: str, name: str, data: bytes) -> str:
    d = os.path.join(run_dir, "raw", node)
    os.makedirs(d, exist_ok=True)
    p = os.path.join(d, name)
    with open(p, "wb") as fh:
        fh.write(data)
    return os.path.relpath(p, run_dir).replace("\\", "/")


# ---------------------------------------------------------------- android ---
class AndroidCollector:
    """Pixel over wireless ADB. `serial` is the adb serial (ip:port for
    wireless); None lets adb pick the only device."""

    PKG = "com.scmessenger.android"

    def __init__(self, serial: Optional[str] = None, adb: str = "adb",
                 runner: Runner = default_runner, package: str = PKG):
        self.serial, self.adb, self.run, self.pkg = serial, adb, runner, package

    def _adb(self, *args: str) -> List[str]:
        base = [self.adb]
        if self.serial:
            base += ["-s", self.serial]
        return base + list(args)

    def collect(self, run_dir: str) -> CollectResult:
        res = CollectResult("android", collected_at=utc_now())
        # UTC + year so timestamps are unambiguous (threadtime alone has neither).
        r = self.run(self._adb("logcat", "-d", "-v", "threadtime", "-v", "year", "-v", "UTC"), 120)
        if r.rc == 0:
            res.files.append(_write(run_dir, "android", "logcat.txt", r.out))
        else:
            res.errors.append(f"logcat rc={r.rc}: {r.err[:200]!r}")
        # App-private files need run-as (debug builds only).
        ls = self.run(self._adb("shell", "run-as", self.pkg, "ls", "files/logs"), 30)
        names = [n for n in ls.out.decode("utf-8", "replace").split() if n.startswith("scmessenger-mesh")] \
            if ls.rc == 0 else []
        if not names:
            names = ["scmessenger-mesh.log"]
        for n in names:
            c = self.run(self._adb("exec-out", "run-as", self.pkg, "cat", f"files/logs/{n}"), 60)
            if c.rc == 0 and c.out:
                res.files.append(_write(run_dir, "android", n, c.out))
            else:
                res.errors.append(f"run-as cat {n} rc={c.rc}")
        for n in ("mesh_diagnostics.log",):
            c = self.run(self._adb("exec-out", "run-as", self.pkg, "cat", f"files/{n}"), 60)
            if c.rc == 0 and c.out:
                res.files.append(_write(run_dir, "android", n, c.out))
        v = self.run(self._adb("shell", "dumpsys", "package", self.pkg), 30)
        if v.rc == 0:
            for line in v.out.decode("utf-8", "replace").splitlines():
                if "versionName=" in line:
                    res.meta["version_name"] = line.split("versionName=", 1)[1].strip()
                    break
        d = self.run(self._adb("shell", "date", "-u", "+%Y-%m-%dT%H:%M:%SZ"), 15)
        if d.rc == 0:
            res.meta["device_clock_utc"] = d.out.decode().strip()
            res.meta["host_clock_utc_at_read"] = utc_now()
        return res


# ---------------------------------------------------------------- windows ---
class WindowsCollector:
    """Local Windows CLI node: hourly-rotated scm.log.* plus /api/diagnostics."""

    def __init__(self, log_dir: Optional[str] = None,
                 diag_url: str = "http://localhost:9001/api/diagnostics",
                 since_epoch: Optional[float] = None,
                 fetch: Optional[Callable[[str, float], bytes]] = None):
        base = os.environ.get("LOCALAPPDATA", "")
        self.log_dir = log_dir or os.path.join(base, "scmessenger", "logs")
        self.diag_url, self.since, self.fetch = diag_url, since_epoch, fetch or self._http_get

    @staticmethod
    def _http_get(url: str, timeout: float) -> bytes:
        with urllib.request.urlopen(url, timeout=timeout) as r:  # noqa: S310 (operator-supplied loopback URL)
            return r.read()

    def collect(self, run_dir: str) -> CollectResult:
        res = CollectResult("windows", collected_at=utc_now())
        paths = sorted(glob.glob(os.path.join(self.log_dir, "scm.log*")))
        if self.since is not None:
            # keep a file if it could contain lines at/after `since` (mtime = last write)
            paths = [p for p in paths if os.path.getmtime(p) >= self.since]
        if not paths:
            res.errors.append(f"no scm.log* under {self.log_dir}")
        for p in paths:
            with open(p, "rb") as fh:
                res.files.append(_write(run_dir, "windows", os.path.basename(p), fh.read()))
        try:
            data = self.fetch(self.diag_url, 8)
            res.files.append(_write(run_dir, "windows", "diagnostics.json", data))
            res.meta["diagnostics"] = _diag_summary(data)
        except Exception as e:  # diagnostics are optional evidence
            res.errors.append(f"diagnostics: {type(e).__name__}: {e}")
        return res


# -------------------------------------------------------------------- aws ---
class AwsCollector:
    """AWS Linux relay (docker container `scm-node`) over ssh. Host/user/key
    are taken from args or env at call time and never persisted."""

    def __init__(self, host: Optional[str] = None, user: Optional[str] = None,
                 key: Optional[str] = None, container: str = "scm-node",
                 since: str = "2h", ssh: str = "ssh", runner: Runner = default_runner,
                 diag_url: str = "http://127.0.0.1:9876/api/diagnostics"):
        self.host = host or os.environ.get("SCM_AWS_HOST")
        self.user = user or os.environ.get("SCM_AWS_USER", "ubuntu")
        self.key = key or os.environ.get("SCM_AWS_KEY")
        self.container, self.since, self.ssh, self.run, self.diag_url = container, since, ssh, runner, diag_url

    def _ssh(self, remote_cmd: str) -> List[str]:
        argv = [self.ssh, "-o", "BatchMode=yes", "-o", "ConnectTimeout=15"]
        if self.key:
            argv += ["-i", self.key]
        return argv + [f"{self.user}@{self.host}", remote_cmd]

    def collect(self, run_dir: str) -> CollectResult:
        res = CollectResult("aws", collected_at=utc_now())
        if not self.host:
            res.errors.append("no AWS host: pass --aws-host or set SCM_AWS_HOST (IP is ephemeral, never stored)")
            return res
        res.meta["host_fingerprint"] = hashlib.sha256(
            (run_dir + "|" + self.host).encode()).hexdigest()[:12]
        c = self.run(self._ssh(f"sudo docker logs -t --since {self.since} {self.container} 2>&1"), 300)
        if c.rc == 0:
            res.files.append(_write(run_dir, "aws", "docker.log", c.out))
        else:
            res.errors.append(f"docker logs rc={c.rc}: {c.err[:200]!r}")
        d = self.run(self._ssh(f"curl -fsS -m 8 {self.diag_url}"), 30)
        if d.rc == 0:
            res.files.append(_write(run_dir, "aws", "diagnostics.json", d.out))
            res.meta["diagnostics"] = _diag_summary(d.out)
        else:
            res.errors.append(f"diagnostics rc={d.rc}")
        meta = self.run(self._ssh(
            f"sudo docker inspect {self.container} --format "
            "'image={{.Config.Image}} started={{.State.StartedAt}} status={{.State.Status}}'; "
            "date -u +%Y-%m-%dT%H:%M:%SZ"), 30)
        if meta.rc == 0:
            lines = meta.out.decode("utf-8", "replace").strip().splitlines()
            res.meta["container"] = lines[0] if lines else ""
            if len(lines) > 1:
                res.meta["host_clock_utc"] = lines[-1]
                res.meta["local_clock_utc_at_read"] = utc_now()
        return res


# --------------------------------------------------------------- from-dir ---
class FromDirCollector:
    """Ingest previously captured raw logs. Layout: <dir>/<node>/<files>, or a
    flat <dir> plus --map node=glob. Files are copied (never moved)."""

    def __init__(self, src_dir: str, node_map: Optional[Dict[str, List[str]]] = None):
        self.src, self.node_map = src_dir, node_map or {}

    def collect(self, run_dir: str) -> List[CollectResult]:
        out: List[CollectResult] = []
        if self.node_map:
            groups = {n: sorted(sum((glob.glob(os.path.join(self.src, g)) for g in gl), []))
                      for n, gl in self.node_map.items()}
        else:
            groups = {}
            for n in sorted(os.listdir(self.src)):
                p = os.path.join(self.src, n)
                if os.path.isdir(p):
                    groups[n] = sorted(os.path.join(p, f) for f in os.listdir(p)
                                       if os.path.isfile(os.path.join(p, f)))
        for node, files in groups.items():
            r = CollectResult(node, collected_at=utc_now())
            r.meta["from_dir"] = True
            for f in files:
                with open(f, "rb") as fh:
                    data = fh.read()
                rel = _write(run_dir, node, os.path.basename(f), data)
                r.files.append(rel)
                if os.path.basename(f).startswith("diagnostics"):
                    try:
                        r.meta["diagnostics"] = _diag_summary(data)
                    except Exception as e:
                        r.errors.append(f"diagnostics parse: {e}")
            if not files:
                r.errors.append("no files matched")
            out.append(r)
        return out


def _diag_summary(data: bytes) -> Dict[str, object]:
    """Extract the few diagnostics fields the verifier uses."""
    obj = json.loads(data.decode("utf-8", "replace"))
    keep = {k: obj[k] for k in ("custody_audit_count", "peers", "running", "timestamp_ms",
                                "connection_path_state") if k in obj}
    if "history_stats" in obj:
        keep["history_stats"] = obj["history_stats"]
    # a node's own libp2p id is the tail of its /p2p-circuit listener
    for l in obj.get("listeners", []) or []:
        if "/p2p-circuit/p2p/" in l:
            keep["self_peer_id"] = l.rsplit("/p2p/", 1)[1]
            break
    return keep
