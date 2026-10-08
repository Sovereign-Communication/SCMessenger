#!/usr/bin/env bash
# FFI Surface Contract Test
# Extracts public symbols from generated Kotlin and Swift bindings,
# then diffs against the checked-in snapshot. Fails on unapproved changes.
#
# Usage:
#   scripts/ffi_surface.sh [--update]
#
# With --update: regenerates the snapshot files.
# Without: diffs against existing snapshots and fails on mismatch.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SNAPSHOT_DIR="$ROOT_DIR/scripts/ffi-snapshots"

mkdir -p "$SNAPSHOT_DIR"

extract_kotlin_symbols() {
    local kt_file="$1"
    if [[ ! -f "$kt_file" ]]; then
        echo "WARN: Kotlin bindings not found at $kt_file" >&2
        return 1
    fi
    # Top-level free functions get their closing KDoc `*/` on the same
    # physical line as `fun` in UniFFI's generated Kotlin, so the pattern
    # must allow (and strip) an optional leading `*/` before the keyword.
    # UniFFI emits async methods as `suspend fun` on the interface (and
    # `override suspend fun` on the impl); allow an optional `suspend ` before
    # `fun ` so async methods are tracked. `override ...` impl lines are still
    # excluded (they start with `override`, not `suspend`/`fun`), so each
    # method is counted once at its interface declaration.
    grep -E '^\s*(\*/\s*)?(suspend )?(fun |class |interface |enum |object |data class |sealed class |value class )' "$kt_file" | \
        sed -E 's/^\s*(\*\/\s*)?//' | sort
}

extract_swift_symbols() {
    local swift_file="$1"
    if [[ ! -f "$swift_file" ]]; then
        echo "WARN: Swift bindings not found at $swift_file" >&2
        return 1
    fi
    grep -E '^\s*(public func |public class |public protocol |public enum |public struct |public typealias |open class |open func )' "$swift_file" | \
        sed 's/^\s*//' | sort
}

# Find generated binding files
KT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" -name "api.kt" -o -name "scmessenger_core.kt" 2>/dev/null | head -1 || true)
SWIFT_FILE=$(find "$ROOT_DIR/core/target/generated-sources" -name "SCMessengerCore.swift" 2>/dev/null | head -1 || true)

UPDATE=false
if [[ "${1:-}" == "--update" ]]; then
    UPDATE=true
fi

EXIT_CODE=0

if [[ -n "$KT_FILE" ]]; then
    CURRENT_KT=$(extract_kotlin_symbols "$KT_FILE")
    if $UPDATE; then
        if [[ -z "$CURRENT_KT" ]]; then
            # Empty extraction means the binding file is present but broken or
            # empty (e.g. a build that died before generation finished). Writing
            # it would silently lower the contract for every future run.
            echo "[FAIL] Refusing to write Kotlin snapshot: no symbols extracted from $KT_FILE"
            echo "       (binding file empty or malformed). Regenerate with:"
            echo "       cargo run -p scmessenger-core --bin gen_kotlin --features gen-bindings"
            EXIT_CODE=1
        else
            echo "$CURRENT_KT" > "$SNAPSHOT_DIR/kotlin-symbols.txt"
            echo "Updated Kotlin snapshot"
        fi
    else
        if [[ -f "$SNAPSHOT_DIR/kotlin-symbols.txt" ]]; then
            EXPECTED_KT=$(cat "$SNAPSHOT_DIR/kotlin-symbols.txt")
            if [[ "$CURRENT_KT" != "$EXPECTED_KT" ]]; then
                echo "ERROR: Kotlin FFI surface changed without snapshot update"
                diff "$SNAPSHOT_DIR/kotlin-symbols.txt" <(echo "$CURRENT_KT") || true
                EXIT_CODE=1
            else
                echo "Kotlin FFI surface: OK"
            fi
        else
            echo "WARN: No Kotlin snapshot found. Run with --update to create."
            EXIT_CODE=1
        fi
    fi
else
    echo "[FAIL] Kotlin bindings not generated (missing under core/target/generated-sources)."
    echo "       Nothing was checked. Regenerate with:"
    echo "       cargo run -p scmessenger-core --bin gen_kotlin --features gen-bindings"
    EXIT_CODE=1
fi

if [[ -n "$SWIFT_FILE" ]]; then
    CURRENT_SWIFT=$(extract_swift_symbols "$SWIFT_FILE")
    if $UPDATE; then
        if [[ -z "$CURRENT_SWIFT" ]]; then
            echo "[FAIL] Refusing to write Swift snapshot: no symbols extracted from $SWIFT_FILE"
            echo "       (binding file empty or malformed). Regenerate with:"
            echo "       cargo run -p scmessenger-core --bin gen_swift --features gen-bindings"
            EXIT_CODE=1
        else
            echo "$CURRENT_SWIFT" > "$SNAPSHOT_DIR/swift-symbols.txt"
            echo "Updated Swift snapshot"
        fi
    else
        if [[ -f "$SNAPSHOT_DIR/swift-symbols.txt" ]]; then
            EXPECTED_SWIFT=$(cat "$SNAPSHOT_DIR/swift-symbols.txt")
            if [[ "$CURRENT_SWIFT" != "$EXPECTED_SWIFT" ]]; then
                echo "ERROR: Swift FFI surface changed without snapshot update"
                diff "$SNAPSHOT_DIR/swift-symbols.txt" <(echo "$CURRENT_SWIFT") || true
                EXIT_CODE=1
            else
                echo "Swift FFI surface: OK"
            fi
        else
            echo "WARN: No Swift snapshot found. Run with --update to create."
            EXIT_CODE=1
        fi
    fi
else
    echo "[FAIL] Swift bindings not generated (missing under core/target/generated-sources)."
    echo "       Nothing was checked. Regenerate with:"
    echo "       cargo run -p scmessenger-core --bin gen_swift --features gen-bindings"
    EXIT_CODE=1
fi

exit $EXIT_CODE
