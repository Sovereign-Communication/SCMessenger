#!/usr/bin/env python3
"""Comprehensive Phase 1 audit: all branches, worktrees, and context via Harness."""
import sys
import subprocess
from pathlib import Path
from harness.condenser import distill_context

def run_git(*args):
    """Run git command and return output."""
    try:
        result = subprocess.run(['git'] + list(args), capture_output=True, text=True, cwd=Path.cwd())
        return result.stdout.strip()
    except Exception:
        return ""

def get_branches():
    """Get all branches (local + remote tracking)."""
    output = run_git('branch', '-a')
    branches = []
    for line in output.split('\n'):
        line = line.strip()
        if line and not line.startswith('*'):
            # Remove "remotes/origin/" prefix
            if line.startswith('remotes/'):
                line = line.replace('remotes/origin/', '')
            branches.append(line)
    branches = list(set(branches))  # deduplicate
    # Always include current branch
    current = run_git('rev-parse', '--abbrev-ref', 'HEAD')
    if current:
        branches.append(current)
    return list(set(branches))

def get_worktrees():
    """Get all git worktree paths."""
    output = run_git('worktree', 'list', '--porcelain')
    worktrees = []
    for line in output.split('\n'):
        if line.startswith('worktree'):
            path = line.split()[1]
            worktrees.append(Path(path))
    return worktrees

def read_critical_files(root: Path, patterns: list[str]) -> dict[str, str]:
    """Read critical files from a directory."""
    files = {}
    for pattern in patterns:
        for path in sorted(root.glob(pattern)):
            if path.is_file() and not any(skip in str(path) for skip in ['.git', 'target', 'node_modules', '.gradle']):
                try:
                    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                        if len(content) < 500000:
                            rel_path = str(path.relative_to(root))
                            files[rel_path] = content
                except Exception:
                    pass
    return files

repo_root = Path.cwd()
critical_patterns = [
    "CLAUDE.md", "AGENTS.md", "docs/CLAUDE_REFERENCE.md", "docs/CURRENT_STATE.md",
    "REMAINING_WORK_TRACKING.md", "SHIP_PLAN.md", "HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md",
    "core/src/iron_core.rs", "core/src/contacts_bridge.rs", "cli/src/main.rs",
    "android/app/src/main/java/com/scmessenger/android/MainActivity.kt",
    "android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt",
    "android/app/src/main/AndroidManifest.xml",
    "core/src/store/backend.rs", "core/src/transport/abstraction.rs", "core/src/crypto/mod.rs",
    "core/src/identity/mod.rs", "core/src/lib.rs", "core/src/routing/mod.rs",
    "core/src/routing/engine.rs", "core/src/transport/mod.rs", "core/src/transport/manager.rs",
    "core/src/transport/behaviour.rs", "core/src/store/mod.rs", "core/src/store/contacts.rs",
    "core/src/store/history.rs", "core/src/store/outbox.rs", "core/src/store/inbox.rs",
    "docs/rules/ANDROID.md", "docs/rules/BUILD_AND_CI.md", "docs/rules/RUST_CONVENTIONS.md",
    "docs/rules/SECURITY_PROTOCOL.md", "Cargo.toml", ".github/workflows/ci.yml", "docker/Dockerfile",
    "android/app/build.gradle.kts", "HANDOFF/todo/ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md",
    "HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md",
    "HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md",
    "HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md", "HANDOFF/in_progress/A-05_IOS_RECEIPT_UNIFICATION.md",
    "HANDOFF/in_progress/D1_DESKTOP_BRIDGE_UNIFFI_VERIFICATION.md",
]

print("[INFO] Comprehensive audit: branches, worktrees, all context", file=sys.stderr)

all_files = {}

# Current checkout
print("[INFO] Reading from current checkout", file=sys.stderr)
current_branch = run_git('rev-parse', '--abbrev-ref', 'HEAD')
current_files = read_critical_files(repo_root, critical_patterns)
for path, content in current_files.items():
    all_files[f"[{current_branch}] {path}"] = content
print(f"  {len(current_files)} files from {current_branch}", file=sys.stderr)

# Main branch (if different)
if current_branch != "main":
    print("[INFO] Reading from main branch", file=sys.stderr)
    try:
        # Stash current changes, checkout main, read files, return
        run_git('stash')
        run_git('checkout', 'main')
        main_files = read_critical_files(repo_root, critical_patterns)
        for path, content in main_files.items():
            all_files[f"[main] {path}"] = content
        print(f"  {len(main_files)} files from main", file=sys.stderr)
        # Return to current branch
        run_git('checkout', current_branch)
        run_git('stash', 'pop')
    except Exception as e:
        print(f"  [WARNING] Failed to audit main: {e}", file=sys.stderr)
        run_git('checkout', current_branch)

# Worktrees
worktrees = get_worktrees()
if worktrees:
    for wt in worktrees:
        if wt.exists():
            print(f"[INFO] Reading from worktree: {wt}", file=sys.stderr)
            try:
                wt_files = read_critical_files(wt, critical_patterns)
                for path, content in wt_files.items():
                    all_files[f"[worktree:{wt.name}] {path}"] = content
                print(f"  {len(wt_files)} files from {wt.name}", file=sys.stderr)
            except Exception as e:
                print(f"  [WARNING] Failed to read worktree {wt}: {e}", file=sys.stderr)

print(f"[INFO] Total files collected: {len(all_files)}", file=sys.stderr)

summary = """
COMPREHENSIVE UNIFIED-PRODUCT AUDIT
Audit scope:
- Current branch (in-flight work)
- Main branch (canonical released state)
- All active worktrees (isolated checkouts)
- Strategic docs, architecture, blockers, platform details

Goal: Identify how to unify SCMessenger across Rust core, Android, CLI, future iOS/web.
Find: alignment points, platform divergence (intentional vs accidental), execution lanes, blockers, highest-impact change.
"""

print("[INFO] Running Harness distill_context()...", file=sys.stderr)
brief = distill_context(
    files=all_files,
    summary=summary,
    focus_symbols=["IronCore", "Swarm", "ContactsStore", "Transport", "Crypto", "StorageBackend", "MainActivity", "MeshForegroundService"]
)

print(brief.to_prompt_context())
print()
print("AUDIT SCOPE: Current branch, main, all worktrees. Full codebase coverage.")
print()
print("QUESTIONS:")
print("1. Architectural alignment points (3-4) that should be shared/synchronized?")
print("2. Platform divergence: intentional (why) vs accidental (cost/fix)?")
print("3. Execution/CI/release lanes for coordinating platforms?")
print("4. Top blocker to unification (build, capability, ownership)?")
print("5. Single highest-impact change toward unified execution?")
print()
print("Answer format: alignment points (abstraction + platforms), divergence (intent vs accident), lanes (name+trigger+criteria), blockers (impact %), impactful change (why+effort).")
print()
print("CRITICAL: Do NOT read files directly. Do NOT call file-read tools. ONLY use Harness MCP distill_context(files=[paths]) to get additional context. Harness manages token budgets; you do not cap it. Call it as needed to answer each question thoroughly.")
