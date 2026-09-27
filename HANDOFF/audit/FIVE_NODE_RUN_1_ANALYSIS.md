# 5-node test, run 1: what happened and what to change

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: COMPLETE for the Windows/Android half. iOS/macOS half pending from GPT.
Date: 2026-08-03
Evidence: `tmp/bundle/` -- 34k lines across 6 sources from 2 nodes.

---

## Direct answers to the operator's questions

### "The drivers should have been getting messages -- I never got a response."

**The Windows node DID receive your message.** At `21:49:44` it logged an
`inbox_receive` with `"kind":"text"` -- a real user message, not protocol
chatter. It decrypted fine: **zero decrypt failures on the Windows node.**

**It never replied, and it never could.** Two reasons, both mine:

1. `grep -cE "auto.?repl|echo.?mode|responder" cli/src/main.rs` returns **0**.
   The CLI has no auto-responder. Nothing in it answers a received message.
2. I launched it with `< /dev/null`, so the interactive console has no stdin.
   Even a human could not have typed a reply into that process.

So "no response" is not a bug and not a delivery failure. It is a missing test
capability. A node that can receive but cannot reply can only ever prove ONE
direction, which makes it useless for the directional-pair method we adopted.

### "Maybe they tried to respond? Maybe they never got the message?"

Neither. It got the message, decrypted it, and had no mechanism to answer.
The 8 `Message delivered successfully` lines on the Windows node are protocol
traffic -- identity_sync and history_sync responses -- not replies to your text.

### What the Windows node received in total

| kind | count | what it is |
|---|---|---|
| `identity_sync` | 4 | protocol |
| `history_sync` | 3 | protocol |
| `text` | 1 | **your actual message** |

---

## The finding that matters most: a directional asymmetry

Android's decrypt failures are attributable to three peers, and one of them is
`12D3KooWD6vZQrUqp...` -- **the Windows CLI node's own peer id**.

So:

| Direction | Result |
|---|---|
| Android -> Windows CLI | **WORKS.** Received, decrypted, 0 failures. |
| Windows CLI -> Android | **FAILS.** `Failed to decrypt ... wrong key` |

Same pair of nodes. Same transport. Opposite outcomes by direction.

Per `DIRECTIONAL_PARITY_DIAGNOSTIC.md` this is the WORKS/FAIL row, which rules
out connection, discovery and transport as causes and points squarely at keys.
It also rules out "stale contact after the Android wipe" as a complete
explanation, because a stale contact would break BOTH directions between the
same pair, not one.

---

## Identity conflict: proven cryptographically, not asserted

Computed `blake3(pubkey)` for every 64-hex value in the logs and checked whether
the result also appears. Three peers are present under BOTH forms:

    pubkey 30d0fa67...05967e  ->  identity_id 985a25f9...f65826   (Windows CLI)
    pubkey b5990fb4...a9d481  ->  identity_id a774f988...d4e7a2
    pubkey 22729ea6...1109cd  ->  identity_id 63b8d0c3...f2e478

The first pair is the Windows node's own `identity` output: it prints
`ID: 985a25f9...` and `Public Key: 30d0fa67...` for one node. Two different
64-hex values, same peer, and nothing distinguishes them by format.

**And the message envelope carries the hash.** The received `text` message
contains `"identity_id":"<64-hex>"` in its sender block. So the wire format
propagates the form that CANNOT be used for encryption, because blake3 is
one-way.

That is the mechanism behind `wrong key`.

---

## What is confirmed WORKING after PR #132

| Layer | Evidence |
|---|---|
| BLE inbound | `mesh_ble_forward` 29 / `mesh_ble_forward_return` 29. Was 264/0. |
| GATT server | service `0000DF01` registered at handle 127 with the `2902` CCCD |
| BLE advertising | 3 advertising sets active |
| Core observability | `files/logs/scmessenger-mesh.log` live and growing |
| LAN/TCP transport | Windows node exchanged real messages with the phone |
| Core crypto | Windows node decrypted everything it received, 0 failures |

Transport is not the problem. It was, this morning. It is not now.

## What is still BROKEN

1. **Identity keying** -- one peer addressed under two indistinguishable forms;
   the envelope carries the hash; encryption needs the key. Blocking.
2. **Directional decrypt failure** -- Windows -> Android fails while
   Android -> Windows works, same pair.
3. **Node-fatal panic** -- `ConnectionClosed` handled per-peer instead of
   per-connection. Root-caused, fix known, not yet landed.
4. **Crash cannot self-recover** -- a stale lock survives the panic and refuses
   the restart with "SCMessenger is already running".
5. **CLI nodes cannot reply** -- no responder, so they can only ever demonstrate
   one direction.

---

## Lessons to bake into run 2

### Test capability gaps -- fix BEFORE running again

- **Give the CLI an echo/auto-reply mode.** Without it, a CLI node can never
  satisfy the receiver-side acceptance criterion in the reverse direction, so
  three of the five nodes cannot participate in a directional pair.
- **Or drive the interactive console** rather than redirecting stdin to
  /dev/null. Either works; doing neither is what made run 1 half-blind.
- **Land the panic fix first.** A node that dies during peer churn and then
  blocks its own restart will corrupt run 2's results, and it will look like a
  messaging failure rather than a node death.

### Method changes that worked and should be kept

- **Directional pairs immediately isolated the fault.** The WORKS/FAIL asymmetry
  between Android and the Windows node eliminated transport, discovery and
  stale-contact explanations in one step.
- **Receiver-side evidence only.** Sender-side "accepted" counters were
  misleading on both platforms; every conclusion here rests on what the RECEIVER
  logged.
- **Core-level logging is decisive.** Every important finding today came from
  `scmessenger-mesh.log`, which did not exist before this morning. iOS should
  get the equivalent -- ask GPT whether it has one.

### Method changes to make

- **Pre-reduce before delegating.** Two analysis lanes failed because 34k lines
  was too much for one dispatch. Extract deterministically with a script, then
  give a model the small focused digest. Polling loops likewise belong in bash,
  not in a model dispatch.
- **Timestamp discipline.** Android logcat is device-local, core tracing is UTC.
  Run 2 should record the offset explicitly at the start of the window.
- **Capture BEFORE and AFTER.** Run 1's Windows log was collected once, after a
  crash and restart, so the pre-crash inbound history is partly lost.

### Sequencing for run 2

1. Land the `num_established` panic fix and the lock-file recovery fix
2. Add CLI reply capability (echo mode is enough)
3. Resolve identity keying -- BLOCKED on GPT's decision, and nothing else here
   matters until it is settled
4. Re-pair both phones after any identity change
5. Confirm from live stack state, not logs: GATT registered, advertising active
6. Post a shared UTC window; every node captures the full window, not a snapshot
7. Record every pair directionally, receiver-side only
