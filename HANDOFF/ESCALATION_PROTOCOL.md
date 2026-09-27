# Escalation Protocol — Overseer Bridge (C)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

This is the contract between Claude Code (the orchestrator/worker) and the
Overseer Bridge — the launchd service that bridges
`HANDOFF/ESCALATIONS/` ↔ `HANDOFF/DIRECTIVES/` ↔ Lucas's Telegram DM.

If you are Claude Code and you need a human decision, follow this protocol.
If you are Lucas reading this from a Telegram DM, the protocol is the same:
reply with one of the directive tokens, and the bridge does the rest.

---

## When to escalate

Escalate (don't just keep going) when **any** of the following is true:

- You are about to make a change that **breaks a public API**, deletes a
  file, or force-pushes a branch.
- You found a **security issue** (vulnerability, secret leak, exposed
  credential) that you cannot fix autonomously per the project's threat
  model.
- You are **blocked on a decision** that only Lucas can make (e.g. "do we
  bump the major version, or do a backport?").
- You are about to **exceed your budget** (quota tier 5/6, or wall-time
  over your slot's limit).
- You need **clarification on the task spec** that would change the
  implementation if answered one way vs. the other.
- You **finished a P0/P1 task** and want eyes on it before merging.
  This is "review-required" — the most common escalation.

Do **not** escalate for routine work, build errors you can fix, or
stylistic questions. The bridge is for real decisions, not ping-pong.

---

## How to write an escalation

Drop a single markdown file into `HANDOFF/ESCALATIONS/`:

```
HANDOFF/ESCALATIONS/ESCALATION_<worker>_<ts>_<reason>.md
```

Example:
```
ESCALATION_claude-code_2026-06-10T18-50-23Z_review-required.md
```

### Recommended frontmatter (top of file)

```yaml
---
worker: claude-code          # which worker / sub-agent is escalating
severity: medium             # critical | high | medium | low | info
reason: review-required      # short slug, matches the filename
esc_id: ESC-A1B2C3D4         # OPTIONAL — generate once and reuse
---
```

The bridge will:
1. Pick up the file within ~3 seconds (SCAN_INTERVAL_S).
2. Forward the body as a Telegram DM to Lucas (chat 6014795323),
   prefixed with severity emoji, worker, reason, and a reply hint.
3. Write `HANDOFF/ESCALATIONS/ACK_<esc_id>.md` so you can poll for the
   answer.
4. Append a row to `HANDOFF/STATE/ESCALATION_LEDGER.md`.

### Body format (what to write in the file)

Plain markdown. Suggested structure:

```markdown
## Summary
One-line: what you did and why it needs a human.

## What changed
- core/src/foo.rs: added X (lines 42-80)
- core/src/bar.rs: modified Y (lines 12-18)

## Why I'm escalating
- [decision] this is review-required, eyes please
- [risk] this changes the public API, see #123

## Suggested reply
REPLY WITH: APPROVE | BLOCK | DEFER
- APPROVE — proceed to merge
- BLOCK  — hold, I'll send notes
- DEFER  — come back to it later
```

The `REPLY WITH:` line is auto-extracted by the bridge and shown at the
top of the Telegram DM. Make it terse and actionable.

---

## How to wait for the answer (don't block forever)

After you write the escalation, do this loop instead of idling:

1. Read `HANDOFF/ESCALATIONS/ACK_<esc_id>.md`. It exists as soon as the
   bridge forwards your escalation. Its `Status` field starts as `PENDING`.
2. Sleep + poll `HANDOFF/DIRECTIVES/DIRECTIVE_*.md` for one with
   `in_reply_to: <your-esc_id>` in the frontmatter.
3. When you see it, apply the directive, then update the ACK file's
   `Status:` line to `RESOLVED` and write a one-line note of what you
   did.
4. Move on to the next task.

The bridge updates the ACK status to `RESOLVED` when it writes the
directive. Your own update is the **second** stamp — a worker-attested
completion.

**Do not block on a single escalation for more than your slot's
runtime budget.** If you have a P0 task waiting on a non-critical
escalation, proceed with safe defaults and document them in the
escalation's ACK file.

---

## How to write a global directive (Lucas, optional)

If you want a directive that is **not** in reply to a specific
escalation — e.g. "KILL all P1 work and focus on the mDNS crash" — DM
the bot with a leading token:

- `KILL <task-id>` — archive the task, log why
- `PAUSE <task-id>` — set to IN_PROGRESS pause, do not dispatch
- `RESUME <task-id>` — flip back to ready
- `REPRIORITIZE <task-id> P0` — change the priority tag
- `FREEZE swarm` — stop dispatching new tasks until `RESUME swarm`

The bridge writes these to `HANDOFF/DIRECTIVES/` with
`in_reply_to: global` in the frontmatter, and the orchestrator's pool
patrol picks them up on its next pass.

---

## Severity guide

| Severity | When | Overseer SLA (informal) |
|----------|------|--------------------------|
| `critical` | Production down, security hole, data loss | ASAP — DM wakes Lucas |
| `high` | P0 broken, public API break | Same session |
| `medium` | Review-required on a meaningful change | Hours |
| `low` | Style / docs / nits | Whenever |
| `info` | FYI, no decision needed | No action |

---

## Files this protocol touches

| Path | Owner | Purpose |
|------|-------|---------|
| `HANDOFF/ESCALATIONS/ESCALATION_*.md` | Claude Code (write) | Outbound escalation |
| `HANDOFF/ESCALATIONS/ACK_<id>.md` | Bridge (write), Claude Code (update) | Status ledger |
| `HANDOFF/DIRECTIVES/DIRECTIVE_*.md` | Bridge (write) | Inbound directive |
| `HANDOFF/STATE/ESCALATION_LEDGER.md` | Bridge (write) | Durable audit trail |
| `~/.hermes/logs/overseer-bridge.log` | Bridge (write) | Service log |

---

## Bridge service management

```bash
# Status
launchctl list | grep overseer

# Tail the log
tail -f ~/.hermes/logs/overseer-bridge.log

# Restart after a config change
launchctl unload ~/Library/LaunchAgents/com.hermes.overseer-bridge.plist
launchctl load ~/Library/LaunchAgents/com.hermes.overseer-bridge.plist

# Stop the bridge
launchctl unload ~/Library/LaunchAgents/com.hermes.overseer-bridge.plist
```
