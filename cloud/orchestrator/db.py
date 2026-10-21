"""
SCMessenger Orchestrator — Async SQLite Database Layer

Provides persistent storage for sprint records and heartbeat logs
using aiosqlite for non-blocking database access.
"""

from __future__ import annotations

import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

import aiosqlite

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

@dataclass
class Sprint:
    """Represents a single CI/CD sprint (build/test/deploy cycle)."""

    id: str
    task_prompt: str
    git_branch: str
    status: str = "pending"          # pending | running | completed | failed | preempted
    worker_zone: str = ""
    worker_ip: str = ""
    worker_instance: str = ""
    git_sha: str = ""
    platform: str = ""               # android | ios | wasm | linux | windows
    chat_id: int = 0
    cost_estimate_usd: float = 0.0
    created_at: str = ""
    updated_at: str = ""
    completed_at: str = ""
    error_message: str = ""


@dataclass
class Heartbeat:
    """A periodic status update received from a running worker."""

    id: int = 0
    sprint_id: str = ""
    phase: str = ""                  # clone | build | test | deploy | idle
    message: str = ""
    created_at: str = ""


# ---------------------------------------------------------------------------
# Database path
# ---------------------------------------------------------------------------

_DB_DIR = Path("/opt/scm-orchestrator/data")
_DB_PATH = _DB_DIR / "orchestrator.db"


def set_db_path(path: Path) -> None:
    """Override the default database path (useful for tests)."""
    global _DB_PATH, _DB_DIR
    _DB_PATH = path
    _DB_DIR = path.parent


def get_db_path() -> Path:
    """Return the current database file path."""
    return _DB_PATH


# ---------------------------------------------------------------------------
# Initialization
# ---------------------------------------------------------------------------

async def init_db() -> None:
    """Create tables if they do not already exist.

    Called once at application startup.
    """
    _DB_DIR.mkdir(parents=True, exist_ok=True)
    logger.info("Initializing database at %s", _DB_PATH)

    async with aiosqlite.connect(str(_DB_PATH)) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS sprints (
                id              TEXT PRIMARY KEY,
                task_prompt     TEXT NOT NULL,
                git_branch      TEXT NOT NULL DEFAULT 'main',
                status          TEXT NOT NULL DEFAULT 'pending',
                worker_zone     TEXT NOT NULL DEFAULT '',
                worker_ip       TEXT NOT NULL DEFAULT '',
                worker_instance TEXT NOT NULL DEFAULT '',
                git_sha         TEXT NOT NULL DEFAULT '',
                platform        TEXT NOT NULL DEFAULT '',
                chat_id         INTEGER NOT NULL DEFAULT 0,
                cost_estimate_usd REAL NOT NULL DEFAULT 0.0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL,
                completed_at    TEXT NOT NULL DEFAULT '',
                error_message   TEXT NOT NULL DEFAULT ''
            )
        """)
        await db.execute("""
            CREATE TABLE IF NOT EXISTS heartbeats (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                sprint_id   TEXT NOT NULL,
                phase       TEXT NOT NULL DEFAULT '',
                message     TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL,
                FOREIGN KEY (sprint_id) REFERENCES sprints(id)
            )
        """)
        await db.execute("""
            CREATE INDEX IF NOT EXISTS idx_heartbeats_sprint
            ON heartbeats(sprint_id, created_at DESC)
        """)
        await db.commit()

    logger.info("Database initialized successfully")


# ---------------------------------------------------------------------------
# Sprint CRUD
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    """Return the current UTC time as an ISO-8601 string."""
    return datetime.now(timezone.utc).isoformat()


def new_sprint_id() -> str:
    """Generate a new UUID-based sprint identifier."""
    return uuid.uuid4().hex[:12]


def _row_to_sprint(row: aiosqlite.Row) -> Sprint:
    """Map a database row to a Sprint dataclass."""
    return Sprint(
        id=row[0],
        task_prompt=row[1],
        git_branch=row[2],
        status=row[3],
        worker_zone=row[4],
        worker_ip=row[5],
        worker_instance=row[6],
        git_sha=row[7],
        platform=row[8],
        chat_id=row[9],
        cost_estimate_usd=row[10],
        created_at=row[11],
        updated_at=row[12],
        completed_at=row[13],
        error_message=row[14],
    )


async def create_sprint(
    sprint_id: str,
    task_prompt: str,
    git_branch: str = "main",
    *,
    platform: str = "",
    chat_id: int = 0,
) -> Sprint:
    """Insert a new sprint record and return it.

    Args:
        sprint_id: Unique sprint identifier (use ``new_sprint_id()``).
        task_prompt: Human-readable description of the task.
        git_branch: Git branch to build from.
        platform: Target platform (android, ios, wasm, linux, windows).
        chat_id: Telegram chat ID that initiated the sprint.

    Returns:
        The newly created Sprint object.
    """
    now = _now_iso()
    logger.info("Creating sprint %s on branch '%s'", sprint_id, git_branch)

    async with aiosqlite.connect(str(_DB_PATH)) as db:
        await db.execute(
            """
            INSERT INTO sprints
                (id, task_prompt, git_branch, status, platform, chat_id, created_at, updated_at)
            VALUES (?, ?, ?, 'pending', ?, ?, ?, ?)
            """,
            (sprint_id, task_prompt, git_branch, platform, chat_id, now, now),
        )
        await db.commit()

    return Sprint(
        id=sprint_id,
        task_prompt=task_prompt,
        git_branch=git_branch,
        status="pending",
        platform=platform,
        chat_id=chat_id,
        created_at=now,
        updated_at=now,
    )


async def update_sprint(sprint_id: str, **kwargs: Any) -> None:
    """Update one or more fields on an existing sprint.

    Args:
        sprint_id: The sprint to update.
        **kwargs: Column-value pairs to set (e.g. ``status='running'``).

    Raises:
        ValueError: If no keyword arguments are supplied.
    """
    if not kwargs:
        raise ValueError("update_sprint requires at least one keyword argument")

    # Always bump updated_at
    kwargs.setdefault("updated_at", _now_iso())

    allowed_columns = {f.name for f in Sprint.__dataclass_fields__.values()} - {"id"}
    bad_keys = set(kwargs.keys()) - allowed_columns
    if bad_keys:
        raise ValueError(f"Unknown sprint columns: {bad_keys}")

    set_clause = ", ".join(f"{col} = ?" for col in kwargs)
    values = list(kwargs.values()) + [sprint_id]

    logger.debug("Updating sprint %s: %s", sprint_id, kwargs)

    async with aiosqlite.connect(str(_DB_PATH)) as db:
        await db.execute(
            f"UPDATE sprints SET {set_clause} WHERE id = ?",  # noqa: S608
            values,
        )
        await db.commit()


async def get_sprint(sprint_id: str) -> Optional[Sprint]:
    """Fetch a single sprint by ID.

    Returns:
        The Sprint if found, otherwise ``None``.
    """
    async with aiosqlite.connect(str(_DB_PATH)) as db:
        cursor = await db.execute("SELECT * FROM sprints WHERE id = ?", (sprint_id,))
        row = await cursor.fetchone()
        return _row_to_sprint(row) if row else None


async def get_active_sprints() -> list[Sprint]:
    """Return all sprints with status 'pending' or 'running'."""
    async with aiosqlite.connect(str(_DB_PATH)) as db:
        cursor = await db.execute(
            "SELECT * FROM sprints WHERE status IN ('pending', 'running') ORDER BY created_at DESC"
        )
        rows = await cursor.fetchall()
        return [_row_to_sprint(r) for r in rows]


# ---------------------------------------------------------------------------
# Heartbeats
# ---------------------------------------------------------------------------

async def add_heartbeat(sprint_id: str, phase: str, message: str) -> None:
    """Record a heartbeat from a running worker.

    Args:
        sprint_id: The sprint this heartbeat belongs to.
        phase: Current worker phase (clone, build, test, deploy, idle).
        message: Free-form status message.
    """
    now = _now_iso()
    logger.debug("Heartbeat for %s [%s]: %s", sprint_id, phase, message)

    async with aiosqlite.connect(str(_DB_PATH)) as db:
        await db.execute(
            "INSERT INTO heartbeats (sprint_id, phase, message, created_at) VALUES (?, ?, ?, ?)",
            (sprint_id, phase, message, now),
        )
        await db.commit()


async def get_latest_heartbeats(sprint_id: str, limit: int = 10) -> list[Heartbeat]:
    """Retrieve the most recent heartbeats for a sprint.

    Args:
        sprint_id: The sprint to query.
        limit: Maximum number of heartbeats to return.

    Returns:
        List of Heartbeat objects, newest first.
    """
    async with aiosqlite.connect(str(_DB_PATH)) as db:
        cursor = await db.execute(
            "SELECT id, sprint_id, phase, message, created_at "
            "FROM heartbeats WHERE sprint_id = ? ORDER BY created_at DESC LIMIT ?",
            (sprint_id, limit),
        )
        rows = await cursor.fetchall()
        return [
            Heartbeat(id=r[0], sprint_id=r[1], phase=r[2], message=r[3], created_at=r[4])
            for r in rows
        ]
