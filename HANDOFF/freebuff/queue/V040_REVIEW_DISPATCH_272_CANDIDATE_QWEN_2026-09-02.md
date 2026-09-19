# V040 REVIEW DISPATCH -- #272 V040 architecture candidate (adversarial, qwen free lane)

Status: **TRIAGED 2026-09-02 -- verdict filed + fix commit fc0f5ae0 pushed.** Verdict: REQUEST_CHANGES (6 findings; 2 verified REAL and fixed, 1 fixed, 3 not-applicable with evidence). File: HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_2026-09-02.md. Inbox: V040_272_REVIEW_TRIAGED_2026-09-02.md.
Priority: P0 -- this SHA (a759e0c7) is the three-node candidate; Rule-8 requires a NON-AUTHOR adversarial APPROVE before merge, and nothing may merge until this review is on file.
Lane: Qwen free -- qwen3.8-2.4t-a95b (ledger-confirmed 100%: 1,000,000 remaining, 1M-context November reserve bucket, docs/QWEN_QUOTA_LEDGER.md 2026-08-31 snapshot; same qwen3.8 generation as qwen3.8-max-0902, which the operator blessed for the #267 hardest-assignment review). Full PR diff + architecture doc supplied as context.
Target: PR #272 cto/v040-candidate-2026-09-02 (7 files: core/src/transport/observation.rs, core/src/transport/swarm.rs, core/src/routing/local.rs, core/src/routing/optimized_engine.rs, core/src/iron_core.rs, cli/Cargo.toml, docs/ARCHITECTURE_SCOPE_V040.md)
Reviewer constraint: MUST NOT be the author. Independent pass only -- do not read the author's PR-body audit note until AFTER your verdict is drafted.

## Why this review exists
The PR makes AddressObserver the SINGLE owner of which observed address may be advertised: observations are admitted only for ports the node currently listens on (empty allowlist fails closed), listener closure and individual address expiry retract their ports, and libp2p's confirmed external-address registry is kept in lockstep with the observer's consensus primary (sync_external_address). Local peer selection was unified onto one deterministic comparator, and dead transport distinctions were removed from iron_core.rs.

Your job: attack the INVARIANT -- "nothing ephemeral or stale may ever be advertised as external on any platform where listen ports exist, and the observer's state can never diverge from the advertised set."

## Attack checklist (attack each; reject hearsay claims)
1. AddressObserver (observation.rs): is the listen-port allowlist the ONLY admission gate? Can an observation on a non-listen port survive (replacement path, recalculate path, ordering)? Does primary_external_address()/external_addresses() ever return an address whose port is not currently listenable? Empty-listener fail-closed?
2. sync_external_address (swarm.rs): can it promote an address the observer would reject? Can it DELETE a legitimately-confirmed address (other add_external_address callers, autonat, relay, other subsystems)? Is the /ip4|/ip6/.../tcp/ reconstruction exact (no port/ip loss)?
3. Listener lifecycle: ListenerClosed and ExpiredListenAddr retract exactly the right ports -- can a closed/expired port stay in bound_addresses or in the allowlist via any path (re-bind, multiple listeners same port, wasm loop mirror)?
4. Promotion sites (2): each records the observation BEFORE syncing; can consensus change between record and sync in a way that leaves a stale address published? Any OTHER path that calls add_external_address?
5. Local routing (local.rs, optimized_engine.rs): is the unified comparator deterministic (no HashMap iteration order, no unstable tie-breaks)? Did removing TransportType::Circuit / endpoint_transport_string / routing_peer_seen break any reachable call path (grep call sites, not just compile)?
6. Empty-input / ordering: empty listener set at startup, observer with zero observations, concurrent set_listen_ports + record_observation ordering.

## Method
- `gh pr diff 272` is supplied in context (tmp/rev272.diff); docs/ARCHITECTURE_SCOPE_V040.md is the ownership contract being committed.
- Worktree on disk if you want the full tree: `git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate diff origin/main` (branch checked out at a759e0c7).
- Do NOT edit code -- review-only. Fixes, if any, go back to the lane author as findings.
- Rule-8: touches core/src/{transport,routing}. Your APPROVE must be explicitly recorded, non-author, and state the attacks you actually tried with evidence (command + line).

## Output contract
1. Verdict file to HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_2026-09-02.md (attack tried -> verdict -> evidence, then a plain APPROVE / REQUEST_CHANGES + findings list, max 80 lines).
2. Reply in HANDOFF/freebuff/inbox/ with a 3-line note (verdict, file path, anything the author must fix).

Do NOT pad. If nothing holds, list the attacks tried and say so plainly. Do NOT post to the PR -- the lane posts only after the operator rules on attribution.
