# Orchestration Control Plane v2 draft PR description

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Draft artifact; remote draft PR creation is owned by the coordinator
Last updated: 2026-08-10

## Objective

Introduce repo-owned, tool-agnostic delegation enforcement: canonical protocol
version 2.0.0, validated manifest, hardened common kernel, thin adapters,
isolated writers, durable lifecycle state, and independent review routing.

## Compatibility matrix

| Entry point | Protocol | Isolation | Role mapping | Direct controller source edits |
| --- | --- | --- | --- | --- |
| Claude | v2 manifest + strict kernel | Native or worktree fallback | Manifest semantic roles | Blocked |
| Codex | v2 manifest + strict kernel | Agent/worktree | GPT-5.6 capability map with runtime fallback | Blocked |
| Qwen | v2 manifest + strict kernel | Native or worktree fallback | Provider capability map | Blocked |
| Gemini | v2 manifest + strict kernel | Worktree fallback | Provider capability map | Blocked |
| Bob | v2 manifest + strict kernel | Native or worktree fallback | Provider capability map | Blocked |
| OpenCode | v2 manifest + strict kernel | Native or worktree fallback | Provider capability map | Blocked |
| `.agents` | v2 manifest + strict kernel | Worktree fallback | Semantic roles | Blocked |
| Direct scripts | v2 manifest + strict kernel | Worktree manager | Actual provider/model provenance | Blocked |

## Implementation checklist

- [x] Canonical human protocol and versioned manifest
- [x] Validated contract and deterministic capability guard
- [x] Fail-closed parser, build-lock composition, actual provenance
- [x] Durable state and isolated writer worktree manager
- [x] Thin frontend adapters and portable bootstrap
- [x] Negative contract/guard/parser/isolation/resume evals, including assigned-review provenance binding
- [ ] Live-provider reviewer dispatch and independently isolated response validation
- [ ] Coordinator creates/updates remote draft PR when credentials permit

## Known limitations

The current Codex installation exposes Terra and Sol, not Luna. The manifest
keeps Luna as the preferred capability and declares Terra fallback; model
availability never changes role authority. Authoritative Windows/mobile gates
remain host-owned and are not asserted by these local tooling evals.

The deterministic reviewer fixture records a synthetic assignment only to test
the durable schema and fail-closed admission path; it does not claim any live
provider ran. A real provider must still produce a recorded dispatch reference,
independent reviewer isolation identity, and canonical footer before this work
can be represented as live-validated or complete.
