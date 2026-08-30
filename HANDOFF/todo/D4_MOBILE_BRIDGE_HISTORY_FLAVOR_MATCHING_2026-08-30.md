# D4 follow-up -- mobile_bridge HistoryManager lacks identity-flavor coalescing

Status: OPEN (filed 2026-08-30, scoped out of PR #244 per adversarial review)
Filed: 2026-08-30 (Windows lane, during the #244 adversarial gate)

## What the review found

The core-store coalescing (PR #244, `core/src/store/history.rs`) fixes the
CLI/WASM surfaces. Android and iOS bind a SEPARATE manager,
`core/src/mobile_bridge.rs` (`recent_internal` at ~:2786, matched by
`&record.peer_id == peer` -- no coalescing, and case-SENSITIVE, unlike the
core store's `eq_ignore_ascii_case`). Mobile callers query by the pubkey
flavor (e.g. `MeshRepository.swift:3664` uses `contact.peerId`), so D4 split
threads persist verbatim on the two shipped clients.

## Deliberately NOT done in #244

The reviewer explicitly instructed: "Do not attempt a store unification here."
The core store and mobile bridge are separate stores; unifying them is out of
scope. This ticket exists so the follow-up does not get lost.

## Acceptance

- Apply the same flavor matching (precomputed `identity_id_from_public_key_hex`
  filter, matched against both flavors, case-insensitive) to
  `mobile_bridge::recent_internal` (and its conversation/remove paths if they
  have the same asymmetry).
- Add a regression test proving a pubkey-flavor query reaches identity_id-keyed
  records through the mobile_bridge manager.
- Re-run the adversarial gate (Rule-8) before merge.
