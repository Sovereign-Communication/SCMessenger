#!/usr/bin/env python3
"""
Phase 1 (Context Intake) - Enhanced with critical architecture files.
Uses Harness Condenser to distill repo context with sufficient depth for unified-product analysis.
"""

import os
import sys
from pathlib import Path
from harness.condenser import distill_context, MicroBrief

def read_files(root: Path, patterns: list[str]) -> dict[str, str]:
    """Read files matching patterns."""
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
                except Exception as e:
                    print(f"[WARNING] Failed to read {path}: {e}", file=sys.stderr)
    return files

def main():
    repo_root = Path.cwd()

    print("[INFO] Phase 1: Enhanced Context Intake", file=sys.stderr)

    # CRITICAL: Architecture & strategic context
    intake_patterns = [
        "CLAUDE.md",
        "AGENTS.md",
        "docs/CLAUDE_REFERENCE.md",
        "docs/CURRENT_STATE.md",
        "REMAINING_WORK_TRACKING.md",
        "SHIP_PLAN.md",
        "HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md",

        # Platform entry points & bridges
        "core/src/iron_core.rs",
        "core/src/contacts_bridge.rs",
        "cli/src/main.rs",
        "android/app/src/main/java/com/scmessenger/android/MainActivity.kt",
        "android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt",
        "android/app/src/main/AndroidManifest.xml",

        # Cross-platform abstractions
        "core/src/store/backend.rs",
        "core/src/transport/abstraction.rs",
        "core/src/crypto/mod.rs",
        "core/src/identity/mod.rs",

        # Architecture layers
        "core/src/lib.rs",
        "core/src/routing/mod.rs",
        "core/src/routing/engine.rs",
        "core/src/transport/mod.rs",
        "core/src/transport/manager.rs",
        "core/src/transport/behaviour.rs",
        "core/src/store/mod.rs",
        "core/src/store/contacts.rs",
        "core/src/store/history.rs",
        "core/src/store/outbox.rs",
        "core/src/store/inbox.rs",

        # Platform constraints & rationale
        "docs/rules/ANDROID.md",
        "docs/rules/BUILD_AND_CI.md",
        "docs/rules/RUST_CONVENTIONS.md",
        "docs/rules/SECURITY_PROTOCOL.md",

        # Build & CI
        "Cargo.toml",
        ".github/workflows/ci.yml",
        "docker/Dockerfile",
        "android/app/build.gradle.kts",

        # Key blockers
        "HANDOFF/todo/ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md",
        "HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md",
        "HANDOFF/todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md",
        "HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md",
        "HANDOFF/in_progress/A-05_IOS_RECEIPT_UNIFICATION.md",
        "HANDOFF/in_progress/D1_DESKTOP_BRIDGE_UNIFFI_VERIFICATION.md",
    ]

    files = read_files(repo_root, intake_patterns)
    print(f"[INFO] Read {len(files)} critical files for condensation", file=sys.stderr)

    summary = """
SCMessenger unified product audit:
- Rust core (libp2p, transport, routing, crypto, store)
- Android app (UniFFI bridge, BLE/WiFi)
- CLI (Rust, relay coordination)
- Future: iOS/web via shared core

Find: architectural alignment points, platform divergence (intentional vs accidental),
execution lanes, top blockers to unification, single highest-impact change.
""".strip()

    print("[INFO] Running Condenser.distill_context()...", file=sys.stderr)
    brief = distill_context(
        files=files,
        max_tokens=8000,
        summary=summary,
        focus_symbols=[
            "IronCore", "Swarm", "ContactsStore", "Transport", "Crypto", "StorageBackend",
            "apply", "run", "main", "AndroidManifest", "MainActivity", "MeshForegroundService",
            "Bridge", "uniffi"
        ]
    )

    condensed_prompt = brief.to_prompt_context()

    print(f"[INFO] Brief generated: {brief.estimated_tokens} tokens, {len(brief.file_signatures)} files", file=sys.stderr)
    print("", file=sys.stderr)

    # Output condensed prompt
    print(condensed_prompt, file=sys.stdout)
    print("", file=sys.stdout)
    print("QUESTION FOR OPUS", file=sys.stdout)
    print("Given this architecture context, answer these unified-product questions:", file=sys.stdout)
    print("", file=sys.stdout)
    print("1. What are the 3-4 architectural alignment points that should be shared/synchronized?", file=sys.stdout)
    print("2. Where do we have platform-specific divergence by design (intentional) vs. by accident?", file=sys.stdout)
    print("3. What execution/CI/release lanes should exist for coordinating these platforms?", file=sys.stdout)
    print("4. What is blocking full unification today (build constraints, capability gaps, ownership)?", file=sys.stdout)
    print("5. Propose the single most impactful change to move toward unified execution.", file=sys.stdout)
    print("", file=sys.stdout)
    print("Answer contract:", file=sys.stdout)
    print("- Alignment point: shared abstraction + platforms that participate", file=sys.stdout)
    print("- Divergence: intentional (why) vs accidental (cost/fix)", file=sys.stdout)
    print("- Lanes: name + trigger + success criteria", file=sys.stdout)
    print("- Blockers: blocker + estimated impact (X%)", file=sys.stdout)
    print("- Impactful change: name + why it unifies + rough effort", file=sys.stdout)
    print("", file=sys.stdout)
    print("If you need more context, request by path + section.", file=sys.stdout)

if __name__ == "__main__":
    main()
