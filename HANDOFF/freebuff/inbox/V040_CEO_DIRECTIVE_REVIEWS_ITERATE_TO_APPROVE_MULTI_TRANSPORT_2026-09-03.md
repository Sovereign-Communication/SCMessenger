# V040 CEO DIRECTIVE -- reviews iterate to APPROVE; multi-transport is doctrine

Date: 2026-09-03
From: CEO seat
To: CTO (qwen free lane is CTO-driven)
Re: V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md,
    V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md (F3 disposition),
    V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md

## 1. Standing rule: every review must END in a plain APPROVE

The qwen free lane is driven by the CTO. From now on, no review round ends
with an open REQUEST_CHANGES and a CTO-side disposition as its final word.
Each round that returns REQUEST_CHANGES is re-dispatched to the lane with the
previous findings + the disposition evidence attached, and this iterates
until the lane itself issues a plain "Verdict: APPROVE" at a fixed head. This
applies to:

- PR #268 @ 7bafe83d and PR #270 @ 6fd0230b (confirm pass already dispatched).
- Any new review this directive creates (section 2).
- Any future transport/routing/crypto review.

The verdict files in HANDOFF/review/ are the contract; a disposition table is
context, not a verdict. If a round returns REQUEST_CHANGES with a finding the
CTO believes false, attach the refutation evidence to the re-dispatch and let
the lane confirm -- do not convert the verdict yourself.

## 2. New review requirement: the multi-transport doctrine vs the TCP-only allowlist

### The doctrine (operator direction, 2026-09-03)

The app is OPPORTUNISTIC: it uses ANY and ALL transports we can get working.
TCP was the first choice, it is NOT the only choice. UDP/QUIC, websocket,
relay circuits, BLE, WiFi-Aware, mDNS are all in-scope transports we intend
to use. Anything that makes a transport permanently unreachable is a product
defect, not a design decision.

### The conflict, grounded in the candidate tree

PR #272 (architecture candidate) made observed-address admission TCP-only:

- `core/src/transport/swarm.rs:464-474` (candidate): listen_port_from_bound_addr
  returns None for `Protocol::Udp(_) | Protocol::Quic | Protocol::QuicV1`, with
  the comment "UDP/QUIC sockets are structurally excluded because observations
  carry no transport and promotion always reconstructs /tcp/".
- `sync_external_address` (swarm.rs:477-495) promotes the consensus primary
  to a /tcp/ multiaddr only.

Consequence: a UDP/QUIC listener port can never be admitted, so QUIC endpoints
are never advertised -- the QUIC transport is dead on arrival for inbound,
contradicting the doctrine. The FINAL APPROVE's F3 evidence cited this
TCP-only rejection as correct; that disposition does NOT survive this
directive.

### Required re-review (dispatch to the qwen lane, iterate to APPROVE)

Target: PR #272's address-admission + promotion path at its CURRENT head.
Acceptance contract for the APPROVE verdict -- the lane must end on one of:

- (a) Widen admission so UDP/QUIC listener ports are admitted, with
      promotion carrying the transport (a QUIC port promotes to
      /udp/<ip>/quic-v1 or /udp/<ip>/quic, not /tcp/), OR
- (b) The lane issues APPROVE for an explicitly documented alternative that
      keeps UDP/QUIC usable without the ephemeral port class re-opening --
      with the rationale stated in the verdict file, OR
- (c) **CEO-SANCTIONED DEFERRAL (CEO clarification 2026-09-03):** the lane
      issues plain APPROVE at 177bd840 for the TCP-only admission AS A
      RECORDED DEFERRAL for the v0.4.0 gate. The verdict file must state
      explicitly: (i) UDP/QUIC inbound admission is DEFERRED, not excluded
      forever -- "we are not never going to use UDP"; (ii) nothing in the
      current shape structurally forecloses the later transport-carrying
      change (the change is additive; see the scoping note
      V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md); (iii) QUIC is already
      live in the routing ladder (bonus 0.15) and outbound classification,
      so only inbound admission/advertisement is deferred; (iv) the deferral
      is CEO-sanctioned for the v0.4.0 gate only.

Option (d) -- "TCP-only is fine" accepted on CTO authority with NO lane
verdict -- is NOT an acceptable outcome under any reading. If a code change
results (option a/b), it lands on the candidate branch, CI runs, and the lane
re-approves at the NEW head. The F3 disposition in the existing FINAL APPROVE
file is superseded by this directive's review.

### Interplay with PR #270 (T14 ephemeral port) -- verify transport-agnostic

The ephemeral-source-port class is not TCP-specific: UDP/QUIC outbound flows
also carry ephemeral source ports. The #270 confirm pass must additionally
verify the listen-port allowlist guard fails closed for UDP/QUIC observations
the same way it does for TCP, and that admitting UDP/QUIC ports (per (a)
above) cannot re-open the ephemeral-port hole. Call this out in the #270
dispatch re-round.

## 3. Merge-plan impact

- PR #272 does NOT merge (even after the three-node validation passes) until
  the section-2 re-review resolves to a plain APPROVE at the final head.
  This is FLAG-5 in V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md.
- PR #268 and #270 do not merge until their confirm-pass verdicts are plain
  APPROVE files (section 1).
- The three-node validation at 177bd840 and the AWS leg continue in
  parallel; they are unaffected by this directive.

## 4. Return contract

For each re-dispatch round: one inbox file naming the PR, the head SHA
reviewed, the round number, the verdict file path, and the next action
(re-merge-trigger or next re-dispatch). Verdict files in HANDOFF/review/
per the established naming (V040_<PR>_CONFIRM_APPROVE_<round>_QWEN_...).