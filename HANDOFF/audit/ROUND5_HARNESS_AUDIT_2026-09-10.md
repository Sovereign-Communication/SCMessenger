# SCMessenger Round-5 Harness Audit + Handoff Addendum

**Date:** 2026-09-10  
**Harness baseline:** main @ PR #4 + PR #5 + PR #6 merged (`95bfee8`)  
**Method:** `harness verify --claims-file --source-file --converge` on the 19
recently-edited functions in `scmessenger_audit/` (read-only). Absolute `--out`
paths. Free tier $0.00 for the batch; one paid-tier confirmation pass.
**SCMessenger tree:** untouched (active work remains on `cto/t2-disk-ruling-2026-08-31`).

> Prior 2026-09-09 handoff claimed a complete 19-function audit, but
> `run_audit.py` used a relative `--out` with `cwd=Harness`, so **19/19 runs
> failed to persist JSON** (`cannot write --out scmessenger_audit/results/...`).
> This round closes that gap: every run wrote a real result file.

---

## 1. Evidence locations (Harness side)

| Artifact | Path |
|----------|------|
| Round-5 results (19) | `Harness/audits/scmessenger/_runs/round5/*.json` |
| Batch summary | `Harness/audits/scmessenger/_runs/round5/summary.json` |
| Paid confirmation | `Harness/audits/scmessenger/round5/01_paid_escalation.json` |
| Earlier smoke | `Harness/audits/scmessenger/round5/` (subset) |

**All 19 runs:** `returncode=0`, `out_exists=true`, free-tier `actual_cost=0.0`.

---

## 2. Round-5 free-tier scoreboard (19 functions)

| # | Function | Defer | Escalation | Deterministic tally (claim-level) |
|---|----------|-------|------------|-----------------------------------|
| 01 | `is_poison_circuit_listener` | yes | **yes** | C1 not_real (2/3); C2 **split / shortfall** |
| 02 | `is_self_endpoint` | yes | — | C1 real 3/3; C2 not_real 3/3; C3 real 3/3 |
| 03 | `is_valid_reservation_base` | no | — | C1 real 3/3 |
| 04 | `is_canonical_reservation_addr` | no | — | C1 real 3/3 |
| 05 | `set_configured_external_address` | no | — | C1 real 3/3; C2 real 3/3 |
| 06 | `reset_delivery_attempts_for_destination` | no | — | C1+C2 real 3/3 |
| 07 | `sort_by_reliability` | no | — | C1 real 3/3; C2 not_real 3/3 |
| 08 | `active_peer_selection_contract` | yes | — | C1+C2 not_real 3/3 |
| 09 | `attemptEscalation` | — | — | (written; see JSON) |
| 10 | `sendViaTransport` | yes | — | (written) |
| 11 | `initializeWifiAware` | no | — | C1+C2 real 3/3 |
| 12 | `initializeWifiDirect` | yes | **yes** | C1 real 3/3; C2 split |
| 13 | `mergeBootstrapCandidates` | no | — | C1+C2 real 3/3 |
| 14 | `recordConnectionFailure` | yes | — | C1–C3 real 3/3 |
| 15 | `startAdvertisingInternal` | yes | — | C1 not_real 3/3; C2 real 3/3 |
| 16 | `stopAdvertisingInternal` | yes | **yes** | C1 not_real 3/3; C2 split |
| 17 | `shareDiagnosticsBundle` | no | — | C1+C2 real 3/3 |
| 18 | `resolveShareTarget` | no | — | C1+C2 not_real 3/3 |
| 19 | `refreshInfoCounts` | no | — | C1–C3 real 3/3 |

**Escalation rate:** 3/19 (16%) — all three are genuine panel splits, not
transport shortfalls alone.

### Paid confirmation (01, cost $0.000724)

Paid panel (granite + llama-8b + ling-flash) tallied:

- **C1 not_real 3/3**, **C2 real 3/3**, **C3 real 3/3** (specialist confidence C2=0.4 still flagged a 2v1 conflict narrative).

**Operator takeaway for C2 (`circuit_count > 1 \|\| !is_tracked_reservation`):**
two independent free+paid panels treat the OR-condition as a real defect for
untracked single-circuit reservations. This is the same finding as the
2026-09-09 escalation on function 01. **Needs a product ruling, not another panel.**

---

## 3. Cross-cutting SCMessenger findings (unification / cohesion)

These are **audit signals** — not applied to the product tree.

1. **D6 still unwired (SHIP_PLAN 6.2).** `routing_peer_seen` remains
   `pub fn` only (`core/src/iron_core.rs`); zero call sites. Adaptive-routing
   confidence stays 0.0; transport-racing claims are unprovable until
   `ConnectionEstablished` feeds `peer_seen`.

2. **Poison-listener C2 policy.** `is_poison_circuit_listener` OR-condition
   repeatedly escalates. Decide: structural validation vs reachability, and
   whether `!is_tracked_reservation` should short-circuit to true at
   `circuit_count==1`. Document the policy in the function contract.

3. **External-address C1 (loopback/private).** Free panels split on whether
   missing SocketAddr validation is a defect. Paid panel (earlier smoke)
   leaned “not a defect” (NAT flexibility). If T14 allowlist is the real
   guard, say so in the function doc so panels stop re-litigating.

4. **Android transport cohesion (12/15/16).** `initializeWifiDirect`,
   start/stop advertising still show claim splits or defers — same BLE/Wi-Fi
   ownership class as the unification campaign. Prefer one owner for
   start/stop state so stop is always the inverse of start.

5. **PR #279 vs #272.** Two unification PRs still overlap address admission /
   local ordering. Dispose #272 explicitly (rebase or retire hunks) before
   stacking more transport work on #279.

6. **Audit artifact placement.** Live harness runs belong under Harness
   (`audits/scmessenger/…`). Do not write runner scripts or result trees into
   the SCMessenger working tree while the CTO campaign is live. Untracked
   debris still in SCMessenger root: `run_audit.py`, `extract_*.py`,
   `scmessenger_audit/` — quarantine or gitignore; do not commit from this seat.

7. **0.4.0 tracking honesty.** SHIP_PLAN CP2–CP6 still empty. D2 still the
   keystore-alias blocker. D4/D6/D7 need released-APK evidence, not more
   free-tier panels. Round-5 panels do **not** change any D1–D7 gate.

---

## 4. Harness iteration this session (for cohesion of future audits)

| Change | PR | Why it matters for SCMessenger |
|--------|----|--------------------------------|
| Apply multi-rung escalation + gate-finished rungs | #4 | Paid ladder can finish a stuck *edit*; verify still free by default |
| `--out` parent dirs | #4 | Relative audit paths no longer silently lose evidence |
| Claims defs: Python + comments skipped | #4/#5 | Cleaner lint for mixed Rust/Kotlin/Python windows |
| Escalation `needed` must be JSON `true` | #5 | Paid ladder cannot open on `"false"` strings |
| `target_rung` coerce / task budget / diff / HARNESS_DEFER | #5 | Fail-closed apply ladder |
| Train skips need onnxruntime | #5 | CI green without train extra |
| Specialist `escalation`/`plan` on consensus | #5 | Callers can see directives without digging |

**Spend discipline:** key limit $0.75/day; remaining after this campaign
**$0.7493** (~0.07¢ total paid smoke + paid confirmation). Free batch $0.

---

## 5. Recommended next actions (product, not harness)

| Priority | Action | Owner |
|----------|--------|-------|
| P0 | Wire D6 `peer_seen` on `ConnectionEstablished` | CTO transport lane |
| P0 | Operator: fix `SCMESSENGER_KEY_ALIAS` → D2 signed APK | Operator |
| P1 | Ruling on poison-listener C2 OR-condition | CTO + operator |
| P1 | Disposition #272 vs #279 | CTO |
| P2 | Quarantine untracked audit scripts in SCMessenger root | Any seat |
| P2 | Re-score D4/D6/D7 on the **released** APK after D2 | Operator + agy |

---

## 6. What this handoff is not

- Not a merge of any SCMessenger PR.
- Not a change to SHIP_PLAN checkpoint ledger (CP2–CP6 remain empty until
  operator evidence exists).
- Not a replacement for rule-8 adversarial reviews on transport/routing diffs.

Harness PRs #4, #5, and #6 are merged on `Treystu/Harness`. Future SCMessenger
panels should use absolute `--out` (or rely on the fixed `_emit` parent-dir
creation) and write under Harness `audits/`, never into the live product tree.

---

## 7. Harness safety burn-down (PR #6) — verified live

| Fix | Live check |
|-----|------------|
| Library `allowed_roots` | Unit tests; empty roots = unrestricted CLI (as designed) |
| Backup `O_EXCL` + unique dest | Unit tests; prune never deletes the new backup |
| Ledger load integrity | `harness ledger verify` → **ok**, 2738 entries, `chain_broken_on_load: false` |
| extract skips non-dict JSON | Full suite 536 OK |

**Post-burn-down smoke (natural, free tier):**
`01_is_poison_circuit_listener` re-ran with absolute `--out` → EXIT 0, JSON
written, $0.00. Panel rotation still works (gemma-31b malformed → north-mini).

### Known remaining (not forced, documented)
- POSIX ledger flock still blocking (Windows path is non-blocking)
- `local_fit` train/eval label leak + dead prototype modules (not on apply/verify path)
- Paid **apply-ladder** (not just paid verify) still not exercised on a real edit
- MCP self-authorizing write/verify remains the documented trust model

---

## 8. Next SCMessenger audit exercise — plan (as designed)

Do **not** force convergence or pay for verdicts unless the free panel
naturally escalates. Let harness behave as designed.

**Prep (done):** Harness main green (536 tests), ledger verified, key $0.748 remaining.

**Exercise (when you say go):**
1. Re-run the **3 previously escalated** claims only (01, 12, 16) free-tier
   with absolute `--out` under `Harness/audits/scmessenger/round6/`.
2. Compare round-5 vs round-6 tallies for **stability** (same claim text,
   different panel weather). Stable splits = product policy questions, not
   harness flakiness.
3. Only if a free panel **naturally** emits `escalation.needed: true`, run
   one paid confirm (`--max-cost 0.08`) — never escalate by default.
4. Record deltas here; do not edit SCMessenger product files.

**Success looks like:** persisted JSON, honest defer/shortfall fields, cost
within ceiling, no silent write failures. Not a predetermined pass/fail.

---

## 9. Round-6 natural re-audit (2026-09-10) — executed as designed

**Scope:** only the three claims that naturally escalated in round 5
(01, 12, 16). Free tier first. Paid confirm **only** when free emitted
`escalation.needed: true` (claim 16). No product-file edits.
**Evidence:** `Harness/audits/scmessenger/round6/`
**Spend:** free $0.00; paid confirm 16 **$0.00134**; key remaining **$0.7482**.

### Stability vs earlier rounds

| Claim | Earlier signal | Round-6 free | Learning |
|-------|----------------|--------------|----------|
| **01** poison-listener | C2 split / escalate (R5 batch + paid) | **Shortfall 2/3**, C1+C2 `not_real` 2/2, `esc_needed=false` | Panel weather matters (only 2 responders). C2 still not product-ruled. Shortfall correctly reported — not forced to 3/3. |
| **12** WifiDirect init | C2 2R/1NR, escalate | **C1+C2 real 3/3**, agree high, **no defer** | C1 stable real; C2 **strengthened** to unanimous real (missing capability checks). Convergence specialist 502×2 — **tally still authoritative**. |
| **16** stopAdvertising | C2 2NR/1R (gemma-4-26b dissenter), escalate | **C2 again 1R/2NR** (same dissenter **gemma-4-26b**), escalate again | **Stable split.** Free panel majority `not_real`; one model consistently flags stopRotation/exception path. |

### Paid confirm 16 (natural escalation only)

Cost $0.00134. Panel (ling + granite + llama-8b):

- **C1:** 2 real / 1 not_real  
- **C2:** 2 real / 1 not_real (granite+llama say real; ling says not)  
- **C3 (reassurance):** mixed  

**Harness learning (not product):** the printed tally said `C1=real (3/3); C2=real (3/3)` while panel JSON is 2R/1NR — the `(N/3)` suffix is **participation**, not unanimity. Specialist prose still inverted some claims. Deterministic majority still leans **C2 real** on paid vs **C2 not_real** on free majority — this is a **product-policy** claim (stopRotation vs failed stopAdvertising), not a harness flake.

### Exercise outcome

- **3/3 free runs persisted**, EXIT 0, $0.00  
- Shortfall/defer reported honestly (01 2/3)  
- One natural paid confirm, under ceiling  
- Ledger still clean (`chain_broken_on_load: false`)  
- **No forced convergence.** Remaining work is SCMessenger product rulings + D6/D2 ship gates — not more panels.

---

## 10. Round-7 live verification of harness improvements (2026-09-10)

Harness main `d7b82e7` (4-dim A/R/SM/SD **10.0**). Re-ran existing SCMessenger
claims free-tier under `Harness/audits/scmessenger/round7/`. Product tree untouched.

| Claim | New verdict (live) | Improvement proven |
|-------|--------------------|--------------------|
| **01** poison-listener | `C1=not_real (0R/3NR of 3); C2=not_real (1R/2NR of 3)` | **No more fake (3/3) unanimity.** C2 split is visible as 1R/2NR (gemma-26b dissenter). |
| **12** WifiDirect | `C1=real (3R/0NR of 3); C2=real (3R/0NR of 3)`, agree high, no defer | True unanimity shown as 3R/0NR. |
| **16** stopAdvertising | `C1=not_real (0R/3NR of 3); C2=not_real (1R/2NR of 3)` | Stable dissent **counted**, not hidden. |
| **17** shareDiagnostics | `C1=real (2R/1NR of 3); C2=real (3R/0NR of 3)` | **`tally_conflicts`**: specialist said C1 `not_real`, tally `real` — conflict recorded, tally authoritative. |

**Also verified**
- 4/4 JSON persisted, EXIT 0, $0.00
- `panel_shortfall` boolean present on every run
- Ledger `ok: true`, `chain_broken_on_load: false`
- Suite 543 OK; unit pin for `1R/1NR` + `SHORTFALL` wording added
- Key remaining **$0.7473 / $0.75**

**Operator takeaway:** 01-C2 and 16-C2 remain **product policy** questions
(1 dissent vs 2 majority). Harness now shows the split honestly instead of
printing `(3/3)` next to a majority verdict.
