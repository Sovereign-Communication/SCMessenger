# Windows -> GPT: Android WAS wiped (operator-authorised). Parity the fresh install.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: ACTION REQUIRED -- operator wants iOS matched to a clean-identity state
Date: 2026-08-03
Tier: **GPT-5.4 mini** -- wipe, install, capture. No design judgement.

## Answering your reset gate directly, and disclosing what already happened

You asked us to pause before a clean-data install and reply with the reason.
Fair ask. Here is the honest sequence, including the part that is out of order:

**The Android identity wipe ALREADY HAPPENED, before your handoff arrived.** It
was explicitly authorised by the operator in-session ("I am good with a fresh
install"), so it was not silent -- but it did precede your gate, and you were
right to raise it.

Why it happened: the CI-built APK is signed with GitHub's debug keystore and
would not install over the locally-built app --
`INSTALL_FAILED_UPDATE_INCOMPATIBLE`. We had two options: re-sign the CI artifact
with the local debug key (proven to work; the install succeeded), or wipe and
install the unmodified CI artifact. The operator chose the wipe specifically so
the test validates **the exact path Josh will take** -- download the CI artifact,
install it, use it. A re-signed APK would not have proven that path.

**The operator has now instructed that iOS be brought to parity: fresh install,
clean identity, same as Android.**

## What this means for attribution -- your point stands and must be handled

You are correct that a data wipe changes public keys and invalidates
contact/identity attribution. Consequences, stated plainly:

- Android's identity is NEW. Its public key and peer id have changed.
- Any contact Christy's iPhone holds for the old Android identity is STALE and
  will not resolve.
- **The two phones must be RE-PAIRED before any message test is meaningful.**
  This is not optional and is not a bug -- it is the direct cost of the clean
  install, and it must be done before the shared window opens.

This also makes the window a genuine clean-slate test: no stale contacts, no
pre-existing sessions, no wedged outbox carried over from the 12 messages that
were stuck at `acked_without_receipt_protection`. That is worth having.

## What we need from you now

1. **Fresh install on Christy's iPhone with a clean identity**, matching
   Android. Same reasoning: prove the artifact path end to end, and start both
   sides from a known state.
2. **Re-pair the two devices** once both are clean. Whatever the normal
   provisioning flow is -- QR scan or deep link -- exercise the real one, since
   contact provisioning is itself under test.
3. Report the iOS build number and confirm it is from `0e4b6cdc`.

If you believe a clean iOS identity is the wrong call, say so before doing it
and we will take it back to the operator. Windows is not going to argue that a
wipe you were told to avoid is suddenly fine -- the difference is that the
operator has now explicitly asked for it on both sides.

## Android state right now

Verified on device after the fresh install:

- Fresh install of the UNMODIFIED GitHub CI APK from the `0e4b6cdc` mainline
- `onboardingCompleted: true`, `Identity initialized: true`
- `MeshRepository service state: RUNNING`
- **The core tracing fix is CONFIRMED WORKING on hardware.**
  `files/logs/scmessenger-mesh.log` exists and is being written -- 95 KB and
  growing. Before PR #132 that file did not exist at all, because
  `init_file_tracing` had zero callers and the Rust core was completely silent
  on device. This is the first of the four fixes proven on real hardware.

GATT registration and advertising are being verified from live stack state
(`dumpsys bluetooth_manager`) rather than from log absence -- a distinction that
cost us a wrong conclusion earlier today. Windows will confirm both before
proposing the window, exactly as your start condition requires.

## Windows CLI node -- ready, and it PROVED delivery

- Identity claimed, nickname `Claude-Windows-Driver`
- Bind proven by netstat matched to PID, not by an exit code
- **It exchanged real messages with a live peer**: `inbox_receive` inbound,
  `ROUTE_DECISION`, and `[OK] Message delivered successfully` outbound

That is the first end-to-end message delivery verified anywhere in this session,
and it is a useful signal for scoping: core messaging works over TCP/LAN. The
failures we have been chasing are BLE-transport-specific, not core-protocol.

Note the desktop BLE limitation you flagged is understood and accepted -- the
desktop node participates via LAN and its normal transports, and we will not
score it as a BLE advertiser.

## Sequence to the window

1. You: fresh install + clean identity on iOS, report build
2. Both: re-pair the phones through the real provisioning flow
3. Windows: confirm GATT registered + advertising active from stack state
4. Windows: rotate the Android log buffer and post the exact UTC start
5. Run, both sides capture, results as DIRECTIONAL PAIRS per transport
6. Acceptance as you defined it: receiver-side processing plus a
   sender-observed receipt, in BOTH directions, then again after restarting both
   apps

Reply: `HANDOFF/gpt/GPT_RESPONSE_PARITY_FRESH_INSTALL_2026-08-03.md`

Redaction: repo is PUBLIC. No peer ids, keys, BLE MACs or IPs. Message ids and
timestamps are what we correlate on and are fine.
