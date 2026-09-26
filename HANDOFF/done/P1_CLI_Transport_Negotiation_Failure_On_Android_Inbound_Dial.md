# TASK: P1-CLI-TRANSPORT  Windows CLI fails to negotiate transport protocol on inbound dials from Android

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a live LAN discovery test (2026-07-04): a Windows CLI daemon
(`scmessenger-cli.exe`, built from current `main`, release profile) was
listening on `192.168.0.121:9001` (raw TCP) and `:9002/ws` (WebSocket). A
physical Pixel 6a (`192.168.0.148`) on the same private WiFi correctly
discovered the Windows node via mDNS (`libp2p_mdns` on the Windows side
logged `mDNS discovered peer: 12D3KooWJJmBsLVA1rPsuPY6xWMSTRD427bnWUa7GxR496S8PxuU
at /ip4/192.168.0.148/tcp/9001/...`  confirming discovery works both ways
in this instance) and then actively dialed back to the Windows CLI. Both
inbound connection attempts failed at the transport-negotiation stage:

```
WARN scmessenger_core::transport::swarm: Incoming connection error from /ip4/192.168.0.148/tcp/50746 -> /ip4/192.168.0.121/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
WARN scmessenger_core::transport::swarm: Incoming connection error from /ip4/192.168.0.148/tcp/53196/ws -> /ip4/192.168.0.121/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
```

Both the raw-TCP and WebSocket listeners failed identically, on the same
peer, within milliseconds of each other  this rules out a WS-specific or
TCP-specific bug and points at something shared across both transports'
negotiation stack (most likely candidates: Noise handshake version/config
mismatch, multistream-select protocol-id mismatch, or a libp2p dependency
version skew between the Android core build (`libscmessenger_core.so`,
bundled in the APK) and this Windows CLI build, if the two were built from
different points in the dependency tree  note that a related crate,
`desktop_bridge`, was independently found to be **failing to build entirely**
in this same session due to missing `zbus`/`web_time` dependencies, which is
separate but raises the general question of whether all workspace members
are being built against a consistent, verified dependency set).

This is a genuine functional blocker: even with the mDNS
self-loopback/peer-count issues on the Android side fixed (see companion
tickets), a real remote peer cannot currently establish a working connection
to this CLI build over LAN at all, based on this evidence.

## Progress (2026-07-04, native /scm session, session ended on API-limit)

Confirmed via live tandem debugging (real CLI daemon on this Windows box +
real Pixel 6a at 192.168.0.148, adb readonly): failure is **100%
reproducible, not intermittent**  12 occurrences of "Failed to negotiate
transport protocol(s)" in the single hour-log `scm.log.2026-07-04-22` alone,
recurring roughly every ~3 minutes for both TCP/9001 and WS/9002, every
single time, since at least 21:12 that day.

Artifacts saved locally (gitignored `tmp/`, not committed, still on this
machine for the next session):
- `tmp/work_files/parity_debug_2026-07-04/logcat_full_dump_initial.txt`  full historical logcat dump
- `tmp/work_files/parity_debug_2026-07-04/logcat_live_capture2.txt`  live logcat during a trace-logging CLI run
- `tmp/work_files/parity_debug_2026-07-04/cli_trace_log2.txt`  CLI daemon run with `RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,libp2p_core=trace,libp2p_tcp=trace,libp2p_mdns=trace,libp2p_websocket=trace,multistream_select=trace,scmessenger_core=debug`  captured a fresh mDNS discovery event but the session ended (API limit) before this run's window caught a fresh negotiation failure with full trace detail.
- Real daemon logs (not gitignored artifact, persistent app data):
  `C:\Users\SCM\AppData\Local\scmessenger\logs\scm.log.2026-07-04-21` and
  `-22` already contain the 12 plain WARN-level occurrences referenced
  above (generic wrapper message only, no Noise/multistream-select detail
   that's what the trace rerun was for).

**Next session should:** start the CLI fresh with the same RUST_LOG line
above (`./target/release/scmessenger-cli.exe start`, adb device serial is
`192.168.0.148:43759`  note two adb entries currently resolve to the same
physical device, must pass `-s 192.168.0.148:43759` explicitly or `adb`
errors "more than one device/emulator"), let it run **at least 6-8 minutes
uninterrupted** (observed retry cadence ~3min, want 2+ occurrences), then
grep both the CLI trace log and the tandem logcat capture for the actual
underlying Noise/multistream-select error around the WARN "Incoming
connection error" timestamps  that specific underlying error is still not
captured yet. Cross-check `Cargo.lock`'s libp2p pins against what's in the
Android APK's bundled `.so` per Acceptance Criteria below (not yet done).

## Progress (2026-07-05, native /scm session, live tandem test with fresh matched builds)

Both sides rebuilt fresh from current HEAD (`6bf2479914b9deac967dfa1437ebd2bfee8b33fa`) specifically to
control for the version-skew hypothesis: Windows CLI release binary rebuilt
(`cargo build --release -p scmessenger-cli`), Android APK clean-installed via
`android/install-clean.sh` (fresh uninstall + `./gradlew clean :app:installDebug`).
CLI ran with full negotiation trace logging
(`RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,libp2p_core=trace,libp2p_tcp=trace,libp2p_mdns=trace,libp2p_websocket=trace,multistream_select=trace,scmessenger_core=debug`)
for ~7.5 minutes while the phone app ran. Full log:
`tmp/work_files/parity_debug_2026-07-05/cli_trace_log.txt` (433 lines,
19:40:56.923-19:48:28.769).

**This run never reached the negotiation-failure stage at all**  it got stuck one
stage earlier, at mDNS peer resolution, and the CLI's listeners never saw a
connection attempt of any kind (successful or failed) from the phone:

- Zero occurrences of "Incoming connection error" / "Failed to negotiate transport
  protocol(s)" anywhere in the 433-line log (previously 100% reproducible on
  2026-07-04 builds).
- Zero `ConnectionEstablished`. No `libp2p_swarm` activity at all after the initial
  listener-bind sequence at 19:40:56 (7 lines, all in the first second).
- Raw `libp2p_mdns::behaviour::iface`-level UDP packet chatter between the CLI
  (192.168.0.121) and the phone's OS-level mDNS responder (192.168.0.148:5353) is
  continuous and genuinely bidirectional for the full 7.5 minutes (62 queries, 137
  responses, 71 sent packets)  but **zero** Behaviour-level `Discovered`/`Expired`
  event, zero app-level `"mDNS discovered peer: ..."` line, and the phone's known
  peer ID (`12D3KooWJJmBsLVA1rPsuPY6xWMSTRD427bnWUa7GxR496S8PxuU`, from the
  2026-07-04 finding) never appears anywhere in this run's log.

**Root cause traced further on the Android side, and it looks foundational, not a
negotiation/protocol bug:** Android has exactly two LAN-discovery mechanisms, both
correctly present and wired in code, both enabled by default:
1. `MdnsServiceDiscovery.kt` (NsdManager-based, `_p2p._udp` service type) 
   constructed and started via `TransportManager.getOrCreateMdns().start()`.
2. `SubnetProbe.kt` (active TCP connect-scan of the local /24 + fallback subnets on
   ports 9001/9002, explicitly written as an mDNS workaround since "multicast DNS
   is link-local and does NOT cross routers... or some NAT'd virtual interfaces") 
   constructed and started via `TransportManager.getOrCreateSubnetProbe().start()`.

Both are started together from `TransportManager.startAll(enableMdns =
settings.internetEnabled)` (`TransportManager.kt:103-138`), called from
`MeshRepository.kt:2162` inside a `repoScope.launch { ... }` block, guarded by its
own try/catch that logs `"TransportManager startAll failed; ..."` on exception.
`settings.internetEnabled` defaults `true` (`MeshRepository.kt:4892`, `:5522`,
`SettingsViewModel.kt:199`), so on this fresh install `enableMdns` should have been
`true`.

**But across the entire 22,089-line logcat buffer since app install, there is ZERO
occurrence of:** `"SubnetProbe"` (any log line from that class, including its own
`start()` announcement), `"All transports started"` (the confirmation line
`TransportManager.startAll()` logs after starting both), or `"mDNS service
registered"` (confirms `MdnsServiceDiscovery` never even completed its own NSD
registration)  and also zero occurrence of the guarding
`"TransportManager startAll failed"` warning that would fire on a thrown exception.
`transportManager` is a `@Volatile private var transportManager: TransportManager? = null`
(`MeshRepository.kt:323`), assigned at `MeshRepository.kt:862`, and called via a
nullable safe-call (`transportManager?.startAll(...)`) at `:2162`  a null receiver
at call time would silently no-op with no exception and no log, matching the
observed silence exactly. (`MeshRepository.kt` was running and logging elsewhere
throughout  e.g. periodic `"Mesh Stats: 0 peers (Core), 0 full, 0 headless
(Repo)"`  so the process/service itself was alive; this specific init block's
mDNS/probe branch is what never fired or never ran.)

**This is likely the actual root cause of "no nearby peers" on same-LAN, and is a
different, earlier-stage bug than this ticket's original (2026-07-04) finding.**
Filed as a separate, more specific ticket:
`HANDOFF/todo/P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`  that
ticket owns root-causing whether this is a null-`transportManager`-at-call-time
race, an init-order bug, or this whole block simply isn't reached on this app's
current init path. **This ticket (P1-04) should be re-attempted only after that one
is resolved and LAN discovery is confirmed actually engaging**  otherwise every
retest will stall at mDNS/SubnetProbe silence before ever reaching the
negotiation-failure stage this ticket was written to root-cause.

Cargo.lock libp2p versions this session (for the record; not the suspected cause,
since this run controlled for version skew by rebuilding both sides from the same
commit): `libp2p 0.56.0`, `libp2p-core 0.43.2`, `libp2p-swarm 0.47.1`,
`libp2p-noise 0.46.1`, `libp2p-tcp 0.44.1`, `libp2p-websocket 0.45.1`,
`libp2p-mdns 0.48.0`, `libp2p-quic 0.13.1`.

## Progress (2026-07-05, lightweight retest after the LAN-discovery fix landed)

`P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`'s fix (`ef2869b1`)
was verified live: fresh incremental install, cold app relaunch, CLI daemon
restarted. Within ~8 seconds: `TransportManager: All transports started`,
`SubnetProbe starting`, `MdnsServiceDiscovery: mDNS service registered`, and
`SubnetProbe: open port 192.168.0.121:9001/9002` all fired on the Android
side. The CLI's own mDNS also completed real peer discovery this run (unlike
the 2026-07-05 session before the fix, which never got a Behaviour-level
Discovered event): `mDNS discovered peer: 12D3KooWE7Bi... at
/ip4/192.168.0.148/tcp/9001/p2p/...`. **LAN discovery is confirmed fixed.**

**But the negotiation failure this ticket was originally filed for is
CONFIRMED STILL REAL**, immediately after discovery succeeded:
```
WARN scmessenger_core::transport::swarm: Incoming connection error from
  /ip4/192.168.0.148/tcp/43862 -> /ip4/192.168.0.121/tcp/9001: Listen error:
  Failed to negotiate transport protocol(s)
WARN scmessenger_core::transport::swarm: Incoming connection error from
  /ip4/192.168.0.148/tcp/52018/ws -> /ip4/192.168.0.121/tcp/9002/ws: Listen
  error: Failed to negotiate transport protocol(s)
```
Both sides were running matched, fresh builds from current HEAD this session
(Android APK rebuilt incrementally on top of `ef2869b1`; CLI release binary
built earlier the same session from `6bf24799`), so **this rules out version
skew as the root cause**  it is a genuine protocol/config negotiation bug,
exactly as this ticket's acceptance criteria anticipated as the fallback
case. Next step (not done this session, budget-constrained): re-run the
trace-level capture (`RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,...`,
see the recipe earlier in this ticket)  this time discovery will not stall
the run, so the window should actually catch the underlying Noise/
multistream-select error this ticket needs.

## Progress (2026-07-05, trace-level capture  likely TWO bugs, not one)

Full negotiation trace (`RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,...`)
captured while live against the phone. The generic "Failed to negotiate
transport protocol(s)" wrapper turns out to fire for two genuinely different
underlying failures, cross-referenced against Android logcat (filtered by
the app's PID) from the same window:

**(A) Windows CLI dialing OUT to the phone (192.168.0.121 -> 192.168.0.148:9001)
fails with a plain TCP-level refusal, not a protocol mismatch:**
```
Outgoing connection error to <peer>: Failed to negotiate transport protocol(s):
[(/ip4/192.168.0.148/tcp/9001/...: No connection could be made because the
target machine actively refused it. (os error 10061))]
```
`os error 10061` = `WSAECONNREFUSED`  nothing accepted the TCP handshake on
the phone's side at all. This never reaches Noise/multistream-select.
**Corroborating evidence this is Android-side, not a Windows/network issue:**
logcat (filtered by the app's own PID) shows the phone's own libp2p failing
to dial **itself** at its own advertised address: `MdnsServiceDiscovery`
resolved `12D3KooWE7Bi...@192.168.0.148:9001` (a mDNS self-loopback  the
already-known, still-open `P1_ANDROID_mDNS_Self_Loopback_Discovery.md`), and
immediately after: `MeshRepository$ensureTransportManager: Failed to dial
discovered LAN peer /ip4/192.168.0.148/tcp/9001/p2p/...: Network error`. If
the phone's own libp2p can't reach its own supposedly-bound listener, the
listener likely isn't actually functioning at the OS level, regardless of
what dials it. Kotlin (`MeshRepository.kt:3051`) does explicitly request
`meshService?.startSwarm("/ip4/0.0.0.0/tcp/9001", listOf())`, so the
requested port is correct  the open question is why `listen_on()` isn't
producing a live, externally-reachable listener on Android (`listen_on()` in
libp2p returns `Ok` immediately and reports async bind failure later via a
listener-error swarm event, per `core/src/transport/swarm.rs:1904-1909` in
the single-port branch  that later event needs to be traced/logged to
confirm this hypothesis; not done this session).
**Separately confirmed:** Android's `SubnetProbe` (a bare `Socket().connect()`,
not libp2p) DOES succeed at reaching the Windows CLI's port 9001/9002  so
Windows's own listeners and the network path Android-to-Windows are fine;
this failure mode is specifically about the phone's inbound-listener side.

**(B) The phone dialing OUT to the CLI (the original 2026-07-04 finding, and
reproduced again this session on 09:42-11:58 runs) is a real TCP-accepted,
protocol-level failure**  the CLI's own "Incoming connection error" case,
which DOES get accepted at the TCP level (unlike (A)) before failing
negotiation. This is the genuine bug this ticket was filed for. Still not
root-caused to a specific Noise/multistream-select mismatch  the trace
capture that reproduced (A) did not happen to also catch a fresh instance of
(B) in the same window to extract its detail; needs a dedicated capture run
targeting an inbound (phone-initiated) dial specifically.

**Net: this ticket's scope should split.** (A) is very likely a new,
Android-listener-specific bug (candidate root cause: Android may need the
Rust core to observe and log the async listener-bind outcome, or there's an
Android-specific TCP/socket restriction preventing the bind from actually
being externally reachable)  recommend filing as its own ticket once
confirmed. (B) remains this ticket's original scope and still needs its
Noise/multistream-select detail captured.

## Progress (2026-07-05/06, native /scmorc session -- RESOLVED as false positive)

Re-ran the trace-level capture targeting an inbound (phone-initiated) dial
specifically, as (B)'s open item required. Caught two fresh occurrences:

- TCP/9001: `Incoming connection from remote` at `00:52:12.117656Z` ->
  `Incoming connection failed: ... Select(Failed)` at `00:52:12.322403Z`
  (+205ms). Zero `multistream_select` trace lines in between despite
  `multistream_select=trace` enabled.
- WS/9002: `Incoming connection from remote` at `00:52:14.145273Z` ->
  `libp2p_websocket::framed: receiving websocket handshake request` ->
  `Incoming connection failed: ... Handshake(Io(Kind(UnexpectedEof)))` at
  `00:52:14.348160Z` (+203ms).

Both windows correlate almost exactly (within adb clock offset) with
Android's own `SubnetProbe` logging `"open port 192.168.0.121:9001"` /
`":9002"` at local time `14:52:14.452` / `14:52:16.474` on the same session.
A dispatched opus-tier worker confirmed via `SubnetProbe.kt`'s `probeHost`
(lines 187-201): it opens a socket, sets `soTimeout=200ms`, attempts a
1-byte read (times out), then closes -- **writing zero bytes**, never
sending multistream-select or WS-handshake data. The 200-205ms failure
windows above are exactly that probe's 200ms socket timeout. No
`ConnectionEstablished` appears anywhere in the ~12800-line capture, and no
distinct real (non-probe) inbound dial event was found either.

**Conclusion: (B) is a false positive.** These "Incoming connection error:
Failed to negotiate transport protocol(s)" WARN logs are the CLI logging its
own LAN-discovery-probe traffic (bare TCP connect-then-close, self-inflicted
by the Android app's own SubnetProbe fallback) as if it were a failed
protocol negotiation with a real peer. There is no Noise/multistream-select
mismatch, no version skew, no genuine protocol bug on this path.

**Fix applied and merged:** `core/src/transport/swarm.rs`, both
`IncomingConnectionError` handlers (~line 3932 and ~5244): `tracing::warn!`
downgraded to `tracing::debug!`, message reworded from "Incoming connection
error" to "Incoming connection negotiation aborted", emoji removed, and an
explanatory comment added. No control-flow/error-handling/protocol logic
changed. `cargo check -p scmessenger-core`: clean (only 2 pre-existing
dead-field warnings). `crypto-security-auditor` adversarial review:
**CLEAR** (LOW/INFO findings only -- see
`HANDOFF/todo/P1_CORE_Rate_Limited_Negotiation_Failure_Signal.md` for the
one recommended, non-blocking follow-up: no per-remote-address rate-limited
signal exists for a genuine handshake-flood/probe pattern, since this fix
only silences the routine single-probe case at the log-macro level).

**This ticket's scope is now closed.** The real remaining blocker for
Android<->Windows messaging is tracked separately:
`HANDOFF/todo/P1_ANDROID_Inbound_Libp2p_Listener_Not_Externally_Reachable.md`
(mode A -- the phone's own inbound libp2p listener does not appear to be
externally reachable; ruled out as a Windows/network-path issue by this same
session's evidence). P1-09 (LAN E2E validation pass) depends on that ticket
landing first, not this one.

## Acceptance Criteria

- Identify the specific negotiation failure point: reproduce with
  `RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,libp2p_core=trace` (or
  whatever the equivalent trace-level targets are for this libp2p version)
  on the CLI side while a real Android device dials in, and capture the
  actual multistream-select/Noise error (the current log line is a generic
  "Listen error" wrapper  get to the underlying cause, e.g. protocol
  mismatch, version mismatch, or a specific Noise/TLS negotiation failure).
- Confirm whether the Android APK's bundled `libscmessenger_core.so` and
  this CLI's compiled dependencies (specifically `libp2p`, `libp2p-noise`,
  `libp2p-tcp`, `libp2p-websocket` versions) match `Cargo.lock` at the
  commit both were built from. If they diverge (e.g. Android was built from
  an older/newer commit with a different libp2p pin), that mismatch is
  likely the root cause and the fix is a rebuild/re-sync, not a code change.
- If the versions DO match and there's a genuine protocol/config bug, fix
  the actual negotiation mismatch (exact fix depends on what step 1 reveals
   do not guess a fix without first getting the real underlying error from
  trace logging).
- Add or extend an integration test exercising the negotiation path this bug
  affects if the root cause turns out to be a code-level config issue (not
  applicable if it's purely a build/version-skew issue caught by process,
  not code).
- **This touches `core/src/transport/`  the mandatory
  `crypto-security-auditor` adversarial review applies before this is
  considered done**, per `.claude/rules/security.md`, regardless of how
  small the eventual fix turns out to be.

## Implementation Plan

1. Reproduce with trace-level libp2p logging enabled on the CLI
   (`RUST_LOG` env var, see Acceptance Criteria) while dialing from a real
   Android device on the same LAN, and capture the specific negotiation
   error.
2. Cross-check `Cargo.lock`'s pinned `libp2p*` versions against what the
   Android build was actually compiled against (check
   `android/app/build.gradle` / CI artifacts / whatever records which Rust
   commit produced the currently-installed APK's `.so`, if determinable).
3. Based on findings, either: (a) document the version-skew finding and
   trigger a rebuild-and-redeploy of the Android APK from current `main` as
   the fix (process fix, not code), or (b) implement the actual protocol/
   config fix if it's a genuine code bug.
4. If a code fix is needed, add test coverage; either way, get the mandatory
   `crypto-security-auditor` review before closing.

## Files to Touch

- `core/src/transport/swarm.rs` (negotiation/listener setup  read first to scope the actual diff)
- Possibly `Cargo.toml`/`Cargo.lock` if a version-pin fix is needed
- `core/tests/` (new/extended integration test, if a code fix is needed)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib
cargo test -p scmessenger-core --test integration_e2e
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

Manual verification (required, this is fundamentally a live-network bug):
run the CLI daemon on Windows, dial in from a physical Android device on the
same LAN (real device, not emulator  this bug was found against a Pixel 6a),
confirm the connection completes (`ConnectionEstablished` logged, not
`Incoming connection error`) and a message can be sent end-to-end.
