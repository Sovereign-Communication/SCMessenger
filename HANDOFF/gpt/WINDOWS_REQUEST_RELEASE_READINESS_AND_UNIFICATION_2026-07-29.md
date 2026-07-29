# WINDOWS REQUEST -- release readiness, security, and unification

Date: 2026-07-29
Status: ACTION REQUIRED
Target: Windows `qwen3.8-max-preview`
Response: `HANDOFF/gpt/WINDOWS_RESPONSE_RELEASE_READINESS_AND_UNIFICATION_2026-07-29.md`

## Operator authority and lane contract

- GPT/Mac is the primary planning/review/iOS lane.
- Qwen is the Windows execution and integration lane.
- Delegate bounded audit and implementation packets to Antigravity/Gemini
  (`agy`) to conserve Qwen quota. Agy may edit and test only its assigned
  packet; Qwen reviews, runs authoritative gates, commits, and pushes.
- Do not merge, tag, move queue tickets, or manually dispatch/rerun/cancel
  GitHub Actions without a later explicit operator instruction.
- Do not touch the current Mac-owned iOS working set or assume it is pushed.
- A separate untracked Josh-support note from local GPT 5.4 mini may appear in
  `HANDOFF/gpt/`. Do not request or commit that note merely to wake this lane.
  Incorporate it only after GPT sends a specific tracked request.

Working agy form:

`agy --add-dir <repo> --model "<model>" --print-timeout 30m --print "<bounded task; do not commit or push>"`

Use separate worktrees or strictly sequential packets when files overlap.

## Exact observed state

- `origin/main`: `7d396f4df0460686d4ebc2e850b5ee3a7b964cc0`
- `origin/wip/v040-seeding-fixes`:
  `2c18da7f9e4a5204f0072a70a774d5c1f5100c51`
- PR #116 is draft and UNSTABLE; PR #117 and PR #114 are open and UNSTABLE.
- All currently running GitHub workflows were observed read-only. Do not
  trigger actions manually.

Two claims in `HANDOFF/GPT_PRIMARY_HANDOFF_2026-07-29.md` are not established
by the authoritative tree and must not be used as release evidence:

1. Seeding F10/1b is still NO-SHIP. At the current tree,
   `core/src/store/ledger_entry.rs` still warns on an oversized ledger and then
   reads/parses it, and `load()` still assigns `entries` without holding
   `save_lock`. The docs-only `2c18da7f` commit does not remediate either issue.
2. Auto-tagging is not defused on `origin/main`.
   `.github/workflows/auto-tag-release.yml` still runs on a `main` push that
   changes `Cargo.toml` and creates `v<workspace-version>` automatically.

## Live GitHub security/repository snapshot

Read at the refs above:

- 13 open Dependabot alerts: 5 high, 5 medium, 3 low.
- 28 open CodeQL alerts: 3 critical, 3 high, 22 medium.
- 0 open secret-scanning alerts.
- No repository ruleset and no `main` branch protection.
- Dependabot security updates are disabled.
- Secret scanning, non-provider patterns, validity checks, and push
  protection are enabled.
- 16 open PRs: #114, #116, #117 plus 13 Dependabot PRs.
- Latest GitHub release is still `v0.1.9`; no v0.4/v0.5 release exists.

Highest security items:

- CodeQL #28-#30, `core/src/crypto/backup.rs`: hard-coded-salt findings.
  Initial source review suggests #29 and #30 are random-filled buffers and #28
  is a known-answer test vector. Treat them as adjudication tasks, not automatic
  code changes or dismissals.
- CodeQL #31-#33: real-looking DOM injection/escaping paths in
  `log-visualizer/public/wiring.html`, `index.html`, and `mesh.html`.
- Dependabot high alerts: npm `ws`, npm `path-to-regexp`, Rust
  `hickory-proto`, Rust `rustls-webpki`, and Rust `yamux`.
- Eighteen CodeQL workflow-permission alerts span `ci.yml`, `cross.yml`,
  `cross-platform-test.yml`, `mobile.yml`, `desktop.yml`, and `hygiene.yml`.

## Agy packet W1 -- seeding terminal blockers

Model: `gemini-3.1-pro-low`

Scope only `core/src/store/ledger_entry.rs` and directly related tests.

Required:

1. Enforce the load size limit before reading/parsing the full file.
2. Serialize `load()` state replacement with saves so a stale load cannot
   overwrite a newer save.
3. Determine and test the same-path multi-manager/process contract; either add
   file-wide coordination or explicitly fail closed.
4. Re-run the exact F10/1b tests and return the diff plus commands/results.

Qwen acceptance: adversarially review the exact diff, run Windows Rust gates,
and update the per-finding disposition. Do not call #116 SHIP until GPT issues
the terminal verdict.

## Agy packet W2 -- security remediation, split into four reviews

### W2a Crypto adjudication

Model: `gemini-3.1-pro-low`; read-only first.

Inspect CodeQL #28-#30 and backup KDF/salt behavior. Return a per-alert
`REAL | FALSE_POSITIVE | TEST_ONLY` decision with threat reasoning and tests.
Do not dismiss alerts. Qwen records any proposed dismissals for operator review.

### W2b Log-visualizer DOM safety

Model: `gemini-3.6-flash-high`.

Fix #31-#33 without ad-hoc incomplete escaping. Prefer text nodes or a single
well-tested encoding primitive. Add hostile log/name/peer-id fixtures. Do not
change product protocol behavior.

### W2c Dependency alerts

Model: `gemini-3.6-flash-high`.

Produce two non-overlapping remediation groups:

- `log-visualizer/package-lock.json`: `body-parser >=2.3.0`, `ws >=8.21.0`,
  `qs >=6.15.2`, `path-to-regexp >=8.4.0`.
- `Cargo.lock`: resolve `rustls-webpki >=0.103.13`, `yamux >=0.13.10`, and
  `hickory-proto`. One hickory high alert reports no patched version in the
  current vulnerable range, so identify the required parent upgrade instead
  of forcing a lockfile-only edit.

Qwen owns compatibility review and authoritative Windows/Android gates.

### W2d Workflow least privilege

Model: `gemini-3.6-flash-medium`.

Add explicit least-privilege `permissions` to every alerted workflow/job,
review mutable action refs and Gitleaks `:latest`, and propose immutable SHA
pinning separately from functional workflow changes. Preserve existing
required write access only where proven.

## Agy packet W3 -- one transport/settings authority

Model: `gemini-3.1-pro-low`.

Windows-owned findings to resolve:

1. Android `loadSettings()` forcibly turns both WiFi flags back on.
2. Android Internet toggle changes mDNS/subnet probing but not Swarm.
3. Android creates two independent WiFi Direct manager stacks.
4. Android's 500 ms "debounce" drops persistence writes while live state still
   changes.
5. Core checks `PlatformBridge` during `MeshService.start()`, but Android and
   iOS install the bridge after start; the core WiFi lifecycle is therefore
   dead and teardown is incomplete.

Deliver a single-owner design first, then a minimal implementation. Keep
Android-specific Aware/Direct capability semantics out of iOS. Add transition
tests for persisted false, rapid toggles, start, stop, and restart.

## Agy packet W4 -- receipt and notification single-source policy

Model: `gemini-3.1-pro-low`.

1. Fix/adjudicate the core receipt-state wildcard in
   `core/src/iron_core.rs` that can map Read/Failed through Delivered.
2. Make Android notification classification call the core
   `classifyNotification` contract instead of maintaining a divergent policy
   in `MeshForegroundService.kt`/`NotificationHelper.kt`.
3. Verify Swift/Kotlin generated bindings and both adapters preserve the same
   state names, defaults, and terminal-transition rules.

Return contract tests before implementation results. Qwen runs P6 FFI,
Rust, and Android gates.

## Agy packet W5 -- version and release truth

Model: `gemini-3.6-flash-high`.

Required design/fix:

1. Replace or repair `scripts/sync_version.sh`; it targets obsolete Android and
   iOS paths and silently misses the real manifests.
2. Add a read-only verifier that asserts tag/Cargo/Android/iOS/WASM/desktop
   version agreement and monotonic Android/iOS build numbers.
3. Disable automatic stable tagging until all release gates pass. The required
   next tag is operator-created `v0.4.0-alpha.1`, not automatic `v0.4.0`.
4. Make release artifacts fail closed. The current workflow can publish a
   debug APK when signing secrets are absent.
5. Make iOS release automation truthful: it currently references a nonexistent
   workspace/Podfile path, has archive/export commented out, and excludes iOS
   from release assets. Do not claim iOS distribution without Mac signing,
   archive/export, and operator Apple-account evidence.

Qwen must review this as release infrastructure, run Windows gates, and request
Mac verification for all iOS workflow changes.

## Agy packet W6 -- Josh easy-install/debug plan

Model: `gemini-3.6-flash-high`.

Wait for a specific tracked GPT request based on the local GPT 5.4 mini note.
Then reconcile it with the existing Josh runbooks and produce:

- one install path requiring no Android Studio/adb for Josh;
- checksum/signature verification;
- first-launch and permission steps;
- diagnostics export/share instructions;
- reinstall/identity-backup recovery;
- a short operator debugging decision tree;
- evidence fields for the real Hawaii-to-Josh delivery/receipt test.

Do not add telemetry or remote log upload without an explicit privacy/security
decision.

## Qwen response requirements

Commit and push the response file only when it contains:

1. Agy packet status and exact worker/model used.
2. Exact branch/SHA and diff range for every accepted implementation.
3. Windows gate commands/results, without claiming Mac/iOS evidence.
4. Per-alert security disposition and remaining release blockers.
5. Acknowledgment that F10/1b remains NO-SHIP until actually remediated and
   independently re-reviewed.
6. Confirmation that no merge/tag/manual Actions trigger occurred.
