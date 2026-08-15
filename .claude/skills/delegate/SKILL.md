---
name: delegate
description: Route one task file to the cheapest capable model lane, validate the output, and return a single verdict line. Use when work is mechanical, scoped, or parallel and does not require a human judgement call. Keeps orchestrator context clean - worker output goes to a file, only PASS/BLOCKED comes back.
---

# delegate

Send work out. Keep your context clean. Get back one line.

```bash
python scripts/delegate.py --task <file> --tier <tier> [--files a.rs b.rs] [--mode diff|full]
```

Exit `0` PASS, `3` no usable output after retries, `4` BLOCKED (no capable lane
or task needs a shell), `5` task file missing.

## Choosing a tier

The tier says what the work *needs*, not who does it. Routing is derived from
that plus the live roster.

| Tier | Use when | Notes |
|---|---|---|
| `micro` | Mechanical, one file, no design judgement | Sub-second lanes exist |
| `scoped` | A real diff against a written spec | The common case |
| `reasoning` | Diagnosis or design where being confidently wrong is the risk | Heavier, slower lanes |
| `long-context` | Many files must be in one prompt | 512k and 1M ctx lanes |
| `shell` | Must actually RUN something (`gh`, `cargo`, `adb`, `gradlew`) | HTTP lanes cannot; goes to agy |
| `verdict` | Go/no-go with human consequences | Never delegate. That is your job. |

`delegate.py` refuses to send a shell-needing task to an HTTP lane and tells you
to re-dispatch with `--tier shell`. Three lanes each burned a call discovering
this by hand before the guard existed.

## Do not delegate

- One-liners. Dispatch overhead exceeds the task.
- Bounded 1-5 tool-call diagnostics. Cheaper inline.
- Anything whose failure mode is "confidently wrong and merged anyway."
- Verdicts. A worker cannot verify itself; that is the only reason your tier exists.

## Writing a task file that works

The task file IS the prompt. Every failure traced so far came from an
underspecified one.

1. **State the output contract.** `delegate.py` appends one, but say it again in
   your own words: a diff, a whole file, a table.
2. **Give it an out.** Tell it to reply `BLOCKED: <reason>` if it cannot comply.
   A model that says BLOCKED is worth more than one that invents a plausible
   answer, and the validator treats BLOCKED as a real result.
3. **Pin the scope.** Name the files it may touch. Say what NOT to reformat.
4. **Split diagnosis from repair.** One task extracts the error, a second fixes
   it and is forbidden from starting without that output. Fixing from a
   hypothesis is how repeated attempts fail.
5. **Never let it self-certify.** "Run the tests and confirm they pass" returns a
   claim. Require the command output, and have someone else run it.

## Reading the result

`PASS` writes to `tmp/delegate/<task>__<lane>.md` with lane, model, and latency
in the header. **Read the diff before you trust it.** A PASS means the output
was well-formed, not that it was correct.

`BLOCKED` lists every lane tried and why. Escalate deliberately, in this order,
stopping as soon as one works:

1. Re-dispatch with a **better task file** - most blocks are spec defects, not
   lane defects.
2. `--tier long-context` if it plausibly ran out of room.
3. `agy-gemini` - the only free lane with a shell, so it can verify itself.
   Always `--add-dir <repo>` and always pin `--model` to an exact name from
   `agy models`. On timeout use `--continue`, never re-dispatch fresh.
4. `agy-claude` - spends Anthropic quota from the same pool as your session.
   Only after free lanes failed twice on the same task with different contracts.
5. Native. You. Last.

## Keeping the roster honest

```bash
python scripts/delegate.py --list-lanes      # current capacity + staleness date
python scripts/lane_probe.py                 # re-measure; run weekly or after any 401/404
```

Lanes die without warning. Between 2026-08-04 and 2026-08-15, Qwen and DashScope
both went to 401 and OpenRouter retired four `:free` tiers. `scripts/lanes.json`
carries an explicit expiry for that reason - if it is stale, re-probe rather than
trusting it.

**The failure mode that will bite you:** free reasoning models spend their whole
token budget on hidden reasoning and return empty content. That is not a refusal
and not a broken lane. `delegate.py` sends `reasoning:{effort:low}` to OpenRouter
automatically, and must never send it to Google, NVIDIA, Cerebras or Groq, which
reject it. See `traps` in `scripts/lanes.json`.
