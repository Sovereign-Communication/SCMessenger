# Phase 0 Panic Vector Audit Report
## SCMessenger Rust Transformation

**Date:** 2025-01-XX  
**Auditor:** Kiro AI  
**Scope:** All production code in core/, cli/, and wasm/ crates

---

## Executive Summary

This audit identified all panic vectors (`.unwrap()`, `.expect()`, `panic!()`) across the SCMessenger Rust codebase. The audit found:

- **Core crate**: 2 production `.expect()` calls, multiple test-only calls
- **CLI crate**: 1 production `.expect()` call, 40+ production `.unwrap()` calls
- **WASM crate**: 0 production `.expect()` calls, 20+ production `.unwrap()` calls

**Critical Findings:**
1. SystemTime unwrap in production code (cli/src/history.rs, cli/src/contacts.rs)
2. Manager creation expect() calls that can panic on storage initialization
3. Extensive use of unwrap() for path conversion and serialization in CLI

---

## Detailed Findings

### 1. Core Crate (core/src/)

#### Production Code Issues

| File | Line | Expression | Context | Severity | Proposed Fix |
|------|------|------------|---------|----------|--------------|
| `core/src/iron_core.rs` | 1318 | `.expect("Failed to create contact manager")` | ContactManager::new() | HIGH | Return Result<ContactManager, Error> |
| `core/src/iron_core.rs` | 1391 | `.expect("Failed to create history manager")` | HistoryManager::new() | HIGH | Return Result<HistoryManager, Error> |
| `core/src/observability.rs` | 110 | `.expect("SystemTime before UNIX EPOCH!")` | SystemTime::now().duration_since() | MEDIUM | Use fallback timestamp or return Result |
| `core/src/observability.rs` | 126 | `.expect("Failed to serialize AuditEvent")` | serde_json::to_string() | MEDIUM | Return Result or use infallible serialization |

#### Test Code (Acceptable)

All `.expect()` calls in the following files are in `#[cfg(test)]` modules and are acceptable:
- `core/src/relay/protocol.rs` (lines 235-363)
- `core/src/relay/server.rs` (lines 376-547)
- `core/src/relay/invite.rs` (lines 333-334)
- `core/src/relay/findmy.rs` (lines 291-420)

---

### 2. CLI Crate (cli/src/)

#### Production Code Issues

| File | Line | Expression | Context | Severity | Proposed Fix |
|------|------|------------|---------|----------|--------------|
| `cli/src/transport_bridge.rs` | 310 | `.expect("valid seed")` | Keypair::ed25519_from_bytes() | HIGH | Return Result<PeerId, Error> |
| `cli/src/history.rs` | 277 | `.unwrap()` | SystemTime::now().duration_since(UNIX_EPOCH) | **CRITICAL** | Use helper function with error handling |
| `cli/src/contacts.rs` | 213 | `.unwrap()` | SystemTime::now().duration_since(UNIX_EPOCH) | **CRITICAL** | Use helper function with error handling |
| `cli/src/api.rs` | 382 | `.unwrap()` | Response::builder().body() | MEDIUM | Use ? operator or match |
| `cli/src/api.rs` | 387 | `.unwrap()` | Response::builder().body() | MEDIUM | Use ? operator or match |
| `cli/src/api.rs` | 394 | `.unwrap()` | Response::builder().body() | MEDIUM | Use ? operator or match |
| `cli/src/main.rs` | 481 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 513 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 651 | `.unwrap()` | info.identity_id | MEDIUM | Use if let or match |
| `cli/src/main.rs` | 666 | `.unwrap()` | info.public_key_hex | MEDIUM | Use if let or match |
| `cli/src/main.rs` | 746 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 791 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 975 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 1046 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 1137 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 1176 | `.unwrap()` | info.identity_id | MEDIUM | Use if let or match |
| `cli/src/main.rs` | 2101 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2556 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2695 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2798 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2822 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2849 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2866 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 2972 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3003 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3030 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3039 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3055 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3080 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3207 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |
| `cli/src/main.rs` | 3317 | `.unwrap()` | storage_path.to_str() | HIGH | Handle None case with error |

#### Test Code (Acceptable)

The following `.unwrap()` calls are in test functions and are acceptable:
- `cli/src/config.rs` (lines 318-319)
- `cli/src/transport_bridge.rs` (line 365)
- `cli/src/ledger.rs` (lines 531-535)
- `cli/src/contacts.rs` (line 246)
- `cli/src/main.rs` test functions (lines 3105-3118)

---

### 3. WASM Crate (wasm/src/)

#### Production Code Issues

| File | Line | Expression | Context | Severity | Proposed Fix |
|------|------|------------|---------|----------|--------------|
| `wasm/src/lib.rs` | 348 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 366 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 381 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 478 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 548 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 915 | `.unwrap()` | serde_json::to_string() | MEDIUM | Return error via map_err |
| `wasm/src/lib.rs` | 995 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1035 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1134 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1343 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1430 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1448 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1478 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1555 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1568 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1592 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1621 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |
| `wasm/src/lib.rs` | 1633 | `.unwrap()` | serde_wasm_bindgen::to_value() | MEDIUM | Return JsValue::NULL or error |

#### Test Code (Acceptable)

All `.unwrap()` calls in the following test functions are acceptable:
- `wasm/src/daemon_bridge.rs` (lines 523-620)
- `wasm/src/lib.rs` test functions (lines 2128-2257)

---

## Priority Recommendations

### Critical (Must Fix Immediately)

1. **SystemTime unwrap in production code**
   - Files: `cli/src/history.rs:277`, `cli/src/contacts.rs:213`
   - Impact: Will panic if system clock is set before Unix epoch
   - Fix: Create helper function that returns Result or uses fallback

2. **Path conversion unwrap pattern (23 occurrences in cli/src/main.rs)**
   - Impact: Will panic if path contains invalid UTF-8
   - Fix: Create helper function that handles None case gracefully

### High Priority

3. **Manager creation expect() calls**
   - Files: `core/src/iron_core.rs:1318`, `core/src/iron_core.rs:1391`
   - Impact: Panics on storage initialization failure
   - Fix: Change return type to Result

4. **Keypair creation expect()**
   - File: `cli/src/transport_bridge.rs:310`
   - Impact: Panics on invalid seed
   - Fix: Return Result<PeerId, Error>

### Medium Priority

5. **WASM serialization unwrap pattern (18 occurrences)**
   - Files: Multiple in `wasm/src/lib.rs`
   - Impact: Panics on serialization failure in browser
   - Fix: Return JsValue::NULL or error object

6. **HTTP Response builder unwrap (3 occurrences)**
   - File: `cli/src/api.rs`
   - Impact: Panics on response construction failure
   - Fix: Use ? operator with proper error handling

---

## Implementation Strategy

### Phase 0.2: Create Error Hierarchy
- Define `MeshError`, `TransportError`, `SerializationError` types
- Use `thiserror` for ergonomic error definitions

### Phase 0.3: Replace Core Unwraps
- Fix `core/src/iron_core.rs` manager creation
- Fix `core/src/observability.rs` SystemTime and serialization

### Phase 0.4: Replace WASM Unwraps
- Create helper for serde_wasm_bindgen conversions
- Return JsValue::NULL or error objects instead of panicking

### Phase 0.5: Replace CLI Unwraps
- Create `path_to_string()` helper function
- Fix SystemTime unwraps with shared helper
- Fix HTTP response builders

### Phase 0.6: Verification
- Run `grep -rn '\.unwrap()\|\.expect(' core/src/ cli/src/ wasm/src/`
- Verify zero matches in production code

---

## Notes

- All test code panic vectors are acceptable and documented
- Some unwraps in test assertions are intentional and correct
- Build scripts (core/build.rs) will be handled separately in Task 0.3

---

**Audit Complete**: Ready to proceed with Task 0.2 (Create Error Hierarchy)
