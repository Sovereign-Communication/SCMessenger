# Comprehensive Issues Analysis - SCMessenger (October 8, 2026)

## Executive Summary

This repository analysis consolidates findings from existing tracking documents (`SHIP_PLAN.md`, `REMAINING_WORK_TRACKING.md`, `POST_TAG_QUEUE.md`, `TRACKING_PRE_V040_TAG_WORK.md`), active todo items, and a quick code scan. The project is SCMessenger, a secure mesh messaging system with Rust core, Android/iOS apps, and CLI.

**Key Status:**
- v0.4.0 alpha is the near-term target with 7 exit criteria (D1-D7) defined in SHIP_PLAN.md
- Main development is focused on getting to a tagged release with proven message+receipt delivery between Windows CLI and Android handset
- Many historical tracking docs exist; SHIP_PLAN.md is the execution authority

## Repository Overview

- **Branch**: `preserve/shared-checkout-20261003` (current checkout)
- **Project**: 3417 indexed files, 205 detected test files
- **History**: 2037 commits, merged 184 PRs
- **Languages/Platforms**: Rust (core/cli), Kotlin/Android, Swift/iOS, TypeScript/JS (web/wasm), Python (scripts)

## Critical Issues & Blockers (Pre-v0.4.0)

### From SHIP_PLAN.md (Execution Authority)

| ID | Issue | Status | Priority |
|---|---|---|---|
| D1 | Main must be green (all CI lanes pass) | Blocked | Critical |
| D2 | Signed APK downloadable from GitHub release | Blocked (keystore config issue) | Critical |
| D3 | README explains product and install | OK (4,309 bytes) | Done |
| D4 | Message + receipt between Android handset ↔ Windows CLI | Partial (proven on dev rig, not on released APK cross-network) | Critical |
| D5 | No long-lived integration branches (#139 merged) | OK | Done |
| D6 | Transport racing demonstrated (fallback when first choice unavailable) | Blocked - `routing_peer_seen` has zero callers in codebase | Critical |
| D7 | Offline proximity messaging (no internet) | Not started | Critical |

### Active Critical Blockers

1. **D6 is unprovable**: `core/src/iron_core.rs:2704` has `routing_peer_seen` with zero callers. Without feeding routing on `ConnectionEstablished`, adaptive routing confidence stays at 0.0. Need to add routing feed in `swarm.rs:5486` handler (derive transport type from multiaddr, call peer_seen).

2. **Keystore signing issue** (D2): `SCMESSENGER_KEY_ALIAS` doesn't match the keystore for Android release builds. Release run `32817839477` failed at `packageRelease`. No signed APK has ever been published from this repo.

3. **Node boot/dial seeding**: CLI node never dials on boot (`connect_to_seed_peers()` only called from mobile bridge). Ledger sharing/gossip needs outbound dials to bootstrap.

4. **Two peer stores that don't converge**: gossiped core ledger vs local CLI peers.json (4,678 entries on Windows) - need unification.

## Code Quality Issues

### Technical Debt
- Many TODO/FIXME/HACK comments across codebase (docs, tests, generated code; fewer in production paths)
- Some unwrap() usage in production code paths (need to verify each case has proper error handling)
- Legacy/archived tracking docs and historical files (large number of .md files in HANDOFF/, docs/historical/)
- Build output in target/ directories (can be large - I20 in SHIP_PLAN mentions 41GB in target)

### Identified Issues from SHIP_PLAN Section 7 (Issue Ledger)

Key issues include:
- **I-01/I-02/I-03**: AWS node identity persistence - fixed in PR #259; mount path corrected in aws_deploy.sh
- **I-04-I-08**: Peer store duplication/pollution (addressed by T2 ticket)
- **I-09**: CLI never calls connect_to_seed_peers() (T1)
- **I-10**: routing_peer_seen has zero callers (T4, blocks D6)
- **I-12**: AWS node advertises many listeners (80/443/8080/9090) causing noise
- **I-14**: Driver watcher persistence - comment vs actual implementation mismatch
- **I-19**: DiagnosticsBundleFormatterTest.kt deleted but formatter class still exists
- **I-21**: ffi_surface.sh passes vacuously if generated sources missing
- **I-26-I-28, I-32-I-33**: Ledger disclosure/routing hint collision concerns (Rule-8 review items)
- **I-31**: Branch protection strict:true causes CI thrashing

## Current Active Todo Items (HANDOFF/todo)

Notable items (48 files total including queue and inbox references):
- `AND_STOP_START_FLOOD_AND_CANCEL_2026-09-23.md` - Android stop/start lifecycle
- `OUTBOX_NO_PERIODIC_RETRY_SWEEP_2026-09-23.md` - Outbox retry sweep (WASM needs wake-up mechanism per MT-07)
- `D2_SEED_DIAL_REDIAL_MISSING_SEED_PEER.md` through `D9_LIBP2P_EITHER_HANDLER_PANIC.md` - D-series issues
- Various P0/P1 Android/CLI/core issues

Also several V040_T* task files referenced in SHIP_PLAN G2 - need to verify if they exist in queue or need creation.

## Recommendations

### Immediate (Pre-v0.4.0) - Top Priority
1. **Fix D6** - Add routing feed on ConnectionEstablished in swarm.rs. This is a small, localized change (60-120 LOC per SHIP_PLAN) but blocks a critical gate.
2. **Resolve keystore/signing** (D2) - Generate fresh release keystore, set secrets, do dress rehearsal build. Document process in ANDROID_RELEASE_SIGNING.md (already exists, may need correction).
3. **Implement T1 (node boot seed dial)** and T2 (unify peer stores) per SHIP_PLAN G2 to enable automatic mesh rejoin on address changes.
4. **Run the 3-node gate** (Android handset + Windows CLI + AWS node) per G3 once blocking code is fixed.

### Short-term (Post-v0.4.0 tag considerations from POST_TAG_QUEUE)
- Address 13 open Dependabot PRs (7 vulnerabilities, 3 high) - S4-1
- External crypto audit for hybrid X25519+ML-KEM-768 - S4-2 (critical for security claims)
- Android last mile (162 unwired functions, 84 in MeshRepository.kt) - burn down only what's exercised
- iOS parity - separate lane
- Docker Integration Suite lane - decide if blocking or not

### Long-term / Hygiene
- Consolidate multiple tracking docs - there are many overlapping documents. SHIP_PLAN is authoritative for execution; consider archiving superseded ones.
- Clean up stale todo items in HANDOFF/archive (79 items moved per POST_TAG_QUEUE) - ensure disposition is documented
- Fix ffi_surface.sh to fail closed when generated sources missing (I-21)
- Address the Rule-8 review findings around ledger disclosure, Kademlia export, routing hints (I-26-I-28, I-32-I-33)
- Clean up large build artifacts and consider .gitignore for any build outputs that might leak
- Fix docs_sync_check.sh - fails on clean main (I-11)

## Files Referenced

Key documents to read for full context:
- [SHIP_PLAN.md](SHIP_PLAN.md) - Current execution plan (authoritative)
- [REMAINING_WORK_TRACKING.md](REMAINING_WORK_TRACKING.md) - Historical tracking (superseded for execution until v0.4.0 tag)
- [POST_TAG_QUEUE.md](POST_TAG_QUEUE.md) - Deferred work post-tag
- [TRACKING_PRE_V040_TAG_WORK.md](TRACKING_PRE_V040_TAG_WORK.md) - Pre-tag tracking
- [COMPREHENSIVE_ISSUES_ANALYSIS_20261008.md](COMPREHENSIVE_ISSUES_ANALYSIS_20261008.md) - This document (new)

## Conclusion

The project is close to v0.4.0 alpha with the core north star (message+receipt between Android and Windows CLI) proven on dev hardware. The main blockers are code/configuration fixes (D6 routing feed, keystore signing, peer store bootstrapping) rather than fundamental architecture issues. Once D1-D7 gates pass with evidence, tagging v0.4.0-alpha.1 is feasible.

**Estimated remaining effort**: ~1,300-2,530 LOC as per SHIP_PLAN section 6.5, mostly in peer store unification, boot seeding, and routing feed. With focused work on these specific items, the tag is achievable.