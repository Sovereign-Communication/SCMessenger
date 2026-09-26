# TASK: Fix ratchet.rs compile errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Fix these 3 remaining compile errors in `core/src/crypto/ratchet.rs`. Do NOT change any other logic.

## Error 1 — E0382: `pq_ss` moved value used twice (line ~564 and ~579)

The variable `pq_ss` (type `Option<Vec<u8>>`) is passed by move to `root_key_ratchet_v2` on first use and then used again.
Fix: clone it on the first call: `pq_ss.clone()`.

## Error 2 — E0308: `decapsulate` argument order is wrong (lines ~700 and ~715)

`crate::crypto::pq::decapsulate` signature is:
```rust
pub fn decapsulate(keypair: &MlKem768KeyPair, ct: &[u8]) -> Result<[u8; 32]>
```
But call sites pass `(pq_kem_ciphertext, our_keypair.private_key())` — wrong order AND wrong types.
Fix: pass `(our_keypair, pq_kem_ciphertext)` and `(prev_keypair, pq_kem_ciphertext)` respectively.

## Error 3 — warning: unused variable `ss_pq` (line ~671)

```rust
let (ct, ss_pq) = crate::crypto::pq::encapsulate(their_encaps_key)?;
```
`ss_pq` is intentionally not used here (it's the sender-side shared secret; the root KDF uses it on the receiver side via decapsulate).
Fix: rename to `_ss_pq`.

Return ONLY the corrected `ratchet.rs` file contents.
