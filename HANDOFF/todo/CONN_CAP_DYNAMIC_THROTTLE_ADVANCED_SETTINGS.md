# TODO: dynamic per-device/situation connection throttling in Advanced Settings

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Repository: Sovereign-Communication/SCMessenger (issues #417 and #418)
Status: the stopgap (per-peer cap 4 to 64) is on branch `fix/conn-cap-per-peer-64`,
PR unmerged and not yet through the adversarial review; everything below is OPEN.
Opened: 2026-09-29, at the operator's request
Tracking: issue #418 (this item). Root cause, retroactive review and the design
follow-ups: issue #417.

## Follow-ups, in order (resolve soon; do not let the stopgap become permanent)

1. [ ] Retroactive Rule-8 review of the cap change (`core/src/transport/`), findings
   dispositioned. Owner: an uninvolved reviewer. Tracked in #417.
2. [ ] Dial-candidate sanity pre-filter, so the larger cap is not spent on
   connections that can never work. On the phone 26 of 133 dials (20%) went to
   Docker `172.17.0.1` or the relay's VPC `172.31.18.74`, and loopback self-dials
   hit its own listener ("Unexpected peer ID <own>"). Rule: a remote-supplied
   address is invalid if it is loopback, unspecified, link-local, multicast, one
   of our own addresses, or private/ULA/CGNAT and not in the same private block
   as one of our own interface addresses. Apply it to remote-supplied candidates
   only (explicit dials and loopback tests must keep working). One pure function
   in `addr_filter.rs`, one call site, unit tests for accept and reject.
   If it cannot be made that safe, the fallback the operator named is a cap of 16.
3. [ ] Reconcile with MT-05 (`per_peer_cap.rs`, supersedes #361 and #372) and MT-08
   (T-CONN-04 port-ladder dedupe, the #359 remainder): keep the cap at 64 or
   above and land exactly one tier.
4. [ ] Re-run TRI-040 case C7 and a phone-side cell handover on a build with the
   stopgap: expect zero `limit ... reached` denials on the relay.
5. [ ] Design fixes so a handover never depends on the cap: evict the oldest
   connection of an authenticated peer instead of denying the new one; close
   every connection on a network-type change on the phone; a shorter ping
   interval and timeout; attribute a denial to a peer rather than an IP.
6. [ ] Dynamic per-device/situation tuning in Advanced Settings (the rest of this
   file), tracked in #418.

## Why this exists

The per-peer connection cap in `core/src/transport/behaviour.rs`
(`with_max_established_per_peer`) is one fixed constant. Four (the value on
`main` at 2026-09-29) starved a Wi-Fi to cellular handover: the relay kept the
phone's dead Wi-Fi sockets booked and denied every fresh cellular connection
for 47 to 58 seconds (see #417). The operator asked for the default to be
raised (64, or 16 if the accompanying dial pre-filter is judged too risky) and
for this item to be recorded so the value can later be tuned per device and per
situation instead of being a single number.

## Requirements

1. Advanced Settings exposes the per-peer cap and the global incoming and
   outgoing caps, each with a sane minimum and maximum, a default and a
   reset-to-default. Values are persisted with the other node settings and
   reported by `/api/diagnostics`.
2. An optional automatic policy, OFF by default, that adjusts the limits by:
   - device class (phone, desktop, always-on relay);
   - network type (Wi-Fi, cellular, BLE), including the moment of a handover;
   - metered connection, battery saver and Doze.
3. The limits must be changeable at runtime. `connection_limits::Behaviour` is
   fixed when the swarm is built, so the design must either rebuild the
   behaviour or replace it with a small limiter owned by the swarm. Prefer the
   second: it also allows "evict the oldest connection of an authenticated peer
   instead of denying the new one".
4. Every deny log line keeps naming the limit and which tier applied.
5. Bounds are enforced in the core, not only in the UI, so a bad setting cannot
   remove the DoS control.

## Design notes to carry forward

- The relay's ZOMBIE-CONNECTION REAP (`core/src/transport/swarm.rs`) attributes
  a denied dial by source IP; a Wi-Fi to cellular handover changes the source
  IP, so the reap never fires there. Any new design should attribute by peer or
  avoid needing to.
- The phone dials many addresses per peer (6 ports of one Windows node) and
  some that can never work from a phone (Docker `172.17.0.1`, the relay's VPC
  `172.31.18.74`, loopback). A sanity pre-filter of remote-supplied dial
  candidates, plus MT-08's port-ladder dedupe, keeps the slots for valid
  connections. Do not raise the caps further without that.
- MT-05 (D9 + D1, two-tier cap in `per_peer_cap.rs`) and MT-08 (T-CONN-04
  dedupe) touch the same code. Reconcile with them first; never land two
  competing tiers.
- `core/src/transport/` requires the adversarial (Rule-8) review before merge.

## Acceptance

- Unit tests: bounds, persistence, a runtime change with live connections, and
  a handover scenario with ghost connections (fresh connections admitted within
  one retry cycle).
- Live: TRI-040 case C7 (Windows to Pixel with Wi-Fi off) delivers, and the
  relay logs zero `limit ... reached` denials for the handover.
