#!/usr/bin/env bash
# Copy UniFFI Swift bindings to iOS project (canonical path only).
set -euo pipefail

# Navigate to project root
cd "$(dirname "$0")/.."

echo "=== Copying Swift bindings to iOS project ==="

# Generate bindings first
echo "1. Generating Swift bindings..."
cd core
cargo build -p scmessenger-core
cargo run --bin gen_swift --features gen-bindings

canonical_dir="../iOS/SCMessenger/SCMessenger/Generated"
legacy_dir="../iOS/SCMessenger/Generated"

echo "2. Creating canonical destination directory..."
mkdir -p "$canonical_dir"

echo "3. Removing legacy generated outputs..."
legacy_removed="not-present"
if [ -d "$legacy_dir" ]; then
  rm -f "$legacy_dir/api.swift" "$legacy_dir/apiFFI.h" "$legacy_dir/apiFFI.modulemap"
  rmdir "$legacy_dir" 2>/dev/null || true
  legacy_removed="cleaned"
fi

echo "4. Copying files..."
# UniFFI names outputs after the module_name in uniffi.toml.  Map them to the
# canonical names expected by the Xcode project (api.swift, apiFFI.h, apiFFI.modulemap).
swift_src="target/generated-sources/uniffi/swift"
if [ -f "$swift_src/SCMessengerCore.swift" ]; then
  awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$swift_src/SCMessengerCore.swift" > "$canonical_dir/api.swift"
  awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$swift_src/scmessenger_core.h" > "$canonical_dir/apiFFI.h"
  sed 's/header "scmessenger_core.h"/header "apiFFI.h"/' \
    "$swift_src/scmessenger_core.modulemap" |
    awk '{ sub(/[[:space:]]+$/, ""); print }' > "$canonical_dir/apiFFI.modulemap"
elif [ -f "$swift_src/api.swift" ]; then
  awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$swift_src/api.swift" > "$canonical_dir/api.swift"
  awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$swift_src/apiFFI.h" > "$canonical_dir/apiFFI.h"
  sed 's/header "scmessenger_core.h"/header "apiFFI.h"/' \
    "$swift_src/apiFFI.modulemap" |
    awk '{ sub(/[[:space:]]+$/, ""); print }' > "$canonical_dir/apiFFI.modulemap"
else
  echo "error: no generated Swift bindings found in $swift_src"
  ls -la "$swift_src/" 2>/dev/null || true
  exit 1
fi

echo "[OK] Swift bindings copied successfully"
echo
echo "Files copied to:"
echo "  - iOS/SCMessenger/SCMessenger/Generated/"
echo "  - legacy generated path removed: $legacy_removed"
echo "  - api.swift ($(wc -l < "$canonical_dir/api.swift") lines)"
echo "  - apiFFI.h"
echo "  - apiFFI.modulemap"

echo
bash "../iOS/assert-generated-path.sh"
