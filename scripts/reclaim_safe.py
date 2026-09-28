#!/usr/bin/env python3
"""Safe-to-reclaim survey and reclamation tool for git worktree build artifacts.

Answers one question per worktree: is it safe to delete its target/ dir?
Optionally deletes target/ for worktrees proven SAFE.

The bug this exists to avoid: `git merge-base --is-ancestor` returns:
  0   = merged
  1   = NOT merged
  128 = REF ERROR (could not determine)

Treating "non-zero" as "not merged" turns 128 into a permanent false negative.
GitHub deletes a branch on merge, `git fetch --prune` drops the local ref, and
the check then reports a merged branch as unmerged forever (e.g. PR #165).

Safety properties, all three required before reclaim is advised:
  1. CLEAN       -- no uncommitted work beyond pending .md line-ending churn
                    (#169 declared *.md text eol=lf without renormalizing).
  2. NO UNPUSHED -- git rev-list --count HEAD --not --remotes == 0. Nothing that
                    exists on no remote (CTO_STATE section 8).
  3. MERGED      -- HEAD is an ancestor of a durable ref, or its PR reports MERGED.
                    On 128/ref-error, returns UNKNOWN (never NOT-MERGED or SAFE),
                    and falls back to gh PR state check.

Reclaim action:
  --reclaim: Deletes target/ inside SAFE worktrees ONLY.
  Never touches core/target/generated-sources/ or any path outside target/.

Usage:
  python scripts/reclaim_safe.py           # survey only (default, safe)
  python scripts/reclaim_safe.py --survey  # survey only (explicit)
  python scripts/reclaim_safe.py --reclaim # delete target/ in SAFE worktrees
  python scripts/reclaim_safe.py --reclaim --dry-run # show what would be deleted
"""

import argparse
import errno
import json
import os
import shutil
import stat
import subprocess
import sys

DEFAULT_DURABLE = ["origin/tracking/pre-v040-tag-work", "origin/main"]

# Managed classes outside the checkout (added 2026-09-17). Imported from
# disk_budget so the reporter and the deleter cannot disagree about what a
# class contains -- a guard that measures a different set than the tool that
# deletes is how a "22 GB cache" becomes an unnoticed 22 GB.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from disk_budget import (  # noqa: E402
    ANDROID_AVD_HOME,
    EMULATOR_STATE_NAMES,
    NEVER_RECLAIMABLE_SUFFIXES,
    SHARED_TARGET_HOME,
    is_build_script_output,
)


def _processes_under(path):
    """PIDs of processes whose image is inside `path`. Returns (pids, ok).

    ok=False means detection itself failed. Callers MUST treat that as a
    refusal: a running node's binary inside target/ is exactly how a sanctioned
    reclaim on 2026-09-17 deleted a live node's restart path.
    """
    if os.name != "nt":
        return [], True
    ps = ("Get-Process | Where-Object { $_.Path -like '*%s*' } | "
          "Select-Object -ExpandProperty Id" % path.replace("\\", "\\"))
    rc, out, err = run_cmd(["powershell", "-NoProfile", "-Command", ps], timeout=90)
    if rc != 0:
        return [], False
    return [l.strip() for l in out.splitlines() if l.strip()], True


def _emulator_processes():
    """Lines from tasklist matching an emulator/qemu process. (lines, ok)."""
    rc, out, err = run_cmd(["tasklist"], timeout=90)
    if rc != 0:
        return [], False
    return [l for l in out.splitlines()
            if "qemu" in l.lower() or "emulator" in l.lower()], True


def _durable_hits(root):
    """Non-build files under root that mean the path is not a pure cache."""
    hits = []
    for dirpath, dirnames, filenames in os.walk(root):
        for n in filenames:
            if not n.lower().endswith(NEVER_RECLAIMABLE_SUFFIXES):
                continue
            rel = os.path.relpath(os.path.join(dirpath, n), root)
            if is_build_script_output(rel):
                continue
            try:
                sz = os.path.getsize(os.path.join(dirpath, n))
            except OSError:
                sz = 0
            hits.append((rel, sz))
    return hits


def reclaim_shared_target(dry_run=False):
    """Delete the children of the shared cargo target cache, keeping the dir."""
    root = SHARED_TARGET_HOME
    print("\n--- Shared cargo target %s ---" % ("[DRY RUN]" if dry_run else "[LIVE]"))
    if not os.path.isdir(root):
        print("[INFO]  %s does not exist; nothing to reclaim." % root)
        return 0

    pids, ok = _processes_under(root)
    if not ok:
        print("[FAIL]  Could not determine whether a process is running from %s." % root)
        print("        Refusing to delete (fail closed).")
        return 0
    if pids:
        print("[FAIL]  Process(es) running from this tree: %s" % ", ".join(pids))
        print("        Refusing to delete: a running binary must not lose its image.")
        return 0
    print("[OK]    No process is running from this tree.")

    durable = _durable_hits(root)
    if durable:
        print("[FAIL]  %d non-build file(s) present -- this is not a pure cache:"
              % len(durable))
        for rel, sz in durable[:20]:
            print("          %10.3f MB  %s" % (sz / 1024 ** 2, rel))
        print("        Refusing to bulk-delete. Reclaim the build output by hand.")
        return 0
    print("[OK]    No durable (non-build) files present.")

    children = [os.path.join(root, n) for n in sorted(os.listdir(root))]
    freed = 0
    for child in children:
        size_bytes = get_dir_size_bytes(child) if os.path.isdir(child) else 0
        if dry_run:
            print("[DRY-RUN] Would reclaim %s (%s)" % (child, format_bytes(size_bytes)))
            freed += size_bytes
            continue
        try:
            if os.path.isdir(child):
                shutil.rmtree(child, onerror=handle_remove_readonly)
            else:
                os.remove(child)
            print("[OK]   Reclaimed %s (%s freed)" % (child, format_bytes(size_bytes)))
            freed += size_bytes
        except Exception as e:
            print("[FAIL] Failed to remove %s: %s" % (child, e))
    print("[DONE] Shared target: %s reclaimed." % format_bytes(freed))
    return freed


def reclaim_emulator_state(dry_run=False):
    """Delete AVD runtime state, keeping the AVD definition so it re-spawns."""
    print("\n--- Emulator runtime state %s ---" % ("[DRY RUN]" if dry_run else "[LIVE]"))
    if not os.path.isdir(ANDROID_AVD_HOME):
        print("[INFO]  %s does not exist; nothing to reclaim." % ANDROID_AVD_HOME)
        return 0

    hits, ok = _emulator_processes()
    if not ok:
        print("[FAIL]  Could not enumerate processes. Refusing to delete (fail closed).")
        return 0
    if hits:
        print("[FAIL]  An emulator appears to be running:")
        for line in hits[:5]:
            print("          %s" % line.strip())
        print("        Refusing to delete live emulator state.")
        return 0
    print("[OK]    No emulator/qemu process is running.")

    freed = 0
    kept = []
    for avd in sorted(os.listdir(ANDROID_AVD_HOME)):
        avd_path = os.path.join(ANDROID_AVD_HOME, avd)
        if not os.path.isdir(avd_path) or not avd.endswith(".avd"):
            continue
        # Guard: never step outside the AVD home.
        if not os.path.abspath(avd_path).startswith(os.path.abspath(ANDROID_AVD_HOME) + os.sep):
            print("[FAIL]  Refusing path outside AVD home: %s" % avd_path)
            continue
        for name in EMULATOR_STATE_NAMES:
            p = os.path.join(avd_path, name)
            if not os.path.exists(p):
                continue
            size_bytes = get_dir_size_bytes(p) if os.path.isdir(p) else os.path.getsize(p)
            if dry_run:
                print("[DRY-RUN] Would reclaim %s (%s)" % (p, format_bytes(size_bytes)))
                freed += size_bytes
                continue
            try:
                if os.path.isdir(p):
                    shutil.rmtree(p, onerror=handle_remove_readonly)
                else:
                    os.remove(p)
                print("[OK]   Reclaimed %s (%s freed)" % (p, format_bytes(size_bytes)))
                freed += size_bytes
            except Exception as e:
                print("[FAIL] Failed to remove %s: %s" % (p, e))
        for keep in ("config.ini", "AVD.conf", "userdata.img"):
            if os.path.exists(os.path.join(avd_path, keep)):
                kept.append(os.path.join(avd, keep))

    print("[INFO]  Kept the AVD definition so it re-spawns fresh:")
    for k in kept:
        print("          %s" % k)
    print("[DONE] Emulator state: %s reclaimed." % format_bytes(freed))
    return freed


def find_repo_root():
    """Dynamically determine the repo root from git or file location."""
    try:
        p = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if p.returncode == 0 and p.stdout.strip():
            return os.path.abspath(p.stdout.strip())
    except Exception:
        pass
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.abspath(os.path.join(script_dir, ".."))


def run_cmd(args, cwd=None, timeout=30):
    """Run a subprocess command and return (returncode, stdout, stderr)."""
    try:
        p = subprocess.run(
            args,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return p.returncode, p.stdout.strip(), p.stderr.strip()
    except subprocess.TimeoutExpired:
        return 124, "", "command timed out after %ds" % timeout
    except Exception as e:
        return 127, "", str(e)


def list_worktrees(repo_root):
    """Return a list of absolute worktree paths registered in git."""
    rc, out, _ = run_cmd(["git", "worktree", "list", "--porcelain"], cwd=repo_root)
    if rc != 0:
        return []
    worktrees = []
    for line in out.splitlines():
        if line.startswith("worktree "):
            wt_path = line.split(" ", 1)[1].strip()
            if wt_path:
                worktrees.append(os.path.abspath(wt_path))
    return worktrees


def check_merged_state(head, durable_refs, cwd=None):
    """Return (verdict, detail).

    Verdict is one of: 'MERGED', 'NOT-MERGED', 'UNKNOWN'.
    Never collapses REF ERROR (rc=128 or other errors) into 'NOT-MERGED'.
    Falls back to GitHub PR query if git merge-base cannot determine or returns non-zero.
    """
    errors = []
    for ref in durable_refs:
        rc, _, err = run_cmd(["git", "merge-base", "--is-ancestor", head, ref], cwd=cwd)
        if rc == 0:
            return "MERGED", "ancestor of %s" % ref
        if rc != 1:
            errors.append("%s rc=%d %s" % (ref, rc, err[:40]))

    # Fall back to GitHub PR state check via gh CLI
    try:
        rc, out, err = run_cmd(
            [
                "gh",
                "pr",
                "list",
                "--state",
                "merged",
                "--search",
                head,
                "--json",
                "number,title,state,mergedAt",
            ],
            cwd=cwd,
            timeout=15,
        )
        if rc == 0 and out:
            prs = json.loads(out)
            if prs and any(p.get("state") == "MERGED" for p in prs):
                pr_num = prs[0].get("number")
                return "MERGED", "merged via PR #%s" % pr_num
    except Exception as e:
        errors.append("gh fallback: %s" % str(e)[:40])

    if errors:
        return "UNKNOWN", "; ".join(errors)
    return "NOT-MERGED", "not an ancestor of any durable ref"


def get_dir_size_bytes(path):
    """Compute total size in bytes of all files within a directory."""
    total = 0
    if not os.path.exists(path):
        return 0
    try:
        for root, _, files in os.walk(path):
            for f in files:
                fp = os.path.join(root, f)
                try:
                    total += os.path.getsize(fp)
                except OSError:
                    pass
    except OSError:
        pass
    return total


def format_bytes(num_bytes):
    """Format bytes into human-readable string."""
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if abs(num_bytes) < 1024.0:
            return "%.2f %s" % (num_bytes, unit)
        num_bytes /= 1024.0
    return "%.2f PB" % num_bytes


def handle_remove_readonly(func, path, exc):
    """Error handler for shutil.rmtree to remove Windows read-only flags."""
    excvalue = exc[1]
    if func in (os.rmdir, os.remove, os.unlink) and excvalue.errno == errno.EACCES:
        try:
            os.chmod(path, stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)
            func(path)
        except Exception:
            raise
    else:
        raise


def resolve_durable_refs(repo_root, refs):
    """Split `refs` into (present, missing) by whether the ref resolves here.

    Why: DEFAULT_DURABLE names `origin/tracking/pre-v040-tag-work`, which does
    not exist on this clone or on origin. `check_merged_state` reports a ref
    error as UNKNOWN rather than a false NOT-MERGED (correct, and documented),
    but a phantom in the default list therefore makes EVERY worktree that is not
    an ancestor of origin/main permanently UNKNOWN -- 7 of 11 on 2026-09-17 --
    and makes the tool exit 1 unconditionally. A safety check that can never
    reach SAFE is not conservative, it is inert.

    Missing refs are printed, not silently dropped (rule 15: visibility fails
    open).
    """
    present, missing = [], []
    for ref in refs:
        rc, out, err = run_cmd(["git", "rev-parse", "--verify", "--quiet", ref],
                               cwd=repo_root, timeout=15)
        (present if rc == 0 else missing).append(ref)
    return present, missing


def survey_worktrees(repo_root, durable_refs):
    """Run safety survey across all registered git worktrees.

    Returns a list of dicts with keys:
      name, path, dirty_count, unpushed_count, merged_verdict, detail, verdict, is_safe
    """
    results = []
    for w in list_worktrees(repo_root):
        norm_w = os.path.abspath(w)
        name = os.path.basename(norm_w.rstrip("/\\"))
        if not os.path.isdir(norm_w):
            results.append({
                "name": name,
                "path": norm_w,
                "dirty_count": "-",
                "unpushed_count": "-",
                "merged_verdict": "PATH-GONE",
                "detail": "registered but not on disk; run git worktree prune",
                "verdict": "HOLD",
                "is_safe": False,
            })
            continue

        rc, head, _ = run_cmd(["git", "rev-parse", "HEAD"], cwd=norm_w)
        if rc != 0 or not head:
            results.append({
                "name": name,
                "path": norm_w,
                "dirty_count": "?",
                "unpushed_count": "?",
                "merged_verdict": "UNKNOWN",
                "detail": "cannot rev-parse HEAD",
                "verdict": "HOLD",
                "is_safe": False,
            })
            continue

        _, st, _ = run_cmd(["git", "status", "--porcelain"], cwd=norm_w)
        # Filter out unstaged markdown line-ending churn from pending renormalization
        real_dirty = [
            l for l in st.splitlines()
            if l.strip() and not (l.startswith(" M ") and (l.endswith(".md") or l.endswith('.md"')))
        ]
        dirty_count = len(real_dirty)

        _, unp, _ = run_cmd(["git", "rev-list", "--count", head, "--not", "--remotes"], cwd=norm_w)
        unpushed_count = unp.strip() if unp.strip().isdigit() else "?"

        merged_verdict, detail = check_merged_state(head, durable_refs, cwd=norm_w)

        is_safe = (dirty_count == 0) and (unpushed_count == "0") and (merged_verdict == "MERGED")
        verdict = "SAFE" if is_safe else "HOLD"

        results.append({
            "name": name,
            "path": norm_w,
            "dirty_count": str(dirty_count),
            "unpushed_count": unpushed_count,
            "merged_verdict": merged_verdict,
            "detail": detail,
            "verdict": verdict,
            "is_safe": is_safe,
        })
    return results


def print_survey_table(results):
    """Print complete survey table per AGENTS.md rule 15 (no truncation)."""
    header_fmt = "%-30s %-6s %-9s %-11s %-6s %s"
    print(header_fmt % ("WORKTREE", "DIRTY", "UNPUSHED", "MERGED", "VERDICT", "WHY"))
    print("-" * 110)
    for r in results:
        merged_col = r["merged_verdict"]
        verdict_col = r["verdict"] if r["merged_verdict"] != "PATH-GONE" else "-"
        print(header_fmt % (
            r["name"],
            r["dirty_count"],
            r["unpushed_count"],
            merged_col,
            verdict_col,
            r["detail"],
        ))

    print()
    print("SAFE to reclaim target/:")
    safe_entries = [r for r in results if r["is_safe"]]
    if safe_entries:
        for r in safe_entries:
            print("  %s" % r["name"])
    else:
        print("  (none)")


def perform_reclaim(results, dry_run=False):
    """Reclaim target/ directories inside SAFE worktrees ONLY.

    Strictly refuses non-SAFE worktrees and preserves core/target/generated-sources/.
    """
    safe_worktrees = [r for r in results if r["is_safe"]]
    if not safe_worktrees:
        print("\n[INFO] No SAFE worktrees available for reclamation.")
        return 0, 0

    print("\n--- Reclamation %s ---" % ("[DRY RUN]" if dry_run else "[LIVE]"))
    total_bytes_freed = 0
    reclaimed_count = 0

    for r in safe_worktrees:
        wt_path = r["path"]
        target_dir = os.path.join(wt_path, "target")

        # Invariant checks: must be named target and directly under worktree path
        if os.path.basename(target_dir) != "target" or os.path.dirname(target_dir) != wt_path:
            print("[ERROR] Safety check failed: invalid target path %s" % target_dir)
            continue

        # Invariant check: never touch generated-sources
        if "generated-sources" in target_dir:
            print("[ERROR] Safety check failed: generated-sources path protected: %s" % target_dir)
            continue

        if not os.path.exists(target_dir):
            print("[INFO]  %-30s : target/ does not exist (0 B)" % r["name"])
            continue

        size_bytes = get_dir_size_bytes(target_dir)
        size_str = format_bytes(size_bytes)

        if dry_run:
            print("[DRY-RUN] Would reclaim %s (%s)" % (target_dir, size_str))
            total_bytes_freed += size_bytes
            reclaimed_count += 1
        else:
            try:
                shutil.rmtree(target_dir, onerror=handle_remove_readonly)
                print("[OK]   Reclaimed %s (%s freed)" % (target_dir, size_str))
                total_bytes_freed += size_bytes
                reclaimed_count += 1
            except Exception as e:
                print("[FAIL] Failed to remove %s: %s" % (target_dir, e))

    print("\n[DONE] Reclaimed %d target directory(ies), %s total freed." % (
        reclaimed_count,
        format_bytes(total_bytes_freed),
    ))
    return reclaimed_count, total_bytes_freed


def main():
    parser = argparse.ArgumentParser(
        description="Safe-to-reclaim survey and reclamation tool for git worktree build artifacts."
    )
    parser.add_argument(
        "--survey",
        action="store_true",
        default=True,
        help="Run survey and print status table (default action).",
    )
    parser.add_argument(
        "--reclaim",
        action="store_true",
        default=False,
        help="Reclaim target/ inside SAFE worktrees only.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=False,
        help="Simulate reclamation without deleting files.",
    )
    parser.add_argument(
        "--durable-ref",
        action="append",
        dest="durable_refs",
        default=None,
        help="Add durable ref to check ancestry against (default: tracking branch and main).",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="json_output",
        default=False,
        help="Output survey results in JSON format.",
    )
    parser.add_argument(
        "--reclaim-shared-target",
        action="store_true",
        default=False,
        help="Reclaim the shared cargo target cache (verified: no process running "
             "from it, no durable non-build files).",
    )
    parser.add_argument(
        "--reclaim-emulator-state",
        action="store_true",
        default=False,
        help="Reclaim AVD runtime state, keeping the AVD definition so it re-spawns.",
    )

    args = parser.parse_args()

    repo_root = find_repo_root()
    durable_refs = args.durable_refs or DEFAULT_DURABLE

    present, missing = resolve_durable_refs(repo_root, durable_refs)
    if missing:
        print("[WARNING] Durable ref(s) do not resolve here and cannot prove a merge:",
              file=sys.stderr)
        for ref in missing:
            print("          %s" % ref, file=sys.stderr)
    if not present:
        print("[WARNING] No durable ref resolved; falling back to origin/main only.",
              file=sys.stderr)
        present = ["origin/main"]
    durable_refs = present

    results = survey_worktrees(repo_root, durable_refs)

    if args.json_output:
        print(json.dumps(results, indent=2))
    else:
        print_survey_table(results)

    if args.reclaim:
        perform_reclaim(results, dry_run=args.dry_run)

    # The two managed classes outside the checkout have their own gates and do
    # not participate in the worktree SAFE/UNKNOWN model.
    if args.reclaim_shared_target:
        reclaim_shared_target(dry_run=args.dry_run)

    if args.reclaim_emulator_state:
        reclaim_emulator_state(dry_run=args.dry_run)

    # Exit non-zero if any worktree has UNKNOWN verdict so callers cannot mistake it for clean
    has_unknown = any(r["merged_verdict"] == "UNKNOWN" for r in results)
    if has_unknown:
        print("\n[WARNING] One or more worktrees have UNKNOWN merged state.", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
