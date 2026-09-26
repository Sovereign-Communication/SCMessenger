<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: kimi-k2-thinking:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_RELEASE_001_v0.2.1_Complete_Notes

**Status:** VERIFIED REMAINING WORK
**Agent:** gatekeeper-reviewer
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 release
**Source:** planfromclaudeforhermes 2 Phase G.3
**Depends on:** ALL other v0.2.1 tasks must be DONE

---

## Verified Gap

No `RELEASE_NOTES_v0.2.1.md` exists. v0.2.1 is the current version in `Cargo.toml` but no formal release notes describe what shipped.

## Scope (~100 LoC of markdown, plus verification)

## File Targets

- `RELEASE_NOTES_v0.2.1.md` [NEW]

## Template (must follow this structure)

```markdown
# SCMessenger v0.2.1  Release Notes

**Release date:** YYYY-MM-DD
**Status:** Stable alpha  verified on Android (Pixel 6a), iOS (physical device), WASM (Firefox/Chrome), CLI (Windows/Linux/macOS)

##  Headline

Sovereign P2P messaging with full multi-transport routing, privacy (onion/cover/padding/timing), and tamper-evident audit chain. This release closes the [dormant module gap](#dormant-modules-now-wired)  Drift, Mycorrhizal Routing, and Privacy modules are now active in production.

##  New Features

### Drift Protocol
- 8 implemented files now active in production send path
- SyncSession triggered on PeerDiscovered for set reconciliation
- DriftEnvelope replaces legacy bincode for all messages
- DriftFrame handles fragmented messages > 16KB

### Mycorrhizal Routing
- All 12 routing files (~5,170 LoC) now active
- Messages route through OptimizedRoutingEngine (not direct/relay)
- Adaptive TTL based on priority + peer distance
- Negative cache blocks unreachable peers
- Reputation-weighted multipath selection

### Privacy (toggleable per `scm config set privacy.<flag>`)
- Onion routing (3 hops by default)
- Cover traffic (1 msg/30s, scalable by intensity)
- Padding to power-of-2 buckets
- Timing obfuscation (priority-aware)

### Security
- IdentityBackupV2 with Argon2id + XChaCha20-Poly1305
- Audit log entries for all identity operations
- Sled compaction on shutdown + low-disk graceful degradation
- API-level consent gate (initialize_identity returns ConsentRequired)

### Performance
- LZ4 compression for messages > 256 bytes
- Outbox flush on PeerDiscovered (Android + WASM)
- BLE scanner stale cache cleanup

##  Bug Fixes

(Pulled from `git log --oneline` between v0.2.0 and HEAD)

##  Platform Status

| Platform | Status | Verification |
|----------|--------|--------------|
| Android (Kotlin, Compose) |  Stable | Pixel 6a + emulator |
| iOS (Swift) |  Stable | Physical device required |
| WASM (browser thin client) |  Stable | Firefox + Chrome |
| CLI (Windows/Linux/macOS) |  Stable | Cross-platform builds |

##  Stats

- 920+ tests passing
- ~2,800 LoC added in this revision
- 4 P0 security gaps closed
- 4 dormant module families wired

##  Known Issues

(Pulled from docs/historical/MASTER_BUG_TRACKER.md that are still open)

##  Install

(Pulled from docs/INSTALL.md, current as of HEAD)

##  Upgrade from v0.2.0

- Identity backups in V1 format are auto-migrated to V2 on first import
- Re-onboarding may be required if running with stale auto-backup data (Android)
- CLI config keys for privacy are new; defaults are conservative

##  Acknowledgments

Built with the SCMessenger swarm: rust-coder, architect-planner, implementer, worker, gatekeeper-reviewer, and the Hermes orchestrator.

`<3` from the minimax-m3 family  Claude Code + Hermes.
```

## Build Verification Commands

```bash
# Confirm file exists and has the right structure
ls -la RELEASE_NOTES_v0.2.1.md
wc -l RELEASE_NOTES_v0.2.1.md  # Should be ~100 lines
head -20 RELEASE_NOTES_v0.2.1.md
```

## Acceptance Gates

1. `RELEASE_NOTES_v0.2.1.md` exists at repo root
2. File follows the template above (all sections present)
3. All "Pulled from X" placeholders replaced with actual content
4. The 17 success criteria from `planfromclaudeforhermes` 7 are all referenced as 
5. Commit: `release: v0.2.1-complete release notes`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KIMI_K2_THINKING] [DEPENDS_ON: ALL_OTHER_TASKS_DONE] [GATEKEEPER_ROLE]
