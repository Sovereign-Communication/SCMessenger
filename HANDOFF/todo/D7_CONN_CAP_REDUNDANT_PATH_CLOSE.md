# D7 -- `[CONN-CAP] closing redundant per-peer path` emitted by the pre-rollout binary only

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- observation only, no confirmed defect
Priority: LOW -- logged as designed; filed so the evidence and the naming are not lost
Filed: 2026-09-26 (during the `bceacb94` rollout record, PR #389)
Node model note: every node runs the same transport, so this applies to any node;
"relay" below is the custody/forward behavior all nodes perform, not a node type
(see `docs/rules/NODE_MODEL.md`).

## What was observed

The Windows node logged `[CONN-CAP] closing redundant per-peer path` during the
`bceacb94` rollout window on 2026-09-26. The behavior is a node closing a second
transport path to the same peer once one path is already established. It was
recorded in `HANDOFF/V040_3NODE_RCA_2026-09-09.md` as "working as designed".

## The naming is misleading, and the evidence says so

The log line's own text in the binary is:

```
CONN-CAP] closing redundant per-peer path to hold the retained bound
```

Verified by direct string search of the two CLI binaries from this rollout:

| Binary | sha256 (short) | `grep -a -c CONN-CAP` |
|---|---|---|
| outgoing `1bc78c8` | `454cc346` | **1** |
| incoming `bceacb94` | `3a1ac11f` | **0** |

Control: `grep -a -c DIAL-BACKOFF` on the `bceacb94` binary returns 1, so the
incoming artifact is readable and the zero above is a real absence, not a failed
read. Reproduce with:

```
grep -a -c CONN-CAP <scmessenger-cli.exe>
```

So the emitting code exists in the **outgoing** binary and is **absent from the
incoming** one. Two consequences:

1. The line is **pre-existing, not introduced by `bceacb94`** -- confirmed at the
   binary level rather than inferred.
2. The line seen in the rollout window therefore came from a carried-over log
   entry in the node's log file, not from the new build. Anyone reading the
   rollout record as "the new binary emitted this" is reading it wrong.

## Evidence gap, stated plainly

The raw log line was **not** captured into the rollout evidence directory
(`tmp/rollout-20260926-bceacb94/` is gitignored and holds node stdout, not the
node's own `scm.log`). `grep -rn CONN-CAP` over that directory returns only this
ticket, the RCA, and the log analysis -- no captured log. The binary-level proof
above is therefore the strongest surviving evidence, and it is sufficient to
establish pre-existence but **not** the occurrence count or the exact peers
involved.

## Relation to the Wave-1 conn-limits note

`HANDOFF/V040_3NODE_LOG_ANALYSIS_2026-09-20.md` lines 61 and 72 record a Wave-1
residual for the stream/connection-cap family (`max sub-streams reached`).
That note predates this observation and never named the `[CONN-CAP]` line. The
two are plausibly the same subsystem but this ticket does not assert they are the
same defect.

## To confirm before acting

- Re-observe on a current build and capture the raw line this time.
- Establish whether the path is absent in `bceacb94` because the redundant-path
  close was removed, renamed, or is now silent. If it was removed, the RCA and
  log-analysis rows describing it as live behavior are stale.
- Decide whether `[CONN-CAP]` should keep a name that reads as a capability
  ceiling when it actually reports a redundant-path close.
