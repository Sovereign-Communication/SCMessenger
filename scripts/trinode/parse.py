"""Line normalization: raw log text -> normalized JSONL events.

Event schema (one JSON object per line):
  node      "android" | "windows" | "aws" (or any --node name)
  ts_utc    ISO-8601 UTC with microseconds, or null when unparseable
  ts_raw    the timestamp text exactly as it appeared
  event     normalized event name (see markers.py)
  msg_id    message id or null
  peer      peer id / prefix the event refers to, or null
  detail    dict of extra captured fields (state, n, addr, ...) incl. marker name
  src_line  "<file>:<lineno>" in the raw capture
"""
from __future__ import annotations

import json
import re
from datetime import datetime, timedelta, timezone
from typing import Dict, Iterable, Iterator, List, Optional, Tuple

from . import markers

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
ISO_RE = re.compile(r"(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}:\d{2})(?:[.,](\d{1,9}))?(Z| ?[+-]\d{2}:?\d{2})?")
# docker logs -t prefix followed by the app's own tracing timestamp
DOCKER_DOUBLE_RE = re.compile(r"^(\S+Z) (\d{4}-\d{2}-\d{2}T\S+Z)\s+(?P<rest>.*)$")
TRACING_RE = re.compile(r"^(?P<ts>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z)\s+(?P<lvl>[A-Z]+)\s+(?P<msg>.*)$")
# adb logcat -v year -v UTC  ->  2026-09-29 21:02:29.617 +0000  1415  1544 I Tag: msg
LOGCAT_YEAR_RE = re.compile(
    r"^(?P<ts>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) (?P<tz>[+-]\d{4})\s+\d+\s+\d+\s+(?P<lvl>[VDIWEF])\s+(?P<tag>[^:]*):\s?(?P<msg>.*)$")
# adb logcat -v threadtime (no year, device-local time)
LOGCAT_THREADTIME_RE = re.compile(
    r"^(?P<ts>\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3})\s+\d+\s+\d+\s+(?P<lvl>[VDIWEF])\s+(?P<tag>[^:]*):\s?(?P<msg>.*)$")
# Timber diagnostics file: "2026-09-29 11:10:44.782 D/Mesh: msg" (device local time, NO zone)
TIMBER_RE = re.compile(
    r"^(?P<ts>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) (?P<lvl>[VDIWEF])/(?P<tag>[^:]*):\s?(?P<msg>.*)$")

UTC = timezone.utc


def _frac_to_us(frac: Optional[str]) -> int:
    if not frac:
        return 0
    return int((frac + "000000")[:6])


def parse_iso(ts: str, naive_offset_min: int = 0) -> Optional[datetime]:
    """Parse an ISO-ish timestamp to aware UTC. Naive stamps are interpreted as
    UTC+naive_offset_min (device-local) and converted to UTC."""
    m = ISO_RE.search(ts)
    if not m:
        return None
    date, hms, frac, tz = m.groups()
    try:
        dt = datetime.strptime(f"{date} {hms}", "%Y-%m-%d %H:%M:%S").replace(microsecond=_frac_to_us(frac))
    except ValueError:
        return None
    if tz in (None, ""):
        return (dt - timedelta(minutes=naive_offset_min)).replace(tzinfo=UTC)
    if tz == "Z":
        return dt.replace(tzinfo=UTC)
    tz = tz.strip().replace(":", "")
    sign = 1 if tz[0] == "+" else -1
    off = sign * (int(tz[1:3]) * 60 + int(tz[3:5]))
    return (dt - timedelta(minutes=off)).replace(tzinfo=UTC)


class Envelope:
    __slots__ = ("ts", "ts_raw", "level", "text", "naive")

    def __init__(self, ts, ts_raw, level, text, naive=False):
        self.ts, self.ts_raw, self.level, self.text, self.naive = ts, ts_raw, level, text, naive


def parse_envelope(line: str, year: int, naive_offset_min: int = 0) -> Optional[Envelope]:
    """Split a raw line into (timestamp, level, message text). None if the line
    has no recognizable timestamp (continuation lines, kernel noise)."""
    line = ANSI_RE.sub("", line.rstrip("\r\n"))
    if not line.strip():
        return None
    s = line.lstrip()
    if s.startswith("{") and s.endswith("}"):
        try:
            obj = json.loads(s)
        except ValueError:
            obj = None
        if isinstance(obj, dict) and "timestamp" in obj:
            fields = obj.get("fields") or {}
            msg = str(fields.get("message", ""))
            kv = " ".join(f"{k}={v}" for k, v in fields.items() if k != "message")
            text = (msg + " " + kv).strip()
            ts_raw = str(obj["timestamp"])
            return Envelope(parse_iso(ts_raw), ts_raw, str(obj.get("level", "")), text)
    m = DOCKER_DOUBLE_RE.match(s)
    if m:
        s = m.group(2) + " " + m.group("rest")
    m = TRACING_RE.match(s)
    if m:
        return Envelope(parse_iso(m.group("ts")), m.group("ts"), m.group("lvl"), m.group("msg"))
    m = LOGCAT_YEAR_RE.match(s)
    if m:
        ts_raw = f"{m.group('ts')} {m.group('tz')}"
        return Envelope(parse_iso(ts_raw), ts_raw, m.group("lvl"), m.group("msg"))
    m = TIMBER_RE.match(s)
    if m:
        return Envelope(parse_iso(m.group("ts"), naive_offset_min), m.group("ts"), m.group("lvl"),
                        m.group("msg"), naive=True)
    m = LOGCAT_THREADTIME_RE.match(s)
    if m:
        ts_raw = m.group("ts")
        dt = parse_iso(f"{year}-{ts_raw.replace(' ', 'T')}", naive_offset_min)
        return Envelope(dt, ts_raw, m.group("lvl"), m.group("msg"), naive=True)
    m = ISO_RE.match(s)
    if m:  # generic ISO prefix (e.g. a tool re-timestamped the line)
        rest = s[m.end():].lstrip()
        return Envelope(parse_iso(s[: m.end()], naive_offset_min), s[: m.end()], "", rest,
                        naive=m.group(4) is None)
    return None


def _peer_from_prefix(raw: str) -> str:
    """'30, d0, fa' -> '30d0fa' (manager.rs:399 prints {:x?} of a byte slice;
    note {:x?} drops leading zeros so bytes < 0x10 print as one digit)."""
    parts = [p.strip() for p in raw.split(",") if p.strip()]
    return "".join(p.zfill(2) for p in parts)


def line_to_events(node: str, src_file: str, lineno: int, line: str, year: int,
                   naive_offset_min: int = 0) -> List[dict]:
    env = parse_envelope(line, year, naive_offset_min)
    if env is None:
        return []
    events = []
    for mk, g in markers.match_line(env.text):
        g = {k: v for k, v in g.items() if v is not None}
        msg_id = g.pop("msg", None)
        peer = g.pop("peer", None)
        if mk.name == "core_connection_established" and peer:
            peer = _peer_from_prefix(peer)
        detail: Dict[str, object] = dict(g)
        detail["marker"] = mk.name
        detail["evidence"] = mk.evidence
        if mk.inferred:
            detail["inferred"] = True
        if env.level:
            detail["level"] = env.level
        if env.naive:
            detail["naive_ts"] = True
        names = (mk.event,) + mk.extra_events
        for ev in names:
            d = dict(detail)
            if ev != mk.event:
                d["derived_from"] = mk.event
            events.append({
                "node": node,
                "ts_utc": env.ts.astimezone(UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ") if env.ts else None,
                "ts_raw": env.ts_raw,
                "event": ev,
                "msg_id": msg_id,
                "peer": peer,
                "detail": d,
                "src_line": f"{src_file}:{lineno}",
            })
    return events


def parse_text(node: str, src_file: str, text: str, year: int = 2026,
               naive_offset_min: int = 0) -> List[dict]:
    out: List[dict] = []
    for i, line in enumerate(text.splitlines(), start=1):
        out.extend(line_to_events(node, src_file, i, line, year, naive_offset_min))
    return out


def parse_files(node: str, paths: Iterable[str], rel_to: str = "", year: int = 2026,
                naive_offset_min: int = 0) -> List[dict]:
    import os
    out: List[dict] = []
    for p in paths:
        with open(p, "r", encoding="utf-8", errors="replace") as fh:
            name = os.path.relpath(p, rel_to) if rel_to else os.path.basename(p)
            out.extend(parse_text(node, name.replace("\\", "/"), fh.read(), year, naive_offset_min))
    return out


def ts(ev: dict) -> Optional[datetime]:
    v = ev.get("ts_utc")
    if not v:
        return None
    return datetime.strptime(v, "%Y-%m-%dT%H:%M:%S.%fZ").replace(tzinfo=UTC)
