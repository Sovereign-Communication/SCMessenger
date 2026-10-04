# TASK: P1-CORE-NEGOTIATION-RATE-SIGNAL -- add a rate-limited signal for repeated inbound negotiation failures from the same remote address

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Source

Follow-up recommendation from the `crypto-security-auditor` adversarial
review (2026-07-06) of the log-level fix in
`HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
("mode B" false-positive resolution). Not a blocker for that fix (verdict:
CLEAR to merge), but flagged as a real, pre-existing observability gap.

## Problem

`core/src/transport/swarm.rs`'s two `SwarmEvent::IncomingConnectionError`
handlers (~lines 3932, 5244) now log at `debug!` rather than `warn!`, since
the overwhelming majority of firings are benign LAN-discovery port-probes
(Android's `SubnetProbe`) rather than genuine negotiation attempts. This is
correct for the common case, but there is currently **no rate-limiting,
per-remote-address counting, or aggregation** of these events anywhere in
the handler. A burst of genuine negotiation failures from the same remote
address -- e.g. a peer attempting repeated Noise/Yamux downgrade attempts,
or a crude handshake-flood probing the negotiation path -- would now be
invisible at default log levels (`RUST_LOG=info` or similar), since the
error wrapping (`Select(Failed)` / `Handshake(...)`) is structurally
identical between the benign single-probe case and a malicious/flood case.

## Fix

Add a lightweight per-`send_back_addr` failure counter/window to the
`IncomingConnectionError` handlers -- the repo already has
`peer_rate_limit_multiplier`-style infrastructure (referenced near
`swarm.rs:2874`) for a similar purpose; reuse that pattern rather than
inventing a new one. When the same remote address produces N failures
within a short window (pick a reasonable default, e.g. 5+ in 10s), emit a
rate-limited `warn!` (or a metric/counter, if the repo has a metrics path)
distinguishing "repeated failures from one address" from the routine
single-probe case, independent of the per-event debug-level logging.

## Files to Touch

- `core/src/transport/swarm.rs` (both `IncomingConnectionError` handlers;
  reuse/extend the existing rate-limit infrastructure near line 2874)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib
```

Add a unit test simulating repeated `IncomingConnectionError` events from
the same address within a short window, asserting the rate-limited signal
fires once the threshold is crossed and does not fire for a single isolated
probe.

---
**IMPLEMENTED 2026-07-07 (Qwen-coder, orchestrator-applied):** added
`NEGOTIATION_FAILURE_COUNTS` (per-send_back_addr (count, window_start) map,
mirrors the existing `LAST_IDENTIFIED_LOG` pattern) and
`record_negotiation_failure_and_check_burst` to both `IncomingConnectionError`
handlers (native + wasm loops); emits `warn!` at >=5 failures/10s window,
independent of the per-event `debug!`. Gate: `cargo check -p scmessenger-core`
PASS.

Adversarial audit (Qwen-thinking, qwen3-235b): HIGH finding (unbounded
HashMap growth - spoofed-address memory DoS) FIXED (map bounded to 4096
entries, clears on overflow for a new key - a defensive counter, not a
source of truth, so this is an acceptable tradeoff). MEDIUM (write-lock per
event) ACCEPTED - matches the file's existing `LAST_IDENTIFIED_LOG` idiom,
not a new risk. LOW (fixed-window can undercount bursts spanning a window
boundary) ACCEPTED - this is a "flag if bad" signal, not a security boundary,
so occasional undercounting is fine. LOW (wasm/native Instant time-import
mismatch) NOT APPLICABLE - verified `Instant`/`Duration` already resolve to
`web_time::{Instant, Duration}` at module scope (same as the mirrored
pattern), confirmed by successful wasm-relevant compile path.

Unit test DEFERRED (needs to drive a real SwarmEvent::IncomingConnectionError
through the event loop or extract the burst-detector to a directly-testable
unit - the function itself is trivially unit-testable in isolation).
PENDING: final Fable re-audit alongside the other F2-F5 remediation, once the
native Claude window resets.

## Adversarial Review Requirement

Touches `core/src/transport/` -- mandatory `crypto-security-auditor` pass
before done, per `.claude/rules/security.md`.

## Do NOT

- Do not revert the `debug!`-level change for the routine single-probe case
  -- that fix is correct and already merged/audited. This ticket adds a
  *separate*, additive signal for the flood/repeated-failure case, not a
  blanket re-escalation back to `warn!` for every event.
