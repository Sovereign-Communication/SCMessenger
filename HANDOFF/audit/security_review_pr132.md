# Adversarial security review: PR #132

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: COMPLETE
Date: 2026-08-03
Reviewer: Windows/Claude, native.

Fusion Lite was attempted first and returned nothing usable: all three panel
models were THINKING models and each was truncated by the token cap before
producing output. $0.018 spent, no result. Two dispatch errors in one call -- a
VERDICT task was sent to a lane at all, and the panel was stacked with reasoning
models on a task that needs output rather than inference.

Scope: `core/src/iron_core.rs`, `core/src/mobile_bridge.rs`,
`android/.../MeshRepository.kt`, `android/.../TransportManager.kt`.

**CONFLICT OF INTEREST, stated up front:** I authored three of the four changes.
This review is therefore NOT independent, and the two substantive findings are
escalated to GPT as the technical escalation point.

---

## F1 [MEDIUM] Stale-Arc window introduced by clone-then-release

`mobile_bridge.rs` now clones the Arc and drops the guard before calling into
IronCore. The mutex is `Mutex<Option<Arc<IronCore>>>` and the Option IS mutated
elsewhere:

- `:346` -- `*self.core.lock() = Some(core.clone());`  (service start)
- `:447` -- `let core = self.core.lock().take();`      (service stop)

So a thread can hold a cloned Arc to an IronCore that stop has since taken. The
Arc keeps that instance alive, so this is NOT use-after-free -- Rust prevents
that. The real hazard is narrower and worth stating precisely: on a rapid
stop/start, a NEW IronCore is constructed at :346 while an in-flight cloned Arc
still references the OLD one, and both are backed by the SAME sled path. Two
live instances writing one store is a correctness hazard (duplicate processing,
interleaved writes), not a memory-safety one.

Severity MEDIUM rather than HIGH because the window is a single in-flight
receive_message call, stop/start is operator-initiated rather than
attacker-triggerable, and sled tolerates concurrent handles.

This is not a casually-introduced regression. The previous code held the guard
across the call, which is exactly what deadlocked all BLE inbound -- 264
forwards with 0 returns on device. The fix trades a guaranteed deadlock for a
narrow race, which is the right trade, but it should be closed properly.
Suggested follow-up: a generation/epoch counter so a stale handle is detected
and dropped rather than used.

## F2 [MEDIUM] Newly-installed file tracing persists peer identifiers to disk

init_file_tracing is now actually called, which is the entire point of the
change -- the core was previously silent on device. The consequence is that
tracing output is now WRITTEN AND RETAINED where before it went nowhere.

The touched files emit peer identifiers at INFO level:

- `mobile_bridge.rs:1371` -- "Peer discovered: {peer_id}"
- `mobile_bridge.rs:1420` -- "Message received from {peer_id}"
- `mobile_bridge.rs:1495`, `:3107` -- similar

The subscriber installs `EnvFilter::new("info")` by default
(`tracing_init.rs:51`) and applies no redaction layer.

So the log under the app files directory now accumulates peer ids. That path is
inside the app sandbox, which is correct, and message BODIES are not logged --
verified, the calls emit ids and types, not plaintext. But this repo is PUBLIC
and logs get pasted into issues, and a redaction inventory committed earlier
today itself leaked LAN addresses.

Recommendation before any user-facing build: either a redaction layer on the
file subscriber, or a documented rule that this log is never pasted un-scrubbed.
Not merge-blocking for the debug builds we are actively diagnosing with.

## F3 [LOW] log_dir is caller-supplied and unvalidated

init_file_tracing does create_dir_all on a path passed from Kotlin. On Android
the value is the app files directory plus "/logs" and is not
attacker-influenced, so there is no traversal path today. Flagged only so a
future caller does not pass something less trustworthy.

## F4 [INFO] Identity-hash rejection: no CPU-burn or timing vector found

Checked the three vectors posed:

- **CPU burn on cache miss.** The contact scan hashes each contact public key,
  but runs ONLY when the recipient is not found by public key, and the send path
  is operator-initiated rather than remotely triggerable. Bounded by contact
  count. No amplification path found.
- **Timing side channel** between fast path and scan. A caller could in
  principle distinguish "known contact" from "unknown" by latency, but the
  caller is the local app, which already knows its own contacts. No
  cross-trust-boundary leak.
- **Unknown recipients still allowed.** YES, deliberately, and this is the
  honest limitation. An arbitrary 32-byte value matching NO known contact hash
  is still accepted and encrypted to. The check closes the OBSERVED failure -- a
  contact stored under the wrong scheme -- but does not make the send path
  type-safe. Full canonicalisation is the actual fix and is blocked on the
  keying decision.

## F5 [INFO] BLE restart loop not reachable from this change

attemptBleRecovery is invoked from settings, from a failure callback, and now
re-runs initializeAndStartBle when components are null. That function
early-returns on disabled BLE and on missing permissions, and
BleGattServer.start early-returns when already running. No self-triggering loop
found: recovery does not raise the failure that calls recovery.

Concurrency is unguarded, though -- initializeAndStartBle is a suspend fun
launched on repoScope with no mutex, so two overlapping invocations are
possible. Effects are idempotent (null-checked construction, guarded starts), so
this is INFO rather than a defect, but a Mutex around it would be cheap
insurance.

---

## VERDICT: SAFE TO MERGE, with two tracked follow-ups

No CRITICAL or HIGH finding. Both MEDIUMs are consequences of fixing worse
problems -- a total BLE inbound deadlock, and a core that was invisible on
device -- and neither is remotely triggerable.

Follow-ups, tracked and not merge-blocking:

1. Close the stale-Arc window (F1) with a generation/epoch check.
2. Decide the logging redaction posture (F2) before any user-facing build.

**ESCALATION TO GPT:** I authored three of these four changes, so this review is
not independent. Please confirm or reject F1 and F2 specifically -- those are the
two where an independent reader is most valuable. F1 in particular is a
judgement call about whether a narrow stop/start race is an acceptable trade for
removing a guaranteed deadlock.
