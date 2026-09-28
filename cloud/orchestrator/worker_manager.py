"""
SCMessenger Orchestrator — GCP Spot Worker Lifecycle Manager

Manages creation, deletion, and monitoring of preemptible (Spot) VM
instances on Google Cloud Platform.  Workers are named by sprint ID
for easy correlation.
"""

from __future__ import annotations

import asyncio
import logging
import os
from typing import Optional

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

GCP_PROJECT: str = os.getenv("GCP_PROJECT", "scmessenger")

# Zones ordered by preference — cheaper / more-available regions first.
ZONE_PRIORITY: list[str] = [
    "us-central1-a",
    "us-central1-b",
    "us-central1-f",
    "us-west1-a",
    "us-west1-b",
    "us-east1-b",
    "us-east1-c",
    "europe-west1-b",
    "europe-west4-a",
]

DEFAULT_MACHINE_TYPE: str = "e2-standard-4"
DEFAULT_IMAGE_FAMILY: str = "scm-worker"
DEFAULT_IMAGE_PROJECT: str = GCP_PROJECT
WORKER_BOOT_DISK_SIZE: str = "50GB"
WORKER_LABEL: str = "scm-worker"


# ---------------------------------------------------------------------------
# Zone selection
# ---------------------------------------------------------------------------

def pick_next_zone(exclude: Optional[set[str]] = None) -> str:
    """Select the next preferred zone, skipping any in *exclude*.

    Args:
        exclude: Set of zone names that recently failed or were preempted.

    Returns:
        The best available zone string.

    Raises:
        RuntimeError: If all zones have been excluded.
    """
    exclude = exclude or set()
    for zone in ZONE_PRIORITY:
        if zone not in exclude:
            logger.info("Selected zone: %s (excluded: %s)", zone, exclude)
            return zone
    raise RuntimeError(f"All {len(ZONE_PRIORITY)} zones exhausted — cannot schedule worker")


# ---------------------------------------------------------------------------
# Instance naming
# ---------------------------------------------------------------------------

def _instance_name(sprint_id: str) -> str:
    """Derive a deterministic GCE instance name from a sprint ID."""
    return f"scm-worker-{sprint_id}"


# ---------------------------------------------------------------------------
# Worker lifecycle
# ---------------------------------------------------------------------------

async def spawn_worker(
    sprint_id: str,
    task_prompt: str,
    git_branch: str,
    machine_type: str = DEFAULT_MACHINE_TYPE,
    zone: Optional[str] = None,
    exclude_zones: Optional[set[str]] = None,
) -> tuple[str, str]:
    """Create a GCP Spot VM to execute a sprint task.

    The instance is configured with:
    - Spot (preemptible) provisioning for cost savings
    - Automatic termination action = STOP
    - Startup metadata carrying the sprint ID and task prompt

    Args:
        sprint_id: Unique sprint identifier.
        task_prompt: Task description passed as instance metadata.
        git_branch: Git branch for the worker to check out.
        machine_type: GCE machine type (default: e2-standard-4).
        zone: Explicit zone override; if ``None``, picks from priority list.
        exclude_zones: Zones to skip when auto-selecting.

    Returns:
        Tuple of (worker_ip, zone) on success.

    Raises:
        RuntimeError: If ``gcloud`` exits with a non-zero code.
    """
    if zone is None:
        zone = pick_next_zone(exclude_zones)

    instance_name = _instance_name(sprint_id)
    logger.info(
        "Spawning worker %s in %s (machine=%s, branch=%s)",
        instance_name, zone, machine_type, git_branch,
    )

    cmd = [
        "gcloud", "compute", "instances", "create", instance_name,
        f"--project={GCP_PROJECT}",
        f"--zone={zone}",
        f"--machine-type={machine_type}",
        f"--image-family={DEFAULT_IMAGE_FAMILY}",
        f"--image-project={DEFAULT_IMAGE_PROJECT}",
        f"--boot-disk-size={WORKER_BOOT_DISK_SIZE}",
        "--provisioning-model=SPOT",
        "--instance-termination-action=STOP",
        "--maintenance-policy=TERMINATE",
        "--no-restart-on-failure",
        f"--labels=role={WORKER_LABEL},sprint={sprint_id}",
        f"--metadata=sprint_id={sprint_id},task_prompt={task_prompt},git_branch={git_branch}",
        "--format=value(networkInterfaces[0].accessConfigs[0].natIP)",
        "--scopes=cloud-platform",
    ]

    proc = await asyncio.create_subprocess_exec(
        *cmd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate()

    if proc.returncode != 0:
        err_msg = stderr.decode().strip()
        logger.error("Failed to spawn worker %s: %s", instance_name, err_msg)
        raise RuntimeError(f"gcloud instance create failed (rc={proc.returncode}): {err_msg}")

    worker_ip = stdout.decode().strip()
    logger.info("Worker %s running at %s in %s", instance_name, worker_ip, zone)
    return worker_ip, zone


async def delete_worker(sprint_id: str, zone: str = "") -> None:
    """Terminate and delete the worker VM for a sprint.

    Args:
        sprint_id: Sprint whose worker should be destroyed.
        zone: The zone the worker lives in.  If empty, the command
              will attempt deletion without specifying a zone (requires
              gcloud to resolve it).
    """
    instance_name = _instance_name(sprint_id)
    logger.info("Deleting worker %s (zone=%s)", instance_name, zone or "auto")

    cmd = [
        "gcloud", "compute", "instances", "delete", instance_name,
        f"--project={GCP_PROJECT}",
        "--quiet",
    ]
    if zone:
        cmd.append(f"--zone={zone}")

    proc = await asyncio.create_subprocess_exec(
        *cmd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate()

    if proc.returncode != 0:
        err_msg = stderr.decode().strip()
        # Not fatal — the instance may already be gone (preempted, manually deleted)
        logger.warning("delete_worker %s returned rc=%d: %s", instance_name, proc.returncode, err_msg)
    else:
        logger.info("Worker %s deleted successfully", instance_name)


async def list_workers() -> list[dict[str, str]]:
    """List all active SCM worker instances in the project.

    Returns:
        List of dicts with keys: name, zone, status, ip.
    """
    cmd = [
        "gcloud", "compute", "instances", "list",
        f"--project={GCP_PROJECT}",
        f"--filter=labels.role={WORKER_LABEL}",
        "--format=csv[no-heading](name,zone,status,networkInterfaces[0].accessConfigs[0].natIP)",
    ]

    proc = await asyncio.create_subprocess_exec(
        *cmd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate()

    if proc.returncode != 0:
        logger.error("list_workers failed: %s", stderr.decode().strip())
        return []

    workers: list[dict[str, str]] = []
    for line in stdout.decode().strip().splitlines():
        parts = line.split(",")
        if len(parts) >= 4:
            workers.append({
                "name": parts[0],
                "zone": parts[1],
                "status": parts[2],
                "ip": parts[3],
            })
    return workers


async def get_worker_logs(sprint_id: str, zone: str, lines: int = 50) -> str:
    """Fetch the tail of the worker's sprint log via SSH.

    Args:
        sprint_id: Sprint whose worker logs to fetch.
        zone: GCE zone of the worker.
        lines: Number of log lines to retrieve (default 50).

    Returns:
        Log output as a string, or an error message on failure.
    """
    instance_name = _instance_name(sprint_id)
    logger.info("Fetching %d log lines from %s", lines, instance_name)

    cmd = [
        "gcloud", "compute", "ssh", instance_name,
        f"--project={GCP_PROJECT}",
        f"--zone={zone}",
        "--command", f"tail -n {lines} /var/log/scm-worker/sprint.log 2>/dev/null || echo '[no logs yet]'",
        "--quiet",
    ]

    try:
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=30.0)
    except asyncio.TimeoutError:
        logger.warning("SSH to %s timed out after 30s", instance_name)
        return f"[timeout] Could not reach worker {instance_name}"

    if proc.returncode != 0:
        err_msg = stderr.decode().strip()
        logger.warning("get_worker_logs %s failed: %s", instance_name, err_msg)
        return f"[error] {err_msg}"

    return stdout.decode().strip()
