# P1: Verified Sender Identity Binding in Core Delegate & WASM Topic Parity (CRYPTO-01 & TRN-03)

**Status:** OPEN
**Priority:** P1 (v0.4.0 Release Blocker)
**Target Branch:** `feat/v040-multi-transport-store-forward`
**Components:** `core/src/iron_core.rs`, `core/src/transport/swarm.rs`
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
1. **Cryptographic Identity Spoofing (CRYPTO-01)**: In `core/src/iron_core.rs:3848-3855`, when `receive_message()` invokes `delegate.on_message_received`, it passes `message.sender_id.clone()` for both `sender_id` and `sender_public_key_hex`. `message.sender_id` is an unauthenticated plaintext string inside the decrypted payload. Any sender with a valid keypair can spoof victim public keys or arbitrary strings in `message.sender_id`, completely misleading recipient UI views and notifications.
2. **WASM Own-Topic Parity Omission (TRN-03)**: In `core/src/transport/swarm.rs:8119-8123`, `start_swarm_wasm` was omitted from commit `6acaa231`. WASM swarms never subscribe to `/scmessenger/peer/<own_hex>/v1`, causing 100% inbound message loss on web/WASM targets.

## Acceptance Criteria
1. In `core/src/iron_core.rs:3848-3855`, pass the authenticated `canonical_peer_id` and `sender_public_key_hex` to `delegate.on_message_received`.
2. In `core/src/transport/swarm.rs:8119-8123`, derive `own_peer_key_hex` and subscribe to own topic before entering the WASM select loop.
3. Guard WASM `gossipsub::Event::Subscribed` with `is_ghost_peer_topic()`.
