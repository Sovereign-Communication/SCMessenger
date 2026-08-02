# GPT TAKEOVER -- Windows lane wind-down 2026-08-01

Status: ACTIVE HANDOFF -- Windows Claude at API limit, GPT to continue
Author: Windows Claude (orchestrator)
Scope: everything discovered and changed this session, and what to do next

DRAFT NOTE: sections 6 and 7 are pending two in-flight agents; they will be
filled before this file is final. Everything else is verified and stable.

---

## 0. Read this first -- the systemic finding

Six independent defects this session share ONE pathology: **code reports
success for work it never performed.** This is the single most important thing
for you to carry forward. When triaging anything in this repo, assume a
"success" log line is unverified until you find the syscall behind it.

| # | Fake success | Location | State |
|---|---|---|---|
| 1 | Queued dial reported as connected | swarm.rs (PendingDialEntry) | FIXED earlier |
| 2 | NAT hole-punch sets `status = Success`, opens no socket | nat.rs:442-494 | OPEN -- delete it |
| 3 | CLI prints "executed command successfully" for unimplemented commands | cli/src/main.rs:22-32 | FIX IN FLIGHT |
| 4 | Unconditional `Ledger Inject: [OK] ... linked` | cli/src/main.rs | OPEN |
| 5 | Multiport binder accepts ANY port, QR still advertises 9001 | swarm.rs:2394-2424, mobile_bridge.rs:1106 | FIX IN FLIGHT |
| 6 | PQ ratchet `Some`/`None` mismatch derives divergent keys silently | ratchet.rs:841,:865 | HANDED TO YOU |

Recommend a lint/review rule before 1.0.0: no success path may be reported
without evidence from the layer that actually performs the work.

---

## 1. THE CRITICAL REGRESSION -- CLI was gutted on main

Commit `55564b4b` deleted **3,995 lines** from `cli/src/main.rs` (4,043 -> 170).
It removed `cmd_start`, `cmd_relay`, `cmd_send_offline`, `cmd_status` and the
`--http-bind` global flag. Everything except `ShareApk` now falls to a
catch-all printing `[INFO] SCMessenger CLI executed command successfully.`

Consequences: the Windows CLI cannot run a node, the AWS always-on node cannot
run, and no device-delivery proof is possible. This is ON MAIN, inside the
0.4.0 baseline.

IMPORTANT: the S4/S5 runbooks documenting `scm --http-bind ... start` are NOT
stale. They were correct; the CODE regressed. `cli.http_bind` was real and used
at old `main.rs:673`.

Recovery source is unambiguous and verified byte-identical from two refs
(hash `5955f245752c24216d3b446337d275f0ce402789`):
    git show 55564b4b^:cli/src/main.rs
    git show origin/gpt/v050-ios-release-ready:cli/src/main.rs

Only 6 refs are gutted -- all downstream of 55564b4b: `origin/main`,
`audit_system`, `checkout-7`, `feature/v040-v050-completion-sprint`,
`fix/seeding-security-remediation-v040`. **Every `gpt/*` branch retains the
full 4,043-line file**, including your own iOS lane.

---

## 2. iPhone <-> Android pairing: root cause PROVEN on hardware

Operator's Pixel 6a, checked live over adb: app running (pid 11582), listening
on 35079/39517/46803, **nothing bound to 9001**.

Chain:
1. `MeshRepository.kt:3333` requests `/ip4/0.0.0.0/tcp/9001`
2. `swarm.rs:2394-2424` multiport succeeds if ANY port binds
3. `mobile_bridge.rs:1106` `is_primary_tcp` accepts any TCP port, never checks
   it got the preferred one
4. `MeshRepository.kt:9335` hardcodes `/ip4/$localIp/tcp/9001` into the QR

iOS dials 9001 -> refused -> the `IoError` in the operator's logs, then
`delivery_attempt ... reason=Peer not connected`.

SECOND BUG, same payload: the QR embeds a DHCP LAN IP. The phone moved
`.137 -> .111` within one session, so a QR can go stale on a lease renewal.

---

## 3. iOS fixes ALREADY DONE (your lane) -- just need merge + rebuild

Commit `c4052f7e` "fix(ios): align identity QR and LAN discovery with Android"
already resolves two operator-reported bugs:
- QR now emits `"identity_id": identity.identityId ?? ""` (fixes the operator's
  "Missing Identity ID in Payload")
- mDNS is now dual-stack: `serviceTypes = ["_p2p._udp", "_scmessenger._tcp"]`,
  and `Info.plist` gained `_p2p._udp`

The dual-stack form is the correct choice -- flipping the constant outright
would have regressed iOS<->iOS discovery.

ACTION: Christy's installed build PREDATES this. She must rebuild or parity
testing will reproduce the old symptoms regardless of Android-side fixes.

---

## 4. Integration merge -- DONE and green

Branch `integration/unify-2026-08-01`, tip `33dbca07`, based on `origin/main`.
NOT pushed. 11 branches merged including the full iOS 0.5.0 train.

Gates at tip: `cargo check --workspace` PASS, `cargo fmt --check` PASS,
`cargo clippy -D warnings` PASS.

Two conflict resolutions worth knowing:
- `ledger_entry.rs`: took the F10 branch's bounded-read/quarantine version;
  it broke `cargo check` (E0282, E0034) and was fixed with an explicit closure
  param type and `std::io::Read::by_ref(&mut file)`.
- `mobile_bridge.rs`: git auto-merged with **no conflict marker** and silently
  reintroduced the PR #128 regression (`escalation_engine`, wrong
  `TransportType` path). Caught by post-merge symbol grep. **Always grep for
  the symbols after resolving in this repo.**

DEBT ON THIS BRANCH (do not skip):
- Mandatory adversarial review not run. It pulled real transport/routing
  changes: `swarm.rs` (`build_seed_dial_candidates` now `DnsPolicy::Reject`)
  and `mesh_routing.rs` (recency rejects stale/zero `seen_at`, clamps future
  timestamps). `.claude/rules/security.md` requires review before main.
- `cargo test --workspace --no-run` not run.

BRANCHES ALREADY LANDED -- close the PRs, do NOT merge (merging would REGRESS
main: older `ws` pin, looser workflow triggers, a stubbed npm test script):
PR #124 `gpt/codeql-regex-remediation`, #123 `gpt/npm-security-remediation`,
#120 `gpt/security-dom-hardening`, #121 `gpt/workflow-least-privilege`.

---

## 5. AWS always-on node

Instance `i-078cb870316683e79`, public IP **54.242.56.150**, t3.micro,
us-east-1, Ubuntu 24.04 (`ami-052355af2a014bd2c`), running.
SSH: `ssh -i ~/.ssh/scm-node-key.pem ubuntu@54.242.56.150`
SG `sg-02288078fa0b39e92`: TCP+UDP 9001 world-open; TCP 9876 + 22 restricted to
the operator's IP only (9876 exposes `/api/send` and `/api/contacts` -- it must
NEVER be world-open).

BLOCKED, not broken: the container cannot run because `scm relay` is one of the
deleted commands (section 1). Rebuilding the image changes nothing -- the image
builds from this same source. Resume after the CLI restore lands.

Tailscale is a RED HERRING. The old node's IP 100.56.248.69 was recorded as a
"Tailscale CGNAT address". It is not -- CGNAT is 100.64.0.0/10 and .56 is below
it; it is an ordinary public AWS IP. The old instance was simply TERMINATED
(zero instances existed in any region). There is no Tailscale anywhere in
product code and no reason to add it.

IAM note: `AllocateAddress` is DENIED for user `scmessenger-relay-orchestrator`,
so no Elastic IP. Operator accepts a rotating IP. RunInstances,
CreateSecurityGroup, CreateKeyPair are allowed.

---

## 6. NOT COMPLETED -- CLI restoration  [YOU MUST FINISH THIS]

Branch `fix/restore-cli-commands`, worktree (LOCKED):
    C:/Users/SCM/Documents/GitHub/SCMessenger/.claude/worktrees/agent-aeb9e43e7b5943b69

STATUS AT WIND-DOWN: **NOT DELIVERED.** The agent ran for a long time and was
still compiling when the session ended. The branch has ZERO commits beyond
`7eb6bd48`, and `git show fix/restore-cli-commands:cli/src/main.rs` still
returns 170 lines -- i.e. the restoration is NOT in git. Any work exists only
as uncommitted files inside that locked worktree. Inspect it before redoing the
task; there may be salvageable in-progress work.

THE TASK (unchanged, see section 1 for the diagnosis):
Merge the old command layer back on top of today's code -- NOT a revert,
because 55564b4b also carried legitimate fixes (mobile_bridge TransportType,
wasm gating) and the NEW `ShareApk` feature that must survive. Restore
`--http-bind` on the Cli struct (`cli/src/cli.rs`, struct Cli ~line 154, which
currently has ONLY a subcommand field). Remove the hardcoded dead relay
`/ip4/100.56.248.69/tcp/9001` and the unconditional fake
`Ledger Inject: [OK] ... linked` line.
Expect ~4 months of core API drift; fix properly, do not stub, and do NOT
reintroduce a catch-all arm that prints success.

Acceptance: `scm --help` lists start/relay/send/status; `scm status` does real
work or returns a real error; `scm relay --help` shows --listen/--http-port.

---

## 7. NOT COMPLETED -- port advertisement truth  [YOU MUST FINISH THIS]

Branch `worktree-agent-ae6817c8e4bd576e2`, worktree (LOCKED):
    C:/Users/SCM/Documents/GitHub/SCMessenger/.claude/worktrees/agent-ae6817c8e4bd576e2

STATUS AT WIND-DOWN: **NOT DELIVERED.** Zero commits beyond `7eb6bd48`; still
compiling at session end. Same caveat -- check the worktree for uncommitted
progress before restarting.

THE TASK (diagnosis in section 2, PROVEN on hardware):
Propagate the actually-bound port through `swarm.rs` -> `mobile_bridge.rs` ->
FFI -> `MeshRepository.kt` so the QR advertises reality.
- `swarm.rs:2394-2424`: report WHICH port bound, not just a boolean.
- `mobile_bridge.rs:1106`: stop accepting any-TCP as success when a preferred
  port was requested; log at ERROR when the preferred port is not obtained.
- `MeshRepository.kt:9335`: build connection_hints from the REAL bound port and
  the CURRENT local IP at generation time. If the real port is unknown, emit NO
  hint -- a missing hint is recoverable, a wrong hint fails silently.
Also fix the DHCP staleness in the same payload (section 2).

Acceptance: a test proving that when the preferred port is unavailable, the
system does NOT report the preferred port as bound.

---

## 7a. Android APK -- INSTALLED BUT CRASHES (my error, read this)

The Pixel 6a now has **0.4.0 installed** (versionCode 14, versionName 0.4.0,
upgraded in place, no data wipe, all six runtime permissions granted, correct
`lib/arm64-v8a/` ABI). It **crashes instantly on launch**:

    java.lang.UnsatisfiedLinkError: Error looking up function
    'uniffi_scmessenger_core_fn_func_auto_block_exempt_peer': undefined symbol
    (at MeshRepository.kt:887 initializeManagers, during Hilt DI init)

CAUSE -- and it was an orchestration mistake, not a code defect: I instructed
the build to use `-x buildRustAndroid` to conserve disk. That reuses the
prebuilt `.so` from **2026-07-28**, while the Kotlin bindings regenerate fresh
from CURRENT core. `auto_block_exempt_peer` exists today at
`core/src/mobile_bridge.rs:3463` and `core/src/iron_core.rs:4139` but is absent
from the July-28 binary.

**TRAP TO AVOID: `-x buildRustAndroid` is only safe when the prebuilt `.so`
files postdate the last change to `core/`.** Otherwise you get a guaranteed
bindings/library mismatch that builds and installs cleanly, then crashes at
runtime -- another silent-success failure mode.

FIX: one real `cargo ndk -t arm64-v8a` build. Do it AFTER section 7 lands,
since that changes `mobile_bridge.rs` and `MeshRepository.kt` anyway -- one
rebuild, not two. `core/target/android-libs` is 1.5 GB and is regenerated by
that build, so it is safe to delete to reclaim space.

The operator's phone is currently unusable for testing. Data is intact; the
next install upgrades cleanly over it. Do NOT uninstall to force a downgrade --
that wipes identity keys and message history, and requires asking the operator
first.

---

## 8. Remaining work for 0.4.0/0.5.0 parity

Landing now (blocking parity):
- Remove hardcoded dead relay from `ApkShareManager.kt:114` (~5 LoC)
- Deep-link listeners + multiaddr VALIDATION in `MainViewModel.kt:285-323`
  (~65-100 LoC). Note: `handleDeepLink` currently cannot carry a multiaddr, so
  the `?bootstrap=` param on the APK download URL is never consumed. Adding it
  creates a NEW attack surface -- arbitrary multiaddrs from an untrusted QR --
  so ship validation (reject private/loopback unless opted in, cap entries,
  rate-limit dials) WITH the feature, never after.
- Delete the fake NAT hole-punch (~200 LoC removed from nat.rs + the
  IronCore::start_hole_punch wrapper at iron_core.rs:3510)
- Topic constants centralization -- REDO properly; see section 9
- iOS inbound-message notification + unknown-sender approval prompt
  (`CoreDelegateImpl.swift`, ~20 LoC)

Deferred to 1.0.0 (agreed with operator):
- Multipeer -> libp2p swarm wiring (~50-200 LoC). iOS-only fallback; buys
  nothing for iPhone<->Android once LAN works. `mobile_bridge.rs:3426-3427`
  maps Multipeer to TransportType::Internet, so it never reaches Android.
- Freenet hole-punch port (~800 LoC, HANDOFF/freenet-lessons-learned.md).
  Sound design, wrong time -- do not add a new crypto handshake mid-freeze.
- iOS TCP subnet probe (~300-400 LoC), redundant once mDNS dual-stack works
- Repo split, PQC-09

---

## 9. Hermes session state (operator stopped it)

Hermes's uncommitted work is preserved at `stash@{0}`
(`hermes-inflight-2026-08-01`). It applies cleanly to the integration tree but
will NOT compile as-is.

- `cli/src/bootstrap.rs`, `cli/src/ledger.rs`: CORRECT and complete -- topic
  constants wired from `scmessenger_core::{TOPIC_LOBBY,TOPIC_MESH}`.
- `core/src/transport/swarm.rs`: **BROKEN -- do not restore.** Contains
  `format!("{}{}", "sc-receipt-convergence".trim_start_matches("sc-"), "")`
  which yields topic `receipt-convergence` instead of `sc-receipt-convergence`.
  That would silently break receipt delivery. Redo the U2 task properly.

Hermes also violated the orchestration rule by implementing directly instead of
dispatching. The redo should go through `delegate_task.py`.

OPEN CONFLICT for you to settle: `ORCHESTRATOR_DISPATCH_PLAN_2026-08-01.md`
lists PQC-09 in the first dispatch wave;
`WINDOWS_QWEN_CONSOLIDATION_REQUEST_2026-08-02.md` (newer, 19:28) says PQC-09 is
PARKED during the 0.4.0 freeze. I treated the newer consolidation request as
authoritative. Confirm.

---

## 10. Your open item: PQC-07

`HANDOFF/gpt/GPT_SOL_ULTRA_PQC07_RATCHET_DESYNC_2026-08-01.md`, pushed to main
as `48aec750`. Contains a correction to a FALSE premise in
`E01B_FABLE_DESIGN_HANDOFF.md` -- read it before designing. Reply as
`GPT_SOL_ULTRA_PQC07_RATCHET_DESYNC_RESPONSE.md`.

---

## 11. Environment facts worth keeping

- Disk: ~13 GB free on C: (95% full), BELOW the repo's 25 GB build threshold.
  Use `cargo check` and `-j6`. Never `cargo clean --target <triple>` -- it
  wipes all of `target/`.
- One build tool at a time on this box; several sessions share the repo.
  `tasklist | grep -iE "cargo|rustc|gradle|java"` before building.
- Android APK: build with `-x buildRustAndroid` to reuse the prebuilt `.so` in
  `core/target/android-libs/` and avoid the disk-heavy Rust rebuild.
- Pixel 6a stable adb handle (survives wireless-debug port rotation):
  `adb -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp ...`
  Phone is currently on **0.3.4 / versionCode 12** -- the 0.4.0 install has NOT
  happened yet. Do not uninstall to force it; that wipes identity keys and
  history. Operator must be asked first.
- `Cargo.lock` is tracked as an EMPTY file on main since at least 55564b4b, so
  every cargo run produces a large uncommitted diff. Pre-existing; fix separately.
- Qwen free tier WORKS but needs model rotation (`qwen-thinking` 404s;
  `qwen3-coder-plus` rate-limits; `qwen3-coder-plus-2025-09-23` succeeded).
  Good for log/evidence analysis, weak without `--files` context.
- Fusion Lite needs vendor-prefixed slugs: `deepseek/deepseek-v4-pro`,
  `moonshotai/kimi-k2-thinking`, `qwen/qwen3-235b-a22b-thinking-2507`.
  Full Tier-B panel + judge ~$0.02-0.03.

---

## 11a. AT-RISK / UNCOMMITTED STATE -- recover or discard deliberately

Nothing below is on `origin`. If these are lost, the work is lost.

| What | Where | Disposition |
|---|---|---|
| Integration merge, 11 branches, all 3 gates green | local branch `integration/unify-2026-08-01`, tip `33dbca07` | **NOT PUSHED.** Highest-value artifact of the session. Push or re-merge. |
| CLI restoration (incomplete) | locked worktree `.claude/worktrees/agent-aeb9e43e7b5943b69` | uncommitted; inspect before redoing |
| Port advertisement fix (incomplete) | locked worktree `.claude/worktrees/agent-ae6817c8e4bd576e2` | uncommitted; inspect before redoing |
| Hermes in-flight work | `stash@{0}` `hermes-inflight-2026-08-01` | bootstrap.rs + ledger.rs GOOD; swarm.rs BROKEN -- see section 9 |
| Two plan documents written this session | `HANDOFF/V040_COMPLETION_PLAN_2026-08-01.md`, `HANDOFF/V040_V050_UNIFIED_PLAN_2026-08-01.md` | untracked; commit if wanted |
| iOS/inbound triage ticket | `HANDOFF/todo/IOS_QR_AND_INBOUND_MESSAGE_TRIAGE.md` | untracked |
| Qwen triage output | `tmp/IOS_QR_AND_INBOUND_MESSAGE_TRIAGE_response.md` | untracked, `tmp/` is gitignored |

Only TWO things reached `origin/main` this session, both docs-only:
`48aec750` (PQC-07 request) and this handoff file.

Stale worktrees pinning old commits and consuming disk (safe to prune):
two Antigravity subagent worktrees under `~/.gemini/antigravity/...`,
`SCMessenger-w1` on `wip-w1-ledger`, and `.claude/worktrees/e01c-pq-mixing`.

---

## 11b. Consolidated remaining work

BLOCKING 0.4.0 TAG:
1. CLI restoration (section 6) -- everything is gated on this
2. Port advertisement truth (section 7)
3. Adversarial security review owed on `integration/unify-2026-08-01`
   (transport/routing changes, section 4) -- `.claude/rules/security.md`
4. `cargo test --workspace --no-run` on the integration branch
5. Rebuild arm64 `.so` + one clean APK; confirm the app LAUNCHES (section 7a)
6. AWS node container relaunch once the CLI runs (section 5)
7. Seeding finding F2 operator decision; F6/F7/F12/F13/NEW-5/NEW-6 are CLOSED
   but explicitly "pending terminal verdict" -- the 040-S2 adversarial verdict
   has never been produced
8. Real ConnectionEstablished + receipt proof, both directions, provenance match

BLOCKING PARITY (0.4.0 Android <-> 0.5.0 iOS):
9. Christy rebuilds iOS from a branch containing `c4052f7e` (section 3)
10. Remove hardcoded dead relay from `ApkShareManager.kt:114` (~5 LoC)
11. Deep-link listeners + multiaddr VALIDATION, `MainViewModel.kt:285-323`
    (~65-100 LoC) -- ship validation WITH the feature, never after
12. Delete fake NAT hole-punch (~200 LoC removed, section 0 item 2)
13. Topic constants centralization -- redo via `delegate_task.py` (section 9)
14. iOS inbound notification + unknown-sender prompt (`CoreDelegateImpl.swift`)
15. Close PRs #120/#121/#123/#124 -- already landed, merging REGRESSES main

DEFERRED TO 1.0.0 (operator-agreed): Multipeer->swarm wiring; Freenet
hole-punch port; iOS TCP subnet probe; repo split; PQC-09.

OPEN QUESTIONS FOR YOU:
- PQC-09 conflict between the two plans (section 9) -- confirm the newer
  consolidation request is authoritative
- PQC-07 design request `48aec750` (section 10)
- `Cargo.lock` tracked as an EMPTY file on main since >= 55564b4b -- decide

---

## 12. Recommended order for you

1. Land the CLI restore (section 6) -- everything else is gated on it.
2. Land the port-advertisement fix (section 7).
3. Run the adversarial review owed on `integration/unify-2026-08-01` (section 4).
4. `cargo test --workspace --no-run`.
5. ONE Android APK build from the unified tree; install on the Pixel; verify
   versionName reads 0.4.0/14.
6. Rebuild Christy's iOS from a branch containing `c4052f7e`, SAME SHA as the
   Android build -- provenance match is a release gate.
7. Pixel <-> iPhone: BLE first (UUIDs already align: DF01 service, DF02-DF04
   chars), then LAN/mDNS.
8. Then the AWS node (section 5), then remaining parity items (section 8).

Do NOT tag 0.4.0 until the seeding adversarial verdict, a real
ConnectionEstablished + receipt proof both directions, and provenance match are
all green.
