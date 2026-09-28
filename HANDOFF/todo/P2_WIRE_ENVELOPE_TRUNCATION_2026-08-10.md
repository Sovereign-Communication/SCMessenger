# P2 -- Wire envelope decode fails with "unexpected end of file" (truncated frames)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Severity: P2 (silent inbound loss; mechanically distinct from CryptoError)
Filed: 2026-08-10
Gate mapping: G3 delivery truth, field-gate Section 6.3 (measurement integrity)
Anchor observed: `68fcc3f1` (installed APK, Pixel 6a)

## Field evidence

Window 2026-08-10T02:00Z -> 15:13Z, `files/logs/scmessenger-mesh.log`:

- **123** occurrences of
  `Failed to decode wire envelope: io error: unexpected end of file`
  on the current build (514 across the full two-day log).
- Emitted at `core/src/iron_core.rs:3285`:
  `tracing::warn!("Failed to decode wire envelope: {:?}", e);`

## Why this is filed separately from the CryptoError ticket

`ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` covers frames that decode but fail
to **decrypt**. This ticket covers frames that fail to **decode at all** --
the envelope is truncated before the crypto layer is reached. Different layer,
different cause, and it is not explained by the retry-storm/duplicate
hypothesis in that ticket.

Both are invisible to message-id correlation: a frame that fails to decode has
no recoverable id by construction. Correlation can only be by timing and peer
attribution.

## Required investigation

1. Identify the framing/length-prefix path feeding the decode at
   `core/src/iron_core.rs:3285`. Where is the frame boundary established, and
   is a partial read being passed to the decoder instead of being buffered
   until complete?
2. Determine whether these are genuinely short reads on a live stream
   (incomplete buffering) or genuinely malformed/truncated frames from a peer.
   These require opposite fixes.
3. Establish whether the count correlates with connection churn. The same
   window shows repeated `Peer discovered/disconnected via Swarm` cycles; a
   connection torn down mid-frame would produce exactly this signature.
4. Confirm whether a truncated frame currently causes any inbound message loss
   that the sender would see as delivered, or whether the sender's outstanding
   delivery obligation correctly survives.

## Acceptance criteria

1. Written analysis with exact `file:line` for the framing path and a stated
   verdict on incomplete-buffering vs malformed-input.
2. If incomplete buffering: a fix that accumulates until the declared length is
   satisfied, with a bounded maximum frame size to avoid a memory DoS.
3. If malformed input: the frame is rejected without terminating the
   connection, and the event is counted per-peer rather than logged per
   occurrence (log volume discipline, PF-11).
4. `cargo test --workspace --no-run` compiles.

## Note on log discipline

Whatever the fix, this warning must not be emitted once per occurrence at WARN
on a hot path. The field-gate reference Section 6.3 requires verbose transport
loops be bounded so logs remain usable for a whole matrix/soak window.
