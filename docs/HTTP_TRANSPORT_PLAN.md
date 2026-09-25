# SCMessenger: HTTP-Compliant Transport Plan

## 0. Problem

Some environments allow **only HTTP(S)** — raw TCP/UDP dials are killed at the
egress, regardless of port. The current multi-port strategy
(`core/src/transport/multiport.rs`, ports 443/80/8080/9090) binds *common
ports* but still speaks raw libp2p TCP/Noise on them. **Port agility does not
help when the protocol itself is filtered.**

Verified empirically (2026-09-25, restricted sandbox): the egress proxy
intercepts every raw TCP socket and kills non-HTTP traffic. libp2p Noise
handshakes to two known-good relays failed identically on v0.2.1 and v0.4.0
clients — the relays were never at fault; the bytes never left the sandbox.
Only proxied HTTP(S) works.

**Goal:** add HTTP-compliant transport rungs to the dial ladder so two nodes
can message when *only* HTTP is available, with zero changes to the E2E
encryption envelope.

## 1. Design

### 1.1 The ladder today

`core/src/transport/dial_policy.rs` already orders candidates as
**direct → relay (circuit) → fallback**. We extend it to:

```
direct TCP/QUIC → proxy-aware WSS → HTTPS mailbox → (fail)
```

Each rung is tried in order; success at any rung stops the ladder. This composes
with the existing per-peer backoff state machine — no new retry logic needed.

### 1.2 Path A — Proxy-aware WebSocket Secure (WSS)

Full libp2p stack (Noise + Yamux) carried over WebSocket Secure, dialed
**through the environment's HTTP proxy** via `HTTP CONNECT`.

- **Dial:** extend `core/src/transport/websocket.rs` (`tokio-tungstenite`
  client). Today it dials raw TCP. Add: read `HTTP_PROXY`/`HTTPS_PROXY`/
  `ALL_PROXY`, issue `CONNECT host:443`, then run the WS upgrade
  (`client_async`) over the tunnel. No proxy env → direct dial as today.
- **Listen:** `/wss` on 443. `multiport.rs` already prioritizes 443; add TLS
  listener wiring in `swarm.rs` next to the existing
  `.with_websocket(...)` builder (swarm.rs:3807).
- **Why first:** standard protocol, preserves the entire libp2p upgrade stack,
  works through most corporate proxies (CONNECT to 443 is usually allowed).
  It is "HTTP-compliant" at the proxy layer: the proxy sees an HTTP CONNECT
  to port 443, nothing else.

### 1.3 Path B — HTTPS mailbox transport (strictest environments)

For environments with **no CONNECT, no raw sockets** — only plain
GET/POST through a proxy (exactly the sandbox that motivated this plan).

- New `TransportType::HttpPoll` in `core/src/transport/abstraction.rs`, with a
  new `core/src/transport/http_poll.rs` client.
- **Server side:** every node already runs an axum HTTP API
  (`cli/src/api_axum.rs`, `--http-bind`). Add mesh routes:
  - `POST /mesh/v1/deliver` — accept an encrypted envelope for a peer
  - `GET  /mesh/v1/inbox?peer_id=…&cursor=…` — long-poll for incoming envelopes
- **Backing store:** `core/src/store/relay_custody.rs` (`CustodyMessage`,
  `RelayCustodyStore`) already implements store-and-forward with retention
  windows — the mailbox is a custody store exposed over HTTPS.
- **Envelope:** reuse the existing E2E-encrypted message envelope unchanged
  (it is transport-independent; crypto self-tests already pass over it).
  The mailbox operator sees **metadata only** (sender/recipient peer IDs,
  timestamps, sizes) — never content.
- **Auth:** requests signed with the node's identity key; nonces + timestamps
  for replay protection.
- **Architecture fit:** "all nodes are relays" extends naturally — **every
  node can serve the mailbox API**, so any node is a potential HTTP relay
  for others. No dedicated infrastructure required.

### 1.4 Path C (stretch) — HTTP CONNECT tunneling for raw TCP

If CONNECT is allowed but WS upgrade is not: make the TCP dial path attempt
`CONNECT` through the proxy on direct-dial failure, then run the normal
Noise/Yamux stack inside the tunnel. Smaller win than A/B; do only if a
real environment needs it.

### 1.5 Environment awareness: detect, then mutate (core requirement)

The transport must not be tuned for any single restricted environment. Instead,
each node **probes its environment, builds a profile, and mutates its transport
strategy to fit** — at startup, on network change, and on persistent failure.

**Probe suite** — new `core/src/transport/env_probe.rs`. All probes run
concurrently with an overall deadline (~5s), short per-probe timeouts (2–3s).
Probe *targets* are the node's own bootstrap/relay nodes (their HTTP/health
endpoints) — never third-party hosts — so probing leaks nothing beyond what
joining the mesh already reveals:

| Probe | Method | Capability bit |
|---|---|---|
| Raw TCP egress | `TcpStream::connect` to bootstrap on non-HTTP port | `raw_tcp_egress` |
| Raw UDP egress | short-timeout QUIC dial attempt to bootstrap | `raw_udp_egress` |
| HTTP proxy | `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` env + `CONNECT` attempt | `proxy: Option<ProxyInfo>` (present? CONNECT allowed?) |
| Plain HTTPS egress | `GET /health` via proxy or direct | `https_egress` |
| DNS | resolve bootstrap hostname | `dns_works` |
| WS upgrade | `ws://` handshake attempt | `ws_upgrade_works` |

**EnvironmentProfile** — the probe output, cached to disk alongside identity
so cold start is fast, re-validated in background:

```rust
pub struct EnvironmentProfile {
    pub raw_tcp_egress: bool,
    pub raw_udp_egress: bool,
    pub proxy: Option<ProxyInfo>, // present, CONNECT allowed?
    pub https_egress: bool,
    pub dns_works: bool,
    pub ws_upgrade_works: bool,
    pub detected_at: Instant,
}
```

**Strategy mutation** — `TransportStrategy` (ordered ladder + listen decisions)
derived from the profile by explicit rules, e.g.:

- `raw_tcp && raw_udp` → `[TCP, QUIC, WSS-direct, mailbox]`
- `raw_tcp` only → `[TCP, WSS-direct, mailbox]`
- `proxy.connect_allowed` → `[WSS-via-proxy, mailbox]`
- `https_egress` only → `[mailbox]`
- none → fail with diagnostics (see below), do not hammer dead paths

Listen side mutates too: if inbound is likely blocked (no raw egress usually
means symmetric NAT), prioritize relay reservation over extra listen sockets.

**Re-probe triggers** (environments change — VPN on/off, captive portals,
carrier switching):
1. Startup (fast path: cached profile, background re-validation).
2. OS network-change signals where available; heuristic fallback: sudden mass
   dial failures across all rungs → re-probe.
3. Failure-driven degradation: if the top rung fails N consecutive dials
   (reuse the existing per-peer backoff in `dial_policy.rs`), demote that rung
   and re-probe the specific capability in background; mutate if confirmed.
4. Periodic bounded re-probe (e.g. bootstrap sweep cadence; back off on mobile).

**Diagnostics** — `scm doctor` (or `scm netinfo`) prints the detected profile
and the chosen strategy with per-rung reasoning. This is the single highest-
leverage debuggability win: on 2026-09-25 it would have distinguished "relay
is dead" from "my egress kills raw TCP" in seconds.

**Immediate value:** even before new transports land, environment awareness
stops the node hammering provably-dead paths and converges faster on the
relay/circuit paths that already exist.

## 2. Capability advertisement & negotiation

- Extend `TransportBridge` (`cli/src/transport_bridge.rs`) `peer_capabilities`
  with the new transport types; advertise via identify push.
- Extend `EscalationPolicy` (`core/src/transport/escalation.rs`): the
  `Balanced` default should score HTTP-compliance (a working slow path beats
  a dead fast path). Consider `PreferHttpCompliance` for locked-down fleets.

## 4. Phases

### Phase 0 — Environment matrix fixture (acceptance environment)
Generalize the restricted-egress sandbox into a **parameterized fixture** with
multiple profiles: `open`, `corporate-proxy` (CONNECT allowed),
`http-only` (GET/POST only — the motivating sandbox), `no-dns`,
`captive-portal`. CI runs the mesh test-suite under each profile. The contract:
**the node must converge on a working strategy in every profile where any
path exists**, and must report a correct diagnosis where none does.

### Phase 1 — Environment probing + strategy mutation
- `env_probe.rs` probe suite; `EnvironmentProfile` model + disk cache;
  strategy-derivation rules; re-probe triggers (startup, network-change,
  failure-driven, periodic).
- Wire the dynamic ladder into `dial_policy.rs`; `escalation.rs` scoring
  consumes the profile; `scm doctor` diagnostics output.
- Bootstrap dial path (`cli/src/bootstrap.rs`) becomes environment-aware too.
- **Accept:** under each fixture profile, the node selects the expected ladder
  order within the probe deadline; `scm doctor` output matches the fixture's
  ground truth; failure injection mid-run triggers re-probe and ladder
  mutation without restart.

### Phase 2 — Proxy-aware WSS
- CONNECT-capable dial in `websocket.rs` (consumes proxy detection from
  Phase 1); `/wss` listener on 443 in `swarm.rs`; TLS cert handling (see §6).
- **Accept:** two nodes exchange messages via WSS through the
  `corporate-proxy` fixture; direct TCP dials fail, WSS rung succeeds.

### Phase 3 — HTTPS mailbox
- `POST /mesh/v1/deliver` + `GET /mesh/v1/inbox` on the axum server;
  `http_poll.rs` client; identity-signed requests; custody-store backing.
- **Accept:** under the `http-only` fixture (raw TCP *and* CONNECT blocked),
  two nodes exchange E2E-encrypted messages via mailbox; a malicious mailbox
  operator learns nothing about content (assert on captured traffic).

### Phase 4 — Ladder integration hardening
- Config surface (env vars + config file, consistent with `SC_BOOTSTRAP_NODES`
  pattern); per-rung metrics (which rung succeeded, latency per rung);
  mobile probe budgets (battery-aware re-probe intervals).

### Phase 5 — Security review & dogfood
- Metadata minimization: padding, batching, dummy traffic (compose with the
  existing `config privacy` onion/padding features).
- Rate limiting per peer on mailbox endpoints; abuse handling.
- Dogfood in real restricted networks (corporate, carrier, national);
  docs.

## 5. Files to touch

| Area | File | Change |
|---|---|---|
| Env detection (new) | `core/src/transport/env_probe.rs` | Probe suite → `EnvironmentProfile` |
| Strategy (new) | `core/src/transport/strategy.rs` | Profile → ladder/listen mutation rules |
| Transport types | `core/src/transport/abstraction.rs` | New `TransportType` variants |
| WS dial | `core/src/transport/websocket.rs` | Proxy-aware CONNECT + upgrade |
| Swarm wiring | `core/src/transport/swarm.rs` | `/wss` listener, transport stack |
| Port strategy | `core/src/transport/multiport.rs` | 443/WSS coordination |
| Dial order | `core/src/transport/dial_policy.rs` | Dynamic ladder from strategy |
| Negotiation | `core/src/transport/escalation.rs` | Profile-aware policy scoring |
| Transport manager | `core/src/transport/manager.rs` | Own the probe/strategy lifecycle |
| Bootstrap | `cli/src/bootstrap.rs` | Environment-aware bootstrap dial |
| Mailbox server | `cli/src/api_axum.rs` | `/mesh/v1/*` routes |
| Mailbox store | `core/src/store/relay_custody.rs` | Backing store reuse |
| Capabilities | `cli/src/transport_bridge.rs` | Advertisement |
| Diagnostics | CLI (`scm doctor`/`scm netinfo`) | Profile + strategy output |
| New client | `core/src/transport/http_poll.rs` | Mailbox polling transport |
| Fixture | `infra/` + `.github/workflows/` | Environment-matrix CI job |

## 6. Open questions

1. **Mailbox ubiquity:** should *every* node expose the mailbox HTTP API by
   default (fits "all nodes are relays"), or opt-in? Default-on maximizes
   the chance an HTTP-only node finds a mailbox; opt-in reduces attack surface.
2. **TLS for WSS:** self-signed + TOFU bound to the node's identity key, or
   Let's Encrypt for public relays? (Tofu-via-identity is the mesh-native
   answer; LE is the ops-heavy one.)
3. **Rendezvous vs. transport:** is the mailbox purely a transport for
   known peers, or should it also serve peer discovery (a directory)? The
   latter is a bigger privacy design task — recommend transport-only for v1.
4. **HTTP/2 / HTTP/3:** mailbox long-poll works fine on HTTP/1.1; multiplexed
   streams would help latency later. Defer.
5. **Probe privacy:** probing bootstrap nodes reveals the prober is online —
   but joining the mesh reveals that anyway. Confirm no probe may target
   third-party hosts; probe targets must be configurable, defaulting to the
   node's own bootstrap set.
6. **Detection default:** environment probing on by default with opt-out
   (`--no-env-probe` / config), or opt-in? Recommend default-on: a node that
   can't detect its environment can't mutate its strategy.
7. **Mobile probe budget:** Android nodes need battery-aware re-probe
   intervals (only on network change + coarse periodic). Define the budget
   in Phase 1, not later.

## 7. Rough sizing

- Phase 0: S (matrix fixture + CI job)
- Phase 1: M (probe suite + strategy engine + dial integration + doctor)
- Phase 2: M (proxy CONNECT + TLS listener)
- Phase 3: M–L (new transport + server endpoints + request signing)
- Phase 4: S–M (config, metrics, mobile budgets)
- Phase 5: process (review, dogfood, docs)

## Appendix — evidence this is needed

- 2026-09-25: v0.2.1 and v0.4.0 (main @ d00cfd5) clients both failed Noise
  handshakes to `34.168.102.7:9001` and `34.135.34.73:9001` from a restricted
  sandbox. Raw-socket probe returned the proxy policy message
  ("Other TCP connections is turned off"), including for plain HTTP to
  `example.com:80`. Only proxied HTTP(S) (curl via `hatch-egress-proxy`)
  works. The relays were never reached; the failures were local egress
  policy, reproducible and deterministic. This environment is one profile
  (`http-only`) in the Phase 0 matrix — not the design target.
