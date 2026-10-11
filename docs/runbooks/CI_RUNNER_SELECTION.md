# Runbook: CI Runner Selection (CI_RUNNER)

Status: Active (pilot)

Repo variable `vars.CI_RUNNER` selects the runner for the heavy Linux jobs.

| Value | Runner |
|---|---|
| `blacksmith` | `blacksmith-8vcpu-ubuntu-2404` (x64) |
| `oci` | `["self-hosted","oci"]` |
| `hosted` | `ubuntu-latest` |
| unset | legacy behaviour (`USE_OCI_RUNNER`; `Test (ubuntu-latest)` stays on `ubuntu-latest`) |

Jobs affected: ci.yml `test` (ubuntu leg only), `docs`, `ffi-surface`, and
security-regression-tests.yml `forgery-regression`.

Only trusted events (push, same-repo PR) get `blacksmith` or `oci`. Fork PRs
always run on `ubuntu-latest`. Required check names are unchanged (the matrix
value stays `ubuntu-latest`). Caching stays on actions/cache and Swatinem/rust-cache.

## Rollback

Set `CI_RUNNER=hosted`, or delete the variable (falls back to legacy behaviour).

```bash
gh variable set CI_RUNNER --body hosted
gh variable delete CI_RUNNER
```
