# Gap-Audit Remediation Plan — Failsafe Execution

Status: Active (subsidiary to SHIP_PLAN.md until v0.4.0 tags)
Created: 2026-08-25
Owner: Operator (Treystu)
Source corpus: `OxAlphaAPI/results/PRIORITIZED_TASKS.md` (3,288 unique findings from 5,320 audited functions)

---

## 0. Prime directive

**No finding becomes a code change until a verifier has confirmed it against real source.**
The audit corpus was produced by an LLM reading function-sized snippets. Expect a
meaningful false-positive rate on MEDIUM/LOW. CRITICAL/HIGH get individual verification;
theme-level verification suffices for clusters sharing one root cause.

## 1. The failsafe loop (every finding passes through all five states)

```
UNVERIFIED -> VERIFIED -> FIX_DISPATCHED -> TESTED -> RE-AUDIT_CLOSED
     |              |
     +-> REJECTED --+  (false positive: record it, see §4)
```

| State | Gate | Owner | Tooling |
|---|---|---|---|
| UNVERIFIED | Finding exists in PRIORITIZED_TASKS.md | — | — |
| VERIFIED | Human/agent opened the real file, confirmed the flaw exists as described at the cited lines | Verifier lane (free) | full-file read, NOT snippet |
| FIX_DISPATCHED | Patch written against verified finding | Fix lane (free: Kiro qwen3-coder-next / oxalpha) | one theme = one branch |
| TESTED | A test exists that fails without the patch and passes with it; `main` gates stay green | CI + reviewer | cargo test / gradle / xcodebuild |
| RE-AUDIT_CLOSED | Re-run gap analysis on changed files; original finding flips NO_ISSUES or finding downgraded with rationale | OxAlphaAPI harness (`node src\analyze.mjs --file <path>`) | automatic |

**No state may be skipped. A fix without RE-AUDIT_CLOSED is not done.**

## 2. Failsafe rules

1. **One theme = one branch = one revert.** If a theme's PR goes sideways, revert is atomic and other themes continue. Never batch two themes in one PR.
2. **`main` stays green (D1).** All fix branches run local gates pre-push. If red, the branch waits — it does not merge red.
3. **Generated code is never hand-edited.** Findings citing `xcframework/Headers/*.swift` get fixed in the UniFFI Rust export definitions; the Swift regenerates.
4. **False-positive ledger.** Verified-REJECTED findings are appended to `results/rejected_findings.jsonl` (file, lines, why-rejected). The re-audit prompt includes this list so the auditor stops re-reporting them.
5. **Drift watch.** `results/drifted.jsonl` (from `node src\validate.mjs`) lists functions whose source changed after audit. Drifted = audit verdict expired; re-audit before merging anything touching them.
6. **Credit discipline (per SHIP_PLAN §1).** Verification summaries and mechanical patches run on free lanes (Kiro qwen3-coder-next @ 0.05x, oxalpha harness). Human/Claude tokens are spent only on: verifying P0s, reviewing diffs, and merge verdicts.
7. **Stop-the-line rule.** Any fix that breaks a passing test or changes a public FFI signature without updating both mobile consumers halts its theme immediately.

## 3. Sprint sequence (dependency-ordered)

### S1 — Crypto correctness (P0, ~10 findings, small diffs)
Verify then fix: Poly1305 key reuse (`dspy/signatures.rs encrypt_xchacha20`), PQ seed persistence (`crypto/pq/mod.rs`), beacon nonce determinism (`transport/discovery.rs`), salt truncation (`EntropyCanvas.kt`), legacy-invite bypass (`relay/bootstrap.rs`), plus `generate_keypair` triplication (crypto files) consolidated FIRST so later crypto fixes can't be undone by a stale copy.
Exit gate: every crypto primitive exercised by a known-answer test.

### S2 — Identity architecture ruling (P0 cluster, ~8 findings)
Operator decision required once: peer IDs derive from keys; public_key fields only ever hold real keys. Sweep: `create_peer_id`, `make_peer_id` x3, `PeerIdValidator.kt` Base58BTC, `reconcile_from_history`, `emergency_recover`, `emergencyContactRecovery`, `joinFromBundle`.
Exit gate: grep sweep proves no site assigns peer_id into a public_key field; peer-ID generator has entropy test.

### S3 — Storage & error contracts (P0/P1, API-changing)
`with_storage`/`with_storage_and_logs` return Result (or storage_mode flag); UniFFI double-try! panics fixed at export definitions; `persist_put/remove` error surfacing. Mobile consumers updated in same PR (Kotlin + Swift).
Exit gate: simulated corrupt-DB startup shows explicit error path in both mobile apps.

### S4 — Unification refactors (101 UNIFICATION findings)
Ordered by the dup index: security-relevant first (done in S1), then Kotlin ViewModel/Repository duplicates, then UI component copies. Mechanical work — free-lane patches with human merge verdicts.
Exit gate: dup index re-run shows zero cross-file duplicate groups outside generated/test exclusions.

### S5 — Hygiene drain (MEDIUM/LOW, continuous)
Theme-by-theme during idle capacity. Verify-before-fix still applies; expected false-positive rate highest here.

## 4. Relationship to SHIP_PLAN and v0.5.0

- Nothing here blocks or supersedes D1-D7. S3 touches constructors used by the alpha demo paths — schedule S3 immediately AFTER v0.4.0 tags, unless a P0 finding touches the D4/D6/D7 demo path itself, in which case it is a ship-blocker and jumps the queue with operator sign-off.
- S1 and S2 may proceed now on branches off `main` (they do not destabilize the alpha surface).

### 4.1 v0.5.0 convergence (Farm Simulation Release)

Per MILESTONE_RELEASE_PLAN.md, v0.5.0 proves the six farm topology scenarios in
12-node simulation. Its tickets and this plan's sprints OVERLAP deliberately:

| v0.5.0 ticket | Remediation sprint | Note |
|---|---|---|
| F2 (custody persistence audit) | **S3** | Audit already confirmed the in-memory fallback (2x CRITICAL) — verify-first step is done; implement the fix once, in S3 |
| A4 (outbox/custody single-ownership) | **S3** | Same constructor surface; same PR |
| Contact provisioning / `/api/identity` | **S2** | peer_id-as-public_key ruling must land BEFORE provisioning code ships, or the farm sim seeds poisoned contacts |
| B5/B6 hostile-network + soak | re-audit gate | Post-fix files re-run through the analyzer before soak starts |

Sequencing consequence: **S2 lands before any v0.5.0 provisioning work; S3 executes AS the F2/A4 tickets inside the v0.5.0 window**, not after v0.4.0 tags. This removes a double-touch of the storage constructors across two releases.

## 5. Distance to v1.0.0

Release chain: `v0.4.0-rc.1` (current) -> v0.4.0 public alpha -> **v0.5.0 farm sim** -> v1.0.0.

1. **v0.4.0 public alpha** — remaining: D1-D7 evidence (two-device receipt demo, transport-racing demo, offline proximity demo, signed APK on releases, README). Per SHIP_PLAN these are demonstration/evidence tasks, not feature gaps.
2. **S1 + S2** — crypto correctness and the identity ruling; land before/alongside alpha hardening. S2 is a prerequisite for v0.5.0 provisioning work.
3. **v0.5.0 farm sim** — S3 executes as its F2/A4 tickets; then B3-B6 rig work and the six-scenario soak.
4. **v1.0.0 gate** — per `HANDOFF/V1_0_0_EXECUTION_PLAN.md`: wired-parity complete, no open CRITICAL/HIGH security findings in a fresh audit corpus run, PQC posture already complete, CI green, release process proven twice.

Honest estimate: **v1.0.0 is gated on S1-S3 completion, a clean farm-sim soak, and sustained two-device demo evidence.** With free-lane throughput demonstrated this week (5,300+ functions audited in ~36h), remediation sprints are days of work each; the long poles are operator verdict bandwidth and rig time for the B6 soak.
