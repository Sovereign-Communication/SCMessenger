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
# BUDGET: 1800
# token_budget: 18000

# P0_SECURITY_007_Identity_Backup_Encryption_V2

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P0 security
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P0.5 (Identity backup stores secret_key_hex in plaintext JSON) + planfromclaudeforhermes 2 Phase B.1
**Depends on:** P0_BUILD_001 (test gate must be green)

---

## Verified Gap

`IdentityBackupV1` serializes the Ed25519 secret key as raw hex in JSON. No passphrase encryption, no KDF. Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P0.5: "This is a critical security gap."

Files involved:
- `core/src/identity/backup.rs`  defines `IdentityBackupV1` struct
- `core/src/identity/mod.rs`  exposes `export_identity` / `import_identity` / `IdentityBackupV1`

## Scope (~150 LoC across 2 files)

### Part A: Define `IdentityBackupV2` (LOC: ~60)

In `core/src/identity/backup.rs`:

```rust
pub const BACKUP_V2_MAGIC: &[u8; 4] = b"SCMv";  // version 2
pub const BACKUP_KDF_PARAMS: KdfParams = KdfParams {
    algorithm: Argon2id,
    memory_kib: 65536,    // 64 MB
    iterations: 3,
    parallelism: 4,
    salt_bytes: 16,
};
pub const BACKUP_CIPHER: Cipher = XChaCha20Poly1305;

pub struct IdentityBackupV2 {
    pub magic: [u8; 4],
    pub kdf_params: KdfParams,
    pub kdf_salt: [u8; 16],
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,  // includes AEAD tag
}
```

### Part B: Implement encryption/decryption (LOC: ~60)

```rust
impl IdentityBackupV2 {
    pub fn encrypt(secret_key: &SecretKey, passphrase: &str) -> Result<Self, BackupError>
    pub fn decrypt(&self, passphrase: &str) -> Result<SecretKey, BackupError>
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BackupError>
    pub fn to_bytes(&self) -> Vec<u8>
    pub fn migrate_v1(v1: IdentityBackupV1, passphrase: &str) -> Result<Self, BackupError>
}
```

Use crates already in `core/Cargo.toml`: `argon2`, `chacha20poly1305`, `blake3` (verify presence; add to `[dependencies]` if missing).

### Part C: Update IronCore surface (LOC: ~30)

In `core/src/identity/mod.rs` and `core/src/iron_core.rs`:

- Add `pub fn export_identity_v2(&self, passphrase: &str) -> Result<IdentityBackupV2>`
- Add `pub fn import_identity_v2(backup: IdentityBackupV2, passphrase: &str) -> Result<Identity>`
- Keep `export_identity_v1` / `import_identity_v1` for backward compat (mark deprecated)
- Add CLI commands: `scm identity export --passphrase-prompt`, `scm identity import --passphrase-prompt`

## File Targets

- `core/src/identity/backup.rs` [EDIT  add IdentityBackupV2 + crypto]
- `core/src/identity/mod.rs` [EDIT  surface v2 API, deprecate v1]
- `core/src/iron_core.rs` [EDIT  add public v2 methods]
- `core/Cargo.toml` [EDIT  verify argon2 + chacha20poly1305 features]
- `cli/src/commands/identity.rs` [EDIT  add passphrase prompt]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib identity
cargo test --workspace --no-run
# CLI smoke
cargo run -p scmessenger-cli -- identity export --passphrase-prompt
# Then
cargo run -p scmessenger-cli -- identity import --passphrase-prompt <file>
# With wrong passphrase  expect error
```

## Acceptance Gates

1. `cargo test --workspace` passes (existing 920+ + new IdentityBackupV2 tests)
2. New tests cover: encrypt/decrypt roundtrip, wrong passphrase fails, tampered ciphertext fails, KDF params roundtrip, v1v2 migration
3. `scm identity export` produces file with magic `SCMv2` (not `SCMv1`)
4. `scm identity import` with wrong passphrase returns clear error: `BackupError::WrongPassphrase`
5. v1 still importable via `scm identity import-v1` (backward compat)
6. Commit: `security: v0.2.1 IdentityBackupV2  Argon2id + XChaCha20-Poly1305`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: CRYPTO] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_BUILD_001] [SECURITY_CRITICAL]
