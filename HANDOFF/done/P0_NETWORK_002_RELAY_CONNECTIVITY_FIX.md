# P0_NETWORK_002: Relay Connectivity Fix & Fallback Implementation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 CRITICAL
**Platform:** Core (Rust) + All Platforms
**Status:** IN_PROGRESS
**Assigned:** implementer (qwen3-coder-next:cloud)
**Routing Tags:** [REQUIRES: NETWORK_SYNC] [REQUIRES: FINALIZATION]

## Objective
Fix relay server connectivity failures and implement proper fallback mechanisms in the Rust core. From ANDROID_PIXEL_6A_AUDIT_2026-04-17, all 4 relay servers are failing with generic "Network error" messages. P0_NETWORK_001 (Android-side racing bootstrap) is DONE; this task addresses the Rust core transport layer.

## Evidence
- Cargo.toml change: Added tokio-tungstenite WebSocket client dependency
- REMAINING_WORK_TRACKING.md: ANR-002 - "Network Bootstrap Complete Failure"
- Relay servers failing: 34.135.34.73, 104.28.216.43 (both UDP/TCP)

## Implementation Plan

### 1. Enhanced Error Diagnostics in Rust Core
**File:** `core/src/transport/bootstrap.rs`
- Add `diagnose_connection_error()` to classify io::Error into RelayError variants
- Variants: ConnectionRefused, Timeout, DnsResolutionFailed, TlsFailed, PortBlocked, Generic
- Integrate with existing BootstrapManager error handling

### 2. WebSocket Fallback Transport
**File:** `core/src/transport/websocket.rs` (already exists with scaffold)
- Complete the WebSocketTransport implementation using tokio-tungstenite
- Support WSS (port 443) and WS (port 80) for cellular network compatibility
- Implement from_multiaddr() to convert libp2p multiaddr to WebSocket URL
- Integrate with the libp2p swarm's .with_websocket() transport

### 3. Multi-Transport Bootstrap with Racing
**File:** `core/src/transport/bootstrap.rs`
- Replace sequential bootstrap with racing: dial all transports to a relay in parallel
- Fallback order per relay: QUIC  TCP  WebSocket (WSS on 443, WS on 80)
- Circuit breaker integration (already exists in core/src/transport/circuit_breaker.rs)
- First successful connection wins

### 4. Alternative Bootstrap Sources
**File:** `core/src/transport/bootstrap.rs`
- DNS-based bootstrap node discovery (discover_fallback_nodes already scaffolded)
- Hardcoded backup relay addresses on standard ports
- Environment variable override (SC_BOOTSTRAP_NODES already supported)

## Files to Modify
1. `core/src/transport/bootstrap.rs` - Enhanced error diagnostics + racing bootstrap
2. `core/src/transport/websocket.rs` - Complete WebSocket transport
3. `core/src/transport/mod.rs` - Multi-transport bootstrap routing
4. `core/src/transport/circuit_breaker.rs` - Verify compatibility with racing

## Success Criteria
- Detailed error diagnostics for all connection failure types
- WebSocket fallback transport working with WSS/WS on standard ports
- Multi-transport bootstrap with parallel racing
- Circuit breaker integration verified
- cargo build --workspace passes

## Priority: CRITICAL
Required for network reliability. P0_NETWORK_001 (Android side) is done; this completes the Rust core side.