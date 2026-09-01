# Rule-8 adversarial review -- PR #262 (ledger unification) and #263 (routing feed)

Status: OPEN (filed 2026-08-31)
Priority: P0 -- #262 cannot merge without a recorded APPROVE
**Lane: NOT Freebuff.** Freebuff authored #262; Rule-8 requires a reviewer that
did not author the change. Route to the DashScope/Qwen lane (reasoning tier:
`qwq-plus`, 906k remaining, or `deepseek-v4-pro-0813`, 1M) per
`docs/QWEN_QUOTA_LEDGER.md`, or to a native seat that did not write the spec.

## What you are reviewing

PR #262, branch `freebuff/v040-t2-unify-peer-ledgers`. +2,172 / -1,666 across 10
files. It unifies two peer stores into one and **changes what this node discloses
to other peers**, which is why it is gated.

```bash
git fetch origin freebuff/v040-t2-unify-peer-ledgers
git diff origin/main...FETCH_HEAD -- core/src/store/ledger_entry.rs cli/src/ledger.rs
```

## Already verified by the CEO seat -- do not redo, do try to break

These were checked and held. Treat them as claims to falsify, not as settled:

1. `core/src/transport/swarm.rs` change is test-only (5 struct fields in a test
   fixture); the production `endpoint.is_dialer()` guard is untouched.
2. The disclosure filter (`.filter(|e| e.locally_verified)`) is present on
   `export_seed_entries_for` and `exchange_response_entries_for_request`, and
   the two public wrappers `export_seed_entries` / `exchange_response_entries`
   delegate to them rather than bypassing.
3. Imported entries land `locally_verified: false`.
4. `peers.json` is no longer written; a one-time migration archives it.

## What has NOT been reviewed -- this is your actual job

The CEO seat read roughly 60 lines of a 2,172-line change and is **not** confident
beyond the four points above. Everything below is unexamined:

1. **Is there any other path to the wire?** `ledger_entry_to_shared` and
   `ledger_entry_to_shared_routing_only` convert entries to the shared format.
   Find every caller. If anything reaches them with an entry that is not
   `locally_verified`, the disclosure rule is bypassed and the unification
   spreads one node's pollution mesh-wide -- strictly worse than the bug it
   fixes. **This is the highest-value thing you can check.**
2. **What actually sets `locally_verified = true`?** It should be a completed
   *outbound* dial and nothing else. Check every assignment site. An inbound
   connection, a gossiped entry, or a migration path that sets it true is a
   disclosure hole.
3. **The migration.** Does it reject self-entries (own identity, own listener
   addresses, historical self identities)? Does it reject ephemeral source
   ports? The real poisoned inputs are preserved for replay: 4,678 entries at
   `%LOCALAPPDATA%\\scmessenger\\peers.json`, and 107 at
   `/opt/scm-relay-data/peers.json.poisoned-backup-1788196` on the AWS node.
4. **The #256/#257 property.** Those PRs reset a peer's failure counter on any
   successful connection, so a reachable peer never sticks in the dead tier.
   Does the new supersession logic preserve that, or can a peer now become
   permanently undialable?
5. **Caps.** `MAX_LEDGER_ENTRIES` (1024) and any per-peer address cap -- are they
   enforced on *every* insert path, including migration and gossip import? The
   old CLI store was uncapped and reached 4,678 entries; if any path skips the
   cap, that returns.
6. **The four defects the author says the rewritten tests surfaced and fixed**
   (stale-identity guard on rebind, canonical-hex/base58 self-filter parity,
   operator bootstraps dialable pre-first-success, hearsay gate at ingress).
   Verify each fix is real and that the test would fail without it.

## Method

Adversarial. Your job is to find the hole, not to confirm the design. Default to
REJECT where you cannot prove a property holds. For each finding give the
`file:line`, the concrete input that triggers it, and the consequence.

Note the conflict of interest you are correcting for: **the CEO seat authored the
T2 specification, including the `locally_verified` disclosure rule.** So do not
treat the spec as authority -- if the spec itself is wrong, say so. A reviewer
who only checks conformance to a flawed design has verified nothing.

## Output

A verdict of **APPROVE** or **REJECT**, with findings, written to
`HANDOFF/freebuff/inbox/RULE8_PR262_VERDICT.md`. APPROVE only if you have
positively checked items 1-6. "I found nothing" without saying what you looked at
is not an APPROVE -- record what you examined and what you could not, and mark
the latter `UNVERIFIED`.

## Rules

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Read-only. Do not fix what you find -- report it.
- Do not merge. Merge is the operator's call after the verdict.

---

# ADDENDUM -- PR #263 also needs this gate (added 2026-08-31)

Same reviewer, same rules, second PR. `freebuff/v040-t4-routing-feed`,
+232/-6 across `core/src/transport/swarm.rs`, `core/src/iron_core.rs`,
`core/src/routing/local.rs`. Two of those are in the merge-blocked set, and it
lands in the **same `ConnectionEstablished` handler** that #262 touches -- so
review them together and check they do not conflict.

## Verified already (falsify, do not redo)

`IronCore::routing_peer_seen` now has a production caller at
`swarm.rs:5571`, inside `ConnectionEstablished`, guarded by `peer_is_blocked`
and a weak-ref upgrade. Confirmed no `#[cfg(test)]` precedes it. That closes the
D6 defect where the function had zero callers repo-wide.

## What to attack

1. **Is this a routing-poisoning vector?** Routing confidence is now fed on
   *every* `ConnectionEstablished`. Can a peer that repeatedly connects and
   disconnects inflate its own routing score, or evict honest peers from
   LocalCell? The `peer_is_blocked` guard covers blocked peers; it does not
   cover an unblocked hostile one. **This is the highest-value question here** --
   the change converts inbound connection events into routing state, which is an
   attacker-influenced input.
2. **`TransportType::Circuit` and the new score rank**
   (BLE < WiFi < Circuit < TCP < QUIC). Does inserting a variant renumber or
   reorder anything persisted, compared, or serialized? Does any match become
   non-exhaustive in a way the compiler did not catch (e.g. a `_ =>` arm that
   now silently swallows Circuit)?
3. **`parse_transport_type` extended for ws/wss/circuit.** The author reports WS
   previously fell through to **BLE**, which if true means every WebSocket peer
   has been scored as BLE. Confirm the old behaviour and that the fix does not
   change scoring for some *other* address shape as a side effect.
4. **Lock discipline.** Is the routing engine's write lock held across an
   `await` anywhere in the new path? `parking_lot` guards are not async-aware
   and this is inside the swarm event loop -- a held lock here stalls every
   peer.
5. **The WASM arm.** The author says the wasm32 check caught a real partial-move
   of `endpoint` that no native gate would find. Verify the fix is correct and
   that native and WASM arms now record the *same* transport for the same
   endpoint -- a divergence would make D6 evidence platform-dependent.
6. **Does it conflict with #262?** Both edit `ConnectionEstablished`. Whichever
   merges second must still apply cleanly and preserve both behaviours.

Verdict for #263 to `HANDOFF/freebuff/inbox/RULE8_PR263_VERDICT.md`, same
standard: APPROVE only on positively checked items, `UNVERIFIED` for anything
you could not reach.
