# Android reinstall MUST re-point the inbox bridge allow-list

Status: Active
Severity: P2 (silent loss of operator control channel, not a crash)
Filed: 2026-08-10
Trigger: any Android reinstall, fresh install, data clear, or identity reset

## What breaks

`scripts/inbox_bridge.py` turns inbound SCMessenger messages into
`HANDOFF/todo/INBOX_*.md` tickets so the operator can message work into the
orchestrator from the phone. It only acts on messages from ONE allow-listed
identity, held in:

    %APPDATA%\scmessenger\inbox_bridge.json  ->  allowed_peer_id

Current value (verified 2026-08-10 against the Windows node's own decrypted
history, then confirmed by a BLIND PIN handshake -- the PIN was sent only over
SCMessenger and never shown in the agent chat, so the operator reading it back
proves real Windows -> Android delivery rather than chat-reading):

    a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a

A reinstall that regenerates the Android identity produces a NEW identifier.
The old one keeps matching nothing.

## Why this is easy to miss

The failure is silent and looks exactly like "nothing happened":

- The phone sends fine. The Windows node receives and decrypts fine.
- The message lands in history and in the node's inbox.
- The bridge reads it, sees a non-matching sender, and deliberately writes
  NO ticket and sends NO ACK -- that silence is a security property (it stops
  the bridge being an oracle that confirms which node is listening), not a bug.
- So there is no error anywhere. The operator simply never gets a reply, and
  no work reaches the orchestrator.

The only visible signal is the `ignored_inbound_in_window` counter in
`%LOCALAPPDATA%\scmessenger\inbox_bridge.status.json` climbing while
`tickets_on_disk` stays flat.

## Fix after any reinstall

1. Make sure the node is running (`soak_supervisor.py status`).
2. Re-learn the identity, letting the phone identify itself:

   ```
   python scripts/inbox_bridge.py learn --write
   ```

   Then send any message from the phone. The next inbound sender is recorded
   as the new allow-list.

3. Confirm end to end: message the node from the phone and expect an
   `[ACK] <id> queued as INBOX_...` reply within ~10s.

## Do NOT take the identity from these sources

- `%LOCALAPPDATA%\scmessenger\storage\ledger.json` -- this ledger binds peer
  identities to addresses that are not theirs (see the P1 ledger ticket).
  Reading an identity from it can allow-list the wrong device.
- Guessing from history content. On 2026-08-10 the identity
  `3854e442...` sent messages containing the word "Android" ("Android artifact
  metadata for the install") but is in fact the **macOS** node. Content topic
  does not identify the sender.
- The contacts store. On 2026-08-10 every contact had a null nickname and none
  of the contact public keys matched any recent message sender.

Use `learn`, or the decrypted conversation itself, and confirm with a PIN
handshake before trusting it.

## Related

- `scripts/inbox_bridge.py` -- the bridge
- `scripts/soak_supervisor.py` -- runs the bridge alongside the node via
  `run --with-bridge`
- `scripts/README.md` -- "Inbound message -> orchestrator bridge"
