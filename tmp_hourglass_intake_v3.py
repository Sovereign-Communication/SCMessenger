#!/usr/bin/env python3
"""Phase 1 (Context Intake) — minimal formatting, maximum signal."""
import sys
from pathlib import Path
from harness.condenser import distill_context

def read_files(root: Path, patterns: list[str]) -> dict[str, str]:
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

intake_patterns = [
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

files = read_files(repo_root, intake_patterns)
print(f"[INFO] Read {len(files)} files", file=sys.stderr)

summary = "SCMessenger unified product audit: Rust core, Android app (UniFFI), CLI, future iOS/web. Find architectural alignment points, platform divergence, execution lanes, blockers to unification, highest-impact change."

brief = distill_context(
    files=files, max_tokens=8000, summary=summary,
    focus_symbols=["IronCore", "Swarm", "ContactsStore", "Transport", "Crypto", "StorageBackend", "MainActivity", "MeshForegroundService"]
)

print(brief.to_prompt_context())
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
print("If you need more context, request by file path + section.")
