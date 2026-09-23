# 0.4.0 Readiness -- 3-Node Confirmation Status

Status: Active (living readiness record; update on every state change)
Created: 2026-09-22
Owner: CTO seat; operator decisions marked [HUMAN]

## Node states (refreshed 2026-09-23; first table 2026-09-22 ~22:20 local)

| Node | Build | Health | Identity / peer | Notes |
|---|---|---|---|---|
| Windows CLI | **`1ec0c242` CI artifact, PR #361 head** (pid 8324, supervised `run_node_supervised.ps1`) | healthy, `/health` 200 | `985a25f9...` / `12D3KooWD6vZ...` | Restored 2026-09-23 from CI artifact `windows-cli-1ec0c242...` after the Freebuff restart killed the old unmanaged node. **D9-DEGRADE confirmed live** on this node (2 side-mismatch events 08:00-08:02Z logged as drops, where the old build panicked) + D1 two-tier cap active. |
| Always-on cloud node | `v040d9degrade-672dffcb-worktree` (OC-deployed 2026-09-22 10:42Z) | healthy | `37eb7561...` / `12D3KooWGvCWJNo...` | D9 degrade + D1. Provenance closes when #361 merges and CI builds the same source. |
| OpenClaw cloud node | `v040d9degrade-672dffcb-worktree` (OC-deployed 2026-09-22 10:43Z) | healthy | `821f161c...` / `12D3KooWDgLQ8jn...` | Runs `scm_bridge.py` app service (a service, not a node type). D9 degrade live-observed here first (~2-3/hour, logged not panicked). |
| Pixel 6a (Dx) | **CI APK, run 35674589429, CI-signed `a4390527`** | installed 22:17, app started | **WIPED** -- old triad `779e9ea3...` / `12D3KooWDx...` no longer on device | [HUMAN] Restore identity from backup (same triad returns) OR onboard fresh (new triad; contacts on all nodes must be updated). The Windows node still carries the old Dx triad as a contact either way. |

## PR readiness stack (refreshed 2026-09-23 via `gh pr view` x10; first table 2026-09-22)

| PR | Content | CI | Blocker to merge |
|---|---|---|---|
| #351 | Stop-fix unbounded wait + **MESSAGE-STORE-LOCK-001 full fix** (`db475b23`) | green | none -- ready |
| #354 | Train/branch status | 19/19 green | none |
| #355 | WP3 inbound completeness | 33/33 green | WP1/WP2 JEV rows per train rules |
| #356 | WP4 delivery-truth watchdog predicate | 33/33 green | same |
| #357 | Canonical status surface | 19/19 green | none |
| #358 | Doctrine rows (CO-A-002/003, CO-E-002) | green (XML `--` fix `24e6d93d` held) | none |
| #359 | V040-T-CONN-04 two-tier per-peer cap + canonical docs | green on last push | **CONFLICTING with main** (parallel doctrine-file edits: AGENTS.md rule 17, CTO_STATE, freebuff README) + **[Rule-8]**. Deconfliction is operator/owner work: the main-checkout working tree carries the OC session's Cargo/vendor WIP (byte-identical to #361's pushed ref), so this lane cannot stash/checkout those files (rules 11/12). |
| #360 | Version floor: refuse harness checkouts older than 0.3.3 | green | **CONFLICTING with main** (same parallel-edit set). Same deconfliction caveat as #359. |
| #361 | **D9 vendored libp2p-swarm graceful degrade + D1 two-tier cap** (`d9-libp2p-degrade`) | green | **[Rule-8] adversarial APPROVE from a non-author reviewer** (panel review ran, no REJECT-level finding, panels are not the gate; this lane authored the patch and cannot self-sign). Mergeable. |
| #362 | Claude-lane docs: Harness lane update in CTO/CEO state (append-only) | green | none |

## The 3-node confirmation (v0.4.0 gate, per SHIP_PLAN D4/D6/D7)

1. [HUMAN] Phone identity: restore from backup or onboard fresh (see table). STILL PENDING as of 2026-09-23.
2. Land the transport PR (#361 carries D9+D1; #359 is the standalone T-CONN-04 -- if both land, #359's remaining value is its docs), then **rebuild all three cloud/CLI nodes from the CI artifact of the merged head** -- this simultaneously closes the version-skew class and puts T-CONN-04 on every node. Windows node roll procedure: stage under `tmp/radio-candidates/<sha>/`, keep rollback, preserve identity (MiMo's roll procedure is the precedent; executed 2026-09-23 for the D9 build, see node table).
3. Re-seed contacts for the phone's (possibly new) triad on the Windows node and both cloud nodes.
4. Run the three gates, scoring receiver-side only (decrypt + durable history + receipt; never transport ACKs/UI counters):
   - **D4** message + receipt across the three nodes;
   - **D6** transport racing (deliver with first-choice transport down);
   - **D7** offline proximity (no internet).
5. Record evidence under `tmp/evidence/<date>/` and reference it here.

## CI-primary findings from today's fresh install

- **S1-1 signing is now PROVEN blocking, not just a release convenience.** CI debug APKs are signed with ephemeral runner keystores, so every CI artifact forces a wipe-install (identity loss) instead of an upgrade. Until a stable signing key is wired, the phone cannot take CI builds without identity loss. [HUMAN] Provide the keystore/secrets (S1-1) -- it converts every future install from destructive to routine.
- CI artifact pipeline works end to end: run 35674589429 -> download -> install, per `docs/runbooks/CI_APK_TO_PHONE.md`.

## OC WIP watch protocol (standing)

- The OC session writes to `~/Documents/GitHub/OC/` (not a git repo). Watch = compare file mtimes against the last recorded value (`SCM_NODES_AUDIT.md` @ 2026-09-21T20:01 as of this doc); new mtime = read the changed files before anything else.
- Standing dispositions: lane audit APPROVED with conditions (`HANDOFF/review/OC_LANE_AUDIT_2026-09-22.md`); D1 fix on PR #359; D2/D3/D5-D6/D8 tickets filed; D9 next with root-cause anchor pinned; `vendor/libp2p-swarm-0.48.0` is its pristine pinned-source reading copy -- keep while D9 is open, delete after, never commit.

## 2026-09-23 refresh (this lane)

- **D9 is fixed AND deployed to all three CLI nodes** (both cloud nodes 10:42-10:43Z 09-22; Windows node restored from the #361 CI artifact 09-23). Live degradation confirmed on all three (logged `D9-DEGRADE` drops, zero panics since). Supersedes the vendor disposition above: the vendored patch is now tracked source on `d9-libp2p-degrade`/PR #361, pending Rule-8.
- **New ticket D10** (`HANDOFF/todo/D10_PARSE_METHOD_SCANNER_NOISE_PUBLIC_PORTS.md`): the node binds public `0.0.0.0:80/443/8080/9002/9090`; internet TLS probes against the HTTP listener produce recurring `hyper::Error(Parse(Method))` noise (~5/h observed over 18h on the old log). MEDIUM: noise + unintended exposure.
- **Unmerged-stack risk**: #359/#360 conflict with main; #361 mergeable. Fastest path to a coherent 0.4.0: merge #361 (Rule-8) first, then deconflict #359/#360 onto the new main.
- Harness JEV dogfood (3 extensions, `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md`): issue-sort 5/5 + live-line batches; two upstream Harness findings drafted (`HANDOFF/harness/UPSTREAM_JEV_ISSUES_DRAFT_2026-09-23.md`, operator files if approved).
- Its remaining defect tickets (D2, D3, D5/D6, D8) plus D9 are the OC-derived 0.4.0-readiness backlog in priority order: D9 (operator-named next), D2 (HIGH), D3 (MEDIUM), D5/D6 (LOW-MEDIUM, config-first), D8 (investigate).
