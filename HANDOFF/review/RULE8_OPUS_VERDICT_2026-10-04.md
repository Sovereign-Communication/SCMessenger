# Rule-8 verdict: PR #451 gated delta, independent Opus review (2026-10-04)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Owned by `Sovereign-Communication/SCMessenger` (this repository).

Code state reviewed: `c9876858a`. Base: `origin/main` = `051dbb5b9cd900669f11ecf5c40c63a38c2e86a6`.
Reviewer: uninvolved Opus session, read-only, no builds run.
Scope: 18 files, +838/-51, under `core/src/{crypto,transport,routing,privacy}/`.

## Verdict: BLOCK

One HIGH finding, latent. No CRITICAL. Nothing else blocks.

The previous diff-only review could not settle V2 or N3. This review had source
access and settled both. It also found a third session-drop site the earlier
review missed entirely.

## The blocking finding: session teardown on a duplicate delivery

`core/src/crypto/encrypt.rs:864`, in the `Err(init_err)` arm of the V2 recovery
path, calls `manager.remove_session(&peer_id)` when re-establishment is
unavailable.

Chain, each step checked in-tree:

1. Nothing filters duplicates or replays before decrypt. `receive_message`
   (`core/src/iron_core.rs:3650`) verifies the signature (`:3663-3725`) and calls
   `decrypt_with_ratchet_fallback` (`:3741-3760`) with no seen-set or replay
   window between. The only dedup, `inbox.is_duplicate` (`iron_core.rs:3952`),
   needs the decrypted plaintext and so runs afterwards. The swarm's
   `RelayGuardrails::is_recent_duplicate` (`swarm.rs:1827`) guards relay
   forwarding only, keyed on a 30 s window, and does not cover direct delivery
   or store-and-forward delay.
2. A duplicate fails decrypt because its message key is already consumed
   (`ratchet.rs:1112-1114`, "behind current chain position"), reaching recovery at
   `encrypt.rs:815`.
3. A duplicate never carries bootstrap fields: after the first decrypt the
   receiver sets `peer_confirmed = true` (`encrypt.rs:339`) and the sender stops
   attaching bootstrap (`encrypt.rs:372-401`), so `pq_kem_ciphertext == None`.
4. `init_receiver_v2_session` therefore builds `hct = None`
   (`encrypt.rs:634-646`) and `create_receiver_session_hybrid` bails at
   `session_manager.rs:185` (0x03) or `:200` (0x02).
5. The bail happens BEFORE the map insert (`session_manager.rs:210-214`), so the
   healthy session is still intact when control reaches `encrypt.rs:864`, which
   then deletes it.

After the drop, the next legitimate message also lacks bootstrap, so it fails the
same way: every later message from that contact fails until the sender resets its
own session. The new comments at `encrypt.rs:711-714`, `840-843` and `860-863`
say "next message will renegotiate"; that is false for traffic after
confirmation.

Anyone holding a copy of a signed envelope - a relay, or a store-and-forward
custodian - can trigger this deliberately by replaying it.

**Why this site and not the other two.** At `:755` and `:844` the rebuild has
already REPLACED the session (`session_manager.rs:93-96`, `210-211`), so dropping
it loses nothing that `main` had not already lost. `:864` is the only site that
deletes an otherwise healthy session, because no rebuild happened there.

**Latent, not live.** `save_contact_bundle` (`core/src/store/contacts.rs:770`)
has no non-test caller anywhere in `core/`, `cli/`, `mobile/` or `wasm/`, and no
bundle API exists in the `.udl`. Verified in this session: only the definition
and two test callers exist. With no bundle, `should_use_ratcheted_encryption`
(`encrypt.rs:453`) selects legacy V1 unless a session already exists, so the V2
hybrid path is unreachable in production today. It becomes live the moment
bundle ingestion is wired - which PR #400 ("verify contact bundle signature")
is adjacent to.

### Recommended minimal fix

In the `Err(init_err)` arm, keep the session and return `Err(first_error)`.
Alternatively skip the drop when the first error is the behind-current-chain
duplicate case. Either way, update
`test_decrypt_fallback_drops_stale_session_on_unrecoverable_failure`
(`encrypt.rs:1126+`), which currently asserts the drop on exactly this path and
would lock the defect in.

## V2 answered: NO dedup before decrypt

See the chain above. A duplicate delivery does reach the drop branch.

## N3 answered: the 0x03 sender-authentication term is REAL

Not a flaw. Independently re-verified in this session:

- Domain separator byte-identical on both sides: `"iron-core session-root v3
  2026-08"` at `ratchet.rs:479` (sender) and `:571` (receiver).
- Both sides feed `dh_static` into `blake3::derive_key` alongside `ss_hybrid` and
  `transcript_hash`: sender `:474-485`, receiver `:566-577`.
- Both use the dedicated X25519 key, and Curve25519 DH is commutative, so both
  compute the same shared secret. `_our_signing_key` is genuinely unused on the
  receiver, so the `PERIMETER-ALLOW-UNDERSCORE` exemption at `ratchet.rs:536-538`
  is honest.
- The transcript binds both Ed25519 identities (`negotiation.rs:39-67`), and the
  X25519/ML-KEM keys are bound to the Ed25519 key by the bundle signature
  (`keys.rs:577-581`), checked before any bundle is stored (`contacts.rs:777`).
  Substituting a bundle would require forging that signature.
- The 0x02 exemptions (`ratchet.rs:652-655`, `730-733`) are honest: that suite
  derives from `[ss_hybrid, transcript_hash]` only, and its sender authentication
  comes from the outer envelope signature.

## Prior findings, corrected

| ID | Verdict | Severity | Note |
|---|---|---|---|
| F1 wasm fails open | Real, but ALREADY ON MAIN, not in this delta | LOW | WASM is shipped (`release.yml:389-456`, published at `:464`/`:501`). The delta only routes both loops through `topic_subscribe_decision`; behaviour unchanged. Reliability issue, not confidentiality. |
| F2 own-key exemption | Not real | none | `own_peer_key_hex` is our own local peer id. Already on main. |
| F3 per-peer cap 4 to 16 | Acceptable | LOW/INFO | PeerIds are free, so the per-peer cap was never a Sybil defence. Global 64 inbound is the real limit; `16 < 64` is enforced at compile time. The floor is a tripwire, not a lock. |
| F4 drop sites | Correct for 755/844, WRONG for 864 | HIGH at 864 | See above. |
| N1 gateway soft cap | Real but BOUNDED, and unreachable | LOW | Each insert evicts at most one entry (`neighborhood.rs:187-189`), so overshoot is attacker-rate times 30 s, not unbounded. Also no production code populates the table. Downgraded from the earlier MEDIUM. |
| N7 `is_self_circuit*` | Dead code; HARMFUL if wired as written | LOW now | No call site anywhere. It returns `true` for this node's own relay-reservation address, so wiring it into listen or external-address filtering would break the node's own reachability. Remove it or scope any future call to dial candidates. |
| N8 unchecked `u64` add | Confirmed, unreachable | INFO | `health.rs:603`. `record_message_success` has no production caller; latency would be locally measured. Release builds wrap rather than panic. |

## Also found

- **Latent unit mismatch, LOW.** `prepare_message` passes `now` in SECONDS
  (`iron_core.rs:1111-1114`) while `get_best_forwarding_path` passes
  MILLISECONDS (`iron_core.rs:4374`). The comparison is against
  `MAX_ROUTE_REQUEST_AGE_SECS = 300`. Before this delta `now` was ignored
  (`_now`); this delta reads it. Latent only because `request_route` has no
  production caller.
- **Pre-existing MEDIUM, same function, not introduced here.** Replaying an early
  bootstrap-carrying envelope makes `init_receiver_v2_session` OVERWRITE a
  healthy session (`session_manager.rs:210-211`), resetting the receiver's
  ratchet. Exists on `main` whenever the path is live. A real fix is a pre-decrypt
  replay check on `(sender, ratchet_dh_public, message_number)`.
- `#![deny(unused_variables)]` on the four perimeter modules is harmless but not
  a real gate: `let _ = param;` also silences it (`wifi_aware.rs:258`, `270`,
  `287`).
- **Not in this delta, therefore not reviewed:** the whole `addr_filter.rs`
  module is already on `main` except the new `is_self_circuit*`; so are zombie
  reaping and exact-IP deny-join. A prior reviewer could not determine this.

## Disposition

#451 must not merge until the `encrypt.rs:864` drop is fixed and the test is
updated. This is now a concrete, one-site fix with a named test, not a general
uncertainty.

Reviewer independence: this session authored three gated commits
(`d7d4bb3be`, `20fa8e425`, `258aafa0b`) and remains disqualified as sign-off
under `SECURITY_PROTOCOL.md:90`. The verdict above is from an independent Opus
session. Per `SECURITY_PROTOCOL.md:98` a single model consulted this way is not
itself the gate; the operator's acceptance of this verdict is the governance
step that remains.