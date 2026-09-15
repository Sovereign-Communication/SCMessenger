# FB: Heartbeat watchdog integration test (P1 wedge closure, CI-gated)

Status: WITHDRAWN 2026-09-15 -- superseded by seat-direct implementation
(landed same session as the helper + probe + integration test; no operator
paste cycle needed for a change this size). Kept for the audit trail of why
the first draft was abandoned: the draft's central premise ("cli is a
binary-only crate, no lib target") was WRONG -- `cli/Cargo.toml` declares
`[lib] name = "scmessenger_cli"` with `src/lib.rs` re-exporting modules for
testing. Verified by reading `cli/Cargo.toml` and `cli/src/lib.rs` before
rewriting. Lesson reconfirmed: verify the premise end-to-end before writing
the task file, not after (docs/rules/FREEBUFF.md section 3).
