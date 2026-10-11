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

`vars.CI_RUNNER_BLACKSMITH_JOBS` is a comma list of `test,docs,ffi,forgery,pr`.
Default (unset) is `test`, push-to-main only; add `pr` to also use Blacksmith
on same-repo PRs. Jobs not listed fall back to hosted (`ubuntu-latest`).

Free tier (https://docs.blacksmith.sh/blacksmith-runners/overview): 3000 x64
2vCPU minutes per organization per month; larger runners consume them
proportionally (4vCPU = 2x, 8vCPU = 4x). Pricing: https://www.blacksmith.sh/pricing
(overage billing mechanics are not documented there; assume it may bill).
Observed 7-day volume (hosted job minutes): test 587, docs 2604, ffi 2710,
forgery 2143, so only a narrow scope fits the allowance.

## Rollback

Set `CI_RUNNER=hosted`, or delete the variable (falls back to legacy behaviour).

```bash
gh variable set CI_RUNNER --body hosted
gh variable delete CI_RUNNER
```
