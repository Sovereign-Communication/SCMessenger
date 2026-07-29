#!/usr/bin/env bash
# Synchronize Cargo's release version to the real platform manifests.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
fail() { echo "[ERROR] $*" >&2; exit 1; }
usage() {
    cat <<'EOF'
Usage: scripts/sync_version.sh (--build-number N | --android-build-number N --ios-build-number N)

Cargo.toml is the source version. Android and desktop retain its complete
SemVer value; iOS CFBundleShortVersionString receives only X.Y.Z because Apple
marketing versions cannot carry prerelease metadata. Build numbers are supplied
by the release operator and must each exceed their current platform value.
EOF
}
plist_value() { awk -v key="$1" '$0 ~ "<key>" key "</key>" { getline; if (match($0, /<string>[^<]+<\/string>/)) { v=substr($0,RSTART,RLENGTH); sub(/^<string>/,"",v); sub(/<\/string>$/,"",v); print v; exit } }' "$2"; }
workspace_version() { awk '/^\[workspace\.package\]$/ { p=1; next } /^\[/ { p=0 } p && /^version = "[^"]+"$/ { v=$0; sub(/^version = "/,"",v); sub(/"$/,"",v); print v; exit }' Cargo.toml; }

ANDROID_BUILD=""
IOS_BUILD=""
SHARED_BUILD=""
while (($#)); do
    case "$1" in
        --build-number) (($# >= 2)) || fail "--build-number requires a value"; SHARED_BUILD="$2"; shift 2 ;;
        --android-build-number) (($# >= 2)) || fail "--android-build-number requires a value"; ANDROID_BUILD="$2"; shift 2 ;;
        --ios-build-number) (($# >= 2)) || fail "--ios-build-number requires a value"; IOS_BUILD="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; fail "Unknown argument: $1" ;;
    esac
done
if [[ -n "$SHARED_BUILD" ]]; then
    [[ -z "$ANDROID_BUILD" && -z "$IOS_BUILD" ]] || fail "--build-number cannot be combined with platform-specific build numbers"
    ANDROID_BUILD="$SHARED_BUILD"
    IOS_BUILD="$SHARED_BUILD"
fi
[[ "$ANDROID_BUILD" =~ ^[1-9][0-9]*$ ]] || fail "Android build number must be a positive integer"
[[ "$IOS_BUILD" =~ ^[1-9][0-9]*$ ]] || fail "iOS build number must be a positive integer"

# Preflight every target before changing any file. WASM inherits Cargo's
# workspace version, so it is verified rather than independently rewritten.
for f in Cargo.toml android/build.gradle iOS/SCMessenger/SCMessenger/Info.plist iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj wasm/Cargo.toml shared/build.gradle.kts; do
    [[ -f "$f" ]] || fail "Required release manifest is missing: $f"
done
VERSION="$(workspace_version)"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]] || fail "Cargo workspace version is invalid: $VERSION"
IOS_MARKETING_VERSION="${VERSION%%[-+]*}"
grep -qE "^[[:space:]]*versionCode[[:space:]]*=[[:space:]]*[0-9]+" android/build.gradle || fail "Android versionCode is missing"
grep -qE "^[[:space:]]*versionName[[:space:]]*=[[:space:]]*'[^']+'" android/build.gradle || fail "Android versionName is missing"
grep -q '<key>CFBundleShortVersionString</key>' iOS/SCMessenger/SCMessenger/Info.plist || fail "iOS marketing version is missing"
grep -q '<key>CFBundleVersion</key>' iOS/SCMessenger/SCMessenger/Info.plist || fail "iOS build number is missing"
grep -q 'MARKETING_VERSION = ' iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj || fail "iOS project marketing version is missing"
grep -q 'CURRENT_PROJECT_VERSION = ' iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj || fail "iOS project build number is missing"
grep -qE '^version\.workspace = true$' wasm/Cargo.toml || fail "WASM must inherit the workspace version"
grep -qE '^version = "[^"]+"$' shared/build.gradle.kts || fail "Desktop version is missing"
CURRENT_ANDROID="$(sed -nE 's/^[[:space:]]*versionCode[[:space:]]*=[[:space:]]*([0-9]+).*/\1/p' android/build.gradle | head -n 1)"
CURRENT_IOS="$(plist_value CFBundleVersion iOS/SCMessenger/SCMessenger/Info.plist)"
[[ "$CURRENT_ANDROID" =~ ^[0-9]+$ ]] || fail "Could not read Android versionCode"
[[ "$CURRENT_IOS" =~ ^[0-9]+$ ]] || fail "Could not read iOS CFBundleVersion"
(( ANDROID_BUILD > CURRENT_ANDROID )) || fail "Android build number $ANDROID_BUILD must exceed $CURRENT_ANDROID"
(( IOS_BUILD > CURRENT_IOS )) || fail "iOS build number $IOS_BUILD must exceed $CURRENT_IOS"

# Only declared release fields are changed; lockfiles and generated files are untouched.
VERSION="$VERSION" ANDROID_BUILD="$ANDROID_BUILD" perl -0pi -e 's/(^[\t ]*versionCode[\t ]*=[\t ]*)[0-9]+/${1}$ENV{ANDROID_BUILD}/m; s/(^[\t ]*versionName[\t ]*=[\t ]*'"'"')[^'"'"']+'"'"'/${1}$ENV{VERSION}'"'"'/m' android/build.gradle
IOS_MARKETING_VERSION="$IOS_MARKETING_VERSION" IOS_BUILD="$IOS_BUILD" perl -0pi -e 's#(<key>CFBundleShortVersionString</key>\s*<string>)[^<]+(</string>)#${1}$ENV{IOS_MARKETING_VERSION}${2}#s; s#(<key>CFBundleVersion</key>\s*<string>)[^<]+(</string>)#${1}$ENV{IOS_BUILD}${2}#s' iOS/SCMessenger/SCMessenger/Info.plist
IOS_MARKETING_VERSION="$IOS_MARKETING_VERSION" IOS_BUILD="$IOS_BUILD" perl -0pi -e 's/(MARKETING_VERSION = )[^;]+;/${1}$ENV{IOS_MARKETING_VERSION};/g; s/(CURRENT_PROJECT_VERSION = )[^;]+;/${1}$ENV{IOS_BUILD};/g' iOS/SCMessenger/SCMessenger.xcodeproj/project.pbxproj
VERSION="$VERSION" perl -0pi -e 's/^version = "[^"]+"$/version = "$ENV{VERSION}"/m' shared/build.gradle.kts

bash scripts/verify_versions.sh
echo "[OK] Synchronized $VERSION (iOS marketing version $IOS_MARKETING_VERSION)"
