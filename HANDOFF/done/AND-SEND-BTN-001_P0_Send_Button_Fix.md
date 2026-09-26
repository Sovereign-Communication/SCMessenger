# AND-SEND-BTN-001: Send Button Not Responding Fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Critical)  
**Platform:** Android
**Status:** DONE
**Source:** docs/historical/MASTER_BUG_TRACKER.md
**Completed:** 2026-04-15

## Problem Description
User reports clicking send button 100+ times with no response. No `SEND_BUTTON_CLICKED` log entries detected.

## Root Cause
UI thread was blocked by synchronous FFI calls during recomposition. Specifically:
- `ChatScreen.kt` lines 56-57 called `viewModel.getContactForPeer()` and line 67 called `viewModel.isPeerAvailable()` synchronously during every recomposition
- These called `canonicalContactId()` which invoked `ironCore?.resolveToIdentityId()`  a synchronous FFI call to Rust
- When the Rust core was busy, these calls blocked the UI thread for seconds, preventing click events from being dispatched
- The `SEND_BUTTON_CLICKED` Timber log never executed because the onClick lambda never ran

## Changes Made

1. **`MeshRepository.kt` - Added `identityIdCache`**: ConcurrentHashMap that caches identity ID resolution results. Since `resolveToIdentityId()` is deterministic (same input always maps to same output), the cache never needs eviction. First call resolves via FFI; all subsequent calls hit the cache in microseconds.

2. **`MeshRepository.kt` - `canonicalContactId()`**: Updated to check `identityIdCache` before making any FFI call, and cache all results (identity ID, fallback normalization).

3. **`ChatScreen.kt`**: Wrapped `getContactForPeer()` and `isPeerAvailable()` in `remember(normalizedPeerId)` so they're only recomputed when the conversation changes, not on every recomposition triggered by new messages or state updates.

4. **`ConversationsViewModel.kt`**: Kept `getContactForPeer()` and `isPeerAvailable()` as non-suspend functions (since ConversationsScreen.kt also uses them in item composables)  the in-memory cache makes repeated calls fast.

## Build Verification
- Rust `cargo check`: PASSED
- Android `compileDebugKotlin`: PASSED