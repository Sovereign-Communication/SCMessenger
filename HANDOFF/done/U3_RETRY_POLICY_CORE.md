# TASK: U3 — Retry policy unified in core

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] — design + implementation  
**Delegation:** `/scmqwen` → CODER model  
**Priority:** F0 gate (used by A1 outbox-flush, enables consistent backoff everywhere)  
**Related:** A1, UNIFICATION_AUDIT_FINDINGS.md  

---

## Problem

Retry/backoff logic is hand-rolled in CLI (`cmd_send_offline`) and will need to be in core for A1 (outbox flush) anyway. Currently:

- `cli/src/main.rs:2869` — `let max_retries = 3; let delay_ms = 100u64 << (attempts - 1);` (exponential backoff)
- Core has no `RetryPolicy` or retry mechanism
- Every platform client invents its own, with inconsistent parameters
- Android will need the same logic; iOS will need the same logic; WASM will need the same logic

A1 (outbox-flush fix) will need this infrastructure in core anyway. Unify now.

---

## Solution

Create `RetryPolicy` struct in `core/src/store/outbox.rs` with helper functions. Provides:
- Configurable max retries, initial delay, backoff factor
- Function to compute next delay given attempt number
- Default that matches existing CLI behavior (3 retries, 100ms initial, 2x exponential)

All retry logic calls these helpers. Zero variation.

### Implementation spec

**File: `core/src/store/outbox.rs`**

Add at the top of the file (after imports):

```rust
use std::time::Duration;

/// Retry configuration for message delivery.
/// 
/// This is the ONLY place retry policy is defined. All platforms
/// (CLI, Android, iOS, WASM) use this struct. Changes to backoff
/// strategy apply everywhere automatically.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (including initial attempt).
    pub max_retries: u32,
    /// Initial delay in milliseconds before first retry.
    pub initial_delay_ms: u64,
    /// Backoff multiplier (2 = exponential, 1 = fixed).
    pub backoff_factor: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,           // CLI baseline
            initial_delay_ms: 100,    // CLI baseline
            backoff_factor: 2,        // exponential: 100ms, 200ms, 400ms
        }
    }
}

impl RetryPolicy {
    /// Compute the delay before the given attempt (1-indexed).
    /// 
    /// Returns None if attempt exceeds max_retries (delivery should be abandoned).
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_retries {
            return None;
        }
        if attempt == 1 {
            // No delay for initial attempt
            return Some(Duration::from_millis(0));
        }
        // exponential: delay = initial * (backoff ^ (attempt - 2))
        // attempt 2: delay = initial * 1 = 100ms
        // attempt 3: delay = initial * 2 = 200ms
        // attempt 4: delay = initial * 4 = 400ms
        let power = (attempt - 2) as u32;
        let multiplier = (self.backoff_factor as u64).saturating_pow(power);
        let delay_ms = self.initial_delay_ms.saturating_mul(multiplier);
        Some(Duration::from_millis(delay_ms))
    }

    /// Whether another retry is possible.
    pub fn can_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    #[test]
    fn test_default_retry_delays() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(0)));
        assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(100)));
        assert_eq!(policy.delay_for_attempt(3), Some(Duration::from_millis(200)));
        assert!(policy.delay_for_attempt(4).is_none()); // exceeds max_retries
    }

    #[test]
    fn test_can_retry() {
        let policy = RetryPolicy::default();
        assert!(policy.can_retry(1));
        assert!(policy.can_retry(2));
        assert!(!policy.can_retry(3)); // 3 is the max
    }
}
```

Export from `core/src/lib.rs`:
```rust
pub use crate::store::outbox::RetryPolicy;
```

**File: `cli/src/main.rs:2869` (cmd_send_offline)**

Replace the hand-rolled retry loop:

- **Before:**
```rust
let max_retries = 3;
for attempt in 1..=max_retries {
    let delay_ms = 100u64 << (attempt - 1);
    if attempt > 1 {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
    // send logic here
}
```

- **After:**
```rust
let retry_policy = scmessenger_core::RetryPolicy::default();
for attempt in 1..=retry_policy.max_retries {
    if let Some(delay) = retry_policy.delay_for_attempt(attempt) {
        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }
    }
    // send logic here
    if !retry_policy.can_retry(attempt) {
        break;
    }
}
```

---

## Acceptance criteria

- [ ] `RetryPolicy` struct defined in `core/src/store/outbox.rs` with helper methods
- [ ] Exported from `core/src/lib.rs`
- [ ] Default matches existing CLI behavior (3 retries, 100ms, 2x backoff)
- [ ] Unit tests pass (`retry_tests` module compiles and all assertions pass)
- [ ] `cli/src/main.rs::cmd_send_offline` uses `RetryPolicy::default()` instead of hand-rolled logic
- [ ] Grep finds 0 remaining hardcoded `max_retries = 3` or delay calculations outside the policy
- [ ] `cargo test --workspace --no-run` passes (compile gate)
- [ ] No behavior change (same delays, same retry count, just centralized)

---

## Notes

- **A1 uses this:** Once A1 lands, it will use `RetryPolicy` for the outbox flush loop.
- **Platform consistency:** Android/iOS/WASM will all call `RetryPolicy::delay_for_attempt()` from their own retry loops, ensuring identical backoff behavior.
- **Configurability:** Farm deployment can pass custom `RetryPolicy` (e.g., more aggressive retries) via config later, without code changes.
- **Saturation:** `saturating_pow` and `saturating_mul` prevent overflow on extreme attempt counts (safe even if someone sets backoff_factor to 1000).

