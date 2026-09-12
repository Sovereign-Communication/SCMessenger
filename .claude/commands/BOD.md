# /bod — convene the SCMessenger Board of Directors

Convene the Board of Directors of SCMessenger to adjudicate key architectural,
cryptographic, or philosophical proposals against the canonical Repo Doctrine.

## Operating boundary

- The Board operates via automated 5-model panel verification plus judge
  concurrence using `sovereign-harness`.
- Consensus requires **100% unanimity (5/5)** of panel models, plus concurrence
  from the judge model. Any dissent or shortfall fails closed (`REJECTED` or
  `DEFERRED`).
- All runs are cost-bounded to a maximum of **$0.10 (10 cents)** per execution.

## Load order

Read these tracked files before acting:

1. `AGENTS.md`
2. `HANDOFF/BOD_STATE.md`
3. `HANDOFF/CEO_STATE.md`
4. `HANDOFF/CTO_STATE.md`

## Invocation

Execute the Board evaluation script:

```bash
python scripts/bod_governance.py --proposal "Proposal to evaluate" --record
```
