# TASK: Pre-existing clippy -D warnings debt in scmessenger-cli + desktop-bridge

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2 (build-gate hygiene; NOT a Phase 1 Windows/Android blocker)
**Discovered:** 2026-07-07, when NEXT_ITER_01 fixed core's dead-code and the
`cargo clippy --workspace -- -D warnings` gate advanced past core to surface
long-accumulated debt (the -D warnings gate had not been enforced in months).
**Recommended lane:** agy-Gemini for the mechanical parts; the unwrap policy
calls may need agy-Claude/judgment.

## Findings (pre-existing, unrelated to the Fable 5 sprint)

### scmessenger-cli (lib) - 8 errors
- 7x `use of a disallowed method Result::unwrap` (repo clippy.toml disallows unwrap).
- 1x `Iterator::last on a DoubleEndedIterator` (use `.next_back()`).
Locations: run `cargo clippy -p scmessenger-cli -- -D warnings` for exact file:line.
NOTE: none are in the F6 edit (cli/src/ble_mesh.rs:332).

### scmessenger-desktop-bridge (lib) - 19 errors
- 3x `unused variable: title/body/urgency` in `send_notification` - the fn body
  is `#[cfg(target_os = "linux")]`-only, so the params are unused on Windows/macOS.
  Fix: `let _ = (&title, &body, &urgency);` on non-linux, or cfg the signature.
- 16x `use of a disallowed method Result::unwrap` in desktop_bridge/src/desktop_bridge.rs.

## Fix guidance
- unused params: cfg-gate or bind to `_` on non-linux.
- unwrap: replace with proper error propagation (`?` / map_err) where the fn
  returns Result; use `.expect("...")` ONLY if expect is allowed by clippy.toml
  (check); OR, if this is intentionally-panicking desktop init code, add a
  scoped `#[allow(clippy::disallowed_methods)]` with a justifying comment.
  Decide per-site; do not blanket-allow.

## Acceptance
- `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` passes.
- No behavior change (desktop_bridge is the Linux desktop client - not on the
  Windows CLI / Android path).

## GATE
`cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`

---
**Exact unwrap() locations (ollama gpt-oss:120b, 2026-07-07):** desktop_bridge/src/desktop_bridge.rs lines 81, 87, 90, 95, 100, 109, 118, 143, 154, 156, 167, 173, 175, 183, 185, 205.
