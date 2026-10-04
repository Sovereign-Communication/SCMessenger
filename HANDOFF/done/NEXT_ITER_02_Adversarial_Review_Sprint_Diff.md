# NEXT_ITER_02: Adversarial Security Review of the Fable 5 Sprint Diff

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0  MANDATORY before the sprint changes are considered mergeable
(CLAUDE.md Adversarial Review Protocol: all changes under `core/src/transport/`
and `core/src/crypto/` require adversarial review)
**Recommended worker:** sonnet, high effort. Findings rated Critical/High escalate to Fable.
**Source:** The review was launched during the 2026-07-05/06 Fable session but the
subagent hit the session token limit before producing output. It must be rerun.

## Scope  review the sprint commit(s) touching:

1. `core/src/transport/swarm.rs`
   - New `SwarmEvent2::ListenerFailed` variant, propagated from
     `SwarmEvent::ListenerError` AND `SwarmEvent::ListenerClosed` in both the
     native and WASM event loops.
   - Reply channels (`mpsc::Sender<Result<(), String>>`) added to
     `SubscribeTopic`/`UnsubscribeTopic`/`PublishTopic`; `SwarmHandle` methods
     now await the reply.
   - Mechanical emoji-strip in log strings (verify no string/format corruption).
2. `core/src/mobile_bridge.rs`
   - `start_swarm` blocks on a `std::sync::mpsc::sync_channel(1)` until first
     `ListeningOn` / `ListenerFailed` / start error, 15s `recv_timeout`.
   - 14 FFI fns now `async fn`; new `*_blocking` internal helpers;
     `set_relay_budget_nonblocking` spawn variant used from `update_device_state`.
3. `core/src/crypto/session_manager.rs`
   - `hex::decode_to_slice(..).ok()` replaced with hard error propagation
     (zombie-session fix).
4. `cli/src/ble_mesh.rs`
   - BLE ingress backoff state machine (HashMap<String, PeripheralState>).

## Attack surfaces to probe (minimum)

- start_swarm handshake: missed/double signals, timeout-then-late-success state
  divergence (swarm thread keeps running after the FFI already returned Err 
  what does the Kotlin layer believe vs. what is true?).
- Reply channels: can an abandoned receiver (caller timed out) make
  `reply.send().await` park the swarm event loop? (bounded channel of 1 
  confirm sends can't block forever when the receiver is dropped; tokio mpsc
  send to a closed channel returns Err immediately  verify that's the case
  in every new arm).
- `*_blocking` helpers: enumerate every caller and prove none can execute on a
  tokio runtime worker thread (Handle::block_on panics there).
- parking_lot guards across `.await` (all new async fns).
- session_manager: does one corrupt session record now block loading ALL
  sessions (DoS surface), or just the corrupt one? Trace the caller.
- ble_mesh: `1u64 << failures` overflow at failures >= 64 (u32 counter,
  unbounded increment); `active: true` leak paths.
- New log lines: no key material / plaintext leakage.

## Output

List of findings with severity (Critical/High/Medium/Low/Info), file:line,
concrete failure scenario. Mark PRE-EXISTING findings as such. Verdict:
MERGEABLE / NOT MERGEABLE. Write the report to
`tmp/audit_reports/fable5_sprint_adversarial_review.md` AND summarize into this
file, then move this file to `HANDOFF/done/`.

## Escalate to Fable if

- Any Critical or High finding  Fable authored the diff and owns the fix.

---
**COMPLETE 2026-07-06 (orchestrator): review delivered. VERDICT: NOT MERGEABLE.**
Report on file: tmp/audit_reports/fable5_sprint_adversarial_review.md.
Findings: F1 HIGH (compile gate break - CONFIRMED independently by Windows `cargo test --workspace --no-run` exit 101 on scmessenger-mobile lib test); F2/F3/F4 MEDIUM (start_swarm handshake: timeout leaves live unowned swarm [privacy], retry returns Ok without listener verification [reintroduces P0], first-listener-of-any race); F5 MEDIUM + F6 LOW (BLE backoff has no success path + shift overflow at failures>=64); F7/F8/I1 LOW/INFO.
No Critical, no key-material leakage. crypto/session_manager change is a genuine improvement, MERGEABLE in isolation. Reply channels + *_blocking helpers + parking_lot-across-await all cleared with reasoning.
Remediation: F1 folds into NEXT_ITER_01 (compile gate); F2/F3/F4/F5 are AUDIT-GATE transport/privacy design fixes ESCALATED to operator/Fable; F6-F8 ride along or follow-up.
