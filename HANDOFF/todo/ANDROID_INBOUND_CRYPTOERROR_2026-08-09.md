# Android drops SOME inbound messages with CryptoError -- 840 occurrences over 31 hours

Status: Active -- DISPOSITIONED 2026-08-24: measurement-pending, code causes absorbed
Disposition: both code-level causes named in the revised hypothesis are FIXED
on main ceabdbd4 -- self-ratchet reset guarded (see
P0_ANDROID_SELF_RATCHET_RESET disposition) and the receipt loop now calls
`mark_message_sent` (iron_core.rs:3533, see the P1 receipts ticket). Closure
condition per this ticket's own text ("confirm the count actually drops") is a
FIELD re-measurement, satisfiable only after the marker ticket closes and a
gate soak runs. Reclassify OBSOLETE if the count collapses; reopen at P0 if
CryptoErrors persist post-gate.
Severity: P1 (was filed P0 -- see the correction immediately below)

## CORRECTION 2026-08-09, same session, before anyone acted on this

This ticket originally said inbound messages "arrive and are discarded" and
implied Android could not decrypt. **A control test refutes the strong form.**

Sent a deliberate Windows -> Android probe and read the device log:

```
07:18:56.362346Z  inbox_receive  message_id=c62c59b5-4e99-44af-9e29-bbafd60824a7
                                 sender_id=985a25f9505372de   [Windows]
07:19:52.302275Z  inbox_receive  message_id=ec95877b-4afa-444d-8380-00d63269303d
                                 sender_id=3854e44295c13848   [macOS]
```

Both decrypted and reached the inbox, on the CURRENT build, with no upgrade.
So Windows -> Android AND macOS -> Android both work. The failure is a
SUBSET of frames, not the channel.

Second correction: the errors are **not per-pair**. `receive_message error`
by source in a recent window: 7x Windows, 7x AWS relay, 4x macOS. A ratchet
desync with one counterparty cannot produce that spread, so the
ratchet-desync hypothesis below is weakened, not confirmed.

**Revised leading hypothesis:** the retry storm is the upstream cause. The
receipt-marker defect (`RECEIPT_MARKER_ID_FLAVOR_MISMATCH_2026-08-09.md`)
makes the sender re-send messages the peer already processed, up to 12 times.
Those duplicates then fail to decrypt against an already-advanced ratchet,
which would produce exactly this: a high error count spread across every peer
we talk to, while fresh first-delivery messages succeed. If that holds, much
of this ticket's 840 is a SYMPTOM and fixing the marker matching removes it.

Severity lowered P0 -> P1 accordingly: it is not blocking delivery, it is
noise plus wasted traffic plus a real but narrower correctness question about
which frames legitimately fail.

Do NOT close this on the marker fix alone -- confirm the count actually drops.

### Controlled timing evidence for the retry->duplicate->CryptoError chain

Single tracked message, both sides observed:

```
07:18:55.xxx  Windows  outbox_enqueue        c62c59b5
07:18:56.362  Android  inbox_receive         c62c59b5   DECRYPTED, delivered
07:19:30.xxx  Windows  outbox_retry_attempt  #1/12      re-sends what Android HAS
07:19:51.978  Android  receive_message error from <Windows peer>: CryptoError
07:20:33.350  Android  Failed to decrypt ratchet message  (x4 burst to 07:20:34.413)
```

The sender re-sent a message the receiver had already decrypted 34 seconds
earlier, and the receiver logged a decrypt failure attributed to that same
peer 21 seconds after the retry.

**Strong but circumstantial, and it must be labelled as such:** the failure
line carries no message id, so it cannot be tied to `c62c59b5` by id. That is
not a logging oversight -- **a frame that fails to decrypt has no recoverable
id by construction.** This is itself an important finding: it is precisely why
cross-lane probes "vanish without trace", and it means any correlation here
can only ever be by timing plus peer attribution.

### Decisive test for whoever takes this

Fix the receipt-marker matching first, then re-measure the CryptoError rate on
an otherwise identical run. If the count collapses, this ticket is mostly a
symptom and closes with it. If it does not, there is an independent decrypt
defect and the ratchet path needs the adversarial pass on its own merits.

Cheaper interim test: send ONE message to an idle peer with nothing else in
flight and let exactly one retry fire. If a decrypt failure appears attributed
to that peer within the retry window and at no other time, the chain is
confirmed with a clean single-variable experiment.

## Original filing follows (severity claim superseded above)

Severity as filed: P0 (delivery truth -- inbound messages arrive and are discarded)
Discovered: 2026-08-09, Windows lane, read-only ADB verification of the Pixel 6a
Gate: crypto/ratchet path -- MANDATORY adversarial review (AGENTS.md rule 8)

## Summary

Frames from the Windows node reach the Android device and then **fail to
decrypt**. The transport is fine; the cryptographic layer rejects them. This
is a far better explanation for "messages do not arrive" than any of the
routing hypotheses the two lanes have been chasing, because the sender sees a
successful send and the receiver never surfaces a message.

## Evidence (device-side, read-only ADB, orchestrator-verified)

Counts from the app's own structured log
(`files/logs/scmessenger-mesh.log`, pulled via
`adb shell run-as com.scmessenger.android cat`, 44,011 lines):

| Count | Line |
|---|---|
| 840 | total lines containing `CryptoError` |
| 573 | `Failed to process received message: CryptoError` |
| 267 | `receive_message error from <Windows peer>: CryptoError` |
| 351 | `Failed to decrypt ratchet message: Decryption failed: invalid ciphertext/wrong key/tampering` |
| 489 | `Failed to decode wire envelope: io error: unexpected end of file` |

The 397/7/378 ledger figures and these counts were re-verified directly by the
orchestrator against the pulled artifacts, not taken on the subagent's word.

Time span: first `2026-08-08T23:40`, last `2026-08-10T06:32` -- **continuous
over ~31 hours**, not a transient burst tied to one build or one session.

Crucially, traffic from the same peer partly succeeds in the same window:
- 222x `Gossipsub message from <Windows peer> topic sc-mesh (345 bytes)`
- 94x `[OK] Message delivered successfully to <Windows peer>`

So the link is up and some paths work while direct message decryption fails.

## Why this matters to the five-node gate

The Mac lane's probe `7fa8367d-d1c9-4875-bee5-3fde1fcc4c47` was searched for
on-device across logcat, `files/pending_outbox.json`, the 44k-line mesh log,
and `strings` of `files/history.db/db` -- **zero matches, all four places**.
A message that fails to decrypt would never reach history, which is consistent
with (though not proof of) that probe having arrived and been discarded.

Pairs with `RECEIPT_MARKER_ID_FLAVOR_MISMATCH_2026-08-09.md`: the sender
cannot clear its outbox because the marker is unmatchable, and the receiver
cannot surface the message because it will not decrypt. Both directions of
delivery truth are broken independently.

## Hypotheses, ranked -- NONE verified yet

1. **Ratchet state desync.** 351 explicit ratchet decrypt failures. If the two
   ends disagree on ratchet state -- e.g. one side re-initialised a session
   the other still holds -- every subsequent message fails exactly like this.
   The retry storm makes this worse: the sender re-sends the same message up
   to 12 times (see the receipt-marker ticket) which can advance one side's
   chain.
2. **Identity-flavor mismatch in the crypto path.** The repo has now produced
   three identifier-flavor defects (blocks, contacts, receipt markers). If a
   message is encrypted to one flavor of the recipient's identifier and
   decryption keys off another, this is the symptom. Note the recently landed
   `sender_id` canonicalization work touched exactly this area.
3. **Envelope framing.** 489 `Failed to decode wire envelope: unexpected end of
   file` suggests some frames are truncated, which is a different defect and
   may account for a share of the CryptoErrors rather than key material.

## What must be done next (do NOT skip to a fix)

1. Correlate one specific message end to end: pick a Windows send with a known
   id and timestamp, find the corresponding Android `receive_message error`
   within the same second, and capture BOTH sides' view of the session and
   ratchet state at that moment.
2. Determine whether the failures cluster by session epoch (supports 1),
   by peer identifier form (supports 2), or by frame size (supports 3).
3. Only then propose a fix, and route it through the adversarial gate.

## Related device facts recorded at the same time

- Android identity SURVIVED the in-place upgrade: `firstInstallTime
  2026-08-08 12:47:45`, `lastUpdateTime 2026-08-09 16:00:26`, PeerId
  `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` present both before
  and after. This answers the macOS lane's question 2 for the Android leg.
- Android is NOT currently connected to the AWS relay. Last success
  `04:43:45Z -> 04:45:09Z` (84s). The next attempt failed with
  `Dial error: Unexpected peer ID <own-peerid> at /ip6/::1/tcp/9001/p2p/<relay>`
  -- the device is trying to reach the RELAY at its OWN loopback address.
  336 occurrences of that self-dial error. Address-hygiene defect, same family
  as `PROMISCUOUS_ACCEPT_UNROUTABLE_ADDR_2026-08-09.md`.
- Android ledger: 397 address rows but only **7 unique peer_ids**, and
  **378 of 397 rows have `success_count == 0`** (verified by parsing the
  pulled `files/ledger.json`). This is on-device confirmation of the render
  root cause in `ANDROID_LEDGER_VISIBILITY_ROOT_CAUSE_2026-08-09.md`.
- No app crash, ANR, or FATAL for pid 12219 in logcat; the process was stable
  throughout.
- Build stamp: **INSUFFICIENT**. No commit SHA anywhere in logcat or the mesh
  log; only `versionName=0.4.0`, `versionCode=14`, and the identify agent
  string `scmessenger/0.4.0/full/relay/<peerid>`. **The Android build carries
  no git provenance**, which means no lane can currently prove which SHA a
  phone is running. That is its own gap and should be fixed before the tag.
