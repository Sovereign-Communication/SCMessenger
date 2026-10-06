# Rule-8 independent verdict -- commit a34c60017 (session-drop fix)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-10-04
Reviewer: isolated, history-free Claude session (`claude -p`), no access to the
authoring conversation, read-only (`dontAsk`)
Model: `claude-haiku-4-5-20251001,claude-opus-5-5[1m]`
Cost: $0.4509 · 6 turns · 59,967 ms · session `8bfa3a21-8f68-4f2b-bd8a-4d3f576f6911`
Subject commit: `a34c60017` "fix(crypto): stop a replayed envelope from tearing
down a healthy session"
Base reviewed: `origin/main` = `051dbb5b9cd900669f11ecf5c40c63a38c2e86a6`

---

## SCOPE LIMITS -- READ BEFORE CITING THIS VERDICT

**This verdict is scoped to commit `a34c60017` alone.** Per
`docs/rules/SECURITY_PROTOCOL.md:105`, a verdict binds only the commit
reviewed. It does **not** clear PR #451 as a whole.

**Not covered by this verdict:**

- commits `20fa8e425`, `258aafa0b` (priority 2, ran out of budget)
- commits `1b2866812`, `0b9aa0c6c`, `1ba9c3e10` (test-only, skimmed)
- commit `d7d4bb3be` (`core/src/routing/global.rs`) -- only the call sites were
  located by search; the 8-byte hint's semantics and truncation-collision risk
  were **not** checked
- 15 of the 18 gated files under `core/src/{crypto,transport,routing,privacy}`:
  `crypto/mod.rs`, `crypto/ratchet.rs`, `privacy/mod.rs`, `routing/mod.rs`,
  `routing/engine.rs`, `routing/negative_cache.rs`, `routing/neighborhood.rs`,
  and all eight `transport/*` files
- no build, test, or lint run was performed (read-only session; the authoring
  host cannot compile Rust -- it dies with `rustc-LLVM ERROR: out of memory`)

Files actually read: `core/src/crypto/encrypt.rs` (partially: the `a34c60017`
diff plus lines 266-274, 334-340, 366-375, 620-660, 800-915) and
`core/src/crypto/session_manager.rs` lines 155-216.

---

## VERDICT (verbatim)

**Verdict: APPROVE**, limited to commit a34c60017. I ran out of budget for
Priority 2, so commits 20fa8e425 and 258aafa0b are not covered by this verdict.
The rest of the gated delta isn't either; the disclosure at the end lists what I
read.

Keeping the session is safe, and it is strictly better than the base code. The
new comment explaining why is only partly true, though, and the function still
lets a replayed envelope wipe out a healthy session by another route. That route
is not new in this commit.

### Answers to the four questions

**1. Is keeping the session a security weakness?** No.
- In the `Err(init_err)` arm (`core/src/crypto/encrypt.rs:854-892`), the
  existing session hasn't changed. The arm returns `Err(first_error)` (line
  893), so nothing is accepted.
- Keeping the session doesn't block re-negotiation. Any later envelope that
  carries bootstrap still goes through the rebuild at `encrypt.rs:815`.
- An attacker who can replay or inject envelopes can trigger this arm as often
  as they like. All they get is "keep the session the receiver already had".
  That isn't a new pin, and it isn't a denial of service.

**2. Is "bails BEFORE it writes" true?** Only for the hybrid suites (0x02 and
0x03).
- The map insert is at `core/src/crypto/session_manager.rs:209-215`:
  `match self.sessions.entry(peer_id.to_string()) { Occupied(mut e) => {
  e.insert(session); ... } Vacant(e) => Ok(e.insert(session)) }`. It does come
  after derivation.
- For suites 0x03 and 0x02, a missing hybrid ciphertext bails first, at lines
  185 and 200.
- For any other negotiated suite, the `else` branch (lines 202-206) builds a
  session with `init_as_receiver` and no bootstrap at all. That succeeds and
  overwrites the session even when the envelope has no bootstrap fields.
- So for a peer on a non-hybrid suite, the "bails before writing" argument does
  not hold. That case never reaches `Err(init_err)`, though, so the change does
  no harm there.

**3. Is it sound to keep the session in one arm and drop it in the other?** It
is coherent, but the reasoning is incomplete.
- The `Err(retry_err)` arm (`encrypt.rs:834-850`) is right that the old session
  is already gone by then.
- But the healthy session was destroyed by the unconditional overwrite in
  `create_receiver_session_hybrid`, before anyone checked that the rebuilt
  session can decrypt. Dropping it afterwards isn't the loss; the overwrite is.

**4. Does the sender stop attaching bootstrap?** Overstated.
- `peer_confirmed` is set only when a session successfully decrypts an incoming
  message (`encrypt.rs:272`, `encrypt.rs:339`).
- The sender checks its own session's flag (`encrypt.rs:372`, `391`, `401`). So
  the sender stops attaching bootstrap only after it has decrypted a reply from
  this peer. The receiver decrypting the sender's first message does not change
  the sender's flag.
- With one-way traffic, the sender keeps attaching bootstrap, so "every later
  message fails" overstates the impact. The bug is real once the peer has
  replied, which is the normal case for a conversation.

### Findings

**HIGH (not introduced by this commit; the fix doesn't close it) --
`session_manager.rs:202-215`, reached from `encrypt.rs:815-850`**
- A replayed envelope can still tear down a healthy session, through the rebuild
  path:
  - **Non-hybrid suite:** a replay without bootstrap rebuilds a receiver session
    from the static key, overwriting the live one. The retry then fails and
    `remove_session` runs (line 844).
  - **Hybrid suite:** a replayed first message that carries bootstrap rebuilds
    the session from that old ciphertext and then succeeds (line 826). The live
    ratchet is reset to its initial state and the old plaintext is returned
    again.
- The root cause is that the rebuild overwrites the map entry before proving the
  new session can decrypt.
- Fix to track: build the session without touching the map, try the decrypt, and
  insert only if it succeeds.
- This is the same threat the commit sets out to fix, so the commit message's
  claim to stop replay teardown is only partly true.

**MEDIUM -- `encrypt.rs:860-886` (the new comment)**
- It states as unconditional two things that are not: that the rebuild never
  writes, and that the sender stops attaching bootstrap after the receiver's
  first decrypt. Both are wrong in the cases described in answers 2 and 4.
  Future reviewers will rely on this comment.

**LOW (outside this delta) -- `encrypt.rs:636-638`**
- `copy_from_slice` panics if `ephemeral_public_key` is not 32 bytes or
  `pq_kem_ciphertext` is not 1088 bytes. I didn't trace whether lengths are
  checked earlier.

**Priority 2, partly checked only:** d7d4bb3be -- `is_route_pending_fresh(&self,
hint: &[u8; 8], now)` (`core/src/routing/global.rs:239`). The query sites I found
by search are `routing/engine.rs:203` and the tests at `global.rs:660-664`. The
type is fixed at compile time and CI is green, so they match. I did not check the
8-byte hint's semantics or whether truncating it causes collisions.

### What I read

Of the 18 gated files, I read only **`crypto/encrypt.rs`**, and only in part:
the a34c60017 diff, and lines 266-274, 334-340, 366-375, 620-660 and 800-915.
- I searched **`routing/global.rs`** and **`routing/engine.rs`** for one
  function name; I did not review them.
- I also read `crypto/session_manager.rs` lines 155-216. That file isn't one of
  the 18 changed files.
- I did not review the other 15: `crypto/mod.rs`, `crypto/ratchet.rs` (one
  search only), `privacy/mod.rs`, `routing/mod.rs`,
  `routing/negative_cache.rs`, `routing/neighborhood.rs`, and all eight
  `transport/*` files.
- I did not review 20fa8e425, 258aafa0b, or the test-only commits 1b2866812,
  0b9aa0c6c and 1ba9c3e10. I did not run any build.

---

## Note on the cited path

The verdict cites `core/src/store/session_manager.rs`. That path does not exist
in this repository; the file is `core/src/crypto/session_manager.rs`. The line
range 202-215 and the substance of the finding were verified against the real
file and hold as written.

## Disposition of the findings

- **HIGH** -- accepted and fixed. `build_receiver_session_hybrid` +
  `RatchetSessionManager::insert_session` now let the V2 recovery path build a
  candidate, attempt the decrypt, and commit only on success. Covered by
  `test_rebuild_that_cannot_decrypt_keeps_existing_session`.
  *Residual, not closed by this fix:* when a replayed bootstrap-carrying
  envelope rebuilds **successfully**, the rebuilt session is still committed and
  the live ratchet is reset. That is inherent to legitimate divergence recovery
  and was explicitly outside the prescribed fix; it remains open for triage.
- **MEDIUM** -- accepted and fixed. The comment now states the hybrid-suite
  condition and the `peer_confirmed` direction correctly.
- **LOW** -- not addressed. `copy_from_slice` length validation in
  `build_receiver_v2_session` is untriaged.
- **Priority 2** -- still unreviewed; blocks a full Rule-8 clearance of #451.