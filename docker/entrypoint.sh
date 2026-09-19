#!/bin/bash
set -e

# Initialize config and data directories
mkdir -p "$SCM_CONFIG_DIR"
mkdir -p "$SCM_DATA_DIR"
CONFIG_FILE="$SCM_CONFIG_DIR/config.json"

# Determine the port to use (priority: LISTEN_PORT > PORT > default 9000)
FINAL_PORT="${LISTEN_PORT:-${PORT:-9000}}"

# Note: Bootstrap nodes are now embedded in the binary at build time
# The SCM CLI will automatically load these defaults when creating new config
# This entrypoint only needs to handle port configuration and environment overrides

# Create initial config if it doesn't exist (CLI will populate with defaults)
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Initializing configuration with embedded bootstrap nodes..."
    # Let the CLI create the default config with embedded bootstraps
    scm config list > /dev/null 2>&1 || true
fi

# Update port from environment if specified
if [ -f "$CONFIG_FILE" ]; then
    tmp=$(mktemp)
    jq --arg port "$FINAL_PORT" '.listen_port = ($port | tonumber)' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
fi

# Add additional bootstrap nodes from environment variable (merged with defaults).
# Must match SC_BOOTSTRAP_NODES exactly: that's the name cli/src/bootstrap.rs's
# default_bootstrap_nodes() actually reads at runtime (independent of this
# config.json merge). Before this fix the two disagreed (this block checked
# plain BOOTSTRAP_NODES), so config.json's bootstrap_nodes array stayed
# permanently empty -- a misleading diagnostic trap for anyone inspecting
# config.json to debug topology issues, even though the running node still
# picked up the right bootstrap peer via its own env-var read.
if [ ! -z "$SC_BOOTSTRAP_NODES" ]; then
    echo "Adding environment bootstrap nodes to defaults..."
    IFS=',' read -ra ADDR <<< "$SC_BOOTSTRAP_NODES"
    for i in "${ADDR[@]}"; do
        node=$(echo "$i" | xargs)  # Trim whitespace
        if [ ! -z "$node" ]; then
            tmp=$(mktemp)
            jq --arg node "$node" '.bootstrap_nodes += [$node] | .bootstrap_nodes |= unique' "$CONFIG_FILE" > "$tmp" && mv "$tmp" "$CONFIG_FILE"
        fi
    done
fi

echo "[OK] Configuration ready (port: $FINAL_PORT)"

# If no port argument is provided to scm start, add it
if [ "$1" = "scm" ] && [ "$2" = "start" ]; then
    # Ensure identity is initialized before starting (tolerate already-initialized)
    scm init || true

    # Export identity to a file for contact provisioning
    if [ ! -z "$NODE_NAME" ]; then
        IDENTITY_FILE="/tmp/scm_identity_${NODE_NAME}.json"
        scm identity 2>/dev/null | awk '
            BEGIN { id=""; peer_id=""; pub_key="" }
            /^  ID:/ { id=$2; }
            /Peer ID \(Network\):/ { peer_id=$NF; }
            /Public Key:/ { pub_key=$3; }
            END {
                print "{\"node_name\":\"'$NODE_NAME'\",\"identity_id\":\"" id "\",\"peer_id\":\"" peer_id "\",\"public_key\":\"" pub_key "\"}"
            }
        ' > "$IDENTITY_FILE" 2>/dev/null || true
    fi

    # Build scm start command with proper flags
    # Order: scm [global flags] start [subcommand flags]
    NEW_ARGS=("$1")  # scm

    # Add global flags before subcommand (default to loopback interface)
    if ! echo "$@" | grep -q "\-\-http-bind"; then
        NEW_ARGS+=("--http-bind" "${SCM_HTTP_BIND:-127.0.0.1:9876}")
    fi

    NEW_ARGS+=("$2")  # start

    # Add start-specific flags
    if ! echo "$@" | grep -q "\-\-port"; then
        NEW_ARGS+=("--port" "$FINAL_PORT")
    fi

    # Add any remaining arguments
    if [ ${#@} -gt 2 ]; then
        NEW_ARGS+=("${@:3}")
    fi

    set -- "${NEW_ARGS[@]}"
fi

# Run the command
exec "$@"