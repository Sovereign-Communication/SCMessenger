# TASK: F1 — Ledger Convergence Test Fix (Multiaddr Dial Issue)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: READY FOR QWEN DELEGATION
Owner: Qwen (technical fix)
Scope: F0 delivery-truth (critical for A3 landing)

## Objective

Fix `core/tests/integration_ledger_convergence.rs` multiaddr dial failure. Test exists, compiles clean, but fails at runtime with "no addresses for peer".

## Root Cause

Multiaddr dial is missing `/p2p/<peer_id>` suffix. The peer is discovered but not addressable because the dialed multiaddr doesn't include the peer identity component.

## Current Failure

```
thread 'test_ledger_convergence' panicked at 'swarm2.dial(node1_addr) failed: no addresses for peer'
```

## Fix Required

1. **Inspect the test file:**
   - File: `core/tests/integration_ledger_convergence.rs`
   - Locate the `swarm.dial()` call site
   - Current multiaddr: likely `/ip4/<addr>/tcp/<port>` (missing peer component)

2. **Add peer identity to dialed multiaddr:**
   ```rust
   // Before:
   let multiaddr = format!("/ip4/{}/tcp/{}", addr, port);
   
   // After:
   let multiaddr = format!("/ip4/{}/tcp/{}/p2p/{}", addr, port, peer_id);
   ```
   - `peer_id` is the discovered peer's libp2p peer ID (as hex or base58)
   - Verify format matches: `/p2p/` prefix + peer ID

3. **Re-run and capture output:**
   - Command: `cargo test --test integration_ledger_convergence -- --nocapture`
   - Expected: test passes, multiaddr resolution succeeds, ledger convergence logs show delivery path
   - Save output to HANDOFF work_files/

4. **Verification:**
   - `cargo test --workspace --no-run` (compile gate)
   - All tests in `integration_ledger_convergence.rs` pass

## Acceptance Criteria

- [ ] Multiaddr dial succeeds (peer found at `/p2p/<id>`)
- [ ] Test `test_ledger_convergence` passes
- [ ] Ledger convergence verified (delivery path in logs)
- [ ] Compile gate passes
- [ ] Commit: `fix: F1 ledger convergence multiaddr dial (add /p2p/<peer_id> suffix)`

## Blocking/Blocked

**Blocked by:** None
**Blocks:** A3 (Android retry suppression)

## Output Location

Save test run output to: `HANDOFF/work_files/F1_ledger_test_run.log`

## 2026-07-28 closure (orchestrator takeover audit)
Moved to done/ on authority of _QUEUE.md header (DONE 2026-07-23: file committed, p2p addr suffix bug fixed). Evidence on disk: core/tests/integration_ledger_convergence.rs (updated 2026-07-26).
