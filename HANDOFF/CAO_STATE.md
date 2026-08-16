# CAO state - live Apple handoff

Status: Active
Last updated: 2026-08-15 17:18 HST
Entry point: `/CAO`. This file is the whole initial project context load.

Everything below names its evidence or re-derivation command. **Re-derive before
acting** - this file ages, the repository and provider APIs do not.

---

## 1. The goal

Own SCMessenger's Apple lane as **Chief Apple Officer**: make iOS and macOS
mature, secure, and behaviorally aligned with Android, Windows, CLI, and cloud
nodes while respecting the operator-set release sequence. The CAO supplies
priorities and acceptance criteria to a supervised Control Plane v2 controller;
the CAO does not write application code.

Human `OPERATOR` authority remains intact for product, release, architecture,
API breaks, technology changes, and material security/privacy choices.

## 2. In flight

| Work | Source / state | Verified status | Next |
|---|---|---|---|
| iOS UniFFI crash | receipt fix `a29e53f3` then binding fix `fee09225`; CAO command/state branch | Binding verifier passed; simulator XCTest 53/53 | Do not cherry-pick `fee09225` alone; route the Core dependency through security review and the Windows authoritative gate before PR acceptance |
| Free/provider adapters | commit `d85eb6b9`; PR #170; branch `gpt/free-api-lanes-20260815` | 11 tests plus deterministic gates; direct Google 3.7 Flash HTTP 200; Google VALIDATOR and Copilot SECOND_OPINION satisfied | Observe PR #170; keep its four-file scope; current Rust lint failures are from base `ebf5411b` |
| CAO harness | deterministic Prime routing plus global fallback skill | Exact fresh-process `/CAO` no-tools smoke passed; state/controller/safety routing correct | Re-derive before each new dispatch |
| Flash High controller | evidence under provider worktree `tmp/orchestration/` | Hardened retry stopped after it continued past required-stop HTTP 404 | No further Flash High retry this cycle; use approved bounded worker/reviewer lanes |
| PR #139 Apple checks | head `4030d166`; read-only observation at 17:11 HST | simulator, Swift lint, and Cross iOS green; macOS native red; Mobile iOS running and Swift bindings queued | Do not duplicate CI or Apple implementation; recheck after in-flight jobs settle |
| Apple parity program | draft `HANDOFF/gpt/GPT_MAC_CAO_TOKEN_CURB_PARITY_ADOPTION_2026-08-15.md` | Read-only parity audit and initial plan complete | Dispatch fresh bounded SCANNER/PLANNER roles only when capacity and CI contention allow |

Latest fetched `origin/main` at this update: `ebf5411b`. The Apple fix branch is
not rebased and must not be force-pushed. Re-derive with:

```bash
git fetch origin main
git log --oneline -3 origin/main
git log --oneline origin/main..gpt/ios-macos-launch-debug-20260810
gh pr list --head gpt/ios-macos-launch-debug-20260810 --state all
```

## 3. Critical path

1. Observe PR #170 without widening it. Its source anchor is `d85eb6b9`; the
   direct API smoke and review outputs are runtime evidence, not source state.
2. Construct the iOS receipt/crash PR only as a dependency-correct stack:
   `a29e53f3` then `fee09225`. The Core change requires the Windows authoritative
   gate and audit review; `fee09225` alone would leave generated checksums stale.
3. Do not relaunch Flash High this cycle. Copilot is a bounded local
   worker/reviewer with exactly `--model auto`; direct free API lanes remain
   workers, never controller authority.
4. Dispatch exact Apple SCANNER/PLANNER packets before any new implementation,
   and only when current CI/resource contention has settled.
5. Resolve stale checked-in XCFramework headers/libraries through a canonical
   rebuild plan. The current build script removes the XCFramework directory;
   explicit operator approval is required before that destructive invocation.
6. Continue read-only PR #139 observation and authoritative simulator/device
   gates without claiming APNs, physical-device, cross-platform, or delivery
   evidence that was not observed.

## 4. What was solved this session

**The XCTest crash.** The crash report and baseline gate agreed on
`SCMessenger/api.swift:11763`: generated Swift expected
`uniffi_scmessenger_core_checksum_method_ironcore_prepare_receipt() == 11532`,
while current Rust generated `25228`. Canonical regeneration updated the Swift
and C bindings. A deterministic generated-text sanitizer keeps repository rules
compatible with UniFFI output; its unit tests pass.

Authoritative Mac evidence:

```text
bash scripts/verify_ios_bindings.sh -> iOS binding verification passed
xcodebuild test ... SCMessengerTests ... -> 53 tests, 0 failures
** TEST SUCCEEDED **
```

Evidence log: `tmp/cao-final-uniffi-gate-20260815.log`. This is simulator gate
evidence only. The crash was the XCTest host launched during this session, not a
proven physical-device or field crash.

**Control hierarchy verified.** Control Plane v2 implementation `46d33a26` was
merged by PR #145 at `ef431acc`; protocol/manifest `2.0.0` validates. The
committed CTO pattern was located at `.claude/commands/CTO.md` and
`HANDOFF/CTO_STATE.md` (initial command/state commit `abbe9f08`) and is the
format mirrored here.

**Exact `/CAO` routing verified.** Prime slash aliases are prompt templates;
skills alone are invoked as `/skill:name`. The tracked template at
`.prime/agent/prompts/CAO.md` and its project setting force deterministic
expansion, while `~/.prime/agent/prompts/CAO.md` and the enabled global skill
provide fallback outside this checkout. A fresh `prime-agent --no-session
--no-tools` process returned CAO, `HANDOFF/CAO_STATE.md`, and
`agy/gemini-3.7-flash-high` exactly. Static expansion also asserts the broad
filesystem stop rule. Evidence: `tmp/cao-enabled-template-smoke-20260815.log`.
The first cheap model selection was unsupported and two pre-enablement routing
attempts failed closed; none used tools or modified repository state.

**Free lanes operationalized.** Commit `d85eb6b9` is pushed and PR #170 is
open with exactly four tracked files. The live adapter called direct Google AI
Studio `gemini-3.7-flash` and received HTTP 200 plus exact
`AISTUDIO_LANE_OK`; no secret value was printed or committed.
`gemini-2.5-flash` is unavailable to new users, while
`gemini-3.1-pro-preview` exposes a zero free-tier quota, so direct THINK fails
closed. Google VALIDATOR and Copilot SECOND_OPINION footers both parsed as
`SATISFIED`. Copilot used only `--model auto`, which selected `gpt-5-mini` with
zero premium requests, no tools, and no file changes. The NVIDIA review retry
timed out; no NVIDIA review acceptance is claimed.

**Controller retry consumed.** The sole hardened Flash High retry did dispatch a
scoped IMPLEMENTER, but the controller continued after a required-stop HTTP 404
and was terminated. All controller/worker process groups exited and their
resource reservations were released. Do not retry Flash High again this cycle.

## 5. Controller and lane policy

Hierarchy:

1. Human OPERATOR.
2. CAO/native GPT: Apple strategy, plan inputs, controller oversight, Mac gate
   verification, escalation.
3. Primary controller: `agy/gemini-3.7-flash-high` as manifest CONTROLLER.
4. Fresh semantic workers in isolated worktrees.
5. Native GPT as final backup controller/reasoning layer, not implementer.

Verified provider evidence:

| Lane | Evidence | Policy |
|---|---|---|
| agy Gemini 3.7 Flash Low | exact `LANE_OK` | Small worker/overflow |
| agy Gemini 3.7 Flash High | exact model listed by `agy models`; hardened retry breached its stop condition | Primary controller in later cycles only; no further retry this cycle |
| NVIDIA NIM | 102 models listed; DeepSeek V4 Flash exact `LANE_OK`; review retry timed out | Free CODER/review lane when healthy; record degraded outcomes |
| Groq | Llama 3.1 8B exact `LANE_OK`; 6000-token rate header observed | Fast bounded FLASH lane |
| Cerebras | fixed USD 5 trial; minimal ZAI GLM 4.7 call HTTP 200, 27 tokens | Manual metered backup, automatic routing disabled |
| Google AI Studio direct | canonical slot `~/.config/scmorc/AIstudio.env`; `gemini-3.7-flash` HTTP 200 and exact sentinel | Free FLASH/CODER; THINK fails closed |
| GitHub Copilot CLI | `auto` selected `gpt-5-mini`; zero premium requests; satisfied SECOND_OPINION | Bounded local worker/reviewer only; use exactly `--model auto`, no explicit model, no commit/push authority |

Secret slots are `~/.config/scmorc/*.env`, mode 0600. Never print or search for
keys outside their canonical slots. Record each inference in the lake ledger.
Preserve the remaining 36% native GPT weekly capacity for CAO oversight through
2026-08-20; do not spend it on implementation or routine review workers.

## 6. Controller safety outcome

The first Flash High controller launch (PID 22339) was terminated by CAO
oversight before dispatch after broad home-directory discovery. The one
hardened retry used an absolute detached worktree and packet, exact `--add-dir`,
a dedicated process group, dynamic reservation, and early process/diff
monitoring. It dispatched one scoped IMPLEMENTER but later continued after a
required-stop HTTP 404, so CAO terminated the entire process group. No tracked
controller-worktree change survived and all reservations were released.

Standing controls:

```text
absolute controller worktree and immutable input packet
agy --add-dir <exact-worktree>
start_new_session=True / dedicated process group
dynamic reservation plus early process-tree and tracked-diff inspection
forbid filesystem discovery outside approved roots
terminate on required-stop, scope, process, or diff breach
no further Flash High retry this cycle
```

## 7. Open - do not guess

1. **iOS fix dependency and audit.** `fee09225` was generated from the Core
   receipt change in `a29e53f3`; do not open a fee-only PR or claim that the
   generated checksum is valid against current `main`. The Core portion routes
   through the Windows authoritative gate and audit before acceptance.
2. **XCFramework rebuild permission.** `scripts/build_xcframework.sh` contains
   `rm -rf` for `iOS/SCMessengerCore.xcframework`. Running it requires explicit
   operator approval for that invocation. Header drift remains a separate
   package-artifact issue even though the app/tests now pass.
3. **Physical device and APNs.** Signing/team/account decisions and a deliberate
   physical-device run are still needed. Simulator results do not close them.
4. **PR #170 base failure.** The PR is mergeable and scoped to four files, but
   both Rust lint jobs currently fail on unformatted `core/src/lib.rs:159` from
   base `ebf5411b`. Do not absorb that unrelated repair into PR #170.
5. **PR #139 remains hard no-go.** No live deployment, responder activation,
   contact/device mutation, sends/replies, service reloads, or delivery claims.
   Current Apple CI evidence is mixed and still evolving; observe, do not rerun.

## 8. Shared-checkout and provenance rules

The shared checkout is intentionally dirty. Known unrelated work includes
`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` and orchestration/resource
files. Never stash, restore, reset, clean, or include them. Stage explicit paths.

Source, artifact, and runtime evidence are separate:

- Source anchor: immutable commit plus named diff.
- Artifact anchor: exact build/test result or content hash.
- Runtime anchor: exact process/device/node and observation time.

A green build is not delivery, a queued send is not receipt, and simulator
execution is not a physical-device result.

## 9. Standing lessons

- Open the file before classifying it. The CTO command/state already existed on
  remote branches even though a filename search of `main` found nothing.
- A known path that triggers broad discovery is a controller safety failure, not
  harmless exploration.
- Short inference caps can finish during reasoning and yield empty content; do
  not call that a useful worker result.
- Provider/model availability never changes semantic authority.
- Repeated worker failure causes rebrief/re-dispatch/escalation, never a direct
  CAO code fix.
- Prime exact slash names require enabled prompt templates; a skill description
  or model-level alias is not deterministic command routing.

## 10. Graceful handoff checklist

Before winding down:

1. Update this file with exact commits, PR/check status, dispatch references,
   gates, blockers, and next safe command.
2. Update the global CAO skill state reference as a local fallback.
3. Verify no live controller/worker/build process and no owned reservation.
4. Push only explicit verified CAO-owned commits on `gpt/*` branches.
5. Start a fresh context with `/CAO`; it must read this file first and re-derive.
