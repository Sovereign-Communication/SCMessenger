# TASK: PQC-11 — Dual-signature verification on relay registration and invite protocol

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-10. Wave 4. Min tier: Haiku.

## Why

Invite tokens and relay/bootstrap registration are signed Ed25519-only (`core/src/relay/invite.rs` — token struct has `signature: Vec<u8>` with an Ed25519 comment at ~line 36; `core/src/relay/bootstrap.rs` verifies inviter keys at ~line 72). Post-CRQC these become forgeable (audit F5). Extend them to carry and verify the dual-signed material from PQC-10.

## Rules

1. Invite token v2: new format tag (existing tokens must keep verifying forever). v2 token carries the inviter's `PublicKeyBundle` (or a hash of it + bundle fetched via existing exchange — choose whichever the current invite flow supports without adding a round trip; state the choice) and BOTH signatures over the token payload (Ed25519 + ML-DSA-65, same AND rule as PQC-10).
2. Relay registration records: same treatment; a relay record carrying a bundle with ML-DSA must AND-verify.
3. Compat window: v1 tokens/records remain accepted this release; every acceptance logs an audit event (`legacy_single_sig_invite` / `legacy_single_sig_relay`).
4. `require_pq = true`: v1 invites/registrations are rejected.

## Steps

1. Inventory all sign/verify sites: `rg -n "sign|verify" core/src/relay --type rust -l`, then per-file function list into this task file before editing.
2. Token/record v2 formats with tag bytes; generation signs with both keys when the local identity has ML-DSA (post PQC-10 migration it always does); verification per rules above.
3. Tests per surface: v1 fixture still verifies; v2 both-valid OK; either-tampered REJECT; strict-mode rejects v1; token size recorded (expect ~6-7 KB with bundle + dual sigs — confirm the invite transport path carries it; if a QR-code path exists and overflows QR capacity, record the number and escalate rather than truncating).

## Definition of Done

- [ ] Standard gates PASS.
- [ ] `cargo test -p scmessenger-core --test integration_registration_protocol` green + new unit tests green.
- [ ] Sign/verify site inventory + size measurements written into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Break v1 token verification.
- Change the invite UX flow or add round trips.
