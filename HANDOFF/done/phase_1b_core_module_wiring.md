# Phase 1B: Core Module Wiring

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0
**Assigned Agent:** rust-coder (glm-5.1:cloud)
**Fallback:** architect (qwen3-coder:480b:cloud)
**Status:** COMPLETED
**Verified:** 2026-04-29
**Depends On:** phase_1a_compilation_baseline

## Objective
Ensure all core module handlers connect to IronCore and the module graph is complete.

## Module Wiring Checklist
- [x] Wire `transport/`  ensure all libp2p Swarm handlers connect to `IronCore`
- [x] Wire `drift/`  protocol framing, compression (lz4), relay custody, sync
- [x] Wire `routing/`  adaptive routing with TTL budgets, multipath, reputation
- [x] Wire `relay/`  bootstrap nodes, client/server, delegate prewarm, peer exchange
- [x] Wire `notification/`  classification and delivery policy
- [x] Wire `abuse/`  spam detection, reputation, auto-block
- [x] Wire `privacy/`  onion routing, cover traffic, padding, timing obfuscation

## Success Criteria
- [x] All modules have `pub fn` entry points callable from `IronCore`
- [x] No orphan modules  every module is wired into the main execution path
- [x] `cargo check --workspace` passes after wiring changes

## Rules
- All state behind `Arc<RwLock<...>>` (parking_lot)
- Crypto path: X25519 ECDH  shared secret  XChaCha20-Poly1305
- Transport priority: BLE  WiFi  mDNS  QUIC/TCP relay  Internet relay
- Storage: sled-backed, behind `Store` module boundary
