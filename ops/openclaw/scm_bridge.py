#!/usr/bin/env python3
"""SCMessenger <-> OpenClaw bridge with a durable, fail-closed inbox."""
import json
import logging
import os
import signal
import sqlite3
import subprocess
import time
import threading
import contextlib
import urllib.error
import urllib.parse
import urllib.request

SCM = "http://127.0.0.1:9876"
AGENT_CMD = ["/home/ec2-user/.openclaw/bin/openclaw", "agent", "-m"]
ALLOWLIST_PATH = "/home/ec2-user/.openclaw/scm-bridge-allowlist.json"
SEEN_PATH = "/home/ec2-user/.openclaw/scm-bridge-seen.json"  # retained for path compatibility
DB_PATH = os.environ.get("SCM_BRIDGE_DB", os.path.join(os.path.dirname(SEEN_PATH), "scm-bridge.sqlite3"))
POLL_SECONDS = 5
AGENT_TIMEOUT = max(1, min(3600, int(os.environ.get("SCM_BRIDGE_AGENT_TIMEOUT", "300"))))
MAX_BATCH = 50
MAX_HISTORY = 2000
MAX_BACKOFF = 300

class HistoryOverflow(RuntimeError):
    pass

class AmbiguousSend(RuntimeError):
    pass

class RetryableSend(RuntimeError):
    pass


class AllowlistError(RuntimeError):
    pass


def load_allowlist(path=ALLOWLIST_PATH):
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
    except Exception as exc:
        raise AllowlistError("allowlist unavailable or invalid") from exc
    if not isinstance(data, dict) or not data:
        raise AllowlistError("allowlist must be a nonempty object")
    if any(not isinstance(k, str) or not k.strip() or not isinstance(v, str) for k, v in data.items()):
        raise AllowlistError("allowlist entries are invalid")
    return data


def http_json(path, method="GET", body=None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(SCM + path, data=data, method=method,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=15) as r:
        return json.loads(r.read().decode())


def peer_keys():
    try:
        d = http_json("/api/peers")
        return {p.get("peer_id"): ((p.get("triad") or {}).get("public_key_hex") or p.get("public_key_hex"))
                for p in d.get("peers", []) if p.get("peer_id") and ((p.get("triad") or {}).get("public_key_hex") or p.get("public_key_hex"))}
    except Exception:
        return {}


def ensure_contact(peer_id, public_key_hex):
    try:
        http_json("/api/contacts", "POST", {"peer_id": peer_id, "public_key": public_key_hex})
        return True
    except urllib.error.HTTPError as e:
        return e.code in (400, 409)
    except Exception:
        return False


def send_reply(peer_id, text, keys):
    # Provision only with a known public key; an empty key can corrupt an existing contact.
    if keys.get(peer_id) and not ensure_contact(peer_id, keys[peer_id]):
        raise RetryableSend("contact provisioning failed before send")
    try:
        result = http_json("/api/send", "POST", {"recipient": peer_id, "message": text})
    except urllib.error.URLError as exc:
        if isinstance(exc.reason, ConnectionRefusedError):
            raise RetryableSend("connection refused before send") from exc
        raise AmbiguousSend("send outcome unknown") from exc
    except Exception as exc:
        raise AmbiguousSend("send outcome unknown") from exc
    if not isinstance(result, dict) or not result.get("message_id"):
        raise AmbiguousSend("send response lacks message id")
    return result


def agent_turn(prompt):
    kwargs = dict(stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if os.name == "nt":
        kwargs["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    else:
        kwargs["start_new_session"] = True
    proc = subprocess.Popen(AGENT_CMD + [prompt], **kwargs)
    try:
        stdout, stderr = proc.communicate(timeout=AGENT_TIMEOUT)
    except subprocess.TimeoutExpired:
        if os.name == "nt":
            proc.send_signal(signal.CTRL_BREAK_EVENT)
        else:
            os.killpg(proc.pid, signal.SIGTERM)
        try:
            stdout, stderr = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            if os.name == "nt":
                proc.kill()
            else:
                os.killpg(proc.pid, signal.SIGKILL)
            stdout, stderr = proc.communicate()
        raise TimeoutError("agent turn timed out; side effects uncertain")
    text = (stdout or "").strip()
    if proc.returncode != 0:
        raise RuntimeError("agent turn failed; side effects uncertain")
    if not text:
        raise RuntimeError("agent turn returned empty reply; side effects uncertain")
    return text


def connect_db(path=DB_PATH):
    os.umask(0o077)
    os.makedirs(os.path.dirname(os.path.abspath(path)), mode=0o700, exist_ok=True)
    db = sqlite3.connect(path, timeout=10)
    db.row_factory = sqlite3.Row
    db.execute("PRAGMA journal_mode=WAL")
    db.execute("PRAGMA synchronous=FULL")
    db.executescript("""
      CREATE TABLE IF NOT EXISTS inbox (
        message_id TEXT PRIMARY KEY, peer_id TEXT NOT NULL, body TEXT NOT NULL,
        state TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0,
        next_at REAL NOT NULL DEFAULT 0, reply TEXT, updated REAL NOT NULL
      );
      CREATE TABLE IF NOT EXISTS outbox (
        message_id TEXT PRIMARY KEY, peer_id TEXT NOT NULL, reply TEXT NOT NULL,
        state TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0,
        next_at REAL NOT NULL DEFAULT 0, updated REAL NOT NULL
      );
      CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    """)
    db.commit()
    if "remote_id" not in {r[1] for r in db.execute("PRAGMA table_info(outbox)")}:
        db.execute("ALTER TABLE outbox ADD COLUMN remote_id TEXT")
        db.commit()
    try:
        os.chmod(path, 0o600)
        if os.name != "nt":
            os.chmod(path + "-wal", 0o600)
            os.chmod(path + "-shm", 0o600)
    except OSError:
        pass
    return db


def migrate_legacy_seen(db, path=SEEN_PATH):
    if db.execute("SELECT 1 FROM metadata WHERE key='seen_migrated'").fetchone():
        return
    try:
        with open(path, encoding="utf-8") as f:
            raw = json.load(f)
    except FileNotFoundError:
        raw = []
    if isinstance(raw, dict):
        raw = raw.get("seen", raw.get("ids", []))
    if not isinstance(raw, list) or any(not isinstance(x, (str, int)) for x in raw):
        raise ValueError("legacy seen file invalid")
    now = time.time()
    with db:
        for mid in raw:
            db.execute("INSERT OR IGNORE INTO inbox(message_id,peer_id,body,state,updated) VALUES(?,'','','ignored',?)", (str(mid), now))
        db.execute("INSERT INTO metadata(key,value) VALUES('seen_migrated','1')")


def poll_peer(db, peer_id, allow):
    limit = MAX_BATCH
    while True:
        path = "/api/history?" + urllib.parse.urlencode({"peer_id": peer_id, "limit": limit})
        messages = http_json(path).get("messages", [])
        if not isinstance(messages, list):
            raise ValueError("invalid history response")
        known = any(db.execute("SELECT 1 FROM inbox WHERE message_id=?", (str(m.get("id") or m.get("message_id")),)).fetchone() for m in messages)
        if len(messages) < limit or known:
            return ingest(db, messages, allow)
        if limit >= MAX_HISTORY:
            raise HistoryOverflow("history window exhausted for authorized peer")
        limit = min(MAX_HISTORY, limit * 2)


def poll_all(db, allow):
    return sum(poll_peer(db, peer_id, allow) for peer_id in allow)


def ingest(db, messages, allow):
    now = time.time()
    count = 0
    for m in messages:
        mid = m.get("id") or m.get("message_id")
        if not isinstance(mid, (str, int)) or not str(mid):
            continue
        direction = str(m.get("direction", "")).lower()
        if direction and direction not in ("inbound", "incoming", "received"):
            continue
        sender = m.get("peer_id") or (m.get("sender") or {}).get("peer_id")
        if not isinstance(sender, str) or not sender:
            continue
        state = "queued" if sender in allow else "ignored"
        body = m.get("content") or m.get("body") or ""
        db.execute("INSERT OR IGNORE INTO inbox(message_id,peer_id,body,state,updated) VALUES(?,?,?,?,?)",
                   (str(mid), sender, str(body), state, now))
        count += db.execute("SELECT changes()").fetchone()[0]
    db.commit()
    return count


def claim(db):
    db.execute("BEGIN IMMEDIATE")
    row = db.execute("SELECT * FROM inbox WHERE state='queued' AND next_at<=? ORDER BY updated,message_id LIMIT 1", (time.time(),)).fetchone()
    if row:
        db.execute("UPDATE inbox SET state='running', attempts=attempts+1, updated=? WHERE message_id=?", (time.time(), row["message_id"]))
    db.commit()
    return row


def process_one(db, turn=agent_turn, allow=None):
    row = claim(db)
    if not row:
        return False
    if allow is not None:
        try:
            current = load_allowlist()
        except AllowlistError:
            current = {}
        if row["peer_id"] not in current:
            db.execute("UPDATE inbox SET state='ignored', updated=? WHERE message_id=?", (time.time(), row["message_id"]))
            db.commit()
            return True
    try:
        reply = turn(row["body"])
    except Exception:
        # A failed/interrupted invocation may have side effects; never run it again automatically.
        db.execute("UPDATE inbox SET state='uncertain', updated=? WHERE message_id=?", (time.time(), row["message_id"]))
        db.commit()
        return True
    now = time.time()
    db.execute("BEGIN IMMEDIATE")
    db.execute("UPDATE inbox SET state='replied', reply=?, updated=? WHERE message_id=?", (reply, now, row["message_id"]))
    db.execute("INSERT OR REPLACE INTO outbox(message_id,peer_id,reply,state,next_at,updated) VALUES(?,?,?,'pending',0,?)",
               (row["message_id"], row["peer_id"], reply, now))
    db.commit()
    return True


def deliver_one(db, keys, send=send_reply):
    row = db.execute("SELECT * FROM outbox WHERE state='pending' AND next_at<=? ORDER BY updated,message_id LIMIT 1", (time.time(),)).fetchone()
    if not row:
        return False
    try:
        db.execute("UPDATE outbox SET state='sending', updated=? WHERE message_id=?", (time.time(), row["message_id"]))
        db.commit()
        result = send(row["peer_id"], row["reply"], keys)
        if not isinstance(result, dict) or not result.get("message_id"):
            raise AmbiguousSend("send response lacks message id")
    except RetryableSend:
        attempts = row["attempts"] + 1
        delay = min(MAX_BACKOFF, 2 ** min(attempts, 8))
        db.execute("UPDATE outbox SET state='pending', attempts=?, next_at=?, updated=? WHERE message_id=?",
                   (attempts, time.time() + delay, time.time(), row["message_id"]))
        db.commit()
    except Exception:
        db.execute("UPDATE outbox SET state='uncertain', updated=? WHERE message_id=?", (time.time(), row["message_id"]))
        db.commit()
    else:
        db.execute("UPDATE outbox SET state=?, remote_id=?, attempts=attempts+1, updated=? WHERE message_id=?",
                   ("delivered" if result.get("delivered") is True else "sent", str(result["message_id"]), time.time(), row["message_id"]))
        db.commit()
    return True


def confirm_outbox(db):
    for row in db.execute("SELECT message_id,remote_id FROM outbox WHERE state='sent' AND remote_id IS NOT NULL").fetchall():
        try:
            receipt = http_json("/api/send/" + urllib.parse.quote(row["remote_id"], safe=""))
        except Exception:
            continue
        if receipt.get("message_id") == row["remote_id"] and receipt.get("delivered") is True:
            db.execute("UPDATE outbox SET state='delivered', updated=? WHERE message_id=?", (time.time(), row["message_id"]))
    db.commit()


def run():
    with instance_lock(DB_PATH):
        load_allowlist()
        db = connect_db()
        migrate_legacy_seen(db)
        # Only the lock owner can recover rows from an interrupted process.
        db.execute("UPDATE inbox SET state='uncertain', updated=? WHERE state='running'", (time.time(),))
        db.execute("UPDATE outbox SET state='uncertain', updated=? WHERE state='sending'", (time.time(),))
        db.commit()
        worker = threading.Thread(target=worker_loop, daemon=True)
        worker.start()
        while True:
            try:
                poll_all(db, load_allowlist())
            except HistoryOverflow:
                logging.error("HistoryOverflow: authorized peer history exceeded bounded window; ingestion blocked")
            except Exception:
                logging.exception("bridge poll failed")
            time.sleep(POLL_SECONDS)


@contextlib.contextmanager
def instance_lock(path):
    if os.name == "nt":
        raise RuntimeError("single-instance lock requires Linux flock")
    import fcntl
    lock_path = path + ".lock"
    os.makedirs(os.path.dirname(os.path.abspath(lock_path)), exist_ok=True)
    with open(lock_path, "a+b") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise RuntimeError("bridge already running") from exc
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def worker_loop():
    db = connect_db()
    while True:
        try:
            load_allowlist()
            confirm_outbox(db)
            deliver_one(db, peer_keys())
            process_one(db, allow=True)
        except Exception:
            logging.exception("bridge worker failed")
        time.sleep(POLL_SECONDS)


if __name__ == "__main__":
    run()
