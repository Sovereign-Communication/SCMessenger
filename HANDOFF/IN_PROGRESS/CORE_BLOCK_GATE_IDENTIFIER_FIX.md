# CORE_BLOCK_GATE_IDENTIFIER_FIX -- block gate misses because sender_id now carries the public key

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: todo
Tier: CODER
Domain: rust-core (security enforcement)
Target Files: core/src/iron_core.rs

## Bug (verified at HEAD 2026-08-04)

On the inbound receive path in `core/src/iron_core.rs` (approx lines
3185-3230, after `decode_message`), both block lookups key on
`message.sender_id`:

    let is_blocked_and_deleted = self.blocked_manager.read()
        .is_blocked_and_deleted(&message.sender_id).unwrap_or(false);
    ...
    let sender_device_id = self.contact_manager.read()
        .get(message.sender_id.clone()) ... .and_then(|c| c.last_known_device_id);
    ...
    let is_blocked = match self.blocked_manager.read()
        .is_blocked(&message.sender_id, sender_device_id.as_deref()) { ... };

After this branch's identity canonicalization, `message.sender_id` carries
the sender's PUBLIC KEY (64-hex ed25519), but `block_peer()` /
`block_and_delete_peer()` store blocks under the identifier the caller
passed -- the BLAKE3 IDENTITY_ID (also 64-hex) in all existing stores and
in the integration tests. The comparison misses. Consequence: blocked
peers' messages are no longer hidden, blocked+deleted peers' payloads are
no longer dropped at ingress. Blocking is silently broken. Three
integration tests correctly detect this (they are RIGHT; do not touch
them): core/tests/integration_contact_block.rs --
test_blocked_message_persisted_but_hidden,
test_unblock_restores_hidden_message_visibility,
test_block_and_delete_purges_messages_and_drops_future_payloads.

## Required fix

Before the block checks, derive the sender's identity_id from the public
key using the single source of truth this branch already added:
`crate::identity::keys::identity_id_from_public_key_hex` (defined at
core/src/identity/keys.rs:37, returns Option<String>, yields None for
anything that is not a 32-byte hex key). Then perform the block lookups
under BOTH identifier flavors:

1. Build the candidate set { message.sender_id as-is } plus, when the
   derivation succeeds, the derived identity_id.
2. `is_blocked_and_deleted`: true if ANY candidate is blocked+deleted ->
   keep returning `Err(IronCoreError::Blocked)` exactly as today.
3. `is_blocked` (with device id): true if ANY candidate is blocked.
   PRESERVE the existing fail-closed semantics verbatim: on a block-store
   read error, log the same warning and treat as blocked (hide, retain).
4. `sender_device_id` contact lookup: try the contact under both flavors
   too (sender_id as-is, then derived identity_id), first hit wins; a
   miss still yields None as today.
5. Everything else in the receive path stays byte-identical: evidentiary
   retention (blocked-only messages are STORED with hidden=true, not
   dropped), receipt classification ordering, dedup/inbox behavior.

Minimal diff. Do NOT modify: any test file, core/src/store/blocked.rs,
core/src/blocked_bridge.rs, the BlockedManager API, or any other file.
Do NOT add dependencies. Do NOT re-derive the hash inline (no blake3
calls in iron_core.rs) -- use the helper.

## Acceptance criteria

- All three tests in core/tests/integration_contact_block.rs pass.
- No other behavior change; no test edits; no new dependencies.
- `cargo fmt --all -- --check` clean on the touched file.

## Worker contract (mandatory)

End your response with exactly this footer:

    ---ORCHESTRATION_METADATA---
    RESULT: DONE|BLOCKED|FAILED
    VERIFICATION: NONE
    FILES: ["core/src/iron_core.rs"]
    NOTES: ["what changed", "anything the verifier must know"]
    ---END---

You have NO execution environment: VERIFICATION must be NONE. Emit one
fenced ```diff block with --- a/ and +++ b/ headers.

## Gate (orchestrator-side, under build_lock, in this order)

cargo test -j6 -p scmessenger-core --lib
cargo test -j6 -p scmessenger-core --test integration_contact_block
cargo test -j6 -p scmessenger-cli --test integration_message_requests

FULL suite as listed -- a scoped run is what let this bug reach CI.

## Provenance

Kickoff item 0-CRITICAL (HANDOFF/todo/_NEXT_ORCHESTRATE_KICKOFF.md,
2026-08-04). Third instance of the identifier-confusion class in PR #136;
first in production security enforcement. Fix will be adversarially
reviewed at THINK tier before commit (operator + Claude-handoff mandate).
