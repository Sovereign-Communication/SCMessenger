#!/usr/bin/env python3
"""Fix the 10 untriaged perimeter underscore-param violations (PR #228 gate).

Byte-level CRLF-safe replacements on the current branch (ci-hardening-228).
"""
import io, sys

CRLF = "\r\n"

def patch(path, replacements):
    with open(path, "rb") as f:
        data = f.read().decode("utf-8")
    for old, new in replacements:
        if old not in data:
            print(f"!! MISS in {path}: {old[:70]!r}")
            sys.exit(1)
        data = data.replace(old, new, 1)
    with open(path, "wb") as f:
        f.write(data.encode("utf-8"))
    print(f"patched {path}")

# ---------------- ratchet.rs (4 comment justifications) ----------------
patch("core/src/crypto/ratchet.rs", [
    (
        "    /// `init_as_receiver_hybrid_suite02`.\n"
        "    pub fn init_as_receiver_hybrid(",
        "    /// `init_as_receiver_hybrid_suite02`.\n"
        "    // PERIMETER-ALLOW-UNDERSCORE: receiver-side sender authentication is\n"
        "    // X25519-based (our_x25519_secret x sender_bundle.x25519_public) in the\n"
        "    // 0x03 derivation; the receiver's Ed25519 signing key never participates.\n"
        "    pub fn init_as_receiver_hybrid(",
    ),
    (
        "    /// `crypto::negotiation::HYBRID_SUITE_IDS` for why.\n"
        "    pub fn init_as_sender_hybrid_suite02(",
        "    /// `crypto::negotiation::HYBRID_SUITE_IDS` for why.\n"
        "    // PERIMETER-ALLOW-UNDERSCORE: suite 0x02 has no sender-authentication DH\n"
        "    // term by design; the signing key must stay unused to preserve the 0x02\n"
        "    // root-key derivation (\"Do NOT change this function's derivation\").\n"
        "    pub fn init_as_sender_hybrid_suite02(",
    ),
    (
        "    /// the suite 0x03 code path.\n"
        "    pub fn init_as_receiver_hybrid_suite02(",
        "    /// the suite 0x03 code path.\n"
        "    // PERIMETER-ALLOW-UNDERSCORE: suite 0x02 receiver derivation has no\n"
        "    // sender-authentication term: neither our signing key nor the sender\n"
        "    // bundle participates (preserved byte-for-byte; do not change).\n"
        "    pub fn init_as_receiver_hybrid_suite02(",
    ),
])

# ---------------- health.rs (use latency in global metrics) ----------------
patch("core/src/transport/health.rs", [
    (
        "    /// Total bytes received\n"
        "    pub total_bytes_received: u64,\n",
        "    /// Total bytes received\n"
        "    pub total_bytes_received: u64,\n"
        "    /// Average message latency in milliseconds (moving average)\n"
        "    #[serde(default)]\n"
        "    pub avg_latency_ms: u64,\n",
    ),
    (
        "            total_bytes_received: 0,\n"
        "            current_active_connections: 0,\n",
        "            total_bytes_received: 0,\n"
        "            avg_latency_ms: 0,\n"
        "            current_active_connections: 0,\n",
    ),
    (
        "    pub fn record_message_success(&mut self, bytes: u64, _latency_ms: u64) {\n"
        "        self.total_messages_sent += 1;\n"
        "        self.total_bytes_sent += bytes;\n"
        "    }\n",
        "    pub fn record_message_success(&mut self, bytes: u64, latency_ms: u64) {\n"
        "        self.total_messages_sent += 1;\n"
        "        self.total_bytes_sent += bytes;\n"
        "\n"
        "        // Update average latency (moving average), mirroring the per-peer\n"
        "        // TransportHealth::record_message_success.\n"
        "        if self.total_messages_sent > 1 {\n"
        "            self.avg_latency_ms = (self.avg_latency_ms + latency_ms) / 2;\n"
        "        } else {\n"
        "            self.avg_latency_ms = latency_ms;\n"
        "        }\n"
        "    }\n",
    ),
    (
        "            metrics.record_message_success(1024, 50);\n"
        "        }\n"
        "\n"
        "        let score = metrics.health_score();\n",
        "            metrics.record_message_success(1024, 50);\n"
        "        }\n"
        "        assert_eq!(metrics.avg_latency_ms, 50, \"moving average of uniform latency\");\n"
        "\n"
        "        let score = metrics.health_score();\n",
    ),
])

# ---------------- manager.rs (justified comment: DSPy recall stub) ----------------
patch("core/src/transport/manager.rs", [
    (
        "    /// Run multi-hop recall to retrieve transport paths for a peer.\n"
        "    /// Returns a list of potential paths through available transports.\n"
        "    pub fn run_multi_hop_path_selection(",
        "    /// Run multi-hop recall to retrieve transport paths for a peer.\n"
        "    /// Returns a list of potential paths through available transports.\n"
        "    // PERIMETER-ALLOW-UNDERSCORE: the DSPy multi-hop recall module is not yet\n"
        "    // wired into production routing (recall() is a stub returning empty -- see\n"
        "    // dspy/modules.rs); peer_id is reserved for peer-scoped recall when wired up.\n"
        "    pub fn run_multi_hop_path_selection(",
    ),
])

# ---------------- engine.rs (route_message uses now for stale pending gate) ----------------
patch("core/src/routing/engine.rs", [
    (
        "        priority: u8,\n"
        "        _now: u64,\n"
        "    ) -> RoutingDecision {",
        "        priority: u8,\n"
        "        now: u64,\n"
        "    ) -> RoutingDecision {",
    ),
    (
        "        let should_request = !self.global.is_route_pending(recipient_hint) && priority >= 100;",
        "        let should_request =\n"
        "            !self.global.is_route_pending_fresh(recipient_hint, now) && priority >= 100;",
    ),
])

# ---------------- global.rs (fresh-pending check + shared window const) ----------------
patch("core/src/routing/global.rs", [
    (
        "/// Global routing table \u2014 sparse, demand-driven",
        "/// How long a route request stays pending before it is considered stale and a\n"
        "/// new RouteDiscovery may be issued (must match cleanup()'s window).\n"
        "const MAX_ROUTE_REQUEST_AGE_SECS: u64 = 300; // 5 minutes\n"
        "\n"
        "/// Global routing table \u2014 sparse, demand-driven",
    ),
    (
        "    /// Check if a route request is pending for a hint\n"
        "    pub fn is_route_pending(&self, hint: &[u8; 4]) -> bool {\n"
        "        self.pending_requests.contains_key(hint)\n"
        "    }\n",
        "    /// Check if a route request is pending for a hint\n"
        "    pub fn is_route_pending(&self, hint: &[u8; 4]) -> bool {\n"
        "        self.pending_requests.contains_key(hint)\n"
        "    }\n"
        "\n"
        "    /// Check if a route request is pending AND still fresh (younger than the\n"
        "    /// staleness window). A stale pending request must not block a new\n"
        "    /// RouteDiscovery decision.\n"
        "    pub fn is_route_pending_fresh(&self, hint: &[u8; 4], now: u64) -> bool {\n"
        "        match self.pending_requests.get(hint) {\n"
        "            Some(req) => now.saturating_sub(req.requested_at) <= MAX_ROUTE_REQUEST_AGE_SECS,\n"
        "            None => false,\n"
        "        }\n"
        "    }\n",
    ),
    (
        "        // Also clean up old route requests (older than 5 minutes)\n"
        "        let max_request_age = 300; // 5 minutes\n"
        "        self.pending_requests\n"
        "            .retain(|_, req| now.saturating_sub(req.requested_at) <= max_request_age);",
        "        // Also clean up old route requests (older than 5 minutes)\n"
        "        self.pending_requests\n"
        "            .retain(|_, req| now.saturating_sub(req.requested_at) <= MAX_ROUTE_REQUEST_AGE_SECS);",
    ),
])

# ---------------- negative_cache.rs (recurring confirmations beat reputation) ----------------
patch("core/src/routing/negative_cache.rs", [
    (
        "    pub fn should_exempt_from_negative_cache(&self, _peer_id: &str, reputation_score: f64) -> bool {\n"
        "        // Peers with overall_score >= 0.5 are considered trusted and exempted\n"
        "        // This prevents marking good peers as unreachable due to transient issues\n"
        "        reputation_score >= 0.5\n"
        "    }",
        "    pub fn should_exempt_from_negative_cache(&self, peer_id: &str, reputation_score: f64) -> bool {\n"
        "        // A peer already negatively cached with repeated confirmations is a\n"
        "        // recurring (not transient) failure: reputation does not mask it.\n"
        "        if let Some(entry) = self.entries.get(peer_id) {\n"
        "            if entry.confirmation_count >= 2 {\n"
        "                return false;\n"
        "            }\n"
        "        }\n"
        "        // Peers with overall_score >= 0.5 are considered trusted and exempted\n"
        "        // This prevents marking good peers as unreachable due to transient issues\n"
        "        reputation_score >= 0.5\n"
        "    }",
    ),
])

# ---------------- neighborhood.rs (prefer evicting genuinely stale gateways) ----------------
patch("core/src/routing/neighborhood.rs", [
    (
        "    /// Evict the gateway with the stalest timestamp\n"
        "    fn evict_stalest_gateway(&mut self, _now: u64) {\n"
        "        if self.gateways.is_empty() {\n"
        "            return;\n"
        "        }\n"
        "\n"
        "        // Evict the gateway with the oldest last_updated timestamp\n"
        "        let gateway_to_evict = *self\n"
        "            .gateways\n"
        "            .values()\n"
        "            .min_by_key(|g| g.last_updated)\n"
        "            .map(|g| &g.gateway_id)\n"
        "            .expect(\"checked gateways non-empty above\");\n"
        "\n"
        "        self.gateways.remove(&gateway_to_evict);\n"
        "    }",
        "    /// Evict the gateway with the stalest timestamp\n"
        "    fn evict_stalest_gateway(&mut self, now: u64) {\n"
        "        if self.gateways.is_empty() {\n"
        "            return;\n"
        "        }\n"
        "\n"
        "        // Prefer evicting a gateway that is actually stale (last_updated older\n"
        "        // than the staleness window) so a burst of fresh updates never evicts\n"
        "        // current information; among stale gateways, evict the stalest. If every\n"
        "        // gateway is fresh, fall back to the stalest overall so the max_gateways\n"
        "        // capacity invariant still holds.\n"
        "        let gateway_to_evict = self\n"
        "            .gateways\n"
        "            .iter()\n"
        "            .filter(|(_, g)| now.saturating_sub(g.last_updated) >= self.max_staleness)\n"
        "            .min_by_key(|(_, g)| g.last_updated)\n"
        "            .map(|(id, _)| *id)\n"
        "            .or_else(|| {\n"
        "                self.gateways\n"
        "                    .iter()\n"
        "                    .min_by_key(|(_, g)| g.last_updated)\n"
        "                    .map(|(id, _)| *id)\n"
        "            })\n"
        "            .expect(\"checked gateways non-empty above\");\n"
        "\n"
        "        self.gateways.remove(&gateway_to_evict);\n"
        "    }",
    ),
])

print("ALL PATCHES APPLIED")
