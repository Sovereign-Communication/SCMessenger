#!/bin/bash
# verify_ios_bindings.sh
# Verifies that the generated iOS Swift bindings are in sync with the UDL definition.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SANITIZER="$ROOT_DIR/scripts/sanitize_generated_text.py"

GENERATED_SWIFT="$ROOT_DIR/core/target/generated-sources/uniffi/swift/SCMessengerCore.swift"
GENERATED_HEADER="$ROOT_DIR/core/target/generated-sources/uniffi/swift/scmessenger_core.h"
GENERATED_MODULEMAP="$ROOT_DIR/core/target/generated-sources/uniffi/swift/scmessenger_core.modulemap"
COMMITTED_SWIFT="$ROOT_DIR/iOS/SCMessenger/SCMessenger/Generated/api.swift"
COMMITTED_HEADER="$ROOT_DIR/iOS/SCMessenger/SCMessenger/Generated/apiFFI.h"
COMMITTED_MODULEMAP="$ROOT_DIR/iOS/SCMessenger/SCMessenger/Generated/apiFFI.modulemap"

echo "Verifying iOS Swift bindings..."

# Build host-native library required by gen_swift. Must be the core package
# itself: only a direct -p scmessenger-core build emits the libscmessenger_core
# cdylib that gen_swift reads (bins and dependent crates only link the rlib).
if ! cargo build -p scmessenger-core; then
    echo "ERROR: Failed to build scmessenger-core"
    exit 1
fi

# Generate Swift bindings (writes to fixed output directory)
if ! cargo run --bin gen_swift --features gen-bindings; then
    echo "ERROR: Failed to generate Swift bindings"
    exit 1
fi

verify_file() {
    local label="$1"
    local committed="$2"
    local generated="$3"

    # UniFFI can emit trailing spaces on otherwise blank lines. Normalize trailing
    # whitespace and strip repo-forbidden Unicode code points so cosmetic generator
    # drift cannot hide real API changes or keep an otherwise current binding set permanently red.
    if ! diff -u \
        <(python3 "$SANITIZER" < "$committed" | awk '{ sub(/[[:space:]]+$/, ""); print }') \
        <(python3 "$SANITIZER" < "$generated" | awk '{ sub(/[[:space:]]+$/, ""); print }'); then
        echo "ERROR: Generated $label is out of sync."
        echo "Regenerate with './iOS/copy-bindings.sh' and commit all three binding outputs together."
        return 1
    fi
}

verify_file "Swift binding" "$COMMITTED_SWIFT" "$GENERATED_SWIFT"
verify_file "C header" "$COMMITTED_HEADER" "$GENERATED_HEADER"

# The Xcode project deploys the generated C header as apiFFI.h. Compare the
# module map after applying that one intentional deployment-name transform and sanitization.
if ! diff -u \
    <(python3 "$SANITIZER" < "$COMMITTED_MODULEMAP" |
        awk '{ sub(/[[:space:]]+$/, ""); print }') \
    <(sed 's/header "scmessenger_core.h"/header "apiFFI.h"/' "$GENERATED_MODULEMAP" |
        python3 "$SANITIZER" |
        awk '{ sub(/[[:space:]]+$/, ""); print }'); then
    echo "ERROR: Generated module map is out of sync."
    echo "Regenerate with './iOS/copy-bindings.sh' and commit all three binding outputs together."
    exit 1
fi

./iOS/assert-generated-path.sh

echo "iOS binding verification passed."
