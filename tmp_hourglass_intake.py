#!/usr/bin/env python3
"""
Phase 1 (Context Intake) of Harness Hourglass for SCMessenger unified-product audit.
Uses Harness Condenser to distill repo context into a focused brief for Opus.
"""

import os
import sys
from pathlib import Path
from harness.condenser import distill_context, MicroBrief
from harness.tokens import estimate_prompt_tokens

def read_files(root: Path, patterns: list[str]) -> dict[str, str]:
    """Read files matching patterns."""
    files = {}
    for pattern in patterns:
        for path in sorted(root.glob(pattern)):
            if path.is_file() and not any(skip in str(path) for skip in ['.git', 'target', 'node_modules', '.gradle']):
                try:
                    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                        if len(content) < 500000:  # Skip huge files
                            rel_path = str(path.relative_to(root))
                            files[rel_path] = content
                except Exception as e:
                    print(f"[WARNING] Failed to read {path}: {e}", file=sys.stderr)
    return files

def main():
    repo_root = Path.cwd()

    # Phase 1: Context Intake
    # Read key architectural and state files
    print("[INFO] Phase 1: Context Intake", file=sys.stderr)

    intake_patterns = [
        # Always-on architecture & governance
        "CLAUDE.md",
        "AGENTS.md",
        "docs/CLAUDE_REFERENCE.md",
        "docs/CURRENT_STATE.md",
        "REMAINING_WORK_TRACKING.md",
        "SHIP_PLAN.md",

        # Platform inventory & binaries
        "core/src/lib.rs",
        "core/src/iron_core.rs",
        "cli/src/main.rs",
        "android/app/build.gradle.kts",
        "android/app/src/main/AndroidManifest.xml",

        # Key module structure (not full content)
        "core/src/crypto/*.rs",
        "core/src/transport/*.rs",
        "core/src/routing/*.rs",
        "core/src/store/*.rs",
        "core/src/identity/*.rs",
        "core/src/contacts_bridge.rs",

        # Build & CI
        "Cargo.toml",
        "docker/Dockerfile",
        ".github/workflows/ci.yml",

        # Handoff status
        "HANDOFF/todo/**/*.md",
        "HANDOFF/in_progress/**/*.md",
    ]

    files = read_files(repo_root, intake_patterns)
    print(f"[INFO] Read {len(files)} files for condensation", file=sys.stderr)

    # Prepare summary for the condenser
    summary = """
SCMessenger is a sovereign peer-to-peer mesh messenger with:
- Rust core (libp2p, cryptography, storage)
- Android app (BLE/WiFi transport)
- CLI (Rust, relay coordination)
- WASM/browser future support
- Hybrid post-quantum crypto (X25519 + ML-KEM-768)

Goal: Unify product execution across all node platforms (Rust, Android, CLI, future iOS/web)
by identifying architectural alignment points, shared abstractions, and execution lanes.
""".strip()

    # Phase 1: Distill into a brief
    print("[INFO] Running Condenser.distill_context()...", file=sys.stderr)
    brief: MicroBrief = distill_context(
        files=files,
        max_tokens=3500,  # Larger for Opus planning phase
        summary=summary,
        focus_symbols=[
            "IronCore", "Swarm", "ContactsStore", "Transport", "Crypto",
            "apply", "run", "main", "AndroidManifest",
        ]
    )

    # Format the condensed prompt
    condensed_prompt = brief.to_prompt_context()

    print("[INFO] Brief generated:", file=sys.stderr)
    print(f"  Estimated tokens: {brief.estimated_tokens}", file=sys.stderr)
    print(f"  Files included: {len(brief.file_signatures)}", file=sys.stderr)
    print("", file=sys.stderr)

    # Output the condensed prompt for Opus
    print("=" * 80, file=sys.stdout)
    print("CONDENSED PROMPT FOR OPUS 5.5", file=sys.stdout)
    print("=" * 80, file=sys.stdout)
    print("", file=sys.stdout)
    print(condensed_prompt, file=sys.stdout)
    print("", file=sys.stdout)
    print("=" * 80, file=sys.stdout)
    print("QUESTION FOR OPUS:", file=sys.stdout)
    print("=" * 80, file=sys.stdout)
    print("""
Given this unified codebase context:

How do we unify this repo to drive a singularly unified (canonical) product across
all SCMessenger node platforms (Rust/core, Android, CLI, future iOS/web) effectively?

Specifically:
1. What are the 3-4 architectural alignment points that should be shared/synchronized?
2. Where do we have platform-specific divergence by design (intentional) vs. by accident?
3. What execution/CI/release lanes (if any) should exist for coordinating these platforms?
4. What is blocking full unification today (build constraints, capability gaps, ownership)?
5. Propose the single most impactful change to move toward unified execution.

Answer contract:
- For each alignment point: the shared abstraction + which platforms participate
- For divergence: intentional (why) vs. accidental (cost/fix)
- For lanes: lane name + what triggers it + success criteria
- For blockers: blocker + estimated impact (blocking X%)
- For the impactful change: name it, why it unifies, rough effort

Keep the answer focused and evidence-based; cite the brief where you base claims.
""".strip(), file=sys.stdout)
    print("", file=sys.stdout)
    print("=" * 80, file=sys.stdout)

if __name__ == "__main__":
    main()
