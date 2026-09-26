# v0.4.0 Completion Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-08-01
Supersedes: the critical-path list in `HANDOFF/todo/_QUEUE.md` (2026-07-28 header)
Authority retained: `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` gate ladder
(040-G0 .. 040-S6) is unchanged and still binding.

Operator goal: a debug APK on a physical Pixel 6a, good enough for a real
two-person mesh test with Josh (Hawaii to Pennsylvania).

Planning verified by Fusion Lite Tier-B panel (deepseek-v4-pro,
kimi-k2-thinking, qwen3-235b-thinking; judge deepseek-v4-pro) at $0.0203.

---

## 1. State reconciliation (git-verified 2026-08-01)

Verified by direct inspection of code and CI, not by reading status docs.
The following `_QUEUE.md` critical-path entries are STALE and are closed here:

| _QUEUE item | Claimed | Actual |
|---|---|---|
| 1a queued-vs-connected false-success | pending | DONE. `core/src/transport/swarm.rs:549-562` `PendingDialEntry`; `:2093` reply held until `ConnectionEstablished`; `:2859-2880` timeout expiry |
| 1d version bump 0.3.5 -> 0.4.0 | pending | DONE. `Cargo.toml:9` = 0.4.0; `android/build.gradle:24-25` versionCode 14, versionName '0.4.0' |

Also verified done: ledger choke-point refactor (22b921ca);
`core/src/transport/dial_policy.rs` present with per-peer exponential backoff.

Two release hazards recorded in the runbooks are already resolved:
- `.github/workflows/release.yml:104-105` now installs `cargo-ndk` (487589d8).
- `.github/workflows/auto-tag-release.yml:18-20` is `workflow_dispatch:`-only
  with `push:` commented out. This neutralises verdict risk #5 (an accidental
  stable `v0.4.0` tag on the version-bump merge).

Stale claim corrected: `V040_S5_JOSH_WAN_RUNBOOK.md:9` says the GPT verdict
file is absent from the repo. It is present at
`HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` (431 lines, merged via PR #115).

No `v0.4.0` tag exists. Latest tag is `v0.3.5`.

---

## 2. The one hard blocker: PR #128

PR #128 (`fix/seeding-security-remediation-v040`) is the reborn PR #116 and
carries the entire ledger-seeding remediation chain. The operator decision of
2026-07-28 requires all seeding findings closed before tagging, so #128 is the
release gate.

#128 is fully rebased on main (0 commits behind; merge-base = main tip
487589d8) yet every CI job fails: Android x3, iOS, Docs, FFI Surface Contract,
Lint, Rust Linting, Test on ubuntu/macos/windows.

Root cause, from the job log rather than inference -- it does not compile:

```
error[E0433]: cannot find `TransportType` in `mobile_bridge`   (x2)
error[E0609]: no field `escalation_engine` on type `&Arc<IronCore>`  (x2)
```

The branch's merge commit 2cacd5ac ("merge: sync main") mis-resolved conflicts
and resurrected pre-fix code at `core/src/mobile_bridge.rs:982-985` and
`:1084-1087`. Those blocks reference `core_ref.escalation_engine` (no such
field) and `crate::mobile_bridge::TransportType` (wrong path; the type is at
`crate::transport::abstraction::TransportType`). Main carries the corrected
form at `mobile_bridge.rs:3374-3421`. The follow-up commit b268d1df
("remove leftover merge conflict markers from ledger_entry.rs") corroborates a
messy merge.

Whole-file delta between main and the branch is +20/-2. This is a surgical fix.

---

## 3. Second blocker: PR #127

PR #127 (`feature/v040-v050-completion-sprint`) is MERGEABLE/CLEAN: 25 files,
+1903/-12933, 10 commits not on main. It carries the compile fixes for the same
`mobile_bridge` defects plus workspace-manifest hygiene.

Hazard: the branch adds `launch_claude.ps1` at the repo root, which violates the
root-layout hygiene rule and is already flagged as an operator hygiene item in
`_QUEUE.md`. It will fail Repository Hygiene. Remove it or move it to `scripts/`.

---

## 4. CI and PR-board state

Main: CI, Lint, Cross, Docker Publish green. Repository Hygiene FAILS on
trailing whitespace -- the offenders are the `audit_system/*.py` files added on
this branch, plus a blank line at EOF in `audit_system/AUDIT_HANDOFF.md`.
Docker Integration Suite fails (pre-existing, not release-gating).

`mobile.yml` has not run green on main recently; the newest run (dependabot)
failed. It needs a confirming run, because it uploads the `android-debug-apk`
artifact from `android/app/build/outputs/apk/debug/` -- the low-friction way to
get an APK onto the Pixel 6a without a local RAM-bound Windows build.

Board hygiene: commits for PRs #120/#121/#123/#124 are already on main
(cb3a2ddc, 977e2ec0, 9105cc8b, d87d2132) while the PRs remain OPEN. #116 and
#114 are CLOSED unmerged. Reconcile the board so "all gates green" is meaningful.

---

## 5. Android readiness

The app is feature-complete enough for the test: ~28 Compose screens, 8
ViewModels, `MeshRepository` FFI bridge, foreground service with notification
channels, BLE + WiFi Aware/Direct + mDNS transports, 500+ strings in
`strings.xml`, complete manifest permissions. Prebuilt `.so` exist for
arm64-v8a / armeabi-v7a / x86_64 (2026-07-28).

The five previously-identified device blockers are all in `HANDOFF/done/`, not
`todo/`: P0 ANR BatteryReceiver synchronous FFI; P1 CLI transport negotiation
failure on Android inbound dial; P1 mDNS self-loopback; P1 LAN discovery not
feeding `peersDiscovered`; P2 BLE MAC rotation.

Caveat that drives Stage 2: these were closed on paper, their closure evidence
is thin, and none were re-verified on physical hardware because the operator's
phone was in repair. The device is back. The correct action is device
verification, not re-implementation.

Known compliance debt, non-blocking: ~10 hardcoded UI strings in
`ui/dialogs/ApkShareDialog.kt` (79, 92, 106, 115, 152, 157, 167, 175, 188) and
`ui/screens/SettingsScreen.kt:1215`.

---

## 6. Seeding findings status

From `HANDOFF/review/V040_FINDING_DISPOSITIONS.md`:

- F2 is the only finding not fixed. The signed-invite import path is dead:
  `import_seed_entries` has no non-test caller wiring `verify_with_policy`
  (grep-confirmed 2026-08-01). Disposition is DOCUMENTED + CI grep canary.
  It needs an operator-signed release decision before tag.
- F6, F7, F12, F13, NEW-5, NEW-6 are CLOSED but each is explicitly "pending
  terminal verdict". They must be re-verified in the 040-S2 adversarial
  verdict, which has NOT been produced. **This is the real remaining security
  gate, not F2.**
- Two residuals deferred post-alpha: sustained-burst anchor aging;
  cross-instance mobile `LedgerManager` on a shared path.

Review-panel correction (recorded deliberately): the Fusion panel asserted that
F2 blocks Josh from importing seeds and proposed relaxing the Android
`public_key` check from exactly-64 to 64-or-more hex chars. **Reject this.** F2
concerns the signed path; the live primary seed path is the unsigned
contact-JSON import carrying a `listeners` array, which works. The exact-64-hex
constraint is that working path's validation. Loosening it would weaken input
validation to fix a problem that does not exist.

---

## 7. Critical path

**Stage 0 -- unblock CI (trivial).**
0.1 Strip trailing whitespace from `audit_system/*.py`; remove the blank line at
EOF of `audit_system/AUDIT_HANDOFF.md`. Fixes Repository Hygiene.

**Stage 1 -- release gate.**
1.1 Land #127 first, with `launch_claude.ps1` removed or moved to `scripts/`.
    It contains the correct `mobile_bridge` blocks.
1.2 Rebase #128 onto the new main. If the compile break survives the rebase,
    apply the surgical fix: restore main's form of `mobile_bridge.rs:982-985`
    and `:1084-1087`.
1.3 Drive #128 to full CI green. Reconcile the already-landed open PRs
    (#120/#121/#123/#124) and formally close #116/#114.
1.4 Produce the 040-S2 adversarial verdict over the final parent..tip range,
    naming every finding F2/F3/F6/F7/F10/F12/F13/F16/NEW-5/NEW-6. Mandatory:
    this touches `core/src/transport/` and `core/src/store/`, so it goes to the
    `crypto-security-auditor` subagent. Fusion Lite is explicitly not a
    substitute for this gate.
1.5 Obtain the operator-signed decision on F2.

**Stage 2 -- proof on real hardware (newly possible).**
2.1 Pull the `android-debug-apk` artifact from a green `mobile.yml` run.
    ABI note: the Pixel 6a is arm64, so the APK must contain
    `lib/arm64-v8a/libscmessenger_core.so`, not the emulator's `lib/x86_64/`.
2.2 Install, grant runtime permissions, cold-launch.
2.3 Re-verify the five `done/` device tickets on real hardware.
2.4 Pixel 6a <-> Windows CLI LAN delivery proof, both directions, with receipts.
    Seed via the unsigned contact-JSON import with a `listeners` array;
    `public_key` must be exactly 64 hex chars.
    Evidence standard: `ConnectionEstablished` on both sides plus an authentic
    decoded receipt. Explicitly disqualified as sole evidence -- "Dialed ... via
    SwarmBridge", `/api/send success:true`, and any dial-queue log line.
    Write the verdict to `HANDOFF/review/V040_S4_DELIVERY_VERDICT.md`.

**Stage 3 -- Josh WAN test.**
3.1 BLOCKED on operator: relay endpoint 100.56.248.69 is a Tailscale CGNAT
    address and the Windows host has no Tailscale, so it is unreachable from
    here. Supply a public endpoint (public IP or DDNS + port forwards per H-04)
    or install Tailscale on the Windows host.
3.2 Run S5 including the restart-persistence arm (re-dial from ledger with no
    re-import) and the disconnect/reconnect queued-delivery arm. Test both the
    cellular and WiFi legs. Write
    `HANDOFF/review/V040_S5_WAN_PROOF_VERDICT.md`.
3.3 The home port-forward arm (TCP 443/80, UDP 443, DDNS) is optional for alpha
    under the signed waiver that defers it to P1-18.

**Stage 4 -- release (040-S6).**
4.1 FFI surface snapshot check (`scripts/ffi_surface.sh`; bindings and
    `scripts/ffi-snapshots/` are present).
4.2 `./scripts/docs_sync_check.sh`; promote CHANGELOG `[Unreleased]` to
    `[0.4.0]` and state the exclusions (no iOS, no farm-sim claims).
4.3 Operator cuts the tag manually:
    `git tag -a v0.4.0-alpha.1 && git push origin v0.4.0-alpha.1`.
    NEVER run `scripts/sync_version.sh` -- it targets wrong paths and corrupts
    versionCode.
4.4 Signing: `SCMESSENGER_KEYSTORE_*` secrets are absent, so the alpha ships a
    debug-signed APK. Fine for sideload; needs an explicit operator decision.

---

## 8. Scope exclusions (operator-confirmed 2026-07-28, reaffirmed)

Out of v0.4.0: iOS (A-05/U6/D-03), P1-14/P1-18 hostile-network, PQC-09,
B1 DNS hardening, farm-sim chain, KMP desktop.

Explicit warning: a backlog triage pass flagged the farm-sim and
contact-provisioning tickets as "v0.4.0 blockers". They are farm-simulation
infrastructure and are NOT on the path to a two-real-device test. The Fusion
panel agreed unanimously. Do not let them expand scope.

---

## 9. Audit system (parallel, not release-gating)

The local qwen2.5-coder dual-pass audit is running: 1834 of 5391 functions
(34%), 4539 findings, 0 errors, started 2026-07-30 13:59, roughly 37
functions/hour, so about four more days to complete.

Severity: 404 high, 564 medium, 1494 low, 2078 info. Top categories:
magic_number 1044, docs 771, error 509, unsafe 268. Hottest files:
`mobile_bridge.rs` 464, `iron_core.rs` 386, `relay/invite.rs` 192.

Triage notes: the largest real high-severity cluster is `unwrap()`/panic (66),
concentrated in `crypto/` (backup.rs, padding.rs, encrypt.rs, compress.rs).
124 of 404 high findings sit in crypto/transport/routing/privacy and would
require adversarial review before any change. A spot-check of the 10
highest-risk findings against real source confirmed 8, though several are in
test setup rather than production paths. False-positive rate varies sharply by
category -- roughly 10% for unwrap findings, roughly 60% for
platform_specific_leak -- so triage must be category-weighted and nothing should
be bulk-applied.

Disposition: v0.4.1+ quality backlog. Do not gate v0.4.0 on it.

---

## 10. Delegation model

| Work | Lane |
|---|---|
| Planning and verification | Fusion Lite Tier-B panel, cents per run |
| Doc ingestion, backlog triage, mechanical greps, compliance fixes, whitespace hygiene | Haiku 4.5 subagents |
| adb/logcat poking, single build commands, local mechanical work | agy / Gemini free lane, pinned `--model` and `--add-dir` |
| Hardest well-scoped questions | GPT Sol 5.6 Ultra via `HANDOFF/gpt/` handoff files |
| Crypto/transport/routing/privacy review (040-S2) | `crypto-security-auditor` subagent -- mandatory, not substitutable |
| Verdicts, gate decisions, commits, merges | Native Claude |

--- END FILE ---
