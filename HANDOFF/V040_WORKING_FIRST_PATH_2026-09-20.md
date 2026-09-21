# V0.4.0 working-first path — operator interview 2026-09-20

Status: Active — **supersedes the tag-pressure reading** of
`HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md` for sequencing and priority.
Where that file still says "path to tag" as the primary driver, THIS document
wins. D1-D7 exit criteria remain the **future** release definition; they are
NOT the current execution bar.

Source: operator interview, MiMo Desktop session, 2026-09-20. Rulings below
are recorded verbatim in substance. Other agents and Freebuff must follow this
order.

Interview record path (filled): `HANDOFF/freebuff/inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md`.

---

## 0. Operator direction (one sentence)

**No tag, no secrets pressure — get the mesh working first.**

Corollary: nobody re-dispatches keystore archaeology, release rehearsal, or
final `v0.4.0` tagging until the working bar in §4 is met.

---

## 1. Interview rulings (complete)

| Topic | Ruling |
|---|---|
| D2 / keystore / secrets / tag | **Deferred.** "No tag, no secrets, just get it working first." |
| Definition of working | **Reliable mesh messaging day-to-day** on Windows CLI + AWS cloud node + Pixel 6a |
| AND-06 crypto validation | **Do it right.** 1) collapse redundant Kotlin copies; 2) full UniFFI cutover via `is_valid_public_key`; delete remaining curve math |
| SEC-03 sled | **(b)** start storage migration on a branch; tagging can still happen later on current sled once working bar is met — not a hold on implementation |
| External crypto audit commission | **Defer until mesh is reliable** |
| Live scoring during the wave | **No scoring until it all lands** |
| Pixel driving | **Operator always drives the phone UI** (standing rule). Orchestrator may `adb install -r` anytime; passive logs only otherwise |
| #325 chat-order + #322 outbox keys | **Land both when green** (orchestrator merge after required checks + pr_scope) |
| Wave 1 contents (multi-select) | Conn-limits; AND-06 A-lite then UniFFI; watchdog positive test; merge #325/#322; **AWS IP-churn autonomous rediscovery**; **beach-join Phase 0-1** (pulled earlier than "after mission DONE") |
| Doc persistence | **Rewrite unified path to working-first** and merge to main |
| Post-wave "working" evidence | **Harness + logs + one operator phone session** |

---

## 2. What "working" means (acceptance bar)

The mesh is working when **all** of the following hold on a **same-SHA** fleet
(CI artifacts, not `target/`):

1. Windows CLI, AWS cloud node, and Pixel run the same candidate SHA (or a
   documented one-commit Android-only delta if unavoidable).
2. Day-to-day messages Android <-> Windows deliver with receipts; outbox
   drains on reconnect; history is durable on the receiver.
3. Cloud node takes store-and-forward custody when a destination is offline
   and delivers on return (custody audit log evidence).
4. Seed-dial / ledger path still works after a node restart.
5. **No mass `connection_limits: limit 4 reached` denial storm** on ordinary
   multi-port dial bursts (compare to 2026-09-19 baseline ~274 WARN/hour).
6. Operator phone session: you send/receive on the Pixel for normal use
   without wedges, reordering surprises that break conversation, or stuck
   outbox.
7. Evidence pack:
   - `scripts/tier_a_conformance.sh` output on the Tier A pair
   - Windows + AWS log slices (seed-dial, custody, outbox, connection_limits)
   - short operator verdict after the phone session

**Not required for this bar:** signed GitHub Release, D2, external audit
completion, formal D4/D6/D7 release scoring, beach-join device proofs on a
second handset (Phase 0-1 is code + single-device verification unless you
bring a second phone).

---

## 3. Wave 1 — implementation order

Freebuff pastes **only** DISPATCHABLE tickets on `origin/main`. Orchestrator
merges PRs after gates. Operator drives Pixel; agents do not.

| Step | Owner | Work | Gate |
|---|---|---|---|
| 1 | Orchestrator | Merge **#325** when required checks green (`scripts/pr_scope.sh 325`) | none beyond CI |
| 2 | Orchestrator | Diagnose **#322** failing checks; merge when required contexts green | pr_scope; escalate if store path surprises |
| 3 | Freebuff paste | `V040_T_CONN_LIMITS_MULTIPORT.md` | **Rule-8** |
| 4 | Freebuff paste | `V040_T_AND06_KOTLIN_COLLAPSE.md` (A-lite) | none if android-only dedupe |
| 5 | Freebuff paste | `V040_T_AND06_UNIFI_CUTOVER.md` (A full) after 4 merges | Rule-8 if FFI/core surface changes |
| 6 | Freebuff paste | `V040_T_WATCHDOG_POSITIVE_TEST.md` | test-only preferred |
| 7 | Freebuff paste | `V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` (AWS/cloud IP change remesh) | **Rule-8** if transport/routing/disclosure |
| 8 | Freebuff paste | Beach-join **Phase 0-1 only** (spec + hotspot share Android) per operator pull-forward | Phase 2-3 still Rule-8; Phase 0-1 android-focused |
| 9 | Orchestrator | SEC-03 storage **migration branch** exploration (parallel; does not block wave) | rule-9 already ruled (b); no merge of engine swap without further operator sign-off |
| 10 | After wave | Scoring session per §2 — **not before** all wave items report landed or explicitly waived |

Queue status gate before every paste wave: `python scripts/check_queue_status.py` exit 0.

---

## 4. Explicitly deferred (do not burn cycles)

| Item | Until |
|---|---|
| Release keystore verify, secret re-set, release.yml rehearsal | Working bar met; then re-open D2 work |
| Final `v0.4.0` tag / GitHub Release publish | After working bar + operator go |
| External crypto audit outreach | After mesh is reliable (operator) |
| Formal D4/D6/D7 release scoring | When/if tag work resumes |
| C4 identity-aware relay admission | 0.5.0 (unchanged) |
| iOS/macOS parity | 0.5.0 (unchanged) |
| Beach-join Phase 2-3 device multi-handset proofs | After Phase 0-1 + working bar, unless operator advances |
| Dependabot / Apple docs bulk merge | Post-wave PR disposition |

---

## 5. Freebuff efficiency rules (unchanged, reinforced)

1. Operator is the only transport into Freebuff desktop.
2. Paste **DISPATCHABLE** tickets only; never re-paste CODE ON MAIN / MERGED tickets.
3. Premise-verify before writing any new ticket; wrong premise = write inbox, do not implement.
4. Freebuff opens PRs; **does not self-merge**, tag, set secrets, or drive Pixel UI.
5. Evidence contract on every status line.
6. Rule-8 before merge on `core/src/{crypto,transport,routing,privacy}`.
7. Everything the lane must see must land on **`origin/main`**.
8. New work in **worktrees** (`tmp/wt-*`), not the dirty shared main checkout.
9. **CI queue hygiene (operator 2026-09-20, standing):** after merges or
   branch updates, cancel superseded Actions runs (older main SHAs, merged-PR
   heads) so the candidate is not blocked behind dead work. Keep artifact
   jobs on the deploy/candidate SHA. Policy: `docs/rules/BUILD_AND_CI.md`.

---

## 6. Open PR disposition under this ruling

| PR | Action |
|---|---|
| #325 | **Merge when green** (wave step 1) |
| #322 | **Merge when green** after check diagnosis (wave step 2) |
| #329 / queue docs | Land with freebuff status tickets as needed |
| Work-aheads #298-#302 | Hold unless they fix a working-bar defect |
| Dependabot / Apple / legacy | Post-wave |
| #215 | **Already closed** (superseded routing wire-up) |

---

## 7. Decision interview transcript (summary for the record)

Q1 D2/keystore — A: no tag, no secrets, just get it working first.

Q2 AND-06 — A: unification; collapse copies if redundant; then finish UniFFI; do it right.

Q3 SEC-03 — A: (b) start migration on a branch; tag can wait / tag later anyway.

Q4 Working definition — A: reliable mesh messaging day-to-day.

Q5 AND-06 sequence — A: 1 collapse copies 2 full UniFFI cutover.

Q6 Scoring — A: Pixel UI is always operator-driven; orchestrator may install anytime; **no scoring until it all lands**.

Q7 #325/#322 — A: land both when green.

Q8 Wave contents — A: conn-limits, AND-06 both steps, watchdog test, #325/#322, AWS IP-churn autonomous rediscovery, beach-join Phase 0-1.

Q9 External audit — A: defer until mesh is reliable.

Q10 Docs — A: rewrite unified path to working-first.

Q11 Post-wave evidence — A: harness + logs + one operator phone session.

---

## 8. Success criterion for "done with Wave 1"

- [ ] #325 and #322 merged or explicitly deferred with operator note
- [ ] Conn-limits fix merged with Rule-8 APPROVE on file
- [ ] AND-06 A-lite merged; UniFFI cutover merged or waived in writing
- [ ] Watchdog positive test green on Windows
- [ ] IP-churn ticket implemented or blocked with live evidence
- [ ] Beach-join Phase 0-1 spec/hotspot work landed or scoped with PREMISE-WRONG
- [ ] SEC-03 migration branch started (even if incomplete)
- [ ] Scoring pack collected and operator phone session completed
- [ ] This checklist updated with commands / run URLs / UNVERIFIED honestly

End of working-first path.
