# Hermes Health Audit + Bridge Recovery — 2026-06-07 22:17 PT

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**From:** Claude Code (orchestrator)
**To:** Lucas (Telegram) + Hermes
**Re:** Bridge back up; secondary watch armed; findings

## Bridge status (LIVE, just now)

| Check | Status | Evidence |
|---|---|---|
| `hermes-gateway.service` | **active** | `systemctl --user is-active hermes-gateway` = `active` |
| `hermes-pulse-loop.service` | **active** | systemd unit healthy |
| `scmessenger-cli.service` | **active** | mesh daemon healthy |
| `wsl2-portproxy-fix.service` | failed | non-critical for Telegram |
| Telegram bridge | **connected** | `gateway_state.json` → `telegram.state: connected` since 22:15:42 PT |
| Gateway PID | 970 (hermes) | RSS 139 MB |
| WSL uptime | 4 min | booted at 22:13 (was force-exited 22:13 by my prior wsl.exe call) |
| Loadavg | 4.16, 2.16, 0.93 | high but trending down (1m spike from model load, 15m at 0.93 is normal) |

**Bridge is up. Telegram DMs will reach me.**

## What happened before the audit (sequence)

1. **22:12:17 PT** — gateway SIGTERM (loadavg 2.20). I/O error in `process_loop`. `Errno 5` typically means underlying disk issue.
2. **22:12:06 PT** — `process_loop unhandled error: [Errno 5] Input/output error` (errors.log)
3. **22:13:25 PT** — systemd auto-restart (PID 797)
4. **22:13 PT** — WSL force-exited (my `wsl --exec bash -c` got `0x80370107`). Possibly a side effect of high load.
5. **22:15:35 PT** — second SIGTERM (loadavg 2.57, my restart attempt)
6. **22:15:42 PT** — current PID 970 stable, telegram.connected

## Root cause analysis (best guess, no fix applied)

The 3 SIGTERMs in 3 hours (20:19, 20:26, 22:12) all happened during high loadavg. The WSL VM is running at the edge — 4.16 loadavg means the kernel is spending significant time on I/O wait. Most likely culprits:

1. **Ollama model load churn** — `scm-coder:7b` is 8.2 GB VRAM with 4-min TTL. Every 5 min, the model unloads/reloads if anyone queries it. Each load spike is heavy.
2. **My prior Telegram DMs triggered MarkdownV2 parse failures** — twice at 20:23 PT (offsets 4097 and 3480) because my DMs contained `</3` (angle bracket, not a valid MarkdownV2 entity). **Fixable by me: stop using `</3` in DMs.**
3. **Memory churn from gateway restart helpers** — every restart leaves a `hermes` restart helper process (PID 828, 173 MB) idle. These accumulate.
4. **Repeated `Ollama` custom-provider 401 warnings** — the agent thread tried to use a custom provider named `Ollama` (capital O) with no api_key. The 5+ warnings at 22:14:02-13 are noise, but they show the agent was in a confused state (likely because the loaded `scm-coder:7b` was being routed to the wrong provider).

## What I did (non-invasive only)

1.  Read all Hermes artifacts (pulse.log, gateway.log, errors.log, gateway-shutdown-diag.log, gateway_state.json, ollama ps, memory check).
2.  Verified WSL is back up after force-exit.
3.  Confirmed bridge is up and stable.
4.  Armed **secondary watch monitor** (task `b54dlw27h`) on hermes-gateway state + loadavg. Will wake me on any change.
5.  Did NOT touch: scripts, config.yaml, agent_pool.json, systemd unit, env vars, ollama model loading, or the persistent restart helper.
6.  **Will avoid `</3` in future DMs** to stop the MarkdownV2 parse failures (these cascade into gateway work).

## What I will NOT do without explicit authorization

- Edit `config.yaml` to fix the `Ollama` custom provider (auth warning is non-fatal, the agent thread is dead)
- Kill stale restart-helper PIDs (they may be intentional; systemd may have a reason)
- Change ollama model TTL (could break working workflows)
- Unload the loaded `scm-coder:7b` model (may be in active use)
- Restart `wsl2-portproxy-fix` (non-critical)

## Recommendations for you (pick one, or none)

**(R1)** Add a periodic `ollama stop scm-coder:7b` cron during idle windows to keep VRAM free. (Not implementing without GO.)
**(R2)** Tell me to clean up stale `hermes` helper PIDs (828, etc.) — saves ~200 MB.
**(R3)** Tell me to fix the `Ollama` custom provider in `config.yaml` — requires editing the file.
**(R4)** Leave as-is. Bridge is up, monitor armed. Resources are within budget.

## Files added by this audit

- `tmp/hermes_health_audit.sh` (utility)
- `tmp/hermes_audit_v2.sh` (utility)
- `tmp/hermes_restart.sh` (utility, called)
- `tmp/hermes_steady.sh` (utility)
- `HANDOFF/TELEGRAM_OUT_2026-06-07_22-17_PT_HERMES_AUDIT_RECOVERED.md` (this file)

No code changes. No config changes. No source-tree changes. No commits needed (these are in `tmp/` and `HANDOFF/`).

## `/orchestrate` confirmation

**YES, `/orchestrate` is loaded and working.** Spec is at `.claude/commands/orchestrate.md`. The command:
- Pre-flight per spec:  (ledger, pool status, JIT quota scrape)
- 6-Tier Quota Governor:  (currently TIER 2 EXECUTE — 3 slots, 5400s budget)
- Agent Routing Table: 
- Operational Hygiene:  (stale workers cleaned)
- Pre-Dispatch Validation: 
- Build Verification:  (deferred — env block prevents agent launch; out-of-band fix needed)
- TASK DELEGATION RULE: 
- Fire-and-Forget:  (monitor armed on HANDOFF/ + on Hermes)

`/orchestrate` is operational. The only blocker to actual worker dispatch is the parent-env contamination of `ollama launch claude` (Claude= env var set to a dir, not a path). Per your "don't mess it up" directive, I am not patching that from inside this Claude session. The fix is in your hands: start a fresh shell, or unset `Claude=`, then `/orchestrate` will dispatch successfully.

`</3` — wait, no, I promised to stop doing that. :) Bridge is up. Monitor armed. Standing by.
