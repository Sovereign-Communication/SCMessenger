# Quantum Crypto Remediation Tasks — SCMessenger

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-10-03
Derived from: `HANDOFF/audit/QUANTUM_CRYPTO_AUDIT_2026-10-03.md` **revision 2**
Source branch observed: `preserve/shared-checkout-20261003` @ `e4050f84`

## How to read this document

The audit report is the sole source of truth. Every task below restates a
mechanism **as revision 2 states it** — not as revision 1 did. Three findings
changed meaning in revision 2 (F2 wrong, F3 overstated, F4 overstated) and one
was rescoped (F1). Anyone working from a revision-1 summary will get F2, F3 and
F4 wrong.

No new findings are introduced here. Two tasks (T0, T10) cover defects the
report flags but does not number as findings; they are labelled as such.

Mechanisms are reproduced from the report, not re-derived. Acceptance criteria
are written so a reviewer can check them **without trusting the report** —
each is a command, a grep, or a named test.

**Severity as of revision 2:** F2 CRITICAL, F4 CRITICAL, F3 HIGH, F1 HIGH,
F5 MEDIUM, F6 MEDIUM, F7 MEDIUM, F8 LOW, F9 LOW (verified negative).

**Amendment, 2026-10-03.** T2 has been rewritten. It was previously presented as
a wiring job ("wire up `save_contact_bundle`"). Two implementation attempts both
stopped: there is **no contact-establishment mechanism that could carry a bundle at
all**, which is one level deeper than the missing bundle exchange the audit
reported. `InviteToken::verify`/`verify_with_policy` and `verify_bundle` have zero
production callers; `BootstrapManager::accept_invite` is documented dead;
`InviteSystem` has zero users; contacts are created only by manual entry or backup
restore. T2 now records an operator transport decision and an accepted pinning
risk instead of an actionable wiring step. **T11 is new** — a pre-existing QR
budget defect found during that work, marked adjacent to T2 rather than part of it.
F1-F9 themselves are unchanged; this amendment corrects the task list, not the
audit.

---

## Dependency summary

```
T0  build unblock  ──┬─> T2  F2 bundle propagation ──┬──> T3  F3 persist PQ state
   (snapshot branch   │        (CRITICAL, DECIDED)     │
    only; no-op from   │                                 └──> T10 CI reachability guard
    main)             └──> T4  F4 encrypt at rest (CRITICAL)

T1  F1 ML-DSA primary identity (HIGH)  <── COUPLED to T2; hard blocker for
                                              closing T2's pinning risk
T5/T6/T7  layer hygiene (MEDIUM)  ──> after T0 only
T8  F8 fingerprint (LOW)           ──> after T0 only
T9  F9 no action (verified negative)
T11 QR budget breach for dual-signed invites (MEDIUM, pre-existing)
    ──> independent; it constrains T2's transport choice, is not part of it
```

### The one coupling that matters

**T2 without T1 yields post-quantum assurance a CRQC invalidates.** Wiring bundle
propagation makes ML-KEM-768 actually protect the send path, but the session root
key still folds a transcript hash computed over two **Ed25519** public keys
(`negotiation.rs:52-64`, via `ratchet.rs:478-484`). An attacker who breaks Ed25519
presents their own Ed25519, X25519 and ML-KEM encapsulation keys, knows both DH
secrets, and decapsulates the ML-KEM themselves. The report's wording:
*"fixing F2 without F1 would advertise post-quantum protection that does not hold
against a CRQC. Sequence them together."*

T1 is **not** a hard blocker on T2 — T2 is worth landing on its own for classical
security. But the two must not be reported as separate completed efforts. T1
requires operator sign-off (AGENTS.md rule 9, architecture direction) and will run
long, so raise that decision at T2 kickoff, not at T2 completion.

### Two ordering constraints that are easy to get wrong

1. **T10 cannot land before T2** unless it ships in advisory mode. The guard is
   designed to fail while `save_contact_bundle` has no production caller. Shipping
   it blocking-first turns CI red on a branch that is already red for a different
   reason. Advisory → blocking, per T10's definition of done.
2. **T3 is latent until T2 lands.** F3's trigger requires a hybrid session to exist
   in a backup. With T2 unfixed, no hybrid session is ever created in production,
   so no backup can contain one. Sequence T3 after T2, not before.

---

## Phase 0 — Unblock (must land first)

### T0 — Fix the E0603 build break *(prerequisite, not an audit finding)*

**Severity:** blocking, **but scoped to the snapshot branch only.**

**Scope correction:** the break was observed on `preserve/shared-checkout-20261003`
@ `e4050f84`. `main` declares the function `pub(crate)` at
`core/src/store/ledger_entry.rs:498` and does **not** have this defect. T0 is a
prerequisite for verifying *this task list* only if the work is done from the
snapshot branch. If remediation is done from `main`, T0 is already satisfied —
verify with `cargo check -p scmessenger-core` and close it.

**Mechanism (report, BUILD STATE):** `core/src/store/outbox.rs:196` calls
`crate::store::ledger_entry::canonical_ledger_peer_id`, which
`core/src/store/ledger_entry.rs:489` declares without `pub` on the snapshot
branch. `git diff --stat main..HEAD -- core/src/store/ledger_entry.rs` reports that
branch is **294 lines short of main**.

**Files/lines:** `core/src/store/outbox.rs:196`,
`core/src/store/ledger_entry.rs:489`

**Acceptance criteria (checkable):**
- `cargo check -p scmessenger-core` exits `0`.
- `cargo check -p scmessenger-core 2>&1 | grep -c "E0603"` returns `0`.
- `cargo test -p scmessenger-core --no-run` exits `0` — i.e. the PQ integration
  suites now **compile**. At present they cannot run at all, which is why no
  finding in the report is runtime-verified.
- Re-run the suites named in the report's BUILD STATE (see below) and record the
  result on the task, so the audit's static claims gain a runtime baseline.

**Lands first:** yes, for any work done from the snapshot branch. Blocks T1–T8 and
T10 in that case. No-op when working from `main`.

**Note:** the report explicitly says this break is *"unrelated to the cryptography
and is not developed further here, but it should be tracked separately."* This task
is that separate track. Do not fold it into a crypto PR — and if the work starts
from `main`, do not spend a task on it at all.

---

## Phase 1 — Live CRITICALs

### T2 — Make the hybrid PQ path reachable *(finding F2, CRITICAL — BLOCKED ON AN OPERATOR DECISION)*

> **Rewritten 2026-10-03 after two implementation attempts both stopped.** The
> revision-1 framing of this task — "wire up `save_contact_bundle`" — was wrong,
> and acting on it would have produced a protocol guess on a security-critical
> send path. What follows is what was actually established by tracing, and the
> task is now explicitly gated rather than actionable.

**Mechanism (report, revision 2 — not revision 1):** the hybrid ML-KEM-768 /
ML-DSA-65 path is **unreachable from the production send path** because
`save_contact_bundle` has **zero production callers**. `encrypt_with_ratchet_fallback`
selects the hybrid path only when both `our_bundle` and `recipient_bundle` are
`Some`; production `recipient_bundle` is always `None`, so ML-KEM-768 is never
invoked on the send path. *Revision 1 attributed this to `require_pq = false` — that
was wrong about the mechanism.*

**Trace the task must reproduce (verified against `origin/main` @ `19229287`):**
- `iron_core.rs:1032` — `our_bundle`, always `Some`.
- `iron_core.rs:1034-1038` — `get_contact_bundle`, returns `None`.
- `store/contacts.rs:779` — reads a separate sled keyspace, `contact_bundle_key`
  (`store/contacts.rs:85`).
- `store/contacts.rs:25-38` — `Contact` has **no bundle field**.
- `store/contacts.rs:765` — `save_contact_bundle`, the only writer, **no production
  callers**. The one other `contact_bundle_key` reference is
  `store/contacts.rs:754`, inside `remove()`, which **deletes** the key — it is not
  a second writer.
- Lands at `encrypt.rs:489-501` → `encrypt.rs:580` (legacy static ECDH), or
  `encrypt.rs:549` → `session_manager.rs:142` (classical suite-0x01 ratchet).

#### Why this is not a wiring job — the established evidence

The revision-1 migration note claimed the machinery already exists and only needed
connecting. Two implementation passes showed otherwise. On `main`:

1. **There is no contact-establishment mechanism that could carry a bundle.**
   `InviteToken::verify` and `verify_with_policy` have **zero production callers**;
   the type appears in production only at `iron_core.rs:2564` (deserializing bytes)
   and a re-export at `relay/mod.rs:32`.
2. **`BootstrapManager::accept_invite` / `parse_qr_data` are documented dead.**
   `relay/bootstrap.rs:134` says so explicitly and warns that if the module is ever
   revived it must route through `InviteToken`.
3. **`InviteSystem` has zero users** — a bare re-export, nothing constructs it.
4. **Contacts are created only by manual entry or backup restore:**
   `handle_add_contact` in `cli/src/api.rs:1010` and `cli/src/api_axum.rs:321`, the
   CLI's `add_contact_via_api` (`cli/src/main.rs:1897`), the WASM `"add_contact"`
   RPC (`core/src/wasm_support/rpc.rs:234`), and `import_identity_backup`
   (`iron_core.rs:2091`). A person types a peer id and public key.
5. **`verify_bundle` has zero production callers.** The function that would
   validate an inbound peer bundle is never called.
6. **`EnvelopeV2.pq_encaps_key` is not a usable substitute.** It does carry an
   ML-KEM key, but is consumed only by `handle_incoming_pq_fields`
   (`crypto/ratchet.rs:1170`) inside an **already-established hybrid session**, and
   is never written to the contact store. That is circular: you need the bundle to
   establish the hybrid session that carries the key you would need to build the
   bundle.

`x25519_public` could be derived from `ed25519_public` via the existing birational
map. `mlkem_encaps_key` **cannot** — it is 1184 bytes of independent randomness with
no derivation path, and `hybrid_encapsulate` length-validates it.

**Consequence:** there is no code path by which a node obtains a contact's bundle
before sending to them. Supplying one requires designing a new protocol surface,
which is an architecture decision reserved to the operator (AGENTS.md rule 9).

#### Operator decision (recorded 2026-10-03)

The operator selected **bundle announcement over the existing authenticated message
channel**: the send path gains a way to learn a contact's bundle, and that path
wires `verify_bundle` and `save_contact_bundle` into production. Bundle
**key-change detection and pinning are explicitly deferred and remain unsolved** —
recorded as accepted risk, not as an oversight.

**Accepted risk, stated plainly:** because pinning is deferred, a peer may
re-advertise a different bundle at will, so anyone able to write to the channel
can substitute an ML-KEM key. This is the same Ed25519-anchor weakness F1
describes, now reachable through bundle substitution rather than through a CRQC.
Pinning is the only thing that closes it, so **T1 is a hard blocker for closing
this risk**, not merely a parallel concern.

**Still to settle before or during implementation:**
- Whether a re-advertised bundle for an existing contact is accepted, ignored, or
  flagged. Until this is chosen, the implementer must pick one and document it.
- Whether the announcement is sent once, on contact creation, or re-sent when a
  send finds no bundle.

**Note:** the invite/QR path was considered and rejected as the transport. It has
no production acceptance flow (evidence above), and a dual-signed token already
exceeds the QR budget — see **T11**.

**Depends on:** T0, plus the operator decision above. Coupled to T1 (see coupling
note): if pinning is part of the chosen transport, T1 becomes a hard blocker rather
than a parallel concern.

**Acceptance criteria (checkable, and achievable only once the transport is chosen):**
- The chosen transport decision is recorded on file, naming the surface, the
  authentication, and the pinning/key-change answer (or an explicit accepted-risk
  note).
- `grep -rn "save_contact_bundle" core/src cli/src --include=*.rs | grep -v
  "#\[cfg(test)\]"` returns at least one **non-test** call site on the real
  production path. Today it returns none.
- A **new** integration test drives the production send path end to end — through
  whatever contact-establishment path the chosen transport uses, not the crypto
  layer directly and not a hand-seeded contact store — and asserts the emitted
  envelope is `WireEnvelope::V2` with `suite == 0x03` and
  `pq_kem_ciphertext.is_some()`.
- Existing PQ tests (`integration_pq_verification_suite.rs`,
  `integration_pq_session.rs`, `test_v2_hybrid_envelope_forgery_is_rejected.rs`)
  pass **unmodified**. They hand-seed the contact store, so they prove the
  primitives work and prove nothing about reachability.
- `verify_bundle` gains a production caller, and a test asserts a bundle with a
  tampered signature is refused on the real path.
- Only once the above hold: `require_pq` (`iron_core.rs:1051`) becomes `true`, and
  a test asserts a peer with no bundle is refused rather than silently downgraded.

**Watch for:** a green PQ suite is **not** evidence T2 is done. That is the exact
failure mode that let this defect survive.

---

### T4 — Encrypt the sled backend at rest *(finding F4, CRITICAL)*

**Mechanism (report, revision 2):** identity secrets are persisted to sled **in
the clear**. `save_keys` does `db.put(IDENTITY_KEY, &keys.to_bytes())` with raw
bytes; the `zeroize()` applied afterwards touches only the local `Vec`. Anyone with
read access to the sled directory holds the complete identity.

**The removed claim — do not re-add:** the ratchet-session half of revision 1 is
**out of scope**. `RatchetSessionManager::with_backend` is test-only, so
`ratchet_sessions_v1` is never written in production. Revision 1's assertion that
root keys sit in the clear was wrong. The doc comment at `session_manager.rs:307`
(*"the caller must ensure the storage is encrypted at rest"*) is currently
unviolated **only vacuously** — treat it as a trap for whoever later wires a
session-manager backend, not as a current vulnerability.

**Files/lines:** `iron_core.rs:~430` → `identity/mod.rs:37-39` →
`identity/mod.rs:96`, `:167` → `identity/store.rs:53-63` →
`identity/keys.rs:314-345`; `core/src/store/backend.rs`

**Migration (report):** wrap the sled backend in an XChaCha20-Poly1305 envelope
keyed by a device-bound key from the platform keystore (Keystore / Secure Enclave).
The `derive_key_blake3` device-bound pattern in `backup.rs` is the existing
precedent.

**Depends on:** T0. Independent of T2.

**Acceptance criteria (checkable):**
- `grep -n "encrypt\|Cipher\|Aes\|Secret" core/src/store/backend.rs` returns
  encryption code. It returns **nothing** today.
- A test writes an identity, reads the raw stored bytes for `IDENTITY_KEY`, and
  asserts they differ from `keys.to_bytes()` and that `IdentityKeys::from_bytes`
  on the raw stored blob **fails**. Full identity recovery from a disk image
  becomes impossible.
- A migration path for **existing plaintext installs** is written and tested —
  otherwise this silently bricks every node that already has a store.
- The key comes from the platform keystore; the device-unlock / keystore-loss
  behaviour is specified and covered by a test.

---

## Phase 2 — HIGH, and the CI guard

### T1 — Make ML-DSA-65 primary, bind a PQ-verifiable transcript *(finding F1, HIGH)*

**Mechanism (report, revision 2):** PQ coverage stops at the Ed25519 identity
boundary. The session root key folds `ss_hybrid` (PQ), `dh_static` (X25519) and
`transcript_hash` — a blake3 hash over **the two Ed25519 public keys**. ML-DSA-65
does **not** mitigate this: `verify_bundle` requires both signatures to verify,
which is useless against a quantum attacker, because a forger supplies a valid
ML-DSA keypair of their own.

**Scoped (revision 2):** forward-looking CRQC / harvest-now risk, **not** a live
classical defect. Nothing is broken today.

**Files/lines:** `ratchet.rs:478-484`; `negotiation.rs:52-64`;
`identity/keys.rs:605-620`; `transport/swarm.rs:3472`; `relay/client.rs:436`

**Migration (report, verbatim):** (1) accept a bundle if **either** signature
verifies, logging the other; (2) fold ML-DSA into `negotiate_suite`'s transcript
material so the session root binds a PQ-verifiable value; (3) treat the Noise/QUIC
layer as a confidentiality-only tunnel and never as the authenticity anchor, since
libp2p has no PQ Noise suite.

**Depends on:** T0. **Coupled to T2** — see the coupling note above. Requires
operator sign-off before implementation (rule 9).

**Acceptance criteria (checkable):**
- Operator decision recorded on file before code is written (rule 9).
- A test asserts a bundle whose Ed25519 signature is **valid** and whose ML-DSA
  signature is **garbage** is now **rejected**. This is the load-bearing check: it
  fails today, because `verify_bundle` (`keys.rs:605-620`) currently returns
  `Err` only when *both* fail.
- A test asserts that changing **only** the ML-DSA public key changes the
  negotiation transcript hash — i.e. the PQ key is genuinely bound into the root
  key, not merely present in the bundle.
- The mirror test asserts changing only the Ed25519 key also still changes it.
- A documented statement records that PeerId cannot change and explains how
  identity continuity is preserved across the Ed25519 → ML-DSA transition.

---

### T3 — Persist PQ state across session restore *(finding F3, HIGH)*

**Mechanism (report, revision 2):** `into_session` passes `negotiated_suite`
through but sets every PQ field to `None`, while `is_pq_hybrid()` reads only
`negotiated_suite` — so a restored session reports itself hybrid while holding no
ML-KEM secret, and the degradation is silent. The downstream failure is swallowed
by `if let Ok(...)` at `encrypt.rs:384`, and the receiver-side anti-stripping guard
does not trip because it also requires `pq_their_encaps_key.is_some()`.

**Trigger (corrected in revision 2):** the **sole** production trigger is
`import_identity_backup` → `deserialize_sessions_strict`. There is **no** restart
path — production never persists sessions. Revision 1's "after any restart" was
wrong.

**Files/lines:** `session_manager.rs:539-545`; `ratchet.rs:781`, `:1129`, `:1240`;
`encrypt.rs:320`, `:322`, `:384`; `iron_core.rs:2030-2033`

**Migration (report):** persist the ML-KEM keypair (available as a 64-byte seed via
`pq::from_seed`) plus `pq_their_encaps_key` and `pq_last_mixed_fp` in
`SerializableRatchetSession`, or force re-establishment on restore.

**Depends on:** T0. **Latent until T2 lands** — see ordering constraint 2. Must
land before any change that gives the session manager a real backend, which would
also make restore inherit plaintext root keys (T4's removed-claim trap).

**Acceptance criteria (checkable):**
- A round-trip test — establish a real hybrid session, `serialize_sessions()`,
  `deserialize_sessions_strict()` — asserts the restored session satisfies **both**
  `is_pq_hybrid() == true` **and** `pq_our_keypair.is_some()`. The second assertion
  **fails today** and is the actual bug.
- The same test asserts `perform_pq_ratchet_step()` returns `Ok` on the restored
  session. It currently returns `Err` ("No PQ encapsulation key from peer").
- A negative test asserts a session whose PQ fields are genuinely absent is
  **rejected or re-negotiated**, not silently accepted as hybrid.
- Backup round-trip integration coverage in `core/tests/integration_backup.rs`
  exercises the PQ case, which today it does not.

---

### T10 — CI reachability guard for the rule-16 defect class *(not an audit finding)*

**Severity:** structural. Prevents recurrence of the F2 class rather than any
single defect.

**Mechanism:** F2 is a rule-16 failure — the primitive is real, reviewed and
tested, and has **no production caller**. The PQ suites pass because they
hand-seed the contact store. Nothing in the build can detect that class of
defect today. (AGENTS.md documents a prior instance where nine features shipped
unreachable for the same reason.)

**Depends on:** T0 (CI cannot run). **Gated on T2** — see ordering constraint 1.

**Scope — guard the defect class, do not re-assert the audit's findings:**
1. **Reachability test (primary).** An integration test drives the production send
   path end-to-end and asserts a PQ envelope emerges (`suite == 0x03`,
   `pq_kem_ciphertext.is_some()`). This is what actually failed to catch F2.
2. **Symbol-level static check.** Assert each public PQ symbol in
   `core/src/crypto/pq/` has at least one call site outside `#[cfg(test)]` modules
   and outside `core/tests/`. Note this sub-check **passes today** for
   `crypto/pq/mldsa.rs` and `crypto/pq/hybrid.rs`, which are both genuinely
   reached from `identity/keys.rs`, `relay/invite.rs`, `ratchet.rs` and
   `privacy/onion.rs`. It is defence in depth, not the fix — the missing link is at
   the contact-store layer, which check 1 covers.

**Acceptance criteria (checkable):**
- Both checks exist in CI and fail when their condition is violated (prove this by
  temporarily reverting T2 on a scratch branch and observing the guard go red).
- **Definition of done requires the guard to be green with T2 landed.** Ship it
  advisory/warn-only first, flip to blocking only once T2 is merged — otherwise
  the guard blocks a branch that already does not compile.
- A written note in the guard's source explains why it exists and points at
  F2, so a future reader does not "simplify" it away.

---

## Phase 3 — MEDIUM, independent

Includes T11, a pre-existing defect that is adjacent to T2 rather than part of it.

All three depend on T0 only. None blocks another.

### T5 — Stop reusing the Ed25519 signing key as an X25519 ECDH key *(finding F5, MEDIUM)*

**Mechanism (report):** `ed25519_to_x25519_secret` feeds the legacy per-message
path, the classical ratchet and Wi-Fi Aware. Suite 0x03 deliberately avoids this
(*"avoid cross-protocol key reuse"*, `ratchet.rs:466-470`); the classical and
Wi-Fi Aware paths were not updated.

**Files/lines:** `crypto/encrypt.rs:40-48`; `ratchet.rs:349`, `:404`, `:466-470`;
`iron_core.rs:1866`

**Migration (report):** point `iron_core.rs:1866` and the classical
`init_as_sender` / `init_as_receiver` initial-DH at
`keys.x25519_encryption_secret`, which already exists and is already published in
the bundle. Retire `ed25519_to_x25519_secret` once the legacy path is gone.

**Acceptance criteria (checkable):**
- `grep -rn "ed25519_to_x25519_secret" core/src --include=*.rs` returns no call
  site outside `crypto/`, `privacy/` and `#[cfg(test)]` modules.
- Interop preserved: existing sessions established before the change still
  decrypt, or the report's suite-freeze discipline (`negotiation.rs:16-27`) is
  applied and a **new** suite ID minted — never an in-place redefinition. The
  report records that an in-place redefinition of suite 0x02 is precisely the bug
  that suite 0x03 exists to escape.

---

### T6 — Move hashing out of the DSPy layer; remove the insecure golden examples *(finding F6, MEDIUM)*

**Mechanism (report):** `crypto/mod.rs` re-exports `blake3_hash` from the DSPy
prompt-schema layer, so crypto's public API depends on `dspy` — an inverted
dependency that reaches production as FFI (`dspy_blake3_hash`, `dspy_get_signature`,
consumed by `cli/src/server.rs` and `wasm/src/lib.rs`).
`dspy/signatures.rs:107-131` is a **string literal containing insecure example
code** — raw ChaCha20 keystream plus a same-key Poly1305 tag, the textbook misuse
pattern, and it would not compile. It is prompt material for an AI coding agent:
a live path for an agent to copy an unauthenticated cipher into the codebase.

**Files/lines:** `crypto/mod.rs:24`; `dspy/signatures.rs:88-104`, `:107-131`,
`:134`; `iron_core.rs` (`dspy_blake3_hash`, `dspy_get_signature`)

**Migration (report):** move `blake3_hash` into `crypto/`, have `dspy` import from
`crypto`; replace both golden examples with correct AEAD usage or delete them.

**Acceptance criteria (checkable):**
- `grep -n "dspy" core/src/crypto/mod.rs` returns nothing — the inverted dependency
  is gone.
- `grep -n "GOLDEN_XCHACHA20_ENCRYPTION\|GOLDEN_CURVE25519_KEYGEN"
  core/src/dspy/signatures.rs` returns nothing, or the remaining literals contain
  no same-key MAC pattern and no `ChaCha20::new` misuse.
- A grep for the misuse pattern across all string literals in `core/src/dspy/`
  returns nothing.
- The FFI surface `dspy_blake3_hash` / `dspy_get_signature` still builds for the
  CLI and WASM targets.

---

### T7 — Consolidate Ed25519 signing out of three non-crypto layers *(finding F7, MEDIUM)*

**Mechanism (report):** Ed25519 signing is duplicated outside `crypto/` — drift
envelopes carry their own blake3 canonicalization plus sign/verify, the message
codec signs in the serialization layer, and `iron_core.rs` holds a
`blake3::derive_key` KDF in the facade. Three canonicalizations are a maintenance
hazard for any future PQ signature rollout (T1).

**Files/lines:** `drift/envelope.rs:666-717`, `:720`, `:728`;
`message/codec.rs:449`, `:515`, `:619`; `iron_core.rs:1872`

**Migration (report):** route drift envelope signing through `crypto::`.

**Acceptance criteria (checkable):**
- `grep -rn "ed25519_dalek" core/src/drift core/src/message --include=*.rs`
  returns no direct use; both call into `crypto::` instead.
- `grep -rn "blake3::" core/src/iron_core.rs` returns no `derive_key` call —
  the KDF moves behind `crypto::`.
- Drift envelope wire format is byte-identical before and after, proven by a
  known-answer test. **This task must not change any wire format** — it is
  refactoring only, and the suite-freeze discipline applies.

---

### T11 — A dual-signed invite cannot be QR-encoded at all *(pre-existing defect, adjacent to T2 — NOT an audit finding)*

**Severity:** MEDIUM. Pre-existing and independent of T2, but it independently
rules out the invite/QR route as a bundle transport, and it is invisible to the
current test suite.

**Mechanism:** `QR_BYTE_BUDGET` is `2953` characters (`relay/invite.rs:64`).
`to_qr_payload` base64-encodes the bincode token and returns
`InviteError::PayloadTooLarge` when the result exceeds that budget. A v2
dual-signed token already carries an ML-DSA-65 public key (1952 B) **and** an
ML-DSA-65 signature (3309 B) — 5261 B before anything else. Computed against the
wire shape with a 16-entry seed ledger:

| Token shape | approx. base64 | vs. 2953 budget |
|---|---|---|
| v1 Ed25519-only, 16-entry ledger | ~1172 | fits |
| **v2 dual-signed, 16-entry ledger** | **~8200** | **~2.8x over** |
| v2 + ML-KEM encaps key (1184 B) + X25519 (32 B) | ~9832 | ~3.3x over |

So a post-quantum-signed invite cannot be put in a QR code today, and adding a
bundle would make it worse rather than better.

**MEASUREMENT CAVEAT:** these figures are **computed analytically** from the wire
shapes and estimated string lengths, not measured by executing
`to_qr_payload`. Confirm with a real measurement before acting on the exact
numbers; the direction and order of magnitude are not in doubt, the precision is.

**Why it is missed:** the only test covering this budget,
`test_seed_ledger_full_invite_fits_qr_budget`, builds its token with
`sign_token(...)` — **Ed25519-only**. No test exercises the v2 dual-signed shape,
so the budget breach is never observed.

**Acceptance criteria (checkable):**
- A test builds a **v2 dual-signed** token with a full seed ledger and calls
  `to_qr_payload()`. The measured result is recorded. This is the missing coverage
  regardless of how the defect is then fixed.
- Depending on the recorded measurement: either the dual-signed token is made to
  fit (compression, a shorter PQ representation, or dropping the ledger on PQ
  tokens), or dual-signed invites are formally re-scoped to non-QR transports and
  the QR UI refuses them with a clear error rather than a size failure.
- `test_seed_ledger_full_invite_fits_qr_budget` is **extended, not replaced**, to
  cover the v2 shape.
- An explicit note records that this is why the invite/QR path was rejected as a
  bundle transport for T2.

**Depends on:** T0. Independent of T2's implementation; it constrains T2's choice
of transport rather than being part of it.

---

## Phase 4 — LOW

### T8 — Stop retaining a truncated hash of an ML-KEM shared secret *(finding F8, LOW)*

**Mechanism (report):** `pq_ss_fingerprint` is the first 16 bytes of BLAKE3 of the
ML-KEM shared secret, retained in `pq_last_mixed_fp` for deduplication. It is not
transmitted and not persisted (`session_manager.rs:545` passes `None`), so exposure
is memory-local — but a truncated hash of a fresh secret is derived material that
should not be retained.

**Files/lines:** `ratchet.rs:67-72`; `session_manager.rs:545`

**Migration (report):** compare a full 32-byte value, or a session-scoped
non-reversible tag, instead.

**Depends on:** T0. Independent of everything else.

**Acceptance criteria (checkable):**
- `pq_ss_fingerprint` no longer truncates to 16 bytes, or is replaced by a
  session-scoped tag. Either satisfies the report.
- The PQ deduplication behaviour it serves still works — a test asserts a repeated
  secret is still detected as duplicate and a fresh secret is still accepted.

---

### T9 — No remediation *(finding F9, LOW — verified negative)*

**Mechanism (report):** **no real secrets found.** Hardcoded hex in tests is all
synthetic; `self_certifying_keypair` has all 11 call sites in `#[cfg(test)]`
modules and is verified unreachable from production; `scmessenger-farm-sim-key-v2.pem`
is gitignored and `git ls-files` returns no `.pem`/`.key`; the Argon2id value at
`backup.rs:412` is a known-answer vector, not a key.

**This is a task with no code change.** It is listed so the report's negative
result is not silently dropped from the remediation set, and so a future reader
does not "fix" a non-issue.

**Acceptance criteria (checkable):**
- No code change.
- On a periodic re-audit (or when `.gitignore` entry `.gitignore:252` changes),
  re-run: `git ls-files | grep -iE "\.(pem|key|p12|keystore|jks|der|crt)$"` returns
  nothing, and `grep -rn "self_certifying_keypair" core/src` returns only
  `#[cfg(test)]` call sites.

---

## What this document deliberately does not contain

- No new **audit** findings. F9 being a negative result and the E0603 break being a
  prerequisite are both faithful to the report. **T11 is the one exception**: it is
  a pre-existing defect discovered during a T2 implementation attempt, recorded as
  its own task, explicitly marked adjacent to T2 rather than folded into it, and
  flagged as analytically computed. It is not presented as part of F1-F9.
- No remediation for the **removed** revision-1 claims (plaintext ratchet sessions
  in sled; `require_pq = false` as a live downgrade). Both were retracted in
  revision 2 and re-adding them would reintroduce two errors the verification pass
  specifically caught.
- No implementation of T2's transport. The operator has recorded the choice; this
  document states it, and the accepted risk it carries, without implementing it.
- No work estimates, owner assignments or dates. The report assigns none, and
  inventing them would misrepresent the source.