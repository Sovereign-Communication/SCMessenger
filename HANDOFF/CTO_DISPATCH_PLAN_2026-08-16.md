# CTO dispatch plan -- 2026-08-16

Status: Active
Supersedes: the merge-train section of `HANDOFF/CTO_STATE.md` (which inverted --
see "What changed since the handoff")
Derived from: CI logs, `git merge-tree` simulations, `scripts/pr_scope.sh 139`

---

## 1. The headline

**The remaining critical path to a green trunk contains almost no code to
write.** Every one of the four red lanes on #139 is already fixed by a PR that
is open, written, and (for the one that needs it) adversarially reviewed.

The work between here and the tag is *merge, verify, tag* -- not implement.
Dispatch should be budgeted accordingly: the expensive lanes are needed for D4,
not for D1.

---

## 2. What changed since the 2026-08-15 handoff

`CTO_STATE.md` says Mobile/KSP UniFFI is red and `Test` passes. **That has
inverted.** Re-derived from the literal logs on run 31918165979 and siblings:

| Lane on #139 | Handoff said | Actually now |
|---|---|---|
| `Mobile` (KSP `NonExistentClass`) | RED | **GREEN** -- all 4 ABIs, Kotlin + Swift bindings, Android Debug APK, Android JVM Unit Tests |
| `Test` (win/ubuntu/macos) | PASSED | **RED** |
| `macOS Native Tests` | not listed | **RED** |
| `Lint` / `Rust Linting` | `core/src/lib.rs:159` | RED, but a **different** diff -- lib.rs:159 is fixed |

The UniFFI bug is solved. What is red now is *newer* breakage: #162's
five-layer integration suites landed on `tracking` carrying an unformatted file
and two genuine transport defects they were written to catch.

---

## 3. Every red lane, root cause, and its fix

| Red lane | Root cause (from the log, not inferred) | Fixed by |
|---|---|---|
| `Lint` | rustfmt diff, `core/tests/integration_wan_swarm_node2.rs` at :83 :92 :444 :751 | **#169** |
| `Rust Linting` | the same four diffs | **#169** |
| `Repository Hygiene` | trailing whitespace -- `ShareReceiver.kt` (CRLF) and `docs/RELEASE_NOTES_v0.4.0-alpha.1.md` (markdown hard-breaks) | **#169** |
| `Test` (windows/ubuntu/macos) + `macOS Native Tests` | `integration_wan_swarm_node2`: 16 passed / **2 failed** -- `layer3_panic_safety_and_boundaries::test_internet_relay_boundary_safety` and `::test_transport_manager_boundary_safety` | **#165** |

Nothing else on #139 is red. 24 other checks pass, including `FFI Surface
Contract`, `WASM`, `Docs`, `CodeQL`, `iOS Build`.

---

## 4. Merge mechanics -- verified, not assumed

Simulated read-only with `git merge-tree --write-tree`. No working tree touched.

- **#169 -> tracking: clean.** Net: `.gitattributes` (+3), five Kotlin files,
  the test file, release notes.
  - **Proven semantically neutral.** Under
    `git diff --ignore-all-space --ignore-cr-at-eol`, the entire ~1,800-line
    Kotlin churn disappears. Only `.gitattributes` (+3) and the rustfmt reflow
    survive. The Kotlin diff is pure line-ending renormalization.
- **#165 -> tracking: clean.** Net: **only** `core/src/transport/internet.rs`
  (+40) and `core/src/transport/manager.rs` (+58).
- **The two are fully disjoint. Merge order does not matter.**
- `git diff tracking..165` appears to delete 1,586 lines. It does not. #165 is
  24 commits *behind* tracking; that is the two-way diff, not the merge result.
  **Ancestry, not diff stat** -- the §8 lesson, recurring.
- **`main` vs `tracking` = `0 193`.** `main` holds zero commits that are not on
  `tracking`. #139 is a pure fast-forward and **cannot conflict.**

### #165's gate is already satisfied

A full adversarial review is posted on the PR: **APPROVE, zero findings,
`CRYPTO_TOUCHED: NO`**, with checks A-E recorded (latency scoring across
`0/100/101/u32::MAX`; `conn_duration.max(1)` proven fail-closed; capabilities
shown to be locally-registered and not attacker-controlled; no crypto touched;
no new panic path).

And the empirical proof: **`macOS Native Tests` PASSES on #165** (36m28s) while
failing on both #139 and #169. The fix is demonstrated, not claimed.

---

## 5. Non-blockers that look like blockers

Do not spend a dispatch on any of these.

- **#165 "Android JVM Unit Tests -- fail, 6h0m15s".** The run is `cancelled`,
  not failed. The `Install host Rust dependencies` step stalled for six hours
  and hit the GitHub Actions job ceiling; `Run JVM unit tests` never executed
  (`skipped`). **Re-run it. There is nothing to debug.**
- **#170 `Lint` / `Rust Linting` red.** The diff is `core/src/lib.rs:159` --
  **inherited from `main`**, which `tracking` already fixed. #170 touches only
  `.py` and `.md`. It goes green by itself once #139 lands.
- **#169's single red lane** is the identical 16-passed/2-failed #165
  signature. Expected. Clears when #165 lands.
- **#152** conflicts with `tracking` and is probably obsolete (whitespace on
  four Android files that #164 and #169 have since renormalized). Verify after
  #139; **do not close it blind.**

---

## 6. Critical path

```
  #167 + #168  (green, tooling)  ---- merge any time, improves dispatch
                                          |
  #169 (fmt + EOL + whitespace) ----+     |
                                    |-> tracking green -> #139 -> main   [D1 + D5]
  #165 (transport saturating fix) --+                        |
                                                             v
                      branch protection -> #154 -> tag v0.4.0-alpha.1    [D2 + D3]
                                                             |
                       docker-publish fires on main push ----+
                                                             v
                            rebuild AWS node to tagged SHA -> D4
```

1. Merge **#167** + **#168** -- both fully green, both improve dispatch
   reliability (guard false positives, dispatch timeout floor).
2. Merge **#169** -> tracking. Clears three lanes.
3. Merge **#165** -> tracking. Clears the fourth. Gate already signed off.
4. Re-run #139's checks; confirm green.
5. `bash scripts/pr_scope.sh 139`, name each surviving blocker explicitly,
   merge **#139 -> main**. This is **D1 + D5**.
6. `bash scripts/apply_branch_protection.sh --apply`. `enforce_admins` true, **0**
   required approvals. **Do NOT list `Docker Integration Suite`** as required.
7. Merge **#154**, then tag `v0.4.0-alpha.1`.
8. Verify the published APK is genuinely release-signed -- `apksigner verify
   --print-certs`; `CN=Android Debug` means it did not sign. **D2 + D3.**
9. `docker-publish.yml` fires on the push to `main` and publishes
   `sha-<7char>`. Rebuild the AWS node to that image. **Never build on the
   t3.micro** (a previous attempt ran 16h and OOMed).
10. Run **D4** per `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md`. Score on **receiver-side
    decrypt + durable history + receipt** -- never transport ACKs, UI counters,
    or BLE local acceptance.

### The five #139 blockers, pre-answered

`pr_scope.sh` will raise five. Four are already resolved; name them, do not
silently override:

1. *"100 commits, is this based on the branch you are merging into?"* --
   intentional. `tracking` **is** the long-lived integration branch and merging
   it is exactly what D5 asks. (The count is also wrong; see §8.)
2. *"touches merge-blocked directories"* (6 files) -- the crypto-security-auditor
   verdict exists; its one HIGH is fixed by #157, already merged into tracking.
3. *"failing checks"* -- **must genuinely be green.** Do not merge on pending.
4. *"no conflicts"* -- clean, and structurally cannot conflict (§4).

---

## 7. Dispatch routing

Lanes re-derived from `scripts/lanes.json`, probed 2026-08-15 (fresh; treat as
stale after 7 days or after any 401). **Never route from a remembered ranking.**

Capacity: 13 free HTTP lanes (workers, **no shell, cannot verify anything they
claim**); `agy-gemini` free-quota **with a shell**; `agy-claude` metered, shell
(spends Anthropic quota); `claude-native` expensive, verdicts only.

| Work | Lane | Why |
|---|---|---|
| Steps 1-5 (merges, re-runs, gate reads) | **CTO seat, inline** | These are merge buttons plus verification. Dispatching them costs more than doing them and adds a trust hop. |
| Watch #139 to green; re-run #165's cancelled Mobile lane | **agy-gemini** (shell, 30m+) | Needs `gh`. Long poll, zero judgement. |
| AWS node rebuild to the tagged SHA | **agy-gemini** (shell, 90m) | Needs `ssh`/`docker`. **`ssh ec2-user@`, never `ubuntu@`** -- Amazon Linux 2023. Address from `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`; the IP is dynamic by design. |
| D4 execution (Pixel 6a <-> node) | **agy-gemini** (shell, 90m) | Needs `adb`. Must return raw evidence, not a summary. |
| **D4 scoring / go-no-go** | **claude-native** | Human-consequential verdict. A worker fabricated a full health report for a node that was down. |
| `pr_scope.sh` commit-count fix (§8) | **cerebras-gptoss-120b** or **google-gemini31-flash-lite** | Free, micro, one-line. Serialize Cerebras -- 5 req/min is enforced as ~1 req/sec. |
| Doc/status sync after the tag | **or-free-router** or **nim-nemotron-super-120b** | Free, text-only, no shell needed. |

**Rules that hold regardless of lane.**

- Only **agy** has a shell. Anything running `gh`/`cargo`/`gradlew`/`adb`/`ssh`
  goes there. An HTTP lane claiming it ran a command is fabricating.
- `gpt-oss-120b-medium` sits in **agy's Claude pool** and spends Anthropic quota
  despite the name. Pin `--model` explicitly; shorthand silently substitutes.
- Always pass `--add-dir`. Without it agy re-discovers the repo path every
  dispatch and often bails before finishing.
- **90 minutes, not 45.** Three "capability failures" on this project were
  too-short timeouts. `--print-timeout` is a TOTAL wait. On timeout use
  `--continue`; check the worktree before re-dispatching -- the relay-ladder fix
  died at 36m with the work complete but unpushed.
- **Dispatch into a `git worktree`, never the shared checkout.** An agent
  switched the live branch under a session on 2026-08-15.
- One build tool at a time. Shared cargo cache:
  `CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target`,
  `CARGO_INCREMENTAL=0`.
- **Validate every completion claim.** A claim without command output is a
  claim. Scope the validation as carefully as the claim.

---

## 8. New finding -- `pr_scope.sh` commit count is truncated

`scripts/pr_scope.sh:121` computes the commit count as
`len(json.load(sys.stdin)["commits"])` from `gh pr view --json commits`. That
field **caps at 100**. On #139 it reports `100 commits`; `git rev-list --count
origin/main..origin/tracking/pre-v040-tag-work` reports **193**.

This is the identical API-cap truncation that #158 repaired for the *file* list,
left in place for *commits*. Severity is LOW -- the blocker still fires
correctly -- but it under-reports scope by 48% on the largest PR in the repo,
and §8 of `CTO_STATE.md` already warns that **exactly 100 means truncated**.

Fix: derive from `git rev-list --count <base>..<head>`, matching the repair
already applied to the file list. Micro task, free lane.

---

## 9. Still open -- do not guess

Carried forward from `CTO_STATE.md` §7, unchanged:

1. Was `ebf5411b`'s deletion of 7 Android sources intentional? Restored on #149.
2. Josh single-transport build -- **not** the v0.4.0 default; ships as v0.3.9 if
   at all.
3. README framing -- CEO was asked to bless the honest-first tone. No reply.
4. Dependency debt -- 7 vulnerabilities on the default branch, 3 high. Deferred
   to post-tag S4. Right call for shipping; should not stay deferred long on a
   security product. The 13 dependabot PRs are that queue: **defer all, close
   none.**
