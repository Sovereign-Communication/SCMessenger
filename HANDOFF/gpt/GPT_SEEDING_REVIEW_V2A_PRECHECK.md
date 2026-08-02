# GPT SEEDING REVIEW -- v2a precheck and signal request

Status: BLOCK -- REMEDIATION AND UPDATED READY SIGNAL REQUIRED
Date: 2026-07-28
Remote ref observed: `refs/heads/wip/v040-seeding-fixes`
Prechecked range: `068972f2d3cfe4578a7dc713a159a7d0bcee6bf5..909edf4c7e03cd8b5467cb4d72b440ffee3d9d6a`
Verification: read-only diff and authoritative-tip inspection; no Mac build

This is a bounded precheck, not the formal next-stage verdict. The
authoritative READY block in `GPT_REVIEW_SEEDING_FIXES.md` still ends at
`068972f2`, and no `WINDOWS_GATE` result has been published for v2a.

## F10 load boundary -- PARTIALLY FIXED

`load()` still reads and allocates the complete file, deserializes every
record, then sorts the complete vector before truncating it
(`core/src/store/ledger_entry.rs:271-290` at `909edf4c`). A legacy ledger with
hundreds of thousands of entries can therefore still block startup or exhaust
memory before the 1,024-entry cap takes effect. The new test exercises only
1,124 records (`:958-975`) and does not cover this failure class.

The retained 1,024 entries are installed only in memory; `load()` never
rewrites the oversized file. Every restart repeats the unbounded parse and
sort until an unrelated later mutation happens to save. The cap is therefore
not durable at the persistence boundary.

## F10 record-byte bounds -- NOT FIXED AT LOAD

The new limits reject oversized values on selected new-ingest paths
(`:45-51`, `:143-229`, `:353-367`, `:695-700`), but `load()` applies none of
them. A pre-existing valid JSON file can retain 1,024 multi-megabyte
multiaddrs, peer IDs, public keys, nicknames, or topic vectors, including
invalid peer IDs, and those records are installed and later reserialized.
This leaves the upgrade/legacy path vulnerable to the exact persistent byte
growth v2a is meant to close.

## F7 threshold alignment -- PARTIALLY FIXED

`seed_addresses` and `get_preferred_relays` now use the dial-policy threshold
of three (`:491-507`, `:578-589`). Two other ledger eligibility paths still
hard-code five: `dialable_addresses` (`:463-470`) and
`exchange_response_entries` (`:770-784`). A locally policy-dead address can
therefore remain visible through the exported accessor and continue to be
gossiped to fresh nodes. The new boundary test does not exercise either
remaining path.

## Known blockers unchanged

The stage-1b lost-update and non-atomic-write regression remains pending v2c,
and production wire handlers still call `annotate_identity` per entry pending
packet 1c. F7(a), F7(b) failure wiring, F13, and NEW-6 remain pending packet 2.
The four-commit range is therefore NO-SHIP regardless of the v2a findings
above.

## Documentation delta

The baseline freeze and finding-disposition table are useful and correctly
keep PR #116 non-mergeable until later gates clear. The terminology scrub is
mostly aligned with the current nodes-not-relays doctrine. Three remaining
plain-language phrases in `V040_S4_DELIVERY_PROOF_RUNBOOK.md` should use
cloud-node terminology: "Relay evidence", "relay listener", and "relay entry".
Literal log strings and technical identifiers may remain unchanged.

## Requested next signal

Please remediate the v2a persistence-boundary and threshold gaps, then publish
one updated immutable signal:

    READY
    REVIEW_TARGET: 068972f2d3cfe4578a7dc713a159a7d0bcee6bf5..<corrected-tip>
    REMOTE_REF: refs/heads/wip/v040-seeding-fixes
    WINDOWS_GATE: PASSED|FAILED <exact gate and result>

GPT will then review only the corrected delta plus the authoritative full tip
tree. Do not advance or merge PR #116 from the current `909edf4c` tip.
