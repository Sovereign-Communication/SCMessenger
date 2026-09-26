# SESSION HANDOFF — 2026-07-13 — V1.0.0 Farm Backlog Orchestration

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active. Read this first if picking up from this session. Governing
plan: `HANDOFF/plans/FARM_FINAL_PLAN.md` (Fable-authored 2026-07-13,
supersedes nothing, refines the Farm Use Case directive). Live pick list:
`HANDOFF/todo/_QUEUE.md`.

## What landed this session (9 commits, all pushed to main)

1. `84e0651d`+ (prior session start) through `6d884f97` **fix(farm): outbox
   flush-on-connect retry (A1) + receipt round-trip (A2)** — the two
   farm-critical delivery-truth bugs. A1 closes
   `CRITICAL_OUTBOX_NEVER_FLUSHES`: `Outbox::flush_peer_messages` now
   actually drains and sends a peer's queued messages on reconnect, with
   exponential backoff and no message loss on transient send failure (a real
   bug Fusion Lite caught and got fixed before commit). A2 closes steps 1-2
   of `CRITICAL_ANDROID_FALSE_DELIVERY...` — core now classifies incoming
   Receipt messages and fires the delivery callback (previously nothing
   consumed them at all), and the CLI/core serialization mismatch
   (bincode-vs-JSON) is fixed. **Step 3 (Android Kotlin retry suppression)
   is still open** — see "What's still open" below.
2. `1b5f8fe0` **feat(farm): --http-bind/--health CLI flag (B3) + onion
   seam-freeze test (H1)** — `scm --http-bind <addr>` (global flag) spawns an
   axum 0.7 health server for cloud relay monitoring. H1 added
   `core/tests/seam_freeze_onion.rs` asserting onion routing stays
   config-gated-off by default; running it for real surfaced a genuine
   finding tracked as its own ticket: `ONION_FFI_RPC_SURFACE_UNGATED.md`
   (mobile/WASM FFI bridges expose onion construction ungated — verified
   exposed-but-unused, not farm-blocking, lowered priority).
3. `f9dbde43`, `5f5ea2ed`, `9d763ac2`, `d5dd4eb5` **infra(aws): free-tier IAM
   policy, credential script, relay provisioning script, budget
   kill-switch** — full B4 (cloud relay) infra prep. **OPERATOR SAID DROP
   THIS FOR NOW (2026-07-13 EOD)** — the credential-injection script was
   never actually run (operator navigated away before completing it), so no
   AWS resources exist. Everything is committed and ready to resume later,
   but is NOT a current priority. See `infra/aws/README.md`.
4. `8f0d3adf` **docs(pqc): E1 pq_ss redesign attempt 2 triangulated ->
   BLOCKED** — see "PQC-07 E1: two blocked attempts" below, the single
   hardest open problem in the backlog.
5. `50b899d7` **fix(crypto): persist skipped ratchet keys across session
   reload (E3)** — closes `PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED`. Real fix
   + a regression test proving it (not just "compiles").
6. `69446c34` **docs(queue): correct stale PQC-08 status** — hygiene fix,
   `_QUEUE.md` said IN PROGRESS for a ticket that had actually been in
   `done/` since 07-11.

## In-flight, NOT yet committed — check this first

**`core/tests/integration_ledger_convergence.rs` (WS-FARM-F1)** exists on
disk, uncommitted. It compiles clean but the real run FAILED (confirmed after
this doc was first drafted — the run just finished):
```
thread 'test_ledger_convergence_between_nodes' panicked at
core\tests\integration_ledger_convergence.rs:93:10:
Failed to dial: Dial error: no addresses for peer.
```
This is at `swarm2.dial(node1_addr.clone())`, where `node1_addr` is the
`Multiaddr` captured from node 1's `SwarmEvent2::ListeningOn` event. The
reference test (`integration_nat_reflection.rs`) uses the EXACT same
`event_rx1.recv()` -> `ListeningOn(addr)` -> `swarm2.dial(addr)` pattern and
presumably works (it's the codebase's one proven multi-swarm test), so the
likely culprits, in order of suspicion:
1. **Missing `/p2p/<peer_id>` suffix.** "no addresses for peer" is the
   characteristic libp2p error when `dial()` receives a bare transport
   multiaddr with no peer ID component, so it can't associate the dial with
   a `PeerId` in `get_peers()`/connection-tracking. Check whether
   `ListeningOn`'s `Multiaddr` needs the peer ID appended manually before
   dialing (`node1_addr.with(Protocol::P2p(keypair1.public().to_peer_id()))`
   or similar) - this may be something `request_address_reflection`'s test
   path handles differently than a bare `dial()`.
2. Confirm the reference test actually still passes as of today (it may
   have bit-rotted, or relies on being run in a specific way) - run
   `cargo test -p scmessenger-core --test integration_nat_reflection --
   --include-ignored` as a sanity check before assuming the pattern is solid.
3. Possible timing issue if `ListeningOn` fires for a `0.0.0.0`/wildcard bind
   address rather than a concrete dialable one - check what `start_swarm`
   actually listens on by default.
**Do not blindly re-dispatch this** - read `swarm.rs`'s `dial()` implementation
and `ListeningOn` emission site directly first (same discipline used
throughout this session: verify against real source before guessing), then
fix and re-run. The harness pattern (event channels, start_swarm calls,
LedgerManager/SharedPeerEntry usage) is otherwise solid - this looks like one
specific address-format bug, not a structural rewrite.

## PQC-07 E1: two blocked attempts, this is the hardest remaining item

The PQ ratchet's shared secret still never mixes into the root key
(`PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md`), cryptographically
inert post-bootstrap. Two designed, triangulated attempts have both been
BLOCKED by adversarial review, for two DIFFERENT reasons:
- **Attempt 1** (prior session): asymmetric mixing tied to a DH crossing —
  receiver mixed, sender didn't — root key desync on reorder.
- **Attempt 2** (this session): decoupled mixing via a new
  `mix_pq_secret()` both sides call after encrypt/decrypt — fixed attempt
  1's bug cleanly, but Fusion Lite (3-panel + judge, real triangulated
  review) found tying the mix to a SPECIFIC message means losing that
  message desyncs the root key anyway — same failure class, different
  trigger (packet loss instead of reorder).

Both attempts' diffs + full review verdicts are preserved at
`HANDOFF/review/PQC_07_ATTEMPT2_*` and the original attempt's patch. The
ticket file has a synthesized "what attempt 3 needs to get right" section:
tie the mix to a DH ratchet step (self-synchronizing via the public envelope
header, survives message loss) while keeping attempt 2's sound symmetric
KDF mechanism. **This needs real design work, not another single-shot
dispatch** — PQC-11/13 stay frozen until it lands (standing rule).

## What's still open, roughly in priority order

1. **F1 test** — confirm pass, commit (see above, do this first).
2. **A3 — Android Kotlin retry suppression** (closes
   `CRITICAL_ANDROID_FALSE_DELIVERY...` fully): transport-success must never
   escalate to failed/corrupted, widen the receipt window, add a Kotlin
   regression test. This is Kotlin-side work, different stack from
   everything else this session touched.
3. **E1 attempt 3** — the PQC-07 redesign above. High-value, high-difficulty.
4. **B1 — DNS-name-first hardening** (the IP-flip mandate, AD-2): re-resolve
   on dial failure, don't poison the negative cache against a hostname,
   `LedgerManager` stores hostnames not raw IPs for DNS-named peers.
   `[AUDIT-GATE]` — touches transport/.
5. **B2 — bootstrap unification**: `SC_BOOTSTRAP_NODES` doesn't wire into the
   CLI `start` path; fold or document the 3-way precedence (CLI promiscuous
   bootstrap / core `BootstrapManager` / config.json).
6. **D1 — Meeting Mode design note** (S4, 6-10 devices one room): connection
   budget/rotation for Android's concurrent-GATT ceiling, in-room star-hub
   election, Multipeer offload for iOS pairs. `[OPUS+/THINK]` — the one
   genuinely novel design item left in the plan.
7. **F2 — drift custody persistence audit** (verify-first: does `MeshStore`
   survive process death on mobile — `iron_core.rs:264` flagged as
   in-memory construction, unconfirmed).
8. **C-lane (iOS)** — gated behind GitHub billing (RESOLVED — repo now under
   `Sovereign-Communication` org, runners work) and PQC-10 landing (for the
   single bindings-regen cycle). Not startable until PQC-10.
9. **AWS/B4** — paused per operator, infra is ready when resumed
   (`infra/aws/README.md` has the exact steps; credential script was never
   completed, no real AWS resources exist).

## Operational lessons from this session (apply immediately, don't relearn)

- **agy CLI needs `--add-dir "<repo path>"` on every dispatch.** Without it,
  agy self-discovers the repo path from scratch each time and often burns
  its whole budget on exploration before reaching the task. Confirmed root
  cause and fix (memory: `feedback_agy_timeout_root_cause`).
- **Chained `git add && git commit && git push` (or + emoji-check) in one
  Bash call hangs at 2 minutes on this host, reproducibly.** Isolate each
  git step into its own tool call (memory:
  `feedback_git_chain_hangs_isolate_steps`).
- **Qwen model rotation**: the `delegate_task.py` tier map is stale
  (`--tier thinking` maps to a vision model that times out on code). Pass
  `--model` explicitly and rotate through the ~65-model text/code/reasoning
  pool on 403 quota errors — full categorized list in memory
  `reference_dashscope_qwen_lane` (HEAVY/CODE/GENERAL/FAST tiers).
- **Fusion Lite is live and doing real work** — found 5+ genuine bugs this
  session across A1/A2/E1 attempt 2 that would otherwise have shipped. Key:
  `~/.config/scmorc/openrouter_fusion.env` (`backup_fusion_Lite_Key`,
  $0.50/day cap — the operator's other OpenRouter key has no spend limit and
  `fusion_lite.py` correctly refuses to run against it). `mistralai/*`
  models are BYOK-denylisted, don't use them in a panel. Always `unset
  OPENROUTER_API_KEY` before re-exporting in this persistent shell — it can
  retain a stale value from earlier in a session.
- **Delegated diffs are frequently incomplete or wrong when a model only
  sees a narrow snippet** — cross-file signature mismatches (e.g. E1's
  `mix_pq_secret` returning `()` vs the wiring diff assuming `Result`),
  wrong import paths, wrong assumed struct fields. Always verify against the
  REAL file before applying, and don't be afraid to hand-apply once you have
  full grounding rather than round-tripping a 3rd/4th dispatch.
- **Verification protocol** (operator-defined, apply going forward): Fusion
  Lite triangulation on every produced change (unanimous clean = instant
  pass = commit; any finding = micro-remediate that exact issue; a finding
  too large for a micro-task = teardown + re-scope, exactly like OUTBOX and
  E1 attempt 2 both were).
- **Zero native Claude subagent spend** was the standing directive this
  session (API budget was critical, hit ~3% at one point). All substantive
  work went through agy/Qwen/Fusion Lite; native context was orchestration
  (dispatch, git, brief diff review) only. Keep this discipline unless the
  operator explicitly lifts it.

## Handoff prompt for the new session

See the message accompanying this document.
