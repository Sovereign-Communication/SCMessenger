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

### Resolution bod-40a0e31f [REJECTED]
- **Timestamp**: 2026-09-13T15:45:41.747482+00:00
- **Verdict**: `REJECTED_DISSENT`
- **Cost**: $0.001533 (Ceiling: $0.10)
- **Summary**: Dissent detected among panel: 4 APPROVE, 1 REJECT. 100% unanimity required.
- **Panel Voting** (4/5 APPROVE):
  - `openai/gpt-4o-mini`: **REJECT** (Score: 0.60) - _While Delta 1 aligns with the doctrine, Delta 2 introduces new violations by expanding Kotlin's cryptographic operations, which go against the established principle of utilizing Rust as the sole cryptographic authority._
  - `meta-llama/llama-3.1-8b-instruct`: **APPROVE** (Score: 0.90) - _The proposed changes are minor and do not introduce any new exceptions to the doctrine. The Delta 1 change is a refinement of the existing probationary window, and the Delta 2 change is an extension of an existing Kotlin implementation that does not weaken cryptographic sovereignty._
  - `ibm-granite/granite-4.0-h-micro`: **APPROVE** (Score: 0.95) - _The proposal adheres to the SCMessenger Repo Philosophy and Canonical Doctrine. Delta 1 makes a minor adjustment to the reliability calculation in mesh_routing.rs, which does not introduce any doctrine violations. It improves the reliability threshold based on successful deliveries and total relay attempts, which is consistent with the doctrine's emphasis on cryptographic integrity and node parity. Delta 2 expands an existing Kotlin BigInteger implementation for Ed25519 point validation in android PeerIdValidator.kt. This expansion of an existing codebase does not introduce any new cryptographic operations or weaken cryptographic sovereignty, as it remains advisory validation of a public hex string without handling key material or performing signature verification. The proposed changes are aligned with the doctrine's emphasis on hygiene, verification, and safety._
  - `deepseek/deepseek-chat`: **APPROVE** (Score: 0.95) - _Delta 1 strictly adheres to doctrine principles, while Delta 2 is a minor extension of pre-existing code that does not weaken core cryptographic sovereignty, warranting approval with a remediation ticket._
  - `inclusionai/ling-3.0-flash`: **APPROVE** (Score: 0.85) - _Delta 1 is fully doctrinally compliant with no new central dependencies or protocol deviations. Delta 2 extends pre-existing Kotlin BigInteger debt already on main rather than introducing a new doctrine violation, and introduces no crypto sovereignty weakening since Rust remains the sole authority; the proposed P1 remediation ticket to relocate the check to UniFFI before the tag provides a clear remediation path, making the whole PR #282 acceptable to merge with that ticket recorded._
- **Judge Model**: `openai/gpt-4o-mini` (Agreed: False)
- **Proposal Text**:
  > DOUBLE-BLIND RULE-8 RE-REVIEW (operator ruling 2026-09-13): PR #282 merge gate re-adjudication.
  > 
  > PRIOR ART: resolution bod-4df59504 APPROVED the mesh_routing.rs probationary
  > window from a summary-level proposal. This re-review supplies the actual delta
  > and one additional change the prior proposal did not mention.
  > 
  > DELTA 1 (rule-8 gated file, core/src/transport/mesh_routing.rs, commit b65bc4d7):
  > In RelayReputation::calculate_score(), the reliability boundary changes from
  >     self.is_reliable = self.score >= 50.0;
  > to
  >     self.is_reliable = self.score >= 50.0
  >         || (self.stats.successful_deliveries == 0 && self.stats.messages_relayed < 3);
  > 
  > Semantics verified against the full file: record_relay_attempt() increments
  > messages_relayed on EVERY relay attempt (success or failure), and
  > successful_deliveries only on success. Therefore the probation clause is
  > true only while a relay has fewer than 3 total attempts and zero successes;
  > after the 3rd failed attempt the clause is false forever and the peer is
  > disqualified exactly as before. A probationary relay still appears in
  > ranked_routes() only via the is_reliable filter, ranked below any
  > recipient-recency signal; direct paths are unaffected. No crypto, no
  > privacy, no storage-surface change. New unit test
  > test_reputation_probationary_period covers both the transient-tolerance and
  > 3-strike-disqualification sides.
  > 
  > ADVERSARIAL QUESTIONS FOR DELTA 1:
  > (a) Does any path keep a dead relay permanently reliable? (Answer expected:
  > no - the 3-attempt clause is computed from counters that only grow.)
  > (b) Does probation let an unverified peer bypass any doctrine bound
  > (nodes-not-relays, no anonymous forwarders, IronCore authority)?
  > (c) Is the liveness motivation (mobile handoffs must not permanently
  > blackball a relay after one failure) consistent with the doctrine's
  > eventual-delivery and node-parity pillars?
  > 
  > DELTA 2 (android only, NOT rule-8 perimeter, but inside the Board's doctrine
  > boundary - commit d35d3883): android PeerIdValidator.kt gains
  > isValidEd25519Point(): a Kotlin BigInteger implementation of Ed25519 curve
  > point decompression and Legendre symbol arithmetic, used by
  > normalizePublicKeyHex() to reject 64-hex strings that are not real keys.
  > 
  > PRIOR BOARD RULING bod-dd336324 states: "Kotlin adapters must not perform
  > BigInteger curve operations and must consume PeerIdValidator from core via
  > UniFFI." However, the equivalent Kotlin BigInteger math ALREADY EXISTS on
  > origin/main today (PeerIdValidator.kt, 105 lines, 7 BigInteger references,
  > shipped in the #281 parity merge); d35d3883 expands the existing file from
  > 105 to 139 lines. The Rust core already exposes is_valid_public_key() in
  > core/src/identity/keys.rs and the UniFFI bridge exists.
  > 
  > QUESTIONS FOR DELTA 2 (answer explicitly):
  > (d) Is EXPANDING an existing Kotlin BigInteger curve-implementation a NEW
  > doctrine violation requiring remediation before merge, or is it a
  > pre-existing debt that d35d3883 merely extends, acceptable to merge with a
  > recorded remediation ticket to move the check behind UniFFI?
  > (e) Does the expansion weaken cryptographic sovereignty in a way that
  > blocks the v0.4.0 tag, given Rust remains the sole signer/decryptor and the
  > Kotlin check is advisory validation of a public hex string (reject/accept
  > only, no key material handling, no signature verification, no ECDH)?
  > 
  > REMEDY OPTIONS IF (d) IS A VIOLATION: (1) strip isValidEd25519Point from
  > d35d3883 before merge and keep the rest of PR #282; (2) merge and file a
  > P1 remediation ticket to relocate the check to UniFFI before the tag;
  > (3) reject PR #282 entirely.
  > 
  > VOTE on the WHOLE PR #282 merge (deltas 1+2 together) under the doctrine.

### Resolution bod-2522bb06 [DEFERRED]
- **Timestamp**: 2026-09-13T15:57:16.046998+00:00
- **Tier**: heavy
- **Verdict**: `DEFERRED_PANEL_SHORTFALL`
- **Cost**: $0.030791 (Ceiling: $0.10)
- **Summary**: Only 4/5 models submitted valid votes. Fails closed.
- **Panel Voting** (2/5 APPROVE):
  - `openai/gpt-4.1`: **APPROVE** (Score: 0.97) - _Delta 1 is strictly within doctrine and fixes a liveness/performance edge case without introducing centralization or privacy risk. Delta 2 expands a pre-existing doctrine debt but does not worsen sovereignty risk, with remediation planned before the v0.4.0 tag. The proposal aligns with core principles and advances the project with strong compliance._
  - `deepseek/deepseek-v3.2`: **REJECT** (Score: 0.30) - _While Delta 1 supports doctrine principles of eventual delivery and node parity, Delta 2's expansion of Kotlin BigInteger curve operations violates the requirement that Rust core is the sole cryptographic authority, creating unacceptable sovereignty risk even as pre-existing technical debt._
  - `google/gemini-2.5-pro`: **UNKNOWN** (Score: 0.00) - _Truncated by max-tokens (Rule 15 fail-closed)_
  - `deepseek/deepseek-chat`: **APPROVE** (Score: 0.95) - _The proposal upholds core doctrine while tolerating minor pre-existing technical debt, with DELTA 1 improving mesh reliability and DELTA 2 maintaining existing validation patterns until full UniFFI migration._
- **Judge Model**: `openai/gpt-5` (Agreed: False)
- **Proposal Text**:
  > DOUBLE-BLIND RULE-8 RE-REVIEW (operator ruling 2026-09-13): PR #282 merge gate re-adjudication.
  > 
  > PRIOR ART: resolution bod-4df59504 APPROVED the mesh_routing.rs probationary
  > window from a summary-level proposal. This re-review supplies the actual delta
  > and one additional change the prior proposal did not mention.
  > 
  > DELTA 1 (rule-8 gated file, core/src/transport/mesh_routing.rs, commit b65bc4d7):
  > In RelayReputation::calculate_score(), the reliability boundary changes from
  >     self.is_reliable = self.score >= 50.0;
  > to
  >     self.is_reliable = self.score >= 50.0
  >         || (self.stats.successful_deliveries == 0 && self.stats.messages_relayed < 3);
  > 
  > Semantics verified against the full file: record_relay_attempt() increments
  > messages_relayed on EVERY relay attempt (success or failure), and
  > successful_deliveries only on success. Therefore the probation clause is
  > true only while a relay has fewer than 3 total attempts and zero successes;
  > after the 3rd failed attempt the clause is false forever and the peer is
  > disqualified exactly as before. A probationary relay still appears in
  > ranked_routes() only via the is_reliable filter, ranked below any
  > recipient-recency signal; direct paths are unaffected. No crypto, no
  > privacy, no storage-surface change. New unit test
  > test_reputation_probationary_period covers both the transient-tolerance and
  > 3-strike-disqualification sides.
  > 
  > ADVERSARIAL QUESTIONS FOR DELTA 1:
  > (a) Does any path keep a dead relay permanently reliable? (Answer expected:
  > no - the 3-attempt clause is computed from counters that only grow.)
  > (b) Does probation let an unverified peer bypass any doctrine bound
  > (nodes-not-relays, no anonymous forwarders, IronCore authority)?
  > (c) Is the liveness motivation (mobile handoffs must not permanently
  > blackball a relay after one failure) consistent with the doctrine's
  > eventual-delivery and node-parity pillars?
  > 
  > DELTA 2 (android only, NOT rule-8 perimeter, but inside the Board's doctrine
  > boundary - commit d35d3883): android PeerIdValidator.kt gains
  > isValidEd25519Point(): a Kotlin BigInteger implementation of Ed25519 curve
  > point decompression and Legendre symbol arithmetic, used by
  > normalizePublicKeyHex() to reject 64-hex strings that are not real keys.
  > 
  > PRIOR BOARD RULING bod-dd336324 states: "Kotlin adapters must not perform
  > BigInteger curve operations and must consume PeerIdValidator from core via
  > UniFFI." However, the equivalent Kotlin BigInteger math ALREADY EXISTS on
  > origin/main today (PeerIdValidator.kt, 105 lines, 7 BigInteger references,
  > shipped in the #281 parity merge); d35d3883 expands the existing file from
  > 105 to 139 lines. The Rust core already exposes is_valid_public_key() in
  > core/src/identity/keys.rs and the UniFFI bridge exists.
  > 
  > QUESTIONS FOR DELTA 2 (answer explicitly):
  > (d) Is EXPANDING an existing Kotlin BigInteger curve-implementation a NEW
  > doctrine violation requiring remediation before merge, or is it a
  > pre-existing debt that d35d3883 merely extends, acceptable to merge with a
  > recorded remediation ticket to move the check behind UniFFI?
  > (e) Does the expansion weaken cryptographic sovereignty in a way that
  > blocks the v0.4.0 tag, given Rust remains the sole signer/decryptor and the
  > Kotlin check is advisory validation of a public hex string (reject/accept
  > only, no key material handling, no signature verification, no ECDH)?
  > 
  > REMEDY OPTIONS IF (d) IS A VIOLATION: (1) strip isValidEd25519Point from
  > d35d3883 before merge and keep the rest of PR #282; (2) merge and file a
  > P1 remediation ticket to relocate the check to UniFFI before the tag;
  > (3) reject PR #282 entirely.
  > 
  > VOTE on the WHOLE PR #282 merge (deltas 1+2 together) under the doctrine.
