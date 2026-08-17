#!/bin/bash

# Dynamic resource management and scaling

set -e

CONFIG_DIR=".claude"
LOG_DIR=".claude/logs"
STATE_DIR=".claude/state"

mkdir -p "$LOG_DIR" "$STATE_DIR"

# Load configuration (kept as a source-compatible helper for callers).
load_config() {
    if [ -f "$CONFIG_DIR/orchestration_config.json" ]; then
        cat "$CONFIG_DIR/orchestration_config.json"
    else
        echo "{}"
    fi
}

config_value() {
    local key="$1"
    local default="$2"
    if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
        echo "$default"
        return 0
    fi
    "$PYTHON_BIN" - "$CONFIG_DIR/orchestration_config.json" "$key" "$default" <<'PY'
import json
import sys
from pathlib import Path

path, key, default = sys.argv[1:]
try:
    value = json.loads(Path(path).read_text(encoding="utf-8"))
    for part in key.split("."):
        value = value[part]
    if isinstance(value, (dict, list)) or value is None:
        raise ValueError
    print(value)
except (OSError, json.JSONDecodeError, KeyError, TypeError, ValueError):
    print(default)
PY
}

# System monitoring functions (Windows compatible)
get_cpu_usage() {
    # Get CPU usage percentage for Windows
    local cpu_usage=$(wmic cpu get loadpercentage 2>/dev/null | awk 'NR==2 {print $1}')
    echo "${cpu_usage:-0}"
}

get_memory_usage() {
    # Get memory usage percentage for Windows
    local mem_info=$(wmic OS get FreePhysicalMemory,TotalVisibleMemorySize /value 2>/dev/null)
    local free_mem=$(echo "$mem_info" | grep FreePhysicalMemory | cut -d'=' -f2)
    local total_mem=$(echo "$mem_info" | grep TotalVisibleMemorySize | cut -d'=' -f2)

    if [ -n "$free_mem" ] && [ -n "$total_mem" ]; then
        local used_mem=$((total_mem - free_mem))
        local mem_usage=$((used_mem * 100 / total_mem))
        echo "$mem_usage"
    else
        echo "0"
    fi
}

get_disk_usage() {
    # Get .claude directory usage in KB
    du -s "$CONFIG_DIR" 2>/dev/null | cut -f1 || echo "0"
}

get_agent_count() {
    # Count running claude.exe processes for Windows
    local count
    count=$(tasklist 2>/dev/null | grep -c "claude.exe" 2>/dev/null || true)
    echo "${count:-0}"
}

# Host-aware admission is the source of truth for local worker/build launches.
# Keep this daemon's legacy CPU/memory percentage checks as an additional
# signal, never as a substitute for reservations and process-tree accounting.
RESOURCE_ADMISSION_SCRIPT="${RESOURCE_ADMISSION_SCRIPT:-scripts/resource_admission.py}"
PYTHON_BIN="${PYTHON_BIN:-python3}"
if ! command -v "$PYTHON_BIN" >/dev/null 2>&1 && command -v python >/dev/null 2>&1; then
    PYTHON_BIN=python
fi

resource_admission_snapshot() {
    [ -f "$RESOURCE_ADMISSION_SCRIPT" ] || return 1
    "$PYTHON_BIN" "$RESOURCE_ADMISSION_SCRIPT" snapshot
}

resource_admission_allows_worker() {
    local snapshot
    snapshot=$(resource_admission_snapshot) || return 1
    "$PYTHON_BIN" -c 'import json, sys
try:
    state = json.load(sys.stdin)
    host = state["host"]
    admission = state["admission"]
    active = state.get("workers", [])
    if len(active) >= int(admission.get("max_workers", 3)):
        raise SystemExit(1)
    if float(state["available_after_reservations_mib"]) < float(admission.get("headroom_mib", 2048)):
        raise SystemExit(1)
    if float(host["available_mib"]) <= 0 or float(host["total_mib"]) <= 0:
        raise SystemExit(1)
except (KeyError, TypeError, ValueError, json.JSONDecodeError):
    raise SystemExit(1)
raise SystemExit(0)' <<< "$snapshot"
}

# Resource-based scaling decisions
should_scale_up() {
    local cpu_usage="$1"
    local mem_usage="$2"
    local current_agents="$3"

    # Extract thresholds with safe cross-platform defaults.
    local cpu_threshold=$(config_value "resource_management.cpu_threshold" "80")
    local mem_threshold=$(config_value "resource_management.memory_threshold" "85")
    local max_agents=$(config_value "resource_management.max_agents" "3")

    if [ "$current_agents" -ge "$max_agents" ]; then
        return 1  # Already at max
    fi

    if ! resource_admission_allows_worker; then
        return 1  # Shared task reservation/host headroom gate is closed
    fi

    if [ "$cpu_usage" -lt "$cpu_threshold" ] && [ "$mem_usage" -lt "$mem_threshold" ]; then
        return 0  # Resources available, can scale up
    fi

    return 1  # Resources constrained
}

should_scale_down() {
    local cpu_usage="$1"
    local mem_usage="$2"
    local current_agents="$3"

    # Extract thresholds with safe cross-platform defaults.
    local cpu_threshold=$(config_value "resource_management.cpu_threshold" "80")
    local mem_threshold=$(config_value "resource_management.memory_threshold" "85")
    local min_agents=$(config_value "resource_management.min_agents" "1")

    if [ "$current_agents" -le "$min_agents" ]; then
        return 1  # Already at min
    fi

    if [ "$cpu_usage" -gt "$((cpu_threshold + 10))" ] || [ "$mem_usage" -gt "$((mem_threshold + 5))" ]; then
        return 0  # Resources constrained, should scale down
    fi

    return 1  # Resources adequate
}

# Priority-based scheduling
get_task_priority() {
    local task_file="$1"

    # Extract priority from task file
    if grep -q "Priority: P0" "$task_file" 2>/dev/null; then
        echo "0"  # Highest priority
    elif grep -q "Priority: P1" "$task_file" 2>/dev/null; then
        echo "1"
    elif grep -q "Priority: P2" "$task_file" 2>/dev/null; then
        echo "2"
    else
        echo "99"  # Default/lowest priority
    fi
}

prioritize_tasks() {
    local todo_dir="HANDOFF/todo"

    if [ ! -d "$todo_dir" ]; then
        return
    fi

    # Create prioritized list
    local prioritized_tasks=""
    for task_file in "$todo_dir"/*.md; do
        if [ -f "$task_file" ]; then
            local priority=$(get_task_priority "$task_file")
            prioritized_tasks="$prioritized_tasks$priority:$task_file\n"
        fi
    done

    # Sort by priority and extract filenames
    echo -e "$prioritized_tasks" | sort -n | cut -d':' -f2-
}

# Agent management
can_start_agent() {
    local agent_type="$1"
    local current_agents="$2"

    # Extract agent limits with safe cross-platform defaults.
    local max_concurrent=$(config_value "agents.max_concurrent" "3")
    local max_per_type=$(config_value "agents.max_per_type.${agent_type}" "1")

    # Count current agents of this type
    local current_of_type=0
    local agent_pids=$(tasklist //fi "IMAGENAME eq claude.exe" //fo csv 2>/dev/null | grep -v "," | cut -d',' -f2 | tr -d '"' | grep -v PID || echo "")

    for pid in $agent_pids; do
        local cmdline=$(wmic process where "processid=$pid" get commandline //value 2>/dev/null | grep -i "commandline" | cut -d'=' -f2-)
        if echo "$cmdline" | grep -qi "$agent_type"; then
            current_of_type=$((current_of_type + 1))
        fi
    done

    if [ "$current_agents" -lt "$max_concurrent" ] && [ "$current_of_type" -lt "$max_per_type" ] && resource_admission_allows_worker; then
        return 0
    fi

    return 1
}

# Main resource management loop
manage_resources() {
    local check_interval=60

    while true; do
        # Get current system state
        local cpu_usage=$(get_cpu_usage)
        local mem_usage=$(get_memory_usage)
        local disk_usage=$(get_disk_usage)
        local agent_count=$(get_agent_count)
        local admission_state="BLOCKED"
        if resource_admission_allows_worker; then
            admission_state="AVAILABLE"
        fi

        # Log current state
        echo "Resource State: CPU=${cpu_usage}%, MEM=${mem_usage}%, DISK=${disk_usage}KB, AGENTS=${agent_count}, ADMISSION=${admission_state}"

        # Make scaling decisions
        if should_scale_up "$cpu_usage" "$mem_usage" "$agent_count"; then
            echo "Scaling UP: Resources available, can start more agents"
            # Implementation would start agents here
        fi

        if should_scale_down "$cpu_usage" "$mem_usage" "$agent_count"; then
            echo "Scaling DOWN: Resources constrained, should reduce agents"
            # Implementation would stop agents here
        fi

        # Check disk usage
        local disk_limit=$(config_value "monitoring.disk_usage_limit_mb" "100")
        disk_limit=$((disk_limit * 1024))  # Convert MB to KB

        if [ "$disk_usage" -gt "$disk_limit" ]; then
            echo "WARNING: Disk usage ${disk_usage}KB exceeds limit ${disk_limit}KB"
            # Implementation would trigger cleanup
        fi

        sleep $check_interval
    done
}

# Command line interface
case "${1:-}" in
    "--start")
        manage_resources
        ;;
    "--status")
        echo "CPU Usage: $(get_cpu_usage)%"
        echo "Memory Usage: $(get_memory_usage)%"
        echo "Disk Usage: $(get_disk_usage)KB"
        echo "Active Agents: $(get_agent_count)"
        if resource_admission_allows_worker; then
            echo "Worker Admission: AVAILABLE"
        else
            echo "Worker Admission: BLOCKED"
        fi
        resource_admission_snapshot || true
        ;;
    "--prioritize")
        prioritize_tasks
        ;;
    "--admission")
        resource_admission_snapshot
        ;;
    "--check-scale")
        cpu=$(get_cpu_usage)
        mem=$(get_memory_usage)
        agents=$(get_agent_count)

        admission="BLOCKED"
        if resource_admission_allows_worker; then
            admission="AVAILABLE"
        fi
        echo "Current: CPU=${cpu}%, MEM=${mem}%, AGENTS=${agents}, ADMISSION=${admission}"

        if should_scale_up "$cpu" "$mem" "$agents"; then
            echo "Decision: SCALE UP"
        elif should_scale_down "$cpu" "$mem" "$agents"; then
            echo "Decision: SCALE DOWN"
        else
            echo "Decision: MAINTAIN CURRENT"
        fi
        ;;
    *)
        echo "Usage: $0 [--start|--status|--admission|--prioritize|--check-scale]"
        echo "  --start        Start resource management daemon"
        echo "  --status       Show current resource usage and admission snapshot"
        echo "  --admission    Show shared task reservation and host telemetry"
        echo "  --prioritize   Show prioritized task list"
        echo "  --check-scale  Check scaling decisions"
        exit 1
        ;;
esac