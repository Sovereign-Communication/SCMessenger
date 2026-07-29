# 040-S3 RELEASE MECHANICS (workflow prep agent, verified file:line)

All evidence gathered. Here is the report.

---

# TASK 040-S3 — Release Mechanics Analysis (read-only)

Scope verified: `.github/workflows/auto-tag-release.yml` (full), `.github/workflows/release.yml` (full), `Cargo.toml`, `android/build.gradle`, `android/app/build.gradle`, `CHANGELOG.md`, `scripts/sync_version.sh`, `scripts/generate_changelog.sh`, PR #115 verdict doc (`HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md`, fetched to `tmp/pr115_verdict.md` — scratch copy left in sanctioned `tmp/`), live CI run history, tag metadata. No files edited, no builds run.

## 1. The auto-tag trap — CONFIRMED

**Trigger** (`.github/workflows/auto-tag-release.yml:12-16`):
```yaml
on:
  push:
    branches: [main]
    paths:
      - 'Cargo.toml'
```
Any push to `main` that touches `Cargo.toml` fires it. There is **no** `workflow_dispatch` in the file today — manual capability does not yet exist and must be *added*, not merely "retained".

**Mechanism**:
- `auto-tag-release.yml:32` extracts `grep -m1 '^version' Cargo.toml` → matches `Cargo.toml:9` (`version = "0.3.5"` under `[workspace.package]`; all member crates use `version.workspace = true` per `core/Cargo.toml:3`, `cli/Cargo.toml:3`, etc.).
- `auto-tag-release.yml:42-46` checks whether `v<version>` exists (full-history checkout, `fetch-depth: 0`).
- If absent, `auto-tag-release.yml:48-55` creates annotated tag `v<version>` and `git push origin` it, under `permissions: contents: write` (line 18-19).

**The trap, exactly**: the moment the `0.3.5 -> 0.4.0` bump merges to main, this workflow extracts `0.4.0`, finds no `v0.4.0` tag (existing tags: only `v0.1.0 v0.1.1 v0.1.9 v0.2.1 v0.3.5`), and pushes **`v0.4.0`** — a *stable* tag, not the locked `v0.4.0-alpha.1`. Consequences:
- `release.yml:3-6` fires on `tags: v*` for `v0.4.0`.
- `release.yml:337` sets `prerelease: ${{ contains(github.ref, 'alpha') || ... }}` → `v0.4.0` → **`prerelease: false`** → a full stable GitHub Release pointing at the bump commit, not the reviewed terminal SHA.
- The operator's later manual `v0.4.0-alpha.1` would then produce a *second*, redundant prerelease.
- PR #115's verdict states the same finding (`tmp/pr115_verdict.md:29-33, 386-391`); §1.5 box (`tmp/pr115_verdict.md:153-159`) prescribes the chosen execution.
- Current state is *latent-safe only by accident*: today `Cargo.toml:9` is `0.3.5` and `v0.3.5` exists (created **manually** by Treystu, not the bot — verified via `git cat-file tag v0.3.5`), so recent auto-tag runs (2026-07-21, 2026-07-25, both 15 s) took the "tag already exists, skip" path (`auto-tag-release.yml:57-60`). A Cargo.toml-touching merge without the bump behaves identically. The trap arms the instant the bump lands.
- auto-tag is the **only** workflow that creates/pushes git tags (grep across `.github/workflows/`).

## 2. Exact edit list for the terminal release PR (defuse + keep manual)

File: `.github/workflows/auto-tag-release.yml`

| Lines | Current | Change |
|---|---|---|
| 12-16 | `on:` + `push:` + `branches: [main]` + `paths: ['Cargo.toml']` | Replace with `on:` + `workflow_dispatch:` and a comment: automatic main-push tagging removed in the v0.4.0 terminal PR per PR #115 §1.5; tags are operator-cut (`v0.4.0-alpha.1`); re-enable the commented-out `push` block only if automatic tagging is ever wanted again. Leave the old `push:` block commented out beneath for reversibility. |
| 3-10 | Header comment asserts "Tagging is now automatic: any push to main..." | Rewrite to describe the inert-manual state (docs-accuracy/hygiene requirement — the comment would otherwise lie). |
| 18-19 | `permissions: contents: write` | **Keep** — required for the manual job to push the tag. |
| 21-60 | job `auto-tag` (extract, check-exists, tag+push, skip) | **No changes needed** — the steps work unchanged under `workflow_dispatch`; manual dispatch tags whatever `[workspace.package] version` currently says, with the same exists-check guard. |

Result: the workflow can never fire automatically, but a one-click manual run still produces the workspace-version tag — the "inert manual definition" from §1.5.

Same terminal PR, non-workflow edits (the bump itself, §1.5: "set the source versions to 0.4.0"):
- `Cargo.toml:9` — `version = "0.3.5"` → `version = "0.4.0"`.
- `android/build.gradle:34` — `versionCode = 13` → advanced value (see §4).
- `android/build.gradle:35` — `versionName = '0.3.5'` → `versionName = '0.4.0'`.
- `CHANGELOG.md` — see §5.
- Optionally (recommended, see §3 hazards): `release.yml` Android prerequisite repair.

**Do NOT run `scripts/sync_version.sh` for the bump.** It is broken against the current layout:
- It targets `android/app/build.gradle`, where `versionName`/`versionCode` are *references* (`app/build.gradle:90-91`: `versionCode = rootProject.ext.versionCode`); the literals live in `android/build.gradle:34-35`.
- `sync_version.sh:47` expects double-quoted Groovy (`versionName "..."`) — no match, but the script still prints "Updated" (false success).
- `sync_version.sh:55` (`sed "s/versionCode [0-9]*/versionCode $VERSION_CODE/"`) matches `versionCode ` + zero digits on `app/build.gradle:90` and would rewrite it to `versionCode 400= rootProject.ext.versionCode` — a **syntax-corrupting edit**. Its iOS target (`iOS/SCMessenger/Info.plist`, line 68) and `wasm/package.json` (line 91) paths don't exist either (actual plist: `iOS/SCMessenger/SCMessenger/Info.plist`; no `wasm/package.json`), so those branches silently no-op.

## 3. release.yml chain for a `v0.4.0-alpha.1` tag

**Trigger** (`release.yml:3-7`): `push: tags: v*` matches `v0.4.0-alpha.1` (also `workflow_dispatch`). Tag-triggered runs check out the workflow **from the tag's commit** — any fix must be merged before the tag is cut.

**Jobs** (all parallel, `fail-fast: false` only inside the CLI matrix):

| Job | Runner | Produces | Notes |
|---|---|---|---|
| `build-cli` x4 (`release.yml:14-81`) | ubuntu-latest, macos-14 x2, windows-latest | `scm-linux-amd64`, `scm-macos-amd64`, `scm-macos-arm64`, `scm-windows-amd64.exe` + per-asset `.sha256` (`release.yml:66-73`, `shasum` on macOS / `sha256sum` elsewhere, `shell: bash`) | Linux installs `pkg-config libdbus-1-dev libssl-dev` (`:46-50`). Builds `--bin scmessenger-cli` (`:53`). |
| `build-android` (`:84-155`) | ubuntu-latest | Always: **debug APK** artifact `android-debug-apk` (`:99-110`, `./gradlew assembleDebug` with `ANDROID_NDK_HOME` from `nttld/setup-ndk@v1` r26b). **Signed AAB + APK** (`bundleRelease assembleRelease`, `:129-140`, artifact `android-release-signed`) **only if** repo secrets `SCMESSENGER_KEYSTORE_BASE64/_PASSWORD/_ALIAS/_PASSWORD` exist (`:122-127`, env-driven signing path `android/app/build.gradle:49-59`). Without secrets → no signed release artifact, release ships the debug-signed APK. | NDK pinned `26.1.10909125` (`android/app/build.gradle:40`); ABIs built: `arm64-v8a, armeabi-v7a, x86_64` (`app/build.gradle:100`, `:357-361`) — `i686` not built in CI. |
| `build-ios` (`:158-221`) | macos-14 | **Nothing released** — Debug build + simulator tests only; IPA/signing commented out (`:192-221`); explicitly excluded from `create-release.needs` (`:289-295`). Failure is visible but non-blocking. |
| `build-wasm` (`:224-286`) | ubuntu-latest | `wasm/pkg/*` artifact (`wasm-pack build --target web --release`, `wasm-opt -Oz` binaryen 116, plus a UMD wrapper). |
| `create-release` (`:293-337`) | ubuntu-latest | GitHub Release via `softprops/action-gh-release@v2` | `needs: [build-cli, build-android, build-wasm]` (`:295`) — **any one failure skips the release**. Downloads all artifacts (`:304-305`); runs `scripts/generate_changelog.sh` (`:307-310`) which overwrites the CI workspace's `CHANGELOG.md` with a conventional-commits log over `v0.3.5..v0.4.0-alpha.1` (`generate_changelog.sh:20-36,44`) and is used as release body (`body_path: CHANGELOG.md`, `:335`) — so GitHub release notes do **not** come from the maintained `CHANGELOG.md`. Generates combined `SHA256SUMS.txt` over `scm-*`, `*.apk`, `*.aab`, `*.ipa`, `*.wasm` (`:312-321`). `prerelease: contains(github.ref,'alpha'||'beta'||'rc')` (`:337`) → **`v0.4.0-alpha.1` = prerelease: true**, which is exactly the desired behavior. |

**Prerequisite hazards — proven, not speculative.** The last tag run (v0.3.5, run `29675248520`, 2026-07-19) **failed all 7 build jobs; `create-release` skipped; no v0.3.5 GitHub release exists** (`gh release list` shows latest = v0.1.9). Two macOS/Windows failure modes (`sha256sum` missing; PowerShell parser error) are already fixed on main (`release.yml:67,69-73`). Two Android hazards remain in current main:
1. `release.yml`'s `build-android` has **no `cargo install cargo-ndk` step** — both `mobile.yml:26-27` and `cross.yml:35-36` install it explicitly; `android/app/build.gradle:336-350` invokes `cargo ndk`, which is not on GitHub's ubuntu image.
2. In the v0.3.5 run the job died with `Execution failed for task ':app:buildRustAndroidArm64' > NDK is not installed` — the `ANDROID_NDK_HOME` env from `setup-ndk` is not reaching AGP's NDK resolution (gradle checks `local.properties ndk.dir` / `android.ndkDirectory` first, `app/build.gradle:372-379`).

Unless these are fixed in-tree **before** the tag, `build-android` fails and `create-release` never runs — no artifacts at all. The iOS job's `xcode-select -s /Applications/Xcode_15.0.app` (`release.yml:165`) also failed in that run (exit 2; image drift) — non-blocking but will show red.

## 4. Android versionCode advancement

- Surface: `android/build.gradle:34-35` (`versionCode = 13`, `versionName = '0.3.5'`), consumed via `rootProject.ext` at `android/app/build.gradle:90-91`.
- `scripts/sync_version.sh:31-34` documents the intended formula `MAJOR*10000 + MINOR*100 + PATCH`, which for 0.3.5 yields **305 — but the repo sits at 13**, so the repo and the script diverged long ago (manual counter in practice).
- Requirement (§1.5, `tmp/pr115_verdict.md:146-148`): versionCode must be **monotonically greater than 13**, and both clean-install and upgrade-install must be smoke-tested. Any value >13 qualifies: `14` (continues the manual counter) or `400` (adopts the formula for 0.4.0, which is also >13 and future-proof). The PR must pick one and record the choice; the formula is preferable for self-consistency but either is legal. versionName must become `'0.4.0'` to match `Cargo.toml:9`.

## 5. CHANGELOG conventions found

- `CHANGELOG.md:5-6` — Keep a Changelog 1.1.0 + SemVer.
- Structure: `## [Unreleased]` (`:11`) with `### Added/Changed/Removed/Fixed` subsections; released entries as `## [0.3.5] - 2026-07-11` (`:44`); pre-0.3.5 history intentionally not itemized (`:8-9`).
- Compare-link footers: `[Unreleased]: .../compare/v0.3.5...HEAD` and `[0.3.5]: .../compare/v0.2.1...v0.3.5` (`:98-99`).
- Terminal-PR implications: promote current `## [Unreleased]` content to a dated `## [0.4.0]` section (source version is 0.4.0 even though the tag is `v0.4.0-alpha.1`), open a fresh empty `## [Unreleased]`, add `[0.4.0]: .../compare/v0.3.5...v0.4.0-alpha.1`, and repoint the Unreleased link to `v0.4.0-alpha.1...HEAD`. Entries must "describe what was actually proven and list exclusions without claiming iOS or farm readiness" (§1.5).
- Caveat 1: the GitHub Release body is regenerated by `scripts/generate_changelog.sh` (`release.yml:307-310`), so the maintained file governs repo/documentation truth, not the release page.
- Caveat 2: `generate_changelog.sh` itself emits emoji section titles (`:185, :200-211`) — pre-existing violation of the repo no-emoji rule; it only affects CI-generated release bodies, and fixing it is out of 040-S3 scope but worth a follow-up ticket.

## 6. Ordered release-PR checklist

1. **Gates**: terminal S2 adversarial verdict is SHIP covering every named finding; S3 (fmt/clippy/build/test/Android/FFI at one SHA), S4 (CLI-to-emulator both directions), S5 (literal Josh WAN proof) all green; working tree clean; required GitHub checks green without rerun (§1.5 boxes 1-8).
2. **Open the terminal PR** containing exactly: the `auto-tag-release.yml` defuse (§2); `Cargo.toml:9` → `0.4.0`; `android/build.gradle:34` versionCode >13 + `:35` versionName `'0.4.0'`; CHANGELOG promotion/links (§5). Do **not** run `sync_version.sh` (§2 corruption).
3. **Same PR (strongly recommended)**: repair `release.yml` Android prerequisites — add `cargo install cargo-ndk` to `build-android` (mirror `mobile.yml:26-27`) and fix NDK path resolution so `setup-ndk`'s NDK is found by `app/build.gradle:372-379` (e.g. write `local.properties ndk.dir` or point `ANDROID_NDK_HOME` before the first gradle invocation). Without this, `create-release` will skip (§3).
4. **Merge order discipline**: this PR is the *last* main-push touching `Cargo.toml`; since defuse and bump merge atomically, no window exists where the live auto-tag could see `0.4.0` without the trigger removal.
5. **Operator** (not Mac lane, not automation): on the merge SHA, `git tag -a v0.4.0-alpha.1 -m "..."` then `git push origin v0.4.0-alpha.1`.
6. **Watch the release run**: `build-cli` (x4) + `build-android` + `build-wasm` must pass; `create-release` then publishes a GitHub **prerelease** (`release.yml:337`, `'alpha'` match) with `scm-*` CLI binaries, APK (debug-signed unless `SCMESSENGER_KEYSTORE_*` secrets are set → signed AAB+APK), `wasm/pkg/*`, and `SHA256SUMS.txt`. iOS job red is non-blocking (`release.yml:289-295`).
7. **Post-release smoke**: download CLI + APK from the release, verify SHA256SUMS, install both, smoke-test (clean + upgrade) — §1.5 box 11.
8. **Hygiene follow-ups (not blocking)**: fix `generate_changelog.sh` emoji titles; decide whether the eventual stable tag is `v0.4.0` (auto-tag now inert, so it too becomes a manual operator cut) or a later version.

---

Sources read: `.github/workflows/auto-tag-release.yml:1-60`, `.github/workflows/release.yml:1-337`, `.github/workflows/mobile.yml:26-36`, `.github/workflows/cross.yml:35-45`, `Cargo.toml:1-9`, `android/build.gradle:1-50`, `android/app/build.gradle:34-101,291-405`, `CHANGELOG.md:1-99`, `scripts/sync_version.sh:1-203`, `scripts/generate_changelog.sh:1-267`, PR #115 body + `HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md` (§1.5, lines 129-161), `gh run view 29675248520`, `gh release list`, `git cat-file tag v0.3.5`.
