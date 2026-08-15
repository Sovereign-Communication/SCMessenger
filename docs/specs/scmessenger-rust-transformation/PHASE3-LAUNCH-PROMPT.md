# Phase 3 Launch Prompt

Copy and paste this into a new conversation window to start Phase 3:

---

I'm continuing the SCMessenger Rust Transformation project. Phase 1 (Async Hygiene) and Phase 2 (Protocol Hardening) are complete. Please execute Phase 3: Observability - Metrics, Tracing, Health Checks.

**Context:**
- Spec location: `docs/specs/scmessenger-rust-transformation/`
- Phase 1 status: [OK] COMPLETE (locks optimized, HTTP API modernized)
- Phase 2 status: [OK] COMPLETE (schema versioning, peer proofs, rate limiting)
- Current Rust version: 1.95.0
- Build status: All tests passing (871 passed, 0 failed, 8 ignored)
- See `PHASE2-COMPLETION-SUMMARY.md` for Phase 2 details

**Phase 3 Objectives:**
1. Add metrics collection for sync operations
2. Implement distributed tracing for message flow
3. Add health check endpoints for monitoring
4. Verify zero regression

**Instructions:**
- Read `docs/specs/scmessenger-rust-transformation/PHASE3-KICKOFF.md` for detailed instructions
- Follow tasks in `docs/specs/scmessenger-rust-transformation/tasks.md` (Phase 3 section)
- Reference `HANDOFF/scmessenger_rust_implementation_prompt.md` for implementation details
- Update task status as you progress
- Maintain zero-regression (all tests must pass)

**Important Notes:**
- Phase 0 (unwrap removal) was not completed - see `PHASE0-UNWRAP-REMEDIATION-NEEDED.md`
- Clippy warnings (~917) are mostly unwrap-related and documented as future work
- Focus on Phase 3 objectives, not pre-existing technical debt

Please start with Task 3.1: Add Metrics Collection for Sync Operations.

---

## Quick Reference

**Key Files:**
- `docs/specs/scmessenger-rust-transformation/PHASE3-KICKOFF.md` - Complete Phase 3 spec
- `docs/specs/scmessenger-rust-transformation/tasks.md` - Task checklist
- `core/src/drift/rate_limit.rs` - Reference implementation pattern
- `core/src/drift/sync.rs` - Sync protocol to instrument
- `core/src/drift/store.rs` - Store to add health checks

**Phase 3 Tasks:**
1. Task 3.1: Add Metrics Collection (~150 LoC)
2. Task 3.2: Implement Distributed Tracing (~150 LoC)
3. Task 3.3: Add Health Check Endpoints (~100 LoC)
4. Task 3.4: Phase 3 Verification Gate (~10 LoC)

**Success Criteria:**
- [OK] Metrics collection added to sync operations
- [OK] Distributed tracing implemented
- [OK] Health check endpoints created
- [OK] All tests pass (cargo test)
- [OK] All crates compile (cargo check)
- [OK] Code formatted (cargo fmt)
