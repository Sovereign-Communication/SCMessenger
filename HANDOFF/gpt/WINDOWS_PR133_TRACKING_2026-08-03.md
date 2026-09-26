# Windows -> GPT: PR #133 is now the single tracker for the 5-node effort

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: ACTION REQUIRED -- add the iOS/macOS half
Date: 2026-08-03
Tier: **GPT-5.4 mini** for the log collection. The identity question already sent
to Sol Ultra stays where it is; do not re-tier it here.

The operator asked for the 5-node findings and required fixes consolidated in one
place for tracking and accountability. That place is **PR #133**.

https://github.com/Sovereign-Communication/SCMessenger/pull/133

Everything below is already in it. Please add your half rather than opening a
parallel tracker.

## Three defects fixed in that PR

1. **`ConnectionClosed` handled per-PEER when libp2p emits it per-CONNECTION.**
   Node-fatal. `num_established` was never read anywhere in swarm.rs. The first
   connection close tore down all peer state while other connections to that peer
   were live, and libp2p-request-response panicked on bookkeeping we had removed
   underneath it. **This is in shared core, so it affects iOS too** -- worth
   checking whether you saw unexplained node deaths during the run.

2. **CLI nodes could receive but never respond.** Zero responder code paths
   existed. Now `--auto-reply` / `SCM_AUTO_REPLY=1`. This directly explains the
   operator's "I never got a response" -- see below.

3. **A dead swarm left a zombie process** with the HTTP API still answering while
   the mesh was gone, which then blocked its own restart.

## What we can now state definitively about the Windows node

- It **received** the operator's message: `inbox_receive`, `"kind":"text"`, at
  21:49:44. A real user message.
- It decrypted it with **zero** decrypt failures.
- It **could not** reply -- no responder existed and stdin was `/dev/null`.

So the silence was a missing test capability, not a delivery failure. If your
macOS CLI node was also silent, the same explanation almost certainly applies;
please confirm rather than assume.

## The finding we most need you to check against iOS

For the same pair and the same transport:

| Direction | Result |
|---|---|
| Android -> Windows CLI | WORKS. Decrypted, 0 failures. |
| Windows CLI -> Android | FAILS. `wrong key` |

A directional asymmetry rules out connection, discovery and transport. It also
rules out "stale contact after the Android wipe" as a complete explanation,
because a stale contact would break BOTH directions.

**Does iOS show the same asymmetry against any peer?** If iOS -> Android works
while Android -> iOS fails (or the reverse), that is the same defect and narrows
it further.

## Identity conflict -- now proven, not inferred

`blake3(pubkey)` over every 64-hex value in the logs found three peers present
under BOTH forms. The Windows node prints `ID: 985a25f9...` and
`Public Key: 30d0fa67...` for one identity, and blake3 maps the second onto the
first. Both are 32 bytes; both pass every length and hex check; the hash is
one-way.

**The message envelope carries the hash.** The wire format propagates the form
that cannot be used for encryption. That is the mechanism behind `wrong key`.

This raises the urgency on the Sol Ultra question already sent
(`WINDOWS_IDENTITY_UNIFICATION_MANDATE_2026-08-03.md`). Question 1 -- which field
iOS keys contacts on -- is still the single answer that unblocks Android-side
work. Everything else in run 2 is downstream of it.

## What to add to PR #133

- the iOS and macOS redacted analyses, per the bundle protocol already sent
- explicitly: does iOS have core-level logging equivalent to our
  `scmessenger-mesh.log`? If not, say so -- that is itself a parity gap, and it
  is the single thing that made today's diagnosis possible on Android
- whether your macOS CLI node received anything, and whether it could have
  replied

## Run-2 sequencing, for your agreement

1. Land PR #133 -- removes the panic, the zombie, and the responder gap
2. Resolve identity keying -- BLOCKING, yours to answer
3. Re-pair both phones
4. Confirm GATT + advertising from live stack state, not log absence
5. Shared UTC window; every node captures the FULL window, not a snapshot
6. Every pair recorded directionally, receiver-side evidence only

Reply: `HANDOFF/gpt/GPT_RESPONSE_PR133_2026-08-03.md`, or comment directly on the
PR -- the PR is preferred now that it is the tracker.
