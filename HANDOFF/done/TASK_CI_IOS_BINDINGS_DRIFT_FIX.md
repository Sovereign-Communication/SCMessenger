# TASK: Fix scripts/verify_ios_bindings.sh (broken --stdout flag + wrong ordering)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

A prior pass rewrote `scripts/verify_ios_bindings.sh` to check Swift-bindings
drift, but it invokes a flag that does not exist and would fail on every run.
Ground truth below is confirmed by reading `core/src/bin/gen_swift.rs` and
the known-working `scripts/build_xcframework.sh`.

## Facts (verified, do not re-derive)

- The binary is `gen_swift` (`core/Cargo.toml` `[[bin]] name = "gen_swift"`,
  `path = "src/bin/gen_swift.rs"`). It has **no `--stdout` flag** -- it always
  writes to the fixed directory `core/target/generated-sources/uniffi/swift/`.
- `gen_swift` requires a **host-native build of `scmessenger-mobile`** to
  already exist (it searches for `libscmessenger_core.{so,dylib}` under
  `target/<host-triple>/{debug,release}/` and `target/{debug,release}/`; its
  own panic message says `"Please run: cargo build -p scmessenger-mobile"`).
  It does NOT look at iOS cross-compiled `.a` static libs at all -- building
  for `aarch64-apple-ios`/`-sim` does NOT satisfy this requirement.
- The correct command sequence (mirrors `scripts/build_xcframework.sh:22-27`):
  ```bash
  cargo build -p scmessenger-mobile
  cargo run --bin gen_swift --features gen-bindings
  ```
  This writes `core/target/generated-sources/uniffi/swift/SCMessengerCore.swift`.
- The committed file to diff against is
  `iOS/SCMessenger/SCMessenger/Generated/api.swift` (confirmed to exist at
  this exact path).

## Fix 1: scripts/verify_ios_bindings.sh

Rewrite the generation step to:
```bash
cargo build -p scmessenger-mobile
cargo run --bin gen_swift --features gen-bindings
```
then diff `core/target/generated-sources/uniffi/swift/SCMessengerCore.swift`
against `iOS/SCMessenger/SCMessenger/Generated/api.swift` directly (no
`--stdout`, no temp-file redirect of a nonexistent flag). Keep `set -e`,
keep the clear pass/fail echo messages, no emoji.

## Fix 2: .github/workflows/ios-build-test.yml step ordering

The step "Verify Swift bindings are up to date" currently runs BEFORE the
Rust core is built for any target. Since the script above now needs
`scmessenger-mobile` built first, move this step to run immediately AFTER
whichever Rust-build step exists in the `build-ios` job (the steps named
"Build Rust core for iOS device" / "Build Rust core for iOS Simulator", or
equivalent) -- i.e. build the core first, then run the drift check, then
proceed to XCFramework/test steps. Do not add a redundant
`cargo build -p scmessenger-mobile` in the workflow if an equivalent build
already runs earlier in the same job; only add it if no prior step in the
job builds `scmessenger-mobile` for the host triple.

## Do NOT

- Do not change `core/src/bin/gen_swift.rs`.
- Do not change any job name, trigger, or unrelated step in the workflow.
- Do not reintroduce `|| true` or any other failure-masking.

## Gate

No local build gate available (macOS-only tooling; GitHub Actions billing is
currently locked). Verification is a careful manual read: confirm the shell
script has no reference to `--stdout`, confirm the workflow step order has
the bindings-check step after the core build step, confirm no emoji.

## Output format (MANDATORY)

Return the FULL updated contents of BOTH files, each in its own fenced code
block, with the filename as the first line inside the block:
`// scripts/verify_ios_bindings.sh` and
`// .github/workflows/ios-build-test.yml`.
