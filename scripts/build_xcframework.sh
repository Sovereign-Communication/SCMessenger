#!/usr/bin/env bash
# Build SCMessengerCore.xcframework for iOS
#
# Usage: scripts/build_xcframework.sh
#
# Produces: iOS/SCMessengerCore.xcframework
# Contains: arm64 (device) + arm64-sim (simulator) static libraries
# with the generated Swift bindings.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGET="aarch64-apple-ios-sim"
BUILD_DIR="$ROOT_DIR/target/xcframework"
OUTPUT="$ROOT_DIR/iOS/SCMessengerCore.xcframework"

echo "Building Rust static libraries..."

cargo build --target "$DEVICE_TARGET" -p scmessenger-core --release
cargo build --target "$SIM_TARGET" -p scmessenger-core --release

echo "Generating Swift bindings..."

# gen_swift reads the host libscmessenger_core cdylib, which only a direct
# -p scmessenger-core build emits (cargo run only links the rlib).
cargo build -p scmessenger-core
cargo run --bin gen_swift --features gen-bindings

# Stage generated Swift bindings where the Xcode project expects them
SWIFT_GEN_DIR="$ROOT_DIR/core/target/generated-sources/uniffi/swift"
IOS_GEN_DIR="$ROOT_DIR/iOS/SCMessenger/SCMessenger/Generated"
mkdir -p "$IOS_GEN_DIR"
# Keep the staged files byte-for-byte compatible with the checked-in
# canonical bindings.  In particular, Xcode deploys apiFFI.h, while UniFFI's
# raw module map refers to scmessenger_core.h.  A raw copy here causes the
# subsequent binding verification step to fail in CI.
awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$SWIFT_GEN_DIR/SCMessengerCore.swift" > "$IOS_GEN_DIR/api.swift"
awk '{ sub(/[[:space:]]+$/, ""); print }' \
    "$SWIFT_GEN_DIR/scmessenger_core.h" > "$IOS_GEN_DIR/apiFFI.h"
sed 's/header "scmessenger_core.h"/header "apiFFI.h"/' \
    "$SWIFT_GEN_DIR/scmessenger_core.modulemap" |
    awk '{ sub(/[[:space:]]+$/, ""); print }' > "$IOS_GEN_DIR/apiFFI.modulemap"

echo "Creating xcframework..."

rm -rf "$OUTPUT"
mkdir -p "$BUILD_DIR"

xcodebuild -create-xcframework \
    -library "$ROOT_DIR/target/$DEVICE_TARGET/release/libscmessenger_core.a" \
    -headers "$ROOT_DIR/core/target/generated-sources/uniffi/swift/" \
    -library "$ROOT_DIR/target/$SIM_TARGET/release/libscmessenger_core.a" \
    -headers "$ROOT_DIR/core/target/generated-sources/uniffi/swift/" \
    -output "$OUTPUT"

rm -rf "$BUILD_DIR"

echo "xcframework created at: $OUTPUT"
