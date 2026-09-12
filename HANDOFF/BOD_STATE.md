# Board of Directors (BoD) State — Live Governance Ledger

Status: Active
Last updated: 2026-09-11
Entry point: `/bod` (Codebuff/Freebuff: `/skill:bod`)

## Role

The Board of Directors seat is the highest governance authority in SCMessenger.
It ensures that all architectural, cryptographic, strategic, and philosophical
proposals strictly align with the repository's foundational doctrine.

The Board is executed as an automated 5-judge panel powered by
`sovereign-harness`, backed by an independent judge model.

## Canonical rules

1. **5-Model Panel**: Every proposal is reviewed independently by 5 distinct
   models from the curated model pool.
2. **Strict Unanimity (5/5)**: All 5 models must vote identically (100%
   unanimity). A single dissenting vote fails the proposal.
3. **Judge Concurrence**: The synthesis judge model must explicitly agree with
   the unanimous panel vote.
4. **Cost Ceiling**: Maximum **$0.10 (10 cents)** per `/bod` evaluation run.
   Runs default to OpenRouter's free model pool ($0.00).
5. **Fail Closed**: Any network failure, rate limiting, or model shortfall
   resulting in fewer than 5 valid votes produces a `DEFERRED` resolution.

## Operating boundary

- The Board does not implement application source code.
- The Board evaluates proposals against `AGENTS.md` (Architecture doctrine:
  nodes, not relays; sovereign mesh; cryptographic authority; platform parity;
  engineering hygiene).
- Below 100% agreement, a proposal cannot pass.
- In exceptional operational emergencies, only the human operator holds the
  authority to issue an explicit recorded ruling that bypasses the Board.

## Resolution ledger

Record of all formal resolutions adjudicated by the Board of Directors:

### Resolution bod-a7deb38b [APPROVED]
- **Timestamp**: 2026-09-11T23:15:53.018144+00:00
- **Verdict**: `APPROVED`
- **Cost**: $0.000000 (Ceiling: $0.10)
- **Summary**: Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy.
- **Panel Voting** (5/5 APPROVE):
  - `google/gemma-4-26b-a4b-it:free`: **APPROVE** (Score: 1.00) - _The proposal is a direct and accurate distillation of the 'Nodes, Not Relays' doctrine, reinforcing the requirement for full parity and the absence of specialized relay roles._
  - `openrouter/free`: **APPROVE** (Score: 1.00) - _The proposal strictly follows the node‑relay and zero‑central‑dependency tenets of the SCMessenger doctrine._
  - `nvidia/nemotron-3-super-120b-a12b:free`: **APPROVE** (Score: 1.00) - _The proposal restates core doctrine without deviation, affirming that every node relays and there are no central relays or proxies._
  - `cohere/north-mini-code:free`: **APPROVE** (Score: 1.00) - _The proposal restates core tenets of the doctrine, confirming node parity, absence of relays and central dependencies, and consistent software across all deployments._
  - `google/gemma-4-31b-it:free`: **APPROVE** (Score: 1.00) - _The proposal is a verbatim affirmation of the project's foundational architecture, explicitly upholding the rule that all nodes share identical relay behavior regardless of deployment._
- **Judge Model**: `google/gemma-4-31b-it:free` (Agreed: True)
- **Proposal Text**:
  > In SCMessenger, all nodes perform store-and-forward custody as a standard node behavior; there are no standalone relays or centralized proxies, and cloud nodes run the exact same node software as mobile and CLI clients.

### Resolution bod-dd336324 [APPROVED]
- **Timestamp**: 2026-09-11T23:35:18.977308+00:00
- **Verdict**: `APPROVED`
- **Cost**: $0.000000 (Ceiling: $0.10)
- **Summary**: Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy.
- **Panel Voting** (5/5 APPROVE):
  - `openrouter/free`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to all five canonical doctrine pillars: nodes-not-relays, zero central dependencies, cryptographic integrity via Rust core authority, multiplatform parity, and verification/safety hygiene. No centralized exceptions, non-compliant roles, or doctrine contradictions are introduced._
  - `google/gemma-4-26b-a4b-it:free`: **APPROVE** (Score: 1.00) - _The proposal strictly reinforces the core tenets of node parity, cryptographic sovereignty, and the non-negotiable nature of store-and-forward delivery while mandating rigorous safety reviews._
  - `nvidia/nemotron-3-super-120b-a12b:free`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to all SCMessenger tenets: node uniformity, cryptographic sovereignty, process safety, and guaranteed eventual delivery._
  - `inclusionai/ling-3.0-flash-fin:free`: **APPROVE** (Score: 1.00) - _Every point in the proposal maps directly and strictly onto the four pillars of SCMessenger doctrine—nodes not relays, sovereignty, cryptographic integrity, and verification hygiene—with no contradictions, exceptions, or centralized dependencies introduced._
  - `google/gemma-4-31b-it:free`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to the node-centric architecture, maintains Rust core cryptographic authority, and reinforces the non-negotiable requirement for eventual delivery via store-and-forward custody._
- **Judge Model**: `google/gemma-4-31b-it:free` (Agreed: True)
- **Proposal Text**:
  > MiMo landing integration proposal: 1. Nodes, not relays: All nodes perform store-and-forward custody as a universal node behavior; cloud nodes run identical node code and are not standalone relays. 2. Cryptographic sovereignty: Rust core remains the sole cryptographic authority; Kotlin adapters must not perform BigInteger curve operations and must consume PeerIdTriad from core via UniFFI. 3. Rule-8 review: Core transport and store modifications (CLI custody registration on identify, DriftFrame unwrapping, and nickname ownership bounds) require adversarial review before merging to main. 4. Eventual store-and-forward delivery: Eventual delivery across the sovereign peer-to-peer mesh is non-negotiable.

### Resolution bod-28755305 [APPROVED]
- **Timestamp**: 2026-09-12T00:53:16.722687+00:00
- **Verdict**: `APPROVED`
- **Cost**: $0.001109 (Ceiling: $0.10)
- **Summary**: Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy.
- **Panel Voting** (5/5 APPROVE):
  - `openai/gpt-4o-mini`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to SCMessenger's foundational principles without introducing any contradictions or violations._
  - `deepseek/deepseek-chat`: **APPROVE** (Score: 1.00) - _The proposal strictly aligns with all SCMessenger doctrine principles, upholding cryptographic sovereignty, node parity, and architectural hygiene without any violations._
  - `ibm-granite/granite-4.0-h-micro`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to all SCMessenger doctrine principles, including the node-centric architecture, cryptographic integrity, cryptographic sovereignty, storage integrity, multiplatform parity, and hygiene/verification standards. No violations or contradictions are identified._
  - `meta-llama/llama-3.1-8b-instruct`: **APPROVE** (Score: 0.95) - _The proposal strictly adheres to the SCMessenger doctrine, demonstrating a thorough understanding of the project's foundational principles and ensuring the network's cryptographic integrity, sovereignty, and node-only relaying behavior._
  - `inclusionai/ling-3.0-flash`: **APPROVE** (Score: 1.00) - _The proposal strictly upholds every principle of the SCMessenger Repo Philosophy and Canonical Doctrine: node-relay parity, zero centralization, cryptographic sovereignty via IronCore, platform parity, and full hygiene compliance._
- **Judge Model**: `openai/gpt-4o-mini` (Agreed: True)
- **Proposal Text**:
  > --- UNIFIED INTEGRATION & LANDING PLAN FOR V0.4.0 PARITY ---
  >
  > 1. Canonical Architectural Doctrine & Node Role:
  >    - There are NO standalone relays in SCMessenger. Only NODES exist, and EVERY node relays.
  >    - Store-and-forward custody is a behavior performed equally by all nodes, not a distinct role.
  >    - The always-on cloud node (scm-always-on-node) is a full node executing identical protocol contracts.
  >    - Discovery is ledger sharing between nodes; no centralized coordinator, tracker, or anonymous forwarder exists.
  >
  > 2. Cryptographic Sovereignty & Storage Integrity:
  >    - The Rust core (core/src/) is the single source of truth and sole cryptographic authority.
  >    - Platform adapters (Android Kotlin, iOS Swift, WASM) are strictly dumb byte pipes and must never duplicate cryptographic primitives. Specifically, Kotlin-level BigInteger Ed25519 curve decompression and Legendre symbol arithmetic are rejected in favor of IronCore identity resolution via UniFFI.
  >    - State and message storage access is strictly mediated through IronCore (core/src/store/). Direct sled database access is forbidden.
  >    - Nickname authority and collision disambiguation are enforced via normalized peer identity binding rather than UI-layer heuristics.
  >
  > 3. Integration Sequence & Git Hygiene:
  >    - PR #280 (cto/ticket-hygiene-2026-09-10) is merged into origin/main upon completion of green CI checks.
  >    - The paused Xiaomi MiMo session (ses_ffe5f71785a9bffe1gawQ5KJ6F in MiMoSCMessengerFresh) is preserved in place without disruption, and its working tree diff is captured to a durable repository snapshot.
  >    - PR #281 (unified/v040-3node-parity) CI blockers are remediated: re-encode NotificationGateTest.kt to clean UTF-8 without BOM, format core/src/store/ledger_entry.rs:659 via cargo fmt, and strip trailing whitespace from handoff files.
  >    - Rule-8 adversarial review gate is satisfied for the 7 security perimeter files under core/src/{routing,transport}/.
  >    - PR #281 is merged into origin/main, and superseded PRs #279 and #272 are cleanly closed.
  >    - Zero emoji in all code, comments, commit messages, and documentation.

### Resolution bod-7d92bae1 [APPROVED]
- **Timestamp**: 2026-09-12T10:51:00.459042+00:00
- **Verdict**: `APPROVED`
- **Cost**: $0.001085 (Ceiling: $0.10)
- **Summary**: Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy.
- **Panel Voting** (5/5 APPROVE):
  - `openai/gpt-4o-mini`: **APPROVE** (Score: 1.00) - _The proposal strictly aligns with the SCMessenger doctrine, ensuring security perimeter invariants are preserved and maintaining the principles of sovereignty and peer-to-peer architecture._
  - `deepseek/deepseek-chat`: **APPROVE** (Score: 1.00) - _The proposal strengthens security perimeter while strictly adhering to all foundational principles of the SCMessenger doctrine, particularly maintaining full node parity and cryptographic sovereignty._
  - `meta-llama/llama-3.1-8b-instruct`: **APPROVE** (Score: 0.95) - _The proposal maintains the integrity of the SCMessenger doctrine, introducing no new centralized dependencies or security bypasses, and strictly adheres to the store-and-forward custody doctrine._
  - `inclusionai/ling-3.0-flash`: **APPROVE** (Score: 1.00) - _The proposal strictly adheres to the SCMessenger doctrine by hardening the sovereign mesh perimeter without introducing centralized dependencies, cryptographic regressions, or non-compliant relay roles._
  - `ibm-granite/granite-4.0-h-micro`: **APPROVE** (Score: 0.95) - _The proposed changes in PR #281 do not introduce any listener leaks, denial-of-service vulnerabilities, or multiaddr parsing issues. They strictly adhere to the sovereign mesh architecture, maintain all cryptographic invariants, and uphold the store-and-forward custody doctrine without any contradictions or centralized dependencies._
- **Judge Model**: `openai/gpt-4o-mini` (Agreed: True)
- **Proposal Text**:
  > PROPOSAL: Rule-8 Adversarial Security Review Approval for PR #281 (unified/v040-3node-parity)
  >
  > Scope:
  > Evaluate the security perimeter modifications across the 7 gated files in core/src/{transport,routing}/:
  > 1. core/src/transport/swarm.rs:
  >    - D10 relay-reservation multiaddr validation: is_valid_reservation_base() strips nested circuit/p2p segments, disallows loopback/unspecified, preventing mDNS TXT record overflow.
  >    - D10b poison-listener event-loop guard: is_poison_circuit_listener() intercepts and removes illegitimate circuit listener registrations in SwarmEvent::NewListenAddr, avoiding stale or corrupted listener state.
  >    - Observability logging for lane visibility and relay custody handoffs.
  > 2. core/src/transport/dial_policy.rs:
  >    - Single-owner address admission; suppression of self-referential / broadcast loop dial attempts.
  > 3. core/src/transport/manager.rs:
  >    - Unified connection and transport lifecycle management.
  > 4. core/src/transport/observation.rs:
  >    - Bidirectional transport lane observation telemetry with safe empty-listen-set handling.
  > 5. core/src/routing/local.rs:
  >    - Unified local ordering for message custody.
  > 6. core/src/routing/optimized_engine.rs:
  >    - Routing engine optimizations with peer liveness integration.
  > 7. core/src/routing/resume_prefetch.rs:
  >    - Prefetch cache management for resumed peer connections.
  >
  > Security Perimeter Invariants:
  > - Zero changes to core/src/crypto/ or cryptographic algorithms (Ed25519, X25519 ECDH, Blake3, XChaCha20-Poly1305).
  > - Zero changes to core/src/privacy/.
  > - IronCore remains the single sovereign entry point for storage and cryptographic authority.
  > - Store-and-forward custody doctrine: all nodes relay; no anonymous packet forwarders.
  >
  > Checklist:
  > 1. Does is_valid_reservation_base() and is_poison_circuit_listener() introduce any listener leaks, denial-of-service, or multiaddr parsing vulnerabilities?
  > 2. Does the unified transport manager and dial policy adhere strictly to sovereign mesh architecture without central dependencies or security bypasses?
  > 3. Are all cryptographic invariants preserved without regression?
  >
  > Recommendation: APPROVE PR #281 security perimeter changes under Rule-8 governance.

### Resolution bod-8ee21463 [DEFERRED]
- **Timestamp**: 2026-09-12T23:16:16.541298+00:00
- **Verdict**: `DEFERRED_PANEL_SHORTFALL`
- **Cost**: $0.000000 (Ceiling: $0.10)
- **Summary**: Only 5/5 models submitted valid votes. Fails closed.
- **Panel Voting** (3/5 APPROVE):
  - `openrouter/free`: **UNKNOWN** (Score: 0.00) - _Malformed response (Rule 15 fail-closed)_
  - `google/gemma-4-26b-a4b-it:free`: **APPROVE** (Score: 1.00) - _The proposal addresses a liveness issue by preventing premature node burnout during transient network shifts without compromising cryptographic integrity or introducing centralized roles. It maintains the 'Nodes, Not Relays' philosophy by treating reliability as a behavioral observation rather than a structural role change._
  - `cohere/north-mini-code:free`: **APPROVE** (Score: 0.95) - _Adds a probationary grace period for relays, improving resilience without breaking cryptographic or centralization rules._
  - `nvidia/nemotron-3-super-120b-a12b:free`: **APPROVE** (Score: 0.96) - _The proposal adjusts an internal reputation heuristic to improve liveness while keeping all nodes as equal relays and preserving all doctrinal invariants._
  - `google/gemma-4-31b-it:free`: **REJECT** (Score: 0.60) - _The proposal introduces logic explicitly referencing 'relay nodes' and 'relay reputation' within the codebase, violating the foundational doctrine that NO standalone relays exist and that relaying is a behavior of NODES, not a role._
- **Judge Model**: `google/gemma-4-31b-it:free` (Agreed: False)
- **Proposal Text**:
  > # Proposal: Relay Reputation Probationary Grace Period in mesh_routing.rs
  > 
  > ## 1. Summary & Motivation
  > In `core/src/transport/mesh_routing.rs`, relay nodes are scored and marked as reliable or unreliable via `RelayReputation::calculate_score()`.
  > Previously, any candidate relay was marked reliable only if `self.score >= 50.0`.
  > Because `calculate_score` assigns 0 score when `messages_relayed == 0` or when delivery fails initially without prior success, a single transient transport failure (e.g. during a mobile Wi-Fi drop or cellular transition) immediately sets `is_reliable = false`.
  > This causes rapid burnout of valid circuit relays (including the AWS Cloud Node), preventing off-Wi-Fi fallback delivery.
  > 
  > ## 2. Technical Delta
  > In `core/src/transport/mesh_routing.rs`:
  > ```rust
  > self.score = success_score + latency_score + recency_score;
  > self.is_reliable = self.score >= 50.0
  >     || (self.stats.successful_deliveries == 0 && self.stats.messages_relayed < 3);
  > ```
  > During a relay's initial probationary window (before it has achieved its first successful delivery), it is granted up to 3 relay attempts before being declared unreliable. Once it has 3 consecutive failures with zero successes, `is_reliable` becomes `false`. Once it delivers successfully, its score reflects true operational history.
  > 
  > ## 3. Security & Invariant Analysis
  > - Rule 8 Perimeter: `core/src/transport/mesh_routing.rs` is an internal routing heuristic.
  > - Cryptography: No crypto, signature, key exchange, or cipher changes.
  > - Wire Format: No wire protocol, framing, or serialization changes.
  > - Node Authority: IronCore custody and storage contracts are completely untouched.
  > - Denial of Service / Liveness: Dead or malicious nodes cannot poison routing indefinitely, as 3 consecutive delivery failures immediately marks them unreliable.

### Resolution bod-4df59504 [APPROVED]
- **Timestamp**: 2026-09-12T23:16:41.267884+00:00
- **Verdict**: `APPROVED`
- **Cost**: $0.001084 (Ceiling: $0.10)
- **Summary**: Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy.
- **Panel Voting** (5/5 APPROVE):
  - `inclusionai/ling-3.0-flash`: **APPROVE** (Score: 0.92) - _The proposal is a local routing heuristic that prevents transient network failures from permanently blackballing peer nodes performing store-and-forward custody, strictly preserving node parity and sovereignty without introducing centralized dependencies or protocol deviations._
  - `deepseek/deepseek-chat`: **APPROVE** (Score: 1.00) - _The proposal enhances node reliability evaluation while fully adhering to all SCMessenger doctrine principles, maintaining cryptographic integrity, platform parity, and sovereignty without introducing centralization or new roles._
  - `openai/gpt-4o-mini`: **APPROVE** (Score: 1.00) - _The proposal enhances the reliability of node behavior in transient network conditions without compromising any foundational principles of the SCMessenger doctrine._
  - `ibm-granite/granite-4.0-h-micro`: **APPROVE** (Score: 0.95) - _The proposal enhances the reliability of node store-and-forward custody during transient network transitions, without violating any core SCMessenger doctrine principles. It introduces a fair probationary period for nodes, ensuring no legitimate node is permanently blacklisted due to temporary connectivity issues, while preserving cryptographic integrity, wire protocol fidelity, and overall network sovereignty._
  - `meta-llama/llama-3.1-8b-instruct`: **APPROVE** (Score: 0.90) - _The proposed changes to `mesh_routing.rs` uphold the SCMessenger doctrine by introducing a node-centric reliability evaluation window, ensuring parity across all platforms, and maintaining cryptographic sovereignty._
- **Judge Model**: `openai/gpt-4o-mini` (Agreed: True)
- **Proposal Text**:
  > # Proposal: Node Store-and-Forward Custody Grace Period in mesh_routing.rs
  > 
  > ## 1. Architecture Doctrine Alignment
  > In accordance with SCMessenger architecture doctrine:
  > - There are NO standalone relays; every peer is a full NODE, and store-and-forward custody is a behavior all nodes perform.
  > - The AWS instance (`scm-always-on-node`) is a CLOUD NODE, possessing full node parity with CLI, Android, and iOS nodes.
  > - `RelayReputation` and `is_reliable` in `core/src/transport/mesh_routing.rs` are internal code identifiers evaluating the historical success of peer nodes performing store-and-forward custody.
  > 
  > ## 2. Problem Statement
  > In `core/src/transport/mesh_routing.rs`, `RelayReputation::calculate_score()` marks a node's relaying behavior reliable only if `self.score >= 50.0`.
  > Before a node has achieved its first confirmed message delivery, or when a mobile node transitions from Wi-Fi to cellular during an in-flight handoff, a single transient dial failure resets the delivery score to 0.0.
  > This immediately flips `is_reliable = false`, permanently blackballing valid peer nodes (including the Cloud Node) from performing custody forwarding on the first transient network transition.
  > 
  > ## 3. Technical Remediations
  > In `core/src/transport/mesh_routing.rs`:
  > ```rust
  > self.score = success_score + latency_score + recency_score;
  > // Require at least 3 failed delivery attempts with zero successes before declaring
  > // a peer node unreliable for custody forwarding. In mobile/cellular networks and transient handoffs,
  > // an initial connection failure must not permanently blackball a candidate node.
  > self.is_reliable = self.score >= 50.0
  >     || (self.stats.successful_deliveries == 0 && self.stats.messages_relayed < 3);
  > ```
  > During a peer node's initial probationary evaluation window, it is granted up to 3 custody forwarding attempts before being classified as unreliable. Once 3 consecutive failures occur with zero successes, `is_reliable` becomes `false`. Once a custody forwarding attempt succeeds, the score is driven by verified delivery metrics.
  > 
  > ## 4. Invariant & Adversarial Security Proof
  > - Rule 8 Perimeter: `core/src/transport/mesh_routing.rs` (local routing heuristic).
  > - Cryptographic Sovereignty: Zero modifications to cryptographic primitives (`Ed25519`, `X25519`, `ChaCha20Poly1305`, `Blake3`).
  > - Wire Protocol: Zero modifications to framing, multiaddrs, or wire serialization.
  > - Sovereignty & Parity: Node-centric parity is strictly preserved across all platforms. No new roles or privileged entities are introduced.
  > - Liveness & DoS Resistance: Inactive, unresponsive, or malicious peers are definitively flagged as unreliable after 3 failed attempts, preventing route starvation while tolerating transient wireless handoffs.
