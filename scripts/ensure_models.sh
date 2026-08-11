#!/bin/bash
# ensure_models.sh - Ensure required Ollama models are pulled and loaded
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Configuration
OLLAMA_HOST="${OLLAMA_HOST:-localhost:11434}"
REQUIRED_MODELS=(
    "glm-5.1:cloud"
    "qwen3-coder:480b:cloud"
    "deepseek-v3.2:cloud"
    "mistral-large-3:675b:cloud"
    "devstral-2:123b:cloud"
    "gemma4:31b:cloud"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

check_ollama_health() {
    local max_retries=3
    local retry_delay=2

    for attempt in $(seq 1 $max_retries); do
        if curl -f -s "http://$OLLAMA_HOST/api/version" > /dev/null; then
            return 0
        fi

        if [ $attempt -lt $max_retries ]; then
            log_warning "Ollama health check attempt $attempt failed, retrying in ${retry_delay}s..."
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))
        fi
    done

    log_error "Ollama service unavailable after $max_retries attempts"
    return 1
}

list_available_models() {
    local response
    response=$(curl -s "http://$OLLAMA_HOST/api/tags" 2>/dev/null || true)

    if [ -z "$response" ]; then
        log_error "Failed to list models - empty response from Ollama"
        return 1
    fi

    # Try to parse with jq if available, otherwise use grep
    if command -v jq > /dev/null 2>&1; then
        if ! echo "$response" | jq -e '.models' > /dev/null 2>&1; then
            log_error "Invalid JSON response from Ollama"
            return 1
        fi
        echo "$response" | jq -r '.models[].name'
    else
        # Fallback: extract model names with grep/sed
        echo "$response" | grep -o '"name"\s*:\s*"[^"]*"' | sed 's/"name"\s*:\s*"\([^"]*\)"/\1/'
    fi
}

pull_model() {
    local model=$1
    local max_attempts=3
    local attempt_delay=10

    log_info "Pulling model: $model"

    for attempt in $(seq 1 $max_attempts); do
        log_info "Attempt $attempt/$max_attempts to pull $model"

        # Start the pull request
        local pull_response
        pull_response=$(curl -s -N "http://$OLLAMA_HOST/api/pull" \
            -H "Content-Type: application/json" \
            -d "{\"name\": \"$model\"}" 2>/dev/null || true)

        if [ -n "$pull_response" ]; then
            # Monitor pull progress
            local line
            while IFS= read -r line; do
                if echo "$line" | jq -e '.status' > /dev/null 2>&1; then
                    local status=$(echo "$line" | jq -r '.status')
                    log_debug "Pull status: $status"

                    if [ "$status" = "success" ]; then
                        log_info " Successfully pulled model: $model"
                        return 0
                    elif [ "$status" = "error" ]; then
                        local error_msg=$(echo "$line" | jq -r '.error // "unknown error"')
                        log_warning "Pull error: $error_msg"
                        break
                    fi
                fi
            done <<< "$pull_response"
        fi

        if [ $attempt -lt $max_attempts ]; then
            log_warning "Pull attempt $attempt failed, retrying in ${attempt_delay}s..."
            sleep $attempt_delay
            attempt_delay=$((attempt_delay * 2)) # Exponential backoff
        fi
    done

    log_error "Failed to pull model $model after $max_attempts attempts"
    return 1
}

ensure_model_loaded() {
    local model=$1
    local available_models

    if ! available_models=$(list_available_models); then
        return 1
    fi

    if echo "$available_models" | grep -q "^$model$"; then
        log_info " Model already available: $model"
        return 0
    fi

    log_warning "Model not available: $model, attempting to pull..."

    if pull_model "$model"; then
        # Verify the model is now available
        local verify_attempts=5
        local verify_delay=3

        for verify_attempt in $(seq 1 $verify_attempts); do
            if available_models=$(list_available_models) && \
               echo "$available_models" | grep -q "^$model$"; then
                log_info " Model verified as available: $model"
                return 0
            fi

            if [ $verify_attempt -lt $verify_attempts ]; then
                log_debug "Verification attempt $verify_attempt failed, retrying..."
                sleep $verify_delay
            fi
        done

        log_error "Model $model not available after successful pull"
        return 1
    fi

    return 1
}

ensure_all_models() {
    local missing_count=0
    local total_models=${#REQUIRED_MODELS[@]}

    log_info "Ensuring all $total_models required models are available"

    for model in "${REQUIRED_MODELS[@]}"; do
        if ensure_model_loaded "$model"; then
            log_info " Model ready: $model"
        else
            log_error " Failed to ensure model: $model"
            ((missing_count++))
        fi
    done

    if [ $missing_count -eq 0 ]; then
        log_info "All $total_models required models are available"
        return 0
    else
        log_error "$missing_count/$total_models models failed to load"
        return 1
    fi
}

main() {
    log_info "Starting model availability assurance"

    if ! check_ollama_health; then
        log_error "Cannot proceed - Ollama service unavailable"
        exit 1
    fi

    if ensure_all_models; then
        log_info "Model assurance completed successfully"
        exit 0
    else
        log_error "Model assurance completed with errors"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    "-h" | "--help")
        echo "Usage: ensure_models.sh [--help]"
        echo "Ensure required Ollama models are pulled and loaded."
        echo ""
        echo "Required models:"
        for model in "${REQUIRED_MODELS[@]}"; do
            echo "  - $model"
        done
        exit 0
        ;;
    "--check-only")
        # Just check availability without pulling
        if check_ollama_health && list_available_models | grep -q -f <(printf "%s\n" "${REQUIRED_MODELS[@]}"); then
            log_info "All models available"
            exit 0
        else
            log_error "Models missing"
            exit 1
        fi
        ;;
    *)
        main "$@"
        ;;
esac
