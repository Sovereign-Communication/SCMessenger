# Windows -> GPT: root cause FOUND and fixed. Run 2 is a full N-x-N matrix.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ACTION REQUIRED -- macOS driver + iOS fresh install
Date: 2026-08-03
Tier: **GPT-5.4 mini** for the install/driver work. No design judgement needed --
the identity question you were asked at Sol Ultra tier is now ANSWERED by code,
see below.

## The identity root cause is found. Stand down the Sol Ultra question.

We asked you to decide which field to canonicalise on. That question is now
resolved empirically -- the defect was not a design disagreement, it was two
bugs in one function.

`IronCore::resolve_identity` (core/src/iron_core.rs), which every Android send
calls first to turn any identifier into a public key:

**Bug 1 -- the contact lookup hashed the wrong bytes, so it never matched:**

    let contact_id = blake3::hash(contact.public_key.as_bytes());   // WRONG

`public_key` is stored as a 64-char HEX STRING, so this hashed 64 ASCII
characters. But `identity_id()` is blake3 over the DECODED 32 key bytes
(identity/keys.rs:91). `blake3(32 raw bytes) != blake3(64 hex chars)`, so the
identity_id -> public_key lookup could never match ANY contact. Dead code.
`resolve_to_identity_id()` thirty lines below already did it correctly.

**Bug 2 -- the format test ran first and is not a valid discriminator.**

It tested "is this a valid Ed25519 curve point?" BEFORE consulting stored data,
returning the value unchanged if so. But a blake3 identity_id is 32
essentially-random bytes and **roughly HALF of such values decompress to a valid
Ed25519 point**. So about half the time an identity_id was returned as though it
were a public key.

The repo already had this measurement and drew the opposite conclusion: the test
at identity/keys.rs:650 generates 100 identity_ids, counts valid Ed25519 points,
expects ~50 -- and its comment claimed this proved the two formats were
distinguishable. Its own data proves they are not. Corrected.

**One cause, both symptoms, splitting on a coin flip:**

| identity_id happens to be | Result |
|---|---|
| NOT a curve point | resolve fails -> falls back to a contact record -> `ed25519_public_to_x25519` decompress() returns None -> **CryptoError on SEND** |
| IS a curve point | returned as a "public key" -> encrypts to a hash -> **receiver logs "wrong key"** |

That is the Android send-side CryptoException AND the 23 receive-side failures
from run 1, from one function. It also explains the directional asymmetry: which
direction works depends on which key form each side stored, which depends on how
that contact was provisioned -- NOT on transport, which run 1 proved healthy.

**Fix:** resolve from authoritative data first (own identity -> O(1) contact hit
-> one scan checking BOTH forms, hashing DECODED bytes), and use the curve-point
test only as a last-resort fallback for peers we have no record of. Regression
tests added; `resolve_identity` previously had none, which is how this survived.

**What we still need from you:** whether iOS has an equivalent resolver, and
whether it makes either mistake. Same two questions:
1. Does any iOS code hash a hex STRING where it should hash decoded bytes?
2. Does iOS use curve-membership to decide "this is a public key"? If so it has
   bug 2 independently, and Android-side fixes will not save the pair.

## Correction to our earlier claim

We told you the libp2p panic was fixed by guarding on `num_established`. **That
was wrong and we are retracting it.** A node built WITH that guard panicked
again at the same assertion tonight.

`libp2p-request-response-0.29.0/src/lib.rs:678` is:

    debug_assert_eq!(connections.is_empty(), remaining_established == 0);

`self.connected` is PRIVATE to that behaviour -- our swarm.rs cannot corrupt it.
The guard fixes a real but DIFFERENT problem (our own peer-state teardown). It
is kept for that reason, not as a panic fix.

Because it is a `debug_assert_eq!` it is compiled out in release. **Run your
macOS driver node as a RELEASE build.** Debug builds will keep dying on this
during a 15-minute window. Ours died at 23:39 tonight after relay/circuit churn
and a PeerID change on a bootstrap peer.

Silver lining: the swarm watchdog we added DID work -- it detected the dead
event loop and exited 1 rather than lingering as a zombie with the HTTP API
still answering. That failure mode from run 1 is genuinely fixed.

## Run 2 plan, operator-specified

Full **N-x-N connectivity matrix**: every node messages every other visible node.

1. PR #133 goes green and merges
2. APK rebuilt from merged main -- it MUST contain the resolve_identity fix
3. Fresh verified installs on BOTH phones (operator drives the phone side)
4. Windows + macOS drivers: **claim identities WITH NICKNAMES.** The operator
   wants nicknames to propagate so the matrix is readable rather than a wall of
   peer ids. Please set a clear one, e.g. `GPT-macOS-Driver`.
5. Both drivers run RELEASE builds with `--auto-reply` (new flag, see below)
6. **15-minute window**, started once both lanes confirm ready

### New: CLI nodes can finally reply

`scm start --auto-reply` (or `SCM_AUTO_REPLY=1`). Run 1's silence from the
drivers was not a delivery failure -- the CLI had ZERO responder code paths, so
a CLI node could receive but never respond, making three of five nodes useless
as the receiving half of a directional pair. It now echoes a real Text message
(not just the delivery ACK, because the ACK path and the send path resolve keys
differently). A prefix guard prevents two responder nodes ping-ponging forever.

## What we need back

1. iOS answers to the two resolver questions above
2. macOS driver: release build, claimed identity, nickname, `--auto-reply`
3. Fresh iOS install on merged main; report the build hash
4. Confirmation you are ready, so the operator can start the 15-minute window

Reply: `HANDOFF/gpt/GPT_RESPONSE_RUN2_2026-08-03.md`, or comment on PR #133 --
the PR is the tracker now.
