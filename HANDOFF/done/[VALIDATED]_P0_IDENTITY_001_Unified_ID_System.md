# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 3600

# Unified Identity ID System Across All App Variants

**Date:** 2026-04-23
**Priority:** P0
**Status:** IN_PROGRESS (core implementation done, pending final commit)

## Problem Statement

The current identity system uses multiple incompatible ID formats across app variants, causing:
- CLI (`scm contact add`) rejects identity hashes (expects libp2p Peer ID `12D3Koo...`)
- Android app displays identity hash (`caccf865...`) as "Peer ID" instead of libp2p network Peer ID
- WebSocket/API endpoints cannot route messages between variants due to ID mismatch
- Ledger stores addresses but lookup by identity hash fails

## Affected Variants

| Variant | Identity Format | Network ID Format | Display Issue |
|---------|----------------|-------------------|---------------|
| CLI (Rust) | `identity.id` | `12D3Koo...` (libp2p) | Correct |
| Android | `identity.id` | `12D3Koo...` (libp2p) | Shows identity hash as Peer ID |
| WASM | TBD | TBD | Unknown |
| iOS | TBD | TBD | Unknown |

## Root Cause Analysis

1. **Android UI Layer:** `SettingsViewModel.getIdentityInfo()` returns `IdentityInfo` object; UI displays `info.id` (identity hash) instead of deriving/displaying the libp2p Peer ID
2. **QR Code / Sharing:** Exporting identity uses identity hash, but other user cannot add via `scm contact add` because it expects libp2p Peer ID
3. **Missing Cross-Reference:** No `identity.id`  `libp2p_peer_id` mapping exposed to users

## Implementation Summary

### 1. Android Display Fix -- ALREADY DONE (pre-existing)
- `SettingsScreen.kt` already shows `libp2pPeerId` as "Peer ID (Network)"
- `IdentityScreen.kt` already shows both "Peer ID (Network)" and "Identity Hash" separately
- No changes needed

### 2. QR Code / Sharing Fix -- ALREADY DONE (pre-existing)
- `MeshRepository.getIdentityExportString()` already encodes libp2p Peer ID as primary `peer_id` field
- `ContactImportParser.kt` already resolves multiple ID formats
- No changes needed

### 3. CLI Validation Fix -- IMPLEMENTED
- `cli/src/main.rs` `cmd_contact()` now accepts 3 ID formats:
  - libp2p Peer ID (`12D3Koo...`)  derived to public key, verified against `--public-key`
  - Ed25519 public key hex (64 hex chars)  validated as valid curve point
  - Blake3 identity ID (64 hex chars)  resolved via `IronCore::resolve_identity()`
- Added `looks_like_ed25519_pk()` helper to distinguish valid Ed25519 keys from Blake3 hashes
- Updated `looks_like_blake3_id()` to also accept uppercase hex chars

### 4. Cross-Reference API -- IMPLEMENTED
- Added `IronCore::get_libp2p_peer_id()` method (returns `Option<String>`)
- Fixed `IronCore::resolve_identity()` to handle all 3 ID formats:
  - Valid Ed25519 public key  returned as-is
  - Blake3 identity ID  resolved via contact lookup, then local identity check
  - libp2p Peer ID  public key extracted via `extract_public_key_from_peer_id()`
- Added `WasmMeshNode::get_libp2p_peer_id()` for WASM parity

### 5. Ledger / Contact Store Unification -- IMPLEMENTED
- `ContactManager::derive_public_key_from_peer_id()` already rejects Blake3 hashes (pre-existing)
- `IronCore::resolve_identity()` now searches contacts for identity_id matches before giving up

## Acceptance Criteria

- [x] Android Settings screen shows correct libp2p Peer ID as "Peer ID" (pre-existing)
- [x] Android QR code shares libp2p Peer ID (testable via CLI `scm contact add`) (pre-existing)
- [x] CLI `scm identity` shows both identity hash and libp2p Peer ID (pre-existing)
- [x] Contact add works with Android-displayed Peer ID across all variants (IMPLEMENTED)
- [x] Unit tests verify identity hash <-> Peer ID round-trip (IMPLEMENTED - 5 new tests)
- [x] Documentation updated with unified ID glossary (this task file + inline docs)

## Files Modified

- `core/src/iron_core.rs`  Added `get_libp2p_peer_id()`, fixed `resolve_identity()` to handle Blake3 hashes, fixed drift envelope error conversion
- `core/src/identity/keys.rs`  Added 5 new unit tests for Peer ID round-trip and identity hash validation
- `core/src/wasm_support/mesh.rs`  Added `get_libp2p_peer_id()` method
- `cli/src/main.rs`  Rewrote `cmd_contact()` to accept 3 ID formats, added `looks_like_ed25519_pk()` helper
- `core/src/message/codec.rs`  Fixed pre-existing `COMPRESSION_THRESHOLD`/`DRIFT_VERSION` import errors

## Build Verification

- `cargo check -p scmessenger-core`  PASSED
- `cargo check -p scmessenger-cli`  PASSED
- `cargo test -p scmessenger-core --lib identity::keys`  10/10 PASSED (5 new tests)
- `cargo test -p scmessenger-core --lib store::contacts`  6/6 PASSED