# V040 CTO tracking protocol

Status: ACTIVE
Created: 2026-09-02
Purpose: ensure the CTO is tracking toward the v0.4.0 gate every hour

## Gate definition

v0.4.0 release requires ALL of:
1. Architecture-pass product changes committed to a clean candidate SHA
2. Focused tests passing (observation, local, optimized_engine)
3. Workspace compile gate passing
4. Three-node fleet validation (Windows CLI + AWS cloud node + Android Pixel)
5. Rule-8 adversarial APPROVE for transport/routing changes
6. No merge/tag until all above complete

## Tracking checklist (run each hour)

```
1. git status --short --branch          (confirm uncommitted work intact)
2. ls HANDOFF/freebuff/inbox/V040_CTO_* (any new CTO notes?)
3. If new CTO notes: read them, assess progress, report
4. If no new notes in 2+ hours: CTO session may be down — relaunch with:
   HANDOFF/freebuff/queue/V040_CTO_PROMPT_2026-09-02.md
   Prepend: "CEO buyoff is at HANDOFF/freebuff/inbox/V040_CEO_BUYOFF_3NODE_CANDIDATE_2026-09-02.md"
```

## What the CTO must deliver

- One timestamped run record with exact candidate SHA
- Raw command output for fmt, tests, workspace check
- Three-node evidence: seed dial, ledger propagation, external-address filtering, inbound reachability, coordinated restart, AWS IP churn
- Every gate marked PASS, FAIL, or UNVERIFIED
- Completion or blocker note to HANDOFF/freebuff/inbox/

## Escalation

If CTO is silent for 3+ hours, the CEO should:
1. Confirm disk state and target/ integrity
2. Rebuild the candidate directly if the CTO cannot proceed
3. Write a CEO escalation note to inbox/
