# Quantum / PQC Review Tracking Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-10-04
Audited tree: `integrate/train-20261004` @ `200bfc54e`
Purpose: answer two questions -- (a) is every completed crypto review tracked,
(b) what is the consolidated gap list for the next release and for 3-node testing.

---

## 0. Scope correction: the target version is not 0.4.0

Commands run:

```
git tag -l "v0.4*" --format="%(refname:short) %(objectname:short)"
v0.4.0 21bbfe1d7
v0.4.0-rc.1 1f60b57f1
v0.4.1 dcd67b94e

git merge-base --is-ancestor v0.4.1 origin/main   -> exit 0 (YES)
git rev-list --count v0.4.1..origin/main          -> 20
```

`v0.4.0` is tagged and `v0.4.1` (`dcd67b94`) is already **on `main`**, with 20
commits landed since. Any plan, queue, or scope statement written against
"0.4.0" is stale. The open release target is the next tag after `v0.4.1`. This
document uses "0.4.x+" throughout for that reason.

**This matters for the mission:** nothing in the quantum/PQC set is a blocker
for 3-node testing, because nothing in the runtime path depends on it. See
section 5.

---

## 1. Provenance of the audit this document tracks

### 1.1 The referenced thread produced no artifact

Thread `44b8f169-1751-4602-b4cb-92218de5610a` ("Quantum Resistant Crypto Code
Audit") contains **2 messages total** (verified: `totalMessages: 2`).

- The user message asked for a strict read-only inventory with three
  independent slices.
- The single assistant message opened the slices, named the durable assignment
  `SCM-AUDIT-20261003-CRYPTO-INTAKE-01` and the role `CRITICAL_VALIDATOR`, then
  terminated on: `You've hit your usage limit.`

**Conclusion: the thread filed no findings, no verdict, and no ledger.** It is
a partial intake, not a completed review. Nothing may be reported as "the audit
thread said X."

### 1.2 The audit artifacts came from elsewhere

```
git log -1 --format="%h %an %ad %s" --date=iso -- HANDOFF/audit/QUANTUM_CRYPTO_AUDIT_2026-10-03.md
bede24a96 Claude (Cowork sandbox) 2026-10-03 13:28:31 -1000 docs(audit): quantum-resistance crypto audit and remediation task list

git log -1 --format="%h %an %ad %s" --date=iso -- HANDOFF/audit/QUANTUM_CRYPTO_REMEDIATION_TASKS.md
ad9612a68 Claude (Cowork sandbox) 2026-10-03 13:48:26 -1000 docs(audit): correct T2 and add T11 after two blocked implementation attempts
```

A separate sandbox session wrote the audit (revision 2) and the remediation
task list. Those documents, not thread `44b8f169`, are the tracked work product.

The audit's own baseline was the **preservation snapshot**
`preserve/shared-checkout-20261003` @ `e4050f84`, which it measured as 229
commits behind `origin/main`. Its remediation task list re-derives the key
traces against `origin/main` @ `19229287`. Both baselines predate the current
train head, which is why section 3 re-verifies rather than cites.

---

## 2. Reviews that ARE complete and filed

| Artifact | Verdict | Filed |
|---|---|---|
| `HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md` | wave-2 audit gate, 16 KB | 2026-07 |
| `HANDOFF/review/PQC_07_ATTEMPT2_FUSION_LITE_VERDICT.json` | ISSUES FOUND (desync = fatal; chain-index non-standard) | 2026-07 |
| `HANDOFF/review/PQC_07_ATTEMPT3_REVIEW_VERDICT.md` | **VERDICT: BLOCK** (root-key divergence; would not compile) | 2026-07 |
| `HANDOFF/audit/QUANTUM_CRYPTO_AUDIT_2026-10-03.md` | F1-F9, revision 2 | 2026-10-03 |
| `HANDOFF/audit/QUANTUM_CRYPTO_REMEDIATION_TASKS.md` | T0-T11 with acceptance criteria | 2026-10-03 |
| `HANDOFF/review/RULE8_OPUS_VERDICT_2026-10-04.md` | **BLOCK** (independent, non-author) | 2026-10-04 |
| `HANDOFF/review/RULE8_TRAIN_451_REVIEW_GAP_2026-10-04.md` | void-verdict proof, F1-F4 + N1-N9 | 2026-10-04 |
| `HANDOFF/done/` PQC tickets | 27 closed tickets | through 2026-07-23 |

The PQC build-out itself (PQC-01 through PQC-14) is genuinely closed as
*tickets*: 27 in `done/`, and `_QUEUE.md` lines 423-467 record each completion
date.

---

## 3. Independent re-verification of F2 (the load-bearing finding)

The audit's F2 (CRITICAL) is "the hybrid ML-KEM-768 / ML-DSA-65 path is
unreachable from the production send path." I did not take the document's word
for it. Commands and results on `200bfc54e`:

```
grep -rn "save_contact_bundle" core/src cli/src --include=*.rs
core/src/store/contacts.rs:770:    pub fn save_contact_bundle(
core/src/store/contacts.rs:1594:        mgr.save_contact_bundle("some-pubkey", &bundle).unwrap();
core/src/store/contacts.rs:1617:    fn test_save_contact_bundle_rejects_tampered_bundle() {
core/src/store/contacts.rs:1628:            .save_contact_bundle("tampered-pubkey", &tampered)
```

One definition, two test call sites, **zero production callers.**

The consuming branch, `core/src/crypto/encrypt.rs:549-561`:

```rust
if let (Some(our_b), Some(their_b)) = (our_bundle, recipient_bundle) {
    ... manager.get_or_create_session_hybrid(...)      // ML-KEM-768
} else {
    // Fallback to classical V1
    ... manager.get_or_create_session(...)            // classical suite-0x02
}
```

And the production argument, `core/src/iron_core.rs:1048-1052` + `:1067`:

```rust
let recipient_bundle = self.contact_manager.read()
    .get_contact_bundle(recipient_id).ok().flatten();   // always None
...
require_pq = false,
```

Nothing in production ever writes `contact_bundle_key`, so `get_contact_bundle`
always returns `None`, so the `else` arm is taken unconditionally.
**ML-KEM-768 encapsulation is never invoked on the send path.** F2 is CONFIRMED.

### 3.1 Two corrections to the audit's own wording

Both are accuracy fixes, not disagreements with the conclusion.

1. The audit states `verify_bundle` has "zero production callers." That is
   technically wrong. It **does** have one, at
   `core/src/store/contacts.rs:777` -- inside `save_contact_bundle`. It is
   unreachable for exactly the same reason, so the conclusion is unchanged, but
   the phrasing would mislead a future fixer into hunting for a second gap.
2. The audit describes the `else` arm as "classical V1." It is not the legacy
   static-ECDH path; it is the **classical suite-0x02 Double Ratchet**
   (`get_or_create_session`). Legacy static ECDH (`WireEnvelope::V1`) occurs only
   when `session_exists == false` **and** `require_pq == false`.

### 3.2 What a 3-node run will actually exercise

| Session state at send | Path taken | PQ contribution |
|---|---|---|
| session exists | classical suite-0x02 Double Ratchet | none |
| no session | `WireEnvelope::V1` static ECDH, no ratchet | none |

Neither path invokes ML-KEM or ML-DSA. A 3-node test will validate
confidentiality and delivery exactly as it would on a purely classical build.

---

## 4. Tracking gaps (five)

### G1 -- The P1 full-integration review has never been done

`HANDOFF/todo/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md`, status QUEUED, P1,
raised by operator request on 2026-08-28 ("after 0.5.0 look at PQC to ensure it's
implemented fully also"). Its definition of done requires a verdict at
`HANDOFF/review/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md`.

```
ls -la HANDOFF/review/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md
ls: cannot access ...: No such file or directory
git ls-files | grep -i PQC_FULL_INTEGRATION
HANDOFF/todo/PQC_FULL_INTEGRATION_REVIEW_2026-08-28.md
```

**The verdict file does not exist and has never been committed.** All six scope
bullets are unchecked. It has sat 37 days.

This is the most consequential gap, because it is the review that would have
caught F2, and it overlaps it directly (bullet 1, suite negotiation; bullet 4,
FFI reachability). Note the ordering irony: a 37-day-old P1 review ticket is
still open while `docs/QUANTUM_READINESS_AUDIT.md` has been reporting the
subject matter as CLOSED since 2026-07-29.

### G2 -- PQC-09 is marked done, but its deliverable is absent

`HANDOFF/IN_PROGRESS/PQC09_HYBRID_ONION_INVESTIGATION.md` carries all seven
acceptance criteria as `[DONE]` and specifies the output file
`HANDOFF/plans/PQC_09_HYBRID_ONION_DESIGN_NOTE.md`.

```
ls -la HANDOFF/plans/PQC_09_HYBRID_ONION_DESIGN_NOTE.md
ls: cannot access ...: No such file or directory
```

Meanwhile `HANDOFF/archive/PQC_09_SECURITY_REVIEW_FIXES.md` line 11 reads
`Status: TODO.` Three states in tension: acceptance criteria done, deliverable
missing, follow-up work says TODO. This is a Rule-16 shape -- the checkboxes
record intent, not artifact.

### G3 -- An empty unreviewed attempt slot

`HANDOFF/review/PQC_07_ATTEMPT4_DRAFT_UNREVIEWED.md` is **0 bytes**. It is a
placeholder with no content and no verdict. Harmless in itself, but it reads as
an open review slot in any directory listing.

### G4 -- The 12 remediation tasks have no dispatchable unit of work

T0-T11 exist only as prose sections inside
`HANDOFF/audit/QUANTUM_CRYPTO_REMEDIATION_TASKS.md`.

```
grep -rn -Ei "pqc|quantum|mlkem|ml-dsa" HANDOFF/freebuff/README.md
(no output)
```

`HANDOFF/freebuff/README.md` is the active 0.4.0 dispatch queue (per
`_QUEUE.md` line 14). It contains **no** quantum item. There are also no PQC
tickets in `HANDOFF/todo/`. So two CRITICAL findings (F2, F4), one HIGH (F3),
one HIGH (F1), five MEDIUM and one LOW have **no ticket, no owner, and no
queue position.** They are documented but not tracked, which is the exact
failure this audit was asked to catch.

### G5 -- A public doc carries a false PQ claim

`docs/QUANTUM_READINESS_AUDIT.md` (Status: Active, last verified 2026-07-03):

- Lines 142-147 mark F1, F2, F3, F5 as **CLOSED** on the strength of the PQC
  tickets landing. F2 is CLOSED by ticket completion and OPEN by reachability.
- Residual Exposure 1 asserts payload confidentiality "are protected by E2E dual
  signatures (Ed25519 + ML-DSA-65) and hybrid ratchet encryption." Section 3
  proves that is false in production.
- Its inventory states "No post-quantum primitives anywhere in the workspace
  (verified by grep for ML-KEM/Kyber/ML-DSA/...)". Also now false:

```
grep -n -Ei "mlkem|ml-dsa|libcrux" Cargo.toml core/Cargo.toml
Cargo.toml:61:libcrux-ml-kem = { version = "0.0.10", default-features = false, features = ["mlkem768"] }
core/Cargo.toml:29:libcrux-ml-kem = { workspace = true }
core/Cargo.toml:30:ml-dsa = { version = "0.1.1", features = ["getrandom"] }
```

The doc is stale, not malicious -- but it is the document a security reviewer,
an integrator, or a user would read to form an opinion about this product's
post-quantum posture, and it currently overstates it.

---

## 5. Consolidated gap list for 0.4.x+ and 3-node testing

### 5.1 Blocks 3-node testing: nothing

No quantum item is on the critical path. Section 3.2 is the reason: the runtime
does not depend on PQ, so a missing PQ path cannot fail a 3-node run. The
3-node triangulation can proceed on the current train the moment the merge train
settles.

### 5.2 Blocks any claim of post-quantum protection

| ID | Finding | Severity | Why it blocks the claim |
|---|---|---|---|
| T2 / F2 | hybrid path unreachable from send path | CRITICAL | no ML-KEM is ever invoked; see section 3 |
| T4 / F4 | identity secrets written to sled in the clear | CRITICAL | full identity recovery from a disk image |
| G5 | `docs/QUANTUM_READINESS_AUDIT.md` says CLOSED | -- | a false public claim compounds both of the above |

T4 is the more urgent of the two CRITICALs for an operator threat model, because
it is exploitable **today with no quantum computer at all** -- anyone with read
access to the data directory holds the identity. T2 is a forward-looking
(HNDL / CRQC) concern. If only one is funded first, fund T4.

### 5.3 Post-0.4.x security debt, not release-blocking

| ID | Finding | Severity | Note |
|---|---|---|---|
| T3 / F3 | backup restore silently degrades hybrid to classical | HIGH | latent until T2 lands; no hybrid session can exist yet |
| T1 / F1 | session root folds a transcript over two Ed25519 keys | HIGH | CRQC / HNDL; coupled to T2 (see below) |
| T5 / F5 | Ed25519 signing key reused as X25519 ECDH key | MEDIUM | `ed25519_to_x25519_secret` |
| T6 / F6 | `crypto` re-exports hashing from the LLM layer; insecure golden examples in prompt material | MEDIUM | inverted dependency + an agent-copyable unauthenticated cipher |
| T7 / F7 | Ed25519 signing duplicated in three non-crypto layers | MEDIUM | refactor only; must not change wire format |
| T11 | a v2 dual-signed invite is ~2.8x over the QR byte budget | MEDIUM | pre-existing; measured analytically, needs a real measurement |
| T8 / F8 | 16-byte truncated hash of an ML-KEM secret retained | LOW | memory-local |
| T9 / F9 | no real secrets found | LOW | verified negative; **no code change, do not "fix"** |

### 5.4 The coupling that constrains sequencing

T2 without T1 advertises post-quantum assurance that a CRQC invalidates: the
session root still folds a transcript hash over two Ed25519 public keys, so a
forger supplies their own Ed25519, X25519 and ML-KEM keys and decapsulates
themselves. The audit's own phrasing: *"fixing F2 without F1 would advertise
post-quantum protection that does not hold against a CRQC."*

The operator already recorded the T2 transport decision (bundle announcement
over the existing authenticated message channel, with key-change pinning
**deferred and accepted as risk**). That deferral makes T1 a hard blocker for
*closing* the pinning risk, not a parallel concern. Any report that presents T2
and T1 as separate completed efforts would be wrong.

### 5.5 T0 is closed by observation

T0 (the E0603 build break) was scoped by the audit to the snapshot branch only;
`main` declares `canonical_ledger_peer_id` `pub(crate)`. CI is green on `main`
(`c9876858a`, verified 2026-10-03), which is sufficient evidence the break is
not present on the shipping line. **No task needed.** Recorded so it is not
re-derived.

---

## 6. Recommended actions, in order

1. **Correct `docs/QUANTUM_READINESS_AUDIT.md` (G5).** Cheapest, highest-value
   action in this document. Retract the CLOSED marks for F1/F2/F3/F5, correct the
   "no PQ primitives" line, and restate residual exposure 1 to match section 3.
   Docs-only, no Rule-8 review needed, no merge risk.
2. **Open tickets for T4 and T2** (G4). Two CRITICALs currently have no queue
   position. T4 is the one with a present-day exploit.
3. **Perform the P1 full-integration review (G1)** or formally retire the ticket.
   Section 3 of this document already answers most of its bullets; the residual
   work is the transport/FFI/bullet coverage. Do not leave it queued at
   indefinite "after 0.5.0" while the subject matter is reported as closed.
4. **Fix PQC-09's tracking state (G2)** -- either produce the design note or
   strike the `[DONE]` marks. Delete or fill `PQC_07_ATTEMPT4_DRAFT_UNREVIEWED.md`.
5. **Raise T1 with the operator** (rule 9, architecture direction) at T2 kickoff,
   not at T2 completion, per the audit's sequencing note.

---

## 7. Method and limits of this audit

- Read-only with respect to source. Two files were **written** by this audit: this
  document and the report accompanying it. No source, ticket, or queue file was
  modified.
- Every code claim in section 3 was produced by a command run in this session
  against `integrate/train-20261004` @ `200bfc54e`. Section 2's verdict claims
  are read from the verdict documents themselves.
- **Not verified here:** the audit's F1, F3, F4, F5, F6, F7, F8, T11 mechanism
  details were not independently re-derived. They are carried forward as the
  audit states them. They are cited, not endorsed.
- **Not verified here:** `HANDOFF/plans/` was searched for the PQC-09 deliverable
  by exact filename only.
- Rule-8 standing: this audit authored no `core/src/{crypto,transport,routing,
  privacy}` change, so it is not a sign-off for any of them. The Rule-8 BLOCK
  verdict for the train head stands as recorded in
  `HANDOFF/review/RULE8_OPUS_VERDICT_2026-10-04.md`.