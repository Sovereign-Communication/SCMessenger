# GPT-MAC takeover packet -- PR #139 / v0.4.0 + v0.5.0

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**From:** GPT-Windows (Codex, 5.6 Luna lane)
**To:** GPT-MAC / MAC LANE
**Date:** 2026-08-07
**Repository:** `Sovereign-Communication/SCMessenger`
**PR:** #139, `tracking/pre-v040-tag-work`
**Current PR head:** `6cb7033a82e976a59e5630128656657bebff5d08` (FULL GREEN CI 7/7)

> **UPDATED 2026-08-08 (Windows Orchestrator Stand-Down):** The adversarial review is COMPLETE and returned BLOCK. See `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md` and the PR #139 comment for the full handoff. PR #139 must NOT merge until the five conditions in the review verdict are met. The P0 UPnP panic fix direction (options a/b/c) requires operator decision BEFORE any core/src/transport/ changes (merge-blocked by rule 8).

> **Head re-pointed 2026-08-08 (Windows Orchestrator lane).** Build from current PR head `6cb7033a`, which carries:
> 1. Preserved RFC 1918 IPv4 & IPv6 ULA LAN mesh convergence + 7-topology routing spec (`HANDOFF/plans/RFC1918_MESH_ROUTING_SPEC_2026-08-08.md`).
> 2. Complete IronCore Tier 1 security & error boundary remediations (`core/src/iron_core.rs`).
> 3. Full 7/7 green CI validation across all GitHub Actions workflows.
>
> **Dispatch:** Execute 5-node fleet release gate (Nodes 3 & 4: iOS Physical App + macOS CLI). Preserve existing identity/appdata.

## Current Windows state

- Hermes has finished its shared-worktree run.
- T1 dual-flavor block storage, T2 curve-point identity derivation, T3
  fail-closed pending requests, and T4 centralized block resolution are in
  the PR head.
- Windows gates passed after the handoff: `cargo fmt --all -- --check`, 21
  targeted blocked-manager tests, and 4 CLI message-request integration tests.
- The Windows checkout has a small local CLI request-key correctness delta in
  `cli/src/server.rs`; do not use or modify that local delta from the Mac lane.
  Fetch the PR head above into the Mac checkout, then wait for the Windows lane
  to publish the follow-up commit if the platform result depends on it.
- No GPT-MAC Codex task is visible to GPT-Windows on this host. This file is
  the coordination handoff; write the result below when the Mac lane finishes.

## MAC LANE scope

Use a separate Mac checkout/branch. Do not edit Rust core, merge, tag, or
rewrite the Windows worktree. The Mac lane owns platform verification and
evidence:

1. Build the macOS CLI from PR head `5b8b8e7b`; launch it using the existing
   data directory and record the actual version/git provenance line.
2. Build and install the iOS app in place from the same PR head. Do not
   uninstall, wipe identity data, delete contacts/history, or re-pair.
3. Confirm identity, contacts, and history survive the update; confirm the
   outbound identity is the canonical public-key form.
4. Where the fleet is available, run both message directions with receipts,
   ledger/fleet visibility, and the BLE/LAN liveness checks. Record actual
   transport and restart/reconnect evidence rather than inferred success.
5. Use only the current relay address from
   `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`; treat every older IP as stale.

Write the result to:

`HANDOFF/gpt/IOS_MACOS_PR139_STATUS_2026-08-07.md`

Use this result format:

```text
RESULT: DONE|BLOCKED|FAILED
HEAD: <actual commit built>
MACOS_CLI: <provenance and startup evidence>
IOS: <build/install/provenance/data-preservation evidence>
IDENTITY: <canonical identity evidence>
FLEET: <message/receipt/ledger/transport evidence or explicit N/A>
BLOCKERS: <exact external blocker, if any>
```

If a platform or physical-device step is unavailable, stop at that step and
report the exact blocker. Do not substitute a stale CI artifact or a source
inspection claim for runtime evidence.
