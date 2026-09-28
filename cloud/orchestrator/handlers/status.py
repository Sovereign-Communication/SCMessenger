"""
SCMessenger Orchestrator — Status Reporting Handlers

Provides Telegram command handlers for:
- /status  — active sprint overview
- /cost    — rough cost estimates
- /logs    — worker log tails
"""

from __future__ import annotations

import logging
from datetime import datetime, timezone

from telegram import Bot

from cloud.orchestrator.db import (
    get_active_sprints,
    get_latest_heartbeats,
    get_sprint,
)
from cloud.orchestrator.worker_manager import get_worker_logs, list_workers

logger = logging.getLogger(__name__)

# Rough hourly costs for GCP Spot instances (USD)
SPOT_COST_PER_HOUR: dict[str, float] = {
    "e2-standard-2": 0.013,
    "e2-standard-4": 0.027,
    "e2-standard-8": 0.054,
    "e2-standard-16": 0.107,
    "n2-standard-4": 0.034,
    "n2-standard-8": 0.068,
    "c2-standard-4": 0.041,
}
DEFAULT_COST_PER_HOUR: float = 0.03


async def _send(bot: Bot, chat_id: int, text: str) -> None:
    """Send a Telegram message, truncating if necessary."""
    if len(text) > 4000:
        text = text[:4000] + "\n… (truncated)"
    try:
        await bot.send_message(chat_id=chat_id, text=text, parse_mode="Markdown")
    except Exception:
        await bot.send_message(chat_id=chat_id, text=text)


# ---------------------------------------------------------------------------
# /status
# ---------------------------------------------------------------------------

async def handle_status(bot: Bot, chat_id: int) -> None:
    """Report all active sprints with their last heartbeats and worker zones.

    Args:
        bot: Telegram Bot instance.
        chat_id: Telegram chat to respond in.
    """
    sprints = await get_active_sprints()

    if not sprints:
        await _send(bot, chat_id, "📭 No active sprints.")
        return

    lines: list[str] = ["📊 *Active Sprints*\n"]

    for sprint in sprints:
        status_emoji = {
            "pending": "🟡",
            "running": "🟢",
            "preempted": "🟠",
        }.get(sprint.status, "⚪")

        lines.append(
            f"{status_emoji} `{sprint.id}` — *{sprint.status}*\n"
            f"   📝 {sprint.task_prompt}\n"
            f"   🌿 `{sprint.git_branch}` | 🖥️ {sprint.platform or 'linux'}"
        )

        if sprint.worker_zone:
            lines.append(f"   🌍 `{sprint.worker_zone}` | 📡 `{sprint.worker_ip}`")

        # Show last heartbeat
        heartbeats = await get_latest_heartbeats(sprint.id, limit=1)
        if heartbeats:
            hb = heartbeats[0]
            lines.append(f"   💓 [{hb.phase}] {hb.message}")

        lines.append("")  # blank separator

    # Also list raw workers if any are orphaned
    workers = await list_workers()
    if workers:
        lines.append(f"\n🔧 *GCP Workers*: {len(workers)} running")
        for w in workers:
            lines.append(f"   • `{w['name']}` — {w['status']} in `{w['zone']}`")

    await _send(bot, chat_id, "\n".join(lines))


# ---------------------------------------------------------------------------
# /cost
# ---------------------------------------------------------------------------

async def handle_cost(bot: Bot, chat_id: int) -> None:
    """Report rough cost estimates for active and recent sprints.

    Estimates are based on sprint duration × Spot hourly rates.

    Args:
        bot: Telegram Bot instance.
        chat_id: Telegram chat to respond in.
    """
    sprints = await get_active_sprints()

    if not sprints:
        await _send(bot, chat_id, "💰 No active sprints — $0.00 estimated.")
        return

    total_cost = 0.0
    lines: list[str] = ["💰 *Cost Estimates*\n"]
    now = datetime.now(timezone.utc)

    for sprint in sprints:
        try:
            created = datetime.fromisoformat(sprint.created_at)
            hours = (now - created).total_seconds() / 3600.0
        except (ValueError, TypeError):
            hours = 0.0

        cost_rate = DEFAULT_COST_PER_HOUR  # TODO: derive from actual machine_type
        cost = hours * cost_rate
        total_cost += cost

        lines.append(
            f"• `{sprint.id}` — {hours:.1f}h × ${cost_rate:.3f}/h = *${cost:.3f}*"
        )

    lines.append(f"\n📊 *Total estimated*: *${total_cost:.3f}*")
    lines.append("\n_Estimates based on Spot pricing; actual costs may differ._")

    await _send(bot, chat_id, "\n".join(lines))


# ---------------------------------------------------------------------------
# /logs
# ---------------------------------------------------------------------------

async def handle_logs(bot: Bot, chat_id: int, sprint_id: str = "") -> None:
    """Fetch and display the last 50 lines of a worker's logs.

    If *sprint_id* is empty, shows logs for all active sprints.

    Args:
        bot: Telegram Bot instance.
        chat_id: Telegram chat to respond in.
        sprint_id: Optional specific sprint to get logs for.
    """
    if sprint_id:
        sprint = await get_sprint(sprint_id)
        if sprint is None:
            await _send(bot, chat_id, f"❌ Sprint `{sprint_id}` not found.")
            return
        sprints_to_log = [sprint]
    else:
        sprints_to_log = await get_active_sprints()

    if not sprints_to_log:
        await _send(bot, chat_id, "📭 No active sprints to fetch logs from.")
        return

    for sprint in sprints_to_log:
        if not sprint.worker_zone or not sprint.worker_ip:
            await _send(bot, chat_id,
                         f"📋 `{sprint.id}` — no worker assigned (status: {sprint.status})")
            continue

        logs = await get_worker_logs(sprint.id, sprint.worker_zone, lines=50)
        header = f"📋 *Logs for sprint* `{sprint.id}`\n\n"
        await _send(bot, chat_id, header + f"```\n{logs}\n```")
