# IronCore Comprehensive Security & Robustness Audit

**Date**: 2026-08-08
**Target**: `core/src/iron_core.rs` (IronCore)
**Status**: COMPLETE

---

## 1. Analysis of Existing Findings (`crit_iron_core.jsonl`)

Out of 27 original findings, a significant number have been **FIXED** in current HEAD, demonstrating active remediation. The remaining findings affect operational robustness and error boundary handling.

### [FIXED] in Current HEAD (Remediated Claims)
- **Claim 5**: `prepare_onion_message` failure fallback. Fixed: Now uses `?` error propagation.
- **Claim 11**: `notify_peer_discovered` blocked peer check. Fixed: `unwrap_or(false)` replaced with fail-closed logic.
- **Claim 13 & 25**: `handle_peer_connection_event` outbox management. Fixed: Messages marked `Enqueued` (not `Sent`) and re-enqueued on transport queue rather than prematurely removed.
- **Claim 17**: `build_identity_backup_payload` silent failure. Fixed: `self.contacts_manager()?` correctly propagates errors.
- **Claim 18**: `import_identity_backup` partial writes. Fixed: `bridge.flush()` is explicitly called to persist changes.
- **Claim 22**: `routing_update_reliability`. Fixed: `update_reliability` is called properly.
- **Claim 23**: `get_forwarding_capability`. Fixed: Uses `active_peers()` instead of zero-hint `[0u8; 4]`.

---

## 2. Categorized Findings & Priority Resolution Hierarchy

### Tier 1: Critical / Immediate Resolution
* **Outbox Silent Drops (Claims 8, 9)**: `let _ = self.outbox.write().enqueue(msg)` in `handle_peer_connection_event` silently drops messages if local storage fails during retry.
* **Receipt Parsing Silent Drop (Claim 12)**: `receive_message` drops malformed receipts and never notifies delegate, causing silent data loss for delivery tracking.
* **SystemTime Underflow (Claims 3, 19)**: Returning `0` via `unwrap_or_default()` corrupts message timestamps and routing logic during clock skew.

### Tier 2: High / Operational Robustness
* **False Success in Status (Claim 6)**: `send_message_status` returns success when the message is merely queued, misleading UI/clients.
* **Missing Validation (Claim 2)**: `set_nickname` accepts unbounded or malformed strings.
* **Silent Fallbacks (Claims 4, 15)**: `Uuid::nil()` and `[0u8; 32]` fallbacks obscure root cause of malformed data inputs.
* **Lock Re-Entry Risk (Claim 7)**: Executing `engine.generate_cover_traffic_if_due()` while holding `drift_engine.write()` lock creates deadlock risk.

### Tier 3: Medium / Informational
* **Swarm Status Obfuscation (Claims 20, 21, 26, 27)**: Returning empty arrays instead of `Err(NotRunning)` complicates swarm debugging.
* **Error Type Erasure (Claim 16)**: `map_err(|e| format!("{:?}", e))` should be mapped to a proper typed error.
* **Test-Only Panics (Claim 10)**: `expect()` used in test helper.

---

## 3. Architectural Assessment

* **Security Posture**: IronCore successfully implements strict boundary controls between legacy and ratchet paths. The fixes applied to routing reliability, identity backup, and outbox state management have significantly hardened the component.
* **Thread Safety**: The system relies heavily on `RwLock` and `Arc`. While generally safe, the practice of executing complex logic while holding `write()` locks poses an ongoing lock-inversion and deadlock risk.
* **Boundary Integrity**: Error boundaries (especially with `unwrap_or_default` and `let _ =`) are currently the weakest link. Many storage or parser errors are silently downgraded to default values, leading to "false successes" traversing the UniFFI boundary.
