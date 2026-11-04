DUAL-ENGINE 24/7 ORCHESTRATION (v1) — addendum to WINDOWS-24-7.md
and CONTINUITY.md. The lanes no longer run on one meter. There are
two engines, two different limit shapes, and one scheduler that
treats limits as terrain, not accidents: hit the wall exactly,
checkpoint, resume at the reset second. Lucas's goal is deliberate:
burn the full Codex weekly allowance, spend the banked reset, burn
the next week too — with zero human whack-a-mole in between.

THE ENGINES
TWO METERS, FULLY SEPARATE (owner-confirmed 2026-10-19, Lucas —
this overrides the earlier conservative draft): sessions on the
connected GPT account burn ONLY GPT/Codex usage and ZERO
Freebucks; Freebuff-native models burn ONLY Freebucks. The pools
are independent and additive — a Codex wall never touches the
Freebucks balance and vice versa. Plan against both at full size.

Evidence classes used in this document: [code] = verified from the
shipped Freebuff desktop code (v0.0.182); [web] = verified against
the named public source; [owner] = owner-confirmed from Lucas's own
account (no public source exists); [box] = only verifiable on the
lane box, carried in the on-box checklist. Load-bearing numbers
carry their class inline; the SOURCES appendix collects them.

Engine A — Codex via the connected ChatGPT account (inside Freebuff),
model GPT-6 Luna. THE ENDURANCE ENGINE (the workhorse): carries all
lanes by default. It is metered in TOKENS (the 5h + weekly windows
below) but its sessions have NO time limit [owner] — a session runs as long
as it needs to run. So Engine A is spent on wall-clock, not tokens:
work that waits, paces, supervises, and judges (see the ALLOCATION
DOCTRINE below).
- Why Luna [web: OpenAI Codex pricing/usage figures]: Luna fits
  ~350–3,000 local messages per 5-hour window vs ~5–45 for GPT-6
  Astra. One to two orders of magnitude more lane throughput per
  window; that is what makes 24/7 multi-lane work fit inside a
  subscription at all.
- Limit shape (account-shared) [web]: a 5-hour window + a
  weekly window, shared across EVERY Codex surface on the account
  (CLI, IDE, web, Freebuff connection) — parallel sessions do NOT
  multiply quota, they only spend it faster. Fast mode burns
  included usage at ~2.5x [web] — BANNED for lane work.
- Banked resets: the account holds at least one banked "full
  reset" (restores BOTH windows). They expire; redeem late (when
  nearly out), never early — unused quota is lost at redemption.

Engine B — Freebuff's own allowance, $8/mo plan. Two surfaces
report this tier differently: Lucas's account shows 250
Freebucks/day; the official plans page expresses the same tier
as ~50 session-hours/day refilled daily + 60 monthly bonus hours,
metered per model multiplier. The scheduler NEVER hardcodes
either number — it reads the live balance/ledger and plans
against what the product actually reports that day.
Known rate (owner-confirmed): GPT-6 Luna native = 20 Freebucks
per hour, so the daily 250 buys ~12.5 hours of native Luna —
a full second Luna engine that owes nothing to the Codex
windows. The roster also includes GPT-6.1 Sol (sometimes),
Gemini 4 Argon, and Muse Spark 1.3 (owner-reported) for
one-off specialist needs. THE THROUGHPUT ENGINE (owner-confirmed
2026-10-19): a Freebuff session is ONE HOUR, hard-ended — but
tokens inside that hour are UNMETERED. Engine B is therefore
metered in TIME, not tokens, the exact opposite of Engine A,
and it is spent on token density: pre-planned execution bursts
that produce the maximum work per session-hour (ALLOCATION
DOCTRINE below). It remains the overflow engine secondarily:
- One-off needs, single-role bursts, and any work that arrives
  while Engine A is walled.
- Hour-block discipline from WINDOWS-24-7.md still applies:
  sessions batch inside a started hour; sleep itself is free.
- Named non-retaining models only (stealth betas retain prompts).

Engine C — Freebuff's cheap/unmetered lanes (MiMo Flash Slow etc.)
for Shepherd-class work: watching CI, polling, journal upkeep —
judgment-light work that should never burn Luna quota.

WORK ALLOCATION DOCTRINE (owner, 2026-10-19)
The engines are metered on opposite axes, so work is allocated
by axis, never by habit:
- Engine A is metered in TOKENS and unlimited in TIME. Spend
  its wall-clock freely and its tokens stingily: assignments
  whose hours are dominated by waiting (CI shepherding — poll,
  sleep, act rarely; gate runs; merge-train pacing; watch
  loops) plus the judgment work only it should do (planning,
  reviews, staging burst plans). Slow, sleep-heavy, low-token
  work is what keeps Engine A alive around the clock.
- Engine B is metered in TIME (one hour, hard end) and
  unlimited in TOKENS inside the hour. Spend its tokens
  maximally and its minutes never on waiting: dense,
  pre-decided execution only. A burst hour spent thinking is
  wasted; a burst hour spent waiting is theft — the hour ends
  either way.

CLASSIFY EVERY QUEUE ITEM on two axes before assignment:
- TOKEN DENSITY: output tokens per minute of work. High =
  bulk generation, large refactors, test suites, doc sweeps,
  mechanical transforms across many files.
- DECISION DENSITY: judgments per step. High = design,
  review verdicts, debugging unknowns.
High token / low decision -> BURST QUEUE (Engine B).
Low token-rate, long-duration, or high decision -> SLOW LANE
(Engine A). An item high on BOTH is SPLIT: the slow lane
decides it into a staged plan first; the execution half then
goes to the burst queue. Default role mapping: Shepherd and
Landing ride the slow lane (few actions, long CI waits);
Reviewer and planning ride the slow lane (judgment); bulk
Implementer work rides bursts; Engine C keeps the pure-watch
work it already has.

THE BURST PROTOCOL (one Engine B session, 60 minutes, hard end)
1. NEVER start a burst without a staged BURST PLAN. With no
   plan ready, the burst runs fallback catalog work only
   (test generation, docs, mechanical sweeps from the
   standing fallback list) — never open-ended exploration.
2. Mission settings for a burst: effort 3 (the 60-minute
   target matches the session length), mission text = the
   burst pointer in MISSION-PROMPT.md Part 0's family: read
   the staged plan, execute it in order, checkpoint as below.
3. Inside the hour: NO questions (a blocker goes to the
   journal / OWNER QUEUE and the session moves to the next
   plan item); NO design decisions (a plan gap is logged and
   given the smallest safe reading, or the item is skipped);
   push at least every 10 minutes; items strictly in plan
   order; a stuck item gets ONE retry, then skip-and-log —
   never sink the hour into a single item.
4. The last 5 minutes are reserved, always: start nothing
   new, push everything, write the BURST HANDOFF into the
   journal and state docs — items done with commit ids,
   items skipped and why, plan gaps found, verification
   state. The hour ending mid-item is normal and expected;
   the handoff is what makes the cutoff free.
5. After the burst, the SLOW lane reviews the burst output
   (Jev gate + review pass — bursts never review their own
   work; authorship exclusion already demands this), queues
   fixes, and stages the next burst plan.

THE BURST PLAN (written by the slow lane: token-cheap to make,
token-dense to execute). An ordered list of fully-specified
items — exact files, exact change, exact acceptance check per
item — sized to OVERFILL 60 minutes by ~25%. Overfill is free:
a plan that runs dry mid-hour burns unmetered tokens on
nothing, while the remainder simply carries into the handoff
and the next plan. Every plan ends with the fallback catalog
pointer for gaps.

SLOW-LANE PACING + THE BURST FACTORY: the slow lane batches
its status checks (one poll per ~10 minutes, never per
minute), prefers assignments with built-in sleeps, and shapes
its token burn to the window (scheduler table rows 3-5).
Standing duty: AT LEAST ONE staged burst plan is ready at all
times. A burst hour lost because no plan existed is a
scheduling failure charged to the slow lane and journaled as
such — the plan factory is part of the slow lane's job, not
an afterthought.

PARALLELISM: bursts run SERIALLY by default. The day's
Freebucks buy a fixed count of session-hours (250/day at
Luna's 20/hour = ~12.5), and parallel bursts spend the same
hours faster for zero extra tokens — tokens inside each hour
are already unmetered, so parallelism buys wall-clock only.
Run parallel bursts only against a hard deadline, recorded
as an owner-visible journal decision.

USAGE TELEMETRY (the scheduler's eyes — scripted, zero agent time)
- Source of truth: Codex `/status` — shows 5-hour % left, weekly
  % left, exact reset timestamps, credits, and banked resets.
  Caveat (verified): values go stale; if >15 min old, send one
  message to refresh before trusting them.
- Scripted read — THE SOURCE FOLLOWS THE SURFACE. Which surface
  runs Luna decides where the numbers live; do not assume the
  Codex CLI's files exist for work done inside Freebuff:
  * Freebuff-connected sessions (the default lanes): the
    in-session /status card is the source, read by the vision
    driver (fallback 2 below is, for these lanes, the primary
    path). The on-box checklist confirms this session's /status
    reports the ChatGPT account's windows.
  * Standalone Codex CLI sessions (if used): the rollout-file
    parse (path 1 below) is primary — it is cheaper and exact.
  Both paths write the same usage.json; the scheduler neither
  knows nor cares which surface produced it.
  1. Parse the newest Codex session rollout file on the box
     (%USERPROFILE%\.codex\sessions\**\*.jsonl): in current Codex
     CLI versions, token_count events carry rate_limits (primary
     = 5h window, secondary = weekly, used_percent + resets_at).
     HEDGE: this field layout is version-dependent and is NOT
     depended on until the on-box checklist confirms it on the
     actual install; the parser validates every field and, on
     any mismatch, logs the failure and falls back to (2) rather
     than guessing. Pure file read — no UI, no agent tokens.
     usage-ledger.ps1 polls this every 10 min alongside
     watchdog.ps1 and writes usage.json (box-local path, below).
  2. Fallback: the Jev vision driver reads the /status card off
     the screen (see VISION DRIVER below).
  3. Freebuff side: watchdog records session-hours per role per
     day in the burn ledger (existing WINDOWS-24-7 mechanism);
     Freebucks balance is read from the Freebuff UI by the vision
     driver when visible, else estimated from the ledger.
  resets_at values are absolute timestamps from Codex itself;
  the scheduler compares them against GitHub server time (the
  CONTINUITY.md clock rule), never the box's local clock math.
- usage.json lives OUTSIDE every git checkout, at
  C:\scm\state\usage.json (box-local, never committed): it is
  rewritten every 10 minutes, and a file like that inside a
  checkout would invite exactly the commit/pull conflicts the
  ff-only discipline exists to avoid. State docs name the path;
  sessions and watchdog read it from there.
- usage.json fields: engineA {h5_used_pct, h5_resets_at,
  weekly_used_pct, weekly_resets_at, banked_resets, as_of},
  engineB {freebucks_est, hours_used_today per role}, engineC
  {unmetered: true}. Every spawn decision reads usage.json first.
  Stale (>20 min) usage.json = scheduler flies blind: it may run
  ONE Luna session only until telemetry refreshes.
- Engine choice never changes the quality bar: every merge path
  passes the same Jev gate and cross-role review regardless of
  which engine ran the cycle, and the two judgment-heaviest acts
  — Rule-8 review verdicts and the three-node release gate —
  run on Engine A (Luna) cycles only. Weaker engines do volume;
  they never get the final say.

DRIVER OWNERSHIP (who continues a session — exactly one at a time)
Three mechanisms can move a session forward: Freebuff's mission
mode (inside the session), the vision driver (pasting prompts),
and the watchdog (spawning fresh sessions). Two of them driving
the same session at once is a defect: the watchdog would park a
session mission mode meant to continue, or mission mode would
retry against a wall the watchdog is already scheduling around.
The split is by phase, and it is absolute:
- INSIDE a live session, within a usage window: mission mode
  owns continuation. The watchdog never injects, pastes, or
  interrupts; vision duty 2 (mission feeding) applies ONLY to
  sessions NOT running mission mode.
- AT a wall, at session death, and across windows: the watchdog
  owns recovery (park, schedule, respawn). Mission mode's job at
  a wall is defined in the mission prompt's hard lines: stop,
  checkpoint, exit — never retry against the wall, never loop
  waiting for a reset from inside a metered session.
- The vision driver is the watchdog's hands, never a third
  driver: it performs watchdog-decided actions (wall watch,
  status reads, reset click) and originates no work and no
  strategy. Its one judgment duty is bounded: resolving
  blocking UI decisions through the DECISION ESCALATION
  PROTOCOL below — Jev decides from repo canon, the driver
  clicks. Anything outside the protocol's authority is
  parked and escalated, never improvised.
MID-RUN WALL DISCIPLINE (the running session's own duty)
Walls do not only arrive between sessions. Telemetry updates as
messages flow, so a session deep in a long build can cross 90%
without any spawn decision ever seeing it. Therefore: at every
step boundary (each commit, each CI wait, each subagent return),
the running session re-reads usage.json — a local file read,
near-zero cost. At >=90% on either window it executes the
checkpoint immediately, mid-objective if necessary: the
journal records the exact step reached, so the next session
resumes the step, not the objective. A wall hit mid-response
with no checkpoint is the accepted residual loss: one response
tail, bounded by the push-at-every-step-boundary rule.

SCHEDULER DECISION TABLE (usage.json + queue state -> action;
the watchdog evaluates top-to-bottom, first matching row wins)
1. Telemetry stale >20 min -> run at most ONE Engine-A session;
   refresh telemetry before any further spawns.
2. Role queue empty -> no spawn (sleep is free), any engine.
3. Engine A 5h <75% AND weekly <95% -> spawn on Engine A (Luna).
4. Engine A 5h 75-90% -> spawn Engine A only for checkpointable
   work (reviews, docs, CI shepherding); long builds wait.
5. Engine A 5h >90% or weekly =100%, no banked reset -> park A;
   role's next cycle on Engine B if its ledger allows, else C
   for Shepherd-class work, else sleep until the reset.
6. Weekly remaining <= REDEEM_AT_REMAINING_PCT (default 0%)
   AND banked reset exists AND auto-redeem armed -> vision
   driver redeems; immediate Luna start (new weekly clock);
   all parked A roles resume per the herd stagger.
7. Trigger reached AND banked reset exists AND auto-redeem
   disarmed -> OWNER-ITEM (one line, with the reset's expiry);
   lanes run on B/C meanwhile. Never nag: one item, updated
   in place.
8. Both engines exhausted, queue non-empty -> sessions park;
   watchdog wakes at the earliest reset (5h, daily refill, or
   weekly, whichever is first); audit reports the stall once.
9. Respawn verification: any spawned session must heartbeat
   within 10 minutes or the spawn counts as failed — retry once,
   then follow CONTINUITY.md's spawn-failure rule (3 consecutive
   failures -> OWNER QUEUE).
10. Engine B dies mid-hour (allowance end): the Freebuff session
    ends wherever it stands; the cycle's checkpoint rule is the
    only protection, which is why cycles keep work pushed at
    every step boundary, not just at cycle end.
11. ALLOCATION (refines rows 3 and 5): within any spawn, work
    is assigned per the ALLOCATION DOCTRINE — burst-queue
    items launch on Engine B when a staged burst plan exists;
    slow-lane items launch on Engine A; an Engine B spawn with
    no staged plan runs fallback catalog work only. If Engine
    B hours remain today and no burst plan is staged, the
    slow lane's next cycle stages one before anything else
    optional.

COMPONENT SPEC — usage-ledger.ps1 (new; ~120 LoC PowerShell)
- Runs from the same 10-min Task Scheduler cadence as watchdog.
- Reads the newest rollout JSONL under %USERPROFILE%\.codex\
  sessions (by last-write time), extracts the LAST token_count
  event's rate_limits; validates: both windows present,
  used_percent in 0-100, resets_at parseable. Any validation
  failure -> writes usage.json with telemetry_status: "degraded"
  + the failure reason, and exits 0 (never crashes the watchdog).
- Merges the Freebuff burn ledger (per-role session-hours today)
  and writes usage.json atomically (temp file + rename) next to
  the state docs. Fields as listed under USAGE TELEMETRY above,
  plus telemetry_status and as_of (GitHub server time anchor).
COMPONENT SPEC — watchdog.ps1 deltas (extends WINDOWS-24-7 L1)
- Before any spawn: read usage.json, apply the DECISION TABLE.
- On any Engine-A park: record the wake time (earliest applicable
  resets_at) in the state docs so a box reboot cannot lose it;
  the AtLogOn sweep re-derives it from usage.json anyway.
- Post-redemption: trigger one immediate Luna spawn (row 6).

SCHEDULING POLICY (watchdog-enforced, not session-enforced)
- Window shaping: the 5h window is a budget to SPEND WELL, not
  merely survive. Early in a fresh window: launch the big
  implementation batches. Above ~75% used: no new long builds —
  only work that checkpoints cleanly in minutes. Above ~90%:
  checkpoint-and-park (write state + next prompt, exit); the
  watchdog respawns at h5_resets_at + 60s.
- Reset herd control: when several roles are parked on the same
  reset, they do NOT all respawn at once — a simultaneous restart
  burns the fresh window's first slice on duplicated state reads
  and lease contention. Respawn order: Landing first, then one
  Implementer, then the rest, staggered 3 minutes apart; each
  confirms its lease before the next starts.
- Concurrency: at most 3 Engine-A sessions at once (Landing +
  2 Implementer/Shepherd checkouts), because quota is shared —
  more sessions add coordination burn, not throughput. Engine B
  sessions don't count against this cap.
- Weekly strategy: run the week at full pace; do not ration Luna
  early in the week to "save" quota — unspent weekly quota does
  not roll over, so rationing is pure loss. When weekly hits
  ~100%: if a banked reset exists, redeem it (below), which
  restarts BOTH windows; else park Engine A until weekly_resets_at
  and shift live lanes to Engine B/C.
- Banked-reset redemption: an account action on the usage page
  (Settings -> Usage -> "Use reset"), driven by an ARMING
  MECHANISM (owner decision 2026-10-19, Lucas): auto-redeem is
  ALLOWED and armed in the app; the trigger is a remaining-%
  threshold on the WEEKLY window — REDEEM_AT_REMAINING_PCT,
  DEFAULT 0% (redeem only when the week is fully spent),
  adjustable by the user to any value 0-100 (e.g. 10 = redeem
  when a tenth of the week remains, so lanes never sit at zero
  waiting for the click). The watchdog mirrors the same two
  settings (armed + threshold) in its config; the vision driver
  performs the click when the trigger fires. A 5-hour wall NEVER
  consumes a banked reset on its own — that window refills in
  hours, and a full reset spent there wastes the weekly refill;
  the threshold governs the weekly window only. Expiry watch:
  if a banked reset expires within 48h and weekly usage is
  already >50%, the OWNER-ITEM escalates (redeem-before-expiry
  beats lose-it) regardless of the threshold.
  Immediately after any redemption, the scheduler starts one
  Luna session at once: the new weekly window's clock starts at
  the first post-reset message, so an idle redeemed week starts
  its 7-day countdown late and wastes allowance.
- Engine failover is per-role, not global: a walled Engine A
  moves THAT role's next cycle to Engine B with the same prompt
  (the mission prompt is engine-agnostic; only the STATE block's
  LAST WALL line changes). Roles return to Engine A at its reset.
  Because the meters are fully separate (owner-confirmed), a
  Codex weekly wall still leaves ~12.5 h/day of native Luna on
  Engine B — the "both engines walled" case now requires the
  Freebucks day to be spent too, which the burn ledger makes
  visible hours in advance.

RECOVERY — EXACTLY WHERE YOU LEFT OFF
The mission prompt's checkpoint rule is the recovery mechanism:
every pass ends with work pushed, state docs updated, journal
appended, and the end-of-pass handoff written from the template.
A limit wall is just an unscheduled pass end: same checkpoint,
plus LAST WALL (engine, limit, reset time from usage.json). The
respawned session's first act is the state read — it resumes the
recorded operation, never a reconstruction. Walls therefore cost
the tail of one response, never the thread of the work.

THE JEV VISION DRIVER (supervision of the GUI itself)
Why it exists: Freebuff mission mode is a GUI loop — a manager
agent (Buffy) auto-continues each tab, but only inside the
desktop app, and only while the app can see and click. CLI
spawning covers headless starts; the vision driver covers
everything the app can't do for itself: a session sitting at a
limit banner, a stalled mission tab that needs its
pick-it-back-up message, a "Use reset" button that needs
clicking on the ChatGPT usage page, and the Continue prompts
and decision pop-ups that freeze a lane mid-work — including
answering those pop-ups, through Jev, when the repo's own
philosophy already contains the answer.
- Build line (already in flight, harness repo): driver-core
  screenshot -> Jev judgment -> action, per docs/hourglass-vision.md
  and the computer-control plan (issues #270-273: launch_app, NL
  planner, vision verification at Jev >= 0.85, GUI wiring).
  Vendored driver-core in harness is v3.4.0; repo is at v3.5.0
  (audit infra only — no capability change; don't chase it).
- Driver duties, in order of value:
  1. WALL WATCH: recognize the Codex/Freebuff limit screens;
     confirm the session checkpointed; record the wall; do NOT
     click through or retry — schedule the resume.
  2. CONTINUE + POP-UP ADJUDICATION: click Continue/resume
     prompts so lanes never sit frozen at a banner, and answer
     decision pop-ups through the DECISION ESCALATION PROTOCOL
     (below) — the driver is how a lane gets its blocking
     question answered at 3am without a human and without
     guessing.
  3. MISSION FEEDING: Buffy auto-continues a mission tab on its
     own — feeding is needed only when the tab reports the
     built-in failure state ("Mission couldn't get a decision…
     Send a message to pick it back up") or sits idle with queue
     remaining: then the driver sends the pick-it-back-up message
     (or the Part 3 brief for spawned-mode sessions), so no
     human nudges a stalled tab at 3am.
  4. STATUS READS: /status card + Freebucks balance as telemetry
     fallback (above).
  5. RESET REDEMPTION: on the usage page, click "Use reset" —
     only when auto-redeem is armed and the remaining-%
     threshold has fired (above).
  6. HUNG-SESSION EVIDENCE: screenshot the stuck state into the
     journal BEFORE the watchdog's PID reap, so a kill leaves a
     diagnosis, not a mystery.

DECISION ESCALATION PROTOCOL (how a blocking pop-up gets
answered without a human — and when it must not be)
A lane blocked on a UI decision is burning its window on
nothing. The driver answers these in three classes:
- CLASS 1, MECHANICAL: Continue / resume / dismiss / retry
  prompts whose only effect is letting the lane keep doing
  what it was already doing. The driver clicks on screen
  recognition alone (Jev vision >= 0.85). No escalation —
  a Continue click decides nothing new.
- CLASS 2, JUDGMENT: a pop-up that chooses between real
  options (approaches, scopes, approval of a next step,
  which file/plan/test to proceed with). The driver assembles
  a CONTEXT PACKET — screenshot, the pop-up's exact text and
  options, the lane's current objective and queue item from
  the state docs, and the governing canon excerpts (roles
  DEFINITIONS, the repo plan docs, the linked issue/PR) —
  and puts it to Jev through the harness driver stack. This
  is the pre-ask check, mechanized: does the repo's
  philosophy yield ONE correct, safe, aligned answer from
  this context? The driver clicks only when Jev returns
  that answer at >= 0.95 calibrated confidence. Below the
  bar — or when two options are both defensible, which means
  it is a genuine fork, not a pop-up — the driver does NOT
  guess: screenshot, journal the packet, park ONLY that lane
  at the decision (other lanes keep working), and route it:
  OWNER QUEUE if it is a genuine end-state or ramifications
  call, otherwise to the lane's next live session, which
  decides with full session context.
- CLASS 3, NEVER THE DRIVER'S: anything on the owner-only
  hard lines (spend, payments, keys/credentials, branch
  deletion without the purge audit, HV-0), anything
  destructive or irreversible (force-push, delete, send,
  publish), and anything the canon does not cover at all.
  Screenshot, journal, OWNER QUEUE, park the lane. No Jev
  score overrides this class: Jev adds reasons to escalate;
  it never manufactures authority.
Every Class 2 and 3 event is journaled with its packet,
verdict, and confidence — the audit trail is what makes
autonomous clicking reviewable after the fact. Every click
is verified with a post-click screenshot; if the resulting
state contradicts the expected one, the driver stops
touching that lane, journals the mismatch, and escalates —
it never clicks its way out of a surprise.

GENERAL RULE, unchanged: every vision judgment that triggers
an ACTION is Jev-gated (>= 0.85 floor from the vision plan;
>= 0.90 for the reset click and for declaring a session hung).
A judgment below bar means the driver does nothing and
journals the screenshot — a vision driver that guesses is
worse than none.
- Degraded mode: until the vision driver lands on the box, the
  watchdog + CLI spawn path (WINDOWS-24-7 Layer 1/2) runs the
  whole system; mission feeding is done by Freebuff's own
  mission mode internally, and walls are detected from
  usage.json + heartbeat staleness instead of screenshots.
  Nothing in this document waits on the vision driver except
  duties 2, 3, 5 and 6 being fully hands-free.

A WALL NIGHT, WORKED EXAMPLE (illustrative, from the rules above)
23:10 — Implementer is mid-batch; usage.json shows 5h at 88%,
  resets 00:40. Scheduler has already barred new long builds.
23:25 — 5h crosses ~90%: session checkpoints (branch pushed,
  journal + next prompt written, LAST WALL recorded), exits.
  Landing's session hits the same wall 23:31, same checkpoint.
00:41 — Watchdog respawns Landing (resets_at + 60s). 00:44 one
  Implementer. 00:47 the second. Each reads state, takes its
  lease, resumes the recorded operation.
04:55 — Weekly remaining hits the armed threshold (0%). A
  banked reset is on the account and auto-redeem is armed:
  vision driver redeems on the usage page (Jev-verified click),
  scheduler immediately starts a Luna session so the new weekly
  clock starts, lanes continue into the fresh week. Total human
  involvement: none. The 6h audit reports: 2 walls, 3 resumes,
  1 reset redeemed.

WHAT LUCAS EVER SEES
- The 6h audit gains three lines: Engine A (5h %, weekly %,
  next reset), Engine B (Freebucks/hours burned today per role),
  walls hit + resumes executed since last audit.
- OWNER QUEUE items only: a banked reset about to expire unused,
  both engines walled at once with queue non-empty. The redeem
  threshold is his in-app setting, never an owner-queue item.
  Everything else is machinery.

ON-BOX CHECKLIST (unverified off-box, ~10 min, one time):
- Confirm the rollout-JSONL rate_limits fields on the actual
  Codex install (path + field names) and pin them in
  usage-ledger.ps1; fall back to vision reads if absent.
- Confirm Freebuff's mission entry point on the box build
  (Queue -> mission text + effort slider) and that the Windows
  build matches the verified Linux build's mechanics (mission
  cap 4,000 chars, Buffy decider); the desktop auto-updates, so
  a newer Windows build may differ — re-verify after big
  updates.
- Confirm the connected-ChatGPT model picker exposes GPT-6 Luna
  by that name in Freebuff on the box, and that /status inside
  that session reports the ChatGPT account's windows.
- SETTLED by the owner 2026-10-19 (no box check needed):
  connected-GPT sessions burn zero Freebucks; Freebuff-native
  Luna is 20 Freebucks/hour. (Was the two-meters open question.)

SOURCES & CONFIDENCE (every load-bearing claim, 2026-10-19)
- /status shows 5h %, weekly %, reset timestamps, credits;
  values can go stale (>15 min = refresh with a message):
  multiple independent Codex-usage guides describing the actual
  CLI card. HIGH confidence.
- 5h + weekly windows are account-shared across Codex surfaces
  (CLI/IDE/web/Work); a banked reset restores BOTH and moves the
  weekly reset date; the new weekly clock starts at the first
  post-reset message; banked resets expire unreplaced:
  OpenAI help-center guidance as reported by whenresets.com,
  tiboreset.com, vanja.io (consistent across all three). HIGH.
- Luna throughput ~350-3,000 local messages per 5h window (Plus
  class) vs Astra ~5-45; Fast mode burns included usage ~2.5x:
  OpenAI Codex pricing page figures via madrobot.blog's dated
  reading of it. MEDIUM-HIGH (secondary reporting of the
  official page; message counts are ranges, not quotas).
- Freebuff $8 tier: Lucas's account reports 250 Freebucks/day
  (owner-reported, authoritative for HIS account); official
  plans page (freebuff.com/plans, read 2026-10-19) expresses the
  tier in session-hours (50/day + 60 monthly bonus; Luna ~13
  hrs/day). Both recorded; the scheduler trusts neither number
  and reads the live balance.
- Freebuff mission mode: VERIFIED from the shipped Freebuff
  Desktop build (app 0.0.182, Linux AppImage from freebuff.com,
  orchestrator bundle modules src/shared/mission-prompt.ts +
  src/server/services/mission.ts, read 2026-10-19). Mechanics:
  a manager agent ("Buffy", freebuff-desktop-autorun) decides
  after each coding turn — one next input (skill + written
  prompt) or stop — from a clipped trace (agent prose, one line
  per tool call, receipts, self-grades), never from code.
  Passes run in the SAME conversation. Mission text hard-caps
  at 4,000 chars. Effort 1-5 = 15/30/60/120/240-min targets.
  Deciders: hosted autorun agent, Claude Code, or Codex
  (read-only sandbox). No auto-redeem code exists in the build.
- Vision driver: harness docs/hourglass-vision.md + computer-
  control plan (issues #270-273) + driver-core versions, all
  read from the local harness checkout 2026-10-19. HIGH.
- Rollout-JSONL rate_limits layout: pattern from current Codex
  CLI versions; explicitly NOT relied on until the on-box
  checklist confirms it (see telemetry hedge).
