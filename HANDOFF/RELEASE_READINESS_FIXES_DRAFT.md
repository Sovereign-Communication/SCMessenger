# Release Readiness Fixes — T1-T7, S4, S5 — Status (2026-07-03)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Environment note (read first)

This pass had **no working cargo/rustc toolchain** available (Linux sandbox
has no Rust installed; attempts to install rustup left the shell session
wedged mid-run). Per the task's own escalation rule: nothing below is
claimed as "verified" unless a prior session's test/file evidence
independently confirms it. **No commit was made in this pass** for that
reason, except that one small mechanical source edit was applied directly
to the working tree (see T2 below) because it was a pure grep-confirmed,
zero-ambiguity one-liner matching the doc's exact prescribed replacement —
but it is still UNRUN. A build-capable session must run the verify commands
below before this is mergeable.

## Summary: most items were already implemented before this session

On reading the current source (not stale docs), the majority of T1-T7/S4/S5
turned out to already be landed in `core/src/iron_core.rs`,
`core/src/crypto/session_manager.rs`, `core/src/crypto/backup.rs`,
`core/src/store/contacts.rs`, and `core/src/mobile_bridge.rs`, including
matching test names from the spec (e.g. `test_unprefixed_contacts_migrate_on_open`,
`test_safety_number_returns_empty_string_on_malformed_keys`). This is
consistent with task-tracker state showing another in-progress session
(#7/#10/#12) already working this exact backlog concurrently. Only one gap
was found and fixed in this pass (T2's lookup fallback).

| Item | Status | Evidence |
|---|---|---|
| T1 | **Already implemented** | `bridge_contacts_json: Option<String>` field on `IdentityBackupPayload`; exported via `contacts_manager().list()` in `build_identity_backup_payload` (iron_core.rs ~1298-1310); restored via bridge with validate-then-commit discipline (iron_core.rs ~1453-1462, ~1505-1513) |
| T2 (store sites) | **Already implemented** | All 5 call sites in `cli/src/server.rs` use `contacts_store_manager()`, not `contacts_manager()`. Confirmed zero `contacts_manager()` references remain anywhere in `cli/src/**/*.rs`. Covered by `cli/tests/integration_message_requests.rs` (4 tests, including one literally titled for T2). |
| T2 (lookup fallback) | **Fixed this pass, UNRUN** | `cli/src/server.rs` `AcceptMessageRequest` handler: changed `.find(|m| m.sender_id == request_id).and_then(|m| m.sender_public_key_hex)` to `.filter(|m| m.sender_id == request_id).filter_map(|m| m.sender_public_key_hex).last()`, per doc spec verbatim. No test yet covers the specific case this guards (first matching inbox record has `None` key, a later one has `Some`) — see "Remaining gaps" below. |
| T3 | **Already implemented** | `RatchetSessionManager::deserialize_sessions_strict()` exists in `core/src/crypto/session_manager.rs` (~line 164), aborts on first per-entry conversion failure. `deserialize_sessions` (lenient) kept for `load()`'s best-effort startup path, with a doc comment pointing callers at the strict variant. Wired into both the import probe (iron_core.rs ~1448) and the actual restore (iron_core.rs ~1494) — both map failures to `IronCoreError::CorruptionDetected`. |
| T4 | **Already implemented** | `iron_core.rs` ~1500: `contact_manager.add(contact)?;` (not `let _ = ...`). |
| T5 | **Already implemented** | `iron_core.rs` ~1293: `self.contact_manager.read().list()?;` (not `.unwrap_or_default()`). |
| T7 | **Already implemented** | `core/src/crypto/backup.rs` ~398-417: known-answer test `derive_key_argon2id("some-passphrase", b"0123456789abcdef")` asserted against a fixed expected byte array, comment explicitly labeled "T7". Old `>= 5ms` wall-clock assertion is gone from what I read; format-tag and fallback-chain tests untouched. |
| S4 | **Already implemented** | `core/src/store/contacts.rs`: `ContactManager::new` calls `migrate_unprefixed_contacts()` before returning. Algorithm matches doc spec exactly: scan_prefix(""), skip already-prefixed keys, parse as `Contact`, disambiguate via `contact.peer_id.as_bytes() == key`, write under `contact_key()`, remove old key, `tracing::info!` count, idempotency guard via a `metadata_contacts_migrated` marker key. Test `test_unprefixed_contacts_migrate_on_open` present and matches spec (plus a bonus `test_migration_ignores_non_contact_records_sharing_the_backend`). |
| S5 | **Already implemented (Rust + both mobile UIs)** | `core/src/mobile_bridge.rs::safety_number` returns `.unwrap_or_default()` = `""` on error (not `"000..."`). Test `test_safety_number_returns_empty_string_on_malformed_keys` present, asserts `safety_number("not-hex", "junk") == ""`. Android `VerifySafetyNumberScreen.kt` checks `safetyNumberRaw.isEmpty()` and renders `verify_safety_number_error_invalid_key_data` instead of the QR/number/mark-verified UI. iOS `VerifySafetyNumberSheet.swift` has `safetyNumberIsInvalid` computed from `safetyNumberRaw?.isEmpty == true`, gating the same way. |

## The one change made in this pass

**File:** `core/../cli/src/server.rs`, `ClientIntent::AcceptMessageRequest` handler.

```rust
// Before:
let public_key_hex = core
    .peek_received_messages()
    .into_iter()
    .find(|m| m.sender_id == request_id)
    .and_then(|m| m.sender_public_key_hex);

// After:
let public_key_hex = core
    .peek_received_messages()
    .into_iter()
    .filter(|m| m.sender_id == request_id)
    .filter_map(|m| m.sender_public_key_hex)
    .last();
```

Rationale (per doc T2): the old `.find().and_then()` stops at the *first*
inbox record matching `request_id`, even if that record's
`sender_public_key_hex` is `None` — meaning a legitimate key on a *later*
message from the same peer is never considered, and accept fails with "No
pending message request found" even though a usable key exists. The fix
scans all matching records and takes the last one with a real key present.

## Remaining gaps / follow-ups for a build-capable session

1. **Run the actual verify commands** — none of these were executed in this
   pass:
   - `cargo test -p scmessenger-core session_manager` (T3)
   - `cargo test -p scmessenger-core backup` (T4/T5/T7, twice for T7 per doc)
   - `cargo test -p scmessenger-core contacts` (S4)
   - `cargo test -p scmessenger-core safety_number` (S5)
   - `cargo test -p scmessenger-core` covering T1's integration test (add
     contact via `contacts_manager()`, export, wipe, import, assert bridge
     contact + `verifiedAt` intact — confirm this specific test exists; it
     was not located by name during this read-only pass, only the
     production-path wiring was confirmed)
   - `cargo test -p scmessenger-cli` (T2, including the new lookup-fallback
     edit above — **no existing test exercises the specific
     first-record-has-no-key / later-record-has-key scenario**; recommend
     adding one before considering T2 fully done)
   - `cargo build --workspace`, `cargo fmt --all -- --check`,
     `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`,
     `cargo test --workspace --no-run`
2. **Mandatory adversarial review still outstanding for T3 and T7** — both
   touch `core/src/crypto/` (`session_manager.rs`, `backup.rs`) per
   `.claude/rules/security.md`. This session could not invoke the
   `crypto-security-auditor` subagent. Both items should be treated as
   "implemented, pending mandatory security review," not fully done, even
   after the build gates above pass.
3. Consider adding the missing T2 regression test described above
   (`accept_message_request` where the first inbox record from a sender has
   `sender_public_key_hex: None` and a second, later record has `Some(key)`
   — assert accept succeeds using the later key).
4. This session found evidence of a concurrent/prior session already
   working the same backlog (task-tracker items #7, #10, #12 were
   `in_progress` on the same repo at start of this pass). Whoever lands this
   should reconcile with that session's own notes/commits to avoid
   duplicate or conflicting commits.

## Explicitly not touched

`core/src/transport/`, `core/src/routing/`, `core/src/privacy/` — out of
scope per the task brief, confirmed untouched.

---

## Follow-up session (2026-07-04): still no toolchain — verify commands NOT run

**Environment check performed:** `which cargo rustc`, `cargo --version`,
`rustc --version` all returned nothing / "command not found" in this
session's sandbox. Same limitation as the prior two sessions
(`RELEASE_READINESS_FIXES_DRAFT.md` original pass and
`SMALL_FIXES_STATUS.md`). **None of the nine verify commands listed above
in "Remaining gaps" item 1 were executed** — not attempted, not faked, no
workaround (e.g. downloading rustup) was tried, per the instruction to
report the limitation directly rather than improvise around it.

So the status of every item in the table above is **unchanged from this
draft**: T1, T2 (store sites), T4, T5, S4, S5 remain "already implemented,
read-only-verified, never compiled." T3 and T7 remain "already implemented"
by file inspection but still require the mandatory
`crypto-security-auditor` adversarial review before they can be called done
— **this session did not perform that review either** (it's gated on human/
subagent invocation and, per its own tool profile, doesn't require cargo,
but was out of scope for this pass, which was strictly "run the verify
commands or report why not"). T2's missing regression test (see below) was
NOT added this session for the same reason — the instruction was to write
it only after confirming compiler access, and there is still no way to
confirm the test compiles or passes.

**No new source or test edits were made in this session.**

### Git state check (this session)

- `git status` ran successfully and cleanly (not blocked) — full output
  shows the same large set of modified/untracked files already described in
  `SMALL_FIXES_STATUS.md` (this is CRLF-normalization + prior work sitting
  uncommitted in the working tree, not new churn from this session).
- `.git/index.lock` **exists** (0-byte file). `ps aux` inside this sandbox
  shows no live git process holding it (only this session's own shell/ps/
  grep commands were present). `stat` shows the lock file's mtime is
  recent (created shortly before this check, same session run) — consistent
  with either a very recently stale lock from a crashed/interrupted git
  invocation, or a concurrent process outside this sandbox's visible
  process namespace (same ambiguity `SMALL_FIXES_STATUS.md` flagged for its
  own run — this sandbox cannot see processes from the host/another
  session).
- Because `git status` (a read) succeeded, read-only operations are
  evidently not blocked by this lock. `git commit` was **not attempted** in
  this session (nothing was changed that needed committing, and per
  `.claude/rules/security.md` this session was told not to force past a
  lock that might belong to a live concurrent commit). **Do not assume the
  lock is safe to remove** without first confirming, from a shell that can
  see the full host process list, that no other git process is actually
  running against this working tree.

### Net effect

This session added **no new verification evidence** beyond what
`RELEASE_READINESS_FIXES_DRAFT.md` and `SMALL_FIXES_STATUS.md` already
recorded. The blocking factor is unchanged: **no session so far in this
environment has had a working cargo/rustc toolchain.** Everything below
still needs a session with real compiler access:

1. All nine verify commands listed in "Remaining gaps" item 1 above.
2. The T2 regression test (first inbox record `sender_public_key_hex: None`,
   later record `Some(key)`, assert `AcceptMessageRequest` succeeds using
   the later key) — still not written; write it in the same session that
   can compile and run it, not blind.
3. `crypto-security-auditor` review of T3 (`core/src/crypto/session_manager.rs`)
   and T7 (`core/src/crypto/backup.rs`) before either can be marked done,
   per `.claude/rules/security.md`'s Adversarial Review Protocol — this is
   independent of the toolchain gap and can be done in parallel by a
   session/subagent with Read access even without cargo.
4. Resolve whether `.git/index.lock` is genuinely stale (safe to remove) or
   belongs to a live concurrent session, before any session attempts a
   commit — from a vantage point that can see the real host process list,
   not this sandboxed subset.

**Status summary for anyone reading only this section:**
- T1, T2 (store sites), T4, T5, S2, S3, S4, S5, S6 = implemented per source
  read, **still unverified by compilation** (no session yet has had cargo).
- T2 (lookup fallback) = implemented, **still unverified by compilation**,
  regression test still missing.
- T3, T7 = implemented per source read, **BLOCKED on mandatory
  crypto-security-auditor adversarial review** (independent of the
  toolchain gap), and also still unverified by compilation.
- S7 = file-level CRLF normalization done in a prior session, **still not
  committed** (git index lock).
- Nothing in this environment can execute `cargo`. This is now the third
  consecutive session to hit that wall — worth escalating to the human
  operator as an environment fix (install Rust toolchain in the sandbox, or
  route this class of task to a session/machine that has one) rather than
  re-attempting the same read-only pass again.
