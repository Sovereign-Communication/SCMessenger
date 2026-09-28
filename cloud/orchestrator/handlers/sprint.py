"""
SCMessenger Orchestrator — Sprint Dispatch Handlers

Creates sprint records, spawns GCP Spot workers or triggers GitHub Actions
workflows, and sends status updates back to Telegram.
"""

from __future__ import annotations

import asyncio
import logging
import os
from typing import Optional

from telegram import Bot

from cloud.orchestrator.db import (
    add_heartbeat,
    create_sprint,
    new_sprint_id,
    update_sprint,
)
from cloud.orchestrator.worker_manager import spawn_worker

logger = logging.getLogger(__name__)

GITHUB_REPO: str = os.getenv("GITHUB_REPO", "nickshouse/SCMessenger")

# Platforms that run on GCP Spot workers
GCP_PLATFORMS = {"android", "wasm", "linux", "windows"}

# Platforms that run via GitHub Actions (Apple silicon required)
GHA_PLATFORMS = {"ios", "macos"}


async def _send(bot: Bot, chat_id: int, text: str) -> None:
    """Send a Telegram message, truncating if necessary."""
    # Telegram max message length is 4096 chars
    if len(text) > 4000:
        text = text[:4000] + "\n… (truncated)"
    try:
        await bot.send_message(chat_id=chat_id, text=text, parse_mode="Markdown")
    except Exception:
        # Fallback without markdown if parsing fails
        await bot.send_message(chat_id=chat_id, text=text)


# ---------------------------------------------------------------------------
# Sprint dispatch
# ---------------------------------------------------------------------------

async def handle_sprint(
    bot: Bot,
    task_prompt: str,
    chat_id: int,
    git_branch: str = "main",
    platform: str = "linux",
    machine_type: str = "e2-standard-4",
) -> str:
    """Create a new sprint, spawn a worker, and notify the chat.

    Args:
        bot: Telegram Bot instance for sending messages.
        task_prompt: Human-readable description of what to build/test.
        chat_id: Telegram chat ID to receive status updates.
        git_branch: Git branch to build (default: main).
        platform: Target platform (android, ios, wasm, linux, windows, macos).
        machine_type: GCE machine type for the worker.

    Returns:
        The newly created sprint ID.
    """
    sprint_id = new_sprint_id()
    logger.info("Starting sprint %s: %s (platform=%s)", sprint_id, task_prompt, platform)

    # Create DB record
    await create_sprint(
        sprint_id=sprint_id,
        task_prompt=task_prompt,
        git_branch=git_branch,
        platform=platform,
        chat_id=chat_id,
    )

    await _send(bot, chat_id, f"🚀 *Sprint {sprint_id}* created\n\n"
                f"📝 Task: {task_prompt}\n"
                f"🌿 Branch: `{git_branch}`\n"
                f"🖥️ Platform: {platform}\n\n"
                f"Spawning worker…")

    # Route to the right execution backend
    if platform in GHA_PLATFORMS:
        await _dispatch_gha(bot, sprint_id, platform, git_branch, chat_id)
    else:
        await _dispatch_gcp(bot, sprint_id, task_prompt, git_branch, machine_type, chat_id)

    return sprint_id


async def _dispatch_gcp(
    bot: Bot,
    sprint_id: str,
    task_prompt: str,
    git_branch: str,
    machine_type: str,
    chat_id: int,
) -> None:
    """Spawn a GCP Spot worker for the sprint."""
    try:
        worker_ip, zone = await spawn_worker(
            sprint_id=sprint_id,
            task_prompt=task_prompt,
            git_branch=git_branch,
            machine_type=machine_type,
        )
        await update_sprint(
            sprint_id,
            status="running",
            worker_ip=worker_ip,
            worker_zone=zone,
            worker_instance=f"scm-worker-{sprint_id}",
        )
        await add_heartbeat(sprint_id, "spawn", f"Worker online at {worker_ip} ({zone})")
        await _send(bot, chat_id,
                     f"✅ *Sprint {sprint_id}* — worker online\n\n"
                     f"🌍 Zone: `{zone}`\n"
                     f"📡 IP: `{worker_ip}`\n\n"
                     f"Worker is cloning and building…")
    except RuntimeError as exc:
        await update_sprint(sprint_id, status="failed", error_message=str(exc))
        await _send(bot, chat_id,
                     f"❌ *Sprint {sprint_id}* — failed to spawn worker\n\n`{exc}`")
        logger.error("Sprint %s spawn failed: %s", sprint_id, exc)


async def _dispatch_gha(
    bot: Bot,
    sprint_id: str,
    platform: str,
    git_branch: str,
    chat_id: int,
) -> None:
    """Trigger a GitHub Actions workflow for iOS/macOS builds.

    Uses the ``gh`` CLI to dispatch the workflow.  The GHA workflow is
    expected to call back to ``/api/gha-callback`` upon completion.
    """
    workflow_file = f"build-{platform}.yml"
    logger.info("Dispatching GHA workflow %s for sprint %s", workflow_file, sprint_id)

    cmd = [
        "gh", "workflow", "run", workflow_file,
        "--repo", GITHUB_REPO,
        "--ref", git_branch,
        "--field", f"sprint_id={sprint_id}",
        "--field", f"callback_url={os.getenv('ORCHESTRATOR_URL', '')}/api/gha-callback",
    ]

    proc = await asyncio.create_subprocess_exec(
        *cmd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate()

    if proc.returncode != 0:
        err_msg = stderr.decode().strip()
        await update_sprint(sprint_id, status="failed", error_message=err_msg)
        await _send(bot, chat_id,
                     f"❌ *Sprint {sprint_id}* — GHA dispatch failed\n\n`{err_msg}`")
        logger.error("GHA dispatch failed for sprint %s: %s", sprint_id, err_msg)
    else:
        await update_sprint(sprint_id, status="running")
        await add_heartbeat(sprint_id, "gha", f"Dispatched {workflow_file} on {git_branch}")
        await _send(bot, chat_id,
                     f"🍎 *Sprint {sprint_id}* — GitHub Actions triggered\n\n"
                     f"📦 Workflow: `{workflow_file}`\n"
                     f"🌿 Branch: `{git_branch}`\n\n"
                     f"Waiting for build result callback…")


# ---------------------------------------------------------------------------
# Build shortcut
# ---------------------------------------------------------------------------

async def handle_build(
    bot: Bot,
    platform: str,
    chat_id: int,
    git_branch: str = "main",
) -> Optional[str]:
    """Convenience wrapper to dispatch a platform-specific build.

    Args:
        bot: Telegram Bot instance.
        platform: One of android, ios, wasm, linux, windows, macos.
        chat_id: Telegram chat ID.
        git_branch: Branch to build.

    Returns:
        Sprint ID on success, ``None`` on validation error.
    """
    valid_platforms = GCP_PLATFORMS | GHA_PLATFORMS
    if platform not in valid_platforms:
        await _send(bot, chat_id,
                     f"❌ Unknown platform `{platform}`.\n"
                     f"Valid: {', '.join(sorted(valid_platforms))}")
        return None

    task_prompt = f"Build {platform} target from branch {git_branch}"
    return await handle_sprint(
        bot=bot,
        task_prompt=task_prompt,
        chat_id=chat_id,
        git_branch=git_branch,
        platform=platform,
    )
