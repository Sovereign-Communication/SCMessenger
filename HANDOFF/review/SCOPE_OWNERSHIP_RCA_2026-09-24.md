# SCMessenger scope-ownership RCA and corrective rule

**Date:** 2026-09-24
**RCA ID:** `SCOPE-MISTAKE-001`
**Change boundary:** Documentation only. No source, test, hook, CI, identity, contact, runtime, installation, deployment, commit, or PR action was performed.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Concrete scope mistake

A review dossier produced for a different product lane was placed under this owner’s `HANDOFF/review` tree and was treated as an owner-local reviewer input. That was a cross-owner import and a scope error. The imported artifact remains quarantined and byte-for-byte unchanged; none of its findings, evidence, status, or remediation is adopted by SCMessenger through this record.

The operations index identifies the artifact as `SCM-Q-001` and records its byte count and SHA-256 without copying its foreign name into this owner handoff.
The quarantined artifact is 8,228 bytes with SHA-256 `20ae1506627079a08c03c8bb62a7735fbc6319a2d37bc5a3b2a0d23c10c92499`.

## Corrective rule

1. A handoff has exactly one owning product repository.
2. A document is `owner-valid` only when it has the exact owner metadata, the owner is identified in the body, the owner-local gate passes on the actual bytes, and the evidence is assignable to this owner.
3. A document with a foreign-product alias in its path or bytes is `quarantined`; it is not edited, copied, committed, published, or used as evidence.
4. A document without a foreign alias but failing the metadata/owner gate is `blocked-by-metadata`; it remains untouched until its owner supplies a valid manifest and remediation.
5. No legacy file is mass-migrated. Migration requires an owner-approved manifest and a deterministic rule, followed by a fresh owner-local gate run.
6. A cross-lane observation must be split into separately evidenced owner records before handoff. This record makes no claim about the other owner’s product state.

## Current disposition

The exact path register and complete classification snapshot are maintained in the OC operations index. The owner-local inventory is `SCOPE_INVENTORY_2026-09-24.md`. The imported dossier is quarantined, not migrated. PR contact is intentionally not attempted because no PR identity is established; the local branch, status, and evidence are the reportable record.
