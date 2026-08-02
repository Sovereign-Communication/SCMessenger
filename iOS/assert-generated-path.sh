#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CANONICAL_DIR="$ROOT_DIR/iOS/SCMessenger/SCMessenger/Generated"
LEGACY_DIR="$ROOT_DIR/iOS/SCMessenger/Generated"
COPY_SCRIPT="$ROOT_DIR/iOS/copy-bindings.sh"

required_files=(
  "api.swift"
  "apiFFI.h"
  "apiFFI.modulemap"
)

for file in "${required_files[@]}"; do
  if [ ! -f "$CANONICAL_DIR/$file" ]; then
    echo "error: missing canonical generated file: $CANONICAL_DIR/$file"
    exit 1
  fi
done

if ! grep -q 'header "apiFFI.h"' "$CANONICAL_DIR/apiFFI.modulemap"; then
  echo "error: canonical module map does not reference the deployed apiFFI.h header."
  exit 1
fi

if grep -q 'header "scmessenger_core.h"' "$CANONICAL_DIR/apiFFI.modulemap"; then
  echo "error: canonical module map references a generated header name that is not deployed."
  exit 1
fi

if [ -d "$LEGACY_DIR" ]; then
  for file in "${required_files[@]}"; do
    if [ -f "$LEGACY_DIR/$file" ]; then
      echo "error: legacy generated output detected: $LEGACY_DIR/$file"
      echo "hint: run ./iOS/copy-bindings.sh to normalize generated paths."
      exit 1
    fi
  done
fi

if grep -q "mkdir -p ../iOS/SCMessenger/Generated" "$COPY_SCRIPT" \
  || grep -q "SCMessenger/Generated/api.swift" "$COPY_SCRIPT" \
  || grep -q "SCMessenger/Generated/apiFFI.h" "$COPY_SCRIPT" \
  || grep -q "SCMessenger/Generated/apiFFI.modulemap" "$COPY_SCRIPT"; then
  echo "error: copy-bindings.sh contains legacy dual-copy logic."
  echo "hint: keep bindings output canonical at iOS/SCMessenger/SCMessenger/Generated only."
  exit 1
fi

echo "Generated path guard: OK (canonical-only)."
