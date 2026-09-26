# Alpha Test Session Findings — 2026-07-19 (Lucas/Josh connectivity attempt)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Session ended without achieving a real end-to-end P2P connection.
Emulators torn down (both Lucas-local and Josh-remote QEMU processes stopped).
Relay and the Josh EC2 instance itself were left running (not terminated) so
the next session can resume without re-provisioning AWS infrastructure.

## Bottom line

No real peer-to-peer connection was ever established between any two nodes
this session, despite fixing several real, independent bugs along the way.
The relay's own connection log shows **zero** established connections in its
entire history. The remaining blocker is understood at the code level (see
"Core unresolved issue" below) but was not successfully fixed — a qwen
dispatch attempt produced an incomplete/non-compiling patch that was
correctly rejected by the tooling's safety checks rather than applied.

## What was fixed and verified this session (real, committed progress)

1. **Alpha-relay recovery** (commit `b0ac901f` and prior): the relay had been
   stuck for 16+ hours attempting a serial from-source Docker build on a
   913MB-RAM t3.micro that could never finish. Fixed by pulling the
   CI-published prebuilt image (`testbotz/scmessenger:latest`) instead of
   building on-box. **Lesson: never build the relay image on the t3.micro —
   always `docker pull` the CI-published image.**

2. **A-09 partial mitigation** (commit `36635cb0`): the relay was dialing
   non-routable addresses (loopback, link-local `fe80::`, site-local
   `fec0::`) that peers advertised via ledger-sharing. This stormed the
   libp2p `request_response` handler ("Dropping inbound stream because we
   are at capacity"), which then dropped legitimate peer connections. Fixed
   with a conservative `is_dialable_multiaddr` filter in `cli/src/ledger.rs`,
   applied at both ingestion and dial-selection points. Unit-tested,
   compiled clean, committed and pushed — CI rebuilt and republished the
   relay image, which was redeployed. This is a real fix but does NOT close
   the full `A-09` backlog ticket (that ticket's scope — connection_limits,
   relay-discovery authentication, dial dedup — is still open).

3. **AWS security group gap**: the Josh AWS emulator instance's security
   group only allowed inbound SSH (port 22) — no P2P port at all. This would
   have silently blocked any test that removes the relay to check direct
   peer-to-peer connectivity, regardless of anything else being fixed.
   Fixed live (opened tcp+udp/9001) and in
   `infra/ec2/launch-android-emulator-node.sh` for future launches.

4. **Lucas (local emulator) ANR loop**: root-caused to `-gpu
   swiftshader_indirect` (forced CPU software graphics rendering) being
   used on a WHPX-hardware-accelerated Windows machine — a flag meant for
   the cloud worker's no-GPU environment, copied into the wrong setup path.
   Starved the Compose UI's render thread badly enough to trigger repeated
   ANRs. Fixed by relaunching with `-gpu host`; confirmed clean via a
   dedicated debug pass (3+ minutes stable, zero recurrence). **Lesson:
   `ALPHA_TEST_LUCAS_JOSH_SETUP.md` must specify `-gpu host` for Lucas's
   local, hardware-accelerated emulator — `-gpu swiftshader_indirect` is
   only correct for a non-accelerated cloud worker.**

5. **U2 unification (partial)**: added `TOPIC_LOBBY`/`TOPIC_MESH` constants
   to `core/src/lib.rs`. Caught and corrected a real error in the ticket
   spec itself — it specified `"scm.lobby"`/`"scm.mesh"` (dot-separated),
   but the actual wire-format strings already live everywhere in
   `swarm.rs`/`bootstrap.rs` are `"sc-lobby"`/`"sc-mesh"` (hyphenated).
   Blindly implementing the ticket as written would have silently
   partitioned old and new nodes onto different topics.

## Core unresolved issue: dial reports success without a real connection

Precisely traced (code-verified, not speculative):

- Android's `MeshRepository.kt` `racingBootstrapWithFallback()` (~line 8528)
  calls `bridge.dial(addr)`, and treats any non-throwing return as success —
  logging `"Bootstrap connected"` and recording circuit-breaker success.
- That call chain goes through `core/src/mobile_bridge.rs`'s `dial()`
  (~line 3231) into `core/src/transport/swarm.rs`'s `SwarmCommand::Dial`
  handler (two near-duplicate copies, ~line 4492 and ~line 5010).
- The handler replies `Ok(())` to the caller as soon as `swarm.dial(addr)`
  returns `Ok(())` — which in libp2p means "dial successfully **queued**",
  not "connection established". It never waits for
  `SwarmEvent::ConnectionEstablished` before replying (line ~4557-4563).
- **Live evidence this is the actual gap, not just a reporting nit:** even
  with the relay fixed, healthy, and reachable (`nc` from inside the
  emulator to the relay succeeds instantly), and Lucas's app logging
  "Bootstrap connected" repeatedly, `ss -tn state all` on the relay for
  port 9001 showed **zero** established or even SYN_RECV connections,
  throughout the entire session, across multiple clean retests (including
  one specifically re-verified with WIFI confirmed CONNECTED+VALIDATED via
  `dumpsys connectivity`, ruling out network instability as the cause).

**Ruled out as causes** (each traced and confirmed not the blocker):
Kotlin `CircuitBreaker` (in-memory only, resets almost every cycle on WiFi —
its reset condition is in practice far less narrow than its own comment
suggests), Kotlin `dialThrottleState` (in-memory, 15s max backoff), the Rust
mobile-bridge `LedgerEntry`/`LedgerManager` (persists to disk but has no
timing fields at all — pure ranking/display data, doesn't gate dials),
self-inflicted WiFi flapping from an earlier diagnostic `svc wifi
disable/enable` toggle (confirmed settled/stable on retest, same result).

**What's needed**: a careful, correct implementation that tracks pending
dials and only reports success/failure based on the actual
`ConnectionEstablished` / `OutgoingConnectionError` swarm event (with a
timeout fallback), for both copies of the `SwarmCommand::Dial` handler in
`core/src/transport/swarm.rs`. A first qwen (thinking-tier, routed to
qwen3-max-preview) attempt produced an incomplete sketch — referenced an
undeclared `pending_dials` map, created channels never wired to the actual
reply mechanism, truncated mid-implementation. `delegate_task.py`'s
vacuous-success detection correctly refused to apply it; nothing was
broken. **This needs a dedicated, careful implementation session** (ideally
broken into smaller steps: declare the pending-dial tracking structure
first, wire the event-handler correlation second, update the reply logic
third — rather than one large one-shot dispatch), followed by the mandatory
`crypto-security-auditor` review this code path requires before merge.
Even after that reporting fix lands, the DEEPER question — why the real
libp2p connection never completes at all, given raw TCP plainly works —
still needs investigation; the reporting fix will at least make that
failure visible/honest instead of silently masked as "connected".

## Why Josh's boot took so long (efficiency lessons for next time)

- The AWS instance (`m7i-flex.large`, tag `Purpose=AndroidEmulatorTest`,
  instance `i-06271d27086498a49`) has **no nested virtualization** — `/dev/kvm`
  doesn't exist, 0 cores report `vmx`/`svm` flags. The emulator has been
  running in pure QEMU TCG software CPU emulation this entire session. This
  is an architectural limitation of this instance type, not a config bug —
  fixing it for real would mean a bare-metal (`.metal`) instance, which
  costs meaningfully more (the user explicitly chose to stay on the cheap
  instance and be patient this session, which is a reasonable call, but
  worth re-litigating if this keeps blocking testing).
- `-no-snapshot` was used on every launch this session, which **disables
  both loading AND saving a boot snapshot** — meaning every single restart
  (and there were many, across ANR debugging, RAM changes, etc.) paid the
  full cold-boot cost again. The AVD's own `config.ini`/`hardware-qemu.ini`
  already have `saveToLocalSnapshot = yes`. **Lesson: never pass
  `-no-snapshot` for iterative testing on a slow (no-KVM) instance — let it
  save a snapshot after the first successful boot, then all future starts
  resume near-instantly.** This session switched to snapshot-enabled boots
  partway through, but the instance never stayed up long enough afterward
  to actually complete a boot and save one.
- Bumping the AVD's `hw.ramSize` (1536M → 4096M) appears to have forced
  Android to redo first-boot-style optimization (a changed virtual hardware
  profile can trigger this), effectively costing as much time as a fresh
  cold boot. **Lesson: decide final resource allocation (RAM/cores) before
  the FIRST real boot, not after — changing it later is not free even
  without a full `-wipe-data`.**
- Net recommendation for next session: either (a) accept the cost and get a
  bare-metal/KVM-capable instance for real testing speed, or (b) if staying
  on the cheap instance, boot it ONCE with final settings decided up front,
  let it fully complete, save a snapshot, and treat every subsequent
  interaction as snapshot-resume only — never `-wipe-data`, never change
  hw.ramSize/ncore, never `-no-snapshot`, after that first boot.

## Recommended next steps (in order)

1. Implement the dial-establishment fix in `core/src/transport/swarm.rs`
   properly (see "Core unresolved issue" above) — this blocks everything
   else regardless of emulator speed, since it reproduces identically on
   the fast local Lucas emulator.
2. Verify the fix in isolation first: local Lucas emulator (now fast and
   GPU-accelerated, ANR-fixed) dialing the relay directly — do NOT wait on
   Josh's slow boot to validate this. Confirm via `ss -tn state established`
   on the relay that a real connection appears.
3. Once Lucas<->relay is confirmed working for real (not just "Bootstrap
   connected" in the log), resume Josh's boot with the snapshot-preserving
   discipline above, and re-run the full three-way test.
4. Crypto-security-auditor review on the `swarm.rs` dial-tracking change
   before considering it mergeable, per repo transport-code policy.
