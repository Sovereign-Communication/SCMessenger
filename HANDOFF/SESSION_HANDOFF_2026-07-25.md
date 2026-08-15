# Session Handoff: 2026-07-25 (overnight, two concurrent sessions)

Status: Active
Last updated: 2026-07-25

Two Claude Code sessions ran against this repo tonight. Both changesets are
committed; neither has passed the full compile gate, for the reason in
"Verification debt" below.

- **Session A (Android/NAT):** NAT hole-punch Priority 1, three-platform parity.
- **Session B (repo/docs):** public-release front-page and repo hygiene pass.

---

## TOP PRIORITY: leaked credential, operator action required

A live `ollama.com` session cookie (an `aid` UUID plus a `__Secure-session`
token) was hardcoded in `OllamaQuotaScraper.ps1:7` and
`OllamaQuotaScraper.sh:39`. The value is deliberately not reproduced here --
retrieve it from the pre-`d6252c9c` history if you need to identify the exact
session to revoke.

Both scripts now read `OLLAMA_SESSION_COOKIE` from the environment instead, but
**scrubbing HEAD does not undo the exposure**: the value is present in **7
commits of history** on a repository that is **public**. `.gitleaks.toml` did
not catch it and does not allowlist those paths.

Required, in order:

1. **Revoke the session** in ollama.com settings (sign out of all sessions).
   Treat it as compromised -- it has been publicly readable for the life of
   those commits.
2. Decide separately whether to rewrite history. That needs an explicit
   decision and a force-push that invalidates existing clones; it was
   deliberately not done unilaterally.
3. Consider adding a gitleaks rule for `__Secure-session` and `aid=` cookie
   patterns so this class of leak fails CI rather than review.

---

## Session A: NAT hole-punch Priority 1 (transport)

Proactive outbound dial to the bootstrap relay on startup, so a NAT mapping
exists before any inbound circuit-relay traffic arrives. Non-fatal on failure --
mesh startup is never blocked by an unreachable relay.

- `core/src/transport/swarm.rs`: new `SwarmCommand::ConnectToBootstrapRelay`,
  `SwarmHandle::connect_to_bootstrap_relay()`, and the command handler, which
  strips the `/p2p/` component and dials each configured bootstrap address until
  one succeeds.
- `core/src/mobile_bridge.rs`: calls it during `MeshService` startup when
  bootstrap addresses are configured.
- `android/.../data/MeshRepository.kt`: `ensureBootstrapRelayConnected()`.
- `iOS/.../Data/MeshRepository.swift`: `ensureBootstrapRelayConnected()`.

### Known issues in this changeset (not blockers, but do not treat as done)

1. **The success log is not proof of connection.** In the
   `ConnectToBootstrapRelay` handler, `swarm.dial()` returning `Ok` means the
   dial was **queued**, not that a connection was established. The handler
   replies `Ok(())` on queue, so `"Connected to bootstrap relay"` is emitted for
   both platforms even when no connection ever forms. This is the same
   queued-vs-connected false-success pattern already known in the generic `Dial`
   handler. To actually verify, the reply must await a
   `SwarmEvent::ConnectionEstablished` for the dialed peer.
2. **Hardcoded relay address on both clients.**
   `DEFAULT_BOOTSTRAP_RELAY` / `defaultBootstrapRelay` =
   `/ip4/100.56.248.69/tcp/9001`, duplicated in Kotlin and Swift. The code
   comments flag this themselves: it should come from `bootstrap.rs`-sourced
   config once that is exposed across the UniFFI boundary. Note this IP is the
   alpha test relay from `HANDOFF/ALPHA_TEST_LUCAS_JOSH_SETUP.md`; if that
   instance is torn down, both clients dial a dead address on every startup.
3. Android and iOS builds were not run against these edits.

---

## Session B: public-release repo pass

### Front page and community health

- `README.md` was **truncated** from 140 to 32 lines by commit `c4600a89`,
  losing everything after the transport section. Restored and rewritten to 200
  lines, with every claim re-verified against code. Corrections made rather than
  restoring the old text: the "transport ladder raced in parallel with sub-500ms
  failover" arrow chain does not exist (selection is a health/score-based
  escalation policy, `Balanced` default); port 9002 is the libp2p WebSocket
  transport, not the WASM/JSON-RPC bridge (that is `/ws` on 9000); the adaptive
  port list is `443, 80, 8080, 9090`; store is sled on native and IndexedDB on
  wasm; `desktop_bridge/` was a missing fifth crate. Added the previously
  undocumented control API on `127.0.0.1:9876`.
- `CODE_OF_CONDUCT.md` was an abridged stub with **no enforcement contact** and
  missing Enforcement Responsibilities, Scope, and the entire four-tier
  Enforcement Guidelines ladder. Replaced with full Contributor Covenant 2.1
  (was claiming 2.0) and a real reporting route.
- **Fabricated contacts removed repo-wide.** `security@`, `conduct@`, and
  `support@scmessenger.org` plus `https://scmessenger.org/docs` were all on an
  **unregistered domain** (DNS: non-existent). The security issue form was
  pushing serious vulnerability reports to a dead address. Sole channel is now
  GitHub private vulnerability reporting. A fabricated `#scmessenger` Matrix room
  was also removed.
- `.github/CODEOWNERS` **deleted** -- it was 32 instances of the literal
  `@YOUR_GITHUB_USERNAME`, which GitHub renders as an invalid file with a public
  syntax-error banner, plus eight invented unassigned maintainer roles.
- `CONTRIBUTING.md` shipped instructions that could not work: `npm run lint`
  with no `wasm/package.json`, `xcodebuild -workspace` with no such workspace,
  CocoaPods with no Podfile, five fictional required CI check names, libp2p 0.53
  vs actual 0.56, `~29K LoC` vs ~72K, and three nonexistent `core/tests/`
  subdirectories. All corrected against the tree.
- Version claims were wrong in three mutually inconsistent ways (SUPPORT said
  v0.2.0, CONTRIBUTING and SECURITY said v0.2.1, PR/issue templates said both);
  all now v0.3.5. `SECURITY.md` claimed support for a `v0.2.0` tag that does not
  exist.
- `CHANGELOG.md`: deleted the `1.0.0-rc2` "Verification" block, which asserted
  six passing gates that no CI run ever backed and that the same file retracted
  50 lines later; that section is now marked `(never released)` since no such
  tag exists. Added Keep-a-Changelog/SemVer boilerplate and compare links.
- `LICENSE`: removed a spliced duplicate sentence; now byte-exact canonical
  Unlicense. `Cargo.toml`: removed fabricated `authors = ["SC Team"]` (and the
  five member-crate `authors.workspace = true` lines that inherited it).
- Stale `Treystu/` org URLs swept from 22 files (archives left as historical
  record).

### Repo layout

Root went from **149 tracked files to 33**. 105 files relocated with `git mv`
(history preserved), 10 junk files deleted, `local.properties` untracked and
gitignored (it held a machine-local Android SDK path).

New/used destinations: `docs/orchestration/` (new), `docs/ops/`,
`docs/historical/{audits,plans,session-reports,iOS}`, `scripts/`, `docker/`.
`ARCHITECTURE.md` was renamed to `docs/ARCHITECTURE_MODULE_MAP.md` to avoid
colliding with the pre-existing and different `docs/ARCHITECTURE.md`.

`DOCUMENTATION.md` was a 384-line internal index listing `docs/historical/*`
files as "Active Canonical Docs" and carrying ~250 lines of WS12.x session log.
Rewritten as a navigable hub; the session log moved verbatim to
`docs/historical/EXECUTION_NOTES_ARCHIVE.md`.

`scm_v1_farm_queue.jsonl` was deliberately **left at the root**:
`scripts/cloud_dispatch.py:6` opens it by relative path and it is live
orchestration state, so moving it would be a runtime break for a cosmetic gain.

### Repo rule that was a false claim

`.claude/rules/build.md` stated "No `.py` in repo root (CI Enforced)" -- there
were 10 root `.py` files and `hygiene.yml` contained no such check. Rather than
weakening the rule, a `Verify root directory layout` step was added to
`hygiene.yml` (root `.py` + tracked build artifacts), which now passes because
the files were moved. That workflow's 162 emoji/box-drawing characters were also
replaced with plain-text tags per the repo no-emoji rule, and the dead
`Validate CODEOWNERS syntax` step was removed.

### A subagent regression that was caught and fixed

The reference-repair subagent corrupted 22 paths across 13 files by naive
substring replacement, rewriting basenames that appeared as **suffixes** of
longer filenames: `TRANSPORT_ARCHITECTURE.md` became
`TRANSPORT_docs/ARCHITECTURE_MODULE_MAP.md`, `task_wire_set_notes.md` became
`task_wire_set_docs/historical/session-reports/notes.md`, and
`2026-06-05_COLD_SWARM_BOOTSTRAP.md` became `..._COLD_SWARM_docs/BOOTSTRAP.md`.
Each was diffed against `HEAD` to confirm it was agent-introduced (all zero at
HEAD) rather than pre-existing, and all 22 were repaired. Superficially similar
paths (`docker/test-scripts/`, `/tmp/scmessenger_docs/`) were confirmed genuine
and left alone.

**Lesson for future path-migration work:** never string-replace a bare basename
across the repo. Anchor on a path separator or a full relative path.

---

## Verification debt (do this first when the machine is free)

Neither changeset has passed the compile gate. An Android Gradle build spawning
`cargo-ndk ndk -t aarch64-linux-android build -p scmessenger-core` was running
for the whole session, and this repo forbids concurrent build-tool invocations
because Gradle spawns cargo-ndk upstream and they contend for the same `target/`
lock. Check with `tasklist | grep -iE "cargo|rustc|gradle|java"` before running:

```
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace --no-run
cd android && ./gradlew assembleDebug -x lint --quiet
```

What *was* verified, text-only: `scripts/docs_sync_check.sh` returns **PASS**
with zero broken links; `hygiene.yml` parses as valid YAML with 10 steps and its
new root-layout gate passes locally; `cargo metadata` parses after the
`authors` removal (this caught a real break -- removing the workspace field
without removing the five inheriting member lines fails the manifest load).

---

## Remaining work, by priority

1. **Revoke the ollama.com session cookie** (see top of this document).
2. **Run the compile gate** for both changesets.
3. **Fix the `ConnectToBootstrapRelay` false success** -- await
   `ConnectionEstablished` instead of replying `Ok` on dial queue. Until then,
   "Connected to bootstrap relay" in Android/iOS logs is not evidence the NAT
   hole-punch worked, and any alpha test result resting on that log line is
   unproven.
4. **De-hardcode the bootstrap relay address** in Kotlin and Swift; source it
   from `bootstrap.rs` config over UniFFI.
5. **CI lanes not proven:** `iOS Build & Test` and `Docker Integration Suite`
   both show `cancelled` (not failed) on `main`. Every other workflow passes.
   The iOS lane being unproven matters given the iOS parity claims.
6. **Nine dead one-off scratch scripts** were moved to `scripts/` rather than
   deleted, because their only referrers are machine-generated inventories
   (`HANDOFF/discovery/REPO_MAP.jsonl`, `log-visualizer/public/data/*.json`):
   `count_braces.py`, `fix_contactmanager.swift`, `fix_swift_generation.py`,
   `fix_swift_strings.py`, `fix_swift_strings_targeted.py`, `list_b1_tasks.py`,
   `run_surgeon.py`, `test.swift`, `test_websocket.py`. Delete them once those
   inventories are regenerated. Note `fix_contactmanager.swift` and `test.swift`
   were modified by commit `13e585ee` ("Unsure if valid - verify needed") on
   2026-07-24, so confirm with whoever made that change first.
7. **Two lowercase `treystu` references remain** in
   `docs/historical/Gemini_Readiness_Audit.md` and
   `HANDOFF/plans/planfromclaudeforhermes.md`, left as historical artifacts.
   Sweep if you want zero.
8. `docs/specs/repository-production-readiness/design.md` is the internal spec
   that was the original source of the fabricated-email pattern. Its four fake
   addresses were replaced, but the document as a whole has not been reviewed
   for other invented content.
9. `REMAINING_WORK_TRACKING.md`'s top section still presents E-00 (ratchet/PQ
   not wired) as `NEW CRITICAL`, but the ticket is `HANDOFF/done/` with
   `Status: DONE 2026-07-17`, build-verified with a unanimous adversarial pass.
   **[ADDRESSED 2026-07-25]** Planning unity pass added a 2026-07-25 header;
   historical 2026-07-17 section marked historical.

## 2026-07-28 UPDATE (orchestrator takeover audit)
Priority #1 (leaked ollama.com session cookie): operator confirms HANDLED -- session revoked; no history rewrite needed (operator decision 2026-07-28). Item CLOSED.
