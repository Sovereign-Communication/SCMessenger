#!/usr/bin/env bash
# Read-only release metadata verifier.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
fail() { echo "[ERROR] $*" >&2; exit 1; }
workspace_version() { awk '/^\[workspace\.package\]$/ { p=1; next } /^\[/ { p=0 } p && /^version = "[^"]+"$/ { v=$0; sub(/^version = "/,"",v); sub(/"$/,"",v); print v; exit }' "$1"; }
android_value() { sed -nE "s/^[[:space:]]*$2[[:space:]]*=[[:space:]]*'?([^'[:space:]]+)'?.*/\\1/p" "$1" | head -n 1; }
plist_value() { awk -v key="$2" '$0 ~ "<key>" key "</key>" { getline; if (match($0, /<string>[^<]+<\/string>/)) { v=substr($0,RSTART,RLENGTH); sub(/^<string>/,"",v); sub(/<\/string>$/,"",v); print v; exit } }' "$1"; }
desktop_version() { sed -nE 's/^version = "([^"]+)"$/\1/p' "$1" | head -n 1; }

REQUIRE_TAG=false
if [[ "${1:-}" == "--require-tag" ]]; then REQUIRE_TAG=true; shift; fi
(($# == 0)) || fail "Usage: scripts/verify_versions.sh [--require-tag]"
for f in Cargo.toml android/build.gradle iOS/SCMessenger/SCMessenger/Info.plist iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj wasm/Cargo.toml shared/build.gradle.kts; do
    [[ -f "$f" ]] || fail "Required release manifest is missing: $f"
done
CARGO_VERSION="$(workspace_version Cargo.toml)"
ANDROID_VERSION="$(android_value android/build.gradle versionName)"
ANDROID_BUILD="$(android_value android/build.gradle versionCode)"
IOS_VERSION="$(plist_value iOS/SCMessenger/SCMessenger/Info.plist CFBundleShortVersionString)"
IOS_BUILD="$(plist_value iOS/SCMessenger/SCMessenger/Info.plist CFBundleVersion)"
IOS_PROJECT_VERSION="$(sed -nE 's/^[[:space:]]*MARKETING_VERSION = ([^;]+);/\1/p' iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj | sort -u)"
IOS_PROJECT_BUILD="$(sed -nE 's/^[[:space:]]*CURRENT_PROJECT_VERSION = ([^;]+);/\1/p' iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj | sort -u)"
DESKTOP_VERSION="$(desktop_version shared/build.gradle.kts)"
[[ "$CARGO_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]] || fail "Cargo version is not SemVer: $CARGO_VERSION"
IOS_CORE="${CARGO_VERSION%%[-+]*}"
[[ "$ANDROID_BUILD" =~ ^[1-9][0-9]*$ ]] || fail "Android versionCode must be positive"
[[ "$IOS_BUILD" =~ ^[1-9][0-9]*$ ]] || fail "iOS CFBundleVersion must be positive"
[[ "$ANDROID_VERSION" == "$CARGO_VERSION" ]] || fail "Android versionName ($ANDROID_VERSION) differs from Cargo ($CARGO_VERSION)"
[[ "$DESKTOP_VERSION" == "$CARGO_VERSION" ]] || fail "Desktop version ($DESKTOP_VERSION) differs from Cargo ($CARGO_VERSION)"
[[ "$IOS_VERSION" == "$IOS_CORE" ]] || fail "iOS marketing version ($IOS_VERSION) must be Cargo numeric core ($IOS_CORE)"
[[ "$IOS_PROJECT_VERSION" == "$IOS_VERSION" ]] || fail "iOS project marketing versions ($IOS_PROJECT_VERSION) differ from Info.plist ($IOS_VERSION)"
[[ "$IOS_PROJECT_BUILD" == "$IOS_BUILD" ]] || fail "iOS project build numbers ($IOS_PROJECT_BUILD) differ from Info.plist ($IOS_BUILD)"
grep -qE '^version\.workspace = true$' wasm/Cargo.toml || fail "WASM does not inherit Cargo's workspace version"

HEAD_COMMIT="$(git rev-parse HEAD)"
if "$REQUIRE_TAG"; then
    TAG="$(git describe --exact-match --tags HEAD 2>/dev/null || true)"
    [[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]] || fail "HEAD must be checked out at a SemVer release tag"
    [[ "${TAG#v}" == "$CARGO_VERSION" ]] || fail "Release tag $TAG differs from Cargo $CARGO_VERSION"
    [[ "${TAG#v}" == "$CARGO_VERSION" && "${TAG#v}" == "$ANDROID_VERSION" && "${TAG#v}" == "$DESKTOP_VERSION" ]] || fail "Tag, Cargo, Android, and desktop must agree"
    TAG_CORE="${TAG#v}"
    TAG_CORE="${TAG_CORE%%[-+]*}"
    [[ "$TAG_CORE" == "$IOS_VERSION" ]] || fail "Tag numeric core $TAG_CORE differs from iOS marketing version $IOS_VERSION"
fi

MAX_ANDROID=0
MAX_IOS=0
while IFS= read -r tag; do
    [[ -n "$tag" ]] || continue
    [[ "$(git rev-parse "$tag^{commit}")" == "$HEAD_COMMIT" ]] && continue
    if git cat-file -e "$tag:android/build.gradle" 2>/dev/null; then
        prior="$(git show "$tag:android/build.gradle" | sed -nE 's/^[[:space:]]*versionCode[[:space:]]*=[[:space:]]*([0-9]+).*/\1/p' | head -n 1)"
        if [[ "$prior" =~ ^[1-9][0-9]*$ ]] && (( prior > MAX_ANDROID )); then MAX_ANDROID="$prior"; fi
    fi
    if git cat-file -e "$tag:iOS/SCMessenger/SCMessenger/Info.plist" 2>/dev/null; then
        prior="$(git show "$tag:iOS/SCMessenger/SCMessenger/Info.plist" | awk '/<key>CFBundleVersion<\/key>/ { getline; if (match($0, /<string>[0-9]+<\/string>/)) { v=substr($0,RSTART,RLENGTH); sub(/^<string>/,"",v); sub(/<\/string>$/,"",v); print v; exit } }')"
        if [[ "$prior" =~ ^[1-9][0-9]*$ ]] && (( prior > MAX_IOS )); then MAX_IOS="$prior"; fi
    fi
done < <(git tag --merged HEAD --list 'v*')
(( ANDROID_BUILD > MAX_ANDROID )) || fail "Android versionCode $ANDROID_BUILD is not greater than prior tagged build $MAX_ANDROID"
(( IOS_BUILD > MAX_IOS )) || fail "iOS CFBundleVersion $IOS_BUILD is not greater than prior tagged build $MAX_IOS"
echo "[OK] Cargo/Android/Desktop/WASM agree at $CARGO_VERSION; iOS marketing version is $IOS_CORE"
echo "[OK] Independent build numbers exceed tagged baselines: Android $ANDROID_BUILD>$MAX_ANDROID, iOS $IOS_BUILD>$MAX_IOS"
