# P0_BUILD_003: Formal Verification Harness (Kani/Proptest)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 (Gatekeeper Blocker)
**Platform:** Core / CI
**Status:** Open
**Assignee:** Platform Engineer / Fast Executer

## Objective
Establish the formal verification harness for the SCMessenger Core. This is a prerequisite for the Gatekeeper's autonomous verification protocol.

## Tasks
1.  **Tool Installation**: Successfully install `cargo-kani` in the local environment.
2.  **Environment Setup**: Run `cargo kani setup` and resolve any missing system dependencies (e.g., minisat, bitwuzla).
3.  **Property-Based Baseline**: Verify that `proptest` is correctly pulling into the `core` build and run a baseline property test on `core/src/transport/address.rs`.
4.  **Kani Proof**: Implement a basic Kani proof for a non-trivial function in `core/src/crypto/` to demonstrate feasibility.

## Verification
- `cargo kani --version` returns a valid version.
- `cargo test --workspace` passes with `proptest` dependencies resolved.
- `cargo kani` run on `core` returns no "Failure" results for the new proofs.
