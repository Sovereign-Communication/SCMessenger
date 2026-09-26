# TASK: Fix should_use_ratcheted_encryption -- v2 peer first message wrongly errors instead of establishing a session

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Bug (confirmed by running the tests, not just compiling)

`core/src/crypto/encrypt.rs`, function `should_use_ratcheted_encryption`
(around line 403-452), the V2-peer-with-suite-0x02 branch:

```rust
Some(bundle) => {
    if bundle.supported_suites.contains(&0x02) {
        if session_exists {
            Ok(true)
        } else {
            // No session yet - must establish hybrid ratchet, not fall back to legacy
            Err(anyhow::anyhow!(
                "V2 peer {} requires hybrid ratchet session, cannot fall back to legacy static-ECDH",
                peer_id
            ))
        }
    } else { ... }
}
```

The comment says "must establish hybrid ratchet" but the code returns `Err`
instead of `Ok(true)`. This contradicts `encrypt_with_ratchet_fallback`'s own
`true`-branch, which is EXACTLY the code that establishes a session on
demand via `manager.get_or_create_session_hybrid(...)` when none exists yet.
The gating function currently never gives that branch a chance to run for a
brand-new v2 peer -- it errors out immediately instead.

Result: `cargo test -p scmessenger-core --test integration_pq_session` has
4 of 5 tests FAILING at runtime (they compile fine) with:
```
V2 peer <id> requires hybrid ratchet session, cannot fall back to legacy static-ECDH
```
This is the very first message to a fresh peer in each failing test --
exactly the case that should auto-establish a session, not error.

## Fix

Change the V2-peer/suite-0x02 branch to return `Ok(true)` unconditionally
(whether or not `session_exists`) -- session establishment vs. reuse is
already handled correctly by the caller (`encrypt_with_ratchet_fallback`'s
`true` branch, which calls `get_or_create_session_hybrid`). Remove the
`Err(...)` arm for this case entirely; there is no longer a legitimate
"error" outcome for a suite-0x02-capable peer, since the ratcheted path can
always establish on demand given a session_manager.

Do NOT change the "V2 peer, does NOT support suite 0x02" branch (the
`else` at line ~424) or the `None` (v1 peer) branch -- those are correct
per the task rules (v1-only-ratchet-if-session-exists,
error-if-require_pq-and-no-session).

Do NOT change `encrypt_with_ratchet_fallback` itself, `decrypt`-path code,
or `should_use_ratcheted_encryption`'s signature.

## Gate (run the actual tests, not just --no-run)

```
cargo test -p scmessenger-core --test integration_e2e --test integration_pq_session --test integration_ironcore_roundtrip
```
All must pass. Also re-run
`cargo test -p scmessenger-core --lib crypto::encrypt` (the branch-coverage
unit tests for `should_use_ratcheted_encryption` itself) and update/add a
unit test asserting: v2 peer, no session, require_pq=false -> `Ok(true)`
(not `Err`).

## Output format (MANDATORY)

Return the FULL updated contents of `core/src/crypto/encrypt.rs` in one
fenced code block with `// core/src/crypto/encrypt.rs` as the first line
inside the block.
