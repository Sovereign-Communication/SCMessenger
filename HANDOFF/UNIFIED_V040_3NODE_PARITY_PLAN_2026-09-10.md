# Unified v0.4.0 3-node parity plan

Status: Active
Created: 2026-09-10
Owner: Operator session (MiMoSCMessengerFresh)
Branch target: `unified/v040-3node-parity`

## Why this exists

Three lines of valuable work diverged from `b2d8d126` and never recombined:

| Line | Tip | PR | What it carries |
|---|---|---|---|
| `main` | `c5b7c530` | (trunk) | T1-T13 (#261-#277): seed dial, F-DHT, T14 ephemeral-port, peer-ledger unify, beach-join docs |
| `cto/v040-candidate-2026-09-02` | `5fe7d664` | #272 | Architecture pass, outbox drop-hop (#276), dead-mark (#273), never-dial-connected (#274), Android inbound/FFI/ANR (#278), mobile_bridge + iron_core expansion |
| `cto/t2-disk-ruling-2026-08-31` | `66a09cbc` | #279 | Live 3-node campaign: D10 poison-listener, T14 configured-external, BLE scanner, bootstrap seed-tier merge, lifecycle pause removal, composition guard |

Neither side alone is the complete app. Operator 3-node test PASSED on the CTO tip; several outbox/FFI/custody defects were fixed only on the v040-candidate line.

## Candidate selection

- **Baseline (behavioral):** `cto/t2-disk-ruling-2026-08-31` @ `66a09cbc` — last proven 3-node live tree.
- **Must absorb:** `origin/main` @ `c5b7c530`.
- **Must absorb:** `origin/cto/v040-candidate-2026-09-02` @ `5fe7d664`.
- **Out of scope for this unify:** iOS/macOS (v0.5.0), dependabot PRs, `feature/v040-v050-completion-sprint` (superseded formatting/restore WIP), `fix/seeding-security-remediation-v040` (largely already landed in main via earlier security commits).

## Merge order

1. Create `unified/v040-3node-parity` from CTO tip `66a09cbc`.
2. Merge `origin/main` (resolve, gate compile).
3. Merge `origin/cto/v040-candidate-2026-09-02` (resolve, gate compile).
4. Do **not** merge dependabot or draft PRs into this candidate.

## Conflict precedence

| Area | Prefer | Reason |
|---|---|---|
| Android lifecycle / BLE / bootstrap merge / diagnostics share | CTO (`66a09cbc`) | Proven on operator 3-node test + live APK |
| `core/src/store/outbox.rs`, `mobile_bridge.rs`, `iron_core.rs`, FFI snapshots, dial_policy dead-mark/never-dial | v040-candidate (`5fe7d664`) | Defects only fixed on that line |
| T1-T13 routing/ledger/seed-dial already on main | main | Already CI-merged trunk work |
| `swarm.rs` / `observation.rs` / `ledger_entry.rs` / `local.rs` | **Semantic merge** | Both sides touched; take CTO transport admission/D10 + v040 dead-mark/nobody-dial + main T14/T13 where not superseded |
| HANDOFF / docs / SHIP_PLAN | Union or latest CTO narrative | Keep campaign record intact |

## Build / parity gates before deploy

1. `cargo check -p scmessenger-core -p scmessenger-cli` (and workspace if disk allows).
2. Targeted tests: address observation, configured-external, dial policy, relay custody if present.
3. Android: `:app:compileDebugKotlin` (or assembleDebug if operator wants APK).
4. Record **one** candidate SHA + per-node artifact hashes.

## 3-node parity deploy (after gates)

| Node | Artifact | Action |
|---|---|---|
| Windows CLI | `scmessenger-cli.exe` from unified SHA | Stop old node, stage rollback outside `target/`, relaunch with recorded env/config |
| AWS cloud node | Docker image from unified SHA | Redeploy via `.codebuff_deploy/aws/`, capture `sha256` + `/version` |
| Pixel 6a | Debug/release APK from unified SHA | **Operator** installs via adb; record `pm path` hash |

Success: all three `/version` or app diagnostics report the **same git hash**, then re-run the 3-node message + receipt matrix (baseline / cell-only / BLE after radio reboot).

## Explicit non-goals

- Tagging `v0.4.0` (CEO/operator decision after green).
- Rule-8 approval (still required for merge to main; this branch is the unification candidate).
- iOS/macOS work.
