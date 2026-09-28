#!/usr/bin/env python3
"""Disk budget guard for this checkout (AGENTS.md rule 17).

Report-only by design. It never deletes anything. Deletion lives in ONE place,
`scripts/reclaim_safe.py`, which has its own safety model (clean + nothing
unpushed + merged before it will touch a worktree's `target/`).

Why this exists: on 2026-09-17 the host hit 100% (1.2 GB free) after eleven git
worktrees each accumulated a cargo `target/`. The single tree that mattered
measured 20.7 GB of build output, and every worktree is capable of growing its
own. Nothing in the toolchain paces that, so it has to be a checked state.

Visibility fails open, the verdict fails closed (rule 15): this prints ALL
candidates it finds, and only the exit code carries the verdict.

Usage:
  python scripts/disk_budget.py            # survey (default)
  python scripts/disk_budget.py --tight 25 --floor 10
  python scripts/disk_budget.py --json

Exit codes:
  0  OK       -- at or above the tight threshold
  1  TIGHT    -- below the tight threshold but above the hard floor
  2  BLOCKED  -- below the hard floor: do not start a build here
"""

import argparse
import json
import os
import shutil
import subprocess
import sys

GB = 1024 ** 3

DEFAULT_TIGHT_GB = 20.0
DEFAULT_FLOOR_GB = 8.0

# --- managed classes OUTSIDE the checkout -----------------------------------
#
# The worktree model below misses these entirely, because they belong to no
# single tree. On 2026-09-17 the shared cargo cache alone measured 22.22 GB and
# the emulator's runtime state another 6.5 GB, while this guard reported the two
# paths it was looking for and called the disk TIGHT. A guard that cannot see a
# class cannot warn about it (rule 15).
SHARED_TARGET_HOME = os.environ.get(
    "SCM_SHARED_TARGET", r"C:\Users\SCM\Documents\GitHub\.scm-shared-target")
ANDROID_AVD_HOME = os.path.join(os.path.expanduser("~"), ".android", "avd")

# Emulator runtime state only. The AVD *definition* -- config.ini, AVD.conf,
# userdata.img -- is deliberately NOT in this list: it is the thing that lets the
# AVD re-spawn fresh, and deleting it is the difference between "clean" and
# "broken".
EMULATOR_STATE_NAMES = [
    "snapshots",              # boot snapshots, rebuilt on first boot
    "userdata-qemu.img.qcow2",  # live userdata overlay, rebuilt from userdata.img
    "cache.img", "cache.img.qcow2",
    "encryptionkey.img", "encryptionkey.img.qcow2",
    "hardware-qemu.ini", "hardware-qemu.ini.lock",
    "multiinstance.lock", "tmpAdbCmds",
    "read-snapshot.txt", "bootcompleted.ini",
]

# A reclaimable class must be 100% regenerable. These extensions must never
# appear under a path this guard reports as reclaimable -- if one does, the
# class is not a cache and the verdict must fail closed (rule 15).
NEVER_RECLAIMABLE_SUFFIXES = (
    ".log", ".pid", ".db", ".pem", ".key", ".jsonl", ".sqlite", ".sqlite3",
    ".wal", ".img", ".apk", ".aab", ".backup", ".bak", ".md", ".py",
    ".ps1", ".sh", ".toml", ".rs", ".kt", ".yml", ".yaml", ".csv",
)

# Build-script outputs live under build/*/out/ and match the suffix list above
# while still being perfectly regenerable. They are the only sanctioned
# exception, and they are matched by path, not by suffix alone.
def is_build_script_output(rel_path):
    parts = rel_path.replace("/", os.sep).split(os.sep)
    return "build" in parts and "out" in parts


def _durable_files(path, budget_seconds=45):
    """Files under `path` that indicate the class is NOT a pure cache."""
    import time
    hits = []
    if not os.path.isdir(path):
        return hits
    started = time.time()
    stack = [path]
    while stack:
        if time.time() - started > budget_seconds:
            break
        d = stack.pop()
        try:
            with os.scandir(d) as it:
                for e in it:
                    try:
                        if e.is_dir(follow_symlinks=False):
                            stack.append(e.path)
                        elif e.is_file(follow_symlinks=False):
                            if e.name.lower().endswith(NEVER_RECLAIMABLE_SUFFIXES):
                                rel = os.path.relpath(e.path, path)
                                if not is_build_script_output(rel):
                                    hits.append((rel, e.stat().st_size))
                    except OSError:
                        pass
        except OSError:
            pass
    return hits


def external_classes():
    """Managed classes outside the checkout, with a durability gate each."""
    out = []

    if os.path.isdir(SHARED_TARGET_HOME):
        children = []
        for name in sorted(os.listdir(SHARED_TARGET_HOME)):
            p = os.path.join(SHARED_TARGET_HOME, name)
            if os.path.isdir(p):
                b, complete = dir_size(p)
                children.append({"path": p, "bytes": b, "complete": complete})
            else:
                try:
                    children.append({"path": p, "bytes": os.path.getsize(p),
                                     "complete": True})
                except OSError:
                    pass
        durable = []
        for c in children:
            durable.extend((c["path"], sz) for _, sz in _durable_files(c["path"]))
        out.append({
            "scope": "shared-target",
            "path": SHARED_TARGET_HOME,
            "bytes": sum(c["bytes"] for c in children),
            "complete": all(c["complete"] for c in children),
            "children": children,
            "durable_files": durable,
            "reclaim_cmd": "python scripts/reclaim_safe.py --reclaim-shared-target",
            "note": "shared cargo warm cache; documented in docs/rules/BUILD_AND_CI.md",
        })

    if os.path.isdir(ANDROID_AVD_HOME):
        for avd in sorted(os.listdir(ANDROID_AVD_HOME)):
            if not avd.endswith(".avd"):
                continue
            avd_path = os.path.join(ANDROID_AVD_HOME, avd)
            total, complete, durable = 0, True, []
            for name in EMULATOR_STATE_NAMES:
                p = os.path.join(avd_path, name)
                if not os.path.exists(p):
                    continue
                if os.path.isdir(p):
                    b, c = dir_size(p)
                    complete = complete and c
                else:
                    try:
                        b = os.path.getsize(p)
                    except OSError:
                        b = 0
                total += b
                durable.extend((p, sz) for _, sz in _durable_files(p))
            out.append({
                "scope": "emulator",
                "path": avd_path,
                "bytes": total,
                "complete": complete,
                "durable_files": durable,
                "reclaim_cmd": "python scripts/reclaim_safe.py --reclaim-emulator-state",
                "note": "runtime state only; config.ini/AVD.conf/userdata.img are KEPT so the AVD re-spawns",
            })

    return out

# Paths inside one checkout that are build output or cached downloads. Every
# entry is regenerable: cargo/gradle rebuild them, the CI artifacts in
# .codebuff_deploy are re-downloadable with `gh run download`.
CANDIDATE_DIRS = [
    "target",
    "core/target",
    "cli/target",
    "android/app/build",
    "iOS/build",
    "DerivedData",
]


def repo_root():
    p = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                       capture_output=True, text=True)
    if p.returncode == 0 and p.stdout.strip():
        return os.path.abspath(p.stdout.strip())
    return os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))


def dir_size(path, budget_seconds=45):
    """Total bytes under path. Returns (bytes, complete).

    complete=False means the walk hit its time budget and the number is a
    FLOOR, not a total -- callers must say so rather than print it bare.
    """
    import time
    if not os.path.isdir(path):
        return 0, True
    started = time.time()
    total = 0
    stack = [path]
    while stack:
        if time.time() - started > budget_seconds:
            return total, False
        d = stack.pop()
        try:
            with os.scandir(d) as it:
                for e in it:
                    try:
                        if e.is_dir(follow_symlinks=False):
                            stack.append(e.path)
                        elif e.is_file(follow_symlinks=False):
                            total += e.stat(follow_symlinks=False).st_size
                    except OSError:
                        pass
        except OSError:
            pass
    return total, True


def worktrees(root):
    p = subprocess.run(["git", "worktree", "list", "--porcelain"],
                       capture_output=True, text=True, cwd=root)
    if p.returncode != 0:
        return []
    out = []
    for line in p.stdout.splitlines():
        if line.startswith("worktree "):
            out.append(line.split(" ", 1)[1].strip())
    return out


def human(n):
    return "%.2f GB" % (n / GB)


def main():
    ap = argparse.ArgumentParser(description="Repo disk budget guard (report-only).")
    ap.add_argument("--tight", type=float, default=DEFAULT_TIGHT_GB,
                    help="warn below this many GB free (default %g)" % DEFAULT_TIGHT_GB)
    ap.add_argument("--floor", type=float, default=DEFAULT_FLOOR_GB,
                    help="hard-stop below this many GB free (default %g)" % DEFAULT_FLOOR_GB)
    ap.add_argument("--json", action="store_true", dest="as_json")
    args = ap.parse_args()

    root = repo_root()
    usage = shutil.disk_usage(root)
    free_gb = usage.free / GB

    if free_gb < args.floor:
        verdict = "BLOCKED"
    elif free_gb < args.tight:
        verdict = "TIGHT"
    else:
        verdict = "OK"

    # Every candidate path in the repo, plus every registered worktree's
    # target/. Duplicates are collapsed so the total is not double-counted.
    candidates = []
    seen = set()
    for rel in CANDIDATE_DIRS:
        p = os.path.join(root, rel)
        if os.path.isdir(p) and p not in seen:
            seen.add(p)
            b, complete = dir_size(p)
            if b > 0:
                candidates.append({"path": p, "bytes": b, "complete": complete,
                                   "scope": "checkout"})
    for wt in worktrees(root):
        p = os.path.join(wt, "target")
        if os.path.isdir(p) and p not in seen:
            seen.add(p)
            b, complete = dir_size(p)
            if b > 0:
                candidates.append({"path": p, "bytes": b, "complete": complete,
                                   "scope": "worktree"})

    candidates.sort(key=lambda c: c["bytes"], reverse=True)

    external = external_classes()
    reclaimable = sum(c["bytes"] for c in candidates)
    reclaimable += sum(e["bytes"] for e in external)

    if args.as_json:
        print(json.dumps({
            "root": root,
            "total_gb": round(usage.total / GB, 2),
            "free_gb": round(free_gb, 2),
            "used_pct": round(100.0 * usage.used / usage.total, 1),
            "tight_gb": args.tight,
            "floor_gb": args.floor,
            "verdict": verdict,
            "reclaimable_bytes": reclaimable,
            "candidates": candidates,
            "external_classes": external,
        }, indent=2))
        return {"OK": 0, "TIGHT": 1, "BLOCKED": 2}[verdict]

    print("Disk budget guard -- %s" % root)
    print("=" * 72)
    print("  filesystem : %s total, %s free (%.1f%% used)"
          % (human(usage.total), human(usage.free),
             100.0 * usage.used / usage.total))
    print("  thresholds : TIGHT below %g GB, BLOCKED below %g GB" % (args.tight, args.floor))
    print("  verdict    : %s" % verdict)
    print()

    print("Reclaimable build output / caches found (%d entries):" % len(candidates))
    if not candidates:
        print("  (none -- no target/ or build output in this checkout or its worktrees)")
    for c in candidates:
        note = "" if c["complete"] else "  [WARNING] size is a FLOOR, walk hit its time budget"
        print("  %-12s %10s  %s%s" % (c["scope"], human(c["bytes"]), c["path"], note))
    print()
    print("  total reclaimable: %s" % human(reclaimable))
    print()

    print("Managed classes OUTSIDE the checkout (%d entries):" % len(external))
    if not external:
        print("  (none found)")
    for e in external:
        note = "" if e["complete"] else "  [WARNING] size is a FLOOR, walk hit its time budget"
        print("  %-14s %10s  %s%s" % (e["scope"], human(e["bytes"]), e["path"], note))
        if e["durable_files"]:
            print("    [WARNING] %d non-build file(s) present -- NOT a pure cache, do not bulk-delete:"
                  % len(e["durable_files"]))
            for rel, sz in e["durable_files"][:10]:
                print("      %10.3f MB  %s" % (sz / 1024 ** 2, rel))
    print()

    print("To reclaim (deletion happens in reclaim_safe.py ONLY):")
    print("  python scripts/reclaim_safe.py                      # safety survey, no deletion")
    print("  python scripts/reclaim_safe.py --reclaim            # delete target/ in SAFE worktrees only")
    print("  python scripts/reclaim_safe.py --reclaim-shared-target   # shared cargo cache")
    print("  python scripts/reclaim_safe.py --reclaim-emulator-state  # AVD runtime state")
    print("  rm -rf target                                       # this checkout's build output")
    print()
    print("NOT reclaimable and never to be deleted by an agent:")
    print("  tmp/ evidence, ~/.scm-purge-backup-*, identity keys, /opt/scm-relay-data,")
    print("  pagefile.sys / hiberfil.sys (system-owned).")

    if verdict == "BLOCKED":
        print()
        print("[FAIL] below the hard floor -- do not start a cargo/gradle build here.")
        print("       Use CI for the build and download the artifact instead:")
        print("       gh run download <run-id> -n <artifact-name> -D tmp/<dir>")
    elif verdict == "TIGHT":
        print()
        print("[WARNING] below the tight threshold -- reclaim before a full build;")
        print("          prefer a single targeted test over a whole-workspace build.")

    return {"OK": 0, "TIGHT": 1, "BLOCKED": 2}[verdict]


if __name__ == "__main__":
    sys.exit(main())
