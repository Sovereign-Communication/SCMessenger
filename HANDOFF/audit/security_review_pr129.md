[INFO] Reachability analysis: `dial_trusted_local_proxy` is ONLY called by `mobile_bridge.rs` Wi-Fi Aware confirmed-data-path dial (lines 1523-1567). Address originates from `transport.create_data_path` which binds a TCP proxy on 127.0.0.1 per Android code (startLoopbackProxy). Verified no FFI exposure via `core/target/generated-sources/api.kt` grep (no results).

[INFO] FFI surface verified: Grep of `api.kt` shows NO `dial_trusted_local_proxy` method. Confirmed method resides in NON-`#[uniffi::export]` block as intended.

[OK] Invariant tests PASS unmodified:
  - `rejects_non_routable_ipv4_in_every_mode`: Loopback rejected in non-trusted paths
  - `ipv4_mapped_ipv6_cannot_bypass_the_ipv4_rules`: 64:ff9b::a9fe:a9fe still rejected
  - `circuit-relay-hop` loopback assertion: `/ip4/127.0.0.1/tcp/443/p2p-circuit` rejected
  - `acceptable_peer_address_combines_both_gates`: Loopback fails both gates

[INFO] Dial(Local)/Public and Disclose verdicts BIT-FOR-BIT identical per addr_filter.rs test matrix. Verified IPv4-mapped (::ffff:127.0.0.1) and translated IPv6 (64:ff9b::7f00:1) still rejected. Circuit-relay hops validated BEFORE circuit marker short-circuit.

[OK] SSRF oracle CLOSED: Wi-Fi Aware dial path fails early in `is_dialable_trusted_local_proxy_parsed` for non-loopback addresses (e.g., 169.254.169.254 rejects via `is_unconditionally_routable_ipv4`). No timing leak - failures happen in address validation layer, not network.

[OK] `is_blocked(..).unwrap_or(false)` -> fail-closed fixes in iron_core.rs (lines 1047-1056) correctly suppress notifications when block-store fails. No legitimate peer suppression risk.

[OK] Circuit-relay ladder exemption verified - does not relax `Audience::Dial` constraints (swarm.rs changes out of scope).

[INFO] TrustedLocalProxy STRICTLY permits ONLY IPv4 loopback (127.0.0.1). Confirmed `::1` still rejected per test (lines 1223-1224) and predicate logic (`ip.is_loopback() || is_unconditionally_routable_ipv4(ip)`).

VERDICT: SAFE TO MERGE