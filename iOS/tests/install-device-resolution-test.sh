#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
FILTER="$ROOT_DIR/iOS/resolve-coredevice.jq"
INSTALL_SCRIPT="$ROOT_DIR/iOS/install-device.sh"
DEVICECTL_FIXTURE="$ROOT_DIR/iOS/tests/fixtures/devicectl-device-resolution.json"
XCDEVICE_FIXTURE="$ROOT_DIR/iOS/tests/fixtures/xcdevice-device-resolution.json"
MOCK_BIN="$ROOT_DIR/iOS/tests/helpers"

mkdir -p "$ROOT_DIR/tmp"
TEST_TMP_DIR="$(mktemp -d "$ROOT_DIR/tmp/install-device-test.XXXXXX")"
cleanup() {
  rm -rf "$TEST_TMP_DIR"
}
trap cleanup EXIT

assert_single_match() {
  local input_id="$1"
  local expected_core_id="$2"
  local expected_xcode_id="$3"

  jq -e \
    --arg id "$input_id" \
    --arg core_id "$expected_core_id" \
    --arg xcode_id "$expected_xcode_id" \
    -f "$FILTER" \
    "$DEVICECTL_FIXTURE" |
    jq -e \
      --arg core_id "$expected_core_id" \
      --arg xcode_id "$expected_xcode_id" \
      'length == 1
       and .[0].coreDeviceIdentifier == $core_id
       and .[0].xcodeIdentifier == $xcode_id' \
      >/dev/null
}

assert_no_match() {
  local input_id="$1"

  jq -e --arg id "$input_id" -f "$FILTER" "$DEVICECTL_FIXTURE" |
    jq -e 'length == 0' >/dev/null
}

run_install() {
  local input_id="$1"
  local xcdevice_json="$2"
  local devicectl_json="$3"

  env -u APPLE_TEAM_ID \
    PATH="$MOCK_BIN:$PATH" \
    MOCK_XCDEVICE_JSON="$xcdevice_json" \
    MOCK_DEVICECTL_JSON="$devicectl_json" \
    DEVICE_UDID="$input_id" \
    DEVICE_RESOLUTION_ONLY=1 \
    bash "$INSTALL_SCRIPT" 2>&1
}

assert_install_succeeds() {
  local output

  if ! output="$(run_install "$1" "$XCDEVICE_FIXTURE" "$DEVICECTL_FIXTURE")"; then
    printf '%s\n' "$output" >&2
    printf '%s\n' "expected resolution-only install preflight to succeed" >&2
    exit 1
  fi

  grep -F "[OK] Device resolution complete; build and install skipped." \
    <<<"$output" >/dev/null
  grep -F "Team ID:             not required for resolution-only mode" \
    <<<"$output" >/dev/null
  if grep -F "Generating/copying UniFFI bindings" <<<"$output" >/dev/null; then
    printf '%s\n' "resolution-only mode continued into build preparation" >&2
    exit 1
  fi
}

assert_install_fails() {
  local input_id="$1"
  local xcdevice_json="$2"
  local devicectl_json="$3"
  local expected_message="$4"
  local output
  local status

  set +e
  output="$(run_install "$input_id" "$xcdevice_json" "$devicectl_json")"
  status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    printf '%s\n' "$output" >&2
    printf '%s\n' "expected resolution-only install preflight to fail" >&2
    exit 1
  fi
  grep -F "$expected_message" <<<"$output" >/dev/null
}

# This fixture shape is redacted from live devicectl/xcdevice output. A paired
# Wi-Fi device remains eligible before CoreDevice opens its tunnel.
assert_single_match "CORE-WIFI-1" "CORE-WIFI-1" "XCODE-WIFI-1"
assert_single_match "XCODE-WIFI-1" "CORE-WIFI-1" "XCODE-WIFI-1"

# Identical display names cannot cross-select because matching is ID-only.
assert_single_match "CORE-WIFI-2" "CORE-WIFI-2" "XCODE-WIFI-2"
assert_no_match "Shared iPhone Name"

# Unpaired, virtual, non-iOS, and unknown records fail closed.
assert_no_match "CORE-UNPAIRED"
assert_no_match "CORE-SIMULATOR"
assert_no_match "CORE-MAC"
assert_no_match "DOES-NOT-EXIST"

assert_install_succeeds "CORE-WIFI-1"
assert_install_succeeds "XCODE-WIFI-1"

jq '
  map(
    if .identifier == "XCODE-WIFI-1"
    then .identifier = "XCODE-OTHER"
    else .
    end
  )
' "$XCDEVICE_FIXTURE" >"$TEST_TMP_DIR/xcdevice-mismatch.json"
assert_install_fails \
  "CORE-WIFI-1" \
  "$TEST_TMP_DIR/xcdevice-mismatch.json" \
  "$DEVICECTL_FIXTURE" \
  "unavailable or ambiguous Xcode destination"

jq '
  map(
    if .identifier == "XCODE-WIFI-1"
    then .available = false
    else .
    end
  )
' "$XCDEVICE_FIXTURE" >"$TEST_TMP_DIR/xcdevice-unavailable.json"
assert_install_fails \
  "CORE-WIFI-1" \
  "$TEST_TMP_DIR/xcdevice-unavailable.json" \
  "$DEVICECTL_FIXTURE" \
  "unavailable or ambiguous Xcode destination"

jq '
  (.result.devices[0] | .identifier = "CORE-WIFI-1-DUPLICATE") as $duplicate
  | .result.devices += [$duplicate]
' "$DEVICECTL_FIXTURE" >"$TEST_TMP_DIR/devicectl-duplicate.json"
assert_install_fails \
  "XCODE-WIFI-1" \
  "$XCDEVICE_FIXTURE" \
  "$TEST_TMP_DIR/devicectl-duplicate.json" \
  "expected one paired physical iOS device"

printf '%s\n' "[OK] install-device CoreDevice resolution tests passed"
