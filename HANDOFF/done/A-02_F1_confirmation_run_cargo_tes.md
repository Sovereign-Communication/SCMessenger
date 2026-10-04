# Task A-02

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Description
F1 confirmation run: cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored; fix any failure; either un-ignore or document runner requirements

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- core/src/store/outbox.rs
- cli/src/main.rs
(Orchestrator will supply exact files via --files args)
