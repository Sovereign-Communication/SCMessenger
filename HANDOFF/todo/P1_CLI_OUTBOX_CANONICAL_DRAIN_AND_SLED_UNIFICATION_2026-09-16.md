# P1: Outbox Canonical Addressing Drain & IronCore Outbox Unification (CLI-03 & CORE-02)

**Status:** OPEN
**Priority:** P1 (v0.4.0 Release Blocker)
**Target Branch:** `feat/v040-multi-transport-store-forward`
**Components:** `cli/src/main.rs`, `core/src/iron_core.rs`, `core/src/store/outbox.rs`
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
1. **Outbox Drain Addressing Mismatch (CLI-03)**: In `cli/src/main.rs:4623`, messages queued for offline delivery are keyed under the recipient's canonical 64-hex public key (`contact.peer_id`), written to Sled with prefix `queue:<64-hex>_`. In `cli/src/main.rs:3746`, `flush_outbox_for_peer` queries `ob.drain_for_peer(&peer_id.to_string())` with base58 `12D3KooW...`. The prefix `queue:12D3KooW..._` never matches the 64-hex key, leaving messages stranded forever in the persistent outbox (159 undelivered backlog).
2. **`IronCore` Outbox Split-Brain (CORE-02)**: In `core/src/iron_core.rs:476`, `IronCore::with_storage` initializes an ephemeral in-memory `Outbox::new()`. When delivery receipts arrive, `IronCore::receive_message` executes `self.mark_message_sent()` against the empty in-memory outbox, logging `removed=false`. The persistent Sled outbox used by CLI is never cleared upon delivery.

## Acceptance Criteria
1. In `cli/src/main.rs:3740-3750` (`flush_outbox_for_peer`), drain the outbox by both the peer's base58 ID and its canonical 64-hex public key (derived via `extract_ed25519_public_key_from_peer_id`).
2. In `core/src/iron_core.rs:476`, instantiate `IronCore`'s internal outbox using the shared persistent Sled storage backend matching the CLI.
3. Verify that receipt processing clears the persistent outbox record and logs `removed=true`.
