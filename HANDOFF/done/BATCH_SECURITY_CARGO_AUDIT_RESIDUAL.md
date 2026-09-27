# BATCH_SECURITY_CARGO_AUDIT_RESIDUAL

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** RESEARCH COMPLETE  BLOCKED (requires libp2p major version upgrade)
**Agent:** rust-coder_1779261571 (glm-5.1:cloud)
**Budget:** 1800s (MIXED tier)
**Source:** REMAINING_WORK_TRACKING.md 2026-04-29 audit entry, Cargo.lock current state
**Completed:** 2026-05-20

---

## Verified Gap

Cargo.lock still contains vulnerable transitive dependency versions. The `quinn-proto` HIGH vulnerability is **already fixed** (0.11.14 >= 0.11.14), but `ring` and `rustls-webpki` remain.

**Current Cargo.lock State (verified 2026-05-20):**

| Crate | Version | Status | CVE/Risk |
|-------|---------|--------|----------|
| `quinn-proto` | 0.11.14 | **FIXED** | HIGH DoS  upgrade satisfied |
| `ring` | 0.16.20 | **VULNERABLE** | MEDIUM AES panic via `rcgen` / `libp2p-tls` transitive |
| `ring` | 0.17.14 | Safe | Used by newer dep chain |
| `rustls-webpki` | 0.101.7 | **VULNERABLE** | Name constraint + CRL parsing issues |
| `rustls-webpki` | 0.102.8 | Safe | |
| `rustls-webpki` | 0.103.13 | Safe | |
| `instant` | 0.1.13 | **Unmaintained** | Warning only |
| `bincode` | 1.3.3 | **Unmaintained** | Warning only |

**Transitive origin verified (CORRECTED 2026-05-20):**
- `ring 0.16.20` is pulled in via `rcgen 0.11.3` -> `ring 0.16.20` <- `libp2p-tls 0.4.1` <- `libp2p-quic 0.10.3` <- `libp2p 0.53.2`
- `rustls-webpki 0.101.7` is pulled in via `libp2p-tls 0.4.1` directly, same chain: <- `libp2p-quic 0.10.3` <- `libp2p 0.53.2`
- `instant 0.1.13` is pulled in via `libp2p 0.53.2` and multiple `libp2p-*` sub-crates
- `bincode 1.3.3` is a direct workspace dependency (workspace `Cargo.toml` line 46)

---

## Research Findings (Part A  COMPLETE)

### Dependency Chain Analysis

Both MEDIUM vulnerabilities are **structurally locked** by semver constraints in the `libp2p` dependency tree:

```
libp2p 0.53.2
  -> libp2p-quic 0.10.3 (requires libp2p-tls ^0.4.0)
    -> libp2p-tls 0.4.1
      -> rcgen ^0.11.3        (semver locks to 0.11.x)
        -> ring 0.16.20       (VULNERABLE  MEDIUM AES panic)
      -> rustls-webpki ^0.101.4  (semver locks to 0.101.x)
        -> rustls-webpki 0.101.7 (VULNERABLE  name constraint + CRL parsing)
      -> ring ^0.17.8         (safe, but rcgen also pulls ring 0.16.20)
```

### Upgrade Path Assessment

| Crate | Current | Latest | Can upgrade within libp2p 0.53? | Notes |
|-------|---------|--------|--------------------------------|-------|
| `libp2p` | 0.53.2 | 0.56.0 | N/A (requires major bump) | Breaking API changes between 0.53 and 0.56 |
| `libp2p-tls` | 0.4.1 | 0.6.2 | No  locked by libp2p-quic ^0.10.2 which requires ^0.4.0 | 0.6.x uses rcgen ^0.13 and rustls-webpki ^0.102+ |
| `rcgen` | 0.11.3 | 0.14.8 | No  libp2p-tls requires ^0.11.3 | 0.13+ uses ring ^0.17 (fixes vuln) |
| `rustls-webpki` | 0.101.7 | 0.104.0-alpha.7 | No  libp2p-tls requires ^0.101.4 | 0.101.7 is the latest 0.101.x (no patch fix available) |
| `instant` | 0.1.13 | (unmaintained) | No  comes from libp2p 0.53.x | Removed in newer libp2p |
| `bincode` | 1.3.3 | 2.x/3.x | Possible but risky | Wire format change breaks sled data compatibility |

### Conclusion

- **`ring 0.16.20` and `rustls-webpki 0.101.7`**: Cannot be fixed without upgrading `libp2p` from 0.53.2 to 0.56.0+. This is a breaking change requiring transport layer API migration.
- **`instant 0.1.13`**: Also blocked by libp2p 0.53.2 upgrade.
- **`bincode 1.3.3`**: Can be migrated to 2.x independently but requires wire-format-compatible data migration for existing sled databases.

### Part B: Upgrade Status  BLOCKED

All four vulnerabilities require dependency upgrades that cannot be completed within the current semver constraints:

1. **ring 0.16.20 / rustls-webpki 0.101.7 / instant 0.1.13**: BLOCKED by libp2p 0.53.2. Requires libp2p 0.53 -> 0.56 upgrade (breaking API changes in transport layer).
2. **bincode 1.3.3**: Migration to 2.x is possible but requires careful data migration strategy for sled persistence layers.

### Proposed Mitigation (Alternative to Full Upgrade)

Since the full libp2p upgrade is beyond MIXED-tier scope, the recommended path is:

1. **Schedule a dedicated task** for `libp2p 0.53 -> 0.56` migration as a HEAVY-LIFT tier task. This should cover:
   - API migration for all libp2p 0.54/0.55/0.56 breaking changes
   - Transport swarm configuration updates
   - Integration test updates
   - Full cargo audit verification

2. **For bincode**: Schedule a separate task to migrate from bincode 1.3 to 2.x with:
   - Wire format compatibility layer for existing sled data
   - Migration strategy: versioned serialization header
   - Test coverage for backward-compatible deserialization

3. **Short-term risk assessment**: Both vulnerabilities (ring AES panic, webpki name constraints) are MEDIUM severity and require specific attack conditions:
   - `ring 0.16.20` AES panic: requires crafted input to reach AES code path through TLS handshake  mitigated by our TLS being used only for libp2p transport (not user-facing HTTPS)
   - `rustls-webpki 0.101.7`: name constraint bypass and CRL parsing  limited exposure since we use libp2p TLS for P2P transport, not web certificate validation

## Scope

### Part A: Determine Upgrade Path (Research)

1. Run `cargo tree -p ring -i` to confirm exact reverse dependency chain
2. Run `cargo tree -p rustls-webpki -i` to confirm exact reverse dependency chain
3. Check if `rcgen` has a newer version that uses `ring 0.17.x`
4. Check if `rustls 0.22+` or `0.23+` is available and which workspace deps need updating

### Part B: Upgrade Vulnerable Transitive Dependencies

1. Update `Cargo.toml` workspace dependencies to eliminate `ring 0.16.20` requirement:
   - If `rcgen` upgrade exists: bump `rcgen` version
   - If `libp2p-tls` can be updated: bump `libp2p` patch version
   - If direct `ring` feature flag change helps: document and apply
2. Update `rustls` or intermediate crates to eliminate `rustls-webpki 0.101.7`
3. Run `cargo update` and verify `Cargo.lock` no longer contains vulnerable versions

### Part C: Compile + Test Verification

1. `cargo build --workspace`  must pass
2. `cargo test --workspace`  must pass (all existing tests)
3. `cargo clippy --workspace -- -D warnings`  must pass
4. Verify `grep 'ring 0.16' Cargo.lock` returns empty
5. Verify `grep 'rustls-webpki 0.101' Cargo.lock` returns empty

## Constraints

- Do NOT break libp2p transport compilation (TLS, QUIC)
- Do NOT change core crypto (XChaCha20-Poly1305, X25519)  this task is dependency management only
- If upgrade path requires `libp2p` version bump, scope it carefully  libp2p upgrades can cascade
- If upgrade is infeasible without breaking changes, document the blocker and propose alternative mitigation (feature flag isolation, etc.)
- All changes require `deepseek-v3.2:cloud` adversarial review per `security.md`

## File Targets

- `Cargo.toml` (workspace root and/or core/Cargo.toml) [EDIT]
- `Cargo.lock` [AUTO-GENERATED  do not hand-edit]
- `core/Cargo.toml` [MAY EDIT if libp2p/rcgen/rustls versions need changing]
- `cli/Cargo.toml` [MAY EDIT if affected]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo fmt --all -- --check
grep 'ring 0\.16' Cargo.lock
grep 'rustls-webpki 0\.101' Cargo.lock
```

## Acceptance Gates

1. `cargo build --workspace` passes  N/A (no code changes made)
2. `cargo test --workspace` passes  N/A
3. `cargo clippy --workspace` passes  N/A
4. `Cargo.lock` contains NO `ring` version < 0.17.0  **BLOCKED** (requires libp2p 0.53 -> 0.56)
5. `Cargo.lock` contains NO `rustls-webpki` version < 0.102.0  **BLOCKED** (requires libp2p 0.53 -> 0.56)
6. `REMAINING_WORK_TRACKING.md` updated with new cargo audit state  DONE
7. Vulnerability blockers documented  DONE (see Research Findings above)

## Outcome

- Part A (Research): COMPLETE  full dependency chain analysis documented
- Part B (Upgrade): BLOCKED  all 4 vulnerabilities require breaking dependency upgrades
- Part C (Verification): N/A  no code changes
- Recommended next action: Create HEAVY-LIFT task for `libp2p 0.53 -> 0.56` migration

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
