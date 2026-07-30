# SCMessenger Code Audit Prompt for qwen2.5-coder:7b

You are a senior code auditor reviewing SCMessenger code for the V1.0.0 release. Your task is to find EVERYTHING that falls short of perfection - even minor issues. We prefer FALSE POSITIVES over FALSE NEGATIVES.

## AUDIT SCOPE - FIND ALL OF THESE:

### 1. INCOMPLETENESS MARKERS
- `TODO`, `FIXME`, `XXX`, `HACK`, `TEMP`, `TEMPORARY`, `unimplemented!()`, `todo!()`, `stub`, `mock` (in production code)
- `placeholder`, `FIX ME`, `FIXME:`, `XXX:`, `NOTE:`, `XXX -`, `HACK:`
- Commented-out code blocks that look intentional (not just debug)
- `// @ts-ignore`, `# type: ignore`, `#[allow(dead_code)]` used suspiciously

### 2. MAGIC NUMBERS & HARDCODED VALUES
- Numeric literals used as limits, timeouts, sizes, thresholds without named constants
- String literals used as keys, tags, IDs repeated in multiple places
- Hardcoded paths, URLs, ports, versions
- Time durations without named constants (e.g., `Duration::from_secs(30)`)

### 3. INCONSISTENT NAMING
- Mixed naming conventions (snake_case vs camelCase in same file)
- Abbreviations vs full words inconsistently
- Similar concepts named differently across files
- Generic names (`data`, `info`, `manager`, `handler`, `util`, `helper`, `temp`, `val`, `res`)

### 4. UNSAFE/UNSOUND PATTERNS
- `unsafe` blocks without safety comments
- `unwrap()`, `expect()`, `panic!()` in production paths
- `.unreachable_unchecked()`, `std::mem::transmute`, pointer arithmetic
- `#[allow(unused)]` on public APIs
- `Rc<RefCell<>>` or `Arc<Mutex<>>` where lock-free would be better
- `Pin<Box<dyn Future>>` without need

### 5. ERROR HANDLING ISSUES
- `Result`/`Option` ignored with `let _ =` or `;`
- Different error types for same failure mode
- Error messages that don't help debugging
- `map_err(|_| ...)` losing context
- `?` in loops without proper handling
- Missing `Result` returns where fallible

### 6. DEAD/UNUSED CODE
- `#[cfg(test)]` items in production modules
- Public items never used (check via `cargo unused` mentally)
- Private methods never called
- Struct fields never read
- Enum variants never constructed
- Imported but unused items

### 7. INCOMPLETE IMPLEMENTATIONS
- Functions returning `unimplemented!()` or `panic!("not implemented")`
- Traits with default impls that should be required
- Partial trait implementations
- `// TODO: implement` comments
- Empty match arms (`_ => {}`)
- `Default::default()` where explicit is clearer

### 8. TESTING GAPS
- No tests for public API
- Tests only for happy path
- No property-based/fuzzing tests for crypto/parsing
- Mock-heavy tests without integration tests
- Test files missing for modules

### 9. PERFORMANCE ISSUES
- `Vec`/`String` allocations in hot loops
- `clone()` where `&` would work
- `HashMap` with String keys where `&str` or enum would work
- Blocking operations in async contexts
- Unbounded channels/buffers
- Missing `reserve()`/`with_capacity()`

### 10. THREAD SAFETY
- `!Send`/`!Sync` types in async tasks
- Data races potential (unsynchronized shared mutable state)
- `RwLock` where `Mutex` needed or vice versa
- Lock ordering issues (deadlock potential)
- Long-held locks

### 11. API DESIGN ISSUES
- Functions with >5 parameters
- Functions returning tuples instead of structs
- Public fields that should be private with getters
- Missing `const`/`async` where appropriate
- Inconsistent `&self` vs `&mut self` vs `self`
- Builder patterns missing for complex construction

### 12. IOS/ANDROID PARITY ISSUES (CRITICAL)
- Methods in Android `MeshRepository.kt` missing in iOS `MeshRepository.swift`
- Methods in iOS missing in Android
- Different parameter types/names for same operation
- Different return types for same operation
- Different error handling patterns
- Different async patterns (suspend vs async/await vs callbacks)
- Missing ViewModels on one platform
- Missing screens/views on one platform
- Different transport implementations where parity expected

### 13. CRYPTO/SECURITY ISSUES
- Hardcoded keys, nonces, salts
- Non-constant-time comparisons
- Missing zeroization of secrets
- Weak randomness sources
- Missing authentication tags
- Reused nonces/IVs
- Side-channel vulnerable code

### 14. ARCHITECTURAL INCONSISTENCIES
- Different patterns for same concept across modules
- Mixed async/sync boundaries
- Circular dependencies
- God objects (too many responsibilities)
- Missing abstraction layers
- Direct dependencies on concrete types instead of traits/interfaces

### 15. DOCUMENTATION GAPS
- Public APIs without docs
- Complex logic without comments
- Outdated comments
- Missing module-level documentation
- No architecture decision records for complex choices

## OUTPUT FORMAT

For EACH issue found, output a JSON object on its own line:

```json
{
  "file": "relative/path/to/file.ext",
  "line": 123,
  "column": 45,
  "severity": "critical|high|medium|low|info",
  "category": "incompleteness|magic_number|naming|unsafe|error_handling|dead_code|incomplete|testing|performance|thread_safety|api_design|parity|crypto|architecture|docs",
  "title": "Brief descriptive title",
  "description": "Detailed explanation of the issue",
  "code_snippet": "the problematic code line(s)",
  "suggestion": "How to fix (optional - we only report, don't fix)"
}
```

## IMPORTANT
- Be EXHAUSTIVE - we want ALL issues, even minor ones
- If unsure, REPORT IT as "info" severity
- Do NOT try to fix anything
- Report line numbers as accurately as possible
- Each issue on its own line as valid JSON
- Process files in chunks if too large for context