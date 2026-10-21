"""
SCMessenger Orchestrator — FastAPI Web Server & Telegram Bot Gateway

This is the main entry point for the orchestrator. It sets up FastAPI endpoints
for worker heartbeats, preemption alerts, and GitHub Action callbacks, and wraps
python-telegram-bot v20+ with webhooks to receive Telegram commands.
"""

from __future__ import annotations

import logging
import os
import sys
from contextlib import asynccontextmanager
from typing import Any, Dict

import uvicorn
from fastapi import FastAPI, Request, Response, status
from pydantic import BaseModel, Field
from telegram import Bot, Update
from telegram.ext import Application, CommandHandler, ContextTypes

from cloud.orchestrator.db import init_db, add_heartbeat, update_sprint, get_sprint
from cloud.orchestrator.handlers.sprint import handle_sprint, handle_build
from cloud.orchestrator.handlers.preemption import handle_preemption
from cloud.orchestrator.handlers.status import handle_status, handle_cost, handle_logs
from cloud.orchestrator.worker_manager import delete_worker

# ---------------------------------------------------------------------------
# Logging & Environment Setup
# ---------------------------------------------------------------------------

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("scm_orchestrator")

TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN")
ORCHESTRATOR_URL = os.getenv("ORCHESTRATOR_URL")  # e.g. https://your-domain.ngrok.app
ADMIN_CHAT_ID = os.getenv("ADMIN_CHAT_ID")        # optional restriction to specific chat/user

if not TELEGRAM_BOT_TOKEN:
    logger.critical("TELEGRAM_BOT_TOKEN environment variable not set")
    raise ValueError("TELEGRAM_BOT_TOKEN is required")

# Global Telegram application and bot references
telegram_app: Application = Application.builder().token(TELEGRAM_BOT_TOKEN).build()
bot: Bot = telegram_app.bot


# ---------------------------------------------------------------------------
# Pydantic Models for Web Endpoints
# ---------------------------------------------------------------------------

class HeartbeatPayload(BaseModel):
    sprint_id: str = Field(..., description="Unique sprint ID")
    phase: str = Field(..., description="Current worker phase (boot|clone|build|test|agent|commit|cleanup)")
    message: str = Field(..., description="Detailed status message")


class PreemptedPayload(BaseModel):
    sprint_id: str = Field(..., description="Preempted sprint ID")
    git_sha: str = Field("", description="Last successfully pushed commit SHA")
    git_branch: str = Field("", description="Working git branch")


class GHACallbackPayload(BaseModel):
    sprint_id: str = Field(..., description="Sprint ID")
    platform: str = Field(..., description="Target platform (ios|macos)")
    status: str = Field(..., description="Build/test status (success|failure)")
    message: str = Field("", description="Summary message of the run")
    artifact_url: str = Field("", description="Optional URL to download build output or logs")


# ---------------------------------------------------------------------------
# Telegram Bot Command Handlers
# ---------------------------------------------------------------------------

def check_chat_allowed(chat_id: int) -> bool:
    """Validate if the request comes from an authorized user/chat."""
    if not ADMIN_CHAT_ID:
        return True
    try:
        return chat_id == int(ADMIN_CHAT_ID)
    except ValueError:
        return False


async def help_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Send help message."""
    if not update.effective_chat:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return

    help_text = (
        "🤖 *SCMessenger Orchestrator Bot*\n\n"
        "*Commands*:\n"
        "• `/sprint <prompt>` — Spawn a Spot worker to execute a coding/test sprint\n"
        "• `/build <platform> [branch]` — Compile target (android, ios, wasm, linux, windows, macos)\n"
        "• `/status` — View active sprints and running GCP workers\n"
        "• `/logs [sprint_id]` — Retrieve latest 50 lines of logs from the active worker\n"
        "• `/kill <sprint_id>` — Manually terminate the worker VM for a sprint\n"
        "• `/cost` — Show rough cost breakdown for active/recent runs\n"
        "• `/help` — Display this usage help"
    )
    await update.message.reply_text(help_text, parse_mode="Markdown")


async def sprint_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Dispatch a custom task sprint."""
    if not update.effective_chat or not update.message:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return

    if not context.args:
        await update.message.reply_text("❌ Usage: `/sprint <task description prompt>`", parse_mode="Markdown")
        return

    task_prompt = " ".join(context.args)
    # Defaulting to main branch, linux platform for generic sprints
    await handle_sprint(
        bot=bot,
        task_prompt=task_prompt,
        chat_id=chat_id,
        git_branch="main",
        platform="linux",
    )


async def build_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Dispatch a build shortcut."""
    if not update.effective_chat or not update.message:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return

    if not context.args:
        await update.message.reply_text(
            "❌ Usage: `/build <platform> [branch]`\nPlatforms: `android`, `ios`, `wasm`, `linux`, `windows`, `macos`",
            parse_mode="Markdown",
        )
        return

    platform = context.args[0].lower()
    branch = context.args[1] if len(context.args) > 1 else "main"

    await handle_build(
        bot=bot,
        platform=platform,
        chat_id=chat_id,
        git_branch=branch,
    )


async def status_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Check system status."""
    if not update.effective_chat:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return
    await handle_status(bot=bot, chat_id=chat_id)


async def cost_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Get sprint cost estimates."""
    if not update.effective_chat:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return
    await handle_cost(bot=bot, chat_id=chat_id)


async def logs_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Tail worker logs."""
    if not update.effective_chat or not update.message:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return

    sprint_id = context.args[0] if context.args else ""
    await handle_logs(bot=bot, chat_id=chat_id, sprint_id=sprint_id)


async def kill_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Kill worker for a sprint."""
    if not update.effective_chat or not update.message:
        return
    chat_id = update.effective_chat.id
    if not check_chat_allowed(chat_id):
        return

    if not context.args:
        await update.message.reply_text("❌ Usage: `/kill <sprint_id>`", parse_mode="Markdown")
        return

    sprint_id = context.args[0]
    sprint = await get_sprint(sprint_id)
    if not sprint:
        await update.message.reply_text(f"❌ Sprint `{sprint_id}` not found.", parse_mode="Markdown")
        return

    await update.message.reply_text(f"⏳ Terminating worker VM for sprint `{sprint_id}`...", parse_mode="Markdown")
    try:
        await delete_worker(sprint_id, zone=sprint.worker_zone)
        await update_sprint(sprint_id, status="failed", error_message="Manually terminated by user command")
        await update.message.reply_text(f"💀 Worker for sprint `{sprint_id}` has been destroyed.", parse_mode="Markdown")
    except Exception as e:
        logger.error("Error killing worker for sprint %s: %s", sprint_id, e)
        await update.message.reply_text(f"⚠️ Error killing worker: `{e}`", parse_mode="Markdown")


# ---------------------------------------------------------------------------
# FastAPI Lifecycle & App Definition
# ---------------------------------------------------------------------------

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Initialize DB
    await init_db()

    # Set webhook on startup
    if ORCHESTRATOR_URL:
        webhook_url = f"{ORCHESTRATOR_URL}/webhook"
        logger.info("Setting Telegram Webhook to %s", webhook_url)
        await bot.set_webhook(url=webhook_url)
    else:
        logger.warning("ORCHESTRATOR_URL not set; webhook NOT registered. Run locally or with public tunnel.")

    # Register Bot commands in GAE framework context
    telegram_app.add_handler(CommandHandler("help", help_command))
    telegram_app.add_handler(CommandHandler("sprint", sprint_command))
    telegram_app.add_handler(CommandHandler("build", build_command))
    telegram_app.add_handler(CommandHandler("status", status_command))
    telegram_app.add_handler(CommandHandler("cost", cost_command))
    telegram_app.add_handler(CommandHandler("logs", logs_command))
    telegram_app.add_handler(CommandHandler("kill", kill_command))

    await telegram_app.initialize()
    await telegram_app.start()

    yield

    # Teardown
    await telegram_app.stop()
    await telegram_app.shutdown()


app = FastAPI(
    title="SCMessenger Cloud Orchestrator",
    description="Decoupled Orchestrator for cloud-based multi-platform emulation, testing, and AI-driven development.",
    version="1.0.0",
    lifespan=lifespan,
)


# ---------------------------------------------------------------------------
# API Endpoints
# ---------------------------------------------------------------------------

@app.get("/health")
def health_check() -> Dict[str, str]:
    """Simple health check endpoint."""
    return {"status": "ok"}


@app.post("/webhook")
async def telegram_webhook(request: Request) -> Response:
    """Telegram webhook integration endpoint."""
    data = await request.json()
    update = Update.de_json(data, bot)
    await telegram_app.process_update(update)
    return Response(status_code=status.HTTP_200_OK)


@app.post("/api/heartbeat")
async def worker_heartbeat(payload: HeartbeatPayload) -> Response:
    """Heartbeat callback invoked by running workers to report progress."""
    logger.info("Heartbeat received: [%s] phase=%s: %s", payload.sprint_id, payload.phase, payload.message)

    await add_heartbeat(payload.sprint_id, payload.phase, payload.message)

    sprint = await get_sprint(payload.sprint_id)
    if sprint and sprint.chat_id:
        # Check if this is a milestone worth pushing a notification for
        # E.g. build completion, test failures, Aider completion, or final cleanups
        milestones = {"build", "test", "agent", "commit", "cleanup", "failed"}
        if payload.phase in milestones or "failed" in payload.message.lower() or "error" in payload.message.lower():
            phase_emoji = {
                "build": "🏗️",
                "test": "🧪",
                "agent": "🤖",
                "commit": "💾",
                "cleanup": "🧹",
                "failed": "💥",
            }.get(payload.phase, "💓")

            try:
                await bot.send_message(
                    chat_id=sprint.chat_id,
                    text=f"{phase_emoji} *Sprint {payload.sprint_id}* — [{payload.phase.upper()}]\n{payload.message}",
                    parse_mode="Markdown",
                )
            except Exception as e:
                logger.warning("Failed to forward heartbeat notification: %s", e)

    return Response(status_code=status.HTTP_204_NO_CONTENT)


@app.post("/api/preempted")
async def worker_preempted(payload: PreemptedPayload) -> Response:
    """Callback triggered by preempted workers before they shut down."""
    logger.warning("Preemption callback triggered for sprint %s", payload.sprint_id)

    # Trigger recovery handler in background
    asyncio.create_task(
        handle_preemption(
            bot=bot,
            sprint_id=payload.sprint_id,
            git_sha=payload.git_sha,
            git_branch=payload.git_branch,
        )
    )

    return Response(status_code=status.HTTP_202_ACCEPTED)


@app.post("/api/gha-callback")
async def gha_callback(payload: GHACallbackPayload) -> Response:
    """Callback invoked by GitHub Actions upon completing iOS/macOS builds."""
    logger.info("GitHub Actions callback for sprint %s (%s): %s", payload.sprint_id, payload.platform, payload.status)

    sprint = await get_sprint(payload.sprint_id)
    if not sprint:
        return Response(status_code=status.HTTP_404_NOT_FOUND)

    status_str = "completed" if payload.status == "success" else "failed"
    await update_sprint(payload.sprint_id, status=status_str, error_message=payload.message if status_str == "failed" else "")
    await add_heartbeat(payload.sprint_id, "gha_callback", f"GitHub Actions {payload.platform} build {payload.status}. {payload.message}")

    if sprint.chat_id:
        emoji = "✅" if payload.status == "success" else "❌"
        msg_text = (
            f"{emoji} *Sprint {payload.sprint_id}* — GHA {payload.platform.upper()} Finished\n\n"
            f"Status: *{payload.status.upper()}*\n"
            f"Message: {payload.message}"
        )
        if payload.artifact_url:
            msg_text += f"\n📦 [Download Artifacts]({payload.artifact_url})"

        try:
            await bot.send_message(chat_id=sprint.chat_id, text=msg_text, parse_mode="Markdown")
        except Exception as e:
            logger.warning("Failed to send GHA callback notification: %s", e)

    return Response(status_code=status.HTTP_204_NO_CONTENT)


# ---------------------------------------------------------------------------
# Server Entry Point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    port = int(os.getenv("PORT", "8080"))
    uvicorn.run("cloud.orchestrator.main:app", host="0.0.0.0", port=port, reload=False)
