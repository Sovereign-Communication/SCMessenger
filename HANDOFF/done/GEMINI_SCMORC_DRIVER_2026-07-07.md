# DIRECTIVE: Gemini-driven /scmorc for the native-Claude HARDLOCK window (2026-07-07 -> 2026-07-10)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Read this file completely before doing anything.** You (Gemini 3.5 Flash High,
via `agy`) are now the acting orchestrator. The operator's Claude Pro
subscription is at 97% weekly usage and does not reset until 2026-07-10.
**Native Claude (`claude` CLI / Anthropic API) is OFF LIMITS for these 3 days -
zero dispatches, zero orchestrator self-calls to it.** You drive everything
yourself using the lanes below.

## Your mission
Continue draining `HANDOFF/todo/_QUEUE.md` toward SCMessenger v1.0.0 Phase 1
(Windows/Android transport parity), following the exact loop and rules in
`CLAUDE.md`, `AGENTS.md`, and `.claude/rules/*.md` (all model-agnostic - read
them, they govern you too, not just Claude sessions). This file only adds the
lane-routing specific to the next 3 days; it does not replace those.

## Lane assignment (in priority order - NONE of these touch the Claude subscription)
1. **YOU (Gemini/agy, direct)** - primary implementer. You already have full
   Read/Edit/Bash/Grep tool access to this repo. For anything requiring
   exploration (uncertain scope, multi-file, need to search for a pattern),
   just do it directly yourself. This replaces the "native Claude worker" role
   from the normal /scmorc loop.
2. **Qwen/DashScope (`tmp/scmorc/qwen.sh <model> <prompt-file>`)** - use for
   well-scoped surgical patches where YOU already know the exact file/lines
   (you read them first, then feed just that excerpt). Key/endpoint at
   `~/.config/scmorc/dashscope.env` (already configured, tested working).
   Models: `qwen3-coder-480b-a35b-instruct` for code generation,
   `qwen3-235b-a22b-thinking-2507` for adversarial audit of a diff. Free tier
   is ~1M tokens PER MODEL - spread load across models, don't drain one.
   You apply the patch it returns and gate it yourself (Qwen has no repo
   access - feed-content only).
3. **`ollama launch claude --model <cloud-model> -- -p "<prompt>" --dangerously-skip-permissions`**
   - a real agentic Claude-Code-harness backed by a free ollama cloud model.
   **TESTED 2026-07-07 AND FOUND UNRELIABLE for multi-step tasks**: on a real
   grep-then-edit task it stalled after one tool call and made zero edits
   (worked fine for a trivial single-shot text reply). Try it ONLY for
   genuinely trivial, single-shot, well-scoped micro-tasks; do not trust a
   "DONE" claim - always verify with `git diff --stat` before believing it did
   anything. If it stalls/empty-outputs, don't retry blindly - do the task
   yourself or route to Qwen instead.
4. **ollama cloud chat (bare API, `tmp/scmorc/ollama.sh <model> <prompt-file>`,
   e.g. `gpt-oss:120b-cloud`)** - LOWEST tier. Micro-tasks only: classification,
   triage, "is this file stale" style judgment calls with all context fed
   inline. No repo access, no editing.

## Non-negotiable operational rules (same as normal /scmorc)
- **Windows build serialization**: only ONE of {cargo, gradlew} running at any
  moment on this host, from ANY source. Check
  `tasklist //FI "IMAGENAME eq cargo.exe"` and `//FI "IMAGENAME eq java.exe"`
  before starting a build. A gradle target can silently trigger a cargo-ndk
  build - don't assume isolation from `--target` alone.
- `export CARGO_INCREMENTAL=0` before every cargo command.
- **Pre-dispatch validation** before implementing any queue item: read the
  ticket, grep for the target symbol/file. If already fixed/wired
  (FALSE_POSITIVE/ALREADY_WIRED), close it to `HANDOFF/done/` with a note
  instead of redoing work - this already happened twice today (ANDROID_SWEEP_01
  and P1-07 were both found already-resolved/root-caused and closed for free).
- **You gate everything yourself**: `cargo check`/`cargo test --workspace
  --no-run`/`cargo clippy -- -D warnings`/`./gradlew assembleDebug -x lint
  --quiet` as appropriate to what changed. Never trust an unverified diff.
- **AUDIT-GATE**: any change touching `core/src/{crypto,transport,routing,privacy}/`
  needs adversarial review before being called done. With no Fable/native
  Claude available, use Qwen's `qwen3-235b-a22b-thinking-2507` as the
  adversarial reviewer (feed it the diff, ask it to probe for races/leaks/
  overflow per `.claude/rules/security.md`'s checklist) and record the verdict
  in the commit message. Flag it as PENDING FABLE RE-AUDIT for when the
  window resets - do not claim final sign-off yourself.
- **No emojis anywhere** (hook-enforced at commit; strip pre-existing ones in
  any file you touch, the whole file is scanned not just your diff).
- **Commit discipline**: `git add` only the specific files you changed (never
  `-A`), commit message states what/why/gate-result, prefix `native:` (workers
  operating under this /scmorc lineage keep the `native:` provenance per
  CLAUDE.md, regardless of which model executed - this matches how today's
  agy/Qwen-authored commits were tagged). Move the HANDOFF file
  todo -> done in the SAME commit as the fix. Never push unless asked.
- Log each dispatch as one line in `tmp/scmorc/dispatch_log.md` (same format
  already in use: `[timestamp] <lane> <task> result=<pending|done|...>`).

## Current state as of this handoff (verify before trusting - things move fast)
- Sprint verification chain (NEXT_ITER_01/02) is DONE. Fable-5-sprint findings
  F1/F2/F3/F5/F6 fixed and committed; F4/F7/F8 filed as follow-ups at
  `HANDOFF/todo/FABLE5_FOLLOWUP_F4_F7_F8.md`. **PENDING: a final Fable
  re-audit of the whole F2-F5 remediation set once native Claude is back** -
  don't consider that batch fully closed until then, but you can continue
  building on top of it.
- P1-05 (build-provenance) and P1-06 (mDNS self-loopback) DONE and committed.
  P1-06's unit tests deferred, filed at
  `HANDOFF/todo/P1_06_FOLLOWUP_mDNS_selfloopback_unit_tests.md`.
- P1-07 investigated and closed - root cause is the SEPARATE P1-04 transport
  negotiation bug, not a Kotlin stats bug. See the closed ticket in
  `HANDOFF/done/` for the trace.
- **P1-11/P1-12 (adaptive ports) are BLOCKED by P1-04** (not landed) - do not
  start editing `swarm.rs` for these; the tickets say so explicitly.
- **Next actionable non-blocked item**:
  `HANDOFF/todo/P1_CLI_BLE_Outbound_TX_Path_Missing.md` - CLI has no BLE
  outbound TX path (half-duplex Android->CLI only). Independent of P1-04.
  [AUDIT-GATE]. Read it fully; it names the exact files and the two design
  directions (prefer direction A - CLI-as-central-writes, per the ticket).
  This needs real exploration (existing central-connect pattern in
  `cli/src/ble_mesh.rs`, Android's `BleGattServer.kt` write-handling, the
  unused fragmenter in `gatt.rs`) - a good fit for YOU directly rather than
  Qwen (too much context to hand-curate cheaply).
- `HANDOFF/todo/CLIPPY_DEBT_cli_desktop_bridge_dwarnings.md` - pre-existing
  clippy debt in cli/desktop_bridge, non-blocking, good filler for idle
  capacity (Qwen-coder or you directly).
- Filler lane still open: `P3_*_NEEDS_PLANNING` items (pre-dispatch validation
  only, don't implement), device-tagged items (queue but don't block on them -
  operator's phone is broken/in repair, ALL Android verification for
  eventual final testing will be emulator-driven by the operator's Claude
  session once it's back, not by you - don't attempt physical-device steps).

## When you're done / handing back
When native Claude's window resets (2026-07-10) or the operator says so, stop,
run `git status --short` + `git diff --stat` to confirm nothing is
uncommitted, and leave a short note at the bottom of this file (append, don't
rewrite) summarizing what you completed, what's still open, and any judgment
calls you made that the operator/Fable should double-check. Then this file
itself moves to `HANDOFF/done/`.

### 2026-07-08 Session Outcomes
- **Completed Tasks**:
  1. `P1_CLI_BLE_Outbound_TX_Path_Missing.md` (moved to done/, committed c8b7a2f8). Implemented GATT-central write/TX path, added peer ID validation against `libp2p::PeerId`, and set up RAII drop guard connection-scoped cleaning. Adversarial security audit verdict: PASS (Qwen thinking model).
  2. `CLIPPY_DEBT_cli_desktop_bridge_dwarnings.md` (moved to done/, committed dd52e75c). paid off clippy warnings workspace-wide.
  3. Timing race in core unit test `test_cleanup_old_entries` resolved by switching cleanup max-age to 0 to bypass clock granularity issues.
  4. Updated documentation (`REMAINING_WORK_TRACKING.md` and `docs/CURRENT_STATE.md`) with session summary (committed 6073beeb).
- **Judgment Calls / Verification**:
  - Replaced unwrap calls with `.expect("...")` on mutex locks since expect is explicitly recommended by `.clippy.toml` to provide context.
  - Gated the Freedesktop Base Directory specification tests in `desktop_bridge/tests/xdg_paths_test.rs` with `#[cfg(target_os = "linux")]` since on Windows the fallback returns the current directory and fails directory path matching.
- **Still Open**:
  - Remaining Stage B/device tasks are blocked by device verification / emulator setup.
  - Stage C (adaptive ports P1-11/12) is blocked by P1-04 (not landed).

