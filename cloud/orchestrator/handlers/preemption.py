"""
SCMessenger Orchestrator — Preemption Recovery Handler

When a GCP Spot VM is preempted, this handler:
1. Updates the sprint record with the preempted status
2. Selects a new zone (excluding the one that was preempted)
3. Spawns a replacement worker
4. Notifies the originating Telegram chat
"""

from __future__ import annotations

import logging
import os

from telegram import Bot

from cloud.orchestrator.db import (
    add_heartbeat,
    get_sprint,
    update_sprint,
)
from cloud.orchestrator.worker_manager import spawn_worker

logger = logging.getLogger(__name__)


async def handle_preemption(
    bot: Bot,
    sprint_id: str,
    git_sha: str = "",
    git_branch: str = "",
) -> bool:
    """Recover from a Spot VM preemption event.

    The preempted worker is expected to have already saved its state
    (e.g., git stash + push) before calling this endpoint.

    Args:
        bot: Telegram Bot instance for notifications.
        sprint_id: The sprint whose worker was preempted.
        git_sha: Last committed SHA on the worker (for resumption).
        git_branch: Branch name (falls back to the sprint's stored branch).

    Returns:
        ``True`` if recovery succeeded, ``False`` otherwise.
    """
    sprint = await get_sprint(sprint_id)
    if sprint is None:
        logger.error("Preemption callback for unknown sprint %s", sprint_id)
        return False

    old_zone = sprint.worker_zone
    branch = git_branch or sprint.git_branch
    chat_id = sprint.chat_id

    logger.warning(
        "Sprint %s preempted in zone %s — attempting recovery", sprint_id, old_zone,
    )

    # Mark current state
    await update_sprint(sprint_id, status="preempted", git_sha=git_sha)
    await add_heartbeat(sprint_id, "preempted", f"Preempted in {old_zone}")

    if chat_id:
        try:
            await bot.send_message(
                chat_id=chat_id,
                text=(
                    f"⚠️ *Sprint {sprint_id}* — worker preempted\n\n"
                    f"🌍 Zone: `{old_zone}`\n"
                    f"🔄 Attempting recovery in a new zone…"
                ),
                parse_mode="Markdown",
            )
        except Exception as exc:
            logger.warning("Failed to send preemption notification: %s", exc)

    # Try to respawn in a different zone
    exclude_zones = {old_zone}
    try:
        worker_ip, new_zone = await spawn_worker(
            sprint_id=sprint_id,
            task_prompt=sprint.task_prompt,
            git_branch=branch,
            exclude_zones=exclude_zones,
        )
        await update_sprint(
            sprint_id,
            status="running",
            worker_ip=worker_ip,
            worker_zone=new_zone,
            worker_instance=f"scm-worker-{sprint_id}",
            git_sha=git_sha,
        )
        await add_heartbeat(
            sprint_id, "recovered", f"Respawned in {new_zone} at {worker_ip}",
        )

        if chat_id:
            try:
                await bot.send_message(
                    chat_id=chat_id,
                    text=(
                        f"✅ *Sprint {sprint_id}* — recovered!\n\n"
                        f"🌍 New zone: `{new_zone}`\n"
                        f"📡 IP: `{worker_ip}`\n\n"
                        f"Worker resuming from commit `{git_sha[:8] if git_sha else 'HEAD'}`…"
                    ),
                    parse_mode="Markdown",
                )
            except Exception as exc:
                logger.warning("Failed to send recovery notification: %s", exc)

        logger.info(
            "Sprint %s recovered: %s -> %s (%s)", sprint_id, old_zone, new_zone, worker_ip,
        )
        return True

    except RuntimeError as exc:
        await update_sprint(sprint_id, status="failed", error_message=str(exc))
        await add_heartbeat(sprint_id, "failed", f"Recovery failed: {exc}")

        if chat_id:
            try:
                await bot.send_message(
                    chat_id=chat_id,
                    text=(
                        f"❌ *Sprint {sprint_id}* — recovery failed\n\n"
                        f"All zones exhausted or unavailable.\n"
                        f"Error: `{exc}`"
                    ),
                    parse_mode="Markdown",
                )
            except Exception as notify_exc:
                logger.warning("Failed to send failure notification: %s", notify_exc)

        logger.error("Sprint %s recovery failed: %s", sprint_id, exc)
        return False
