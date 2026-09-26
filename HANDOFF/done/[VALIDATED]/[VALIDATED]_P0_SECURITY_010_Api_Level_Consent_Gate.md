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
# MODEL: glm-5.1:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_SECURITY_010_Api_Level_Consent_Gate

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 security
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P0.1 (Consent gate is UI-only) + planfromclaudeforhermes 2 Phase B.4
**Depends on:** P0_BUILD_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P0.1: "Consent gate blocks `initialize_identity()` in Rust core until platform confirms consent (currently consent is UI-only, not gated at API level)."

UI-side consent exists in:
- Android: `OnboardingScreen.kt` with `ConsentView`
- iOS: `OnboardingFlow.swift` with 6-step onboarding
- WASM: `ui/app.js` onboarding modal
- CLI: `scm init` gates identity creation

But the Rust core's `initialize_identity()` doesn't enforce  the platform can call it without UI consent, bypassing the gate.

## Scope (~90 LoC across 3 files)

### Part A: Add ConsentRequired error variant (LOC: ~15)

In `core/src/iron_core.rs` or a new `core/src/consent.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConsentError {
    #[error("Consent required: {reason}")]
    Required { reason: String },
    #[error("Consent token invalid or expired")]
    InvalidToken,
    #[error("Consent already granted")]
    AlreadyGranted,
}
```

### Part B: Gate initialize_identity (LOC: ~50)

In `core/src/iron_core.rs`:

```rust
pub struct IronCore {
    // ... existing fields ...
    consent_state: Arc<RwLock<ConsentState>>,
}

pub struct ConsentState {
    pub granted: bool,
    pub granted_at: Option<SystemTime>,
    pub token: Option<[u8; 32]>,  // random per-session
}

impl IronCore {
    pub fn confirm_consent(&self) -> Result<[u8; 32], ConsentError> {
        let mut state = self.consent_state.write();
        if state.granted { return Err(ConsentError::AlreadyGranted); }
        let token: [u8; 32] = random();
        state.granted = true;
        state.granted_at = Some(SystemTime::now());
        state.token = Some(token);
        Ok(token)
    }

    pub fn initialize_identity(&self, nickname: &str) -> Result<Identity, Error> {
        let state = self.consent_state.read();
        if !state.granted {
            return Err(Error::Consent(ConsentError::Required {
                reason: "User must confirm consent before creating identity".into()
            }));
        }
        drop(state);
        // ... existing identity creation logic ...
    }
}
```

### Part C: Platform-side confirmation (LOC: ~25)

In `core/src/mobile_bridge.rs` (Android/iOS) and `core/src/wasm_support/rpc.rs` (WASM):

- Add `confirm_consent()` to UniFFI surface
- Add `confirm_consent` JSON-RPC method to WASM RPC
- CLI: `scm init` already calls confirm  ensure it goes through the new API

## File Targets

- `core/src/iron_core.rs` [EDIT  add ConsentState, confirm_consent, gate initialize_identity]
- `core/src/mobile_bridge.rs` [EDIT  add confirm_consent to UniFFI]
- `core/src/wasm_support/rpc.rs` [EDIT  add confirm_consent JSON-RPC handler]
- `core/src/lib.rs` [EDIT  re-export ConsentError]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib -- consent
# CLI smoke: should fail without consent
cargo run -p scmessenger-cli -- identity create test-id 2>&1 | grep "Consent required"
# Then with consent
cargo run -p scmessenger-cli -- consent confirm
cargo run -p scmessenger-cli -- identity create test-id  # Should succeed
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: initialize_identity fails with ConsentRequired before confirm, succeeds after, double-confirm returns AlreadyGranted, gated across all platform bridges
3. `scm identity create` without prior `scm consent confirm` returns clear error
4. Android: ConsentView completion calls `confirm_consent()` then enables identity creation
5. iOS: 6-step onboarding completion calls `confirm_consent()` 
6. WASM: onboarding modal close calls `confirm_consent()` via JSON-RPC
7. Commit: `security: v0.2.1 API-level consent gate  initialize_identity returns ConsentRequired`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_BUILD_001] [CROSS_PLATFORM]
