# Reticulum Audit for SCMessenger

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active (canonical record of the 2026-09-21 audit)
Companion plan: `HANDOFF/plans/V050_PHILOSOPHY_AND_BORROW_PLAN.md` (the execution queue for everything this audit defers)
Auditor: Freebuff lane session, 2026-09-21

---

## Governing rule

> Adopt Reticulum/LXMF **designs and philosophy only. Adopt zero RNS/LXMF code.**
> No wire-compatibility claims, no "RNS-like" self-description, no crypto
> downgrades. RNS is a network stack; SCMessenger is a messenger product --
> borrowing happens at the design-note level, per element, behind the existing
> gates (`core/src/{crypto,transport,routing,privacy}` adversarial review,
> operator escalation on architecture).

## Sources (what was actually read)

- Reticulum manual, chapter "Understanding Reticulum" (markqvist.github.io, RNS 1.5.4 docs)
- RNS master-branch README (public mirror)
- "Zen of Reticulum" (master branch; text truncates mid section V)
- LXMF master-branch README
- SCMessenger: `README.md`, `docs/CLAUDE_REFERENCE.md`, `SHIP_PLAN.md` (lines 1-150 of 539),
  `docs/V1_KNOWN_LIMITATIONS.md`, `HANDOFF/todo/_QUEUE.md`, plus code verification
  listed under "Verified corrections" below.

Not read: RNS/LXMF source code. This is a manual/docs-level audit. Any
implementation-adoption decision must begin by reading the relevant LXMF/RNS
source; see "Confidence and limits."

---

## 0. Executive verdict

**Does Reticulum get SCMessenger shipped faster? No -- not for v0.4.0, and not
by adoption.** Nothing in RNS touches D1-D7 (green main, signed APK, two-device
proof, transport racing, offline proximity). The v0.4.0 blockers are
build/release/test problems, and RNS offers nothing there.

**The audit is still valuable.** RNS is the most philosophically aligned system
reviewed against SCMessenger: sovereign, encryption-as-gravity,
store-and-forward as primary mode, portable identity. It ships several designs
SCMessenger will eventually need. The posture is: **adopt zero code, borrow a
small set of designs, internalize three meta-lessons.** The borrow list is
post-tag work (v0.5.0 / v1.0 / v1.1+), sequenced in the companion plan.

Layer clarification framing the whole audit: **Reticulum is a network stack;
SCMessenger is a messenger product.** RNS sits below its messaging layer (LXMF)
the way SCMessenger's libp2p transport + drift/ framing sits below its message
layer. Element by element, SCMessenger already holds or exceeds RNS on crypto
and platform reach; RNS holds or exceeds SCMessenger on metadata minimization,
low-bandwidth discipline, and propagation-store federation (the last now
partially converged -- see correction below).

---

## Verified corrections (operator, 2026-09-21, code-verified this session)

### Correction 1 -- device_id (audit element 1.2)

SCMessenger binds a `device_id` alongside the Ed25519 identity. Verified in
source this session:

- `core/src/identity/store.rs` -- UUIDv4 generated at identity-store creation,
  persisted under `identity_device_id`, revalidated as UUIDv4 on every load.
- `core/src/identity/mod.rs` tests -- same store reopened yields the same id
  (stable); a fresh store yields a different id.
- `core/src/transport/behaviour.rs` -- transport registration validates it.
- `core/src/store/contacts.rs:795` -- `last_known_device_id` is used as
  `intended_device_id` for WS13 tight-pair routing.
- `core/src/store/relay_custody.rs` -- custody registration binds
  `(identity_id, device_id, seniority_timestamp)`.

Nuance: RNS-style *immutability* (address derived from identity keys) belongs
to SCMessenger's Ed25519 identity -- the "true name." `device_id` is a
per-device-store registry handle: stable for the store's life, regenerating on
a fresh store **by design** (multi-device support). One audit-relevant find:
`device_id` is serialized **on the wire** in the identity envelope sender block
(`core/src/message/identity_envelope.rs:29`) -- recorded in the envelope-v3
metadata-minimization wish list in the companion plan.

### Correction 2 -- custody is all-nodes, not AWS-only (audit element 1.7)

The AGENTS.md doctrine already declares every node a store-and-forward custody
holder ("custody is a behavior, not a role"); the AWS always-on node was the
first validation target, not the end state. Element 1.7 verdict upgraded from
"gap" to **"convergent doctrine -- the only borrow remaining is the
custody-store peering/sync protocol design."**

---

## 1. Element-by-element audit

Each element: Pros / Cons / Philosophy alignment / Verdict.

### 1.1 Wholesale adoption of RNS as the transport layer

- Pros: Proven multi-medium support (LoRa, packet radio, serial, IP) down to
  5 bits/s; mature announce/path/link machinery; initiator anonymity at the
  network layer.
- Cons: The reference implementation is Python -- it cannot be embedded in the
  Rust core feeding UniFFI Kotlin/Swift and WASM. A Rust reimplementation would
  land exactly where RNS's "Brandolini's Reference" warning aims: the project
  publicly polices wire-incompatible and LLM-generated reimplementations,
  listing only two community ports (C++ and Go) after 18+ months of proven
  wire-compat each. Crypto would be a regression: RNS is X25519/Ed25519 +
  AES-256-CBC+HMAC, no post-quantum, no double ratchet -- SCMessenger's hybrid
  ML-KEM-768/ML-DSA-65 + ratchet is strictly ahead, and adopting RNS links
  means downgrading to get anonymity. Every line touched lands in the
  adversarial-review-gated `core/src/{crypto,transport,routing,privacy}`.
  Replacing libp2p is a months-long re-architecture that relitigates settled
  decisions.
- Philosophy alignment: High on values, zero on strategy.
- Verdict: **REJECT as adoption.** A different layer's answer to a different
  layer's problem.

### 1.2 Destination addressing (hash-of-identity, not IP) and portable identity

- Pros: RNS's core insight -- your address is a hash of who you are, not where
  you are; messages route to a person, and the network discovers where they
  currently are -- is the clean articulation of what SCMessenger's identity/
  layer already does. The "nomadism" framing (change radios, keep the
  identity) is exactly SCM's multiport ladder and transport racing story.
- Cons: Nothing to adopt. SCMessenger already binds addressing to Ed25519
  identity keys, not endpoints, and additionally binds a per-device registry
  handle (`device_id`, see Correction 1).
- Verdict: **CONVERGENT.** Worth stealing only the prose for the README --
  "routes to a person, not a place" explains SCM better than SCM currently
  explains itself.

### 1.3 The announce mechanism (signed, bandwidth-capped, hop-prioritized)

- Pros: One packet type merges discovery, public-key distribution, and path
  establishment. The governance is the valuable part: per-interface bandwidth
  caps (default 2%), dedup tables, hop-count-prioritized queues, random
  delays -- all designed for links where every byte costs battery and airtime.
  SCMessenger's ledger gossip over BLE has no such discipline today.
- Cons: SCMessenger deliberately chose invite/QR-seeded ledger sharing
  (V050-B1/B2 contract) over ambient announce flooding. Periodic re-announces
  are an ambient radio fingerprinting surface -- RNS mitigates with
  randomization, but a privacy analysis would be needed before borrowing the
  pattern, and SCM's threat model (farm, localized adversary) prefers seeded
  discovery. Also: announce propagation *is* the path table in RNS; SCM
  separates discovery (ledger) from routing (routing/ module), and that
  separation is load-bearing.
- Verdict: **BORROW THE GOVERNANCE, NOT THE MECHANISM.** Rate caps, dedup, and
  priority queues for ledger gossip over constrained transports -- a small,
  gated `core/src/transport/` change, post-tag.

### 1.4 Source-free packets and hop-by-hop paths

- Pros: The strongest privacy idea in RNS that SCMessenger lacks: no source
  addresses on any packet, no node knows the full path, transport nodes learn
  only the next hop. SCM's onion routing module exists but is parked and wired
  into no live path. A metadata-minimized envelope (v3) would close the gap
  between SCM's stated threat model ("expensive to determine who talks to
  whom") and its wire format, which today carries peer identifiers in libp2p
  framing and `device_id` in the identity envelope sender block.
- Cons: A real protocol change, not a port; the drift/ envelope is versioned
  and shared across four platform clients. Zero benefit to D1-D7.
- Verdict: **DEFER with intent.** Envelope v3 wish list lives in the companion
  plan; design inspiration is RNS, implementation is ours.

### 1.5 Links (3 packets / 297 bytes, initiator anonymity, FS by default)

- Pros: The economics are a superb target metric -- 297 bytes to establish a
  verified encrypted link, 0.45 bits/s keepalive. SCM's BLE session
  establishment should be able to state its own byte cost and keepalive budget
  the way RNS does. Initiator anonymity on session setup is a genuine property
  SCM lacks.
- Cons: SCM's hybrid session establishment is cryptographically stronger;
  adopting RNS links would trade PQ security for anonymity -- the wrong trade.
  The anonymity idea returns as a feature of the parked onion path, not as a
  transport swap.
- Verdict: **ADOPT THE METRIC, NOT THE MECHANISM.** Publish "bytes to
  establish a session" and "idle keepalive cost" as doc metrics.

### 1.6 Opportunistic + propagated dual delivery (LXMF's two modes)

- Pros: LXMF delivers opportunistically when the recipient is reachable and
  via propagation nodes when not -- precisely SCM's outbox flush + relay
  custody design, independently invented.
- Verdict: **CONVERGENT.** No action.

### 1.7 Propagation nodes (peering, syncing, encrypted distributed message store)

- Pros: The highest-value design in RNS for SCMessenger *at the protocol
  level*. LXMF propagation nodes peer with each other and synchronize
  automatically, so a user retrieves pending messages from any available node.
  Per Correction 2, SCM's doctrine already says every node holds custody; the
  missing piece is a peering/sync protocol between custody stores, so a
  message parked on one node is reachable through another. RNS/LXMF gives a
  working design to study instead of inventing one.
- Cons: Sync means conflict resolution, retention policy, and storage caps on
  phones; more than the three-node north star needs for v0.4.0; custody work
  is `core/src/store/` + `core/src/drift/` gated.
- Philosophy alignment: Highest of any element -- propagation as an elected
  node behavior peering into a federated store is the same "custody is a
  behavior, not a role" logic as SCM's doctrine.
- Verdict: **ADOPT AS DESIGN, POST-TAG.** Design note first; first action of
  the note is reading the LXMF router source. See companion plan.

### 1.8 Resources (reliable arbitrary-size transfer)

- Pros: The model to beat for the farm's real transfer needs -- APK sharing,
  file/attachment sync over flaky BLE. Sequencing + progress + checksums done
  once, at the right layer.
- Cons: libp2p already gives streams/request-response; drift/ has framing +
  lz4. Borrow the UX contract, not the code.
- Verdict: **CONVERGENT / borrow UX only.**

### 1.9 Paper messages (encrypted LXMF as QR / URI / analog transport)

- Pros: Cheap, demoable, farm-aligned: a message encoded as a QR that can be
  scanned and re-imported is the dead-zone handoff story. SCM already has QR
  machinery (invites, APK sharing); an envelope-compaction pass plus
  `scm qr-export` / import pair is a small feature that demonstrates the
  "any medium, including paper" philosophy. RNS notes QR/URI encoding as a
  first-class LXMF capability -- credibility by precedent.
- Cons: QR payload ceilings (~2-3 KB) constrain message size; needs envelope
  compaction; pure feature-polish, so it ranks below D1-D7 by the ship plan's
  own rule.
- Verdict: **ADOPT POST-TAG, v1.0 demo candidate.**

### 1.10 LoRa / RNode / medium agnosticism down to 5 bits/s

- Pros: The 28-acre farm with far-field dead zones is the exact use case. RNS
  proves the whole path (RNode open firmware, packet radio, serial). A
  serial-pipe transport in SCM would be the thinnest possible bridge to that
  world.
- Cons: New transport = hardware procurement + the `core/src/transport/`
  adversarial gate + real engineering. BLE sneakernet already covers the
  tractor case in v1.0.0 scope. Post-v0.4.0 by years, not months.
- Verdict: **DEFER to v1.1+ evaluation; keep the serial-pipe idea in the
  pocket.**

### 1.11 Network identity (signed admin domain for interface discovery)

- Pros: Elegant for the farm: a signed "farm network" key gating the WiFi
  backbone, encrypted discovery announces, and later signed spam/blocklist
  distribution mapping naturally onto SCM's abuse/ module.
- Cons: Introduces a privileged administrative key into a network whose
  doctrine says no privileged nodes exist. SCM's invite-seeded trust is
  simpler and already decided.
- Verdict: **REJECT for now;** revisit only if the farm operator demands
  cryptographic gate-keeping on the LAN.

### 1.12 Ecosystem posture: tools, docs, "the implementation IS the spec," honesty

- Pros: Three things to internalize. (a) Ship diagnostics with the stack --
  `rnstatus`/`rnpath`/`rnprobe` equivalents (`scm status`, `scm probe <peer>`)
  would have caught the five-node-gate failures sooner and cost almost
  nothing; SCM's CLI is already a full node. (b) The honesty posture -- RNS
  states un-audited status and limitations bluntly, exactly the register SCM's
  README already hits; keep it. (c) "The reference implementation IS the spec"
  -- SCM's architecture already agrees (Rust core is the spec; UniFFI
  bindings are generated, never written). This is an argument for staying the
  course and against writing parallel protocol documents that drift from code.
- Cons: One caution and one structural difference. Caution: RNS's aggressive
  anti-derivative stance means SCMessenger must never claim Reticulum interop
  or describe itself as RNS-like without proof -- "inspired by" only. The
  difference: RNS can make "code is the spec" work because one implementation
  exists; SCM has four platform clients over one core, so the equivalent
  invariant is "bindings are generated and CI-verified," which already holds.
- Verdict: **ADOPT THE POSTURE** (diagnostics tools post-tag, honesty forever).

### 1.13 Crypto deep-compare

| Dimension | Reticulum (per its docs) | SCMessenger (per its docs) |
|---|---|---|
| KEX / signatures | X25519 / Ed25519 | X25519 + ML-KEM-768 / Ed25519 + ML-DSA-65 (hybrid) |
| Symmetric | AES-256-CBC + HMAC-SHA256 per packet | ChaCha20-Poly1305, Double Ratchet FS |
| Post-quantum | None | Hybrid, enforced-in-progress |
| External audit | None (LXMF says so itself) | None (README says so itself) |
| Formal methods | None claimed | Kani proofs, adversarial review gates |

SCMessenger is ahead on every row, tied on honesty. The AES-CBC-then-HMAC
choice is defensible but ChaCha20-Poly1305 is the simpler,
constant-time-friendlier construction. Conclusion: **any borrowing must be
design-only; no RNS crypto touches the core.** Both systems share the deepest
philosophical commitment -- encryption is not a feature, it is the routing
substrate. SCM should say this out loud in its docs the way the Zen of
Reticulum does ("encryption is gravity" is the best one-line threat-model
statement in the space).

---

## 2. What NOT to adopt, in one paragraph

Do not adopt the Python stack, do not chase wire compatibility (the
Brandolini policing makes an unfinished Rust port a reputational trap, and the
two projects solve different layers), do not swap libp2p for RNS paths, do not
downgrade crypto to gain anonymity, and do not introduce Network Identities or
anonymous forwarders -- the latter is expressly forbidden by the AGENTS.md
doctrine ("no anonymous packet forwarder exists or may be introduced"; the
2026-07-11 investigation already sized identity-optional relaying as a big
design item requiring operator escalation, and RNS's blind-forwarding pattern
is the reference *if* that door ever opens, not a reason to open it).

---

## 3. Borrow list (summary; authoritative sequencing in the companion plan)

| When | Item | Source element | Cost |
|---|---|---|---|
| Now (docs only) | "Routes to a person, not a place" + "encryption is the substrate, not a feature" in README/threat-model prose | 1.2, 1.13 | Hours |
| Post-tag (v0.5.0) | `scm status` / `scm probe` diagnostics commands | 1.12 | Days |
| Post-tag (v0.5.0) | Gossip governance: rate caps, dedup, hop-priority for ledger sync over constrained transports | 1.3 | Small, gated |
| Post-tag (v0.5.0) | Publish session-establishment byte budget + idle keepalive cost as doc metrics | 1.5 | Hours |
| Post-tag (v0.5.0/v1.0) | Custody-store peering/sync design note, studying LXMF's protocol before inventing | 1.7 | Design note first |
| v1.0 demo | QR "paper message" export/import as the dead-zone sneakernet story | 1.9 | Small feature |
| v1.1+ | Envelope v3 metadata minimization (no source addressing; on-wire device_id observation; sender metadata review); LoRa serial-pipe transport evaluation; initiator anonymity via the parked onion path | 1.4, 1.10, 1.5 | Research lane |

---

## 4. Meta-lessons (the part that ages well)

1. **Evidence discipline is the shared religion.** RNS polices fake
   implementations by demanding working artifacts; SCM's ship plan scores runs
   on receiver-side decrypt + durable history, "NOT transport ACKs." Same
   principle, same conclusion: claims are not evidence. Worth stating in
   `CONTRIBUTING.md` as a project value, because SCM occupies the same niche
   RNS's LLM-fake warning describes.
2. **Scarcity as a design teacher.** The farm's BLE links are SCM's "5 bits
   per second" moments. Publishing per-message and per-session byte budgets
   (as RNS does for links) would force the efficiency the farm actually needs
   and would differentiate SCM's docs from every other messenger's.
3. **Store-and-forward as primary mode, not fallback.** RNS's
   asynchronous-by-default stance matches the farm's reality (messages arrive
   when they arrive). SCM's UX should stop treating custody delivery as
   degraded service -- "placed in the network's keeping" is a feature to
   surface in the UI, not hide.

---

## 5. Confidence and limits

Per the no-silent-truncation rule: this audit rests on the Reticulum manual
chapter "Understanding Reticulum," the RNS and LXMF master-branch READMEs, and
the Zen of Reticulum (read up to section V; the fetched text truncated there).
No RNS or LXMF source code was read, so implementation-quality judgments
(threading, memory, actual bandwidth behavior) are out of scope; any adoption
decision on the custody-sync design must begin by reading `lxmf`'s router
source. On the SCM side the auditor read `README.md`,
`docs/CLAUDE_REFERENCE.md`, `SHIP_PLAN.md` (lines 1-150 of 539),
`docs/V1_KNOWN_LIMITATIONS.md`, `HANDOFF/todo/_QUEUE.md`, and the code files
named in "Verified corrections"; the deeper HANDOFF plan files
(FARM_FINAL_PLAN, V1_0_0_EXECUTION_PLAN) are cited by the queue but were not
opened this session, so farm-need statements inherit the queue's summary of
them. All audit fetches and searches were run 2026-09-21.
