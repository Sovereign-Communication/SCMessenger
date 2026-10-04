# SEC-03 storage migration — branch brief (safe-ahead)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN — authorized parallel exploration only (operator ruling 2026-09-20 option b)
Priority: Prepped; not Wave 1 paste
Lane: Native/orchestrator design first; Freebuff may implement **after** a chosen engine + plan on file
Authority: `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`

## Ruling

Operator: start migration on a branch; tagging can proceed later on current
sled once the working mesh bar is met. Engine swap to `main` still needs a
further operator sign-off (rule 9).

## Why

`deny.toml` waives multiple RUSTSEC ids for unmaintained sled/transitives with
no expiry. Working-first does not mean ignore supply-chain debt — it means do
not block mesh reliability on a storage rewrite.

## Safe-ahead work (no Freebuff paste yet)

1. Inventory storage trait surface: `core/src/store/` IronCore entry points,
   what actually touches sled vs memory.
2. Shortlist engines (redb, fjall, other) with license + Windows + Android NDK
   notes.
3. Draft migration design: dual-read/write window or offline convert;
   `DegradedStorage` path; identity/ledger/custody key formats.
4. Keep waivers **dated** in a follow-up deny.toml edit once an owner/expiry
   is chosen (orchestrator can file the one-line waiver expiry after operator
   names a date — do not invent one).

## Do not

- Merge a new storage engine without explicit operator approve.
- Block Wave 1 freebuff pastes on this brief.
- Delete sled while live nodes hold `/data` identities.

## Exit for "ready to implement"

Engine chosen + design on file + operator GO for a specific branch name.
