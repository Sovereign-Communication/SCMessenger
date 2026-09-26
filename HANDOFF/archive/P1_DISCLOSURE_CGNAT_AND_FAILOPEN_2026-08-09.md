# P1 -- Disclosure gate: CGNAT /24 collision + concrete_local_ips fail-open

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active -- DEFERRED BY OPERATOR to the iteration after the first
five-node test, and to be fixed BEFORE the second five-node test.
Severity: MEDIUM x2 (neither is a regression introduced by PR #139)
Filed: 2026-08-09 (Windows/Claude lane)
Source: `core/src/transport/addr_filter.rs`, `is_disclosable_on_rfc1918_network`
Found by: independent model review (deepseek-v4-flash-0731), premises verified
against the code by the Windows lane before filing.

Operator decision 2026-08-09: *"Predated issues are still issues - fix it!
However we're gonna deploy for 5 node test now, and we can address D1/D2 for
next iteration before the following 5node test."*

## D1 -- CGNAT /24 collision discloses to an unrelated carrier subscriber

`is_safe_private_ip` accepts CGNAT:

```rust
&& (ip.is_private() || is_cgnat(ip))       // is_cgnat = 100.64.0.0/10
```

and `same_private_subnet` compares a /24:

```rust
let left = left.octets(); let right = right.octets();
left[..3] == right[..3]
```

Carrier-grade NAT pools routinely place **unrelated subscribers** in the same
/24. So:

| variable | value |
|---|---|
| `my_addrs` | `/ip4/100.64.0.10/tcp/9001` (us, behind CGNAT) |
| `requester_addr` | `/ip4/100.64.0.11/tcp/9001` (attacker, same carrier pool) |
| entry | `/ip4/100.64.0.12/tcp/9001` |

All three pass `is_safe_private_ip`; entry and requester share a /24; the
requester matches our /24. The gate returns **true** and we disclose private
ledger entries to a peer who is not a LAN neighbour in any meaningful sense.

**Why it matters here specifically:** this fleet tests on cellular. The
2026-08-08 cellular-only leg put Android on `rmnet16` with `10.16.109.218/32`
(carrier-NAT interface). A mobile node on CGNAT is not hypothetical for this
project.

**Fix:** require RFC1918 proper for the *requester* leg -- drop CGNAT from the
"proves local adjacency" role. CGNAT can remain acceptable as a *dialable*
class; it simply cannot serve as evidence of L2 adjacency.

## D2 -- `concrete_local_ips.is_empty()` is a fail-open

```rust
concrete_local_ips.is_empty()
    || concrete_local_ips.iter().any(|local_ip| same_private_subnet(local_ip, &requester_ip))
```

When no concrete local IP can be derived, the gate stops consulting `my_addrs`
entirely and discloses on the entry-requester subnet match alone. The inline
comment documents this as deliberate ("the authenticated direct observed address
remains the source of truth in that case").

The concern: **the original F1 finding was precisely that `my_addrs` gated
nothing.** This branch reintroduces a path where `my_addrs` gates nothing. It is
much narrower than F1 -- requester and entry must still share a /24, and the
requester address is observed rather than self-reported -- but it is
structurally the same shape the adversarial review called CRITICAL.

Reachable when listeners yield no IP component: DNS-only listeners, loopback-only
(filtered by `is_safe_private_ip`), or an empty listener set during early startup
before `NewListenAddr` has fired.

**Fix:** return `false` when `concrete_local_ips` is empty. Absence of evidence
about our own network is not evidence of the requester's adjacency.

## Regression tests required

- CGNAT requester on the same /24 as both us and the entry must receive **zero**
  entries.
- With `my_addrs` yielding no IP component, a same-/24 requester must receive
  **zero** private entries.

Both must be written to fail against the current code before the fix lands,
following the F4 pattern -- the existing suite passes while both conditions hold.

## Explicitly NOT included

The same review raised two further items that were correctly self-limited and
are **not** security issues:

- `first_ip_component` ordering with multiple IP components -- causes denial of
  legitimate disclosure, not a bypass.
- The fixed /24 mask ignoring the real interface prefix -- a /16 LAN sees
  legitimate neighbours denied. Functionality, not security.
