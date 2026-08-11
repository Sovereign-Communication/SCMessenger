#!/usr/bin/env python3
"""Bridge inbound SCMessenger messages into the orchestrator HANDOFF queue.

Option A of the message-handoff design: a poller that lives entirely outside
the Rust daemon. It reads the node's own decrypted history over the local
control API, turns each new inbound message from the allow-listed peer into a
HANDOFF/todo ticket, and only then sends an acknowledgement back to the phone.

Design rules this file is built around:

1.  The ACK is emitted AFTER the ticket is on disk and fsync'd, never on
    receipt. An ACK that fires on decrypt proves the transport worked and says
    nothing about whether the orchestrator will ever see the message.
2.  Idempotency is keyed on message_id. The retry -> duplicate delivery path is
    a known behaviour of this stack, so a redelivered message must collapse to
    the ticket that already exists rather than dispatching the orchestrator
    twice.
3.  The allow-list is exact-match on the peer identifier as the node itself
    reports it in history. Nothing is derived from storage/ledger.json: that
    ledger is known to bind peer identities to addresses that are not theirs,
    so it is not an identity source.
4.  Non-allow-listed senders get no ticket and no ACK. Staying silent avoids
    turning the bridge into an oracle that confirms which node is listening.
5.  Inbound content from an allow-listed peer splits into three routes.
    First, the Android/iOS apps wrap EVERY outbound message -- protocol
    housekeeping and real human text alike -- in the same scm.message.*
    JSON envelope, so the envelope is unwrapped first (see
    classify_content()/extract_text()) to get the actual text underneath.
    A message whose kind marks it as protocol housekeeping (identity_sync,
    history_sync, history_sync_data -- periodic re-sync pings the phone
    sends every 1-3 minutes on its own, not something a human typed) or
    whose unwrapped text is empty gets NO ticket, NO chat-log entry, and NO
    reply of any kind -- it is only marked seen so it is not reprocessed.
    Of what remains, a message whose unwrapped text starts with /handoff
    becomes a HANDOFF ticket exactly as before (the /handoff token itself is
    stripped before the body reaches the ticket). Anything else is ordinary
    chat -- no ticket, just an append to HANDOFF/logs/chat/<date>.jsonl and
    a lighter [SEEN] acknowledgement instead of [ACK].

Usage:
    python scripts/inbox_bridge.py discover   # list peers seen in history
    python scripts/inbox_bridge.py selftest   # check wiring, write nothing
    python scripts/inbox_bridge.py once       # single drain pass
    python scripts/inbox_bridge.py run        # poll forever
    python scripts/inbox_bridge.py status     # print the heartbeat file
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_API = "http://127.0.0.1:9876"
DEFAULT_POLL_SECS = 3
HISTORY_LIMIT = 100
HTTP_TIMEOUT_SECS = 5
# Bounds the state file. Well above any plausible redelivery window, while
# keeping the dedupe set from growing without limit over months of uptime.
MAX_SEEN_IDS = 5000

REPO_ROOT = Path(__file__).resolve().parent.parent
HANDOFF_TODO = REPO_ROOT / "HANDOFF" / "todo"
# Checked for pre-existing tickets so a message already picked up by the
# orchestrator is never re-filed if the state file is lost.
HANDOFF_SCAN_DIRS = (
    HANDOFF_TODO,
    REPO_ROOT / "HANDOFF" / "IN_PROGRESS",
    REPO_ROOT / "HANDOFF" / "done",
)
# Ordinary (non-/handoff) chat from an allow-listed peer lands here instead
# of becoming a ticket: one JSON object per line, one file per UTC date.
HANDOFF_CHAT_LOG_DIR = REPO_ROOT / "HANDOFF" / "logs" / "chat"
HANDOFF_COMMAND = "/handoff"


def _appdata() -> Path:
    base = os.environ.get("APPDATA")
    if base:
        return Path(base) / "scmessenger"
    return Path.home() / ".config" / "scmessenger"


def _localappdata() -> Path:
    base = os.environ.get("LOCALAPPDATA")
    if base:
        return Path(base) / "scmessenger"
    return Path.home() / ".local" / "share" / "scmessenger"


CONFIG_PATH = _appdata() / "inbox_bridge.json"
STATE_PATH = _localappdata() / "inbox_bridge.state.json"
STATUS_PATH = _localappdata() / "inbox_bridge.status.json"
# Single-instance guard for `run` (the persistent poller/responder). Lives
# under the same soak/ directory that soak_supervisor.py locks its own
# persistent loop with (LOCK_PATH -> supervisor.lock), so both of this repo's
# long-lived processes are guarded by the same pattern for consistency.
RUN_LOCK_PATH = _localappdata() / "soak" / "inbox_bridge.lock"


# --------------------------------------------------------------------------
# small helpers
# --------------------------------------------------------------------------


def now_ms() -> int:
    return int(time.time() * 1000)


def iso(ts_ms: int) -> str:
    return datetime.fromtimestamp(ts_ms / 1000, tz=timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )


def normalize_ts(value) -> int:
    """History timestamps are milliseconds; tolerate a seconds-valued field."""
    try:
        ts = int(value)
    except (TypeError, ValueError):
        return now_ms()
    # Anything below this is a seconds-since-epoch value, not milliseconds.
    if 0 < ts < 100_000_000_000:
        return ts * 1000
    return ts


def write_json_atomic(path: Path, payload: dict) -> None:
    """Write + fsync + rename, so a crash cannot leave a half-written file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    with open(tmp, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def write_text_atomic(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    with open(tmp, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def read_json(path: Path, default=None):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError):
        return default


# --------------------------------------------------------------------------
# control API
# --------------------------------------------------------------------------


class NodeUnreachable(Exception):
    pass


def api_call(api: str, path: str, payload=None, timeout=HTTP_TIMEOUT_SECS):
    url = api.rstrip("/") + path
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=data, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")[:300]
        raise NodeUnreachable("HTTP %s on %s: %s" % (exc.code, path, detail)) from exc
    except (urllib.error.URLError, OSError, TimeoutError) as exc:
        raise NodeUnreachable("%s unreachable: %s" % (path, exc)) from exc
    if not body:
        return {}
    try:
        return json.loads(body)
    except json.JSONDecodeError as exc:
        raise NodeUnreachable("%s returned non-JSON: %s" % (path, body[:200])) from exc


def probe_node(api: str) -> dict:
    """Liveness probe.

    /health is a static literal in the daemon -- it proves the HTTP task is
    alive and nothing more. peer_count from /api/peers is the signal that
    actually distinguishes a meshed node from a wedged one, so both are
    recorded and the caller is expected to look at peer_count.
    """
    result = {"health_ok": False, "peer_count": None, "error": None}
    try:
        health = api_call(api, "/health", timeout=2)
        result["health_ok"] = str(health.get("status", "")).lower() == "healthy"
    except NodeUnreachable as exc:
        result["error"] = str(exc)
        return result
    try:
        peers = api_call(api, "/api/peers", timeout=3)
        if isinstance(peers, dict):
            for key in ("peers", "connected_peers"):
                if isinstance(peers.get(key), list):
                    result["peer_count"] = len(peers[key])
                    break
    except NodeUnreachable as exc:
        result["error"] = str(exc)
    return result


def fetch_history(api: str, limit=HISTORY_LIMIT) -> list:
    """Pull recent history unfiltered.

    The allow-list is applied in this process rather than via the API's peer_id
    filter, so a mismatch in identifier flavour (identity id vs peer id) shows
    up as a visible zero-match in `discover` instead of silently returning an
    empty conversation.
    """
    response = api_call(api, "/api/history", payload={"limit": limit})
    messages = response.get("messages", []) if isinstance(response, dict) else []
    return messages if isinstance(messages, list) else []


def send_message(api: str, recipient: str, message: str) -> dict:
    return api_call(api, "/api/send", payload={"recipient": recipient, "message": message})


# --------------------------------------------------------------------------
# config / state
# --------------------------------------------------------------------------


def _normalize_peer_ids(value) -> list:
    """allowed_peer_id accepts either a single string or a list of strings.

    A plain string is the original, still-supported config shape -- no
    migration is required. Order is preserved and duplicates are dropped so
    downstream code (and log/print output) is deterministic.
    """
    candidates = value if isinstance(value, list) else [value]
    seen = set()
    out = []
    for item in candidates:
        peer = str(item).strip()
        if peer and peer not in seen:
            seen.add(peer)
            out.append(peer)
    return out


def load_config(api_override=None) -> dict:
    config = read_json(CONFIG_PATH)
    if config is None:
        sys.stderr.write(
            "[FAIL] No bridge config at %s\n"
            "       Create it with the phone's identifier exactly as it appears\n"
            "       in `python scripts/inbox_bridge.py discover`:\n\n"
            "       {\n"
            '         "allowed_peer_id": "<phone identifier from discover>",\n'
            '         "poll_interval_secs": %d,\n'
            '         "api": "%s"\n'
            "       }\n\n"
            "       allowed_peer_id may also be a list of identifiers.\n"
            % (CONFIG_PATH, DEFAULT_POLL_SECS, DEFAULT_API)
        )
        raise SystemExit(2)
    peer_ids = _normalize_peer_ids(config.get("allowed_peer_id", ""))
    if not peer_ids:
        sys.stderr.write(
            "[FAIL] allowed_peer_id is empty in %s -- refusing to run without an\n"
            "       allow-list. Run `discover` to find the phone's identifier.\n"
            % CONFIG_PATH
        )
        raise SystemExit(2)
    # allowed_peer_ids is the canonical list used everywhere internally;
    # allowed_peer_id is kept as the first entry for backward compatibility
    # with any code (or operator habit) that reads the singular key.
    config["allowed_peer_ids"] = peer_ids
    config["allowed_peer_id"] = peer_ids[0]
    config.setdefault("api", DEFAULT_API)
    config.setdefault("poll_interval_secs", DEFAULT_POLL_SECS)
    if api_override:
        config["api"] = api_override
    return config


def load_state() -> dict:
    state = read_json(STATE_PATH) or {}
    # First run must not treat the entire existing conversation as new work.
    # Without this, adopting the bridge on a node with history files a ticket
    # for every message ever received and fires an ACK storm at the phone.
    state.setdefault("baselined", False)
    state.setdefault("seen_message_ids", [])
    state.setdefault("ack_pending", {})
    state.setdefault("tickets_written_total", 0)
    state.setdefault("acks_sent_total", 0)
    state.setdefault("last_received_message_at", None)
    return state


def save_state(state: dict) -> None:
    seen = state.get("seen_message_ids", [])
    if len(seen) > MAX_SEEN_IDS:
        state["seen_message_ids"] = seen[-MAX_SEEN_IDS:]
    write_json_atomic(STATE_PATH, state)


# --------------------------------------------------------------------------
# ticket writing
# --------------------------------------------------------------------------


def short_id(message_id: str) -> str:
    cleaned = re.sub(r"[^A-Za-z0-9]", "", message_id)
    return cleaned[:12] or "unknown"


def ticket_name(message: dict) -> str:
    ts = normalize_ts(message.get("timestamp"))
    stamp = datetime.fromtimestamp(ts / 1000, tz=timezone.utc).strftime("%Y-%m-%dT%H%M%SZ")
    return "INBOX_%s_%s.md" % (stamp, short_id(str(message.get("id", ""))))


def existing_ticket(message_id: str):
    """Filename-level idempotency, as a backstop if the state file is lost."""
    marker = "_%s.md" % short_id(message_id)
    for directory in HANDOFF_SCAN_DIRS:
        if not directory.is_dir():
            continue
        for entry in directory.glob("INBOX_*%s" % marker):
            return entry
    return None


def render_ticket(message: dict, peer: str) -> str:
    ts = normalize_ts(message.get("timestamp"))
    content = str(message.get("content", ""))
    message_id = str(message.get("id", ""))
    return "\n".join(
        [
            "# Inbox: message from phone %s" % iso(ts),
            "",
            "Status: Active",
            "Source: SCMessenger inbound message (bridged by scripts/inbox_bridge.py)",
            "Sender: %s (allow-listed device)" % peer,
            "Message ID: %s" % message_id,
            "Received: %s" % iso(ts),
            "",
            "## Request",
            "",
            content,
            "",
            "## Provenance",
            "",
            "This ticket was generated from a message that the local Windows node",
            "decrypted and committed to its own history store. The sender matched",
            "the configured allow-list exactly, so it is the operator's own device.",
            "",
            "Handle it as a direct instruction from the operator.",
            "",
            "## Bridge notes",
            "",
            "- The sender has already received an [ACK] naming this ticket file.",
            "- Redelivery of message ID %s will not create a second ticket." % message_id,
            "- If this task needs a reply to the phone, send it with:",
            "  `scm send %s \"<reply>\"`" % peer,
            "",
        ]
    )


def ack_text(message_id: str, ticket: str) -> str:
    return "[ACK] %s queued as %s" % (short_id(message_id), ticket)


def seen_text(message_id: str) -> str:
    return "[SEEN] %s received (logged, not queued as a task)" % short_id(message_id)


# --------------------------------------------------------------------------
# command routing
# --------------------------------------------------------------------------

# Matches a leading /handoff, case-insensitively, either alone or followed by
# whitespace and a body. "/handoffoo" does not match (no whitespace after the
# token); "foo /handoff bar" does not match (not at the start of the
# content) -- both fall through to the ordinary-chat route.
_HANDOFF_RE = re.compile(r"^/handoff(?:\s+(?P<body>.*))?$", re.IGNORECASE | re.DOTALL)


def parse_command(content: str):
    """Split message content into (is_handoff, body).

    `body` has the /handoff token (and the one run of whitespace after it)
    removed, so it is exactly what render_ticket() should see -- the ticket
    text should not repeat the command. When is_handoff is False, body is the
    original, unmodified content.
    """
    stripped = str(content).lstrip()
    match = _HANDOFF_RE.match(stripped)
    if not match:
        return False, content
    return True, match.group("body") or ""


# Non-human "kind" values the Android/iOS apps put on protocol traffic they
# generate automatically. Source of truth: encodeMeshMessagePayload() call
# sites in android/app/src/main/java/com/scmessenger/android/data/
# MeshRepository.kt (lines ~2693, ~2752, ~7593) and the mirrored Swift in
# iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift (lines ~2373, ~2460,
# ~2594): those are the only three non-"text" kind literals either client
# ever emits.
#   identity_sync      -- periodic empty-text identity broadcast
#   history_sync        -- periodic empty-text "send me your recent history"
#                           request
#   history_sync_data   -- the reply to history_sync: a JSON array of the
#                           sender's own message rows, carried in `text`.
#                           This one is NOT empty-text, so it would slip past
#                           an empty-text-only check -- it must be matched by
#                           kind, which is why kind is the primary signal
#                           below and "empty text" is only a backstop.
# Deliberately NOT included: `schema`. Every envelope this protocol sends --
# control traffic and real human text alike -- carries the same
# "scm.message.identity.v1" schema string. Confirmed against this node's own
# history: message e7a8b366 ("confirmed delivery probe.", a verified human
# reply logged in HANDOFF/todo/INBOX_2026-08-11T043333Z_e7a8b366ff27.md) has
# the identical schema field as the identity_sync/history_sync pings sitting
# next to it in HANDOFF/logs/chat/2026-08-11.jsonl. Treating "schema starts
# with scm.message." as a housekeeping signal would misclassify every human
# message from this peer -- exactly the failure mode the conservative
# fallback below exists to avoid.
CONTROL_KINDS = frozenset({"identity_sync", "history_sync", "history_sync_data"})

# Keys that identify a bare transport delivery receipt, as emitted by peers
# that do not wrap outbound traffic in the scm.message.* envelope (the CLI and
# desktop lanes). Matched as an exact key-set subset with no text-bearing
# field, so an ordinary message cannot be mistaken for one.
RECEIPT_REQUIRED_KEYS = frozenset({"message_id", "status"})
RECEIPT_ALLOWED_KEYS = frozenset({"message_id", "status", "timestamp"})


def _is_delivery_receipt(candidate: dict) -> bool:
    """True for {"message_id":..,"status":..[,"timestamp":..]} and nothing else.

    Deliberately strict: every key must be known, both required keys must be
    present, and any text-bearing field disqualifies it. A receipt carries no
    human payload, so refusing to ACK one cannot swallow a human message --
    which is the invariant classify_content() is built around.
    """
    keys = set(candidate)
    if not RECEIPT_REQUIRED_KEYS.issubset(keys):
        return False
    if not keys.issubset(RECEIPT_ALLOWED_KEYS):
        return False
    return not str(candidate.get("status", "")).strip() == ""


def classify_content(content) -> tuple:
    """Split raw /api/history `content` into (category, text).

    category is "housekeeping" or "content". `text` is the human-readable
    payload to actually act on: for the scm.message.* envelope format the
    Android/iOS apps wrap every outbound message in (control traffic and
    real chat alike -- see CONTROL_KINDS above for the evidence), that means
    the envelope's own `text` field; for anything else, `text` is `content`
    itself, unchanged, since plain-string content (e.g. from a CLI/desktop
    peer, which never wraps in this envelope) already IS the message.

    Housekeeping rules, in order, each conservative-by-construction: when in
    doubt this returns "content", never "housekeeping" -- silently
    swallowing a human message is worse than logging one stray control ping.

    1. Nothing here at all (content empty/whitespace after stripping) --
       there is no message to route either way, so it is dropped rather than
       logged or ticketed.
    2. content parses as a JSON object carrying a `kind` field (or a
       `schema`+`text` pair as a fallback shape): `kind` in CONTROL_KINDS is
       housekeeping unconditionally (this is what catches
       history_sync_data's non-empty text). `kind` missing/blank defaults to
       "text", mirroring the apps' own decode default (see
       decodeMessageWithIdentityHints()/DecodedMessagePayload in the Kotlin/
       Swift referenced above) -- an unrecognized/absent kind is NOT treated
       as housekeeping by itself.
    3. Whatever kind resolves to (recognized "text", or anything not in
       CONTROL_KINDS), an empty/whitespace `text` field inside the envelope
       is housekeeping -- there is nothing to relay, so nothing is swallowed
       by dropping it.

    Anything that doesn't parse as this envelope shape at all (no JSON, JSON
    that isn't an object, or an object with neither `kind` nor a
    `schema`+`text` pair) is passed straight through as ordinary content --
    including a message that merely contains JSON-looking text, or whose
    text happens to equal a control-kind name like "history_sync" as a
    literal string rather than a JSON envelope.
    """
    raw = content if isinstance(content, str) else str(content)
    stripped = raw.strip()
    if not stripped:
        return "housekeeping", raw

    envelope = None
    if stripped.startswith("{"):
        try:
            candidate = json.loads(stripped)
        except (json.JSONDecodeError, ValueError):
            candidate = None
        if isinstance(candidate, dict) and (
            "kind" in candidate or ("schema" in candidate and "text" in candidate)
        ):
            envelope = candidate
        elif isinstance(candidate, dict) and _is_delivery_receipt(candidate):
            # Rule 2b: a bare delivery receipt from a peer that does not wrap
            # in the scm.message.* envelope (CLI/desktop lanes emit
            # {"message_id":..,"status":"Delivered","timestamp":..}). This
            # MUST be housekeeping: ACKing a receipt makes the far node emit a
            # receipt for the ACK, which arrives as fresh allow-listed inbound
            # and is ACKed in turn -- an unbounded feedback loop. Observed
            # 2026-08-11: ~1400 [SEEN] ACKs emitted in minutes after a second
            # peer was allow-listed. Narrow by construction: a human message
            # is never exactly this key set with no text-bearing field.
            return "housekeeping", raw

    if envelope is None:
        return "content", raw

    kind = str(envelope.get("kind") or "text").strip() or "text"
    text = envelope.get("text", "")
    text = text if isinstance(text, str) else str(text)

    if kind in CONTROL_KINDS:
        return "housekeeping", text
    if not text.strip():
        return "housekeeping", text
    return "content", text


def extract_text(content) -> str:
    """The human-readable text `content` actually carries, unwrapping the
    scm.message.* envelope if present. See classify_content() for the full
    reasoning; this is the convenience form for callers that already know
    (via route_message()) that the message is not housekeeping and just want
    the text to act on.
    """
    _, text = classify_content(content)
    return text


def route_message(peer_id: str, content: str, allowed_peer_ids) -> str:
    """Pure routing decision: "ignored", "housekeeping", "handoff", or "chat".

    No I/O, so tests can exercise the exact decision drain_once() makes
    without a live node. drain_once() also uses this directly, so the
    behaviour under test is the behaviour that actually runs.

    The /handoff check runs against the EXTRACTED text, never the raw
    `content`: for this bridge's allow-listed peer (the Android phone),
    every inbound message -- human or not -- arrives wrapped in the
    scm.message.* envelope (`{"schema":...,"kind":"text","text":"/handoff
    ...","sender":{...}}`), so a check against the raw envelope string could
    never match a leading "/handoff".
    """
    allowed = set(str(p) for p in allowed_peer_ids)
    if str(peer_id) not in allowed:
        return "ignored"
    category, text = classify_content(content)
    if category == "housekeeping":
        return "housekeeping"
    is_handoff, _ = parse_command(text)
    return "handoff" if is_handoff else "chat"


def append_json_line_atomic(path: Path, record: dict) -> None:
    """Append one JSON line, fsync'd before returning.

    Append mode (rather than read-modify-atomic-replace) means a crash mid
    write can only corrupt the line being written, never a previously
    committed one.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    line = json.dumps(record, ensure_ascii=False) + "\n"
    with open(path, "a", encoding="utf-8", newline="\n") as handle:
        handle.write(line)
        handle.flush()
        os.fsync(handle.fileno())


def chat_log_path(ts_ms: int) -> Path:
    date_str = datetime.fromtimestamp(ts_ms / 1000, tz=timezone.utc).strftime("%Y-%m-%d")
    return HANDOFF_CHAT_LOG_DIR / ("%s.jsonl" % date_str)


def append_chat_log(message: dict, peer: str) -> Path:
    """Append one chat-log line, storing the unwrapped human text.

    `content` here is deliberately the extract_text() result, not the raw
    /api/history content -- for this bridge's peer that raw value is a full
    scm.message.* JSON envelope (schema/kind/sender and all), which made the
    log an unreadable wall of escaped JSON per line. Plain-string content
    (not enveloped) passes through extract_text() unchanged, so this is a
    strict readability improvement with no behaviour change for that case.
    """
    ts = normalize_ts(message.get("timestamp"))
    path = chat_log_path(ts)
    record = {
        "message_id": str(message.get("id", "")),
        "peer_id": peer,
        "timestamp": iso(ts),
        "content": extract_text(message.get("content", "")),
    }
    append_json_line_atomic(path, record)
    return path


# --------------------------------------------------------------------------
# drain
# --------------------------------------------------------------------------


def drain_once(config: dict, state: dict, status: dict) -> int:
    """One poll pass. Returns the number of tickets written."""
    api = config["api"]
    peer_ids = config.get("allowed_peer_ids") or _normalize_peer_ids(config["allowed_peer_id"])

    probe = probe_node(api)
    status["node_health_ok"] = probe["health_ok"]
    status["node_peer_count"] = probe["peer_count"]
    if not probe["health_ok"]:
        status["node_reachable"] = False
        status["consecutive_node_failures"] = status.get("consecutive_node_failures", 0) + 1
        status["last_error"] = probe["error"] or "node not reachable"
        return 0

    try:
        messages = fetch_history(api)
    except NodeUnreachable as exc:
        status["node_reachable"] = False
        status["consecutive_node_failures"] = status.get("consecutive_node_failures", 0) + 1
        status["last_error"] = str(exc)
        return 0

    status["node_reachable"] = True
    status["consecutive_node_failures"] = 0
    status["last_error"] = None

    if not state.get("baselined"):
        existing = [str(m.get("id")) for m in messages if m.get("id")]
        state["seen_message_ids"] = existing[-MAX_SEEN_IDS:]
        state["baselined"] = True
        state["baselined_at"] = iso(now_ms())
        save_state(state)
        status["baselined_at"] = state["baselined_at"]
        print("[INFO] first run: baselined %d existing message(s) as already-seen." % len(existing))
        print("[INFO] Only messages arriving from now on become tickets.")
        return 0

    seen = set(state["seen_message_ids"])
    allowed = set(peer_ids)
    inbound = [
        m
        for m in messages
        if str(m.get("direction", "")) == "received" and str(m.get("peer_id", "")) in allowed
    ]
    status["allowlisted_inbound_in_window"] = len(inbound)
    status["ignored_inbound_in_window"] = sum(
        1
        for m in messages
        if str(m.get("direction", "")) == "received" and str(m.get("peer_id", "")) not in allowed
    )
    if inbound:
        state["last_received_message_at"] = max(
            normalize_ts(m.get("timestamp")) for m in inbound
        )

    fresh = [m for m in inbound if str(m.get("id", "")) not in seen]
    fresh.sort(key=lambda m: normalize_ts(m.get("timestamp")))

    written = 0
    for message in fresh:
        message_id = str(message.get("id", ""))
        if not message_id:
            continue

        prior = existing_ticket(message_id)
        if prior is not None:
            # Already filed in a previous life of this bridge. Record it as
            # seen so we neither re-file nor re-ACK it.
            state["seen_message_ids"].append(message_id)
            save_state(state)
            continue

        sender = str(message.get("peer_id", ""))
        content = message.get("content", "")
        route = route_message(sender, content, peer_ids)
        if route == "ignored":
            # Belt-and-suspenders: `inbound` above already filtered to
            # peer_ids, so this should be unreachable. If it ever triggers,
            # stay silent -- same rule as any other non-allow-listed sender.
            state["seen_message_ids"].append(message_id)
            save_state(state)
            continue

        if route == "housekeeping":
            # Protocol housekeeping (identity_sync/history_sync/
            # history_sync_data, or an envelope with nothing in `text`):
            # marked seen for dedupe only. No ticket, no chat-log entry, and
            # -- unlike the "chat" branch -- no ack_pending entry either, so
            # this never produces a reply of any kind.
            state["seen_message_ids"].append(message_id)
            save_state(state)
            print(
                "[INFO] housekeeping suppressed (no reply) <- message %s"
                % short_id(message_id)
            )
            continue

        # Both remaining routes ("handoff" and "chat") act on the unwrapped
        # text, never the raw envelope -- see route_message()'s docstring.
        text = extract_text(content)

        if route == "handoff":
            _, body = parse_command(text)
            ticket_message = dict(message)
            ticket_message["content"] = body
            name = ticket_name(message)
            write_text_atomic(HANDOFF_TODO / name, render_ticket(ticket_message, sender))

            # Ticket is durable before anything is promised to the phone.
            state["seen_message_ids"].append(message_id)
            state["tickets_written_total"] += 1
            state["ack_pending"][message_id] = {"peer": sender, "kind": "ack", "ticket": name}
            save_state(state)
            written += 1
            print("[OK] ticket %s <- message %s" % (name, short_id(message_id)))
        else:
            log_path = append_chat_log(message, sender)

            # Log entry is durable before anything is promised to the phone.
            state["seen_message_ids"].append(message_id)
            state["ack_pending"][message_id] = {"peer": sender, "kind": "seen", "ticket": None}
            save_state(state)
            print(
                "[SEEN] logged to %s <- message %s"
                % (log_path.name, short_id(message_id))
            )

    flush_acks(config, state, status)
    return written


def flush_acks(config: dict, state: dict, status: dict) -> None:
    """Send outstanding ACKs (and [SEEN] acks). A failure here retries next
    pass, never re-files or re-logs."""
    pending = state.get("ack_pending", {})
    if not pending:
        status["acks_pending"] = 0
        return
    # Fallback recipient for entries written by a pre-multi-peer version of
    # this script, where ack_pending[message_id] was a bare ticket name
    # string rather than a {peer, kind, ticket} dict.
    fallback_peer = config.get("allowed_peer_id", "")
    for message_id, entry in list(pending.items()):
        if isinstance(entry, str):
            entry = {"peer": fallback_peer, "kind": "ack", "ticket": entry}
        recipient = entry.get("peer") or fallback_peer
        if entry.get("kind") == "seen":
            text = seen_text(message_id)
        else:
            text = ack_text(message_id, entry.get("ticket") or "")
        try:
            response = send_message(config["api"], recipient, text)
        except NodeUnreachable as exc:
            status["last_error"] = "ACK send failed for %s: %s" % (short_id(message_id), exc)
            break
        if isinstance(response, dict) and response.get("success"):
            del pending[message_id]
            state["acks_sent_total"] += 1
            save_state(state)
            print("[OK] ack sent for %s" % short_id(message_id))
        else:
            error = ""
            if isinstance(response, dict):
                error = str(response.get("error") or response)
            status["last_error"] = "ACK rejected for %s: %s" % (short_id(message_id), error)
            break
    status["acks_pending"] = len(pending)


def count_tickets_on_disk() -> int:
    """Durable ticket count.

    The in-state counter resets if the state file is lost, so the number that
    gets reported is derived from the tickets themselves.
    """
    total = 0
    for directory in HANDOFF_SCAN_DIRS:
        if directory.is_dir():
            total += sum(1 for _ in directory.glob("INBOX_*.md"))
    return total


def write_status(status: dict, state: dict) -> None:
    status["last_loop_at"] = iso(now_ms())
    status["last_loop_at_ms"] = now_ms()
    status["tickets_on_disk"] = count_tickets_on_disk()
    status["tickets_written_this_session"] = state.get("tickets_written_total", 0)
    status["acks_sent_total"] = state.get("acks_sent_total", 0)
    last_rx = state.get("last_received_message_at")
    status["last_received_message_at"] = iso(last_rx) if last_rx else None
    write_json_atomic(STATUS_PATH, status)


def new_status() -> dict:
    return {
        "bridge_pid": os.getpid(),
        "bridge_started_at": iso(now_ms()),
        "node_reachable": False,
        "node_health_ok": False,
        "node_peer_count": None,
        "consecutive_node_failures": 0,
        "acks_pending": 0,
        "last_error": None,
    }


# --------------------------------------------------------------------------
# single-instance lock (guards `run` only -- see RUN_LOCK_PATH)
# --------------------------------------------------------------------------


def pid_alive(pid: int) -> bool:
    """Cross-checks a recorded PID against the live process table.

    Mirrors soak_supervisor.py's pid_alive(): tasklist is the only
    dependency-free way to ask Windows "is this PID real" without pulling in
    psutil, and a stale/reused PID in the lock file must never be trusted
    blindly.
    """
    try:
        result = subprocess.run(
            ["tasklist", "/FI", "PID eq %d" % pid, "/NH"],
            capture_output=True, text=True, timeout=10,
        )
        return str(pid) in result.stdout
    except (OSError, subprocess.SubprocessError):
        return False


def acquire_run_lock() -> bool:
    """Hard single-instance guard: refuses to start a second `run` poller.

    Two `run` processes polling the same node would each maintain their own
    ack_pending queue against the same history and each send ACKs/[SEEN]
    replies for the messages they independently notice first -- doubling the
    replies the phone sees. That is the exact "duplicate messages" failure
    mode this guard exists to make structurally impossible rather than just
    unlikely (process dedup at the launcher level does not cover a bridge
    started by hand alongside a supervised one).

    Stale-lock detection: if the recorded PID is not alive, the lock is
    reclaimed rather than left blocking forever, so a crashed bridge does not
    permanently prevent restart.
    """
    RUN_LOCK_PATH.parent.mkdir(parents=True, exist_ok=True)
    existing = read_json(RUN_LOCK_PATH)
    if existing:
        pid = existing.get("pid")
        if pid and pid != os.getpid() and pid_alive(pid):
            sys.stderr.write(
                "[FAIL] another inbox_bridge instance is running (pid %d); exiting\n" % pid
            )
            return False
        # Stale lock: owning PID is gone (crash, kill, normal exit that
        # skipped cleanup). Safe to reclaim.
    write_json_atomic(RUN_LOCK_PATH, {"pid": os.getpid(), "started_at": iso(now_ms())})
    return True


def release_run_lock() -> None:
    existing = read_json(RUN_LOCK_PATH)
    if existing and existing.get("pid") == os.getpid():
        try:
            RUN_LOCK_PATH.unlink()
        except OSError:
            pass


# --------------------------------------------------------------------------
# commands
# --------------------------------------------------------------------------


def cmd_discover(args) -> int:
    api = args.api or DEFAULT_API
    try:
        messages = fetch_history(api, limit=300)
    except NodeUnreachable as exc:
        print("[FAIL] %s" % exc)
        print("       Start the node first: scm start")
        return 1
    peers = {}
    for message in messages:
        if str(message.get("direction", "")) != "received":
            continue
        peer = str(message.get("peer_id", ""))
        entry = peers.setdefault(peer, {"count": 0, "last_ts": 0, "last": ""})
        entry["count"] += 1
        ts = normalize_ts(message.get("timestamp"))
        if ts >= entry["last_ts"]:
            entry["last_ts"] = ts
            entry["last"] = str(message.get("content", ""))[:60]
    if not peers:
        print("[WARNING] No inbound messages in the last 300 history entries.")
        print("          Send one from the phone, then run discover again.")
        return 1
    print("Peers that have sent this node a message (most recent first):")
    print()
    for peer, entry in sorted(peers.items(), key=lambda kv: -kv[1]["last_ts"]):
        print("  peer_id : %s" % peer)
        print("  messages: %d, last at %s" % (entry["count"], iso(entry["last_ts"])))
        print("  last msg: %s" % entry["last"])
        print()
    print("Put the phone's peer_id into %s as allowed_peer_id." % CONFIG_PATH)
    return 0


def cmd_learn(args) -> int:
    """Identify the phone by having it speak, rather than by inference.

    Records the current inbound message IDs, then waits for a NEW one. Whoever
    sends the next message is, by construction, the device in your hand. This
    avoids guessing from peer ledgers (which bind identities to addresses that
    are not theirs) or from history heuristics (several nodes in this mesh look
    alike at a glance).
    """
    api = args.api or DEFAULT_API
    try:
        baseline = {str(m.get("id")) for m in fetch_history(api, limit=300)}
    except NodeUnreachable as exc:
        print("[FAIL] %s" % exc)
        print("       Start the node first.")
        return 1

    print("[INFO] Baseline taken (%d messages already in history)." % len(baseline))
    print()
    print("  >>> Now send any message to this node FROM THE ANDROID PHONE. <<<")
    print()
    print("[INFO] Waiting up to %ds for a new inbound message..." % args.timeout)

    deadline = time.time() + args.timeout
    while time.time() < deadline:
        time.sleep(2)
        try:
            messages = fetch_history(api, limit=300)
        except NodeUnreachable as exc:
            print("[WARNING] history read failed: %s" % exc)
            continue
        fresh = [
            m for m in messages
            if str(m.get("id")) not in baseline and str(m.get("direction")) == "received"
        ]
        if not fresh:
            continue
        fresh.sort(key=lambda m: normalize_ts(m.get("timestamp")))
        winner = fresh[-1]
        peer = str(winner.get("peer_id"))
        print()
        print("[OK] New inbound message from:")
        print("     peer_id : %s" % peer)
        print("     content : %s" % str(winner.get("content", ""))[:80])
        print("     at      : %s" % iso(normalize_ts(winner.get("timestamp"))))
        print()
        if args.write:
            existing = read_json(CONFIG_PATH) or {}
            existing.update({
                "allowed_peer_id": peer,
                "api": existing.get("api", DEFAULT_API),
                "poll_interval_secs": existing.get("poll_interval_secs", DEFAULT_POLL_SECS),
            })
            write_json_atomic(CONFIG_PATH, existing)
            print("[OK] wrote allow-list to %s" % CONFIG_PATH)
            print("[INFO] Confirm that identifier really is the phone before soaking on it.")
        else:
            print("[INFO] Re-run with --write to save this as the allow-list.")
        return 0

    print("[FAIL] No new inbound message within %ds." % args.timeout)
    print("       The phone may not be reaching this node at all -- that is itself")
    print("       the finding. Check `scripts/soak_supervisor.py status` for peer count.")
    return 1


def cmd_selftest(args) -> int:
    config = load_config(args.api)
    ok = True
    print("[INFO] config      : %s" % CONFIG_PATH)
    print("[INFO] allow-list  : %s" % ", ".join(config["allowed_peer_ids"]))
    print("[INFO] handoff dir : %s" % HANDOFF_TODO)
    print("[INFO] chat log dir: %s" % HANDOFF_CHAT_LOG_DIR)

    if not HANDOFF_TODO.is_dir():
        print("[FAIL] HANDOFF/todo does not exist")
        ok = False
    else:
        probe_file = HANDOFF_TODO / ".inbox_bridge_write_probe"
        try:
            write_text_atomic(probe_file, "probe\n")
            probe_file.unlink()
            print("[OK]   handoff dir writable")
        except OSError as exc:
            print("[FAIL] handoff dir not writable: %s" % exc)
            ok = False

    probe = probe_node(config["api"])
    if not probe["health_ok"]:
        print("[FAIL] node not reachable at %s (%s)" % (config["api"], probe["error"]))
        print("       /health is a static literal, so this means the daemon is down.")
        ok = False
    else:
        print("[OK]   node reachable at %s" % config["api"])
        if probe["peer_count"] is None:
            print("[WARNING] peer count unavailable -- cannot confirm the mesh is up")
        elif probe["peer_count"] == 0:
            print("[WARNING] node has 0 connected peers -- HTTP is up but nothing is meshed")
        else:
            print("[OK]   %d connected peer(s)" % probe["peer_count"])

    state = load_state()
    last_rx = state.get("last_received_message_at")
    print(
        "[INFO] last inbound: %s" % (iso(last_rx) if last_rx else "none recorded yet")
    )
    print("[INFO] tickets on disk: %d" % count_tickets_on_disk())
    print("[INFO] tickets this session: %d, acks sent: %d, acks pending: %d" % (
        state.get("tickets_written_total", 0),
        state.get("acks_sent_total", 0),
        len(state.get("ack_pending", {})),
    ))
    print()
    print("[INFO] End-to-end proof is not this command: message the node from the")
    print("       phone and confirm an [ACK] comes back within ~10s.")
    return 0 if ok else 1


def cmd_once(args) -> int:
    config = load_config(args.api)
    state = load_state()
    status = new_status()
    written = drain_once(config, state, status)
    write_status(status, state)
    if not status.get("node_reachable"):
        print("[FAIL] node unreachable: %s" % status.get("last_error"))
        return 1
    print("[OK] drain complete, %d new ticket(s)" % written)
    return 0


def cmd_run(args) -> int:
    if not acquire_run_lock():
        return 1
    try:
        config = load_config(args.api)
        state = load_state()
        status = new_status()
        interval = int(config.get("poll_interval_secs", DEFAULT_POLL_SECS))
        print("[INFO] bridge running, polling %s every %ds" % (config["api"], interval))
        print("[INFO] allow-list: %s" % ", ".join(config["allowed_peer_ids"]))
        print("[INFO] status file: %s" % STATUS_PATH)
        print("[INFO] lock file: %s (pid %d)" % (RUN_LOCK_PATH, os.getpid()))
        while True:
            try:
                drain_once(config, state, status)
            except Exception as exc:  # keep the loop alive; the status file records it
                status["last_error"] = "loop error: %r" % exc
                status["node_reachable"] = False
            write_status(status, state)
            time.sleep(interval)
    finally:
        release_run_lock()


def cmd_status(args) -> int:
    status = read_json(STATUS_PATH)
    if status is None:
        print("[FAIL] no status file at %s -- the bridge has never run" % STATUS_PATH)
        return 1
    age = (now_ms() - int(status.get("last_loop_at_ms", 0))) / 1000
    print(json.dumps(status, indent=2))
    print()
    if age > 60:
        print("[FAIL] status is %.0fs stale -- the bridge is not running" % age)
        return 1
    if not status.get("node_reachable"):
        print("[FAIL] bridge alive but node unreachable: %s" % status.get("last_error"))
        return 1
    if status.get("acks_pending"):
        print("[WARNING] %d ACK(s) pending delivery" % status["acks_pending"])
    print("[OK] bridge alive, last loop %.0fs ago" % age)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--api", default=None, help="control API base URL")
    sub = parser.add_subparsers(dest="command", required=True)
    learn = sub.add_parser("learn", help="wait for the phone to send, then record it")
    learn.add_argument("--timeout", type=int, default=180)
    learn.add_argument("--write", action="store_true", help="save the result as the allow-list")
    learn.set_defaults(handler=cmd_learn)

    for name, handler in (
        ("discover", cmd_discover),
        ("selftest", cmd_selftest),
        ("once", cmd_once),
        ("run", cmd_run),
        ("status", cmd_status),
    ):
        sub.add_parser(name).set_defaults(handler=handler)
    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n[INFO] bridge stopped")
        sys.exit(0)
