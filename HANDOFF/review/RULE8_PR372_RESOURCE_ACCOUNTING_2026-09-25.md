<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).


## Per-site resource accounting for the eight drop sites

Added 2026-09-25 in response to round-2 MEDIUM finding 2: *"the patch must
demonstrate per-site acquire/release balance-neutrality at each of the eight
converted drop sites, since drop-instead-of-panic at event-combinator boundaries
can silently leak substream reservations/upgrade state that a panic's unwind
would have released."*

This is the requested argument, written so a reviewer can check it against the
code rather than take it on trust. It is an argument from ownership, NOT a
runtime measurement, and the last section says exactly what it cannot settle.

Reviewed head for every claim below: `ec5f0135`.

### The decisive fact

The vendored crate has exactly **one** `Drop` impl in production code:

```text
$ grep -rn "impl.*Drop for" vendor/libp2p-swarm-0.48.0/src/
vendor/libp2p-swarm-0.48.0/src/connection/pool.rs:867:impl Drop for NewConnection {
```

and its whole body is a notification to the pool:

```rust
fn drop(&mut self) {
    if let Some(connection) = self.connection.take() {
        let _ = self.drop_sender.take()
            .expect("`drop_sender` to always be `Some`")
            .send(connection);
    }
}
```

`NewConnection` is owned by the **connection pool**, not by any handler event.
None of the eight drop sites can hold, and none can skip, a `NewConnection`.
Pool accounting therefore cannot be affected by the drop path -- this was the
largest part of the "leak" hypothesis and it does not hold.

### What each site owns, and what dropping it releases

Line numbers are the `D9-DEGRADE` log statements, verified by grep against
`ec5f0135`.

| # | Site (file:line) | Value dropped | Resource released by the drop |
|---|---|---|---|
| 1 | `handler/either.rs:84` `FullyNegotiatedInbound::transpose` -> `None` | the whole `FullyNegotiatedInbound { protocol, info }`, consumed by `match self` | `protocol: IP::Output` is the inbound upgrade's **negotiated output** (`handler.rs:321-324`), i.e. the negotiated substream; `info` is the upgrade info. Both are owned and are dropped. |
| 2 | `handler/either.rs:113` `ListenUpgradeError::transpose` -> `None` | `ListenUpgradeError { error, info }`, consumed by `match self` | an error value and its info. No stream was ever acquired on this path (the upgrade failed), so nothing is retained. |
| 3 | `handler/either.rs:158` `on_behaviour_event` side mismatch | the `FromBehaviour` event, owned (`fn on_behaviour_event(&mut self, event: Self::FromBehaviour)`) | the event payload. Behaviour events are commands/values; a dropped oneshot sender surfaces as an error to its receiver rather than being lost. |
| 4 | `handler/either.rs:220` `FullyNegotiatedInbound` handler-side mismatch | the already-`None`d upgrade, or the `Some(fni)` that no arm claimed | site 1 already ran; in the `Some(_)` case the unclaimed `fni` is dropped, closing its substream. |
| 5 | `handler/either.rs:236` `FullyNegotiatedOutbound` handler-side mismatch | the unclaimed `FullyNegotiatedOutbound` | the outbound negotiated stream is closed on drop. |
| 6 | `handler/either.rs:248` `DialUpgradeError` handler-side mismatch | the unclaimed error | no stream acquired (the dial upgrade failed). |
| 7 | `handler/either.rs:262` `ListenUpgradeError` handler-side mismatch | `None`, or the unclaimed `lue` | as site 2 / site 4. |
| 8 | `behaviour/either.rs:157` `on_connection_handler_event` side mismatch | the `FromConnectionHandlerEvent` event, owned | as site 3, at the behaviour level rather than the handler level. |

At every site the matched value is **consumed by value** (`match self`,
`match (x.transpose(), self)`), so Rust's deterministic drop runs its destructor
at the end of the arm. No site returns early holding a borrow of the dropped
value, and no site stores the value anywhere that outlives the arm.

### What the panic released that the drop does not

A panic in these arms unwound the whole tokio task, which owned the connection
future. That dropped the `Connection`, and with it the `NewConnection`, firing
`drop_sender` and making the pool release the connection immediately.

So the honest statement of the difference is:

> The drop releases **one event and everything that event owns**. The panic
> additionally released **the entire connection**, because the task died. The
> patch's entire purpose is to keep that connection alive and let the pool
> continue owning it.

That is a deliberate change of behaviour, not a missed release. The retained
connection is still pool-owned, still closable by a `Close` event, and still
subject to the pool's own limits and idle timeouts.

### What this argument does NOT settle

1. **Retention amplification under a desync flood.** Because the connection now
   survives where it used to die, a peer able to induce repeated desyncs on one
   connection would keep it open until the pool's limits or idle timeout reap
   it, rather than having it released immediately by task death. The bound for
   this lives in the pool's limits, not in `either.rs`, and is **not verified
   here**.

   This one deserves weight rather than a footnote, because upstream states the
   opposite intent explicitly. `handler.rs:318-319`, immediately above the
   `FullyNegotiatedInbound` definition, reads:

   ```text
   /// [`ConnectionHandler`] implementation to stop a malicious remote node to open and keep alive
   /// an excessive amount of inbound substreams.
   ```

   That is the inbound-substream limiter's own doc comment. The D9 patch makes a
   connection survive events that upstream would have let kill the task, so the
   limiter now sees a connection that keeps existing. Whether the limiter still
   caps *inbound substreams* on that connection is upstream behaviour and is
   untouched by the patch; whether the connection's own lifetime is bounded
   tightly enough under a desync flood is the part **not verified here**.
2. **Whether a remote peer can induce a desync at all.** Still the open
   question in `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md`. This argument
   is conditional on the desync being locally caused, which is what the field
   evidence suggests but does not prove.
3. **Runtime balance under load.** Nothing here was measured. An instrumented
   run counting substreams/connections before and after a desync would close
   (1) and (2) together; that needs a live node and is not authorised.

### Evidence normalisation note

`evidence/d9-vendor-delta.patch` had trailing whitespace stripped from 23 diff
CONTEXT lines to satisfy the Repository Hygiene gate. This was verified
whitespace-only: the 249 `+`/`-` content lines and all 22 hunk headers are
byte-identical before and after, and the file length is unchanged at 457 lines.
Original SHA-256 `876c1c23cbe02e8e70dc85a5a10388610bc6186ea15e4a0c18f5f693309de9ed`,
normalised `918c61dcbd7a6dfbcd750ab9a83dbb259a5b20eba879a0d915c10743bfc8c9aa`.
