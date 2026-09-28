# TASK: Version bump 0.3.4 -> 0.3.5 across all version references

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

The workspace has accumulated substantial work since the version was last
set (PQC-01 through PQC-08, orchestration unification, docs cleanup,
compile-gate fix restoring `cargo test --workspace --no-run`) without a
version bump. Update every version reference below to `0.3.5`.

## Files and exact changes

1. `Cargo.toml` (repo root) line ~9: `version = "0.3.4"` -> `version = "0.3.5"`.
   (This is the workspace version; `core/Cargo.toml` and `cli/Cargo.toml`
   use `version.workspace = true` and inherit it automatically -- do NOT
   edit those two files.)

2. `android/build.gradle` line ~34-35:
   ```
   versionCode = 12
   versionName = '0.3.4'
   ```
   ->
   ```
   versionCode = 13
   versionName = '0.3.5'
   ```

3. `CLAUDE.md` line ~15: `**Active release line:** v0.3.4, working toward
   v1.0.0.` -> `**Active release line:** v0.3.5, working toward v1.0.0.`

4. `README.md` line ~5: `**Version**: v0.3.4 (alpha, driving to v1.0.0)` ->
   `**Version**: v0.3.5 (alpha, driving to v1.0.0)`. Also update the
   `**Last updated**` line just above it to today's date, 2026-07-11.

5. `CHANGELOG.md`: insert a new section directly below `## [Unreleased]`'s
   existing content (i.e. between the end of the current [Unreleased]
   section and the `## [1.0.0-rc2]` heading), reading:

   ```
   ## [0.3.5] — 2026-07-11

   ### Added
   - Post-quantum hybrid migration (PQC-01 through PQC-08): ML-KEM-768
     primitives, hybrid X25519+ML-KEM-768 session establishment, suite
     negotiation (0x01 legacy / 0x02 hybrid), PQ-augmented double ratchet,
     legacy static-ECDH retirement gating with audit logging.
   - `docs/ORCHESTRATION.md`: unified cross-mode orchestration protocol
     (state machine, dispatcher, tier routing, commit authority) covering
     native Claude, Qwen/DashScope, OpenRouter, agy/Gemini, and Ollama lanes.
   - `scripts/delegate_task.py`: `--verify`/`--max-rounds` auto-fix loop and
     `--mode diff` unified-diff support, reducing compile-fix round trips.

   ### Fixed
   - Restored the `cargo test --workspace --no-run` compile gate: fixed a
     UniFFI enum/UDL mismatch (`LegacyStaticEcdhSend`), 41 stale-struct-shape
     errors in `core/src/crypto/{encrypt,ratchet}.rs` unit tests, a
     production bug where legacy-ECDH audit events recorded the peer under
     the wrong field, and a test bug where a hybrid-ratchet receiver test
     decapsulated a mismatched ciphertext.
   - iOS CI workflow (`ios-build-test.yml`): removed failure-masking
     (`xcpretty || true`), fixed lowercase path references, added a Swift
     bindings drift gate.

   ### Changed
   - Repository hygiene: archived 25 stale/superseded docs to
     `docs/historical/`, rewrote `README.md` and GitHub repo metadata for
     accuracy, groomed `HANDOFF/todo/` to live tasks only.
   ```

## Do NOT

- Do not touch `core/Cargo.toml`, `cli/Cargo.toml`, or any other
  `Cargo.toml` that uses `version.workspace = true`.
- Do not touch `iOS/**/project.pbxproj` `MARKETING_VERSION` (iOS uses an
  independent versioning scheme -- out of scope).
- Do not modify the `## [1.0.0-rc2]` section or anything below it in
  CHANGELOG.md.
- Do not add, remove, or reformat any [Unreleased] content already present
  above your insertion point.

## Gate

No build gate applies (pure version-string/doc changes). Verify with:
```
grep -rn "0.3.4" Cargo.toml android/build.gradle CLAUDE.md README.md
```
This should return NOTHING after your edit (all four bumped to 0.3.5).

## Output format (MANDATORY)

Return the FULL updated contents of all five files, each in its own fenced
code block, filename as the first line inside the block:
`// Cargo.toml`, `// android/build.gradle`, `// CLAUDE.md`, `// README.md`,
`// CHANGELOG.md`.
