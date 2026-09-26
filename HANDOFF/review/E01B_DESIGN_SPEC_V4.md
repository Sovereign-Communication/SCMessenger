# E-01b DESIGN SPEC V4: PQ-secret -> root-key mixing

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** DESIGN COMPLETE — awaiting adversarial review (gate for E-01c)
**Author:** Fable (Claude native), 2026-07-17
**Supersedes:** Qwen THINK draft (`tmp/E-01b-design-spec_response.md`) — REFUTED AS WRITTEN, direction salvaged (Section 2)
**Scope:** Design only. No code, no wire-format change, no KDF change, cadence trigger (%100) unchanged.

**Design name:** Receiver-anchored deferred mixing with authenticated trial adoption.

**One-paragraph summary:** The decapsulated PQ secret is held in explicit
pending state on both sides and mixed into the root-key transition owned by
the ct-RECEIVER's next DH ratchet step (its second `root_key_ratchet_v2`
call). The ct-SENDER never guesses whether the peer mixed: at its own next DH
step it derives the receiving chain both ways (with and without its
outstanding secret) and adopts whichever variant the message's AEAD tag
authenticates. Every commitment is causally downstream of cryptographic proof
of the peer's state, so loss, reorder, and stripping can only defer the mix,
never desynchronize it. Three companion hardenings (skipped-keys-first,
commit-on-success, AAD binding of ratchet/PQ header fields) are required and
specified; they also close two pre-existing session-brick vectors in the
classical path.

---

## 0. Corrections to the handoff's line references

Verified against `core/src/crypto/ratchet.rs` as of 2026-07-17 (1048 lines).
The handoff's numbers drifted; E-01c must use these:

| Item | Handoff said | Actual |
|---|---|---|
| `encrypt()` | 485 | 485-521 (correct) |
| `decrypt()` | 524 | 524-566 (correct) |
| `handle_dh_ratchet()` | 568 | 568-620 (correct) |
| `perform_pq_ratchet_step()` | 688 | **663-685** (discard at **674**) |
| `handle_incoming_pq_fields()` | 713 | **688-728** |
| `root_key_ratchet_v2()` | 782 | 782-798 (correct) |
| `RatchetSession` struct fields | ~449-468 | **114-153** (PQ fields 144-152) |

Call-site verification (handoff suspected-flaw 3):
`perform_pq_ratchet_step` is called from exactly one production site:
`encrypt.rs:379` inside `encrypt_message_ratcheted` (cadence block 375-384),
guarded by `if let Ok(...)` (failures silently swallowed — see Section 9,
P4). `handle_incoming_pq_fields` is called from exactly one production site:
`encrypt.rs:325` inside `decrypt_message_ratcheted_v2`; its returned
`Vec<u8>` (the decapsulated ss) is **discarded** (`?` propagates errors only).
`clear_pq_pending_ct` (ratchet.rs:742-744) has **zero callers**.
`force_ratchet` (referenced by PQC_07 history) no longer exists in the tree.

---

## 1. Verified code structure and the transition model

### 1.1 What actually executes where

- `encrypt()` (ratchet.rs:485-521): advances the sending chain
  (`next_message_key`, ratchet.rs:105-110), stamps the CURRENT
  `our_dh_public` into the result (515-520). **No DH rotation, no root KDF,
  ever.**
- `decrypt()` (ratchet.rs:524-566): computes `dh_changed` by byte-comparing
  the header key against stored `their_dh_public` (539-542). Any difference
  — including regression to an OLD key — triggers `handle_dh_ratchet`
  (544-546). Then `get_message_key` (548, body 622-660: skipped-key cache
  lookup 627-630, skip-ahead loop 646-657), then AEAD (557-565).
- `handle_dh_ratchet()` (ratchet.rs:568-620): **receiver-side only.**
  Two root KDF calls:
  - KDF-1 (590-594): `root_key_ratchet_v2(root_key, dh1, pq_ss)` where
    `dh1 = first_dh_secret.diffie_hellman(their_new_dh)` (578);
    `first_dh_secret` is the identity secret on the very first call, else
    the current `our_dh_secret` (569-576). Output: new root + RECEIVING
    chain key.
  - Fresh DH keypair generated (598-602).
  - KDF-2 (605-609): `root_key_ratchet_v2(new_root_key, dh2, pq_ss)` where
    `dh2 = new_dh_secret.diffie_hellman(their_new_dh)` (604). Output: final
    root + SENDING chain key.
  - `pq_ss` is hardcoded `None` for both calls (580-588) — the defect under
    repair. State committed at 611-617; both chains rebuilt via
    `Chain::new` (index 0).
- PQ cadence (encrypt.rs:375-384): fires when the sending chain's
  chain-local message number satisfies `> 0 && % 100 == 0`. It is
  **mid-epoch and fully decoupled from DH steps** — the receiver of the
  cadence message normally sees `dh_changed == false`.
- PQ receive (encrypt.rs:310-327): processed only when `peer_confirmed`,
  BEFORE `session.decrypt` (329), i.e., before any authentication of the
  carrying message. Anti-stripping check gated on `message_number % 100 == 0`
  (318-321).

### 1.2 The transition model (analytical core of this spec)

Both parties compute one common linear sequence of root transitions:

```
R0 --T1--> R1 --T2--> R2 --T3--> R3 --T4--> ...
Tn: (R_{n-1}, dh_n [, pq_n]) -> (R_n, chainkey_n)   via root_key_ratchet_v2
```

Facts, all forced by the code above:

- **F1.** Each side computes every transition exactly once. The side that
  generates the epoch's DH keypair computes Tn EARLY, as KDF-2 of its
  `handle_dh_ratchet` (chainkey_n becomes its SENDING chain). The peer
  computes the same Tn LATE, as KDF-1 of its next `handle_dh_ratchet`,
  triggered by first sighting of that DH public in a header (chainkey_n
  becomes its RECEIVING chain). dh_n is equal on both sides by DH
  commutativity (Alice: 578 with her stored secret x peer's new public;
  Bob: 604 with his new secret x Alice's stored public). Special case: T1's
  early side is `init_as_sender_hybrid` (ratchet.rs:379-383, pq input
  `None`).
- **F2.** One `handle_dh_ratchet` call therefore computes two CONSECUTIVE
  transitions with opposite roles: KDF-1 recomputes a transition the peer
  already committed (in the past); KDF-2 creates a transition the peer will
  recompute (in the future). **The two `root_key_ratchet_v2` calls in one
  invocation belong to two different cross-side pairings and may therefore
  require two DIFFERENT pq inputs.** Any design passing one value to both
  calls (as the current `pq_ss.clone()` / `pq_ss` shape suggests) forces
  every transition in the session to carry the same value — structurally
  wrong.
- **F3 (strict alternation).** X can run `handle_dh_ratchet` again only
  after observing a DH public the peer generated in the peer's own
  `handle_dh_ratchet`, and vice versa. Neither side can get two DH steps
  ahead. Consequently transitions are consumed strictly in order and an
  epoch can never be skipped — total loss of an epoch's traffic stalls the
  ratchet (safe) rather than forking it. This lemma underpins every proof
  below. It holds only if spurious ratchets on stale keys are impossible —
  hence hardening H1 is REQUIRED (Section 5).
- **F4.** Roots are NEVER equal across sides at rest: the last side to run
  `handle_dh_ratchet` is one transition ahead (matches PQC_07's "receiver
  one preemptive round ahead"). Correctness is equality PER TRANSITION, not
  equality of instantaneous `root_key` values. Tests must compare transition
  sequences, not live roots (Section 7).

### 1.3 Consequence: the real design question

For each transition Tn, the EARLY computer fixes pq_n; the LATE computer
must reproduce pq_n exactly, from information available at its (later)
KDF-1. The early side cannot know what the peer has received (two-generals);
the late side, however, always knows at least as much as the early side did.
So the only robust shape is: **the early side commits based on what it
provably possesses; the late side VERIFIES the early side's choice rather
than guessing it.** The verification oracle already exists: chainkey_n keys
the AEAD of the epoch's messages, so the AEAD tag of the very message that
triggers KDF-1 authenticates which pq_n the early side used. This
observation dissolves the synchronization problem that killed attempts 1-3.

---

## 2. Verdict on the Qwen THINK draft: REFUTED AS WRITTEN, direction salvaged

The draft's core intuition — defer mixing via explicit `pending_pq` state,
anchor at a DH epoch boundary, let the reply round-trip synchronize — is
correct and is retained. Its mechanism is unimplementable against the real
code, in five specifics:

- **Q1. Nonexistent send-side DH rotation.** Draft Sections 2-3 place logic
  inside `encrypt()` under "existing DH rotation logic" and an `is_dh_step`
  predicate, and have the receiver "mix pending_pq in encrypt() when
  sending a DH-step reply". `encrypt()` (ratchet.rs:485-521) contains no
  rotation and no root KDF; both new chains — including the sending chain
  used for the reply — are fixed inside the RECEIVE-path
  `handle_dh_ratchet` (605-615) before any reply exists. The receiver-side
  mixing point described by the draft does not exist. Force-mapping it onto
  the real code ("receiver mixes at its HDR, sender at its own later HDR,
  each taking their single pending slot into whichever KDF call comes
  first") reproduces attempt 3's one-round-trip-shifted asymmetry exactly:
  one side commits a mixed transition the other recomputes unmixed, AEAD
  fails on the first message of that epoch, session bricked.
- **Q2. Single-KDF model.** The draft's `handle_dh_ratchet` sketch shows one
  `root_key_ratchet_v2` call with `self.pending_pq.take()`. The real
  function has two calls (591, 606) with the cross-pairing of F2. The draft
  never specifies the second call's input; `take()` at the first call
  silently starves the second. Handoff suspected-flaw 2 confirmed.
- **Q3. PQ step is not DH-coupled.** The draft assumes encapsulation happens
  on DH-step messages ("KEM encapsulation required for every DH-step",
  draft Section 9). Actually `perform_pq_ratchet_step` is called only from
  the cadence block (encrypt.rs:375-384) at chain-local message number
  %100 — mid-epoch, where the receiver sees `dh_changed == false`. The
  draft's "sender stores ss_pq during its DH-step send" flow does not align
  with the message that carries the ciphertext. Handoff suspected-flaw 3
  confirmed.
- **Q4. Imaginary stale-DH detection.** The draft's reorder safety rests on
  "out-of-order messages with stale DH keys are processed on old chains
  without affecting pending PQ state". The real `decrypt()` has no key
  ordering: ANY header key differing from `their_dh_public` — newer or
  older — fires `handle_dh_ratchet` (539-546), replacing root, chains, and
  our DH keypair. A single delayed cross-epoch message today trashes even a
  classical session (no rollback on the subsequent AEAD failure). The
  draft's reorder argument is void without hardening H1, which this spec
  makes mandatory.
- **Q5. Unconfirmed overwrite.** The draft overwrites `pending_pq` on each
  new PQ step ("pending state is overwritten by subsequent messages"). With
  a lost reply, the peer can still hold and later mix the OLD secret while
  the initiator has already overwritten it — under the draft, the initiator
  can then never reproduce the peer's transition. Fixed here by O1
  (never create a new PQ step while one is outstanding) plus the dedupe
  marker (R4) and trial adoption (R5), which make stale-candidate desync
  impossible by construction.

Verdict: the draft is NOT salvageable by local correction (Q1 is
structural), but its deferred-pending-state direction survives in the
corrected design below.

---

## 3. The design

### 3.1 New session state (`RatchetSession`, ratchet.rs:114-153)

```rust
/// Outstanding secret we ENCAPSULATED (we are the ct sender), awaiting
/// proof the peer mixed it. Cleared only by trial adoption (R5).
pq_pending_sent: Option<PendingPqSecret>,   // { ss: RatchetKey-like zeroizing 32B, created_step: u32 }
/// Secret we DECAPSULATED (we are the ct receiver), armed for mixing at
/// our next handle_dh_ratchet KDF-2. Consumed by R3.
pq_pending_recv: Option<ZeroizingSecret>,
/// Fingerprint (blake3, 16 bytes) of the last secret we MIXED at KDF-2.
/// Blocks re-arming the same secret from re-attached/late ciphertexts (R4).
pq_last_mixed_fp: Option<[u8; 16]>,
```

All three must be added to `reconstruct()` (ratchet.rs:158-198) and to
`SerializableRatchetSession` (session_manager.rs) — see P1, which is
REQUIRED, not optional. All secrets zeroize on drop. ML-KEM-768 ss is 32
bytes; store as fixed arrays, not `Vec`.

### 3.2 Rules (identical state machine on both sides — no roles)

- **R1 (create + retain).** In `perform_pq_ratchet_step` (ratchet.rs:663-685):
  replace line 674's discard with capture; set
  `pq_pending_sent = Some({ss, dh_step_count})`. Keep the existing keypair
  rotation (677-679) and `pq_pending_ct` (682).
  **O1 (defer-while-outstanding):** in the cadence block (encrypt.rs:375-384),
  if `pq_pending_sent.is_some()`, do NOT call `perform_pq_ratchet_step`;
  re-attach the existing `pq_pending_ct` + current own encaps key instead.
  At most one outstanding secret per direction, ever.
- **R2 (re-attach until confirmed).** While `pq_pending_sent.is_some()`,
  attach `(pq_pending_ct, our current encaps key)` to EVERY outgoing
  suite-0x02 ratcheted envelope (this finally implements the
  `pq_pending_ct` field's documented "keep sending until peer acks",
  ratchet.rs:151-152, and survives loss of any individual carrier,
  including across our own epoch turns). Wire fields unchanged; only which
  messages populate them changes. Bandwidth note: Section 8, O-2.
- **R3 (arm on decapsulation; mix at KDF-2).** On an incoming envelope with
  PQ fields (decrypt path, staged per H2): decapsulate
  (`handle_incoming_pq_fields`, ratchet.rs:688-728). If the resulting ss's
  fingerprint equals `pq_last_mixed_fp` or equals the fingerprint of the
  current `pq_pending_recv`, ignore (idempotent). Else set
  `pq_pending_recv = Some(ss)`. In `handle_dh_ratchet`, the **KDF-2 call
  (605-609) takes `pq_ss_2 = pq_pending_recv.take()`**; if it mixed
  `Some(ss)`, set `pq_last_mixed_fp = fp(ss)`. Decapsulation failure
  becomes NON-FATAL (soft skip, no arming) — the current `bail!` at
  ratchet.rs:727 would otherwise reject an authentic message carrying a
  ct staled past the prev-keypair window (R2 makes that reachable); AEAD
  (via D5) is the integrity gate now, not decapsulation success.
- **R4 (dedupe).** The fingerprint check in R3. Needed because R2 re-attaches
  the same ct after the peer already mixed it: in-flight duplicates must not
  re-arm and cause a second (one-sided) mix. Depth 1 suffices: by F3, no
  older secret than the last mixed one can still legitimately arrive and
  arm before the next epoch turn. (Optional depth-2 hardening noted in
  Section 8.)
- **R5 (trial adoption at KDF-1).** In `handle_dh_ratchet`, the **KDF-1 call
  (590-594) input is not guessed**. Candidates, in order:
  `[None, pq_pending_sent.ss]` (present only if outstanding). For each
  candidate: derive `(root', recv_chain')` via KDF-1, derive the triggering
  message's key from `recv_chain'` (including any skip-ahead), attempt the
  AEAD open (557-565 equivalent) against the triggering envelope. Exactly
  one candidate can verify (AEAD unforgeability + KDF collision
  resistance); adopt it, then proceed to KDF-2 (which uses R3's slot —
  evaluated AFTER adoption so an adopting HDR still mixes its own inbound
  slot correctly). If the adopted candidate was `pq_pending_sent.ss`:
  clear `pq_pending_sent` and `pq_pending_ct` (confirmed — stop
  re-attaching). If ALL candidates fail: the message is rejected and NO
  state changes (H2) — semantically identical to today's AEAD failure.
  Trials occur only at epoch boundaries (at most once per received epoch)
  and cost at most 2 KDF+AEAD attempts.
- **R6 (no signaling needed for KDF-2's choice).** A side never announces
  whether its KDF-2 mixed; the peer discovers it via R5. This is the
  load-bearing inversion relative to attempts 1-3: KDF-1 never assumes —
  it verifies.

### 3.3 Explicit pq_ss specification for BOTH `root_key_ratchet_v2` calls

(Handoff deliverable 4.) For every `handle_dh_ratchet` invocation on either
side:

| Call | Line | Chain produced | pq input | Cross-side pairing and equality argument |
|---|---|---|---|---|
| KDF-1 | 590-594 | receiving | **trial-adopted value**: `None` or own `pq_pending_sent.ss` (R5) | Recomputes the transition the peer committed at ITS KDF-2 (F2). Equality is not assumed but AEAD-verified: only the candidate equal to the peer's actual KDF-2 input yields the chain that opens the triggering envelope. Peer's KDF-2 input was its `pq_pending_recv` = the ss decapsulated from OUR ct = our `pq_pending_sent.ss` (KEM correctness, D5 authenticity), or `None` — both are in our candidate set, and O1+R4 guarantee no third possibility exists. |
| KDF-2 | 605-609 | sending | `pq_pending_recv.take()` (R3): `Some(ss)` if armed, else `None` | Creates the transition the peer recomputes LATER at its KDF-1. The peer's future candidate set `[None, peer.pq_pending_sent.ss]` contains our input by the same identity read right-to-left: our `pq_pending_recv` came from decapsulating the peer's outstanding ct, so it equals `peer.pq_pending_sent.ss`; if we mixed `None`, `None` is in the set. |

The two calls take INDEPENDENT values from INDEPENDENT slots (inbound vs
outbound direction). The two cadence directions (A->B and B->A) therefore
compose without interference: one invocation may simultaneously adopt at
KDF-1 (confirming our outbound secret) and mix at KDF-2 (our inbound
armed secret) — distinct transitions, distinct slots. The current code's
`pq_ss.clone()` / same-value-both-calls shape must NOT be preserved.

Both sides use identical values per transition — which is the actual
requirement (F4): instantaneous root equality across sides never holds in
this ratchet, with or without PQ.

---

## 4. Full round-trip trace against actual code (handoff deliverable 2)

Setting: steady state, suite 0x02, `peer_confirmed` both sides. Common
sequence position: Alice last ran `handle_dh_ratchet` (or is mid-epoch),
so with epoch keys named `A_k`, `B_k`:
`R_alice = R_n` (she computed T_n at her KDF-2, generating `A_k`);
Bob saw `A_k`, ran his HDR: computed T_n (KDF-1) and T_{n+1} (KDF-2,
generating `B_{k+1}`), so `R_bob = R_{n+1}` (F4). Bob has not yet sent.
Alice's sending chain = chainkey from T_n; message number ~99.

**Step 1 — Alice sends M100 (cadence fires).**
`encrypt_with_ratchet_fallback` (encrypt.rs:503) -> `encrypt_message_ratcheted`
(356) -> `session.encrypt` (362 -> ratchet.rs:485): chain T_n advances,
header `(A_k, 100)`. Cadence block (375-384): `perform_pq_ratchet_step`
(ratchet.rs:663): encapsulate to Bob's advertised encaps key -> `(ct1, ss1)`;
**R1: `pq_pending_sent = ss1`** (replaces the 674 discard); KEM keypair
rotated (677-679); `pq_pending_ct = ct1` (682). Envelope carries
`(A_k, 100, ct1, ek_A')`. R2: messages 101, 102, ... also carry
`(ct1, ek_A')`. `R_alice = R_n` — unchanged. No root movement at creation.

**Step 2 — Bob receives M100** (or ANY ct1-carrying sibling first — same
outcome, that is the point of R2).
`decrypt_with_ratchet_fallback` (encrypt.rs:581) -> `decrypt_message_ratcheted_v2`
(276): PQ fields staged (322-326 region, restructured per H2): decapsulate
(ratchet.rs:697-710, current-keypair path) -> ss1; R4 fingerprint check
passes; **stage `pq_pending_recv = ss1`**, stage `pq_their_encaps_key = ek_A'`
(702). `session.decrypt` (329 -> ratchet.rs:524): `dh_changed == false`
(539-542, header still `A_k`) -> NO `handle_dh_ratchet` -> `get_message_key`
(548; skip-ahead if M100 arrived early) -> AEAD verifies (D5 AAD covers the
PQ fields) -> H2 commits staged state. `R_bob = R_{n+1}` — **unchanged.
Decapsulation arms; it never mixes.**

**Step 3 — Bob replies.**
`encrypt_message_ratcheted` -> `session.encrypt` (ratchet.rs:485): sending
chain from T_{n+1} advances; header `(B_{k+1}, m)`. No root KDF on send
(the Qwen draft's supposed mixing point — it does not exist). Bob's KDF-2
mix happens inside his NEXT `handle_dh_ratchet`, not when replying.
`R_bob = R_{n+1}`.

**Step 4 — Alice receives Bob's reply (first sighting of `B_{k+1}`).**
`decrypt` (524): `dh_changed == true` (539-542) -> `handle_dh_ratchet(B_{k+1})`
under R5 trial:
- KDF-1 (590-594) recomputes T_{n+1}. Bob committed T_{n+1} at his KDF-2
  BEFORE ct1 existed (his HDR predated Step 1), i.e. with `None`.
  Candidates `[None, ss1]`: `ss1` variant fails AEAD; `None` verifies ->
  adopt `None`. `pq_pending_sent = ss1` RETAINED (not adopted). Output:
  `(R_{n+1}, recv chain for B_{k+1} epoch)` — bitwise equal to Bob's T_{n+1}.
- New keypair `A_{k+1}` (598-602).
- KDF-2 (605-609): `pq_pending_recv.take()` = `None` (Alice armed nothing).
  T_{n+2} = v2(R_{n+1}, dh(a_{k+1}, B_{k+1}), None) -> `(R_{n+2}, S_A')`.
`R_alice = R_{n+2}`. R2 continues: Alice's `A_{k+1}` epoch messages still
carry `(ct1, ek_A')`.

**Step 5 — Bob receives first `A_{k+1}` message.**
PQ fields: decapsulate ct1 again -> ss1; R4: fingerprint equals current
`pq_pending_recv` -> idempotent no-op. `dh_changed == true` ->
`handle_dh_ratchet(A_{k+1})`:
- KDF-1 trial: Bob's candidates `[None]` (+ his own `pq_pending_sent` if his
  reverse-direction cadence were outstanding — not here). Alice's T_{n+2}
  KDF-2 used `None` -> verifies. Output `(R_{n+2}, recv chain)`.
- KDF-2: **`pq_pending_recv.take()` = `Some(ss1)`** ->
  **T_{n+3} = v2(R_{n+2}, dh(b_{k+2}, A_{k+1}), Some(ss1))** ->
  `(R_{n+3}, S_B')`; `pq_last_mixed_fp = fp(ss1)`.
**THE MIX.** `R_bob = R_{n+3}`. Every transition from T_{n+3} onward is
poisoned against a classical-DH-only adversary.

**Step 6 — Bob replies under `B_{k+2}`; Alice receives first sighting.**
Alice's `handle_dh_ratchet(B_{k+2})`, R5 trial for T_{n+3}: candidates
`[None, ss1]` — `ss1` variant verifies (Bob mixed it) -> **adopt
`Some(ss1)`**: T_{n+3} recomputed identically; **clear `pq_pending_sent`
and `pq_pending_ct`** (confirmed; re-attachment stops). KDF-2: `None`.
`R_alice = R_{n+4}` after her KDF-2 (T_{n+4}).

Converged: ss1 mixed into exactly one transition (T_{n+3}) by both sides,
`fp(ss1)` recorded, all downstream roots and chains ss1-dependent.
Alternative interleaving (Bob's HDR postdates ct arrival because Alice's
epoch turned before the cadence, ping-pong style): Bob mixes at his FIRST
HDR after arming and Alice adopts one epoch earlier — steps 5-6 collapse
into steps 2-4. Both orderings converge because R5 follows Bob's actual
choice rather than a schedule; worst-case latency from cadence to confirmed
mix is 1.5 round trips.

---

## 5. Required companion hardenings

These are REQUIRED for the constraints to hold. H1/H2 also fix pre-existing
classical-path brick vectors that adversarial review of attempts 1-3 kept
rediscovering in PQ disguise.

- **H1 (skipped-keys before ratchet).** In `decrypt()` (524-566), attempt
  the skipped-key cache `(their_dh, message_number)` (627-630) BEFORE the
  `dh_changed` check, and never run `handle_dh_ratchet` for a header key
  that fails to advance the epoch (concretely: a delayed message from a
  previous epoch must route to the cache or fail cleanly — it must NOT
  re-trigger a ratchet). Today, ANY stale header key fires HDR (539-546),
  irreversibly replacing root/chains/keypair on an unauthenticated header
  — one delayed cross-epoch message bricks even a classical session. F3
  (strict alternation), and with it every argument in this spec and
  Signal-standard reorder tolerance, requires H1.
- **H2 (commit-on-success).** All session mutation performed during a
  decrypt — PQ staging (R3/R4), `handle_dh_ratchet` state (611-617),
  trial adoption (R5), skip-ahead cache inserts (646-657), `peer_confirmed`
  — commits only after AEAD success; on failure the session is untouched.
  Implementation shape (E-01c's choice): operate on a session clone and
  swap on success, or an explicit staging struct. This closes: (a) the
  attacker-injected-header ratchet-poisoning DoS that exists today, (b)
  the pre-verification PQ mutation at encrypt.rs:322-326 (which currently
  updates `pq_their_encaps_key` and clears `pq_pending_ct` on an
  unauthenticated message), and (c) makes R5's all-candidates-fail path
  exactly equivalent to today's failed decrypt.
- **D5 (AAD binding).** For suite-0x02 ratcheted envelopes, the message AEAD
  AAD becomes the canonical, length-prefixed concatenation:
  `sender_public_key || ratchet_dh_public || ratchet_message_number_le ||
  pq_kem_ciphertext? || pq_encaps_key?` (absent optional fields encode as
  zero-length; exact canonical encoding fixed in E-01c and unit-tested).
  Today the AAD is the sender public key alone (encrypt.rs:362, :292), so
  PQ fields are attacker-malleable: ML-KEM implicit rejection means a
  garbled ct "successfully" decapsulates to garbage on one side only —
  under ANY mixing design that is a guaranteed desync-brick primitive.
  With D5, garbling or stripping PQ fields fails the AEAD: equivalent to
  dropping the message, which the design already survives. **Not a wire
  format change** (no field added/removed/moved; AAD is computed, not
  transmitted) but it IS a decrypt-compatibility break for suite-0x02
  ratchet traffic between mixed versions — flagged as operator gate O-1
  (Section 8). Note: `ratchet_dh_public` and `message_number` are today
  only IMPLICITLY bound (via key selection); D5 makes header integrity
  explicit, which R5's trial logic prefers (a tampered header now fails
  closed uniformly).
- **P1 (persist pending state).** `SerializableRatchetSession`
  (session_manager.rs) currently reconstructs with ALL PQ state dropped —
  `None, None, None, None` at 506-509 (`pq_our_keypair`, `pq_prev_keypair`,
  `pq_their_encaps_key`, `pq_pending_ct`). Under this design that gap
  becomes fatal: if Alice restarts and loses `pq_pending_sent` after Bob
  armed, Bob mixes ss1 at his next KDF-2 and Alice's trial set is `[None]`
  — every candidate fails, permanently. E-01c MUST serialize
  `pq_pending_sent`, `pq_pending_recv`, `pq_last_mixed_fp`, AND the four
  existing dropped PQ fields (same defect class), zeroizing on
  serialization buffers. These are key-material-at-rest; storage goes
  through the existing session store posture (`store/` module boundary per
  `.claude/rules/rust.md`) — no new storage surface.

---

## 6. E-01a constraints — satisfaction walkthroughs (handoff deliverable 3)

**C1 SYNCHRONIZATION.** Mixing exists only inside `handle_dh_ratchet`
transitions. For each transition the early side's input is a deterministic
function of its own committed state (R1/R3/O1); the late side's input is
AEAD-verified equal (R5) — not inferred from timing or content. Identical
logical points by construction; observable anchors: header DH public
(epoch identity) + the triggering envelope's AEAD tag (choice identity).

**C2 LOSS-SAFETY.** Walkthroughs against actual code paths:
- L1: cadence message #100 lost. ct1 rides #101+ (R2). Bob arms on the first
  arrival; skip-ahead (646-657) covers the chain gap. If EVERY carrier is
  lost through the epoch turn, R2 keeps attaching in the next epoch. If ct1
  never arrives at all: Bob never arms, his KDF-2s mix `None`, Alice's
  trials keep resolving `None` (retaining `pq_pending_sent`) — the
  protocol runs exactly like today's inert-PQ behavior until a carrier
  lands. Nobody half-mixes. This is precisely the scenario that killed
  attempt 2, now inert.
- L2: ct1 delivered; Bob mixes at T_{n+3}; then ALL of Bob's `B_{k+2}`-epoch
  messages are lost. Alice stalls at R_{n+2}, keeps sending `A_{k+1}`
  traffic (still carrying ct1); Bob sees `dh_changed == false` on all of it
  (no second HDR — F3) and R4 blocks re-arming ss1. Whenever any `B_{k+2}`
  message finally lands, Alice's R5 adopts ss1. No interleaving of losses
  produces divergent committed transitions — only deferral.
- L3: Alice adopts (Step 6) but her subsequent epoch's messages are lost:
  adoption was Alice-local and consistent (she recomputed exactly Bob's
  committed T_{n+3}); Bob simply waits; alternation stalls safely.

**C3 REORDER-SAFETY.**
- Within-epoch: #103 (with ct1) before #100: arming is order-independent
  (R2 puts ct1 on both; R4 dedupes), AEAD via skipped keys (622-660).
- Cross-epoch: a delayed `A_k` message arriving after Bob processed
  `A_{k+1}`: H1 routes it to the skipped-key cache or clean failure — no
  ratchet, no PQ effect (R4 blocks its stale ct). Without H1 this reorder
  bricks classical sessions today; attempt 1's "reorder desync" lives in
  this pre-existing hole as much as in its asymmetric mixing.
- Epoch-boundary races (full-duplex crossing sends): covered by F3 — HDRs
  strictly alternate, each side runs exactly one HDR per peer epoch, and
  R5 verifies rather than assumes. The double-cadence case (both
  directions outstanding simultaneously) uses disjoint slots per direction
  (Section 3.3) and composes.

**C4 KDF SOUNDNESS.** Same `root_key_ratchet_v2` (782-798), same input
tuple per transition on both sides: root (inductively equal), dh_output
(DH commutativity, F1), pq (KEM correctness for existence + R5 AEAD
verification for agreement). No role-dependent parameters anywhere in the
state machine (R1-R6 are side-symmetric). Domain separation unchanged
(`ROOT_KDF_CONTEXT_V2`, ratchet.rs:31).

**C5 RATCHET EPOCH BINDING.** Literal: mixing occurs only inside
`handle_dh_ratchet`, whose sole trigger is the public DH component in the
envelope header (539-546). The cadence (%100) only SCHEDULES creation of
material (unchanged, per PQC_07 "do not weaken the cadence trigger");
commitment happens exclusively at epoch boundaries.

**C6 SYMMETRY PROOF.** Section 7.

Adversarial cases beyond the six:
- Stripping PQ fields (MITM): D5 -> AEAD failure -> message dropped ->
  L1 behavior. Compare: without D5+R5, stripping converts any deferred
  design into one-sided mixing — a remote unauthenticated brick.
- Garbled ct (implicit rejection): D5 -> AEAD failure pre-commit (H2).
- Replay of a ct-carrying message: message key already consumed
  (skipped-key `remove` 627-630) or behind-chain bail (637-639) -> AEAD/H2
  reject; R4 blocks re-arming even inside the same commit window.
- Fake DH public injection: H2 -> no surviving state change.

---

## 7. Symmetry proof outline (handoff deliverable 5)

**Invariant I (per-transition input equality).** For every transition index
t committed by both sides, the tuples `(t, hash(dh_output_t),
pq_present_t, fp(pq_t))` recorded at each side's `root_key_ratchet_v2`
call are equal, and each side's sequence is a prefix-interleaving of the
same total order (no side commits t+1 before t; KDF-1 recomputations match
the peer's KDF-2 originals bitwise).

**Instrumentation (test-utils feature only).** A `#[cfg(feature =
"test-utils")]` per-session transition log appended inside
`handle_dh_ratchet` (and `init_as_sender_hybrid`'s T1): entries
`(transition_index = 2*dh_step-ish counter maintained explicitly,
blake3(dh_output)[..8], pq_present, fp(pq))`. Root values themselves stay
out of logs; `root_key_bytes()` (217-220) already exists under test-utils
for endpoint assertions.

**Test 1 — deterministic dual-session harness (catches all three prior
failure modes).** Drive two real `RatchetSession`s through a scripted
delivery schedule (a `Vec<Deliver(msg_id) | Drop(msg_id) | Hold/Release>`
executed against real envelopes). After every delivered-and-verified
message, assert Invariant I over the logs. Regression scripts:
- FM-1 (attempt 1, asymmetric mix at crossing): script = burst 101 messages
  one-directionally, deliver #100 FIRST (the reorder from the auditor's
  brick trace). A buggy implementation that mixes receiver-side-only
  violates I at the first boundary transition; the correct design shows
  `pq_present == false` on both logs there (mix deferred).
- FM-2 (attempt 2, loss desync): script = drop every ct-carrying message,
  continue 3 epoch turns bidirectionally. Any mix recorded on one log and
  not the other violates I; correct design records none.
- FM-3 (attempt 3, one-round-trip-shifted): script = normal ping-pong with
  cadence; assert additionally that for the mixed transition t*, BOTH logs
  show `pq_present` at the SAME t*, and that every message of every epoch
  decrypts (the FM-3 bug fails decryption of the first message of the
  shifted epoch, which the harness surfaces as a hard test failure before
  the invariant is even checked).
- Load-bearing check (PQC_07 DoD): after the mixed epoch settles, corrupt
  one side's `pq_pending_sent` fingerprint-equivalent in a cloned session
  and assert its trial adoption REJECTS the peer's traffic (proves the pq
  input is load-bearing in the KDF, not decorative), while the untouched
  session continues.
- Restart persistence: serialize/deserialize one side mid-pending (P1) and
  assert convergence still occurs.
- test-utils counters: `pq_mixes_performed` equal on both sides == 1 per
  cadence cycle; `pq_last_mixed_fp` equal.
**Then rewrite the disabled assertion** in
`integration_pq_session.rs::test_pq_ratchet_cadence_refreshes_shared_secret`
(332-484): the assertion AS WRITTEN (471-483) compares Alice's root
immediately after a ONE-DIRECTIONAL 105-message loop; under ANY
epoch-bounded loss-safe design (constraints C1/C5) her root MUST be
unchanged at that point, because no epoch boundary occurred — the
assertion encodes attempt-2 semantics and would only pass for a
loss-UNSAFE design. Replacement: after the loop, run one full round trip
(Bob reply -> Alice receive -> Alice send -> Bob receive -> Bob reply ->
Alice receive, per Section 4), then assert (a) both roots differ from
their pre-cadence values, (b) transition logs satisfy Invariant I with
exactly one `pq_present` transition, (c) traffic still flows both ways.
The one-directional-flood-refresh gap remains real and remains explicitly
out of scope here (tracked: `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md`).

**Test 2 — property test (proptest, existing harness at
`core/src/crypto/proptest_harness.rs`).** Strategy generates: message
counts per direction (0..300), per-message fate (deliver / drop / delay-k
within a bounded window), cadence positions. Property: Invariant I holds
at every quiescent point, and any message the schedule delivers within the
skip window (MAX_SKIP_KEYS = 256, ratchet.rs:36) decrypts. Shrinking gives
minimal desync counterexamples if the implementation drifts.

**Test 3 — Kani harness (kani-proofs feature, per `.claude/rules/rust.md`).**
Bounded model, crypto stubbed deterministically (KDF = concatenation hash
stub; KEM = shared nondet 32B array delivered-or-not): state machine of
Section 3.2 driven by `kani::any()` schedule over a bounded horizon (2
cadence events, 4 epoch turns, loss/reorder booleans). Prove: (i) Invariant
I; (ii) no reachable state where one side's committed transition count
leads the peer's by more than 2 (F3); (iii) `pq_pending_sent` is cleared
only via a verified adoption; (iv) no double-mix of the same fingerprint.
This is the proof shape that would have rejected attempts 1 (i), 2 (i via
loss branch), and 3 (i + decrypt-failure unreachable check) before review.

---

## 8. Decisions and operator gates

- **chain.index: RESET to 0 (resolved — handoff deliverable 6).** Both
  chains are rebuilt via `Chain::new` (index 0) at 614-615, and this spec
  keeps that. Justification: (a) message numbers are chain-local key-slot
  indices; the skipped-key cache is keyed by `(their_dh, number)`
  (627-630), so cross-epoch continuity is provided by the cache, not by
  index continuation — preserving indices buys nothing the cache doesn't
  already provide; (b) the cadence trigger reads the chain-local index
  (encrypt.rs:375-378) — preserving indices across epochs would silently
  change cadence frequency semantics; (c) Signal-standard semantics (N=0
  per chain), matching every existing test's expectations
  (`message_number == 0` after epoch turns, e.g. integration flows);
  (d) preserve-proponents' loss argument (panelist split in PQC_07) is
  actually an argument for a previous-chain-length (PN) field — a wire
  format change, out of scope by mandate. With H1 in place, reset has no
  remaining downside surfaced by any prior review.
- **O-1 (operator gate): D5 AAD compatibility.** Suite-0x02 ratcheted
  traffic between a D5 peer and a pre-D5 peer fails decryption (clean
  AEAD failure, no state damage under H2). Recommendation: accept — the
  ongoing-PQ cadence path has never shipped in a GA release (v1.0.0
  pending; farm ships bootstrap-only per handoff Section 9), and the
  alternative (unauthenticated PQ fields) is a standing desync primitive
  under ANY mixing design. If rejected, E-01c must NOT proceed: the
  garbled-ct brick (Section 5, D5) has no non-AAD mitigation because
  ML-KEM implicit rejection makes garbage indistinguishable from honest
  decapsulation on the receiving side.
- **O-2 (operator tunable): re-attachment bandwidth.** R2 costs ~2272
  bytes/message (ct 1088 + ek 1184) while a secret is outstanding
  (typically one epoch turn; bounded by conversation latency, not by
  count — O1 prevents pileup). Acceptable default for the mesh; if later
  profiling objects, attach-every-Nth is safe WITHOUT redesign (R5/R4 are
  attachment-schedule-agnostic; only mix latency changes). Not a
  correctness parameter.
- **Optional hardening (non-blocking):** depth-2 `pq_last_mixed_fp`
  history, and a trial-candidate slot for one superseded
  `pq_pending_sent` generation. Redundant under O1+F3 by the arguments
  above; cheap insurance against unmodeled interleavings if the auditor
  wants belt-and-braces.

---

## 9. Pre-existing defects found during verification (flagged, not fixed here)

- **P2. `init_as_receiver_hybrid` stores a CIPHERTEXT as the peer's encaps
  key** (ratchet.rs:447: `pq_their_encaps_key =
  Some(hct.mlkem_ciphertext.to_vec())` — 1088-byte ct where a 1184-byte
  encapsulation key belongs). Bob's first own-direction cadence therefore
  attempts `encapsulate()` against a ciphertext and errors — silently, see
  P4. Self-heals only after the initiator's first genuine `pq_encaps_key`
  arrives (702). Under this design R2 delivers that key redundantly, so
  the window shrinks, but the initialization is simply wrong. File as its
  own ticket; one-line fix; touches `crypto/` so it rides E-01c's
  adversarial review.
- **P3. Bootstrap-ct conflation on the wire read path:**
  `decrypt_with_ratchet_fallback` (encrypt.rs:627-638) reads
  `pq_kem_ciphertext` as the bootstrap HCT for session creation — correct
  for message 1, but it means the SAME field is bootstrap-ct pre-
  confirmation and cadence-ct post-confirmation. The `peer_confirmed`
  gate at 310 handles it; documenting here because R2 increases how often
  the field is populated and future readers will ask.
- **P4. `if let Ok(...)` swallows cadence failures** (encrypt.rs:379): a
  failing PQ step (e.g., P2's malformed key) skips PQ fields with no log.
  E-01c should surface this via the observability path when touching the
  block for R1/O1.
- **P5. PQ state persistence gap** (session_manager.rs:506-509) — covered
  as P1 in Section 5 because this design upgrades it from "PQ cadence
  resets on restart" to "must fix".

---

## 10. What this design does NOT do (mandate compliance)

- No wire format change: `pq_kem_ciphertext` / `pq_encaps_key` field
  definitions untouched (R2 changes only population frequency; D5 changes
  only locally-computed AAD bytes).
- No KDF change: `root_key_ratchet_v2` (782-798) used as-is, both calls.
- No cadence-trigger change: the %100 logic (encrypt.rs:375-384) is
  preserved verbatim; O1 only gates re-entry while outstanding.
- No fix for one-directional-flood PQ refresh (no epoch boundary => no
  mix, by C5): explicitly out of scope, tracked in
  `PQC_07_PQ_REFRESH_WITHOUT_DH_CROSSING.md`.
- No code: E-01c implements R1-R6, O1, H1, H2, D5, P1 only after this spec
  carries an adversarial PASS in `HANDOFF/review/`.

## 11. E-01c Definition of Done (for the implementation task)

1. R1-R6 + O1 implemented per Sections 3; both `root_key_ratchet_v2` inputs
   wired per the 3.3 table; the 580-588 hardcoded-None block deleted.
2. H1, H2, D5, P1 implemented; D5 canonical AAD encoding unit-tested,
   including absent-field cases.
3. Test 1 harness with FM-1/FM-2/FM-3 regression scripts, load-bearing
   negative test, restart test; rewritten cadence integration assertion
   green (`--features test-utils`).
4. Test 2 proptest added to the existing harness; Test 3 Kani harness
   compiles and passes under `kani-proofs`.
5. Standard gates: `cargo build --workspace`, `cargo test --workspace
   --no-run`, fmt, clippy per `.claude/rules/build.md`; adversarial review
   per `.claude/rules/security.md` (crypto + this file's Sections 5-7 as
   the probe list).
