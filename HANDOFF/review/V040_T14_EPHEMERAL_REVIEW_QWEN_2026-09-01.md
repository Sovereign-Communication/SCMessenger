# V040-T14 P0 (ephemeral port) -- ADVERSARIAL REVIEW (qwen free lane)

Reviewer: **qwen3-30b-a3b-thinking-2507** (DashScope free, non-author; qwq-plus
non-responsive -> same-tier fallback). Target: PR #270 `freebuff/v040-t14-ephemeral-port`.
Dispatched 2026-09-01 via tmp/qwen_review_dispatch.py. Status: **REQUEST_CHANGES
(one substantive finding -- verified REAL below) + verification addendum by the lane.**

## Lane resolution (2026-09-01 -- supersedes the first addendum; finding REJECTED)

The first addendum (below) marked the reviewer's Critical "verified REAL". Full
investigation on the branch then showed the premise was FALSE -- and the first
addendum repeated the same error (it checked the site's local vicinity, not the
enclosing cfg):

- Both promotion sites (swarm.rs:4061 consensus, :5185 Identify) sit inside the
  `#[cfg(not(target_arch = "wasm32"))]` event loop -- the entire native swarm
  construction of `start_swarm_with_config` is wrapped in that cfg block.
- The wasm32 branch has its OWN event loop: its Identify handler (swarm.rs:7956)
  and address-reflection handler (:7584) record observations for diagnostics
  parity only. File-wide `add_external_address` count = 2, both native; the wasm
  build has zero promotion calls.
- The wasm branch states the transport posture: "Browser nodes cannot open TCP
  listeners... Listen => unsupported, GetListeners => empty."
- Residual-2 as originally filed ("wasm accept-all default -- no promoters exist
  there, diagnostics parity only") is ACCURATE.
- Accept-any-on-empty was considered and REJECTED: it cannot help wasm (no
  promotion path exists there) and would weaken the guard on any build that
  reaches the sites with no bound listeners -- the exact ephemeral-source-port
  class this P0 removes.
- Resolution: guard kept unchanged; commit `6fd0230b` (comments only, +16/-1)
  documents the empty-set semantics at both guards and the wasm
  diagnostics-only parity at the recording site. Effective disposition for the
  finding: APPROVE (no defect). Gates re-run green; PR #270 body updated.

## First addendum (2026-09-01, superseded -- marked the finding REAL; see above)

The reviewer's Critical (empty listen_ports on wasm never advertises) was checked
against the tree and is **VALID as a documentation/behavior gap**:
- The Identify promotion site (swarm.rs:5193) is NOT wasm-gated; identify is
  configured unconditionally (behaviour.rs:72,509). On the wasm build,
  bound_addresses stays empty -> listen_ports empty -> the new guard refuses,
  where the pre-PR code advertised unconditionally.
- Consequence: the PR body's residual-2 claim ("wasm has no promoters -- parity
  only") is factually wrong at this shared site.
- Suggested disposition for the author (not applied): make the guard accept-any
  on an empty listen set (`listen_ports.is_empty() || listen_ports.contains(...)`)
  to restore the documented wasm parity, or update residual-2 to state the new
  behavior. Reviewer's suggested fix "remove the guard" is NOT accepted (the
  guard is the defense-in-depth layer; empty-set handling is the right fix).
- Impact scope is limited (browser nodes connect via WS/WebRTC, not advertised
  TCP addrs) -- but the parity claim must be corrected either way.
- The two Low findings (redundant filters) are acceptable-as-is; the timing
  finding ("bound_addresses updated before set_listen_ports") is benign.

Final: ANALYSIS only -- the Rule-8 APPROVE decision stays with the CEO seat.

# Adversarial Security Review: PR #270

## Severity: Critical
**Evidence**: `observation.rs` line 64-65: `if !self.listen_ports.is_empty() && !self.listen_ports.contains(&address.port())`\\
**Issue**: Empty `listen_ports` (wasm) is treated as "accept all", but `swarm.rs` line 5337: `address_observer.set_listen_ports(listen_ports_from_multiaddrs(&bound_addresses))`\\
**Analysis**: `bound_addresses` is empty for wasm (no listeners), so `listen_ports_from_multiaddrs` returns empty `listen_ports`. This causes `record_observation` to accept all observations (correct), but `swarm.rs` line 5185: `if listen_ports.contains(&primary.port())` fails for all addresses (since `listen_ports` is empty), causing `swarm` to never advertise any external address. This breaks the mesh for wasm clients.

## Severity: Medium
**Evidence**: `swarm.rs` line 5185: `if listen_ports.contains(&primary.port())`\\
**Issue**: Defense-in-depth check is redundant with `AddressObserver` filtering, but fails to handle empty `listen_ports` correctly.\\
**Analysis**: For non-wasm nodes, `listen_ports` is non-empty, so the check works. For wasm, `listen_ports` is empty, so the check fails and logs a warning, but the mesh still works because `AddressObserver` accepts all observations. However, the redundant check is unnecessary and could cause confusion.

## Severity: Low
**Evidence**: `observation.rs` line 142: `if !self.listen_ports.is_empty() && !self.listen_ports.contains(&obs.address.port())`\\
**Issue**: The `recalculate_consensus` filter is redundant with `record_observation` filtering.\\
**Analysis**: The `record_observation` filter already drops invalid observations, so `recalculate_consensus`'s filter is unnecessary. This is a minor performance overhead but not a security issue.

## Severity: Low
**Evidence**: `swarm.rs` line 5337: `address_observer.set_listen_ports(listen_ports_from_multiaddrs(&bound_addresses))`\\
**Issue**: `bound_addresses` is updated before `set_listen_ports` is called, but `bound_addresses` may contain addresses with ports not in the current listener set.\\
**Analysis**: This is correct behavior. `bound_addresses` contains all current listeners, so `listen_ports_from_multiaddrs` correctly extracts the current listen ports.

## Verdict
REQUEST_CHANGES

The PR breaks wasm mesh functionality by failing to advertise external addresses when `listen_ports` is empty. The defense-in-depth check in `swarm.rs` should be removed or fixed to handle empty `listen_ports` correctly.