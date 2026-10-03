# PQC-05/06/07 Adversarial Review — Consolidated Verdict

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## UPDATE 2026-07-12: gap did NOT close, changed shape — VERDICT STILL FAIL

The 2026-07-11 pass below found the PQ ratchet functions were completely
UNWIRED (never called from the live encrypt/decrypt path) and recorded that
`PQC_07_WIRE_RATCHET_STEP.md` landed to fix it. It did NOT fully fix it.
Later the same session (2026-07-11, before this file's next update),
empirical `eprintln!` tracing during the PQC-07 cadence-test work found
that even after the wiring landed, `handle_dh_ratchet`
(`core/src/crypto/ratchet.rs`, ~line 650-657) hardcodes its `pq_ss`
parameter to `None` unconditionally — so `perform_pq_ratchet_step`/
`handle_incoming_pq_fields` now genuinely get CALLED (the original
"unwired" bug is fixed), but their output still never reaches
`root_key_ratchet_v2`, because the specific line that would pass the fresh
PQ secret through instead passes a hardcoded `None`. Filed as
`HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md`, still open,
NOT yet fixed.

A fresh adversarial pass today (2026-07-12, qwen3-vl-235b-a22b-thinking,
dispatched via a direct raw call after finding `delegate_task.py`'s prompt
wrapper unfit for review tasks — see Tooling note appended at the bottom of
this file) independently re-confirmed the SAME bug, same file, same
function, from a cold read with no knowledge of the earlier finding. Two
independent methods (empirical trace, static AI review) agree. Also caught
one CONFIRMED FALSE POSITIVE from the same review pass (a "CRITICAL" over
`pq/mod.rs`'s `decapsulate` misreading ML-KEM's standard implicit-rejection
countermeasure as a vulnerability — do not act on it) and one CONFIRMED,
if-imprecisely-described, MEDIUM finding (`skipped_keys` genuinely does not
survive session persistence — `reconstruct()` has no such parameter at all,
and `session_manager.rs` never mentions `skipped_keys`, so it's not
"discarded on reconstruct" so much as "never entered the persisted format
to begin with"; same net effect). See the bottom of this file for the full
2026-07-12 verdict detail.

**Bottom line: this checkpoint has now failed twice, on the same
underlying CRITICAL gap (PQ secret never reaches the root key), across two
separate wiring/fix attempts. PQC-09+ remains frozen until
`PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` is actually fixed AND a
THIRD review pass confirms it — do not treat "the wiring exists now" as
sufficient again without independently tracing that the secret reaches
`root_key_ratchet_v2`.**

---

Date: 2026-07-11
Reviewers: Qwen3-VL-235B-thinking (Part A: pq/mod.rs, session_manager.rs,
negotiation.rs), NVIDIA Nemotron-3-Ultra-550B via OpenRouter (Part B:
ratchet.rs, encrypt.rs, integration_pq_session.rs). Orchestrator
(this session) independently verified the CRITICAL findings against
current source before recording them below.

Process note: Qwen3-VL-235B-thinking failed Part B twice (once combined,
once ratchet.rs-alone) by hallucinating a plausible-but-wrong
reconstruction of the file instead of reviewing it (e.g. silently
inventing a version where the PQ shared secret is always discarded) —
neither hallucinated response was applied (no `--apply` used). OpenRouter's
Nemotron-3-Ultra produced a real prose-only verdict on the first retry with
a reinforced anti-code-dump prompt.

## VERDICT: FAIL (one CRITICAL integration gap, confirmed) -- WIRING FIX LANDED 2026-07-11

`PQC_07_WIRE_RATCHET_STEP.md` is implemented and committed: sender side now
triggers `perform_pq_ratchet_step()` every 100 messages post-confirmation;
receiver side calls `validate_pq_fields_present()` +
`handle_incoming_pq_fields()` for post-confirmation messages only (the
bootstrap ciphertext on the first message is deliberately NOT reprocessed
here, since `init_as_receiver_hybrid` already consumes it at session
setup). Full workspace compile gate green, all existing tests pass.
REMAINING GAP: no test exercises the 100-message cadence path itself (see
`HANDOFF/todo/PQC_07_CADENCE_TEST_COVERAGE.md`) -- treat this finding as
"fix landed, not yet proven by a dedicated test" until that lands. A
second adversarial pass should re-review the actual wiring code once that
test exists, since the fix was hand-applied by the orchestrator after two
Qwen dispatch failures (truncation, then a diff that didn't apply) rather
than a clean model-generated-and-verified round trip.

## CONFIRMED CRITICAL FINDING

**PQC-07's PQ ratchet is implemented but never wired into the live
encrypt/decrypt path.**

- `encrypt_message_ratcheted` (`core/src/crypto/encrypt.rs:330-378`) never
  calls `RatchetSession::perform_pq_ratchet_step()`. It only ever sends the
  ORIGINAL bootstrap ML-KEM ciphertext (`session.bootstrap_hct`) on the
  first message before `peer_confirmed`; every subsequent message sends
  `(None, None)` for `pq_kem_ciphertext`/`pq_encaps_key` (line 341, 346).
- `decrypt_message_ratcheted_v2` (`core/src/crypto/encrypt.rs:276-313`)
  never reads `envelope.pq_kem_ciphertext`/`pq_encaps_key` and never calls
  `session.handle_incoming_pq_fields(...)`. It calls `session.decrypt(...)`
  directly with no PQ-field handling at all.
- Consequence: `self.pq_ss` (the ML-KEM shared secret mixed into
  `root_key_ratchet_v2` on every DH ratchet step, `ratchet.rs`
  `handle_dh_ratchet`) is set ONCE at session bootstrap
  (`init_as_sender_hybrid`/`init_as_receiver_hybrid`) and never refreshed.
  Every later DH ratchet step reuses the SAME initial ML-KEM shared secret
  via `self.pq_ss.clone()` — there is no periodic PQ re-encapsulation in
  the live path, despite `perform_pq_ratchet_step`/
  `handle_incoming_pq_fields` existing, compiling, and passing their OWN
  isolated unit tests.
- Practical impact: the hybrid session gets ONE dose of post-quantum
  protection at bootstrap. It does not get the PQ-forward-secrecy property
  PQC-07 was meant to add (an attacker who later breaks ML-KEM-768 and also
  learns the bootstrap ciphertext can compute the same `pq_ss` that EVERY
  subsequent ratchet step in the session's lifetime still depends on).
  Classical (X25519) forward secrecy is unaffected — this is specifically
  a PQ-forward-secrecy gap, not a break of the whole session.
- Fix: wire `perform_pq_ratchet_step` into `encrypt_message_ratcheted` (call
  it on a cadence -- e.g. every DH ratchet step or every N messages -- and
  place the returned ciphertext/encaps-key into the envelope every time,
  not just pre-confirmation) and wire `handle_incoming_pq_fields` into
  `decrypt_message_ratcheted_v2` (extract `pq_kem_ciphertext`/
  `pq_encaps_key` from the envelope, call it BEFORE `session.decrypt`, and
  thread the result so `handle_dh_ratchet` sees the fresh secret, not the
  stale bootstrap one).
- Follow-up implementation task: `HANDOFF/todo/PQC_07_WIRE_RATCHET_STEP.md`.

## OTHER FINDINGS (MEDIUM/LOW, from Qwen Part A — spot-checked, mostly hold)

- MEDIUM (session_manager.rs `load()`, ~line 60-62, NOT line 120 as
  originally cited): the `json` String read from the sled backend and
  passed to `deserialize_sessions(&json)` is not zeroized after use, even
  though it contains hex-encoded session secrets. Minor hygiene gap.
- MEDIUM (session_manager.rs, `bootstrap_hct` persistence): serialized
  sessions retain `bootstrap_hct` (containing the ML-KEM ciphertext) even
  after handshake completion / `peer_confirmed`. Not independently
  verified against a specific line this session — re-check before acting.
- LOW (pq/mod.rs:65, encapsulation randomness zeroization): not
  independently re-verified this session.
- FALSE POSITIVE (originally cited as LOW, "identity_secret bytes buffer
  not zeroized before copy"): verified FALSE. `session_manager.rs:410-421`
  already calls `bytes.zeroize()` (line 418) and `arr.zeroize()` (line 420)
  correctly. No action needed.

## OTHER FINDINGS (from OpenRouter Part B — not yet independently verified
against current source; treat as PLAUSIBLE pending spot-check before acting)

- HIGH: `handle_incoming_pq_fields` lacks upfront ciphertext-length
  validation before attempting decapsulation in some call order, and
  decapsulation may not be constant-time (implicit rejection). Needs
  re-check: production code path currently under discussion (this
  function is itself never called live per the CRITICAL finding above, so
  severity is somewhat theoretical until the wiring fix lands — re-audit
  the wired version once PQC_07_WIRE_RATCHET_STEP is implemented).
- HIGH: `RatchetSession::reconstruct()`/serialization does not persist
  `skipped_keys` — after a process restart, previously-skipped
  out-of-order message keys are lost, which could cause legitimate
  messages to become undecryptable (availability issue, not
  confidentiality).
- HIGH: downgrade risk when `require_pq=false` (the default) — a
  first-message-suppression MITM could potentially force a v1-only
  session between two v2-capable peers. Needs verification against the
  actual `should_use_ratcheted_encryption`/negotiation code (PQC-08 already
  fixed one bug in this area today; re-verify this specific claim before
  treating as confirmed).
- MEDIUM: `get_message_key`'s skipped-key eviction uses `keys().min()`
  (lexicographic on the tuple key), not true LRU/insertion-order eviction.
- LOW: `init_as_receiver_hybrid` clones `our_mlkem_keypair` into the
  session rather than moving it, unnecessarily duplicating potentially
  sensitive key material in memory.

## MISSING TESTS (combined, both reviewers agree on the shape of this list)

- Downgrade attack: two suite-0x02-capable peers forced to suite 0x01.
- PQ shared secret actually changes derived root keys across a ratchet
  step (currently only tested at session-bootstrap time).
- Tampered/bit-flipped `pq_kem_ciphertext` is rejected, not silently
  accepted or causing wrong-but-successful decryption.
- Session restore after restart does not lose skipped keys.
- Both directions of a full `perform_pq_ratchet_step` /
  `handle_incoming_pq_fields` round trip END TO END through the wired
  encrypt/decrypt functions (not just the isolated ratchet.rs unit tests).
- `validate_pq_fields_present` is actually enforced somewhere in the live
  decrypt path (currently appears to be unused/orphaned, same as the two
  PQ ratchet functions above — verify during the wiring fix).

## Gating decision

PQC-09/10 (hybrid onion, ML-DSA identity) do not depend on the ratchet
wiring gap and MAY proceed in parallel. PQC-11 (relay invite hybrid auth)
and PQC-13 (verification suite) MUST wait for
`PQC_07_WIRE_RATCHET_STEP.md` to land and pass its own gate + a follow-up
review pass, since they build on/verify the ratchet's actual security
properties. PQC-08 (legacy retirement) is otherwise unaffected by this
finding (it gates classical fallback, not the PQ ratchet cadence) and its
own bugs (found and fixed this session) are separate from this gap.

---

## 2026-07-12 pass detail (qwen3-vl-235b-a22b-thinking, direct raw dispatch)

Two file groups reviewed: A (`pq/mod.rs`, `session_manager.rs`,
`negotiation.rs`) and B (`ratchet.rs`, `encrypt.rs`,
`integration_pq_session.rs`). Every finding below was independently checked
by the orchestrator against current source before being recorded — several
needed correction.

### CRITICAL (Group B): PQ shared secret never mixed into root key — CONFIRMED

Same bug as the 2026-07-12 update at the top of this file. See there for
detail; not repeated here.

### Group A's CRITICAL finding: FALSE POSITIVE — do not act on it

Reviewer flagged `core/src/crypto/pq/mod.rs`'s `decapsulate` (line ~95-111)
as CRITICAL for returning a *different* shared secret on a tampered
ciphertext instead of erroring, framing it as a chosen-ciphertext oracle.
This is backwards: ML-KEM (FIPS 203 / CRYSTALS-Kyber) uses "implicit
rejection" as its designed CCA2 countermeasure — `Decaps` never returns an
explicit failure for a malformed ciphertext; it always returns some 32-byte
value (real secret for a valid ciphertext, a value deterministically tied
to a secret rejection seed for an invalid one), specifically so an attacker
cannot build the oracle the finding describes. Confirmed by reading
`decapsulate`: it calls `mlkem768::decapsulate(...)` and returns its output
unconditionally, matching upstream libcrux-ml-kem's own API (no "invalid
ciphertext" error variant exists there, by design).
`test_tampered_ciphertext` asserting the two shared secrets differ (rather
than erroring) is testing the CORRECT, intended behavior. No follow-up
filed for this.

### HIGH (Group B): "Downgrade vulnerability in suite negotiation" — OVERSTATED

Reviewer claimed suite negotiation has no transcript binding at all.
Checked `core/src/crypto/encrypt.rs` (~line 294-301): there IS enforcement —
```rust
if !session.peer_confirmed {
    if let (Some(expected_hash), Some(envelope_hash)) = (&session.transcript_hash, &envelope.transcript_hash) {
        if expected_hash.as_slice() != envelope_hash.as_slice() {
            bail!("Transcript hash mismatch");
        }
    }
}
```
This rejects a mismatched transcript hash pre-confirmation, contradicting
"no binding at all." Narrower possible gap, NOT confirmed either way: the
check only fires when BOTH hashes are `Some` — a `None` on either side
(e.g. an envelope that omits the field) skips the check entirely rather
than failing closed. Whether an active attacker can force a `None` on a
suite-0x02-capable exchange (real bypass) vs. this only ever happening for
legitimately pre-negotiation-feature peers (benign) was not determined this
session. Needs a dedicated look before treating as confirmed.

### HIGH (Group B): "Error message leaks session state" — plausible, not verified

Cited `ratchet.rs:715`, claim that a decapsulation-failure error message
("...with either current or previous keypair") reveals key-rotation state
to an attacker. Not independently checked this session (exact line not
confirmed). Plausible minor information-leak / defense-in-depth item, low
urgency relative to the CRITICAL finding above.

### MEDIUM (Group B): Skipped keys not restored across session persistence — CONFIRMED, description corrected

Reviewer's framing ("`reconstruct` discards keys that were stored in
serialized state") is imprecise but the underlying gap is real. Checked:
`reconstruct`'s parameter list (`ratchet.rs:158-176`) has NO `skipped_keys`
parameter at all, and `session_manager.rs` never mentions `skipped_keys`
anywhere (zero grep hits) — so it's not that reconstruction "discards"
existing data; skipped keys were never added to the persisted session
format to begin with. Same net effect either way: any out-of-order message
key held in `skipped_keys` at the moment of persistence is permanently lost
on reload, so a legitimately-delayed message arriving after a session
restore may be wrongly rejected as behind the current chain position.
Follow-up filed: `HANDOFF/todo/PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED.md`.

### Tooling note

`scripts/delegate_task.py` hard-codes a code-implementation prompt
("provide full files" / "return unified diffs") regardless of what the
task file actually asks for. On this checkpoint it caused the model to
echo the input files back verbatim on the first attempt (`--mode full`
default). A standalone direct-call script
(scratchpad `qwen_raw_review.py`, not committed) with an explicit
"follow the task's own output format" system prompt was used instead for
both groups on the retry. Worth adding a real `--mode analyze` template to
`delegate_task.py` given how much of the remaining PQC backlog gates on
exactly this kind of read-only review dispatch.
