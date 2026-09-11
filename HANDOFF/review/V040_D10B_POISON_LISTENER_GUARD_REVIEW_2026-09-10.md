# V040 D10b — Poison-listener event-loop guard (rule-8 review packet)

- Filed: 2026-09-10 ~06:40Z by the CTO seat
- Perimeter: `core/src/transport/swarm.rs` (rule-8 gated: crypto/transport/
  routing/privacy). Branch work proceeds per T14/D10 precedent; MERGE TO MAIN
  BLOCKED until an independent adversarial APPROVE is recorded in this
  directory.
- Related: D10 packet (`V040_D10_REVIEWER_DISPATCH_PACKET_2026-09-10.md`) —
  this is the round-2 hardening of the same defect class.

## Change under review

`is_poison_circuit_listener(address, is_tracked_reservation)` plus a guard in
the `SwarmEvent::NewListenAddr` handler: any listener address containing
circuit segments is removed via `swarm.remove_listener(listener_id)` unless it
is a tracked single-circuit relay reservation (tracked =
`successful_relay_reservations` contains its ListenerId; single-circuit
enforced by the predicate).

## Why the D10 base gate alone was insufficient (live evidence)

Round 2, Windows node, 2026-09-10T05:05Z hour log
(`scm.log.2026-09-10-05:529+`, recurring every advertisement cycle):

```
WARN libp2p_mdns::behaviour::iface::dns: Excluding address from response:
TxtRecordTooLong address=/ip4/172.31.18.74/tcp/9090/p2p/12D3KooWGvCWJ.../p2p-circuit/
p2p/12D3KooWR9io.../p2p-circuit/p2p/12D3KooWD6vZ...
```

Shape analysis: the reported listener address carries p2p segments BETWEEN two
circuit segments. The guarded reservation path (`relay_reservation_multiaddr`)
strips ALL p2p/circuit segments from the base before appending
`p2p/<relay>/p2p-circuit`, so a listener produced by that path can never have
this shape. Conclusion: a second listen_on path (relay-assist / peer-broadcast
reactive listener) registered a raw circuit-route listener on the D10 tree.
The D10 validate-at-use gate only covers the identify-driven reservation path.

## Fix rationale (root-cause class closure)

Rather than hunting every listen_on call site forever, the guard closes the
CLASS at the single point where every listener becomes visible: the event
loop. Enforcement there is (a) total — no listener escapes it, (b) minimal —
legitimate tracked reservations pass untouched, (c) observable — a WARN line
with the removed address.

Security consideration (reviewer attention point): `remove_listener` on a
poison circuit listener also aborts any legitimate relay reservation that the
guarded path did not track (e.g. created by a future call site). The intended
invariant is: ALL circuit listeners must be created through the guarded
reservation path. Reviewers should verify no other listen_on call site
intentionally creates circuit listeners — the test
`poison_listener_guard_rejects_untracked_single_circuit` documents the
strictness.

## Test plan

Three new unit tests in `swarm.rs` test module:
1. `poison_listener_guard_rejects_nested_double_circuit` — the exact live
   poison address from the 05:05Z log; poison regardless of tracked flag.
2. `poison_listener_guard_rejects_untracked_single_circuit` — single circuit,
   untracked = poison; same address tracked = legitimate.
3. `poison_listener_guard_passes_direct_listeners` — plain TCP + loopback
   listeners never poison.

Gates run under the build lock: `cargo fmt --check`,
`cargo clippy -p scmessenger-core -- -D warnings
-A clippy::empty_line_after_doc_comments` (CI-exact invocation),
`cargo test -p scmessenger-core --lib poison_listener_guard`.

## Operational verification (post-deploy)

Windows node log must show ZERO `TxtRecordTooLong` lines in a full
advertisement cycle with mesh active; any `[D10b] Poison-listener guard:`
WARN indicates the guard caught an attempt (and names the source address for
follow-up).

## Reviewer verdict

PENDING — independent reviewer required (not the author).

### Dispatch attempt log (2026-09-10, CTO seat, all lanes exercised)

| Lane | Model | Result |
|---|---|---|
| qwen (tier=thinking) | qwen3-30b-a3b (rotated) | Responded but output was a 6-line verdict shell with ZERO analysis — invalid as adversarial review evidence |
| qwen (--model qwen-max) | qwen-max -> qwen3-14b (rotated) | 400: input length limit 30,720 tokens (full swarm.rs too large); retried 14b — degenerate repetition-loop echo, unusable |
| qwen (regions restructure) | qwen3-coder-plus -> qwen3-coder-plus-2025-09-23 | Degenerate repetition-loop echo (383 lines of source echo), unusable |
| gemini | gemini-2.5-flash | API key not configured |
| groq | llama-3.3-70b-versatile | 404 model not found; retries exhausted |
| qwenpaid | (retired) | Removed by operator ruling 2026-08-31 — must not restore |

Conclusion: the free review lanes are currently UNABLE to produce a valid
rule-8 verdict (capacity/degeneracy/credential failures). Reviewer assignment
escalated to the operator: MAC lane (GPT/Codex) or any human/model reviewer
the operator picks. Dispatch packet: this file +
`tmp/cto/REV_D10/REVIEW_TASK_D10_D10B.md` + diffs under `tmp/cto/REV_D10/`.
MERGE TO MAIN REMAINS BLOCKED until an independent APPROVE is recorded here.
