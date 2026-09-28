<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

VERDICT: BLOCK

The proposed design reintroduces the exact asymmetric mixing desync that killed attempt 1, shifted by one message round-trip. Tying the PQ mix to `handle_dh_ratchet` fails the core synthesized requirement because `handle_dh_ratchet` is strictly a receiver-side operation triggered by the peer's DH rotation. The sender and receiver do not reach this point together for the same message, guaranteeing an immediate root-key divergence and AEAD failure.

### Concrete Failure Sequence
1. **Alice sends M100 (PQ cadence trigger):** She calls `perform_pq_ratchet_step`, generates `ss_pq`, and stores it in `self.pending_pq_ss`. She encrypts M100 using her current sending chain, which was derived from her **unmixed** root key (`RK_unmixed`). M100 contains her current DH key (`DH_A50`) and the PQ fields.
2. **Bob receives M100:** `dh_changed` is true (Bob's last seen key from Alice was `DH_A49`). Bob calls `handle_incoming_pq_fields`, decapsulates `ss_pq`, and stores it in `self.pending_pq_ss`. 
3. **Bob calls `handle_dh_ratchet`:** He reads `self.pending_pq_ss`, mixes `ss_pq` into his root key via `root_key_ratchet_v2`, and clears the pending field. He derives his receiving chain for M100 from this newly **mixed** root key (`RK_mixed`).
4. **AEAD Failure:** Bob attempts to decrypt M100 using a chain derived from `RK_mixed`. Alice encrypted it using a chain derived from `RK_unmixed`. The authentication tag fails. The session is permanently bricked on the exact message that carries the PQ material. Alice will not mix `ss_pq` into her root key until she receives M101 and triggers her own `handle_dh_ratchet`, but the session is already dead.

### Code Defects
1. **Syntax Error in `handle_dh_ratchet`:** The diff introduces a stray `None };` after the `if/else` block that clears `pending_pq_ss`. This will not compile.
   ```rust
           // Clear pending PQ secret after using it (idempotent if already None)
           self.pending_pq_ss = None;
   +
               None
           };
   ```
2. **Type Mismatch in `handle_incoming_pq_fields`:** The function signature remains `-> Result<Vec<u8>>`, but the diff changes the success paths to `return Ok(());`. This is a compile error.

### DoD Gaps
1. **Missing Symmetry Proof:** No Kani or unit proof of root-key mix symmetry was added, despite this being an explicit requirement to catch exactly this class of desync.
2. **Unresolved `chain.index` Question:** The ticket explicitly required resolving whether `chain.index` should reset to 0 or be preserved on chain-key replacement. The diff implicitly resets it to 0 via `Chain::new(...)` without any design justification or explicit resolution in the response.
3. **Architectural Violation:** The design fails the requirement that the mix must happen at a point "BOTH sides are guaranteed to reach together". Because `encrypt()` does not rotate DH keys and `handle_dh_ratchet` is only called on receive when `dh_changed == true`, the sender and receiver mix the PQ secret at completely different, uncoordinated message boundaries.