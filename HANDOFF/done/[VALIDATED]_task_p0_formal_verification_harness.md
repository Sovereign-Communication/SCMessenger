# MODEL: deepseek-v4-pro:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# FALLBACK: glm-5.1:cloud
# BUDGET: 3600
# TARGET: core/src/crypto/, core/src/transport/
# NOTE: Retriaged from [FAILED]_  switched from deepseek-v3.2:cloud. cargo-kani must be installed first.

## P0: Formal Verification Harness (Kani + Proptest Baseline)

**Source:** HANDOFF/backlog/IN_PROGRESS_P0_BUILD_003_Formal_Verification_Harness.md

### Objective
Establish the formal verification harness for SCMessenger Core. Required by security rules: all crypto module changes must pass existing Kani proofs.

### Tasks
1. Install `cargo-kani` if not already available; run `cargo kani setup`
2. Verify existing proptest harness in `core/src/crypto/proptest_harness.rs` compiles and runs under `kani-proofs` feature
3. Run baseline Kani verification on existing proofs in `core/src/crypto/`
4. Add Kani proof for a non-trivial function in `core/src/crypto/` if coverage gaps exist
5. Document the verification workflow

### Verification
- `cargo kani --version` returns valid version
- `cargo test -p scmessenger-core --features kani-proofs` passes
- `cargo kani` on core returns no failures
