# PR #139 Five-Node Gate Status — 2026-08-13

**Authority:** Windows Orchestrator (from Mac lane exit handoff)  
**Session constraint:** No builds/tests (Antigravity heavy compute)  
**Status:** Ready for operator to proceed with node rebuild and 5-node gate

---

## Summary of Completed Work

### 1. Branch Merge [OK]
- Merged `gpt/pr139-receipt-fix-20260812` into `tracking/pre-v040-tag-work`
- Merged all commits including `860f5ed5` (P0 panic fix)
- Push to trigger CI complete at `feb89335`
- **CI Status: GREEN** — All checks passing

### 2. PF-1 Fix Applied [OK]
- Removed `MAX_DELIVERY_ATTEMPTS` constant from:
  - `core/src/store/outbox.rs:66`
  - `core/src/store/relay_custody.rs:746`
- Added `compute_next_retry_at()` with bounded exponential backoff (base 2s, cap 300s, ±25% jitter)
- Modified `record_attempt()` and `mark_dispatching()` to use backoff instead of hard cap
- Worker output: `tmp/PF1_remove_finite_attempt_abandonment_response.md`

### 3. Rule 8 Review [OK]
- Verdict: **CONDITIONAL_PASS**
- Key findings:
  - Field-order dependency is load-bearing; `connection_limits` must stay first
  - Debug_assert is NOT the enforcement; connection_limits still rejects in release
  - Dial-layer cap (`c242fb53`) must handle denied connections

### 4. CI Verification [OK]
- Repository Hygiene: [OK] PASS
- CodeQL: [OK] PASS
- FFI Surface Contract: [OK] PASS
- Android (arm64-v8a, armeabi-v7a): [OK] PASS
- Kotlin/Swift Linting: [OK] PASS
- macOS: [OK] PASS

---

## Current State After Freeze Too

**Frozen SHA: `9f54b1078ad512c895b68029c9e79a1870d7f286`**

| Node | Status | Issues |
|------|--------|--------|
| Windows CLI | Running on stale SHA | Not rebuilt to frozen |
| Android | v0.4.0/14 | No git_hash embedded |
| AWS | Running on stale SHA | Not rebuilt to frozen |
| macOS | OFFLINE | Lane exited |
| iOS | OFFLINE | Lane exited |

---

## Next Steps for Operator

### Priority 1: Rebuild Nodes to Frozen SHA

**Windows CLI:**
```bash
# Stop running node
taskkill /PID <pid_from_before> /F
# Checkout frozen SHA
git checkout 9f54b107
# Rebuild (wait for Antigravity to free)
cargo build -p scmessenger-cli --release
# Verify version
./target/release/scmessenger-cli.exe --version
# Restart with preserved identity
```

**AWS:**
- Terminate current instance (tag Name=scm-always-on-node)
- Launch new with same SG/key pair
- Pull `testbotz/scmessenger@sha256:<digest>` (use digest, not :latest)
- Run `scm start --port 9000 --http-bind 0.0.0.0:9876`

**Android:**
- Download CI artifact for `9f54b107`
- `adb install -r` (preserves identity)
- Verify mesh log for git_hash

### Priority 2: Bootstrap Propagation

Update `config.json` on all nodes to reference the NEW AWS PeerId:
- Old: `12D3KooWPJK6...` (staging)
- New: `12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`

### Priority 3: 5-Node Gate

1. Stop all nodes
2. Verify version endpoint shows frozen SHA
3. Start in order (Windows → AWS → Android → [macOS when alive] → [iOS when alive])
4. Run Matrix Pass 1 (wait for ledger convergence)
5. Run Matrix Pass 2
6. 60-minute soak test

---

## Forked Branches to Watch

| Branch | SHAs | Notes |
|--------|------|-------|
| tracking/pre-v040-tag-work | `ab9c34f7` → `feb89335` | Main branch (our work) |
| gpt/pr139-receipt-fix-20260812 | `860f5ed5` → `7538e4e9` | Receipt filtering, P0 fix |
| windows/build-9f54 | `9f54b107` | Running node SHA |

The running nodes are on `9f54b107` which is in BOTH `gpt/pr139-receipt-fix-20260812` and `windows/build-9f54`. This SHA is NOT in `tracking/pre-v040-tag-work` head. A fresh build from the merged HEAD is needed.

---

## Files Modified

- `core/src/store/outbox.rs` — Removed MAX_DELIVERY_ATTEMPTS, added backoff
- `core/src/store/relay_custody.rs` — Removed attempt cap guard
- `core/src/transport/behaviour.rs` — Moved connection_limits first (P0 fix)
- `.github/workflows/mobile.yml` — Minor CI fix from merge
- Various platform files (Android/iOS Swift/Kotlin) — Receipt classification

---

## Handoff Checklist

- [ ] Operator review of PF-1 changes
- [ ] Operator review of Rule 8 findings
- [ ] Rebuild Windows CLI from HEAD
- [ ] Rebuild AWS from Docker image digest
- [ ] Rebuild Android from CI artifact
- [ ] Update bootstrap on all nodes
- [ ] Run 5-node matrix gate

**Target:** v0.4.0 release ready