# Agent Task: Integrate Security Tooling

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Delegated To:** triage-router (gemini-3-flash-preview:cloud)  
**Priority:** P1 (Phase 3D prerequisite)  
**Status:** pending_agent_dispatch  
**Depends On:** Phase 1A compilation baseline passing

## Objective
Integrate 3 automated security tools to complement the manual adversarial review protocol in Phase 3.

## Tasks

### 1. cargo-deny — Supply Chain Audit
- Install: `cargo install cargo-deny`
- Create `deny.toml` config in repo root
- Run: `cargo deny check advisories`
- Document any findings in HANDOFF/done/

### 2. cargo-audit — CVE Scan  
- Install: `cargo install cargo-audit`
- Run: `cargo audit`
- Document any CVEs found

### 3. miri — Unsafe Block Validation
- Install via rustup: `rustup +nightly component add miri`
- Target files with `// SAFETY:` comments in:
  - core/src/crypto/
  - core/src/transport/
- Run: `cargo +nightly miri test` on applicable test targets

## Evidence Log (Orchestrator)

**Status: PARTIAL — cargo-audit executed; cargo-deny and miri pending.**
**Date: 2026-05-01**
**Agent: Master Orchestrator (kimi-k2.6:cloud)**

### cargo-audit Results
```
14 vulnerabilities found!
10 allowed warnings found
```

| Severity | Crate | Version | Issue |
|----------|-------|---------|-------|
| **HIGH (8.7)** | `quinn-proto` | 0.11.13 | Denial of service in Quinn endpoints |
| Medium | `hickory-proto` | 0.24.4 | CPU exhaustion O(n²) name compression |
| Medium | `ring` | 0.16.20 | AES panic when overflow checking enabled |
| Medium | `rustls-webpki` | 0.101.7, 0.102.8, 0.103.9 | Name constraint acceptance bugs + CRL parsing panics (4 CVEs across 3 versions) |
| Low | `bincode` | 1.3.3 | Unmaintained |
| Low | `core2` | 0.4.0 | Unmaintained, yanked |
| Low | `fxhash` | 0.2.1 | Unmaintained |
| Low | `instant` | 0.1.13 | Unmaintained |
| Low | `ring` | 0.16.20 | Versions prior to 0.17 unmaintained |
| Low | `lru` | 0.11.1 | Unmaintained |

### Delta from Previous Audit (2026-04-29)
- **New finding**: `hickory-proto` CPU exhaustion (not in prior report)
- **Increased count**: `rustls-webpki` expanded from 4 to 8 advisories (multiple version instances)
- **Carry-forward**: `quinn-proto` HIGH DoS, `ring` AES panic remain unpatched

### Remaining Actions
- [ ] `cargo-deny` install + `deny.toml` creation + advisories check
- [ ] `miri` install + unsafe block validation on `core/src/crypto/` and `core/src/transport/`
- [ ] Vulnerability remediation plan (dependency upgrades or documented justification)

### Review Gate
- [ ] Wiring-verifier approval required before moving to `done/`.

## Success Criteria
- All three tools installed and configured
- Zero actionable CRITICAL/HIGH findings (or documented with justification)
- Handoff report filed to HANDOFF/done/security_tooling_integration.md
