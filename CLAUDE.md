# SCMessenger -- Agent Instructions

Only always-loaded instructions; everything else loads on demand via the table
below. **Hard cap 3.5 KB** -- re-paid uncached by every subagent and `claude -p`
spawn. If an addition breaches it, move something to `docs/rules/` in the same
change. `AGENTS.md` is the model-agnostic contract for every harness; this is
its Claude superset, and AGENTS.md governs cross-agent behaviour.

## Invariants

No hook, CI job, or compiler catches these.

**Builds** (Windows, 16 cores / 11.8 GB RAM -- RAM-bound, not core-bound)

- `cargo -j12` default; `-j6` if RAM contended; `-j4` cold post-clean. Keep
  `CARGO_INCREMENTAL=0`.
- Never run two build tools at once -- sessions share this repo, Gradle spawns
  cargo-ndk.
- Never `cargo clean --target <triple>`: wipes ALL of `target/` (44.7 GB lost).
  Use `scripts/clean_target.sh`.
- rustc crashes (`STATUS_STACK_BUFFER_OVERRUN`, "can't find crate") mean
  resource exhaustion, not corruption. Retry lower `-j`; check `df -h /c`.

**Shared checkout** -- other agents and the operator work here concurrently.

- Touch only what your task requires. Never revert, delete, stash, or commit a
  file you did not create; unrecognised edits are someone else's work and
  discarding them is unrecoverable.
- A clean `git status` is NOT a goal. Leaving unrelated changes alone is right.
- `git commit -a` stages everyone's changes -- stage explicit paths.
- Told you touched something you shouldn't have? STOP and report. Never undo in
  a way that destroys more state.
- Need isolation? `git worktree add <path>`.

**Code**

- No emojis. Use `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`. Hook-enforced.
- Rust: state behind `Arc<RwLock<..>>` (parking_lot); `IronCore` the only entry
  point; no sled outside `store/`; Ed25519 signs, X25519 encrypts,
  XChaCha20-Poly1305 seals; never `unwrap()` in production paths.
- `core/src/{crypto,transport,routing,privacy}/` is merge-blocked until
  adversarial review signs off.

**Compile gate:** `cargo test --workspace --no-run` before any task is complete.

## Routing table

| About to... | Read/run first | Else |
|---|---|---|
| Clean or delete build artifacts | `scripts/clean_target.sh --dry-run` | 44.7 GB wiped |
| Destructive git, `rm -rf`, force-push | `docs/rules/SECURITY_PROTOCOL.md` | operator approval; hook-blocked |
| Dispatch a worker | `docs/rules/DELEGATION.md` | timeouts, truncation, quota burn |
| Dispatch to Freebuff | `docs/rules/FREEBUFF.md`, `HANDOFF/freebuff/` | wasted operator paste cycles |
| Run a build or gate | deconflict, `df -h /c`, `build-verify` | concurrent builds corrupt `target/` |
| Edit crypto/transport/routing/privacy | `docs/rules/SECURITY_PROTOCOL.md` + `crypto-security-auditor` | merge blocked |
| Edit other Rust | `docs/rules/RUST_CONVENTIONS.md` | boundary violation caught late |
| Android work | `docs/rules/ANDROID.md`, `android-qa` | hardcoded strings, missing FGS channel |
| Change documentation | `docs-sync` skill | sync check fails at finalize |
| Orchestration run | `docs/ORCHESTRATION.md`, **in full** | fragments -> wrong lane |
| Finalize or commit | `finalize-checklist` skill | secrets, unverified build |
| Anything else | `docs/DOCUMENT_STATUS_INDEX.md` | 30 KB -- not a reflex |

Tier 0 is this file; Tier 1 is `docs/rules/` and skills. `.claude/rules/*.md`
are stubs so old cross-references resolve -- never re-inline detail there, that
directory auto-loads into every spawn. Before adding a rule here: if a hook, CI
job, or compiler already catches it, it belongs in `docs/rules/`.
