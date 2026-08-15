# CAO state - live Apple handoff

Status: Active
Last updated: 2026-08-15 10:35 HST
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
| iOS UniFFI crash | `gpt/ios-macos-launch-debug-20260810` at `fee09225`, pushed | Checksum mismatch fixed; binding verifier passed; simulator XCTest 53/53 | Open/manage PR; observe CI; independent patch acceptance |
| Free/provider adapters | worktree `tmp/free-api-lanes-20260815`, branch `gpt/free-api-lanes-20260815` from `ef431acc` | Worker patch reviewed; six unit tests, py_compile, rules, docs-sync, JSON parse pass | Independent review; live adapter smoke if justified; commit/push/PR |
| CAO harness | global skill `~/.prime/agent/skills/cao/` plus these two project files | Skill/frontmatter and global `/CAO` alias created | Commit/push these files; fresh no-tools command smoke |
| Flash High controller | startup packet under provider worktree `tmp/orchestration/plans/` | First launch stopped for out-of-scope home-directory discovery | One hardened absolute-path retry only |
| Apple parity program | draft `HANDOFF/gpt/GPT_MAC_CAO_TOKEN_CURB_PARITY_ADOPTION_2026-08-15.md` | Read-only parity audit and initial plan complete | Feed bounded plan to controller; dispatch fresh SCANNER/PLANNER roles |

Latest fetched `origin/main` at this update: `ebf5411b`. The Apple fix branch is
not rebased and must not be force-pushed. Re-derive with:

```bash
git fetch origin main
git log --oneline -3 origin/main
git log --oneline origin/main..gpt/ios-macos-launch-debug-20260810
gh pr list --head gpt/ios-macos-launch-debug-20260810 --state all
```

## 3. Critical path

1. Land and smoke-test the `/CAO` command/state without touching unrelated dirty
   files.
2. Open the iOS crash-fix PR from `fee09225`; record exact source and CI artifact
   anchors separately.
3. Finish provider-adapter independent review and PR lifecycle from its isolated
   branch.
4. Retry `agy/gemini-3.7-flash-high` once with absolute packet/worktree paths,
   `--add-dir`, process-group containment, dynamic resource admission, and early
   drift monitoring.
5. Accept only tmp-owned controller artifacts, then dispatch exact Apple
   SCANNER/PLANNER packets before any implementation.
6. Resolve stale checked-in XCFramework headers/libraries through a canonical
   rebuild plan. The current build script removes the XCFramework directory;
   explicit operator approval is required before that destructive invocation.
7. Continue authoritative simulator/device gates without claiming APNs,
   physical-device, cross-platform, or delivery evidence that was not observed.

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
| agy Gemini 3.7 Flash High | exact model listed by `agy models` | Primary controller |
| NVIDIA NIM | 102 models listed; DeepSeek V4 Flash exact `LANE_OK` | Primary free CODER/review lane |
| Groq | Llama 3.1 8B exact `LANE_OK`; 6000-token rate header observed | Fast bounded FLASH lane |
| Cerebras | fixed USD 5 trial; minimal ZAI GLM 4.7 call HTTP 200, 27 tokens | Manual metered backup, automatic routing disabled |
| Google AI Studio direct | operator reported key added, canonical `gemini.env` was empty on last check | Verify key location/model IDs before enabling |

Secret slots are `~/.config/scmorc/*.env`, mode 0600. Never print or search for
keys outside their canonical slots. Record each inference in the lake ledger.

## 6. Controller startup regression

The first Flash High controller launch (PID 22339) was terminated by CAO
oversight before dispatch. It ran `find /Users/christylove` while trying to
locate the known CAO packet. That crossed approved scope. No scanner was
launched, no tracked file changed, and its reservation was released.

Mandatory retry controls:

```text
absolute controller worktree
absolute input packet
agy --add-dir <exact-worktree>
start_new_session=True / dedicated process group
early inspect process tree and git diff
forbid filesystem discovery outside approved roots
one retry only; then native GPT backup controller
```

## 7. Open - do not guess

1. **Direct AI Studio key location.** The operator said it was added, but it was
   not visible in `GEMINI_API_KEY`, `GOOGLE_API_KEY`, or the canonical
   `~/.config/scmorc/gemini.env` at last check. Ask for confirmation of the
   canonical slot; never scan arbitrary files.
2. **XCFramework rebuild permission.** `scripts/build_xcframework.sh` contains
   `rm -rf` for `iOS/SCMessengerCore.xcframework`. Running it requires explicit
   operator approval for that invocation. Header drift remains a separate
   package-artifact issue even though the app/tests now pass.
3. **Physical device and APNs.** Signing/team/account decisions and a deliberate
   physical-device run are still needed. Simulator results do not close them.
4. **Provider adapter base.** `origin/main` advanced from `ef431acc` to
   `ebf5411b` after the adapter worktree was created. Do not rebase destructively;
   evaluate a safe merge/update in isolation after review.
5. **PR139 remains hard no-go.** No live deployment, responder activation,
   contact/device mutation, sends/replies, service reloads, or delivery claims.

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

## 10. Graceful handoff checklist

Before winding down:

1. Update this file with exact commits, PR/check status, dispatch references,
   gates, blockers, and next safe command.
2. Update the global CAO skill state reference as a local fallback.
3. Verify no live controller/worker/build process and no owned reservation.
4. Push only explicit verified CAO-owned commits on `gpt/*` branches.
5. Start a fresh context with `/CAO`; it must read this file first and re-derive.
