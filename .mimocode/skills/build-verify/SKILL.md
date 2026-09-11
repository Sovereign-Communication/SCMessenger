---
name: build-verify
description: Run the SCMessenger build verification gates (cargo build/check/clippy/fmt/test-compile, and optionally Android/WASM). Use before considering any Rust or Android change done, or when asked to "verify the build" or "run the build gates".
argument-hint: "[full|rust|android|wasm|compile_gate]"
allowed-tools: Bash
---

Run the repo's existing build verification script, scoped by `$ARGUMENTS` (default to `full` if no argument was given):

```bash
bash .claude/skills/build_verify.sh $ARGUMENTS
```

On Windows hosts where `bash` is not on PATH, invoke it through Git Bash:
`"C:\Program Files\Git\bin\bash.exe" .claude/skills/build_verify.sh $ARGUMENTS`

Gate scopes:
- `full` — Rust (build/check/clippy/fmt/compile-gate) + Android (assembleDebug + RoleNavigationPolicyTest)
- `rust` — Rust gates only
- `android` — Android gates only
- `wasm` — `cargo build`/`check -p scmessenger-wasm --target wasm32-unknown-unknown`
- `compile_gate` — `cargo test --workspace --no-run` only

Report the PASS/FAIL summary line from the script's output. If any gate failed, quote its actual failing output (not just "FAIL") and do not report the calling task as complete until the failure is fixed or explicitly called out as a known/deferred issue.
