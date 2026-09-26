# SCMessenger owner handoff — amendment 1: the Android fix scope is three PRs, and a silent Dependabot cargo outage

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This is an assist-only handoff. The owning repository retains all decisions, edits, merges, and publication authority.

It **amends**, and does not replace, `HANDOFF/JEV_AUDIT_ANDROID_GATE_2026-09-25.md`. Two things in that document are now known to be wrong or incomplete:

1. Its Finding 1 names a single carrier (#361) for the two broken `it.action ==` assertions. There are **three**, and all three are based on `main`.
2. It carries no finding at all for a cargo dependency-update outage that has been silently failing.

Everything else in the 2026-09-25 document still holds at the current head.

## Pinned state

- Measured `main` at `bceacb94` ("feat(handoff): add an auditable, per-document waiver register (#387)"), 2026-09-26. The two commits since `cd511fa0` are handoff/governance only and touch **no product code**, so every line number cited below is current.
- 49 pull requests were open at measurement time. All were enumerated; none were modified.
- All evidence below is read-only. No product worktree was touched, no branch of this repository was used for measurement, nothing was deployed.

---

## Amendment A — the broken assertions are carried by three pull requests, not one

### What the previous handoff said, and why it is incomplete

`HANDOFF/JEV_AUDIT_ANDROID_GATE_2026-09-25.md` reports the root cause of the red `Android JVM Unit Tests` gate and recommends cherry-picking `e7466639b59f6c07f19090ecd9867b39ac36b58e` onto **#361**. That advice is correct for #361 and insufficient overall. Because the other two carriers are also based on `main`, fixing only #361 leaves a pull request able to land the defect on `main`.

### Method, so the scope can be checked rather than trusted

All 49 open pull requests were enumerated. For each one, the single test file at issue was fetched at that pull request's exact head commit and hashed:

```
android/app/src/test/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModelTest.kt
```

via `GET /repos/Sovereign-Communication/SCMessenger/contents/<path>?ref=<head sha>`. Result: **3 carriers, 46 clean, 0 absent.** The search was over every open pull request, not over the two previously known, so this scope is complete as of measurement.

### The three carriers

The test file on all three heads is **byte-identical** — `sha256:0c4aff33456757e1…` on each — with 9 `@Test` methods and `it.action` at lines 102 and 118.

| PR | Head branch | Head commit | Base | State | `Android JVM Unit Tests` |
|---|---|---|---|---|---|
| #359 | `glm/canonical-outlier-audit` | `3f41005d1a6a2b9b608dd3960640a642c92f792e` | `main` | `CONFLICTING/DIRTY` | **never run** — see below |
| #361 | `d9-libp2p-degrade` | `63f4a7d73ef05ace437b54d11c14a256c71eb923` | `main` | `MERGEABLE/BEHIND` | **fail**, 20m 41s, run `35787291582` |
| #364 | `freebuff/and-stopflood-outbox-sweep-040` | `45b0f8b8a8019f94b9c39105e45a84315ef314b7` | `main` | `CONFLICTING/DIRTY` | **fail**, 19m 38s, run `35894514034` |

For contrast, the same file on `main` is `sha256:7943c4f2e115de32…`, 7 `@Test`, no `it.action` — so `main` is clean and the defect lives only on these branches.

### An honest distinction between the two measured failures and the one inferred

- **#361** and **#364** have each had the Android gate run, and each failed with the identical signature: `353`/`357` tests completed, `2 failed, 3 skipped`, the same two test names (`toggle during STARTING|STOPPING`), and the same `Only one matching call to Context(#926)/Context(#965)` null-argument signature mismatch. That is measured, from CI logs.
- **#359 has never had the Android gate run.** Its head commit has exactly one workflow run — the CodeQL "PR #359" run `35836517373`, success, 6 check-runs, no Android job. So #359 is a carrier **by file identity, not by a measured failure**. Because its test file is byte-identical to the two that do fail, it is expected to fail the same way; that expectation is inference and is labelled as such.

### The two assertions, verbatim

From the shared carrier file (identical on all three heads):

- line 101-103 — `mockContext.startService(match {` / `it.action == MeshForegroundService.ACTION_STOP` / `})`
- line 117-119 — `mockContext.startForegroundService(match {` / `it.action == MeshForegroundService.ACTION_START` / `})`

The assertions sit in the two tests named `toggle during STARTING …` and `toggle during STOPPING …`.

### The corrected assertions already exist in this repository

PR **#372** (`fix/361-review-blockers`, head `2cb046cf8cab34c15327a4fbfbb90fb91ab212f4`) carries `e7466639` as an ancestor. Its test file is `sha256:e7769b6b2b39dd4a…` — 9 `@Test` methods and **zero** `it.action`. So the fix is already written and already reviewed in this repository; it is simply not on the two branches that need it, and not on `main`.

Two caveats stated plainly: #372 is `MERGEABLE/BLOCKED` and is not a route to `main` on its own, and at the time of writing its `Android JVM Unit Tests` job (run `36269937310`) was still **pending** — so the corrected file is verified here by content hash, not by a green gate.

### Root cause, restated against the current code (line numbers verified at `bceacb94`)

Unchanged from the 2026-09-25 handoff, and re-verified at this head:

- `android/app/build.gradle:232-234` — `unitTests { returnDefaultValues = true }`.
- `android/app/build.gradle:307-308` — the comment documenting that `android.jar` stubs no-op under `returnDefaultValues`, which is what makes a stubbed `Context.startService(intent)` drop the `intent`.
- `android/app/build.gradle:315-319` — records that Robolectric was removed on 2026-07-27; Robolectric is nowhere a dependency, so nothing restores real `Intent` objects.
- The **production** code is correct and is not the defect: `MeshServiceViewModel.kt:87-91` uses `startForegroundService` with `ACTION_START`, and `:106-109` uses `startService` with `ACTION_STOP`.

`returnDefaultValues = true` turns the mocked `Context` into a no-op, so `it` in the matcher is `null` and the assertion fails. Robolectric, or a test that asserts on the `Intent` that production actually builds, are the two ways out; the corrected version on #372 is the second.

### Requested owner actions

1. Treat the fix scope as **#359, #361 and #364** — not #361 alone. All three are `base=main`.
2. Apply the corrected test file (available on #372, or as commit `e7466639b59f6c07f19090ecd9867b39ac36b58e`, which applies cleanly onto the #361 head) to **each** of the three branches.
3. Re-run `Android JVM Unit Tests` on each of the three after the change. For #359 this is the first run that branch will ever have had.
4. Do not merge any of the three with the file unchanged. As it stands, resolving #359 or #364 and merging it puts the defect onto `main`.

---

## Amendment B — cargo dependency updates have been silently failing

This is new, and it is not a footnote: it is an outage of the automated-update path, and the outage is currently invisible to CI and to releases.

### Symptom

Workflow **Dependabot Updates**, run `36264033807`, `main` at `9e668682`, 2026-09-26 18:51Z, conclusion `failure`. This is the first failure since 2026-09-21.

Every cargo dependency fails with the same error, e.g. for `clap`:

```
Handled error whilst updating clap: dependency_file_not_resolvable {message:
"error: failed to load manifest for workspace member `dependabot_tmp_dir/wasm`
referenced by workspace at `dependabot_tmp_dir/Cargo.toml`

Caused by:
  failed to read `dependabot_tmp_dir/wasm/Cargo.toml`

Caused by:
  No such file or directory (os error 2)"}
```

The same error is recorded 82 times in that run — for `mockall`, `curve25519-dalek`, `camino`, `clap`, `windows`, `tokio-tungstenite`, `btleplug`, `tower`, `crc32fast`, `uniffi`, `chacha20poly1305`, `async-trait`, `argon2`, `axum`, `blake3`, `lz4_flex` and the rest — i.e. the entire cargo dependency set. Across the whole run there is exactly one error shape (`dependency_file_not_resolvable`, 82 occurrences) and no reference to the `github-actions` or `gradle` entries, so those two entries are not implicated.

`.github/dependabot.yml` configures cargo with `directory: "/"` and no `exclude-paths`; the job definition confirms `"package-manager":"cargo"`, `"directory":"/."`, `"exclude-paths":null`.

### Root cause — reproduced, not inferred

Root `Cargo.toml` lists `wasm` **twice**:

- line 2 — `members = ["core", "cli", "wasm", "mobile", "desktop_bridge"]`
- line 5 — `exclude = ["wasm"]`

On the cargo version the Dependabot log reports (1.98.1), `exclude` does **not** remove a path that is also named in `members`. Proof: with the tree exactly as on `main`, `cargo metadata --no-deps` reports `scmessenger-wasm` among the workspace packages. Cargo therefore still requires `wasm/Cargo.toml` to exist, and Dependabot's working directory did not contain it.

Local reproduction, byte-for-byte the same failure:

| Tree | `cargo metadata --no-deps` |
|---|---|
| `main` as-is, all five member directories present | exit 0 |
| `main` with `wasm/` removed (mimics the Dependabot directory) | **exit 101** — `failed to load manifest for workspace member …/wasm` / `failed to read …/wasm/Cargo.toml` |

### The fix: one line, from `members` — and *which* list was determined by test, not by reading

Both lists were edited in a scratch copy and the result measured:

- **Removing `"wasm"` from `members` (line 2)** → `cargo metadata --no-deps` at the workspace root **resolves, exit 0**.
- **Deleting the `exclude` line (line 5)** instead → **still exit 101**, with the identical `failed to read …/wasm/Cargo.toml` error.

So the line to remove is from **`members`**, not from `exclude`. `exclude` is inert for this failure, and removing it fixes nothing.

### Required follow-on: one removal is necessary but not sufficient

Removing `wasm` from `members` makes it a non-member, and `wasm/Cargo.toml` is written to inherit from the workspace: `version.workspace`, `edition.workspace`, `license.workspace`, and 13 dependencies declared as `{ workspace = true }`. It then fails to parse on its own:

```
error: failed to parse manifest at …/wasm/Cargo.toml
Caused by:
  error inheriting `edition` from workspace root manifest's `workspace.package.edition`
```

Adding an empty `[workspace]` table to `wasm/Cargo.toml` and spelling out the three package fields advances it to the next failure, `error inheriting 'anyhow' from workspace root manifest's 'workspace.dependencies.anyhow'`, and 13 `workspace = true` dependencies remain to be spelled out.

That matters because CI builds this package: `cargo build --target wasm32-unknown-unknown -p scmessenger-wasm --release` at `.github/workflows/cross.yml:143`, plus the `wasm32-unknown-unknown` steps at `.github/workflows/cross-platform-test.yml:179` and `.github/workflows/release.yml:357`. A change to `members` that is not accompanied by making `wasm/Cargo.toml` standalone will fix Dependabot and break those.

### What is not claimed

*Why* Dependabot omits `wasm/Cargo.toml` from its working directory is internal to Dependabot and cannot be verified from this repository. The double-listing is the strongest candidate trigger — a fetcher honouring `exclude` when choosing which member manifests to materialise — but that mechanism is inference and is not asserted here. The reproduction above does not depend on it: it establishes that any consumer which materialises the workspace without `wasm/Cargo.toml` fails, and that removing `wasm` from `members` is the change that makes the root workspace resolve without it.

### Requested owner actions

1. Remove `"wasm"` from `members` in root `Cargo.toml` line 2. Confirm locally with `cargo metadata --no-deps` (expect exit 0).
2. Make `wasm/Cargo.toml` standalone in the same change: an empty `[workspace]` table, literal `version`/`edition`/`license`, and the 13 `{ workspace = true }` dependencies spelled out.
3. Confirm `Cross` and `release` still build `scmessenger-wasm` for `wasm32-unknown-unknown`.
4. Re-run the Dependabot job and confirm the cargo entry succeeds.
5. Separately and cheaply: `Cargo.toml:3-4` comments that `wasm` is "excluded from default members". `exclude` has no effect on default members at all — that is `default-members` — so the comment is wrong on its own terms and is part of what made this misconfiguration look intentional.

---

## Severity and limits of this handoff

- **Severity of Amendment A**: high. Two `base=main` pull requests can currently land a red test suite, and a third has never been gated at all.
- **Severity of Amendment B**: medium-high. It does not block CI or releases; it means Rust dependency updates — including security updates — have not run for five days, and the failure is silent by construction.
- **Limit**: the Android gate cannot be executed locally; all Android findings rest on CI logs and on file content read at the exact head commits named above.
- **Limit**: no statement here should be read as a merge, an approval, or a sign-off. The owning repository decides what lands.
