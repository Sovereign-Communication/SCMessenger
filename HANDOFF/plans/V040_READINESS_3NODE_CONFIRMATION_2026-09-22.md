# 0.4.0 Readiness -- 3-Node Confirmation Status

Status: Active (living readiness record; update on every state change)
Created: 2026-09-22
Owner: CTO seat; operator decisions marked [HUMAN]

## Node states (verified 2026-09-22 ~22:20 local)

| Node | Build | Health | Identity / peer | Notes |
|---|---|---|---|---|
| Windows CLI | `56d66f7` staged artifact (pid 5924) | healthy, `/health` 200 | `985a25f9...` / `12D3KooWD6vZ...` | **Pre-T-CONN-04 build.** Roll onto a CI artifact built from the merged PR #359 head as part of confirmation. |
| Always-on cloud node | `v040tconn04-lru` (OC-deployed) | healthy | `37eb7561...` / `12D3KooWGvCWJNo...` | Has the D1 fix; provenance closes when #359 merges and CI builds the same source. |
| OpenClaw cloud node | `v040tconn04-lru` (OC-deployed) | healthy | `821f161c...` / `12D3KooWDgLQ8jn...` | Runs `scm_bridge.py` app service (a service, not a node type). |
| Pixel 6a (Dx) | **CI APK, run 35674589429, CI-signed `a4390527`** | installed 22:17, app started | **WIPED** -- old triad `779e9ea3...` / `12D3KooWDx...` no longer on device | [HUMAN] Restore identity from backup (same triad returns) OR onboard fresh (new triad; contacts on all nodes must be updated). The Windows node still carries the old Dx triad as a contact either way. |

## PR readiness stack (as of this writing)

| PR | Content | CI | Blocker to merge |
|---|---|---|---|
| #351 | Stop-fix unbounded wait + **MESSAGE-STORE-LOCK-001 full fix** (`db475b23`) | green, re-running on new commit | none -- ready |
| #354 | Train/branch status | 19/19 green | none |
| #355 | WP3 inbound completeness | 33/33 green | WP1/WP2 JEV rows per train rules |
| #356 | WP4 delivery-truth watchdog predicate | 33/33 green | same |
| #357 | Canonical status surface | 19/19 green | none |
| #358 | Doctrine rows (CO-A-002/003, CO-E-002) | XML double-hyphen fixed `24e6d93d`, CI re-running | CI green |
| #359 | V040-T-CONN-04 two-tier per-peer cap + canonical docs | 6+ green on push | **[Rule-8] adversarial APPROVE from a non-author reviewer** |

## The 3-node confirmation (v0.4.0 gate, per SHIP_PLAN D4/D6/D7)

1. [HUMAN] Phone identity: restore from backup or onboard fresh (see table).
2. Land #359 (Rule-8), then **rebuild all three cloud/CLI nodes from the CI artifact of the merged head** -- this simultaneously closes the version-skew class and puts T-CONN-04 on every node. Windows node roll procedure: stage under `tmp/radio-candidates/<sha>/`, keep rollback, preserve identity (MiMo's roll procedure is the precedent).
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
- Its remaining defect tickets (D2, D3, D5/D6, D8) plus D9 are the OC-derived 0.4.0-readiness backlog in priority order: D9 (operator-named next), D2 (HIGH), D3 (MEDIUM), D5/D6 (LOW-MEDIUM, config-first), D8 (investigate).
