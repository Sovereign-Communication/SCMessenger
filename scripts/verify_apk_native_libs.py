#!/usr/bin/env python3
"""SCMessenger APK Native Library Gate.

Verifies that a built APK actually CONTAINS the Rust core shared library
(libscmessenger_core.so) for every required ABI. A green gradle build does
not prove the native library was packaged: a cargo-profile/output-dir
mismatch in android/app/build.gradle once let assembleDebug produce zero
core .so files while BUILD SUCCESSFUL still printed, and every APK crashed
on launch with UnsatisfiedLinkError. This gate inspects the finished APK
(a zip) directly, so the artifact itself is the evidence.

Checks Performed:
  1. Prints the FULL lib/ inventory of the APK (never truncated).
  2. Asserts lib/<abi>/libscmessenger_core.so exists for every requested ABI.

Usage:
  python scripts/verify_apk_native_libs.py <apk> [abi ...]

Default ABI list: arm64-v8a armeabi-v7a x86_64 x86

Exit codes:
  0 -- every requested ABI has lib/<abi>/libscmessenger_core.so in the APK
  1 -- APK missing/unreadable, or at least one requested ABI is absent
"""

import os
import sys
import zipfile

DEFAULT_ABIS = ["arm64-v8a", "armeabi-v7a", "x86_64", "x86"]
CORE_LIB = "libscmessenger_core.so"


def lib_inventory(apk_path):
    """Return every zip entry under lib/, sorted. Full list, never truncated."""
    with zipfile.ZipFile(apk_path) as zf:
        return sorted(name for name in zf.namelist() if name.startswith("lib/"))


def missing_abis_from_entries(entries, abis):
    """Return the ABIs whose core library entry is absent from the inventory."""
    entry_set = set(entries)
    return [abi for abi in abis if "lib/{0}/{1}".format(abi, CORE_LIB) not in entry_set]


def missing_abis(apk_path, abis):
    """Return the ABIs whose core library entry is absent from the APK."""
    return missing_abis_from_entries(lib_inventory(apk_path), abis)


def main(argv):
    if not argv:
        print("usage: python scripts/verify_apk_native_libs.py <apk> [abi ...]", file=sys.stderr)
        return 1

    apk_path = argv[0]
    abis = argv[1:] if argv[1:] else DEFAULT_ABIS

    if not os.path.isfile(apk_path):
        print("[FAIL] APK not found: {0}".format(apk_path))
        return 1

    try:
        entries = lib_inventory(apk_path)
    except zipfile.BadZipFile:
        print("[FAIL] not a readable zip/APK file: {0}".format(apk_path))
        return 1

    print("[INFO] lib/ inventory of {0} ({1} entries):".format(apk_path, len(entries)))
    if entries:
        for name in entries:
            print("    {0}".format(name))
    else:
        print("    (no lib/ entries)")

    missing = missing_abis_from_entries(entries, abis)
    if missing:
        for abi in missing:
            print("[FAIL] missing lib/{0}/{1}".format(abi, CORE_LIB))
        print("[FAIL] {0} of {1} requested ABI(s) missing from {2}".format(len(missing), len(abis), apk_path))
        return 1

    print("[OK] {0} contains {1} for all {2} requested ABI(s): {3}".format(
        apk_path, CORE_LIB, len(abis), ", ".join(abis)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
