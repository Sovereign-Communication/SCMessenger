# Loopback Dial Design for Wi-Fi Aware

## Problem
Wi-Fi Aware transport on Android uses loopback address (127.0.0.1) for proxy, but the address filter rejects it for `NetworkMode::Local`, breaking the transport. The security tests require loopback to be rejected in all modes, but this is a special case that should be allowed.

## Design Choice: Option (b) - Trusted Dial Flag
We choose to add a `trusted: bool` field to `SwarmCommand::Dial`. The dial filter will skip the loopback rejection for trusted dials.

### Why not Option (a) or (c)
- Option (c) is infeasible: the Wi-Fi Aware peer requires a link-local IPv6 address with scope-id, which the Multiaddr parser cannot handle. Using loopback is necessary to avoid scope-id.
- Option (a) (new `NetworkMode`) would be more invasive (changing an enum used in multiple places) and unnecessary.

## Security Argument
- **Four tests remain passing**: The existing security tests run with untrusted dials (without `trusted=true`). We only change behavior for trusted dials, so the tests still pass.
- **Loopback only allowed for Wi-Fi Aware**: The `trusted` flag is set only in the Wi-Fi Aware transport (by the Android code), so only those dials are allowed to use loopback.
- **Untrusted input unaffected**: All untrusted input uses `NetworkMode::Public` or `NetworkMode::Local` without the `trusted` flag, so loopback is still rejected for them.
- **Loopback not disclosed to remote peers**: The loopback address is used as a local proxy on the same device, so it is not advertised to the network (only sent to the Wi-Fi Aware peer on the same device). This satisfies the non-disclosure requirement.

## Implementation Plan
1. Add `trusted: bool` to `SwarmCommand::Dial` in `core/src/transport/swarm.rs`
2. Add `trusted: bool` parameter to `is_unconditionally_routable_ipv4`
3. In `is_unconditionally_routable_ipv4`, for `NetworkMode::Local`, allow loopback only when `trusted` is `true`
4. In `is_dialable_multiaddr_parsed`, pass `trusted` from command to address filter
5. Update Android Wi-Fi Aware transport to set `trusted=true` for dial commands

## New Tests to Add
- `core/tests/integration_wifi_aware.rs`: Verify Wi-Fi Aware loopback dial succeeds
- `core/src/transport/addr_filter.rs`: Verify attacker-supplied loopback address is rejected

## Verification
- All four security tests (`rejects_non_routable_ipv4_in_every_mode`, `ipv4_mapped_ipv6_cannot_bypass_the_ipv4_rules`, `the_circuit-relay-hop_loopback_assertion`, `acceptable_peer_address_combines_both_gates`) must pass unmodified
- Loopback address in Wi-Fi Aware transport must dial successfully
- Attackers must not be able to use loopback address via untrusted paths
