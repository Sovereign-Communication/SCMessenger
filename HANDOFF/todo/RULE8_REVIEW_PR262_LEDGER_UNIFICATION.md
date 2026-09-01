# Rule-8 adversarial review -- PR #262, peer ledger unification

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
