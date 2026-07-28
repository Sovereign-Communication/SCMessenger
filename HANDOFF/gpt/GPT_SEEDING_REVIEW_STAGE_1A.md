# GPT ADVERSARIAL REVIEW -- Wave 1b stage 1a

Status: BLOCK -- REMEDIATION REQUIRED
Reviewed range: `ed13500abaf372836be37bef93f3eaf5a24765a6..d258fd7fecf84363a286093e6f236c0d4b7fa677`
Remote ref: `refs/heads/wip/v040-seeding-fixes`
Verification: read-only diff and surrounding-source inspection; no Mac build

## F10 -- NOT FIXED

`load()` still installs an arbitrarily large legacy vector unchanged
(`core/src/store/ledger_entry.rs:175-186`); only new-insert arms enforce the
cap (`:275-277`, `:378-380`, `:628-630`). A previously persisted 80,000-entry
ledger therefore remains oversized through existing-entry updates and saves,
while the first unknown insertion repeatedly scans and `Vec::remove`s until
1,023 entries remain (`:108-132`), producing quadratic work under the mutex.
`seed_addresses(limit)` also clones and sorts every eligible record before
applying `limit` (`:425-441`), so an oversized legacy file makes even
`seed_addresses(0)` perform attacker-amplified work. The pending stage 1b/1c
batch and save changes are not scored against this stage, but F10's cap is not
effective at the persistence boundary yet.

## F7(b) seed ordering -- REGRESSION

Invite imports store `last_seen=None` (`ledger_entry.rs:631-640`), whereas
wire-learned annotations receive receiver-local `Some(now)` (`:375-390`);
eviction selects `None` first (`:113-123`) and the new sort ranks every `Some`
ahead of every `None` (`:425-441`). One accepted 64-entry hostile exchange can
therefore evict all 16 invite anchors and monopolize the eight seed-dial slots.
At capacity, importing a 16-seed invite into an all-`Some` ledger retains only
the last imported seed because each new `None` evicts the preceding `None`,
although `import_seed_entries` reports all 16 as added (`:607-646`). Equal
timestamps and `None` values have no canonical address tie-break, and the
`success_count` tie-break is vacuous after filtering for `success_count == 0`,
so peers with the same set can choose different top-eight subsets.

## F7(b) failure rotation -- NOT FIXED

The planned dial-policy gate and this ledger ordering have incompatible
thresholds: `DialPolicyManager` marks an address dead after three failures
(`core/src/transport/dial_policy.rs:47-84`), but `seed_addresses` retains it
until five (`ledger_entry.rs:425-430`). Because only the top eight seeds are
materialized before any policy gate (`core/src/transport/swarm.rs:5547-5565`),
eight policy-dead entries can permanently hide seed nine and every invite
anchor for the rest of the process; they can never receive failures four and
five. Proven candidates are worse: `get_preferred_relays` ignores
`failure_count` entirely, so the higher-priority proven tier does not rotate
dead entries either (`ledger_entry.rs:512-522`).

## Remaining storage DoS -- NEW ISSUE

The count cap does not bound record bytes. Ledger-exchange responses permit
16 MiB (`core/src/transport/behaviour.rs:372-377`), and the mobile merge stores
an arbitrary wire `last_peer_id` string without first requiring it to parse as
a `PeerId` (`core/src/mobile_bridge.rs:1074-1080`). Repeated large records can
therefore turn a nominally capped 1,024-entry ledger into multi-gigabyte memory
and JSON state. No new DNS-form or IP-literal SSRF admission bypass was found
in stage 1a; the existing address gates are unchanged.

## Test adequacy

The sole new test disables persistence and inserts zero-success records first
(`ledger_entry.rs:1410-1441`), so a naive oldest-index eviction satisfies both
of its policy assertions. It does not exercise legacy load, save/reload,
full-ledger invite import, all-proven eviction, ordering ties, failure
threshold interaction, or record-byte growth.

## Stage decision

NO-SHIP for stage 1a as reviewed. Please publish the remediation as another
commit on `wip/v040-seeding-fixes` with its exact parent and tip; GPT will
re-review that delta while continuing to monitor stages 1b, 1c, and 2.
