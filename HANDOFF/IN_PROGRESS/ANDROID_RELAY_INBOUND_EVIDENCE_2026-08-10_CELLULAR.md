# Evidence drop: Windows -> AWS relay -> Android (cellular) delivered NOTHING

Status: Active -- evidence for the in-flight Android transport work
Filed: 2026-08-10 16:05 UTC by the Windows soak-node session
For: the session on `windows-lane/android-parity-dial-dedup` (worktree
`C:\Users\SCM\Documents\GitHub\scm-winlane`)

Controlled test run by the operator: phone dropped Wi-Fi, went **cellular
only**, Windows sent one message. Both sides captured. This is not a report of
a vague symptom -- the exact frame is traceable end to end.

**Device clock is `Pacific/Honolulu` (UTC-10).** Android logcat stamps are HST;
Windows stamps are UTC. Every pairing below is converted. Getting this wrong
makes the Android logs look 10 hours stale when they are live.

## The frame

    message_id  01bcd14f-3c8e-446d-96e1-ea9d9f7f60db
    from        Windows node (identity a43772fe.. addressed; resolved to
                public key c0a682ef.. = peer 12D3KooWNnPi..)
    to          Pixel 6a, cellular only, Wi-Fi off
    relay       12D3KooWPJK6.. (AWS 54.226.67.101)

## Windows side: the fallback chain worked

    15:54:31.814 UTC  ROUTE_DECISION attempt=1 route=direct
                      policy_reason=STORE_AND_CARRY relay_score=0.200
    15:54:31.827 UTC  [FAIL] Direct send outbound failure: Failed to dial
    15:54:32.397 UTC  ROUTE_DECISION attempt=2 route=relay
                      relay=12D3KooWPJK6.. policy_reason=RELAY_SUCCESS_SCORE
                      relay_score=50.000
    15:54:32.662 UTC  [OK] Message relayed successfully via 12D3KooWPJK6..
                      to 12D3KooWNnPi.. (264ms)
    15:55:00.374 UTC  [DIAL-BACKOFF] Peer marked as dead after 3 failed attempts

Direct dial failed, relay fallback engaged, relay accepted custody in 264 ms
and reported success. `/api/send/<id>` nevertheless stayed
`status=pending, delivered=false` indefinitely.

## Android side: it never arrived

Count of `inbox_receive` events in the app log for the whole day:

    0

The app was running throughout (`pid 29295`, `com.scmessenger.android`,
uptimeSecs climbing continuously across the window). It was not killed, not
Dozed, not crashed. It simply never received the frame.

## The 20-second window that explains it

    05:54:06.379 HST (15:54:06 UTC)  Bootstrap: no proven ledger relay
                                     candidates; network=CELLULAR, cellular=true
    05:54:32     HST (15:54:32 UTC)  <-- Windows relay reports SUCCESS here
    05:54:38.815 HST (15:54:38 UTC)  Bootstrap: no proven ledger relay
                                     candidates; network=CELLULAR, cellular=true
    05:54:52.319 HST (15:54:52 UTC)  Core notified identified:
                                     12D3KooWPJK6.. (73 addresses)
    05:54:52.394 HST                 Promoting peer 12D3KooWPJK6.. to full node
    05:54:54.764 HST                 StatsUpdated(peersDiscovered=1, ...)

`peersDiscovered` was **0** at the moment the relay accepted the frame, and
Android was reporting **no proven ledger relay candidates** on cellular. It
first identified and promoted the AWS relay at 05:54:52 -- **20 seconds after**
the relay had already reported success to Windows.

So the relay took custody for a destination that had no circuit at that
instant. Nothing on the Windows side can detect this: from Windows, the send
looks like a clean success.

**This is the transport-ACK trap in its purest form.** "Message relayed
successfully ... (264ms)" means the relay accepted the bytes. It does not mean
a circuit to the destination existed, and it does not mean anyone decrypted
anything. Do not score this path on that line.

Open question this evidence does NOT settle: whether the relay dropped the
frame outright when no circuit existed, or queued it and failed to flush on
circuit establishment 20 s later. That needs the AWS relay's own custody log
for 15:54:32Z -- `Relay custody audit log count: 300` is being emitted on
Windows, so custody records exist somewhere and should be read before assuming
a drop.

## Second defect: Android advertises malformed nested circuit addresses

From `refreshAddressesSnapshots` during the same window:

    /ip4/192.168.0.129/tcp/43567/ws/p2p/12D3KooWNnPi../p2p-circuit
        /p2p/12D3KooWPJK6../p2p-circuit/p2p/12D3KooWNnPi..

    /ip4/192.168.0.121/tcp/9090/ws/p2p/12D3KooWD6vZ../p2p-circuit
        /p2p/12D3KooWPJK6../p2p-circuit/p2p/12D3KooWNnPi..

    /ip6/::1/tcp/37505/ws/p2p/12D3KooWNnPi../p2p-circuit
        /p2p/12D3KooWD6vZ../p2p-circuit/p2p/12D3KooWNnPi..

Three problems, all in addresses the phone publishes to the mesh:

1. **Double-nested `p2p-circuit`.** Relay-over-relay is not a supported dial
   path. Any peer that tries these burns dial attempts on addresses that
   cannot resolve.
2. **Self-looping circuits.** The first and third both start at
   `12D3KooWNnPi..` (the phone) and terminate at `12D3KooWNnPi..` -- a circuit
   from the phone, through a relay, back to itself.
3. **Stale LAN addresses advertised while cellular.** `192.168.0.129` and
   `::1`-based circuit addresses are still being published after the Wi-Fi
   interface is down.

This is a plausible contributor to the `Failed to dial` on the Windows side and
to `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts` -- it is
directly adjacent to the dial-dedup work already in flight, which is why this
is being handed over rather than filed as a separate ticket.

## Third defect: Android outbox is wedged on transport-acked messages

Currently repeating on the device (16:04 UTC), across at least a dozen distinct
message IDs:

    delivery_state msg=<id> state=held
        detail=acked_without_receipt_protection acked_count=1 attempt=1
    Skipping retry for <id>: transport-acked message cannot be downgraded
        acked_count=1

Messages that got a transport ACK but no receipt are pinned in `held` and are
explicitly refused retry. If the ACK came from a relay that never delivered
(exactly the failure above), these can never drain -- the safety rule that
prevents downgrade also prevents recovery. Worth checking whether `held`
needs an escape hatch keyed on receipt timeout rather than ACK count.

## Also present, unrelated to delivery

    E/BluetoothLeAdvertiser(29295): Legacy advertiser should be only disabled
                                    on timeout, but was enabled!

Note for anyone testing BLE as an alternate path: the Windows node had
`enable_ble: false` for this entire test, so BLE was never a candidate
transport. It has since been set to `true` in
`%APPDATA%\scmessenger\config.json` (previous config backed up alongside).
Windows implements the BLE **peripheral** side (GATT server, service `DF01`,
identity `DF02`, message `DF03`, in `cli/src/ble_windows.rs`), so any BLE test
requires Android to act as central and connect inward.

## Reproducing

Windows node runs under `scripts/soak_supervisor.py` (pinned build
`0.4.0 e5284b7b`, sha256 `ba888428..`). Live logs:

    %LOCALAPPDATA%\scmessenger\logs\scm.log.<date>-<hour>
    %LOCALAPPDATA%\scmessenger\soak\artifacts\   (bundles on each failure)

Android, with two adb transports attached (use `-t <id>`, plain `adb` errors on
ambiguity and returns empty output that looks like "no logs"):

    adb devices -l
    adb -t <id> logcat -d -v time --pid=$(adb -t <id> shell pidof com.scmessenger.android)

## Related

- `HANDOFF/todo/ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` -- the
  `acked_without_receipt_protection` backlog here is likely the same retry
  storm that ticket identifies as the upstream cause.
- `HANDOFF/todo/ANDROID_REINSTALL_UPDATE_INBOX_BRIDGE_ALLOWLIST.md` -- if the
  Android identity is regenerated during this work, the operator's
  message-to-orchestrator bridge must be re-pointed or it silently stops.
