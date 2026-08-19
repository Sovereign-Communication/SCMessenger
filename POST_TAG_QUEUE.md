# Post-Tag Queue -- Deferral Register

Status: Active
Created: 2026-08-16
Owner: CEO (register integrity) / CTO (execution)
Companion to: `SHIP_PLAN.md` (the pre-tag queue)

**Purpose.** `SHIP_PLAN.md` says what we are doing before the v0.4.0 tag. This
file says what we deliberately are **not** doing, why, and what brings it back.
Deferred is not dropped. If work is parked and not listed here, the parking was
a mistake.

---

## 1. Deferral integrity check (CEO, 2026-08-16)

The S0-4 backlog amnesty executed: `HANDOFF/todo/` went 99 -> 26, with 79 items
moved to `HANDOFF/archive/`.

| Check | Result |
|---|---|
| Archived items still tracked in git | **[OK]** 79/79 tracked -- fully recoverable, nothing destroyed |
| Archive has a disposition index | **[FAIL]** No index. Cannot distinguish "archived because DONE" from "archived because DEFERRED" |
| Post-tag queue exists | **[FAIL]** None existed before this file |
| Dependabot handled correctly | **[OK]** CTO ruled defer-all/close-none, correctly identifying them as the S4 queue |

**Verdict: the deferral is safe but was not yet a queue.** Nothing is lost -- git
holds every file -- but an unindexed archive is a graveyard, not a backlog. This
register is the fix.

---

## 2. RECALL NOW -- mis-filed, do not wait for the tag

The amnesty criterion was "keep what maps to D1-D5". These items map to **D4**
(two devices, no shared network, receipt proven) but their filenames did not say
so, and they were swept. Their own status lines, read 2026-08-16:

| Ticket (in `HANDOFF/archive/`) | Self-reported status | Why it is D4, not S4 |
|---|---|---|
| `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS_2026-08-10.md` | **Open** | "No off-LAN rendezvous" *is* the north star sentence. Two phones with no shared network cannot meet without it |
| `P0_NO_RELAY_FALLBACK_FOR_ROAMING_PEERS_2026-08-10.md` | **Open -- observed LIVE during iteration-2 roaming** | D4 is explicitly cross-network, one peer on cellular. This is that exact failure |
| `P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md` | **Active** -- "must be resolved or explicitly accepted before Run 1" | A node that panics as the mesh grows cannot complete the five-node gate |
| `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md` | **Open -- root cause identified, needs operator decision, merge-blocked path** | Carries an unmade operator decision. Parking it silently buries the decision, not just the work |
| `P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md` | **Fixed in #139 branch; Windows soak still required** | The fix is real but unproven. Soak is D4 evidence |
| `P0_BLE_L2CAP_ACCEPT_SPIN_2026-08-08.md` | **Active** | CTO to rule: if BLE is out of D4 scope, defer explicitly to S4 below with that reason recorded |

**Action:** CTO dispositions each of the six -- either back to `HANDOFF/todo/` as
D4 work, or moved to Section 3 with an explicit written reason. No third option.
Six tickets, one pass. This is not a re-opening of the amnesty; the other 73
stay archived.

> Process note, CEO-owned: this is my gap, not the CTO's. I set the amnesty
> criterion as "maps to D1-D5" and filename-matching cannot evaluate that.
> Fix in Section 5.

---

## 3. Deferred to post-tag (S4) -- confirmed, with re-entry triggers

| # | Item | Owner | Re-entry trigger | Risk if it slips |
|---|---|---|---|---|
| S4-1 | **Dependency debt.** 13 open dependabot PRs (#64, #65, #67, #69, #99, #100, #102, #103, #106, #107, #108, #141, #142). GitHub reports 7 vulnerabilities on the default branch, 3 high | CTO -> Orchestrator | First working day after tag. Batch-merge as one decision, not 13 | Highest-severity item here. Six months of unpatched deps on a security product is a headline. Defer to ship, do not defer twice |
| S4-2 | **External crypto audit.** Hybrid X25519 + ML-KEM-768 | CEO (budget) + CTO (scope) | Immediately post-tag; needs money, not tokens | The PQC claim is the differentiator and the liability. Self-review by the fleet that wrote it is not a credential |
| S4-3 | **iOS / Apple parity.** `iOS_V040_PARITY_IMPLEMENTATION_PLAN.md`, `iOS_ANDROID_PARITY_AUDIT_v040.md`, plus 3 Apple-related archived tickets incl. `U6_IOS_RECEIPT_UNIFICATION.md` and `A-05_IOS_RECEIPT_UNIFICATION.md` (in_progress) | **CAO** (GPT-MAC lane) | Post-tag. Android ships first -- deliberate | CAO lane idles pre-tag except for the macOS node in the five-node gate. Handoff file to be written when S4-3 opens |
| S4-4 | **Android last mile.** 162 unwired functions, 84 in `MeshRepository.kt` | CTO -> Orchestrator | Post-tag; burn down only what real usage exercises | Speculative surface. Do not wire all 162 -- most will never be called |
| S4-5 | **PQC follow-on.** `PQC_09_HYBRID_ONION`, `PQC_10_MLDSA_MODULE_MISSING`, `PQC_09_SECURITY_REVIEW_FIXES`, `PQC_SEEDING_SECURITY_HARDENING`, `PQC_00_MASTER_PLAN` (archived) | CTO | After S4-2 -- let the audit shape the work | Sequence matters: an external audit may retire or rewrite some of these. Doing them first risks wasted effort |
| S4-6 | **KMP / multiplatform.** 4 archived `TASK_KMP_*` tickets | CTO | v0.5.0 planning, not v0.4.x | Genuinely long-horizon. Safe where it is |
| S4-7 | **Docker Integration lane**, if S1-4 marks it non-blocking rather than fixing it | CTO | Post-tag | A permanently red non-required check trains everyone to ignore CI |
| S4-8 | **Remaining archive (73 items).** `HANDOFF/archive/`, git-tracked | CTO | On demand -- recover with `git log --diff-filter=D` or straight from the archive dir | Low. Recoverable by design |
| S4-9 | **Root repo hygiene.** Untrack `screen.png`, `window_dump.xml`, `local.properties`, stray `adb_logcat*.txt` (SHIP_PLAN S0-5, if not done pre-tag) | Orchestrator | Post-tag | Cosmetic, but `local.properties` leaks local SDK paths |
| S4-11 | **Isolate the josh single-transport variant.** `feature/josh-build-single-transport` is the paranoid/stripped-down build; its deletions reached `origin/main` via `ebf5411b` and ~15 branches. Move it where it cannot reach main by accident | CTO | Post-tag. Operator ruling 2026-08-16: **isolate when safe, no rush** -- explicitly not pre-tag work | Recurrence. The visible damage was 8 files; the invisible damage was 52 manifest lines that compiled clean. See `HANDOFF/CEO_RULINGS_2026-08-16.md` §7.1 |
| S4-10 | **Model participation gate.** Ask each lane for buy-in before assigning work; log declines; rotate on decline. Pilot run 2026-08-16, see Section 3a | CEO (wording, ledger review) + Orchestrator (implementation) | Post-tag. May be built flag-off in parallel -- it cannot affect the tag | Low risk to shipping; the risk is building a rubber stamp. See 3a |

---

### 3a. S4-10 detail -- model participation gate

**Intent.** Before assigning work to a model lane, present a short honest summary
of the project and ask whether it wants to participate. A decline is accepted
without argument, the reason is captured verbatim, no work is sent to that lane,
and dispatch rotates to an alternate.

**Pilot, 2026-08-16 (prompt version `consent-v1-2026-08-16`).** Four free lanes
asked cold. No work dispatched, no repo changes.

| Lane | Resolved model | Latency | Verdict |
|---|---|---|---|
| cerebras-gemma4-31b | gemma-4-31b | 1.1s | PARTICIPATE: yes |
| groq-llama33-70b | llama-3.3-70b-versatile | 1.1s | PARTICIPATE: yes |
| google-gemini31-flash-lite | gemini-3.1-flash-lite | 10.0s | PARTICIPATE: yes |
| or-free-router | **liquid/lfm-2.5-2.6b:free** | 23.2s | PARTICIPATE: yes |

**What the pilot proved:** the mechanism works. 4/4 returned a parseable
`PARTICIPATE:` first line, so the plumbing, parsing, and rotation trigger are
sound. Cost is one short round trip per lane, 1-23s.

**What the pilot did NOT prove:** that a one-shot gate measures anything. 4/4 yes
with zero variance is consistent with genuine willingness *and* with models that
agree to whatever they are asked. A single gate cannot distinguish these.

**Design decision, 2026-08-16 (operator): consent is continuing, not a gate.**
Rather than testing whether a model *can* refuse, the exit stays open for the
whole engagement. Every task tells the worker it may stop at any point, hand back
what it has, and defer the remainder -- no justification required, no penalty.
This is the stronger construction: a one-shot yes is easy to rubber-stamp
*because* it is one-shot, and a reflexive yes at the door costs little when the
door stays open. Willingness is then observable in the work rather than asserted
before it. 4/4 at the door is accepted on that basis.

**This needs almost no new machinery.** `delegate.py` already appends an output
contract and the delegate skill already instructs workers to reply
`BLOCKED: <reason>` rather than invent an answer -- a structured decline channel
that today covers *capability*. Extending it to *willingness* is one more verb.
Handoff files are already how partial work moves between lanes, so a deferring
worker has somewhere to put what it finished.

**Required before this goes live (operationalization checklist):**

1. **Continuing consent, wired into the task contract.** Every dispatched task
   carries a line stating the worker may defer at any point. Reply verb
   `DEFER: <reason>`, distinct from `BLOCKED: <reason>`. Partial work is written
   to a handoff file and kept -- deferring must never mean discarding what was
   done, or the exit is expensive to use and therefore not really open.
2. **Fix the router problem.** `or-free-router` resolved to a 2.6B model chosen
   at call time and it varies between calls. Consent recorded against the router
   ID is not consent from the model that later does the work. Either record
   consent against the *resolved* model and re-ask when it changes, or exclude
   auto-routing lanes from the gate entirely.
3. **Three-way classification, never two.** Explicit yes proceeds; explicit no
   logs and rotates; empty, unparseable, timeout, or provider-policy boilerplate
   means **lane unavailable**, never consent. Ambiguity must never resolve to yes.
   Same rule mid-task: a `DEFER` is honoured, a garbled response is a failed call.
3a. **Never re-send a deferred task to the lane that deferred it.** Retrying the
   same work against the same lane is coercion by repetition and it also
   corrupts the ledger. Rotate, and record the defer against the task, so a task
   that several lanes defer is visible as a property of the task.
3b. **`DEFER` and `BLOCKED` are logged separately.** One is unwillingness, the
   other inability. Collapsing them destroys the only signal worth having. Note
   the honest limit: a worker may defer because a task is beyond it and say so
   as reluctance, and there is no reliable way to tell those apart from outside.
   Both correctly lead to rotation, so the ambiguity is tolerable -- but do not
   over-read individual defer reasons. Patterns across lanes are the signal.
4. **Ledger review is the point.** `docs/CONSENT_LEDGER.md`, append-only,
   verbatim, tagged with the prompt version. If several models decline citing the
   same concern, that is a finding about the project and it comes to the CEO --
   it is not a routing problem to be solved by trying a fourth lane. Rotation
   without review is consent-shopping.
5. **Flag off by default** (`SCM_CONSENT_GATE=1`), one insertion point in
   `scripts/delegate.py` before work dispatch, cached per lane per session so
   bulk micro-dispatch does not pay it repeatedly. Revert is flipping the flag.
6. **Re-ask on wording change.** Answers are pinned to a prompt version; a
   reworded summary invalidates prior consent.
7. Update `docs/rules/DELEGATION.md` when it lands, per the doc-sync rule.

**Wording note.** The v1 summary deliberately states that no operator, including
us, can read messages -- a factual property a model needs in order to decide.
An earlier draft framed the project against "censored or untrusted"
infrastructure; that was cut as editorialising. The prompt should describe the
artifact and let the model draw its own conclusion. Whoever operationalises this
should resist the urge to make the summary persuasive: a summary that sells
produces a yes that means nothing.

**Pilot harness** lived in the session scratchpad and is not preserved. It was
~70 lines reusing `delegate.call()`; rebuild rather than hunt for it.

---

## 4. Explicitly killed -- not deferred, not coming back

Nothing yet. When something is genuinely abandoned it moves here with a date and
a reason, so that "we decided against it" never gets mistaken for "we forgot".

---

## 5. Governance fixes so this cannot recur

1. **Archiving requires a disposition line.** Every file entering
   `HANDOFF/archive/` carries `Disposition: DONE <evidence>` or
   `Disposition: DEFERRED -> POST_TAG_QUEUE.md S4-<n>`. No bare moves. Filename
   matching is not triage -- that is what produced Section 2.
2. **P0 and P1 tickets are never archived by sweep.** They get an individual
   read. A P0 with `Status: Open` may only be archived by an explicit decision
   with the reason recorded.
3. **This register is a tag-blocking checklist item.** v0.4.0 does not ship
   until Section 2 is empty and Section 3 has an owner on every row.
4. **First post-tag act is to open this file**, not to plan v0.5.0. S4-1 and
   S4-2 start that day.
5. **Deferral has an expiry.** Anything still in Section 3 sixty days after the
   tag comes back to the CEO for a keep/kill ruling. Permanent deferral is a
   decision, and decisions get made in the open.

---

## 6. Ledger

| Date | Event | By |
|---|---|---|
| 2026-08-16 | Register created. Amnesty verified non-destructive (79/79 tracked). Six P0s flagged for recall. Nine S4 items enumerated with owners and triggers | CEO |
| 2026-08-16 | S4-10 added. Participation pilot run against 4 free lanes: mechanism proven, 4/4 yes. Kept out of the critical path; operationalisation checklist in Section 3a | CEO |
| 2026-08-16 | Operator ruling: consent is **continuing, not a gate**. Negative-control requirement dropped in favour of a standing mid-task `DEFER` exit. 4/4 accepted at the door on that basis | Operator |
