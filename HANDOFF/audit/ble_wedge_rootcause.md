# BLE Wedge Root Cause Audit

## (a) Full lock map of `receive_message`

- `self.identity.read()`: acquired at `core/src/iron_core.rs:3007`, held until the function completes.
- `self.ratchet_sessions.write()`: acquired at `core/src/iron_core.rs:3041`, held until the function completes.
- `self.inbox.write()`: acquired at `core/src/iron_core.rs:3129`, held until the function completes.
- `self.audit_log.write()`: acquired at `core/src/iron_core.rs:3154`, held until the function completes.

No transitive helpers acquire additional locks during the `receive_message` critical section.

## (b) Top 3 root causes

1. **Non-reentrant lock in send path crypto function**
   (a) Interleaving: Send path acquires `ratchet_sessions.write()` (line 754) and calls `encrypt_with_ratchet_fallback`, which internally re-acquires `ratchet_sessions.write()` → send path deadlocks. Receive path blocks at `ratchet_sessions.write()` (line 3041).
   (b) Line: `core/src/crypto/encrypt.rs` (in `encrypt_with_ratchet_fallback`), first `ratchet_sessions.write()` call.
   (c) Confirm: Send thread logs `encrypt_with_ratchet_fallback` stack with `ratchet_sessions.write()`; receive thread blocks on same lock. Eliminate: If `encrypt_with_ratchet_fallback` logs completion and receive path still blocks.

2. **Identity writer blocked on ratchet_sessions**
   (a) Interleaving: Identity writer (line 621) acquires `identity.write()`, then blocks on `ratchet_sessions.write()` (held by send path). Receive path blocks at `identity.read()` (line 3007).
   (b) Line: `core/src/iron_core.rs` line 621 (identity write acquisition).
   (c) Confirm: Receive path blocks at line 3007 (identity.read()); identity writer thread blocked on ratchet_sessions. Eliminate: If receive path skips line 3007 and blocks later at 3041.

3. **Audit log write hold during blocking operation**
   (a) Interleaving: Send path holds `audit_log.write()` (line 755) for blocking I/O. Receive path blocks at `audit_log.write()` (line 3154).
   (b) Line: `core/src/iron_core.rs` line 755 (send) and 3154 (receive).
   (c) Confirm: Receive path logs at line 3154 and never proceeds. Eliminate: If send path releases `audit_log.write()` within 2 seconds.

## (c) Highest-value instrumentation change

Add `tracing::debug!("Acquiring ratchet_sessions.write() in receive_message");` at `core/src/iron_core.rs:3041` to log every receive path attempt. This will show: `[OK] Acquiring ratchet_sessions.write() in receive_message` followed by no further log if blocked, definitively localizing the hang.

---
11