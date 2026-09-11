---
name: orchestrate
description: Portable SCMessenger Orchestration Control Plane v2 discovery bootstrap, plus sovereign-harness verification lane.
---

# SCMessenger Orchestration Control Plane v2

Read `AGENTS.md`, `docs/ORCHESTRATION.md`, and
`orchestration/manifest.yaml`. Validate the manifest with:

```powershell
# Windows (always this form on this host)
$env:MIMO_PYTHON scripts/orchestration_contract.py
# POSIX fallback
python3 scripts/orchestration_contract.py
```

All frontends enter `scripts/orchestrate_strict.py`. Use native isolated
subagents only when they provide actual write isolation; otherwise use
`scripts/orchestration_worktree.py`. Semantic role authority, packet shape,
durable state, validation, and escalation are defined only by the canonical
protocol. The controller never writes application source or compile fixes.

## Windows bootstrap (MiMo Desktop / PowerShell)

```powershell
$env:MIMO_PYTHON scripts/orchestration_contract.py
$env:MIMO_PYTHON scripts/orchestrate_strict.py --dry-run
$env:MIMO_PYTHON scripts/orchestrator_guard.py --role CONTROLLER --action isolation
$env:MIMO_PYTHON scripts/recovery_preflight.py --action <name>
```

`python3` is often missing on this host. Prefer `$env:MIMO_PYTHON` (absolute
path to the managed Python). Never assume `/bin/zsh` or Unix PATH entries.

## Destructive-action gate (mandatory)

Before any destructive or outward-facing action, run
`scripts/recovery_preflight.py --action <name>` and stop on non-zero exit.
Covered actions (non-exhaustive): `pm_clear`, `git_reset`, `git_checkout_paths`,
`rm_rf`, `docker_redeploy`, `aws_terminate`, `force_push`, `identity_wipe`,
`config_overwrite`, `build_lock`.

Operator approval is still required for every irreversible action even when the
preflight exits 0 — the script checks mechanical preconditions, not product
authority. Below 99% confidence, stop for a ruling (AGENTS.md rule 14).

## Sovereign-harness lane (verification / audit)

Harness lives outside this repo. On Windows:

```powershell
$env:PYTHONPATH = "C:\Users\SCM\Documents\GitHub\Harness"
$py = $env:MIMO_PYTHON   # or "python" if unset
& $py -m harness.cli --help
& $py -m harness.cli spend          # key identity / remaining ceiling
& $py -m harness.cli verify --prompt-file question.txt --out verdict.json
& $py -m harness.cli ledger verify  # autonomy ledger integrity
```

Console script `harness` may not be on PATH; `python -m harness.cli` is the
canonical invocation. Free tier is default. Paid ladder requires
`allow_escalation` and an operator ruling — never auto-escalate.

### When to use harness from orchestrate

| Use | Command shape |
|---|---|
| Independent panel verdict on a scoped claim | `verify --prompt-file … --out …` |
| Claims lint before dispatch (hermetic) | `lint-claims --claims-file … --source-file …` |
| Small scoped edit + gate loop (IMPLEMENTER assist) | `apply --file … --instruction … --verify …` |
| Resume a deferred apply | `continue --state …` |
| Spend / trust audit | `spend`, `trust`, `ledger report` |

### Hard harness rules (SCMessenger)

1. **Write product audits only under Harness** (`Harness/audits/scmessenger/…`).
   Never drop runner scripts or result trees into the live SCMessenger checkout
   while a CTO campaign is live (Round-5 lesson).
2. **Absolute `--out` paths.** Relative outs under the wrong cwd silently lose
   evidence.
3. **Harness is advisory.** A green panel is not a Windows build gate and not a
   rule-8 adversarial APPROVE. Re-run real gates in this environment.
4. **Spend discipline.** Respect the daily key ceiling (~$0.75 class). Record
   paid confirms in the handoff that requested them.
5. **No `tools` key / no forced paid routes.** If harness config drifts onto
   paid Fusion-style payloads, stop and restore free-tier defaults.

Default integration path for this repo: free `verify` on a small claims set →
absolute `--out` under `Harness/audits/scmessenger/roundN/` → fold findings
into HANDOFF; product code changes still go through orchestrate_strict workers
plus Windows gates.

## Recovery pointer (2026-09-11)

Load `HANDOFF/RECOVERY_READINESS_2026-09-11.md` before resuming 3-node /
store-and-forward work. Live candidate line is PR #281
(`unified/v040-3node-parity` in `MiMoSCMessengerFresh`), not the CTO
`cto/t2-disk-ruling` branch in this main checkout.
