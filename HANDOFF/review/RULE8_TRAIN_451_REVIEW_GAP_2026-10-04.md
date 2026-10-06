# Rule-8 review gap on PR #451 (integration train)  -  18 gated files, no live verdict

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-10-04
Head reviewed: `c9876858a` (`integrate/train-20261004`), the head of PR #451.
Base: `origin/main` = `051dbb5b9cd900669f11ecf5c40c63a38c2e86a6`
Method: enumerated from git, not from the PR board.

## Status: NOT REVIEWED. Do not merge #451 as a gated change.

## 1. What the gate requires

Owned by `Sovereign-Communication/SCMessenger` (this repository).

`docs/rules/SECURITY_PROTOCOL.md:90` - an agent that authored, proposed, or
specified a fix cannot provide the sign-off. `SECURITY_PROTOCOL.md:105` - a
verdict is scoped to the commit it reviewed; if later commits touch the
reviewed modules, the clean bills are void for those areas.

## 2. Why the authoring lane is disqualified

Three of the train's gated commits were authored by the integration agent:

| commit | file | change |
|---|---|---|
| `d7d4bb3be` | `core/src/routing/global.rs` | `is_route_pending_fresh` widened to `&[u8; 8]` |
| `20fa8e425` | `core/src/crypto/encrypt.rs` | test-module import of `RatchetSessionManager` |
| `258aafa0b` | `core/src/crypto/encrypt.rs` | `blake3::hash(&...to_bytes())` borrow fix (E0308) |

Two of the three are compiler-error fixes in `#[cfg(test)]` code, but they land
in gated directories and the rule is path-based. No sign-off was requested from
an uninvolved reviewer for any of the three.

## 3. Existing verdicts, and why none of them cover this head

Seven Rule-8 verdict documents exist in `HANDOFF/review/`. Checked each:

| verdict | verdict head | ancestor of train head? | covers current tree? |
|---|---|---|---|
| `V040_PR281_GHOST_IDENTITY_001_RULE8_HARNESS_APPROVE_2026-09-11.md` | `fe6f895f` | **NO** | no - tree absent |
| `RULE8_PR305_VERDICT_2026-09-18.md` | `880765c9` | yes | **no** - 5 later swarm.rs commits |
| `RULE8_ZOMBIE_FIX_VERDICT_2026-09-18.md` | `8cc356b8` | yes | **no** - superseded by later commits |
| `V040_PR282_RULE8_ADVERSARIAL_SECURITY_APPROVAL_2026-09-12.md` | `b65bc4d7` | **NO** | no - tree absent |
| `RULE8_PR289/PR296/PR297_VERDICT_2026-09-17.md` | (see docs) | n/a | no - no gated-path coverage |

`git log fe6f895f..origin/integrate/train-20261004 -- core/src/transport/swarm.rs`
returns **19 commits**. Under `SECURITY_PROTOCOL.md:105` the PR281 ghost-identity
verdict is therefore void for `swarm.rs`, and the newest live swarm verdict
(`8cc356b8`, 2026-09-18) is itself followed by 18 further gated commits.

Post-verdict gated commits with **zero** citations in any review document:

```
1e0453b17  2026-10-02  fix(transport): allow clippy double_must_use ... (rule 8 review required) (#427)
7a6c27192  2026-09-29  fix(transport): bound per-peer cap to 16 for handover
a6bb5c1ef  2026-09-29  fix(transport): raise the per-peer connection cap from 4 to 64
dd146c5df  2026-09-21  feat(core): one auto-subscribe decision for both gossip loops (WP3)
c4108d99a  2026-08-22  transport(addr_filter): reject self-referential and self-destination circuits
832c8f42b  2026-09-26  Drop stale ratchet session on unrecoverable decrypt failure (#394)
258aafa0b  2026-10-03  fix: borrow the key bytes for blake3
20fa8e425  2026-10-03  fix(crypto): import RatchetSessionManager in the encrypt test module
1025be436 / ac74c7b5c / 1070bc51c / edef05658 / 8670072a8  (train merge commits)
```

`d7d4bb3be` appears in exactly one review document: this repository's own
integration resolution ledger, authored by the same agent that wrote the fix.
That is not an independent review.

Note `1e0453b17` self-declares `(rule 8 review required)` in its own subject
line and no such review is on file.

## 4. Scale of the unreviewed delta

`git diff --stat origin/main..c9876858a -- core/src/{crypto,transport,routing,privacy}`
-> **18 files, +838 / -51**, from 17 commits. Largest:

```
core/src/transport/addr_filter.rs  +203 / -1
core/src/transport/swarm.rs       +183 / -23
core/src/crypto/encrypt.rs        +144 / -14
core/src/routing/neighborhood.rs  +103 / -2
```

## 5. Candidate findings (NOT a substitute for the gate)

Surfaced by reading the diff. `docs/rules/SECURITY_PROTOCOL.md:98` is explicit
that panels find candidate findings and do not clear the gate.

**F1  -  `is_ghost_peer_topic` fails OPEN on `wasm32`.** The `wasm32` branch
returns `false` ("not a ghost") unconditionally, while the native branch fails
closed when the ledger is unavailable. On wasm every 64-hex peer topic is
auto-subscribed. This is pre-existing (commit `629a3eefa`), so it is not a
regression from this train, but it is the opposite of the stated GHOST-IDENTITY-001
intent and is now reached through the shared `topic_subscribe_decision`. Worth an
explicit accept-or-fix decision.

**F2  -  own-key exemption precedes ledger consultation.** `is_ghost_peer_topic`
returns `false` for the node's own key before any provenance check. A node that
knows a peer's public key can therefore cause that peer to auto-subscribe. This
is inherent to "always trust your own topic" and is corroborated as intended by
`6acaa2317` (nodes never subscribed to their own topic), so it is expected
behaviour, recorded here so the reviewer rules on it deliberately rather than by
omission.

**F3  -  per-peer connection cap 4 -> 16.** `MAX_ESTABLISHED_PER_PEER` is now 16 with
a compile-time floor assert of 16, meaning the constant cannot be tightened
without editing the assert. Global bounds (64 incoming / 128 outgoing / 32
pending) are unchanged. Motivation is mobile Wi-Fi to Cellular handover. Needs a
judgement on whether 4x per-peer is acceptable on constrained devices.

**F4  -  `832c8f42b` ratchet session drop.** An unrecoverable decrypt failure drops
the cached ratchet session. Needs confirmation that this cannot be used to force
a downgrade or a rollback to a prior session.

## 5a. Independent adversarial review (2026-10-04, uninvolved reviewer)

The Harness MCP panel could not run (section 6). An isolated, audit-only
Claude CLI session reviewed the 18-file packet instead. It had no repository
write access and edited nothing. Verdict: **BLOCK, medium confidence**, with
the block resting on F4. Summary of what it found:

| # | Finding | Its verdict | Severity |
|---|---|---|---|
| F1 | wasm32 fails OPEN on `is_ghost_peer_topic` | REAL but predates this delta | Low-Medium |
| F2 | own-key exemption precedes ledger | NOT REAL as a bypass | Info |
| F3 | per-peer cap 4 -> 16 | REAL trade-off, not a vulnerability | Low |
| F4 | ratchet session dropped on unrecoverable decrypt | REAL availability regression, no downgrade/rollback found | Medium |
| N1 | `max_gateways` became a soft cap in `neighborhood.rs` (deferral below 30 s age removes the hard bound on untrusted peer gossip) | NEW | Medium |
| N2 | auto-subscribe accepts any non-ghost topic a peer subscribes to | NEW-to-review, pre-existing behaviour | Medium |
| N3 | `PERIMETER-ALLOW-UNDERSCORE` exemption on `init_as_receiver_hybrid`; sender-auth static-static DH term into the 0x03 root not confirmable from the diff | UNVERIFIED | potentially High |
| N4 | V1 static-key session rebuild permits replay + reset to initial state | pre-existing | Medium-High |
| N5 | `is_route_pending_fresh` is a NEW function taking `&[u8;8]`, matching sibling `is_route_pending`; not a widening. Confirms the merge-order-hazard account | correction to this packet's premise | Low |
| N6 | `negative_cache` two-confirmation reputation override | NEW | Low |
| N7 | `addr_filter::is_self_circuit*` added but not called in this delta (Rule 16: unreferenced) | NEW | Info |
| N8 | `health.rs` `avg_latency_ms = (avg + latency)/2` unchecked `u64` add | NEW | Low |
| N9 | `#![deny(unused_variables)]` on four perimeter modules; cfg-gated vars now break builds | NEW | Info |

### Verification of the two blocking claims (F4), performed in-tree

The reviewer could only read the diff, so its two blocking preconditions were
checked against the source directly:

- **V1 (signature precedes decrypt): HOLDS.** `core/src/iron_core.rs` rejects
  any envelope that is neither Drift-signed nor decodable as a signed
  `WireSignedEnvelope`, returning `IronCoreError::CryptoError` before any
  decrypt. Unsigned envelopes cannot reach
  `decrypt_with_ratchet_fallback`, so `peer_id` is not third-party-spoofable at
  that call site. This is the ONLY production caller in the tree (the other 25
  references are tests).
- **V2 (dedup precedes decrypt): NOT ESTABLISHED.** No message-ID dedup or
  replay filter appears inside `decrypt_with_ratchet_fallback`
  (`core/src/crypto/encrypt.rs:663-860`). Whether one runs earlier in the
  receive path is still open and needs a human or a wider read.

### F4 bounded by code reading

Both `remove_session` sites fire only AFTER a session rebuild was attempted and
also failed (`:755` V1 path, `:845` V2 `init_err`/retry path). So a session is
not dropped on first failure; it is dropped when re-establishment genuinely
cannot recover. That is materially narrower than "one bad ciphertext kills a
healthy session", but it remains new behaviour and is now locked in by a test.

### Net effect on the merge decision

F4 is Medium rather than High given V1 holds, but V2 is unproven and N3 is
unverified at potentially High. The reviewer's own condition stands: **do not
merge #451 until V2 and N3 are checked.** This handoff's section 7 options are
unchanged, with N3 added as a required check under option 2.

## 6. Why no fresh panel was run

Attempted via the Harness MCP `panel_verify`. Refused twice, fail-closed:

```
HarnessError: panel_verify task_max_cost 0.1 exceeds remaining session budget 0.049998
HarnessError: worst-case estimate $0.286017 exceeds remaining ceiling $0.050000
HarnessError: worst-case estimate $0.226340 exceeds remaining ceiling $0.050000
```

The worst-case estimate is dominated by fixed panel + judge lane overhead, not
prompt length: a 1400-character prompt still estimated $0.226. The 10-cent cap
requested was therefore unreachable against a 5-cent remaining ceiling at any
prompt size tried. No verdict was produced.

## 7. Options for the operator

1. **Fund a Rule-8 panel on the current head** and review `c9876858a` end to end
   against all 18 files. This is the only option that satisfies the gate as
   written. Needs budget on the Harness account; prior verdicts cost $0.004 to
   $0.10 each.
2. **Human adversarial review** of the 18 files by a reviewer who authored none
   of `1e0453b17 7a6c27192 a6bb5c1ef dd146c5df c4108d99a 832c8f42b 258aafa0b
   20fa8e425 d7d4bb3be`.
3. **Land the non-gated majority and hold the gated paths.** #451 is 130 files;
   112 are outside the merge-blocked set. Splitting is a real option if the goal
   is closing out the train quickly, but it conflicts with the train's
   single-wave purpose and the Android cluster's shared-file collisions.
4. **Explicit operator override**, recorded in the merge body naming which files
   were merged unreviewed. This is a governance decision, not a technical one.

Rule 8 as written permits none of these automatically. Do not merge #451 until
one of 1-4 is chosen and recorded.

## Non-blocking note

`Mobile` is **not** a required context on `main` (required set is exactly
`Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`,
`Handoff ownership scope`, strict) and `main` runs no `Mobile` job at all. The
four Android jobs were verified green on `c9876858a` from their logs, but Android
regressions cannot block a merge through the current protection config.