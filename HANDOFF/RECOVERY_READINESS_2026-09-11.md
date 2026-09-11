# Recovery Readiness — 2026-09-11 (second pass)

Status: Active
Owner seats: CONTROLLER / CTO / CEO (MiMo Desktop)
Scope: post-snag setup so `/orchestrate`, `/cto`, `/ceo` work here, with
destructive-action gates and the sovereign-harness lane wired.

## 1. What snarled (from session DB, not memory)

| Thread | Outcome |
|---|---|
| Settings mixup | Project `.mimocode/mimocode.json` disabled MiMo/Xiaomi and pinned OpenRouter free models without a key → requests died only inside `SCMessenger/`. **FIXED.** Backup: `.mimocode/mimocode.json.bak-openrouter-override-20260910`. Active config is a neutral shell. This session runs in that folder — fix verified. |
| 3-node deploy session (`GitHub\test`) | Landed PR **#281** `unified/v040-3node-parity` in `MiMoSCMessengerFresh`. Identity triad, CLI custody registration, CI fixes, AWS host-net restore, unwrap custody fix. Died mid-RCA on Pixel **Message Store Unavailable** after `pm clear`; hypothesis = dual Sled open on `filesDir` (managers + `MeshService.withStorageAndLogs`). **NOT FIXED.** |
| Windows BLE | Hardware MT7921 code 43 (predated ops). Not software-recoverable this seat. |
| Parallel harness audit | Round-5/6/7 under `Harness/audits/scmessenger/`. Debris in SCMessenger root (`run_audit.py`, `scmessenger_audit/`) must stay untracked. |

## 2. Live line of work

| Item | Path / ID |
|---|---|
| Deploy candidate | PR **#281** `unified/v040-3node-parity` |
| Worktree | `C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh` |
| This checkout | `cto/t2-disk-ruling-2026-08-31` (PR #279, CONFLICTING) + uncommitted NOTIF-GATE — **not** the ship line |
| Recovery isolation WT | `tmp/wt-recovery-setup-20260911` branch `recovery/setup-gates-20260911` |

Next product work (in order):

1. Finish Message Store start-path RCA/fix (single Sled owner or split paths).
2. Reinstall Pixel **without** another blind `pm clear` (identity export first).
3. Passive 3-node log proof; keep CI green on #281.

## 2b. Pixel verification protocol (operator 2026-09-11)

**PASSIVE ONLY for every agent seat.** The seat may install and collect logs.
The operator drives the phone.

| Allowed on Pixel | Forbidden on Pixel |
|---|---|
| `adb install -r` | UI taps / swipes / Dashboard navigation |
| `adb shell am start` / force-stop | sending messages from the phone |
| `adb logcat` / `run-as` file pull | mesh start/stop from Settings UI |
| wait + re-pull after operator action | `pm clear` without identity export + preflight + approval |

Evidence path: `files/logs/scmessenger-mesh.log` (UTF-16) +
`files/mesh_diagnostics.log`. FileLoggingTree captures Timber
(`GHOST-IDENTITY-001`, `UNIFICATION loadPeers`). Windows (`/api/diagnostics`)
and AWS (`ssh` / docker logs) remain actively drivable.

If Dashboard `loadPeers` evidence is required, **ask the operator to open the
peers list** — do not tap it yourself.
4. Rule-8 / operator gates still apply for merge to main.

## 3. Skills installed for this desktop

Copied into the MiMo write root so `/orchestrate`, `/cto`, `/ceo` load here:

| Skill | Path |
|---|---|
| orchestrate | `.mimocode/skills/orchestrate/SKILL.md` (harness lane added) |
| cto | `.mimocode/skills/cto/SKILL.md` (live-line + destructive table) |
| ceo | `.mimocode/skills/ceo/SKILL.md` (live-line + read-only audit gates) |
| onboard | `.mimocode/skills/onboard/` |
| finalize-checklist | `.mimocode/skills/finalize-checklist/` |
| build-verify | `.mimocode/skills/build-verify/` |

Source of truth for other frontends remains `.agents/skills/`. New
conversations pick up `.mimocode/skills` copies.

## 4. Destructive-action preflight

```powershell
$env:MIMO_PYTHON scripts/recovery_preflight.py --action audit_readiness
$env:MIMO_PYTHON scripts/recovery_preflight.py --action pm_clear
$env:MIMO_PYTHON scripts/recovery_preflight.py --action docker_redeploy
$env:MIMO_PYTHON scripts/recovery_preflight.py --action config_overwrite
```

Actions covered: `audit_readiness`, `pm_clear`, `git_reset`,
`git_checkout_paths`, `rm_rf`, `force_push`, `docker_redeploy`,
`aws_terminate`, `config_overwrite`, `build_lock`, `identity_wipe`.

**Exit 0 = mechanical clear, not authorization.** Exit 1 = BLOCK.
Operator approval is still mandatory for irreversible acts (AGENTS.md 12/14).

Lessons encoded from this snag:

- `pm clear` after identity wipe cost a full phone identity + proved nothing
  (store still failed) → preflight demands identity backup evidence.
- AWS redeploy without `--network host` dropped `:9001` → preflight inspects
  NetworkMode before replace.
- Editing `mimocode.json` without a dated `.bak` caused the request-outage
  class → preflight requires backup sibling and flags disabled xiaomi providers.
- `rm -rf` / mass `git checkout --` remain operator-only (AGENTS.md 12).

## 5. Sovereign-harness integration

| Item | Value |
|---|---|
| Checkout | `C:\Users\SCM\Documents\GitHub\Harness` |
| Seat wrapper | `$env:MIMO_PYTHON scripts/harness_gate.py …` |
| Direct invoke | `$env:PYTHONPATH = "C:\Users\SCM\Documents\GitHub\Harness"` then `$env:MIMO_PYTHON -m harness.cli …` |
| Subcommands | `verify`, `apply`, `continue`, `lint-claims`, `ledger`, `spend`, `trust`, `models` |
| Spend key | live; limit **$0.75/day**, remaining **$0.75** at integration time |
| **Required policy** | **All substantive changes** must be harness-verified |
| Default tier | **free** |
| Paid ceiling | **$0.10 per escalation/use** (`--allow-paid --max-cost 0.10`) |
| Write rule | Product audits → `Harness/audits/scmessenger/…` with **absolute** `--out` only |

Harness is **required additional evidence**, not a substitute for Windows
build gates or rule-8 APPROVE. Incomplete harness coverage = BLOCKED/UNVERIFIED.
Do not put harness runner debris in the SCMessenger worktree.

Readiness smoke (2026-09-11): free `verify` panel 3/3 + judge `real`,
cost **$0.00**, evidence
`Harness/audits/scmessenger/_runs/readiness-20260911/smoke_verify.json`.
`spend` + `ledger verify` + free `models` + `trust` all OK.
`harness_gate.py --max-cost 0.25` correctly **BLOCK** (exit 4).
`harness_gate.py --kind smoke` exit 0.

## 6. Onboarding commands (Windows)

```powershell
$env:MIMO_PYTHON scripts/orchestration_contract.py
$env:MIMO_PYTHON scripts/orchestrate_strict.py --dry-run
$env:MIMO_PYTHON scripts/recovery_preflight.py --action audit_readiness
$env:MIMO_PYTHON scripts/harness_gate.py --kind smoke
git config core.hooksPath   # must be .githooks
gh pr view 281 --repo Sovereign-Communication/SCMessenger
gh pr checks 281 --repo Sovereign-Communication/SCMessenger
```

## 7. Explicit non-claims

- No claim of 0.4.0 tag readiness.
- No claim of Pixel store fix.
- No claim of BLE recovery.
- No claim that PR #281 is merge-approved (CI green ≠ operator/rule-8 merge).
