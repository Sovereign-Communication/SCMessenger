# TASK: P0-COMPILE-GATE  Get a ground-truth full workspace compile/test result before dispatching any further wiring or feature work

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Why this is P0 and first

A comment surfaced (source: gemini.google.com web UI reading the GitHub repo
statically  **not** a live build, not the agy/Antigravity session that has a
real local toolchain and physical-device access) claiming:

> "a few critical architectural blockages and operational risks are flying
> under the radar. The main compile gate is fundamentally broken. There are
> currently 8 pre-existing compile errors from earlier agent wiring work.
> Before the agents can safely finish the rest of the SCMessenger wiring
> backlog, you need to clear the compile gate."

**This claim has NOT been independently verified, in either direction, as of
this writing.** Here is exactly what is and isn't known:

- The 350-task wiring backlog itself is genuinely closed (verified 2026-07-04:
  `HANDOFF/todo/` has 0 `task_wire_*` files, `HANDOFF/done/` has 351;
  `HANDOFF/WIRING_TASK_INDEX.md` / `WIRING_PATCH_MANIFEST.{json,md}` were
  regenerated and confirm `Total tasks: 0`). If Gemini's comment is about
  *that* backlog needing more wiring, that specific claim is contradicted by
  direct evidence and is very likely stale/wrong.
- The much more plausible source of the "8 errors" comment: a same-day
  sandbox session (no local Rust toolchain, could only run `cargo check`
  before hitting sandbox timeouts on the link step) implemented
  `HANDOFF/DESKTOP_BRIDGE_WIRING_SPEC.md`  wiring 6 previously-unreachable
  modules into the `scmessenger-desktop-bridge` crate (new `types.rs`, `pub
  mod` registration, a `uniffi::setup_scaffolding!()` call, fixes to a few
  real bugs found along the way: an `Arc<Option<...>>` that should have been
  `Arc<Mutex<Option<...>>>`, zbus 4.x API mismatches in `ble.rs`, a lifetime
  escape in `notification.rs`). That subagent reported `cargo check -p
  scmessenger-desktop-bridge` passing, but could **never get `cargo build
  --workspace` or `cargo test` to actually finish**  timeouts in a 2-CPU
  sandbox, not confirmed compile failures. Separately, a CLI transport bug
  task written the same day
  (`HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`)
  mentions in passing: "a related crate, `desktop_bridge`, was independently
  found to be **failing to build entirely** in this same session due to
  missing `zbus`/`web_time` dependencies"  but checking `desktop_bridge/
  Cargo.toml` right now shows both `zbus = "4"` and `web-time = "1.1"` ARE
  present under `[target.'cfg(target_os = "linux")'.dependencies]`, so either
  that observation predates the fix, or there's a different, still-real
  problem being described inexactly.
- A static read of all `desktop_bridge/src/*.rs` files found no obvious
  syntax errors (brace-matching is clean in every file), but this is not a
  substitute for an actual compiler run  proc-macro expansion, trait bound
  errors, and UniFFI codegen issues are invisible to a manual read.

**Bottom line: nobody has actually run a clean `cargo build --workspace` on
current `main` and reported the real, authoritative result.** Every session
so far has hit either a toolchain gap (sandboxed sessions) or hasn't been
asked to (agy was mid-PQC-01, not building the full workspace). This task
exists to close that gap before anyone dispatches more work on the assumption
that the tree either builds or doesn't.

## Acceptance Criteria

- [x] A real `cargo build --workspace` (or at minimum `cargo check
      --workspace`) has been run to completion, on a machine with a working
      Rust toolchain, against current `main` (or whatever branch/commit this
      task is picked up on  record the exact commit hash in this file when
      you run it). -> commit `fdd315f3e73eea053109776f910bafd18dfafaa6`.
- [x] It's NOT clean  see Results below. Gemini's literal "8 compile
      errors" count is not reproduced (real count: 21 in one cluster + 5 in
      an unrelated cluster), recorded as investigated-and-partially-
      corroborated (real failures exist, count/shape doesn't match).
- [x] Exact compiler errors pasted verbatim below  see Results.
- [x] Confirmed: failure is scoped to `scmessenger-desktop-bridge` only for
      `cargo build --workspace` (orphan crate, zero dependents). A second,
      unrelated failure in `core`'s test module blocks `cargo test
      --workspace --no-run` separately.
- [x] All four commands run  see Results. build: FAIL, test --no-run:
      FAIL, fmt --check: PASS, clippy: FAIL.

## Implementation Plan

1. `git pull` / confirm you're on current `main`, record the commit hash.
2. `export CARGO_INCREMENTAL=0` (required per `.claude/rules/build.md` on
   Windows to avoid rlib corruption).
3. `cargo build -p scmessenger-desktop-bridge`  isolate this crate first,
   since it's the most recently and heavily modified area and the most
   likely source of any real failure.
4. `cargo build --workspace`.
5. `cargo test --workspace --no-run`.
6. `cargo fmt --all -- --check`.
7. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`.
8. Paste every command's real output (or the last ~50 lines if it's long,
   but keep any actual error text verbatim, never paraphrased) into this
   file under a new "## Results" heading.
9. Based on the real results:
   - **If clean:** mark this done, move to `HANDOFF/done/`. No further
     wiring-backlog work is needed  that backlog is already closed and
     verified separately. Resume normal priority order from
     `HANDOFF/V1_0_0_UNIFICATION_PLAN.md` / `HANDOFF/RESUME_STATE_2026-07-04.md`.
   - **If broken:** do NOT attempt speculative fixes in this same task.
     Write one new, precisely-scoped `HANDOFF/todo/` task per distinct
     error (or per tightly-related cluster of errors in one file), quoting
     the exact rustc error text, following the existing task file
     conventions (see any file in `HANDOFF/todo/` for the template). If any
     resulting fix touches `core/src/crypto|transport|routing|privacy`, flag
     the mandatory `crypto-security-auditor` review requirement in that
     task file per `.claude/rules/security.md`.

## Files to Touch

None directly by this task  it is a verification/triage task. Follow-up
fix tasks (if needed) will name their own files based on real compiler
output.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build -p scmessenger-desktop-bridge
cargo build --workspace
cargo test --workspace --no-run
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

## Results (2026-07-04, native /scm session)

**Commit under test:** `fdd315f3e73eea053109776f910bafd18dfafaa6` (main)
**Toolchain:** rustc 1.96.1 (31fca3adb 2026-06-26), cargo 1.96.1 (356927216 2026-06-26), Windows

`CARGO_INCREMENTAL=0` set per `.claude/rules/build.md`.

### Gemini's "8 pre-existing compile errors" claim: investigated, NOT reproduced as stated

The tree is **not** clean, but the real picture is different from and larger
than "8 errors": two distinct, unrelated bug clusters exist, one with 21
errors and one with 5. Neither matches "8" as a literal count. The claim is
recorded here as partially-corroborated-but-inaccurately-quantified  real
compile failures do exist, but not in the shape or count described, and not
where the same-day sandbox session's own partial `cargo check -p
scmessenger-desktop-bridge` pass would have suggested (that pass was correct
in isolation; the bug only surfaces because of a cfg-gating issue at the
`pub mod` boundary, invisible to a single-crate `cargo check` run under
default target settings if the developer's own machine happened to be
Linux  see below).

### 1. `cargo build -p scmessenger-desktop-bridge`  FAIL (exit 101)

21x `error[E0433]`, all `desktop_bridge\src\ble.rs`, all "cannot find module
or crate `zbus`/`web_time`". Full output: `tmp/work_files/compile_gate/desktop_bridge_build.log`.

Representative errors (verbatim):
```
error[E0433]: cannot find module or crate `zbus` in this scope
  --> desktop_bridge\src\ble.rs:32:22
   |
32 |     let connection = zbus::Connection::system()
   |                      ^^^^ use of unresolved module or unlinked crate `zbus`
   |
   = help: if you wanted to use a crate named `zbus`, use `cargo add zbus` to add it to your `Cargo.toml`
...
error[E0433]: cannot find module or crate `web_time` in this scope
   --> desktop_bridge\src\ble.rs:188:29
    |
188 |             let last_seen = web_time::SystemTime::now()
    |                             ^^^^^^^^ use of unresolved module or unlinked crate `web_time`
...
error: could not compile `scmessenger-desktop-bridge` (lib) due to 21 previous errors
```

**Root cause (confirmed, not guessed):** `desktop_bridge/Cargo.toml` gates
`zbus`/`web-time` to `[target.'cfg(target_os = "linux")'.dependencies]`
(both ARE present  the CLI-transport-task's "missing dependency" framing
was imprecise, not wrong-in-spirit). `desktop_bridge/src/ble.rs`'s own doc
comment (line 11) says "Only compiled on Linux: `#[cfg(target_os =
"linux")]`" but no such attribute actually exists in the file. `desktop_bridge/src/lib.rs:47`
declares `pub mod ble;` with no `#[cfg(target_os = "linux")]` gate at all.
On any non-Linux host (this Windows machine), the module is compiled
unconditionally while its only two Linux-gated dependencies are absent from
the dependency graph  hence 21 unresolved-crate errors.

**Blast radius:** confirmed scoped to `scmessenger-desktop-bridge` only.
`grep -rl "desktop_bridge" --include=Cargo.toml .` shows nothing depends on
it  not `cli`, not `mobile`, not `core`. It is an orphan workspace member.
It does not block `cargo build -p scmessenger-cli` or `cargo build -p
scmessenger-mobile` (the actual Windows/Android build paths); it only
blocks the aggregate `cargo build --workspace` / `cargo test --workspace
--no-run` commands.

### 2. `cargo build --workspace`  FAIL (exit 101)

Identical 21 errors, identical file, identical root cause as #1  confirmed
via `grep -E "^error" tmp/work_files/compile_gate/workspace_build.log`
(only zbus/web_time E0433s, nothing else). `core`, `cli`, `mobile`, `wasm`
all compile clean under plain `cargo build` (2 pre-existing dead-code
warnings in `core/src/iron_core.rs:143` and `core/src/transport/swarm.rs:1394`,
not errors). **The workspace-wide build failure is caused entirely by
`scmessenger-desktop-bridge`; nothing else is broken under `cargo build`.**

### 3. `cargo test --workspace --no-run`  FAIL (exit 101)

A **second, unrelated** bug cluster  5 errors, all in
`core/src/transport/swarm.rs`'s `#[cfg(test)] mod tests` block (lines
5341-5578+), never reaching `desktop_bridge` (its zbus errors do not even
appear in this log  cargo aborts on `scmessenger-core`'s test-compile
failure first). Full output: `tmp/work_files/compile_gate/test_no_run.log`.

Verbatim:
```
error[E0433]: cannot find type `RegistrationMessage` in this scope
    --> core\src\transport\swarm.rs:5541:55
     |
5541 |             verify_registration_message(&wrong_peer, &RegistrationMessage::Register(request)),
     |                                                       ^^^^^^^^^^^^^^^^^^^ use of undeclared type `RegistrationMessage`
     |
help: consider importing this enum through its public re-export
     |
5342 +     use crate::transport::RegistrationMessage;
     |

error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5548:19
     |
5548 |         let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9101".parse().unwrap();
     |                   ^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5554:19
error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5560:19

error[E0433]: cannot find type `Libp2pPeerId` in this scope
    --> core\src\transport\swarm.rs:5574:23
     |
5574 |             let key = Libp2pPeerId::random();
     |                       ^^^^^^^^^^^^ use of undeclared type `Libp2pPeerId`

error: could not compile `scmessenger-core` (lib test) due to 5 previous errors
```

**Root cause investigation:** the test module's `use super::{...}` block
(swarm.rs:5342-5348) never imports `RegistrationMessage` (compiler's own
suggested fix: `use crate::transport::RegistrationMessage;`  confirmed
exported at `core/src/transport/mod.rs:32`). `Multiaddr` is imported at
swarm.rs:49 (`use libp2p::{..., Multiaddr, PeerId};`) but that's an
outer-module `use`, not visible inside `mod tests` since the test module
only imports specific names via `use super::{...}`, not `use super::*`.
`Libp2pPeerId` is the odd one: **grepped the entire `core/src` tree  this
name is not defined or aliased anywhere else in the crate.** It is used
exactly once, at swarm.rs:5574, and nowhere else. This is not simply a
missing-import bug like the other two; either the test meant `PeerId` (the
real libp2p type, imported at swarm.rs:49) and typo'd/aliased it during a
prior edit, or an intended `use libp2p::PeerId as Libp2pPeerId;` alias was
never added. A follow-up task must resolve which, not guess.

**Blast radius:** `core` only, test-cfg only (`#[cfg(test)]`). Does not
affect `cargo build -p scmessenger-core`, `cli`, `mobile`, or the Android
Gradle/cargo-ndk build  none of those compile core's test module. This is
purely a "compile gate" (`cargo test --workspace --no-run`) blocker, not a
runtime or shipped-build blocker.

### 4. `cargo fmt --all -- --check`  PASS (exit 0)

Clean, no output.

### 5. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`  FAIL (exit 101)

2 errors  the same 2 pre-existing dead-code *warnings* seen in the build
logs, promoted to hard errors by `-D warnings`:
```
error: fields `log_directory` and `security_audit_pipeline` are never read
   --> core\src\iron_core.rs:143:5
    |
111 | pub struct IronCore {
    |            -------- fields in this struct
...
143 |     log_directory: Option<String>,
    |     ^^^^^^^^^^^^^
...
213 |     pub(crate) security_audit_pipeline: Arc<crate::dspy::modules::OptimizerPipeline>,
    |                ^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `-D dead-code` implied by `-D warnings`
    = help: to override `-D warnings` add `#[expect(dead_code)]` or `#[allow(dead_code)]`

error: field `core_handle` is never read
   --> core\src\transport\swarm.rs:1394:5
     |
1392 | pub struct SwarmHandle {
     |            ----------- field in this struct
1393 |     command_tx: mpsc::Sender<SwarmCommand>,
1394 |     core_handle: Option<Weak<crate::IronCore>>,
     |     ^^^^^^^^^^^
     |
     = note: `SwarmHandle` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

error: could not compile `scmessenger-core` (lib) due to 2 previous errors
```
Note: clippy's log terminates at `scmessenger-core`'s errors  it does not
reach `scmessenger-desktop-bridge`'s independent zbus/E0433 issue in this
run (same reason as #3: cargo stops once an early dependency fails under
`-D warnings`). Once core's 2 dead-code fields and desktop_bridge's cfg-gate
are both fixed, clippy must be re-run to get a true clean/dirty verdict for
the whole workspace  this is not yet fully confirmed clean beyond these 2
findings.

### Conclusion

Tree is **not clean**. Two independent, precisely-scoped bugs, neither of
which blocks the actual Windows CLI or Android app build paths:

1. `desktop_bridge/src/lib.rs:47`  `pub mod ble;` needs
   `#[cfg(target_os = "linux")]` to match its Cargo.toml dependency gating.
   Orphan crate, zero dependents, only breaks `--workspace` aggregate
   commands. -> follow-up task written: `P0_DESKTOP_BRIDGE_Missing_Linux_Cfg_Gate_On_ble_Module.md`
2. `core/src/transport/swarm.rs` test module (lines 5342-5578) missing 2
   imports (`RegistrationMessage`, `Multiaddr`) and referencing an
   undefined type `Libp2pPeerId` that exists nowhere else in the crate.
   `#[cfg(test)]`-only, does not affect shipped builds, but blocks the
   mandatory `cargo test --workspace --no-run` compile gate. -> follow-up
   task written: `P0_CORE_swarm_rs_Test_Module_Broken_Imports_Blocking_Compile_Gate.md`

Per operator direction (2026-07-04), both follow-ups are filed as P0
because the compile gate itself is a repo-wide mandatory check, but neither
is a blocker for the concurrently-prioritized Windows/Android parity
effort  see `REMAINING_WORK_TRACKING.md` for the current priority
reordering.

Full raw logs retained at `tmp/work_files/compile_gate/*.log` for the
follow-up tasks' implementers to consult directly rather than re-running
the gate from scratch.

## Do NOT

- Do not write or land any speculative fix for "8 compile errors" without
  first seeing the real compiler output  the count, location, and nature
  of any actual errors is currently unknown; guessing wastes a whole
  implementation cycle if the guess is wrong or the errors don't exist.
- Do not treat this task as evidence the 350-task wiring backlog is
  reopened  that closure was independently verified via file counts in
  `HANDOFF/done/` vs `HANDOFF/todo/` and manifest regeneration, and stands
  regardless of this task's outcome.
- Do not skip straight to fixing `desktop_bridge` based on the prior
  session's partial `cargo check` pass  that was on one crate in isolation
  under `cargo check` (type-checking only, not full codegen/linking); a full
  workspace `cargo build` can still fail even when `cargo check` on one
  member passes, especially with proc-macro-heavy crates like this one using
  UniFFI.
