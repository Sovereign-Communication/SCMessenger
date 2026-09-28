#!/usr/bin/env python3
"""Comprehensive Phase 1 audit v2: dedupe identical content across branches/worktrees, no aggressive truncation."""
import sys
import hashlib
import subprocess
from pathlib import Path
from harness.condenser import distill_context

def run_git(*args):
    try:
        result = subprocess.run(['git'] + list(args), capture_output=True, text=True, cwd=Path.cwd())
        return result.stdout.strip()
    except Exception:
        return ""

def get_worktrees():
    output = run_git('worktree', 'list', '--porcelain')
    worktrees = []
    for line in output.split('\n'):
        if line.startswith('worktree'):
            path = line.split()[1]
            worktrees.append(Path(path))
    return worktrees

def read_critical_files(root: Path, patterns: list[str]) -> dict[str, str]:
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

print("[INFO] Comprehensive audit v2: dedupe across branches/worktrees", file=sys.stderr)

# path -> {hash: content}, and path -> {hash: [sources]}
by_path_hash = {}      # (path, hash) -> content
hash_sources = {}      # (path, hash) -> [source labels]

current_branch = run_git('rev-parse', '--abbrev-ref', 'HEAD')

def ingest(label: str, root: Path):
    fs = read_critical_files(root, critical_patterns)
    for path, content in fs.items():
        h = hashlib.sha1(content.encode('utf-8', errors='ignore')).hexdigest()[:8]
        key = (path, h)
        by_path_hash[key] = content
        hash_sources.setdefault(key, []).append(label)
    return len(fs)

n = ingest(current_branch, repo_root)
print(f"  {n} files from {current_branch} (current checkout)", file=sys.stderr)

if current_branch != "main":
    try:
        run_git('stash')
        run_git('checkout', 'main')
        n = ingest("main", repo_root)
        print(f"  {n} files from main", file=sys.stderr)
    except Exception as e:
        print(f"  [WARNING] main audit failed: {e}", file=sys.stderr)
    finally:
        run_git('checkout', current_branch)
        run_git('stash', 'pop')

for wt in get_worktrees():
    if wt.exists() and wt.resolve() != repo_root.resolve():
        try:
            n = ingest(wt.name, wt)
            print(f"  {n} files from worktree:{wt.name}", file=sys.stderr)
        except Exception as e:
            print(f"  [WARNING] worktree {wt} failed: {e}", file=sys.stderr)

print(f"[INFO] Unique (path,content) pairs: {len(by_path_hash)}", file=sys.stderr)

# Build dedup'd file dict for distill_context: path -> content, but annotate
# variants when the same path has multiple distinct contents across sources.
path_variant_count = {}
for (path, h) in by_path_hash:
    path_variant_count[path] = path_variant_count.get(path, 0) + 1

dedup_files = {}
variant_notes = []
for (path, h), content in by_path_hash.items():
    sources = hash_sources[(path, h)]
    if path_variant_count[path] > 1:
        # multiple distinct versions of this path exist across sources
        key = f"{path} [variant {h} :: {','.join(sources)}]"
        variant_notes.append(f"{path}: {path_variant_count[path]} distinct versions across {sum(len(v) for (p,hh),v in hash_sources.items() if p==path)} sources")
    else:
        key = f"{path} [{','.join(sources)}]"
    dedup_files[key] = content

print(f"[INFO] Deduped to {len(dedup_files)} entries (identical content across branches/worktrees collapsed)", file=sys.stderr)

summary = """COMPREHENSIVE UNIFIED-PRODUCT AUDIT. Scope: current branch, main, all active worktrees. Identical file content across sources is deduplicated (one copy, tagged with all sources that share it); files that DIFFER across sources are kept as separate variants (tagged with source + hash) so divergence is visible. Goal: unify SCMessenger across Rust core, Android, CLI, future iOS/web. Find alignment points, platform divergence (intentional vs accidental), execution lanes, blockers, highest-impact change."""

print("[INFO] Running Harness distill_context() with no aggressive truncation...", file=sys.stderr)
# Set max_tokens high enough that the real content survives; Harness still
# structures/extracts signatures, it just won't chop everything to near-nothing.
brief = distill_context(
    files=dedup_files,
    max_tokens=60000,
    summary=summary,
    focus_symbols=["IronCore", "Swarm", "ContactsStore", "Transport", "Crypto", "StorageBackend", "MainActivity", "MeshForegroundService"]
)

print(brief.to_prompt_context())
print()
if variant_notes:
    print("DIVERGENT FILES (differ across branches/worktrees):")
    for note in sorted(set(variant_notes)):
        print(f"- {note}")
    print()
print("AUDIT SCOPE: current branch + main + all worktrees. Deduplicated, divergence flagged.")
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
