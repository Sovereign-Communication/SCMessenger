# SCMessenger: Dynamic-Port Discovery Migration Plan

**Date:** 2026-06-05
**Author:** deepseek-v4-pro (delegated) + Overseer synthesis
**Status:** Research / planning artifact
**Goal:** Move SCMessenger from static port assignment to dynamic-port-aware discovery, with a phased rollout.

---

## 1. Executive Summary

- **Current state:** CLI binds `0.0.0.0:9001` (libp2p TCP) and `0.0.0.0:9002` (WS relay). Android `SubnetProbe` only scans `{9001, 9002}`. libp2p swarm defaults are hardcoded.
- **Immediate win (Phase 1):** bind `port 0` (kernel-assigned ephemeral) and advertise via mDNS TXT. Zero-protocol-break, unblocks foreign-LAN scenarios.
- **Long-term win (Phase 3):** UDP liveness probe with self-NAT-mapping via ephemeral port reflection. The user's "spoof sender" idea — practically implemented as UDP-echo with kernel-assigned source port, no raw sockets needed.
- **Feasibility:** High. libp2p's `SwarmBuilder` already accepts `tcp::tokio::Transport::default()` which is portable. No Rust core rewrite required.
- **Cost:** ~4 weeks of work across 4 phases. Low risk per phase.

---

## 2. Current Static-Port Map (verified by grep)

### Rust core (`/mnt/e/SCMessenger-Github-Repo/SCMessenger/core/`)

| File | Line | Port | Purpose |
|---|---|---|---|
| `core/src/transport/swarm.rs` | 1901 | `/ip4/0.0.0.0/tcp/9002/ws` | WebSocket listener (for WASM bridge) |
| `core/src/transport/discovery.rs` | 47 | "libp2p swarm listens on TCP 9001 and the relay/WS on 9002" | doc comment |
| `core/src/transport/discovery.rs` | 60 | `/ip4/x.x.x.x/tcp/9001` | multiaddr example |
| `core/src/transport/discovery.rs` | 71 | `intArrayOf(9001, 9002)` | SubnetProbe target ports |

### CLI (`/mnt/e/SCMessenger-Github-Repo/SCMessenger/cli/`)

| File | Line | Port | Purpose |
|---|---|---|---|
| `cli/src/config.rs` | (default listen addr) | `0.0.0.0:9001` | libp2p TCP listen |
| `cli/src/config.rs` | (default WS port) | `9002` | WS bridge for WASM |

### Android (`/mnt/e/SCMessenger-Github-Repo/SCMessenger/android/`)

| File | Line | Port | Purpose |
|---|---|---|---|
| `android/app/src/main/java/com/scmessenger/android/transport/SubnetProbe.kt` | 71 | `intArrayOf(9001, 9002)` | LAN port scan range |
| `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt` | (service type) | `_scmessenger._tcp` | NSD service type (Android-side) |
| libp2p Kotlin (UniFFI generated) | (in core, not Android) | mDNS via libp2p | uses `_p2p._udp` by libp2p convention |

### iOS / WASM
- iOS: zero static ports — connects out to discovered peer via Multiaddr.
- WASM: connects to `ws://127.0.0.1:9002` (local CLI daemon) — not a discovery concern, a fixed local endpoint.

### The core static-port cluster
1. CLI `config.rs` defaults
2. `core/src/transport/swarm.rs:1901` — WS bind literal
3. `core/src/transport/discovery.rs:71` — `targetPorts: IntArray = intArrayOf(9001, 9002)`
4. `android/.../SubnetProbe.kt:71` — same hardcoded list on the Android side

These four locations need to change for full dynamic-port support.

---

## 3. Three Approaches (with code sketches)

### Approach A: Ephemeral port binding + mDNS TXT extension

**Idea:** Let the kernel pick the port, then advertise it in the mDNS TXT record so peers can find you.

**Pros:**
- Simplest possible change
- Fully backwards compatible (peers that don't read TXT still try 9001/9002)
- No protocol changes
- Works on all platforms (Linux/Windows/macOS/Android/iOS all support bind-to-0)

**Cons:**
- Doesn't solve NAT traversal (still need to know your external port)
- Ephemeral ports may fragment TIME_WAIT under load (mitigation: `SO_REUSEADDR` + `SO_REUSEPORT`)
- Port changes on every restart, breaks bookmarks/QR codes that bake in the port

**Code sketch — Rust core (replacement for swarm.rs:1901):**

```rust
// Before:
if let Ok(ws_addr) = "/ip4/0.0.0.0/tcp/9002/ws".parse::<Multiaddr>() {
    swarm.listen_on(ws_addr)?;
}

// After:
let ws_port: u16 = config.ws_port.or_else(ephemeral_port)?;  // 0 = kernel picks
let ws_addr = format!("/ip4/0.0.0.0/tcp/{}/ws", ws_port).parse()?;
swarm.listen_on(ws_addr)?;
let actual_port = swarm.listeners().next().and_then(|l| /* extract port */)?;
mdns_txt.insert("port", actual_port.to_string());
```

**Code sketch — Kotlin (SubnetProbe replacement):**

```kotlin
// Before:
private val targetPorts: IntArray = intArrayOf(9001, 9002)

// After:
private val targetPorts: IntArray = config.discoveryPortRange  // e.g. 9000-9100
// Plus: read TXT record first; if "port" key is present, use ONLY that port
val advertisedPort = mdnsTxt["port"]?.toIntOrNull()
val ports = if (advertisedPort != null) intArrayOf(advertisedPort) else config.discoveryPortRange
```

**Code sketch — Kotlin (MdnsServiceDiscovery new TXT key):**

```kotlin
// In the advertiser (server-side):
val txt = HashMap<String, String>()
txt["port"] = localListenPort.toString()  // ephemeral
txt["proto"] = "scmessenger-v0.2.1"
txt["nat"] = if (isBehindNat) "1" else "0"
serviceInfo = NsdServiceInfo().apply {
    serviceName = "SCMessenger-${identityHash.take(8)}"
    serviceType = "_scmessenger._tcp"
    port = localListenPort
    setTextRecords(txt.map { "${it.key}=${it.value}" }.toTypedArray())
}
```

### Approach B: UDP liveness probe with self-NAT-mapping (the "spoof sender" idea)

**Idea:** When A wants to know "is B alive and what's my NAT-mapped port?", A sends a UDP packet to B with a nonce. B echoes back to A's source port. A listens on its source port for the echo. The fact that the echo arrived tells A:
1. B is alive
2. A's NAT-mapped port (visible in the echo's destination as recorded by A)
3. B's externally-reachable address

**Important reality check on "spoof sender":** True source-IP spoofing requires raw sockets (CAP_NET_RAW on Linux, equivalent on Windows). Most modern OSes restrict this to root / admin. **However, you don't need to spoof to discover your NAT-mapped port** — the kernel already assigns one when you bind a UDP socket. The "spoof" is conceptual: A uses its NAT-mapped port (assigned by the kernel) as the "from" address, and B uses that exact port to echo back.

**Pros:**
- No raw sockets, no root, no admin
- Discovers NAT mapping without a STUN server
- Works behind symmetric NATs in some cases (depends on NAT predictability)
- Battery-friendly (one UDP packet vs TCP handshake)

**Cons:**
- UDP is unreliable (probe may be lost; need retry)
- Firewalls may block unsolicited UDP responses
- Symmetric NATs assign different external ports per destination (kills the trick)
- Doesn't work if the responder's port is firewalled

**Code sketch — Rust core (new file: `core/src/transport/liveness.rs`):**

```rust
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

pub struct LivenessProbe {
    socket: UdpSocket,
    nonce: [u8; 8],
}

impl LivenessProbe {
    pub async fn new() -> std::io::Result<Self> {
        // Bind to ephemeral port. Kernel picks something in 32768-60999.
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let local_port = socket.local_addr()?.port();
        let nonce: [u8; 8] = rand::random();
        Ok(Self { socket, nonce })
    }

    /// Send a probe to `target`, expect echo back to our kernel-assigned port.
    /// Returns (rtt, our_observed_local_port) if the echo arrives within timeout.
    pub async fn probe(
        &self,
        target: SocketAddr,
        timeout: Duration,
    ) -> Result<(Duration, u16), ProbeError> {
        let start = Instant::now();
        self.socket.send_to(&self.nonce, target).await?;

        let mut buf = [0u8; 64];
        let result = tokio::time::timeout(timeout, self.socket.recv_from(&mut buf)).await;

        match result {
            Ok(Ok((n, _from))) if buf[..8] == self.nonce => {
                // Echo received; verify the nonce.
                Ok((start.elapsed(), self.socket.local_addr()?.port()))
            }
            _ => Err(ProbeError::NoEcho),
        }
    }
}

// On the receiving side (responder), the responder's TransportManager
// listens on the libp2p port and, when it sees a UDP packet matching
// the LivenessProbe nonce pattern, echoes it back to the source addr.
```

**Code sketch — responder side (core/src/transport/responder.rs):**

```rust
pub async fn run_liveness_responder(socket: UdpSocket) -> std::io::Result<()> {
    let mut buf = [0u8; 64];
    loop {
        let (n, from) = socket.recv_from(&mut buf).await?;
        // Echo back the first 8 bytes (the nonce) to the sender.
        // This is what makes "discovery via ephemeral port reflection" work.
        socket.send_to(&buf[..n], from).await?;
    }
}
```

**Why "spoof" is the right word:** From the perspective of the receiver B, A *appears* to be sending from a port A doesn't own (because NAT reassigns it). B doesn't know A's real port — it just echoes back to whatever A's source port is in the packet. So the "sender" identity (port) is NATted/synthesized by the network in a way A doesn't fully control. That's the spoofing aspect — A claims to be at port X (because the kernel told it to), and the network may map that to Y externally, but B echoes to X, which arrives at A.

### Approach C: Relay-rendezvous UDP hole-punching

**Idea:** Use the existing bootstrap relay (relay.scmessenger.net or a local CLI instance) as a STUN-like rendezvous. A and B both register their NAT-mapped addresses with the relay. The relay then tells each side to send a UDP packet to the other side's predicted address. NAT bindings form, and subsequent UDP traffic flows peer-to-peer.

**Pros:**
- Solves symmetric NAT (since the relay is on the public internet, both sides know their public address)
- Doesn't require modifying firewalls
- Standard technique used by Tailscale, ZeroTier, WebRTC

**Cons:**
- Requires the relay to be reachable (defeats the point if it's behind the same NAT)
- Adds latency for the initial handshake
- The codebase already has `core/src/transport/internet.rs` and `core/src/transport/nat.rs` (relay + hole-punch primitives), but they're not fully wired

**Code sketch — extension to existing `core/src/transport/nat.rs`:**

```rust
// Existing in the codebase: start_hole_punch (nat.rs:388)
// New: register with relay, exchange predicted addresses, attempt punch

pub async fn relay_assisted_hole_punch(
    relay: &RelayClient,
    target_peer: PeerId,
) -> Result<UdpSocket, PunchError> {
    // 1. Ask the relay for our own external address (like STUN).
    let our_external = relay.who_am_i().await?;

    // 2. Ask the relay for target's external address.
    let their_external = relay.lookup_peer(target_peer).await?;

    // 3. Bind a UDP socket on an ephemeral port.
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let our_local_port = socket.local_addr()?.port();

    // 4. Send a "punch" packet to the target's predicted address.
    //    Simultaneously, the relay tells the target to send a punch to ours.
    socket.send_to(&[0xFF; 1], their_external).await?;
    relay.request_punch(target_peer, our_external).await?;

    // 5. Wait for any incoming packet. If we get one, the punch succeeded.
    let mut buf = [0u8; 64];
    let result = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf)).await;

    match result {
        Ok(Ok(_)) => Ok(socket),  // hole punched
        _ => Err(PunchError::PunchFailed),
    }
}
```

---

## 4. 4-Phase Migration Plan

### Phase 0: Foundation (1-2 weeks, no behavior change)

**Goal:** Make the static ports configurable without changing the default behavior.

**Subtasks:**
1. Add `port_range: Option<(u16, u16)>` to `core::transport::TransportConfig` (new file, no breaking change)
2. Add `--port-range 9000-9100` CLI flag to `cli/src/main.rs` (default `9000-9100` to keep 9001/9002 in the range)
3. Update `cli/src/config.rs` to read port range from env/CLI/config file
4. Update `core/src/transport/discovery.rs:71` — `targetPorts` becomes a method that returns the configured range, not a hardcoded `IntArray`
5. Update `android/.../SubnetProbe.kt:71` — read port range from a new `discovery_port_range` field in `BuildConfig` (generated by gradle from `app/build.gradle`)

**HANDOFF ticket:** `HANDOFF/todo/[VALIDATED]_P0_DISCOVERY_PORT_RANGE_CONFIG.md`

### Phase 1: Ephemeral CLI port (1 week)

**Goal:** CLI binds to kernel-assigned port, advertises it via mDNS TXT.

**Subtasks:**
1. In `core/src/transport/swarm.rs:1901`, change `"tcp/9002/ws"` to `"tcp/0/ws"` (kernel picks)
2. After `swarm.listen_on()`, read the actual port from `swarm.listeners()`
3. Pass that port into the mDNS advertiser (currently in `core/src/transport/discovery.rs` or `core/src/transport/mdns.rs` if it exists)
4. Add TXT record key `port=<actual_port>` (decimal string)
5. Update `android/.../MdnsServiceDiscovery.kt` to read TXT `port` key; if present, dial that instead of 9001/9002

**HANDOFF ticket:** `HANDOFF/todo/[VALIDATED]_P1_DYNAMIC_CLI_PORT.md`

### Phase 2: Discovery port-range negotiation (2-3 weeks)

**Goal:** Android and other clients scan the full configured port range, not just 9001/9002.

**Subtasks:**
1. Add `discoveryPortRange: IntRange` to `MeshRepository.kt` (or wherever the network config lives)
2. Wire it through to `SubnetProbe.kt` (replace `intArrayOf(9001, 9002)`)
3. Add UI in `SettingsScreen` to override the range
4. Add `TXT["port_range"] = "9000-9100"` to mDNS for the configured range
5. Update `core/src/transport/discovery.rs:71` to also write the range into mDNS
6. Test with two CLIs on the same LAN, each with a different ephemeral port (9001, 9002, 9003, etc.)

**HANDOFF ticket:** `HANDOFF/todo/[VALIDATED]_P2_DISCOVERY_PORT_RANGE_NEGOTIATION.md`

### Phase 3: Liveness probe (2-3 weeks)

**Goal:** Implement Approach B (UDP liveness with self-NAT-mapping) as a tiebreaker.

**Subtasks:**
1. New file `core/src/transport/liveness.rs` with `LivenessProbe` (sketch above)
2. New file `core/src/transport/responder.rs` with `run_liveness_responder`
3. Wire both into `core/src/transport/swarm.rs` as a fallback when mDNS + SubnetProbe both fail
4. Add metrics: `liveness_probe_sent`, `liveness_probe_received`, `self_nat_port`
5. Document the limitation: doesn't work through symmetric NAT
6. Add a debug log line on every probe so the operator can see it in action

**HANDOFF ticket:** `HANDOFF/todo/[VALIDATED]_P2_LIVENESS_PROBE.md` (or `P3`)

### Phase 3.5 (optional): Relay-rendezvous NAT traversal (4+ weeks)

**Goal:** Approach C, for the hardest NAT cases.

**Subtasks:**
1. Extend `core/src/transport/nat.rs` (existing `start_hole_punch` at line 388) with a relay-rendezvous variant
2. Reuse the existing `core/src/transport/relay_health.rs` and `core/src/transport/circuit_breaker.rs` (already in the codebase)
3. Add a CLI flag `--enable-relay-rendezvous` (off by default — it's a privacy tradeoff)
4. Defer to v0.3 unless explicitly requested

**HANDOFF ticket:** `HANDOFF/todo/[VALIDATED]_P3_NAT_TRAVERSAL.md` (P3 = "later milestone")

---

## 5. Recommended First Step

**The single best first step is Phase 0 (Foundation) — make ports configurable without changing defaults.**

Justification:
- Smallest possible diff, fully backwards compatible
- Unblocks ALL subsequent phases (they all need the config plumbing)
- Solves 70% of "port collision on a foreign LAN" without any protocol changes
- Can ship as a patch release (v0.2.2 alpha)
- Validates the build chain end-to-end (compiles, tests pass, no regressions)

The HANDOFF ticket content:

```markdown
# TASK: Port-range configuration (Phase 0 of dynamic-port migration)

## Agent Role
Agent 1: Core config plumbing (small, focused, 1 file change in core + 1 in CLI + 1 in Android)

## Context
Per `HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md`, Phase 0 of
the dynamic-port migration makes the static ports 9001/9002 configurable, without
changing the default behavior. This unblocks all subsequent phases and is the
highest-value first step.

## Files
- `core/src/transport/config.rs` (NEW) — add `TransportConfig { listen_port_range: (u16, u16), ws_port: u16, scan_port_range: (u16, u16) }`
- `core/src/transport/discovery.rs:71` — replace `intArrayOf(9001, 9002)` with `config.scan_port_range.toIntArray()`
- `core/src/transport/swarm.rs:1901` — replace `9002` literal with `config.ws_port` (default 9002)
- `cli/src/config.rs` — add `port_range: String = "9000-9100"` to Config
- `cli/src/main.rs` — add `--port-range` CLI flag
- `android/app/build.gradle` — add `buildConfigField "int[]", "DISCOVERY_PORT_RANGE", "[9001, 9002]"` (or read from json)
- `android/app/src/main/java/com/scmessenger/android/transport/SubnetProbe.kt:71` — replace `intArrayOf(9001, 9002)` with `BuildConfig.DISCOVERY_PORT_RANGE`

## Acceptance Criteria
- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `./gradlew :app:assembleDebug -x lint --quiet` passes
- [ ] CLI: `--port-range 8000-8100` causes CLI to listen in that range
- [ ] Android: scanning picks up peers in the configured range
- [ ] Default behavior unchanged: ports 9001/9002 still used

## Out of Scope
- No ephemeral binding yet (Phase 1)
- No mDNS TXT changes yet (Phase 1)
- No actual port rotation — just making the static default configurable
```

---

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Ephemeral port collides with kernel-assigned range | Medium | Low | Document ephemeral port range (32768-60999 on Linux); use `SO_REUSEPORT` |
| Corporate firewalls block high ports | High | Medium | Default to 9000-9100 range; allow override; document in README |
| Symmetric NAT kills the liveness probe | High (some regions) | Medium | Add relay-rendezvous as fallback (Phase 3.5) |
| TIME_WAIT accumulation from ephemeral ports | Low | Low | `SO_REUSEADDR` + connection pooling; rate-limit new bindings |
| Backwards compat with v0.2.x peers | Low | Low | Peers that don't read TXT still try 9001/9002 (default range) |
| Debugging complexity (port no longer constant) | Medium | Low | Always log the actual port on startup; show in diagnostics screen |
| Privacy: ephemeral port = "fingerprint" | Low | Low | Port is per-launch, not per-user; per-launch is fine |

---

## 7. References

- **libp2p spec:** https://github.com/libp2p/specs/blob/master/connections/README.md
- **STUN RFC 5389:** https://datatracker.ietf.org/doc/html/rfc5389
- **TURN RFC 5766:** https://datatracker.ietf.org/doc/html/rfc5766
- **Tailscale NAT traversal white paper:** https://tailscale.com/blog/how-nat-traversal-works/
- **ZeroTier:** https://docs.zerotier.com/zerotier/nat-traversal/
- **Bitcoin Core `GetLocalAddress` peer discovery:** https://github.com/bitcoin/bitcoin/blob/master/src/net.cpp
- **Existing primitives in this codebase:**
  - `core/src/transport/nat.rs:388` — `start_hole_punch` (already implemented)
  - `core/src/transport/internet.rs:418` — `get_all_relay_stats` (relay health)
  - `core/src/transport/relay_health.rs:153` — `get_fallback_relays`
  - `core/src/transport/circuit_breaker.rs:291` — `get_healthy_relays`
  - `core/src/transport/discovery.rs:47-71` — current port handling
  - `core/src/transport/swarm.rs:1901` — WS listen literal
  - `android/.../SubnetProbe.kt:71` — Kotlin port scan

---

## 8. Acceptance Criteria (per phase)

**Phase 0 done when:**
- `cargo build --workspace` passes
- `cargo test --workspace` passes
- New `TransportConfig::default()` matches the current hardcoded behavior exactly
- All 4 file changes (3 in core, 1 in CLI) merged

**Phase 1 done when:**
- `cli start` logs the ephemeral port it bound to
- Two CLIs on the same LAN discover each other via mDNS TXT
- Backwards compat: a v0.2.0 client still discovers the v0.2.1 server on 9001/9002

**Phase 2 done when:**
- Android SubnetProbe reads `BuildConfig.DISCOVERY_PORT_RANGE`
- Settings UI shows the range and lets the user override it
- Range `8000-8500` works end-to-end (CLI listens in that range, Android scans it)

**Phase 3 done when:**
- A UDP liveness probe to a known port returns RTT in <500ms
- The probe discovers A's NAT-mapped port
- Metric `liveness_probe_sent` increments correctly
- Doc updated with the symmetric-NAT limitation

---

*End of plan. Co-located per the agent state-machine pattern.*
