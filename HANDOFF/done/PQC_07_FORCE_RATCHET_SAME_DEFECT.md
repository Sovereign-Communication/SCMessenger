# TASK [MEDIUM]: force_ratchet has the same hardcoded pq_ss=None defect

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO. Split out 2026-07-12 while fixing
`PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` — found but deliberately
NOT fixed as part of that ticket (kept that fix narrowly scoped to the
`decrypt()`/`handle_dh_ratchet()` path it was asked to fix).

## The defect

`RatchetSession::force_ratchet` (`core/src/crypto/ratchet.rs`, currently
~line 649-671) hardcodes `None` for its own `root_key_ratchet_v2` call:

```rust
pub fn force_ratchet(&mut self) -> Result<[u8; 32]> {
    ...
    if let Some(their_dh) = self.their_dh_public {
        let dh_output = self.our_dh_secret.diffie_hellman(&their_dh);
        let (new_root_key, sending_chain_key) = if self.negotiated_suite == Some(0x02) {
            root_key_ratchet_v2(&self.root_key, dh_output.as_bytes(), None)
        } else {
            root_key_ratchet_v1(&self.root_key, dh_output.as_bytes())
        };
        ...
    }
    ...
}
```

Same shape as the bug just fixed in `handle_dh_ratchet` — a suite-0x02
session using `force_ratchet` never gets PQ material mixed into the
resulting root key either.

## Why this needs its own look (not a copy-paste fix)

`force_ratchet` is a manually-triggered, self-initiated ratchet step — NOT
reached via the normal `decrypt()` receive path that `handle_dh_ratchet`
serves. It's unclear from a quick read:

1. Who actually calls `force_ratchet` today, and under what circumstances
   (grep `force_ratchet(` across the workspace — if it's unused/dead code
   entirely, this ticket might resolve to "remove it" rather than "fix it").
2. If it IS used, whether it has access to any fresh PQ material at its
   call site the same way `handle_incoming_pq_fields`'s return value does
   for the normal path, or whether `force_ratchet` would need its OWN PQ
   re-encapsulation step (calling something like `perform_pq_ratchet_step`
   internally) rather than just accepting a passed-in secret.
3. Whether fixing this interacts with the open design question in
   `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md` (a manually-forced ratchet
   is, by definition, not gated on a DH crossing from the peer — this may
   be exactly the mechanism that ticket's design question needs, or it may
   be an unrelated code path entirely; determine this before assuming
   either).

## Gate

Touches `core/src/crypto/ratchet.rs` — mandatory adversarial review before
merge regardless of what the fix turns out to be.
