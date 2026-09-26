# B1 DNS-name-first hardening (FARM AD-2, IP-flip mandate) - attempt 1 held

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: IMPLEMENTED + reviewed, NOT committed. Working tree reverted clean;
attempt preserved as a patch for a second cycle. `[AUDIT-GATE]` (transport/).

## What attempt 1 did (agy/Gemini, compiled clean)
Patch: `HANDOFF/review/B1_DNS_HARDENING_ATTEMPT1.patch` (apply with
`git apply` from repo root). Changes `core/src/transport/swarm.rs` +
`core/src/mobile_bridge.rs`:
- (a) resolver `cache_size = 0` so DNS hostnames re-resolve on dial failure.
- (b) on `OutgoingConnectionError` for a `/dns*` bootstrap addr, apply the
  backoff/negative penalty only to the failed RESOLVED IP(s) from
  `DialError::Transport`, not to the hostname multiaddr, so the hostname stays
  dialable after an IP flip. Added `is_dns_multiaddr` helper.
- (c) `LedgerManager::record_connection`/`annotate_identity` preserve an
  existing DNS-hostname entry (matched by peer id + port) instead of
  overwriting it with a raw resolved IP.
`cargo check -p scmessenger-core` was clean.

## Why it was held (free-lane adversarial review, qwen3-235b-thinking)
Full verdict: `HANDOFF/review/B1_DNS_HARDENING_REVIEW.md`. Two legitimate
concerns on this audit-gated path, neither cleanly resolved:
1. `cache_size = 0` disables DNS caching GLOBALLY for every dial, not just the
   farm-anchor domains - over-broad; adds resolver load on a mesh that dials
   often. Wants per-domain scoping, not a global knob.
2. Backoff-only-on-resolved-IP risks under-throttling a genuinely-dead
   hostname (reviewer wanted the hostname backed off too). NOTE: this is
   partially mitigated already - per-IP backoff + re-resolution does throttle
   the common case (a dead IP gets backed off; a flipped IP is a fresh entry).
   The real open question is the tension between goal (b) "don't poison the
   hostname" and "don't tight-loop on a hostname whose every resolved IP is
   dead" - needs an explicit, gentler hostname-level backoff that still allows
   re-resolution, which neither attempt 1 nor the reviewer's over-correction
   (which reverted (a) and (b) wholesale) got right.

## What attempt 2 needs
- Scope the re-resolution/cache behavior to farm-anchor/DNS-named peers, not a
  global resolver `cache_size = 0`.
- Keep goal (b) (don't hard-poison the hostname) but add a SEPARATE, gentler
  hostname-level throttle so an all-IPs-dead hostname doesn't tight-loop while
  still re-resolving to catch an IP flip.
- Re-run the transport `[AUDIT-GATE]` review before commit.
Not beta-blocking - farm-reach-lane resilience for the DDNS anchor; the app
functions without it. B2 (bootstrap unification) already landed.
