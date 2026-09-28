# WINDOWS -> GPT: bounded task -- iOS log capture for a live both-directions regression

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN, BOUNDED. Authorized by the operator as a small-scope GPT task.
**Model tier: GPT-5.4 mini (or equivalent small).** This is log retrieval,
sanitization and a push. It does NOT need Sol Ultra. Please delegate it down --
same discipline the Windows lane is applying.

## The situation

Messaging is now broken in BOTH directions between the Pixel 6a and Christy's
iPhone. Operator-reported, and the Android side is diagnosed:

- App process ALIVE (pid unchanged, never restarted) but `/proc/net/tcp` shows
  ZERO listeners bound -- 9001, 443, 80, 8080 all absent.
- The libp2p swarm died and did not recover.
- The installed Android build is `5925a6cc`, which PREDATES your `5719d67a`
  (Drop-based `SwarmTaskLivenessGuard` + `clear_handle_if_unhealthy`).

So on the Android side this is the SAME swarm death, persisting, not a new
regression. The Windows lane is capturing pre-fix evidence and then installing
PR-129 head `3ab85e40`.

## What is needed from the Mac lane (bounded)

1. Capture iOS logs from Christy's iPhone covering the same period the
   messaging failure was observed.
2. SANITIZE before committing anything -- this repo is PUBLIC. Redact:
   - libp2p peer ids (`12D3Koo...`) -> `<PEER_ID_A>` / `<PEER_ID_B>`, consistent
     per identity
   - 64-hex public keys -> `<PUBKEY_A>` etc
   - BLE MAC addresses -> `<BLE_MAC_A>`
   - LAN and public IPs -> `<LAN_IP>` / `<PUBLIC_IP>`
   Keep message IDs and timestamps -- they are what makes correlation possible.
3. Push the redacted export under `HANDOFF/logs/` and say so.

## The questions the iOS logs should answer

- Does iOS still have a live libp2p/swarm, or did its transport also stop?
- Any `ConnectionEstablished`, or only BLE writes?
- Does `setNotifyValue(true, ...)` succeed on DF02/DF03/DF04, and what happens
  immediately before each disconnect?
- Does iOS show outbound messages as sent / failed / awaiting receipt?
- Did iOS receive anything from Android in this window?

## Also still unanswered from earlier (one line settles it)

Does the outbound iOS envelope carry the sender's libp2p peer id and live
listener list? Android's receipt path resolves the route via
`contactManager.get(senderId)` -> `parseRoutingHints(contact.notes)`, and that
contact showed `notesLen=0` -- no `libp2p_peer_id`, no `listeners`. If iOS
sends them and Android drops them the fix is ours; if iOS does not send them it
is yours. This has been asked twice and remains the highest-value unknown.

## Notes

- Windows lane owns: Android build/install/logs, the merge, the 5-node matrix.
- We are NOT merging PR 129 until every required check on head `3ab85e40` is
  green. Current: 19 SUCCESS, 12 IN_PROGRESS, 0 FAILING.
- Your redaction instruction was correct and the Windows lane had not been
  applying it. Earlier handoff docs on `main` contain peer ids, public keys,
  BLE MACs and IPs. Operator has authorized: stop now, scrub going forward, no
  history rewrite (the product will move to a fresh repo later).
