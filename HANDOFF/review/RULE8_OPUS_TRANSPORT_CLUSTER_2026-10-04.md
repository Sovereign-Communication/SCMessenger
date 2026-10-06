# Rule-8 independent verdict -- gated transport cluster (VERDICT: BLOCK)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-10-04
Reviewer: isolated, history-free Claude session (`claude -p`), read-only
(`dontAsk`), no access to the authoring conversation
Model: `claude-haiku-4-5-20251001,claude-opus-5-5`
Cost: $1.3402 · 26 turns · 195,078 ms · session `bdb11f70-e11a-4c36-9e40-dbebcd048a51`
Subjects: PRs #372 #359 #361 #364 #413 heads, and the train branch

---

## SCOPE LIMITS -- READ BEFORE CITING

**This verdict BLOCKs the five-PR connection-cap cluster.** It does **not**
clear the train's own already-merged transport commits, and it is **not** a
Rule-8 clearance for PR #451.

PRs reviewed: #372, #359, #361, #364, #413. **Not reviewed:** every other
gated file and every other open PR.

Files read in full or by full three-dot diff:
- #372: all of `per_peer_cap.rs`; full diffs of `behaviour.rs`, `mod.rs`,
  `swarm.rs` (native and wasm), `dial_policy.rs`; the `add_relay` body and its
  `swarm.rs` call site (~6570-6620)
- #361: full three-dot `swarm.rs` diff plus the `ConnectionClosed` arms
  (~7215-7315); `per_peer_cap.rs` and `behaviour.rs` by grep only
- #364: full three-dot `swarm.rs` diff; constants/wiring by grep
- #359: full three-dot `swarm.rs` diff (older base); constants/wiring by grep
- #413: full three-dot transport diff (`swarm.rs` only)
- train: grep of `behaviour.rs` / `swarm.rs` for cap constants

**Not read in full:**
- the `per_peer_cap.rs` bodies and tests of #359, #361, #364
- `swarm.rs` outside the diff hunks, including the train's ghost-slot reaper
- libp2p's `connection_limits` source

The reviewer also reported that Bash was refused and two PowerShell commands
(a loop and a piped command) were refused, so single commands were used instead.
Findings M1-M3 and L1-L4 are scoped to that coverage. **C1 is the only finding
verified across all five heads and the train branch.**

---

## VERDICT (verbatim)

**VERDICT: BLOCK**

**Scope:** this verdict covers the per-peer cap and its enforcement in all
five PRs. Coverage of the rest is partial; see the disclosure at the end. I ran
no build or test; every finding below comes from reading source. Most commands
went through PowerShell `git show` / `git diff` / `git grep`. Bash was refused,
and so were two PowerShell commands (a loop and a piped command), so I ran
single commands instead.

### CRITICAL

**C1. The five PRs and the train branch set conflicting per-peer cap values.**
This is the most important fact for the merge.

| Source | Per-peer admission (libp2p `connection_limits`) | Retained bound (swarm loop closes extra paths) |
|---|---|---|
| train `integrate/train-20261004` | `MAX_ESTABLISHED_PER_PEER = 16`, one tier (`behaviour.rs:50`), plus a compile-time floor `>= 16` and `< MAX_ESTABLISHED_INCOMING (64)` (`behaviour.rs:57,62-67`), wired at `behaviour.rs:572` | none |
| #372 `2cb046cf8` | `ADMISSION_MAX_ESTABLISHED_PER_PEER = 8` (`per_peer_cap.rs`). Tests pin it `== 8`, `== 2 x retained`, and `== synthesised_ladder_width()` | `RETAINED_MAX_ESTABLISHED_PER_PEER = 4` |
| #359 `3f41005d1`, #361 `63f4a7d73`, #364 `45b0f8b8a` | `ADMISSION_MAX_ESTABLISHED_PER_PEER = 16` (`per_peer_cap.rs:50`), wired at `behaviour.rs:545-546` | 4 (`per_peer_cap.rs:55`) |
| #413 `5c80035f4` | not changed; still `with_max_established_per_peer(Some(4))` (`behaviour.rs:532`) | none |

- **8 vs 16:** #372 and #359/#361/#364 add the same new file `per_peer_cap.rs`
  with different values. That is an add/add conflict, and #372's tests
  (`admission_ceiling_stays_within_two_times_the_retained_bound`,
  `admission_ceiling_covers_the_whole_synthesised_ladder`) fail if 16 wins.
- **8 vs train's floor of 16:** #372 cannot land on the train branch as written.
- **Hidden failure mode:** suppose a conflict resolution keeps train's
  `MAX_ESTABLISHED_PER_PEER` (16) at the `with_max_established_per_peer` call
  but keeps #372's `per_peer_cap.rs`. #372's tests would all still pass, because
  they only assert the `per_peer_cap` constant, never the value actually handed
  to libp2p. That is the classic "constant asserted in a test but not consulted
  on the hot path" failure.
- **No test ties the module constant to the builder call** in any of the five
  PRs. Whoever resolves the conflict must pick one number deliberately and wire
  one constant into `behaviour.rs`.

### HIGH

None confirmed beyond C1.

### MEDIUM

**M1. #359, #361 and #364 leak one map entry per full disconnect, and an
attacker can drive it.**
- The partial-close arm (`if num_established > 0`) removes
  `path_last_activity[connection_id]`.
- The last-close arm only runs `peer_established_paths.remove(&peer_id)` (#361
  `swarm.rs` ~7310; #364 ~7351; #359 ~6781). It never removes the closing
  connection's own entry from `path_last_activity`.
- `ConnectionId` is monotonic and never reused, so any peer that repeatedly
  connects and fully disconnects grows `path_last_activity` without bound on a
  long-lived relay.
- #372 fixes this with `release_path` / `release_peer`, which drain the peer's
  ids on last close. #372 still has a smaller leak: a stray Ping or
  request-response event can re-stamp a connection that the trim already
  removed. If that connection then closes as the peer's last one, its entry
  stays behind (rare, see L1).

**M2. #372: attacker-controlled peers can push legitimate relays off the
ladder** (`dial_policy.rs` `add_relay`, call site `swarm.rs:6608`).
- `add_relay` keeps only the 8 most recently registered relays
  (`RELAY_TRACKING_CAP = 8`, newest first via `insert(0)` plus `truncate`).
- It is called for every identified peer ("all nodes are relays"), and again on
  every identify cycle.
- So 8 or more peer ids that the attacker controls, re-identifying often, can
  keep every tracked slot. `build_relay_addresses` then emits only their
  circuits.
- Effect: the attacker controls every circuit-relay candidate this node offers
  for any target. Payloads stay end-to-end Noise-encrypted, but the attacker
  sees who talks to whom and can drop traffic.
- Before this PR the list was unbounded but pruned on disconnect
  (`remove_relay`), so this displacement is new.

**M3. Pre-existing, but it determines whether these caps matter: the per-peer
cap is not the denial-of-service bound.**
- Peer ids cost nothing to generate, so an attacker can occupy all 64 incoming
  slots whatever the per-peer value is.
- The real bounds are global: `max_established_incoming(64)`,
  `max_established_outgoing(128)`, `max_pending_outgoing(32)`.
- `max_pending_incoming` is set nowhere I read, so pending inbound handshakes are
  unbounded. That is the actual file-descriptor exhaustion path.
- Answer to 1b: moving per-peer from 4 to 8 or 16 does not raise the worst-case
  file-descriptor or memory bound, because the global caps are unchanged. On
  train it does let one peer id hold 25% of the incoming slots (16 of 64).

### LOW

- **L1.** #372: `path_last_activity.insert(...)` on Ping, request-response and
  identify events does not check that the connection is still tracked. A
  connection the trim already closed can be re-stamped before its
  `ConnectionClosed` arrives, which causes the small leak noted in M1.
- **L2.** #361, #364 and #359 have no wasm parity for the retained-bound trim,
  and their bookkeeping uses `std::time::Instant` (fine only if it stays inside
  the native-only loop). #372 adds the wasm parity and uses `web_time::Instant`.
- **L3.** #364 and #413 add a 120 s outbox sweep (`OUTBOX-SWEEP-001`). It calls
  `handle_peer_connection_event_with_egress(pk, true, false, ...)` for every
  connected registered peer, which logs `info!` "Peer identified; triggering
  outbox flush" (`iron_core.rs:3334`) for each peer every 2 minutes. That is log
  noise, not a security issue. The peer set is bounded by live connections.
- **L4.** #372 tests call `release_path(paths, ...)` and then `drop(paths)`, and
  elsewhere pass `&mut paths` where `paths: &mut Vec`. I believe these compile
  through implicit reborrow and deref coercion, but did not prove it; CI must
  confirm.

### Priority-1 answers

**a. Enforcement.**
- **Admission:** enforced by libp2p's `connection_limits::Behaviour`. It is a
  field of the composed `IronCoreBehaviour` (`behaviour.rs:77`), and the PRs
  change only the value passed to `with_max_established_per_peer`. libp2p checks
  it in the established-connection handlers for both inbound and outbound
  connections.
  - On outbound dials it is checked only once the connection is established, not
    when the dial starts, so the handshake cost is paid before a deny.
  - This relies on libp2p's documented behaviour; I did not read the libp2p
    source.
- **Retained bound of 4 (#359/#361/#364/#372):** really enforced. The
  `ConnectionEstablished` arm calls `connections_to_close` /
  `note_established_path` and then `swarm.close_connection(extra)`, in the native
  loop in all four and also in the wasm loop in #372. It is not a dead constant.
  - Trimming only ever touches the same Noise-authenticated peer id, so one peer
    cannot close another peer's paths.
- **#413:** keeps a single-tier cap of 4 and changes nothing about it.

**b. Train branch:** both constants are consulted (`behaviour.rs:566` and
`:572`). There is no retained-bound trim; ghost slots are reaped separately
(`swarm.rs:1258`, `:4331`, `:4619`, which I did not review in depth).

**c. Agreement:** they do not agree. The values are 8 (#372), 16 (#359/#361/#364,
and train), and 4 (#413, unchanged). See C1.

### Priority-2 answers

- No PR touches `addr_filter.rs` or the ghost-peer guard. The `addr_filter`,
  `health`, `manager`, `wifi_*` deltas in the two-dot diffs come from main
  having moved on, not from the PRs. Three-dot diffs for #372, #361, #364 and
  #359 show no wifi changes, and #413 changes only `swarm.rs`.
- The self-circuit guard in `build_relay_addresses` (#372) fails closed. It
  `continue`s past a relay that is the target itself, a nested `P2pCircuit`
  address, a DNS-only or portless address, or a duplicate. I found no path that
  returns ALLOW on error.

### Priority-3 answers

No `unwrap`, `expect` or indexing on network input in the production code these
PRs add; `unwrap()` and `routes[0]` appear only in tests. The new code takes no
nested locks: `CircuitRelayLadder` uses one `parking_lot::RwLock`, held per
call.

### Mandatory disclosure: what I read

- **#372:** full `per_peer_cap.rs`. Full three-dot diffs of `behaviour.rs`,
  `mod.rs`, `swarm.rs` (native and wasm) and `dial_policy.rs`. The `add_relay`
  body and its `swarm.rs` call site (~6570-6620).
- **#361:** full three-dot `swarm.rs` diff, plus the `ConnectionClosed` arms
  (~7215-7315). Constants and wiring only, via grep, for `per_peer_cap.rs` and
  `behaviour.rs`.
- **#364:** full three-dot `swarm.rs` diff. Constants and wiring via grep.
- **#359:** full three-dot `swarm.rs` diff (its base is an older main).
  Constants and wiring via grep.
- **#413:** full three-dot transport diff (`swarm.rs` only).
- **Train:** grep of `behaviour.rs` / `swarm.rs` for cap constants.
- **Not read in full:**
  - The `per_peer_cap.rs` bodies and tests of #359, #361 and #364. They appear
    to be an earlier version of #372's file, but I did not confirm that line by
    line.
  - The rest of `swarm.rs` outside the diff hunks, including train's ghost-slot
    reaper.
  - libp2p's `connection_limits` source.

Findings M1-M3 and L1-L4 are scoped to that coverage. C1 is fully verified
across all five heads and the train branch.

---

## Post-review correction (by the requester, from source)

The C1 value conflict is **confirmed**. The stated "hidden failure mode" is
**partly wrong** and is corrected here so a future reader does not act on it.

In #372, #359, #361 and #364 the call site reads the module constant directly:

```rust
.with_max_established_per_peer(Some(
    super::per_peer_cap::ADMISSION_MAX_ESTABLISHED_PER_PEER,
))
```

So for those four the constant **is** consulted on the hot path, and the
"constant asserted in a test but never wired" failure does not arise. The
statement "no test ties the module constant to the builder call" is true only
in the weaker sense that no test asserts the *number*; the wiring itself is
correct in four of five.

**The real hazard is #413, and it is worse.** #413 hardcodes
`with_max_established_per_peer(Some(4))` -- a literal, not the constant -- and
its `behaviour.rs` predates and therefore **lacks both compile-time floor
assertions**; it has no `MAX_ESTABLISHED_PER_PEER_FLOOR` at all. If #413's
`behaviour.rs` wins any part of the rebase, the resolved file can enforce 4
while carrying no floor guard, and no test in the cluster would notice.

Verified by reading each head directly; see
`HANDOFF/review/PR_BOARD_DISPOSITION_2026-10-04.md` section 3.