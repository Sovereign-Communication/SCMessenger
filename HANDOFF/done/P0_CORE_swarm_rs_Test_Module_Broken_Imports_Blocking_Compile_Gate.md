# TASK: P0-CORE-SWARM-TEST-IMPORTS  `core/src/transport/swarm.rs` test module fails to compile (5 errors), blocks `cargo test --workspace --no-run`

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Source

Found by ground-truth `cargo test --workspace --no-run` run in
`HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md` (2026-07-04, commit
`fdd315f3e73eea053109776f910bafd18dfafaa6`). Real rustc output, not a guess.

## Problem (exact, verified)

`core/src/transport/swarm.rs`'s `#[cfg(test)] mod tests` block
(starts line 5340) fails with 5 errors:

```
error[E0433]: cannot find type `RegistrationMessage` in this scope
    --> core\src\transport\swarm.rs:5541:55
     |
5541 |             verify_registration_message(&wrong_peer, &RegistrationMessage::Register(request)),
     |                                                       ^^^^^^^^^^^^^^^^^^^ use of undeclared type `RegistrationMessage`
     |
help: consider importing this enum through its public re-export
     |
5342 +     use crate::transport::RegistrationMessage;
     |

error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5548:19
     |
5548 |         let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9101".parse().unwrap();
     |                   ^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5554:19
     |
5554 |         let addr: Multiaddr = "/ip4/0.0.0.0/udp/9876".parse().unwrap();
     |                   ^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `Multiaddr` in this scope
    --> core\src\transport\swarm.rs:5560:19
     |
5560 |         let addr: Multiaddr = "/dns4/example.com/tcp/443".parse().unwrap();
     |                   ^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `Libp2pPeerId` in this scope
    --> core\src\transport\swarm.rs:5574:23
     |
5574 |             let key = Libp2pPeerId::random();
     |                       ^^^^^^^^^^^^ use of undeclared type `Libp2pPeerId`

error: could not compile `scmessenger-core` (lib test) due to 5 previous errors
```

Full log: `tmp/work_files/compile_gate/test_no_run.log`.

## Root Cause Investigation (already done  do not re-derive)

The test module's import block is at `swarm.rs:5342-5348`:
```rust
use super::{
    extract_ed25519_public_key_from_peer_id, should_apply_delivery_convergence_marker,
    validate_delivery_convergence_marker_shape, verify_registration_message,
    DeliveryConvergenceMarker, PendingCustodyDispatch, PendingMessage, RelayAbuseGuardrails,
    RELAY_DUPLICATE_WINDOW_MS, RELAY_PEER_BUCKET_BURST_CAPACITY,
    RELAY_PEER_BUCKET_REFILL_PER_SEC,
};
use crate::identity::IdentityKeys;
use crate::store::relay_custody::RelayCustodyStore;
use std::collections::HashMap;
```

1. **`RegistrationMessage`**  not imported. Confirmed exported at
   `core/src/transport/mod.rs:32` (`... RegistrationMessage, ...`). Fix:
   add `use crate::transport::RegistrationMessage;` (or add it to the
   existing `use super::{...}` list if `swarm.rs` re-exports it  check
   which is more consistent with the rest of the file before picking).

2. **`Multiaddr`**  `swarm.rs:49` has `use libp2p::{..., Multiaddr,
   PeerId};` at the *outer* module scope, but `mod tests` only imports
   specific names via `use super::{...}`, not `use super::*`, so the outer
   `use` isn't visible inside the test module. Fix: add `use
   libp2p::Multiaddr;` inside the test module's import block.

3. **`Libp2pPeerId`**  **investigated, not a simple missing-import case.**
   Grepped the entire `core/src` tree for `Libp2pPeerId`  it appears
   **exactly once**, at `swarm.rs:5574`, and is not defined, aliased, or
   imported anywhere else in the crate. The real libp2p type is imported as
   plain `PeerId` at `swarm.rs:49`. Two possible fixes, and the implementer
   must determine which is correct by reading the test's intent (context:
   `identify_log_dedup_suppresses_within_ttl`, lines ~5566-5578, testing a
   dedup map keyed by peer ID):
   - (a) the test meant `PeerId` (already imported outer-scope) and should
     use `use libp2p::PeerId;` inside the test module  simplest, most
     likely correct if no other file in this codebase uses a
     `Libp2pPeerId` alias convention; OR
   - (b) if other transport code elsewhere in the codebase has a
     documented convention of aliasing `libp2p::PeerId` as `Libp2pPeerId`
     (e.g. to disambiguate from an internal `PeerId`-shaped type), that
     alias needs to be defined and imported instead. Grep for any other
     `XxxPeerId` alias pattern in `core/src/transport/` before assuming (a).
   Do not guess blindly  actually check both possibilities before editing.

## Blast Radius

`#[cfg(test)]`-only, confined to `core/src/transport/swarm.rs`. Does not
affect `cargo build -p scmessenger-core`, `scmessenger-cli`,
`scmessenger-mobile`, or the Android Gradle/cargo-ndk cross-compile path 
none of those compile core's test module. **Not a blocker for the current
Windows/Android parity priority.** Blocks the mandatory `cargo test
--workspace --no-run` compile gate only.

## Adversarial Review Requirement

This touches `core/src/transport/` test code only (not the `swarm.rs`
production code paths). Per `.claude/rules/security.md`, the Adversarial
Review Protocol applies to changes in `core/src/transport/` broadly  since
this is test-only import fixes with no production logic change, a full
`crypto-security-auditor` pass is likely unnecessary, but flag this to the
implementer to make the call explicitly rather than skip silently if the
fix ends up touching anything beyond imports.

## Files to Touch

- `core/src/transport/swarm.rs` (test module import block, lines
  ~5340-5352 and whichever exact line the `Libp2pPeerId` fix lands on)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test --workspace --no-run
cargo test -p scmessenger-core --lib
```

## Do NOT

- Do not delete or `#[ignore]` these 3 tests to "fix" the compile gate 
  they test real registration/multiaddr/peer-dedup logic; the fix is
  correcting imports/aliasing, not removing coverage.
- Do not guess the `Libp2pPeerId` fix without checking for an existing
  alias convention elsewhere in `core/src/transport/` first (see Root
  Cause Investigation, point 3).
