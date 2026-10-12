# Runbook: CI Runner Selection (CI_RUNNER)

Status: Active (pilot)

Repo variable `vars.CI_RUNNER` selects the runner for the heavy Linux jobs.

| Value | Runner |
|---|---|
| `blacksmith` | `blacksmith-4vcpu-ubuntu-2404` (x64), scoped by `CI_RUNNER_BLACKSMITH_JOBS` |
| `oci` | `["self-hosted","oci"]` |
| `hosted` | `ubuntu-latest` |
| unset | legacy behaviour (`USE_OCI_RUNNER`; `Test (ubuntu-latest)` stays on `ubuntu-latest`) |

Jobs affected: ci.yml `test` (ubuntu leg only), `docs`, `ffi-surface`, and
security-regression-tests.yml `forgery-regression`.

Only trusted events (push, same-repo PR) get `blacksmith` or `oci`. Fork PRs
always run on `ubuntu-latest`. Required check names are unchanged (the matrix
value stays `ubuntu-latest`). Caching stays on actions/cache and Swatinem/rust-cache.

## Blacksmith scope and free tier

`vars.CI_RUNNER_BLACKSMITH_JOBS` is a comma list of `test,docs,ffi,forgery,light,pr`.
Default (unset) is `test`, push-to-main only; add `pr` to also use Blacksmith
on same-repo PRs. Jobs not listed fall back to hosted (`ubuntu-latest`).

Free tier (https://docs.blacksmith.sh/blacksmith-runners/overview): 3000 x64
2vCPU minutes per organization per month; larger runners consume them
proportionally (4vCPU = 2x, 8vCPU = 4x). Pricing: https://www.blacksmith.sh/pricing
(overage billing mechanics are not documented there; assume it may bill).
Observed 7-day volume (hosted job minutes): test 587, docs 2604, ffi 2710,
forgery 2143, so only a narrow scope fits the allowance.

## The `light` token (blacksmith-2vcpu-ubuntu-2404)

When hosted `ubuntu-latest` is saturated, small gate jobs queue behind heavy
ones and delay the Blacksmith-routed `Test` job (it `needs: changes`). With
`CI_RUNNER=blacksmith` and `light` listed in `CI_RUNNER_BLACKSMITH_JOBS`, these
jobs run on `blacksmith-2vcpu-ubuntu-2404` (the cheapest size, 1x free-tier burn):

- ci.yml: `changes` (Detect changed areas), `handoff-scope`, `naming-policy`,
  `lint` (Lint), `orchestration`
- lint.yml: `changes`, `rust` (Rust Linting)
- hygiene.yml: `hygiene-checks` (Repository Hygiene Checks)

Same rules as the heavy jobs: push is trusted; a same-repo PR also needs `pr`;
fork PRs and every other event stay on `ubuntu-latest`; anything else falls
back to `ubuntu-latest` (these jobs never used OCI). Check names, `needs`, `if`
and the fail-closed guard on the `changes` result are unchanged. Jobs needing
Linux build deps use `.github/actions/linux-build-deps`, which handles
Blacksmith. Example: `gh variable set CI_RUNNER_BLACKSMITH_JOBS --body test,light,pr`.

`scripts/test_ci_runner_selector.py` (run by Repository Hygiene) evaluates all
12 jobs over every runner/scope/event combination.

## Rollback

Set `CI_RUNNER=hosted`, or delete the variable (falls back to legacy behaviour).

```bash
gh variable set CI_RUNNER --body hosted
gh variable delete CI_RUNNER
```
