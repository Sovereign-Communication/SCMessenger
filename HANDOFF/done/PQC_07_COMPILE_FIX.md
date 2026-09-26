# TASK: PQC-07 Compile Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

The recent changes to `core/src/crypto/ratchet.rs` and `core/src/crypto/session_manager.rs` introduced some compilation errors. Please fix them.

Here is the cargo check output:

```
error[E0061]: this function takes 17 arguments but 13 arguments were supplied
   --> core\src\crypto\session_manager.rs:453:12
    |
453 |                       Ok(RatchetSession::reconstruct(
    |  ____________^^^^^^^^^^^^^^^^^^^^^^^^^^^-
454 | |                           our_dh_secret,
455 | |                           our_dh_public,
456 | |                           their_dh_public,
...   |
466 | |                           bootstrap_hct,
467 | |                   ))
    | |_________- multiple arguments are missing
    |
note: associated function defined here
   --> core\src\crypto\ratchet.rs:159:19
    |
159 |           pub(crate) fn reconstruct(
    |                                       ^^^^^^^^^^^
...
173 |                   pq_our_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
    |                   ----------------------------------------------------------
174 |                   pq_prev_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
    |                   -----------------------------------------------------------
175 |                   pq_their_encaps_key: Option<Vec<u8>>,
    |                   ------------------------------------
176 |                   pq_pending_ct: Option<Vec<u8>>,
    |                   ------------------------------
help: provide the arguments
    |
453 |                   Ok(RatchetSession::reconstruct(
...
466 |                           bootstrap_hct,
467 ~                           /* std::option::Option<MlKem768KeyPair> */,
468 +                           /* std::option::Option<MlKem768KeyPair> */,
469 +                           /* std::option::Option<Vec<u8>> */,
470 +                           /* std::option::Option<Vec<u8>> */,
471 ~                   ))


error[E0382]: use of moved value: `pq_ss`
   --> core\src\crypto\ratchet.rs:579:72
    |
554 |                   let pq_ss = if self.negotiated_suite == Some(0x02) {
    |                           ----- move occurs because `pq_ss` has type `std::option::Option<Vec<u8>>`, which does not implement the `Copy` trait
...
564 |                           root_key_ratchet_v2(&self.root_key, dh_output.as_bytes(), pq_ss)
    |                                                                                                                                               ----- value moved here
...
579 |                           root_key_ratchet_v2(&new_root_key, dh_output_2.as_bytes(), pq_ss)
    |                                                                                                                                                 ^^^^^ value used here after move
    |
note: consider changing this parameter type in function `root_key_ratchet_v2` to borrow instead if owning the value isn't necessary
   --> core\src\crypto\ratchet.rs:780:12
    |
777 |   fn root_key_ratchet_v2(
    |         ------------------- in this function
...
780 |           pq_ss: Option<Vec<u8>>,
    |                         ^^^^^^^^^^^^^^^ this parameter takes ownership of the value
help: consider cloning the value if the performance cost is acceptable
    |
564 |                           root_key_ratchet_v2(&self.root_key, dh_output.as_bytes(), pq_ss.clone())
```

Also, in `core/src/crypto/ratchet.rs:671:18` there is an unused variable `ss_pq`. Make sure to store `ss_pq` somewhere if it should be used, or prefix it with `_`. (If it's the shared secret from encapsulation, maybe you need to store it in `self` or pass it properly?). Wait, the design says `ss_pq` is used in the root update! 
Wait, `let (ct, ss_pq) = crate::crypto::pq::encapsulate(their_encaps_key)?` happens in `perform_pq_ratchet_step`. Does it return it? Yes, `return Ok((ct, ss_pq.to_vec()))`! So just fix the unused variable by returning it properly, or checking how it's used.

Please fix these compilation errors by returning the FULL file contents for `core/src/crypto/session_manager.rs` and `core/src/crypto/ratchet.rs`.
