# SCMessenger Global Documentation Pass and Rollout Plan

Last updated: **2026-02-23**

> **Doctrine correction (2026-09-21, canonical outlier audit CO-E-002).**
> Decision 8 below ("third-party relays/bootstrap nodes are both valid")
> predates the ledger-sharing pivot and contradicts `AGENTS.md`: there is no
> dedicated relay-server tier (every node relays), and bootstrap address lists
> are deprecated in favour of **ledger-sharing-first discovery** (V050-B1/B2).
> Read decision 8 as: infrastructure is community-operated and any node may
> serve discovery and store-and-forward custody, with no privileged tier.
> Authoritative statements: `docs/BOOTSTRAP.md`, `docs/BOOTSTRAP_GOVERNANCE.md`.

This plan defines a full-repository documentation pass and release rollout strategy for one unified Android+iOS+Web product.

## 1) Locked Product Decisions

These decisions are now canonical unless explicitly superseded:

1. Primary target clients: **Android + iOS + Web together**.
2. Canonical identity for cross-platform persistence and user exchange: **`public_key_hex` (Ed25519 public key)**.
   - `identity_id` and `libp2p_peer_id` are derived/operational identifiers.
3. Bootstrap configuration model: **environment-configurable at startup + dynamically fetched**.
   - Hardcoded bootstrap nodes are fallback only.
4. Relay control semantics: **relay toggle remains user-controlled**, and when OFF it must block **all inbound and outbound relay messaging**.
   - Local history remains readable offline.
5. Privacy controls: **parity-first** across Android, iOS, and Web for all privacy toggles.
6. Web/WASM status: currently thinner than mobile and must be promoted to parity before global GA.
7. Rollout model: **global organic growth**, no region-targeted gate sequencing.
8. Infrastructure model: **community-operated** (self-hosted and third-party nodes are both valid; there is no privileged relay-server tier -- see the doctrine correction above).
9. Alpha language scope: **English-only**, with i18n scaffold kept as backlog.
10. First-run UX: **mandatory consent gate** describing security/privacy boundaries.
11. Storage: **policy-bounded retention** (no unbounded local growth).
12. Alpha launch scope: abuse controls and regional compliance mapping are **not blocking gates**.
13. Device strategy: **80/20 support matrix** (smallest support set covering most users).
14. Critical UX parity policy: no temporary lead platform for critical UX controls across Android, iOS, and Web.
15. Beta gate policy: anti-abuse controls must be implemented and validated before beta.

## 2) Scope of Documentation Pass

The documentation pass covers all tracked repository files (source, docs, scripts, manifests, platform projects).
Transient hidden build artifacts (for example `.build/` intermediates) are excluded from this documentation scope.

Inventory snapshot from `rg --files` at plan creation:

- Total tracked files: **356**
- Top-level heavy areas:
  - `core/` (88)
  - `android/` (89)
  - `iOS/` (55)
  - `docker/` (21)
  - `docs/` (22)
- Major file types:
  - Rust (`.rs`): 102
  - Markdown (`.md`): 77
  - Kotlin (`.kt`): 67
  - Swift (`.swift`): 53

File-by-file checklist artifact: `docs/historical/DOC_PASS_TRACKER.md`.

## 2.1) Coverage Baseline (Current Tracker State)

From `docs/historical/DOC_PASS_TRACKER.md`:

- Validated: 79
- Follow-up (mixed-status section tagging applied): 278
- Pending review: 0

Top-level pending concentration:

- none

Full-file coverage is complete. Remaining work is implementation and documentation remediation from `followup` findings.

## 2.2) Execution Waves for Complete Context Coverage

Wave ordering is based on risk + dependency criticality:

1. `docs/` + `(root)` canonical alignment
2. `core/` implementation and interface documentation pass
3. `android/`, `iOS/`, and `wasm/` tri-platform parity documentation pass
4. `cli/` and `scripts/` operator/runtime documentation pass
5. `docker/`, `reference/`, `SCMessengerCore.xcframework/`, and remaining support assets

Exit condition for this track:

- `docs/historical/DOC_PASS_TRACKER.md` has zero `pending` entries.
- All mixed-status docs have section-level markers and extracted canonical actions.

## 2.3) Progress Snapshot (Current)

- Wave 1 (`docs/` + `(root)`) is complete for first-pass classification:
  - `docs/` pending reduced to 0
  - `(root)` pending reduced to 0
- Wave 2 and Wave 3 are complete for first-pass classification:
  - `core/` pending reduced to 0
  - `android/`, `iOS/`, and `wasm/` pending reduced to 0
- Wave 4 and Wave 5 are complete for first-pass classification:
  - `cli/`, `docker/`, `scripts/`, `reference/`, `wasm/`, `mobile/`, `ui/`, and `SCMessengerCore.xcframework/` pending reduced to 0
- Remaining pending workload:
  - none (`docs/historical/DOC_PASS_TRACKER.md` pending = 0)

## 3) Documentation Pass Objectives

1. Establish one canonical source-of-truth chain for current state.
2. Apply section-level status tagging to outdated status/snapshot docs and point each section to canonical docs.
3. Ensure each module directory has a current local README.
4. Align docs with actual code behavior (not historical intent).
5. Encode rollout-critical semantics in canonical docs:
   - identity canonicalization
   - relay toggle behavior
   - bootstrap config strategy
   - Android/iOS/Web parity requirements

## 4) Workstreams

## Workstream A: Canonical Docs and Taxonomy

Deliverables:

- `README.md` (entrypoint)
- `DOCUMENTATION.md` (index + governance)
- `docs/REPO_CONTEXT.md` (cross-component architecture)
- `docs/CURRENT_STATE.md` (verified status)
- `REMAINING_WORK_TRACKING.md` (active backlog only)

Acceptance criteria:

- No conflicting “current status” claims outside canonical docs.
- Every mixed-status section has a clear pointer to canonical docs.

## Workstream B: Tri-Platform (Android + iOS + Web) Parity Docs

Deliverables:

- `android/README.md` updated for parity-first directives.
- `iOS/README.md` updated with authoritative path mapping:
  - active project path
  - legacy/reference paths
- `wasm/README.md` updated with parity directives, known gaps, and CI/runtime expectations.
- Shared parity checklist section in backlog.

Acceptance criteria:

- Android+iOS+Web docs describe identical identity, relay, and privacy semantics.
- iOS folder authority is unambiguous.

## Workstream C: Identity, Relay, Bootstrap Semantics

Deliverables:

- Canonical identity decision documented in architecture/context docs.
- Relay ON/OFF behavior documented in app and core-facing docs.
- Startup env + dynamic bootstrap fetch model documented with fallback behavior.

Acceptance criteria:

- No doc suggests inconsistent identity canonicalization.
- No doc suggests partial inbound/outbound behavior when relay is OFF.
- Bootstrap docs explicitly allow community-operated node topologies (self-hosted and third-party).

## Workstream D: Historical Snapshot Consolidation

Deliverables:

- Mixed-status docs tagged with section-level status markers (`[Current]`, `[Historical]`, `[Needs Revalidation]`).
- Key still-valid operational notes extracted into canonical docs/backlog.

Acceptance criteria:

- No blanket whole-document deprecation where mixed-value content exists.
- Historical context is preserved without obscuring current verified sections.

## Workstream E: Rollout Readiness and Global Launch Gates

Deliverables:

- Phase gates and go/no-go criteria for Android+iOS+Web rollout.
- Test matrix and operational checklist.
- CI gate alignment so tri-platform parity is continuously enforced.

Acceptance criteria:

- Launch criteria are measurable and tied to scripts/tests in repo.
- CI verifies Android, iOS, and Web build/runtime readiness in addition to Rust workspace checks.
- Rollout sequencing is documented as organic/global (not region-targeted cohorts).

## 5) Phase Plan (Execution)

## Phase 0: Baseline and Freeze (Day 0)

- Freeze canonical decision set (Section 1).
- Snapshot current build/test commands and results.
- Confirm file inventory (`docs/historical/DOC_PASS_TRACKER.md`).

## Phase 1: Canonical Docs Stabilization (Day 0-1)

- Normalize root and docs index flow.
- Ensure module READMEs exist and are linked.
- Add explicit Web/WASM status: experimental today, parity-critical before GA.

Gate:

- Canonical docs internally consistent and link-valid.

## Phase 2: Historical Deprecation Sweep (Day 1-2)

- Add section-level status tagging to status/report snapshots.
- Extract valid residual actions into `REMAINING_WORK_TRACKING.md`.

Gate:

- No stale “final/complete/current” claims remain untagged at section level.

## Phase 3: Android+iOS+Web Parity Doc Pass (Day 2-4)

- Verify and document parity for:
  - identity representation and exchange
  - relay ON/OFF enforcement (inbound + outbound)
  - privacy toggles across all clients
  - bootstrap config resolution order

Gate:

- Android, iOS, and Web docs align on behavior and known gaps.

## Phase 4: Rollout Validation Matrix (Day 4-6)

- Field matrix for mixed networks and NAT cases.
- Device matrix for Android WiFi Aware and iOS background behavior.

Gate:

- Minimum pass rate and no P0/P1 launch blockers.

## Phase 4.5: CI Gate Alignment (Day 4-6, in parallel)

- Extend/align CI so Android+iOS+Web parity checks are launch-gating.
- Keep existing Rust workspace checks and Docker suites, but add tri-platform parity gates.
- Ensure release criteria are enforceable by automation, not only manual runs.

## Phase 5: Organic Global Rollout

1. Internal dogfood cohort
2. Open global alpha (organic growth, no region gating)
3. Community scale-up with tri-platform reliability guardrails
4. GA readiness review and release

Each stage requires:

- test matrix pass
- incident response playbook ready
- rollback and feature-flag strategy documented

## 6) Rollout Gates (Go/No-Go)

## Required for Android+iOS+Web GA

1. Build and test baseline:
   - `cargo test --workspace` pass
   - Android build and smoke pass
   - iOS simulator + device-target compile pass
   - Web browser-runtime smoke/test pass
   - CI workflows include and enforce the Android+iOS+Web gates above
2. Relay semantics:
   - ON: inbound/outbound functional
   - OFF: inbound/outbound blocked, local history readable
3. Identity consistency:
   - `public_key_hex` canonical end-to-end in storage/export/import docs
4. Bootstrap behavior:
   - env override + dynamic fetch + fallback path verified
   - community-operated topology support documented and validated
5. Privacy parity:
   - toggles mapped and behaviorally equivalent across Android/iOS/Web
6. Observability:
   - connection/discovery failure classes documented with operator actions
7. Consent + retention policy:
   - first-run consent gate implemented on Android/iOS/Web
   - bounded retention policy implemented and verified
8. Support policy:
   - 80/20 support matrix documented and enforced in release criteria
9. Critical UX parity:
   - critical UX controls and behavior are parity-verified across Android+iOS+Web with no temporary lead-platform exceptions
10. Beta anti-abuse gate:
   - abuse-control protections are enabled and validated before beta release

## 7) Risks and Mitigations

1. Historical doc drift reappears
   - Mitigation: enforce canonical chain; tag snapshots on creation.
2. Platform behavior divergence
   - Mitigation: parity checklist in PR review for Android+iOS+Web changes.
3. Bootstrap infrastructure instability
   - Mitigation: fallback node set + cached bootstrap list + timeout strategy.
4. NAT/discovery regressions in field
   - Mitigation: staged rollout with transport telemetry and quick rollback.
5. Stale TODO/FIXME backlog drifts from real implementation state
   - Mitigation: recurring TODO/FIXME audit pass with canonical backlog sync.

## 8) Ownership Model

- Architecture + canonical docs: core maintainers
- Android parity docs: Android owner
- iOS parity docs: iOS owner
- Cross-platform semantics (identity/relay/bootstrap/privacy): joint sign-off required
- Rollout go/no-go: release lead + platform owners

## 9) Immediate Next Actions

1. Complete component-status banner sweep for stale status docs.
2. Update Android+iOS+Web docs with parity directives and active paths.
3. Update canonical docs with locked identity/relay/bootstrap/WASM decisions.
4. Convert unresolved discovery/remediation notes into backlog items with owners and test criteria.
5. Add Android+iOS+Web CI gating strategy to close the current Rust-only `ci.yml` gap.
6. Run TODO/FIXME accuracy sweep and sync backlog wording with current code paths.
