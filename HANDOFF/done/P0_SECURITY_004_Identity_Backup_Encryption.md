# P0_SECURITY_004: Identity Backup Encryption

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Critical Security)
**Platform:** Core/Rust
**Status:** Completed  
**Source:** REMAINING_WORK_TRACKING.md

## Problem Description
Identity backup stores `secret_key_hex` in plaintext JSON - no passphrase encryption. Backup files are completely unprotected.

## Security Impact
- Identity backups contain plaintext private keys
- No encryption or protection for backup files
- Complete compromise if backup accessed
- Violates basic security hygiene

## Implementation Required
1. Add passphrase-based encryption to `exportIdentityBackup()`
2. Implement secure key derivation (PBKDF2, Argon2)
3. Add encryption to backup file format
4. Ensure secure import with passphrase verification

## Key Files
- `core/src/api.rs` - Export/import functions
- `core/src/crypto/backup.rs` (new)
- Backup file format specification

## Expected Outcome
- Encrypted identity backup files
- Passphrase-based protection
- Secure key derivation
- Protected private key material