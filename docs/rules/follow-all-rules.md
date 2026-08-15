---
trigger: always_on
---

# SCMessenger Core Repository Rules & Guidelines

This document serves as the **Single Source of Truth** for all development operations, agent actions, swarm orchestration, and script hygiene within the SCMessenger repository. All AI agents (Orchestrators, Swarms, and Sub-agents) and human contributors must strictly adhere to these rules.

---

##  1. Banned & Forbidden Behaviors (P0 - Non-Negotiable)

*   **Only Lines of Code (LOC) Estimates**: Never use time-based estimates, durations, or any other estimation format. Use LOC magnitudes instead (e.g., *“~50 LOC change”*).
*   **No Shell Commands for File Editing**: Never use shell/terminal commands (such as `echo`, `cat`, `sed`, `awk`, etc.) to write or edit file contents. Always use native file edit APIs/tools.
*   **No Local/System Temp Paths**: Never use system `/tmp`, `/var/tmp`, or `/dev/shm` directories. All scripts and operations must use repo-local `tmp/` paths (such as `tmp/session_logs/`, `tmp/work_files/`, `tmp/audit_reports/`).
*   **No Committed Build Artifacts**: Never commit log files, process IDs, or device captures to Git. Verify there are no committed artifacts with: `git ls-files "*.log" "*.pid" "*.logcat"`.
*   **No Hardcoded Paths or IDs**: Hardcoding absolute system paths, home directory shortcuts (`~`), or device UDIDs/serials in scripts or configurations is strictly forbidden. Use relative paths derived from script location and dynamic device detection.
*   **No Unchecked Command Substitution**: Never use unchecked command substitutions in bash (e.g., `result=$(command)` where a failure is silent). Always handle failures explicitly (e.g., `result=$(command) || { ... }`).
*   **No Centralized Dependencies**: SCMessenger is a sovereign mesh only. Centralized dependencies or third-party tracking services are forbidden.

---

##  2. Agent State-Machine & Swarm Operations

*Ignore unless the user invokes the /Orchestrate skill.*
When the user invokes the /Orchestrate skill, all agent work must follow the exact lifecycle defined below to preserve swarm consistency and automated accounting. **If the user does not explicitly request to run the orchestrate function, this section is completely irrelevant. You must ignore it, move down the list, and proceed directly with task execution.**

1.  **Claim**: Read and claim a task from the `HANDOFF/todo/` or `HANDOFF/IN_PROGRESS/` backlog.
2.  **Execute**: Implement changes, perform verification, and run compile gates (`cargo check --workspace` or `./gradlew assembleDebug`).
3.  **Move (CRITICAL)**: You are **FORBIDDEN** from considering a task complete until you execute a file move/rename to relocate the task markdown file from `todo/` (or `IN_PROGRESS/`) to `HANDOFF/done/`.
4.  **Checkpoint**: Immediately after moving the task file to `done/`, run `git add -A` and commit locally with `git commit -m "swarm: completed [Task Name]"`. Do not push to remote.

### Swarm Concurrency & Delegation Rules
*   **Concurrency Limits**: Max 2 concurrent CLI-only sub-agents (enforced by the Orchestrator, with up to 3 total active agents in the ecosystem).
*   **Heavy Implementation Delegation**: The Lead Orchestrator (Claude Code) acts as the CI Gatekeeper. It must **DELEGATE** heavy implementations to the Python Swarm and perform only minor surgical/trivial fixes.
*   **Orchestrator Fire-and-Forget Protocol**: Once the Orchestrator has formulated `task.json`, launched workers (filling the 2 slots), and validated pool status, it **MUST** exit the active session immediately. It must never use `sleep` or poll for completion; let the system cron wake it up later.
*   **Productivity Tracking**: Track swarm progress via `HANDOFF/` file state changes, not via agent run logs. Always check `pool status` before launching new tasks.

### Lead Orchestrator Backlog Pipeline (Gemini/Claude)
*   **Sequential Dispatch**: When grinding through a backlog (e.g. `HANDOFF/todo/`), the Lead Orchestrator should use a sequential shell loop that parses the `## Target Files` block from each task markdown and dynamically pipes them into `scripts/delegate_task.py --apply`. Do not run `delegate_task.py --apply` in parallel within the same workspace due to `cargo` compile locks.
*   **Swarm Escalation**: The shell loop must be configured to `break` upon any `delegate_task.py` failure (which happens after 3 failed LLM rounds). When the loop breaks, the Lead Orchestrator must wake up, act as the CI gatekeeper, surgically fix the compilation failure natively, checkpoint the task, and resume the sequential loop.

---

##  3. Architecture & Cryptography Rules

*   **Rust-First Sovereignty**: Cryptographic authority and core system state live solely in the Rust core (`core/src/`). Platform adapters (Kotlin, Swift, WASM) are strictly dumb byte pipes and MUST NOT redefine or duplicate cryptographic behavior.
*   **Strict Cryptography Stack**: Never substitute core algorithms under any circumstances:
    *   **Identity Signing**: Ed25519 (keys must never leave the device).
    *   **Identity Hash**: Blake3(`ed25519_pubkey`), which represents `identity_id`.
    *   **Key Exchange**: Ephemeral X25519 ECDH per-message.
    *   **KDF**: Blake3 `derive_key`.
    *   **Encryption**: XChaCha20-Poly1305 with a 24-byte nonce (authenticated).
    *   **Sender Auth**: AAD binding combined with Ed25519 envelope signature.
*   **Store-and-Forward Mandatory**: Real-time delivery is never assumed. Eventual delivery via store-and-forward is non-negotiable.
*   **Zero Unsafe Rust Blocks**: No `unsafe` blocks are permitted in Rust code without an associated `// SAFETY:` explanatory doc comment and passing Kani formal verification.

---

##  4. Verification, Gatekeeping & Builds

Before finalizing any run or task completion, the following gates must pass:

1.  **Build Verification**: If Rust code was edited, execute `cargo build --workspace`. If Android code was edited, execute `./gradlew assembleDebug`. Record target build status directly in your commit message.
2.  **Doc Sync Check**: Run `scripts/docs_sync_check.sh` (or `.ps1`) and ensure it succeeds.
3.  **Canonical Documentation Updates**: If the run changes behavior, scope, risk posture, scripts, tests, or verification workflow, you **MUST** update canonical docs (such as `CLAUDE.md`, `DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, etc.) in the exact same run.
4.  **Final Summary**: Clearly state which documentation files were updated (or why no doc updates were needed) and report build verification status for edited targets.
5.  **Formal Verification**: All cryptography (`core/src/crypto/`) and transport routing (`core/src/transport/`) logic must pass Property-Based Testing (`proptest`) and Formal Verification (`kani`).
6.  **Build & Compilation Hygiene**: Proactively perform tidy operations like `cargo clean` (for Rust core/CLI) or `./gradlew clean` (for Android) to free up space when the disk starts getting full, or when encountering unexpected build cache issues, stale metadata, UniFFI binding mismatches, or prior to generating a clean final release package. Ensure local system directories stay clean and unburdened by orphaned `.rlib` files or old build artifacts.

---

##  5. Windows-Specific Development & Compilation Rules

*   **Git Bash Pathing**: On Windows, all shell scripts (`scripts/*.sh`) must be invoked using the full path to Git Bash (`"C:\Program Files\Git\bin\bash.exe" <script>`) or executed from inside the Git Bash emulator.
*   **Incremental Compilation Prevention**:
    *   Incremental compilation is disabled via `.cargo/config.toml` to prevent page file errors.
    *   **Runtime rule**: Before executing `cargo check` or `cargo build`, you **MUST** run `export CARGO_INCREMENTAL=0` in your terminal to prevent `.rlib` file-lock corruption.
*   **Swarm Dual-Kill Method**: If a zombie swarm agent must be terminated on Windows, the standard Linux `kill` command is insufficient. You must use the dual-kill sequence:
    ```bash
    kill -9 <PID> 2>/dev/null
    taskkill //F //T //PID <PID> 2>/dev/null
    ```
*   **CLI Agent Stop Rule**: Never invoke `Stop-Process`, `taskkill`, or `kill` directly from the Orchestrator; always stop agents through `.claude/orchestrator_manager.sh pool stop <agent_id>`.

---

##  6. Script Hygiene, Logging & Automation Rules

*   **Required Script Headers**:
    *   **Shell Scripts**: Must start with `#!/usr/bin/env bash`, followed by `set -euo pipefail`, and a comprehensive usage and parameter block comment.
    *   **Python Scripts**: Must start with `#!/usr/bin/env python3`, followed by a standard docstring describing usage, options, and environment variables.
*   **Required Cleanup Handlers**: Shell scripts must register cleanup handlers using `trap cleanup EXIT INT TERM` to clean up temporary files and kill background processes.
*   **Timestamped Logging**: All scripts must log actions with consistent timestamped outputs using ANSI-colored `log_info`, `log_warn`, and `log_error` utilities. Always strip ANSI codes before parsing log outputs.
*   **Dry-Run Mode Support**: All scripts that modify state must support a dry-run mode using `DRY_RUN="${DRY_RUN:-0}"` and a `dry_run` execution wrapper.
*   **Argument Handling**: All scripts must support `--help` / `-h` flags cleanly. Python scripts must use `argparse` for standard argument handling.
*   **Log Capture & Rotation**: Capturing scripts must limit log sizes using `MAX_LOG_SIZE_MB="${MAX_LOG_SIZE_MB:-100}"` checks to prevent disk bloat.
*   **Credential Handling**: Hardcoding credentials or API keys is strictly forbidden. Retrieve them safely from environment variables (e.g., `GCP_KEY="${GCP_KEY:?GCP_KEY not set}"`).
*   **Log Sanitization**: Exported logs must be sanitized of sensitive identifiers like Peer IDs, UUIDs, and IP addresses via standard sed/regex replacements.
*   **Timeout Enforcements**: All network/SSH/logcat capture commands must have explicit timeout boundaries (such as `ConnectTimeout=10` or the `timeout` utility).

---

##  7. Path & Directory Conventions

*   **iOS Path Casing**: Always refer to the iOS directory using an uppercase-I: `iOS/`. Lowercase `ios/` will fail path-governance checks during CI.
*   **XCFramework Location**: Pre-compiled frameworks must reside at `iOS/SCMessengerCore.xcframework/`. Never place them in the root of the workspace.
*   **No Root Python Scripts**: Python scripts are forbidden in the root directory; locate them under `scripts/`.
*   **No Standalone mixed-state Files**: Always consult canonical documentation under `docs/` or `HANDOFF/` for the current execution state rather than relying solely on mixed-state files like `docs/historical/SCMessengerSKILL.md`.

---

##  8. Escalation Policy

AI agents must escalate decisions to the human operator for the following scenarios:
1.  Architectural direction changes that alter the project's core design philosophy.
2.  Trade-offs affecting security vs. performance or privacy vs. convenience.
3.  Technology stack migrations or introducing new system dependencies.
4.  API contract breaking changes that disrupt platform interoperability.
5.  Release timing, tags, and versioning strategy.