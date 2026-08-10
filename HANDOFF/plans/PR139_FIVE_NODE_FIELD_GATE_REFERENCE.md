# SCMessenger PR #139 Unified Five-Node Field-Gate Reference

**Mac lane + Windows lane + headless infrastructure node + delivery reliability + merge gate**

> **Purpose:** Provide one authoritative, repo-ready operational reference for reconciling PR #139, fixing known pre-freeze runtime blockers, rebuilding the five-node qualification harness, freezing one trustworthy runtime candidate, and proving it through two complete G1-G6 matrix passes plus one continuous 60-minute five-node soak.

| Field | Current reference value |
|---|---|
| **Status** | **ACTIVE field-gate execution reference** |
| **Authority** | PR #139 qualification scope and operator-approved field-gate decisions |
| **Last reconciled** | 2026-08-10 04:57 HST |
| **PR #139** | Open, `tracking/pre-v040-tag-work` |
| **Observed PR #139 head** | `e5284b7b7af194a53d4207f37d845cc16d2d7c56` |
| **Observed `main` head** | `d8ba796e2524128c868dfb06f301dfcf19333243` |
| **Branch relationship at reconciliation** | Diverged from merge base `8646a2ca366efe1e96d3fbdd2f749b36c1932e5e`; `main` had 34 commits not in PR #139 while PR #139 had 98 commits not in `main` |
| **Last identified PR-branch runtime candidate** | `7e527df0988c2c0a0cda56ac0c73edac6163c73b` - **NOT approved for freeze as-is** |
| **Runtime freeze** | **NOT DECLARED** |
| **Qualification bar** | Two complete G1-G6 matrix passes + one continuous 60-minute full-fleet soak |
| **Release signing** | Outside the PR #139 merge gate; preserve current test signing lineage where needed |
| **Post-merge target** | Wider real-world testing, including Josh in Pennsylvania |

> **Precedence:** Once committed to the repository and referenced from PR #139, this file is the field-gate execution authority for the operator decisions and qualification semantics it contains. Historical PR body text, handoff snapshots, and older runbooks remain evidence, not current authority, where they conflict with this file. Implementation facts must still be re-verified against the exact current SHA before code changes.

## How orchestrators should use this file

1. Read the repository's current canonical orchestration rules first (`AGENTS.md`, `docs/ORCHESTRATION.md`, and the machine-readable orchestration manifest once Control Plane v2 lands).
2. Read this document completely before classifying PR #139 work.
3. Reconcile each historical finding against the current PR branch and current `main`; do not assume an old ticket is still open or still fixed.
4. Classify work into **MUST FIX BEFORE FREEZE**, **MUST VERIFY BEFORE FREEZE**, **FIELD-GATE ONLY**, and **OUT OF SCOPE / POST-MERGE**.
5. Use fresh scoped workers for substantive investigation, implementation, and validation. The persistent controller coordinates; it does not take over source edits when a worker fails.
6. Security/transport/routing/delivery-sensitive changes require independent review according to current repo policy.
7. Persist execution state and evidence in the repo/HANDOFF surfaces so a fresh controller can resume without chat history.
8. Keep the runtime anchor explicit. Documentation-only commits may advance the PR head without changing the frozen runtime SHA, but that distinction must be recorded.

# 1. Executive Directive


> Bottom line
> Do not merge PR #139 first and “test after landing.” Do not freeze
> runtime SHA 7e527df0 as-is. First reconcile the finite-retry behavior
> with SCMessenger’s durable-delivery philosophy, close or consciously
> accept every known pre-freeze blocker, then freeze one exact runtime SHA
> and deploy that identical candidate to the four messaging endpoints plus
> the headless node.


The recommended sequence is: known-scope preload -> delivery-state
correction -> CI/adversarial review -> hard runtime freeze ->
deployment/provenance verification -> Matrix Pass 1 -> Matrix Pass 2
-> 60-minute five-node soak -> merge PR #139 -> wider real-world
testing with Josh in Pennsylvania. Mobile release-signing cleanup may
follow the merge and does not block this field gate.

## 1.1 Operator decisions now locked

| **Decision**        | **Locked answer**        | **Operational meaning**                                                                                                                                       |
|---------------------|--------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| AWS role            | Headless node            | Infrastructure for rendezvous/relay/store-and-forward custody. It is not a normal chat endpoint and does not need to originate ordinary user messages.        |
| Reproducibility bar | 2 matrices + 1-hour soak | Two complete G1-G6 matrix passes, then one uninterrupted 60-minute full-fleet soak.                                                                           |
| Freeze rule         | Hard runtime freeze      | Pre-load all known required scope first. Once frozen, no runtime-code drift during the gate; any runtime fix creates a new anchor and restarts qualification. |
| Signing             | Post-merge concern       | Signing lineage is important for distribution, but it is not part of the PR #139 merge gate.                                                                 |
| Near-term outcome   | Real user testing        | Reach a merged, field-proven build suitable for testing with Josh in Pennsylvania.                                                                            |

## 1.2 Definition of “unified path”

- One runtime candidate SHA, explicitly named and immutable during
  qualification.

- One shared scoring contract for both Mac and Windows lanes.

- Two platform-specific lane drivers that gather evidence without
  redefining PASS/FAIL.

- One five-node topology definition: Windows CLI, Android Pixel, macOS
  CLI, physical iPhone, and AWS headless node.

- One delivery philosophy: accepted undelivered messages remain an
  outstanding delivery obligation indefinitely; active transmission is
  opportunistic and backoff-aware, not blind and finite.

- One merge gate: two matrix passes plus one 60-minute full-fleet soak
  on the frozen runtime candidate.

# 2. Current Repository Position

PR #139 (“tracking: pre-v0.4.0 tag work summary”) is open and
mergeable. Its current head is e5284b7b7af194a53d4207f37d845cc16d2d7c56.
The commits after runtime candidate 7e527df0 are
documentation/handoff-oriented, so 7e527df0 is the last identified
runtime-code candidate within the branch as of this reference.

CI at the PR head and at the runtime candidate was observed green across
the major workflow groups. That is necessary but not sufficient: PR
#139 has repeatedly exposed defects only under physical, multi-peer
fleet growth and real transport failover.


> Do not infer field readiness from green CI
> The most consequential historical failures appeared only when the
> fleet grew beyond a one-peer soak: request-response bookkeeping panic,
> stale/malformed address churn, relay fallback gaps, identity
> instability, and receipt convergence behavior. The field gate exists
> specifically because unit/CI success does not exercise those
> interactions.


## 2.1 Candidate status: 7e527df0

| **Area**                    | **Status**   | **Interpretation**                                                                                                                                              |
|-----------------------------|--------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Receipt convergence fix     | Present      | Delivered / legacy Read receipts clear sender retry state through mark_message_sent().                                                                          |
| Build provenance logging    | Present      | Exact git/ref/build-time provenance was added for Android and runtime logs.                                                                                     |
| Finite retry terminal state | BLOCKER      | Code still contains hard terminal behavior at attempt thresholds, including 12-attempt failure and another path that becomes permanent after 3 attempts.        |
| Full multi-peer field proof | Not complete | Must prove request-response stability, relay behavior, liveness, and fleet convergence on five nodes.                                                           |
| Runtime freeze              | NO           | Do not call 7e527df0 the frozen candidate until finite-attempt abandonment semantics are corrected or explicitly redesigned to satisfy the operator philosophy. |

## 2.2 Findings added after the original reference

The original reference captured the major PR #139 runtime and field-test decisions, but subsequent cross-lane reconciliation added several important corrections and constraints.

### A. `main` and PR #139 are now evidence sources that must be reconciled, not blindly merged

At this reconciliation, `main` and PR #139 are materially diverged. Newer `main` commits are heavily documentation/evidence oriented and contain field findings that post-date older PR #139 handoffs, while PR #139 contains later runtime work not present on `main`. Do **not** merge `main` wholesale into the PR branch merely to collect those findings. First classify each `main`-only finding as:

- already fixed on PR #139;
- still open on PR #139;
- superseded / measurement error;
- documentation-only evidence to carry forward;
- unrelated to the PR #139 gate.

### B. Request-response panic evidence became stronger

A later field reproduction on the historical anchor showed the request-response assertion panic still occurred even with **address-level dial deduplication**. The failure involved one peer reachable at several stale/current addresses, so the contended resource is effectively the **peer**, not only the address. The later PR #139 runtime line claims a per-peer established-connection cap (historically reported as 2), but the field gate must prove that exact current implementation under fleet growth. A one-peer soak is not meaningful evidence for this class of failure.

### C. Relay fallback diagnosis was corrected

A later investigation retracted the earlier claim that relay circuits were never attempted. The measurement regex stopped at `/tcp/<port>` and discarded `/p2p-circuit`, so circuit attempts were invisible. Corrected evidence showed relay/circuit attempts did occur.

The remaining concern is more precise: **candidate construction and ordering may select the wrong hop or stale/self addresses before useful headless-node paths**. Field evidence included repeated loopback/self-dial and stale LAN candidates for a roaming peer, plus circuit candidates anchored on poor base addresses. Therefore:

- never use “zero relay attempts” as the current root-cause statement;
- capture the full candidate ladder including `/p2p-circuit` suffixes;
- record which hop/base address each circuit candidate uses;
- distinguish “relay attempted through bad hop” from “relay not attempted”;
- classify route success using receiver/custody evidence, not candidate presence alone.

### D. NAT traversal evidence needs directional/context-aware interpretation

DCUtR/hole-punch support exists and should be observed explicitly in roaming tests. The later field plan notes that UPnP is not a reliable alternative in the current path, which makes DCUtR plus headless-node relay/custody especially important. Do not blanket-blacklist an address merely because it matches the local node's external IP: peers behind the same NAT may legitimately share the same public address. Self-dial prevention must be identity/direction/context aware rather than a simplistic external-IP blacklist.

### E. Measurement blind spots caused multiple false conclusions

The harness must defend against these known observability failures:

- truncated API output mistaken for absence of state;
- sync/control envelopes mistaken for broken user messaging;
- regexes that strip route suffixes such as `/p2p-circuit`;
- `tail -F` or equivalent collectors going deaf across process/node restart;
- panic watchers that themselves exit on panic and therefore lose the fallback evidence channel;
- Android log-ring eviction under high-volume logging;
- platform-command quirks such as `tasklist /FO CSV /NH` under Git Bash returning misleadingly empty output.

Absence in a collector that could not observe the thing is **not evidence of absence**.

### F. Android in-place upgrade path is currently usable

The Pixel's installed app signing certificate was matched to the local `~/.android/debug.keystore`, while the CI debug artifact uses a different throwaway key. A local build can therefore be installed with `adb install -r` while preserving Pixel identity/history. This is sufficient for the PR #139 field gate. Managed signing remains release-process work and is not a merge blocker.

### G. Historical adversarial-review BLOCK findings must be explicitly reconciled

A PR #139 adversarial review filed at the common ancestor reported a security BLOCK around trust-scoped RFC1918 disclosure and transport-block semantics, including requester-context mistakes, class-vs-subnet granularity, loopback/link-local disclosure, redistribution of unproven remote addresses, and a claimed peer-ID block gate not being fully wired. Later PR-branch commits may have remediated some or all of those findings. Before freezing, a fresh security reconciliation must map every historical BLOCK item to exact current code/tests and produce a current PASS/FAIL disposition. Do not assume green CI or a later commit message closes a security review finding.

### H. Android/BLE observability and liveness findings must be checked, not assumed fixed

Later `main` evidence documented an Android BLE L2CAP accept-loop storm capable of producing roughly 100 stack traces/second after the server socket died, both impairing inbound BLE acceptance and evicting useful log evidence. Other historical BLE work found callback/thread stalls and receipt-related test issues. The PR #139 branch may contain fixes, but the field gate must verify:

- inbound BLE acceptance recovers after transport failure;
- no tight accept/retry loop persists;
- log volume remains bounded enough to preserve the test window;
- BLE failures do not mask LAN/relay recovery;
- evidence collection survives the exact failure being diagnosed.

### I. The PR body is historical, not the current runbook

PR #139's body still describes earlier rollout status, older open decisions, and a generic “must run twice” requirement. This document carries the later operator decision: **two complete matrices followed by one continuous 60-minute soak**, with a hard pre-test runtime freeze and signing outside the merge gate.


# 3. Delivery Philosophy: Canonical Interpretation


> Owner clarification
> Messages/history may be retained permanently according to product
> policy, and an accepted undelivered message must remain eligible for
> future delivery even if the recipient is unreachable for months or
> years. A finite count of network attempts must never silently erase that
> delivery obligation.


The critical distinction is between message/history retention, the
outstanding delivery obligation, and individual network transmission
attempts. Those are three different lifecycle concepts and must not be
represented by one overloaded “retry count.”

## 3.1 Required lifecycle semantics

| **Concept**         | **Required behavior**           | **Why it matters**                                                                                                                                |
|---------------------|---------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| History record      | Durable after acceptance        | Conversation record remains according to message-history policy; successful delivery does not imply deleting history.                             |
| Delivery obligation | Indefinite until terminal truth | Remains outstanding until confirmed delivered, explicitly cancelled by the user/policy, or rejected for a genuinely irreversible protocol reason. |
| Network attempt     | Finite and adaptive             | A single attempt can fail. Future attempts are scheduled based on opportunity and bounded backoff; the lifetime number of attempts is not capped. |
| Headless custody    | Intermediate state              | Custody/store-and-forward means a relay has accepted responsibility to carry/forward. It is not equivalent to final recipient delivery.           |
| Receipt             | Delivery truth                  | A valid application delivery receipt satisfies the active delivery obligation and stops further transmission for that message.                    |

## 3.2 Opportunistic retry triggers

The system should prefer event-driven opportunities over blind polling.
Appropriate retry triggers include:

- peer identified/reconnected or a previously unreachable peer becomes
  reachable;

- new viable direct/LAN/BLE address is learned or validated;

- a healthy headless-node relay path becomes available;

- custody state changes or a store-and-forward carrier reports a
  delivery opportunity;

- network interface / Wi-Fi / cellular transition materially changes
  path viability;

- application wake/start/reconnect reconciliation where pending
  obligations are re-evaluated;

- backoff timer expiry only when there is still a plausible route, with
  jitter and a bounded cadence to avoid network churn.

## 3.3 What is explicitly wrong

- A static “attempt #12 -> Failed -> next_retry_at=None” rule for
  transient/unreachable delivery.

- A “3 failed transport attempts -> permanent failure” interpretation
  when the peer may simply be offline.

- Dropping an accepted message because it has been offline too long.

- Continuing to transmit after a valid delivery receipt has already
  satisfied the obligation.

- Treating relay custody as final delivery or, conversely, continuing
  direct hammering while good custody exists without a reasoned policy.

## 3.4 Philosophy-canon cleanup required

The repository Philosophy Canon already states that durable
store-and-forward prevents message loss under transient connectivity and
that delivery/reconnect are eventual-consistency targets that converge
toward 100%. Its separate “bounded retention” language must be clarified
so future agents do not interpret storage bounds as permission to expire
an accepted undelivered delivery obligation. Resource bounds should be
handled through capacity policy, user-visible backpressure,
archival/compaction, or explicit operator policy - not silent delivery
abandonment.

# 4. Pre-Freeze Scope: What Must Be Resolved Before Deployment

The purpose of pre-freeze scope loading is to avoid spending hours
rebuilding five devices against a candidate that already contains a
known gate-breaking behavior. Runtime freeze is declared only after this
list is satisfied or an explicit owner-approved exception is recorded.

| **ID** | **Item**                                  | **Priority**     | **Required action**                                                                                                                                 | **Acceptance proof**                                                 |
|--------|-------------------------------------------|------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------|
| PF-1   | Finite-attempt abandonment                | P0 / philosophy  | Remove hard terminal retry-count behavior for transient reachability. Design durable outstanding-delivery state and opportunistic retry scheduling. | Unit + persistence + real two-node offline/reconnect proof.          |
| PF-2   | Receipt/outbox convergence                | P0 / G3          | Preserve the existing Delivered/Read -> mark_message_sent path and prove sender state converges without re-sending delivered messages.             | Sender API + receiver inbox + receipt evidence on real pair.         |
| PF-3   | Request-response stability on mesh growth | P0 / G5          | Verify connection-cap/bookkeeping fix under multi-peer growth, relay + direct overlap, and sustained churn.                                         | No panic/swarm death in matrices or soak.                            |
| PF-4   | Headless relay fallback                   | P0 / G2/G5       | Prove unreachable endpoints can route through the connected headless node using a viable circuit/custody path. Relay attempts are known to occur; the key risk is wrong-hop/stale/self candidate ordering and unusable circuit construction. | Receiver evidence + route evidence through AWS headless node.        |
| PF-5   | Identity stability                        | P0 / G4/G6       | Preserve persistent data and identities across rebuild/restart. No wipe during qualification.                                                       | Same PeerId/public identity before/after restart and throughout run. |
| PF-6   | Exact provenance                          | P0 / G6          | All five nodes must expose the frozen SHA/build stamp; headless container should be anchored to immutable image/digest.                             | Manifest collected before each matrix and soak.                      |
| PF-7   | Harness parity                            | P1 / operability | Replace outdated run5 topology with shared contract plus Mac/Windows lane drivers.                                                                  | Both lane outputs score with same schema and can be merged.          |
| PF-8   | Signing lineage                           | Post-gate        | Document but do not block PR #139 merge. Local Pixel in-place signing is usable for current test.                                                  | Tracked separately for release/distribution.                         |
| PF-9   | Security BLOCK reconciliation             | P0 / security    | Reconcile every historical PR #139 adversarial-review BLOCK finding against the exact candidate; do not infer closure from CI or commit messages. | Current independent adversarial PASS/explicit dispositions. |
| PF-10  | Candidate ordering / stale self-dial      | P0 / G2/G5       | Prove loopback, stale LAN, shared-external-IP, and circuit candidates are ordered/filtered contextually without suppressing legitimate DCUtR/relay paths. | Full candidate-ladder evidence plus successful roaming/fallback proof. |
| PF-11  | BLE liveness + evidence preservation      | P0/P1 / G2/G5    | Verify Android BLE accept/recovery behavior is bounded and does not destroy log observability or require app restart. | Controlled BLE failure/recovery proof with preserved logs. |
| PF-12  | Accepted-work capacity semantics          | P1 / philosophy  | Audit queue/retention maintenance so already accepted undelivered work cannot be silently discarded due to age or lifetime attempt count. If capacity is exhausted, prefer explicit backpressure/rejection before acceptance over hidden loss. | Tests for persistence across restart/capacity boundaries and explicit policy behavior. |

## 4.1 Runtime changes reset the anchor

Any runtime-code change after the candidate is frozen invalidates the
previous deployment. The new commit becomes a candidate only after
CI/review passes; all five nodes must be re-anchored to that exact SHA
before Matrix Pass 1 can start. Documentation-only evidence commits may
advance the PR branch without changing the runtime anchor, but the
distinction must be recorded explicitly.

## 4.2 Current pre-freeze decision rules

Before declaring a candidate, every item above must have one of these explicit dispositions in the execution ledger:

- **FIXED + VERIFIED** - code/test/review evidence exists on the candidate;
- **ALREADY FIXED + REVERIFIED** - historical ticket is stale but current code was inspected and tested;
- **FIELD-GATE VERIFICATION** - no known code blocker remains; physical proof is intentionally deferred to matrix/soak;
- **OUT OF SCOPE** - explicitly unrelated to the PR #139 merge gate;
- **OPERATOR EXCEPTION** - only for a known risk the operator consciously accepts.

“Probably fixed,” “CI green,” and “ticket says done” are not dispositions.


# 5. Five-Node Topology and Roles

| **#** | **Node**          | **Role**                                     | **Required behavior**                                                                                    |
|--------|-------------------|----------------------------------------------|----------------------------------------------------------------------------------------------------------|
| 1      | Windows CLI       | Messaging endpoint / Windows lane controller | Stable persisted identity; sends/receives normal messages; captures API/log evidence.                    |
| 2      | Android Pixel 6a  | Messaging endpoint / Windows lane mobile     | Physical Android; in-place install preferred to preserve identity/history; BLE/LAN/relay coverage.       |
| 3      | AWS headless node | Infrastructure only                          | Rendezvous/relay/store-and-forward custody. Must not be counted as a normal pairwise user-chat endpoint. |
| 4      | macOS CLI         | Messaging endpoint / Mac lane controller     | Stable persisted identity; no data wipe during a run; captures CLI/API/log evidence.                     |
| 5      | Physical iPhone   | Messaging endpoint / Mac lane mobile         | Real iOS behavior including Wi-Fi/cellular transition and BLE/LAN/relay observations.                    |

### 5.0 Field-gate role semantics vs. product node doctrine

For **this PR #139 qualification matrix**, the AWS participant is scored as the headless infrastructure node: relay/routing/store-and-forward/custody behavior is required, but ordinary human-chat G1 flows are scored across the four user endpoints only. This is a **test-role definition**, not permission for an agent to rewrite the repository's broader architectural doctrine about what a node is. If current canonical docs describe all deployments as full nodes, preserve that product doctrine unless the operator explicitly changes it; simply exclude the headless participant from the user-endpoint G1 combinatorial matrix for this gate.

Historical PeerIds may be used only for orientation. The run manifest
must capture the live identity of every node immediately before each
matrix/soak because identities can change if persistent data is
accidentally replaced.

## 5.1 Pairwise messaging matrix

Because the headless node is infrastructure, G1 pairwise bidirectional
messaging applies to the four user/messaging endpoints. That produces
six endpoint pairs and twelve directional message flows per complete
matrix:

| **Endpoint pair**     | **Required flows** | **Notes**                                                                              |
|-----------------------|--------------------|----------------------------------------------------------------------------------------|
| Windows \<-> Android | 2 directions       | Local/LAN where possible; relay fallback where forced; receipt on both directions.     |
| Windows \<-> macOS   | 2 directions       | Cross-lane internet/direct/relay behavior; high-value request-response stability pair. |
| Windows \<-> iPhone  | 2 directions       | Remote/mobile path; relay fallback and roaming observations.                           |
| Android \<-> macOS   | 2 directions       | Cross-platform mobile/desktop interoperability.                                        |
| Android \<-> iPhone  | 2 directions       | BLE/LAN and mobile transport failover focus.                                           |
| macOS \<-> iPhone    | 2 directions       | Same-lane desktop/mobile baseline plus identity/history stability.                     |

# 6. Harness Strategy: Rebuild run5 for the Current Fleet

The historical scripts/run5.sh models a different topology and should
not be used as the final PR #139 qualification harness. Reuse concepts,
not assumptions.


> Recommended shape
> Create a shared gate contract/scorer plus two lane collectors:
> run5-windows.ps1 for Windows + Pixel + headless evidence and
> run5-macos.sh for macOS + iPhone + headless evidence. Both emit the same
> manifest and test-result schema so a shared scorer can produce one fleet
> verdict.


## 6.1 Shared components

- gate-manifest schema: candidate SHA, build stamps, PeerIds/public
  identity, node role, OS/device, data-directory fingerprint, listeners,
  headless image digest;

- shared test-case IDs and PASS/FAIL vocabulary for G1-G6;

- message correlation IDs generated before send and recorded on sender +
  receiver;

- route evidence schema: direct, LAN, BLE, relay, custody,
  hole-punch/DCUtR, unknown;

- failure taxonomy: provenance, identity, routing, transport, receipt,
  retry-state, custody, crash/swarm death, evidence collection;

- one scorer that refuses to pass a message from sender-side “accepted”
  alone.

## 6.2 Harness safety rules

- No automatic git pull/rebuild when running a gate. Deployment is a
  separate explicit step.

- No use of Docker :latest or a mutable image tag for the headless node.
  Record immutable digest / exact SHA-derived image.

- No deletion/recreation of Mac or Windows persistent data during a run.

- No fresh-install mobile reset unless explicitly required; prefer
  in-place deployment preserving identity and history.

- No measurement that truncates API responses or silently suppresses
  command failures.

- No PASS when the relevant evidence collector failed to run.

- No single-lane reinterpretation of shared criteria.

## 6.3 Measurement rules learned from failed investigations

The harness must prove that each collector can observe the evidence it is asked to score.

- Preserve complete multiaddrs including `/p2p/<peer>/p2p-circuit/...` suffixes.
- Record raw candidate lists separately from parsed route classifications.
- Keep a process-independent/persistent failure channel so a panic does not erase the only evidence watcher.
- Re-open log streams after node restart rather than assuming a tail survives rotation/replacement.
- Bound verbose transport loops so logs remain usable for the whole matrix/soak window.
- Capture full API responses to files before filtering for human display.
- Distinguish user-message envelopes from ledger/discovery/control traffic.
- When a tool returns “nothing,” first validate the tool invocation on that platform before concluding the process/state is absent.
- Score “unknown” as **unproven**, not PASS and not automatically a product bug.

## 6.4 Execution governance / orchestration contract

The field-gate plan is tool-agnostic. The repository's orchestration control plane governs *how* the work is delegated.

Until Orchestration Control Plane v2 lands, use the current canonical `docs/ORCHESTRATION.md` and `AGENTS.md` rules but apply the operator-approved stricter principle: the persistent controller coordinates and integrates evidence; substantive investigation, implementation, repair, planning, and validation should run in fresh scoped worker contexts. A worker failure should cause re-brief/re-dispatch/escalation, not controller takeover of application source.

Once Control Plane v2 is merged, use its machine-readable manifest and hardened `scripts/orchestrate_strict.py` as the common kernel. This PR #139 document remains the field-gate **what/acceptance** authority; the control plane remains the **how/delegation** authority.

For Codex with the GPT-5.6 family, the intended cost/capability mapping is:

| Semantic role | Codex default | Effort |
|---|---|---|
| Controller / dispatcher | GPT-5.6 Luna | medium |
| Scanner / investigator | GPT-5.6 Luna | low; medium only when needed |
| Evidence / mechanical QA | GPT-5.6 Luna | low-medium |
| Micro implementation | GPT-5.6 Luna | medium |
| Standard implementation | GPT-5.6 Terra | medium |
| Complex but already-designed implementation | GPT-5.6 Terra | high |
| Standard independent validation | GPT-5.6 Terra | high |
| Planner / architecture escalation | GPT-5.6 Sol | high |
| Critical validator | GPT-5.6 Sol | high |
| Deep second opinion | GPT-5.6 Sol | xhigh |
| Owner/product decision | Human operator | n/a |

Other frontends should map their own models onto the same semantic roles. Model availability changes the worker, not the role's authority.


# 7. Windows Lane Action Plan

## 7.1 Deployment and provenance

1. Rebuild/deploy Windows CLI from the frozen SHA with persistent data
preserved.

2. Build Android from the same frozen SHA and install in-place using
the local signing lineage that matches the currently installed Pixel
app.

3. Deploy/verify the AWS headless node from an immutable artifact tied
to the same frozen source SHA.

4. Capture Windows, Android, and headless node identity/build/listener
manifest before message testing.

5. Prove the Pixel did not lose first-install state, identity,
contacts, or history during update.

## 7.2 Qualification sequence

1. Baseline Windows \<-> Android bidirectional message with receiver
inbox evidence and application receipt.

2. Force/observe a route that uses the AWS headless node; capture
custody/relay evidence without counting the headless node as recipient.

3. Queue a message while the recipient is deliberately unreachable.
Confirm the message remains an outstanding durable obligation without
being abandoned after an attempt threshold.

4. Restore a viable route and confirm opportunistic delivery occurs;
sender state converges on receipt and further network transmission for
that message stops.

5. Restart Windows while preserving data; verify identity stability and
pending/delivered state reconciliation.

6. Disrupt Android LAN/Wi-Fi or BLE path as appropriate; verify
surviving transport/fallback resumes without app restart.

7. Keep all three Windows-lane nodes active while Mac/iPhone join the
fleet; monitor multi-peer connection count, request-response warnings,
and swarm lifecycle.

## 7.3 Windows lane PASS evidence

- Exact candidate SHA and stable identities for Windows, Pixel, and
  headless node.

- Sender message ID, receiver inbox event, receipt event, and sender
  convergence for every tested flow.

- Evidence an offline message remains durable beyond transient attempt
  failures and later delivers when opportunity appears.

- No request-response assertion/panic, swarm event loop death,
  uncontrolled dial storm, or repeated resend after confirmed receipt.

- Headless-node route/custody evidence for at least one forced relay
  scenario.

# 8. Mac Lane Action Plan

## 8.1 Deployment and provenance

1. Rebuild/deploy macOS CLI from the frozen SHA without wiping or
replacing the persistent data directory.

2. Build/install the physical iPhone candidate from the same frozen
SHA/build provenance; preserve identity if possible.

3. Capture macOS + iPhone identities, build stamps, listener/transport
diagnostics, and headless-node reachability before testing.

4. If the macOS PeerId changes unexpectedly, stop the run immediately
and diagnose persistence before generating more evidence.

## 8.2 Qualification sequence

1. Baseline macOS \<-> iPhone bidirectional messaging with
receiver-side evidence and receipts.

2. Mac \<-> Windows bidirectional cross-lane flow while all other
nodes remain running.

3. iPhone \<-> Android bidirectional mobile flow, including BLE/LAN
evidence where practical.

4. Queue a message to iPhone while it is unreachable; verify indefinite
outstanding delivery state without terminal attempt-count failure.

5. Restore iPhone connectivity and prove delivery is triggered by a
real opportunity, then stops after receipt.

6. Perform a Wi-Fi -> cellular (or equivalent public-network)
transition and observe relay/DCUtR behavior without app restart.

7. During full-fleet growth, monitor macOS for request-response
bookkeeping errors and verify identity remains stable.

## 8.3 Mac lane PASS evidence

- Exact candidate provenance and stable Mac/iPhone identities.

- Receiver inbox + receipt + sender-state convergence for each
  directional flow.

- At least one mobile roaming/fallback observation with route classified
  rather than assumed.

- No data wipe, identity churn, panic, swarm death, resend-after-receipt
  loop, or evidence-capture failure.

# 9. Matrix Pass 1 and Matrix Pass 2

Each matrix is a complete fleet qualification on the same frozen runtime
SHA. Matrix 2 is a fresh repetition, not merely a continuation of
Matrix 1. A matrix is invalid if runtime code, identity, or provenance
changes mid-pass.

## 9.1 Required matrix content

| **Gate**                         | **Required proof**                                                                                                             | **Scoring rule**                                                                            |
|----------------------------------|--------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------|
| G1 - Pairwise bidirectional      | All six messaging-endpoint pairs pass both directions (12 directional flows).                                                  | Sender acceptance is insufficient. Must have receiver inbox + receipt + sender convergence. |
| G2 - Transport coverage          | Demonstrate LAN/Wi-Fi, BLE where applicable, and internet/headless relay/custody.                                              | Coverage can be distributed across pairs; every claimed transport must have route evidence. |
| G3 - Delivery truth & durability | No false failure for delivered messages; undelivered accepted messages remain durable indefinitely; opportunistic retry works. | Test at least one deliberate offline/reconnect scenario in each lane.                       |
| G4 - Fleet convergence           | Messaging endpoints learn/recover expected fleet information; restart reconverges without re-pair.                             | Headless node is infrastructure but must remain discoverable/reachable as designed.         |
| G5 - Liveness                    | Network disruption/reconnect recovers without app restart; multi-peer growth does not crash swarm.                             | Includes relay/direct overlap and fleet-growth stress.                                      |
| G6 - Provenance                  | Every node reports the same frozen source anchor; headless artifact is immutable and traceable.                                | Any mismatch invalidates the matrix.                                                        |

## 9.2 Matrix execution order

1. Preflight manifest: identities, exact SHA/build stamp, listeners,
headless digest, clocks, log capture status.

2. Baseline same-lane flows: Windows \<-> Android and macOS \<->
iPhone.

3. Cross-lane desktop flow: Windows \<-> macOS.

4. Cross-lane mobile flow: Android \<-> iPhone.

5. Remaining pairwise flows: Windows \<-> iPhone and Android \<->
macOS.

6. Transport/failover scenarios: BLE/LAN and forced headless
relay/custody coverage.

7. Offline durability scenario: queue while unreachable, wait through
failed opportunities/backoff, restore viability, prove eventual
delivery.

8. Restart/reconvergence scenario with identities preserved.

9. Closeout audit: zero unclassified message flows, no missing
receipts, no unknown route for required G2 samples, no panic/swarm
death.

## 9.3 Matrix reset rules

- Runtime code change -> new SHA -> redeploy all nodes -> restart
  from Matrix Pass 1.

- Unexpected identity change -> diagnose and restore persistence ->
  restart the current matrix.

- Panic/swarm death -> matrix FAIL; fix/re-anchor if runtime change is
  needed.

- Missing receiver evidence for any required flow -> that flow is
  unproven, not assumed PASS.

- Evidence collector failure -> affected test must be rerun; “no errors
  found” from a dead collector is invalid.

# 10. Continuous 60-Minute Five-Node Soak

The soak begins only after Matrix Passes 1 and 2 pass. The one-hour
clock is not a substitute for the matrices; it is a stability/liveness
gate after functional correctness has been proven twice.

## 10.1 Start conditions

- All five nodes are on the same frozen source anchor / immutable
  artifact provenance.

- All identities are stable and recorded.

- Both matrices have passed with complete evidence.

- No known unresolved P0 affecting delivery, receipt truth, relay
  fallback, swarm stability, or identity persistence.

- Continuous log capture is active on both lanes and the headless node.

## 10.2 During the hour

- Maintain the full fleet rather than reducing to one peer.

- Generate periodic low-rate user messages across multiple pairs to
  exercise ongoing receipt convergence without creating an artificial
  flood.

- Exercise at least one controlled network transition/failover after
  steady state is established.

- Observe connection growth, relay/direct path transitions, outbox
  state, custody state, and retry scheduling.

- Confirm no delivered message resumes active transmission after
  receipt.

- Confirm unreachable outstanding messages remain durable and
  dormant/backed off until opportunity returns.

- Keep identity and provenance snapshots at start, midpoint, and end.

## 10.3 Clock-reset conditions

| **Condition**                                                                   | **Outcome**                                                                |
|---------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| Panic / process crash / swarm event-loop death                                  | Immediate FAIL; reset after fix/re-anchor.                                 |
| Identity changes unexpectedly                                                   | Immediate FAIL; persistence issue must be resolved.                        |
| Confirmed delivered message keeps actively re-sending                           | Immediate FAIL of G3.                                                      |
| Accepted undelivered message becomes permanently abandoned due to attempt count | Immediate FAIL of delivery philosophy/G3.                                  |
| Required headless fallback cannot be used when direct path is unavailable       | FAIL G2/G5.                                                                |
| Missing receiver or receipt evidence for soak probes                            | Affected probe unproven; investigate before clock can count as valid.      |
| Route classification unknown for required transport samples                     | Does not satisfy G2 until classified.                                      |
| Documentation-only commit lands on PR branch                                    | No reset if runtime artifact/provenance is unchanged and clearly recorded. |

# 11. Evidence Contract

Every gate result should be reconstructable by another engineer without
relying on operator memory. Evidence should be machine-readable where
possible and human-readable where necessary.

## 11.1 Per-run directory layout

Recommended conceptual structure (exact filenames may vary):

run/\<timestamp>/  
manifest.json  
matrix-1/results.json  
matrix-2/results.json  
soak/results.json  
windows/windows.log  
windows/android.log  
mac/macos.log  
mac/ios.log  
headless/headless.log  
messages/\<test-id>.json  
summary.md

## 11.2 Per-message evidence fields

| **Field**           | **Meaning**                                                  |
|---------------------|--------------------------------------------------------------|
| test_id             | Stable case ID, e.g. M1-WIN-ANDROID-W2A-01                   |
| message_id          | Application message identifier used for correlation          |
| sender / receiver   | Node role + live PeerId/public identity                      |
| accepted_at         | Sender accepted/enqueued timestamp                           |
| route               | direct / LAN / BLE / relay / custody / hole-punch / unknown  |
| receiver_inbox_at   | Receiver-side durable receive evidence                       |
| receipt_at          | Application receipt evidence                                 |
| sender_converged_at | Sender reports delivered / obligation satisfied              |
| retry_state         | Attempts/backoff/opportunity state before and after delivery |
| provenance          | Frozen SHA/build stamp for sender and receiver               |

# 12. GO / NO-GO Decision Framework

## 12.1 GO to freeze

- Finite-attempt permanent abandonment has been removed or redesigned so
  accepted undelivered messages remain delivery-eligible indefinitely.

- Receipt convergence and outbox release are covered by regression tests
  and real two-node proof.

- Request-response mesh-growth fix has credible code/review coverage and
  is ready for field proof.

- Headless relay fallback path is implemented/expected to be functional
  enough to test.

- CI is green on the exact runtime candidate and required adversarial
  review is complete.

- No known P0 is being intentionally deferred into a five-node
  deployment without an explicit owner exception.

- Historical PR #139 adversarial-review BLOCK findings have a current independent disposition on the exact candidate; no security gate is considered closed solely because a later commit claims a fix.

## 12.2 GO to Matrix Pass 1

- All five nodes deployed from the frozen candidate with exact
  provenance recorded.

- Persistent identities are stable; no accidental fresh profiles.

- Headless node is reachable and its immutable artifact is verified.

- Both lane collectors are active and the shared scorer can ingest their
  output.

## 12.3 GO to 60-minute soak

- Matrix Pass 1: PASS.

- Matrix Pass 2: PASS.

- No runtime code changed between them.

- No unclassified/missing critical evidence remains.

## 12.4 GO to merge PR #139

- Both matrix passes are fully evidenced.

- Continuous 60-minute five-node soak passes without reset condition.

- Frozen SHA is documented as the runtime tested candidate even if PR
  head also contains later documentation-only commits.

- Any non-blocking release-signing work is explicitly moved to
  post-merge tracking rather than silently forgotten.

- Josh/PA field-test handoff identifies the tested build and expected
  limitations.

## 12.5 NO-GO examples

- “CI is green, so let’s merge and test later.”

- “The process stayed alive for an hour with only one peer.”

- “The sender API said accepted, so the message passed.”

- “It failed 12 times, therefore the message is permanently failed.”

- “The relay is in the ledger, so relay fallback must work.”

- “Our regex showed zero relay attempts.” (The known regex dropped `/p2p-circuit`; capture complete addresses first.)

- “The candidate dialed a circuit address, so relay fallback passed.” (Attempted via the wrong/stale hop is not a successful route.)

- “The old adversarial ticket says fixed, so the security gate is closed.”

- “We rebuilt the Mac/iPhone and the identity changed, but that probably
  doesn’t matter.”

# 13. Failure Triage: Fix the Right Class of Problem

| **Symptom**                                       | **Likely class**                          | **First action**                                                                                      |
|---------------------------------------------------|-------------------------------------------|-------------------------------------------------------------------------------------------------------|
| Provenance mismatch                               | Different SHA/build stamp/image digest    | Stop. Redeploy before debugging network behavior.                                                     |
| Identity churn                                    | PeerId/public identity changed            | Stop. Diagnose persistence/install data before continuing.                                            |
| Sender accepted, receiver absent                  | No inbox evidence                         | Classify reachability/route; do not call delivery failure until transport evidence is inspected.      |
| Receiver has message, sender stays pending        | Receipt convergence                       | Inspect receipt path, message ID correlation, and mark_message_sent/outbox state.                     |
| Delivered message keeps transmitting              | Delivery obligation not released          | G3 blocker. Fix receipt/outbox/custody state handling.                                                |
| Undelivered message stops forever after threshold | Finite retry abandonment                  | Philosophy blocker. Restore indefinite outstanding state and opportunity-driven scheduling.           |
| Many stale/self dials or unusable circuit path         | Relay fallback / address selection        | Inspect full candidate ordering, loopback/self filtering, stale-address reaping, circuit hop/base-address selection, DCUtR attempts, and headless-node connectivity. |
| Crash only when fleet grows                       | Connection bookkeeping / request-response | Reproduce with multi-peer topology; single-peer soak is not meaningful proof.                         |
| Transport claimed absent                          | Measurement limitation                    | Verify collector can actually see full API/log data before filing a code bug.                         |
| Relay/circuit “absent” only in parsed output        | Collector/parser blind spot                | Inspect raw full multiaddrs; ensure `/p2p-circuit` suffix was not truncated by regex/parser. |
| Circuit attempts exist but roaming peer unreachable | Candidate ordering / wrong hop / stale base | Capture every candidate and hop, self/loopback decisions, stale-address age, DCUtR state, and receiver evidence. |
| Android BLE logs explode / useful window disappears | BLE accept-loop or logging storm            | Bound the loop, recreate/recover transport as needed, enlarge/persist logs, and repeat with evidence collector verified. |
| Security review says BLOCK but code later changed    | Stale review vs. unproven remediation       | Map every finding to exact current code/test and run a fresh independent review before freeze. |

# 14. Recommended PR / Merge Strategy

The cleanest reconciliation remains a stacked PR based on PR #139, not a premature merge of #139 into main. Because current `main` and PR #139 are diverged, first import/reconcile the *findings* from `main` without blindly merging unrelated runtime history. The stacked PR should first define
and implement the missing durable-delivery semantics and field-gate
harness. Once it is integrated into the PR #139 line and a runtime SHA
is frozen, the physical gate can proceed against the exact candidate
intended for merge.

## 14.1 Recommended stacked PR scope


> Suggested title
> fix: enforce durable delivery and define PR139 field gate


- Correct finite-attempt terminal behavior for transient/offline
  delivery.

- Define the durable-delivery lifecycle and opportunistic scheduling
  policy in canonical docs.

- Clarify Philosophy Canon: bounded resources do not mean bounded
  delivery lifetime.

- Add/update regression tests around offline persistence, restart,
  opportunity-triggered resend, receipt convergence, and resend
  suppression after receipt.

- Rebuild the five-node harness around the current
  Windows/Android/headless/macOS/iPhone topology.

- Provide Windows PowerShell and macOS shell lane collectors with a
  shared manifest/scorer.

- Document freeze, matrix, soak, and evidence rules in one canonical
  field-gate plan.

## 14.2 What should remain outside this PR/merge gate

- Managed production Android/iOS signing infrastructure and distribution
  lineage cleanup.

- Post-alpha security/dependency cleanup already queued separately
  unless it blocks runtime correctness.

- Tag naming/flavor choice beyond what is needed to identify the tested
  candidate.

- Broad refactors unrelated to the concrete five-node blockers.

## 14.3 Why not merge #139 first

PR #139 has functioned as the integration line for the runtime changes
that the field gate is intended to prove. Merging before that proof
converts a pre-merge safety gate into post-merge incident discovery. The
repository already has examples where problems surfaced only after fleet
growth, so preserving the gate as a pre-merge condition reduces rollback
ambiguity and keeps the tested SHA tied to the change set being
approved.

# 15. Ready-for-Josh / Pennsylvania Handoff

The objective after PR #139 merge is not release perfection; it is a
known-good, provenance-stamped build that has survived the local
five-node qualification and can be exercised across a genuinely remote
network.

## 15.1 Minimum handoff packet

- Exact merged commit / tested runtime SHA and build identifiers.

- Install/update instructions that preserve identity/history wherever
  possible.

- Known headless-node bootstrap/relay address or configuration and how
  to verify connectivity.

- Expected message-state semantics: accepted, pending/outstanding,
  custody, delivered.

- What evidence to capture if a message is delayed: sender ID/message
  ID, recipient identity, timestamps, route/connection state, receiver
  inbox, receipt/status.

- Known non-blocking limitations (including signing/distribution
  caveats) stated separately from delivery correctness.

# 16. Master Execution Checklist

1.  **A. Reconcile scope:** Update canonical delivery philosophy;
    identify every finite terminal retry path; define retry opportunity
    policy; decide handling of irrecoverable protocol errors; define
    headless custody semantics.

2.  **B. Implement pre-freeze fixes:** Remove transient attempt-count
    abandonment; preserve receipt-based release; add persistence/restart
    tests; verify multi-peer connection-cap/request-response fix; verify
    relay fallback candidate construction.

3.  **C. Review/CI:** Run formatting/unit/integration/mobile/desktop
    workflows; perform adversarial review on core delivery/routing
    changes; record exact candidate SHA.

4.  **D. Declare hard freeze:** No runtime code changes after
    declaration. Documentation may move only if runtime SHA remains
    explicit.

5.  **E. Deploy Windows lane:** Windows CLI + Pixel + headless node on
    exact candidate; preserve data; capture manifest.

6.  **F. Deploy Mac lane:** macOS CLI + iPhone on exact candidate;
    preserve data; capture manifest.

7.  **G. Matrix Pass 1:** Run all six endpoint pairs both directions;
    transport/failover coverage; offline durability;
    restart/reconvergence; score G1-G6.

8.  **H. Matrix Pass 2:** Repeat complete matrix from fresh run state on
    same runtime anchor.

9.  **I. 60-minute soak:** Full five-node fleet, periodic messages, one
    controlled transition/failover, continuous evidence, no reset
    condition.

10. **J. Merge decision:** If all pass: merge PR #139 line; if runtime
    failure: fix, re-anchor, restart at Matrix Pass 1.

11. **K. Josh handoff:** Package tested build + provenance + concise
    test instructions; begin Pennsylvania remote-network testing.

12. **L. Post-merge release work:** Resolve managed signing/distribution
    and other non-gate release hygiene separately.

# 17. Repository Reference Map

Use these paths as the first places to reconcile implementation,
historical evidence, and gate definitions. Some handoff files are
historical snapshots; always compare their stated SHA to the current
frozen candidate before acting.

| **Reference**          | **Path / anchor**                                                                 | **Use**                                                                                                                                                                        |
|------------------------|-----------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| PR #139               | tracking/pre-v040-tag-work                                                        | Open integration/tracking PR. Current head observed: e5284b7b7af194a53d4207f37d845cc16d2d7c56.                                                                                 |
| Runtime receipt logic  | core/src/iron_core.rs @ 7e527df0                                                  | Delivered/Read receipt clears outbox/drift retry state via mark_message_sent(). Also contains reconnect/outbox retry behavior that must be audited for finite terminal states. |
| Outbox model           | core/src/store/outbox.rs @ 7e527df0                                               | Defines queued message state, attempt counter, MAX_DELIVERY_ATTEMPTS, persistence, and queue behavior.                                                                         |
| Philosophy canon       | reference/PHILOSOPHY_CANON.md                                                     | Durable store-and-forward and eventual-consistency language; bounded-retention wording requires owner clarification to avoid finite delivery lifetime.                         |
| Original gate tracker  | TRACKING_PRE_V040_TAG_WORK.md                                                     | Defines G1-G6 and twice-reproducible requirement.                                                                                                                              |
| Orchestration state    | HANDOFF/gpt/PR139_ORCHESTRATION_STATE_2026-08-10.md                               | Historical five-node run state, roster, one-hour gate rules, and field evidence.                                                                                               |
| Windows takeover       | HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-10_WINDOWS_LANE.md                          | Historical live lane inventory, transport gotchas, blockers, and measurement cautions.                                                                                         |
| Receipt ticket         | HANDOFF/todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md             | Historical root cause showing receipt decoded without outbox convergence; later PR #139 runtime code appears to address it.                                                   |
| Request-response panic | HANDOFF/todo/P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md | Why fleet-growth stress is mandatory.                                                                                                                                          |
| Unified field plan     | HANDOFF/plans/FIVE_NODE_UNIFIED_TEST_PLAN_2026-08-09.md                           | Useful historical test-plan material; reconcile with this document’s updated topology and delivery semantics.                                                                  |
| Legacy harness         | scripts/run5.sh                                                                   | Outdated topology; source for reusable concepts only.                                                                                                                          |
| Unified capture        | scripts/capture_logs.sh                                                           | Existing log-capture primitive that can be reused/extended by lane collectors.                                                                                                 |

## 17.1 Additional reconciliation evidence added after the original reference

| Evidence | Anchor | Why it matters |
|---|---|---|
| Main head takeover packet | `d8ba796e2524128c868dfb06f301dfcf19333243` / `HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-10_WINDOWS_LANE.md` | Consolidates latest lane inventory, transport contract differences, blockers, and measurement gotchas. |
| Request-response panic reproduction | `9b9c27dc6249044eac236751eebb9c84e733b10c` | Shows address-level dedup alone did not prevent multi-address same-peer connection explosion; motivates per-peer cap verification. |
| NAT/DCUtR iteration plan | `12ec10cd7de9a07ddf35c319a5ee6822c2be63a5` | Confirms DCUtR path exists and warns against naive shared-external-IP blacklisting. |
| Relay measurement correction | `65b543c29f4e94d1f3bb6abe6505f0e0fb289206` | Retracts “zero relay attempts”; identifies parser blindness and shifts focus to wrong-hop/stale candidate ordering. |
| Historical PR #139 adversarial BLOCK | merge-base handoff `8646a2ca366efe1e96d3fbdd2f749b36c1932e5e` + `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md` | Requires exact-candidate security reconciliation before freeze. |
| Android local signing lineage | PR #139 head commit `e5284b7b7af194a53d4207f37d845cc16d2d7c56` | Confirms local debug keystore can upgrade installed Pixel in place; CI debug key differs. |
| Orchestration Control Plane v2 work | `HANDOFF/todo/ORCHESTRATE_STRICT_HARDENING.md`, `scripts/orchestrate_strict.py`, planned `agent/orchestration-control-plane-v2` | Governs delegation/isolation/resume mechanics; not itself a PR #139 runtime requirement. |


# Appendix A. Ready-to-Use PR Description

The following can be pasted into a future stacked PR once repository
write access is available.

**## Purpose  
**  
Reconcile PR #139 around one field-testable runtime path before the
five-node gate. This PR closes the gap between SCMessenger's durable
store-and-forward philosophy and finite terminal retry behavior, then
defines one shared Mac/Windows five-node qualification contract.  
  
**## Operator decisions  
**  
- AWS is a headless node used for rendezvous / relay / store-and-forward
custody, not a normal chat endpoint.  
- Freeze one exact runtime SHA only after known gate-breaking scope is
loaded.  
- Qualification is two complete G1-G6 matrix passes plus one continuous
60-minute five-node soak.  
- Mobile production-signing cleanup does not block this merge gate.  
- Goal after merge: a field-proven build suitable for remote testing
with Josh in Pennsylvania.  
  
**## Delivery contract  
**  
An accepted undelivered message remains an outstanding delivery
obligation indefinitely. Individual network attempts may fail and back
off, but transient/offline reachability must not become permanent
failure after a static attempt count. Delivery should be opportunistic
when a viable direct, LAN/BLE, relay, custody, reconnect, or
network-transition opportunity appears. A valid final delivery receipt
satisfies the obligation and stops further active transmission for that
message while history remains durable.  
  
**## Required changes  
**  
- Remove attempt-count terminal abandonment for transient/offline
delivery paths.  
- Preserve and test receipt -> outbox/drift convergence.  
- Clarify canonical philosophy: bounded resources are not bounded
delivery lifetime.  
- Add persistence/restart/offline/reconnect/opportunity-triggered
delivery regression coverage.  
- Rebuild five-node harness for Windows CLI + Android + AWS headless +
macOS CLI + physical iPhone.  
- Provide Windows and Mac lane collectors with one shared manifest, test
IDs, evidence schema, and scorer.  
- Enforce runtime provenance and identity-stability preflight.  
  
**## Field gate  
**  
1. Freeze exact runtime SHA after CI and adversarial review.  
2. Deploy identical source anchor to all five nodes without wiping
persistent identities.  
3. Matrix Pass 1: all six messaging endpoint pairs, both directions,
plus G2-G6 disruption/durability/provenance evidence.  
4. Matrix Pass 2: repeat complete matrix on the same runtime anchor.  
5. Continuous 60-minute full-fleet soak. Any panic, swarm death,
identity change, false delivery state, finite-attempt abandonment,
relay-fallback failure, or missing critical evidence invalidates the
gate.  
  
**## Merge condition  
**  
Merge the PR #139 integration line only after both matrix passes and
the one-hour soak pass on the same frozen runtime candidate.
Signing/distribution hardening remains tracked separately.

# Appendix B. Recommended PR #139 Bootstrap Comment

Use a short PR comment to point future orchestrators to this repository file rather than pasting the entire plan into the PR discussion:

```md
## PR #139 execution authority

The current authoritative field-gate execution plan is:

`HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md`

All orchestrators and workers must read that plan before PR #139 reconciliation, implementation, deployment, or qualification work.

Use the repository's canonical orchestration protocol/control plane. Reconcile historical findings against the exact current SHA before treating them as open or fixed. Do not infer readiness from this PR's older body/comments.

Required high-level sequence:
1. reconcile current PR/main evidence;
2. resolve MUST-FIX pre-freeze blockers;
3. independent validation + CI;
4. select and hard-freeze one exact runtime SHA;
5. deploy all five nodes preserving identity/state;
6. Matrix Pass 1;
7. Matrix Pass 2;
8. one continuous 60-minute soak;
9. evidence-backed GO/NO-GO for merge.

Runtime code changes after freeze invalidate the candidate.
```

# Appendix C. Definitions

| **Term**          | **Definition**                                                                                                                                               |
|-------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Accepted          | The local application/core has durably accepted the outbound message/obligation. Not proof of remote delivery.                                               |
| Outstanding       | Final delivery is not yet confirmed; the message remains eligible for future opportunity-driven delivery.                                                    |
| Attempt           | A concrete transmission/dial/send operation. Attempts may be bounded in rate and backoff, but not capped for the lifetime of a valid outstanding obligation. |
| Custody           | A headless/relay node has accepted a store-and-forward responsibility under the protocol. Intermediate, not final recipient delivery.                        |
| Delivered         | The recipient has durably received the message and a valid application-level receipt/convergence signal has satisfied the sender obligation.                 |
| Failed - terminal | Reserved only for genuinely irreversible conditions or explicit user/policy cancellation, not ordinary offline/unreachable state.                            |
| Frozen candidate  | The exact runtime source SHA/artifact provenance deployed to all five nodes for qualification. Runtime changes invalidate the freeze.                        |
| Matrix pass       | A complete G1-G6 evidence run across the current five-node topology.                                                                                         |
| Soak              | A continuous full-fleet stability period after functional matrices pass; here, 60 minutes.                                                                   |


> Reference precedence
> When this document conflicts with an older handoff snapshot about the
> agreed operator decisions in this session, use this document for the
> field-gate scope. For implementation facts, always inspect the exact
> frozen SHA before changing code.
